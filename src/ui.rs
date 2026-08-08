use ratatui::{
  Frame,
  buffer::Buffer,
  layout::{Position, Rect, Size},
  symbols::border,
  text::{Line, Text},
  widgets::{
    Axis, Bar, BarChart, Block, Chart, Clear, Dataset, Fill, Gauge, LineGauge, List, ListItem,
    MascotEyeColor, Paragraph, RatatuiLogo, RatatuiLogoSize, RatatuiMascot, RenderDirection, Row,
    Scrollbar, ScrollbarState, Sparkline, StatefulWidget, Table, Tabs, Widget, Wrap,
    calendar::{CalendarEventStore, Monthly},
    canvas::{Canvas, Points},
  },
};
use tui_widgets::{
  bar_graph::BarGraph,
  big_text::BigText,
  box_text::BoxChar,
  cards::Card,
  equalizer::{Band, Equalizer},
  popup::{KnownSizeWrapper, Popup},
  prompts::{SelectOptionList, SelectPrompt, SelectState, TextPrompt, TextState},
  qrcode::QrCodeWidget,
  scrollbar::{GlyphSet, ScrollBar as FractionalScrollbar, ScrollLengths},
  scrollview::{ScrollView, ScrollViewState},
};

use crate::{
  config::{AxisSpec, BlockSpec, FractionalGlyphs, Handler, SuiteWidget, UiNode},
  effect::EffectRegistry,
};

/// A rendered button region used for mouse hit-testing.
#[derive(Clone)]
pub struct HitTarget {
  pub id: String,
  pub area: Rect,
  pub handler: Option<Handler>,
}

impl UiNode {
  /// Renders a widget tree using persistent state for declarative effect wrappers.
  pub(crate) fn render_with_effects(
    &self,
    frame: &mut Frame<'_>,
    area: Rect,
    hits: &mut Vec<HitTarget>,
    effects: &mut EffectRegistry,
  ) {
    self.render_to_buffer(frame.buffer_mut(), area, hits, effects);
  }

  /// Renders a widget tree into any Ratatui buffer, including off-screen scroll buffers.
  fn render_to_buffer(
    &self,
    buffer: &mut Buffer,
    area: Rect,
    hits: &mut Vec<HitTarget>,
    effects: &mut EffectRegistry,
  ) {
    match self {
      Self::Effect { id, effect, child } => {
        child.render_to_buffer(buffer, area, hits, effects);
        effects.apply(id, effect, buffer, area);
      },
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
          child.render_to_buffer(buffer, child_area, hits, effects);
        }
      },
      Self::Paragraph {
        text,
        block,
        alignment,
        wrap,
        scroll_x,
        scroll_y,
      } => {
        let mut widget = Paragraph::new(text.clone())
          .alignment(*alignment)
          .style(block.style)
          .block(make_block(block))
          .scroll((*scroll_y, *scroll_x));
        if *wrap {
          widget = widget.wrap(Wrap { trim: true });
        }
        widget.render(area, buffer);
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
        widget.render(area, buffer);
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
        Widget::render(
          List::new(items).style(block.style).block(make_block(block)),
          area,
          buffer,
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
        gauge.render(area, buffer);
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
        Widget::render(widget, area, buffer);
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
        widget.render(area, buffer);
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
        widget.render(area, buffer);
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
        widget.render(area, buffer);
      },
      Self::Clear => Clear.render(area, buffer),
      Self::Fill { symbol, style } => {
        Fill::new(symbol.as_str())
          .style(*style)
          .render(area, buffer);
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
        widget.render(area, buffer);
      },
      Self::Logo { small } => Widget::render(
        RatatuiLogo::new(if *small {
          RatatuiLogoSize::Small
        } else {
          RatatuiLogoSize::Tiny
        }),
        area,
        buffer,
      ),
      Self::Mascot { blink } => Widget::render(
        RatatuiMascot::new().set_eye(if *blink {
          MascotEyeColor::Red
        } else {
          MascotEyeColor::Default
        }),
        area,
        buffer,
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
        StatefulWidget::render(widget, area, buffer, &mut state);
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
        widget.render(area, buffer);
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
        Widget::render(widget, area, buffer);
      },
      Self::Tabs {
        titles,
        selected,
        divider,
        block,
        style,
        highlight_style,
      } => Widget::render(
        Tabs::new(titles.iter().map(String::as_str))
          .select(*selected)
          .divider(divider.as_str())
          .block(make_block(block))
          .style(*style)
          .highlight_style(*highlight_style),
        area,
        buffer,
      ),
      Self::Suite(widget) => render_suite_widget(widget, buffer, area, hits, effects),
      Self::Spacer => {},
    }
  }
}

/// Renders one widget supplied by the external `tui-widgets` suite.
fn render_suite_widget(
  widget: &SuiteWidget,
  buffer: &mut Buffer,
  area: Rect,
  hits: &mut Vec<HitTarget>,
  effects: &mut EffectRegistry,
) {
  match widget {
    SuiteWidget::BarGraph {
      data,
      min,
      max,
      bar_style,
    } => {
      let mut widget = BarGraph::new(data.clone()).with_bar_style(*bar_style);
      if let Some(min) = min {
        widget = widget.with_min(*min);
      }
      if let Some(max) = max {
        widget = widget.with_max(*max);
      }
      widget.render(area, buffer);
    },
    SuiteWidget::BigText {
      text,
      block,
      alignment,
      pixel_size,
    } => {
      BigText::builder()
        .lines(vec![Line::raw(text.clone())])
        .style(block.style)
        .alignment(*alignment)
        .pixel_size(*pixel_size)
        .block(make_block(block))
        .build()
        .render(area, buffer);
    },
    SuiteWidget::BoxText { character } => BoxChar::new(*character).render(area, buffer),
    SuiteWidget::Card { rank, suit } => Card::new(*rank, *suit).render(area, buffer),
    SuiteWidget::Equalizer { bands, brightness } => Equalizer {
      bands: bands.iter().copied().map(Band::from).collect(),
      brightness: *brightness,
    }
    .render(area, buffer),
    SuiteWidget::Popup {
      text,
      block,
      width,
      height,
    } => {
      let body = Text::raw(text.clone());
      let measured_width = body.width();
      let measured_height = body.height();
      let body = KnownSizeWrapper::new(
        body,
        width.unwrap_or(measured_width),
        height.unwrap_or(measured_height),
      );
      Widget::render(
        Popup::new(body)
          .title(Line::raw(block.title.clone().unwrap_or_default()))
          .style(block.style)
          .borders(block.borders)
          .border_set(popup_border_set(block.border_type))
          .border_style(block.border_style),
        area,
        buffer,
      );
    },
    SuiteWidget::TextPrompt {
      message,
      value,
      status,
      focus,
      render_style,
      show_status,
      block,
    } => {
      let mut prompt = TextPrompt::new(message.clone().into())
        .with_block(make_block(block))
        .with_render_style(*render_style);
      if !show_status {
        prompt = prompt.without_status_symbol();
      }
      let mut state = TextState::new()
        .with_value(value.clone())
        .with_status(*status)
        .with_focus(*focus);
      StatefulWidget::render(prompt, area, buffer, &mut state);
    },
    SuiteWidget::SelectPrompt {
      label,
      options,
      selected,
      status,
      focus,
      block,
    } => {
      let options = SelectOptionList::from(options.clone());
      let prompt = SelectPrompt::new(label.clone().into(), options).with_block(make_block(block));
      let mut state = SelectState::new().with_status(*status).with_focus(*focus);
      state.set_focused_index(*selected);
      StatefulWidget::render(prompt, area, buffer, &mut state);
    },
    SuiteWidget::QrCode {
      data,
      quiet_zone,
      scaling,
      colors,
      style,
    } => QrCodeWidget::new(qrcode::QrCode::new(data.as_bytes()).expect("validated QR data"))
      .quiet_zone(*quiet_zone)
      .scaling(*scaling)
      .colors(*colors)
      .style(*style)
      .render(area, buffer),
    SuiteWidget::FractionalScrollbar {
      content_length,
      viewport_length,
      position,
      orientation,
      arrows,
      glyphs,
      track_style,
      thumb_style,
      arrow_style,
    } => FractionalScrollbar::new(
      *orientation,
      ScrollLengths {
        content_len: *content_length,
        viewport_len: *viewport_length,
      },
    )
    .offset(*position)
    .arrows(*arrows)
    .glyph_set(match glyphs {
      FractionalGlyphs::Minimal => GlyphSet::minimal(),
      FractionalGlyphs::BoxDrawing => GlyphSet::box_drawing(),
      FractionalGlyphs::Legacy => GlyphSet::symbols_for_legacy_computing(),
      FractionalGlyphs::Unicode => GlyphSet::unicode(),
    })
    .track_style(*track_style)
    .thumb_style(*thumb_style)
    .arrow_style(*arrow_style)
    .render(area, buffer),
    SuiteWidget::ScrollView {
      width,
      height,
      scroll_x,
      scroll_y,
      vertical_scrollbar,
      horizontal_scrollbar,
      child,
    } => {
      let mut scroll_view = ScrollView::new(Size::new(*width, *height))
        .vertical_scrollbar_visibility(*vertical_scrollbar)
        .horizontal_scrollbar_visibility(*horizontal_scrollbar);
      let content_area = scroll_view.area();
      let mut child_hits = Vec::new();
      child.render_to_buffer(
        scroll_view.buf_mut(),
        content_area,
        &mut child_hits,
        effects,
      );
      let mut state = ScrollViewState::with_offset(Position::new(*scroll_x, *scroll_y));
      StatefulWidget::render(&scroll_view, area, buffer, &mut state);
      let offset = state.offset();
      hits.extend(
        child_hits
          .into_iter()
          .filter_map(|hit| translate_scrolled_hit(hit, area, offset)),
      );
    },
  }
}

/// Maps Ratatui border types to the popup crate's symbol sets.
fn popup_border_set(border_type: ratatui::widgets::BorderType) -> border::Set<'static> {
  match border_type {
    ratatui::widgets::BorderType::Plain => border::PLAIN,
    ratatui::widgets::BorderType::Rounded => border::ROUNDED,
    ratatui::widgets::BorderType::Double => border::DOUBLE,
    ratatui::widgets::BorderType::Thick => border::THICK,
    ratatui::widgets::BorderType::QuadrantInside => border::QUADRANT_INSIDE,
    ratatui::widgets::BorderType::QuadrantOutside => border::QUADRANT_OUTSIDE,
    _ => border::PLAIN,
  }
}

/// Translates an off-screen child hit target into the visible scroll-view rectangle.
fn translate_scrolled_hit(
  mut hit: HitTarget,
  viewport: Rect,
  offset: Position,
) -> Option<HitTarget> {
  let left = i32::from(viewport.x) + i32::from(hit.area.x) - i32::from(offset.x);
  let top = i32::from(viewport.y) + i32::from(hit.area.y) - i32::from(offset.y);
  let right = left + i32::from(hit.area.width);
  let bottom = top + i32::from(hit.area.height);
  let clipped_left = left.max(i32::from(viewport.x));
  let clipped_top = top.max(i32::from(viewport.y));
  let clipped_right = right.min(i32::from(viewport.right()));
  let clipped_bottom = bottom.min(i32::from(viewport.bottom()));
  if clipped_left >= clipped_right || clipped_top >= clipped_bottom {
    return None;
  }
  hit.area = Rect::new(
    clipped_left as u16,
    clipped_top as u16,
    (clipped_right - clipped_left) as u16,
    (clipped_bottom - clipped_top) as u16,
  );
  Some(hit)
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
  use std::time::Duration;

  use nu_protocol::{Record, Value};
  use ratatui::{
    Terminal,
    backend::TestBackend,
    style::{Color, Modifier},
  };

  use crate::{config::UiNode, effect::EffectRegistry};

  /// Builds a test record value from concise key/value pairs.
  fn record(fields: Vec<(&str, Value)>) -> Value {
    let mut record = Record::new();
    for (name, value) in fields {
      record.push(name, value);
    }
    Value::test_record(record)
  }

  /// Renders a node once with a fresh effect registry.
  fn render(node: &UiNode, frame: &mut ratatui::Frame<'_>, hits: &mut Vec<super::HitTarget>) {
    let mut effects = EffectRegistry::default();
    effects.begin_frame(Duration::ZERO);
    node.render_with_effects(frame, frame.area(), hits, &mut effects);
    effects.end_frame();
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
      .draw(|frame| render(&node, frame, &mut hits))
      .expect("draw succeeds");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "save");
    assert!(hits[0].contains(0, 0));
    let rendered = terminal.backend().to_string();
    assert!(rendered.contains("Save"));
  }

  /// Verifies ANSI SGR colors and modifiers are applied to rendered cells.
  #[test]
  fn renders_ansi_styled_paragraph() {
    let paragraph = record(vec![
      ("type", Value::test_string("paragraph")),
      ("text", Value::test_string("\u{1b}[31;44;1mR\u{1b}[0m")),
      ("ansi", Value::test_bool(true)),
    ]);
    let node = UiNode::parse(&paragraph).expect("valid ANSI paragraph");
    let backend = TestBackend::new(1, 1);
    let mut terminal = Terminal::new(backend).expect("test terminal");

    terminal
      .draw(|frame| render(&node, frame, &mut Vec::new()))
      .expect("draw succeeds");

    let cell = terminal
      .backend()
      .buffer()
      .cell((0, 0))
      .expect("rendered cell");
    assert_eq!(cell.symbol(), "R");
    assert_eq!(cell.fg, Color::Red);
    assert_eq!(cell.bg, Color::Blue);
    assert!(cell.modifier.contains(Modifier::BOLD));
  }

  /// Verifies both paragraph scroll offsets affect the visible text.
  #[test]
  fn renders_scrolled_paragraph() {
    let paragraph = record(vec![
      ("type", Value::test_string("paragraph")),
      ("text", Value::test_string("abcd\nefgh\nijkl")),
      ("wrap", Value::test_bool(false)),
      ("scroll-x", Value::test_int(2)),
      ("scroll-y", Value::test_int(1)),
    ]);
    let node = UiNode::parse(&paragraph).expect("valid scrolled paragraph");
    let backend = TestBackend::new(2, 1);
    let mut terminal = Terminal::new(backend).expect("test terminal");

    terminal
      .draw(|frame| render(&node, frame, &mut Vec::new()))
      .expect("draw succeeds");

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.cell((0, 0)).expect("first cell").symbol(), "g");
    assert_eq!(buffer.cell((1, 0)).expect("second cell").symbol(), "h");
  }

  /// Verifies the suite's prompt widget renders its declarative value snapshot.
  #[test]
  fn renders_tui_widgets_text_prompt() {
    let prompt = record(vec![
      ("type", Value::test_string("text-prompt")),
      ("message", Value::test_string("Name")),
      ("value", Value::test_string("Ada")),
      ("focused", Value::test_bool(true)),
    ]);
    let node = UiNode::parse(&prompt).expect("valid text prompt");
    let backend = TestBackend::new(30, 3);
    let mut terminal = Terminal::new(backend).expect("test terminal");

    terminal
      .draw(|frame| render(&node, frame, &mut Vec::new()))
      .expect("draw succeeds");

    let rendered = terminal.backend().to_string();
    assert!(rendered.contains("Name"));
    assert!(rendered.contains("Ada"));
  }

  /// Verifies QR payloads render through the suite rather than a hand-built approximation.
  #[test]
  fn renders_tui_widgets_qr_code() {
    let qr_code = record(vec![
      ("type", Value::test_string("qr-code")),
      ("data", Value::test_string("nu_plugin_ratatui")),
      ("no-quiet-zone", Value::test_bool(true)),
    ]);
    let node = UiNode::parse(&qr_code).expect("valid QR code");
    let backend = TestBackend::new(40, 20);
    let mut terminal = Terminal::new(backend).expect("test terminal");

    terminal
      .draw(|frame| render(&node, frame, &mut Vec::new()))
      .expect("draw succeeds");

    assert!(terminal.backend().to_string().contains('█'));
  }

  /// Verifies scroll views render their child into an off-screen buffer at the requested offset.
  #[test]
  fn renders_tui_widgets_scroll_view_offset() {
    let child = record(vec![
      ("type", Value::test_string("paragraph")),
      ("text", Value::test_string("first\nsecond\nthird")),
      ("wrap", Value::test_bool(false)),
    ]);
    let scroll_view = record(vec![
      ("type", Value::test_string("scroll-view")),
      ("width", Value::test_int(10)),
      ("height", Value::test_int(3)),
      ("child", child),
      ("scroll-y", Value::test_int(1)),
      ("vertical-scrollbar", Value::test_string("never")),
      ("horizontal-scrollbar", Value::test_string("never")),
    ]);
    let node = UiNode::parse(&scroll_view).expect("valid scroll view");
    let backend = TestBackend::new(10, 1);
    let mut terminal = Terminal::new(backend).expect("test terminal");

    terminal
      .draw(|frame| render(&node, frame, &mut Vec::new()))
      .expect("draw succeeds");

    let rendered = terminal.backend().to_string();
    assert!(rendered.contains("second"), "rendered {rendered:?}");
  }
}
