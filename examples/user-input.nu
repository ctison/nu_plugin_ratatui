# Port of Ratatui's examples/apps/user-input.
# Press e to edit, Escape to return to normal mode, Enter to save, and q to quit.
(tui
  --state {mode: normal input: [] cursor: 0 messages: []}
  --quit-on-esc false
  --on-key { |event|
    if $event.state.mode == normal {
      match $event.code {
        "e" => {state: ($event.state | update mode editing)}
        "q" => {state: $event.state quit: true}
        _ => null
      }
    } else {
      match $event.code {
        "escape" => {state: ($event.state | update mode normal)}
        "enter" => {
          let message = $event.state.input | str join
          {state: ($event.state | update messages ($event.state.messages | append $message) | update input [] | update cursor 0)}
        }
        "left" => {state: ($event.state | update cursor ([0 ($event.state.cursor - 1)] | math max))}
        "right" => {state: ($event.state | update cursor ([($event.state.input | length) ($event.state.cursor + 1)] | math min))}
        "backspace" => {
          if $event.state.cursor > 0 {
            let input = $event.state.input | drop nth ($event.state.cursor - 1)
            {state: ($event.state | update input $input | update cursor ($event.state.cursor - 1))}
          }
        }
        _ => {
          if ($event.code | str length) == 1 and not ("control" in $event.modifiers) {
            let input = $event.state.input | insert $event.state.cursor $event.code
            {state: ($event.state | update input $input | update cursor ($event.state.cursor + 1))}
          }
        }
      }
    }
  }
  { |app|
    let help = if $app.mode == normal {
      "Press q to exit, e to start editing."
    } else {
      "Press Esc to stop editing, Enter to record the message."
    }
    let before = $app.input | take $app.cursor | str join
    let after = $app.input | skip $app.cursor | str join
    let cursor = if $app.mode == editing { "▏" } else { "" }
    let messages = $app.messages | enumerate | each { |message| $"($message.index): ($message.item)" }
    {
      type: layout
      direction: vertical
      constraints: [{length: 1} {length: 3} {fill: 1}]
      children: [
        {type: paragraph text: $help style: {bold: ($app.mode == normal)}}
        {type: paragraph text: $"($before)($cursor)($after)" title: " Input " border: true style: {fg: (if $app.mode == editing {"yellow"} else {"white"})}}
        {type: list items: $messages title: " Messages " border: true}
      ]
    }
  }
)
