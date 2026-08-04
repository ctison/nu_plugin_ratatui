use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Gauge, List, ListItem, Paragraph, Wrap},
};

use crate::config::{BlockSpec, Handler, UiNode};

/// A rendered button region used for mouse hit-testing.
#[derive(Clone)]
pub struct HitTarget {
    pub id: String,
    pub area: Rect,
    pub handler: Option<Handler>,
}

impl UiNode {
    /// Renders a widget tree and records all interactive regions.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, hits: &mut Vec<HitTarget>) {
        match self {
            Self::Layout {
                direction,
                constraints,
                children,
            } => {
                let areas = ratatui::layout::Layout::default()
                    .direction(*direction)
                    .constraints(constraints.clone())
                    .split(area);
                for (child, child_area) in children.iter().zip(areas.iter().copied()) {
                    child.render(frame, child_area, hits);
                }
            }
            Self::Paragraph {
                text,
                block,
                alignment,
                wrap,
            } => {
                let mut widget = Paragraph::new(text.as_str())
                    .alignment(*alignment)
                    .style(block.style)
                    .block(make_block(block));
                if *wrap {
                    widget = widget.wrap(Wrap { trim: true });
                }
                frame.render_widget(widget, area);
            }
            Self::Button {
                id,
                label,
                block,
                alignment,
                handler,
            } => {
                let widget = Paragraph::new(label.as_str())
                    .alignment(*alignment)
                    .style(block.style)
                    .block(make_block(block));
                frame.render_widget(widget, area);
                hits.push(HitTarget {
                    id: id.clone(),
                    area,
                    handler: handler.clone(),
                });
            }
            Self::List { items, block } => {
                let items = items
                    .iter()
                    .map(|item| ListItem::new(item.as_str()))
                    .collect::<Vec<_>>();
                frame.render_widget(
                    List::new(items).style(block.style).block(make_block(block)),
                    area,
                );
            }
            Self::Gauge {
                ratio,
                label,
                block,
                gauge_style,
            } => {
                let mut gauge = Gauge::default()
                    .ratio(*ratio)
                    .style(block.style)
                    .gauge_style(*gauge_style)
                    .block(make_block(block));
                if let Some(label) = label {
                    gauge = gauge.label(label.as_str());
                }
                frame.render_widget(gauge, area);
            }
            Self::Spacer => {}
        }
    }
}

impl HitTarget {
    /// Returns whether a terminal coordinate falls inside this target.
    pub fn contains(&self, column: u16, row: u16) -> bool {
        column >= self.area.x
            && column < self.area.x.saturating_add(self.area.width)
            && row >= self.area.y
            && row < self.area.y.saturating_add(self.area.height)
    }
}

/// Builds a Ratatui block from shared widget presentation fields.
fn make_block(spec: &BlockSpec) -> Block<'_> {
    let mut block = Block::default()
        .borders(spec.borders)
        .border_type(spec.border_type)
        .border_style(spec.border_style)
        .style(spec.style);
    if let Some(title) = &spec.title {
        block = block.title(title.as_str());
    }
    block
}

#[cfg(test)]
mod tests {
    use nu_protocol::{Record, Value};
    use ratatui::{Terminal, backend::TestBackend};

    use crate::config::UiNode;

    /// Builds a test record value from concise key/value pairs.
    fn record(fields: Vec<(&str, Value)>) -> Value {
        let mut record = Record::new();
        for (name, value) in fields {
            record.push(name, value);
        }
        Value::test_record(record)
    }

    /// Verifies that buttons render text and expose their complete hit region.
    #[test]
    fn renders_button_and_hit_target() {
        let button = record(vec![
            ("type", Value::test_string("button")),
            ("id", Value::test_string("save")),
            ("label", Value::test_string("Save")),
        ]);
        let node = UiNode::parse(&button).expect("valid button");
        let backend = TestBackend::new(20, 3);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut hits = Vec::new();

        terminal
            .draw(|frame| node.render(frame, frame.area(), &mut hits))
            .expect("draw succeeds");

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "save");
        assert!(hits[0].contains(0, 0));
        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("Save"));
    }
}
