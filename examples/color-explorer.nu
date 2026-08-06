# Port of Ratatui's examples/apps/color-explorer.
# The grid covers all 256 indexed terminal colors without extra dependencies.
let rows = 0..15 | each { |row|
  let cells = 0..15 | each { |column|
    let index = $row * 16 + $column
    let foreground = if ($index in 16..27) or ($index in 52..63) or ($index in 88..99) or ($index in 124..135) or ($index in 160..171) or ($index in 196..207) or ($index in 232..243) {
      "white"
    } else {
      "black"
    }
    (tui paragraph
      --text ($index | into string | fill --alignment right --width 3)
      --alignment center
      --style {fg: $foreground bg: $index})
  }
  tui layout --direction horizontal --children $cells
}

(tui
  (tui layout
    --direction vertical
    --constraints ([{length: 2}] | append (0..15 | each { {fill: 1} }))
    --children ([(tui paragraph
      --text "Ratatui indexed color explorer — press any key to exit"
      --alignment center
      --style {bold: true})] | append $rows))
  --on-key { |event| {state: $event.state quit: true} }
)
