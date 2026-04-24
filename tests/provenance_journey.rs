//! Integration tests for source-line provenance on journey diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn journey_task_carries_line() {
    // Lines (1-based):
    //   1: journey
    //   2: title My journey
    //   3: section Morning
    //   4: Wake up: 3: Me
    //   5: Drink coffee: 5: Me
    let svg = render(
        "journey\ntitle My journey\nsection Morning\nWake up: 3: Me\nDrink coffee: 5: Me\n",
    );
    for line in [4, 5] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "journey task from line {line} missing; SVG was:\n{svg}"
        );
    }
}
