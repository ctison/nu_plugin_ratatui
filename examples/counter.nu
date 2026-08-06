# Run with `nu --plugins target/debug/nu_plugin_ratatui examples/counter.nu` after `cargo build`.
tui run {
  state: 0
  tick-rate-ms: 100
  view: { |count|
    {
      type: layout
      direction: vertical
      constraints: [{length: 3} {fill: 1}]
      children: [
        {
          type: button
          id: increment
          label: $" Clicked ($count) times "
          border: true
          border-type: rounded
          alignment: center
          style: {fg: cyan bold: true}
          on-click: { |event|
            {state: ($event.state + 1)}
          }
        }
        {
          type: paragraph
          text: "Click the button, then press Escape to return the final count."
          alignment: center
          style: {fg: dark_gray}
        }
      ]
    }
  }
}
