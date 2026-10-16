# Potential module splits

This is a future-maintenance note. The candidates below are internal reorganizations and should
preserve the current public API by re-exporting items from their existing module paths.

## Strong candidates

### `src/arguments/value.rs`

This is the clearest opportunity. It currently contains:

- `Value` and the German `Wert` mirror, including conversions;
- constructors;
- help-text rendering;
- typed-to-string conversion;
- recognition for merged short, standalone short, and long forms; and
- validation and consumption of parsed values.

A possible layout:

- `arguments/value.rs` — public types, constructors, and core methods;
- `arguments/value/parsing.rs` — `parse_*_value_forms` helpers;
- `arguments/value/help.rs` — `create_value_help_text` and `show_elements`;
- optionally `arguments/value/german.rs` — the `Wert` mirror, conversions, and German methods.

The parsing helpers are independently substantial and align with the existing staged-parser
structure in `arguments/parsing.rs`.

### `src/description.rs`

This combines three distinct concerns:

- name matching and recognition (`Name` and merged-short handling);
- the public description model (`Description` and `Beschreibung`); and
- flexible construction traits (`LongNames` and `ShortNames`).

A possible layout:

- `description/name.rs` — `Name` and flag/value/merged-short recognition;
- `description/input.rs` — `ArgumentInput` and merged-short remainder state;
- `description/names.rs` — `LongNames`/`ShortNames` implementations;
- `description.rs` or `description/mod.rs` — `Description`/`Beschreibung` and re-exports.

This would isolate the low-level, correctness-sensitive recognition code and make it easier to
navigate and test independently.

### `kommandozeilen_argumente_derive/src/parse/types.rs`

Despite its name, this is a general derive-support module containing:

- parsed-item conversion (`ParseItem`, `try_into_parse_item`);
- language and field parsing strategy;
- program help/version configuration;
- generated prefix/name configuration; and
- derive-specific errors.

A possible layout:

- `parse/item.rs` — `ParseItem` and supported-item conversion;
- `parse/settings.rs` — `Language`, `ProgramSettings`, and help/version settings;
- `parse/names.rs` — prefix and long/short-name settings.

Afterward, either retain a small shared `types.rs` or remove it.

## Moderate candidates

### `src/arguments.rs`

Parsing is already isolated in `src/arguments/parsing.rs`. The useful next extraction is
help/version early-exit composition:

- `arguments/program_help.rs` or `arguments/early_exit_helpers.rs` — `with_*_early_exit`,
  `max_syntax_width`, and `write_argument_or_alternatives`.

This would leave `arguments.rs` focused on the `Arguments` parser tree and error-type conversion.

### `src/arguments/parsing.rs`

The file is coherent, so split only if it grows further. A natural boundary is:

- recognition-state types and their merge/compatibility logic (`ParsedForms`,
  `RecognizedArguments`, and alternative-branch bookkeeping); and
- `Arguments` stage traversal plus the top-level parse/exit convenience APIs.

Do not split it solely for its present length; the two parts are tightly coupled.

### `kommandozeilen_argumente_derive/src/parse/attributes.rs`

Potentially split by attribute scope:

- program-level attributes (`program`, `help`, `version`, and language);
- field-level value/name attributes.

Its recursive parsing is likely more coupled than `types.rs`; tackle `types.rs` first.

## Modules to leave intact for now

- `src/parse.rs`: one public `Parse` trait and its forwarding convenience methods. Splitting trait
  methods harms API documentation and discovery.
- `src/unicode.rs`: English/German mirrors aside, `Normalized`, `Compare`, and their conversions
  form one cohesive abstraction; splitting mainly adds plumbing.
- `src/arguments/single_argument.rs`: bilingual duplication exists, but it is the central
  dispatcher for `Flag`, `Value`, and `EarlyExit`. Splitting German mirrors is possible but lower
  priority than splitting `value.rs`.
- `src/arguments/argumente/`: already decomposed by concern.

## Priority

1. `src/arguments/value.rs`
2. `src/description.rs`
3. `kommandozeilen_argumente_derive/src/parse/types.rs`
4. `src/arguments.rs`
