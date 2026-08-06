# Port of Ratatui's examples/apps/input-form without serde or JSON dependencies.
# Tab moves focus, Enter submits, and Escape cancels.
tui run {
  state: {focus: 0 first-name: [] last-name: [] age: 0 result: editing}
  quit-on-esc: false
  on-key: { |event|
    match $event.code {
      "escape" => {state: ($event.state | update result canceled) quit: true}
      "enter" => {state: ($event.state | update result submitted) quit: true}
      "tab" | "back-tab" => {state: ($event.state | update focus (($event.state.focus + 1) mod 3))}
      "backspace" => {
        match $event.state.focus {
          0 => {state: ($event.state | update first-name ($event.state.first-name | drop))}
          1 => {state: ($event.state | update last-name ($event.state.last-name | drop))}
          2 => {state: ($event.state | update age (($event.state.age / 10) | math floor))}
        }
      }
      "up" | "k" => {
        if $event.state.focus == 2 {
          {state: ($event.state | update age ([130 ($event.state.age + 1)] | math min))}
        }
      }
      "down" | "j" => {
        if $event.state.focus == 2 {
          {state: ($event.state | update age ([0 ($event.state.age - 1)] | math max))}
        }
      }
      _ => {
        if ($event.code | str length) == 1 and not ("control" in $event.modifiers) {
          match $event.state.focus {
            0 => {state: ($event.state | update first-name ($event.state.first-name | append $event.code))}
            1 => {state: ($event.state | update last-name ($event.state.last-name | append $event.code))}
            2 => {
              if $event.code =~ '^[0-9]$' {
                let age = $event.state.age * 10 + ($event.code | into int)
                if $age <= 130 { {state: ($event.state | update age $age)} }
              }
            }
          }
        }
      }
    }
  }
  view: { |form|
    let values = [($form.first-name | str join) ($form.last-name | str join) ($form.age | into string)]
    let labels = ["First Name" "Last Name" "Age"]
    let fields = 0..2 | each { |index|
      {
        type: paragraph
        text: ($values | get $index)
        title: $" ($labels | get $index) "
        border: true
        border-type: rounded
        border-style: {fg: (if $form.focus == $index {"yellow"} else {"dark_gray"}) bold: ($form.focus == $index)}
      }
    }
    {
      type: layout
      direction: vertical
      constraints: [{length: 2} {length: 3} {length: 3} {length: 3} {fill: 1}]
      children: ([{type: paragraph text: "Input Form — Tab focus • Enter submit • Esc cancel" alignment: center style: {bold: true}}] | append $fields | append [{type: spacer}])
    }
  }
}
