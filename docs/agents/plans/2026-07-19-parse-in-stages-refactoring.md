---
date: 2026-07-19T15:43:42Z
git_commit: fd055a06062d02e230eed1ff0835cf7659e28a61
branch: parse-in-stages
topic: "English-primary API rename, then multi-stage argument parsing"
tags: [plan, argumente, parsing, rename, breaking-change]
status: draft
---

# English-Primary Rename + Multi-Stage Parsing Implementation Plan

## Overview

This branch (`parse-in-stages`) is mid-way through replacing the crate's single-pass
argument parser with an explicit 6-stage pipeline (`doc/parsing.md`). Before implementing
the pipeline, the crate's entire public API is inverted from German-primary/English-secondary
to **English-primary/German-secondary**: every public data-carrying type that currently has a
German name (with an English synonym implemented as a thin wrapper or type alias) is rewritten
so the **English name is the real type with all logic**, and the **German name becomes a fully
separate mirror type** with idiomatic German field/variant names, connected via bidirectional
`From` impls. This is a breaking change to the still-unreleased `0.3.0` (the last actually
published release was `0.2.1`), so no further version bump is required — both crates simply stay
at `0.3.0` and ship this as part of that not-yet-released version.

Once the rename lands and compiles (with existing tests passing), the 6-stage pipeline is
implemented stage by stage on top of the renamed API.

## Current State Analysis

- Nearly every public German-named item has a same-shaped English "synonym" today, but almost
  all of them are implemented backwards: the German type/method holds the real logic, the
  English one is a `pub type X = Y;` alias or a one-line delegating wrapper. Confirmed alias
  pairs: `Arguments = Argumente`, `Value = Wert`, `Description = Beschreibung`,
  `Result = Ergebnis`, `Error = Fehler`, `ParseError = ParseFehler`,
  `AnnotatedParseError = KommentierterParseFehler`, `IntermediateResult = ZwischenErgebnis`,
  `Normalized = Normalisiert`, `Compare = Vergleich`, `Language = Sprache`. Dozens of method
  pairs (`neu`/`new`, `parse_vollständig`/`parse_complete`, ...) follow the same pattern.
- Some pipeline-central types have no English name at all yet: `EinzelArgument`, `Kombiniere`
  (trait), `Hilfe`, `Alternativen` (help-text enum), `ErzeugeHilfeText` (trait).
- `Flag<'t, T>` and `Name<'t>` are single types with no German/English duality (the word "Flag"/
  "Name" is identical in both languages), but their **fields** are German-named
  (`beschreibung`, `invertiere_präfix`, `invertiere_infix`, `konvertiere`, `anzeige` /
  `lang_präfix`, `lang`, `kurz_präfix`, `kurz`).
- The staged-parsing pipeline (`doc/parsing.md`) is largely unimplemented: `Argumente::parse`
  is `todo!()` (predates this branch), `Flag`/`Wert`/`FrühesBeenden::parse_merged_short_forms`
  are `todo!()`, the `Kombiniere` tuple impls' actual parsing logic is commented out (only
  `erzeuge_hilfe_text`/`debug_fmt` are live), and `Argumente::parse_merged_short_forms`'s
  `Kombiniere` arm is `todo!()`. `src/argumente/parser.rs`'s `Parser` newtype has no call sites
  anywhere.
- `src/beschreibung.rs`'s low-level name-matching (`Name::parse_flag`, `parse_mit_wert`,
  `parse_*_merge_short_forms`) already works and predates this branch; it must not regress
  during the rename.
- The derive-macro crate (`kommandozeilen_argumente_derive/`) generates code that calls
  directly into the main crate's current German-named API (`Sprache::DEUTSCH`/`ENGLISH` field
  access, `Beschreibung::neu_mit_sprache`, `Wert::neu_mit_sprache`/`neu_enum_mit_sprache`,
  `Flag::neu_mit_sprache`, `EinzelArgument::{flag,wert,frühes_beenden_mit_wert}`,
  `Argumente::kombiniere`/`combine!`, `Normalisiert::neu(..).eq_mit_case(..)`,
  `unicode::Vergleich { string, case }` literals) — all call sites must be updated to the new
  English-primary names as part of the rename, not left pointing at the (now-mirror) German API.
- Both `Cargo.toml`s are at `version = "0.3.0"`, which is unreleased (the last published release
  is `0.2.1`), so this breaking rename can ship as part of `0.3.0` without a further version bump.
- `tests/derive.rs` and `tests/hilfe.rs` exercise the crate through the derive macro and use a
  mix of German (`Flag::neu_mit_sprache`, `Beschreibung::neu_mit_sprache`,
  `EinzelArgument::flag`, `Argumente::einzel_argument`,
  `mit_hilfe_frühes_beenden_mit_sprache`) and possibly-stale API (`tests/hilfe.rs` calls
  `.parse_rekursiv(...)`, a method that does not exist anywhere in the current codebase — this
  is pre-existing dead/broken test code, unrelated to this refactor, and is fixed opportunistically
  while updating the test's other call sites in Phase 1).

## Desired End State

- The crate's public API is English-primary: e.g. `Arguments`, `Description`, `Value`,
  `EarlyExit`, `Result`, `Error`, `ParseError`, `AnnotatedParseError`, `IntermediateResult`,
  `Normalized`, `Compare`, `Language`, `SingleArgument`, `Help`, `Alternatives`, `Combine` hold
  all real logic and documentation. Every one of these has a fully separate, idiomatically
  German-named mirror struct/enum (`Argumente`, `Beschreibung`, `Wert`, `FrühesBeenden`,
  `Ergebnis`, `Fehler`, `ParseFehler`, `KommentierterParseFehler`, `ZwischenErgebnis`,
  `Normalisiert`, `Vergleich`, `Sprache`, `EinzelArgument`) connected via bidirectional `From`
  (recursive through nested mirrored types), with the same field/variant meaning translated to
  German names.
- Traits (`Combine`, `CreateHelpText`) and macros (`combine!`) exist only under their English
  name; there is no `Kombiniere` trait or `kombiniere!` macro anymore (Rust has no stable
  trait/macro aliasing, so no mirror is attempted for these two categories — this matches how
  the codebase already treats non-duplicable items).
- `Flag<'t, T>` and `Name<'t>` remain single (non-mirrored) types (the words are identical in
  both languages) with all-English field names (`description`, `invert_prefix`,
  `invert_infix`, `convert`, `display` / `long_prefix`, `long`, `short_prefix`, `short`).
- Public module paths are English: `src/arguments.rs` (was `argumente.rs`),
  `src/arguments/{flag,value,early_exit,single_argument,combine,help}.rs`,
  `src/description.rs` (was `beschreibung.rs`), `src/outcome.rs` (was `ergebnis.rs`, see
  "Module naming" decision below), `src/language.rs` (was `sprache.rs`). `src/unicode.rs` and
  `src/dyn_to_owned.rs` keep their names (already English); `src/argumente/parser.rs` (the
  unused `Parser` newtype) is deleted.
- Both crates are at version `0.4.0`; `kommandozeilen_argumente_derive`'s generated code
  references only the new English-primary names/fields.
- `tests/derive.rs` and `tests/hilfe.rs` compile and pass against the renamed API.
- The 6-stage parsing pipeline (per `doc/parsing.md`) is fully implemented on top of the
  renamed types: `Arguments::parse` (English-primary entry point) works end-to-end for flags,
  early-exit flags, value arguments, combined (tuple/derive) arguments, and alternatives.

## What We're NOT Doing

- Not touching the derive crate's **attribute-parsing keys** (e.g. the string literals a user
  writes inside `#[kommandozeilen_argumente(...)]`) — only the Rust code the derive macro
  *generates*, which calls into the main crate's API.
- Not implementing `Arguments::convert_error`/`Argumente::konvertiere_fehler` beyond what's
  needed to compile — it stays a documented stub if `doc/parsing.md` doesn't require it for the
  6 stages (verified during Phase 7).
- Not adding new user-facing features beyond what's needed to land the rename and the staged
  parser; no new CLI syntax, no new configuration options.
- Not attempting to preserve source compatibility with `0.3.0` — this is an intentional breaking
  release; no deprecated re-exports of old names are kept.
- Not mirroring `Flag`/`Name` into separate German types (see Desired End State) — nor
  `Combine`/`CreateHelpText` traits or the `combine!` macro.

## Global Design Decisions

These are binding decisions made while writing this plan, so that no open questions remain:

1. **Mirroring rule**: every public struct/enum that has (or, for newly-introduced pipeline
   types, is given) a genuine German/English name distinction becomes two separate types: the
   English one is primary (owns all methods, trait impls, and doc comments), the German one is
   a mirror with translated field/variant names and `impl From<English> for German` +
   `impl From<German> for English` (recursing into nested mirrored types field-by-field). The
   German mirror gets its own constructors (`neu`, `neu_mit_sprache`, ...) implemented as
   one-line conversions through the English type, e.g.
   `Beschreibung::neu(...) -> Self { Description::new(...).into() }`.
2. **No-mirror rule**: traits and `macro_rules!` macros are renamed outright to their English
   name with no German counterpart (`Kombiniere` trait → `Combine`, `kombiniere!` macro is
   deleted, only `combine!` remains, `ErzeugeHilfeText` → `CreateHelpText`). Where a mirrored
   enum's variant payload is a trait object of one of these non-mirrored traits (e.g.
   `Argumente::Kombiniere(Box<dyn Combine<...>>)`), **both** the English and German enum
   variants reference the same English trait object type — there is no `Box<dyn Kombiniere<...>>`.
3. **Single-identity types**: `Flag<'t, T>` and `Name<'t>` have no German/English duality (the
   words are spellically identical) and are **not** mirrored; their fields are renamed directly
   to English. Since `Name` has no mirror, its fields use the English `Compare`/`Normalized`
   types directly (not `Vergleich`/`Normalisiert`) so that the hot-path name-matching code in
   `description.rs` (formerly `beschreibung.rs`) never pays a mirror-conversion cost per parse
   call — conversion between `Compare`↔`Vergleich` and `Normalized`↔`Normalisiert` only happens
   at the public construction boundary (`Beschreibung::neu`/`Description::new`), not inside the
   matching algorithm.
4. **`dyn_to_owned.rs` traits**: `Bool` and `Parse` keep their names (already English). `Anzeige`
   is renamed to `Show` (not `Display`, to avoid colliding with `std::fmt::Display`).
5. **`src/parse.rs`'s `Parse` trait**: its German-named associated type `Fehler` is renamed to
   `Error`, and its method `kommandozeilen_argumente` is renamed to `arguments` (plain renames —
   `Parse` is a trait, no mirror, per decision 2 above). `src/argumente/hilfe.rs`'s `Standard`/
   `Default` unit-struct pair (implementing `ErzeugeHilfeText`/`CreateHelpText`) already has an
   independent, meaningful English name (`Default`) distinct from its German name (`Standard`) —
   it is kept as a plain existing pair, not further mirrored; only the trait it implements is
   renamed (`ErzeugeHilfeText` → `CreateHelpText`) per decision 2. The two private (non-`pub`)
   helper traits `KonvertiereFehler`/`AnzeigeFehler` in `src/argumente.rs` are renamed outright
   (plain rename, no mirror, since they are traits) to `ConvertError`/`DisplayError`.
6. **Module naming**: `argumente` → `arguments`, `beschreibung` → `description`, `sprache` →
   `language`. `ergebnis` → `outcome` (not `result`), specifically to avoid the module path
   stuttering/confusing with `std::result::Result` once the primary type inside is literally
   named `Result` (`kommandozeilen_argumente::outcome::Result`, re-exported at the crate root as
   `kommandozeilen_argumente::Result`). `unicode` and `dyn_to_owned` are unchanged.
   `src/argumente/parser.rs` (dead `Parser` scaffolding, no call sites) is deleted, not renamed.
7. **Derive crate**: its generated code is updated to call the new English-primary API
   directly (not the German mirror) wherever it constructs types, since generated code has no
   reason to prefer the German spelling. Both crates bump to `0.4.0`.

## Architecture and Code Reuse

The existing, already-working low-level matching logic in `beschreibung.rs`/`description.rs`
(`Name::parse_flag`, `parse_mit_wert`, `parse_*_merge_short_forms`,
`AdjustedMergedShortNames`/`ArgumentInput`) is reused as-is by the staged pipeline — Phase 1
only renames its containing types/fields, Phases 3-6 call into it unchanged.

High-level file tree (post-rename layout):

- `src/`
  - `lib.rs` — re-export list updated to new module/type names
  - `arguments.rs` (was `argumente.rs`) — `Arguments<'t,T,Error>` (primary: `Single`/`Combined`/
    `Alternatives`), `Argumente<'t,T,Fehler>` (mirror: `EinzelArgument`/`Kombiniere`/
    `Alternativen`), bidirectional `From`, all `parse_*`/`with_*`/`convert_error` method pairs,
    `ParsedEarlyExit`/`ParsedShortFlag`/`ParsedValueName`/`ParsedValue`/
    `ParseMergedShortFormsResult` (English-only, newly documented, no German mirror needed since
    these are internal pipeline-plumbing structs not part of the constructor-facing API — see
    Phase 2 task list for the explicit decision)
  - `arguments/`
    - `single_argument.rs` (was `einzelargument.rs`) — `SingleArgument`/`EinzelArgument` mirror
      pair
    - `flag.rs` — `Flag<'t,T>` (single type, English fields)
    - `value.rs` (was `wert.rs`) — `Value`/`Wert` mirror pair, `EnumArgument` trait (unchanged,
      already dual-named)
    - `early_exit.rs` (was `frühes_beenden.rs`) — `EarlyExit`/`FrühesBeenden` mirror pair
    - `combine.rs` (was `kombiniere.rs`) — `Combine` trait (renamed from `Kombiniere`, no
      mirror), `combine!` macro (only; `kombiniere!` deleted), `impl_combine_tuple!` macro
      (renamed from `impl_kombiniere_tuple!`) — **old commented-out single-pass `parse`/
      `parse_merged_short_forms` sketch bodies are deleted, not carried forward**
    - `help.rs` (was `hilfe.rs`) — `Help`/`Hilfe`, `Alternatives`/`Alternativen`,
      `CreateHelpText` (renamed from `ErzeugeHilfeText`, no mirror)
    - `parser.rs` — **deleted** (dead scaffolding, no call sites)
  - `description.rs` (was `beschreibung.rs`) — `Name<'t>` (single type, English fields:
    `long_prefix`, `long`, `short_prefix`, `short`), `Description`/`Beschreibung` mirror pair,
    `AdjustedMergedShortNames`/`ArgumentInput`/`ArgumentInputRef`/`MergedShortNameSuffix`
    (English-only, no German duality today, stay as-is), `LangNamen`/`KurzNamen` traits renamed
    to `LongNames`/`ShortNames` (no mirror, traits)
  - `outcome.rs` (was `ergebnis.rs`) — `Result`/`Ergebnis`, `IntermediateResult`/
    `ZwischenErgebnis`, `Error`/`Fehler`, `AnnotatedParseError`/`KommentierterParseFehler`,
    `ParseError`/`ParseFehler` mirror pairs; `SingeArgResult` stays English-only (already is)
  - `language.rs` (was `sprache.rs`) — `Language`/`Sprache` mirror pair, each holding its own
    locale constants (`Language::GERMAN`/`Language::ENGLISH`, `Sprache::DEUTSCH`/
    `Sprache::ENGLISH` kept for back-compat naming on the German side only)
  - `unicode.rs` — `Normalized`/`Normalisiert`, `Compare`/`Vergleich` mirror pairs, `Case` enum
    unchanged
  - `dyn_to_owned.rs` — `Bool`, `Parse` unchanged; `Anzeige` → `Show`
  - `parse.rs` — `ParseArgument`/`Parse` traits, call sites updated to new `Arguments`-primary
    API
- `kommandozeilen_argumente_derive/src/`
  - `parse.rs`, `enum_argument.rs`, `utility.rs` — generated-code call sites updated to new
    English-primary names (see Phase 1 task list for exact line references)
- `tests/derive.rs`, `tests/hilfe.rs` — call sites updated; `tests/hilfe.rs`'s stale
  `.parse_rekursiv(...)` call replaced with the equivalent current API
- `Cargo.toml`, `kommandozeilen_argumente_derive/Cargo.toml` — `version = "0.4.0"`

## Performance Considerations

Mirror-type conversions (`From` impls) only run at construction time (when a user builds a
`Beschreibung`/`Wert`/`Flag`/... via the German-named constructors) or at the rare boundary
where a caller explicitly mixes German and English types. The hot per-argument matching path
(`Name::parse_flag`, `parse_mit_wert`, merged-short-form variants) operates purely on the
English `Compare`/`Normalized` types with no conversion overhead, per the Global Design
Decisions above.

## Migration Notes

This is a breaking `0.3.0` → `0.4.0` release. There is no compatibility shim: code using the old
German-primary names must be updated to either keep using the (now-mirrored, differently-shaped)
German types or switch to the new English-primary types. Downstream consumers relying on
`Arguments`/`Value`/`Description`/... being *type aliases* of the German types (e.g. matching on
field names) will need to update field names to the new English ones.

---

## Phase 1: Global English-primary rename

This phase is the bulk of the work: invert every dual-named type, translate fields, rename
modules/files, update the derive crate's generated code, update existing tests, and bump both
crate versions. No new parsing logic is implemented in this phase — the crate must compile and
existing tests must pass afterward (parsing behavior stays exactly as `todo!()`-stubbed as it is
today, just under new names).

**Note on current baseline** (verified against the pre-Phase-1 codebase before writing this
plan): `cargo build --workspace --all-features` succeeds today (with ~147 pre-existing warnings:
missing docs, a few unused private functions). `cargo test --workspace --all-features` does
**not** currently compile (the `parse_rekursiv` and unit-struct `Combine` issues addressed by
sub-phase 1.10 below), and `cargo clippy --workspace --all-features -- -D warnings` does **not**
currently pass (one pre-existing `unnecessary_semicolon` error in
`kommandozeilen_argumente_derive/src/parse.rs:1005`, plus the ~147 warnings would all become hard
errors under `-D warnings`). The `-D warnings`-clean bar and the full test pass are deferred to
Phase 8's final cleanup sweep; each Phase 1 sub-phase below is scoped to be individually
commit-sized and independently buildable in sequence.

**Why the sub-phase order is what it is**: each sub-phase only touches types whose dependencies
were already renamed by an earlier sub-phase (e.g. `description.rs` needs `Compare`/`Normalized`
from 1.1 first). The one structural wrinkle is `Combine`/`Argumente` mutual reference: sub-phase
1.10 renames the `Kombiniere` trait to `Combine` and the macros, but its tuple impls keep
referencing the not-yet-renamed `Argumente` enum type (still valid Rust, since `Argumente` itself
isn't touched until 1.11) — 1.11 then renames `Argumente`→`Arguments`/`Argumente`-mirror and, in
the same commit, updates `combine.rs`'s `Argumente` references to `Arguments`.

**Derive-crate compilation note**: the derive crate keeps compiling unchanged through sub-phases
1.1-1.5 because every renamed type in those sub-phases keeps a German-named mirror with identical
method names (`Beschreibung::neu_mit_sprache`, `Sprache::DEUTSCH`, ...). It **breaks starting at
sub-phase 1.6** (`Flag` has no German mirror by design — Global Design Decision 3 — so
`Flag::neu_mit_sprache` stops existing) and **stays broken through sub-phase 1.13**; this is
expected. Use `cargo build -p kommandozeilen_argumente` (main crate only) as the Automated
Verification command for sub-phases 1.1 through 1.13; sub-phase 1.14 is what makes
`cargo build --workspace` succeed again.

### Phase 1.1: Foundational primitives — `unicode.rs` and `dyn_to_owned.rs`

No dependencies on other sub-phases; these are leaf modules everything else builds on.

**Tasks**:

- [x] Rename `src/unicode.rs` types: make `Normalized<'t>`/`Compare<'t>` the primary structs
  (all methods/impls: `new`, `as_str`, `eq_with_case`, `cow_ref`, `cow`, `into_owned`,
  `strip_as_prefix`/`strip_as_prefix_n` renamed from `strip_als_präfix`/`strip_als_präfix_n`).
  Add `Normalisiert<'t>`/`Vergleich<'t>` as mirror structs (same shape, German method names
  `neu`, `eq_mit_case`, ...) with `impl From<Normalized<'_>> for Normalisiert<'_>` and back
  (same for `Compare`/`Vergleich`, recursing through `Normalized`↔`Normalisiert`). `Case` enum
  is unchanged (already English-only).
- [x] Rename `src/dyn_to_owned.rs`'s `Anzeige<'t,T>` trait to `Show<'t,T>` (plain rename, no
  mirror); update its one macro invocation line and all call sites crate-wide.

**Automated Verification**:

- [x] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.2: Language — `sprache.rs` → `language.rs`

Depends on: **Phase 1.1** (sequenced after, no actual type dependency).

**Tasks**:

- [ ] Rename `src/sprache.rs` → `src/language.rs`. Make `Language` the primary struct with
  English field names (`long_prefix`, `short_prefix`, `invert_prefix`, `invert_infix`,
  `value_infix`, `meta_var`, `options`, `default`, `allowed_values`, `missing_flag`,
  `missing_value`, `parse_error`, `invalid_string`, `unused_argument`, `help_description`,
  `help_long`, `help_short`, `version_description`, `version_long`, `version_short`,
  `syntax_prefix`, `syntax_padding`, `alternative_prefix`, `alternative_separator`), with
  content constants `Language::GERMAN` and `Language::ENGLISH`. Add `Sprache` as the German
  mirror struct (current field names, current `Sprache::DEUTSCH`/`Sprache::ENGLISH` constants)
  with bidirectional `From`.

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.3: Description — `beschreibung.rs` → `description.rs`

Depends on: **Phase 1.1** (uses `Compare`/`Normalized`).

**Tasks**:

- [ ] Rename `src/beschreibung.rs` → `src/description.rs`. Rename `Name<'t>`'s fields to
  `long_prefix`, `long`, `short_prefix`, `short` (type `Compare<'t>`/`NonEmpty<Compare<'t>>`/
  `Vec<Compare<'t>>`), update all internal matching methods (`parse_flag`, `parse_flag_aux`,
  `parse_merged_short_name`, `parse_flag_merge_short_forms(_aux)`, `parse_frühes_beenden` →
  `parse_early_exit`(`_merge_short_forms`), `parse_mit_wert` → `parse_with_value`
  (`_merge_short_forms`), `möglichkeiten_als_regex` → `alternatives_as_regex`) to the new field
  names, operating on `Normalized`/`Compare` throughout (no `Normalisiert`/`Vergleich` usage
  inside this file, per the hot-path decision). Make `Description<'t,T>` the primary struct
  (fields `name`, `help`, `default`) with all constructors (`new`, `new_with_language`, `convert`,
  `as_ref`); add `Beschreibung<'t,T>` mirror (fields `name`, `hilfe`, `standard`, methods `neu`,
  `neu_mit_sprache`, `konvertiere`) with bidirectional `From`. Rename `LangNamen`/`KurzNamen`
  traits to `LongNames`/`ShortNames` (plain rename, methods `long_names`/`short_names`); update
  their blanket impls for `String`/`&str`/`Compare`/`NonEmpty<...>`/`Vec<...>`.

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.4: Outcome — `ergebnis.rs` → `outcome.rs`

Depends on: **Phase 1.3** (uses `Name`/`Normalized`).

**Tasks**:

- [ ] Rename `src/ergebnis.rs` → `src/outcome.rs`. Make `Result<'t,T,E>` (`Value`/`EarlyExit`/
  `Error` variants), `IntermediateResult<'t,T,E,A>` (`Value`/`EarlyExit`/`Error`/`Incomplete`),
  `Error<'t,E>` (`MissingFlag{name,invert_prefix,invert_infix}`/
  `MissingValue{name,value_infix,meta_var}`/`ParseError(AnnotatedParseError<'t,E>)`),
  `AnnotatedParseError<'t,E>` (fields `name`,`value_infix`,`meta_var`,`error`), `ParseError<E>`
  (`InvalidString(OsString)`/`ParseError(E)`) the primary types with all methods (`convert`,
  `convert_error`, `convert_incomplete`, `create_error_message`,
  `create_error_message_with_language`). Add `Ergebnis`/`ZwischenErgebnis`/`Fehler`/
  `KommentierterParseFehler`/`ParseFehler` as German mirrors (current variant/field/method
  names) with bidirectional `From` (recursing through `Error`↔`Fehler`,
  `AnnotatedParseError`↔`KommentierterParseFehler`, `ParseError`↔`ParseFehler`,
  `Name`/`Normalized` shared as-is). `SingeArgResult<'t,T,E>` stays as the sole (English-only)
  type, only its embedded `ParseFehler<E>` references update to `ParseError<E>`.

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.5: Help text + dead-code removal — `hilfe.rs` → `help.rs`, delete `parser.rs`

Depends on: **Phase 1.4** (sequenced after, no direct type dependency).

**Tasks**:

- [ ] Delete `src/argumente/parser.rs` (dead `Parser` newtype, no call sites).
- [ ] Rename `src/argumente/hilfe.rs` → `src/arguments/help.rs`. Make `Help{syntax,help}` and
  `Alternatives{Single(Help), Alternatives(Box<NonEmpty<Alternatives>>), Empty}` the primary
  types; add `Hilfe{syntax,hilfe}`/`Alternativen{EinzelArgument(Hilfe), Alternativen(...), Leer}`
  mirrors with bidirectional `From`. Rename `ErzeugeHilfeText` trait to `CreateHelpText` (plain
  rename, method `create_help_text`), no mirror. Update `Standard`/`Default`'s
  `impl ErzeugeHilfeText for ...` blocks to `impl CreateHelpText for ...` — the `Standard`/
  `Default` structs themselves keep their current names unchanged (per Global Design Decision 5).

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.6: `Flag`

Depends on: **Phase 1.1, 1.2, 1.3, 1.5**.

**Tasks**:

- [ ] Rename `src/argumente/flag.rs` → `src/arguments/flag.rs`. Rename `Flag<'t,T>`'s fields to
  `description: Description<'t,T>`, `invert_prefix: Compare<'t>`, `invert_infix: Compare<'t>`,
  `convert: Cow<'t, dyn Bool<'t,T>>`, `display: Cow<'t, dyn Show<'t,T>>`. Rename methods `neu`/
  `neu_mit_sprache` → keep as the sole constructors renamed to `new`/`new_with_language` (no
  German duplicate — `Flag` is a single-identity type per Global Design Decision 3), same for
  `erzeuge_hilfe_text` → `create_help_text`, `als_string_flag` → `as_string_flag`,
  `parse_merged_short_forms` (name unchanged, still `todo!()`, update field destructuring).

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds
- [ ] `cargo build --workspace --all-features` is now **expected to fail** on the derive crate
  (`Flag::neu_mit_sprache` no longer exists) — confirm the failure is exactly this and nothing
  else, i.e. no unrelated main-crate breakage

---

### Phase 1.7: `Value`

Depends on: **Phase 1.1, 1.2, 1.3, 1.4, 1.5**.

**Tasks**:

- [ ] Rename `src/argumente/wert.rs` → `src/arguments/value.rs`. Make `Value<'t,T,Error>` the
  primary struct (fields `description`, `value_infix`, `meta_var`, `possible_values`, `parse`,
  `display`, `display_error`) with constructors `new`/`new_with_language`/`new_enum`/
  `new_enum_with_language`, `create_help_text`, `as_string_value`, `parse_merged_short_forms`.
  Add `Wert<'t,T,Fehler>` mirror (current fields/methods) with bidirectional `From`. Update
  `EnumArgument` trait's doc references only (already dual-named `varianten`/`variants`, no
  structural change needed).

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.8: `EarlyExit`

Depends on: **Phase 1.3, 1.5**.

**Tasks**:

- [ ] Rename `src/argumente/frühes_beenden.rs` → `src/arguments/early_exit.rs`. Make
  `EarlyExit<'t>` the primary struct (fields `description: Description<'t,Void>`,
  `message: Cow<'t,str>`) with `new`, `create_help_text`, `parse_merged_short_forms`. Add
  `FrühesBeenden<'t>` mirror (fields `beschreibung`, `nachricht`) with bidirectional `From`.

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.9: `SingleArgument`

Depends on: **Phase 1.6, 1.7, 1.8**.

**Tasks**:

- [ ] Rename `src/argumente/einzelargument.rs` → `src/arguments/single_argument.rs`. Introduce
  `SingleArgument<'t,T,Error>` as the new primary enum: `Flag(Flag<'t,T>)`,
  `EarlyExit{early_exit: EarlyExit<'t>, value: T, display: Cow<'t, dyn Show<'t,T>>}`,
  `Value(Value<'t,T,Error>)`, with constructors `flag`/`early_exit_with_value`/`early_exit`/
  `value`, `create_help_text`, `as_string_value`, `parse_merged_short_forms` (dispatching to
  `Flag`/`EarlyExit`/`Value`'s own methods, same structure as today). Turn `EinzelArgument` into
  the German mirror (`Flag`/`FrühesBeenden{frühes_beenden,wert,anzeige}`/`Wert`, current method
  names) with bidirectional `From` (recursing through `EarlyExit`↔`FrühesBeenden`,
  `Value`↔`Wert`; `Flag` is shared unchanged since it has no mirror).

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.10: `Combine` trait and macros

Depends on: **Phase 1.4, 1.5** (still references the not-yet-renamed `Argumente` enum type from
`src/argumente.rs`, which is untouched until Phase 1.11 — see the sub-phase-order rationale
above).

**Tasks**:

- [ ] Rename `src/argumente/kombiniere.rs` → `src/arguments/combine.rs`. Rename `Kombiniere`
  trait to `Combine` (methods `create_help_text`/`debug_fmt`); delete the `kombiniere!` macro
  entirely, keep only `combine!` (now the sole macro, not delegating to a deleted macro — inline
  its former body). Rename `impl_kombiniere_tuple!` to `impl_combine_tuple!`; **delete** the
  commented-out old single-pass `parse`/`parse_merged_short_forms` sketch bodies (both tuple
  impls) instead of carrying them forward — this is a clean-implementation decision, the new
  staged logic is written fresh in Phases 3-7. Update the live `create_help_text`/`debug_fmt`
  bodies' field/type names. These impls still reference the not-yet-renamed `Argumente<...>` and
  `ZwischenErgebnis<...>` types verbatim at this point (both still exist, unchanged, until Phase
  1.11 renames `Argumente`).
- [ ] Fix the pre-existing (rename-unrelated) unit-struct `Combine` bug: `#[derive(Parse)]` on a
  zero-field struct generates `Argumente::kombiniere((closure,))` — a bare 1-tuple that no
  `impl_kombiniere_tuple!`-generated impl satisfies, since all generated impls start at 2-element
  tuples `(F, Argumente<A>)`. Add a 0-argument base case to `impl_combine_tuple!`/an explicit
  `impl<'t, F: 't + Fn() -> T, T, Error: Debug> Combine<'t, T, Error> for (F,)` (and the matching
  `(F, IntermediateResult<...>)`-free equivalent, i.e. just `(F,)`) so `combine!(|| Empty)`
  compiles; this is required for `tests/derive.rs`'s `struct Empty` to compile again, verified as
  part of this phase (not deferred to Phase 7) since it's a structural/arity gap, not a parsing
  semantics gap — its `create_help_text`/`debug_fmt` bodies mirror the 1-arity case trivially.

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.11: `Arguments`/`Argumente` core enum

Depends on: **Phase 1.9, 1.10**.

**Tasks**:

- [ ] Rename `src/argumente.rs` → `src/arguments.rs`. Introduce `Arguments<'t,T,Error>` as the
  new primary enum: `Single(SingleArgument<'t,T,Error>)`,
  `Combined(Box<dyn Combine<'t,T,Error>>)`, `Alternatives(Box<NonEmpty<Self>>)`. Turn `Argumente`
  into the German mirror (`EinzelArgument(EinzelArgument<'t,T,Fehler>)`,
  `Kombiniere(Box<dyn Combine<'t,T,Fehler>>)` — reusing the single English `Combine` trait per
  Global Design Decision 2 — `Alternativen(Box<NonEmpty<Self>>)`) with bidirectional `From`
  (recursing through `SingleArgument`↔`EinzelArgument`, mapping `NonEmpty<Self>` element-wise).
  Move all constructor methods (`single_argument`/`combine`/`alternatives`/`alternatives_boxed`)
  onto `Arguments`; give `Argumente` the mirrored equivalents (`einzel_argument`/`kombiniere`/
  `alternativen`/`alternativen_boxed`) as one-line `Arguments`-conversion wrappers. Rename
  `ParsedEarlyExit`/`ParsedShortFlag`/`ParsedValueName`/`ParsedValue`/
  `ParseMergedShortFormsResult` fields to English if any are German (verify against current
  `/// TODO`-only definitions; keep them English-only, no mirror, per Architecture section).
  Rename the huge `parse_*`/`with_*`/`convert_error` method-pair family
  (`parse_aus_env`/`parse_from_env`, `parse_mit_frühen_beenden`/`parse_with_early_exit`,
  `parse_vollständig`/`parse_complete`, `parse_vollständig_mit_sprache`/
  `parse_complete_with_language`, `parse_mit_fehlermeldung`/`parse_with_error_message`,
  `*_aus_env` variants, `konvertiere_fehler`/`convert_error`, `fehler_from`/`error_from`,
  `fehler_from_void`/`error_from_void`, `mit_version_frühes_beenden`/`with_version_early_exit`,
  `mit_hilfe_frühes_beenden`/`with_help_early_exit`,
  `mit_hilfe_und_version_frühes_beenden`/`with_help_and_version_early_exit`) so every method
  lives primarily on `Arguments` (full body) with a one-line delegating equivalent on `Argumente`
  that converts to `Arguments`, calls through, and converts the result back. Rename the private
  helpers `max_syntax_breite`/`schreibe_argument_oder_alternativen` to
  `max_syntax_width`/`write_argument_or_alternatives`. Rename the private (non-`pub`) helper
  traits `KonvertiereFehler`/`AnzeigeFehler` to `ConvertError`/`DisplayError` (plain rename, no
  mirror, per Global Design Decision 5) and update their `clone_trait_object!` invocations.
- [ ] Update `src/arguments/combine.rs`'s tuple impls (from Phase 1.10) to reference the now-
  renamed `Arguments`/`Result`-style types instead of `Argumente`/`ZwischenErgebnis` where the
  impl is for the primary (English) side, keeping the German-mirror-typed impl block referencing
  `Argumente`/`ZwischenErgebnis` unchanged.

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.12: `src/parse.rs`

Depends on: **Phase 1.11**.

**Tasks**:

- [ ] Update `src/parse.rs`: change every reference to the old German-primary types
  (`Argumente`, `Wert`, `Beschreibung`, `EinzelArgument`, `Ergebnis`, ...) to the new English
  types (`Arguments`, `Value`, `Description`, `SingleArgument`, `Result`, ...); rename the `Parse`
  trait's associated type `Fehler` to `Error` and its method `kommandozeilen_argumente` to
  `arguments` (plain renames per Global Design Decision 5), updating every one of its own
  `parse_*`/`parse_*_aus_env`/`parse_vollständig*` convenience-method bodies that reference
  `Self::Fehler`/`Self::kommandozeilen_argumente()`; rename `OptionHelper`/
  `erstelle_ergebnis_anpassen` internals to English if they contain German identifiers.

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds

---

### Phase 1.13: `src/lib.rs` re-exports

Depends on: **Phase 1.1 through 1.12** (touches the full re-export surface).

**Tasks**:

- [ ] Update `src/lib.rs`'s module declarations and re-export list for all renamed
  modules/types (`arguments`, `arguments::flag::Flag`, `arguments::early_exit::{EarlyExit,
  FrühesBeenden}`, `arguments::value::{EnumArgument, Value, Wert}`, `arguments::{Arguments,
  Argumente}`, `description::{ArgumentInput, Description, Beschreibung}`,
  `outcome::{Result, Error, Fehler, ParseError, ParseFehler, Ergebnis}`,
  `parse::{Parse, ParseArgument}`, `language::{Language, Sprache}`,
  `unicode::{Case, Compare, Normalized, Normalisiert, Vergleich}`).

**Automated Verification**:

- [ ] `cargo build -p kommandozeilen_argumente --all-features` succeeds
- [ ] `cargo doc -p kommandozeilen_argumente --all-features --no-deps` succeeds with no broken
  intra-doc links

---

### Phase 1.14: Derive crate

Depends on: **Phase 1.13** (the main crate's renamed API must be complete and stable first).

This is the sub-phase that restores `cargo build --workspace` to green.

**Tasks**:

- [ ] Update `kommandozeilen_argumente_derive/src/parse.rs` generated-code call sites (per the
  completed API-surface audit) to construct `Description`/`Value`/`Flag`/`SingleArgument`/
  `Arguments`/`Compare`/`Normalized`/`Language` instead of the German types, including the
  `Language::GERMAN`/`Language::ENGLISH`-field-access call sites (was `Sprache::DEUTSCH`/
  `Sprache::ENGLISH`), the `combine!`-macro-shaped code generation (was `kombiniere!`), and the
  `Compare { string: Normalized::new(...), case }` literal construction (was
  `unicode::Vergleich { string: Normalisiert::neu(...), case }`). Update the hardcoded path
  `::#crate_name::argumente::hilfe::Standard` (line ~208, used as the `variante` argument to
  `mit_hilfe_frühes_beenden`) to `::#crate_name::arguments::help::Standard` (module path only —
  the `Standard` struct name itself is unchanged per Global Design Decision 5). Update the
  generated `impl ::#crate_name::Parse for #name { type Fehler = String; fn
  kommandozeilen_argumente<'t>() -> ... }` block (lines ~1430-1436) to
  `impl ::#crate_name::Parse for #name { type Error = String; fn arguments<'t>() -> ... }`, and
  the `FeldArgument::Parse` code-generation arm's `quote!(::#crate_name::Parse::kommandozeilen_argumente())`
  (line ~115) to `quote!(::#crate_name::Parse::arguments())`.
- [ ] Update `kommandozeilen_argumente_derive/src/enum_argument.rs` (lines referencing
  `Normalisiert::neu(...).eq_mit_case(...)`) to `Normalized::new(...).eq_with_case(...)`.
- [ ] Update `kommandozeilen_argumente_derive/src/utility.rs` call sites analogous to the above.

**Automated Verification**:

- [ ] `cargo build --workspace --all-features` succeeds

---

### Phase 1.15: Update existing tests

Depends on: **Phase 1.14**.

**Tasks**:

- [ ] Update `tests/derive.rs` and `tests/hilfe.rs` call sites to the new English-primary names
  (or their still-valid, newly-field-renamed German mirror equivalents, whichever the specific
  test line is exercising). Replace `tests/hilfe.rs`'s stale `.parse_rekursiv(...)` call (a
  method that doesn't exist anywhere in the current codebase) with the current equivalent
  top-level entry point for generating a full parse from arguments, or delete the assertion if
  it was testing since-removed behavior — confirm before deleting by checking git blame/history
  for what `parse_rekursiv` used to do.

**Automated Verification**:

- [ ] `cargo build --workspace --all-features` succeeds
- [ ] `cargo test --workspace --all-features` **compiles** (no `E0599`/`E0277`-style errors); the
  existing `tests/derive.rs` and `tests/hilfe.rs` test bodies pass to the extent they don't
  depend on unimplemented parsing behavior (i.e. `todo!()` panics from stages not yet
  implemented in Phases 2-7 are an expected, acceptable failure at this point — recorded as a
  known list of still-panicking tests, not silently ignored)
- [ ] `cargo clippy --workspace --all-features` passes with no **errors** (warnings are
  permitted at this stage; the `-D warnings`-clean bar is Phase 8's responsibility)
- [ ] `grep -rln 'kommandozeilen_argumente\b' kommandozeilen_argumente_derive/src/` — manually confirm (see Phase 8 for the exhaustive sweep) that none of the matches still call the pre-rename `Parse::kommandozeilen_argumente()`/`Sprache::`/`Vergleich {`/`Normalisiert::` spellings; this is a spot-check, not a pass/fail gate — Phase 8 owns the authoritative zero-leftover check
- [ ] `grep -rn --exclude=src/arguments.rs 'Argumente::einzel_argument\|EinzelArgument::flag\|Beschreibung::neu\b' src/ tests/ kommandozeilen_argumente_derive/src/` returns no results (these are legitimate only inside `src/arguments.rs`'s `Argumente` mirror-wrapper methods, expected to be zero everywhere else)

---

## Phase 2: Shared staged-parsing infrastructure

Depends on: **Phase 1**.

Builds the shared plumbing every stage (3-7) needs: fleshing out the per-stage result structs
and wiring `Arguments::parse_merged_short_forms`'s dispatch for the `Combined` variant, without
yet implementing any single argument's actual matching logic.

**Tasks**:

- [ ] Document and finalize the fields of `ParsedEarlyExit`, `ParsedShortFlag`, `ParsedValueName`,
  `ParsedValue`, `ParseMergedShortFormsResult` in `src/arguments.rs` (replace `/// TODO` with
  real doc comments), matching `doc/parsing.md`'s stage-1 output description (definition
  reference, collected early-exits, collected flags, value-name → raw-value map, remaining
  not-yet-consumed input).
- [ ] Implement `Arguments::parse_merged_short_forms`'s `Combined` arm (previously the
  `Kombiniere`/`todo!()` arm) by delegating to `Combine::parse_merged_short_forms` once that
  trait method exists (added in this phase as a new trait method on `Combine`, default-bodied
  `todo!()` so the tuple impls compile without requiring per-arity logic yet — actual per-tuple
  logic is written in Phase 7 once stages 1-6 all exist to compose).
- [ ] Add the `Combine::parse_merged_short_forms` trait method signature (mirroring
  `EinzelArgument`/`SingleArgument::parse_merged_short_forms`'s signature) to
  `src/arguments/combine.rs`, with a default `todo!()` body so existing tuple impls keep
  compiling.

**Automated Verification**:

- [ ] `cargo build --workspace --all-features` succeeds
- [ ] `cargo doc --workspace --all-features --no-deps` succeeds with no broken intra-doc links
  for the newly-documented structs

---

## Phase 3: Stage 1 — merged short-name flag/value parsing

Depends on: **Phase 2**.

Implements `Flag::parse_merged_short_forms`, `EarlyExit::parse_merged_short_forms`,
`Value::parse_merged_short_forms`, and `SingleArgument::parse_merged_short_forms`'s dispatch,
using the already-working `Name::parse_flag_merge_short_forms`/`parse_with_value_merge_short_forms`
matching in `description.rs`.

**Tasks**:

- [ ] Implement `Flag::parse_merged_short_forms` (`src/arguments/flag.rs`) using
  `self.description.name.parse_flag_merge_short_forms`/`parse_flag`, producing a
  `ParseMergedShortFormsResult` with a single matched flag or none, per `doc/parsing.md`'s rules
  (single-grapheme short names only, must share short prefix).
- [ ] Implement `EarlyExit::parse_merged_short_forms` (`src/arguments/early_exit.rs`) similarly,
  using `parse_early_exit_merge_short_forms`.
- [ ] Implement `Value::parse_merged_short_forms` (`src/arguments/value.rs`) using
  `parse_with_value_merge_short_forms`, handling the "at most one value argument per block, must
  be last, optionally followed by value-infix + value substring" rule.
- [ ] Verify `SingleArgument::parse_merged_short_forms`'s existing dispatcher (renamed in Phase
  1) correctly routes to the three implementations above without further changes.

**Automated Verification**:

- [ ] New unit tests in `src/arguments/flag.rs`, `src/arguments/early_exit.rs`,
  `src/arguments/value.rs` (`#[cfg(test)] mod tests`) covering: single merged short flag match,
  multiple merged short flags in one block, merged short value at end of block (with and without
  value-infix), no match, short-name-merging disabled for an argument — all pass
- [ ] `cargo test --workspace --all-features` passes

---

## Phase 4: Stage 2 — non-merged short-name argument parsing

Depends on: **Phase 3**.

**Tasks**:

- [ ] Implement non-merged short-name matching for `Flag`, `EarlyExit`, `Value` (new methods,
  e.g. `parse_short_form`, using `Name::parse_flag`/`parse_with_value` directly against a
  standalone `-x`/`-x value` argument, not a merged block).
- [ ] Wire `SingleArgument::parse_short_form` dispatcher analogous to
  `parse_merged_short_forms`.

**Automated Verification**:

- [ ] New unit tests covering standalone short flag, short flag with next-arg value, short value
  with inline value (`-xvalue`), short value with next-arg value, non-match — all pass
- [ ] `cargo test --workspace --all-features` passes

---

## Phase 5: Stage 3 — long-name argument parsing

Depends on: **Phase 4**.

**Tasks**:

- [ ] Implement long-name matching for `Flag`, `EarlyExit`, `Value` (new methods, e.g.
  `parse_long_form`, using `Name::parse_flag`/`parse_with_value` against the long-name prefix,
  including the invert-prefix/invert-infix flag-negation path and the value-infix
  `--name=value` path).
- [ ] Wire `SingleArgument::parse_long_form` dispatcher.

**Automated Verification**:

- [ ] New unit tests covering long flag, inverted long flag (`--no-flag`), long flag with
  next-arg value, `--name=value`, `--name value`, non-match — all pass
- [ ] `cargo test --workspace --all-features` passes

---

## Phase 6: Stage 4 — typed value parsing via `Box<dyn Any>`

Depends on: **Phase 5**.

Converts the raw `OsString` values collected by stages 1-3 into typed values, per
`doc/parsing.md`'s design of sharing heterogeneous value types in one map via `Box<dyn Any>` +
`TypeId`.

**Tasks**:

- [ ] Add a value-parsing step that, given a `ParsedValue` (raw `OsString` + originating
  `Value<'t,T,Error>`'s `parse` closure), produces `Result<Box<dyn Any>, AnnotatedParseError>`,
  keyed by `TypeId::of::<T>()` for later downscasting.
- [ ] Extend `ParseMergedShortFormsResult`/its stage-2/3 equivalents to carry the typed-value map
  instead of (or alongside) the raw-string map, per `doc/parsing.md`.

**Automated Verification**:

- [ ] New unit tests: successful typed parse, parse error produces `AnnotatedParseError`,
  multiple distinct value types coexist in the same map and downcast correctly — all pass
- [ ] `cargo test --workspace --all-features` passes

---

## Phase 7: Stages 5 & 6 — alternative selection, result accumulation, and wiring `Arguments::parse`

Depends on: **Phase 6**.

Implements picking the winning `Alternatives` branch, accumulating typed pieces into the final
result via the user-supplied combine function (`Combine::parse_merged_short_forms`'s real body
for all tuple arities, replacing the Phase 2 `todo!()` default), and wires the top-level
`Arguments::parse` entry point so every `parse_*`/`with_*` convenience method becomes usable
end-to-end.

**Tasks**:

- [ ] Implement `Combine::parse_merged_short_forms`'s real body in `impl_combine_tuple!`
  (`src/arguments/combine.rs`) for both the `Arguments`-tuple and `IntermediateResult`-tuple
  impls: parse each sub-argument, accumulate errors/early-exits/incomplete-state, and only
  invoke the user function once all sub-results are values (fresh implementation, not the
  deleted commented-out sketch).
- [ ] Implement `Arguments::Alternatives` selection logic: try each alternative in order, keep
  the first that produces `Result::Value`, else combine errors from all failed alternatives.
- [ ] Implement `Arguments::parse` (`src/arguments.rs`), composing stages 1-6 in the order
  specified by `doc/parsing.md`.
- [ ] Verify all existing `parse_*`/`with_*`/`convert_error` method families on `Arguments`
  (and their `Argumente` mirror equivalents) now function end-to-end since they all bottom out
  in `Arguments::parse`.
- [ ] Remove any now-unnecessary `todo!()`s discovered to be reachable during this phase.

**Automated Verification**:

- [ ] `tests/derive.rs`'s 6 test functions all pass with real parsing (not just help-text
  generation)
- [ ] `tests/hilfe.rs` passes
- [ ] New integration tests covering: single flag, single value, combined multi-argument struct
  (derive macro), alternatives (first alternative fails, second succeeds), early-exit (`--help`/
  `--version`) short-circuiting result accumulation — all pass
- [ ] `cargo test --workspace --all-features` passes

**Manual Verification**:

- [ ] Build one of the crate's example binaries (or a small throwaway binary using
  `#[derive(Parse)]`) and confirm `--help`, a valid flag/value combination, and an invalid
  argument all produce the expected CLI behavior and exit codes

---

## Phase 8: Final cleanup pass

Depends on: **Phase 7**.

A dedicated sweep to catch anything left over from the rename or the staged-parsing
implementation before considering the refactor complete.

This phase also owns bringing the crate to a fully `-D warnings`-clean state, deferred here from
Phase 1 (see Phase 1's baseline note): a pre-existing `unnecessary_semicolon` clippy error and
~147 pre-existing warnings (missing docs, unused private functions/fields) were present before
this refactor started, plus whatever new warnings the rename/staged-parsing work introduced
(e.g. an unused mirror-type method, a newly-dead `#[allow(dead_code)]`-worthy helper).

**Tasks**:

- [ ] `grep -rn` for lingering references to deleted items across `src/`, `tests/`,
  `kommandozeilen_argumente_derive/src/`: `kombiniere!`, `Kombiniere`, `ErzeugeHilfeText`,
  `Anzeige`, `parser::Parser`, `parse_rekursiv`.
- [ ] Confirm no dead code remains from the old commented-out single-pass sketch in
  `combine.rs` (should already be deleted in Phase 1, verify nothing was reintroduced).
- [ ] Fix the pre-existing `unnecessary_semicolon` clippy error in
  `kommandozeilen_argumente_derive/src/parse.rs:1005`.
- [ ] Resolve every warning `cargo clippy --workspace --all-features -- -D warnings` reports:
  add missing documentation, remove or `#[allow(dead_code)]`-annotate-with-justification any
  genuinely-unused private items (e.g. `filter_prefix`, `strip_as_prefix`/`strip_as_prefix_n` if
  still unused after the rename), and fix the `OptionHelper`/`ergebnis_anpassen`-style unread-field
  warning if it still exists.
- [ ] Run `cargo +nightly udeps` (or equivalent) to catch unused *dependencies* introduced or
  exposed by the rename.
- [ ] Review `src/lib.rs`'s re-export list once more against every public item actually defined,
  to confirm nothing new was left unexported and nothing deleted is still (accidentally)
  exported.
- [ ] Confirm both `Cargo.toml` files still read `version = "0.4.0"` and `CHANGELOG`/`README`
  (if present) mention the breaking rename, adding an entry if such files exist in the repo.
- [ ] Re-run the full test/lint/doc suite one final time.

**Automated Verification**:

- [ ] `cargo build --workspace --all-features` succeeds with zero warnings
- [ ] `cargo test --workspace --all-features` passes
- [ ] `cargo clippy --workspace --all-features -- -D warnings` passes
- [ ] `cargo doc --workspace --all-features --no-deps` succeeds with no broken links
- [ ] `grep -rn 'kombiniere!\|Kombiniere\|ErzeugeHilfeText\|dyn_to_owned::Anzeige\|parse_rekursiv' src/ tests/ kommandozeilen_argumente_derive/src/` returns no results

---

## References

- `docs/agents/research/2026-07-19-parse-in-stages-refactoring-context.md` — original research
  report (6-stage design summary, implementation status, bilingual-API convention)
- `doc/parsing.md` — design of the 6-stage pipeline
- `src/argumente.rs`, `src/argumente/{einzelargument,flag,wert,frühes_beenden,kombiniere,hilfe}.rs`,
  `src/beschreibung.rs`, `src/ergebnis.rs`, `src/sprache.rs`, `src/unicode.rs`,
  `src/dyn_to_owned.rs`, `src/lib.rs`, `src/parse.rs` — pre-rename source read in full while
  planning
- `kommandozeilen_argumente_derive/src/{parse,enum_argument,utility}.rs` — derive-crate
  generated-code call sites audited for Phase 1
- `tests/derive.rs`, `tests/hilfe.rs` — existing integration test surface
