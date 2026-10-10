# Parsing

General idea: split up parsing in multiple stages.
Doing it all at ones becomes way too complicated.
Focus on a single thing per stage to keep the scope manageable.

All stages can encounter parse-errors.
They will not stop parsing.
Instead, they are collected and presented as a vector at the end.

## Parse merged short name arguments

This has to happen first.
When the prefix for short and long names is equal this might influence the result of later stages.
Merged short names always take precedence.

Rules to allow merging of short names:

- All short names in the same string share the same (short) prefix.
- Only short names consisting of a single [grapheme](https://docs.rs/unicode-segmentation/1.8.0/unicode_segmentation/trait.UnicodeSegmentation.html#tymethod.graphemes) participate.
- At most one value argument per block.
    It must be the last argument name in the string, optionally followed by \[a value-infix and\] the value sub-string.
- Merging of short names must be allowed for this particular argument.

Results of this stage are a `NonEmpty` (alternatives) of:

- `Argumente<'_, T, F>` of the alternative taken.
- A vector of early\_exit arguments, containing name, message & original input.
- A vector of flag-arguments with their name (all are true) & the original input.
- A map of value-arguments with name -> (value-string, original input).
- Remaining arguments with the parsed merged short names and associated value-strings removed.

## Parse non-merged short name arguments

Input: Remaining arguments of the previous stage.

Parsed in this stage (only short names):

- A flag/early\_exit argument
- A value argument, followed by \[a value-infix and\] the value sub-string.
- A value argument, followed by the value string in the next input-OsString.

Results of this stage are a `NonEmpty` (alternatives) of:

- `Argumente<'_, T, F>` of the alternative taken.
- A vector of early\_exit arguments, containing name, message & original input.
- A vector of flag-arguments with their name (all are true) & the original input.
- A map of value-arguments with name -> (value-string, original input).
- Remaining arguments with the parsed short names and associated value-strings removed.

Parsed arguments are then merged with the previous stage.

## Parse long name arguments

Input: Remaining arguments of the previous stage.

Parsed in this stage (only long names):

- A flag/early\_exit argument
- A value argument, followed by \[a value-infix and\] the value sub-string.
- A value argument, followed by the value string in the next input-OsString.

Results of this stage are a `NonEmpty` (alternatives) of:

- `Argumente<'_, T, F>` of the alternative taken.
- A vector of early\_exit arguments, containing name, message & original input.
- A vector of flag-arguments with their name, value (might be inverted) & the original input.
- A map of value-arguments with name -> (value-string, original input).
- Remaining arguments with the parsed long names and associated value-strings removed.

Parsed arguments are then merged with the previous stage.

## Parse OsString for value arguments

Execute for each alternative.

Inputs:

- `Argumente<'_, T, F>` of the alternative taken.
- A vector of early\_exit arguments, containing name, message & original input.
- A vector of flag-arguments with their name, value (might be inverted) & the original input.
- A map of value-arguments with name -> `(OsString, original input)`.
- Remaining arguments with the parsed long names and associated value-strings removed.

Stage responsibility:
Keep the raw-value map intact until its owning typed `Value<'t, T, F>` is accumulated. That
argument removes its own entry and invokes its own parse-value function, yielding `T` directly or
an `AnnotatedParseError`. The type remains in the statically typed argument/combine tree; it is
not erased into `Any` and therefore may borrow for the definition lifetime (`T: 't`).

This parsing is performed while evaluating each alternative, not after one has already been
chosen. Each candidate receives an independent raw-state view (normally a clone of the raw map),
so a failed conversion neither selects that candidate nor consumes input needed by the next one.

Outputs:

- A candidate-specific raw-value map, consumed incrementally by typed value arguments during
  candidate accumulation.
- Remaining arguments with the parsed long names and associated value-strings removed.

## Pick alternative

Inputs: `NonEmpty` (alternatives) of:

- A vector of early\_exit arguments, containing name, message & original input.
- A vector of flag-arguments with their name (all are true) & the original input.
- A map of value-arguments with name -> `(OsString, original input)`.
- Remaining arguments with the parsed arguments and associated value-strings removed.

Stage responsibility:
For each alternative in order, parse its raw values and accumulate its typed result using an
independent copy/view of the complete staged state. Pick the first candidate that fully succeeds.
A value conversion error rejects only that candidate and evaluation continues with the next one.
If no candidate succeeds, return the accumulated errors from every failed candidate.

Outputs:

- The successful candidate's typed result and its remaining arguments; or all candidate errors.

## Accumulate results

Inputs:

- One candidate's independent staged state: early exits, flags, raw values, and remaining
  arguments.

Stage responsibility:
Convert one candidate's parsed pieces to the result struct through their typed owning definitions.
This is part of evaluating an alternative and happens before the winning alternative is selected.

- For flag-arguments, convert the boolean according to the configured method.
- For value-arguments, the matching typed `Value<'t, T, F>` removes and parses its raw entry,
  returning `T` directly; no `TypeId` lookup or runtime downcast occurs.
- Use the combine-function to construct larger structs.

Outputs: (Result enum, remaining arguments)

Result variants:

- EarlyExit-messages, errors if any
- Result struct, errors if any
- errors if no result can be constructed
