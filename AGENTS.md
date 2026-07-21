# AGENTS.md

This file provides guidance to LLMs like Claude Code (claude.ai/code) when working with code in this repository.

## Project

`kommandozeilen_argumente` is a Rust command-line argument parser crate with optional derive-macro
support, published bilingually: every public API item has a German name (the original) and an
English synonym. It is currently mid-refactor on the `parse-in-stages` branch (see "Active
refactor" below) — check current branch/state before assuming the README's described API is fully
implemented.

Cargo workspace with two crates:

- Root crate (`kommandozeilen_argumente`, `src/`) — the parser itself.
- `kommandozeilen_argumente_derive/` — proc-macro crate providing `#[derive(Parse)]` and
  `#[derive(EnumArgument)]`, gated behind the `derive` feature.

## Common commands

```sh
cargo build --all-features                  # build both crates
cargo test                                  # unit + integration tests (excludes derive-gated tests)
cargo test --all-features                   # includes tests/derive.rs (requires "derive" feature)
cargo test --all-features --test derive     # just the derive-macro integration test
cargo test --all-features <test_name>       # single test by name substring
cargo clippy --all-features --all-targets   # lint (extensive lint config in Cargo.toml)
cargo fmt                                   # format (see rustfmt.toml — max_width 100, Unix newlines)
cargo run --example derive_en --features derive   # run an example (also: derive, function, funktion)
```

Note: `tests/derive.rs` and the `derive`-featured examples require `--features derive` (or
`--all-features`) to build at all.

## Architecture

### Bilingual API duality

The defining structural pattern of this crate: nearly every public type/function has a German
name (canonical/original, e.g. `Argumente`, `Wert`, `Beschreibung`, `Ergebnis`, `Fehler`,
`Sprache`) and an English synonym (`Arguments`, `Value`, `Description`, `Result`, `Error`,
`Language`). Historically the German type held the real logic and the English name was a thin
alias/wrapper; the in-progress refactor (see below) inverts this so English is primary. When
touching any public type, check both names are kept in sync and that field/variant names in the
German mirror type are idiomatically German (not just transliterated).

### Core modules (`src/`)

- `unicode.rs` — grapheme-aware, case-sensitivity-aware string comparison (`Normalisiert`/
  `Normalized`, `Vergleich`/`Compare`). Short names are defined to be exactly one grapheme
  cluster (via `unicode-segmentation`), which is what makes merged short-flag parsing (`-fgh`)
  well-defined.
- `beschreibung.rs` — `Name`/argument metadata and the low-level name-matching logic
  (`parse_flag`, `parse_mit_wert`, `parse_*_merge_short_forms`). This matching logic predates
  the current refactor and must not regress.
- `argumente.rs` + `src/argumente/*.rs` — the `Argumente`/`Arguments` enum and its variants:
  `flag.rs` (on/off flags, with invert-prefix support), `wert.rs` (value-taking arguments),
  `frühes_beenden.rs` (early-exit arguments like `--help`/`--version`), `kombiniere.rs`
  (combinator trait/macros for composing multiple arguments into a struct), `einzelargument.rs`,
  `hilfe.rs` (help-text generation).
- `parse.rs` — the `Parse`/`ParseArgument` traits that the derive macro generates impls for.
- `ergebnis.rs` — `Ergebnis`/`Result`, `Fehler`/`Error` types for parse outcomes.
- `sprache.rs` — `Sprache`/`Language` for localized default strings (German/English built in).

### Derive macro (`kommandozeilen_argumente_derive/`)

Generates `Parse` impls for structs by calling directly into the root crate's current API
surface (constructors like `Beschreibung::neu_mit_sprache`, `Wert::neu_mit_sprache`, the
`combine!`/`kombiniere!` macros, etc.). Any rename of root-crate public API must update every
call site here too — this crate has no independent logic of its own beyond attribute parsing
(`utility.rs`, `enum_argument.rs`) and code generation (`parse.rs`).

### Active refactor: staged parsing

`doc/parsing.md` describes a planned 6-stage parsing pipeline (parse merged short names → parse
non-merged short names → parse long names → parse `OsString` values via `Box<dyn Any>` →
pick alternative → accumulate results into the result struct), replacing a single-pass parser.
Several core methods are currently `todo!()` pending this work (e.g. `Argumente::parse`,
`parse_merged_short_forms` on multiple types). Design/status docs for this effort live under
`docs/agents/plans/` and `docs/agents/research/` — check the most recent dated file there before
planning further parsing changes, since it tracks current phase/progress in detail.

## Lint configuration

`Cargo.toml` sets an unusually strict workspace-wide clippy configuration (`pedantic`, `cargo`,
and a large curated set of `restriction` lints, each individually enabled/disabled with a German
comment explaining why). Read the `[workspace.lints.clippy]` table before adding new `#[allow]`
attributes — most common restriction lints are already deliberately toggled one way or the other
project-wide, and per-item `#[allow]`s are used to disable aggressive lints locally.
