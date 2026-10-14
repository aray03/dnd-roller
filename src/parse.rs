use std::fmt;

use crate::dice::DiceExpr;

/// Why a dice expression couldn't be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The text isn't shaped like `NdM` or `dM`. Holds the whole expression.
    InvalidSyntax(String),
    /// A number is too large to fit. Holds the number's text.
    InvalidNumber(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidSyntax(expr) => write!(
                f,
                "invalid dice expression \"{expr}\" (expected NdM, e.g. 3d6 or d20)"
            ),
            ParseError::InvalidNumber(number) => write!(f, "number \"{number}\" is too large"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses one dice expression such as `3d10` or `d20` (`dM` means `1dM`).
/// The `d` can be upper or lower case.
pub fn parse_expr(input: &str) -> Result<DiceExpr, ParseError> {
    let Some((count_text, sides_text)) = input.split_once(['d', 'D']) else {
        return Err(ParseError::InvalidSyntax(input.to_string()));
    };

    let count = if count_text.is_empty() {
        1
    } else {
        parse_number(count_text, input)?
    };
    let sides = parse_number(sides_text, input)?;

    Ok(DiceExpr {
        count,
        sides,
        modifiers: Vec::new(),
    })
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

    fn dice(count: u32, sides: u32) -> DiceExpr {
        DiceExpr {
            count,
            sides,
            modifiers: Vec::new(),
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
            "invalid dice expression \"3x10\" (expected NdM, e.g. 3d6 or d20)"
        );
        assert_eq!(
            parse_expr("1d99999999999").unwrap_err().to_string(),
            "number \"99999999999\" is too large"
        );
    }
}
