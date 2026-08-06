# Dependency-free adaptation of Ratatui's examples/apps/mouse-drawing.
# Drag the left mouse button to draw points; c clears and q quits.
(tui
  --state {points: []}
  --quit-on-esc false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "c" => {state: {points: []}}
      _ => null
    }
  }
  --on-event { |event|
    if $event.type == mouse and $event.kind in [down drag] {
      let point = {x: $event.column y: (100 - $event.row) color: cyan}
      {state: ($event.state | update points ($event.state.points | append $point))}
    }
  }
  { |app|
    {
      type: layout
      direction: vertical
      constraints: [{length: 2} {fill: 1}]
      children: [
        {type: paragraph text: $"Mouse Drawing — drag to draw • c clear • q quit • ($app.points | length) points" alignment: center style: {bold: true}}
        {type: canvas points: $app.points x-bounds: [0 200] y-bounds: [0 100] marker: braille title: " Draw here " border: true border-type: rounded}
      ]
    }
  }
)
