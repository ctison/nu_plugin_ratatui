# Seeded, subtree-scoped effects composed from ordinary Nushell records.
let final_state = tui --state 0 { |generation|
  let text_cells = tui effect filter text
  let visible_cells = tui effect filter non-empty
  let accent = if ($generation mod 2) == 0 { "cyan" } else { "magenta" }

  let reveal = tui effect coalesce 650 --interpolation sine-in-out --filter $text_cells --seed 42
  let color_wash = tui effect fade-from-fg $accent 900 --interpolation smooth-step --filter $visible_cells
  let sweep = tui effect sweep-in left-to-right 8 2 black 900 --interpolation quad-out --filter $text_cells --seed 42
  let animation = tui effect sequence [
    $reveal
    (tui effect parallel [$color_wash $sweep])
  ]

  tui effect $"hero-($generation)" $animation (
    tui paragraph "TachyonFX — press Space to retrigger" --alignment center --border --border-type rounded --style {fg: white bold: true}
  )
} --on-key { |event|
  match $event.code {
    " " => ($event.state + 1)
    "q" => {quit: true}
    _ => null
  }
}

$final_state | ignore
