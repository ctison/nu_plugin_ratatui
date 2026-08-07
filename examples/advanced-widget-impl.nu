# Declarative adaptation of Ratatui's examples/apps/advanced-widget-impl.
# Records and closures illustrate the same owned, borrowed, mutable, and stateful render roles.
(tui
  --state {count: 0}
  --tick-rate-ms 250
  --on-event { |event|
    if $event.type == tick {
      {state: ($event.state | update count ($event.state.count + 1))}
    }
  }
  { |app|
    (tui layout
      --direction vertical
      --constraints [{length: 2} {fill: 1} {fill: 1}]
      [
        (tui paragraph "Advanced widget implementation patterns" --alignment center --style {bold: true})
        (tui layout
          --direction horizontal
          [
            (tui paragraph "Owned record\n\nA concrete widget value." --title " Widget " --border --border-style {fg: cyan} --alignment center)
            (tui paragraph "Shared record\n\nThe same value can be nested." --title " &Widget " --border --border-style {fg: green} --alignment center)
          ])
        (tui layout
          --direction horizontal
          [
            (tui paragraph $"Reactive closure\n\nFrame ($app.count)" --title " &mut Widget " --border --border-style {fg: yellow} --alignment center)
            (tui gauge (($app.count mod 20) / 20) --label "separate state" --title " StatefulWidget " --border --gauge-style {fg: magenta})
          ])
      ])
  }
)
