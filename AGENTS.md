# AGENTS.md

This file provides guidance to LLMs like Claude Code (claude.ai/code) when working with code in this repository.

## Project

`kommandozeilen_argumente` is a Rust command-line argument parser crate with optional derive-macro
support, published bilingually: every public API item has an English primary name and a German
mirror type or synonym. The staged parser and English-primary API refactor are complete on the
`parse-in-stages` branch.

Cargo workspace with two crates:

- Root crate (`kommandozeilen_argumente`, `src/`) — the parser itself.
- `kommandozeilen_argumente_derive/` — proc-macro crate providing `#[derive(Parse)]` and
  `#[derive(EnumArgument)]`, gated behind the `derive` feature.

## Common commands

```sh
cargo build --all-features                  # build both crates
cargo test                                  # unit + default-feature integration tests (including tests/help.rs)
cargo test --workspace --all-features       # run all workspace tests, including derive-crate unit tests and derive-/german-gated tests
cargo test --all-features --test derive     # just the derive-macro integration test
cargo test --all-features <test_name>       # single test by name substring
cargo clippy --all-features --all-targets   # lint (extensive lint config in Cargo.toml)
cargo +nightly fmt ; cargo fmt              # apply the project's nightly rustfmt configuration,
                                            # then run stable rustfmt (see rustfmt.toml — max_width 100, Unix newlines)
cargo run --example derive_en --features derive   # run an example (also: derive, function, funktion)
```

Note: `tests/derive.rs` and the `derive`-featured examples require `--features derive` (or
`--all-features`) to build at all.

## Architecture

### Bilingual API duality

The defining structural pattern of this crate: nearly every public type/function has an English
primary name (for example `Arguments`, `Value`, `Description`, `Result`, `Error`, `Language`) and
a German mirror (`Argumente`, `Wert`, `Beschreibung`, `Ergebnis`, `Fehler`, `Sprache`). The
English-primary API is always available; the German mirror API is behind the `german` feature.
`Language::GERMAN` remains available without that feature, but the `Sprache` mirror requires it.
The English types contain the implementation; German mirrors convert to and from them where
distinct field or variant names are necessary. When touching a public type, keep both APIs in sync,
ensure builds work both with and without `german`, and use idiomatic German names in the mirror
rather than transliterations.

### Core modules (`src/`)

- `unicode.rs` — grapheme-aware, case-sensitivity-aware string comparison (`Normalisiert`/
  `Normalized`, `Vergleich`/`Compare`). Short names are defined to be exactly one grapheme
  cluster (via `unicode-segmentation`), which is what makes merged short-flag parsing (`-fgh`)
  well-defined.
- `description.rs` — `Name`/argument metadata, `ArgumentInput`, and low-level name matching.
  Its grapheme-aware matching logic underpins every recognition stage and must not regress.
- `arguments.rs` + `src/arguments/*.rs` — the primary `Arguments` enum and staged parser;
  `argumente.rs` is the German `Argumente` mirror. The submodules implement `Flag`, `Value`,
  `EarlyExit`, `SingleArgument`, `Combine`, and help-text generation.
- `parse.rs` — the `Parse`/`ParseArgument` traits used by the derive macro.
- `outcome.rs` — `Result`/`Error` plus their `Ergebnis`/`Fehler` mirrors.
- `language.rs` — `Language`/`Sprache` for localized defaults.
- `unicode.rs` — `Normalized`/`Compare` plus German mirrors; it defines the grapheme rule for
  short names.

### Derive macro (`kommandozeilen_argumente_derive/`)

Generates `Parse` impls for structs by calling directly into the root crate's current API
surface (constructors like `Beschreibung::neu_mit_sprache`, `Wert::neu_mit_sprache`, the
`combine!`/`kombiniere!` macros, etc.). Any rename of root-crate public API must update every
call site here too — this crate has no independent logic of its own beyond attribute parsing
(`utility.rs`, `enum_argument.rs`) and code generation (`parse.rs`).

### Staged parsing

The parser is implemented as a six-step pipeline documented in `docs/parsing.md`: recognize
merged short forms, standalone short forms, and long forms; validate typed values for each
candidate; select the first successful candidate; then consume the selected definition to
accumulate its result. Recognition preserves `ArgumentInput` state, so an adjusted remainder from
a merged short block is never reused as a value. Raw values remain `OsString`s until their owning
`Value` parses them; there is no `Any`/downcast stage.

Consult `docs/parsing.md` and the implementation when changing parser behavior. The completed
refactor's transient planning and research documents were deliberately removed.

## Lint configuration

`Cargo.toml` sets an unusually strict workspace-wide clippy configuration (`pedantic`, `cargo`,
and a large curated set of `restriction` lints, each individually enabled/disabled with a German
comment explaining why). Read the `[workspace.lints.clippy]` table before adding new `#[allow]`
attributes — most common restriction lints are already deliberately toggled one way or the other
project-wide, and per-item `#[allow]`s are used to disable aggressive lints locally.
