# Port of Ratatui's examples/apps/todo-list.
# Use j/k to select, h to unselect, l/Enter to toggle, g/G for first/last.
let initial_items = [
  {done: false todo: "Rewrite everything with Rust!" info: "I can't hold my inner voice. It tells me to rewrite the complete universe with Rust."}
  {done: true todo: "Rewrite all of your TUI apps with Ratatui" info: "Yes, you heard that right. Go and replace your TUI with Ratatui."}
  {done: false todo: "Pet your cat" info: "Minnak loves to be petted. Don't forget the treats!"}
  {done: false todo: "Walk with your dog" info: "Max is bored; go walk with him!"}
  {done: true todo: "Pay the bills" info: "Pay the train subscription."}
  {done: true todo: "Refactor list example" info: "If you see this, the refactor is complete."}
]

(tui
  --state {items: $initial_items selected: 0}
  --quit-on-esc=false
  --on-key { |event|
    let last_index = ($event.state.items | length) - 1
    match $event.code {
      "q" | "escape" => {state: $event.state quit: true}
      "h" | "left" => {state: ($event.state | update selected (-1))}
      "j" | "down" => {state: ($event.state | update selected ([$last_index ($event.state.selected + 1)] | math min))}
      "k" | "up" => {state: ($event.state | update selected ([0 ($event.state.selected - 1)] | math max))}
      "g" | "home" => {state: ($event.state | update selected 0)}
      "G" | "end" => {state: ($event.state | update selected $last_index)}
      "l" | "right" | "enter" => {
        if $event.state.selected >= 0 {
          let current = $event.state.items | get $event.state.selected
          let items = $event.state.items | update $event.state.selected ($current | update done (not $current.done))
          {state: ($event.state | update items $items)}
        }
      }
      _ => null
    }
  }
  { |app|
    let labels = $app.items | enumerate | each { |entry|
      let cursor = if $entry.index == $app.selected { ">" } else { " " }
      let mark = if $entry.item.done { "✓" } else { "☐" }
      $"($cursor) ($mark) ($entry.item.todo)"
    }
    let info = if $app.selected >= 0 {
      let item = $app.items | get $app.selected
      let status = if $item.done { "✓ DONE" } else { "☐ TODO" }
      $"($status): ($item.info)"
    } else {
      "Nothing selected..."
    }
    (tui layout
      --direction vertical
      --constraints [{length: 2} {fill: 1} {fill: 1} {length: 1}]
      [
        (tui paragraph "Ratatui Todo List Example" --alignment center --style {fg: "#e2e8f0" bg: "#1e40af" bold: true})
        (tui list $labels --title " TODO List " --border --border-style {fg: "#1e40af"} --style {fg: "#e2e8f0" bg: "#020617"})
        (tui paragraph $info --title " TODO Info " --border --border-style {fg: "#1e40af"} --style {fg: "#e2e8f0" bg: "#020617"})
        (tui paragraph "↓↑ move • ← unselect • → toggle • g/G top/bottom • q quit" --alignment center)
      ])
  }
)
