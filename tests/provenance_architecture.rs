//! Integration tests for source-line provenance on architecture diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn architecture_service_carries_line() {
    // Lines:
    //   1: architecture-beta
    //   2: group public
    //   3: service api(internet)[API] in public
    //   4: service db[Database]
    //   5: api --> db
    let svg = render(
        "architecture-beta\n\
         group public\n\
         service api(internet)[API] in public\n\
         service db[Database]\n\
         api --> db\n",
    );
    for line in [3, 4, 5] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "architecture element from line {line} missing; SVG was:\n{svg}"
        );
    }
}
