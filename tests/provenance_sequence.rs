//! Integration tests for source-line provenance on sequence-diagram SVG.
//! See `provenance_flowchart.rs` for the feature-gate pattern.
//!
//! Coverage:
//! - Edge (message) carries the line it was sent on
//! - SequenceFrame (`alt`, `loop`, `par`, `opt`) carries the
//!   opening-directive line, not the closing `end`
//! - SequenceNote carries its declaration line
//! - SequenceActivation (`activate` / `deactivate`) carries its
//!   directive line

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::layout;
use mermaid_rs_renderer::{RenderOptions, parse_mermaid, render_svg};

/// Render without the preflight validator, which (at fork rev 84e95ab)
/// treats any bare `end` as a subgraph close and rejects sequence
/// `alt`/`loop`/`par`/`opt` blocks that use `end` to mark their close.
/// Fixing the validator to understand sequence frames is tracked
/// separately.
fn render(input: &str) -> String {
    let parsed = parse_mermaid(input).expect("parse should succeed");
    let options = RenderOptions::default();
    let lay = layout::compute_layout(&parsed.graph, &options.theme, &options.layout);
    render_svg(&lay, &options.theme, &options.layout)
}

#[test]
fn sequence_message_carries_line() {
    //   1: sequenceDiagram
    //   2: Alice ->> Bob: Hello
    let svg = render("sequenceDiagram\nAlice ->> Bob: Hello\n");
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "message at line 2 missing; SVG was:\n{svg}"
    );
}

#[test]
fn sequence_frame_uses_opening_line() {
    //   1: sequenceDiagram
    //   2: Alice ->> Bob: Hi
    //   3: alt success
    //   4:   Bob ->> Alice: OK
    //   5: end
    let svg = render(
        "sequenceDiagram\nAlice ->> Bob: Hi\nalt success\n  Bob ->> Alice: OK\nend\n",
    );
    // alt frame opens on line 3, closes on line 5.
    assert!(
        svg.contains(r#"data-source-line="3""#),
        "alt frame at line 3 missing; SVG was:\n{svg}"
    );
    // Line 5 (`end`) must not appear as a data-source-line.
    assert!(!svg.contains(r#"data-source-line="5""#));
}

#[test]
fn sequence_note_carries_line() {
    //   1: sequenceDiagram
    //   2: Alice ->> Bob: Hello
    //   3: Note over Alice: Hi there
    let svg = render(
        "sequenceDiagram\nAlice ->> Bob: Hello\nNote over Alice: Hi there\n",
    );
    assert!(
        svg.contains(r#"data-source-line="3""#),
        "note at line 3 missing; SVG was:\n{svg}"
    );
}

#[test]
fn sequence_loop_frame_uses_opening_line() {
    //   1: sequenceDiagram
    //   2: Alice ->> Bob: Hi
    //   3: loop every minute
    //   4:   Bob -->> Alice: OK
    //   5: end
    let svg = render(
        "sequenceDiagram\nAlice ->> Bob: Hi\nloop every minute\n  Bob -->> Alice: OK\nend\n",
    );
    assert!(svg.contains(r#"data-source-line="3""#));
    assert!(!svg.contains(r#"data-source-line="5""#));
}

#[test]
fn sequence_activation_carries_line() {
    //   1: sequenceDiagram
    //   2: Alice ->> Bob: request
    //   3: activate Bob
    //   4: Bob -->> Alice: response
    //   5: deactivate Bob
    let svg = render(
        "sequenceDiagram\nAlice ->> Bob: request\nactivate Bob\nBob -->> Alice: response\ndeactivate Bob\n",
    );
    // The activation bar spans from the activate line; we expect
    // line 3 on the activation rect.
    assert!(
        svg.contains(r#"data-source-line="3""#),
        "activate at line 3 missing; SVG was:\n{svg}"
    );
}
