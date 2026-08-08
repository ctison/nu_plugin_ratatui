use std::{
  collections::{HashMap, HashSet},
  time::Duration,
};

use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Category, LabeledError, Record, Signature, SyntaxShape, Type, Value};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use tachyonfx::{
  CellFilter, Effect, EffectTimer, Interpolation, Motion, SimpleRng,
  fx::{self, RepeatMode},
};

use crate::{
  config::{UiNode, parse_color},
  plugin::TuiPlugin,
};

/// The target cadence used while at least one declarative effect is running.
pub(crate) const ANIMATION_FRAME: Duration = Duration::from_millis(16);

/// A parsed, comparable cell filter used by declarative leaf effects.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum EffectFilterSpec {
  Text,
  NonEmpty,
  FgColor(Color),
  BgColor(Color),
  Not(Box<Self>),
  AllOf(Vec<Self>),
  AnyOf(Vec<Self>),
}

/// A parsed, comparable effect specification used as the registry fingerprint.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum EffectSpec {
  FadeFrom {
    fg: Color,
    bg: Color,
    timer: TimerSpec,
    filter: Option<EffectFilterSpec>,
  },
  FadeTo {
    fg: Color,
    bg: Color,
    timer: TimerSpec,
    filter: Option<EffectFilterSpec>,
  },
  FadeFromFg {
    color: Color,
    timer: TimerSpec,
    filter: Option<EffectFilterSpec>,
  },
  FadeToFg {
    color: Color,
    timer: TimerSpec,
    filter: Option<EffectFilterSpec>,
  },
  Dissolve {
    timer: TimerSpec,
    filter: Option<EffectFilterSpec>,
    seed: Option<u32>,
  },
  Coalesce {
    timer: TimerSpec,
    filter: Option<EffectFilterSpec>,
    seed: Option<u32>,
  },
  SweepIn(DirectionalSpec),
  SweepOut(DirectionalSpec),
  SlideIn(DirectionalSpec),
  SlideOut(DirectionalSpec),
  Sequence(Vec<Self>),
  Parallel(Vec<Self>),
  Repeat {
    effect: Box<Self>,
    mode: RepeatSpec,
  },
  PingPong(Box<Self>),
}

/// Timing fields shared by every finite leaf effect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TimerSpec {
  duration_ms: u32,
  interpolation: Interpolation,
}

/// Fields shared by sweep and slide effects.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DirectionalSpec {
  direction: Motion,
  gradient_length: u16,
  randomness: u16,
  color: Color,
  timer: TimerSpec,
  filter: Option<EffectFilterSpec>,
  seed: Option<u32>,
}

/// Repeat termination selected by the declarative repeat record.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RepeatSpec {
  Forever,
  Times(u32),
  Duration(u32),
}

/// One live TachyonFX effect and the record specification that created it.
struct EffectEntry {
  spec: EffectSpec,
  effect: Effect,
}

/// Stateful effects keyed by wrapper ID and reconciled once per rendered frame.
#[derive(Default)]
pub(crate) struct EffectRegistry {
  entries: HashMap<String, EffectEntry>,
  seen: HashSet<String>,
  elapsed: Duration,
}

impl EffectRegistry {
  /// Starts reconciliation for a frame and records its actual elapsed time.
  pub(crate) fn begin_frame(&mut self, elapsed: Duration) {
    self.seen.clear();
    self.elapsed = elapsed;
  }

  /// Applies one wrapper effect after its child has rendered into the current area.
  pub(crate) fn apply(&mut self, id: &str, spec: &EffectSpec, buffer: &mut Buffer, area: Rect) {
    let is_new = match self.entries.get(id) {
      Some(entry) => entry.spec != *spec,
      None => true,
    };
    if is_new {
      self.entries.insert(
        id.to_owned(),
        EffectEntry {
          spec: spec.clone(),
          effect: spec.build(),
        },
      );
    }
    self.seen.insert(id.to_owned());
    let elapsed = if is_new { Duration::ZERO } else { self.elapsed };
    let entry = self.entries.get_mut(id).expect("effect entry was inserted");
    entry.effect.process(elapsed, buffer, area);
  }

  /// Removes registry entries whose wrappers were absent from the rendered tree.
  pub(crate) fn end_frame(&mut self) {
    self.entries.retain(|id, _| self.seen.contains(id));
  }

  /// Reports whether another animation-cadence frame is needed.
  pub(crate) fn any_running(&self) -> bool {
    self.entries.values().any(|entry| entry.effect.running())
  }
}

impl EffectSpec {
  /// Parses and validates an ordinary Nushell effect record recursively.
  pub(crate) fn parse(value: &Value) -> Result<Self, LabeledError> {
    let record = effect_record(value, "effect")?;
    let effect_type = required_string(record, "type", value)?;
    match effect_type.as_str() {
      "fade-from" => Ok(Self::FadeFrom {
        fg: required_color(record, "fg", value)?,
        bg: required_color(record, "bg", value)?,
        timer: parse_timer(record, value)?,
        filter: optional_filter(record)?,
      }),
      "fade-to" => Ok(Self::FadeTo {
        fg: required_color(record, "fg", value)?,
        bg: required_color(record, "bg", value)?,
        timer: parse_timer(record, value)?,
        filter: optional_filter(record)?,
      }),
      "fade-from-fg" => Ok(Self::FadeFromFg {
        color: required_color(record, "color", value)?,
        timer: parse_timer(record, value)?,
        filter: optional_filter(record)?,
      }),
      "fade-to-fg" => Ok(Self::FadeToFg {
        color: required_color(record, "color", value)?,
        timer: parse_timer(record, value)?,
        filter: optional_filter(record)?,
      }),
      "dissolve" => Ok(Self::Dissolve {
        timer: parse_timer(record, value)?,
        filter: optional_filter(record)?,
        seed: optional_u32(record, "seed")?,
      }),
      "coalesce" => Ok(Self::Coalesce {
        timer: parse_timer(record, value)?,
        filter: optional_filter(record)?,
        seed: optional_u32(record, "seed")?,
      }),
      "sweep-in" => Ok(Self::SweepIn(parse_directional(record, value)?)),
      "sweep-out" => Ok(Self::SweepOut(parse_directional(record, value)?)),
      "slide-in" => Ok(Self::SlideIn(parse_directional(record, value)?)),
      "slide-out" => Ok(Self::SlideOut(parse_directional(record, value)?)),
      "sequence" => Ok(Self::Sequence(parse_effects(record, value)?)),
      "parallel" => Ok(Self::Parallel(parse_effects(record, value)?)),
      "repeat" => parse_repeat(record, value),
      "ping-pong" => Ok(Self::PingPong(Box::new(Self::parse(required(
        record, "effect", value,
      )?)?))),
      other => Err(effect_error(
        format!("unsupported effect type `{other}`"),
        value,
      )),
    }
  }

  /// Builds a fresh TachyonFX value at the start of an effect lifecycle.
  fn build(&self) -> Effect {
    match self {
      Self::FadeFrom {
        fg,
        bg,
        timer,
        filter,
      } => with_filter(fx::fade_from(*fg, *bg, timer.build()), filter),
      Self::FadeTo {
        fg,
        bg,
        timer,
        filter,
      } => with_filter(fx::fade_to(*fg, *bg, timer.build()), filter),
      Self::FadeFromFg {
        color,
        timer,
        filter,
      } => with_filter(fx::fade_from_fg(*color, timer.build()), filter),
      Self::FadeToFg {
        color,
        timer,
        filter,
      } => with_filter(fx::fade_to_fg(*color, timer.build()), filter),
      Self::Dissolve {
        timer,
        filter,
        seed,
      } => with_seed(with_filter(fx::dissolve(timer.build()), filter), *seed),
      Self::Coalesce {
        timer,
        filter,
        seed,
      } => with_seed(with_filter(fx::coalesce(timer.build()), filter), *seed),
      Self::SweepIn(spec) => spec.build(fx::sweep_in),
      Self::SweepOut(spec) => spec.build(fx::sweep_out),
      Self::SlideIn(spec) => spec.build(fx::slide_in),
      Self::SlideOut(spec) => spec.build(fx::slide_out),
      Self::Sequence(specs) => fx::sequence(&specs.iter().map(Self::build).collect::<Vec<_>>()),
      Self::Parallel(specs) => fx::parallel(&specs.iter().map(Self::build).collect::<Vec<_>>()),
      Self::Repeat { effect, mode } => fx::repeat(effect.build(), mode.build()),
      Self::PingPong(effect) => fx::ping_pong(effect.build()),
    }
  }
}

impl TimerSpec {
  /// Builds the upstream timer represented by this declarative value.
  fn build(self) -> EffectTimer {
    EffectTimer::from_ms(self.duration_ms, self.interpolation)
  }
}

impl DirectionalSpec {
  /// Builds one of the four upstream directional effects with shared options.
  fn build(
    &self,
    constructor: impl FnOnce(Motion, u16, u16, Color, EffectTimer) -> Effect,
  ) -> Effect {
    with_seed(
      with_filter(
        constructor(
          self.direction,
          self.gradient_length,
          self.randomness,
          self.color,
          self.timer.build(),
        ),
        &self.filter,
      ),
      self.seed,
    )
  }
}

impl RepeatSpec {
  /// Converts the declarative repeat mode to TachyonFX's repeat mode.
  fn build(self) -> RepeatMode {
    match self {
      Self::Forever => RepeatMode::Forever,
      Self::Times(times) => RepeatMode::Times(times),
      Self::Duration(duration_ms) => {
        RepeatMode::Duration(Duration::from_millis(duration_ms.into()))
      },
    }
  }
}

impl EffectFilterSpec {
  /// Parses and validates an ordinary Nushell filter record recursively.
  pub(crate) fn parse(value: &Value) -> Result<Self, LabeledError> {
    let record = effect_record(value, "effect filter")?;
    let filter_type = required_string(record, "type", value)?;
    match filter_type.as_str() {
      "text" => Ok(Self::Text),
      "non-empty" => Ok(Self::NonEmpty),
      "fg-color" => Ok(Self::FgColor(required_color(record, "color", value)?)),
      "bg-color" => Ok(Self::BgColor(required_color(record, "color", value)?)),
      "not" => Ok(Self::Not(Box::new(Self::parse(required(
        record, "filter", value,
      )?)?))),
      "all-of" => Ok(Self::AllOf(parse_filters(record, value)?)),
      "any-of" => Ok(Self::AnyOf(parse_filters(record, value)?)),
      other => Err(effect_error(
        format!("unsupported effect filter type `{other}`"),
        value,
      )),
    }
  }

  /// Builds the upstream cell filter represented by this record.
  fn build(&self) -> CellFilter {
    match self {
      Self::Text => CellFilter::Text,
      Self::NonEmpty => CellFilter::NonEmpty,
      Self::FgColor(color) => CellFilter::FgColor(*color),
      Self::BgColor(color) => CellFilter::BgColor(*color),
      Self::Not(filter) => CellFilter::Not(Box::new(filter.build())),
      Self::AllOf(filters) => CellFilter::AllOf(filters.iter().map(Self::build).collect()),
      Self::AnyOf(filters) => CellFilter::AnyOf(filters.iter().map(Self::build).collect()),
    }
  }
}

/// Applies an optional parsed filter to a newly built leaf effect.
fn with_filter(effect: Effect, filter: &Option<EffectFilterSpec>) -> Effect {
  match filter {
    Some(filter) => effect.with_filter(filter.build()),
    None => effect,
  }
}

/// Applies a deterministic RNG when a seed was supplied.
fn with_seed(effect: Effect, seed: Option<u32>) -> Effect {
  match seed {
    Some(seed) => effect.with_rng(SimpleRng::new(seed)),
    None => effect,
  }
}

/// Parses a required positive millisecond duration and interpolation.
fn parse_timer(record: &Record, parent: &Value) -> Result<TimerSpec, LabeledError> {
  let duration_value = required(record, "duration-ms", parent)?;
  let duration_ms = positive_u32(duration_value, "duration-ms")?;
  let interpolation = match record.get("interpolation") {
    Some(value) => parse_interpolation(value)?,
    None => Interpolation::Linear,
  };
  Ok(TimerSpec {
    duration_ms,
    interpolation,
  })
}

/// Parses every upstream interpolation from its kebab-case API spelling.
fn parse_interpolation(value: &Value) -> Result<Interpolation, LabeledError> {
  let name = value
    .as_str()
    .map_err(|_| effect_error("`interpolation` must be a string", value))?;
  let interpolation = match name {
    "back-in" => Interpolation::BackIn,
    "back-out" => Interpolation::BackOut,
    "back-in-out" => Interpolation::BackInOut,
    "bounce-in" => Interpolation::BounceIn,
    "bounce-out" => Interpolation::BounceOut,
    "bounce-in-out" => Interpolation::BounceInOut,
    "circ-in" => Interpolation::CircIn,
    "circ-out" => Interpolation::CircOut,
    "circ-in-out" => Interpolation::CircInOut,
    "cubic-in" => Interpolation::CubicIn,
    "cubic-out" => Interpolation::CubicOut,
    "cubic-in-out" => Interpolation::CubicInOut,
    "elastic-in" => Interpolation::ElasticIn,
    "elastic-out" => Interpolation::ElasticOut,
    "elastic-in-out" => Interpolation::ElasticInOut,
    "expo-in" => Interpolation::ExpoIn,
    "expo-out" => Interpolation::ExpoOut,
    "expo-in-out" => Interpolation::ExpoInOut,
    "linear" => Interpolation::Linear,
    "quad-in" => Interpolation::QuadIn,
    "quad-out" => Interpolation::QuadOut,
    "quad-in-out" => Interpolation::QuadInOut,
    "quart-in" => Interpolation::QuartIn,
    "quart-out" => Interpolation::QuartOut,
    "quart-in-out" => Interpolation::QuartInOut,
    "quint-in" => Interpolation::QuintIn,
    "quint-out" => Interpolation::QuintOut,
    "quint-in-out" => Interpolation::QuintInOut,
    "reverse" => Interpolation::Reverse,
    "smooth-step" => Interpolation::SmoothStep,
    "spring" => Interpolation::Spring,
    "sine-in" => Interpolation::SineIn,
    "sine-out" => Interpolation::SineOut,
    "sine-in-out" => Interpolation::SineInOut,
    other => {
      return Err(effect_error(
        format!("unsupported interpolation `{other}`"),
        value,
      ));
    },
  };
  Ok(interpolation)
}

/// Parses fields shared by sweep and slide records.
fn parse_directional(record: &Record, parent: &Value) -> Result<DirectionalSpec, LabeledError> {
  let direction_value = required(record, "direction", parent)?;
  let direction = match direction_value
    .as_str()
    .map_err(|_| effect_error("`direction` must be a string", direction_value))?
  {
    "left-to-right" => Motion::LeftToRight,
    "right-to-left" => Motion::RightToLeft,
    "up-to-down" => Motion::UpToDown,
    "down-to-up" => Motion::DownToUp,
    other => {
      return Err(effect_error(
        format!("unsupported effect direction `{other}`"),
        direction_value,
      ));
    },
  };
  Ok(DirectionalSpec {
    direction,
    gradient_length: u16_field(record, "gradient-length", parent)?,
    randomness: u16_field(record, "randomness", parent)?,
    color: required_color(record, "color", parent)?,
    timer: parse_timer(record, parent)?,
    filter: optional_filter(record)?,
    seed: optional_u32(record, "seed")?,
  })
}

/// Parses a non-empty list of nested effects.
fn parse_effects(record: &Record, parent: &Value) -> Result<Vec<EffectSpec>, LabeledError> {
  let value = required(record, "effects", parent)?;
  let values = value
    .as_list()
    .map_err(|_| effect_error("`effects` must be a list of effect records", value))?;
  if values.is_empty() {
    return Err(effect_error(
      "`effects` must contain at least one effect",
      value,
    ));
  }
  values.iter().map(EffectSpec::parse).collect()
}

/// Parses a repeat record and enforces its mutually exclusive modes.
fn parse_repeat(record: &Record, parent: &Value) -> Result<EffectSpec, LabeledError> {
  let effect = Box::new(EffectSpec::parse(required(record, "effect", parent)?)?);
  let times = record
    .get("times")
    .map(|value| positive_u32(value, "times"))
    .transpose()?;
  let duration = record
    .get("duration-ms")
    .map(|value| positive_u32(value, "duration-ms"))
    .transpose()?;
  let mode = match (times, duration) {
    (Some(_), Some(_)) => {
      return Err(effect_error(
        "`times` and `duration-ms` cannot be used together",
        parent,
      ));
    },
    (Some(times), None) => RepeatSpec::Times(times),
    (None, Some(duration)) => RepeatSpec::Duration(duration),
    (None, None) => RepeatSpec::Forever,
  };
  Ok(EffectSpec::Repeat { effect, mode })
}

/// Parses an optional nested filter.
fn optional_filter(record: &Record) -> Result<Option<EffectFilterSpec>, LabeledError> {
  record
    .get("filter")
    .map(EffectFilterSpec::parse)
    .transpose()
}

/// Parses a non-empty list of nested filters.
fn parse_filters(record: &Record, parent: &Value) -> Result<Vec<EffectFilterSpec>, LabeledError> {
  let value = required(record, "filters", parent)?;
  let values = value
    .as_list()
    .map_err(|_| effect_error("`filters` must be a list of filter records", value))?;
  if values.is_empty() {
    return Err(effect_error(
      "`filters` must contain at least one filter",
      value,
    ));
  }
  values.iter().map(EffectFilterSpec::parse).collect()
}

/// Converts a Nushell value to a record for effect diagnostics.
fn effect_record<'a>(value: &'a Value, context: &str) -> Result<&'a Record, LabeledError> {
  value
    .as_record()
    .map_err(|_| effect_error(format!("{context} must be a record"), value))
}

/// Returns a required record field.
fn required<'a>(
  record: &'a Record,
  field: &str,
  parent: &Value,
) -> Result<&'a Value, LabeledError> {
  record
    .get(field)
    .ok_or_else(|| effect_error(format!("missing required field `{field}`"), parent))
}

/// Parses a required string field.
fn required_string(record: &Record, field: &str, parent: &Value) -> Result<String, LabeledError> {
  let value = required(record, field, parent)?;
  value
    .as_str()
    .map(str::to_owned)
    .map_err(|_| effect_error(format!("`{field}` must be a string"), value))
}

/// Parses a required color with the widget parser's shared color rules.
fn required_color(record: &Record, field: &str, parent: &Value) -> Result<Color, LabeledError> {
  parse_color(required(record, field, parent)?)
}

/// Parses a strictly positive integer that fits TachyonFX's u32 inputs.
fn positive_u32(value: &Value, field: &str) -> Result<u32, LabeledError> {
  let integer = value
    .as_int()
    .map_err(|_| effect_error(format!("`{field}` must be an integer"), value))?;
  let integer = u32::try_from(integer).map_err(|_| {
    effect_error(
      format!("`{field}` must be between 1 and {}", u32::MAX),
      value,
    )
  })?;
  if integer == 0 {
    return Err(effect_error(format!("`{field}` must be positive"), value));
  }
  Ok(integer)
}

/// Parses a required unsigned 16-bit field.
fn u16_field(record: &Record, field: &str, parent: &Value) -> Result<u16, LabeledError> {
  let value = required(record, field, parent)?;
  let integer = value
    .as_int()
    .map_err(|_| effect_error(format!("`{field}` must be an integer"), value))?;
  u16::try_from(integer).map_err(|_| {
    effect_error(
      format!("`{field}` must be between 0 and {}", u16::MAX),
      value,
    )
  })
}

/// Parses an optional unsigned 32-bit field.
fn optional_u32(record: &Record, field: &str) -> Result<Option<u32>, LabeledError> {
  record
    .get(field)
    .map(|value| {
      let integer = value
        .as_int()
        .map_err(|_| effect_error(format!("`{field}` must be an integer"), value))?;
      u32::try_from(integer).map_err(|_| {
        effect_error(
          format!("`{field}` must be between 0 and {}", u32::MAX),
          value,
        )
      })
    })
    .transpose()
}

/// Creates a consistently labelled effect validation error.
fn effect_error(message: impl Into<String>, value: &Value) -> LabeledError {
  LabeledError::new("Invalid TUI effect")
    .with_label(message, value.span())
    .with_code("nu_plugin_tui::invalid_effect")
}

/// Identifies one wrapper, effect, or filter record constructor command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectCommandKind {
  Wrapper,
  FadeFrom,
  FadeTo,
  FadeFromFg,
  FadeToFg,
  Dissolve,
  Coalesce,
  SweepIn,
  SweepOut,
  SlideIn,
  SlideOut,
  Sequence,
  Parallel,
  Repeat,
  PingPong,
  FilterText,
  FilterNonEmpty,
  FilterFgColor,
  FilterBgColor,
  FilterNot,
  FilterAllOf,
  FilterAnyOf,
}

impl EffectCommandKind {
  /// Contains every effect-related command in registration order.
  pub(crate) const ALL: [Self; 22] = [
    Self::Wrapper,
    Self::FadeFrom,
    Self::FadeTo,
    Self::FadeFromFg,
    Self::FadeToFg,
    Self::Dissolve,
    Self::Coalesce,
    Self::SweepIn,
    Self::SweepOut,
    Self::SlideIn,
    Self::SlideOut,
    Self::Sequence,
    Self::Parallel,
    Self::Repeat,
    Self::PingPong,
    Self::FilterText,
    Self::FilterNonEmpty,
    Self::FilterFgColor,
    Self::FilterBgColor,
    Self::FilterNot,
    Self::FilterAllOf,
    Self::FilterAnyOf,
  ];

  /// Returns the fully qualified Nushell command name.
  pub(crate) fn command_name(self) -> &'static str {
    match self {
      Self::Wrapper => "tui effect",
      Self::FadeFrom => "tui effect fade-from",
      Self::FadeTo => "tui effect fade-to",
      Self::FadeFromFg => "tui effect fade-from-fg",
      Self::FadeToFg => "tui effect fade-to-fg",
      Self::Dissolve => "tui effect dissolve",
      Self::Coalesce => "tui effect coalesce",
      Self::SweepIn => "tui effect sweep-in",
      Self::SweepOut => "tui effect sweep-out",
      Self::SlideIn => "tui effect slide-in",
      Self::SlideOut => "tui effect slide-out",
      Self::Sequence => "tui effect sequence",
      Self::Parallel => "tui effect parallel",
      Self::Repeat => "tui effect repeat",
      Self::PingPong => "tui effect ping-pong",
      Self::FilterText => "tui effect filter text",
      Self::FilterNonEmpty => "tui effect filter non-empty",
      Self::FilterFgColor => "tui effect filter fg-color",
      Self::FilterBgColor => "tui effect filter bg-color",
      Self::FilterNot => "tui effect filter not",
      Self::FilterAllOf => "tui effect filter all-of",
      Self::FilterAnyOf => "tui effect filter any-of",
    }
  }

  /// Returns the `type` stored in the constructed ordinary record.
  fn record_type(self) -> &'static str {
    self
      .command_name()
      .rsplit_once(' ')
      .map_or("effect", |(_, name)| name)
  }

  /// Returns whether this command constructs a filter rather than an effect.
  fn is_filter(self) -> bool {
    matches!(
      self,
      Self::FilterText
        | Self::FilterNonEmpty
        | Self::FilterFgColor
        | Self::FilterBgColor
        | Self::FilterNot
        | Self::FilterAllOf
        | Self::FilterAnyOf
    )
  }

  /// Returns required positional record fields in signature order.
  fn required_fields(self) -> &'static [&'static str] {
    match self {
      Self::Wrapper => &["id", "effect", "child"],
      Self::FadeFrom | Self::FadeTo => &["fg", "bg", "duration-ms"],
      Self::FadeFromFg | Self::FadeToFg => &["color", "duration-ms"],
      Self::Dissolve | Self::Coalesce => &["duration-ms"],
      Self::SweepIn | Self::SweepOut | Self::SlideIn | Self::SlideOut => &[
        "direction",
        "gradient-length",
        "randomness",
        "color",
        "duration-ms",
      ],
      Self::Sequence | Self::Parallel => &["effects"],
      Self::Repeat | Self::PingPong => &["effect"],
      Self::FilterFgColor | Self::FilterBgColor => &["color"],
      Self::FilterNot => &["filter"],
      Self::FilterAllOf | Self::FilterAnyOf => &["filters"],
      Self::FilterText | Self::FilterNonEmpty => &[],
    }
  }

  /// Defines the typed constructor signature for this record kind.
  fn signature(self) -> Signature {
    let mut signature = Signature::build(self.command_name())
      .input_output_type(Type::Nothing, Type::Record(Default::default()))
      .category(Category::Experimental);
    for field in self.required_fields() {
      let shape = match *field {
        "duration-ms" | "gradient-length" | "randomness" => SyntaxShape::Int,
        "effects" | "filters" => SyntaxShape::List(Box::new(record_shape())),
        "effect" | "filter" | "child" => record_shape(),
        "fg" | "bg" | "color" => color_shape(),
        _ => SyntaxShape::String,
      };
      signature = signature.required(*field, shape, positional_description(field));
    }
    if self.is_timed_leaf() {
      signature = signature
        .named(
          "interpolation",
          SyntaxShape::String,
          "Kebab-case easing name (default: linear)",
          None,
        )
        .named(
          "filter",
          record_shape(),
          "Cell filter record (default: all cells)",
          None,
        );
    }
    if self.is_randomized() {
      signature = signature.named(
        "seed",
        SyntaxShape::Int,
        "Unsigned deterministic RNG seed (default: random)",
        None,
      );
    }
    if self == Self::Repeat {
      signature = signature
        .named(
          "times",
          SyntaxShape::Int,
          "Positive repeat count (default: repeat forever)",
          None,
        )
        .named(
          "duration-ms",
          SyntaxShape::Int,
          "Positive repeat duration (default: repeat forever)",
          None,
        );
    }
    signature
  }

  /// Returns whether this constructor accepts timing and filtering options.
  fn is_timed_leaf(self) -> bool {
    matches!(
      self,
      Self::FadeFrom
        | Self::FadeTo
        | Self::FadeFromFg
        | Self::FadeToFg
        | Self::Dissolve
        | Self::Coalesce
        | Self::SweepIn
        | Self::SweepOut
        | Self::SlideIn
        | Self::SlideOut
    )
  }

  /// Returns whether this constructor accepts a deterministic random seed.
  fn is_randomized(self) -> bool {
    matches!(
      self,
      Self::Dissolve
        | Self::Coalesce
        | Self::SweepIn
        | Self::SweepOut
        | Self::SlideIn
        | Self::SlideOut
    )
  }
}

/// A single typed `tui effect ...` record constructor.
pub(crate) struct TuiEffect {
  kind: EffectCommandKind,
}

impl TuiEffect {
  /// Creates a command for one effect-related record kind.
  pub(crate) const fn new(kind: EffectCommandKind) -> Self {
    Self { kind }
  }
}

impl SimplePluginCommand for TuiEffect {
  type Plugin = TuiPlugin;

  /// Returns the constructor command name exposed to Nushell.
  fn name(&self) -> &str {
    self.kind.command_name()
  }

  /// Defines positional fields and typed options for this constructor.
  fn signature(&self) -> Signature {
    self.kind.signature()
  }

  /// Describes the ordinary record produced by the command.
  fn description(&self) -> &str {
    if self.kind == EffectCommandKind::Wrapper {
      "Wrap a widget subtree in a stateful TachyonFX effect"
    } else if self.kind.is_filter() {
      "Declare a TachyonFX cell filter record"
    } else {
      "Declare a TachyonFX effect record"
    }
  }

  /// Constructs and validates a manually authorable Nushell record.
  fn run(
    &self,
    _plugin: &TuiPlugin,
    _engine: &EngineInterface,
    call: &EvaluatedCall,
    _input: &Value,
  ) -> Result<Value, LabeledError> {
    build_effect_record(self.kind, call)
  }
}

/// Builds an open record syntax shape.
fn record_shape() -> SyntaxShape {
  SyntaxShape::Record(Default::default())
}

/// Builds the name-or-index color syntax shape shared with widget records.
fn color_shape() -> SyntaxShape {
  SyntaxShape::OneOf(vec![SyntaxShape::Int, SyntaxShape::String])
}

/// Returns concise positional help for one record field.
fn positional_description(field: &str) -> &'static str {
  match field {
    "id" => "Unique effect wrapper ID",
    "effect" => "Nested effect record",
    "child" => "Wrapped widget record",
    "fg" => "Foreground color",
    "bg" => "Background color",
    "color" => "Effect color",
    "duration-ms" => "Positive duration in milliseconds",
    "direction" => "left-to-right, right-to-left, up-to-down, or down-to-up",
    "gradient-length" => "Gradient length in cells",
    "randomness" => "Maximum random offset",
    "effects" => "Non-empty list of effect records",
    "filter" => "Nested filter record",
    "filters" => "Non-empty list of filter records",
    _ => "Required value",
  }
}

/// Copies constructor arguments into a record and validates it through the runtime parser.
fn build_effect_record(
  kind: EffectCommandKind,
  call: &EvaluatedCall,
) -> Result<Value, LabeledError> {
  let mut record = Record::new();
  record.push("type", Value::string(kind.record_type(), call.head));
  for (index, field) in kind.required_fields().iter().enumerate() {
    let value = call.positional.get(index).cloned().ok_or_else(|| {
      LabeledError::new("Missing required effect argument")
        .with_label(format!("missing positional `{field}`"), call.head)
    })?;
    record.push(*field, value);
  }
  for (name, value) in &call.named {
    if let Some(value) = value {
      record.push(name.item.clone(), value.clone());
    }
  }
  let value = Value::record(record, call.head);
  if kind == EffectCommandKind::Wrapper {
    UiNode::parse(&value)?;
  } else if kind.is_filter() {
    EffectFilterSpec::parse(&value)?;
  } else {
    EffectSpec::parse(&value)?;
  }
  Ok(value)
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use nu_plugin::{EvaluatedCall, SimplePluginCommand};
  use nu_protocol::{IntoSpanned, Record, Span, Value};
  use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect, style::Color};

  use super::{
    EffectCommandKind, EffectFilterSpec, EffectRegistry, EffectSpec, TuiEffect, build_effect_record,
  };
  use crate::config::UiNode;

  /// Builds a concise ordinary record value.
  fn record(fields: Vec<(&str, Value)>) -> Value {
    let mut record = Record::new();
    for (name, value) in fields {
      record.push(name, value);
    }
    Value::test_record(record)
  }

  /// Builds a valid timed leaf record for parser tests.
  fn fade(color: &str, duration: i64) -> Value {
    record(vec![
      ("type", Value::test_string("fade-to-fg")),
      ("color", Value::test_string(color)),
      ("duration-ms", Value::test_int(duration)),
    ])
  }

  /// Builds a test command call from positional and named values.
  fn call(positional: Vec<Value>, named: Vec<(&str, Value)>) -> EvaluatedCall {
    let span = Span::test_data();
    let mut call = EvaluatedCall::new(span);
    for value in positional {
      call.add_positional(value);
    }
    for (name, value) in named {
      call.add_named(name.to_owned().into_spanned(span), value);
    }
    call
  }

  /// Wraps one child record in a declarative effect widget.
  fn wrapper(id: &str, effect: Value, child: Value) -> Value {
    record(vec![
      ("type", Value::test_string("effect")),
      ("id", Value::test_string(id)),
      ("effect", effect),
      ("child", child),
    ])
  }

  /// Builds a styled single-line paragraph record.
  fn paragraph(text: &str, color: &str) -> Value {
    record(vec![
      ("type", Value::test_string("paragraph")),
      ("text", Value::test_string(text)),
      ("style", record(vec![("fg", Value::test_string(color))])),
    ])
  }

  /// Draws one parsed node while preserving registry state between calls.
  fn draw_node(
    node: &UiNode,
    terminal: &mut Terminal<TestBackend>,
    registry: &mut EffectRegistry,
    elapsed: Duration,
  ) -> Buffer {
    registry.begin_frame(elapsed);
    terminal
      .draw(|frame| node.render_with_effects(frame, frame.area(), &mut Vec::new(), registry))
      .unwrap();
    registry.end_frame();
    terminal.backend().buffer().clone()
  }

  /// Verifies all supported leaf and composition effect records parse.
  #[test]
  fn parses_every_effect_record() {
    let filter = record(vec![("type", Value::test_string("text"))]);
    let mut effects = vec![
      record(vec![
        ("type", Value::test_string("fade-from")),
        ("fg", Value::test_string("red")),
        ("bg", Value::test_string("black")),
        ("duration-ms", Value::test_int(10)),
      ]),
      record(vec![
        ("type", Value::test_string("fade-to")),
        ("fg", Value::test_string("red")),
        ("bg", Value::test_string("black")),
        ("duration-ms", Value::test_int(10)),
      ]),
      record(vec![
        ("type", Value::test_string("fade-from-fg")),
        ("color", Value::test_string("red")),
        ("duration-ms", Value::test_int(10)),
      ]),
      fade("red", 10),
      record(vec![
        ("type", Value::test_string("dissolve")),
        ("duration-ms", Value::test_int(10)),
        ("filter", filter.clone()),
        ("seed", Value::test_int(u32::MAX.into())),
      ]),
      record(vec![
        ("type", Value::test_string("coalesce")),
        ("duration-ms", Value::test_int(10)),
      ]),
    ];
    for name in ["sweep-in", "sweep-out", "slide-in", "slide-out"] {
      effects.push(record(vec![
        ("type", Value::test_string(name)),
        ("direction", Value::test_string("left-to-right")),
        ("gradient-length", Value::test_int(2)),
        ("randomness", Value::test_int(1)),
        ("color", Value::test_string("black")),
        ("duration-ms", Value::test_int(10)),
      ]));
    }
    let leaf = fade("blue", 10);
    effects.extend([
      record(vec![
        ("type", Value::test_string("sequence")),
        ("effects", Value::test_list(vec![leaf.clone()])),
      ]),
      record(vec![
        ("type", Value::test_string("parallel")),
        ("effects", Value::test_list(vec![leaf.clone()])),
      ]),
      record(vec![
        ("type", Value::test_string("repeat")),
        ("effect", leaf.clone()),
        ("times", Value::test_int(2)),
      ]),
      record(vec![
        ("type", Value::test_string("repeat")),
        ("effect", leaf.clone()),
        ("duration-ms", Value::test_int(20)),
      ]),
      record(vec![
        ("type", Value::test_string("repeat")),
        ("effect", leaf.clone()),
      ]),
      record(vec![
        ("type", Value::test_string("ping-pong")),
        ("effect", leaf),
      ]),
    ]);
    for effect in effects {
      EffectSpec::parse(&effect).expect("supported effect parses");
    }
  }

  /// Verifies every filter record kind parses recursively.
  #[test]
  fn parses_every_filter_record() {
    let text = record(vec![("type", Value::test_string("text"))]);
    let filters = vec![
      text.clone(),
      record(vec![("type", Value::test_string("non-empty"))]),
      record(vec![
        ("type", Value::test_string("fg-color")),
        ("color", Value::test_string("red")),
      ]),
      record(vec![
        ("type", Value::test_string("bg-color")),
        ("color", Value::test_string("blue")),
      ]),
      record(vec![
        ("type", Value::test_string("not")),
        ("filter", text.clone()),
      ]),
      record(vec![
        ("type", Value::test_string("all-of")),
        ("filters", Value::test_list(vec![text.clone()])),
      ]),
      record(vec![
        ("type", Value::test_string("any-of")),
        ("filters", Value::test_list(vec![text])),
      ]),
    ];
    for filter in filters {
      EffectFilterSpec::parse(&filter).expect("supported filter parses");
    }
  }

  /// Verifies timing, integer ranges, nesting, direction, and repeat modes are validated.
  #[test]
  fn rejects_invalid_effect_fields() {
    for duration in [0, -1, i64::from(u32::MAX) + 1] {
      assert!(EffectSpec::parse(&fade("red", duration)).is_err());
    }
    let bad_interpolation = record(vec![
      ("type", Value::test_string("fade-to-fg")),
      ("color", Value::test_string("red")),
      ("duration-ms", Value::test_int(10)),
      ("interpolation", Value::test_string("wobbly")),
    ]);
    let bad_direction = record(vec![
      ("type", Value::test_string("sweep-in")),
      ("direction", Value::test_string("diagonal")),
      ("gradient-length", Value::test_int(1)),
      ("randomness", Value::test_int(0)),
      ("color", Value::test_string("red")),
      ("duration-ms", Value::test_int(10)),
    ]);
    let both = record(vec![
      ("type", Value::test_string("repeat")),
      ("effect", fade("red", 10)),
      ("times", Value::test_int(2)),
      ("duration-ms", Value::test_int(20)),
    ]);
    let nested_filter = record(vec![
      ("type", Value::test_string("sequence")),
      (
        "effects",
        Value::test_list(vec![record(vec![("type", Value::test_string("text"))])]),
      ),
    ]);
    let zero_times = record(vec![
      ("type", Value::test_string("repeat")),
      ("effect", fade("red", 10)),
      ("times", Value::test_int(0)),
    ]);
    let large_seed = record(vec![
      ("type", Value::test_string("dissolve")),
      ("duration-ms", Value::test_int(10)),
      ("seed", Value::test_int(i64::from(u32::MAX) + 1)),
    ]);
    let large_gradient = record(vec![
      ("type", Value::test_string("slide-out")),
      ("direction", Value::test_string("up-to-down")),
      ("gradient-length", Value::test_int(i64::from(u16::MAX) + 1)),
      ("randomness", Value::test_int(0)),
      ("color", Value::test_string("red")),
      ("duration-ms", Value::test_int(10)),
    ]);
    assert!(EffectSpec::parse(&bad_interpolation).is_err());
    assert!(EffectSpec::parse(&bad_direction).is_err());
    assert!(EffectSpec::parse(&both).is_err());
    assert!(EffectSpec::parse(&nested_filter).is_err());
    assert!(EffectSpec::parse(&zero_times).is_err());
    assert!(EffectSpec::parse(&large_seed).is_err());
    assert!(EffectSpec::parse(&large_gradient).is_err());
  }

  /// Verifies omitted interpolation is exactly the documented linear default.
  #[test]
  fn defaults_interpolation_to_linear() {
    let implicit = EffectSpec::parse(&fade("red", 10)).unwrap();
    let explicit = EffectSpec::parse(&record(vec![
      ("type", Value::test_string("fade-to-fg")),
      ("color", Value::test_string("red")),
      ("duration-ms", Value::test_int(10)),
      ("interpolation", Value::test_string("linear")),
    ]))
    .unwrap();

    assert_eq!(implicit, explicit);
  }

  /// Verifies malformed logical filter nesting and empty lists are rejected.
  #[test]
  fn rejects_invalid_filter_nesting() {
    let empty = record(vec![
      ("type", Value::test_string("all-of")),
      ("filters", Value::test_list(vec![])),
    ]);
    let effect_as_filter = record(vec![
      ("type", Value::test_string("not")),
      ("filter", fade("red", 10)),
    ]);

    assert!(EffectFilterSpec::parse(&empty).is_err());
    assert!(EffectFilterSpec::parse(&effect_as_filter).is_err());
  }

  /// Verifies all upstream interpolation spellings, including special curves, are accepted.
  #[test]
  fn accepts_every_interpolation_name() {
    let names = [
      "back-in",
      "back-out",
      "back-in-out",
      "bounce-in",
      "bounce-out",
      "bounce-in-out",
      "circ-in",
      "circ-out",
      "circ-in-out",
      "cubic-in",
      "cubic-out",
      "cubic-in-out",
      "elastic-in",
      "elastic-out",
      "elastic-in-out",
      "expo-in",
      "expo-out",
      "expo-in-out",
      "linear",
      "quad-in",
      "quad-out",
      "quad-in-out",
      "quart-in",
      "quart-out",
      "quart-in-out",
      "quint-in",
      "quint-out",
      "quint-in-out",
      "reverse",
      "smooth-step",
      "spring",
      "sine-in",
      "sine-out",
      "sine-in-out",
    ];
    for name in names {
      let value = record(vec![
        ("type", Value::test_string("fade-to-fg")),
        ("color", Value::test_string("red")),
        ("duration-ms", Value::test_int(10)),
        ("interpolation", Value::test_string(name)),
      ]);
      EffectSpec::parse(&value).expect("interpolation parses");
    }
  }

  /// Verifies every command is registered distinctly and timed defaults are documented.
  #[test]
  fn defines_every_constructor_signature() {
    let mut names = EffectCommandKind::ALL
      .iter()
      .map(|kind| kind.command_name())
      .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), EffectCommandKind::ALL.len());
    assert!(names.contains(&"tui effect"));
    assert!(names.contains(&"tui effect filter any-of"));
    for kind in EffectCommandKind::ALL {
      for flag in TuiEffect::new(kind)
        .signature()
        .named
        .into_iter()
        .filter(|flag| flag.long != "help")
      {
        assert!(
          flag.desc.contains("(default:"),
          "{} --{} lacks a default",
          kind.command_name(),
          flag.long
        );
      }
    }
  }

  /// Verifies constructor output remains ordinary parser-compatible records.
  #[test]
  fn constructs_typed_effect_record() {
    let value = build_effect_record(
      EffectCommandKind::FadeToFg,
      &call(
        vec![Value::test_string("red"), Value::test_int(50)],
        vec![("interpolation", Value::test_string("quad-out"))],
      ),
    )
    .expect("valid constructor");
    assert_eq!(
      value
        .as_record()
        .unwrap()
        .get("type")
        .unwrap()
        .as_str()
        .unwrap(),
      "fade-to-fg"
    );
  }

  /// Verifies duplicate wrapper IDs are rejected across one parsed widget tree.
  #[test]
  fn rejects_duplicate_wrapper_ids() {
    let child = record(vec![("type", Value::test_string("spacer"))]);
    let wrapper = |id: &str| {
      record(vec![
        ("type", Value::test_string("effect")),
        ("id", Value::test_string(id)),
        ("effect", fade("red", 10)),
        ("child", child.clone()),
      ])
    };
    let tree = record(vec![
      ("type", Value::test_string("layout")),
      (
        "children",
        Value::test_list(vec![wrapper("same"), wrapper("same")]),
      ),
    ]);
    assert!(UiNode::parse(&tree).is_err());
  }

  /// Verifies registry creation, preservation, restart, cleanup, and completion stability.
  #[test]
  fn reconciles_registry_lifecycle() {
    let first = EffectSpec::parse(&fade("red", 10)).unwrap();
    let changed = EffectSpec::parse(&fade("blue", 10)).unwrap();
    let mut registry = EffectRegistry::default();
    let mut terminal = Terminal::new(TestBackend::new(1, 1)).unwrap();
    registry.begin_frame(Duration::ZERO);
    terminal
      .draw(|frame| {
        let area = frame.area();
        registry.apply("a", &first, frame.buffer_mut(), area);
      })
      .unwrap();
    registry.end_frame();
    assert_eq!(registry.entries.len(), 1);
    let original = registry.entries.get("a").unwrap().effect.timer();
    registry.begin_frame(Duration::from_millis(5));
    terminal
      .draw(|frame| {
        let area = frame.area();
        registry.apply("a", &first, frame.buffer_mut(), area);
      })
      .unwrap();
    registry.end_frame();
    assert_ne!(registry.entries.get("a").unwrap().effect.timer(), original);
    registry.begin_frame(Duration::from_millis(5));
    terminal
      .draw(|frame| {
        let area = frame.area();
        registry.apply("a", &changed, frame.buffer_mut(), area);
      })
      .unwrap();
    registry.end_frame();
    assert_eq!(registry.entries.get("a").unwrap().effect.timer(), original);
    registry.begin_frame(Duration::from_millis(20));
    terminal
      .draw(|frame| {
        let area = frame.area();
        registry.apply("a", &changed, frame.buffer_mut(), area);
      })
      .unwrap();
    registry.end_frame();
    assert!(!registry.any_running());
    registry.begin_frame(Duration::from_millis(20));
    terminal
      .draw(|frame| {
        let area = frame.area();
        registry.apply("a", &changed, frame.buffer_mut(), area);
      })
      .unwrap();
    registry.end_frame();
    assert!(!registry.any_running());
    registry.begin_frame(Duration::ZERO);
    terminal
      .draw(|frame| {
        let area = frame.area();
        registry.apply("b", &changed, frame.buffer_mut(), area);
      })
      .unwrap();
    registry.end_frame();
    assert!(!registry.entries.contains_key("a"));
    assert!(registry.entries.get("b").unwrap().effect.running());
    registry.begin_frame(Duration::ZERO);
    registry.end_frame();
    assert!(registry.entries.is_empty());
  }

  /// Verifies a wrapper changes only its allocated subtree rectangle.
  #[test]
  fn scopes_effect_to_wrapped_subtree() {
    let layout = record(vec![
      ("type", Value::test_string("layout")),
      ("direction", Value::test_string("horizontal")),
      (
        "constraints",
        Value::test_list(vec![
          record(vec![("length", Value::test_int(1))]),
          record(vec![("length", Value::test_int(1))]),
        ]),
      ),
      (
        "children",
        Value::test_list(vec![
          wrapper("left", fade("black", 10), paragraph("L", "white")),
          paragraph("R", "white"),
        ]),
      ),
    ]);
    let node = UiNode::parse(&layout).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(2, 1)).unwrap();
    let mut registry = EffectRegistry::default();
    draw_node(&node, &mut terminal, &mut registry, Duration::ZERO);
    let buffer = draw_node(
      &node,
      &mut terminal,
      &mut registry,
      Duration::from_millis(10),
    );

    assert_eq!(buffer.cell((0, 0)).unwrap().fg, Color::Black);
    assert_eq!(buffer.cell((1, 0)).unwrap().fg, Color::White);
  }

  /// Verifies nested wrapper effects apply from the innermost child outward.
  #[test]
  fn applies_nested_effects_inside_out() {
    let child = paragraph("X", "white");
    let node = UiNode::parse(&wrapper(
      "outer",
      fade("blue", 10),
      wrapper("inner", fade("red", 10), child),
    ))
    .unwrap();
    let mut terminal = Terminal::new(TestBackend::new(1, 1)).unwrap();
    let mut registry = EffectRegistry::default();
    draw_node(&node, &mut terminal, &mut registry, Duration::ZERO);
    let buffer = draw_node(
      &node,
      &mut terminal,
      &mut registry,
      Duration::from_millis(10),
    );

    assert_eq!(buffer.cell((0, 0)).unwrap().fg, Color::Blue);
  }

  /// Verifies leaf filters select cells by their rendered foreground color.
  #[test]
  fn filters_effect_cells() {
    let filter = record(vec![
      ("type", Value::test_string("fg-color")),
      ("color", Value::test_string("red")),
    ]);
    let filtered = record(vec![
      ("type", Value::test_string("fade-to-fg")),
      ("color", Value::test_string("green")),
      ("duration-ms", Value::test_int(10)),
      ("filter", filter),
    ]);
    let child = record(vec![
      ("type", Value::test_string("layout")),
      ("direction", Value::test_string("horizontal")),
      (
        "constraints",
        Value::test_list(vec![
          record(vec![("length", Value::test_int(1))]),
          record(vec![("length", Value::test_int(1))]),
        ]),
      ),
      (
        "children",
        Value::test_list(vec![paragraph("R", "red"), paragraph("B", "blue")]),
      ),
    ]);
    let node = UiNode::parse(&wrapper("filtered", filtered, child)).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(2, 1)).unwrap();
    let mut registry = EffectRegistry::default();
    draw_node(&node, &mut terminal, &mut registry, Duration::ZERO);
    let buffer = draw_node(
      &node,
      &mut terminal,
      &mut registry,
      Duration::from_millis(10),
    );

    assert_eq!(buffer.cell((0, 0)).unwrap().fg, Color::Green);
    assert_eq!(buffer.cell((1, 0)).unwrap().fg, Color::Blue);
  }

  /// Verifies identical seeds produce identical randomized dissolve frames.
  #[test]
  fn seeded_effects_are_deterministic() {
    let dissolve = record(vec![
      ("type", Value::test_string("dissolve")),
      ("duration-ms", Value::test_int(100)),
      ("seed", Value::test_int(42)),
    ]);
    let node = UiNode::parse(&wrapper(
      "seeded",
      dissolve,
      paragraph("DETERMINISTIC", "white"),
    ))
    .unwrap();
    let mut first_terminal = Terminal::new(TestBackend::new(13, 1)).unwrap();
    let mut second_terminal = Terminal::new(TestBackend::new(13, 1)).unwrap();
    let mut first_registry = EffectRegistry::default();
    let mut second_registry = EffectRegistry::default();
    draw_node(
      &node,
      &mut first_terminal,
      &mut first_registry,
      Duration::ZERO,
    );
    draw_node(
      &node,
      &mut second_terminal,
      &mut second_registry,
      Duration::ZERO,
    );
    let first = draw_node(
      &node,
      &mut first_terminal,
      &mut first_registry,
      Duration::from_millis(50),
    );
    let second = draw_node(
      &node,
      &mut second_terminal,
      &mut second_registry,
      Duration::from_millis(50),
    );

    assert_eq!(first, second);
  }

  /// Verifies a live wrapper always uses the post-resize allocation rectangle.
  #[test]
  fn uses_current_area_after_resize() {
    let node = UiNode::parse(&wrapper(
      "resized",
      fade("red", 10),
      paragraph("XXXX", "white"),
    ))
    .unwrap();
    let mut terminal = Terminal::new(TestBackend::new(2, 1)).unwrap();
    let mut registry = EffectRegistry::default();
    draw_node(&node, &mut terminal, &mut registry, Duration::ZERO);
    terminal.backend_mut().resize(4, 1);
    terminal.autoresize().unwrap();
    let buffer = draw_node(
      &node,
      &mut terminal,
      &mut registry,
      Duration::from_millis(10),
    );

    assert_eq!(buffer.area, Rect::new(0, 0, 4, 1));
    assert!((0..4).all(|x| buffer.cell((x, 0)).unwrap().fg == Color::Red));
  }
}
