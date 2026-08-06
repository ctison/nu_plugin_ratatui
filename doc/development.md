# Development

## Repository layout

| Path | Purpose |
| --- | --- |
| `src/main.rs` | Starts the Nushell plugin protocol loop. |
| `src/plugin.rs` | Registers the plugin and its commands. |
| `src/run.rs` | Implements the root `tui` command, terminal lifecycle, events, and handlers. |
| `src/config.rs` | Parses application arguments and widget records. |
| `src/ui.rs` | Renders widget trees and records button hit targets. |
| `examples/` | Runnable Nushell applications. |
| `doc/` | User and contributor documentation. |

## Local checks

Format and test the Rust code from the repository root:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Build the plugin and run the example through a one-process plugin declaration:

```sh
cargo build
nu --plugins target/debug/nu_plugin_ratatui examples/counter.nu
```

The interactive example requires a real terminal. Automated Rust tests use Ratatui's test backend and do not enter raw terminal mode.

## Adding behavior

Keep the parser, renderer, and documentation aligned when adding a widget or field:

1. Parse and validate the Nushell value in `src/config.rs`.
2. Render the resulting node in `src/ui.rs`.
3. Add parser or rendering tests.
4. Update the appropriate file in `doc/` and any affected example.

New commands must also be registered by the plugin implementation in `src/plugin.rs`.
