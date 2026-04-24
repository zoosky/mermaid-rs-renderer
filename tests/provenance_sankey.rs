//! Integration tests for source-line provenance on sankey diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn sankey_link_carries_line() {
    // Lines:
    //   1: sankey-beta
    //   2: A,B,10
    //   3: B,C,5
    let svg = render("sankey-beta\nA,B,10\nB,C,5\n");
    for line in [2, 3] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "sankey link from line {line} missing; SVG was:\n{svg}"
        );
    }
}
