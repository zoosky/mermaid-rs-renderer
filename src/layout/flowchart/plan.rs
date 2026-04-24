#![allow(dead_code)]

//! Internal flowchart layout plan model.
//!
//! This module is the migration seam for the flowchart layout redesign.  The
//! first version mirrors facts already produced by the existing pipeline without
//! changing output.  Later phases can move bundle, route, and label decisions
//! into this model one stage at a time.

use std::collections::{BTreeMap, HashMap};

use crate::config::LayoutConfig;
use crate::ir::Graph;

use super::super::routing::{
    EdgePortInfo, EdgeSide, anchor_point_for_node, edge_pair_key, port_stub_length,
};
use super::super::{
    FLOWCHART_PORT_ROUTE_BIAS_MAX_RATIO, FLOWCHART_PORT_ROUTE_BIAS_RATIO, MULTI_EDGE_OFFSET_RATIO,
    NodeLayout, SubgraphLayout, TextBlock, anchor_layout_for_edge,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct BundleKey {
    pub(super) a: String,
    pub(super) b: String,
}

#[derive(Clone, Debug)]
pub(super) struct FlowchartLayoutPlan {
    pub(super) edge_count: usize,
    pub(super) ports: Vec<EdgePortPlan>,
    pub(super) bundles: Vec<EdgeBundlePlan>,
    pub(super) routes: Vec<EdgeRoutePlan>,
    pub(super) labels: Vec<EdgeLabelPlan>,
}

#[derive(Clone, Debug)]
pub(super) struct EdgePortPlan {
    pub(super) edge_idx: usize,
    pub(super) from: String,
    pub(super) to: String,
    pub(super) start_side: EdgeSide,
    pub(super) end_side: EdgeSide,
    pub(super) start_offset: f32,
    pub(super) end_offset: f32,
    pub(super) start_point: (f32, f32),
    pub(super) end_point: (f32, f32),
    pub(super) stub_len: f32,
}

#[derive(Clone, Debug)]
pub(super) struct EdgeBundlePlan {
    pub(super) key: BundleKey,
    pub(super) lanes: Vec<EdgeLanePlan>,
}

#[derive(Clone, Debug)]
pub(super) struct EdgeLanePlan {
    pub(super) edge_idx: usize,
    pub(super) lane_index: usize,
    pub(super) lane_count: usize,
    pub(super) base_offset: f32,
    pub(super) cross_edge_offset: f32,
    pub(super) effective_offset: f32,
}

#[derive(Clone, Debug)]
pub(super) struct EdgeRoutePlan {
    pub(super) edge_idx: usize,
    pub(super) points: Vec<(f32, f32)>,
    pub(super) approach_start: EdgeSide,
    pub(super) approach_end: EdgeSide,
}

#[derive(Clone, Debug)]
pub(super) struct EdgeLabelPlan {
    pub(super) edge_idx: usize,
    pub(super) label: Option<TextBlock>,
    pub(super) anchor: Option<(f32, f32)>,
    pub(super) reserved_center: Option<(f32, f32)>,
}

impl FlowchartLayoutPlan {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_current_pipeline(
        graph: &Graph,
        nodes: &BTreeMap<String, NodeLayout>,
        subgraphs: &[SubgraphLayout],
        edge_ports: &[EdgePortInfo],
        pair_counts: &HashMap<(String, String), usize>,
        pair_index: &[usize],
        cross_edge_offsets: &[f32],
        routed_points: &[Vec<(f32, f32)>],
        label_anchors: &[Option<(f32, f32)>],
        route_label_centers: &[Option<(f32, f32)>],
        edge_route_labels: &[Option<TextBlock>],
        config: &LayoutConfig,
    ) -> Self {
        let mut ports = Vec::with_capacity(graph.edges.len());
        let mut bundle_lanes: HashMap<BundleKey, Vec<EdgeLanePlan>> = HashMap::new();
        let mut routes = Vec::with_capacity(graph.edges.len());
        let mut labels = Vec::with_capacity(graph.edges.len());

        for (idx, edge) in graph.edges.iter().enumerate() {
            let Some(port_info) = edge_ports.get(idx).copied() else {
                continue;
            };
            let from_layout = nodes.get(&edge.from).expect("from node missing");
            let to_layout = nodes.get(&edge.to).expect("to node missing");
            let temp_from = from_layout.anchor_subgraph.and_then(|anchor_idx| {
                subgraphs
                    .get(anchor_idx)
                    .map(|sub| anchor_layout_for_edge(from_layout, sub, graph.direction, true))
            });
            let temp_to = to_layout.anchor_subgraph.and_then(|anchor_idx| {
                subgraphs
                    .get(anchor_idx)
                    .map(|sub| anchor_layout_for_edge(to_layout, sub, graph.direction, false))
            });
            let from = temp_from.as_ref().unwrap_or(from_layout);
            let to = temp_to.as_ref().unwrap_or(to_layout);
            let start_point =
                anchor_point_for_node(from, port_info.start_side, port_info.start_offset);
            let end_point = anchor_point_for_node(to, port_info.end_side, port_info.end_offset);
            let stub_len = port_stub_length(config, from, to);

            ports.push(EdgePortPlan {
                edge_idx: idx,
                from: edge.from.clone(),
                to: edge.to.clone(),
                start_side: port_info.start_side,
                end_side: port_info.end_side,
                start_offset: port_info.start_offset,
                end_offset: port_info.end_offset,
                start_point,
                end_point,
                stub_len,
            });

            let pair = edge_pair_key(edge);
            let total = *pair_counts.get(&pair).unwrap_or(&1);
            let lane_index = pair_index.get(idx).copied().unwrap_or_default();
            let base_offset = if total > 1 {
                (lane_index as f32 - (total as f32 - 1.0) / 2.0)
                    * (config.node_spacing * MULTI_EDGE_OFFSET_RATIO)
            } else {
                0.0
            };
            let cross_edge_offset = cross_edge_offsets.get(idx).copied().unwrap_or_default();
            let raw_bias =
                (port_info.start_offset - port_info.end_offset) * FLOWCHART_PORT_ROUTE_BIAS_RATIO;
            let max_bias = (config.node_spacing * FLOWCHART_PORT_ROUTE_BIAS_MAX_RATIO).max(8.0);
            let effective_offset =
                base_offset + cross_edge_offset + raw_bias.clamp(-max_bias, max_bias);
            bundle_lanes
                .entry(BundleKey {
                    a: pair.0,
                    b: pair.1,
                })
                .or_default()
                .push(EdgeLanePlan {
                    edge_idx: idx,
                    lane_index,
                    lane_count: total,
                    base_offset,
                    cross_edge_offset,
                    effective_offset,
                });

            routes.push(EdgeRoutePlan {
                edge_idx: idx,
                points: routed_points.get(idx).cloned().unwrap_or_default(),
                approach_start: port_info.start_side,
                approach_end: port_info.end_side,
            });

            labels.push(EdgeLabelPlan {
                edge_idx: idx,
                label: edge_route_labels.get(idx).cloned().unwrap_or_default(),
                anchor: label_anchors.get(idx).copied().unwrap_or_default(),
                reserved_center: route_label_centers.get(idx).copied().unwrap_or_default(),
            });
        }

        let mut bundles: Vec<EdgeBundlePlan> = bundle_lanes
            .into_iter()
            .map(|(key, mut lanes)| {
                lanes.sort_by_key(|lane| lane.edge_idx);
                EdgeBundlePlan { key, lanes }
            })
            .collect();
        bundles.sort_by(|a, b| a.key.a.cmp(&b.key.a).then_with(|| a.key.b.cmp(&b.key.b)));

        Self {
            edge_count: graph.edges.len(),
            ports,
            bundles,
            routes,
            labels,
        }
    }

    pub(super) fn is_consistent(&self) -> bool {
        self.ports.len() == self.edge_count
            && self.routes.len() == self.edge_count
            && self.labels.len() == self.edge_count
            && self
                .bundles
                .iter()
                .flat_map(|bundle| bundle.lanes.iter())
                .count()
                == self.edge_count
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use crate::config::LayoutConfig;
    use crate::ir::{Direction, Edge, EdgeStyle, Graph, NodeShape};
    use crate::layout::flowchart::plan::FlowchartLayoutPlan;
    use crate::layout::routing::{EdgePortInfo, EdgeSide};
    use crate::layout::{NodeLayout, TextBlock};

    fn node(id: &str, x: f32, y: f32) -> NodeLayout {
        NodeLayout {
            id: id.to_string(),
            x,
            y,
            width: 80.0,
            height: 40.0,
            label: TextBlock {
                lines: vec![id.to_string()],
                width: 10.0,
                height: 10.0,
            },
            shape: NodeShape::Rectangle,
            style: Default::default(),
            link: None,
            anchor_subgraph: None,
            hidden: false,
            icon: None,
        }
    }

    fn edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            label: Some("label".to_string()),
            start_label: None,
            end_label: None,
            directed: true,
            arrow_start: false,
            arrow_end: true,
            arrow_start_kind: None,
            arrow_end_kind: None,
            start_decoration: None,
            end_decoration: None,
            style: EdgeStyle::Solid,
        }
    }

    #[test]
    fn plan_groups_parallel_edges_into_bundle_lanes() {
        let mut graph = Graph::new();
        graph.direction = Direction::LeftRight;
        graph.edges.push(edge("A", "B"));
        graph.edges.push(edge("A", "B"));

        let mut nodes = BTreeMap::new();
        nodes.insert("A".to_string(), node("A", 0.0, 0.0));
        nodes.insert("B".to_string(), node("B", 160.0, 0.0));

        let ports = vec![
            EdgePortInfo {
                start_side: EdgeSide::Right,
                end_side: EdgeSide::Left,
                start_offset: -8.0,
                end_offset: -8.0,
            },
            EdgePortInfo {
                start_side: EdgeSide::Right,
                end_side: EdgeSide::Left,
                start_offset: 8.0,
                end_offset: 8.0,
            },
        ];
        let mut pair_counts = HashMap::new();
        pair_counts.insert(("A".to_string(), "B".to_string()), 2);
        let pair_index = vec![0, 1];
        let routed_points = vec![
            vec![(80.0, 12.0), (160.0, 12.0)],
            vec![(80.0, 28.0), (160.0, 28.0)],
        ];
        let labels = vec![None, None];
        let plan = FlowchartLayoutPlan::from_current_pipeline(
            &graph,
            &nodes,
            &[],
            &ports,
            &pair_counts,
            &pair_index,
            &[0.0, 0.0],
            &routed_points,
            &[Some((120.0, 12.0)), Some((120.0, 28.0))],
            &[None, None],
            &labels,
            &LayoutConfig::default(),
        );

        assert!(plan.is_consistent());
        assert_eq!(plan.bundles.len(), 1);
        assert_eq!(plan.bundles[0].lanes.len(), 2);
        assert!(
            plan.bundles[0].lanes[0].effective_offset < plan.bundles[0].lanes[1].effective_offset
        );
    }
}
