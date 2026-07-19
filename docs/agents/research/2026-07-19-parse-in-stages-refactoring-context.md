---
date: 2026-07-19T14:41:15+00:00
git_commit: 898e7488955c084bc5e47778a661701abe5335ce
branch: parse-in-stages
topic: "Context required for the multi-stage parsing refactoring ('great refactoring')"
tags: [research, codebase, parsing, argumente, refactoring]
status: complete
---

# Research: Context required for the multi-stage parsing refactoring

## Research Question

"There is a great refactoring in planning. Find out the required context."

Interpreted as: identify the planned refactoring in this repository/branch, and map the codebase
context (design doc, data structures, existing implementation state, call sites, tests) someone
would need in order to plan or implement it.

## Summary

This repository (`kommandozeilen_argumente`, a German/English-bilingual Rust CLI-argument-parsing
library) is on branch `parse-in-stages`, mid-way through a large architectural rewrite of its
argument-parsing algorithm. The design for the new algorithm is written down in
`doc/parsing.md`. The rewrite replaces a single-pass parser (which itself was never finished — the
core `Argumente::parse` entry point has been `todo!()` since before this branch existed) with an
explicit multi-stage pipeline:

1. Parse merged short-name flags (e.g. `-fgh`)
2. Parse non-merged short-name arguments
3. Parse long-name arguments
4. Parse the value strings collected in stages 1–3 into typed values
5. Pick the winning alternative (when `Argumente::Alternativen` is used)
6. Accumulate the typed pieces into the final result struct

All stages accumulate errors instead of stopping at the first one; errors are only reported
(as a `NonEmpty` vector) at the very end.

Work has begun (branch commits) on stage 1 (merged short-name parsing) but nearly every function
that stage relies on is still an unimplemented `todo!()` stub, and the higher-level combination
logic (`Kombiniere` for tuples, used by the `kombiniere!`/`combine!` macros and the derive macro)
has its actual parsing logic commented out, leaving only the help-text generation working.

```
src/
├── lib.rs                     — public re-exports (Argumente, Ergebnis, Beschreibung, Parse, ...)
├── parse.rs                   — ParseArgument/Parse traits: how to build an Argumente<T,F> per Rust type
│                                 (bool, String, numbers, Option<T>, enums via EnumArgument);
│                                 Parse::parse* convenience wrappers all delegate to Argumente::parse
├── argumente.rs                — core `Argumente<'t, T, Fehler>` enum + top-level parse API
│   ├── einzelargument.rs       — `EinzelArgument` enum: Flag | FrühesBeenden | Wert
│   ├── flag.rs                 — `Flag<T>`: on/off argument, invertible with a prefix
│   ├── wert.rs                 — `Wert<T, Fehler>` (~"Value"): argument taking a value string
│   ├── frühes_beenden.rs       — `FrühesBeenden` (~"EarlyExit"): --help/--version-style flags
│   ├── kombiniere.rs            — `Kombiniere` trait + macro-generated tuple impls (combine N args)
│   ├── hilfe.rs                 — help-text data types (`Hilfe`, `Alternativen`, `ErzeugeHilfeText`)
│   └── parser.rs                — `Parser<T,F>` newtype wrapping a parsing closure (currently unused
│                                    scaffolding, no call sites found)
├── beschreibung.rs             — `Name`/`Beschreibung` (argument name/description) + all the
│                                 low-level name-matching logic (`parse_flag`, `parse_mit_wert`,
│                                 `parse_*_merge_short_forms`, `AdjustedMergedShortNames`, `ArgumentInput`)
├── ergebnis.rs                  — `Ergebnis`/`ZwischenErgebnis` (Result/IntermediateResult),
│                                 `Fehler`/`ParseFehler`, `SingeArgResult` (per-argument parse outcome)
├── unicode.rs                   — `Normalisiert` (NFC-normalized string), `Vergleich` (name-compare
│                                 config: string + case-sensitivity), prefix-stripping helpers
├── dyn_to_owned.rs               — macro-generated `ToOwned` for `dyn Fn` trait objects (Bool/Parse/Anzeige)
└── sprache.rs                   — language ("Sprache"/"Language") string tables (not read in full;
                                    only referenced)
doc/
└── parsing.md                   — the design document for the new staged-parsing algorithm
tests/
├── derive.rs                    — integration tests via the `#[derive(Parse)]` macro (6 #[test] fns)
└── hilfe.rs                     — integration tests for help-text generation
kommandozeilen_argumente_derive/ — proc-macro crate implementing `#[derive(Parse)]`/`#[derive(EnumArgument)]`
                                    (depends on the `Argumente`/`Beschreibung` API surface; not read in
                                    detail, but is a consumer that must keep compiling through the refactor)
```

## Detailed Findings

### The design document (`doc/parsing.md`)

Describes six stages (see Summary). Key structural decisions baked into the design:

- Every stage can produce parse errors; they never abort early — errors are collected and reported
  as a `Vec`/`NonEmpty` at the end.
- Merged short-name parsing must happen **first**, because if the short-name and long-name prefixes
  are configured to be identical, merged-short-name matches take precedence over what a later stage
  might otherwise interpret differently.
- Rules for what may be merged into a single short-name block (e.g. `-fgh`): all names share the
  short prefix, only single-grapheme short names participate, at most one value-argument per block
  and it must be last (optionally followed by a value-infix and the value substring), and the
  argument definition must opt in to merging.
- Each stage's output is a `NonEmpty` of **alternatives** (because `Argumente::Alternativen` allows
  multiple candidate argument-set definitions) each carrying: the `Argumente` alternative used,
  collected early-exit triggers, collected flags, a map of value-name → raw value string, and the
  remaining not-yet-consumed input.
- A late stage turns the raw value strings into `Box<dyn Any>` (with `TypeId`) so heterogeneous
  value types can share one map; a final "accumulate" stage downcasts them back and applies the
  user-supplied combine function to build the result struct.

### Core data model: `Argumente<'t, T, Fehler>` (`src/argumente.rs:58`)

Three variants:
- `EinzelArgument(EinzelArgument<'t, T, Fehler>)` — a single argument (flag/value/early-exit)
- `Kombiniere(Box<dyn Kombiniere<'t, T, Fehler>>)` — combination of multiple `Argumente` via a
  user function (built by the `kombiniere!`/`combine!` macros, or the derive macro)
- `Alternativen(Box<NonEmpty<Self>>)` — take the first alternative that doesn't error

`EinzelArgument<'t, T, Fehler>` (`src/argumente/einzelargument.rs:26`) is itself:
- `Flag(Flag<'t, T>)`
- `FrühesBeenden { frühes_beenden, wert, anzeige }`
- `Wert(Wert<'t, T, Fehler>)`

### What's implemented vs. stubbed

The top-level entry point `Argumente::parse` (`src/argumente.rs:342-347`) is `todo!()`. This was
already true before the `parse-in-stages` branch started (verified via
`git show e8a9e00:src/argumente.rs`), i.e. the crate has never had a working parse implementation
for the new `Argumente` enum design — the whole rewrite (of which staged-parsing is the current
plan) has been in progress across multiple prior sessions.

New scaffolding added on this branch, all still `todo!()`:
- `Flag::parse_merged_short_forms` (`src/argumente/flag.rs:186-192`)
- `Wert::parse_merged_short_forms` (`src/argumente/wert.rs:401-416`)
- `FrühesBeenden::parse_merged_short_forms` (`src/argumente/frühes_beenden.rs:100-106`)
- `Argumente::parse_merged_short_forms` for the `Kombiniere` variant
  (`src/argumente.rs:315-333`, the `Kombiniere(kombiniere) => todo!()` arm; `EinzelArgument` and
  `Alternativen` arms are implemented by delegating/flattening)
- `Argumente::konvertiere_fehler` (`src/argumente.rs:820-826`, used by `fehler_from`/`error_from`)

The tuple-based `Kombiniere` trait impls (`src/argumente/kombiniere.rs`, generated for 1..26-tuples
by `impl_kombiniere_tuple!`) have their actual parsing logic (`parse`, `parse_merged_short_forms`)
entirely commented out (see lines 109-207 and 263-277) — only `erzeuge_hilfe_text`/`debug_fmt` are
live. The commented-out code is itself a **previous, single-pass parsing sketch** (loops that call
a since-removed `parse_rekursiv`/`parse_rekursiv_merged_short_forms` per sub-argument and merge
`ZwischenErgebnis`s), left in place as reference/history rather than deleted.

`src/argumente/parser.rs` defines a `Parser<'t, T, F>(Box<dyn Fn(ArgumentList) -> Vec<(ArgumentList, Ergebnis<'t,T,F>)>>)`
newtype that has no call sites elsewhere in `src/` — it looks like preparatory/abandoned scaffolding
for one of the stages, not yet wired up.

### Supporting types already built for the new pipeline

`src/argumente.rs:250-294` defines the per-stage result shapes described in `doc/parsing.md`:
`ParsedEarlyExit`, `ParsedShortFlag`, `ParsedValueName`, `ParsedValue`, and
`ParseMergedShortFormsResult<'s, T, F>` (definition reference, early_exits vec, flags vec, values
map, remaining `Vec<Option<OsString>>`). All fields are currently documented only with `/// TODO`.

`src/beschreibung.rs` contains the actual low-level string/name matching that stage 1 and 2 build
on:
- `Name::parse_flag` / `parse_flag_aux` — non-merged flag matching (already working, pre-dates this
  branch)
- `Name::parse_merged_short_name` (`beschreibung.rs:97`) and
  `Name::parse_flag_merge_short_forms`/`_aux` (`beschreibung.rs:171-254`) — the actual merged
  short-name matching logic, already implemented and used by tests indirectly (not directly,
  since `Flag::parse_merged_short_forms` itself is still `todo!()`)
- `Name::parse_mit_wert` / `parse_mit_wert_merge_short_forms` — matching a name followed by a value,
  with a merged-short-form variant that tracks progressive stripping of graphemes from a short-name
  block (`AdjustedMergedShortNames`, `MergedShortNameSuffix::{Removed,Unchanged}`)
- `ArgumentInput`/`ArgumentInputRef` — wraps a raw `OsString` input, tagging whether it has already
  been partially consumed by merged-short-name stripping (`Unchanged` vs
  `AdjustedMergedShortNames`)

`src/ergebnis.rs` defines the intermediate/final result plumbing:
- `Ergebnis<'t, T, E>` (final: `Wert`/`FrühesBeenden`/`Fehler`) vs.
  `ZwischenErgebnis<'t, T, E, A>` (adds an `Incomplete(A)` variant for "an intermediate stage
  couldn't produce an unambiguous result yet")
- `SingeArgResult<'t, T, E>` (`ergebnis.rs:20-42`) — three-way outcome of parsing a single OsString
  argument: `FullParse` (fully resolved, e.g. `--name=value`/`-nvalue`), `NameOnly` (name matched,
  value expected in the *next* OsString), `IncompleteParse` (more input needed, with a continuation
  closure) — each carrying an `Option<AdjustedMergedShortNames>` for the leftover merged-short-name
  state.
- `Fehler`/`ParseFehler`/`KommentierterParseFehler` (annotated parse error) — already fleshed out
  including human-readable message rendering (`erstelle_fehlermeldung`) in both languages.

### `ParseArgument`/`Parse` traits (`src/parse.rs`)

`ParseArgument` is how each concrete Rust type (bool, String, numeric primitives via
`impl_parse_argument!`, `Option<T>`, and any `EnumArgument`-implementing enum) builds an
`Argumente<'t, Self, String>` from a `Beschreibung`. The `Option<T>` implementation
(`parse.rs:289-384`) is the most involved: it wraps the inner type's `Argumente` and — depending on
which `EinzelArgument`/`Kombiniere`/`Alternativen` variant the inner type produced — adjusts
display/parse closures and injects a default-value fallback via `OptionHelper` (a `Kombiniere`
impl used purely for its `erzeuge_hilfe_text` passthrough) and `erstelle_ergebnis_anpassen`
(post-processes an `Ergebnis` to substitute the configured default when nothing else went wrong).

The `Parse` trait's many `parse_*`/`parse_*_aus_env`/`parse_vollständig*` convenience methods
(`parse.rs:434-854`) are thin wrappers that all bottom out in `Self::kommandozeilen_argumente().parse(args)`
— i.e. every one of them is currently unusable end-to-end because `Argumente::parse` is `todo!()`.
The equivalent inherent methods live on `Argumente` itself in `src/argumente.rs:336-799` (same
wrapper structure, same terminal dependency on `Argumente::parse`).

### Tests

`tests/derive.rs` (6 `#[test]` functions) and `tests/hilfe.rs` exercise the crate through the
`#[derive(Parse)]` macro. Given `Argumente::parse` is `todo!()`, any test that actually invokes
parsing (as opposed to only generating help text) would currently panic; `tests/hilfe.rs`'s name
suggests it targets the help-text path specifically, which does not depend on the unfinished parse
pipeline.

### Derive macro crate

`kommandozeilen_argumente_derive/` (proc-macro crate) generates `ParseArgument`/`EnumArgument`
implementations and is a consumer of the `Argumente`/`Beschreibung`/`Kombiniere` public API. It was
not read in detail in this pass, but any change to field names/shapes in `Argumente`,
`EinzelArgument`, `Beschreibung`, or the `kombiniere!` macro's expected call shape is a
cross-crate-compatibility concern for it.

## Code References

- `doc/parsing.md` — full design of the 6-stage pipeline
- `src/argumente.rs:58` — `Argumente<'t, T, Fehler>` enum definition
- `src/argumente.rs:250-294` — per-stage result structs (`ParseMergedShortFormsResult` etc.), all `/// TODO`
- `src/argumente.rs:315-333` — `Argumente::parse_merged_short_forms`, `Kombiniere` arm is `todo!()`
- `src/argumente.rs:342-347` — `Argumente::parse`, the main entry point, `todo!()`
- `src/argumente.rs:820-826` — `Argumente::konvertiere_fehler`, `todo!()`
- `src/argumente/einzelargument.rs:26-60` — `EinzelArgument` enum (Flag/FrühesBeenden/Wert)
- `src/argumente/flag.rs:186-192` — `Flag::parse_merged_short_forms`, `todo!()`
- `src/argumente/wert.rs:401-416` — `Wert::parse_merged_short_forms`, `todo!()`
- `src/argumente/frühes_beenden.rs:100-106` — `FrühesBeenden::parse_merged_short_forms`, `todo!()`
- `src/argumente/kombiniere.rs:109-207,263-277` — commented-out (old single-pass) `parse`/`parse_merged_short_forms` for tuple `Kombiniere` impls
- `src/argumente/kombiniere.rs:280-292` — `erzeuge_hilfe_text`/`debug_fmt` `todo!()` for the `ZwischenErgebnis`-tuple `Kombiniere` impl
- `src/argumente/parser.rs` — `Parser<'t,T,F>` newtype, currently unreferenced elsewhere
- `src/beschreibung.rs:57-93` — `AdjustedMergedShortNames`, `ArgumentInput`, `MergedShortNameSuffix`
- `src/beschreibung.rs:97-216` — merged-short-name matching (`parse_merged_short_name`, `parse_flag_merge_short_forms_aux`)
- `src/beschreibung.rs:278-422` — value-with-name matching, incl. merged-short-form variant
- `src/ergebnis.rs:20-89` — `SingeArgResult` (per-argument outcome: FullParse/NameOnly/IncompleteParse)
- `src/ergebnis.rs:98-119` — `ZwischenErgebnis` (adds `Incomplete(A)` over `Ergebnis`)
- `src/parse.rs:289-384` — `ParseArgument` impl for `Option<T>` (most complex existing consumer of the `Argumente` variants)
- `src/parse.rs:434-460` — `Parse::parse`, bottoms out in `Argumente::parse`
- `tests/derive.rs`, `tests/hilfe.rs` — existing integration test surface

## Architecture Documentation

- Bilingual API convention: nearly every public German-named item (`Argumente`, `Beschreibung`,
  `Ergebnis`, `Fehler`, `konvertiere`, ...) has a 1:1 English synonym (`Arguments`, `Description`,
  `Result`, `Error`, `convert`, ...) implemented as a thin wrapper or type alias. This doubling
  applies throughout `src/argumente.rs`, `src/ergebnis.rs`, `src/beschreibung.rs`, and `src/parse.rs`,
  and would need to be maintained for any new methods added by the refactor.
- Closures stored in structs (`Flag.konvertiere`, `Wert.parse`/`anzeige`/`anzeige_fehler`,
  `EinzelArgument::FrühesBeenden.anzeige`) are held as `Cow<'t, dyn Trait>` where `Trait` is one of
  the `dyn_to_owned.rs`-generated `Bool`/`Parse`/`Anzeige` traits, giving them a `Clone`-via-`ToOwned`
  capability despite being trait objects (via `dyn_clone`).
- Name matching is unicode-normalization-aware and case-sensitivity-configurable throughout
  (`Normalisiert`, `Vergleich`, `Case`), not just ASCII string equality.
- The crate models "can't decide yet" as a first-class result state (`ZwischenErgebnis::Incomplete`,
  `SingeArgResult::{NameOnly,IncompleteParse}`) rather than only success/error, which is what a
  multi-stage, streaming (`OsString`-by-`OsString`) parser needs in order to say "the value for
  `--name` is the *next* argument" without special-casing.

## Open Questions

- `src/argumente/parser.rs`'s `Parser` newtype has no current call sites — unclear whether it is
  intended to be the eventual home of the stage-4/5 execution logic or is dead scaffolding to be
  removed.
- `src/sprache.rs` (language string tables) was not read in this pass; not expected to be affected
  by the parsing-algorithm rewrite but is a dependency of nearly every module touched here.
- `kommandozeilen_argumente_derive/` internals were not inspected; whether it assumes anything about
  `Kombiniere`'s currently-commented-out `parse`/`parse_merged_short_forms` methods (as opposed to
  only `erzeuge_hilfe_text`) is unverified.
- No explicit task list or ordering exists yet beyond `doc/parsing.md`'s six stage descriptions and
  the four recent commit messages on this branch — the actual implementation plan (which stub to
  fill in first, how `SingeArgResult`/`ZwischenErgebnis`/`ParseMergedShortFormsResult` compose across
  stage boundaries) has not been written down anywhere found in the repo.

## Follow-up Research 2026-07-19T14:48:46+00:00

**Stated direction for the refactoring (not yet reflected in the current codebase):** the
main structs and methods should be named using the English terms, with the German-named
variants becoming the optional/secondary synonyms. This is the reverse of the current state
of the codebase (see "Architecture Documentation" above): today the German names are the
primary implementation (e.g. `Argumente`, `Beschreibung`, `Ergebnis`, `Fehler`, `Flag`,
`Wert`, `FrühesBeenden`, `neu`, `parse_merged_short_forms`'s German-first doc comments) and
the English names (`Arguments`, `Description`, `Result`, `Error`, `Value`, `EarlyExit`,
`new`, ...) are thin wrapper types/methods or type aliases pointing back to the German ones,
as seen throughout `src/argumente.rs`, `src/argumente/flag.rs`, `src/argumente/wert.rs`,
`src/argumente/frühes_beenden.rs`, `src/ergebnis.rs`, `src/beschreibung.rs`, and `src/parse.rs`.
This is recorded here as context/requirement for planning the refactor, since it affects how
any newly implemented pipeline code (the six stages, the still-`todo!()` methods enumerated
above) should be named and how the German/English wrapper relationship should be structured
going forward.
