# Port of Ratatui's examples/apps/weather without rand or other dependencies.
# The deterministic hourly temperatures keep the example reproducible.
let temperatures = [52 51 50 50 51 54 58 63 68 73 77 80 82 84 83 80 76 71 66 62 59 57 55 53]
let bars = $temperatures | enumerate | each { |sample|
  let color = if $sample.item >= 80 {
    "#ef4444"
  } else if $sample.item >= 70 {
    "#f97316"
  } else if $sample.item >= 60 {
    "#eab308"
  } else {
    "#84cc16"
  }
  {
    label: ($sample.index | into string | fill --alignment right --width 2)
    value: $sample.item
    style: {fg: $color}
  }
}

tui run {
  view: {
    type: layout
    direction: vertical
    constraints: [{length: 2} {fill: 1} {length: 1}]
    children: [
      {type: paragraph text: "Weather demo" alignment: center style: {bold: true}}
      {
        type: bar-chart
        bars: $bars
        max: 90
        bar-width: 3
        bar-gap: 1
        value-style: {reversed: true}
        label-style: {fg: gray}
        border: true
        title: " Hourly temperature (°F) "
      }
      {type: paragraph text: "00                                      12                                      23" alignment: center style: {fg: dark_gray}}
    ]
  }
}
