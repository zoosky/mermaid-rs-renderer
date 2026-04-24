//! Integration tests for source-line provenance on block diagrams.
//!
//! Block diagrams reuse standard Node / Edge IR + NodeLayout /
//! EdgeLayout, so source_loc propagates through the standard pipeline.
//! The block parser populates `Node.source_loc` and `Edge.source_loc`
//! at every construction site.

#![cfg(feature = "source-provenance")]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

#[test]
fn block_node_carries_declaration_line() {
    // Lines (1-based):
    //   1: block-beta
    //   2: columns 2
    //   3: A B
    //   4: C D
    let svg = render("block-beta\ncolumns 2\nA B\nC D\n");
    for line in [3, 4] {
        assert!(
            svg.contains(&format!(r#"data-source-line="{line}""#)),
            "block node from line {line} missing data-source-line; SVG was:\n{svg}"
        );
    }
}

#[test]
fn block_edge_carries_declaration_line() {
    // Lines:
    //   1: block-beta
    //   2: columns 2
    //   3: A
    //   4: B
    //   5: A --> B
    // Node lines are 3 and 4; edge line is 5. Checks that the edge's
    // <path> itself carries data-source-line="5" (not only the nodes).
    let svg = render("block-beta\ncolumns 2\nA\nB\nA --> B\n");
    // The edge <path> should carry line 5.
    let path_with_prov = svg
        .split("<path ")
        .skip(1)
        .any(|seg| seg.contains(r#"data-source-line="5""#));
    assert!(
        path_with_prov,
        "block edge <path> from line 5 missing data-source-line=\"5\"; SVG was:\n{svg}"
    );
}
