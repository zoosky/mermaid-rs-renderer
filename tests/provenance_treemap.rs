//! Integration tests for source-line provenance on treemap diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn treemap_nodes_carry_lines() {
    // Lines (1-based):
    //   1: treemap
    //   2: Root
    //   3:   Child1: 10
    //   4:   Child2: 20
    let svg = render("treemap\nRoot\n  Child1: 10\n  Child2: 20\n");
    for line in [2, 3, 4] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "treemap node from line {line} missing; SVG was:\n{svg}"
        );
    }
}
