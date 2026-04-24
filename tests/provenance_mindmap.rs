//! Integration tests for source-line provenance on mindmap diagrams.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn mindmap_nodes_carry_lines() {
    // Lines (1-based):
    //   1: mindmap
    //   2:   Root
    //   3:     Child1
    //   4:     Child2
    let svg = render("mindmap\n  Root\n    Child1\n    Child2\n");
    for line in [2, 3, 4] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "mindmap node from line {line} missing; SVG was:\n{svg}"
        );
    }
}

#[test]
fn mindmap_tree_edge_carries_child_line() {
    // Lines (1-based):
    //   1: mindmap
    //   2:   Root
    //   3:     Child1
    // The tree-connection edge Root -> Child1 was declared on the
    // child's line (3). The rendered edge <path> should carry
    // data-source-line="3".
    let svg = render("mindmap\n  Root\n    Child1\n");
    let path_with_prov = svg
        .split("<path ")
        .skip(1)
        .any(|seg| seg.contains(r#"data-source-line="3""#));
    assert!(
        path_with_prov,
        "mindmap edge <path> from line 3 missing data-source-line=\"3\"; SVG was:\n{svg}"
    );
}
