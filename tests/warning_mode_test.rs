//! Integration tests for Feature f160f (lenient render mode).
//!
//! Verifies the contract of `render_with_mode` with
//! `RenderMode::Lenient` against fixtures in
//! `tests/fixtures/warning/`: malformed inputs produce a complete
//! inline SVG warning box, while the same inputs in strict mode
//! still return `Err`.

use mermaid_rs_renderer::{
    RenderMode, RenderOptions, render_warning_box, render_with_mode, render_with_options,
};
use std::fs;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/warning");
    path.push(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn lenient() -> RenderOptions {
    RenderOptions::default().with_mode(RenderMode::Lenient)
}

#[test]
fn default_mode_is_strict() {
    assert_eq!(RenderOptions::default().mode, RenderMode::Strict);
    assert_eq!(RenderMode::default(), RenderMode::Strict);
}

#[test]
fn lenient_malformed_flowchart_returns_ok_svg() {
    let input = fixture("malformed-flowchart.mmd");
    let svg = render_with_mode(&input, lenient()).expect("lenient returned Err");
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("data-kind=\"warning-box\""));
    assert!(svg.contains("class=\"mermaid-warning\""));
}

#[test]
fn strict_malformed_flowchart_returns_err() {
    let input = fixture("malformed-flowchart.mmd");
    assert!(render_with_options(&input, RenderOptions::default()).is_err());
}

#[test]
fn lenient_warning_box_includes_error_text() {
    let input = fixture("malformed-flowchart.mmd");
    let svg = render_with_mode(&input, lenient()).unwrap();
    // Leading arrow is caught by the validator as UnexpectedToken.
    assert!(svg.contains("unexpected token"));
}

#[test]
fn lenient_warning_box_includes_full_source() {
    let input = fixture("malformed-flowchart.mmd");
    let svg = render_with_mode(&input, lenient()).unwrap();
    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // Every non-empty source line should appear in the SVG text
        // content (XML-escaped for special chars; plain for the
        // simple fixture lines used here).
        let escaped = line
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        assert!(
            svg.contains(&escaped),
            "warning SVG missing source line: {line:?}\nsvg: {svg}"
        );
    }
}

#[test]
fn lenient_warning_box_has_highlight_rect_for_error_line() {
    // Leading arrow is on line 2, so the highlight rect should
    // appear somewhere within the SVG.
    let input = fixture("malformed-flowchart.mmd");
    let svg = render_with_mode(&input, lenient()).unwrap();
    assert!(svg.contains("class=\"mermaid-warning-highlight\""));
}

#[test]
fn lenient_warning_box_references_css_variables_with_fallbacks() {
    let input = fixture("malformed-flowchart.mmd");
    let svg = render_with_mode(&input, lenient()).unwrap();
    assert!(svg.contains("var(--mermaid-warning-bg,"));
    assert!(svg.contains("var(--mermaid-warning-border,"));
    assert!(svg.contains("var(--mermaid-warning-highlight,"));
}

#[test]
fn lenient_unclosed_subgraph_box_names_opening_line() {
    let input = fixture("unclosed-subgraph.mmd");
    let svg = render_with_mode(&input, lenient()).unwrap();
    // UnclosedSubgraph::Display -> "unclosed subgraph opened at line 2".
    assert!(
        svg.contains("unclosed subgraph"),
        "expected unclosed-subgraph message in SVG"
    );
    assert!(svg.contains("line 2"), "expected opening line 2 reference");
}

#[test]
fn strict_unclosed_subgraph_still_errors() {
    let input = fixture("unclosed-subgraph.mmd");
    assert!(render_with_options(&input, RenderOptions::default()).is_err());
}

#[test]
fn lenient_mode_does_not_error_on_valid_utf8_input() {
    // Arbitrary garbage that is still valid UTF-8 should not
    // propagate an error under lenient mode -- either it parses
    // successfully (unlikely) or it renders a warning box.
    let inputs = [
        "",
        "\n\n",
        "notADiagramType something",
        "flowchart LR\n--> stray\n",
    ];
    for input in inputs {
        let got = render_with_mode(input, lenient());
        assert!(got.is_ok(), "lenient errored on {input:?}: {:?}", got.err());
    }
}

#[test]
fn strict_mode_preserves_existing_render_behaviour_for_valid_input() {
    // Existing callers that use the default `render_with_options`
    // must continue to get SVG out for a valid input.
    let svg = render_with_options("flowchart LR; A --> B", RenderOptions::default())
        .expect("strict rendering");
    assert!(svg.contains("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(!svg.contains("data-kind=\"warning-box\""));
}

#[test]
fn render_warning_box_directly_works_with_synthetic_error() {
    use mermaid_rs_renderer::ParseError;

    let err = ParseError::UnknownParticipant {
        name: "Aliec".to_string(),
        line: 4,
        candidates: vec!["Alice".to_string()],
    };
    let svg = render_warning_box(
        "sequenceDiagram\n  participant Alice\n  participant Bob\n  Aliec->>Bob: hi\n",
        &err,
        &RenderOptions::default(),
    );
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("data-kind=\"warning-box\""));
    // Suggestion text is XML-escaped in the output.
    assert!(svg.contains("&apos;Alice&apos;"));
    assert!(svg.contains("Suggestion:"));
    assert!(svg.contains("line 4"));
}
