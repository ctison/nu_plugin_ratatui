# Port of Ratatui's examples/apps/table with deterministic data.
let rows = [
  ["Ada Lovelace" "12 St James's Square\nLondon" "ada@example.com"]
  ["Grace Hopper" "Arlington, Virginia" "grace@example.com"]
  ["Edsger Dijkstra" "Nuenen, Netherlands" "edsger@example.com"]
  ["Margaret Hamilton" "Cambridge, Massachusetts" "margaret@example.com"]
  ["Barbara Liskov" "Los Angeles, California" "barbara@example.com"]
  ["Donald Knuth" "Milwaukee, Wisconsin" "donald@example.com"]
]

tui run {
  view: {
    type: layout
    direction: vertical
    constraints: [{length: 2} {fill: 1} {length: 1}]
    children: [
      {type: paragraph text: "Ratatui Table Example" alignment: center style: {fg: cyan bold: true}}
      {
        type: table
        header: [Name Address Email]
        rows: $rows
        widths: [{percentage: 25} {percentage: 40} {percentage: 35}]
        column-spacing: 2
        title: " People "
        border: true
        border-type: rounded
        border-style: {fg: cyan}
        style: {fg: white}
      }
      {type: paragraph text: "Press Esc to quit" alignment: center style: {fg: dark_gray}}
    ]
  }
}
