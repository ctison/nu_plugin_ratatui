# Port of Ratatui's examples/apps/release-header.
let main_dishes = ["> ratatui" "> ratatui-core" "> ratatui-widgets" "> ratatui-macros"]
let backends = ["> ratatui-crossterm" "> ratatui-termion" "> ratatui-termina" "> ratatui-termwiz"]
let background = tui fill --symbol " " --style {fg: "#f6d6bb" bg: "#141432"}

(tui
  (tui layout
    --direction vertical
    --constraints [{fill: 1} {length: 13} {fill: 1}]
    --children [
      $background
      (tui layout
        --direction horizontal
        --constraints [{fill: 1} {length: 29} {length: 2} {length: 25} {fill: 1}]
        --children [
          $background
          (tui layout
            --direction vertical
            --constraints [{fill: 1} {length: 6} {length: 2} {length: 1}]
            --children [
              $background
              (tui logo --size small)
              (tui logo --size small)
              (tui paragraph --text 'v0.30.1 "Bryndza"' --style {fg: "#f6d6bb" bg: "#141432" dim: true})
            ])
          $background
          (tui layout
            --direction vertical
            --constraints [{length: 6} {length: 6}]
            --children [
              (tui list --items $main_dishes --title " Main Courses " --border true --border-type rounded --border-style {fg: "#ffffa0"} --style {fg: "#f6d6bb" bg: "#141432"})
              (tui list --items $backends --title " Pairings " --border true --border-type rounded --border-style {fg: "#ffffa0"} --style {fg: "#f6d6bb" bg: "#141432"})
            ])
          $background
        ])
      $background
    ])
)
