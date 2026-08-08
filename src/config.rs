use std::{collections::HashSet, str::FromStr, time::Duration};

use ansi_to_tui::IntoText as _;
use nu_protocol::{LabeledError, Record, Span, Spanned, Value, engine::Closure};
use ratatui::{
  layout::{Alignment, Constraint, Direction},
  style::{Color, Modifier, Style},
  symbols::Marker,
  text::Text,
  widgets::{BorderType, Borders, GraphType, ScrollbarOrientation},
};
use tui_widgets::{
  bar_graph::BarStyle as SuiteBarStyle,
  big_text::PixelSize,
  cards::{Rank, Suit},
  prompts::{FocusState, Status, TextRenderStyle},
  qrcode::{Colors as QrColors, QuietZone, Scaling as QrScaling},
  scrollbar::{ScrollBarArrows, ScrollBarOrientation as FractionalScrollbarOrientation},
  scrollview::ScrollbarVisibility,
};

use crate::effect::EffectSpec;

/// A Nushell closure paired with the source span that supplied it.
pub type Handler = Spanned<Closure>;

/// Runtime configuration parsed from the root `tui` command arguments.
#[derive(Clone)]
pub struct AppConfig {
  pub state: Value,
  pub view: Value,
  pub on_event: Option<Handler>,
  pub on_key: Option<Handler>,
  pub quit_on_esc: bool,
  pub tick_rate: Duration,
}

/// Shared presentation options supported by bordered widgets.
#[derive(Clone, Debug)]
pub struct BlockSpec {
  pub title: Option<String>,
  pub borders: Borders,
  pub border_type: BorderType,
  pub style: Style,
  pub border_style: Style,
}

/// One labelled value in a bar chart.
#[derive(Clone, Debug)]
pub struct BarSpec {
  pub label: String,
  pub value: u64,
  pub style: Style,
  pub value_style: Style,
}

/// One plotted dataset in a chart.
#[derive(Clone, Debug)]
pub struct DatasetSpec {
  pub name: Option<String>,
  pub data: Vec<(f64, f64)>,
  pub graph_type: GraphType,
  pub marker: Marker,
  pub style: Style,
}

/// Presentation and scale fields for one chart axis.
#[derive(Clone, Debug)]
pub struct AxisSpec {
  pub title: Option<String>,
  pub bounds: [f64; 2],
  pub labels: Vec<String>,
  pub style: Style,
}

/// One colored coordinate rendered by a canvas.
#[derive(Clone, Copy, Debug)]
pub struct CanvasPoint {
  pub x: f64,
  pub y: f64,
  pub color: Color,
}

/// Glyph family used by the suite's fractional scrollbar.
#[derive(Clone, Copy, Debug)]
pub enum FractionalGlyphs {
  Minimal,
  BoxDrawing,
  Legacy,
  Unicode,
}

/// Declarative widgets supplied by the external `tui-widgets` suite.
#[derive(Clone, Debug)]
pub enum SuiteWidget {
  BarGraph {
    data: Vec<f64>,
    min: Option<f64>,
    max: Option<f64>,
    bar_style: SuiteBarStyle,
  },
  BigText {
    text: String,
    block: BlockSpec,
    alignment: Alignment,
    pixel_size: PixelSize,
  },
  BoxText {
    character: char,
  },
  Card {
    rank: Rank,
    suit: Suit,
  },
  Equalizer {
    bands: Vec<f64>,
    brightness: f64,
  },
  Popup {
    text: String,
    block: BlockSpec,
    width: Option<usize>,
    height: Option<usize>,
  },
  TextPrompt {
    message: String,
    value: String,
    status: Status,
    focus: FocusState,
    render_style: TextRenderStyle,
    show_status: bool,
    block: BlockSpec,
  },
  SelectPrompt {
    label: String,
    options: Vec<String>,
    selected: usize,
    status: Status,
    focus: FocusState,
    block: BlockSpec,
  },
  QrCode {
    data: String,
    quiet_zone: QuietZone,
    scaling: QrScaling,
    colors: QrColors,
    style: Style,
  },
  FractionalScrollbar {
    content_length: usize,
    viewport_length: usize,
    position: usize,
    orientation: FractionalScrollbarOrientation,
    arrows: ScrollBarArrows,
    glyphs: FractionalGlyphs,
    track_style: Style,
    thumb_style: Style,
    arrow_style: Style,
  },
  ScrollView {
    width: u16,
    height: u16,
    scroll_x: u16,
    scroll_y: u16,
    vertical_scrollbar: ScrollbarVisibility,
    horizontal_scrollbar: ScrollbarVisibility,
    child: Box<UiNode>,
  },
}

/// A declarative widget tree parsed from a Nushell record.
#[derive(Clone, Debug)]
pub enum UiNode {
  Effect {
    id: String,
    effect: EffectSpec,
    child: Box<UiNode>,
  },
  Layout {
    direction: Direction,
    constraints: Vec<Constraint>,
    children: Vec<UiNode>,
  },
  Paragraph {
    text: Text<'static>,
    block: BlockSpec,
    alignment: Alignment,
    wrap: bool,
    scroll_x: u16,
    scroll_y: u16,
  },
  Button {
    id: String,
    label: String,
    block: BlockSpec,
    alignment: Alignment,
    handler: Option<Handler>,
  },
  List {
    items: Vec<String>,
    block: BlockSpec,
  },
  Gauge {
    ratio: f64,
    label: Option<String>,
    block: BlockSpec,
    gauge_style: Style,
  },
  BarChart {
    bars: Vec<BarSpec>,
    block: BlockSpec,
    direction: Direction,
    max: Option<u64>,
    bar_width: u16,
    bar_gap: u16,
    bar_style: Style,
    value_style: Style,
    label_style: Style,
  },
  Calendar {
    year: i32,
    month: u8,
    block: BlockSpec,
    show_month: Option<Style>,
    show_weekdays: Option<Style>,
    show_surrounding: Option<Style>,
    default_style: Style,
  },
  Canvas {
    points: Vec<CanvasPoint>,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    marker: Marker,
    background: Color,
    block: BlockSpec,
  },
  Chart {
    datasets: Vec<DatasetSpec>,
    x_axis: AxisSpec,
    y_axis: AxisSpec,
    block: BlockSpec,
    style: Style,
  },
  Clear,
  Fill {
    symbol: String,
    style: Style,
  },
  LineGauge {
    ratio: f64,
    label: Option<String>,
    block: BlockSpec,
    filled_style: Style,
    unfilled_style: Style,
    filled_symbol: String,
    unfilled_symbol: String,
  },
  Logo {
    small: bool,
  },
  Mascot {
    blink: bool,
  },
  Scrollbar {
    content_length: usize,
    position: usize,
    viewport_length: usize,
    orientation: ScrollbarOrientation,
    thumb_style: Style,
    track_style: Style,
  },
  Sparkline {
    data: Vec<Option<u64>>,
    max: Option<u64>,
    right_to_left: bool,
    block: BlockSpec,
    style: Style,
    absent_style: Style,
    absent_symbol: String,
  },
  Table {
    rows: Vec<Vec<String>>,
    header: Option<Vec<String>>,
    widths: Vec<Constraint>,
    column_spacing: u16,
    block: BlockSpec,
    style: Style,
  },
  Tabs {
    titles: Vec<String>,
    selected: Option<usize>,
    divider: String,
    block: BlockSpec,
    style: Style,
    highlight_style: Style,
  },
  Suite(SuiteWidget),
  Spacer,
}

impl AppConfig {
  /// Parses and validates the assembled top-level application arguments.
  pub fn parse(value: &Value, call_span: Span) -> Result<Self, LabeledError> {
    let record = as_record(value, "config")?;
    let view = required(record, "view", value.span())?.clone();
    if !matches!(view, Value::Record { .. } | Value::Closure { .. }) {
      return Err(config_error(
        "`view` must be a widget record or closure",
        view.span(),
      ));
    }

    let state = record
      .get("state")
      .cloned()
      .unwrap_or_else(|| Value::nothing(call_span));
    let on_event = optional_handler(record, "on-event")?;
    let on_key = optional_handler(record, "on-key")?;
    let quit_on_esc = optional_bool(record, "quit-on-esc")?.unwrap_or(true);
    let tick_rate_ms = optional_int(record, "tick-rate-ms")?.unwrap_or(250);
    if tick_rate_ms < 1 {
      return Err(config_error(
        "`tick-rate-ms` must be at least 1",
        record
          .get("tick-rate-ms")
          .map(Value::span)
          .unwrap_or(call_span),
      ));
    }

    Ok(Self {
      state,
      view,
      on_event,
      on_key,
      quit_on_esc,
      tick_rate: Duration::from_millis(tick_rate_ms as u64),
    })
  }
}

impl UiNode {
  /// Parses a widget record and all nested children.
  pub fn parse(value: &Value) -> Result<Self, LabeledError> {
    Self::parse_with_effect_ids(value, &mut HashSet::new())
  }

  /// Parses a widget record while rejecting repeated effect wrapper IDs.
  fn parse_with_effect_ids(
    value: &Value,
    effect_ids: &mut HashSet<String>,
  ) -> Result<Self, LabeledError> {
    let record = as_record(value, "widget")?;
    let widget_type = required_string(record, "type", value.span())?;
    match widget_type.as_str() {
      "effect" => parse_effect_wrapper(record, value, effect_ids),
      "layout" => parse_layout(record, value.span(), effect_ids),
      "paragraph" => parse_paragraph(record, value.span()),
      "button" => parse_button(record, value.span()),
      "list" => parse_list(record, value.span()),
      "gauge" => parse_gauge(record, value.span()),
      "bar-chart" => parse_bar_chart(record, value.span()),
      "calendar" | "monthly" => parse_calendar(record, value.span()),
      "canvas" => parse_canvas(record, value.span()),
      "chart" => parse_chart(record, value.span()),
      "clear" => Ok(Self::Clear),
      "fill" => Ok(Self::Fill {
        symbol: optional_string(record, "symbol")?.unwrap_or_else(|| " ".into()),
        style: parse_style(record.get("style"))?,
      }),
      "line-gauge" => parse_line_gauge(record, value.span()),
      "logo" | "ratatui-logo" => parse_logo(record, value.span()),
      "mascot" | "ratatui-mascot" => Ok(Self::Mascot {
        blink: optional_bool(record, "blink")?.unwrap_or(false),
      }),
      "scrollbar" => parse_scrollbar(record, value.span()),
      "sparkline" => parse_sparkline(record, value.span()),
      "table" => parse_table(record, value.span()),
      "tabs" => parse_tabs(record, value.span()),
      "bar-graph" => parse_bar_graph(record, value.span()),
      "big-text" => parse_big_text(record, value.span()),
      "box-text" => parse_box_text(record, value.span()),
      "card" => parse_card(record, value.span()),
      "equalizer" => parse_equalizer(record, value.span()),
      "popup" => parse_popup(record, value.span()),
      "text-prompt" => parse_text_prompt(record, value.span()),
      "select-prompt" => parse_select_prompt(record, value.span()),
      "qr-code" => parse_qr_code(record, value.span()),
      "fractional-scrollbar" => parse_fractional_scrollbar(record, value.span()),
      "scroll-view" => parse_scroll_view(record, value.span(), effect_ids),
      "spacer" => Ok(Self::Spacer),
      other => Err(config_error(
        format!("unsupported widget type `{other}`"),
        required(record, "type", value.span())?.span(),
      )),
    }
  }
}

/// Parses a stateful effect wrapper and its single child subtree.
fn parse_effect_wrapper(
  record: &Record,
  value: &Value,
  effect_ids: &mut HashSet<String>,
) -> Result<UiNode, LabeledError> {
  let id = required_string(record, "id", value.span())?;
  if !effect_ids.insert(id.clone()) {
    return Err(config_error(
      format!("duplicate effect wrapper ID `{id}`"),
      record.get("id").map(Value::span).unwrap_or(value.span()),
    ));
  }
  Ok(UiNode::Effect {
    id,
    effect: EffectSpec::parse(required(record, "effect", value.span())?)?,
    child: Box::new(UiNode::parse_with_effect_ids(
      required(record, "child", value.span())?,
      effect_ids,
    )?),
  })
}

/// Parses a layout node with constraints matched to its children.
fn parse_layout(
  record: &Record,
  span: Span,
  effect_ids: &mut HashSet<String>,
) -> Result<UiNode, LabeledError> {
  let direction = match optional_string(record, "direction")?.as_deref() {
    None | Some("vertical") => Direction::Vertical,
    Some("horizontal") => Direction::Horizontal,
    Some(other) => {
      return Err(config_error(
        format!("unsupported layout direction `{other}`"),
        record.get("direction").map(Value::span).unwrap_or(span),
      ));
    },
  };
  let child_values = required(record, "children", span)?
    .as_list()
    .map_err(|error| LabeledError::from_diagnostic(&error))?;
  let children = child_values
    .iter()
    .map(|child| UiNode::parse_with_effect_ids(child, effect_ids))
    .collect::<Result<Vec<_>, _>>()?;
  let constraints = match record.get("constraints") {
    Some(value) => value
      .as_list()
      .map_err(|error| LabeledError::from_diagnostic(&error))?
      .iter()
      .map(parse_constraint)
      .collect::<Result<Vec<_>, _>>()?,
    None => vec![Constraint::Fill(1); children.len()],
  };
  if constraints.len() != children.len() {
    return Err(config_error(
      "`constraints` must contain one item per child",
      record.get("constraints").map(Value::span).unwrap_or(span),
    ));
  }
  Ok(UiNode::Layout {
    direction,
    constraints,
    children,
  })
}

/// Parses one Ratatui layout constraint record.
fn parse_constraint(value: &Value) -> Result<Constraint, LabeledError> {
  let record = as_record(value, "constraint")?;
  if record.len() != 1 {
    return Err(config_error(
      "a constraint must contain exactly one of: length, percentage, ratio, min, max, fill",
      value.span(),
    ));
  }
  let (kind, amount) = record.iter().next().expect("record length checked");
  let amount = amount
    .as_int()
    .map_err(|error| LabeledError::from_diagnostic(&error))?;
  let amount = u16::try_from(amount).map_err(|_| {
    config_error(
      "constraint values must be between 0 and 65535",
      value.span(),
    )
  })?;
  match kind.as_str() {
    "length" => Ok(Constraint::Length(amount)),
    "percentage" => Ok(Constraint::Percentage(amount)),
    "min" => Ok(Constraint::Min(amount)),
    "max" => Ok(Constraint::Max(amount)),
    "fill" => Ok(Constraint::Fill(amount)),
    other => Err(config_error(
      format!("unsupported constraint `{other}`"),
      value.span(),
    )),
  }
}

/// Parses a paragraph widget record.
fn parse_paragraph(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let text_value = required(record, "text", span)?;
  let text = text_value
    .as_str()
    .map_err(|error| LabeledError::from_diagnostic(&error))?;
  let text = if optional_bool(record, "ansi")?.unwrap_or(false) {
    text.into_text().map_err(|error| {
      config_error(
        format!("failed to parse ANSI text: {error}"),
        text_value.span(),
      )
    })?
  } else {
    Text::raw(text.to_owned())
  };
  Ok(UiNode::Paragraph {
    text,
    block: parse_block(record, false)?,
    alignment: parse_alignment(record, span)?,
    wrap: optional_bool(record, "wrap")?.unwrap_or(true),
    scroll_x: optional_u16(record, "scroll-x")?.unwrap_or(0),
    scroll_y: optional_u16(record, "scroll-y")?.unwrap_or(0),
  })
}

/// Parses an interactive button widget record.
fn parse_button(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  Ok(UiNode::Button {
    id: required_string(record, "id", span)?,
    label: required_string(record, "label", span)?,
    block: parse_block(record, true)?,
    alignment: parse_alignment(record, span)?,
    handler: optional_handler(record, "on-click")?,
  })
}

/// Parses a list widget record.
fn parse_list(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let items = required(record, "items", span)?
    .as_list()
    .map_err(|error| LabeledError::from_diagnostic(&error))?
    .iter()
    .map(|value| {
      value
        .as_str()
        .map(str::to_owned)
        .map_err(|error| LabeledError::from_diagnostic(&error))
    })
    .collect::<Result<Vec<_>, _>>()?;
  Ok(UiNode::List {
    items,
    block: parse_block(record, false)?,
  })
}

/// Parses a gauge widget record.
fn parse_gauge(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let ratio = parse_ratio(record, span)?;
  Ok(UiNode::Gauge {
    ratio,
    label: optional_string(record, "label")?,
    block: parse_block(record, false)?,
    gauge_style: parse_style(record.get("gauge-style"))?,
  })
}

/// Parses a bar chart and its labelled values.
fn parse_bar_chart(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let bars = required(record, "bars", span)?
    .as_list()
    .map_err(|error| LabeledError::from_diagnostic(&error))?
    .iter()
    .map(|value| {
      let bar = as_record(value, "bar")?;
      Ok(BarSpec {
        label: required_string(bar, "label", value.span())?,
        value: required_u64(bar, "value", value.span())?,
        style: parse_style(bar.get("style"))?,
        value_style: parse_style(bar.get("value-style"))?,
      })
    })
    .collect::<Result<Vec<_>, LabeledError>>()?;
  let direction = parse_direction(record, span)?;
  Ok(UiNode::BarChart {
    bars,
    block: parse_block(record, false)?,
    direction,
    max: optional_u64(record, "max")?,
    bar_width: optional_u16(record, "bar-width")?.unwrap_or(1),
    bar_gap: optional_u16(record, "bar-gap")?.unwrap_or(1),
    bar_style: parse_style(record.get("bar-style"))?,
    value_style: parse_style(record.get("value-style"))?,
    label_style: parse_style(record.get("label-style"))?,
  })
}

/// Parses a monthly calendar with an explicit year and month.
fn parse_calendar(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let year_value = required(record, "year", span)?;
  let year = i32::try_from(
    year_value
      .as_int()
      .map_err(|error| LabeledError::from_diagnostic(&error))?,
  )
  .map_err(|_| config_error("`year` is outside the supported range", year_value.span()))?;
  if !(-9999..=9999).contains(&year) {
    return Err(config_error(
      "`year` must be between -9999 and 9999",
      year_value.span(),
    ));
  }
  let month = required_u64(record, "month", span)?;
  if !(1..=12).contains(&month) {
    return Err(config_error(
      "`month` must be between 1 and 12",
      required(record, "month", span)?.span(),
    ));
  }
  Ok(UiNode::Calendar {
    year,
    month: month as u8,
    block: parse_block(record, false)?,
    show_month: optional_style(record, "month-style")?,
    show_weekdays: optional_style(record, "weekday-style")?,
    show_surrounding: optional_style(record, "surrounding-style")?,
    default_style: parse_style(record.get("style"))?,
  })
}

/// Parses a canvas containing colored point records.
fn parse_canvas(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let points = required(record, "points", span)?
    .as_list()
    .map_err(|error| LabeledError::from_diagnostic(&error))?
    .iter()
    .map(|value| {
      let point = as_record(value, "canvas point")?;
      Ok(CanvasPoint {
        x: required_number(point, "x", value.span())?,
        y: required_number(point, "y", value.span())?,
        color: point
          .get("color")
          .map(parse_color)
          .transpose()?
          .unwrap_or(Color::Reset),
      })
    })
    .collect::<Result<Vec<_>, LabeledError>>()?;
  Ok(UiNode::Canvas {
    points,
    x_bounds: optional_bounds(record, "x-bounds")?.unwrap_or([-10.0, 10.0]),
    y_bounds: optional_bounds(record, "y-bounds")?.unwrap_or([-10.0, 10.0]),
    marker: parse_marker(record.get("marker"))?,
    background: record
      .get("background-color")
      .map(parse_color)
      .transpose()?
      .unwrap_or(Color::Reset),
    block: parse_block(record, false)?,
  })
}

/// Parses a cartesian chart with axes and datasets.
fn parse_chart(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let datasets = required(record, "datasets", span)?
    .as_list()
    .map_err(|error| LabeledError::from_diagnostic(&error))?
    .iter()
    .map(parse_dataset)
    .collect::<Result<Vec<_>, _>>()?;
  Ok(UiNode::Chart {
    datasets,
    x_axis: parse_axis(record.get("x-axis"), "x-axis")?,
    y_axis: parse_axis(record.get("y-axis"), "y-axis")?,
    block: parse_block(record, false)?,
    style: parse_style(record.get("style"))?,
  })
}

/// Parses one chart dataset record.
fn parse_dataset(value: &Value) -> Result<DatasetSpec, LabeledError> {
  let record = as_record(value, "dataset")?;
  let data = required(record, "data", value.span())?
    .as_list()
    .map_err(|error| LabeledError::from_diagnostic(&error))?
    .iter()
    .map(|point| {
      let coordinates = point
        .as_list()
        .map_err(|error| LabeledError::from_diagnostic(&error))?;
      if coordinates.len() != 2 {
        return Err(config_error(
          "chart points must contain `[x y]`",
          point.span(),
        ));
      }
      Ok((
        parse_number(&coordinates[0])?,
        parse_number(&coordinates[1])?,
      ))
    })
    .collect::<Result<Vec<_>, LabeledError>>()?;
  let graph_type = match optional_string(record, "graph-type")?.as_deref() {
    None | Some("scatter") => GraphType::Scatter,
    Some("line") => GraphType::Line,
    Some("bar") => GraphType::Bar,
    Some("area") => GraphType::Area,
    Some(other) => {
      return Err(config_error(
        format!("unsupported graph type `{other}`"),
        value.span(),
      ));
    },
  };
  Ok(DatasetSpec {
    name: optional_string(record, "name")?,
    data,
    graph_type,
    marker: parse_marker(record.get("marker"))?,
    style: parse_style(record.get("style"))?,
  })
}

/// Parses one optional chart axis record.
fn parse_axis(value: Option<&Value>, name: &str) -> Result<AxisSpec, LabeledError> {
  let Some(value) = value else {
    return Ok(AxisSpec {
      title: None,
      bounds: [0.0, 100.0],
      labels: Vec::new(),
      style: Style::default(),
    });
  };
  let record = as_record(value, name)?;
  Ok(AxisSpec {
    title: optional_string(record, "title")?,
    bounds: optional_bounds(record, "bounds")?.unwrap_or([0.0, 100.0]),
    labels: optional_string_list(record, "labels")?.unwrap_or_default(),
    style: parse_style(record.get("style"))?,
  })
}

/// Parses a one-line gauge.
fn parse_line_gauge(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  Ok(UiNode::LineGauge {
    ratio: parse_ratio(record, span)?,
    label: optional_string(record, "label")?,
    block: parse_block(record, false)?,
    filled_style: parse_style(record.get("filled-style"))?,
    unfilled_style: parse_style(record.get("unfilled-style"))?,
    filled_symbol: optional_string(record, "filled-symbol")?.unwrap_or_else(|| "─".into()),
    unfilled_symbol: optional_string(record, "unfilled-symbol")?.unwrap_or_else(|| "─".into()),
  })
}

/// Parses the Ratatui logo size.
fn parse_logo(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let small = match optional_string(record, "size")?.as_deref() {
    None | Some("tiny") => false,
    Some("small") => true,
    Some(other) => {
      return Err(config_error(
        format!("unsupported logo size `{other}`"),
        span,
      ));
    },
  };
  Ok(UiNode::Logo { small })
}

/// Parses a stateful scrollbar record.
fn parse_scrollbar(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let orientation = match optional_string(record, "orientation")?.as_deref() {
    None | Some("vertical-right") => ScrollbarOrientation::VerticalRight,
    Some("vertical-left") => ScrollbarOrientation::VerticalLeft,
    Some("horizontal-bottom") => ScrollbarOrientation::HorizontalBottom,
    Some("horizontal-top") => ScrollbarOrientation::HorizontalTop,
    Some(other) => {
      return Err(config_error(
        format!("unsupported scrollbar orientation `{other}`"),
        span,
      ));
    },
  };
  Ok(UiNode::Scrollbar {
    content_length: required_usize(record, "content-length", span)?,
    position: optional_usize(record, "position")?.unwrap_or(0),
    viewport_length: optional_usize(record, "viewport-length")?.unwrap_or(0),
    orientation,
    thumb_style: parse_style(record.get("thumb-style"))?,
    track_style: parse_style(record.get("track-style"))?,
  })
}

/// Parses a sparkline with optional missing values.
fn parse_sparkline(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let data = required(record, "data", span)?
    .as_list()
    .map_err(|error| LabeledError::from_diagnostic(&error))?
    .iter()
    .map(|value| match value {
      Value::Nothing { .. } => Ok(None),
      _ => value
        .as_int()
        .map_err(|error| LabeledError::from_diagnostic(&error))
        .and_then(|number| {
          u64::try_from(number)
            .map(Some)
            .map_err(|_| config_error("sparkline values must be non-negative", value.span()))
        }),
    })
    .collect::<Result<Vec<_>, _>>()?;
  let right_to_left = match optional_string(record, "direction")?.as_deref() {
    None | Some("left-to-right") => false,
    Some("right-to-left") => true,
    Some(other) => {
      return Err(config_error(
        format!("unsupported sparkline direction `{other}`"),
        span,
      ));
    },
  };
  Ok(UiNode::Sparkline {
    data,
    max: optional_u64(record, "max")?,
    right_to_left,
    block: parse_block(record, false)?,
    style: parse_style(record.get("style"))?,
    absent_style: parse_style(record.get("absent-style"))?,
    absent_symbol: optional_string(record, "absent-symbol")?.unwrap_or_else(|| " ".into()),
  })
}

/// Parses table rows, header, widths, and presentation.
fn parse_table(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let rows = parse_string_rows(required(record, "rows", span)?)?;
  let header = record.get("header").map(parse_string_list).transpose()?;
  let column_count = header
    .as_ref()
    .map(Vec::len)
    .or_else(|| rows.first().map(Vec::len))
    .unwrap_or(0);
  if rows.iter().any(|row| row.len() != column_count) {
    return Err(config_error(
      "every table row must have the same number of cells",
      span,
    ));
  }
  if header.as_ref().is_some_and(|row| row.len() != column_count) {
    return Err(config_error("table header width must match its rows", span));
  }
  let widths = match record.get("widths") {
    Some(value) => value
      .as_list()
      .map_err(|error| LabeledError::from_diagnostic(&error))?
      .iter()
      .map(parse_constraint)
      .collect::<Result<Vec<_>, _>>()?,
    None => vec![Constraint::Fill(1); column_count],
  };
  if widths.len() != column_count {
    return Err(config_error(
      "`widths` must contain one constraint per table column",
      span,
    ));
  }
  Ok(UiNode::Table {
    rows,
    header,
    widths,
    column_spacing: optional_u16(record, "column-spacing")?.unwrap_or(1),
    block: parse_block(record, false)?,
    style: parse_style(record.get("style"))?,
  })
}

/// Parses a tab bar.
fn parse_tabs(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let titles = parse_string_list(required(record, "titles", span)?)?;
  let selected = optional_usize(record, "selected")?;
  if selected.is_some_and(|index| index >= titles.len()) {
    return Err(config_error("`selected` must index an existing tab", span));
  }
  Ok(UiNode::Tabs {
    titles,
    selected,
    divider: optional_string(record, "divider")?.unwrap_or_else(|| "│".into()),
    block: parse_block(record, false)?,
    style: parse_style(record.get("style"))?,
    highlight_style: parse_style(record.get("highlight-style"))?,
  })
}

/// Parses a `tui-widgets` subcell bar graph.
fn parse_bar_graph(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let data = parse_number_list(required(record, "data", span)?, "data")?;
  if data.is_empty() {
    return Err(config_error("`data` must not be empty", span));
  }
  let min = optional_number(record, "min")?;
  let max = optional_number(record, "max")?;
  if min.zip(max).is_some_and(|(min, max)| min >= max) {
    return Err(config_error("`min` must be less than `max`", span));
  }
  let bar_style = match optional_string(record, "bar-style")?.as_deref() {
    None | Some("braille") => SuiteBarStyle::Braille,
    Some("solid") => SuiteBarStyle::Solid,
    Some("quadrant") => SuiteBarStyle::Quadrant,
    Some("octant") => SuiteBarStyle::Octant,
    Some(other) => {
      return Err(config_error(
        format!("unsupported bar style `{other}`"),
        span,
      ));
    },
  };
  Ok(UiNode::Suite(SuiteWidget::BarGraph {
    data,
    min,
    max,
    bar_style,
  }))
}

/// Parses oversized pixel text from the widget suite.
fn parse_big_text(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let pixel_size = match optional_string(record, "pixel-size")?.as_deref() {
    None | Some("full") => PixelSize::Full,
    Some("half-height") => PixelSize::HalfHeight,
    Some("half-width") => PixelSize::HalfWidth,
    Some("quadrant") => PixelSize::Quadrant,
    Some("third-height") => PixelSize::ThirdHeight,
    Some("sextant") => PixelSize::Sextant,
    Some("quarter-height") => PixelSize::QuarterHeight,
    Some("octant") => PixelSize::Octant,
    Some(other) => {
      return Err(config_error(
        format!("unsupported pixel size `{other}`"),
        span,
      ));
    },
  };
  Ok(UiNode::Suite(SuiteWidget::BigText {
    text: required_string(record, "text", span)?,
    block: parse_block(record, false)?,
    alignment: parse_alignment(record, span)?,
    pixel_size,
  }))
}

/// Parses one character rendered with box-drawing glyphs.
fn parse_box_text(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let text = required_string(record, "character", span)?;
  let mut characters = text.chars();
  let character = characters
    .next()
    .ok_or_else(|| config_error("`character` must not be empty", span))?;
  if characters.next().is_some() {
    return Err(config_error(
      "`character` must contain exactly one character",
      span,
    ));
  }
  Ok(UiNode::Suite(SuiteWidget::BoxText { character }))
}

/// Parses a standard playing card.
fn parse_card(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let rank = match required_string(record, "rank", span)?
    .to_ascii_lowercase()
    .as_str()
  {
    "ace" | "a" => Rank::Ace,
    "two" | "2" => Rank::Two,
    "three" | "3" => Rank::Three,
    "four" | "4" => Rank::Four,
    "five" | "5" => Rank::Five,
    "six" | "6" => Rank::Six,
    "seven" | "7" => Rank::Seven,
    "eight" | "8" => Rank::Eight,
    "nine" | "9" => Rank::Nine,
    "ten" | "10" => Rank::Ten,
    "jack" | "j" => Rank::Jack,
    "queen" | "q" => Rank::Queen,
    "king" | "k" => Rank::King,
    other => {
      return Err(config_error(
        format!("unsupported card rank `{other}`"),
        span,
      ));
    },
  };
  let suit = match required_string(record, "suit", span)?
    .to_ascii_lowercase()
    .as_str()
  {
    "spades" | "spade" => Suit::Spades,
    "hearts" | "heart" => Suit::Hearts,
    "diamonds" | "diamond" => Suit::Diamonds,
    "clubs" | "club" => Suit::Clubs,
    other => {
      return Err(config_error(
        format!("unsupported card suit `{other}`"),
        span,
      ));
    },
  };
  Ok(UiNode::Suite(SuiteWidget::Card { rank, suit }))
}

/// Parses normalized equalizer band levels.
fn parse_equalizer(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let bands = parse_number_list(required(record, "bands", span)?, "bands")?;
  if bands.iter().any(|value| !(0.0..=1.0).contains(value)) {
    return Err(config_error(
      "equalizer bands must be between 0 and 1",
      span,
    ));
  }
  let brightness = optional_number(record, "brightness")?.unwrap_or(1.0);
  if !(0.0..=1.0).contains(&brightness) {
    return Err(config_error("`brightness` must be between 0 and 1", span));
  }
  Ok(UiNode::Suite(SuiteWidget::Equalizer { bands, brightness }))
}

/// Parses an automatically centered textual popup.
fn parse_popup(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let width = optional_usize(record, "width")?;
  let height = optional_usize(record, "height")?;
  if width == Some(0) || height == Some(0) {
    return Err(config_error("popup dimensions must be at least 1", span));
  }
  Ok(UiNode::Suite(SuiteWidget::Popup {
    text: required_string(record, "text", span)?,
    block: parse_block(record, true)?,
    width,
    height,
  }))
}

/// Parses a declarative snapshot of a text prompt.
fn parse_text_prompt(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let render_style = match optional_string(record, "render-style")?.as_deref() {
    None | Some("default") => TextRenderStyle::Default,
    Some("password") => TextRenderStyle::Password,
    Some("invisible") => TextRenderStyle::Invisible,
    Some(other) => {
      return Err(config_error(
        format!("unsupported prompt render style `{other}`"),
        span,
      ));
    },
  };
  Ok(UiNode::Suite(SuiteWidget::TextPrompt {
    message: required_string(record, "message", span)?,
    value: optional_string(record, "value")?.unwrap_or_default(),
    status: parse_prompt_status(record, span)?,
    focus: parse_prompt_focus(record)?,
    render_style,
    show_status: !optional_bool(record, "hide-status")?.unwrap_or(false),
    block: parse_block(record, false)?,
  }))
}

/// Parses a declarative snapshot of a select prompt.
fn parse_select_prompt(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let options = parse_string_list(required(record, "options", span)?)?;
  let selected = optional_usize(record, "selected")?.unwrap_or(0);
  if !options.is_empty() && selected >= options.len() {
    return Err(config_error("`selected` must identify an option", span));
  }
  Ok(UiNode::Suite(SuiteWidget::SelectPrompt {
    label: required_string(record, "label", span)?,
    options,
    selected,
    status: parse_prompt_status(record, span)?,
    focus: parse_prompt_focus(record)?,
    block: parse_block(record, false)?,
  }))
}

/// Parses and validates QR payload and rendering options.
fn parse_qr_code(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let data = required_string(record, "data", span)?;
  qrcode::QrCode::new(data.as_bytes())
    .map_err(|error| config_error(format!("failed to encode QR data: {error}"), span))?;
  let scale_width = optional_u16(record, "scale-width")?.unwrap_or(1);
  let scale_height = optional_u16(record, "scale-height")?.unwrap_or(1);
  if scale_width == 0 || scale_height == 0 {
    return Err(config_error("QR scale dimensions must be at least 1", span));
  }
  let scaling = match optional_string(record, "scaling")?.as_deref() {
    None | Some("exact") => QrScaling::Exact(scale_width, scale_height),
    Some("min") => QrScaling::Min,
    Some("max") => QrScaling::Max,
    Some(other) => {
      return Err(config_error(
        format!("unsupported QR scaling `{other}`"),
        span,
      ));
    },
  };
  Ok(UiNode::Suite(SuiteWidget::QrCode {
    data,
    quiet_zone: if optional_bool(record, "no-quiet-zone")?.unwrap_or(false) {
      QuietZone::Disabled
    } else {
      QuietZone::Enabled
    },
    scaling,
    colors: if optional_bool(record, "inverted")?.unwrap_or(false) {
      QrColors::Inverted
    } else {
      QrColors::Normal
    },
    style: parse_style(record.get("style"))?,
  }))
}

/// Parses the suite's fractional scrollbar without replacing Ratatui's native scrollbar.
fn parse_fractional_scrollbar(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
  let orientation = match optional_string(record, "orientation")?.as_deref() {
    None | Some("vertical") => FractionalScrollbarOrientation::Vertical,
    Some("horizontal") => FractionalScrollbarOrientation::Horizontal,
    Some(other) => {
      return Err(config_error(
        format!("unsupported scrollbar orientation `{other}`"),
        span,
      ));
    },
  };
  let arrows = match optional_string(record, "arrows")?.as_deref() {
    None | Some("none") => ScrollBarArrows::None,
    Some("start") => ScrollBarArrows::Start,
    Some("end") => ScrollBarArrows::End,
    Some("both") => ScrollBarArrows::Both,
    Some(other) => {
      return Err(config_error(
        format!("unsupported scrollbar arrows `{other}`"),
        span,
      ));
    },
  };
  let glyphs = match optional_string(record, "glyphs")?.as_deref() {
    None | Some("legacy") => FractionalGlyphs::Legacy,
    Some("minimal") => FractionalGlyphs::Minimal,
    Some("box-drawing") => FractionalGlyphs::BoxDrawing,
    Some("unicode") => FractionalGlyphs::Unicode,
    Some(other) => {
      return Err(config_error(
        format!("unsupported scrollbar glyphs `{other}`"),
        span,
      ));
    },
  };
  Ok(UiNode::Suite(SuiteWidget::FractionalScrollbar {
    content_length: required_usize(record, "content-length", span)?,
    viewport_length: required_usize(record, "viewport-length", span)?,
    position: optional_usize(record, "position")?.unwrap_or(0),
    orientation,
    arrows,
    glyphs,
    track_style: parse_style(record.get("track-style"))?,
    thumb_style: parse_style(record.get("thumb-style"))?,
    arrow_style: parse_style(record.get("arrow-style"))?,
  }))
}

/// Parses a scrollable off-screen child buffer and explicit viewport offset.
fn parse_scroll_view(
  record: &Record,
  span: Span,
  effect_ids: &mut HashSet<String>,
) -> Result<UiNode, LabeledError> {
  let width =
    optional_u16(record, "width")?.ok_or_else(|| config_error("missing `width`", span))?;
  let height =
    optional_u16(record, "height")?.ok_or_else(|| config_error("missing `height`", span))?;
  if width == 0 || height == 0 {
    return Err(config_error(
      "scroll-view dimensions must be at least 1",
      span,
    ));
  }
  Ok(UiNode::Suite(SuiteWidget::ScrollView {
    width,
    height,
    scroll_x: optional_u16(record, "scroll-x")?.unwrap_or(0),
    scroll_y: optional_u16(record, "scroll-y")?.unwrap_or(0),
    vertical_scrollbar: parse_scrollbar_visibility(record, "vertical-scrollbar", span)?,
    horizontal_scrollbar: parse_scrollbar_visibility(record, "horizontal-scrollbar", span)?,
    child: Box::new(UiNode::parse_with_effect_ids(
      required(record, "child", span)?,
      effect_ids,
    )?),
  }))
}

/// Parses a prompt completion status shared by text and select prompts.
fn parse_prompt_status(record: &Record, span: Span) -> Result<Status, LabeledError> {
  match optional_string(record, "status")?.as_deref() {
    None | Some("pending") => Ok(Status::Pending),
    Some("done") => Ok(Status::Done),
    Some("aborted") => Ok(Status::Aborted),
    Some(other) => Err(config_error(
      format!("unsupported prompt status `{other}`"),
      span,
    )),
  }
}

/// Parses whether a declarative prompt snapshot is focused.
fn parse_prompt_focus(record: &Record) -> Result<FocusState, LabeledError> {
  Ok(if optional_bool(record, "focused")?.unwrap_or(false) {
    FocusState::Focused
  } else {
    FocusState::Unfocused
  })
}

/// Parses one scroll-view scrollbar visibility policy.
fn parse_scrollbar_visibility(
  record: &Record,
  field: &str,
  span: Span,
) -> Result<ScrollbarVisibility, LabeledError> {
  match optional_string(record, field)?.as_deref() {
    None | Some("automatic") => Ok(ScrollbarVisibility::Automatic),
    Some("always") => Ok(ScrollbarVisibility::Always),
    Some("never") => Ok(ScrollbarVisibility::Never),
    Some(other) => Err(config_error(
      format!("unsupported scrollbar visibility `{other}`"),
      span,
    )),
  }
}

/// Parses the common block and style fields for a widget.
fn parse_block(record: &Record, default_border: bool) -> Result<BlockSpec, LabeledError> {
  let has_border = optional_bool(record, "border")?.unwrap_or(default_border);
  let border_type = match optional_string(record, "border-type")?.as_deref() {
    None | Some("plain") => BorderType::Plain,
    Some("rounded") => BorderType::Rounded,
    Some("double") => BorderType::Double,
    Some("thick") => BorderType::Thick,
    Some(other) => {
      return Err(config_error(
        format!("unsupported border type `{other}`"),
        record
          .get("border-type")
          .map(Value::span)
          .unwrap_or(Span::unknown()),
      ));
    },
  };
  Ok(BlockSpec {
    title: optional_string(record, "title")?,
    borders: if has_border {
      Borders::ALL
    } else {
      Borders::NONE
    },
    border_type,
    style: parse_style(record.get("style"))?,
    border_style: parse_style(record.get("border-style"))?,
  })
}

/// Parses a widget alignment string.
fn parse_alignment(record: &Record, span: Span) -> Result<Alignment, LabeledError> {
  match optional_string(record, "alignment")?.as_deref() {
    None | Some("left") => Ok(Alignment::Left),
    Some("center") => Ok(Alignment::Center),
    Some("right") => Ok(Alignment::Right),
    Some(other) => Err(config_error(
      format!("unsupported alignment `{other}`"),
      record.get("alignment").map(Value::span).unwrap_or(span),
    )),
  }
}

/// Parses vertical or horizontal direction strings.
fn parse_direction(record: &Record, span: Span) -> Result<Direction, LabeledError> {
  match optional_string(record, "direction")?.as_deref() {
    None | Some("vertical") => Ok(Direction::Vertical),
    Some("horizontal") => Ok(Direction::Horizontal),
    Some(other) => Err(config_error(
      format!("unsupported direction `{other}`"),
      record.get("direction").map(Value::span).unwrap_or(span),
    )),
  }
}

/// Parses and validates a gauge ratio.
fn parse_ratio(record: &Record, span: Span) -> Result<f64, LabeledError> {
  let value = required(record, "ratio", span)?;
  let ratio = parse_number(value)?;
  if !(0.0..=1.0).contains(&ratio) {
    return Err(config_error(
      "`ratio` must be between 0 and 1",
      value.span(),
    ));
  }
  Ok(ratio)
}

/// Parses a numeric Nushell value as a finite float.
fn parse_number(value: &Value) -> Result<f64, LabeledError> {
  let number = match value {
    Value::Float { val, .. } => *val,
    Value::Int { val, .. } => *val as f64,
    _ => return Err(config_error("expected a number", value.span())),
  };
  if !number.is_finite() {
    return Err(config_error("numbers must be finite", value.span()));
  }
  Ok(number)
}

/// Parses a required numeric record field.
fn required_number(record: &Record, field: &str, span: Span) -> Result<f64, LabeledError> {
  parse_number(required(record, field, span)?)
}

/// Parses a list containing only finite integer or float values.
fn parse_number_list(value: &Value, context: &str) -> Result<Vec<f64>, LabeledError> {
  value
    .as_list()
    .map_err(|_| {
      config_error(
        format!("`{context}` must be a list of numbers"),
        value.span(),
      )
    })?
    .iter()
    .map(parse_number)
    .collect()
}

/// Returns an optional finite numeric field.
fn optional_number(record: &Record, field: &str) -> Result<Option<f64>, LabeledError> {
  record.get(field).map(parse_number).transpose()
}

/// Parses a two-number bound list and requires increasing values.
fn optional_bounds(record: &Record, field: &str) -> Result<Option<[f64; 2]>, LabeledError> {
  record
    .get(field)
    .map(|value| {
      let values = value
        .as_list()
        .map_err(|error| LabeledError::from_diagnostic(&error))?;
      if values.len() != 2 {
        return Err(config_error(
          format!("`{field}` must contain `[min max]`"),
          value.span(),
        ));
      }
      let bounds = [parse_number(&values[0])?, parse_number(&values[1])?];
      if bounds[0] >= bounds[1] {
        return Err(config_error(
          format!("`{field}` minimum must be below its maximum"),
          value.span(),
        ));
      }
      Ok(bounds)
    })
    .transpose()
}

/// Parses one of Ratatui's built-in plotting markers.
fn parse_marker(value: Option<&Value>) -> Result<Marker, LabeledError> {
  let Some(value) = value else {
    return Ok(Marker::Braille);
  };
  match value
    .as_str()
    .map_err(|error| LabeledError::from_diagnostic(&error))?
  {
    "dot" => Ok(Marker::Dot),
    "block" => Ok(Marker::Block),
    "bar" => Ok(Marker::Bar),
    "braille" => Ok(Marker::Braille),
    "half-block" => Ok(Marker::HalfBlock),
    "quadrant" => Ok(Marker::Quadrant),
    "sextant" => Ok(Marker::Sextant),
    "octant" => Ok(Marker::Octant),
    other => Err(config_error(
      format!("unsupported marker `{other}`"),
      value.span(),
    )),
  }
}

/// Parses a string list from one value.
fn parse_string_list(value: &Value) -> Result<Vec<String>, LabeledError> {
  value
    .as_list()
    .map_err(|error| LabeledError::from_diagnostic(&error))?
    .iter()
    .map(|value| {
      value
        .as_str()
        .map(str::to_owned)
        .map_err(|error| LabeledError::from_diagnostic(&error))
    })
    .collect()
}

/// Parses an optional string-list field.
fn optional_string_list(record: &Record, field: &str) -> Result<Option<Vec<String>>, LabeledError> {
  record.get(field).map(parse_string_list).transpose()
}

/// Parses a nested list of table cell strings.
fn parse_string_rows(value: &Value) -> Result<Vec<Vec<String>>, LabeledError> {
  value
    .as_list()
    .map_err(|error| LabeledError::from_diagnostic(&error))?
    .iter()
    .map(parse_string_list)
    .collect()
}

/// Parses a style record containing colors and text modifiers.
pub(crate) fn parse_style(value: Option<&Value>) -> Result<Style, LabeledError> {
  let Some(value) = value else {
    return Ok(Style::default());
  };
  let record = as_record(value, "style")?;
  let mut style = Style::default();
  if let Some(value) = record.get("fg") {
    style = style.fg(parse_color(value)?);
  }
  if let Some(value) = record.get("bg") {
    style = style.bg(parse_color(value)?);
  }
  for (field, modifier) in [
    ("bold", Modifier::BOLD),
    ("italic", Modifier::ITALIC),
    ("underlined", Modifier::UNDERLINED),
    ("reversed", Modifier::REVERSED),
    ("dim", Modifier::DIM),
  ] {
    if optional_bool(record, field)?.unwrap_or(false) {
      style = style.add_modifier(modifier);
    }
  }
  Ok(style)
}

/// Distinguishes a missing style field from a supplied empty style.
fn optional_style(record: &Record, field: &str) -> Result<Option<Style>, LabeledError> {
  record
    .get(field)
    .map(|value| parse_style(Some(value)))
    .transpose()
}

/// Parses a named, indexed, or hexadecimal Ratatui color.
pub(crate) fn parse_color(value: &Value) -> Result<Color, LabeledError> {
  match value {
    Value::Int { val, .. } => u8::try_from(*val)
      .map(Color::Indexed)
      .map_err(|_| config_error("indexed colors must be between 0 and 255", value.span())),
    Value::String { val, .. } => Color::from_str(val)
      .map_err(|_| config_error(format!("unsupported color `{val}`"), value.span())),
    _ => Err(config_error(
      "colors must be a name, #RRGGBB string, or 0-255 index",
      value.span(),
    )),
  }
}

/// Returns a required field or a source-labelled configuration error.
fn required<'a>(record: &'a Record, field: &str, span: Span) -> Result<&'a Value, LabeledError> {
  record
    .get(field)
    .ok_or_else(|| config_error(format!("missing required field `{field}`"), span))
}

/// Returns a required string field.
fn required_string(record: &Record, field: &str, span: Span) -> Result<String, LabeledError> {
  required(record, field, span)?
    .as_str()
    .map(str::to_owned)
    .map_err(|error| LabeledError::from_diagnostic(&error))
}

/// Returns an optional string field.
fn optional_string(record: &Record, field: &str) -> Result<Option<String>, LabeledError> {
  record
    .get(field)
    .map(|value| {
      value
        .as_str()
        .map(str::to_owned)
        .map_err(|error| LabeledError::from_diagnostic(&error))
    })
    .transpose()
}

/// Returns an optional boolean field.
fn optional_bool(record: &Record, field: &str) -> Result<Option<bool>, LabeledError> {
  record
    .get(field)
    .map(|value| {
      value
        .as_bool()
        .map_err(|error| LabeledError::from_diagnostic(&error))
    })
    .transpose()
}

/// Returns an optional integer field.
fn optional_int(record: &Record, field: &str) -> Result<Option<i64>, LabeledError> {
  record
    .get(field)
    .map(|value| {
      value
        .as_int()
        .map_err(|error| LabeledError::from_diagnostic(&error))
    })
    .transpose()
}

/// Returns a required non-negative integer field.
fn required_u64(record: &Record, field: &str, span: Span) -> Result<u64, LabeledError> {
  let value = required(record, field, span)?;
  let number = value
    .as_int()
    .map_err(|error| LabeledError::from_diagnostic(&error))?;
  u64::try_from(number)
    .map_err(|_| config_error(format!("`{field}` must be non-negative"), value.span()))
}

/// Returns an optional non-negative integer field.
fn optional_u64(record: &Record, field: &str) -> Result<Option<u64>, LabeledError> {
  record
    .get(field)
    .map(|value| {
      let number = value
        .as_int()
        .map_err(|error| LabeledError::from_diagnostic(&error))?;
      u64::try_from(number)
        .map_err(|_| config_error(format!("`{field}` must be non-negative"), value.span()))
    })
    .transpose()
}

/// Returns an optional integer that fits Ratatui's cell dimensions.
fn optional_u16(record: &Record, field: &str) -> Result<Option<u16>, LabeledError> {
  optional_u64(record, field)?
    .map(|number| {
      u16::try_from(number).map_err(|_| {
        config_error(
          format!("`{field}` must be at most 65535"),
          record.get(field).expect("field exists").span(),
        )
      })
    })
    .transpose()
}

/// Returns a required integer usable as an in-memory collection size.
fn required_usize(record: &Record, field: &str, span: Span) -> Result<usize, LabeledError> {
  usize::try_from(required_u64(record, field, span)?).map_err(|_| {
    config_error(
      format!("`{field}` is too large for this platform"),
      required(record, field, span).expect("field exists").span(),
    )
  })
}

/// Returns an optional integer usable as an in-memory collection size.
fn optional_usize(record: &Record, field: &str) -> Result<Option<usize>, LabeledError> {
  optional_u64(record, field)?
    .map(|number| {
      usize::try_from(number).map_err(|_| {
        config_error(
          format!("`{field}` is too large for this platform"),
          record.get(field).expect("field exists").span(),
        )
      })
    })
    .transpose()
}

/// Returns an optional closure field with its original span.
fn optional_handler(record: &Record, field: &str) -> Result<Option<Handler>, LabeledError> {
  record
    .get(field)
    .map(|value| {
      value
        .as_closure()
        .cloned()
        .map(|item| Spanned {
          item,
          span: value.span(),
        })
        .map_err(|error| LabeledError::from_diagnostic(&error))
    })
    .transpose()
}

/// Converts a Nushell value to a record with contextual diagnostics.
fn as_record<'a>(value: &'a Value, context: &str) -> Result<&'a Record, LabeledError> {
  value.as_record().map_err(|_| {
    config_error(
      format!("{context} must be a record, found {}", value.get_type()),
      value.span(),
    )
  })
}

/// Creates a consistently labelled configuration error.
fn config_error(message: impl Into<String>, span: Span) -> LabeledError {
  LabeledError::new("Invalid TUI configuration")
    .with_label(message, span)
    .with_code("nu_plugin_tui::invalid_config")
}

#[cfg(test)]
mod tests {
  use nu_protocol::{Record, Span, Value};

  use super::{AppConfig, UiNode};

  /// Builds a test record value from concise key/value pairs.
  fn record(fields: Vec<(&str, Value)>) -> Value {
    let mut record = Record::new();
    for (name, value) in fields {
      record.push(name, value);
    }
    Value::test_record(record)
  }

  /// Verifies parsing of the smallest valid static application.
  #[test]
  fn parses_minimal_application() {
    let view = record(vec![
      ("type", Value::test_string("paragraph")),
      ("text", Value::test_string("hello")),
    ]);
    let config = record(vec![("view", view)]);

    let parsed = AppConfig::parse(&config, Span::test_data()).expect("valid config");

    assert!(matches!(parsed.view, Value::Record { .. }));
    assert!(parsed.quit_on_esc);
    assert_eq!(parsed.tick_rate.as_millis(), 250);
  }

  /// Verifies nested layouts default to one fill constraint per child.
  #[test]
  fn parses_layout_with_default_constraints() {
    let child = record(vec![("type", Value::test_string("spacer"))]);
    let layout = record(vec![
      ("type", Value::test_string("layout")),
      ("children", Value::test_list(vec![child.clone(), child])),
    ]);

    let parsed = UiNode::parse(&layout).expect("valid layout");

    match parsed {
      UiNode::Layout {
        constraints,
        children,
        ..
      } => {
        assert_eq!(constraints.len(), 2);
        assert_eq!(children.len(), 2);
      },
      _ => panic!("expected layout"),
    }
  }

  /// Verifies that mismatched layout constraints produce a useful diagnostic.
  #[test]
  fn rejects_mismatched_constraints() {
    let child = record(vec![("type", Value::test_string("spacer"))]);
    let constraint = record(vec![("length", Value::test_int(1))]);
    let layout = record(vec![
      ("type", Value::test_string("layout")),
      ("children", Value::test_list(vec![child.clone(), child])),
      ("constraints", Value::test_list(vec![constraint])),
    ]);

    let error = UiNode::parse(&layout).expect_err("constraints should not match");

    assert!(error.labels[0].text.contains("one item per child"));
  }

  /// Verifies a standalone block is no longer part of the declarative widget schema.
  #[test]
  fn rejects_removed_block_widget() {
    let block = record(vec![("type", Value::test_string("block"))]);

    let error = UiNode::parse(&block).expect_err("block should be unsupported");

    assert!(
      error.labels[0]
        .text
        .contains("unsupported widget type `block`")
    );
  }

  /// Verifies paragraph scroll offsets fit Ratatui's terminal coordinate range.
  #[test]
  fn rejects_invalid_paragraph_scroll_offsets() {
    let negative = record(vec![
      ("type", Value::test_string("paragraph")),
      ("text", Value::test_string("preview")),
      ("scroll-x", Value::test_int(-1)),
    ]);
    let too_large = record(vec![
      ("type", Value::test_string("paragraph")),
      ("text", Value::test_string("preview")),
      ("scroll-y", Value::test_int(65_536)),
    ]);

    let negative_error = UiNode::parse(&negative).expect_err("negative scroll should fail");
    let too_large_error = UiNode::parse(&too_large).expect_err("large scroll should fail");

    assert!(
      negative_error.labels[0]
        .text
        .contains("must be non-negative")
    );
    assert!(
      too_large_error.labels[0]
        .text
        .contains("must be at most 65535")
    );
  }

  /// Verifies exact Ratatui struct names remain usable as widget aliases.
  #[test]
  fn parses_ratatui_widget_name_aliases() {
    let monthly = record(vec![
      ("type", Value::test_string("monthly")),
      ("year", Value::test_int(2026)),
      ("month", Value::test_int(8)),
    ]);
    let logo = record(vec![("type", Value::test_string("ratatui-logo"))]);
    let mascot = record(vec![("type", Value::test_string("ratatui-mascot"))]);

    UiNode::parse(&monthly).expect("monthly alias parses");
    UiNode::parse(&logo).expect("logo alias parses");
    UiNode::parse(&mascot).expect("mascot alias parses");
  }

  /// Verifies calendar years outside the upstream date range are rejected before rendering.
  #[test]
  fn rejects_out_of_range_calendar_year() {
    let calendar = record(vec![
      ("type", Value::test_string("calendar")),
      ("year", Value::test_int(10_000)),
      ("month", Value::test_int(1)),
    ]);

    let error = UiNode::parse(&calendar).expect_err("calendar year should be invalid");

    assert!(error.labels[0].text.contains("-9999 and 9999"));
  }

  /// Verifies every external suite widget has a parser-compatible minimal record.
  #[test]
  fn parses_every_tui_widgets_suite_record() {
    let spacer = record(vec![("type", Value::test_string("spacer"))]);
    let widgets = vec![
      record(vec![
        ("type", Value::test_string("bar-graph")),
        (
          "data",
          Value::test_list(vec![Value::test_float(0.25), Value::test_float(0.75)]),
        ),
      ]),
      record(vec![
        ("type", Value::test_string("big-text")),
        ("text", Value::test_string("TUI")),
      ]),
      record(vec![
        ("type", Value::test_string("box-text")),
        ("character", Value::test_string("R")),
      ]),
      record(vec![
        ("type", Value::test_string("card")),
        ("rank", Value::test_string("ace")),
        ("suit", Value::test_string("spades")),
      ]),
      record(vec![
        ("type", Value::test_string("equalizer")),
        (
          "bands",
          Value::test_list(vec![Value::test_float(0.2), Value::test_float(0.8)]),
        ),
      ]),
      record(vec![
        ("type", Value::test_string("popup")),
        ("text", Value::test_string("hello")),
      ]),
      record(vec![
        ("type", Value::test_string("text-prompt")),
        ("message", Value::test_string("Name")),
      ]),
      record(vec![
        ("type", Value::test_string("select-prompt")),
        ("label", Value::test_string("Language")),
        (
          "options",
          Value::test_list(vec![Value::test_string("Rust"), Value::test_string("Nu")]),
        ),
      ]),
      record(vec![
        ("type", Value::test_string("qr-code")),
        ("data", Value::test_string("https://ratatui.rs")),
      ]),
      record(vec![
        ("type", Value::test_string("fractional-scrollbar")),
        ("content-length", Value::test_int(100)),
        ("viewport-length", Value::test_int(20)),
      ]),
      record(vec![
        ("type", Value::test_string("scroll-view")),
        ("width", Value::test_int(40)),
        ("height", Value::test_int(20)),
        ("child", spacer),
      ]),
    ];

    for widget in widgets {
      assert!(matches!(UiNode::parse(&widget), Ok(UiNode::Suite(_))));
    }
  }
}
