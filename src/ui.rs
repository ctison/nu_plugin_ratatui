use ratatui::{
  Frame,
  layout::Rect,
  widgets::{
    Axis, Bar, BarChart, Block, Chart, Clear, Dataset, Fill, Gauge, LineGauge, List, ListItem,
    MascotEyeColor, Paragraph, RatatuiLogo, RatatuiLogoSize, RatatuiMascot, RenderDirection, Row,
    Scrollbar, ScrollbarState, Sparkline, Table, Tabs, Wrap,
    calendar::{CalendarEventStore, Monthly},
    canvas::{Canvas, Points},
  },
};

use crate::config::{AxisSpec, BlockSpec, Handler, UiNode};

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
      },
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
      },
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
      },
      Self::List { items, block } => {
        let items = items
          .iter()
          .map(|item| ListItem::new(item.as_str()))
          .collect::<Vec<_>>();
        frame.render_widget(
          List::new(items).style(block.style).block(make_block(block)),
          area,
        );
      },
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
      },
      Self::BarChart {
        bars,
        block,
        direction,
        max,
        bar_width,
        bar_gap,
        bar_style,
        value_style,
        label_style,
      } => {
        let bars = bars
          .iter()
          .map(|bar| {
            Bar::with_label(bar.label.as_str(), bar.value)
              .style(bar.style)
              .value_style(bar.value_style)
          })
          .collect::<Vec<_>>();
        let mut widget = BarChart::new(bars)
          .block(make_block(block))
          .direction(*direction)
          .bar_width(*bar_width)
          .bar_gap(*bar_gap)
          .bar_style(*bar_style)
          .value_style(*value_style)
          .label_style(*label_style)
          .style(block.style);
        if let Some(max) = max {
          widget = widget.max(*max);
        }
        frame.render_widget(widget, area);
      },
      Self::Calendar {
        year,
        month,
        block,
        show_month,
        show_weekdays,
        show_surrounding,
        default_style,
      } => {
        let events = CalendarEventStore::today(*default_style);
        let today = events
          .0
          .keys()
          .next()
          .copied()
          .expect("today event store contains one date");
        let mut date = today
          .replace_day(1)
          .and_then(|date| date.replace_year(*year))
          .expect("validated calendar year");
        while u8::from(date.month()) < *month {
          date = date.next_day().expect("validated calendar date");
        }
        while u8::from(date.month()) > *month {
          date = date.previous_day().expect("validated calendar date");
        }
        let mut widget = Monthly::new(date, events)
          .block(make_block(block))
          .default_style(*default_style);
        if let Some(style) = show_month {
          widget = widget.show_month_header(*style);
        }
        if let Some(style) = show_weekdays {
          widget = widget.show_weekdays_header(*style);
        }
        if let Some(style) = show_surrounding {
          widget = widget.show_surrounding(*style);
        }
        frame.render_widget(widget, area);
      },
      Self::Canvas {
        points,
        x_bounds,
        y_bounds,
        marker,
        background,
        block,
      } => {
        let widget = Canvas::default()
          .block(make_block(block))
          .x_bounds(*x_bounds)
          .y_bounds(*y_bounds)
          .marker(*marker)
          .background_color(*background)
          .paint(|context| {
            for point in points {
              context.draw(&Points {
                coords: &[(point.x, point.y)],
                color: point.color,
              });
            }
          });
        frame.render_widget(widget, area);
      },
      Self::Chart {
        datasets,
        x_axis,
        y_axis,
        block,
        style,
      } => {
        let datasets = datasets
          .iter()
          .map(|spec| {
            let mut dataset = Dataset::default()
              .data(&spec.data)
              .graph_type(spec.graph_type)
              .marker(spec.marker)
              .style(spec.style);
            if let Some(name) = &spec.name {
              dataset = dataset.name(name.as_str());
            }
            dataset
          })
          .collect::<Vec<_>>();
        let widget = Chart::new(datasets)
          .block(make_block(block))
          .x_axis(make_axis(x_axis))
          .y_axis(make_axis(y_axis))
          .style(*style);
        frame.render_widget(widget, area);
      },
      Self::Clear => frame.render_widget(Clear, area),
      Self::Fill { symbol, style } => {
        frame.render_widget(Fill::new(symbol.as_str()).style(*style), area);
      },
      Self::LineGauge {
        ratio,
        label,
        block,
        filled_style,
        unfilled_style,
        filled_symbol,
        unfilled_symbol,
      } => {
        let mut widget = LineGauge::default()
          .block(make_block(block))
          .ratio(*ratio)
          .filled_style(*filled_style)
          .unfilled_style(*unfilled_style)
          .filled_symbol(filled_symbol)
          .unfilled_symbol(unfilled_symbol);
        if let Some(label) = label {
          widget = widget.label(label.as_str());
        }
        frame.render_widget(widget, area);
      },
      Self::Logo { small } => frame.render_widget(
        RatatuiLogo::new(if *small {
          RatatuiLogoSize::Small
        } else {
          RatatuiLogoSize::Tiny
        }),
        area,
      ),
      Self::Mascot { blink } => frame.render_widget(
        RatatuiMascot::new().set_eye(if *blink {
          MascotEyeColor::Red
        } else {
          MascotEyeColor::Default
        }),
        area,
      ),
      Self::Scrollbar {
        content_length,
        position,
        viewport_length,
        orientation,
        thumb_style,
        track_style,
      } => {
        let widget = Scrollbar::new(orientation.clone())
          .thumb_style(*thumb_style)
          .track_style(*track_style);
        let mut state = ScrollbarState::new(*content_length)
          .position(*position)
          .viewport_content_length(*viewport_length);
        frame.render_stateful_widget(widget, area, &mut state);
      },
      Self::Sparkline {
        data,
        max,
        right_to_left,
        block,
        style,
        absent_style,
        absent_symbol,
      } => {
        let mut widget = Sparkline::default()
          .block(make_block(block))
          .data(data.iter().copied())
          .direction(if *right_to_left {
            RenderDirection::RightToLeft
          } else {
            RenderDirection::LeftToRight
          })
          .style(*style)
          .absent_value_style(*absent_style)
          .absent_value_symbol(absent_symbol);
        if let Some(max) = max {
          widget = widget.max(*max);
        }
        frame.render_widget(widget, area);
      },
      Self::Table {
        rows,
        header,
        widths,
        column_spacing,
        block,
        style,
      } => {
        let rows = rows
          .iter()
          .map(|row| Row::new(row.iter().map(String::as_str)))
          .collect::<Vec<_>>();
        let mut widget = Table::new(rows, widths.clone())
          .block(make_block(block))
          .column_spacing(*column_spacing)
          .style(*style);
        if let Some(header) = header {
          widget = widget.header(Row::new(header.iter().map(String::as_str)));
        }
        frame.render_widget(widget, area);
      },
      Self::Tabs {
        titles,
        selected,
        divider,
        block,
        style,
        highlight_style,
      } => frame.render_widget(
        Tabs::new(titles.iter().map(String::as_str))
          .select(*selected)
          .divider(divider.as_str())
          .block(make_block(block))
          .style(*style)
          .highlight_style(*highlight_style),
        area,
      ),
      Self::Spacer => {},
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

/// Builds a Ratatui chart axis from an owned declarative specification.
fn make_axis(spec: &AxisSpec) -> Axis<'_> {
  let mut axis = Axis::default()
    .bounds(spec.bounds)
    .labels(spec.labels.iter().map(String::as_str))
    .style(spec.style);
  if let Some(title) = &spec.title {
    axis = axis.title(title.as_str());
  }
  axis
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
