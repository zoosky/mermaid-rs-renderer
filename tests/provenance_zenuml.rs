//! Integration tests for source-line provenance on ZenUML diagrams.
//!
//! Only compiled when the `source-provenance` cargo feature is on
//! (the default).

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::layout;
use mermaid_rs_renderer::{RenderOptions, parse_mermaid, render_svg};

fn render(input: &str) -> String {
    let parsed = parse_mermaid(input).expect("parse should succeed");
    let options = RenderOptions::default();
    let lay = layout::compute_layout(&parsed.graph, &options.theme, &options.layout);
    render_svg(&lay, &options.theme, &options.layout)
}

#[test]
fn zenuml_message_carries_line() {
    // Lines (1-based):
    //   1: zenuml
    //   2: A -> B: hello
    //   3: B -> C: forward
    let svg = render("zenuml\nA -> B: hello\nB -> C: forward\n");
    for line in [2, 3] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "zenuml message from line {line} missing data-source-line; SVG was:\n{svg}"
        );
    }
}
