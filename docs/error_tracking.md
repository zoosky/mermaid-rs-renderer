# Error Line and Column Conventions

`ParseError` variants in `src/error.rs` carry `line` and/or `col`
fields. This document pins the semantics so detection paths added
in follow-up work stay consistent with the existing preflight
validator and the integration tests in `tests/parse_errors.rs`.

## Line numbers

- **1-based.** The first line of input is line `1`, not `0`.
- **Counted in raw input.** No comment stripping, blank-line
  collapsing, or directive-removal is applied before numbering.
  The line numbers observed by a user reading their source file
  match the numbers returned in errors.
- Produced by `enumerate()` on `input.lines()` with `+ 1`; the
  preflight validator centralises this conversion in
  `validator::u32_from_index()`.

## Columns

- **1-based.** The first character of a line is column `1`.
- **Character offsets**, not byte offsets. UTF-8 multibyte
  characters (e.g. emoji in node labels) count as one column
  each, matching how editors position a cursor.
- Computed by `validator::col_of_first_nonws()` for "the start
  of the offending token on this line", or by
  `validator::col_of_char_offset()` when the detection path has
  a specific byte offset in hand.

## When columns are `1` or unavailable

- `ParseError::UnclosedSubgraph` carries only `opened_at: u32`.
  The opening keyword itself is the anchor; the column is not
  reported because the useful context is the enclosing block,
  not the token position on the opening line.
- When a detection path has a line number but no useful column
  (e.g. "this whole line is malformed"), report column `1`.
  Zero is reserved for "no positional information at all" and
  should not be emitted by any path that has a concrete line.

## Rationale for the 1-based, character-offset choice

- Matches editor behaviour (column indicators in Vim, Emacs,
  VS Code, and most IDEs).
- Matches the LSP spec's `position` semantics for line (1-based)
  while diverging from LSP's UTF-16 column counting -- this
  crate is not an LSP server and consumers are expected to be
  human-facing tooling, where character-level counting is less
  surprising.
- Ruff, rustc, cargo, and most Rust error reporters use 1-based
  line + 1-based character-column. Consistency with that
  ecosystem outweighs LSP alignment.

## Contract for new detection paths

Any new detection path added alongside `validator::validate()`
must:

1. Use `u32_from_index(idx + 1)` (or equivalent arithmetic) when
   converting `enumerate()` indices into `line` fields.
2. Use `col_of_first_nonws()` or `col_of_char_offset()` when
   populating `col` fields, never raw byte offsets.
3. Saturate at `u32::MAX` rather than panic on overflow
   (helpers already do this via `u32::try_from(...).unwrap_or(u32::MAX)`).
4. Add test coverage in the `#[cfg(test)]` block of `validator.rs`
   **and** at least one integration test in `tests/parse_errors.rs`
   that asserts the line/col numbers for a known input, so
   regressions surface immediately.

See `tests/parse_errors.rs::unknown_participant_reports_line_number`
for a worked example.
