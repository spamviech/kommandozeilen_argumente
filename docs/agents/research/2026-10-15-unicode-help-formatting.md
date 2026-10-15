---
date: 2026-10-15T13:53:46.203812+00:00
git_commit: 1700fb81e91a01bb17793692a497814e14239266
branch: parse-in-stages
topic: "How should generated help text be formatted while fully accounting for Unicode characters?"
tags: [research, codebase, help-text, unicode, terminal-width]
status: complete
---

# Research: Unicode-aware help-text formatting

## Research Question

How would proper formatting of the generated help text look while fully accounting for Unicode characters?

## Summary

The current help renderer aligns its description column using UTF-8 **byte lengths** (`str::len()`), rather than terminal display-column widths. This is deliberate in the current test suite: `tests/help.rs` calls this behavior `unicode_help_uses_byte_length_padding`. Parsing does account for Unicode normalization and grapheme boundaries, but those parsing facilities are not used when rendering help text.

For terminal-aligned help, each rendered syntax entry and prefix needs a measurement representing its number of occupied terminal cells. The renderer must calculate the largest such width, then pad each entry by the difference in cells. This makes the descriptions begin in the same display column for ordinary ASCII, accented text, combining sequences, emoji sequences, and wide CJK characters, subject to the chosen terminal-width policy.

The relevant distinction is:

| Unit | Example `föö` | Suitable for alignment? |
|---|---:|---|
| UTF-8 bytes (`len()`) | 5 | No |
| Unicode scalar values (`chars().count()`) | 3 | No |
| Grapheme clusters | 3 | No; clusters can have different cell widths |
| Terminal display cells | 3 in ordinary terminals | Yes |

For example, the current byte-based output has the descriptions at different columns:

```text
  --[no-]föö | -f Unicode flag. Default: false
  --help | -h       Show this text.
```

A cell-width-aware renderer would render the same entries as:

```text
  --[no-]föö | -f  Unicode flag. Default: false
  --help | -h       Show this text.
```

Both syntax strings occupy 15 terminal cells; the descriptions therefore start in the same column. (The existing renderer includes one mandatory separator cell between syntax and description.)

### Decision

Will not be implemented at the moment. Document kept for potential future use.

## Relevant Files

```text
src/
├── arguments.rs                 # Builds and writes the complete help message
├── unicode.rs                   # NFC/CJK-compatibility normalization and grapheme-aware parsing helpers
├── language.rs                  # Localizable syntax prefixes, padding and alternative separator
└── arguments/
    ├── help.rs                  # Help and Alternatives intermediate representations
    ├── flag.rs                  # Flag syntax construction
    ├── value.rs                 # Value-argument syntax construction
    └── early_exit.rs            # Early-exit syntax construction

tests/
└── help.rs                      # Current byte-length Unicode-padding regression test
```

## Detailed Findings

### Help-text construction and rendering

`Arguments::with_help_early_exit` builds a `NonEmpty<Alternatives>` representation from the argument tree, appends the generated help early-exit argument, and writes the header and each entry (`src/arguments.rs:994-1048`). `Help` contains a prebuilt syntax `String` and an optional description `String`; `Alternatives` captures single entries, nested alternatives, and omitted fixed values (`src/arguments/help.rs:11-61`).

Syntax strings are assembled before alignment:

- Flags combine long form, optional inversion notation, and optionally a short form (`src/arguments/flag.rs:102-132`).
- Values add their value separator/metavariable and optional short form (`src/arguments/value.rs:504-553`).
- Early-exit arguments render long and optional short forms (`src/arguments/early_exit.rs:62-76`).

The renderer currently obtains the maximum syntax width recursively with `argument.syntax.len()`, adding `alternative_prefix.len()` for nested alternatives (`src/arguments.rs:1183-1202`). It writes a line by subtracting `current_prefix.len()` and `syntax.len()` from that maximum, then repeats `syntax_padding` that many times (`src/arguments.rs:1218-1234`). It applies the same byte-based calculation to the separator written between alternative lines (`src/arguments.rs:1236-1267`).

The configured English and German defaults use two leading spaces, ASCII space padding, `"| "` alternative prefixes, and `'-'` alternative separators (`src/language.rs:51-105`). These defaults are one display cell per character, but the public formatting method accepts arbitrary Unicode strings/chars for them (`src/arguments.rs:994-1007`). Unicode-aware formatting must measure these configurable presentation values by display width as well, not assume ASCII.

### Existing Unicode support is for parsing, not layout

`Normalized::new` performs CJK compatibility-variant normalization followed by NFC normalization (`src/unicode.rs:44-64`). `Compare::strip_as_prefix_n` uses grapheme boundaries while comparing and splitting input (`src/unicode.rs:350-375`). The parser documentation also specifies that mergeable short names must be exactly one Unicode grapheme cluster (`docs/parsing.md:29-41`).

Those semantics correctly address argument recognition. They do not determine how many terminal columns text occupies. NFC makes canonically equivalent spellings consistent, and grapheme segmentation prevents splitting visible user characters, but neither answers whether a grapheme occupies zero, one, or two terminal cells.

### Required display-width model

A Unicode-complete alignment model has three stages for every output fragment contributing to a column position:

```text
UTF-8 string
    │
    ├─ remove nonprinting terminal control sequences, if the renderer emits any
    ├─ normalize/select a Unicode width policy
    └─ calculate terminal display cells
             │
             └─ max syntax width and per-line padding
```

The current renderer emits plain text only; no ANSI escape-sequence processing exists in the codebase. If callers are permitted to put terminal escapes in names, metavariables, help descriptions, or localization fields, a display-width utility must treat recognized escape sequences as zero-width. Otherwise, terminal colors/styles must remain out of the measured syntax values.

For visible Unicode, the selected width function needs at least these outcomes:

| Text category | Typical cell width | Alignment treatment |
|---|---:|---|
| ASCII letters and punctuation | 1 per character | Count normally |
| Precomposed accented letter, such as `ö` | 1 | One cell, despite two UTF-8 bytes |
| Combining mark, such as `e` + U+0301 | 0 for the mark | The combined grapheme occupies one cell |
| CJK wide/fullwidth character, such as `界` | 2 | Count as two cells |
| Emoji / emoji ZWJ sequence | commonly 2 for the sequence | Measure according to terminal-width policy, not bytes/scalars |
| Variation selectors / zero-width joiners | 0 themselves | Included in the surrounding rendered grapheme/emoji behavior |
| East Asian ambiguous-width characters | 1 or 2 depending on policy/terminal locale | Policy must be explicit and used consistently |
| Tabs, newlines, and other controls | context-dependent | Do not treat as ordinary one-cell syntax characters; either constrain or handle their cursor effects |

Under that model, the rendering arithmetic is conceptually:

```text
entry_width = display_width(current_prefix) + display_width(syntax)
column_width = maximum display width of each syntax path
padding_cells = column_width + separator_cells - entry_width
```

For the default ASCII space padding character, writing `padding_cells` spaces produces the required column position. Because `with_help_early_exit` exposes `syntax_padding: char`, fully general behavior also needs to establish that its display width is exactly one cell. A zero-width or double-width padding character cannot represent an arbitrary residual cell count without additional rules. The default `Language` values satisfy this requirement because `syntax_padding` is `' '`.

Nested alternatives follow the same model. Their `alternative_prefix` contributes visible cells to the child entry, while the separator line needs enough copies of `alternative_separator` to occupy the remaining cells. With arbitrary Unicode separators, repeated-character count is not generally equal to a requested cell width; a correct renderer must either require a one-cell separator or use a width-aware fill rule.

## Architecture Documentation

The help system separates syntax construction from final layout:

```text
Flag / Value / EarlyExit metadata
             │
             ▼
       Help { syntax, help }
             │
             ▼
  Alternatives (possibly nested)
             │
             ▼
Arguments::with_help_early_exit
  ├─ max_syntax_width(...)
  └─ write_argument_or_alternatives(...)
             │
             ▼
      EarlyExit message shown by --help
```

This is the location where display-cell measurement belongs: the syntax text has already been generated, and the layout code has all prefixes and separators available. The parsing-oriented `unicode.rs` utilities are related Unicode infrastructure but do not currently provide terminal column measurement.

No dependency that provides terminal display-width calculation is listed in `Cargo.toml`; the Unicode dependencies are `unicode-normalization` and `unicode-segmentation` (`Cargo.toml:56-65`, `Cargo.toml:102-109`).

## Test Coverage

`tests/help.rs:27-46` demonstrates and asserts the present behavior with a non-ASCII flag name, `föö`. It expects:

```text
  --[no-]föö | -f Unicode flag. Default: false
  --help | -h       Show this text.
```

`föö` uses five UTF-8 bytes while `help` uses four, so the current byte-based calculation places both descriptions after equal byte counts, not equal terminal-cell counts. The test name explicitly documents that it is validating byte-length padding.

A full terminal-width test matrix would exercise the layout outcome—not only the Unicode parser—with representative syntax values containing a precomposed accent, a decomposed combining sequence, a CJK wide character, an emoji sequence, and alternatives with Unicode presentation configuration. Its expected assertions would compare the display-column where each description starts under the selected width policy.

## Code References

- `src/arguments.rs:994-1048` — help header creation, entry aggregation, and rendering invocation.
- `src/arguments.rs:1183-1202` — current maximum width calculation using UTF-8 byte length.
- `src/arguments.rs:1208-1270` — current line and alternative rendering/padding using byte length.
- `src/arguments/help.rs:11-61` — intermediate help entries and nested alternatives.
- `src/arguments/flag.rs:102-132` — flag syntax and description construction.
- `src/arguments/value.rs:504-553` — value syntax and description construction.
- `src/arguments/early_exit.rs:62-76` — early-exit syntax construction.
- `src/language.rs:5-45,51-105` — configurable layout strings and default language settings.
- `src/unicode.rs:44-64,350-375` — normalization and grapheme-aware parser comparisons.
- `docs/parsing.md:29-41` — one-grapheme rule for merged short names.
- `tests/help.rs:27-46` — test documenting current byte-length padding.

## Open Questions

- Which terminal-width convention should define ambiguous-width characters and emoji sequences for this crate's supported environments?
- Should help syntax and localization fields be constrained to printable, single-line text, or should the renderer interpret controls and ANSI SGR escape sequences?
- Should arbitrary Unicode `syntax_padding` and `alternative_separator` values remain configurable, and if so, what behavior is defined when their cell widths do not divide the required fill width?

## Follow-up Research 2026-10-15T13:55:36.529043+00:00

### How terminal emoji-width policy can be determined

There is no standard, reliable terminal capability that reports its Unicode, emoji, or East Asian ambiguous-character width policy. Terminfo reports structural capabilities and dimensions (such as `cols#80`), but it does not describe how the terminal renderer advances the cursor for a particular Unicode sequence. Environment variables such as `TERM`, `COLORTERM`, `LANG`, and `LC_CTYPE` identify broad terminal/locale context only; they do not establish emoji cell widths.

The current research shell reports `TERM=xterm-256color`, UTF-8 locale `LANG=en_US.UTF-8`, and 80 columns. Its `xterm-256color` terminfo entry reports `cols#80`, but no emoji-width behavior. Thus this information cannot select an emoji-width policy.

A program can empirically measure a particular interactive terminal by:

1. verifying that stdout and stdin are a TTY;
2. saving the cursor position;
3. querying the cursor position with the ANSI Device Status Report `CSI 6n`;
4. printing a selected Unicode test sequence;
5. querying cursor position again and subtracting the columns, accounting for wrapping; and
6. restoring the cursor state.

This measures the current emulator, font/rendering configuration, and sequence at runtime. It needs robust timeout and input handling because the response arrives on terminal input and may be absent, filtered, or interleaved with user input. It is not suitable as a general assumption: terminals, terminal multiplexers, remote connections, CI output capture, and non-TTY streams can suppress or alter the query/response. A test must also be run separately for every sequence of interest (`😀`, U+2764 with variation selector, ZWJ family sequences, flags, keycaps, etc.), because emoji-sequence width is not reducible to a single emoji-wide setting.

In normal command-line help generation, the portable approach is to select and document a library-side Unicode width policy rather than attempt interactive detection. Such a policy remains an approximation of the user’s renderer, but it produces deterministic output for redirected streams and all terminals.
