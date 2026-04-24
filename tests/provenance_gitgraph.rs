//! Integration tests for source-line provenance on gitGraph diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn gitgraph_commits_carry_lines() {
    // Lines (1-based):
    //   1: gitGraph
    //   2: commit
    //   3: commit
    //   4: branch feature
    //   5: commit
    let svg = render("gitGraph\ncommit\ncommit\nbranch feature\ncommit\n");
    for line in [2, 3, 5] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "gitgraph commit from line {line} missing; SVG was:\n{svg}"
        );
    }
}
