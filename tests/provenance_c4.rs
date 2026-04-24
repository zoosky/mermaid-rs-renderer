//! Integration tests for source-line provenance on C4 diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn c4_shape_carries_line() {
    // Lines (1-based):
    //   1: C4Context
    //   2: Person(user, "Customer")
    //   3: System(api, "API")
    //   4: Rel(user, api, "uses")
    let svg = render(
        "C4Context\nPerson(user, \"Customer\")\nSystem(api, \"API\")\nRel(user, api, \"uses\")\n",
    );
    for line in [2, 3, 4] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "c4 element from line {line} missing; SVG was:\n{svg}"
        );
    }
}
