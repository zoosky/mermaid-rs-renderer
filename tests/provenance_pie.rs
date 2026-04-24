//! Integration tests for source-line provenance on pie chart SVG.
//!
//! Only compiled when the `source-provenance` cargo feature is on
//! (the default).

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn pie_slice_carries_declaration_line() {
    // Lines (1-based):
    //   1: pie title Breakdown
    //   2: "Alpha" : 40
    //   3: "Beta"  : 35
    //   4: "Gamma" : 25
    let svg = render("pie title Breakdown\n\"Alpha\" : 40\n\"Beta\" : 35\n\"Gamma\" : 25\n");
    for line in [2, 3, 4] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "pie slice from line {line} missing data-source-line; SVG was:\n{svg}"
        );
    }
}

#[test]
fn pie_single_slice_carries_line() {
    // Lines:
    //   1: pie
    //   2: "Only" : 100
    let svg = render("pie\n\"Only\" : 100\n");
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "single pie slice from line 2 missing data-source-line; SVG was:\n{svg}"
    );
}
