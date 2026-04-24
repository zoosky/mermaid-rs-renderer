//! Integration tests for source-line provenance on state-diagram SVG.
//! See `provenance_flowchart.rs` for the feature-gate pattern.
//!
//! Coverage:
//! - State Node carries the line it was first mentioned on
//! - State transition Edge carries the line it was declared on
//! - Composite state Subgraph uses the opening `state X {` line,
//!   not the closing `}`

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn state_transition_carries_line() {
    //   1: stateDiagram-v2
    //   2: Idle --> Active
    //   3: Active --> Idle
    let svg = render("stateDiagram-v2\nIdle --> Active\nActive --> Idle\n");
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "transition at line 2 missing; SVG was:\n{svg}"
    );
    assert!(
        svg.contains(r#"data-source-line="3""#),
        "transition at line 3 missing; SVG was:\n{svg}"
    );
}

#[test]
fn state_alias_carries_line() {
    //   1: stateDiagram-v2
    //   2: state "Warming up" as Warm
    //   3: Warm --> Running
    let svg = render(
        "stateDiagram-v2\nstate \"Warming up\" as Warm\nWarm --> Running\n",
    );
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "state alias at line 2 missing; SVG was:\n{svg}"
    );
}

#[test]
fn state_composite_uses_opening_line() {
    //   1: stateDiagram-v2
    //   2: state Outer {
    //   3:   Idle --> Active
    //   4: }
    let svg = render("stateDiagram-v2\nstate Outer {\n  Idle --> Active\n}\n");
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "composite subgraph opening at line 2 missing; got:\n{svg}"
    );
    // Closing `}` is at line 4; must not be attributed.
    assert!(!svg.contains(r#"data-source-line="4""#));
}

#[test]
fn state_description_line_carries_line() {
    //   1: stateDiagram-v2
    //   2: Active : Processing request
    //   3: Idle --> Active
    let svg = render(
        "stateDiagram-v2\nActive : Processing request\nIdle --> Active\n",
    );
    // Active is first described on line 2, so its node should carry line 2.
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "state description at line 2 missing; SVG was:\n{svg}"
    );
}

#[test]
fn state_start_end_markers_do_not_leak_synthetic_lines() {
    //   1: stateDiagram-v2
    //   2: [*] --> Idle
    //   3: Idle --> [*]
    let svg = render("stateDiagram-v2\n[*] --> Idle\nIdle --> [*]\n");
    // Lines 2 and 3 should be present.
    assert!(svg.contains(r#"data-source-line="2""#));
    assert!(svg.contains(r#"data-source-line="3""#));
}
