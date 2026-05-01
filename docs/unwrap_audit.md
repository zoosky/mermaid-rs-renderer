# Parser `.unwrap()` Audit

Performed 2026-04-21 as part of the upstream structured-error work
(see `CHANGELOG.md` `[Unreleased]` and the accompanying PR).

A prior audit framed the crate as "91 parser unwraps that can panic
on malformed input". Under actual inspection, 81 of those are inside
`#[cfg(test)]` blocks and ship zero code. Of the ten production
call sites, nine are `Lazy<Regex>::new()` at module load (so the
regex literals compile once at startup, never against user input),
and the remaining one is guarded by an explicit length check. This
document records the full audit so future maintainers do not have
to repeat it.

## Scope

All `.unwrap()` call sites under `src/` that are reachable from the
library's public API. Call sites inside `#[cfg(test)]`,
`#[cfg(bench)]`, `tests/`, `benches/`, or doc-comment examples
(`///` / `//!`) are out of scope.

Count command:

```bash
# All occurrences (tests + prod):
grep -c '\.unwrap()' src/parser.rs src/render.rs src/lib.rs

# Production-only (below the #[cfg(test)] boundary of each file):
awk '/^#\[cfg\(test\)\]/{t=1} !t && /\.unwrap\(\)/{n++} END{print n}' src/parser.rs
```

## Findings

| file:line | kind | classification | action |
|-----------|------|----------------|--------|
| `src/parser.rs:14` | `Lazy::new(\|\| Regex::new(r"^(flowchart\|graph)\s+(\w+)").unwrap())` | compile-time-safe: literal regex | keep |
| `src/parser.rs:15` | `Lazy::new(\|\| Regex::new(r"^subgraph\s+(.*)$").unwrap())` | compile-time-safe | keep |
| `src/parser.rs:17` | `Lazy::new(\|\| Regex::new(r"^%%\{\s*init\s*:\s*(\{.*\})\s*\}%%").unwrap())` | compile-time-safe | keep |
| `src/parser.rs:22` | `Regex::new(...).unwrap()` (arrow/edge literal) | compile-time-safe | keep |
| `src/parser.rs:28` | `Regex::new(...).unwrap()` (arrow/edge literal) | compile-time-safe | keep |
| `src/parser.rs:34` | `Regex::new(...).unwrap()` (arrow/edge literal) | compile-time-safe | keep |
| `src/parser.rs:40` | `Regex::new(...).unwrap()` (arrow/edge literal) | compile-time-safe | keep |
| `src/parser.rs:46` | `Regex::new(...).unwrap()` (arrow/edge literal) | compile-time-safe | keep |
| `src/parser.rs:49` | `Regex::new(...).unwrap()` (arrow/edge literal) | compile-time-safe | keep |
| `src/parser.rs:5405` | `parts.last().unwrap().to_string()` in `parse_class_line` | guarded: `if parts.len() < 3 { return; }` at `:5402` makes `.last()` always `Some` | rewritten as `.expect("parts.len() >= 3 checked above")` for clarity |
| `src/render.rs:5412` | `usvg::Size::from_wh(800.0, 600.0).unwrap()` | const literal: `800.0 × 600.0` always produces a valid `Size` | keep |

**Ten production unwraps. Zero user-input-reachable panic sites.**

## Counts

| Bucket | Count |
|--------|------:|
| `src/parser.rs` total `.unwrap()` occurrences | 91 |
| `src/parser.rs` inside `#[cfg(test)]` blocks | 81 |
| `src/parser.rs` production | 10 |
| `src/render.rs` production | 1 |
| `src/lib.rs` production | 0 (all six hits are `///` / `//!` doc-comment examples) |
| **Total production `.unwrap()` sites reviewed** | **11** |
| Compile-time-safe (kept) | 10 |
| Guarded (rewritten to `.expect()` with an explanatory message) | 1 |
| Runtime user-input-reachable | **0** |

## Implication

The f160a `catch_unwind` wrapper in Accent's `src/render/diagram/mermaid/`
remains a defence-in-depth layer -- it guards against panics from
future code paths, not from any panic site existing today. The real
value of the structured-error work is reporting malformed input as
typed `ParseError` variants (currently absent: the parser's per-type
functions end with unconditional `Ok(ParseOutput { ... })` on any
input). See `CHANGELOG.md` `[Unreleased]` for the five starter
detection paths introduced alongside this audit.
