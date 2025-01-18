# Parsing

General idea: split up parsing in multiple stages.
Doing it all at ones becomes way too complicated.
Focus on a single thing per stage to keep the scope manageable.

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
- Remaining arguments with the parsed merged short names and associated value-strings removed

## Parse non-merged short name arguments
## Parse long name arguments
## Parse OsString for value arguments
