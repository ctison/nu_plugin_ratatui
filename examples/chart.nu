# Port of Ratatui's examples/apps/chart.
# The two signals move on every tick; use h/l or the arrow keys to pan.
(tui
  --state {offset: 0 phase: 0}
  --tick-rate-ms 80
  --quit-on-esc=false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "h" | "left" => {state: ($event.state | update offset ($event.state.offset - 1))}
      "l" | "right" => {state: ($event.state | update offset ($event.state.offset + 1))}
      _ => null
    }
  }
  --on-event { |event|
    if $event.type == tick {
      {state: ($event.state | update phase ($event.state.phase + 1))}
    }
  }
  { |app|
    let wave = 0..40 | each { |x|
      let y = (($x + $app.phase) * 0.22 | math sin) * 4 + 5
      [($x + $app.offset) $y]
    }
    let wave2 = 0..40 | each { |x|
      let y = (($x + $app.phase) * 0.14 | math cos) * 3 + 5
      [($x + $app.offset) $y]
    }
    (tui layout
      --direction vertical
      --constraints [{length: 2} {fill: 1}]
      [
        (tui paragraph
          "Chart Example — h/l pan • q quit"
          --alignment center
          --style {fg: cyan bold: true})
        (tui chart
          --title " Animated signals "
          --border
          --border-type rounded
          [
            {name: sine data: $wave graph-type: line marker: braille style: {fg: cyan}}
            {name: cosine data: $wave2 graph-type: scatter marker: dot style: {fg: yellow}}
          ]
          --x-axis {
            title: "x"
            bounds: [$app.offset ($app.offset + 40)]
            labels: [$"($app.offset)" $"($app.offset + 20)" $"($app.offset + 40)"]
            style: {fg: gray}
          }
          --y-axis {title: "y" bounds: [0 10] labels: ["0" "5" "10"] style: {fg: gray}})
      ])
  }
)
