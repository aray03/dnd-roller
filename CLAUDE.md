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
- Output: one line per expression (expression, individual dice, modifiers, subtotal; columns aligned), then a grand total:
  ```
  3d10++5    [4, 9, 2] ++5          → 30
  5d20+30    [3, 17, 8, 11, 20] +30 → 89
  Total: 119
  ```
- Errors: message on stderr naming the bad token, exit code 1, nothing rolled if any expression is invalid.
- Limits: dice count 1–1000, sides ≥ 2, `i64` with checked arithmetic.
- `--help` / `-h` print usage. Only those exact tokens trigger help, since `--` is also the per-die subtract modifier.

### Design choices

- **Args via `std::env::args`, not clap.** clap would treat `-2` and `--` as flags/separators, which clash with the modifier syntax.
- **`rand` 0.10 for randomness.** Roll functions take `&mut impl Rng` so tests can pass a seeded RNG (`StdRng::seed_from_u64`) and get deterministic results. Note the 0.10 API: `Rng` is the base trait, and helper methods like `random_range` live on the `RngExt` trait, which must be imported (`use rand::{Rng, RngExt};`). Older docs/tutorials that call `rng.gen_range` or use `thread_rng()` are out of date.
- **Library + binary split.** `src/lib.rs` declares the modules as `pub mod`; `src/main.rs` is a thin CLI that uses the library (`dice_roller::...`). This avoids dead-code warnings for functions that aren't wired into `main` yet, and lets integration tests in `tests/` call the library directly.
- **Module layout:** `src/lib.rs` (module list), `src/main.rs` (CLI glue), `src/roll.rs` (`roll_die`, later evaluate into `RollResult`), planned: `src/parse.rs` (tokenize + parse into `DiceExpr`), `src/output.rs` (pure formatting functions).

## Commands

```
cargo run -- <args>            # e.g. cargo run -- 3d10 ++ 5 1d20-2
cargo fmt
cargo clippy --all-targets -- -D warnings   # --all-targets also lints test code
cargo test
```

Run fmt, clippy and test before every commit.

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
- The owner reviews and merges the PR. Claude never merges. After merge: switch to `main`, pull, delete the local branch and the remote branch (`git push origin --delete <branch>`), mark the issue Done in Linear.
- Don't rename a branch that has an open PR: GitHub closes the PR (it only moves PRs over when the *target* branch is renamed). Pick the right name before pushing.
- The remote uses SSH (`git@github.com:aray03/dnd-roller.git`). HTTPS has no credentials on this machine.

## Working style

- Claude implements each issue; the owner reviews.
- In each PR and summary, explain the Rust concepts the change uses (ownership, `Result`, traits, etc.). The point of this project is learning Rust.
- Keep issues small and stick to the issue's scope; put follow-up ideas into new Linear issues instead.
