# Port of Ratatui's examples/apps/modifiers using the modifiers supported by the plugin.
let samples = [
  {name: default style: {}}
  {name: bold style: {bold: true}}
  {name: italic style: {italic: true}}
  {name: underlined style: {underlined: true}}
  {name: reversed style: {reversed: true}}
  {name: dim style: {dim: true}}
]
let colors = [black dark_gray gray white red]
let rows = $colors | each { |background|
  let cells = $samples | each { |sample|
    {
      type: paragraph
      text: $sample.name
      alignment: center
      style: ($sample.style | merge {fg: white bg: $background})
    }
  }
  {type: layout direction: horizontal children: $cells}
}

tui run {
  view: {
    type: layout
    direction: vertical
    constraints: [{length: 2} {fill: 1} {fill: 1} {fill: 1} {fill: 1} {fill: 1}]
    children: ([{
      type: paragraph
      text: "Not every terminal supports every modifier — press any key to exit"
      alignment: center
      style: {fg: red bold: true}
    }] | append $rows)
  }
  on-key: { |event| {state: $event.state quit: true} }
}
