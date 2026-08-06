# Dependency-free adaptation of Ratatui's examples/apps/custom-widget.
# Click a cell to change its state; r resets and q quits.
(tui
  --state {selected: 4}
  --quit-on-esc false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "r" => {state: {selected: 4}}
      _ => null
    }
  }
  { |app|
    let cells = 0..8 | each { |index|
      let active = $index == $app.selected
      {
        type: button
        id: $"cell-($index)"
        label: (if $active { $"\nSelected ($index)" } else { $"\nCell ($index)" })
        alignment: center
        border: true
        border-type: rounded
        border-style: {fg: (if $active {"yellow"} else {"dark_gray"}) bold: $active}
        style: {fg: (if $active {"black"} else {"white"}) bg: (if $active {"yellow"} else {"black"})}
        on-click: { |event| {state: ($event.state | update selected $index)} }
      }
    }
    let rows = $cells | chunks 3 | each { |row| {type: layout direction: horizontal children: $row} }
    {
      type: layout
      direction: vertical
      constraints: [{length: 2} {fill: 1} {fill: 1} {fill: 1}]
      children: ([{type: paragraph text: "Custom Widget — click a cell • r reset • q quit" alignment: center style: {bold: true}}] | append $rows)
    }
  }
)
