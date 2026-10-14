use std::process::ExitCode;

use dice_roller::parse::parse_all;
use dice_roller::roll::roll;

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

    let mut rng = rand::rng();
    for expr in &expressions {
        // One subtotal per line for now; the full breakdown comes in ARA-15.
        println!("{}", roll(expr, &mut rng).subtotal);
    }
    ExitCode::SUCCESS
}
