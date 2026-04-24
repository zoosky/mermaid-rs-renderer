//! Integration tests for source-line provenance on class-diagram SVG.
//!
//! Class diagrams reuse the standard Node / Edge IR + NodeLayout /
//! EdgeLayout pipeline. The class-diagram parser populates source_loc
//! at every ensure_node / edges.push site.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn class_relation_carries_line() {
    // Lines (1-based):
    //   1: classDiagram
    //   2: Animal <|-- Duck
    //   3: Animal <|-- Fish
    let svg = render("classDiagram\nAnimal <|-- Duck\nAnimal <|-- Fish\n");
    for line in [2, 3] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "class relation from line {line} missing data-source-line; SVG was:\n{svg}"
        );
    }
}

#[test]
fn class_declaration_carries_line() {
    // Lines:
    //   1: classDiagram
    //   2: class Animal
    //   3: Animal <|-- Duck
    let svg = render("classDiagram\nclass Animal\nAnimal <|-- Duck\n");
    assert!(
        svg.contains(r#"data-source-line="2""#),
        "class declaration from line 2 missing data-source-line; SVG was:\n{svg}"
    );
}
