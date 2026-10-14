# Staged parsing

The parser recognizes command-line syntax before it constructs typed values. Splitting the work
into stages keeps name precedence, merged short forms, alternatives, and value conversion
independent. Every candidate is evaluated without stopping at the first error; errors are
returned only when no candidate succeeds.

The public entry point is `Arguments::parse`. With the optional `german` feature, the
`Argumente` mirror is available as well. It executes the six steps below.

## Shared recognition state

The first three stages pass input forward as `ArgumentInput`:

- `ArgumentInput::Unchanged(OsString)` is an untouched operating-system argument. It may match a
  name or, where applicable, supply the following raw value.
- `ArgumentInput::AdjustedMergedShortNames` is a remainder of a merged short-name block. It is
  carried forward for continued merged-short recognition, but is never matched as a standalone name
  and is never itself used as a raw value.

Recognition records early exits, flags, names with missing values, raw `OsString` values, and
remaining input. Repeated value names retain their last value. The three recognition-stage
outputs are merged into one `RecognizedArguments` state before typed evaluation.

## 1. Recognize merged short forms

Merged short names are recognized first, so they take precedence when short and long prefixes
are identical. For example, compatible `f`, `g`, and `h` flags can recognize `-fgh`.

A name participates only when:

- all names in the block share the short prefix;
- each short name is exactly one Unicode grapheme cluster;
- merging is enabled for that argument; and
- at most one value argument occurs, and it is last. Its inline value may follow directly or
  after the value infix.

A value argument can also take the next `Unchanged` input as its raw value. The stage may leave
an adjusted remainder for later recognizers. Value-like text after a short value name is
ambiguous: it may be that value or subsequent merged flags. Both recognition candidates are
retained, with the value interpretation first.

## 2. Recognize standalone short forms

Only `Unchanged` inputs participate. This stage recognizes short flags and early exits, as well
as short value names with either an inline value (optionally separated by the value infix) or the
following `Unchanged` input. Adjusted merged-block remainders pass through unchanged.

## 3. Recognize long forms

Only `Unchanged` inputs participate. This stage recognizes long flags, including inverted flags,
early exits, and value names with an inline value after the value infix or with the following
`Unchanged` input. Adjusted merged-block remainders again pass through unchanged.

## 4. Validate typed values

Each complete recognition candidate is traversed through the typed `Arguments`/`Combine` tree.
Each `Value` removes its own raw entry from a candidate-local copy of the raw-value map and calls
its parse callback. Flags use their most recently recognized value; defaults satisfy omitted
arguments. Missing required flags and values, and value conversion failures, become parser
errors.

An early exit validates immediately and does not require the other arguments to be present.
Definitions and callbacks need not implement `Clone`: only the candidate's recognition state is
cloned. Parse callbacks should be pure, because a selected value is parsed during validation and
again during final accumulation.

## 5. Select an alternative

`Arguments::Alternatives` evaluates its branches in declaration order using the complete input.
The first branch whose validation produces a value or an early exit wins. A failed value
conversion rejects only that branch; later branches can still succeed. If all branches fail,
the parser returns the accumulated errors from all failed branches.

The same first-successful-candidate rule resolves ambiguities introduced by merged short values.

## 6. Accumulate the selected result

After validation selects a candidate, the parser consumes the selected `Arguments` definition
once. The typed tree parses its selected raw values, applies flag conversions/defaults, and calls
the `Combine` function to build the result. An early exit instead returns its messages. The
unconsumed `ArgumentInput`s are returned alongside the result for the caller or a complete-parse
convenience method to handle.

Raw values are never erased to `Any` or downcast. Their static type remains in the owning
`Value<'t, T, E>`, so callbacks may produce values borrowing from the argument-definition
lifetime.
