use std::collections::HashSet;
use std::fmt;

use crate::ir::DiagramKind;

use super::{DiagramData, Layout, TextBlock};

const EPS: f32 = 0.1;
const FLOWCHART_LABEL_ROUTE_CLEARANCE: f32 = 0.0;
const SEQUENCE_LABEL_LIFELINE_PAD_X: f32 = 4.0;
const SEQUENCE_LABEL_LIFELINE_PAD_Y: f32 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutInvariantError {
    pub path: String,
    pub message: String,
}

impl LayoutInvariantError {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for LayoutInvariantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Validate cross-cutting layout invariants that renderers rely on.
///
/// This is intentionally independent from tests so parser, CLI, fuzzers, and
/// future fallible layout APIs can share the same contract checks.
pub fn validate_layout_invariants(layout: &Layout) -> Result<(), Vec<LayoutInvariantError>> {
    let mut errors = Vec::new();

    check_finite_positive(&mut errors, "layout.width", layout.width);
    check_finite_positive(&mut errors, "layout.height", layout.height);

    for (id, node) in &layout.nodes {
        let path = format!("nodes[{id}]");
        check_rect(&mut errors, &path, node.x, node.y, node.width, node.height);
        check_text_block(&mut errors, &format!("{path}.label"), &node.label);
        if !node.hidden {
            check_inside_layout(
                &mut errors,
                &path,
                node.x,
                node.y,
                node.width,
                node.height,
                layout,
            );
        }
    }

    for (idx, subgraph) in layout.subgraphs.iter().enumerate() {
        let path = format!("subgraphs[{idx}:{}]", subgraph.label);
        check_rect(
            &mut errors,
            &path,
            subgraph.x,
            subgraph.y,
            subgraph.width,
            subgraph.height,
        );
        check_text_block(&mut errors, &format!("{path}.label"), &subgraph.label_block);
        check_inside_layout(
            &mut errors,
            &path,
            subgraph.x,
            subgraph.y,
            subgraph.width,
            subgraph.height,
            layout,
        );
    }

    for (idx, edge) in layout.edges.iter().enumerate() {
        let path = format!("edges[{idx}:{}->{}]", edge.from, edge.to);
        for (point_idx, (x, y)) in edge.points.iter().copied().enumerate() {
            check_finite(&mut errors, &format!("{path}.points[{point_idx}].x"), x);
            check_finite(&mut errors, &format!("{path}.points[{point_idx}].y"), y);
        }
        if let Some(label) = &edge.label {
            check_text_block(&mut errors, &format!("{path}.label"), label);
            check_anchor(
                &mut errors,
                &format!("{path}.label_anchor"),
                edge.label_anchor,
            );
        }
        if let Some(label) = &edge.start_label {
            check_text_block(&mut errors, &format!("{path}.start_label"), label);
            check_anchor(
                &mut errors,
                &format!("{path}.start_label_anchor"),
                edge.start_label_anchor,
            );
        }
        if let Some(label) = &edge.end_label {
            check_text_block(&mut errors, &format!("{path}.end_label"), label);
            check_anchor(
                &mut errors,
                &format!("{path}.end_label_anchor"),
                edge.end_label_anchor,
            );
        }
    }

    match &layout.diagram {
        DiagramData::Graph { state_notes } => {
            for (idx, note) in state_notes.iter().enumerate() {
                check_rect(
                    &mut errors,
                    &format!("state_notes[{idx}]"),
                    note.x,
                    note.y,
                    note.width,
                    note.height,
                );
            }
        }
        DiagramData::Sequence(seq) => {
            for lifeline in &seq.lifelines {
                check_finite(
                    &mut errors,
                    &format!("lifelines[{}].x", lifeline.id),
                    lifeline.x,
                );
                check_finite(
                    &mut errors,
                    &format!("lifelines[{}].y1", lifeline.id),
                    lifeline.y1,
                );
                check_finite(
                    &mut errors,
                    &format!("lifelines[{}].y2", lifeline.id),
                    lifeline.y2,
                );
            }
        }
        _ => {}
    }

    if layout.kind == DiagramKind::Flowchart {
        validate_flowchart_invariants(layout, &mut errors);
    }
    validate_sequence_invariants(layout, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_flowchart_invariants(layout: &Layout, errors: &mut Vec<LayoutInvariantError>) {
    for (idx, left) in layout.subgraphs.iter().enumerate() {
        let left_nodes: HashSet<&str> = left.nodes.iter().map(String::as_str).collect();
        for right in layout.subgraphs.iter().skip(idx + 1) {
            if right
                .nodes
                .iter()
                .any(|node| left_nodes.contains(node.as_str()))
            {
                continue;
            }
            if rects_overlap(
                (left.x, left.y, left.width, left.height),
                (right.x, right.y, right.width, right.height),
            ) {
                errors.push(LayoutInvariantError::new(
                    "subgraphs",
                    format!(
                        "flowchart subgraphs '{}' and '{}' overlap",
                        left.label, right.label
                    ),
                ));
            }
        }
    }

    for (edge_idx, edge) in layout.edges.iter().enumerate() {
        let edge_path = format!("edges[{edge_idx}:{}->{}]", edge.from, edge.to);
        if let (Some(label), Some(anchor)) = (&edge.label, edge.label_anchor) {
            let rect = centered_text_rect(anchor, label, FLOWCHART_LABEL_ROUTE_CLEARANCE);
            if path_intersects_rect(&edge.points, rect) {
                errors.push(LayoutInvariantError::new(
                    format!("{edge_path}.label"),
                    "route overlaps its own center label box",
                ));
            }
        }

        for (node_id, node) in &layout.nodes {
            if node.hidden || node_id == &edge.from || node_id == &edge.to {
                continue;
            }
            let rect = (node.x, node.y, node.width, node.height);
            if path_intersects_rect(&edge.points, rect) {
                errors.push(LayoutInvariantError::new(
                    edge_path.clone(),
                    format!("route intersects non-endpoint node '{node_id}'"),
                ));
            }
        }
    }
}

fn validate_sequence_invariants(layout: &Layout, errors: &mut Vec<LayoutInvariantError>) {
    let DiagramData::Sequence(seq) = &layout.diagram else {
        return;
    };
    for (edge_idx, edge) in layout.edges.iter().enumerate() {
        let (Some(label), Some(anchor)) = (&edge.label, edge.label_anchor) else {
            continue;
        };
        let label_rect = centered_text_rect(
            anchor,
            label,
            SEQUENCE_LABEL_LIFELINE_PAD_X.max(SEQUENCE_LABEL_LIFELINE_PAD_Y),
        );
        for lifeline in &seq.lifelines {
            if lifeline.id == edge.from || lifeline.id == edge.to {
                continue;
            }
            let line_rect = (
                lifeline.x - 1.5,
                lifeline.y1,
                3.0,
                lifeline.y2 - lifeline.y1,
            );
            if rects_overlap(label_rect, line_rect) {
                errors.push(LayoutInvariantError::new(
                    format!("edges[{edge_idx}:{}->{}].label", edge.from, edge.to),
                    format!("sequence label overlaps lifeline '{}'", lifeline.id),
                ));
            }
        }
    }
}

fn check_rect(
    errors: &mut Vec<LayoutInvariantError>,
    path: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    check_finite(errors, &format!("{path}.x"), x);
    check_finite(errors, &format!("{path}.y"), y);
    check_finite(errors, &format!("{path}.width"), width);
    check_finite(errors, &format!("{path}.height"), height);
    if width < 0.0 {
        errors.push(LayoutInvariantError::new(
            format!("{path}.width"),
            "must be non-negative",
        ));
    }
    if height < 0.0 {
        errors.push(LayoutInvariantError::new(
            format!("{path}.height"),
            "must be non-negative",
        ));
    }
}

fn check_inside_layout(
    errors: &mut Vec<LayoutInvariantError>,
    path: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    layout: &Layout,
) {
    if x < -EPS || y < -EPS || x + width > layout.width + EPS || y + height > layout.height + EPS {
        errors.push(LayoutInvariantError::new(
            path,
            "rectangle exceeds layout bounds",
        ));
    }
}

fn check_text_block(errors: &mut Vec<LayoutInvariantError>, path: &str, text: &TextBlock) {
    check_finite(errors, &format!("{path}.width"), text.width);
    check_finite(errors, &format!("{path}.height"), text.height);
    if text.width < 0.0 {
        errors.push(LayoutInvariantError::new(
            format!("{path}.width"),
            "must be non-negative",
        ));
    }
    if text.height < 0.0 {
        errors.push(LayoutInvariantError::new(
            format!("{path}.height"),
            "must be non-negative",
        ));
    }
}

fn check_anchor(errors: &mut Vec<LayoutInvariantError>, path: &str, anchor: Option<(f32, f32)>) {
    match anchor {
        Some((x, y)) => {
            check_finite(errors, &format!("{path}.x"), x);
            check_finite(errors, &format!("{path}.y"), y);
        }
        None => errors.push(LayoutInvariantError::new(
            path,
            "label is missing its anchor",
        )),
    }
}

fn check_finite_positive(errors: &mut Vec<LayoutInvariantError>, path: &str, value: f32) {
    check_finite(errors, path, value);
    if value <= 0.0 {
        errors.push(LayoutInvariantError::new(path, "must be positive"));
    }
}

fn check_finite(errors: &mut Vec<LayoutInvariantError>, path: &str, value: f32) {
    if !value.is_finite() {
        errors.push(LayoutInvariantError::new(path, "must be finite"));
    }
}

fn centered_text_rect(anchor: (f32, f32), text: &TextBlock, pad: f32) -> (f32, f32, f32, f32) {
    (
        anchor.0 - text.width / 2.0 - pad,
        anchor.1 - text.height / 2.0 - pad,
        text.width + pad * 2.0,
        text.height + pad * 2.0,
    )
}

fn path_intersects_rect(points: &[(f32, f32)], rect: (f32, f32, f32, f32)) -> bool {
    points
        .windows(2)
        .any(|segment| segment_intersects_rect(segment[0], segment[1], rect))
}

fn rects_overlap(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

fn segment_intersects_rect(a: (f32, f32), b: (f32, f32), rect: (f32, f32, f32, f32)) -> bool {
    let (rx, ry, rw, rh) = rect;
    if rw <= 0.0 || rh <= 0.0 {
        return false;
    }
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let p = [-dx, dx, -dy, dy];
    let q = [a.0 - rx, rx + rw - a.0, a.1 - ry, ry + rh - a.1];
    let mut u1 = 0.0f32;
    let mut u2 = 1.0f32;

    for (pi, qi) in p.into_iter().zip(q) {
        if pi.abs() <= f32::EPSILON {
            if qi < 0.0 {
                return false;
            }
            continue;
        }
        let t = qi / pi;
        if pi < 0.0 {
            if t > u2 {
                return false;
            }
            if t > u1 {
                u1 = t;
            }
        } else {
            if t < u1 {
                return false;
            }
            if t < u2 {
                u2 = t;
            }
        }
    }

    true
}
