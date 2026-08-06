# Port of Ratatui's examples/apps/colors-rgb.
# This animated true-color palette uses a fixed set of RGB colors instead of palette.rs.
let colors = [
  "#ff0040" "#ff4000" "#ff8000" "#ffc000" "#ffff00" "#bfff00" "#80ff00" "#40ff00"
  "#00ff40" "#00ff80" "#00ffc0" "#00ffff" "#00bfff" "#0080ff" "#0040ff" "#0000ff"
  "#4000ff" "#8000ff" "#c000ff" "#ff00ff" "#ff00bf" "#ff0080" "#ff0040" "#ff0000"
]

(tui
  --state {offset: 0}
  --tick-rate-ms 80
  --on-key { |event| {state: $event.state quit: true} }
  --on-event { |event|
    if $event.type == tick {
      {state: ($event.state | update offset (($event.state.offset + 1) mod ($colors | length)))}
    }
  }
  { |app|
    let palette = 0..23 | each { |index| $colors | get (($index + $app.offset) mod 24) }
    let rows = 0..11 | each { |row|
      let value = 255 - ($row * 18)
      let cells = $palette | each { |color|
        tui fill --symbol "▀" --style {fg: $color bg: $color bold: ($value > 150)}
      }
      tui layout --direction horizontal --children $cells
    }
    (tui layout
      --direction vertical
      --constraints ([{length: 1}] | append (0..11 | each { {fill: 1} }))
      --children ([(tui paragraph
        --text "colors_rgb example — press any key to quit"
        --alignment center)] | append $rows))
  }
)
