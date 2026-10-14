use crate::dice::{DiceExpr, Modifier};
use crate::roll::RollResult;

/// Formats every roll as one aligned line, followed by a grand total:
///
/// ```text
/// 3d10++5  [9, 14, 7]              → 30
/// 5d20+30  [3, 17, 8, 11, 20] +30  → 89
/// Total: 119
/// ```
///
/// Each line shows the expression, the dice with any per-die modifiers
/// (`++N` / `--N`) already applied, the flat modifiers (`+N` / `-N`), and the
/// subtotal. `total` is passed in (rather than summed here) so the caller can
/// compute it with overflow checking. The returned string has no trailing
/// newline.
pub fn format_rolls(rolls: &[(DiceExpr, RollResult)], total: i64) -> String {
    let rows: Vec<Row> = rolls
        .iter()
        .map(|(expr, result)| Row::new(expr, result))
        .collect();

    let expr_width = rows.iter().map(|row| row.expr.len()).max().unwrap_or(0);
    let breakdown_width = rows
        .iter()
        .map(|row| row.breakdown.len())
        .max()
        .unwrap_or(0);
    let subtotal_width = rows
        .iter()
        .map(|row| row.subtotal.to_string().len())
        .max()
        .unwrap_or(0);

    let mut lines: Vec<String> = rows
        .iter()
        .map(|row| {
            format!(
                "{:<expr_width$}  {:<breakdown_width$}  → {:>subtotal_width$}",
                row.expr, row.breakdown, row.subtotal
            )
        })
        .collect();

    lines.push(format!("Total: {total}"));
    lines.join("\n")
}

/// The text pieces of one output line, before padding.
struct Row {
    /// e.g. `3d10++5`
    expr: String,
    /// e.g. `[9, 14, 7] +2`
    breakdown: String,
    subtotal: i64,
}

impl Row {
    fn new(expr: &DiceExpr, result: &RollResult) -> Row {
        let dice: Vec<String> = result.adjusted.iter().map(|die| die.to_string()).collect();
        let mut breakdown = format!("[{}]", dice.join(", "));
        // Per-die modifiers are already in the dice, so only list flat ones.
        for modifier in &expr.modifiers {
            if let Modifier::PerDie(_) = modifier {
                continue;
            }
            breakdown.push(' ');
            breakdown.push_str(&modifier.to_string());
        }

        Row {
            expr: expr.to_string(),
            breakdown,
            subtotal: result.subtotal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::Modifier::{Flat, PerDie};

    /// Builds a roll from the raw `dice`, applying the per-die modifiers to
    /// get the adjusted dice the way `roll` would.
    fn roll_of(
        count: u32,
        sides: u32,
        modifiers: Vec<Modifier>,
        dice: Vec<i64>,
        subtotal: i64,
    ) -> (DiceExpr, RollResult) {
        let per_die: i64 = modifiers
            .iter()
            .map(|modifier| match *modifier {
                PerDie(n) => n,
                Flat(_) => 0,
            })
            .sum();
        let adjusted = dice.iter().map(|die| die + per_die).collect();
        (
            DiceExpr {
                count,
                sides,
                modifiers,
            },
            RollResult {
                dice,
                adjusted,
                subtotal,
            },
        )
    }

    #[test]
    fn single_roll_without_modifiers() {
        let rolls = [roll_of(1, 20, vec![], vec![14], 14)];
        assert_eq!(format_rolls(&rolls, 14), "1d20  [14]  → 14\nTotal: 14");
    }

    #[test]
    fn columns_line_up_across_rolls() {
        let rolls = [
            roll_of(3, 10, vec![PerDie(5)], vec![4, 9, 2], 30),
            roll_of(5, 20, vec![Flat(30)], vec![3, 17, 8, 11, 20], 89),
            roll_of(1, 100, vec![], vec![7], 7),
        ];
        let expected = "\
3d10++5  [9, 14, 7]              → 30
5d20+30  [3, 17, 8, 11, 20] +30  → 89
1d100    [7]                     →  7
Total: 126";
        assert_eq!(format_rolls(&rolls, 126), expected);
    }

    #[test]
    fn shows_chained_and_negative_modifiers() {
        let rolls = [
            roll_of(2, 6, vec![Flat(3), PerDie(-1)], vec![5, 2], 8),
            roll_of(1, 4, vec![Flat(-10)], vec![3], -7),
        ];
        let expected = "\
2d6+3--1  [4, 1] +3  →  8
1d4-10    [3] -10    → -7
Total: 1";
        assert_eq!(format_rolls(&rolls, 1), expected);
    }

    #[test]
    fn per_die_modifiers_are_applied_to_the_dice() {
        // The example from ARA-25: 3d20++3 rolling 5, 6 and 12.
        let rolls = [roll_of(3, 20, vec![PerDie(3)], vec![5, 6, 12], 32)];
        assert_eq!(
            format_rolls(&rolls, 32),
            "3d20++3  [8, 9, 15]  → 32\nTotal: 32"
        );
    }

    #[test]
    fn prints_the_given_total() {
        let rolls = [
            roll_of(1, 6, vec![], vec![6], 6),
            roll_of(1, 6, vec![], vec![1], 1),
        ];
        assert!(format_rolls(&rolls, 7).ends_with("\nTotal: 7"));
    }
}
