//! Integration tests for source-line provenance on ER diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn er_relation_carries_line() {
    // Lines:
    //   1: erDiagram
    //   2: CUSTOMER ||--o{ ORDER : places
    //   3: ORDER ||--|{ LINE-ITEM : contains
    let svg = render(
        "erDiagram\nCUSTOMER ||--o{ ORDER : places\nORDER ||--|{ LINE-ITEM : contains\n",
    );
    for line in [2, 3] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "ER relation from line {line} missing data-source-line; SVG was:\n{svg}"
        );
    }
}

#[test]
fn er_entity_declaration_carries_line() {
    // Lines:
    //   1: erDiagram
    //   2: CUSTOMER {
    //   3:   string name
    //   4: }
    let svg = render("erDiagram\nCUSTOMER {\n  string name\n}\n");
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "ER entity declaration from line 2 missing; SVG was:\n{svg}"
    );
}
