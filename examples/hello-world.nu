# Port of Ratatui's examples/apps/hello-world.
# Run with `nu --plugins target/debug/nu_plugin_ratatui examples/hello-world.nu`.
(tui
  (tui layout
    --direction vertical
    --constraints [{fill: 1} {length: 3} {fill: 1}]
    [
      (tui spacer)
      (tui paragraph
        "Hello World!"
        --border
        --border-type rounded
        --border-style {fg: cyan}
        --style {fg: white bold: true}
        --alignment center)
      (tui spacer)
    ])
)
