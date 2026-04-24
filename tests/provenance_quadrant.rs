//! Integration tests for source-line provenance on quadrant chart SVG.
//!
//! Only compiled when the `source-provenance` cargo feature is on
//! (the default).

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn quadrant_point_carries_declaration_line() {
    // Lines (1-based):
    //   1: quadrantChart
    //   2: title Reach and engagement
    //   3: x-axis Low Reach --> High Reach
    //   4: y-axis Low Engagement --> High Engagement
    //   5: Campaign A: [0.3, 0.6]
    //   6: Campaign B: [0.45, 0.23]
    let svg = render(
        "quadrantChart\n\
         title Reach and engagement\n\
         x-axis Low Reach --> High Reach\n\
         y-axis Low Engagement --> High Engagement\n\
         Campaign A: [0.3, 0.6]\n\
         Campaign B: [0.45, 0.23]\n",
    );
    for line in [5, 6] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "quadrant point from line {line} missing data-source-line; SVG was:\n{svg}"
        );
    }
}
