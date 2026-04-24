//! Integration tests for source-line provenance on packet diagrams.
//!
//! Only compiled when the `source-provenance` cargo feature is on
//! (the default).

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn packet_fields_carry_lines() {
    // Lines (1-based):
    //   1: packet-beta
    //   2: 0-7: "Source Port"
    //   3: 8-15: "Destination Port"
    let svg = render("packet-beta\n0-7: \"Source Port\"\n8-15: \"Destination Port\"\n");
    for line in [2, 3] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "packet field from line {line} missing data-source-line; SVG was:\n{svg}"
        );
    }
}
