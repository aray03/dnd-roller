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
