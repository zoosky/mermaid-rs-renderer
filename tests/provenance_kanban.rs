//! Integration tests for source-line provenance on kanban diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn kanban_card_and_column_carry_lines() {
    // Lines (1-based):
    //   1: kanban
    //   2: Todo
    //   3:   Task A
    //   4:   Task B
    //   5: Doing
    //   6:   Task C
    let svg = render("kanban\nTodo\n  Task A\n  Task B\nDoing\n  Task C\n");
    for line in [3, 4, 6] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "kanban card from line {line} missing; SVG was:\n{svg}"
        );
    }
}
