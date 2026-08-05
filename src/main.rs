use nu_plugin::{MsgPackSerializer, serve_plugin};

mod config;
mod plugin;
mod run;
mod ui;
mod widget;

use plugin::TuiPlugin;

/// Starts the Nushell plugin protocol loop.
fn main() {
    serve_plugin(&TuiPlugin, MsgPackSerializer);
}
