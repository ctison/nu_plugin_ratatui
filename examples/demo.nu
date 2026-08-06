# Dependency-free port of Ratatui's examples/apps/demo.
# Use h/l or arrow keys to change tabs; q quits.
(tui
  --state {tab: 0 progress: 0}
  --tick-rate-ms 100
  --quit-on-esc false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "h" | "left" => {state: ($event.state | update tab ([0 ($event.state.tab - 1)] | math max))}
      "l" | "right" => {state: ($event.state | update tab ([3 ($event.state.tab + 1)] | math min))}
      _ => null
    }
  }
  --on-event { |event|
    if $event.type == tick {
      {state: ($event.state | update progress (($event.state.progress + 1) mod 101))}
    }
  }
  { |app|
    let content = match $app.tab {
      0 => (tui layout --direction horizontal --children [
        (tui list --items ["List item 1" "List item 2" "List item 3" "List item 4"] --title " List " --border true --border-style {fg: cyan})
        (tui paragraph --text "This demo combines the declarative widgets exposed by nu_plugin_ratatui.\n\nResize the terminal and move through the tabs." --title " About " --border true --wrap true)
      ])
      1 => (tui layout --direction vertical --children [
        (tui gauge --ratio ($app.progress / 100) --label $"($app.progress)%" --title " Gauge " --border true --gauge-style {fg: magenta})
        (tui line-gauge --ratio ($app.progress / 100) --title " Line Gauge " --border true --filled-symbol "━" --filled-style {fg: cyan})
        (tui sparkline --data [1 3 2 5 8 4 6 9 7 10 8 12 9 14] --title " Sparkline " --border true --style {fg: yellow})
      ])
      2 => (tui layout --direction horizontal --children [
        (tui bar-chart --bars [{label: B1 value: 9} {label: B2 value: 12} {label: B3 value: 5} {label: B4 value: 8}] --bar-width 5 --title " Bar chart " --border true --bar-style {fg: cyan})
        (tui chart --datasets [{name: data data: [[0 1] [1 3] [2 2] [3 5] [4 4] [5 8]] graph-type: line marker: braille style: {fg: yellow}}] --x-axis {bounds: [0 5] labels: ["0" "5"]} --y-axis {bounds: [0 10] labels: ["0" "10"]} --title " Chart " --border true)
      ])
      _ => (tui layout --direction horizontal --children [
        (tui table --header [Item Cost] --rows [[Coffee "$3"] [Tea "$2"] [Cake "$5"]] --widths [{fill: 2} {fill: 1}] --title " Table " --border true)
        (tui calendar --year 2026 --month 8 --month-style {fg: cyan bold: true} --weekday-style {fg: yellow} --surrounding-style {fg: dark_gray dim: true} --title " Calendar " --border true)
      ])
    }
    (tui layout
      --direction vertical
      --constraints [{length: 3} {fill: 1} {length: 1}]
      --children [
        (tui tabs --titles [Overview Metrics Charts Data] --selected $app.tab --highlight-style {fg: yellow bold: true} --border true --title " Ratatui Demo ")
        $content
        (tui paragraph --text "h/l change tab • q quit" --alignment center --style {fg: dark_gray})
      ])
  }
)
