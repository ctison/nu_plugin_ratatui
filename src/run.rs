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

/// The `tui run` command implementation.
pub struct TuiRun;

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

impl SimplePluginCommand for TuiRun {
    type Plugin = TuiPlugin;

    /// Returns the command name exposed to Nushell.
    fn name(&self) -> &str {
        "tui run"
    }

    /// Defines the accepted application record and final-state output.
    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "config",
                SyntaxShape::Record(Vec::new().into()),
                "Application record containing `view`, handlers, and optional state",
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
        "The `view` field may be a widget record or a `{ |state| ... }` closure returning one. \
Handlers receive one event record containing the current `state`. A handler may return a new state \
directly, or an action record such as `{ state: $next, quit: false }`. Supported action fields are \
`state`, `view`, and `quit`. Escape exits by default; Ctrl-C always exits."
    }

    /// Provides a compact clickable counter example in `help tui run`.
    fn examples(&self) -> Vec<Example<'_>> {
        vec![Example {
            example: r#"tui run {
  state: 0
  view: { |count|
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
        let config_value: Value = call.req(0).map_err(|error| labeled_shell_error(&error))?;
        let config = AppConfig::parse(&config_value, call.head)?;
        run_application(engine, self.name(), &config, call.head)
    }
}

/// Runs a parsed application with the shared foreground terminal lifecycle.
pub(crate) fn run_application(
    engine: &EngineInterface,
    command_name: &str,
    config: &AppConfig,
    span: Span,
) -> Result<Value, LabeledError> {
    if engine.is_using_stdio() {
        return Err(LabeledError::new(format!(
            "`{command_name}` requires Nushell's local-socket plugin mode"
        ))
        .with_label(
            "stdio is occupied by the plugin protocol and cannot host a TUI",
            span,
        )
        .with_help(
            "Register and invoke the plugin with a Nushell build that supports local sockets",
        ));
    }

    let foreground = engine
        .enter_foreground()
        .map_err(|error| labeled_shell_error(&error))?;
    let mut session = TerminalSession::enter(span)?;
    let result = run_loop(engine, &mut session.terminal, config, span);
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
        }
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
            return Err(LabeledError::new("Invalid handler action")
                .with_label("`view` must be a widget record or closure", view.span()));
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
        MouseEventKind::Down(button)
        | MouseEventKind::Up(button)
        | MouseEventKind::Drag(button) => button,
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
    use nu_protocol::{Record, Span, Value};
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    use super::{AppState, apply_handler_result, is_escape_event, key_event_value};

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
