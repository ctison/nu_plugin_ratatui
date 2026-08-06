# Port of Ratatui's examples/apps/hello-world.
# Run with `nu --plugins target/debug/nu_plugin_ratatui examples/hello-world.nu`.
(tui
  (tui layout
    --direction vertical
    --constraints [{fill: 1} {length: 3} {fill: 1}]
    --children [
      (tui spacer)
      (tui paragraph
        --text "Hello World!"
        --border true
        --border-type rounded
        --border-style {fg: cyan}
        --style {fg: white bold: true}
        --alignment center)
      (tui spacer)
    ])
)
