# A compact gallery for the widgets supplied by the tui-widgets crate.
let title = tui big-text "NU" --pixel-size quadrant --alignment center --style {fg: cyan}
let glyphs = tui layout [
  (tui box-text "R")
  (tui card ace hearts)
  (tui equalizer [0.15 0.4 0.8 1.0 0.65 0.25])
] --direction horizontal --constraints [{length: 12} {length: 12} {fill: 1}]
let graph = tui bar-graph [0.1 0.35 0.8 0.55 1.0 0.65] --bar-style braille
let prompts = tui layout [
  (tui text-prompt "Name" --value "Ada" --focused --border)
  (tui select-prompt "Language" [Nushell Rust] --selected 0 --focused --border)
] --direction horizontal
let code = tui qr-code "https://ratatui.rs" --no-quiet-zone
let popup = tui popup "Widgets from crates.io/crates/tui-widgets" --title "Popup"
let scrollbar = tui fractional-scrollbar 100 25 --position 35 --orientation horizontal --arrows both

let content = (tui layout [$title $glyphs $graph $prompts $code $popup $scrollbar]
  --constraints [
    {length: 6}
    {length: 12}
    {length: 8}
    {length: 5}
    {length: 18}
    {length: 5}
    {length: 1}
  ])

tui (tui scroll-view 70 55 $content --vertical-scrollbar always)
