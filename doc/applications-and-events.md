# Applications and events

`tui run` accepts one application record and returns the application's final state.

## Application fields

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `view` | record or closure | yes | — | A widget tree, or `{ \|state\| ... }` returning one. |
| `state` | any | no | `null` | The initial application state. |
| `on-event` | closure | no | — | Handles every terminal event, including ticks. |
| `on-key` | closure | no | — | Handles key press events. |
| `quit-on-esc` | bool | no | `true` | Whether an Escape key press closes the application. |
| `tick-rate-ms` | int | no | `250` | Event polling and redraw interval in milliseconds; must be at least 1. |

A closure-valued `view` receives the current state as its argument and pipeline input. It is evaluated before each frame is drawn.

## Handler input

Handlers receive one event record as their argument and pipeline input. The plugin adds the current `state` field before invoking each handler.

### Key events

```nu
{
  type: key
  code: enter
  kind: press
  modifiers: [control]
  state: null
}
```

`kind` is `press`, `repeat`, or `release`. Common `code` values include characters, `enter`, `escape`, `backspace`, arrow names, `home`, `end`, `page-up`, `page-down`, `tab`, `back-tab`, `delete`, `insert`, and `f1` through the available function keys.

`modifiers` is a list containing any of `shift`, `control`, `alt`, `super`, `hyper`, or `meta`.

### Mouse events

```nu
{
  type: mouse
  kind: up
  button: left
  column: 12
  row: 4
  modifiers: []
  widget: save
  state: null
}
```

`kind` is `down`, `up`, `drag`, `moved`, `scroll-down`, `scroll-up`, `scroll-left`, or `scroll-right`. `button` is present for button-bearing events and is `left`, `right`, or `middle`. `widget` is present when the coordinates overlap a button.

A button's `on-click` handler runs on a left-button release over that button.

### Other events

| `type` | Additional fields | Description |
| --- | --- | --- |
| `tick` | — | Emitted when no terminal event arrives within `tick-rate-ms`. |
| `resize` | `columns`, `rows` | The terminal size changed. |
| `focus-gained` | — | The terminal gained focus. |
| `focus-lost` | — | The terminal lost focus. |
| `unknown` | — | An unsupported terminal event was received. |

## Handler results

A handler may return:

- `null` to leave the application unchanged.
- Any plain value to replace the state.
- An action record containing at least one of `state`, `view`, or `quit`.

For example, this action updates state and closes the application:

```nu
{state: "saved" quit: true}
```

An action's `view` must be a widget record or a view closure. A record without any action fields is treated as a plain state value.

## Dispatch order and quitting

For a left-button release over a button, handlers run in this order:

1. The button's `on-click` handler.
2. The application's `on-event` handler.

For a key press, `on-key` runs before `on-event`. Key repeats and releases go only to `on-event`. State changes made by an earlier handler are visible to later handlers for the same event.

Ctrl-C always closes the application after handlers run. Escape also closes it when `quit-on-esc` is `true`. The final state is returned to Nushell.

## Example keyboard handler

```nu
tui run {
  state: 0
  quit-on-esc: false
  on-key: { |event|
    match $event.code {
      "+" => ($event.state + 1)
      "q" => {state: $event.state quit: true}
      _ => null
    }
  }
  view: { |count|
    {
      type: paragraph
      text: $"Count: ($count) — press + or q"
      alignment: center
    }
  }
}
```
