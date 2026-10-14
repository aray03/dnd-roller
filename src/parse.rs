use std::fmt;

use crate::dice::{DiceExpr, Modifier};

/// Why a dice expression couldn't be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The text isn't shaped like `NdM` or `dM` plus optional modifiers.
    /// Holds the whole expression.
    InvalidSyntax(String),
    /// A number is too large to fit. Holds the number's text.
    InvalidNumber(String),
    /// A modifier like `+5` appeared before any dice. Holds the modifier's text.
    ModifierWithoutDice(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidSyntax(expr) => write!(
                f,
                "invalid dice expression \"{expr}\" (expected e.g. 3d6, d20 or 2d6+3)"
            ),
            ParseError::InvalidNumber(number) => write!(f, "number \"{number}\" is too large"),
            ParseError::ModifierWithoutDice(modifier) => {
                write!(f, "modifier \"{modifier}\" has no dice before it")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Splits a whole command line into dice expressions and parses each one.
///
/// Spaces around modifiers are allowed, so `3d10 ++ 5 1d100` gives two
/// expressions: `3d10++5` and `1d100`. A piece continues the current
/// expression if it starts with `+`/`-`, or if the expression so far ends with
/// `+`/`-` (so the `5` in `++ 5` belongs to it). Anything else starts a new
/// expression.
pub fn parse_all(input: &str) -> Result<Vec<DiceExpr>, ParseError> {
    let mut expressions: Vec<String> = Vec::new();

    for piece in input.split_whitespace() {
        match expressions.last_mut() {
            Some(current) if piece.starts_with(['+', '-']) || current.ends_with(['+', '-']) => {
                current.push_str(piece);
            }
            None if piece.starts_with(['+', '-']) => {
                return Err(ParseError::ModifierWithoutDice(piece.to_string()));
            }
            _ => expressions.push(piece.to_string()),
        }
    }

    expressions.iter().map(|expr| parse_expr(expr)).collect()
}

/// Parses one dice expression such as `3d10`, `d20` (`dM` means `1dM`) or
/// `2d6+3--1`. The `d` can be upper or lower case.
pub fn parse_expr(input: &str) -> Result<DiceExpr, ParseError> {
    let Some((count_text, rest)) = input.split_once(['d', 'D']) else {
        return Err(ParseError::InvalidSyntax(input.to_string()));
    };

    // The sides run up to the first `+` or `-`; everything after is modifiers.
    let modifiers_start = rest.find(['+', '-']).unwrap_or(rest.len());
    let (sides_text, modifiers_text) = rest.split_at(modifiers_start);

    let count = if count_text.is_empty() {
        1
    } else {
        parse_number(count_text, input)?
    };
    let sides = parse_number(sides_text, input)?;
    let modifiers = parse_modifiers(modifiers_text, input)?;

    Ok(DiceExpr {
        count,
        sides,
        modifiers,
    })
}

/// Parses a chain of modifiers like `+3--1` into `[Flat(3), PerDie(-1)]`.
fn parse_modifiers(mut text: &str, whole_expr: &str) -> Result<Vec<Modifier>, ParseError> {
    let mut modifiers = Vec::new();

    while !text.is_empty() {
        // Check the two-character operators first, so `++` isn't read as `+` then `+`.
        let (per_die, sign, after_op) = if let Some(after) = text.strip_prefix("++") {
            (true, 1, after)
        } else if let Some(after) = text.strip_prefix("--") {
            (true, -1, after)
        } else if let Some(after) = text.strip_prefix('+') {
            (false, 1, after)
        } else if let Some(after) = text.strip_prefix('-') {
            (false, -1, after)
        } else {
            return Err(ParseError::InvalidSyntax(whole_expr.to_string()));
        };

        let digits_end = after_op
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after_op.len());
        let (number_text, remaining) = after_op.split_at(digits_end);
        let value = sign * i64::from(parse_number(number_text, whole_expr)?);

        modifiers.push(if per_die {
            Modifier::PerDie(value)
        } else {
            Modifier::Flat(value)
        });
        text = remaining;
    }

    Ok(modifiers)
}

/// Parses a run of ASCII digits. `whole_expr` is only used in the error message.
fn parse_number(text: &str, whole_expr: &str) -> Result<u32, ParseError> {
    // Checked by hand because `str::parse::<u32>` would also accept a leading `+`.
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ParseError::InvalidSyntax(whole_expr.to_string()));
    }
    text.parse()
        .map_err(|_| ParseError::InvalidNumber(text.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use Modifier::{Flat, PerDie};

    fn dice(count: u32, sides: u32) -> DiceExpr {
        DiceExpr {
            count,
            sides,
            modifiers: Vec::new(),
        }
    }

    fn dice_with(count: u32, sides: u32, modifiers: Vec<Modifier>) -> DiceExpr {
        DiceExpr {
            count,
            sides,
            modifiers,
        }
    }

    #[test]
    fn parses_count_and_sides() {
        assert_eq!(parse_expr("3d10"), Ok(dice(3, 10)));
    }

    #[test]
    fn missing_count_means_one() {
        assert_eq!(parse_expr("d20"), Ok(dice(1, 20)));
    }

    #[test]
    fn d_is_case_insensitive() {
        assert_eq!(parse_expr("1D100"), Ok(dice(1, 100)));
        assert_eq!(parse_expr("D20"), Ok(dice(1, 20)));
    }

    #[test]
    fn rejects_malformed_expressions() {
        for bad in [
            "3x10", "d", "3d", "abc", "", "+3d6", "3d+6", "3d10d6", "3 d6",
        ] {
            assert_eq!(
                parse_expr(bad),
                Err(ParseError::InvalidSyntax(bad.to_string())),
                "input: {bad:?}"
            );
        }
    }

    #[test]
    fn parses_single_modifiers() {
        assert_eq!(parse_expr("3d10++5"), Ok(dice_with(3, 10, vec![PerDie(5)])));
        assert_eq!(parse_expr("5d20+30"), Ok(dice_with(5, 20, vec![Flat(30)])));
        assert_eq!(parse_expr("1d20-2"), Ok(dice_with(1, 20, vec![Flat(-2)])));
        assert_eq!(parse_expr("2d6--1"), Ok(dice_with(2, 6, vec![PerDie(-1)])));
    }

    #[test]
    fn parses_chained_modifiers_in_order() {
        assert_eq!(
            parse_expr("2d6+3--1"),
            Ok(dice_with(2, 6, vec![Flat(3), PerDie(-1)]))
        );
        assert_eq!(
            parse_expr("d20++1+2-3--4"),
            Ok(dice_with(
                1,
                20,
                vec![PerDie(1), Flat(2), Flat(-3), PerDie(-4)]
            ))
        );
    }

    #[test]
    fn rejects_malformed_modifiers() {
        for bad in [
            "3d10+", "3d10++", "3d10+++5", "3d10+x", "3d10+5x", "3d10+-5", "3d10*2",
        ] {
            assert_eq!(
                parse_expr(bad),
                Err(ParseError::InvalidSyntax(bad.to_string())),
                "input: {bad:?}"
            );
        }
    }

    #[test]
    fn rejects_modifier_too_large() {
        assert_eq!(
            parse_expr("1d20+99999999999"),
            Err(ParseError::InvalidNumber("99999999999".to_string()))
        );
    }

    #[test]
    fn rejects_numbers_too_large() {
        assert_eq!(
            parse_expr("1d99999999999"),
            Err(ParseError::InvalidNumber("99999999999".to_string()))
        );
    }

    #[test]
    fn error_messages_are_readable() {
        assert_eq!(
            parse_expr("3x10").unwrap_err().to_string(),
            "invalid dice expression \"3x10\" (expected e.g. 3d6, d20 or 2d6+3)"
        );
        assert_eq!(
            parse_expr("1d99999999999").unwrap_err().to_string(),
            "number \"99999999999\" is too large"
        );
    }

    #[test]
    fn parse_all_splits_the_example_command() {
        assert_eq!(
            parse_all("3d10 ++ 5 5d20+30 8d20++90 1d100"),
            Ok(vec![
                dice_with(3, 10, vec![PerDie(5)]),
                dice_with(5, 20, vec![Flat(30)]),
                dice_with(8, 20, vec![PerDie(90)]),
                dice(1, 100),
            ])
        );
    }

    #[test]
    fn parse_all_allows_spaces_on_either_side_of_a_modifier() {
        let expected = Ok(vec![dice_with(3, 10, vec![PerDie(5)])]);
        assert_eq!(parse_all("3d10 ++5"), expected);
        assert_eq!(parse_all("3d10++ 5"), expected);
        assert_eq!(parse_all("3d10 ++ 5"), expected);
        assert_eq!(parse_all("3d10++5"), expected);
    }

    #[test]
    fn parse_all_handles_spaced_chains() {
        assert_eq!(
            parse_all("2d6 + 3 -- 1 d20"),
            Ok(vec![
                dice_with(2, 6, vec![Flat(3), PerDie(-1)]),
                dice(1, 20)
            ])
        );
    }

    #[test]
    fn parse_all_of_nothing_is_empty() {
        assert_eq!(parse_all(""), Ok(vec![]));
        assert_eq!(parse_all("   "), Ok(vec![]));
    }

    #[test]
    fn parse_all_rejects_modifier_without_dice() {
        assert_eq!(
            parse_all("+5 1d20"),
            Err(ParseError::ModifierWithoutDice("+5".to_string()))
        );
        assert_eq!(
            parse_all("+5 1d20").unwrap_err().to_string(),
            "modifier \"+5\" has no dice before it"
        );
    }

    #[test]
    fn parse_all_reports_the_bad_expression() {
        // A bare number after a finished expression starts a new (invalid) one.
        assert_eq!(
            parse_all("3d10 5"),
            Err(ParseError::InvalidSyntax("5".to_string()))
        );
        assert_eq!(
            parse_all("1d6 3x7 1d8"),
            Err(ParseError::InvalidSyntax("3x7".to_string()))
        );
        // Modifiers are joined first, so the error shows the whole expression.
        assert_eq!(
            parse_all("3d10 + x"),
            Err(ParseError::InvalidSyntax("3d10+x".to_string()))
        );
    }
}
