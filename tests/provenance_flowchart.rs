//! Integration tests for source-line provenance on flowchart SVG.
//!
//! Only compiled when the `source-provenance` cargo feature is on
//! (the default). When disabled, these tests compile to no-ops so
//! `cargo test --no-default-features --features cli,png` stays green.
//!
//! Coverage for f160e Phase 1 flowchart acceptance criteria:
//! - Every Node gets a `data-source-line` attribute on its `<g>` wrapper
//! - Every Edge gets a `data-source-line` attribute on its `<path>`
//! - Every Subgraph gets a `data-source-line` attribute on its `<g>` wrapper
//! - Multi-line subgraph constructs carry the opening-line number,
//!   not the closing `end`
//! - First-mention wins when the same node id is referenced from
//!   multiple lines

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn flowchart_edge_carries_opening_line() {
    // Lines (1-based):
    //   1: flowchart LR
    //   2: A --> B
    let svg = render("flowchart LR\nA --> B\n");
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "edge from line 2 missing data-source-line; SVG was:\n{svg}"
    );
}

#[test]
fn flowchart_node_carries_first_mention_line() {
    // Node A first mentioned on line 2, referenced again on line 3.
    // First-mention line should win.
    //   1: flowchart LR
    //   2: A --> B
    //   3: A --> C
    let svg = render("flowchart LR\nA --> B\nA --> C\n");
    // Expect at least one group with data-source-line="2" (node A or edge 2).
    assert!(svg.contains(r#"data-source-line="2""#));
    // Node C is first-mentioned on line 3.
    assert!(
        svg.contains(r#"data-source-line="3""#),
        "node C from line 3 missing data-source-line; SVG was:\n{svg}"
    );
}

#[test]
fn flowchart_subgraph_uses_opening_line_not_end() {
    //   1: flowchart LR
    //   2: subgraph Outer
    //   3:   A --> B
    //   4: end
    let svg = render("flowchart LR\nsubgraph Outer\n  A --> B\nend\n");
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "subgraph opening at line 2 missing from SVG; got:\n{svg}"
    );
    // The closing `end` is at line 4. We must NOT attribute the subgraph to 4.
    // (Node B and edge are at line 3, so line 4 should not appear as a data-source-line.)
    assert!(
        !svg.contains(r#"data-source-line="4""#),
        "subgraph closing line 4 should not be attributed; got:\n{svg}"
    );
}

#[test]
fn flowchart_with_blank_and_comment_lines() {
    // Blank line and `%%` comment should not offset the line numbers
    // that reach the IR — the source lines remain 1-based against the
    // original input.
    //   1: flowchart LR
    //   2: (blank)
    //   3: %% this is a comment
    //   4: A --> B
    let svg = render("flowchart LR\n\n%% this is a comment\nA --> B\n");
    assert!(
        svg.contains(r#"data-source-line="4""#),
        "edge from original line 4 missing; SVG was:\n{svg}"
    );
    // Should NOT contain data-source-line="2" or "3" (those were blank/comment).
    assert!(!svg.contains(r#"data-source-line="2""#));
    assert!(!svg.contains(r#"data-source-line="3""#));
}

#[test]
fn flowchart_click_directive_carries_line() {
    //   1: flowchart LR
    //   2: A --> B
    //   3: click A "https://example.com"
    let svg = render("flowchart LR\nA --> B\nclick A \"https://example.com\"\n");
    // The click-generated <a> wrapper surfaces in SVG output; its
    // provenance lives on the containing Node group at line 2
    // (first-mention of A). The `click` line itself is recorded on
    // the NodeLink, but we don't currently emit link-specific
    // attributes -- that's a follow-up. This assertion just verifies
    // the flowchart still renders without losing the line-2 edge.
    assert!(svg.contains(r#"data-source-line="2""#));
}
