use std::fmt;

use rand::{Rng, RngExt};

use crate::dice::{DiceExpr, Modifier};

/// A result was too large (or too negative) to fit in an `i64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverflowError;

impl fmt::Display for OverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "result is too large to calculate")
    }
}

impl std::error::Error for OverflowError {}

/// The outcome of rolling one `DiceExpr`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollResult {
    /// Each die's face value, before any modifiers, in the order rolled.
    pub dice: Vec<i64>,
    /// Each die with the per-die modifiers (`++N` / `--N`) applied, in the
    /// same order as `dice`. Equal to `dice` if there are none.
    pub adjusted: Vec<i64>,
    /// Sum of `adjusted` with the flat modifiers (`+N` / `-N`) applied.
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
/// `PerDie(n)` adds `n` to each die (see `RollResult::adjusted`); `Flat(n)`
/// adds `n` once to the subtotal.
/// Returns `OverflowError` instead of panicking if the math doesn't fit in an
/// `i64`. Panics if `expr.sides` is 0 (the parser never produces that).
pub fn roll(expr: &DiceExpr, rng: &mut impl Rng) -> Result<RollResult, OverflowError> {
    let dice: Vec<i64> = (0..expr.count).map(|_| roll_die(expr.sides, rng)).collect();

    let per_die = checked_total(
        expr.modifiers
            .iter()
            .filter_map(|modifier| match *modifier {
                Modifier::PerDie(n) => Some(n),
                Modifier::Flat(_) => None,
            }),
    )?;
    let flat = checked_total(
        expr.modifiers
            .iter()
            .filter_map(|modifier| match *modifier {
                Modifier::Flat(n) => Some(n),
                Modifier::PerDie(_) => None,
            }),
    )?;

    let adjusted: Vec<i64> = dice
        .iter()
        .map(|die| die.checked_add(per_die).ok_or(OverflowError))
        .collect::<Result<_, _>>()?;
    let subtotal = checked_total(adjusted.iter().copied())?
        .checked_add(flat)
        .ok_or(OverflowError)?;

    Ok(RollResult {
        dice,
        adjusted,
        subtotal,
    })
}

/// Adds up numbers, returning `OverflowError` instead of panicking or
/// wrapping around if the sum doesn't fit in an `i64`.
pub fn checked_total(values: impl IntoIterator<Item = i64>) -> Result<i64, OverflowError> {
    values.into_iter().try_fold(0_i64, |total, value| {
        total.checked_add(value).ok_or(OverflowError)
    })
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

    /// Rolls `expr` with a fixed seed and checks the dice, adjusted dice and
    /// subtotal against rolling the same dice one by one with an identically
    /// seeded RNG.
    fn check_roll(expr: &DiceExpr, expected_per_die: i64, expected_modifier_total: i64) {
        let result = roll(expr, &mut StdRng::seed_from_u64(99)).unwrap();

        let mut rng = StdRng::seed_from_u64(99);
        let expected_dice: Vec<i64> = (0..expr.count)
            .map(|_| roll_die(expr.sides, &mut rng))
            .collect();

        assert_eq!(result.dice, expected_dice);
        let expected_adjusted: Vec<i64> = expected_dice
            .iter()
            .map(|die| die + expected_per_die)
            .collect();
        assert_eq!(result.adjusted, expected_adjusted);
        assert_eq!(
            result.subtotal,
            expected_dice.iter().sum::<i64>() + expected_modifier_total
        );
    }

    #[test]
    fn no_modifiers_is_just_the_dice_sum() {
        check_roll(&expr(4, 6, vec![]), 0, 0);
    }

    #[test]
    fn per_die_modifier_applies_to_every_die() {
        // 3d10++5: +5 on each of 3 dice.
        check_roll(&expr(3, 10, vec![PerDie(5)]), 5, 15);
    }

    #[test]
    fn flat_modifier_applies_once() {
        // 5d20+30
        check_roll(&expr(5, 20, vec![Flat(30)]), 0, 30);
    }

    #[test]
    fn chained_modifiers_combine() {
        // 2d6+3--1: +3 once, -1 on each of 2 dice.
        check_roll(&expr(2, 6, vec![Flat(3), PerDie(-1)]), -1, 3 - 2);
    }

    #[test]
    fn several_per_die_modifiers_add_up() {
        // 2d6++3--1: each die gets +3 - 1 = +2.
        check_roll(&expr(2, 6, vec![PerDie(3), PerDie(-1)]), 2, 4);
    }

    #[test]
    fn subtotal_can_go_negative() {
        // 1d4-10 is always between -9 and -6.
        let result = roll(&expr(1, 4, vec![Flat(-10)]), &mut StdRng::seed_from_u64(5)).unwrap();
        assert!((-9..=-6).contains(&result.subtotal));
    }

    #[test]
    fn largest_parseable_roll_does_not_overflow() {
        // The parser allows up to 1000 dice and numbers up to u32::MAX.
        let big = i64::from(u32::MAX);
        let result = roll(
            &expr(1000, u32::MAX, vec![PerDie(big), Flat(big)]),
            &mut StdRng::seed_from_u64(3),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn overflowing_modifiers_are_an_error_not_a_panic() {
        let mut rng = StdRng::seed_from_u64(4);
        // Adding the per-die modifier to a die overflows.
        assert_eq!(
            roll(&expr(2, 6, vec![PerDie(i64::MAX)]), &mut rng),
            Err(OverflowError)
        );
        // Adding to the subtotal overflows.
        assert_eq!(
            roll(&expr(1, 6, vec![Flat(i64::MAX), Flat(i64::MAX)]), &mut rng),
            Err(OverflowError)
        );
        // Going too far negative overflows too. (`i64::MIN` alone still fits
        // once the die is added, so subtract a bit more.)
        assert_eq!(
            roll(&expr(1, 6, vec![Flat(i64::MIN), Flat(-10)]), &mut rng),
            Err(OverflowError)
        );
    }

    #[test]
    fn checked_total_adds_or_reports_overflow() {
        assert_eq!(checked_total([]), Ok(0));
        assert_eq!(checked_total([1, 2, -4]), Ok(-1));
        assert_eq!(checked_total([i64::MAX, 1]), Err(OverflowError));
        assert_eq!(
            OverflowError.to_string(),
            "result is too large to calculate"
        );
    }
}
