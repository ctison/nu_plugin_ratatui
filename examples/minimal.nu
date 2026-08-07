# Port of Ratatui's examples/apps/minimal.
# Run with `nu --plugins target/debug/nu_plugin_ratatui examples/minimal.nu`.
(tui
  (tui paragraph
    "Hello Ratatui! (press Esc to quit)"
    --style {fg: cyan bold: true}
    --alignment center)
)
