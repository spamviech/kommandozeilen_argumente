---
date: 2026-10-14T21:22:36.646990+00:00
git_commit: ac611672f201b998954269cf04d94468bdc978be
branch: parse-in-stages
topic: "Expand parser test coverage"
tags: [plan, tests, unicode, staged-parser, derive, german-api, help]
status: draft
---

# Expand Parser Test Coverage Implementation Plan

## Overview

Add regression coverage for Unicode-aware name recognition, staged-parser candidate handling, public German-mirror entry points, derive-generated definitions, and help output. The work is test-only except for removing the now-obsolete Unicode-name TODO; it must preserve the current parser API and behavior.

## Current State Analysis

- `src/unicode.rs` implements NFC/CJK normalization, case-aware equality, and grapheme-boundary prefix stripping, but has no unit tests.
- `src/description.rs` owns long/short name recognition and merged-short segmentation, also with no direct tests.
- `src/arguments/value.rs:879-911` already covers normalized long and short *value* names at the recognition level, but flags, early exits, full parser accumulation, case-insensitive matching, and multi-grapheme rules are uncovered.
- `tests/staged_parser.rs` has only six end-to-end tests. It does not cover ambiguous merged short values, adjusted remainders, defaults/missing inputs/repeated values, all-failed alternatives, or non-Unicode `OsString` values.
- `tests/derive.rs`, `tests/help.rs`, and `tests/hilfe.rs` exercise their basic paths but do not cover Unicode user-defined names.
- `src/arguments/argumente.rs:33` retains `// TODO tests mit Unicode-namen`, even though parsing is delegated to the English-primary `Arguments` implementation and selected Unicode value recognition tests already exist.
- `docs/parsing.md` documents the six-stage rules that the tests must preserve, including one-grapheme merged short names and the prohibition on reusing adjusted remainders as standalone names or values.

## Desired End State

The suite directly protects the Unicode/string primitives and verifies the documented parser contract through public parsing APIs. All feature combinations continue to build, the German mirror has a public regression test, derive-generated Unicode definitions parse correctly, and Unicode help syntax is asserted as emitted by the current byte-length-based formatter. The stale TODO is removed.

## What We're NOT Doing

- No parser, Unicode-normalization, derive-macro, or help-layout behavior changes.
- No switch from byte-length to terminal-display-width help alignment; existing `String::len()` formatting is tested as-is.
- No compile-fail derive test harness or new testing dependency.
- No Windows emulation of Unix non-UTF-8 `OsString` behavior; that assertion is Unix-gated.
- No tests for unimplemented subcommands or positional arguments.

## Architecture and Code Reuse

Use existing test conventions and public construction APIs rather than adding test-only hooks:

```text
Unicode primitive tests              Name-recognition tests
src/unicode.rs                       src/description.rs
        │                                     │
        └───────────────┬─────────────────────┘
                        ▼
              staged recognition/validation
              src/arguments/{flag,value,early_exit}.rs
                        │
                        ▼
               public `Arguments::parse`
               tests/staged_parser.rs
                 ├── derive-generated definitions: tests/derive.rs
                 ├── German mirror: tests/hilfe.rs
                 └── generated help: tests/{help,hilfe}.rs
```

- Reuse `Description::new` / `Beschreibung::neu` to configure explicit names and `Case` values.
- Reuse `Arguments::combine`, `Arguments::alternatives`, `Value`, `Flag`, `EarlyExit`, `SingleArgument`, and `nonempty!` as in `tests/staged_parser.rs`.
- Reuse `ArgumentInput::AdjustedMergedShortNames` plus `AdjustedMergedShortNames`/`MergedShortNameSuffix` assertions already demonstrated in `tests/derive.rs`.
- Reuse `OsStringExt::from_vec` behind `#[cfg(unix)]` for the raw non-UTF-8 test; values already convert such inputs to `ParseError::InvalidString` (`src/arguments/value.rs:227-230`).
- Use `unicode-normalization` behavior already encapsulated by `Normalized`, and `unicode-segmentation` semantics already used by `Compare::strip_as_prefix_n` and `Name` parsing; no direct new third-party APIs are required.

Affected files:

- `src/unicode.rs` — add unit tests for normalization, equality, and grapheme-safe prefix removal.
- `src/description.rs` — add unit tests for name matching and merged-short segmentation rules.
- `src/arguments/argumente.rs` — remove obsolete Unicode test TODO.
- `tests/staged_parser.rs` — add public, end-to-end staged parsing scenarios.
- `tests/derive.rs` — add a derived struct with Unicode names and case configuration.
- `tests/help.rs` — assert English generated help containing Unicode syntax and its current padding.
- `tests/hilfe.rs` — assert German mirror parsing and generated help containing Unicode syntax.
- `docs/parsing.md` — only amend if an implemented test exposes an undocumented invariant; otherwise leave unchanged.

## Performance Considerations

The changes add only small deterministic unit/integration cases. They do not alter runtime code or introduce test dependencies. Candidate-ambiguity cases should use the smallest possible argument definitions to avoid combinatorial candidate expansion.

## Migration Notes

No migration is required. This is additive test coverage with removal of a stale developer comment.

## Phase 1: Unicode Primitives and Name Recognition

Add local unit tests around the behavior that controls every parser stage, then remove the TODO once direct and integration coverage exists.

**Tasks**:
- [ ] In `src/unicode.rs`, add a `#[cfg(test)]` module that verifies `Normalized::new` canonicalizes a decomposed accent to NFC and normalizes the CJK compatibility form handled by `cjk_compat_variants`.
- [ ] In `src/unicode.rs`, test `Compare`/`Normalized::eq_with_case` with a non-ASCII case pair (for example `Ä`/`ä`) under both `Case::Sensitive` and `Case::Insensitive`.
- [ ] In `src/unicode.rs`, test `Compare::strip_as_prefix_n` with a decomposed/composed prefix and with a multi-scalar grapheme, asserting the returned prefix/suffix split occurs only at a grapheme boundary.
- [ ] In `src/description.rs`, add a `#[cfg(test)]` module that constructs `Name`/`Description` values and verifies composed definition names match canonically equivalent decomposed long and short inputs.
- [ ] In `src/description.rs`, test merged short recognition with a multi-scalar but single grapheme short name followed by an ASCII short name; assert each recognition preserves whole graphemes in `AdjustedMergedShortNames`.
- [ ] In `src/description.rs`, test that a configured short name containing two graphemes remains usable as a standalone name but is not recognized as a merged-short component.
- [ ] In `src/description.rs`, test that an input starting with the long prefix is never reinterpreted as a short-prefix merged block, including when the short and long prefixes overlap.
- [ ] Remove `// TODO tests mit Unicode-namen` from `src/arguments/argumente.rs`; do not replace it with another broad TODO.

**Automated Verification**:
- [ ] The new `unicode` unit tests pass with `cargo test unicode::tests`.
- [ ] The new `description` unit tests pass with `cargo test description::tests`.
- [ ] `cargo test` passes.

## Phase 2: End-to-End Staged Parser Contract

Extend the public parser integration suite with compact tests that traverse all six documented stages and assert outcomes plus unconsumed `ArgumentInput` exactly where stage boundaries matter.

**Tasks**:
- [ ] In `tests/staged_parser.rs`, add a combined public-parser test for canonically equivalent Unicode long and short names covering a flag, a typed value, and an early exit; assert successful values/messages and no remaining input.
- [ ] Add an end-to-end merged-short test combining a multi-scalar single-grapheme short flag with an ASCII flag, proving the entire grapheme is consumed rather than its Unicode scalar components.
- [ ] Add an end-to-end test distinguishing standalone and merged behavior for a multi-grapheme configured short name: it may match as a complete standalone form, but it must not be consumed one grapheme at a time as a merged-short component.
- [ ] Add an ambiguity test where a merged short value name is followed by text that can be interpreted either as its inline value or as subsequent flags; assert the first fully validating candidate follows the documented value-first ordering and that fallback reaches the flags interpretation when the value candidate cannot validate.
- [ ] Add an adjusted-remainder regression test showing it is neither reconsidered by standalone short/long stages nor consumed as a following raw value; compare the exact `ArgumentInput::AdjustedMergedShortNames` representation.
- [ ] Add public-parser tests for a missing required value, an omitted value satisfied by its default, and repeated values retaining the last raw value.
- [ ] Extend alternative tests to cover an all-failed set (aggregated errors), declaration-order selection when more than one branch validates, and an early-exit branch winning without requiring unrelated arguments.
- [ ] Add a Unix-only integration test using `std::os::unix::ffi::OsStringExt::from_vec` to show a non-UTF-8 raw value reaches the typed value path and produces `ParseError::InvalidString` rather than being normalized or silently consumed; keep the test absent on non-Unix targets.

**Automated Verification**:
- [ ] Each new `tests/staged_parser.rs` case passes under `cargo test --all-features --test staged_parser`.
- [ ] The non-UTF-8 case is compiled and passes on Unix via `cargo test --all-features --test staged_parser`.
- [ ] `cargo test --all-features` passes.

## Phase 3: Derived, German-Mirror, and Help Public APIs

Exercise the remaining user-facing entry points with Unicode names, preserving the exact output semantics already implemented.

**Tasks**:
- [ ] In `tests/derive.rs`, add a `#[derive(Parse)]` fixture with explicit Unicode long and short attributes, an automatically derived short name whose first grapheme is multi-scalar, and a non-ASCII case-insensitive name; parse composed/decomposed input and a merged short block to verify macro-generated `Description` definitions preserve the core parser behavior.
- [ ] In `tests/hilfe.rs`, add a German-feature public API test that constructs `Beschreibung`, `Wert`, and `Argumente` directly with Unicode names, parses canonical-equivalent input, and asserts the expected `Ergebnis` and no remaining arguments.
- [ ] In `tests/help.rs`, configure an English Unicode-named argument and assert its help early-exit message contains the expected Unicode syntax and the exact byte-length-based padding currently emitted by `Arguments::with_help_early_exit_with_language`.
- [ ] In `tests/hilfe.rs`, add the analogous German generated-help assertion using `Argumente::mit_hilfe_frühes_beenden_mit_sprache`, checking localized headings/syntax and current byte-length-based padding.
- [ ] Review `docs/parsing.md` against the completed tests. Update it only if a tested invariant is missing or inaccurate; otherwise make no documentation edit and record that its current rules already specify the covered behavior.

**Automated Verification**:
- [ ] The Unicode derive case passes with `cargo test --all-features --test derive`.
- [ ] The English help test passes with `cargo test --test help`.
- [ ] The German mirror and help tests pass with `cargo test --features german --test hilfe`.
- [ ] `cargo test --all-features` passes.

## Phase 4: Feature Matrix and Lint Verification

Confirm that the additional test code respects feature gating and the workspace’s strict lint policy.

**Tasks**:
- [ ] Run the default-feature test suite to ensure unit tests and non-feature-gated integrations still compile without `german` or `derive`.
- [ ] Run the German-only feature test suite to validate feature gates independently of derive.
- [ ] Run the all-feature test suite and targeted derive test after formatting.
- [ ] Format changed Rust sources with the project-prescribed nightly-then-stable sequence.
- [ ] Run strict workspace Clippy and fix test-code lint findings through idiomatic test design first. Use a narrowly scoped `#[expect(..., reason = ...)]` only when the test necessarily needs the flagged construct and no clearer compliant alternative exists; follow the existing test style for any unavoidable expectation.

**Automated Verification**:
- [ ] `cargo test` passes.
- [ ] `cargo test --features german` passes.
- [ ] `cargo test --all-features` passes.
- [ ] `cargo +nightly fmt ; cargo fmt` completes with no remaining formatting changes.
- [ ] `cargo clippy --all-features --all-targets` passes.

## References

- Parsing contract: `docs/parsing.md:13-75`
- Unicode normalization and prefix logic: `src/unicode.rs:43-105`, `src/unicode.rs:334-375`
- Name and merged-short recognition: `src/description.rs:105-238`, `src/description.rs:414-486`
- Existing normalized value tests: `src/arguments/value.rs:879-911`
- Parser candidate selection: `src/arguments.rs:460-531`
- Existing staged-parser integration patterns: `tests/staged_parser.rs:21-124`
- Existing adjusted-remainder assertion: `tests/derive.rs:279-314`
- Derive short-name grapheme extraction: `kommandozeilen_argumente_derive/src/parse.rs:535-565`
- Help byte-length formatter: `src/arguments.rs:1009-1045`, `src/arguments.rs:1180-1269`
