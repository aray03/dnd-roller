use std::fmt;

/// One dice expression from the command line, e.g. `3d10++5-2`.
///
/// ```
/// use dice_roller::dice::{DiceExpr, Modifier};
///
/// // 3d10++5-2: roll three d10s, add 5 to each die, then subtract 2 once.
/// let expr = DiceExpr {
///     count: 3,
///     sides: 10,
///     modifiers: vec![Modifier::PerDie(5), Modifier::Flat(-2)],
/// };
/// assert_eq!(expr.modifiers.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceExpr {
    /// How many dice to roll (the `3` in `3d10`).
    pub count: u32,
    /// How many faces each die has (the `10` in `3d10`).
    pub sides: u32,
    /// Modifiers in the order they were written.
    pub modifiers: Vec<Modifier>,
}

/// A number added to (or subtracted from) a roll.
///
/// The value is signed: subtraction is stored as a negative number, so
/// `-2` is `Flat(-2)` and `--1` is `PerDie(-1)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    /// `+N` / `-N`: applied once to the roll's total. `5d20+30` adds 30.
    Flat(i64),
    /// `++N` / `--N`: applied to every die. `3d10++5` adds 5 × 3 = 15.
    PerDie(i64),
}

/// Writes the expression the way it would be typed, e.g. `3d10++5-2`.
/// `d20` is written as `1d20`.
impl fmt::Display for DiceExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}d{}", self.count, self.sides)?;
        for modifier in &self.modifiers {
            write!(f, "{modifier}")?;
        }
        Ok(())
    }
}

/// Writes the modifier as typed: `+3`, `-2`, `++5` or `--1`.
impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (operator, value) = match *self {
            Modifier::Flat(n) if n < 0 => ("-", n),
            Modifier::Flat(n) => ("+", n),
            Modifier::PerDie(n) if n < 0 => ("--", n),
            Modifier::PerDie(n) => ("++", n),
        };
        write!(f, "{operator}{}", value.unsigned_abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Modifier::{Flat, PerDie};

    #[test]
    fn modifiers_display_as_typed() {
        assert_eq!(Flat(3).to_string(), "+3");
        assert_eq!(Flat(-2).to_string(), "-2");
        assert_eq!(PerDie(5).to_string(), "++5");
        assert_eq!(PerDie(-1).to_string(), "--1");
    }

    #[test]
    fn expressions_display_as_typed() {
        let expr = DiceExpr {
            count: 2,
            sides: 6,
            modifiers: vec![Flat(3), PerDie(-1)],
        };
        assert_eq!(expr.to_string(), "2d6+3--1");

        let plain = DiceExpr {
            count: 1,
            sides: 20,
            modifiers: vec![],
        };
        assert_eq!(plain.to_string(), "1d20");
    }
}
