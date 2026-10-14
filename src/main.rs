use std::process::ExitCode;

use dice_roller::parse::parse_expr;
use dice_roller::roll::roll;

/// What gets rolled when no arguments are given.
const DEFAULT_EXPR: &str = "1d20";

fn main() -> ExitCode {
    // The first arg is the program's own path, so skip it.
    let args: Vec<String> = std::env::args().skip(1).collect();

    let input = match args.as_slice() {
        [] => DEFAULT_EXPR,
        [expr] => expr.as_str(),
        _ => {
            // Multiple expressions per command come in ARA-14.
            eprintln!("error: only one dice expression is supported for now");
            return ExitCode::FAILURE;
        }
    };

    let expr = match parse_expr(input) {
        Ok(expr) => expr,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let result = roll(&expr, &mut rand::rng());
    println!("{}", result.subtotal);
    ExitCode::SUCCESS
}
