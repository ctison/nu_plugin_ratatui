# Getting started

The plugin targets Nushell 0.114.1 and provides the `tui run` and `tui example` commands.

## Build and run without installing

Build the debug binary from the repository root:

```sh
cargo build
```

Then let Nushell load that binary for one process:

```sh
nu --plugins target/debug/nu_plugin_tui examples/counter.nu
```

This does not change the Nushell plugin registry.

## Explore built-in widget examples

Each widget has its own zero-argument example subcommand, so you can open a focused demonstration without writing a widget record:

```sh
nu --plugins target/debug/nu_plugin_tui -c 'tui example paragraph'
```

The available subcommands are `tui example layout`, `tui example paragraph`, `tui example button`, `tui example list`, `tui example gauge`, and `tui example spacer`. Press Escape or Ctrl-C to exit.

## Install the plugin

Build an optimized binary:

```sh
cargo build --release
```

At a Nushell prompt, register it:

```nu
plugin add (pwd | path join target release nu_plugin_tui)
```

Wait for registration to finish. At the next prompt, run the example:

```nu
source examples/counter.nu
```

Nushell must parse `plugin add` before it can recognize `tui run`. Do not put registration and the first invocation in the same script or block. If the command is still unavailable, start a new Nushell process so it reloads the plugin registry.

The TUI also requires Nushell's local-socket plugin transport. Standard-I/O transport cannot be used because the terminal interface needs stdin and stdout.

## Smallest application

```nu
tui run {
  view: {
    type: paragraph
    text: "Hello from Nushell"
    alignment: center
  }
}
```

Press Escape to close the application. With no initial state or handlers, the command returns `null`.

## Reactive application

Pass a closure as `view` to render from the current state:

```nu
tui run {
  state: 0
  view: { |count|
    {
      type: button
      id: increment
      label: $"Count: ($count)"
      on-click: { |event| $event.state + 1 }
    }
  }
}
```

Each click returns a new state from `on-click`; the view closure receives that value on the next redraw. `tui run` returns the final state when the application closes.

Continue with [Applications and events](applications-and-events.md) or browse the [Widget reference](widgets.md).
