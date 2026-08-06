# Declarative adaptation of Ratatui's examples/apps/advanced-widget-impl.
# Records and closures illustrate the same owned, borrowed, mutable, and stateful render roles.
tui run {
  state: {count: 0}
  tick-rate-ms: 250
  on-event: { |event|
    if $event.type == tick {
      {state: ($event.state | update count ($event.state.count + 1))}
    }
  }
  view: { |app|
    {
      type: layout
      direction: vertical
      constraints: [{length: 2} {fill: 1} {fill: 1}]
      children: [
        {type: paragraph text: "Advanced widget implementation patterns" alignment: center style: {bold: true}}
        {
          type: layout
          direction: horizontal
          children: [
            {type: paragraph text: "Owned record\n\nA concrete widget value." title: " Widget " border: true border-style: {fg: cyan} alignment: center}
            {type: paragraph text: "Shared record\n\nThe same value can be nested." title: " &Widget " border: true border-style: {fg: green} alignment: center}
          ]
        }
        {
          type: layout
          direction: horizontal
          children: [
            {type: paragraph text: $"Reactive closure\n\nFrame ($app.count)" title: " &mut Widget " border: true border-style: {fg: yellow} alignment: center}
            {type: gauge ratio: (($app.count mod 20) / 20) label: "separate state" title: " StatefulWidget " border: true gauge-style: {fg: magenta}}
          ]
        }
      ]
    }
  }
}
