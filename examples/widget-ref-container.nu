# Declarative port of Ratatui's examples/apps/widget-ref-container.
# Nushell records take the role of boxed WidgetRef values in the upstream example.
let widgets = [
  (tui paragraph "A paragraph stored in a widget list" --alignment center --border --title " Paragraph ")
  (tui gauge 0.72 --label "72%" --gauge-style {fg: cyan} --border --title " Gauge ")
  (tui sparkline [1 3 2 5 8 5 3 6 9 7 8 10] --style {fg: yellow} --border --title " Sparkline ")
  (tui bar-chart [{label: A value: 3} {label: B value: 8} {label: C value: 5}] --bar-width 4 --border --title " Bar chart ")
]

(tui
  (tui layout
    --direction vertical
    --constraints [{length: 2} {fill: 1} {fill: 1} {fill: 1} {fill: 2}]
    ([(tui paragraph "Widget container example — records compose heterogenous widgets" --alignment center --style {bold: true})] | append $widgets))
)
