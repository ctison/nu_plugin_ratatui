# Select Git subcommands on the left and inspect their ANSI-colored output on the right.
let commands = [
  {
    name: "Status"
    args: [--no-pager -c color.ui=always status --short --branch]
  }
  {
    name: "Recent commits"
    args: [--no-pager -c color.ui=always log --graph --decorate --oneline -20]
  }
  {
    name: "Working tree diff"
    args: [--no-pager -c color.ui=always diff --color=always]
  }
  {
    name: "Staged diff"
    args: [--no-pager -c color.ui=always diff --cached --color=always]
  }
]

# Runs one selected command and turns its complete result into preview state.
let run_preview = { |commands, index|
  let command = $commands | get $index
  let result = do { ^git ...$command.args } | complete
  mut output = $result.stdout
  if not ($result.stderr | is-empty) {
    $output = $"($output)(if ($output | is-empty) { '' } else { '\n' })(ansi red_bold)stderr:(ansi reset)\n($result.stderr)"
  }
  if ($output | is-empty) {
    $output = "(command produced no output)"
  }
  let status = if $result.exit_code == 0 {
    $"(ansi green_bold)exit 0(ansi reset)"
  } else {
    $"(ansi red_bold)exit ($result.exit_code)(ansi reset)"
  }
  {
    output: $"($status)\n($output)"
    exit_code: $result.exit_code
  }
}

# Changes selection, refreshes the cached preview, and resets both scroll axes.
let select_command = { |state, commands, index|
  let preview = do $run_preview $commands $index
  $state
  | update selected $index
  | update output $preview.output
  | update exit_code $preview.exit_code
  | update scroll_x 0
  | update scroll_y 0
}

let initial_preview = do $run_preview $commands 0

(tui
  --state {
    selected: 0
    focus: selector
    output: $initial_preview.output
    exit_code: $initial_preview.exit_code
    scroll_x: 0
    scroll_y: 0
  }
  --quit-on-esc=false
  --on-key { |event|
    let state = $event.state
    let last_index = ($commands | length) - 1
    match $event.code {
      "q" | "escape" => {state: $state quit: true}
      "tab" | "back-tab" => {
        $state | update focus (if $state.focus == selector { "preview" } else { "selector" })
      }
      _ => {
        if $state.focus == selector {
          let next = match $event.code {
            "down" | "j" => ([$last_index ($state.selected + 1)] | math min)
            "up" | "k" => ([0 ($state.selected - 1)] | math max)
            "home" => 0
            "end" => $last_index
            _ => $state.selected
          }
          if $next == $state.selected {
            null
          } else {
            do $select_command $state $commands $next
          }
        } else {
          match $event.code {
            "left" | "h" => ($state | update scroll_x ([0 ($state.scroll_x - 1)] | math max))
            "right" | "l" => ($state | update scroll_x ([65535 ($state.scroll_x + 1)] | math min))
            "up" | "k" => ($state | update scroll_y ([0 ($state.scroll_y - 1)] | math max))
            "down" | "j" => ($state | update scroll_y ([65535 ($state.scroll_y + 1)] | math min))
            "page-up" => ($state | update scroll_y ([0 ($state.scroll_y - 10)] | math max))
            "page-down" => ($state | update scroll_y ([65535 ($state.scroll_y + 10)] | math min))
            "home" => ($state | update scroll_x 0 | update scroll_y 0)
            _ => null
          }
        }
      }
    }
  }
  { |app|
    let labels = $commands | enumerate | each { |entry|
      let cursor = if $entry.index == $app.selected { "›" } else { " " }
      $"($cursor) ($entry.item.name)"
    }
    let selector_color = if $app.focus == selector { "cyan" } else { "dark_gray" }
    let preview_color = if $app.focus == preview { "cyan" } else { "dark_gray" }
    let selected_name = $commands | get $app.selected | get name

    (tui layout
      --direction vertical
      --constraints [{fill: 1} {length: 1}]
      [
        (tui layout
          --direction horizontal
          --constraints [{percentage: 30} {percentage: 70}]
          [
            (tui list $labels --title " Commands " --border --border-type rounded --border-style {fg: $selector_color})
            (tui paragraph $app.output
              --ansi
              --wrap=false
              --scroll-x $app.scroll_x
              --scroll-y $app.scroll_y
              --title $" ($selected_name) "
              --border
              --border-type rounded
              --border-style {fg: $preview_color})
          ])
        (tui paragraph "Tab changes focus • arrows or hjkl navigate • PgUp/PgDn scroll • Home resets preview • q quits" --alignment center)
      ])
  }
)
