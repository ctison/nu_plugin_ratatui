# Port of Ratatui's examples/apps/popup using nested layouts for centering.
# Press p to toggle the popup and q to quit.
(tui
  --state {visible: false}
  --quit-on-esc false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "p" => {state: ($event.state | update visible (not $event.state.visible))}
      _ => null
    }
  }
  { |app|
    if $app.visible {
      (tui layout
        --direction vertical
        --constraints [{fill: 1} {length: 7} {fill: 1}]
        --children [
          (tui spacer)
          (tui layout
            --direction horizontal
            --constraints [{fill: 1} {percentage: 50} {fill: 1}]
            --children [
              (tui spacer)
              (tui paragraph --text "This is a centered popup.\n\nPress p to close it." --title " Popup " --border true --border-type double --alignment center --style {fg: white bg: "#1e293b"} --border-style {fg: cyan})
              (tui spacer)
            ])
          (tui spacer)
        ])
    } else {
      tui paragraph --text "Popup Example\n\nPress p to open a centered popup, q to quit." --alignment center --border true --title " Background " --style {fg: gray}
    }
  }
)
