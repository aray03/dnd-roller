use rand::{Rng, RngExt};

/// Rolls one die with `sides` faces and returns a value in `1..=sides`.
///
/// Takes the random number generator as a parameter so tests can pass a
/// seeded one and get the same results every run.
pub fn roll_die(sides: u32, rng: &mut impl Rng) -> i64 {
    i64::from(rng.random_range(1..=sides))
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
}
