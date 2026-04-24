//! Integration tests for source-line provenance on XY chart SVG.
//!
//! Only compiled when the `source-provenance` cargo feature is on
//! (the default).

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn xychart_bar_series_carries_declaration_line() {
    // Lines (1-based):
    //   1: xychart-beta
    //   2:   x-axis [Q1, Q2, Q3]
    //   3:   y-axis "Units"
    //   4:   bar [10, 20, 30]
    let svg = render(
        "xychart-beta\n  x-axis [Q1, Q2, Q3]\n  y-axis \"Units\"\n  bar [10, 20, 30]\n",
    );
    assert!(
        svg.contains(r#"data-source-line="4""#),
        "xychart bar from line 4 missing data-source-line; SVG was:\n{svg}"
    );
}

#[test]
fn xychart_line_series_carries_declaration_line() {
    // Lines:
    //   1: xychart-beta
    //   2:   x-axis [Jan, Feb, Mar]
    //   3:   line [5, 10, 15]
    let svg = render("xychart-beta\n  x-axis [Jan, Feb, Mar]\n  line [5, 10, 15]\n");
    assert!(
        svg.contains(r#"data-source-line="3""#),
        "xychart line series from line 3 missing data-source-line; SVG was:\n{svg}"
    );
}
