# Port of Ratatui's examples/apps/calendar-explorer.
# Use n/p to change year and s to cycle the calendar presentation.
(tui
  --state {year: 2026 style-index: 0}
  --quit-on-esc=false
  --on-key { |event|
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "n" | "right" => {state: ($event.state | update year ($event.state.year + 1))}
      "p" | "left" => {state: ($event.state | update year ($event.state.year - 1))}
      "s" => {state: ($event.state | update style-index (($event.state.style-index + 1) mod 4))}
      _ => null
    }
  }
  { |app|
    let show_weekdays = $app.style-index in [1 3]
    let show_surrounding = $app.style-index in [2 3]
    let style_name = [Default "Weekday headers" "Surrounding dates" "Headers + surrounding"] | get $app.style-index
    let months = 1..12 | each { |month|
      mut calendar = (tui calendar
        $app.year
        $month
        --month-style {fg: cyan bold: true}
        --style {fg: white bg: "#323232" bold: true})
      if $show_weekdays {
        $calendar = $calendar | insert weekday-style {fg: green bold: true}
      }
      if $show_surrounding {
        $calendar = $calendar | insert surrounding-style {fg: dark_gray dim: true}
      }
      $calendar
    }
    let rows = $months | chunks 4 | each { |row|
      tui layout --direction horizontal $row
    }
    (tui layout
      --direction vertical
      --constraints [{length: 3} {fill: 1} {fill: 1} {fill: 1}]
      ([(tui paragraph
        $"Calendar Example — ($app.year) — ($style_name)\nn/p year • s style • q quit"
        --alignment center
        --style {bold: true})] | append $rows))
  }
)
