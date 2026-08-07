# Dependency-free adaptation of Ratatui's examples/apps/demo2 dashboard.
# Use h/l or arrow keys to move between application tabs.
let weather = [{label: Mon value: 72} {label: Tue value: 68} {label: Wed value: 75} {label: Thu value: 81} {label: Fri value: 77}]

(tui
  --state {tab: 0}
  --quit-on-esc=false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "h" | "left" => {state: ($event.state | update tab ([0 ($event.state.tab - 1)] | math max))}
      "l" | "right" => {state: ($event.state | update tab ([4 ($event.state.tab + 1)] | math min))}
      _ => null
    }
  }
  { |app|
    let panel = match $app.tab {
      0 => (tui paragraph "Ratatui Demo2\n\nA dashboard-style showcase built entirely from Nushell records.\n\nThe upstream example uses rich custom rendering; this port retains its five-tab application structure." --alignment center --border --title " About " --style {fg: "#e2e8f0" bg: "#0f172a"})
      1 => (tui list ["● release@ratatui.rs — Version 0.30 is out" "○ team@example.com — Design review" "○ ci@example.com — Build succeeded" "○ community@example.com — Weekly digest"] --title " Email " --border --border-type rounded --border-style {fg: cyan})
      2 => (tui paragraph "Bryndza Toast\n\n1. Toast the bread.\n2. Spread bryndza generously.\n3. Add paprika and chives.\n4. Serve while warm." --title " Recipe " --border --border-type rounded --style {fg: "#f6d6bb"})
      3 => (tui table --header [Hop Host Latency] [[1 gateway "1 ms"] [2 edge.paris "7 ms"] [3 transit.fr "14 ms"] [4 ratatui.rs "21 ms"]] --widths [{length: 5} {fill: 1} {length: 12}] --title " Traceroute " --border --border-type rounded)
      _ => (tui bar-chart $weather --max 90 --bar-width 7 --bar-gap 2 --title " Weather " --border --bar-style {fg: "#38bdf8"} --value-style {reversed: true})
    }
    (tui layout
      --direction vertical
      --constraints [{length: 3} {fill: 1} {length: 1}]
      [
        (tui tabs [About Email Recipe Traceroute Weather] --selected $app.tab --divider " " --highlight-style {fg: black bg: cyan bold: true} --border --title " Demo2 ")
        $panel
        (tui paragraph "◄ h / l ► change tab • q quit" --alignment center --style {fg: dark_gray})
      ])
  }
)
