# Port of Ratatui's examples/apps/constraints.
let examples = [
  {label: "Length(20) + Fill(1)" constraints: [{length: 20} {fill: 1}] colors: ["#334155" "#020617"]}
  {label: "Percentage(25) + Fill(1)" constraints: [{percentage: 25} {fill: 1}] colors: ["#1e293b" "#020617"]}
  {label: "Min(30) + Fill(1)" constraints: [{min: 30} {fill: 1}] colors: ["#1e3a8a" "#020617"]}
  {label: "Max(30) + Fill(1)" constraints: [{max: 30} {fill: 1}] colors: ["#1e40af" "#020617"]}
  {label: "Fill(1) + Fill(2) + Fill(3)" constraints: [{fill: 1} {fill: 2} {fill: 3}] colors: ["#334155" "#1e293b" "#0f172a"]}
]
let rows = $examples | each { |example|
  let cells = $example.colors | enumerate | each { |color|
    {type: paragraph text: $example.label alignment: center border: true style: {fg: white bg: $color.item}}
  }
  {type: layout direction: horizontal constraints: $example.constraints children: $cells}
}

tui run {
  view: {
    type: layout
    direction: vertical
    constraints: [{length: 2} {fill: 1} {fill: 1} {fill: 1} {fill: 1} {fill: 1}]
    children: ([{type: paragraph text: "Constraint examples — resize the terminal to see them react" alignment: center style: {bold: true}}] | append $rows)
  }
}
