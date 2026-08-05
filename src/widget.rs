use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, LabeledError, Record, Signature, SyntaxShape, Type, Value};

use crate::{config::UiNode, plugin::TuiPlugin};

/// A declarative widget supported by record constructors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WidgetKind {
    BarChart,
    Block,
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
    Spacer,
}

impl WidgetKind {
    /// Contains every supported widget in command registration order.
    pub(crate) const ALL: [Self; 20] = [
        Self::BarChart,
        Self::Block,
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
        Self::Spacer,
    ];

    /// Returns the widget type stored in the emitted record.
    fn record_type(self) -> &'static str {
        match self {
            Self::BarChart => "bar-chart",
            Self::Block => "block",
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
            Self::Spacer => "spacer",
        }
    }

    /// Returns the full constructor command name.
    fn command_name(self) -> &'static str {
        match self {
            Self::BarChart => "tui bar-chart",
            Self::Block => "tui block",
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
            Self::Spacer => "tui spacer",
        }
    }

    /// Returns a short description for a widget record constructor.
    fn description(self) -> &'static str {
        match self {
            Self::BarChart => "Declare a bar chart widget",
            Self::Block => "Declare a block widget",
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
            Self::Spacer => "Declare a spacer widget",
        }
    }

    /// Adds this widget's required and optional record fields as named flags.
    fn signature(self, command_name: &str) -> Signature {
        let signature = Signature::build(command_name)
            .input_output_type(Type::Nothing, Type::Record(Default::default()))
            .category(Category::Experimental);
        match self {
            Self::BarChart => with_presentation(
                signature
                    .required_named("bars", list(record_shape()), "Bar records", None)
                    .named(
                        "direction",
                        SyntaxShape::String,
                        "vertical or horizontal",
                        None,
                    )
                    .named("max", SyntaxShape::Int, "Maximum chart value", None)
                    .named("bar-width", SyntaxShape::Int, "Bar width in cells", None)
                    .named("bar-gap", SyntaxShape::Int, "Gap between bars", None)
                    .named("bar-style", record_shape(), "Default bar style", None)
                    .named("value-style", record_shape(), "Value style", None)
                    .named("label-style", record_shape(), "Label style", None),
            ),
            Self::Block => with_presentation(signature),
            Self::Calendar => with_presentation(
                signature
                    .required_named("year", SyntaxShape::Int, "Calendar year", None)
                    .required_named("month", SyntaxShape::Int, "Month from 1 through 12", None)
                    .named("month-style", record_shape(), "Month header style", None)
                    .named(
                        "weekday-style",
                        record_shape(),
                        "Weekday header style",
                        None,
                    )
                    .named(
                        "surrounding-style",
                        record_shape(),
                        "Surrounding days style",
                        None,
                    ),
            ),
            Self::Canvas => with_presentation(
                signature
                    .required_named("points", list(record_shape()), "Canvas point records", None)
                    .named(
                        "x-bounds",
                        list(SyntaxShape::Number),
                        "Horizontal bounds",
                        None,
                    )
                    .named(
                        "y-bounds",
                        list(SyntaxShape::Number),
                        "Vertical bounds",
                        None,
                    )
                    .named("marker", SyntaxShape::String, "Point marker", None)
                    .named(
                        "background-color",
                        color_shape(),
                        "Canvas background color",
                        None,
                    ),
            ),
            Self::Chart => with_presentation(
                signature
                    .required_named("datasets", list(record_shape()), "Chart datasets", None)
                    .named("x-axis", record_shape(), "Horizontal axis", None)
                    .named("y-axis", record_shape(), "Vertical axis", None),
            ),
            Self::Clear => signature,
            Self::Fill => signature
                .named("symbol", SyntaxShape::String, "Repeated cell symbol", None)
                .named("style", record_shape(), "Fill style", None),
            Self::Layout => signature
                .required_named(
                    "children",
                    list(record_shape()),
                    "Child widget records",
                    None,
                )
                .named(
                    "direction",
                    SyntaxShape::String,
                    "vertical or horizontal",
                    None,
                )
                .named(
                    "constraints",
                    list(record_shape()),
                    "One layout constraint per child",
                    None,
                ),
            Self::Paragraph => with_presentation(
                signature
                    .required_named("text", SyntaxShape::String, "Text to display", None)
                    .named("alignment", SyntaxShape::String, "Text alignment", None)
                    .named("wrap", SyntaxShape::Boolean, "Whether text wraps", None),
            ),
            Self::Button => with_presentation(
                signature
                    .required_named("id", SyntaxShape::String, "Event widget identifier", None)
                    .required_named("label", SyntaxShape::String, "Button label", None)
                    .named("alignment", SyntaxShape::String, "Label alignment", None)
                    .named(
                        "on-click",
                        SyntaxShape::Closure(None),
                        "Left-button release handler",
                        None,
                    ),
            ),
            Self::List => with_presentation(signature.required_named(
                "items",
                list(SyntaxShape::String),
                "List item strings",
                None,
            )),
            Self::Gauge => with_presentation(
                signature
                    .required_named("ratio", SyntaxShape::Number, "Completion from 0 to 1", None)
                    .named("label", SyntaxShape::String, "Gauge label", None)
                    .named("gauge-style", record_shape(), "Filled area style", None),
            ),
            Self::LineGauge => with_presentation(
                signature
                    .required_named("ratio", SyntaxShape::Number, "Completion from 0 to 1", None)
                    .named("label", SyntaxShape::String, "Gauge label", None)
                    .named("filled-style", record_shape(), "Filled line style", None)
                    .named(
                        "unfilled-style",
                        record_shape(),
                        "Unfilled line style",
                        None,
                    )
                    .named("filled-symbol", SyntaxShape::String, "Filled symbol", None)
                    .named(
                        "unfilled-symbol",
                        SyntaxShape::String,
                        "Unfilled symbol",
                        None,
                    ),
            ),
            Self::Logo => signature.named("size", SyntaxShape::String, "tiny or small", None),
            Self::Mascot => signature.named(
                "blink",
                SyntaxShape::Boolean,
                "Render the alternate eye",
                None,
            ),
            Self::Scrollbar => signature
                .required_named(
                    "content-length",
                    SyntaxShape::Int,
                    "Total content length",
                    None,
                )
                .named("position", SyntaxShape::Int, "Current offset", None)
                .named(
                    "viewport-length",
                    SyntaxShape::Int,
                    "Visible content length",
                    None,
                )
                .named(
                    "orientation",
                    SyntaxShape::String,
                    "Scrollbar orientation",
                    None,
                )
                .named("thumb-style", record_shape(), "Scroll thumb style", None)
                .named("track-style", record_shape(), "Scroll track style", None),
            Self::Sparkline => with_presentation(
                signature
                    .required_named(
                        "data",
                        list(SyntaxShape::Any),
                        "Integer or null samples",
                        None,
                    )
                    .named("max", SyntaxShape::Int, "Maximum sample value", None)
                    .named(
                        "direction",
                        SyntaxShape::String,
                        "Rendering direction",
                        None,
                    )
                    .named(
                        "absent-symbol",
                        SyntaxShape::String,
                        "Missing sample symbol",
                        None,
                    )
                    .named("absent-style", record_shape(), "Missing sample style", None),
            ),
            Self::Table => with_presentation(
                signature
                    .required_named("rows", list(list(SyntaxShape::String)), "Table rows", None)
                    .named("header", list(SyntaxShape::String), "Header cells", None)
                    .named("widths", list(record_shape()), "Column constraints", None)
                    .named("column-spacing", SyntaxShape::Int, "Column spacing", None),
            ),
            Self::Tabs => with_presentation(
                signature
                    .required_named("titles", list(SyntaxShape::String), "Tab titles", None)
                    .named("selected", SyntaxShape::Int, "Selected tab index", None)
                    .named("divider", SyntaxShape::String, "Title divider", None)
                    .named(
                        "highlight-style",
                        record_shape(),
                        "Selected title style",
                        None,
                    ),
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

    /// Explains how constructor output feeds layouts and applications.
    fn extra_description(&self) -> &str {
        "Returns a validated widget record. Compose records in `tui layout --children [...]` \
and pass the result as the `view` field of `tui run`."
    }

    /// Converts supplied flags into a validated declarative widget record.
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

/// Adds fields shared by widgets rendered inside a Ratatui block.
fn with_presentation(signature: Signature) -> Signature {
    signature
        .named("title", SyntaxShape::String, "Block title", None)
        .named(
            "border",
            SyntaxShape::Boolean,
            "Whether to draw borders",
            None,
        )
        .named(
            "border-type",
            SyntaxShape::String,
            "Block border type",
            None,
        )
        .named("style", record_shape(), "Widget style", None)
        .named("border-style", record_shape(), "Border style", None)
}

/// Builds a list shape containing values of one shape.
fn list(shape: SyntaxShape) -> SyntaxShape {
    SyntaxShape::List(Box::new(shape))
}

/// Builds an open record shape for nested declarative values.
fn record_shape() -> SyntaxShape {
    SyntaxShape::Record(Default::default())
}

/// Builds the string-or-index shape accepted by Ratatui colors.
fn color_shape() -> SyntaxShape {
    SyntaxShape::OneOf(vec![SyntaxShape::Int, SyntaxShape::String])
}

/// Copies supplied named flags into a record and validates it with the runtime parser.
fn build_widget_record(widget: WidgetKind, call: &EvaluatedCall) -> Result<Value, LabeledError> {
    let mut record = Record::new();
    record.push("type", Value::string(widget.record_type(), call.head));
    for (name, value) in &call.named {
        if let Some(value) = value {
            record.push(name.item.clone(), value.clone());
        }
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

    /// Builds a call carrying named values at test spans.
    fn call(named: Vec<(&str, Value)>) -> EvaluatedCall {
        let span = Span::test_data();
        let mut call = EvaluatedCall::new(span);
        for (name, value) in named {
            call.add_named(name.to_owned().into_spanned(span), value);
        }
        call
    }

    /// Verifies that flags become fields on a parser-compatible widget record.
    #[test]
    fn builds_valid_widget_record_from_flags() {
        let call = call(vec![
            ("text", Value::test_string("hello")),
            ("alignment", Value::test_string("center")),
            ("border", Value::test_bool(true)),
        ]);

        let value = build_widget_record(WidgetKind::Paragraph, &call).expect("valid paragraph");
        let record = value.as_record().expect("record output");

        assert_eq!(record.get("type").unwrap().as_str().unwrap(), "paragraph");
        assert_eq!(record.get("text").unwrap().as_str().unwrap(), "hello");
        assert!(record.get("border").unwrap().as_bool().unwrap());
    }

    /// Verifies that nested constructor records compose into a valid layout.
    #[test]
    fn composes_widget_records_in_layout() {
        let paragraph = build_widget_record(
            WidgetKind::Paragraph,
            &call(vec![("text", Value::test_string("hello"))]),
        )
        .expect("valid child");
        let layout = build_widget_record(
            WidgetKind::Layout,
            &call(vec![("children", Value::test_list(vec![paragraph]))]),
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
        assert!(names.iter().all(|name| !name.starts_with("tui widget ")));
    }
}
