# Port of Ratatui's examples/apps/gauge using only plugin widgets.
# Press Enter or Space to start the gauges, r to reset, and q to quit.
(tui
  --state {running: false progress: 0}
  --tick-rate-ms 50
  --quit-on-esc false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "enter" | " " => {state: ($event.state | update running true)}
      "r" => {state: {running: false progress: 0}}
      _ => null
    }
  }
  --on-event { |event|
    if $event.type == tick and $event.state.running and $event.state.progress < 100 {
      {state: ($event.state | update progress ($event.state.progress + 1))}
    }
  }
  { |app|
    let ratio = $app.progress / 100
    {
      type: layout
      direction: vertical
      constraints: [{length: 2} {fill: 1} {fill: 1} {fill: 1} {fill: 1} {length: 1}]
      children: [
        {
          type: paragraph
          text: "Ratatui Gauge Example"
          alignment: center
          style: {fg: "#cbd5e1" bold: true}
        }
        {
          type: gauge
          ratio: $ratio
          label: $"($app.progress)%"
          title: " Gauge with percentage "
          gauge-style: {fg: "#991b1b"}
          border: true
        }
        {
          type: gauge
          ratio: $ratio
          label: $"($app.progress).0/100"
          title: " Gauge with ratio and custom label "
          gauge-style: {fg: "#166534" bold: true italic: true}
          border: true
        }
        {
          type: line-gauge
          ratio: $ratio
          label: $"($app.progress)%"
          title: " Line gauge "
          filled-style: {fg: "#1e40af"}
          border: true
        }
        {
          type: line-gauge
          ratio: $ratio
          label: $"($app.progress)%"
          title: " Unicode line gauge "
          filled-symbol: "━"
          unfilled-symbol: "─"
          filled-style: {fg: "#9a3412" bold: true}
          unfilled-style: {fg: dark_gray}
          border: true
        }
        {
          type: paragraph
          text: "Enter/Space start • r reset • q quit"
          alignment: center
          style: {fg: "#cbd5e1"}
        }
      ]
    }
  }
)
