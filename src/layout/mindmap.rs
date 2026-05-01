use super::*;

mod tidy_tree {
    /// Port of the `non-layered-tidy-tree-layout` algorithm by A. J. van der Ploeg /
    /// J. van Roosmalen used by Mermaid JS for the `tidy-tree` mindmap layout.
    /// Indices into `Arena::nodes` are used in place of the original linked-list
    /// pointers so the structure can live in a single vector.
    pub struct TidyNode {
        pub w: f32,
        pub h: f32,
        pub y: f32,
        pub children: Vec<usize>,
        pub x: f32,
        prelim: f32,
        modf: f32,
        shift: f32,
        change: f32,
        tl: Option<usize>,
        tr: Option<usize>,
        el: Option<usize>,
        er: Option<usize>,
        msel: f32,
        mser: f32,
    }

    pub struct Arena {
        pub nodes: Vec<TidyNode>,
    }

    impl Arena {
        pub fn new() -> Self {
            Self { nodes: Vec::new() }
        }

        pub fn alloc(&mut self, w: f32, h: f32, y: f32, children: Vec<usize>) -> usize {
            let idx = self.nodes.len();
            self.nodes.push(TidyNode {
                w,
                h,
                y,
                children,
                x: 0.0,
                prelim: 0.0,
                modf: 0.0,
                shift: 0.0,
                change: 0.0,
                tl: None,
                tr: None,
                el: None,
                er: None,
                msel: 0.0,
                mser: 0.0,
            });
            idx
        }
    }

    fn bottom(arena: &Arena, t: usize) -> f32 {
        arena.nodes[t].y + arena.nodes[t].h
    }

    fn set_extremes(arena: &mut Arena, t: usize) {
        let cs = arena.nodes[t].children.len();
        if cs == 0 {
            arena.nodes[t].el = Some(t);
            arena.nodes[t].er = Some(t);
            arena.nodes[t].msel = 0.0;
            arena.nodes[t].mser = 0.0;
        } else {
            let first = arena.nodes[t].children[0];
            let last = arena.nodes[t].children[cs - 1];
            arena.nodes[t].el = arena.nodes[first].el;
            arena.nodes[t].msel = arena.nodes[first].msel;
            arena.nodes[t].er = arena.nodes[last].er;
            arena.nodes[t].mser = arena.nodes[last].mser;
        }
    }

    fn next_left_contour(arena: &Arena, t: usize) -> Option<usize> {
        if arena.nodes[t].children.is_empty() {
            arena.nodes[t].tl
        } else {
            Some(arena.nodes[t].children[0])
        }
    }

    fn next_right_contour(arena: &Arena, t: usize) -> Option<usize> {
        let cs = arena.nodes[t].children.len();
        if cs == 0 {
            arena.nodes[t].tr
        } else {
            Some(arena.nodes[t].children[cs - 1])
        }
    }

    fn distribute_extra(arena: &mut Arena, t: usize, i: usize, si: usize, distance: f32) {
        if si + 1 != i {
            let nr = (i - si) as f32;
            let mid = arena.nodes[t].children[si + 1];
            let target = arena.nodes[t].children[i];
            arena.nodes[mid].shift += distance / nr;
            arena.nodes[target].shift -= distance / nr;
            arena.nodes[target].change -= distance - distance / nr;
        }
    }

    fn move_subtree(arena: &mut Arena, t: usize, i: usize, si: usize, distance: f32) {
        let child = arena.nodes[t].children[i];
        arena.nodes[child].modf += distance;
        arena.nodes[child].msel += distance;
        arena.nodes[child].mser += distance;
        distribute_extra(arena, t, i, si, distance);
    }

    fn set_left_thread(arena: &mut Arena, t: usize, i: usize, cl: usize, modsumcl: f32) {
        let first = arena.nodes[t].children[0];
        let li = arena.nodes[first].el.unwrap();
        arena.nodes[li].tl = Some(cl);
        let diff = (modsumcl - arena.nodes[cl].modf) - arena.nodes[first].msel;
        arena.nodes[li].modf += diff;
        arena.nodes[li].prelim -= diff;
        let target = arena.nodes[t].children[i];
        arena.nodes[first].el = arena.nodes[target].el;
        arena.nodes[first].msel = arena.nodes[target].msel;
    }

    fn set_right_thread(arena: &mut Arena, t: usize, i: usize, sr: usize, modsumsr: f32) {
        let cur = arena.nodes[t].children[i];
        let prev = arena.nodes[t].children[i - 1];
        let ri = arena.nodes[cur].er.unwrap();
        arena.nodes[ri].tr = Some(sr);
        let diff = (modsumsr - arena.nodes[sr].modf) - arena.nodes[cur].mser;
        arena.nodes[ri].modf += diff;
        arena.nodes[ri].prelim -= diff;
        arena.nodes[cur].er = arena.nodes[prev].er;
        arena.nodes[cur].mser = arena.nodes[prev].mser;
    }

    fn separate(arena: &mut Arena, t: usize, i: usize, ih_stack: &mut Vec<(f32, usize)>) {
        let mut sr = Some(arena.nodes[t].children[i - 1]);
        let mut mssr = arena.nodes[sr.unwrap()].modf;
        let mut cl = Some(arena.nodes[t].children[i]);
        let mut mscl = arena.nodes[cl.unwrap()].modf;
        while let (Some(srv), Some(clv)) = (sr, cl) {
            if bottom(arena, srv) > ih_stack.last().unwrap().0 {
                ih_stack.pop();
            }
            let distance =
                mssr + arena.nodes[srv].prelim + arena.nodes[srv].w
                    - (mscl + arena.nodes[clv].prelim);
            if distance > 0.0 {
                mscl += distance;
                let ih_index = ih_stack.last().unwrap().1;
                move_subtree(arena, t, i, ih_index, distance);
            }
            let sy = bottom(arena, srv);
            let cy = bottom(arena, clv);
            if sy <= cy {
                sr = next_right_contour(arena, srv);
                if let Some(s) = sr {
                    mssr += arena.nodes[s].modf;
                }
            }
            if sy >= cy {
                cl = next_left_contour(arena, clv);
                if let Some(c) = cl {
                    mscl += arena.nodes[c].modf;
                }
            }
        }
        if sr.is_none() && cl.is_some() {
            set_left_thread(arena, t, i, cl.unwrap(), mscl);
        } else if sr.is_some() && cl.is_none() {
            set_right_thread(arena, t, i, sr.unwrap(), mssr);
        }
    }

    fn position_root(arena: &mut Arena, t: usize) {
        let cs = arena.nodes[t].children.len();
        let first = arena.nodes[t].children[0];
        let last = arena.nodes[t].children[cs - 1];
        let prelim = (arena.nodes[first].prelim
            + arena.nodes[first].modf
            + arena.nodes[last].modf
            + arena.nodes[last].prelim
            + arena.nodes[last].w)
            / 2.0
            - arena.nodes[t].w / 2.0;
        arena.nodes[t].prelim = prelim;
    }

    fn first_walk(arena: &mut Arena, t: usize) {
        let cs = arena.nodes[t].children.len();
        if cs == 0 {
            set_extremes(arena, t);
            return;
        }
        let first = arena.nodes[t].children[0];
        first_walk(arena, first);
        let mut ih_stack: Vec<(f32, usize)> = Vec::new();
        let low_y = bottom(arena, arena.nodes[first].el.unwrap());
        ih_stack.push((low_y, 0));
        for i in 1..cs {
            let child = arena.nodes[t].children[i];
            first_walk(arena, child);
            let er = arena.nodes[child].er.unwrap();
            let min_y = bottom(arena, er);
            separate(arena, t, i, &mut ih_stack);
            // updateIYL: pop hidden then push (min_y, i)
            while let Some(&(low_y, _)) = ih_stack.last() {
                if min_y >= low_y {
                    ih_stack.pop();
                } else {
                    break;
                }
            }
            ih_stack.push((min_y, i));
        }
        position_root(arena, t);
        set_extremes(arena, t);
    }

    fn add_child_spacing(arena: &mut Arena, t: usize) {
        let cs = arena.nodes[t].children.len();
        let mut d = 0.0;
        let mut modsumdelta = 0.0;
        for i in 0..cs {
            let c = arena.nodes[t].children[i];
            d += arena.nodes[c].shift;
            modsumdelta += d + arena.nodes[c].change;
            arena.nodes[c].modf += modsumdelta;
        }
    }

    fn second_walk(arena: &mut Arena, t: usize, modsum: f32) {
        let modsum = modsum + arena.nodes[t].modf;
        arena.nodes[t].x = arena.nodes[t].prelim + modsum;
        add_child_spacing(arena, t);
        let cs = arena.nodes[t].children.len();
        for i in 0..cs {
            let c = arena.nodes[t].children[i];
            second_walk(arena, c, modsum);
        }
    }

    pub fn layout(arena: &mut Arena, root: usize) {
        first_walk(arena, root);
        second_walk(arena, root, 0.0);
    }
}

fn place_radial_layout(
    root_id: &str,
    info_map: &HashMap<String, MindmapNodeInfo>,
    nodes: &mut BTreeMap<String, NodeLayout>,
    horizontal_gap: f32,
    vertical_gap: f32,
) {
    let mut subtree_heights: HashMap<String, f32> = HashMap::new();
    mindmap_subtree_height(root_id, info_map, nodes, &mut subtree_heights, vertical_gap);
    let root_center = (0.0_f32, 0.0_f32);
    if let Some(root_node) = nodes.get_mut(root_id) {
        root_node.x = root_center.0 - root_node.width / 2.0;
        root_node.y = root_center.1 - root_node.height / 2.0;
    }
    let mut left_children: Vec<String> = Vec::new();
    let mut right_children: Vec<String> = Vec::new();
    if let Some(info) = info_map.get(root_id) {
        for child_id in &info.children {
            let section = info_map
                .get(child_id)
                .and_then(|child| child.section)
                .unwrap_or(0);
            if section.is_multiple_of(2) {
                right_children.push(child_id.clone());
            } else {
                left_children.push(child_id.clone());
            }
        }
    }
    let root_width = nodes.get(root_id).map(|n| n.width).unwrap_or(0.0);

    place_mindmap_children(
        &right_children,
        1.0,
        root_center,
        root_width,
        info_map,
        nodes,
        &subtree_heights,
        horizontal_gap,
        vertical_gap,
    );
    place_mindmap_children(
        &left_children,
        -1.0,
        root_center,
        root_width,
        info_map,
        nodes,
        &subtree_heights,
        horizontal_gap,
        vertical_gap,
    );
}

/// Lay out a mindmap using the non-layered tidy-tree algorithm. When
/// `lr_only` is true, every branch grows to the right of the root (matching
/// the requested `lr-tree` algorithm). Otherwise children alternate between
/// the left and right halves like Mermaid JS's `tidy-tree` algorithm.
fn place_tidy_tree(
    root_id: &str,
    info_map: &HashMap<String, MindmapNodeInfo>,
    nodes: &mut BTreeMap<String, NodeLayout>,
    horizontal_gap: f32,
    vertical_gap: f32,
    lr_only: bool,
) {
    let root_center = (0.0_f32, 0.0_f32);
    let root_width = nodes.get(root_id).map(|n| n.width).unwrap_or(0.0);

    if let Some(root_node) = nodes.get_mut(root_id) {
        root_node.x = root_center.0 - root_node.width / 2.0;
        root_node.y = root_center.1 - root_node.height / 2.0;
    }

    let Some(info) = info_map.get(root_id) else {
        return;
    };

    let mut left_children: Vec<String> = Vec::new();
    let mut right_children: Vec<String> = Vec::new();
    if lr_only {
        right_children = info.children.clone();
    } else {
        for (idx, child_id) in info.children.iter().enumerate() {
            if idx.is_multiple_of(2) {
                left_children.push(child_id.clone());
            } else {
                right_children.push(child_id.clone());
            }
        }
    }

    let h_gap = horizontal_gap.max(1.0);
    let v_gap = vertical_gap.max(1.0);

    if !right_children.is_empty() {
        let positions = layout_horizontal_subtrees(
            &right_children,
            info_map,
            nodes,
            h_gap,
            v_gap,
        );
        let edge_x = root_center.0 + root_width / 2.0 + h_gap;
        for (id, dx, cy) in positions {
            if let Some(node) = nodes.get_mut(&id) {
                node.x = edge_x + dx;
                node.y = root_center.1 + cy - node.height / 2.0;
            }
        }
    }

    if !left_children.is_empty() {
        let positions = layout_horizontal_subtrees(
            &left_children,
            info_map,
            nodes,
            h_gap,
            v_gap,
        );
        let edge_x = root_center.0 - root_width / 2.0 - h_gap;
        for (id, dx, cy) in positions {
            if let Some(node) = nodes.get_mut(&id) {
                node.x = edge_x - dx - node.width;
                node.y = root_center.1 + cy - node.height / 2.0;
            }
        }
    }
}

/// Run the tidy-tree algorithm over a forest rooted at the given children.
/// Returns triples of `(node id, depth offset from root edge, vertical center)`
/// in the original (un-rotated) frame so the caller can place the nodes either
/// side of the root.
fn layout_horizontal_subtrees(
    roots: &[String],
    info_map: &HashMap<String, MindmapNodeInfo>,
    nodes: &BTreeMap<String, NodeLayout>,
    horizontal_gap: f32,
    vertical_gap: f32,
) -> Vec<(String, f32, f32)> {
    let mut arena = tidy_tree::Arena::new();
    let mut id_lookup: Vec<String> = Vec::new();

    fn build(
        arena: &mut tidy_tree::Arena,
        id_lookup: &mut Vec<String>,
        node_id: &str,
        info_map: &HashMap<String, MindmapNodeInfo>,
        nodes: &BTreeMap<String, NodeLayout>,
        horizontal_gap: f32,
        vertical_gap: f32,
        depth_y: f32,
    ) -> usize {
        let (w, h) = nodes
            .get(node_id)
            .map(|n| (n.width, n.height))
            .unwrap_or((10.0, 10.0));
        // Tidy-tree expects vertical trees; transpose so the rendered tree
        // grows horizontally. Width/height are also padded by gap so the
        // algorithm leaves room for sibling spacing.
        let tt_w = h + vertical_gap;
        let tt_h = w + horizontal_gap;
        let children = info_map
            .get(node_id)
            .map(|i| i.children.clone())
            .unwrap_or_default();
        let mut child_indices = Vec::with_capacity(children.len());
        for child in &children {
            let idx = build(
                arena,
                id_lookup,
                child,
                info_map,
                nodes,
                horizontal_gap,
                vertical_gap,
                depth_y + tt_h,
            );
            child_indices.push(idx);
        }
        let idx = arena.alloc(tt_w, tt_h, depth_y, child_indices);
        debug_assert_eq!(idx, id_lookup.len());
        id_lookup.push(node_id.to_string());
        idx
    }

    let mut child_indices = Vec::with_capacity(roots.len());
    for root in roots {
        let idx = build(
            &mut arena,
            &mut id_lookup,
            root,
            info_map,
            nodes,
            horizontal_gap,
            vertical_gap,
            0.0,
        );
        child_indices.push(idx);
    }

    // Virtual super-root with negligible footprint so the tidy algorithm
    // arranges multiple top-level subtrees together.
    let virt = arena.alloc(0.0, 0.0, -1.0, child_indices);
    id_lookup.push(String::new());

    tidy_tree::layout(&mut arena, virt);

    // Translate & rotate: tidy.x is the position along the perpendicular to
    // the growth axis, tidy.y the depth. After 90° rotation the depth becomes
    // horizontal distance from the root edge and tidy.x becomes vertical center.
    let mut min_center_y = f32::MAX;
    let mut max_center_y = f32::MIN;
    let mut raw: Vec<(String, f32, f32, f32)> = Vec::new();
    for (idx, id) in id_lookup.iter().enumerate() {
        if id.is_empty() || idx == virt {
            continue;
        }
        let n = &arena.nodes[idx];
        // Center in the transposed frame:
        let center_along = n.x + n.w / 2.0; // → vertical center after rotation
        let depth_offset = n.y;             // → horizontal distance from root edge
        // Width/height in original frame:
        let original_height = (n.w - vertical_gap).max(0.0);
        let original_width = (n.h - horizontal_gap).max(0.0);
        let _ = original_width;
        min_center_y = min_center_y.min(center_along - original_height / 2.0);
        max_center_y = max_center_y.max(center_along + original_height / 2.0);
        raw.push((id.clone(), depth_offset, center_along, original_height));
    }

    if raw.is_empty() {
        return Vec::new();
    }
    let mid = (min_center_y + max_center_y) / 2.0;
    raw.into_iter()
        .map(|(id, dx, cy, _h)| (id, dx, cy - mid))
        .collect()
}

#[derive(Clone)]
struct MindmapPalette {
    section_fills: Vec<String>,
    section_labels: Vec<String>,
    section_lines: Vec<String>,
    root_fill: String,
    root_text: String,
}

#[derive(Clone)]
struct MindmapNodeInfo {
    level: usize,
    section: Option<usize>,
    children: Vec<String>,
}

fn mindmap_palette(theme: &Theme, config: &LayoutConfig) -> MindmapPalette {
    let mindmap = &config.mindmap;
    let section_fills = if mindmap.section_colors.is_empty() {
        vec!["#ECECFF".to_string()]
    } else {
        mindmap.section_colors.clone()
    };
    let section_labels = if mindmap.section_label_colors.is_empty() {
        vec![theme.primary_text_color.clone()]
    } else {
        mindmap.section_label_colors.clone()
    };
    let section_lines = if mindmap.section_line_colors.is_empty() {
        vec![theme.primary_border_color.clone()]
    } else {
        mindmap.section_line_colors.clone()
    };
    let root_fill = mindmap
        .root_fill
        .clone()
        .unwrap_or_else(|| theme.git_colors[0].clone());
    let root_text = mindmap
        .root_text
        .clone()
        .unwrap_or_else(|| theme.git_branch_label_colors[0].clone());
    MindmapPalette {
        section_fills,
        section_labels,
        section_lines,
        root_fill,
        root_text,
    }
}

fn pick_palette_color(values: &[String], idx: usize) -> String {
    if values.is_empty() {
        return String::new();
    }
    let index = idx % values.len();
    values[index].clone()
}

fn mindmap_node_size(
    shape: crate::ir::NodeShape,
    label: &TextBlock,
    config: &LayoutConfig,
) -> (f32, f32) {
    let mindmap = &config.mindmap;
    match shape {
        crate::ir::NodeShape::MindmapDefault => (
            label.width + mindmap.padding * 4.0,
            label.height + mindmap.padding,
        ),
        crate::ir::NodeShape::Rectangle => {
            let pad = mindmap.rect_padding;
            (label.width + pad * 2.0, label.height + pad * 2.0)
        }
        crate::ir::NodeShape::RoundRect => {
            let pad = mindmap.rounded_padding;
            (label.width + pad * 2.0, label.height + pad * 2.0)
        }
        crate::ir::NodeShape::Circle | crate::ir::NodeShape::DoubleCircle => {
            let pad = mindmap.circle_padding;
            let size = label.width.max(label.height) + pad * 2.0;
            (size, size)
        }
        crate::ir::NodeShape::Hexagon => {
            let pad_x = mindmap.rect_padding * mindmap.hexagon_padding_multiplier;
            let pad_y = mindmap.rect_padding;
            (label.width + pad_x * 2.0, label.height + pad_y * 2.0)
        }
        _ => {
            let pad = mindmap.rect_padding;
            (label.width + pad * 2.0, label.height + pad * 2.0)
        }
    }
}

fn mindmap_subtree_height(
    node_id: &str,
    info: &HashMap<String, MindmapNodeInfo>,
    nodes: &BTreeMap<String, NodeLayout>,
    memo: &mut HashMap<String, f32>,
    spacing: f32,
) -> f32 {
    if let Some(value) = memo.get(node_id) {
        return *value;
    }
    let Some(node) = nodes.get(node_id) else {
        return 0.0;
    };
    let mut height = node.height;
    if let Some(node_info) = info.get(node_id)
        && !node_info.children.is_empty()
    {
        let mut total = 0.0;
        for child in &node_info.children {
            total += mindmap_subtree_height(child, info, nodes, memo, spacing);
        }
        if node_info.children.len() > 1 {
            total += spacing * (node_info.children.len() as f32 - 1.0);
        }
        height = height.max(total);
    }
    memo.insert(node_id.to_string(), height);
    height
}

fn assign_mindmap_positions(
    node_id: &str,
    direction: f32,
    center_x: f32,
    center_y: f32,
    info: &HashMap<String, MindmapNodeInfo>,
    nodes: &mut BTreeMap<String, NodeLayout>,
    subtree_heights: &HashMap<String, f32>,
    horizontal_gap: f32,
    vertical_gap: f32,
) {
    let parent_width = if let Some(node) = nodes.get_mut(node_id) {
        node.x = center_x - node.width / 2.0;
        node.y = center_y - node.height / 2.0;
        node.width
    } else {
        return;
    };
    let Some(node_info) = info.get(node_id) else {
        return;
    };
    if node_info.children.is_empty() {
        return;
    }
    let mut total = 0.0;
    for child in &node_info.children {
        total += subtree_heights.get(child).copied().unwrap_or(0.0);
    }
    if node_info.children.len() > 1 {
        total += vertical_gap * (node_info.children.len() as f32 - 1.0);
    }
    let mut cursor = center_y - total / 2.0;
    for child_id in &node_info.children {
        let child_height = subtree_heights.get(child_id).copied().unwrap_or(0.0);
        let child_width = nodes.get(child_id).map(|node| node.width).unwrap_or(0.0);
        let child_center_y = cursor + child_height / 2.0;
        let child_center_x =
            center_x + direction * (parent_width / 2.0 + child_width / 2.0 + horizontal_gap);
        assign_mindmap_positions(
            child_id,
            direction,
            child_center_x,
            child_center_y,
            info,
            nodes,
            subtree_heights,
            horizontal_gap,
            vertical_gap,
        );
        cursor += child_height + vertical_gap;
    }
}

fn place_mindmap_children(
    children: &[String],
    direction: f32,
    parent_center: (f32, f32),
    parent_width: f32,
    info: &HashMap<String, MindmapNodeInfo>,
    nodes: &mut BTreeMap<String, NodeLayout>,
    subtree_heights: &HashMap<String, f32>,
    horizontal_gap: f32,
    vertical_gap: f32,
) {
    if children.is_empty() {
        return;
    }
    let mut total = 0.0;
    for child in children {
        total += subtree_heights.get(child).copied().unwrap_or(0.0);
    }
    if children.len() > 1 {
        total += vertical_gap * (children.len() as f32 - 1.0);
    }
    let mut cursor = parent_center.1 - total / 2.0;
    for child_id in children {
        let child_height = subtree_heights.get(child_id).copied().unwrap_or(0.0);
        let child_width = nodes.get(child_id).map(|node| node.width).unwrap_or(0.0);
        let child_center_y = cursor + child_height / 2.0;
        let child_center_x =
            parent_center.0 + direction * (parent_width / 2.0 + child_width / 2.0 + horizontal_gap);
        assign_mindmap_positions(
            child_id,
            direction,
            child_center_x,
            child_center_y,
            info,
            nodes,
            subtree_heights,
            horizontal_gap,
            vertical_gap,
        );
        cursor += child_height + vertical_gap;
    }
}

pub(super) fn compute_mindmap_layout(
    graph: &Graph,
    theme: &Theme,
    config: &LayoutConfig,
) -> Layout {
    let palette = mindmap_palette(theme, config);
    let mut nodes: BTreeMap<String, NodeLayout> = BTreeMap::new();
    let mut info_map: HashMap<String, MindmapNodeInfo> = HashMap::new();

    for node in &graph.mindmap.nodes {
        let label_text = graph
            .nodes
            .get(&node.id)
            .map(|n| n.label.clone())
            .unwrap_or_else(|| node.label.clone());
        let mut label = measure_label(&label_text, theme, config);
        label.width *= config.mindmap.text_width_scale;
        if config.mindmap.use_max_width {
            label.width = label.width.min(config.mindmap.max_node_width);
        }
        let shape = graph
            .nodes
            .get(&node.id)
            .map(|n| n.shape)
            .unwrap_or(crate::ir::NodeShape::MindmapDefault);
        let (width, height) = mindmap_node_size(shape, &label, config);
        let mut style = resolve_node_style(node.id.as_str(), graph);
        let is_root = node.level == 0;
        if is_root {
            if style.fill.is_none() {
                style.fill = Some(palette.root_fill.clone());
            }
            if style.text_color.is_none() {
                style.text_color = Some(palette.root_text.clone());
            }
        } else if let Some(section) = node.section {
            let index = section + 1;
            if style.fill.is_none() {
                style.fill = Some(pick_palette_color(&palette.section_fills, index));
            }
            if style.text_color.is_none() {
                style.text_color = Some(pick_palette_color(&palette.section_labels, index));
            }
            if style.line_color.is_none() {
                style.line_color = Some(pick_palette_color(&palette.section_lines, index));
            }
        }
        if style.stroke.is_none() {
            style.stroke = Some("none".to_string());
        }
        if style.stroke_width.is_none() {
            style.stroke_width = Some(0.0);
        }

        nodes.insert(
            node.id.clone(),
            NodeLayout {
                id: node.id.clone(),
                x: 0.0,
                y: 0.0,
                width,
                height,
                label,
                shape,
                style,
                link: graph.node_links.get(&node.id).cloned(),
                anchor_subgraph: None,
                hidden: false,
                icon: None,
            },
        );

        info_map.insert(
            node.id.clone(),
            MindmapNodeInfo {
                level: node.level,
                section: node.section,
                children: node.children.clone(),
            },
        );
    }

    let root_id = graph
        .mindmap
        .root_id
        .clone()
        .or_else(|| graph.mindmap.nodes.first().map(|node| node.id.clone()));

    let mut horizontal_gap = config.mindmap.rank_spacing * config.mindmap.rank_spacing_multiplier;
    let mut vertical_gap = config.mindmap.node_spacing * config.mindmap.node_spacing_multiplier;
    let node_count = graph.mindmap.nodes.len();
    let density_scale = if node_count >= 10 {
        0.7
    } else if node_count >= 6 {
        0.8
    } else {
        1.0
    };
    horizontal_gap = (horizontal_gap * density_scale).max(theme.font_size * 1.1);
    vertical_gap = (vertical_gap * density_scale).max(theme.font_size * 0.9);

    let algorithm = config.mindmap.layout_algorithm.to_ascii_lowercase();
    if let Some(root_id) = root_id.as_ref() {
        match algorithm.as_str() {
            "tidy-tree" | "tidy_tree" | "tidytree" => place_tidy_tree(
                root_id,
                &info_map,
                &mut nodes,
                horizontal_gap,
                vertical_gap,
                false,
            ),
            "lr-tree" | "lr_tree" | "lrtree" => place_tidy_tree(
                root_id,
                &info_map,
                &mut nodes,
                horizontal_gap,
                vertical_gap,
                true,
            ),
            _ => place_radial_layout(
                root_id,
                &info_map,
                &mut nodes,
                horizontal_gap,
                vertical_gap,
            ),
        }
    }

    let mut edges = Vec::new();
    for edge in &graph.edges {
        let Some(from_layout) = nodes.get(&edge.from) else {
            continue;
        };
        let Some(to_layout) = nodes.get(&edge.to) else {
            continue;
        };
        let from_center = (
            from_layout.x + from_layout.width / 2.0,
            from_layout.y + from_layout.height / 2.0,
        );
        let to_center = (
            to_layout.x + to_layout.width / 2.0,
            to_layout.y + to_layout.height / 2.0,
        );
        let mut override_style = crate::ir::EdgeStyleOverride::default();
        if let Some(child_info) = info_map.get(&edge.to)
            && let Some(section) = child_info.section
        {
            let index = section + 1;
            override_style.stroke = Some(pick_palette_color(&palette.section_fills, index));
        }
        let parent_level = info_map.get(&edge.from).map(|info| info.level).unwrap_or(0);
        let edge_depth = parent_level + 1;
        override_style.stroke_width = Some(
            config.mindmap.edge_depth_base_width
                + config.mindmap.edge_depth_step * (edge_depth as f32 + 1.0),
        );
        edges.push(EdgeLayout {
            from: edge.from.clone(),
            to: edge.to.clone(),
            label: None,
            start_label: None,
            end_label: None,
            label_anchor: None,
            start_label_anchor: None,
            end_label_anchor: None,
            points: vec![from_center, to_center],
            directed: false,
            arrow_start: false,
            arrow_end: false,
            arrow_start_kind: None,
            arrow_end_kind: None,
            start_decoration: None,
            end_decoration: None,
            style: crate::ir::EdgeStyle::Solid,
            override_style,
        });
    }

    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for node in nodes.values() {
        min_x = min_x.min(node.x);
        min_y = min_y.min(node.y);
        max_x = max_x.max(node.x + node.width);
        max_y = max_y.max(node.y + node.height);
    }
    for edge in &edges {
        for point in &edge.points {
            min_x = min_x.min(point.0);
            min_y = min_y.min(point.1);
            max_x = max_x.max(point.0);
            max_y = max_y.max(point.1);
        }
    }
    if min_x == f32::MAX || min_y == f32::MAX {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 1.0;
        max_y = 1.0;
    }
    let pad = config.mindmap.padding.max(8.0);
    let shift_x = pad - min_x;
    let shift_y = pad - min_y;
    if shift_x.abs() > 1e-3 || shift_y.abs() > 1e-3 {
        for node in nodes.values_mut() {
            node.x += shift_x;
            node.y += shift_y;
        }
        for edge in &mut edges {
            for point in &mut edge.points {
                point.0 += shift_x;
                point.1 += shift_y;
            }
        }
        min_x += shift_x;
        min_y += shift_y;
        max_x += shift_x;
        max_y += shift_y;
    }
    let width = (max_x - min_x + pad * 2.0).max(1.0);
    let height = (max_y - min_y + pad * 2.0).max(1.0);

    Layout {
        kind: graph.kind,
        nodes,
        edges,
        subgraphs: Vec::new(),
        width,
        height,
        diagram: DiagramData::Graph {
            state_notes: Vec::new(),
        },
    }
}
