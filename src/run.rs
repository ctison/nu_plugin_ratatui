use std::io::{self, stdout};

use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{
  Category, Example, LabeledError, Record, Signature, Span, Spanned, SyntaxShape, Type, Value,
  engine::Closure,
};
use ratatui::{DefaultTerminal, crossterm};

use crossterm::{
  event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
  },
  execute,
};

use crate::{
  config::{AppConfig, Handler, UiNode},
  plugin::TuiPlugin,
  ui::HitTarget,
};

/// The root `tui` command implementation.
pub struct Tui;

/// Mutable application values that handlers may replace.
struct AppState {
  state: Value,
  view: Value,
  quit: bool,
}

/// A terminal event plus metadata used for handler dispatch.
struct DispatchEvent {
  value: Value,
  button_handler: Option<Handler>,
  is_key_press: bool,
  force_quit: bool,
}

/// Owns Ratatui terminal initialization and guarantees restoration on drop.
struct TerminalSession {
  terminal: DefaultTerminal,
}

impl SimplePluginCommand for Tui {
  type Plugin = TuiPlugin;

  /// Returns the command name exposed to Nushell.
  fn name(&self) -> &str {
    "tui"
  }

  /// Defines the positional view, application flags, and final-state output.
  fn signature(&self) -> Signature {
    Signature::build(self.name())
      .required(
        "view",
        SyntaxShape::OneOf(vec![
          SyntaxShape::Record(Vec::new().into()),
          view_closure_shape(),
        ]),
        "Widget record, or closure(any) taking state and returning a widget record",
      )
      .named(
        "state",
        SyntaxShape::Any,
        "Initial application state (default: null)",
        None,
      )
      .named(
        "on-event",
        handler_shape(),
        "Closure taking an event record and returning null, state, or an action record (default: no handler)",
        None,
      )
      .named(
        "on-key",
        handler_shape(),
        "Closure taking a key event record and returning null, state, or an action record (default: no handler)",
        None,
      )
      .switch(
        "quit-on-esc",
        "Whether Escape exits the application (default: true)",
        None,
      )
      .named(
        "tick-rate-ms",
        SyntaxShape::Int,
        "Event polling and redraw interval in milliseconds (default: 250)",
        None,
      )
      .input_output_type(Type::Nothing, Type::Any)
      .category(Category::Experimental)
  }

  /// Describes the command in completion and help output.
  fn description(&self) -> &str {
    "Run a Ratatui application described by Nushell records and closures"
  }

  /// Documents the reactive view and handler return contracts.
  fn extra_description(&self) -> &str {
    r#"Closure contracts:

The positional `view` closure receives one `state` parameter of type `any` and the same state as
pipeline input. It must return a widget record.

The `--on-key` closure receives one `event` parameter and the same record as pipeline input:

{
  type: 'key'
  code: string
  kind: 'press' | 'repeat' | 'release'
  modifiers: list<'shift' | 'control' | 'alt' | 'super' | 'hyper' | 'meta'>
  state: any
}

The `--on-event` closure receives one `event` parameter and the same record as pipeline input. Key
events have the shape above. Mouse events have this shape:

{
  type: 'mouse'
  kind: 'down' | 'up' | 'drag' | 'moved' | 'scroll-down' | 'scroll-up' | 'scroll-left' | 'scroll-right'
  button?: 'left' | 'right' | 'middle'
  column: int
  row: int
  modifiers: list<'shift' | 'control' | 'alt' | 'super' | 'hyper' | 'meta'>
  widget?: string
  state: any
}

Resize events have this shape:

{
  type: 'resize'
  columns: int
  rows: int
  state: any
}

Focus, tick, and unknown events have this shape:

{
  type: 'focus-gained' | 'focus-lost' | 'tick' | 'unknown'
  state: any
}

Each event handler may return `null` (no change), any value (new state), or this action record, in
which every field is optional:

{
  state?: any
  view?: record | closure(any)
  quit?: bool
}

The command returns the final state (`any`). Escape exits by default; Ctrl-C always exits."#
  }

  /// Provides a compact clickable counter example in `help tui`.
  fn examples(&self) -> Vec<Example<'_>> {
    vec![Example {
      example: r#"tui --state 0 { |count|
    {
      type: layout
      direction: vertical
      constraints: [{length: 3} {fill: 1}]
      children: [
        {
          type: button
          id: increment
          label: $"Clicked ($count) times"
          border: true
          border-type: rounded
          alignment: center
          on-click: { |event| {state: ($event.state + 1)} }
        }
        {type: paragraph text: "Press Escape to exit" alignment: center}
      ]
    }
}"#,
      description: "Run a button whose closure updates application state",
      result: None,
    }]
  }

  /// Parses the application and owns its foreground terminal lifecycle.
  fn run(
    &self,
    _plugin: &TuiPlugin,
    engine: &EngineInterface,
    call: &EvaluatedCall,
    _input: &Value,
  ) -> Result<Value, LabeledError> {
    let config = parse_config(call)?;
    run_application(engine, self.name(), &config, call.head)
  }
}

/// Builds the shape for a reactive view closure accepting one state parameter.
fn view_closure_shape() -> SyntaxShape {
  SyntaxShape::Closure(Some(vec![SyntaxShape::Any]))
}

/// Builds the shape for an event handler closure accepting one record parameter.
fn handler_shape() -> SyntaxShape {
  SyntaxShape::Closure(Some(vec![SyntaxShape::Record(Default::default())]))
}

/// Assembles and validates application configuration from command arguments.
fn parse_config(call: &EvaluatedCall) -> Result<AppConfig, LabeledError> {
  let view: Value = call.req(0).map_err(|error| labeled_shell_error(&error))?;
  let mut record = Record::new();
  record.push("view", view);
  for (name, value) in &call.named {
    if let Some(value) = value {
      record.push(name.item.clone(), value.clone());
    }
  }
  let config_value = Value::record(record, call.head);
  AppConfig::parse(&config_value, call.head)
}

/// Runs a parsed application with the shared foreground terminal lifecycle.
pub(crate) fn run_application(
  engine: &EngineInterface,
  command_name: &str,
  config: &AppConfig,
  span: Span,
) -> Result<Value, LabeledError> {
  run_in_terminal(engine, command_name, span, |terminal| {
    run_loop(engine, terminal, config, span)
  })
}

/// Runs work with exclusive foreground access and a restored Ratatui terminal.
pub(crate) fn run_in_terminal(
  engine: &EngineInterface,
  command_name: &str,
  span: Span,
  run: impl FnOnce(&mut DefaultTerminal) -> Result<Value, LabeledError>,
) -> Result<Value, LabeledError> {
  if engine.is_using_stdio() {
    return Err(
      LabeledError::new(format!(
        "`{command_name}` requires Nushell's local-socket plugin mode"
      ))
      .with_label(
        "stdio is occupied by the plugin protocol and cannot host a TUI",
        span,
      )
      .with_help("Register and invoke the plugin with a Nushell build that supports local sockets"),
    );
  }

  let foreground = engine
    .enter_foreground()
    .map_err(|error| labeled_shell_error(&error))?;
  let mut session = TerminalSession::enter(span)?;
  let result = run(&mut session.terminal);
  drop(session);
  let leave_result = foreground
    .leave()
    .map_err(|error| labeled_shell_error(&error));

  match (result, leave_result) {
    (Err(error), _) => Err(error),
    (Ok(_), Err(error)) => Err(error),
    (Ok(value), Ok(())) => Ok(value),
  }
}

impl TerminalSession {
  /// Enters raw alternate-screen mode and enables mouse reporting.
  fn enter(span: Span) -> Result<Self, LabeledError> {
    let terminal = ratatui::try_init()
      .map_err(|error| labeled_io_error("failed to initialize terminal", &error, span))?;
    if let Err(error) = execute!(stdout(), EnableMouseCapture) {
      let _ = ratatui::try_restore();
      return Err(labeled_io_error(
        "failed to enable mouse capture",
        &error,
        span,
      ));
    }
    Ok(Self { terminal })
  }
}

impl Drop for TerminalSession {
  /// Disables mouse reporting and restores the user's terminal.
  fn drop(&mut self) {
    let _ = execute!(stdout(), DisableMouseCapture);
    let _ = ratatui::try_restore();
  }
}

/// Runs redraws, terminal polling, and closure dispatch until the app quits.
fn run_loop(
  engine: &EngineInterface,
  terminal: &mut DefaultTerminal,
  config: &AppConfig,
  span: Span,
) -> Result<Value, LabeledError> {
  let mut app = AppState {
    state: config.state.clone(),
    view: config.view.clone(),
    quit: false,
  };

  while !app.quit {
    let mut hits = draw_view(engine, terminal, &app, span)?;
    let event = if event::poll(config.tick_rate)
      .map_err(|error| labeled_io_error("failed to poll terminal events", &error, span))?
    {
      let terminal_event = event::read()
        .map_err(|error| labeled_io_error("failed to read terminal event", &error, span))?;
      dispatch_event(terminal_event, &mut hits, span)
    } else {
      DispatchEvent {
        value: record_value(vec![("type", Value::string("tick", span))], span),
        button_handler: None,
        is_key_press: false,
        force_quit: false,
      }
    };

    if let Some(handler) = event.button_handler {
      invoke_handler(engine, &handler, &event.value, &mut app)?;
    }
    if event.is_key_press
      && let Some(handler) = config.on_key.clone()
    {
      invoke_handler(engine, &handler, &event.value, &mut app)?;
    }
    if let Some(handler) = config.on_event.clone() {
      invoke_handler(engine, &handler, &event.value, &mut app)?;
    }
    if event.force_quit {
      app.quit = true;
    }
    if config.quit_on_esc && is_escape_event(&event.value) {
      app.quit = true;
    }
  }

  Ok(app.state)
}

/// Evaluates a reactive view, parses it, and draws a complete frame.
fn draw_view(
  engine: &EngineInterface,
  terminal: &mut DefaultTerminal,
  app: &AppState,
  span: Span,
) -> Result<Vec<HitTarget>, LabeledError> {
  let view_value = match &app.view {
    Value::Closure { val, .. } => {
      let closure = Spanned {
        item: (**val).clone(),
        span: app.view.span(),
      };
      engine
        .eval_closure(&closure, vec![app.state.clone()], Some(app.state.clone()))
        .map_err(|error| labeled_shell_error(&error))?
    },
    value => value.clone(),
  };
  let root = UiNode::parse(&view_value)?;
  let mut hits = Vec::new();
  terminal
    .draw(|frame| {
      root.render(frame, frame.area(), &mut hits);
    })
    .map_err(|error| labeled_io_error("failed to draw terminal frame", &error, span))?;
  Ok(hits)
}

/// Converts one Crossterm event into a Nushell event record and handler target.
fn dispatch_event(event: Event, hits: &mut [HitTarget], span: Span) -> DispatchEvent {
  match event {
    Event::Key(key) => DispatchEvent {
      value: key_event_value(key, span),
      button_handler: None,
      is_key_press: key.kind == KeyEventKind::Press,
      force_quit: key.kind == KeyEventKind::Press
        && key.code == KeyCode::Char('c')
        && key.modifiers.contains(KeyModifiers::CONTROL),
    },
    Event::Mouse(mouse) => mouse_dispatch_event(mouse, hits, span),
    Event::Resize(columns, rows) => DispatchEvent {
      value: record_value(
        vec![
          ("type", Value::string("resize", span)),
          ("columns", Value::int(i64::from(columns), span)),
          ("rows", Value::int(i64::from(rows), span)),
        ],
        span,
      ),
      button_handler: None,
      is_key_press: false,
      force_quit: false,
    },
    Event::FocusGained => basic_event("focus-gained", span),
    Event::FocusLost => basic_event("focus-lost", span),
    _ => basic_event("unknown", span),
  }
}

/// Converts a mouse event and performs button hit-testing for left-button releases.
fn mouse_dispatch_event(mouse: MouseEvent, hits: &mut [HitTarget], span: Span) -> DispatchEvent {
  let target = hits
    .iter()
    .find(|target| target.contains(mouse.column, mouse.row));
  let mut fields = vec![
    ("type", Value::string("mouse", span)),
    ("kind", Value::string(mouse_kind(mouse.kind), span)),
    ("column", Value::int(i64::from(mouse.column), span)),
    ("row", Value::int(i64::from(mouse.row), span)),
    ("modifiers", modifiers_value(mouse.modifiers, span)),
  ];
  if let Some(button) = mouse_button(mouse.kind) {
    fields.push(("button", Value::string(button, span)));
  }
  if let Some(target) = target {
    fields.push(("widget", Value::string(target.id.clone(), span)));
  }
  let button_handler = if matches!(mouse.kind, MouseEventKind::Up(MouseButton::Left)) {
    target.and_then(|target| target.handler.clone())
  } else {
    None
  };
  DispatchEvent {
    value: record_value(fields, span),
    button_handler,
    is_key_press: false,
    force_quit: false,
  }
}

/// Invokes a Nushell handler with an event record containing current state.
fn invoke_handler(
  engine: &EngineInterface,
  handler: &Spanned<Closure>,
  event: &Value,
  app: &mut AppState,
) -> Result<(), LabeledError> {
  let mut context = event
    .clone()
    .into_record()
    .map_err(|error| labeled_shell_error(&error))?;
  context.push("state", app.state.clone());
  let context = Value::record(context, event.span());
  let result = engine
    .eval_closure(handler, vec![context.clone()], Some(context))
    .map_err(|error| labeled_shell_error(&error))?;
  apply_handler_result(result, app)
}

/// Applies a returned state value or explicit action record to the application.
fn apply_handler_result(result: Value, app: &mut AppState) -> Result<(), LabeledError> {
  if matches!(result, Value::Nothing { .. }) {
    return Ok(());
  }
  let Value::Record { val: record, .. } = &result else {
    app.state = result;
    return Ok(());
  };
  let is_action = ["state", "view", "quit"]
    .iter()
    .any(|field| record.get(*field).is_some());
  if !is_action {
    app.state = result;
    return Ok(());
  }
  if let Some(state) = record.get("state") {
    app.state = state.clone();
  }
  if let Some(view) = record.get("view") {
    if !matches!(view, Value::Record { .. } | Value::Closure { .. }) {
      return Err(
        LabeledError::new("Invalid handler action")
          .with_label("`view` must be a widget record or closure", view.span()),
      );
    }
    app.view = view.clone();
  }
  if let Some(quit) = record.get("quit") {
    app.quit = quit
      .as_bool()
      .map_err(|error| labeled_shell_error(&error))?;
  }
  Ok(())
}

/// Creates a key event record with normalized code, kind, and modifiers.
fn key_event_value(key: KeyEvent, span: Span) -> Value {
  record_value(
    vec![
      ("type", Value::string("key", span)),
      ("code", Value::string(key_code(key.code), span)),
      ("kind", Value::string(key_kind(key.kind), span)),
      ("modifiers", modifiers_value(key.modifiers, span)),
    ],
    span,
  )
}

/// Converts a Crossterm key code into a stable Nushell string.
fn key_code(code: KeyCode) -> String {
  match code {
    KeyCode::Char(character) => character.to_string(),
    KeyCode::F(number) => format!("f{number}"),
    KeyCode::Backspace => "backspace".into(),
    KeyCode::Enter => "enter".into(),
    KeyCode::Left => "left".into(),
    KeyCode::Right => "right".into(),
    KeyCode::Up => "up".into(),
    KeyCode::Down => "down".into(),
    KeyCode::Home => "home".into(),
    KeyCode::End => "end".into(),
    KeyCode::PageUp => "page-up".into(),
    KeyCode::PageDown => "page-down".into(),
    KeyCode::Tab => "tab".into(),
    KeyCode::BackTab => "back-tab".into(),
    KeyCode::Delete => "delete".into(),
    KeyCode::Insert => "insert".into(),
    KeyCode::Null => "null".into(),
    KeyCode::Esc => "escape".into(),
    other => format!("{other:?}").to_lowercase(),
  }
}

/// Converts a Crossterm key event kind into a stable string.
fn key_kind(kind: KeyEventKind) -> &'static str {
  match kind {
    KeyEventKind::Press => "press",
    KeyEventKind::Repeat => "repeat",
    KeyEventKind::Release => "release",
  }
}

/// Converts a Crossterm mouse kind into a stable string.
fn mouse_kind(kind: MouseEventKind) -> &'static str {
  match kind {
    MouseEventKind::Down(_) => "down",
    MouseEventKind::Up(_) => "up",
    MouseEventKind::Drag(_) => "drag",
    MouseEventKind::Moved => "moved",
    MouseEventKind::ScrollDown => "scroll-down",
    MouseEventKind::ScrollUp => "scroll-up",
    MouseEventKind::ScrollLeft => "scroll-left",
    MouseEventKind::ScrollRight => "scroll-right",
  }
}

/// Extracts the mouse button name from button-bearing event kinds.
fn mouse_button(kind: MouseEventKind) -> Option<&'static str> {
  let button = match kind {
    MouseEventKind::Down(button) | MouseEventKind::Up(button) | MouseEventKind::Drag(button) => {
      button
    },
    _ => return None,
  };
  Some(match button {
    MouseButton::Left => "left",
    MouseButton::Right => "right",
    MouseButton::Middle => "middle",
  })
}

/// Converts modifier flags into a Nushell list of names.
fn modifiers_value(modifiers: KeyModifiers, span: Span) -> Value {
  let mut names = Vec::new();
  for (modifier, name) in [
    (KeyModifiers::SHIFT, "shift"),
    (KeyModifiers::CONTROL, "control"),
    (KeyModifiers::ALT, "alt"),
    (KeyModifiers::SUPER, "super"),
    (KeyModifiers::HYPER, "hyper"),
    (KeyModifiers::META, "meta"),
  ] {
    if modifiers.contains(modifier) {
      names.push(Value::string(name, span));
    }
  }
  Value::list(names, span)
}

/// Creates a non-interactive event with only its type field.
fn basic_event(event_type: &str, span: Span) -> DispatchEvent {
  DispatchEvent {
    value: record_value(vec![("type", Value::string(event_type, span))], span),
    button_handler: None,
    is_key_press: false,
    force_quit: false,
  }
}

/// Checks whether a normalized event record is an Escape key press.
fn is_escape_event(value: &Value) -> bool {
  let Ok(record) = value.as_record() else {
    return false;
  };
  record.get("type").and_then(|value| value.as_str().ok()) == Some("key")
    && record.get("code").and_then(|value| value.as_str().ok()) == Some("escape")
    && record.get("kind").and_then(|value| value.as_str().ok()) == Some("press")
}

/// Constructs a Nushell record value from ordered key/value fields.
fn record_value(fields: Vec<(&str, Value)>, span: Span) -> Value {
  let mut record = Record::new();
  for (name, value) in fields {
    record.push(name, value);
  }
  Value::record(record, span)
}

/// Converts a Nushell shell error into its protocol-safe labelled form.
fn labeled_shell_error(error: &nu_protocol::ShellError) -> LabeledError {
  LabeledError::from_diagnostic(error)
}

/// Creates a source-labelled terminal I/O error.
fn labeled_io_error(context: &str, error: &io::Error, span: Span) -> LabeledError {
  LabeledError::new(context)
    .with_label(error.to_string(), span)
    .with_code("nu_plugin_tui::terminal_io")
}

#[cfg(test)]
mod tests {
  use nu_plugin::{EvaluatedCall, SimplePluginCommand};
  use nu_protocol::{IntoSpanned, Record, Span, Value};
  use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

  use super::{
    AppState, Tui, apply_handler_result, is_escape_event, key_event_value, parse_config,
  };

  /// Verifies root help documents closure parameters, pipeline input, and result types.
  #[test]
  fn documents_root_closure_contracts() {
    let help = Tui.extra_description();

    assert!(help.contains("`state` parameter"));
    assert!(help.contains("`event` parameter"));
    assert!(help.contains("pipeline input"));
    assert!(help.contains("return `null`"));
    assert!(help.contains("type: 'key'"));
    assert!(help.contains("kind: 'press' | 'repeat' | 'release'"));
    assert!(help.contains("type: 'mouse'"));
    assert!(help.contains("button?: 'left' | 'right' | 'middle'"));
    assert!(help.contains("type: 'resize'"));
    assert!(help.contains("type: 'focus-gained' | 'focus-lost' | 'tick' | 'unknown'"));
    assert!(
      help.contains("modifiers: list<'shift' | 'control' | 'alt' | 'super' | 'hyper' | 'meta'>")
    );
    assert!(help.contains("view?: record | closure(any)"));
    assert!(help.contains("final state (`any`)"));
  }

  /// Verifies every optional root flag documents its runtime default.
  #[test]
  fn documents_defaults_for_all_root_flags() {
    for flag in Tui
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

  /// Verifies that the positional view and named flags form application configuration.
  #[test]
  fn parses_flattened_command_arguments() {
    let span = Span::test_data();
    let mut view = Record::new();
    view.push("type", Value::test_string("spacer"));
    let mut call = EvaluatedCall::new(span);
    call.add_positional(Value::test_record(view));
    call.add_named("state".into_spanned(span), Value::test_int(42));
    call.add_named("quit-on-esc".into_spanned(span), Value::test_bool(false));
    call.add_named("tick-rate-ms".into_spanned(span), Value::test_int(75));

    let config = parse_config(&call).expect("valid flattened arguments");

    assert_eq!(config.state.as_int().expect("integer state"), 42);
    assert!(!config.quit_on_esc);
    assert_eq!(config.tick_rate.as_millis(), 75);
  }

  /// Builds a test record value from concise key/value pairs.
  fn record(fields: Vec<(&str, Value)>) -> Value {
    let mut record = Record::new();
    for (name, value) in fields {
      record.push(name, value);
    }
    Value::test_record(record)
  }

  /// Creates an empty application state for action tests.
  fn app_state() -> AppState {
    AppState {
      state: Value::test_nothing(),
      view: record(vec![("type", Value::test_string("spacer"))]),
      quit: false,
    }
  }

  /// Verifies that explicit action fields update state and quit together.
  #[test]
  fn applies_handler_action() {
    let action = record(vec![
      ("state", Value::test_int(42)),
      ("quit", Value::test_bool(true)),
    ]);
    let mut app = app_state();

    apply_handler_result(action, &mut app).expect("valid action");

    assert_eq!(app.state.as_int().expect("integer state"), 42);
    assert!(app.quit);
  }

  /// Verifies that plain handler values become application state directly.
  #[test]
  fn applies_plain_state_result() {
    let mut app = app_state();

    apply_handler_result(Value::test_string("ready"), &mut app).expect("valid state");

    assert_eq!(app.state.as_str().expect("string state"), "ready");
  }

  /// Verifies normalized Escape press detection without matching repeats.
  #[test]
  fn detects_escape_press() {
    let span = Span::test_data();
    let press = key_event_value(
      KeyEvent::new_with_kind(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Press),
      span,
    );
    let repeat = key_event_value(
      KeyEvent::new_with_kind(KeyCode::Esc, KeyModifiers::NONE, KeyEventKind::Repeat),
      span,
    );

    assert!(is_escape_event(&press));
    assert!(!is_escape_event(&repeat));
  }
}
