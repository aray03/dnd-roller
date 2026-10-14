//! End-to-end tests: run the real `dice-roller` binary and check what it
//! prints and how it exits.

use std::process::Command;

/// What one run of the program produced.
struct Run {
    stdout: String,
    stderr: String,
    code: i32,
}

fn run(args: &[&str]) -> Run {
    // Cargo builds the binary before integration tests and puts its path here.
    let output = Command::new(env!("CARGO_BIN_EXE_dice-roller"))
        .args(args)
        .output()
        .expect("failed to run dice-roller");
    Run {
        stdout: String::from_utf8(output.stdout).expect("stdout is not UTF-8"),
        stderr: String::from_utf8(output.stderr).expect("stderr is not UTF-8"),
        code: output
            .status
            .code()
            .expect("dice-roller was killed by a signal"),
    }
}

/// One parsed line of roll output, e.g. `3d10++5  [4, 9, 2] ++5  → 30`.
struct RollLine {
    expr: String,
    dice: Vec<i64>,
    subtotal: i64,
}

fn parse_roll_line(line: &str) -> RollLine {
    let expr = line.split_whitespace().next().expect("empty line");
    let open = line.find('[').expect("no `[` in line");
    let close = line.find(']').expect("no `]` in line");
    let dice = line[open + 1..close]
        .split(", ")
        .map(|die| die.parse().expect("die is not a number"))
        .collect();
    let (_, subtotal) = line.split_once('→').expect("no `→` in line");
    RollLine {
        expr: expr.to_string(),
        dice,
        subtotal: subtotal.trim().parse().expect("subtotal is not a number"),
    }
}

/// Splits successful output into the roll lines and the grand total.
fn parse_output(stdout: &str) -> (Vec<RollLine>, i64) {
    let lines: Vec<&str> = stdout.lines().collect();
    let (total_line, roll_lines) = lines.split_last().expect("no output");
    let total = total_line
        .strip_prefix("Total: ")
        .expect("last line isn't `Total: N`")
        .parse()
        .expect("total is not a number");
    (
        roll_lines
            .iter()
            .map(|line| parse_roll_line(line))
            .collect(),
        total,
    )
}

#[test]
fn no_args_rolls_one_d20() {
    // Run several times since the result is random.
    for _ in 0..20 {
        let run = run(&[]);
        assert_eq!(run.code, 0);
        assert_eq!(run.stderr, "");

        let (rolls, total) = parse_output(&run.stdout);
        assert_eq!(rolls.len(), 1);
        assert_eq!(rolls[0].expr, "1d20");
        assert_eq!(rolls[0].dice.len(), 1);
        assert!((1..=20).contains(&rolls[0].subtotal));
        assert_eq!(total, rolls[0].subtotal);
    }
}

#[test]
fn example_command_rolls_every_expression() {
    let run = run(&["3d10", "++", "5", "5d20+30", "8d20++90", "1d100"]);
    assert_eq!(run.code, 0);
    assert_eq!(run.stderr, "");

    let (rolls, total) = parse_output(&run.stdout);
    let expected = [
        ("3d10++5", 3, 1..=10, 15, 18..=45),
        ("5d20+30", 5, 1..=20, 30, 35..=130),
        ("8d20++90", 8, 1..=20, 720, 728..=880),
        ("1d100", 1, 1..=100, 0, 1..=100),
    ];
    assert_eq!(rolls.len(), expected.len());

    for (roll, (expr, count, faces, modifier_total, subtotal_range)) in rolls.iter().zip(expected) {
        assert_eq!(roll.expr, expr);
        assert_eq!(roll.dice.len(), count, "{expr}: wrong number of dice");
        assert!(
            roll.dice.iter().all(|die| faces.contains(die)),
            "{expr}: die out of range"
        );
        assert_eq!(
            roll.subtotal,
            roll.dice.iter().sum::<i64>() + modifier_total,
            "{expr}: subtotal doesn't match dice + modifiers"
        );
        assert!(
            subtotal_range.contains(&roll.subtotal),
            "{expr}: subtotal out of range"
        );
    }

    let sum: i64 = rolls.iter().map(|roll| roll.subtotal).sum();
    assert_eq!(total, sum);
}

#[test]
fn spaced_per_die_subtract_still_rolls() {
    // `--` on its own is a modifier, not an option separator.
    let run = run(&["2d6", "--", "1"]);
    assert_eq!(run.code, 0);

    let (rolls, _) = parse_output(&run.stdout);
    assert_eq!(rolls.len(), 1);
    assert_eq!(rolls[0].expr, "2d6--1");
    assert_eq!(rolls[0].subtotal, rolls[0].dice.iter().sum::<i64>() - 2);
}

#[test]
fn invalid_input_fails_with_a_message() {
    let cases: [&[&str]; 8] = [
        &["0d6"],
        &["1d1"],
        &["5000d6"],
        &["3x7"],
        &["1d20+99999999999999999999"],
        &["+5", "1d20"],
        &["3d10+"],
        &["1d20", "3x7"],
    ];
    for args in cases {
        let run = run(args);
        assert_eq!(run.code, 1, "{args:?}: wrong exit code");
        assert_eq!(run.stdout, "", "{args:?}: nothing should be rolled");
        assert!(
            run.stderr.starts_with("error: "),
            "{args:?}: stderr was {:?}",
            run.stderr
        );
    }
}

#[test]
fn error_message_names_the_bad_expression() {
    let run = run(&["1d20", "3x7"]);
    assert_eq!(
        run.stderr,
        "error: invalid dice expression \"3x7\" (expected e.g. 3d6, d20 or 2d6+3)\n"
    );
}

#[test]
fn help_prints_usage_and_succeeds() {
    for flag in ["--help", "-h"] {
        let run = run(&[flag]);
        assert_eq!(run.code, 0, "{flag}");
        assert_eq!(run.stderr, "", "{flag}");
        assert!(run.stdout.contains("Usage: dice-roller"), "{flag}");
    }
}
