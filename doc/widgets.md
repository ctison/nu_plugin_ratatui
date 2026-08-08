# Widget reference

Every widget is a Nushell record with a `type` field. A `layout` combines widgets into a tree; any other widget can also be the root view.

Every widget also has a `tui <type>` constructor. Required record fields become positional parameters, optional fields become named flags, and every constructor returns a validated record that can be nested or passed as `tui`'s positional view:

```nu
let message = tui paragraph "Ready" --alignment center
let meter = tui gauge 0.75 --label "75%" --gauge-style {fg: cyan}

let view = tui layout [$message $meter] --direction vertical
tui $view
```

Boolean fields are switches. Use a bare flag such as `--border` for `true`, or an equals value such as `--wrap=false` for `false`. Optional flags omitted from a constructor are also omitted from its record, allowing the runtime defaults documented below to apply.

Ratatui's 16 content widgets are exposed as `bar-chart`, `calendar`, `canvas`, `chart`, `clear`, `fill`, `gauge`, `line-gauge`, `list`, `logo`, `mascot`, `paragraph`, `scrollbar`, `sparkline`, `table`, and `tabs`. The plugin also provides `layout`, `button`, `spacer`, and `effect` for composition and interaction. The exact Ratatui struct-name aliases `monthly`, `ratatui-logo`, and `ratatui-mascot` are also accepted as record types.

## Layout

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `layout`. |
| `direction` | string | no | `vertical` | `vertical` or `horizontal`. |
| `children` | list of records | yes | — | Widgets allocated in list order. |
| `constraints` | list of records | no | one `{fill: 1}` per child | One constraint per child. |

Each constraint is a record containing exactly one field:

- `{length: 3}` requests an exact number of cells.
- `{percentage: 50}` requests a percentage of the available area.
- `{min: 10}` sets a minimum size.
- `{max: 20}` sets a maximum size.
- `{fill: 1}` distributes remaining space by weight.

Constraint values must be integers from 0 through 65535. If `constraints` is supplied, its length must match `children`.

```nu
{
  type: layout
  direction: horizontal
  constraints: [{percentage: 30} {fill: 1}]
  children: [
    {type: list items: ["One" "Two"] title: "Menu" border: true}
    {type: paragraph text: "Content"}
  ]
}
```

## Paragraph

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `paragraph`. |
| `text` | string | yes | — | Text to display. |
| `alignment` | string | no | `left` | `left`, `center`, or `right`. |
| `wrap` | bool | no | `true` | Wrap long text and trim leading whitespace on wrapped lines. |
| `ansi` | bool | no | `false` | Interpret ANSI SGR colors and text modifiers. |
| `scroll-x` | int | no | `0` | Horizontal scroll offset in cells. |
| `scroll-y` | int | no | `0` | Vertical scroll offset in lines. |

Paragraphs also support the [presentation fields](#presentation-fields).

ANSI paragraphs support named, indexed, and truecolor SGR colors plus text modifiers such as bold,
italic, underline, and strikethrough. Unknown or malformed escape sequences are ignored; paragraphs
are not full terminal emulators, so cursor-addressed applications are not supported. Set `wrap` to
`false` when using `scroll-x` so long lines remain horizontally scrollable.

```nu
tui paragraph $command_output --ansi --wrap=false --scroll-x 4 --scroll-y 10 --border
```

## Button

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `button`. |
| `id` | string | yes | — | Identifier reported as `event.widget`. |
| `label` | string | yes | — | Text to display. |
| `alignment` | string | no | `left` | `left`, `center`, or `right`. |
| `on-click` | closure | no | — | Handles a left-button release over the button. |

Buttons support the [presentation fields](#presentation-fields) and enable `border` by default.

```nu
{
  type: button
  id: save
  label: "Save"
  alignment: center
  on-click: { |event| {state: "saved"} }
}
```

## List

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `type` | string | yes | Must be `list`. |
| `items` | list of strings | yes | Lines to display. |

Lists are display-only and support the [presentation fields](#presentation-fields).

## Gauge

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `gauge`. |
| `ratio` | number | yes | — | Completion from `0.0` through `1.0`. |
| `label` | string | no | Ratatui default | Text drawn over the gauge. |
| `gauge-style` | style record | no | terminal default | Style of the filled gauge area. |

Gauges also support the [presentation fields](#presentation-fields).

```nu
{
  type: gauge
  ratio: 0.65
  label: "65%"
  border: true
  gauge-style: {fg: cyan bold: true}
}
```

## Spacer

```nu
{type: spacer}
```

A spacer reserves the area assigned by its parent layout without drawing anything.

## Effect wrapper

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `effect`. |
| `id` | string | yes | — | Unique state key within the current widget tree. |
| `effect` | effect record | yes | — | Parsed TachyonFX effect specification. |
| `child` | widget record | yes | — | Widget subtree rendered before the effect is applied. |

Use `tui effect <id> <effect> <child>` to construct this record. The wrapper always uses the child's current allocated rectangle, including after layout changes and terminal resizes. Nested wrappers apply from the innermost child outward.

Effect state starts when an ID first appears and survives equivalent reactive view redraws. Changing the effect specification under an existing ID restarts it; changing the ID creates a fresh effect. Entries disappear when their wrappers disappear, and completed effects remain at their final state without restarting. Duplicate IDs in one tree are rejected before drawing.

### Timed leaf effects

Every timed leaf requires a positive `duration-ms` integer no larger than `4294967295`, accepts `interpolation` (default `linear`), and accepts an optional `filter` record.

| Type and constructor | Required fields / positional parameters | Additional options |
| --- | --- | --- |
| `fade-from`, `tui effect fade-from` | `fg`, `bg`, `duration-ms` | `--interpolation`, `--filter` |
| `fade-to`, `tui effect fade-to` | `fg`, `bg`, `duration-ms` | `--interpolation`, `--filter` |
| `fade-from-fg`, `tui effect fade-from-fg` | `color`, `duration-ms` | `--interpolation`, `--filter` |
| `fade-to-fg`, `tui effect fade-to-fg` | `color`, `duration-ms` | `--interpolation`, `--filter` |
| `dissolve`, `tui effect dissolve` | `duration-ms` | `--interpolation`, `--filter`, `--seed` |
| `coalesce`, `tui effect coalesce` | `duration-ms` | `--interpolation`, `--filter`, `--seed` |

Fade colors use the same named, indexed, or `#RRGGBB` syntax as widget styles. `seed` is an optional integer from `0` through `4294967295` that makes randomized output reproducible.

Directional effects have the common schema `{type: string direction: string gradient-length: int randomness: int color: color duration-ms: int interpolation?: string filter?: record seed?: int}`. Their constructors are:

- `tui effect sweep-in <direction> <gradient-length> <randomness> <color> <duration-ms>`
- `tui effect sweep-out <direction> <gradient-length> <randomness> <color> <duration-ms>`
- `tui effect slide-in <direction> <gradient-length> <randomness> <color> <duration-ms>`
- `tui effect slide-out <direction> <gradient-length> <randomness> <color> <duration-ms>`

Directions are `left-to-right`, `right-to-left`, `up-to-down`, and `down-to-up`. Gradient length and randomness range from `0` through `65535`. All four constructors accept `--interpolation`, `--filter`, and `--seed`.

Supported kebab-case interpolation names are `linear`, `reverse`, `smooth-step`, `spring`; `back-in`, `back-out`, `back-in-out`; `bounce-in`, `bounce-out`, `bounce-in-out`; `circ-in`, `circ-out`, `circ-in-out`; `cubic-in`, `cubic-out`, `cubic-in-out`; `elastic-in`, `elastic-out`, `elastic-in-out`; `expo-in`, `expo-out`, `expo-in-out`; `quad-in`, `quad-out`, `quad-in-out`; `quart-in`, `quart-out`, `quart-in-out`; `quint-in`, `quint-out`, `quint-in-out`; and `sine-in`, `sine-out`, `sine-in-out`.

### Effect composition

| Record schema | Constructor | Meaning |
| --- | --- | --- |
| `{type: sequence effects: list<record>}` | `tui effect sequence <effects>` | Run a non-empty list in order. |
| `{type: parallel effects: list<record>}` | `tui effect parallel <effects>` | Run a non-empty list together. |
| `{type: repeat effect: record times?: int duration-ms?: int}` | `tui effect repeat <effect> [--times int] [--duration-ms int]` | Repeat by positive count, positive duration, or forever when neither option is present. The two options are mutually exclusive. |
| `{type: ping-pong effect: record}` | `tui effect ping-pong <effect>` | Run forward and then backward. |

Effects nest recursively and invalid nested record types are rejected.

### Effect filters

Filters are ordinary records accepted by `filter` / `--filter` on leaf effects:

| Record schema | Constructor | Selected cells |
| --- | --- | --- |
| `{type: text}` | `tui effect filter text` | Non-whitespace text. |
| `{type: non-empty}` | `tui effect filter non-empty` | Cells with content or styling. |
| `{type: fg-color color: color}` | `tui effect filter fg-color <color>` | Matching foreground color. |
| `{type: bg-color color: color}` | `tui effect filter bg-color <color>` | Matching background color. |
| `{type: not filter: record}` | `tui effect filter not <filter>` | Logical negation. |
| `{type: all-of filters: list<record>}` | `tui effect filter all-of <filters>` | Cells matching every filter in a non-empty list. |
| `{type: any-of filters: list<record>}` | `tui effect filter any-of <filters>` | Cells matching at least one filter in a non-empty list. |

Effects advance from actual elapsed frame time and redraw at roughly 60 FPS while running. Animation-only wakeups never invoke event handlers; application `tick` events retain the configured `--tick-rate-ms` interval.

The initial effect API does not expose raw TachyonFX DSL, spatial patterns, non-core effects, or handler-driven cancellation/completion events. See [`examples/effects.nu`](../examples/effects.nu) for seeded randomness, composition, filtering, subtree scoping, and a reactive ID/specification change that retriggers animation.

## Bar chart

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `bar-chart`. |
| `bars` | list of records | yes | — | Each record has `label`, non-negative integer `value`, and optional `style` and `value-style`. |
| `direction` | string | no | `vertical` | `vertical` or `horizontal`. |
| `max` | int | no | largest value | Value that fills the available bar length. |
| `bar-width` | int | no | `1` | Width of each bar. |
| `bar-gap` | int | no | `1` | Gap between bars. |
| `bar-style` | style | no | terminal default | Default bar style. |
| `value-style` | style | no | terminal default | Values drawn on bars. |
| `label-style` | style | no | terminal default | Bar labels. |

```nu
{type: bar-chart bars: [{label: A value: 3} {label: B value: 8}] bar-width: 3 border: true}
```

## Calendar

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `calendar`. |
| `year` | int | yes | — | Calendar year from -9999 through 9999. |
| `month` | int | yes | — | Month from 1 through 12. |
| `month-style` | style | no | hidden | Show and style the month header when present. |
| `weekday-style` | style | no | hidden | Show and style weekday names when present. |
| `surrounding-style` | style | no | hidden | Show and style days outside the selected month when present. |

Calendar supports presentation fields; `style` controls ordinary dates.

```nu
{type: calendar year: 2026 month: 8 month-style: {fg: cyan bold: true} weekday-style: {} border: true}
```

## Canvas

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `canvas`. |
| `points` | list of records | yes | — | Records with numeric `x`, numeric `y`, and optional `color`. |
| `x-bounds` | two numbers | no | `[-10 10]` | Increasing horizontal bounds. |
| `y-bounds` | two numbers | no | `[-10 10]` | Increasing vertical bounds. |
| `marker` | string | no | `braille` | `dot`, `block`, `bar`, `braille`, `half-block`, `quadrant`, `sextant`, or `octant`. |
| `background-color` | color | no | terminal default | Canvas background. |

Canvas also supports presentation fields.

```nu
{type: canvas points: [{x: -1 y: -1 color: cyan} {x: 0 y: 0 color: cyan} {x: 1 y: 1 color: cyan}]}
```

## Chart

`datasets` is required. Each dataset record has `data` (a list of `[x y]` number pairs), optional `name`, `style`, `marker`, and `graph-type` (`scatter`, `line`, `bar`, or `area`).

Optional `x-axis` and `y-axis` records accept `title`, increasing `bounds`, string `labels`, and `style`; bounds default to `[0 100]`. Chart also supports presentation fields.

```nu
{
  type: chart
  datasets: [{name: load data: [[0 2] [5 7] [10 4]] graph-type: line style: {fg: cyan}}]
  x-axis: {bounds: [0 10] labels: ["0" "5" "10"]}
  y-axis: {bounds: [0 10] labels: ["0" "5" "10"]}
  border: true
}
```

## Clear

```nu
{type: clear}
```

Clear resets every cell in its assigned area. It is mainly useful when a view is redrawn over earlier terminal content.

## Fill

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `fill`. |
| `symbol` | string | no | space | Repeated cell symbol. |
| `style` | style | no | terminal default | Fill style. |

## Line gauge

Line gauges require `ratio` from `0.0` through `1.0`. They accept optional `label`, `filled-style`, `unfilled-style`, `filled-symbol` (default `─`), and `unfilled-symbol` (default `─`), plus presentation fields.

```nu
{type: line-gauge ratio: 0.65 label: "65%" filled-symbol: "━" filled-style: {fg: cyan}}
```

## Logo

```nu
{type: logo size: small}
```

`size` is `tiny` by default and may be `small`.

## Mascot

```nu
{type: mascot blink: false}
```

The Ratatui mascot is 32 cells wide by 16 cells high. `blink: true` renders its alternate red eye.

## Scrollbar

| Field | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `type` | string | yes | — | Must be `scrollbar`. |
| `content-length` | int | yes | — | Total non-negative content length. |
| `position` | int | no | `0` | Current non-negative offset. |
| `viewport-length` | int | no | track length | Visible content length. |
| `orientation` | string | no | `vertical-right` | `vertical-right`, `vertical-left`, `horizontal-bottom`, or `horizontal-top`. |
| `thumb-style` | style | no | terminal default | Scroll thumb style. |
| `track-style` | style | no | terminal default | Track style. |

## Sparkline

`data` is a required list of non-negative integers or null values. Optional fields are `max`, `direction` (`left-to-right` or `right-to-left`), `absent-symbol`, `absent-style`, plus presentation fields.

```nu
{type: sparkline data: [1 4 2 null 8 3] style: {fg: cyan} border: true}
```

## Table

`rows` is a required nested list of strings. Every row must have the same cell count. `header` is an optional string list of the same width. `widths` accepts one layout constraint per column and defaults to equal fills. `column-spacing` defaults to `1`. Tables also support presentation fields.

```nu
{type: table header: [Name Value] rows: [[CPU "24%"] [RAM "1.2 GB"]] border: true}
```

## Tabs

`titles` is a required string list. `selected` is an optional zero-based index, `divider` defaults to `│`, and `highlight-style` styles the selected title. Tabs also support presentation fields.

```nu
{type: tabs titles: [Home Metrics Help] selected: 1 highlight-style: {fg: cyan bold: true}}
```

## tui-widgets suite

The plugin also exposes every concrete widget family enabled by [`tui-widgets`](https://crates.io/crates/tui-widgets) 0.7.10. Their constructors use the same `tui <type>` convention as the built-in Ratatui widgets.

| Type | Required fields | Optional fields and defaults |
| --- | --- | --- |
| `bar-graph` | `data`: number list | `min` and `max` inferred from data; `bar-style`: `braille`, `solid`, `quadrant`, or `octant` (`braille`) |
| `big-text` | `text`: string | `pixel-size`: `full`, `half-height`, `half-width`, `quadrant`, `third-height`, `sextant`, `quarter-height`, or `octant` (`full`); `alignment` (`left`); presentation fields |
| `box-text` | `character`: one-character string | none |
| `card` | `rank`, `suit`: strings | ranks accept names, numbers, or `a`/`j`/`q`/`k`; suits are `spades`, `hearts`, `diamonds`, or `clubs` |
| `equalizer` | `bands`: numbers from 0 through 1 | `brightness`: number from 0 through 1 (`1`) |
| `popup` | `text`: string | `width` and `height` measured from text; presentation fields, with borders enabled by default |
| `text-prompt` | `message`: string | `value` (empty), `status`: `pending`, `done`, or `aborted` (`pending`), `focused` (`false`), `render-style`: `default`, `password`, or `invisible` (`default`), `hide-status` (`false`), presentation fields |
| `select-prompt` | `label`: string, `options`: string list | `selected` (`0`), `status` (`pending`), `focused` (`false`), presentation fields |
| `qr-code` | `data`: string | `no-quiet-zone` (`false`), `scaling`: `exact`, `min`, or `max` (`exact`), `scale-width` and `scale-height` (`1`), `inverted` (`false`), `style` |
| `fractional-scrollbar` | `content-length`, `viewport-length`: non-negative integers | `position` (`0`), `orientation`: `vertical` or `horizontal` (`vertical`), `arrows`: `none`, `start`, `end`, or `both` (`none`), `glyphs`: `legacy`, `unicode`, `box-drawing`, or `minimal` (`legacy`), `track-style`, `thumb-style`, `arrow-style` |
| `scroll-view` | `width`, `height`: positive integers; `child`: widget record | `scroll-x`, `scroll-y` (`0`); `vertical-scrollbar`, `horizontal-scrollbar`: `automatic`, `always`, or `never` (`automatic`) |

```nu
let banner = tui big-text "NU" --pixel-size quadrant --style {fg: cyan}
let levels = tui equalizer [0.15 0.5 0.9 0.65 0.3]
let prompt = tui text-prompt "Name" --value "Ada" --focused
let code = tui qr-code "https://ratatui.rs" --no-quiet-zone

tui (tui layout [$banner $levels $prompt $code]
  --constraints [{length: 6} {length: 6} {length: 3} {fill: 1}])
```

Prompt records are declarative snapshots. Keep their `value`, `selected`, `status`, and `focused` fields in ordinary application state and update that state from `--on-key`; the next view rebuild renders the new snapshot. A scroll view works the same way: update `scroll-x` or `scroll-y` from handlers to move its viewport.

`fractional-scrollbar` is the suite's subcell-precision widget. The existing `scrollbar` type remains Ratatui's native scrollbar, preserving its orientations and behavior.

## Presentation fields

All block-wrapped widgets accept these fields: bar charts, buttons, calendars, canvases, charts, gauges, line gauges, lists, paragraphs, sparklines, tables, tabs, big text, popups, and both prompt types.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `title` | string | — | Block title. |
| `border` | bool | `false` | Draw all borders. Buttons default to `true`. |
| `border-type` | string | `plain` | `plain`, `rounded`, `double`, or `thick`. |
| `style` | style record | terminal default | Widget foreground, background, and modifiers. |
| `border-style` | style record | terminal default | Border foreground, background, and modifiers. |

A style record supports:

| Field | Type | Description |
| --- | --- | --- |
| `fg` | color | Foreground color. |
| `bg` | color | Background color. |
| `bold` | bool | Enable bold text. |
| `italic` | bool | Enable italic text. |
| `underlined` | bool | Enable underlining. |
| `reversed` | bool | Swap foreground and background. |
| `dim` | bool | Enable dim text. |

Colors can be Ratatui color names, six-digit hexadecimal strings such as `#5fd7ff`, or indexed color integers from 0 through 255. Color availability and text modifier rendering depend on the terminal.

### Style constructor

Use `tui style` to create the same record with validated constructor flags:

```nu
let warning = tui style --fg yellow --bg "#202020" --bold
tui paragraph "Careful" --style $warning
```

Every field is optional. Omitted colors use the terminal default, and omitted modifiers default to `false`. `--underline` is accepted as an alias for `--underlined`.
