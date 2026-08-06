# Port of Ratatui's examples/apps/scrollbar.
# Use h/j/k/l or arrow keys to move the four scrollbar examples.
(tui
  --state {vertical: 0 horizontal: 0}
  --quit-on-esc false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "j" | "down" => {state: ($event.state | update vertical ([90 ($event.state.vertical + 1)] | math min))}
      "k" | "up" => {state: ($event.state | update vertical ([0 ($event.state.vertical - 1)] | math max))}
      "l" | "right" => {state: ($event.state | update horizontal ([90 ($event.state.horizontal + 1)] | math min))}
      "h" | "left" => {state: ($event.state | update horizontal ([0 ($event.state.horizontal - 1)] | math max))}
      _ => null
    }
  }
  { |app|
    (tui layout
      --direction vertical
      --constraints [{length: 2} {fill: 1} {length: 3} {length: 3}]
      --children [
        (tui paragraph --text "Use h j k l or ◄ ▲ ▼ ► to scroll • q quit" --alignment center --style {bold: true})
        (tui layout
          --direction horizontal
          --constraints [{length: 3} {fill: 1} {length: 3}]
          --children [
            (tui scrollbar --content-length 100 --position $app.vertical --viewport-length 10 --orientation vertical-left --thumb-style {fg: cyan} --track-style {fg: dark_gray})
            (tui paragraph --text $"Vertical scroll position: ($app.vertical)\n\nThis central panel stands in for the long text in the upstream example." --title " Vertical scrollbars " --border true --alignment center --style {fg: gray})
            (tui scrollbar --content-length 100 --position $app.vertical --viewport-length 10 --orientation vertical-right --thumb-style {fg: yellow} --track-style {fg: dark_gray})
          ])
        (tui scrollbar --content-length 100 --position $app.horizontal --viewport-length 10 --orientation horizontal-top --thumb-style {fg: cyan} --track-style {fg: dark_gray})
        (tui scrollbar --content-length 100 --position $app.horizontal --viewport-length 10 --orientation horizontal-bottom --thumb-style {fg: yellow} --track-style {fg: dark_gray})
      ])
  }
)
