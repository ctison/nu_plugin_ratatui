use nu_plugin::{MsgPackSerializer, serve_plugin};

mod config;
mod plugin;
mod run;
mod ui;

use plugin::TuiPlugin;

/// Starts the Nushell plugin protocol loop.
fn main() {
    serve_plugin(&TuiPlugin, MsgPackSerializer);
}
