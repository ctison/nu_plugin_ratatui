use std::io;

use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{
  Category, Example, LabeledError, PipelineData, Signature, Span, SyntaxShape, Type, Value,
};
use ratatui::{
  DefaultTerminal, Frame,
  crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
  layout::{Alignment, Constraint, Layout},
  style::{Color, Modifier, Style},
  text::{Line, Span as TextSpan, Text},
  widgets::{Block, List, ListItem, ListState, Paragraph},
};

use crate::{plugin::TuiPlugin, run::run_in_terminal};

/// One Nushell example embedded in the plugin binary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EmbeddedExample {
  name: &'static str,
  source: &'static str,
}

/// Interactive selection and code-preview state.
struct ChooserState {
  list: ListState,
  preview_scroll: u16,
}

/// ANSI style accumulated while parsing `nu-highlight` output.
#[derive(Clone, Copy, Default)]
struct AnsiStyle {
  foreground: Option<Color>,
  background: Option<Color>,
  modifiers: Modifier,
}

/// All embedded examples, ordered by display name.
const EXAMPLES: &[EmbeddedExample] = &[
  EmbeddedExample {
    name: "advanced-widget-impl",
    source: include_str!("../examples/advanced-widget-impl.nu"),
  },
  EmbeddedExample {
    name: "calendar-explorer",
    source: include_str!("../examples/calendar-explorer.nu"),
  },
  EmbeddedExample {
    name: "canvas",
    source: include_str!("../examples/canvas.nu"),
  },
  EmbeddedExample {
    name: "chart",
    source: include_str!("../examples/chart.nu"),
  },
  EmbeddedExample {
    name: "color-explorer",
    source: include_str!("../examples/color-explorer.nu"),
  },
  EmbeddedExample {
    name: "colors-rgb",
    source: include_str!("../examples/colors-rgb.nu"),
  },
  EmbeddedExample {
    name: "command-preview",
    source: include_str!("../examples/command-preview.nu"),
  },
  EmbeddedExample {
    name: "constraint-explorer",
    source: include_str!("../examples/constraint-explorer.nu"),
  },
  EmbeddedExample {
    name: "constraints",
    source: include_str!("../examples/constraints.nu"),
  },
  EmbeddedExample {
    name: "counter",
    source: include_str!("../examples/counter.nu"),
  },
  EmbeddedExample {
    name: "custom-widget",
    source: include_str!("../examples/custom-widget.nu"),
  },
  EmbeddedExample {
    name: "demo",
    source: include_str!("../examples/demo.nu"),
  },
  EmbeddedExample {
    name: "demo2",
    source: include_str!("../examples/demo2.nu"),
  },
  EmbeddedExample {
    name: "effects",
    source: include_str!("../examples/effects.nu"),
  },
  EmbeddedExample {
    name: "gauge",
    source: include_str!("../examples/gauge.nu"),
  },
  EmbeddedExample {
    name: "hello-world",
    source: include_str!("../examples/hello-world.nu"),
  },
  EmbeddedExample {
    name: "input-form",
    source: include_str!("../examples/input-form.nu"),
  },
  EmbeddedExample {
    name: "minimal",
    source: include_str!("../examples/minimal.nu"),
  },
  EmbeddedExample {
    name: "modifiers",
    source: include_str!("../examples/modifiers.nu"),
  },
  EmbeddedExample {
    name: "mouse-drawing",
    source: include_str!("../examples/mouse-drawing.nu"),
  },
  EmbeddedExample {
    name: "popup",
    source: include_str!("../examples/popup.nu"),
  },
  EmbeddedExample {
    name: "release-header",
    source: include_str!("../examples/release-header.nu"),
  },
  EmbeddedExample {
    name: "scrollbar",
    source: include_str!("../examples/scrollbar.nu"),
  },
  EmbeddedExample {
    name: "table",
    source: include_str!("../examples/table.nu"),
  },
  EmbeddedExample {
    name: "todo-list",
    source: include_str!("../examples/todo-list.nu"),
  },
  EmbeddedExample {
    name: "user-input",
    source: include_str!("../examples/user-input.nu"),
  },
  EmbeddedExample {
    name: "volatility-surface",
    source: include_str!("../examples/volatility-surface.nu"),
  },
  EmbeddedExample {
    name: "weather",
    source: include_str!("../examples/weather.nu"),
  },
  EmbeddedExample {
    name: "widget-ref-container",
    source: include_str!("../examples/widget-ref-container.nu"),
  },
];

/// The `tui example` command implementation.
pub struct TuiExample;

impl SimplePluginCommand for TuiExample {
  type Plugin = TuiPlugin;

  /// Returns the command name exposed to Nushell.
  fn name(&self) -> &str {
    "tui example"
  }

  /// Defines the optional embedded example name.
  fn signature(&self) -> Signature {
    Signature::build(self.name())
      .optional(
        "name",
        SyntaxShape::String,
        "Embedded example name; omit to choose interactively",
      )
      .input_output_type(Type::Nothing, Type::Any)
      .category(Category::Experimental)
  }

  /// Describes source retrieval and interactive selection.
  fn description(&self) -> &str {
    "Return an embedded Nushell example, or choose one interactively"
  }

  /// Explains named lookup and interactive selection.
  fn extra_description(&self) -> &str {
    "With a name, returns that example's Nushell source. Without a name, opens a keyboard-driven \
chooser and returns the selected example's source."
  }

  /// Provides an example for source retrieval.
  fn examples(&self) -> Vec<Example<'_>> {
    vec![Example {
      example: "tui example volatility-surface",
      description: "Return the volatility surface example source",
      result: None,
    }]
  }

  /// Resolves an example from its name or the chooser, then returns its source.
  fn run(
    &self,
    _plugin: &TuiPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    _input: &Value,
  ) -> Result<Value, LabeledError> {
    let name: Option<String> = call.opt(0).map_err(|error| labeled_shell_error(&error))?;
    let example = match name {
      Some(name) => find_example(&name, call.head)?,
      None => match choose_example(engine, call.head)? {
        Some(example) => example,
        None => return Ok(Value::nothing(call.head)),
      },
    };

    Ok(Value::string(example.source, call.head))
  }
}

/// Finds one embedded example and reports the valid names on failure.
fn find_example(name: &str, span: Span) -> Result<&'static EmbeddedExample, LabeledError> {
  EXAMPLES
    .iter()
    .find(|example| example.name == name)
    .ok_or_else(|| {
      let names = EXAMPLES
        .iter()
        .map(|example| example.name)
        .collect::<Vec<_>>()
        .join(", ");
      LabeledError::new("Unknown example")
        .with_label(format!("no embedded example named `{name}`"), span)
        .with_help(format!("Available examples: {names}"))
    })
}

/// Opens the foreground chooser and returns its selected example.
fn choose_example(
  engine: &EngineInterface,
  span: Span,
) -> Result<Option<&'static EmbeddedExample>, LabeledError> {
  let previews = highlight_examples(engine, span)?;
  let value = run_in_terminal(engine, "tui example", span, |terminal| {
    choose_in_terminal(terminal, &previews, span).map(|selection| match selection {
      Some(index) => Value::int(index as i64, span),
      None => Value::nothing(span),
    })
  })?;
  match value {
    Value::Int { val, .. } => Ok(EXAMPLES.get(val as usize)),
    Value::Nothing { .. } => Ok(None),
    _ => unreachable!("chooser only returns an index or nothing"),
  }
}

/// Runs the keyboard event loop for the embedded example chooser.
fn choose_in_terminal(
  terminal: &mut DefaultTerminal,
  previews: &[Text<'static>],
  span: Span,
) -> Result<Option<usize>, LabeledError> {
  let mut state = ChooserState {
    list: ListState::default().with_selected(Some(0)),
    preview_scroll: 0,
  };
  loop {
    terminal
      .draw(|frame| draw_chooser(frame, &mut state, previews))
      .map_err(|error| labeled_io_error("failed to draw example chooser", &error, span))?;
    let terminal_event = event::read()
      .map_err(|error| labeled_io_error("failed to read example chooser input", &error, span))?;
    let Event::Key(key) = terminal_event else {
      continue;
    };
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
      continue;
    }
    let selected = state.list.selected().unwrap_or_default();
    match key.code {
      KeyCode::Up | KeyCode::Char('k') => state.list.select_previous(),
      KeyCode::Down | KeyCode::Char('j') => state.list.select_next(),
      KeyCode::Home => state.list.select_first(),
      KeyCode::End => state.list.select_last(),
      KeyCode::PageUp => state.preview_scroll = state.preview_scroll.saturating_sub(10),
      KeyCode::PageDown => state.preview_scroll = state.preview_scroll.saturating_add(10),
      KeyCode::Enter => return Ok(state.list.selected()),
      KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
      KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(None),
      _ => {},
    }
    if state.list.selected() != Some(selected) {
      state.preview_scroll = 0;
    }
  }
}

/// Draws the example list, highlighted code preview, and keyboard help line.
fn draw_chooser(frame: &mut Frame<'_>, state: &mut ChooserState, previews: &[Text<'static>]) {
  let [main_area, help_area] =
    Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());
  let list_width = if main_area.width >= 80 {
    28
  } else {
    (main_area.width / 3).max(16)
  };
  let [list_area, preview_area] =
    Layout::horizontal([Constraint::Length(list_width), Constraint::Min(1)]).areas(main_area);
  let items = EXAMPLES
    .iter()
    .map(|example| ListItem::new(example.name))
    .collect::<Vec<_>>();
  let list = List::new(items)
    .block(Block::bordered().title(" tui examples "))
    .highlight_symbol("› ")
    .highlight_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD));
  frame.render_stateful_widget(list, list_area, &mut state.list);
  let selected = state.list.selected().unwrap_or_default();
  let preview = Paragraph::new(previews[selected].clone())
    .block(Block::bordered().title(format!(" {}.nu ", EXAMPLES[selected].name)))
    .scroll((state.preview_scroll, 0));
  frame.render_widget(preview, preview_area);
  frame.render_widget(
    Paragraph::new("↑/k ↓/j choose  •  PgUp/PgDn scroll code  •  Enter select  •  Esc/q cancel")
      .alignment(Alignment::Center),
    help_area,
  );
}

/// Highlights every embedded script with Nushell's active syntax-color configuration.
fn highlight_examples(
  engine: &EngineInterface,
  span: Span,
) -> Result<Vec<Text<'static>>, LabeledError> {
  let highlighter = engine
    .find_decl("nu-highlight")
    .map_err(|error| labeled_shell_error(&error))?
    .ok_or_else(|| {
      LabeledError::new("Cannot highlight examples")
        .with_label("Nushell's `nu-highlight` command is unavailable", span)
    })?;

  EXAMPLES
    .iter()
    .map(|example| {
      let highlighted = engine
        .call_decl(
          highlighter,
          EvaluatedCall::new(span),
          PipelineData::value(Value::string(example.source, span), None),
          true,
          false,
        )
        .map_err(|error| labeled_shell_error(&error))?;
      let highlighted = highlighted
        .into_value(span)
        .map_err(|error| labeled_shell_error(&error))?;
      let highlighted = highlighted
        .coerce_into_string()
        .map_err(|error| labeled_shell_error(&error))?;
      Ok(ansi_to_text(&highlighted))
    })
    .collect()
}

/// Converts ANSI SGR output into owned Ratatui text without displaying escape bytes.
fn ansi_to_text(input: &str) -> Text<'static> {
  let mut lines = Vec::new();
  let mut spans = Vec::new();
  let mut style = AnsiStyle::default();
  let mut offset = 0;

  while let Some(relative_escape) = input[offset..].find("\u{1b}[") {
    let escape = offset + relative_escape;
    push_styled_text(&input[offset..escape], style, &mut spans, &mut lines);
    let parameters_start = escape + 2;
    let Some(relative_end) = input[parameters_start..].find('m') else {
      push_styled_text(&input[escape..], style, &mut spans, &mut lines);
      offset = input.len();
      break;
    };
    let parameters_end = parameters_start + relative_end;
    apply_sgr(&input[parameters_start..parameters_end], &mut style);
    offset = parameters_end + 1;
  }
  push_styled_text(&input[offset..], style, &mut spans, &mut lines);
  lines.push(Line::from(spans));
  Text::from(lines)
}

/// Appends styled content while preserving source line boundaries.
fn push_styled_text(
  text: &str,
  style: AnsiStyle,
  spans: &mut Vec<TextSpan<'static>>,
  lines: &mut Vec<Line<'static>>,
) {
  for segment in text.split_inclusive('\n') {
    let content = segment.strip_suffix('\n').unwrap_or(segment);
    if !content.is_empty() {
      spans.push(TextSpan::styled(content.to_owned(), style.into_ratatui()));
    }
    if segment.ends_with('\n') {
      lines.push(Line::from(std::mem::take(spans)));
    }
  }
}

/// Applies one ANSI Select Graphic Rendition sequence to the current style.
fn apply_sgr(parameters: &str, style: &mut AnsiStyle) {
  let codes = if parameters.is_empty() {
    vec![0]
  } else {
    parameters
      .split(';')
      .filter_map(|parameter| parameter.parse::<u16>().ok())
      .collect::<Vec<_>>()
  };
  let mut index = 0;
  while index < codes.len() {
    let code = codes[index];
    match code {
      0 => *style = AnsiStyle::default(),
      1 => style.modifiers.insert(Modifier::BOLD),
      2 => style.modifiers.insert(Modifier::DIM),
      3 => style.modifiers.insert(Modifier::ITALIC),
      4 => style.modifiers.insert(Modifier::UNDERLINED),
      5 => style.modifiers.insert(Modifier::SLOW_BLINK),
      6 => style.modifiers.insert(Modifier::RAPID_BLINK),
      7 => style.modifiers.insert(Modifier::REVERSED),
      8 => style.modifiers.insert(Modifier::HIDDEN),
      9 => style.modifiers.insert(Modifier::CROSSED_OUT),
      22 => style.modifiers.remove(Modifier::BOLD | Modifier::DIM),
      23 => style.modifiers.remove(Modifier::ITALIC),
      24 => style.modifiers.remove(Modifier::UNDERLINED),
      25 => style
        .modifiers
        .remove(Modifier::SLOW_BLINK | Modifier::RAPID_BLINK),
      27 => style.modifiers.remove(Modifier::REVERSED),
      28 => style.modifiers.remove(Modifier::HIDDEN),
      29 => style.modifiers.remove(Modifier::CROSSED_OUT),
      30..=37 | 90..=97 => style.foreground = basic_ansi_color(code),
      39 => style.foreground = None,
      40..=47 | 100..=107 => style.background = basic_ansi_color(code - 10),
      49 => style.background = None,
      38 | 48 => {
        let (color, consumed) = extended_ansi_color(&codes[index + 1..]);
        if code == 38 {
          style.foreground = color;
        } else {
          style.background = color;
        }
        index += consumed;
      },
      _ => {},
    }
    index += 1;
  }
}

/// Maps the standard and bright ANSI foreground codes to Ratatui colors.
fn basic_ansi_color(code: u16) -> Option<Color> {
  Some(match code {
    30 => Color::Black,
    31 => Color::Red,
    32 => Color::Green,
    33 => Color::Yellow,
    34 => Color::Blue,
    35 => Color::Magenta,
    36 => Color::Cyan,
    37 => Color::Gray,
    90 => Color::DarkGray,
    91 => Color::LightRed,
    92 => Color::LightGreen,
    93 => Color::LightYellow,
    94 => Color::LightBlue,
    95 => Color::LightMagenta,
    96 => Color::LightCyan,
    97 => Color::White,
    _ => return None,
  })
}

/// Parses an ANSI indexed or true-color suffix and reports consumed parameters.
fn extended_ansi_color(codes: &[u16]) -> (Option<Color>, usize) {
  match codes {
    [5, color, ..] => (u8::try_from(*color).ok().map(Color::Indexed), 2),
    [2, red, green, blue, ..] => (
      u8::try_from(*red)
        .ok()
        .zip(u8::try_from(*green).ok())
        .zip(u8::try_from(*blue).ok())
        .map(|((red, green), blue)| Color::Rgb(red, green, blue)),
      4,
    ),
    _ => (None, 0),
  }
}

impl AnsiStyle {
  /// Converts the accumulated ANSI attributes to a Ratatui style.
  fn into_ratatui(self) -> Style {
    let mut style = Style::new().add_modifier(self.modifiers);
    if let Some(foreground) = self.foreground {
      style = style.fg(foreground);
    }
    if let Some(background) = self.background {
      style = style.bg(background);
    }
    style
  }
}

/// Converts a Nushell shell error into its protocol-safe labelled form.
fn labeled_shell_error(error: &nu_protocol::ShellError) -> LabeledError {
  LabeledError::from_diagnostic(error)
}

/// Creates a source-labelled I/O error.
fn labeled_io_error(context: &str, error: &io::Error, span: Span) -> LabeledError {
  LabeledError::new(context)
    .with_label(error.to_string(), span)
    .with_code("nu_plugin_tui::example_io")
}

#[cfg(test)]
mod tests {
  use std::{collections::BTreeSet, fs};

  use nu_protocol::Span;
  use ratatui::style::{Color, Modifier, Style};

  use super::{EXAMPLES, ansi_to_text, find_example};

  /// Verifies lookup returns the exact source embedded from disk.
  #[test]
  fn finds_embedded_example_source() {
    let example = find_example("volatility-surface", Span::test_data()).expect("known example");

    assert_eq!(
      example.source,
      include_str!("../examples/volatility-surface.nu")
    );
  }

  /// Verifies unknown names produce a focused command error.
  #[test]
  fn rejects_unknown_example_name() {
    let error = find_example("missing", Span::test_data()).expect_err("unknown example");

    assert_eq!(error.msg, "Unknown example");
  }

  /// Verifies the catalog stays sorted for predictable chooser navigation.
  #[test]
  fn keeps_example_catalog_sorted() {
    let names = EXAMPLES
      .iter()
      .map(|example| example.name)
      .collect::<Vec<_>>();
    let mut sorted = names.clone();
    sorted.sort_unstable();

    assert_eq!(names, sorted);
  }

  /// Verifies every Nushell file in examples is embedded exactly once.
  #[test]
  fn embeds_every_example_file() {
    let embedded = EXAMPLES
      .iter()
      .map(|example| format!("{}.nu", example.name))
      .collect::<BTreeSet<_>>();
    let files = fs::read_dir("examples")
      .expect("examples directory")
      .map(|entry| entry.expect("example entry").file_name())
      .filter_map(|name| name.into_string().ok())
      .filter(|name| name.ends_with(".nu"))
      .collect::<BTreeSet<_>>();

    assert_eq!(embedded, files);
  }

  /// Verifies ANSI colors and modifiers become Ratatui spans on the correct lines.
  #[test]
  fn converts_highlighted_ansi_to_styled_text() {
    let text = ansi_to_text(
      "\u{1b}[1;36mlet\u{1b}[0m x = \u{1b}[38;2;1;2;3m1\u{1b}[0m\n\u{1b}[48;5;42m# hi\u{1b}[0m",
    );

    assert_eq!(text.lines.len(), 2);
    assert_eq!(text.lines[0].spans[0].content, "let");
    assert_eq!(
      text.lines[0].spans[0].style,
      Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    );
    assert_eq!(text.lines[0].spans[1].content, " x = ");
    assert_eq!(
      text.lines[0].spans[2].style,
      Style::new().fg(Color::Rgb(1, 2, 3))
    );
    assert_eq!(text.lines[1].spans[0].content, "# hi");
    assert_eq!(
      text.lines[1].spans[0].style,
      Style::new().bg(Color::Indexed(42))
    );
  }

  /// Verifies trailing source newlines remain visible in the code preview.
  #[test]
  fn preserves_highlighted_line_boundaries() {
    let text = ansi_to_text("one\n\ntwo\n");

    assert_eq!(text.lines.len(), 4);
    assert_eq!(text.lines[0].spans[0].content, "one");
    assert!(text.lines[1].spans.is_empty());
    assert_eq!(text.lines[2].spans[0].content, "two");
    assert!(text.lines[3].spans.is_empty());
  }
}
