//! Integration tests for source-line provenance on gantt charts.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn gantt_task_carries_line() {
    // Lines (1-based):
    //   1: gantt
    //   2: title Project
    //   3: dateFormat YYYY-MM-DD
    //   4: section Phase 1
    //   5: Design: des1, 2024-01-01, 5d
    //   6: Build : bld1, after des1, 10d
    let svg = render(
        "gantt\ntitle Project\ndateFormat YYYY-MM-DD\nsection Phase 1\nDesign: des1, 2024-01-01, 5d\nBuild : bld1, after des1, 10d\n",
    );
    for line in [5, 6] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "gantt task from line {line} missing; SVG was:\n{svg}"
        );
    }
}
