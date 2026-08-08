# Examples

Build the plugin, then run any example directly without registering it:

```sh
cargo build
nu --plugins target/debug/nu_plugin_ratatui examples/gauge.nu
```

Every example is a standalone Nushell script and uses only Nushell built-ins plus the plugin. Press `Esc` or `q` to exit interactive examples; the instruction line in each application lists any additional controls.

Installed plugins embed this entire gallery. Retrieve a named example's source, or omit the name to open a chooser with a syntax-highlighted code preview:

```nu
tui example gauge
tui example
```

## Ratatui app ports

These 26 scripts adapt examples from [Ratatui's `examples/apps` directory](https://github.com/ratatui/ratatui/tree/main/examples/apps):

| Upstream app | Nushell port | Notes |
| --- | --- | --- |
| advanced-widget-impl | [`advanced-widget-impl.nu`](advanced-widget-impl.nu) | Shows equivalent declarative ownership/state patterns. |
| calendar-explorer | [`calendar-explorer.nu`](calendar-explorer.nu) | Interactive year and calendar-style selection. |
| canvas | [`canvas.nu`](canvas.nu) | Point-based shapes, markers, and movement. |
| chart | [`chart.nu`](chart.nu) | Animated line and scatter datasets. |
| color-explorer | [`color-explorer.nu`](color-explorer.nu) | All 256 indexed terminal colors. |
| colors-rgb | [`colors-rgb.nu`](colors-rgb.nu) | Animated true-color palette. |
| constraint-explorer | [`constraint-explorer.nu`](constraint-explorer.nu) | Interactive supported constraint types. |
| constraints | [`constraints.nu`](constraints.nu) | Length, percentage, min, max, and fill layouts. |
| custom-widget | [`custom-widget.nu`](custom-widget.nu) | Clickable custom cell grid. |
| demo | [`demo.nu`](demo.nu) | Multi-tab built-in widget showcase. |
| demo2 | [`demo2.nu`](demo2.nu) | Multi-tab dashboard adaptation. |
| gauge | [`gauge.nu`](gauge.nu) | Animated gauges and line gauges. |
| hello-world | [`hello-world.nu`](hello-world.nu) | Centered bordered greeting. |
| input-form | [`input-form.nu`](input-form.nu) | Focused name and age fields. |
| minimal | [`minimal.nu`](minimal.nu) | Smallest complete application. |
| modifiers | [`modifiers.nu`](modifiers.nu) | Supported text modifier grid. |
| mouse-drawing | [`mouse-drawing.nu`](mouse-drawing.nu) | Dependency-free point drawing. |
| popup | [`popup.nu`](popup.nu) | Centered popup using nested layouts. |
| release-header | [`release-header.nu`](release-header.nu) | Logo and release menu banner. |
| scrollbar | [`scrollbar.nu`](scrollbar.nu) | Four interactive orientations. |
| table | [`table.nu`](table.nu) | Header, rows, and column constraints. |
| todo-list | [`todo-list.nu`](todo-list.nu) | Selection and status changes. |
| user-input | [`user-input.nu`](user-input.nu) | Editing, cursor movement, and history. |
| volatility-surface | [`volatility-surface.nu`](volatility-surface.nu) | Animated projected point cloud. |
| weather | [`weather.nu`](weather.nu) | Deterministic hourly bar chart. |
| widget-ref-container | [`widget-ref-container.nu`](widget-ref-container.nu) | Heterogeneous widget records in a container. |

[`counter.nu`](counter.nu) is the plugin's original mouse-handler example rather than an upstream port.
[`command-preview.nu`](command-preview.nu) demonstrates a two-column selector that runs Git
subcommands on selection and displays cached ANSI output with two-axis scrolling.

## Not ported

Six upstream applications rely on behavior outside the current plugin model:

| Upstream app | Reason |
| --- | --- |
| async-github | Requires asynchronous HTTP access and a live GitHub service. |
| flex | Requires Ratatui's `Flex` layout setting, which the plugin does not expose. |
| hyperlink | Requires styled spans containing OSC 8 terminal control sequences. |
| inline | Requires an inline terminal viewport rather than the plugin's alternate screen. |
| panic | Demonstrates Rust panic hooks and terminal restoration. |
| tracing | Requires the Rust `tracing` stack and file logging. |
