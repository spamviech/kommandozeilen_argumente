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

Results of this stage are:

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

Results of this stage are:

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

Results of this stage are:

- A vector of early\_exit arguments, containing name, message & original input.
- A vector of flag-arguments with their name (all are true) & the original input.
- A map of value-arguments with name -> (value-string, original input).
- Remaining arguments with the parsed long names and associated value-strings removed.

Parsed arguments are then merged with the previous stage.

## Parse OsString for value arguments

Inputs:

- A map of value-arguments with name -> (value-string, original input).

Stage responsibility:
Value-strings are parsed according to the parse-value-function.
The result is stored with as `dyn std::any::Any` and a `TypeId`,
to allow holding them in the same map.

Outputs: A map of value-arguments with name -> `(value: Box<dyn Any>, TypeId)`

## Accumulate results

Inputs:

- A vector of early\_exit arguments, containing name, message & original input.
- A vector of flag-arguments with their name (all are true) & the original input.
- A map of value-arguments with name -> (value: Any, TypeId)
- Remaining arguments with the parsed arguments and associated value-strings removed.

Stage responsibility:
Convert the type-less parsed types to the result-struct.

- For flag-arguments, convert the boolean according to the configured method.
- For value-arguments, use the TypeId to downcast to the actual type.
- Use the combine-function to construct larger structs.

Outputs: (Result enum, remaining arguments)

Result variants:

- EarlyExit-messages, errors if any
- Result struct, errors if any
- errors if no result can be constructed
