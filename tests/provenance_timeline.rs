//! Integration tests for source-line provenance on timeline SVG.
//!
//! Only compiled when the `source-provenance` cargo feature is on
//! (the default).

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn timeline_event_carries_declaration_line() {
    // Lines (1-based):
    //   1: timeline
    //   2: title History
    //   3: 2000 : Event A
    //   4: 2010 : Event B
    let svg = render("timeline\ntitle History\n2000 : Event A\n2010 : Event B\n");
    for line in [3, 4] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "timeline event from line {line} missing data-source-line; SVG was:\n{svg}"
        );
    }
}

#[test]
fn timeline_event_with_multiple_descriptions_uses_opening_line() {
    // Lines:
    //   1: timeline
    //   2: 2020 : Alpha : Beta : Gamma
    let svg = render("timeline\n2020 : Alpha : Beta : Gamma\n");
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "timeline event from line 2 missing data-source-line; SVG was:\n{svg}"
    );
}
