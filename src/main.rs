use std::process::ExitCode;

use dice_roller::dice::DiceExpr;
use dice_roller::output::format_rolls;
use dice_roller::parse::{DEFAULT_SIDES, MAX_DICE, MIN_SIDES, parse_all};
use dice_roller::roll::{OverflowError, RollResult, checked_total, roll};

/// What gets rolled when no arguments are given.
const DEFAULT_EXPR: &str = "1d20";

fn main() -> ExitCode {
    // The first arg is the program's own path, so skip it.
    let args: Vec<String> = std::env::args().skip(1).collect();

    if wants_help(&args) {
        println!("{}", usage());
        return ExitCode::SUCCESS;
    }

    let input = if args.is_empty() {
        DEFAULT_EXPR.to_string()
    } else {
        args.join(" ")
    };

    // Parse everything before rolling anything, so one bad expression
    // means nothing is rolled.
    let expressions = match parse_all(&input) {
        Ok(expressions) => expressions,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    match roll_all(expressions) {
        Ok((rolls, total)) => {
            println!("{}", format_rolls(&rolls, total));
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Rolls every expression and adds up the grand total.
fn roll_all(
    expressions: Vec<DiceExpr>,
) -> Result<(Vec<(DiceExpr, RollResult)>, i64), OverflowError> {
    let mut rng = rand::rng();
    let rolls = expressions
        .into_iter()
        .map(|expr| roll(&expr, &mut rng).map(|result| (expr, result)))
        .collect::<Result<Vec<_>, _>>()?;

    let total = checked_total(rolls.iter().map(|(_, result)| result.subtotal))?;
    Ok((rolls, total))
}

/// True if any argument is exactly `-h` or `--help`.
///
/// Only those exact tokens count, because `--` on its own (or `--1`) is the
/// per-die subtract modifier, e.g. `2d6 -- 1`.
fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-h" || arg == "--help")
}

fn usage() -> String {
    format!(
        "\
Roll D&D dice from the command line.

Usage: dice-roller [EXPRESSION]...

Each expression is NdM (N dice with M sides; dM means 1dM, and a plain number
N means Nd{DEFAULT_SIDES}), followed by any number of modifiers:
  +X    add X once to the roll's total      5d20+30  → dice + 30
  -X    subtract X once                     1d20-2   → dice - 2
  ++X   add X to every die                  3d10++5  → dice + 15
  --X   subtract X from every die           2d6--1   → dice - 2

Spaces around modifiers are fine (3d10 ++ 5). Several expressions can be rolled
at once. With no expressions, rolls {DEFAULT_EXPR}. Up to {MAX_DICE} dice per
expression; dice need at least {MIN_SIDES} sides.

Examples:
  dice-roller                    roll a d20
  dice-roller 2d6+3              two d6s plus 3
  dice-roller 3++5               three d{DEFAULT_SIDES}s, plus 5 on each
  dice-roller 3d10 ++ 5 1d100    several rolls with a combined total

Options:
  -h, --help    show this help"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn help_flags_trigger_help() {
        assert!(wants_help(&args(&["--help"])));
        assert!(wants_help(&args(&["-h"])));
        assert!(wants_help(&args(&["1d20", "--help"])));
    }

    #[test]
    fn per_die_modifiers_do_not_trigger_help() {
        assert!(!wants_help(&args(&[])));
        assert!(!wants_help(&args(&["2d6", "--", "1"])));
        assert!(!wants_help(&args(&["2d6", "--1"])));
        assert!(!wants_help(&args(&["2d6--1", "-2"])));
        assert!(!wants_help(&args(&["--helpful"])));
    }

    #[test]
    fn usage_mentions_the_real_limits() {
        let text = usage();
        assert!(text.contains("rolls 1d20"));
        assert!(text.contains(&format!("N means Nd{DEFAULT_SIDES}")));
        assert!(text.contains(&format!("Up to {MAX_DICE} dice")));
        assert!(text.contains(&format!("at least {MIN_SIDES} sides")));
    }

    #[test]
    fn usage_fits_in_80_columns() {
        for line in usage().lines() {
            assert!(line.chars().count() <= 80, "too long: {line:?}");
        }
    }
}
