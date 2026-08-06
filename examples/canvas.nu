# Port of Ratatui's examples/apps/canvas using the plugin's point-based canvas.
# Enter changes markers; h/j/k/l moves the yellow cursor; q quits.
let circle = 0..72 | each { |step|
  let angle = $step * 0.0872665
  {x: (($angle | math cos) * 25) y: (($angle | math sin) * 25) color: cyan}
}
let wave = -100..100 | each { |x|
  {x: $x y: (($x * 0.08 | math sin) * 20) color: magenta}
}
let axes = (-100..100 | each { |x| {x: $x y: 0 color: dark_gray} }) | append (-50..50 | each { |y| {x: 0 y: $y color: dark_gray} })

(tui
  --state {x: 0 y: 0 marker-index: 0}
  --quit-on-esc false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "h" | "left" => {state: ($event.state | update x ($event.state.x - 2))}
      "l" | "right" => {state: ($event.state | update x ($event.state.x + 2))}
      "j" | "down" => {state: ($event.state | update y ($event.state.y - 2))}
      "k" | "up" => {state: ($event.state | update y ($event.state.y + 2))}
      "enter" => {state: ($event.state | update marker-index (($event.state.marker-index + 1) mod 8))}
      _ => null
    }
  }
  { |app|
    let marker = [dot braille block half-block quadrant sextant octant bar] | get $app.marker-index
    let points = $axes | append $circle | append $wave | append [{x: $app.x y: $app.y color: yellow}]
    {
      type: layout
      direction: vertical
      constraints: [{length: 2} {fill: 1}]
      children: [
        {type: paragraph text: $"Canvas Example — marker: ($marker) • Enter marker • hjkl move • q quit" alignment: center style: {bold: true}}
        {
          type: canvas
          points: $points
          x-bounds: [-100 100]
          y-bounds: [-50 50]
          marker: $marker
          title: " Shapes and points "
          border: true
          border-type: rounded
        }
      ]
    }
  }
)
