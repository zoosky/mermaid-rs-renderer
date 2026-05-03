use super::*;

struct MeasuredTimelineEvent {
    time: TextBlock,
    events: Vec<TextBlock>,
    height: f32,
}

pub(super) fn compute_timeline_layout(
    graph: &Graph,
    theme: &Theme,
    config: &LayoutConfig,
) -> Layout {
    let data = &graph.timeline;
    let direction = data.direction.unwrap_or_else(|| {
        Direction::from_timeline_token(&config.timeline.direction).unwrap_or(Direction::LeftRight)
    });
    let font_size = theme.font_size;
    let event_font_size = theme.font_size * 0.9;
    let padding = 30.0_f32;
    let event_width = 120.0_f32;
    let min_event_height = 80.0_f32;
    let event_spacing = 40.0_f32;
    let event_text_width = (event_width - 16.0).max(1.0);
    let title_height = if data.title.is_some() { 40.0 } else { 0.0 };

    let title = data.title.as_ref().map(|t| measure_label(t, theme, config));

    let mut max_event_height = min_event_height;
    let mut measured_events = Vec::with_capacity(data.events.len());
    for event in &data.events {
        let time_block = measure_label_with_font_size(
            &event.time,
            font_size,
            config,
            false,
            theme.font_family.as_str(),
        );
        let event_blocks: Vec<TextBlock> = event
            .events
            .iter()
            .map(|e| {
                measure_label_with_max_width(
                    e,
                    event_font_size,
                    event_text_width,
                    config,
                    true,
                    theme.font_family.as_str(),
                )
            })
            .collect();

        let time_extra =
            time_block.lines.len().saturating_sub(1) as f32 * font_size * config.label_line_height;
        let description_height = event_blocks
            .iter()
            .map(|block| block.lines.len() as f32 * event_font_size * config.label_line_height)
            .sum::<f32>();
        let event_height = (40.0 + time_extra + description_height + 16.0).max(min_event_height);
        max_event_height = max_event_height.max(event_height);

        measured_events.push(MeasuredTimelineEvent {
            time: time_block,
            events: event_blocks,
            height: event_height,
        });
    }

    let (events, width, height, line_y, line_start_x, line_end_x, line_start_y, line_end_y) =
        if direction == Direction::TopDown {
            let content_top = padding + title_height + 30.0;
            let line_x = padding + 20.0;
            let event_x = line_x + 40.0;
            let content_height = if measured_events.is_empty() {
                min_event_height
            } else {
                measured_events
                    .iter()
                    .map(|event| event.height)
                    .sum::<f32>()
                    + (measured_events.len() - 1) as f32 * event_spacing
            };
            let width = event_x + event_width + padding;
            let height = content_top + content_height + padding;
            let mut y = content_top;
            let mut events = Vec::with_capacity(measured_events.len());

            for measured in measured_events {
                let circle_y = y + measured.height / 2.0;
                events.push(TimelineEventLayout {
                    time: measured.time,
                    events: measured.events,
                    x: event_x,
                    y,
                    width: event_width,
                    height: measured.height,
                    circle_y,
                });
                y += measured.height + event_spacing;
            }

            (
                events,
                width,
                height,
                content_top,
                line_x,
                line_x,
                content_top,
                content_top + content_height,
            )
        } else {
            let line_y = padding + title_height + 60.0;
            let num_events = measured_events.len().max(1);
            let total_events_width =
                num_events as f32 * event_width + (num_events - 1) as f32 * event_spacing;
            let width = padding * 2.0 + total_events_width;
            let height = padding * 2.0 + title_height + max_event_height + 100.0;
            let mut events = Vec::with_capacity(measured_events.len());

            for (i, measured) in measured_events.into_iter().enumerate() {
                let x = padding + i as f32 * (event_width + event_spacing);
                let y = line_y + 30.0;
                events.push(TimelineEventLayout {
                    time: measured.time,
                    events: measured.events,
                    x,
                    y,
                    width: event_width,
                    height: measured.height,
                    circle_y: line_y,
                });
            }

            (
                events,
                width,
                height,
                line_y,
                padding,
                width - padding,
                line_y,
                line_y,
            )
        };

    let sections: Vec<TimelineSectionLayout> = data
        .sections
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let label = measure_label(section, theme, config);
            TimelineSectionLayout {
                label,
                x: padding + i as f32 * 200.0,
                y: padding,
                width: 180.0,
                height: 30.0,
            }
        })
        .collect();

    let mut nodes = BTreeMap::new();
    nodes.insert(
        "__timeline_metrics_content".to_string(),
        NodeLayout {
            id: "__timeline_metrics_content".to_string(),
            x: padding,
            y: padding,
            width: (width - padding * 2.0).max(1.0),
            height: (height - padding * 2.0).max(1.0),
            label: TextBlock {
                lines: vec![String::new()],
                width: 0.0,
                height: 0.0,
            },
            shape: crate::ir::NodeShape::Rectangle,
            style: crate::ir::NodeStyle::default(),
            link: None,
            anchor_subgraph: None,
            hidden: false,
            icon: None,
        },
    );

    Layout {
        kind: graph.kind,
        nodes,
        edges: Vec::new(),
        subgraphs: Vec::new(),
        diagram: DiagramData::Timeline(TimelineLayout {
            title,
            title_y: padding + font_size,
            events,
            sections,
            direction,
            line_y,
            line_start_x,
            line_end_x,
            line_start_y,
            line_end_y,
            width,
            height,
        }),
        width,
        height,
    }
}
