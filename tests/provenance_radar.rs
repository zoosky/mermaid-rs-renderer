//! Integration tests for source-line provenance on radar charts.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn radar_curve_carries_line() {
    // Lines (1-based):
    //   1: radar
    //   2: axis Strength, Speed, Stamina
    //   3: curve Hero {80, 60, 90}
    let svg = render("radar\naxis Strength, Speed, Stamina\ncurve Hero {80, 60, 90}\n");
    assert!(
        svg.contains(r#"data-source-line="3""#),
        "radar curve from line 3 missing; SVG was:\n{svg}"
    );
}
