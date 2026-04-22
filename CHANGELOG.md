# Changelog

## [Unreleased]

### Added: `embedded-font` cargo feature

- New cargo feature `embedded-font` (not in `default`) that ships
  Inter Regular + Inter Bold (TrueType, SIL OFL 1.1) as
  `include_bytes!` constants under `assets/fonts/`. Adds ~822 KB to
  the binary when enabled.
- When the feature is on, the text-metric loader skips
  `fontdb::Database::load_system_fonts()` and populates the
  database with the bundled bytes instead. This removes the
  startup filesystem-scan cost that dominates first-render latency
  on servers and sandboxed environments, and makes font resolution
  deterministic across hosts.
- Generic-family fallbacks (`sans-serif`, `serif`, `monospace`,
  `cursive`, `fantasy`) are all aliased to Inter via
  `Database::set_sans_serif_family` (and siblings), so CSS such as
  `font-family: "foo", "bar", sans-serif` still resolves to the
  bundled face when the named families are absent from the DB --
  without this, queries that miss the named families would fall
  through to `fontdb`'s hardcoded "Arial" / "Times New Roman" /
  "Courier New" anchors which are not registered here, and
  callers would silently regress to character-count heuristics.
- `--fastText` remains orthogonal; callers can combine
  `embedded-font` with the fast-text fallback path.
- New bench `benches/font_startup.rs` measures the cost of
  populating a fresh `fontdb::Database` with this feature on or off.
  Measured on macOS arm64 (Apple M-series, 2026-04-22):
  - `db_load_system_fonts`: ~11.3 ms per fresh database
    (filesystem scan).
  - `db_load_embedded_fonts`: ~1.1 µs per fresh database
    (zero-copy via `Source::Binary(Arc<&'static [u8]>)`) --
    roughly 10,000x faster.
- The embedded-font loader uses `Database::load_font_source` with
  `Arc<&'static [u8]>` so the static font bytes are referenced
  directly from rodata, avoiding a per-startup ~822 KB heap copy
  that `Database::load_font_data(Vec<u8>)` would require.
- Font files live at `assets/fonts/Inter-Regular.ttf`,
  `assets/fonts/Inter-Bold.ttf`, with the license text at
  `assets/fonts/OFL.txt` (SIL Open Font License 1.1).

### Added: Structured `ParseError` and strict library entry points

- New `ParseError` enum in `src/error.rs` with four variants:
  `UnknownParticipant`, `UnclosedSubgraph`, `UnexpectedToken`, and
  `InvalidDirective`. Marked `#[non_exhaustive]` so future variants
  can be added without breaking semver. Derives `thiserror::Error`.
- New public entry points in `lib.rs`:
  - `parse_mermaid_strict(input: &str) -> Result<ParseOutput, ParseError>`
  - `render_strict(input: &str, options: RenderOptions) -> Result<String, ParseError>`
- New preflight validator (`src/validator.rs`) runs before the
  per-type parsers and reports malformed input as typed `ParseError`
  variants. Initial coverage: six starter detection paths --
  invalid `%%{init}%%` JSON, unclosed subgraph, stray `end`,
  leading-arrow lines, unbalanced `click` quotes, and
  unknown-participant references in sequence diagrams that declare
  at least one participant explicitly.
- New integration tests in `tests/parse_errors.rs`: 25 cases,
  covering every `ParseError` variant with at least 5 tests each.

### Changed

- Existing `render(...)` and `render_with_options(...)` now delegate
  to `render_strict` internally and map `ParseError` through
  `.into()` for `anyhow`. Public signatures remain
  `anyhow::Result<String>`; all existing call sites continue to
  compile and behave identically on valid inputs.
- `parse_class_line` in `src/parser.rs`: rewrote the guarded
  `parts.last().unwrap()` as `.expect("parts.len() >= 3 checked above")`.
  The guard above makes this unreachable today; the `.expect()` message
  documents the invariant.

### Documentation

- New `docs/unwrap_audit.md`: full audit of every `.unwrap()` call
  under `src/`, classifying each as compile-time-safe
  (`Lazy<Regex>::new()` at module load), guarded (length-checked
  above), or runtime-reachable.
- New `docs/error_tracking.md`: pins the 1-based line / 1-based
  character-column conventions that `ParseError` variants use, so
  follow-up detection paths stay consistent.

### Follow-up

- Full per-diagram-type error detection (i.e., wiring `ParseError`
  returns into the 23 per-type parsers rather than a single
  preflight pass) is tracked separately downstream and will land
  as additive detection paths without changing the public API.

## v0.2.2 (2026-04-23)

### Visual and Layout Fixes
- Fixed sequence diagram `alt` frame geometry and prevented wide section labels from panicking layout.
- Fixed compact flowchart label decorations.
- Made dotted edges visually distinct from solid edges.
- Fixed class diagram stereotypes being rendered as members.
- Fixed class diagram arrowheads being hidden under node boxes.
- Fixed state diagram description lines so titles are preserved and descriptions accumulate.
- Fixed empty-subgraph layout panic by keeping graph-level and local subgraph indexes mapped correctly.

### Rendering and Theme Fixes
- Fixed invalid non-ASCII hex color values causing panics.
- Preserved quoted font-family normalization for SVG text output.

### Gantt
- Added compact Gantt display mode via YAML frontmatter (`displayMode: compact`).

### Dependencies and Release
- Updated `anyhow`, `clap`, `criterion`, `regex`, and release action dependencies.
- Added release workflow automation for Homebrew and AUR package updates.

## v0.2.0 (2026-02-07)

### Layout Engine Overhaul
- Rewrote flowchart layout with improved routing, subgraph compaction, and tighter node spacing
- Auto-place edge labels with collision-aware search grid
- Added edge label relaxation for Flowchart, State, ER, and Requirement diagrams
- Node overlap resolver now runs for all diagram types when overlaps are detected
- Finer-grained label placement search for closer label-to-edge proximity

### Visual Quality Improvements
- Redesigned ER diagram tables with cleaner styling
- Redesigned pie charts with improved label readability
- Redesigned journey diagram layout
- Improved state diagram composite labels and marker sizing
- Improved gantt chart rendering: section bands, color coding, in-bar labels
- Improved mindmap, class, and flowchart rendering polish
- Compact subgraph sizing across diagram types

### Parser Fixes
- Parse `-- "text" -->` quoted edge label syntax (fixes #27)

### Performance
- Added font cache for text metrics — avoids redundant font lookups
- Added `--fastText` option for approximate text width metrics

### Benchmarking & Quality
- Layout quality scoring vs mermaid-cli
- 16 new stress fixtures for benchmarks
- Expanded comparison examples across all diagram types
- Sankey link path detection in quality checks

## v0.1.3 (2026-02-02)

Initial public release with 13 diagram types and 100-1400x performance vs mermaid-cli.
