use std::time::Duration;

use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Record, Signature, Span, Type, Value};

use crate::{config::AppConfig, plugin::TuiPlugin, run::run_application};

/// A widget with its own `tui example` subcommand.
#[derive(Clone, Copy)]
pub(crate) enum WidgetExample {
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

impl WidgetExample {
    /// Contains every widget example in command registration order.
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

    /// Returns the complete Nushell command name for this widget.
    fn command_name(self) -> &'static str {
        match self {
            Self::BarChart => "tui example bar-chart",
            Self::Block => "tui example block",
            Self::Calendar => "tui example calendar",
            Self::Canvas => "tui example canvas",
            Self::Chart => "tui example chart",
            Self::Clear => "tui example clear",
            Self::Fill => "tui example fill",
            Self::Layout => "tui example layout",
            Self::Paragraph => "tui example paragraph",
            Self::Button => "tui example button",
            Self::List => "tui example list",
            Self::Gauge => "tui example gauge",
            Self::LineGauge => "tui example line-gauge",
            Self::Logo => "tui example logo",
            Self::Mascot => "tui example mascot",
            Self::Scrollbar => "tui example scrollbar",
            Self::Sparkline => "tui example sparkline",
            Self::Table => "tui example table",
            Self::Tabs => "tui example tabs",
            Self::Spacer => "tui example spacer",
        }
    }

    /// Returns a short help description for this widget demonstration.
    fn description(self) -> &'static str {
        match self {
            Self::BarChart => "Demonstrate a TUI bar chart",
            Self::Block => "Demonstrate a TUI block",
            Self::Calendar => "Demonstrate a TUI monthly calendar",
            Self::Canvas => "Demonstrate a TUI canvas",
            Self::Chart => "Demonstrate a TUI chart",
            Self::Clear => "Demonstrate clearing a TUI area",
            Self::Fill => "Demonstrate filling a TUI area",
            Self::Layout => "Demonstrate a TUI layout",
            Self::Paragraph => "Demonstrate a TUI paragraph",
            Self::Button => "Demonstrate a TUI button",
            Self::List => "Demonstrate a TUI list",
            Self::Gauge => "Demonstrate a TUI gauge",
            Self::LineGauge => "Demonstrate a TUI line gauge",
            Self::Logo => "Demonstrate the Ratatui logo",
            Self::Mascot => "Demonstrate the Ratatui mascot",
            Self::Scrollbar => "Demonstrate a TUI scrollbar",
            Self::Sparkline => "Demonstrate a TUI sparkline",
            Self::Table => "Demonstrate a TUI table",
            Self::Tabs => "Demonstrate TUI tabs",
            Self::Spacer => "Demonstrate a TUI spacer",
        }
    }

    /// Builds the focused widget view for this example.
    fn view(self, span: Span) -> Value {
        match self {
            Self::BarChart => bar_chart_example(span),
            Self::Block => block_example(span),
            Self::Calendar => calendar_example(span),
            Self::Canvas => canvas_example(span),
            Self::Chart => chart_example(span),
            Self::Clear => clear_example(span),
            Self::Fill => fill_example(span),
            Self::Layout => layout_example(span),
            Self::Paragraph => paragraph_example(span),
            Self::Button => button_example(span),
            Self::List => list_example(span),
            Self::Gauge => gauge_example(span),
            Self::LineGauge => line_gauge_example(span),
            Self::Logo => logo_example(span),
            Self::Mascot => mascot_example(span),
            Self::Scrollbar => scrollbar_example(span),
            Self::Sparkline => sparkline_example(span),
            Self::Table => table_example(span),
            Self::Tabs => tabs_example(span),
            Self::Spacer => spacer_example(span),
        }
    }
}

/// One concrete zero-argument `tui example` widget command.
pub struct TuiExample {
    widget: WidgetExample,
}

impl TuiExample {
    /// Creates the command for one supported widget.
    pub(crate) const fn new(widget: WidgetExample) -> Self {
        Self { widget }
    }
}

impl SimplePluginCommand for TuiExample {
    type Plugin = TuiPlugin;

    /// Returns the command name exposed to Nushell.
    fn name(&self) -> &str {
        self.widget.command_name()
    }

    /// Defines a zero-argument command with terminal-only output.
    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::Nothing)
            .category(Category::Experimental)
    }

    /// Describes this built-in widget demonstration.
    fn description(&self) -> &str {
        self.widget.description()
    }

    /// Documents the keys that close the demonstration.
    fn extra_description(&self) -> &str {
        "Press Escape or Ctrl-C to exit."
    }

    /// Shows this concrete command invocation in help output.
    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            example: self.widget.command_name(),
            description: self.widget.description(),
            result: None,
        }]
    }

    /// Builds and runs this command's widget demonstration.
    fn run(
        &self,
        _plugin: &TuiPlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let config = example_config(self.widget, call.head);
        run_application(engine, self.name(), &config, call.head)
    }
}

/// Builds a complete application configuration for one widget demonstration.
fn example_config(widget: WidgetExample, span: Span) -> AppConfig {
    AppConfig {
        state: Value::nothing(span),
        view: with_instructions(widget.view(span), span),
        on_event: None,
        on_key: None,
        quit_on_esc: true,
        tick_rate: Duration::from_millis(250),
    }
}

/// Builds a labelled vertical bar chart demonstration.
fn bar_chart_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("bar-chart", span)),
            (
                "bars",
                Value::list(
                    [("A", 3), ("B", 7), ("C", 5), ("D", 9)]
                        .into_iter()
                        .map(|(label, value)| {
                            record(
                                vec![
                                    ("label", Value::string(label, span)),
                                    ("value", Value::int(value, span)),
                                ],
                                span,
                            )
                        })
                        .collect(),
                    span,
                ),
            ),
            ("title", Value::string(" Bar chart ", span)),
            ("border", Value::bool(true, span)),
            ("bar-width", Value::int(3, span)),
            ("bar-style", style("cyan", false, span)),
            ("value-style", style("yellow", true, span)),
        ],
        span,
    )
}

/// Builds a standalone rounded block demonstration.
fn block_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("block", span)),
            ("title", Value::string(" Standalone block ", span)),
            ("border-type", Value::string("rounded", span)),
            ("border-style", style("cyan", true, span)),
        ],
        span,
    )
}

/// Builds an August 2026 monthly calendar demonstration.
fn calendar_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("calendar", span)),
            ("year", Value::int(2026, span)),
            ("month", Value::int(8, span)),
            ("title", Value::string(" Calendar ", span)),
            ("border", Value::bool(true, span)),
            ("month-style", style("cyan", true, span)),
            ("weekday-style", style("yellow", false, span)),
            ("surrounding-style", style("dark_gray", false, span)),
        ],
        span,
    )
}

/// Builds a braille point canvas demonstration.
fn canvas_example(span: Span) -> Value {
    let points = (-8..=8)
        .map(|x| {
            record(
                vec![
                    ("x", Value::float(f64::from(x), span)),
                    ("y", Value::float(f64::from(x * x) / 8.0 - 8.0, span)),
                    ("color", Value::string("cyan", span)),
                ],
                span,
            )
        })
        .collect();
    record(
        vec![
            ("type", Value::string("canvas", span)),
            ("points", Value::list(points, span)),
            ("x-bounds", number_list([-10.0, 10.0], span)),
            ("y-bounds", number_list([-10.0, 10.0], span)),
            ("title", Value::string(" Canvas ", span)),
            ("border", Value::bool(true, span)),
        ],
        span,
    )
}

/// Builds a cartesian line chart demonstration.
fn chart_example(span: Span) -> Value {
    let data = [(0.0, 0.0), (2.0, 4.0), (4.0, 3.0), (6.0, 8.0), (10.0, 6.0)]
        .into_iter()
        .map(|(x, y)| number_list([x, y], span))
        .collect();
    record(
        vec![
            ("type", Value::string("chart", span)),
            (
                "datasets",
                Value::list(
                    vec![record(
                        vec![
                            ("name", Value::string("series", span)),
                            ("data", Value::list(data, span)),
                            ("graph-type", Value::string("line", span)),
                            ("style", style("cyan", true, span)),
                        ],
                        span,
                    )],
                    span,
                ),
            ),
            ("x-axis", axis("X", span)),
            ("y-axis", axis("Y", span)),
            ("title", Value::string(" Chart ", span)),
            ("border", Value::bool(true, span)),
        ],
        span,
    )
}

/// Builds a clear widget demonstration whose content area is intentionally blank.
fn clear_example(span: Span) -> Value {
    record(vec![("type", Value::string("clear", span))], span)
}

/// Builds a patterned fill demonstration.
fn fill_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("fill", span)),
            ("symbol", Value::string("·", span)),
            ("style", style("dark_gray", false, span)),
        ],
        span,
    )
}

/// Builds a horizontal layout containing two contrasting child widgets.
fn layout_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("layout", span)),
            ("direction", Value::string("horizontal", span)),
            (
                "constraints",
                Value::list(
                    vec![
                        record(vec![("percentage", Value::int(35, span))], span),
                        record(vec![("fill", Value::int(1, span))], span),
                    ],
                    span,
                ),
            ),
            (
                "children",
                Value::list(
                    vec![
                        bordered_paragraph("35%", "Percentage constraint", "cyan", span),
                        bordered_paragraph("Fill", "Remaining space", "magenta", span),
                    ],
                    span,
                ),
            ),
        ],
        span,
    )
}

/// Builds a styled, wrapped paragraph demonstration.
fn paragraph_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("paragraph", span)),
            (
                "text",
                Value::string(
                    "Paragraphs display text, support wrapping, alignment, borders, and styles.",
                    span,
                ),
            ),
            ("title", Value::string(" Paragraph ", span)),
            ("border", Value::bool(true, span)),
            ("border-type", Value::string("rounded", span)),
            ("alignment", Value::string("center", span)),
            ("style", style("cyan", true, span)),
        ],
        span,
    )
}

/// Builds a centered button demonstration with its default border behavior.
fn button_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("button", span)),
            ("id", Value::string("example-button", span)),
            ("label", Value::string(" Button ", span)),
            ("title", Value::string(" Button ", span)),
            ("border-type", Value::string("double", span)),
            ("alignment", Value::string("center", span)),
            ("style", style("yellow", true, span)),
            ("border-style", style("yellow", false, span)),
        ],
        span,
    )
}

/// Builds a bordered list demonstration.
fn list_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("list", span)),
            (
                "items",
                Value::list(
                    ["Layout", "Paragraph", "Button", "List", "Gauge", "Spacer"]
                        .into_iter()
                        .map(|item| Value::string(item, span))
                        .collect(),
                    span,
                ),
            ),
            ("title", Value::string(" Widgets ", span)),
            ("border", Value::bool(true, span)),
            ("border-type", Value::string("rounded", span)),
            ("style", style("green", false, span)),
        ],
        span,
    )
}

/// Builds a styled gauge demonstration at 65 percent completion.
fn gauge_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("gauge", span)),
            ("ratio", Value::float(0.65, span)),
            ("label", Value::string("65%", span)),
            ("title", Value::string(" Gauge ", span)),
            ("border", Value::bool(true, span)),
            ("border-type", Value::string("thick", span)),
            ("gauge-style", style("cyan", true, span)),
        ],
        span,
    )
}

/// Builds a styled line gauge demonstration.
fn line_gauge_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("line-gauge", span)),
            ("ratio", Value::float(0.65, span)),
            ("label", Value::string("65%", span)),
            ("title", Value::string(" Line gauge ", span)),
            ("border", Value::bool(true, span)),
            ("filled-symbol", Value::string("━", span)),
            ("unfilled-symbol", Value::string("─", span)),
            ("filled-style", style("cyan", true, span)),
            ("unfilled-style", style("dark_gray", false, span)),
        ],
        span,
    )
}

/// Builds the small Ratatui logo demonstration.
fn logo_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("logo", span)),
            ("size", Value::string("small", span)),
        ],
        span,
    )
}

/// Builds the Ratatui mascot demonstration.
fn mascot_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("mascot", span)),
            ("blink", Value::bool(false, span)),
        ],
        span,
    )
}

/// Builds a positioned vertical scrollbar demonstration.
fn scrollbar_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("scrollbar", span)),
            ("content-length", Value::int(100, span)),
            ("position", Value::int(35, span)),
            ("viewport-length", Value::int(20, span)),
            ("thumb-style", style("cyan", true, span)),
            ("track-style", style("dark_gray", false, span)),
        ],
        span,
    )
}

/// Builds a sparkline demonstration with one absent sample.
fn sparkline_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("sparkline", span)),
            (
                "data",
                Value::list(
                    vec![1, 4, 2, 8, 5, 9, 3]
                        .into_iter()
                        .map(|value| Value::int(value, span))
                        .collect(),
                    span,
                ),
            ),
            ("title", Value::string(" Sparkline ", span)),
            ("border", Value::bool(true, span)),
            ("style", style("cyan", false, span)),
        ],
        span,
    )
}

/// Builds a two-column table demonstration.
fn table_example(span: Span) -> Value {
    let row = |left, right| string_list([left, right], span);
    record(
        vec![
            ("type", Value::string("table", span)),
            ("header", row("Widget", "Status")),
            (
                "rows",
                Value::list(
                    vec![
                        row("Gauge", "ready"),
                        row("Chart", "ready"),
                        row("Table", "ready"),
                    ],
                    span,
                ),
            ),
            ("title", Value::string(" Table ", span)),
            ("border", Value::bool(true, span)),
            ("style", style("cyan", false, span)),
        ],
        span,
    )
}

/// Builds a selected tab bar demonstration.
fn tabs_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("tabs", span)),
            ("titles", string_list(["Home", "Metrics", "Help"], span)),
            ("selected", Value::int(1, span)),
            ("title", Value::string(" Tabs ", span)),
            ("border", Value::bool(true, span)),
            ("highlight-style", style("cyan", true, span)),
        ],
        span,
    )
}

/// Builds a layout whose blank middle region demonstrates a spacer.
fn spacer_example(span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("layout", span)),
            ("direction", Value::string("vertical", span)),
            (
                "constraints",
                Value::list(
                    vec![
                        record(vec![("length", Value::int(3, span))], span),
                        record(vec![("fill", Value::int(1, span))], span),
                        record(vec![("length", Value::int(3, span))], span),
                    ],
                    span,
                ),
            ),
            (
                "children",
                Value::list(
                    vec![
                        bordered_paragraph("Above", "First widget", "blue", span),
                        record(vec![("type", Value::string("spacer", span))], span),
                        bordered_paragraph("Below", "Second widget", "blue", span),
                    ],
                    span,
                ),
            ),
        ],
        span,
    )
}

/// Adds a fixed footer explaining how to close the demonstration.
fn with_instructions(widget: Value, span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("layout", span)),
            ("direction", Value::string("vertical", span)),
            (
                "constraints",
                Value::list(
                    vec![
                        record(vec![("fill", Value::int(1, span))], span),
                        record(vec![("length", Value::int(1, span))], span),
                    ],
                    span,
                ),
            ),
            (
                "children",
                Value::list(
                    vec![
                        widget,
                        record(
                            vec![
                                ("type", Value::string("paragraph", span)),
                                (
                                    "text",
                                    Value::string("Press Escape or Ctrl-C to exit", span),
                                ),
                                ("alignment", Value::string("center", span)),
                                ("style", style("dark_gray", false, span)),
                            ],
                            span,
                        ),
                    ],
                    span,
                ),
            ),
        ],
        span,
    )
}

/// Builds a bordered paragraph used by compound demonstrations.
fn bordered_paragraph(text: &str, title: &str, color: &str, span: Span) -> Value {
    record(
        vec![
            ("type", Value::string("paragraph", span)),
            ("text", Value::string(text, span)),
            ("title", Value::string(title, span)),
            ("border", Value::bool(true, span)),
            ("border-type", Value::string("rounded", span)),
            ("alignment", Value::string("center", span)),
            ("style", style(color, true, span)),
        ],
        span,
    )
}

/// Builds a chart axis covering zero through ten.
fn axis(title: &str, span: Span) -> Value {
    record(
        vec![
            ("title", Value::string(title, span)),
            ("bounds", number_list([0.0, 10.0], span)),
            ("labels", string_list(["0", "5", "10"], span)),
            ("style", style("white", false, span)),
        ],
        span,
    )
}

/// Builds a Nushell list from string values.
fn string_list<const N: usize>(values: [&str; N], span: Span) -> Value {
    Value::list(
        values
            .into_iter()
            .map(|value| Value::string(value, span))
            .collect(),
        span,
    )
}

/// Builds a Nushell list from floating-point values.
fn number_list<const N: usize>(values: [f64; N], span: Span) -> Value {
    Value::list(
        values
            .into_iter()
            .map(|value| Value::float(value, span))
            .collect(),
        span,
    )
}

/// Builds a foreground color style with optional bold text.
fn style(color: &str, bold: bool, span: Span) -> Value {
    record(
        vec![
            ("fg", Value::string(color, span)),
            ("bold", Value::bool(bold, span)),
        ],
        span,
    )
}

/// Constructs a Nushell record value from ordered key/value fields.
fn record(fields: Vec<(&str, Value)>, span: Span) -> Value {
    let mut record = Record::new();
    for (name, value) in fields {
        record.push(name, value);
    }
    Value::record(record, span)
}

#[cfg(test)]
mod tests {
    use nu_protocol::Span;
    use ratatui::{Terminal, backend::TestBackend};

    use crate::config::UiNode;

    use super::{WidgetExample, example_config};

    /// Verifies that every registered subcommand produces a valid widget tree.
    #[test]
    fn builds_every_supported_widget_example() {
        for widget in WidgetExample::ALL {
            let config = example_config(widget, Span::test_data());
            UiNode::parse(&config.view).expect("example should produce a valid widget tree");
        }
    }

    /// Verifies that every widget has a distinct three-word subcommand name.
    #[test]
    fn exposes_distinct_widget_subcommands() {
        let mut names = WidgetExample::ALL.map(WidgetExample::command_name).to_vec();
        names.sort_unstable();
        names.dedup();

        assert_eq!(names.len(), WidgetExample::ALL.len());
        assert!(names.iter().all(|name| name.starts_with("tui example ")));
    }

    /// Verifies that every widget example renders successfully on Ratatui's test backend.
    #[test]
    fn renders_every_supported_widget_example() {
        for widget in WidgetExample::ALL {
            let config = example_config(widget, Span::test_data());
            let node = UiNode::parse(&config.view).expect("example should parse");
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("test terminal");
            let mut hits = Vec::new();

            terminal
                .draw(|frame| node.render(frame, frame.area(), &mut hits))
                .expect("example should render");
        }
    }
}
