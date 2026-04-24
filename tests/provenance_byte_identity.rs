//! Regression test for the byte-identity-when-feature-off contract.
//!
//! When `source-provenance` is disabled, rendered SVG must not contain
//! any `data-source-line` or `data-source-col` attribute — nor any
//! extra `<g>`/`</g>` wrappers introduced purely for carrying those
//! attributes. This file only runs in the feature-off configuration.

#![cfg(not(feature = "source-provenance"))]

use mermaid_rs_renderer::{RenderOptions, render_with_options};

fn render(input: &str) -> String {
    render_with_options(input, RenderOptions::default()).expect("render should succeed")
}

fn assert_no_prov(svg: &str, tag: &str) {
    assert!(
        !svg.contains("data-source-line"),
        "{tag} SVG leaked data-source-line when feature was off:\n{svg}"
    );
    assert!(
        !svg.contains("data-source-col"),
        "{tag} SVG leaked data-source-col when feature was off:\n{svg}"
    );
}

#[test]
fn flowchart_no_attrs_feature_off() {
    assert_no_prov(&render("flowchart LR\nA --> B\n"), "flowchart");
}

#[test]
fn pie_no_attrs_feature_off() {
    assert_no_prov(&render("pie\n\"A\" : 40\n\"B\" : 60\n"), "pie");
}

#[test]
fn state_no_attrs_feature_off() {
    assert_no_prov(
        &render("stateDiagram-v2\nnote right of S : hi\nS --> T\n"),
        "state",
    );
}

#[test]
fn gantt_no_attrs_feature_off() {
    assert_no_prov(
        &render("gantt\ntitle P\ndateFormat YYYY-MM-DD\nTask: t, 2024-01-01, 5d\n"),
        "gantt",
    );
}

#[test]
fn gitgraph_no_attrs_feature_off() {
    assert_no_prov(&render("gitGraph\ncommit\ncommit\n"), "gitgraph");
}

#[test]
fn timeline_no_attrs_feature_off() {
    assert_no_prov(&render("timeline\n2020 : A\n2021 : B\n"), "timeline");
}

#[test]
fn journey_no_attrs_feature_off() {
    assert_no_prov(
        &render("journey\nsection Morning\nWake: 3: Me\n"),
        "journey",
    );
}

#[test]
fn xychart_no_attrs_feature_off() {
    assert_no_prov(
        &render("xychart-beta\nx-axis [A, B]\nbar [1, 2]\nline [3, 4]\n"),
        "xychart",
    );
}

#[test]
fn quadrant_no_attrs_feature_off() {
    assert_no_prov(
        &render("quadrantChart\nPoint A: [0.3, 0.5]\n"),
        "quadrant",
    );
}

#[test]
fn architecture_no_attrs_feature_off() {
    assert_no_prov(
        &render(
            "architecture-beta\ngroup pub\nservice api[API] in pub\nservice db[DB]\napi --> db\n",
        ),
        "architecture",
    );
}
