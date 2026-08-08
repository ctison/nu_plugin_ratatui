use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, LabeledError, Record, Signature, SyntaxShape, Type, Value};

use crate::{config::UiNode, plugin::TuiPlugin};

/// A declarative widget supported by record constructors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WidgetKind {
  BarChart,
  Calendar,
  Canvas,
  Chart,
  Clear,
  Fill,
  Layout,
  Paragraph,
  Button,
  List,
  Gauge,
  LineGauge,
  Logo,
  Mascot,
  Scrollbar,
  Sparkline,
  Table,
  Tabs,
  BarGraph,
  BigText,
  BoxText,
  Card,
  Equalizer,
  Popup,
  TextPrompt,
  SelectPrompt,
  QrCode,
  FractionalScrollbar,
  ScrollView,
  Spacer,
}

impl WidgetKind {
  /// Contains every supported widget in command registration order.
  pub(crate) const ALL: [Self; 30] = [
    Self::BarChart,
    Self::Calendar,
    Self::Canvas,
    Self::Chart,
    Self::Clear,
    Self::Fill,
    Self::Layout,
    Self::Paragraph,
    Self::Button,
    Self::List,
    Self::Gauge,
    Self::LineGauge,
    Self::Logo,
    Self::Mascot,
    Self::Scrollbar,
    Self::Sparkline,
    Self::Table,
    Self::Tabs,
    Self::BarGraph,
    Self::BigText,
    Self::BoxText,
    Self::Card,
    Self::Equalizer,
    Self::Popup,
    Self::TextPrompt,
    Self::SelectPrompt,
    Self::QrCode,
    Self::FractionalScrollbar,
    Self::ScrollView,
    Self::Spacer,
  ];

  /// Returns the widget type stored in the emitted record.
  fn record_type(self) -> &'static str {
    match self {
      Self::BarChart => "bar-chart",
      Self::Calendar => "calendar",
      Self::Canvas => "canvas",
      Self::Chart => "chart",
      Self::Clear => "clear",
      Self::Fill => "fill",
      Self::Layout => "layout",
      Self::Paragraph => "paragraph",
      Self::Button => "button",
      Self::List => "list",
      Self::Gauge => "gauge",
      Self::LineGauge => "line-gauge",
      Self::Logo => "logo",
      Self::Mascot => "mascot",
      Self::Scrollbar => "scrollbar",
      Self::Sparkline => "sparkline",
      Self::Table => "table",
      Self::Tabs => "tabs",
      Self::BarGraph => "bar-graph",
      Self::BigText => "big-text",
      Self::BoxText => "box-text",
      Self::Card => "card",
      Self::Equalizer => "equalizer",
      Self::Popup => "popup",
      Self::TextPrompt => "text-prompt",
      Self::SelectPrompt => "select-prompt",
      Self::QrCode => "qr-code",
      Self::FractionalScrollbar => "fractional-scrollbar",
      Self::ScrollView => "scroll-view",
      Self::Spacer => "spacer",
    }
  }

  /// Returns the full constructor command name.
  fn command_name(self) -> &'static str {
    match self {
      Self::BarChart => "tui bar-chart",
      Self::Calendar => "tui calendar",
      Self::Canvas => "tui canvas",
      Self::Chart => "tui chart",
      Self::Clear => "tui clear",
      Self::Fill => "tui fill",
      Self::Layout => "tui layout",
      Self::Paragraph => "tui paragraph",
      Self::Button => "tui button",
      Self::List => "tui list",
      Self::Gauge => "tui gauge",
      Self::LineGauge => "tui line-gauge",
      Self::Logo => "tui logo",
      Self::Mascot => "tui mascot",
      Self::Scrollbar => "tui scrollbar",
      Self::Sparkline => "tui sparkline",
      Self::Table => "tui table",
      Self::Tabs => "tui tabs",
      Self::BarGraph => "tui bar-graph",
      Self::BigText => "tui big-text",
      Self::BoxText => "tui box-text",
      Self::Card => "tui card",
      Self::Equalizer => "tui equalizer",
      Self::Popup => "tui popup",
      Self::TextPrompt => "tui text-prompt",
      Self::SelectPrompt => "tui select-prompt",
      Self::QrCode => "tui qr-code",
      Self::FractionalScrollbar => "tui fractional-scrollbar",
      Self::ScrollView => "tui scroll-view",
      Self::Spacer => "tui spacer",
    }
  }

  /// Returns a short description for a widget record constructor.
  fn description(self) -> &'static str {
    match self {
      Self::BarChart => "Declare a bar chart widget",
      Self::Calendar => "Declare a monthly calendar widget",
      Self::Canvas => "Declare a canvas widget",
      Self::Chart => "Declare a chart widget",
      Self::Clear => "Declare a clear widget",
      Self::Fill => "Declare a fill widget",
      Self::Layout => "Declare a widget layout",
      Self::Paragraph => "Declare a paragraph widget",
      Self::Button => "Declare an interactive button widget",
      Self::List => "Declare a list widget",
      Self::Gauge => "Declare a gauge widget",
      Self::LineGauge => "Declare a line gauge widget",
      Self::Logo => "Declare a Ratatui logo widget",
      Self::Mascot => "Declare a Ratatui mascot widget",
      Self::Scrollbar => "Declare a scrollbar widget",
      Self::Sparkline => "Declare a sparkline widget",
      Self::Table => "Declare a table widget",
      Self::Tabs => "Declare a tabs widget",
      Self::BarGraph => "Declare a tui-widgets subcell bar graph",
      Self::BigText => "Declare tui-widgets oversized pixel text",
      Self::BoxText => "Declare a tui-widgets box-drawing character",
      Self::Card => "Declare a tui-widgets playing card",
      Self::Equalizer => "Declare a tui-widgets equalizer",
      Self::Popup => "Declare a tui-widgets centered popup",
      Self::TextPrompt => "Declare a tui-widgets text prompt snapshot",
      Self::SelectPrompt => "Declare a tui-widgets select prompt snapshot",
      Self::QrCode => "Declare a tui-widgets QR code",
      Self::FractionalScrollbar => "Declare a tui-widgets fractional scrollbar",
      Self::ScrollView => "Declare a tui-widgets scroll view",
      Self::Spacer => "Declare a spacer widget",
    }
  }

  /// Returns record field names supplied as required positional arguments.
  fn required_fields(self) -> &'static [&'static str] {
    match self {
      Self::BarChart => &["bars"],
      Self::Calendar => &["year", "month"],
      Self::Canvas => &["points"],
      Self::Chart => &["datasets"],
      Self::Layout => &["children"],
      Self::Paragraph => &["text"],
      Self::Button => &["id", "label"],
      Self::List => &["items"],
      Self::Gauge | Self::LineGauge => &["ratio"],
      Self::Scrollbar => &["content-length"],
      Self::Sparkline => &["data"],
      Self::Table => &["rows"],
      Self::Tabs => &["titles"],
      Self::BarGraph => &["data"],
      Self::BigText => &["text"],
      Self::BoxText => &["character"],
      Self::Card => &["rank", "suit"],
      Self::Equalizer => &["bands"],
      Self::Popup => &["text"],
      Self::TextPrompt => &["message"],
      Self::SelectPrompt => &["label", "options"],
      Self::QrCode => &["data"],
      Self::FractionalScrollbar => &["content-length", "viewport-length"],
      Self::ScrollView => &["width", "height", "child"],
      Self::Clear | Self::Fill | Self::Logo | Self::Mascot | Self::Spacer => &[],
    }
  }

  /// Adds required positional fields and optional named fields for this widget.
  fn signature(self, command_name: &str) -> Signature {
    let signature = Signature::build(command_name)
      .input_output_type(Type::Nothing, Type::Record(Default::default()))
      .category(Category::Experimental);
    match self {
      Self::BarChart => with_presentation(
        signature
          .required("bars", list(record_shape()), "Bar records")
          .named(
            "direction",
            SyntaxShape::String,
            "vertical or horizontal (default: vertical)",
            None,
          )
          .named(
            "max",
            SyntaxShape::Int,
            "Maximum chart value (default: largest bar value)",
            None,
          )
          .named(
            "bar-width",
            SyntaxShape::Int,
            "Bar width in cells (default: 1)",
            None,
          )
          .named(
            "bar-gap",
            SyntaxShape::Int,
            "Gap between bars (default: 1)",
            None,
          )
          .named(
            "bar-style",
            record_shape(),
            "Default bar style (default: terminal default)",
            None,
          )
          .named(
            "value-style",
            record_shape(),
            "Value style (default: terminal default)",
            None,
          )
          .named(
            "label-style",
            record_shape(),
            "Label style (default: terminal default)",
            None,
          ),
        false,
      ),
      Self::Calendar => with_presentation(
        signature
          .required("year", SyntaxShape::Int, "Calendar year")
          .required("month", SyntaxShape::Int, "Month from 1 through 12")
          .named(
            "month-style",
            record_shape(),
            "Month header style (default: hidden)",
            None,
          )
          .named(
            "weekday-style",
            record_shape(),
            "Weekday header style (default: hidden)",
            None,
          )
          .named(
            "surrounding-style",
            record_shape(),
            "Surrounding days style (default: hidden)",
            None,
          ),
        false,
      ),
      Self::Canvas => with_presentation(
        signature
          .required("points", list(record_shape()), "Canvas point records")
          .named(
            "x-bounds",
            list(SyntaxShape::Number),
            "Horizontal bounds (default: [-10 10])",
            None,
          )
          .named(
            "y-bounds",
            list(SyntaxShape::Number),
            "Vertical bounds (default: [-10 10])",
            None,
          )
          .named(
            "marker",
            SyntaxShape::String,
            "Point marker (default: braille)",
            None,
          )
          .named(
            "background-color",
            color_shape(),
            "Canvas background color (default: terminal default)",
            None,
          ),
        false,
      ),
      Self::Chart => with_presentation(
        signature
          .required("datasets", list(record_shape()), "Chart datasets")
          .named(
            "x-axis",
            record_shape(),
            "Horizontal axis (default: bounds [0 100], no labels)",
            None,
          )
          .named(
            "y-axis",
            record_shape(),
            "Vertical axis (default: bounds [0 100], no labels)",
            None,
          ),
        false,
      ),
      Self::Clear => signature,
      Self::Fill => signature
        .named(
          "symbol",
          SyntaxShape::String,
          "Repeated cell symbol (default: space)",
          None,
        )
        .named(
          "style",
          record_shape(),
          "Fill style (default: terminal default)",
          None,
        ),
      Self::Layout => signature
        .required("children", list(record_shape()), "Child widget records")
        .named(
          "direction",
          SyntaxShape::String,
          "vertical or horizontal (default: vertical)",
          None,
        )
        .named(
          "constraints",
          list(record_shape()),
          "One layout constraint per child (default: one {fill: 1} per child)",
          None,
        ),
      Self::Paragraph => with_presentation(
        signature
          .required("text", SyntaxShape::String, "Text to display")
          .named(
            "alignment",
            SyntaxShape::String,
            "Text alignment (default: left)",
            None,
          )
          .switch(
            "wrap",
            "Whether text wraps (default: true)",
            None,
          )
          .switch(
            "ansi",
            "Whether to interpret ANSI SGR styles (default: false)",
            None,
          )
          .named(
            "scroll-x",
            SyntaxShape::Int,
            "Horizontal scroll offset in cells (default: 0)",
            None,
          )
          .named(
            "scroll-y",
            SyntaxShape::Int,
            "Vertical scroll offset in lines (default: 0)",
            None,
          ),
        false,
      ),
      Self::Button => with_presentation(
        signature
          .required("id", SyntaxShape::String, "Event widget identifier")
          .required("label", SyntaxShape::String, "Button label")
          .named(
            "alignment",
            SyntaxShape::String,
            "Label alignment (default: left)",
            None,
          )
          .named(
            "on-click",
            handler_shape(),
            "Closure taking a mouse event record and returning null, state, or an action record (default: no handler)",
            None,
          ),
        true,
      ),
      Self::List => with_presentation(
        signature.required(
          "items",
          list(SyntaxShape::String),
          "List item strings",
        ),
        false,
      ),
      Self::Gauge => with_presentation(
        signature
          .required("ratio", SyntaxShape::Number, "Completion from 0 to 1")
          .named(
            "label",
            SyntaxShape::String,
            "Gauge label (default: Ratatui-generated percentage)",
            None,
          )
          .named(
            "gauge-style",
            record_shape(),
            "Filled area style (default: terminal default)",
            None,
          ),
        false,
      ),
      Self::LineGauge => with_presentation(
        signature
          .required("ratio", SyntaxShape::Number, "Completion from 0 to 1")
          .named(
            "label",
            SyntaxShape::String,
            "Gauge label (default: Ratatui-generated percentage)",
            None,
          )
          .named(
            "filled-style",
            record_shape(),
            "Filled line style (default: terminal default)",
            None,
          )
          .named(
            "unfilled-style",
            record_shape(),
            "Unfilled line style (default: terminal default)",
            None,
          )
          .named(
            "filled-symbol",
            SyntaxShape::String,
            "Filled symbol (default: ─)",
            None,
          )
          .named(
            "unfilled-symbol",
            SyntaxShape::String,
            "Unfilled symbol (default: ─)",
            None,
          ),
        false,
      ),
      Self::Logo => signature.named(
        "size",
        SyntaxShape::String,
        "tiny or small (default: tiny)",
        None,
      ),
      Self::Mascot => signature.switch(
        "blink",
        "Render the alternate eye (default: false)",
        None,
      ),
      Self::Scrollbar => signature
        .required("content-length", SyntaxShape::Int, "Total content length")
        .named(
          "position",
          SyntaxShape::Int,
          "Current offset (default: 0)",
          None,
        )
        .named(
          "viewport-length",
          SyntaxShape::Int,
          "Visible content length (default: 0, meaning track length)",
          None,
        )
        .named(
          "orientation",
          SyntaxShape::String,
          "Scrollbar orientation (default: vertical-right)",
          None,
        )
        .named(
          "thumb-style",
          record_shape(),
          "Scroll thumb style (default: terminal default)",
          None,
        )
        .named(
          "track-style",
          record_shape(),
          "Scroll track style (default: terminal default)",
          None,
        ),
      Self::Sparkline => with_presentation(
        signature
          .required("data", list(SyntaxShape::Any), "Integer or null samples")
          .named(
            "max",
            SyntaxShape::Int,
            "Maximum sample value (default: largest sample)",
            None,
          )
          .named(
            "direction",
            SyntaxShape::String,
            "Rendering direction (default: left-to-right)",
            None,
          )
          .named(
            "absent-symbol",
            SyntaxShape::String,
            "Missing sample symbol (default: space)",
            None,
          )
          .named(
            "absent-style",
            record_shape(),
            "Missing sample style (default: terminal default)",
            None,
          ),
        false,
      ),
      Self::Table => with_presentation(
        signature
          .required("rows", list(list(SyntaxShape::String)), "Table rows")
          .named(
            "header",
            list(SyntaxShape::String),
            "Header cells (default: no header)",
            None,
          )
          .named(
            "widths",
            list(record_shape()),
            "Column constraints (default: one {fill: 1} per column)",
            None,
          )
          .named(
            "column-spacing",
            SyntaxShape::Int,
            "Column spacing (default: 1)",
            None,
          ),
        false,
      ),
      Self::Tabs => with_presentation(
        signature
          .required("titles", list(SyntaxShape::String), "Tab titles")
          .named(
            "selected",
            SyntaxShape::Int,
            "Selected tab index (default: none)",
            None,
          )
          .named(
            "divider",
            SyntaxShape::String,
            "Title divider (default: │)",
            None,
          )
          .named(
            "highlight-style",
            record_shape(),
            "Selected title style (default: terminal default)",
            None,
          ),
        false,
      ),
      Self::BarGraph => signature
        .required("data", list(SyntaxShape::Number), "Normalized or scaled samples")
        .named("min", SyntaxShape::Number, "Lower graph bound (default: smallest sample)", None)
        .named("max", SyntaxShape::Number, "Upper graph bound (default: largest sample)", None)
        .named(
          "bar-style",
          SyntaxShape::String,
          "braille, solid, quadrant, or octant (default: braille)",
          None,
        ),
      Self::BigText => with_presentation(
        signature
          .required("text", SyntaxShape::String, "Text rendered with an 8x8 pixel font")
          .named(
            "alignment",
            SyntaxShape::String,
            "Text alignment (default: left)",
            None,
          )
          .named(
            "pixel-size",
            SyntaxShape::String,
            "full, half-height, half-width, quadrant, third-height, sextant, quarter-height, or octant (default: full)",
            None,
          ),
        false,
      ),
      Self::BoxText => signature.required(
        "character",
        SyntaxShape::String,
        "One character rendered with box-drawing glyphs",
      ),
      Self::Card => signature
        .required("rank", SyntaxShape::String, "Card rank from ace through king")
        .required("suit", SyntaxShape::String, "spades, hearts, diamonds, or clubs"),
      Self::Equalizer => signature
        .required("bands", list(SyntaxShape::Number), "Band levels from 0 to 1")
        .named(
          "brightness",
          SyntaxShape::Number,
          "Color brightness from 0 to 1 (default: 1)",
          None,
        ),
      Self::Popup => with_presentation(
        signature
          .required("text", SyntaxShape::String, "Popup body text")
          .named(
            "width",
            SyntaxShape::Int,
            "Fixed inner width (default: measured from text)",
            None,
          )
          .named(
            "height",
            SyntaxShape::Int,
            "Fixed inner height (default: measured from text)",
            None,
          ),
        true,
      ),
      Self::TextPrompt => with_presentation(
        signature
          .required("message", SyntaxShape::String, "Prompt label")
          .named("value", SyntaxShape::String, "Current value (default: empty)", None)
          .named(
            "status",
            SyntaxShape::String,
            "pending, done, or aborted (default: pending)",
            None,
          )
          .switch("focused", "Whether the prompt is focused (default: false)", None)
          .named(
            "render-style",
            SyntaxShape::String,
            "default, password, or invisible (default: default)",
            None,
          )
          .switch("hide-status", "Hide the status symbol (default: false)", None),
        false,
      ),
      Self::SelectPrompt => with_presentation(
        signature
          .required("label", SyntaxShape::String, "Prompt label")
          .required("options", list(SyntaxShape::String), "Selectable option strings")
          .named("selected", SyntaxShape::Int, "Focused option index (default: 0)", None)
          .named(
            "status",
            SyntaxShape::String,
            "pending, done, or aborted (default: pending)",
            None,
          )
          .switch("focused", "Whether the prompt is focused (default: false)", None),
        false,
      ),
      Self::QrCode => signature
        .required("data", SyntaxShape::String, "Text encoded into the QR symbol")
        .switch("no-quiet-zone", "Disable the surrounding quiet zone (default: false)", None)
        .named(
          "scaling",
          SyntaxShape::String,
          "exact, min, or max (default: exact)",
          None,
        )
        .named("scale-width", SyntaxShape::Int, "Exact module width (default: 1)", None)
        .named("scale-height", SyntaxShape::Int, "Exact module height (default: 1)", None)
        .switch("inverted", "Invert light and dark modules (default: false)", None)
        .named("style", record_shape(), "QR foreground and background style (default: black on white)", None),
      Self::FractionalScrollbar => signature
        .required("content-length", SyntaxShape::Int, "Total logical content length")
        .required("viewport-length", SyntaxShape::Int, "Visible logical content length")
        .named("position", SyntaxShape::Int, "Current logical offset (default: 0)", None)
        .named(
          "orientation",
          SyntaxShape::String,
          "vertical or horizontal (default: vertical)",
          None,
        )
        .named(
          "arrows",
          SyntaxShape::String,
          "none, start, end, or both (default: none)",
          None,
        )
        .named(
          "glyphs",
          SyntaxShape::String,
          "legacy, unicode, box-drawing, or minimal (default: legacy)",
          None,
        )
        .named("track-style", record_shape(), "Track style (default: terminal default)", None)
        .named("thumb-style", record_shape(), "Thumb style (default: terminal default)", None)
        .named("arrow-style", record_shape(), "Arrow style (default: terminal default)", None),
      Self::ScrollView => signature
        .required("width", SyntaxShape::Int, "Off-screen content width")
        .required("height", SyntaxShape::Int, "Off-screen content height")
        .required("child", record_shape(), "Widget tree rendered into the scroll buffer")
        .named("scroll-x", SyntaxShape::Int, "Horizontal offset (default: 0)", None)
        .named("scroll-y", SyntaxShape::Int, "Vertical offset (default: 0)", None)
        .named(
          "vertical-scrollbar",
          SyntaxShape::String,
          "automatic, always, or never (default: automatic)",
          None,
        )
        .named(
          "horizontal-scrollbar",
          SyntaxShape::String,
          "automatic, always, or never (default: automatic)",
          None,
        ),
      Self::Spacer => signature,
    }
  }
}

/// One concrete widget constructor command under the `tui` namespace.
pub struct TuiWidget {
  widget: WidgetKind,
}

impl TuiWidget {
  /// Creates one widget constructor.
  pub(crate) const fn new(widget: WidgetKind) -> Self {
    Self { widget }
  }
}

impl SimplePluginCommand for TuiWidget {
  type Plugin = TuiPlugin;

  /// Returns the constructor command name exposed to Nushell.
  fn name(&self) -> &str {
    self.widget.command_name()
  }

  /// Defines the named widget fields and record output.
  fn signature(&self) -> Signature {
    self.widget.signature(self.name())
  }

  /// Describes the widget record this command constructs.
  fn description(&self) -> &str {
    self.widget.description()
  }

  /// Explains how constructor output feeds layouts and documents handler contracts.
  fn extra_description(&self) -> &str {
    self.widget.extra_description()
  }

  /// Converts supplied arguments into a validated declarative widget record.
  fn run(
    &self,
    _plugin: &TuiPlugin,
    _engine: &EngineInterface,
    call: &EvaluatedCall,
    _input: &Value,
  ) -> Result<Value, LabeledError> {
    build_widget_record(self.widget, call)
  }
}

impl WidgetKind {
  /// Returns constructor help, including the complete button handler contract.
  fn extra_description(self) -> &'static str {
    match self {
      Self::Button => {
        r#"Returns a validated widget record. Compose records in `tui layout [...]` and pass the
result as the positional view of `tui`.

The `--on-click` closure receives one `event` parameter and the same record as pipeline input:

{
  type: string
  kind: string
  button: string
  column: int
  row: int
  modifiers: list<string>
  widget: string
  state: any
}

It may return `null` (no change), any value (new state), or this action record, in which every field
is optional:

{
  state?: any
  view?: record | closure(any)
  quit?: bool
}"#
      },
      _ => {
        "Returns a validated widget record. Compose records in `tui layout [...]` \
and pass the result as the positional view of `tui`."
      },
    }
  }
}

/// Adds fields shared by widgets rendered inside a Ratatui block.
fn with_presentation(signature: Signature, default_border: bool) -> Signature {
  let border_description = if default_border {
    "Whether to draw borders (default: true)"
  } else {
    "Whether to draw borders (default: false)"
  };
  signature
    .named(
      "title",
      SyntaxShape::String,
      "Block title (default: no title)",
      None,
    )
    .switch("border", border_description, None)
    .named(
      "border-type",
      SyntaxShape::String,
      "Block border type (default: plain)",
      None,
    )
    .named(
      "style",
      record_shape(),
      "Widget style (default: terminal default)",
      None,
    )
    .named(
      "border-style",
      record_shape(),
      "Border style (default: terminal default)",
      None,
    )
}

/// Builds a list shape containing values of one shape.
fn list(shape: SyntaxShape) -> SyntaxShape {
  SyntaxShape::List(Box::new(shape))
}

/// Builds an open record shape for nested declarative values.
fn record_shape() -> SyntaxShape {
  SyntaxShape::Record(Default::default())
}

/// Builds the shape for a handler closure accepting one event record parameter.
fn handler_shape() -> SyntaxShape {
  SyntaxShape::Closure(Some(vec![record_shape()]))
}

/// Builds the string-or-index shape accepted by Ratatui colors.
fn color_shape() -> SyntaxShape {
  SyntaxShape::OneOf(vec![SyntaxShape::Int, SyntaxShape::String])
}

/// Copies supplied arguments into a record and validates it with the runtime parser.
fn build_widget_record(widget: WidgetKind, call: &EvaluatedCall) -> Result<Value, LabeledError> {
  let mut record = Record::new();
  record.push("type", Value::string(widget.record_type(), call.head));
  for (index, field) in widget.required_fields().iter().enumerate() {
    let value = call.positional.get(index).cloned().ok_or_else(|| {
      LabeledError::new("Missing required widget argument")
        .with_label(format!("missing positional `{field}`"), call.head)
    })?;
    record.push(*field, value);
  }
  for (name, value) in &call.named {
    let value = value
      .clone()
      .unwrap_or_else(|| Value::bool(true, name.span));
    record.push(name.item.clone(), value);
  }
  let value = Value::record(record, call.head);
  UiNode::parse(&value)?;
  Ok(value)
}

#[cfg(test)]
mod tests {
  use nu_plugin::EvaluatedCall;
  use nu_protocol::{IntoSpanned, Span, Value};

  use super::{WidgetKind, build_widget_record};

  /// Builds a call carrying positional and named values at test spans.
  fn call(positional: Vec<Value>, named: Vec<(&str, Value)>) -> EvaluatedCall {
    let span = Span::test_data();
    let mut call = EvaluatedCall::new(span);
    for value in positional {
      call.add_positional(value);
    }
    for (name, value) in named {
      call.add_named(name.to_owned().into_spanned(span), value);
    }
    call
  }

  /// Verifies that arguments become fields on a parser-compatible widget record.
  #[test]
  fn builds_valid_widget_record_from_arguments() {
    let call = call(
      vec![Value::test_string("hello")],
      vec![
        ("alignment", Value::test_string("center")),
        ("border", Value::test_bool(true)),
        ("ansi", Value::test_bool(true)),
        ("scroll-x", Value::test_int(2)),
        ("scroll-y", Value::test_int(3)),
      ],
    );

    let value = build_widget_record(WidgetKind::Paragraph, &call).expect("valid paragraph");
    let record = value.as_record().expect("record output");

    assert_eq!(record.get("type").unwrap().as_str().unwrap(), "paragraph");
    assert_eq!(record.get("text").unwrap().as_str().unwrap(), "hello");
    assert!(record.get("border").unwrap().as_bool().unwrap());
    assert!(record.get("ansi").unwrap().as_bool().unwrap());
    assert_eq!(record.get("scroll-x").unwrap().as_int().unwrap(), 2);
    assert_eq!(record.get("scroll-y").unwrap().as_int().unwrap(), 3);
  }

  /// Verifies bare widget switches are emitted as true record fields.
  #[test]
  fn builds_true_field_from_bare_switch() {
    let span = Span::test_data();
    let mut call = call(vec![Value::test_string("hello")], vec![]);
    call.add_flag("border".into_spanned(span));

    let value = build_widget_record(WidgetKind::Paragraph, &call).expect("valid paragraph");

    assert!(
      value
        .as_record()
        .unwrap()
        .get("border")
        .unwrap()
        .as_bool()
        .unwrap()
    );
  }

  /// Verifies that nested constructor records compose into a valid layout.
  #[test]
  fn composes_widget_records_in_layout() {
    let paragraph = build_widget_record(
      WidgetKind::Paragraph,
      &call(vec![Value::test_string("hello")], vec![]),
    )
    .expect("valid child");
    let layout = build_widget_record(
      WidgetKind::Layout,
      &call(vec![Value::test_list(vec![paragraph])], vec![]),
    )
    .expect("valid layout");

    assert_eq!(
      layout
        .as_record()
        .unwrap()
        .get("type")
        .unwrap()
        .as_str()
        .unwrap(),
      "layout"
    );
  }

  /// Verifies every widget exposes a distinct constructor command name.
  #[test]
  fn exposes_one_command_for_every_widget() {
    let mut names = WidgetKind::ALL
      .into_iter()
      .map(WidgetKind::command_name)
      .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();

    assert_eq!(names.len(), WidgetKind::ALL.len());
    assert!(names.contains(&"tui paragraph"));
    assert!(!names.contains(&"tui block"));
    assert!(names.iter().all(|name| !name.starts_with("tui widget ")));
  }

  /// Verifies button help documents its closure parameter, pipeline input, and result types.
  #[test]
  fn documents_button_handler_contract() {
    let help = WidgetKind::Button.extra_description();

    assert!(help.contains("`event` parameter"));
    assert!(help.contains("pipeline input"));
    assert!(help.contains("return `null`"));
    assert!(help.contains("modifiers: list<string>"));
    assert!(help.contains("view?: record | closure(any)"));
  }

  /// Verifies record assembly uses the same positional field order as each signature.
  #[test]
  fn keeps_required_field_mapping_aligned_with_signatures() {
    for widget in WidgetKind::ALL {
      let signature = widget.signature(widget.command_name());
      let fields = signature
        .required_positional
        .iter()
        .map(|argument| argument.name.as_str())
        .collect::<Vec<_>>();

      assert_eq!(fields, widget.required_fields());
    }
  }

  /// Verifies every optional constructor flag documents its runtime default.
  #[test]
  fn documents_defaults_for_all_widget_flags() {
    for widget in WidgetKind::ALL {
      let signature = widget.signature(widget.command_name());
      for flag in signature
        .named
        .into_iter()
        .filter(|flag| flag.long != "help")
      {
        assert!(
          flag.desc.contains("(default:"),
          "{} --{} does not document a default",
          widget.command_name(),
          flag.long,
        );
      }
    }
  }
}
