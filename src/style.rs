use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, Record, Signature, SyntaxShape, Type, Value};

use crate::{config::parse_style, plugin::TuiPlugin};

/// The `tui style` record constructor.
pub struct TuiStyle;

impl SimplePluginCommand for TuiStyle {
  type Plugin = TuiPlugin;

  /// Returns the command name exposed to Nushell.
  fn name(&self) -> &str {
    "tui style"
  }

  /// Defines Ratatui color and text-modifier fields with their defaults.
  fn signature(&self) -> Signature {
    Signature::build(self.name())
      .named(
        "fg",
        color_shape(),
        "Foreground color (default: terminal default)",
        None,
      )
      .named(
        "bg",
        color_shape(),
        "Background color (default: terminal default)",
        None,
      )
      .switch("bold", "Enable bold text (default: false)", None)
      .switch("italic", "Enable italic text (default: false)", None)
      .switch("underlined", "Enable underlining (default: false)", None)
      .switch("underline", "Alias for --underlined (default: false)", None)
      .switch(
        "reversed",
        "Swap foreground and background (default: false)",
        None,
      )
      .switch("dim", "Enable dim text (default: false)", None)
      .input_output_type(Type::Nothing, Type::Record(Default::default()))
      .category(Category::Experimental)
  }

  /// Describes the style record produced by this command.
  fn description(&self) -> &str {
    "Declare a Ratatui style record"
  }

  /// Explains where the constructed style record can be used.
  fn extra_description(&self) -> &str {
    "Returns a validated record accepted by `--style`, `--border-style`, and other style-bearing \
widget flags. Colors may be Ratatui names, `#RRGGBB` strings, or indexes from 0 through 255."
  }

  /// Provides a composable style-constructor example.
  fn examples(&self) -> Vec<Example<'_>> {
    vec![Example {
      example: "tui style --fg yellow --bg '#202020' --bold",
      description: "Declare a bold yellow style on a dark background",
      result: None,
    }]
  }

  /// Converts supplied style flags into a validated ordinary record.
  fn run(
    &self,
    _plugin: &TuiPlugin,
    _engine: &EngineInterface,
    call: &EvaluatedCall,
    _input: &Value,
  ) -> Result<Value, LabeledError> {
    build_style_record(call)
  }
}

/// Builds the string-or-index shape accepted by Ratatui colors.
fn color_shape() -> SyntaxShape {
  SyntaxShape::OneOf(vec![SyntaxShape::Int, SyntaxShape::String])
}

/// Copies style flags into a record and validates them with the runtime parser.
fn build_style_record(call: &EvaluatedCall) -> Result<Value, LabeledError> {
  let mut record = Record::new();
  for (name, value) in &call.named {
    let value = value
      .clone()
      .unwrap_or_else(|| Value::bool(true, name.span));
    let field = if name.item == "underline" {
      "underlined"
    } else {
      name.item.as_str()
    };
    if field == "underlined" && record.get(field).is_some() {
      return Err(LabeledError::new("Conflicting style flags").with_label(
        "use either `--underlined` or `--underline`, not both",
        name.span,
      ));
    }
    record.push(field, value);
  }
  let value = Value::record(record, call.head);
  parse_style(Some(&value))?;
  Ok(value)
}

#[cfg(test)]
mod tests {
  use nu_plugin::{EvaluatedCall, SimplePluginCommand};
  use nu_protocol::{IntoSpanned, Span, Value};

  use super::{TuiStyle, build_style_record};

  /// Builds a style call carrying concise named test values.
  fn call(named: Vec<(&str, Value)>) -> EvaluatedCall {
    let span = Span::test_data();
    let mut call = EvaluatedCall::new(span);
    for (name, value) in named {
      call.add_named(name.to_owned().into_spanned(span), value);
    }
    call
  }

  /// Verifies supplied style fields are preserved in the constructor record.
  #[test]
  fn builds_valid_style_record() {
    let value = build_style_record(&call(vec![
      ("fg", Value::test_string("yellow")),
      ("bold", Value::test_bool(true)),
    ]))
    .expect("valid style");
    let record = value.as_record().expect("record output");

    assert_eq!(record.get("fg").unwrap().as_str().unwrap(), "yellow");
    assert!(record.get("bold").unwrap().as_bool().unwrap());
  }

  /// Verifies bare switches mean true while equals syntax preserves explicit false.
  #[test]
  fn supports_bare_and_false_modifier_switches() {
    let span = Span::test_data();
    let mut bare = call(vec![]);
    bare.add_flag("underlined".into_spanned(span));
    let bare = build_style_record(&bare).expect("bare switch");
    let explicit_false = build_style_record(&call(vec![("underline", Value::test_bool(false))]))
      .expect("explicit false");

    assert!(
      bare
        .as_record()
        .unwrap()
        .get("underlined")
        .unwrap()
        .as_bool()
        .unwrap()
    );
    assert!(
      !explicit_false
        .as_record()
        .unwrap()
        .get("underlined")
        .unwrap()
        .as_bool()
        .unwrap()
    );
  }

  /// Verifies indexed colors outside Ratatui's range are rejected.
  #[test]
  fn rejects_invalid_indexed_color() {
    let error = build_style_record(&call(vec![("fg", Value::test_int(256))]))
      .expect_err("indexed color should be invalid");

    assert!(error.labels[0].text.contains("between 0 and 255"));
  }

  /// Verifies every style flag documents the value used when it is omitted.
  #[test]
  fn documents_defaults_for_all_style_flags() {
    for flag in TuiStyle
      .signature()
      .named
      .into_iter()
      .filter(|flag| flag.long != "help")
    {
      assert!(
        flag.desc.contains("(default:"),
        "--{} does not document a default",
        flag.long,
      );
    }
  }
}
