# Port of Ratatui's examples/apps/constraint-explorer for supported constraints.
# h/l selects a constraint and j/k changes its value.
let kinds = [length percentage min max fill]

tui run {
  state: {selected: 0 amount: 20}
  quit-on-esc: false
  on-key: { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "h" | "left" => {state: ($event.state | update selected ([0 ($event.state.selected - 1)] | math max))}
      "l" | "right" => {state: ($event.state | update selected ([4 ($event.state.selected + 1)] | math min))}
      "j" | "down" => {state: ($event.state | update amount ([0 ($event.state.amount - 1)] | math max))}
      "k" | "up" => {state: ($event.state | update amount ([100 ($event.state.amount + 1)] | math min))}
      _ => null
    }
  }
  view: { |app|
    let kind = $kinds | get $app.selected
    let constraint = {} | insert $kind $app.amount
    let tabs = {
      type: tabs
      titles: $kinds
      selected: $app.selected
      highlight-style: {fg: yellow bold: true reversed: true}
      border: true
      title: " Constraints — h/l select • j/k adjust • q quit "
    }
    {
      type: layout
      direction: vertical
      constraints: [{length: 3} {length: 3} {fill: 1}]
      children: [
        $tabs
        {type: paragraph text: $"($kind): ($app.amount)" alignment: center style: {fg: cyan bold: true}}
        {
          type: layout
          direction: horizontal
          constraints: [$constraint {fill: 1}]
          children: [
            {type: paragraph text: $"($kind)\n($app.amount)" alignment: center border: true border-style: {fg: cyan} style: {bg: "#1e3a8a"}}
            {type: paragraph text: "Fill(1)" alignment: center border: true border-style: {fg: dark_gray} style: {bg: "#0f172a"}}
          ]
        }
      ]
    }
  }
}
