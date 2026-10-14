# dice-roller

A command-line dice roller for D&D, written in Rust. Roll any number of dice
expressions at once and see a bunch of dice data.

```
$ dice-roller 3d10 ++ 5 5d20+30 8d20++90 1d100
3d10++5   [9, 14, 11]                             →  34
5d20+30   [1, 2, 15, 20, 9] +30                   →  77
8d20++90  [102, 95, 105, 94, 105, 103, 100, 110]  → 814
1d100     [66]                                    →  66
Total: 991
```

## Installation

You need [Rust](https://rustup.rs) 1.85 or newer.

```
git clone https://github.com/aray03/dnd-roller.git
cd dnd-roller
cargo install --path .
```

This builds an optimized binary and copies it to `~/.cargo/bin/dice-roller`.
Make sure `~/.cargo/bin` is on your `PATH`.

To try it without installing, use `cargo run -- <expressions>` from the repo.

## Usage

```
dice-roller [EXPRESSION]...
```

Each expression is `NdM`: roll **N** dice with **M** sides. `dM` is short for
`1dM`, and the `d` can be upper or lower case. A plain number **N** is short
for `Nd20`, so `3` rolls three d20s and `2+14` is `2d20+14`. With no
expressions, `dice-roller` rolls a single `1d20`.

### Modifiers

Add any number of modifiers after the dice:

| Modifier | Meaning                          | Example    | Result       |
|----------|----------------------------------|------------|--------------|
| `+X`     | add X once to the total          | `5d20+30`  | dice + 30    |
| `-X`     | subtract X once from the total   | `1d20-2`   | dice − 2     |
| `++X`    | add X to **every** die           | `3d10++5`  | dice + 5 × 3 |
| `--X`    | subtract X from **every** die    | `2d6--1`   | dice − 1 × 2 |

Modifiers can be chained (`2d6+3--1`), and spaces around them are fine:
`3d10 ++ 5` is the same as `3d10++5`.

In the output, `++X` / `--X` are already applied to the dice in the brackets,
so `3d20++3` rolling 5, 6 and 12 shows `[8, 9, 15]`. `+X` / `-X` are listed
after the brackets, since they apply to the total rather than to any one die.

### Limits

- 1 to 1000 dice per expression
- dice need at least 2 sides

## Examples

Roll a d20:

```
$ dice-roller
1d20  [18]  → 18
Total: 18
```

Two d6s plus 3:

```
$ dice-roller 2d6+3
2d6+3  [4, 1] +3  → 8
Total: 8
```

Chained modifiers, +3 once and −1 per die:

```
$ dice-roller 2d6+3--1
2d6+3--1  [0, 5] +3  → 8
Total: 8
```

Three d20s with +5 on each, written as a plain number:

```
$ dice-roller 3++5
3d20++5  [17, 9, 23]  → 49
Total: 49
```

Several rolls at once, with a combined total:

```
$ dice-roller 4d6 1d8+2 1d20
4d6    [1, 5, 6, 6]  → 18
1d8+2  [4] +2        →  6
1d20   [13]          → 13
Total: 37
```

## Errors

Invalid input prints a message to stderr and exits with code 1. If any
expression is invalid, nothing is rolled.

```
$ dice-roller 3x7
error: invalid dice expression "3x7" (expected e.g. 3d6, d20 or 2d6+3)

$ dice-roller 0d6
error: invalid dice count in "0d6" (must be between 1 and 1000)
```

Run `dice-roller --help` for a quick reference.

## Development

```
cargo test                                 # unit, integration and doc tests
cargo clippy --all-targets -- -D warnings  # lint
cargo fmt                                  # format
```
