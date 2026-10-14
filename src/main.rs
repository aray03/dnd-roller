use std::process::ExitCode;

use dice_roller::dice::DiceExpr;
use dice_roller::output::format_rolls;
use dice_roller::parse::parse_all;
use dice_roller::roll::{OverflowError, RollResult, checked_total, roll};

/// What gets rolled when no arguments are given.
const DEFAULT_EXPR: &str = "1d20";

fn main() -> ExitCode {
    // The first arg is the program's own path, so skip it.
    let args: Vec<String> = std::env::args().skip(1).collect();

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
