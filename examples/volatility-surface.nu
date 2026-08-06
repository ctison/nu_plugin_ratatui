# Dependency-free point-cloud adaptation of Ratatui's examples/apps/volatility-surface.
# Arrow keys rotate, p changes palette, Space pauses, and q quits.
let palettes = [cyan magenta yellow green]

(tui
  --state {angle: 0 tilt: 0.45 palette: 0 paused: false}
  --tick-rate-ms 60
  --quit-on-esc false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "left" => {state: ($event.state | update angle ($event.state.angle - 0.12))}
      "right" => {state: ($event.state | update angle ($event.state.angle + 0.12))}
      "up" => {state: ($event.state | update tilt ([1.2 ($event.state.tilt + 0.08)] | math min))}
      "down" => {state: ($event.state | update tilt ([-0.2 ($event.state.tilt - 0.08)] | math max))}
      "p" => {state: ($event.state | update palette (($event.state.palette + 1) mod 4))}
      " " => {state: ($event.state | update paused (not $event.state.paused))}
      _ => null
    }
  }
  --on-event { |event|
    if $event.type == tick and not $event.state.paused {
      {state: ($event.state | update angle ($event.state.angle + 0.025))}
    }
  }
  { |app|
    let color = $palettes | get $app.palette
    let cosine = $app.angle | math cos
    let sine = $app.angle | math sin
    let points = -12..12 | each { |x|
      -12..12 | each { |y|
        let xf = $x / 3
        let yf = $y / 3
        let z = (($xf * 1.4 | math sin) + ($yf * 1.1 | math cos)) * 2.4
        let rotated_x = $xf * $cosine - $z * $sine
        let depth = $xf * $sine + $z * $cosine
        {x: ($rotated_x * 10) y: (($yf * 7) + ($depth * $app.tilt * 5)) color: $color}
      }
    } | flatten
    (tui layout
      --direction vertical
      --constraints [{length: 2} {fill: 1}]
      --children [
        (tui paragraph --text $"Volatility Surface — arrows rotate • p palette • Space pause • q quit($app.paused | if $in { ' [PAUSED]' } else { '' })" --alignment center --style {bold: true})
        (tui canvas --points $points --x-bounds [-60 60] --y-bounds [-45 45] --marker braille --title " Implied volatility point cloud " --border true --border-type rounded)
      ])
  }
)
