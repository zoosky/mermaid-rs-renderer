//! Integration tests for source-line provenance on requirement diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn requirement_relation_carries_line() {
    // Lines (1-based):
    //   1: requirementDiagram
    //   2: requirement test_req {
    //   3:   id: 1
    //   4: }
    //   5: element test_entity {
    //   6:   type: simulation
    //   7: }
    //   8: test_entity - satisfies -> test_req
    let svg = render(
        "requirementDiagram\nrequirement test_req {\n  id: 1\n}\nelement test_entity {\n  type: simulation\n}\ntest_entity - satisfies -> test_req\n",
    );
    assert!(
        svg.contains(r#"data-source-line="8""#),
        "requirement relation from line 8 missing; SVG was:\n{svg}"
    );
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "requirement block header from line 2 missing; SVG was:\n{svg}"
    );
}
