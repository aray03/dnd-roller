use rand::{Rng, RngExt};

use crate::dice::{DiceExpr, Modifier};

/// The outcome of rolling one `DiceExpr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollResult {
    /// Each die's face value, before any modifiers, in the order rolled.
    pub dice: Vec<i64>,
    /// Sum of the dice with every modifier applied.
    pub subtotal: i64,
}

/// Rolls one die with `sides` faces and returns a value in `1..=sides`.
///
/// Takes the random number generator as a parameter so tests can pass a
/// seeded one and get the same results every run.
pub fn roll_die(sides: u32, rng: &mut impl Rng) -> i64 {
    i64::from(rng.random_range(1..=sides))
}

/// Rolls every die in `expr` and applies its modifiers.
///
/// `Flat(n)` adds `n` once; `PerDie(n)` adds `n` for each die rolled.
pub fn roll(expr: &DiceExpr, rng: &mut impl Rng) -> RollResult {
    let dice: Vec<i64> = (0..expr.count).map(|_| roll_die(expr.sides, rng)).collect();

    let modifier_total: i64 = expr
        .modifiers
        .iter()
        .map(|modifier| match *modifier {
            Modifier::Flat(n) => n,
            Modifier::PerDie(n) => n * i64::from(expr.count),
        })
        .sum();

    let subtotal = dice.iter().sum::<i64>() + modifier_total;
    RollResult { dice, subtotal }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::collections::HashSet;

    #[test]
    fn rolls_stay_within_range() {
        let mut rng = StdRng::seed_from_u64(42);
        for sides in [2, 4, 6, 8, 10, 12, 20, 100] {
            for _ in 0..1_000 {
                let value = roll_die(sides, &mut rng);
                assert!(
                    (1..=i64::from(sides)).contains(&value),
                    "d{sides} rolled {value}"
                );
            }
        }
    }

    #[test]
    fn every_face_shows_up() {
        let mut rng = StdRng::seed_from_u64(7);
        let seen: HashSet<i64> = (0..1_000).map(|_| roll_die(6, &mut rng)).collect();
        assert_eq!(seen, (1..=6).collect());
    }

    #[test]
    fn same_seed_gives_same_rolls() {
        let mut a = StdRng::seed_from_u64(1);
        let mut b = StdRng::seed_from_u64(1);
        for _ in 0..100 {
            assert_eq!(roll_die(20, &mut a), roll_die(20, &mut b));
        }
    }

    use Modifier::{Flat, PerDie};

    fn expr(count: u32, sides: u32, modifiers: Vec<Modifier>) -> DiceExpr {
        DiceExpr {
            count,
            sides,
            modifiers,
        }
    }

    /// Rolls `expr` with a fixed seed and checks the dice and subtotal against
    /// rolling the same dice one by one with an identically seeded RNG.
    fn check_roll(expr: &DiceExpr, expected_modifier_total: i64) {
        let result = roll(expr, &mut StdRng::seed_from_u64(99));

        let mut rng = StdRng::seed_from_u64(99);
        let expected_dice: Vec<i64> = (0..expr.count)
            .map(|_| roll_die(expr.sides, &mut rng))
            .collect();

        assert_eq!(result.dice, expected_dice);
        assert_eq!(
            result.subtotal,
            expected_dice.iter().sum::<i64>() + expected_modifier_total
        );
    }

    #[test]
    fn no_modifiers_is_just_the_dice_sum() {
        check_roll(&expr(4, 6, vec![]), 0);
    }

    #[test]
    fn per_die_modifier_applies_to_every_die() {
        // 3d10++5: +5 on each of 3 dice.
        check_roll(&expr(3, 10, vec![PerDie(5)]), 15);
    }

    #[test]
    fn flat_modifier_applies_once() {
        // 5d20+30
        check_roll(&expr(5, 20, vec![Flat(30)]), 30);
    }

    #[test]
    fn chained_modifiers_combine() {
        // 2d6+3--1: +3 once, -1 on each of 2 dice.
        check_roll(&expr(2, 6, vec![Flat(3), PerDie(-1)]), 3 - 2);
    }

    #[test]
    fn subtotal_can_go_negative() {
        // 1d4-10 is always between -9 and -6.
        let result = roll(&expr(1, 4, vec![Flat(-10)]), &mut StdRng::seed_from_u64(5));
        assert!((-9..=-6).contains(&result.subtotal));
    }
}
