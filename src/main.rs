use nu_plugin::{MsgPackSerializer, serve_plugin};

mod config;
mod effect;
mod example;
mod plugin;
mod run;
mod style;
mod ui;
mod widget;

use plugin::TuiPlugin;

/// Starts the Nushell plugin protocol loop.
fn main() {
  serve_plugin(&TuiPlugin, MsgPackSerializer);
}
