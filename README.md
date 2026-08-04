# nu_plugin_tui

`nu_plugin_tui` exposes [Ratatui](https://ratatui.rs/) as declarative Nushell commands. Build a widget tree with Nu records, keep application state as any Nu value, and use ordinary Nu closures for click and keyboard handlers.

## Quick start

The plugin targets Nushell 0.114.1.

```nu
cargo build
nu --plugins target/debug/nu_plugin_tui examples/counter.nu
```

This one-shot command does not modify your plugin registry.

## Install

Run these commands at a Nushell prompt:

```nu
cargo build --release
plugin add (pwd | path join target release nu_plugin_tui)
```

Wait for `plugin add` to finish. Then, at the **next prompt**, run:

```nu
source examples/counter.nu
```

Nushell must parse `plugin add` before it can recognize `tui run`. Do not put registration and the example in the same script or `{ ... }` block. If `tui run` is still not found after registration, start a new Nushell process so it reloads the plugin registry.

The interactive command requires Nushell's local-socket plugin transport because stdin and stdout must remain available to the terminal.

## Counter example

```nu
tui run {
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
}
```

`tui run` returns the final state, so the example evaluates to the click count after the UI closes. A complete runnable version is in [`examples/counter.nu`](examples/counter.nu).

## Application record

| Field | Type | Meaning |
| --- | --- | --- |
| `view` | record or closure | Required widget tree, or `{ \|state\| ... }` returning one |
| `state` | any | Initial state; defaults to `null` |
| `on-event` | closure | Handles every terminal event |
| `on-key` | closure | Handles key press events |
| `quit-on-esc` | bool | Exit on Escape; defaults to `true` |
| `tick-rate-ms` | int | Redraw and tick interval; defaults to `250` |

Handlers receive one event record. Every event includes `type` and the current `state`; key events also contain `code`, `kind`, and `modifiers`, while mouse events contain coordinates, the mouse action, and `widget` when over an interactive widget.

A handler can return:

- `null` to make no change.
- Any plain value to replace application state.
- An action record containing one or more of `state`, `view`, and `quit`.

Ctrl-C always exits. Escape exits unless `quit-on-esc` is false.

## Widgets

All widgets are records with a `type` field.

- `layout`: `direction` (`vertical` or `horizontal`), `children`, and optional `constraints`.
- `paragraph`: `text`, optional `alignment`, and `wrap`.
- `button`: `id`, `label`, and optional `on-click` closure.
- `list`: string `items`.
- `gauge`: `ratio` from `0.0` to `1.0` and optional `label`.
- `spacer`: reserves its layout area without drawing.

Constraints are one-field records: `{length: 3}`, `{percentage: 50}`, `{min: 10}`, `{max: 20}`, or `{fill: 1}`. When omitted, every child gets `{fill: 1}`.

Widget presentation fields include `title`, `border`, `border-type` (`plain`, `rounded`, `double`, or `thick`), `style`, and `border-style`. Styles accept `fg`, `bg`, `bold`, `italic`, `underlined`, `reversed`, and `dim`. Colors may be Ratatui color names, hex strings such as `#5fd7ff`, or indexed color integers from 0 to 255.
