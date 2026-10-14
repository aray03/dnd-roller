# CLAUDE.md

Guidance for Claude when working in this repo. Keep it current: whenever something non-obvious is learned or decided during an issue, update this file in that issue's PR.

## Project

`dice-roller` is a Rust CLI for rolling D&D dice. The owner (GitHub `aray03`) is learning Rust.

### Spec

```
dice-roller 3d10 ++ 5 5d20+30 8d20++90 1d100
```

- An expression is `NdM` (or `dM`, meaning `1dM`; `d` is case-insensitive) followed by zero or more modifiers:
  - `+N` / `-N`: add/subtract N **once** to that roll's total. `5d20+30` → dice + 30.
  - `++N` / `--N`: add/subtract N to **each die**. `3d10++5` → dice + 15.
  - Modifiers chain: `2d6+3--1` → dice + 3 − 2.
- Spaces around modifiers are allowed (`3d10 ++ 5`). All args are joined, then tokenized; a new expression starts at each `NdM`/`dM`. A modifier with no dice before it is an error.
- No arguments → roll `1d20`.
- Output: one line per expression (expression written canonically, e.g. `d20` → `1d20`; raw dice; modifiers; subtotal right-aligned after `→`; columns separated by two spaces and aligned), then a grand total:
  ```
  3d10++5  [4, 9, 2] ++5           → 30
  5d20+30  [3, 17, 8, 11, 20] +30  → 89
  Total: 119
  ```
- Errors: message on stderr naming the bad token, exit code 1, nothing rolled if any expression is invalid.
- Limits: dice count 1–1000 (`parse::MAX_DICE`), sides ≥ 2 (`parse::MIN_SIDES`), checked in `parse_expr` → `ParseError::InvalidCount` / `InvalidSides`. Numbers are `u32` when parsed; all roll math is `i64` with checked arithmetic (`checked_add`/`checked_mul`), so overflow is an `OverflowError`, never a panic or a wrapped value.
- `--help` / `-h` print usage. Only those exact tokens trigger help, since `--` is also the per-die subtract modifier.

### Design choices

- **Args via `std::env::args`, not clap.** clap would treat `-2` and `--` as flags/separators, which clash with the modifier syntax.
- **`rand` 0.10 for randomness.** Roll functions take `&mut impl Rng` so tests can pass a seeded RNG (`StdRng::seed_from_u64`) and get deterministic results. Note the 0.10 API: `Rng` is the base trait, and helper methods like `random_range` live on the `RngExt` trait, which must be imported (`use rand::{Rng, RngExt};`). Older docs/tutorials that call `rng.gen_range` or use `thread_rng()` are out of date.
- **Library + binary split.** `src/lib.rs` declares the modules as `pub mod`; `src/main.rs` is a thin CLI that uses the library (`dice_roller::...`). This avoids dead-code warnings for functions that aren't wired into `main` yet, and lets integration tests in `tests/` call the library directly.
- **Module layout:** `src/lib.rs` (module list), `src/main.rs` (CLI glue: `fn main() -> ExitCode`, errors via `eprintln!` + `ExitCode::FAILURE`, real rolls use `rand::rng()`; `wants_help` checks for the exact args `-h`/`--help` and `usage()` builds the help text from the real constants, kept within 80 columns; has its own unit tests, which run as the binary's test target), `src/dice.rs` (data types: `DiceExpr`, `Modifier` enum with `Flat`/`PerDie`, values signed so subtraction is negative), `src/parse.rs` (`parse_all(&str)` splits the joined command line into expressions, then `parse_expr` parses each one → `Result<DiceExpr, ParseError>`, including modifier chains; when matching operators, try `++`/`--` before `+`/`-`. Splitting rule: a whitespace piece continues the current expression if it starts with `+`/`-` or the expression so far ends with `+`/`-`; otherwise it starts a new one. A modifier before any dice is `ParseError::ModifierWithoutDice`), `src/roll.rs` (`roll_die`; `roll(&DiceExpr, rng) -> Result<RollResult, OverflowError>` where `RollResult { dice, subtotal }` has the raw face values and the subtotal with all modifiers; `checked_total` sums with overflow checking), `src/output.rs` (`format_rolls(&[(DiceExpr, RollResult)], total) -> String`, a pure function with no trailing newline; the total is passed in so `main` can compute it with `checked_total`; `DiceExpr`/`Modifier` implement `Display` in `dice.rs` to print themselves as typed).
- **Integration tests:** `tests/cli.rs` runs the real binary via `std::process::Command::new(env!("CARGO_BIN_EXE_dice-roller"))` (no extra crates) and parses its output. Because rolls are random, they check structure and consistency (dice count, each die in range, subtotal = dice + modifiers, total = sum of subtotals, exit codes, stderr) rather than exact numbers. If the output format changes, update `parse_roll_line` there.
- **Parsing numbers:** check that text is all ASCII digits before calling `str::parse::<u32>()`, because `parse` also accepts a leading `+` (`"+3"` → 3), which would let `+3d6` or `3d+6` slip through. Non-digit text → `ParseError::InvalidSyntax`; digits that overflow `u32` → `ParseError::InvalidNumber`.

## Commands

```
cargo run -- <args>            # e.g. cargo run -- 3d10 ++ 5 1d20-2
cargo fmt
cargo clippy --all-targets -- -D warnings   # --all-targets also lints test code
cargo test
```

Run fmt, clippy and test before every commit, **in the same shell command as `git add`/`git commit`** (chained with `&&`), so the files can't change between being checked and being committed. ARA-23: running them as separate commands let an untested, non-compiling `parse.rs` get committed and merged.

## CI

`.github/workflows/ci.yml` runs on every PR into `main` and on pushes to `main`: one job named **`Check`** (ubuntu-latest) that runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`, the same commands as above. Actions used: `actions/checkout@v7`, `dtolnay/rust-toolchain@stable` (with rustfmt + clippy), `Swatinem/rust-cache@v2`. If CI fails on a PR, read the failing step's log (`gh pr checks`, `gh run view --log-failed`) and fix it with a new commit on the same branch.

**`Check` is a required status check on `main`** (branch protection, pinned to the GitHub Actions app, "require branches to be up to date" on): a PR can't be merged until `Check` passes on the PR combined with the latest `main`. If `main` moved after CI ran, the PR needs "Update branch" and a fresh green run. If the job is ever renamed, update the required check in branch protection too, or every PR will be stuck waiting for a check that never reports.

## Linear

- Use the **`linear-dnd`** MCP server: workspace `aray03-personal`, team `ARA`, project "D&D Dice Roller". Do **not** use the user-level `linear` server, which points to a different workspace (TheMaiar).
- Statuses: Backlog → Todo → In Progress → Done. Labels: Feature, Improvement, Bug.
- Milestones: M1 Single roll, M2 Multiple rolls & output, M3 Polish & distribution. Issues are chained with "blocked by" relations in the order they should be done.

## Git workflow

- **Never commit, push, or open/edit a PR unless the owner explicitly says to.** Making file changes is fine; stop before `git commit`/`git push`/`gh pr` and ask.
- **Never commit or push directly to `main`.** `main` is protected on GitHub (PR required, applies to admins too), so direct pushes are rejected anyway.
- One branch per issue, created from an up-to-date `origin/main`.
- **Branch names are short:** `<number>-<1–3 word name>`, lowercase, hyphenated, no `ara-` or `ray/` prefix. E.g. ARA-22 "Simplify Branch Names" → `22-branch-names`, ARA-8 "Add rand and a roll_die function" → `8-roll-die`. Do **not** use Linear's `gitBranchName` (too long).
- **PR titles are `ARA-<number>: <full Linear issue title>`**, copied exactly from Linear. E.g. `ARA-22: Simplify Branch Names`. The `ARA-<number>` in the title and `Fixes ARA-<number>` in the body are what link the PR to the Linear issue, since branch names no longer contain it.
- Commit messages are prefixed with the issue ID: `ARA-6: Add CLAUDE.md`.
- Push with `git push -u origin HEAD`, then `gh pr create --base main` with `Fixes ARA-N` in the body.
- **Stacked PRs:** if the previous issue's PR isn't merged yet and the new issue builds on it, branch from the previous issue's branch and open the PR with `--base <previous-branch>` so the diff only shows the new issue. Note it at the top of the PR body and tell the owner the merge order. When the base PR is merged, GitHub deletes its branch and retargets the stacked PR to `main` automatically.
- The owner reviews and merges the PR. Claude never merges. After merge: switch to `main`, pull, delete the local branch and the remote branch (`git push origin --delete <branch>`), mark the issue Done in Linear.
- Don't rename a branch that has an open PR: GitHub closes the PR (it only moves PRs over when the *target* branch is renamed). Pick the right name before pushing.
- The remote uses SSH (`git@github.com:aray03/dnd-roller.git`). HTTPS has no credentials on this machine.

## Working style

- Claude implements each issue; the owner reviews.
- In each PR and summary, explain the Rust concepts the change uses (ownership, `Result`, traits, etc.). The point of this project is learning Rust.
- Keep issues small and stick to the issue's scope; put follow-up ideas into new Linear issues instead.
