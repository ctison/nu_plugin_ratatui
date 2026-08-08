# nu_plugin_ratatui

`nu_plugin_ratatui` exposes [Ratatui](https://ratatui.rs/) as declarative Nushell commands. Build a widget tree with Nu records, keep application state as any Nu value, and use ordinary Nu closures for click and keyboard handlers.

Detailed guides and reference material are available in [`doc/`](doc/README.md).

## Quick start

The plugin targets Nushell 0.114.1.

```nu
cargo build
nu --plugins target/debug/nu_plugin_ratatui examples/counter.nu
```

This one-shot command does not modify your plugin registry.

Widgets can also be declared with constructor commands. The `tui` namespace exposes one subcommand per widget and returns ordinary records:

```nu
let content = tui paragraph "Composable widgets" --alignment center
let progress = tui gauge 0.65 --label "65%" --border

tui (tui layout [$content $progress]
    --direction vertical
    --constraints [{fill: 1} {length: 3}])
```

Required widget fields are positional parameters; optional fields remain named flags and are emitted only when supplied. Constructor output is validated against the same schema used by `tui`.

## Install a release

Download the archive for your platform from the GitHub Release, then verify it against `SHA256SUMS` and extract it. Release archives are named with their Rust target:

- Linux AMD64: `x86_64-unknown-linux-musl`
- Linux ARM64: `aarch64-unknown-linux-musl`
- macOS AMD64: `x86_64-apple-darwin`
- macOS ARM64: `aarch64-apple-darwin`
- Windows AMD64: `x86_64-pc-windows-msvc`
- Windows ARM64: `aarch64-pc-windows-msvc`

Register the extracted executable at a Nushell prompt:

```nu
plugin add /path/to/nu_plugin_ratatui
```

On Windows, the executable is named `nu_plugin_ratatui.exe`. Wait for `plugin add` to finish, then start a new Nushell prompt so it reloads the plugin registry.

## Build from source

Run these commands at a Nushell prompt:

```nu
cargo build --release
plugin add (pwd | path join target release nu_plugin_ratatui)
```

Wait for `plugin add` to finish. Then, at the **next prompt**, run:

```nu
source examples/counter.nu
```

Nushell must parse `plugin add` before it can recognize `tui`. Do not put registration and the example in the same script or `{ ... }` block. If `tui` is still not found after registration, start a new Nushell process so it reloads the plugin registry.

The interactive command requires Nushell's local-socket plugin transport because stdin and stdout must remain available to the terminal.

## Embedded examples

Every `examples/*.nu` script is embedded in the plugin. Pass its filename without the `.nu` suffix to print the source:

```nu
tui example volatility-surface
tui example command-preview
```

Omit the name to choose from an interactive list with a `nu-highlight` syntax-colored source preview. Use Page Up and Page Down to scroll the preview:

```nu
tui example
```

To evaluate returned source explicitly, start a child Nushell with the command text:

```nu
nu --commands (tui example volatility-surface)
```

Nushell 0.114 resolves `source` paths at parse time and plugin commands are not const, so `source (tui example ...)` cannot be evaluated by Nushell itself.

## Counter example

```nu
(tui
  --state 0
  { |count|
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
)
```

`tui` returns the final state, so the example evaluates to the click count after the UI closes. A complete runnable version is in [`examples/counter.nu`](examples/counter.nu).

The [`examples/`](examples/README.md) gallery also contains dependency-free Nushell ports of 26 applications from Ratatui's upstream examples, plus a focused ANSI command-preview example.

## Application command

| Parameter | Type | Meaning |
| --- | --- | --- |
| `view` | positional record or closure | Required widget tree, or `{ \|state\| ... }` returning one |
| `--state` | any | Initial state; defaults to `null` |
| `--on-event` | closure | Handles every terminal event |
| `--on-key` | closure | Handles key press events |
| `--quit-on-esc` | bool | Exit on Escape; defaults to `true` |
| `--tick-rate-ms` | int | Redraw and tick interval; defaults to `250` |

Handlers receive one event record. Every event includes `type` and the current `state`; key events also contain `code`, `kind`, and `modifiers`, while mouse events contain coordinates, the mouse action, and `widget` when over an interactive widget.

A handler can return:

- `null` to make no change.
- Any plain value to replace application state.
- An action record containing one or more of `state`, `view`, and `quit`.

Ctrl-C always exits. Escape exits unless `--quit-on-esc` is false.

## Widgets

All widgets are records with a `type` field. Write them directly or construct them with `tui <type>` flags.

- Ratatui widgets: `bar-chart`, `calendar`, `canvas`, `chart`, `clear`, `fill`, `gauge`, `line-gauge`, `list`, `logo`, `mascot`, `paragraph`, `scrollbar`, `sparkline`, `table`, and `tabs`.
- Plugin composition widgets: `layout`, `button`, and `spacer`.

The records for data-driven widgets use ordinary Nu values: bar records for `bar-chart`, `[x y]` pairs for `chart`, point records for `canvas`, nested string lists for `table`, and integer or null samples for `sparkline`. See [`doc/widgets.md`](doc/widgets.md) for every field and a runnable record for each widget.

Constraints are one-field records: `{length: 3}`, `{percentage: 50}`, `{min: 10}`, `{max: 20}`, or `{fill: 1}`. When omitted, every child gets `{fill: 1}`.

Widget presentation fields include `title`, `border`, `border-type` (`plain`, `rounded`, `double`, or `thick`), `style`, and `border-style`. Styles accept `fg`, `bg`, `bold`, `italic`, `underlined`, `reversed`, and `dim`. Colors may be Ratatui color names, hex strings such as `#5fd7ff`, or indexed color integers from 0 to 255.

Use `tui style --fg cyan --bold` to construct a validated style record for any style-bearing widget flag. Boolean switches accept explicit false values with equals syntax, such as `--underlined=false`; `--underline` is also accepted as an alias.
