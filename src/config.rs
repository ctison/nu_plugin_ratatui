use std::{str::FromStr, time::Duration};

use nu_protocol::{LabeledError, Record, Span, Spanned, Value, engine::Closure};
use ratatui::{
    layout::{Alignment, Constraint, Direction},
    style::{Color, Modifier, Style},
    widgets::{BorderType, Borders},
};

/// A Nushell closure paired with the source span that supplied it.
pub type Handler = Spanned<Closure>;

/// Runtime configuration parsed from the `tui run` record.
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

/// A declarative widget tree parsed from a Nushell record.
#[derive(Clone, Debug)]
pub enum UiNode {
    Layout {
        direction: Direction,
        constraints: Vec<Constraint>,
        children: Vec<UiNode>,
    },
    Paragraph {
        text: String,
        block: BlockSpec,
        alignment: Alignment,
        wrap: bool,
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
    Spacer,
}

impl AppConfig {
    /// Parses and validates the top-level application record.
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
        let record = as_record(value, "widget")?;
        let widget_type = required_string(record, "type", value.span())?;
        match widget_type.as_str() {
            "layout" => parse_layout(record, value.span()),
            "paragraph" => parse_paragraph(record, value.span()),
            "button" => parse_button(record, value.span()),
            "list" => parse_list(record, value.span()),
            "gauge" => parse_gauge(record, value.span()),
            "spacer" => Ok(Self::Spacer),
            other => Err(config_error(
                format!("unsupported widget type `{other}`"),
                required(record, "type", value.span())?.span(),
            )),
        }
    }
}

/// Parses a layout node with constraints matched to its children.
fn parse_layout(record: &Record, span: Span) -> Result<UiNode, LabeledError> {
    let direction = match optional_string(record, "direction")?.as_deref() {
        None | Some("vertical") => Direction::Vertical,
        Some("horizontal") => Direction::Horizontal,
        Some(other) => {
            return Err(config_error(
                format!("unsupported layout direction `{other}`"),
                record.get("direction").map(Value::span).unwrap_or(span),
            ));
        }
    };
    let child_values = required(record, "children", span)?
        .as_list()
        .map_err(|error| LabeledError::from_diagnostic(&error))?;
    let children = child_values
        .iter()
        .map(UiNode::parse)
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
    Ok(UiNode::Paragraph {
        text: required_string(record, "text", span)?,
        block: parse_block(record, false)?,
        alignment: parse_alignment(record, span)?,
        wrap: optional_bool(record, "wrap")?.unwrap_or(true),
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
    let ratio_value = required(record, "ratio", span)?;
    let ratio = match ratio_value {
        Value::Float { val, .. } => *val,
        Value::Int { val, .. } => *val as f64,
        _ => {
            return Err(config_error(
                "`ratio` must be a float between 0 and 1",
                ratio_value.span(),
            ));
        }
    };
    if !(0.0..=1.0).contains(&ratio) {
        return Err(config_error(
            "`ratio` must be between 0 and 1",
            ratio_value.span(),
        ));
    }
    Ok(UiNode::Gauge {
        ratio,
        label: optional_string(record, "label")?,
        block: parse_block(record, false)?,
        gauge_style: parse_style(record.get("gauge-style"))?,
    })
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
        }
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

/// Parses a style record containing colors and text modifiers.
fn parse_style(value: Option<&Value>) -> Result<Style, LabeledError> {
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

/// Parses a named, indexed, or hexadecimal Ratatui color.
fn parse_color(value: &Value) -> Result<Color, LabeledError> {
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
            }
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
}
