use nu_plugin::{Plugin, PluginCommand};

use crate::{
    example::{TuiExample, WidgetExample},
    run::TuiRun,
};

/// Nushell plugin exposing Ratatui commands.
pub struct TuiPlugin;

impl Plugin for TuiPlugin {
    /// Returns the plugin package version.
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    /// Returns the commands exported by this plugin.
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        let mut commands: Vec<Box<dyn PluginCommand<Plugin = Self>>> = vec![Box::new(TuiRun)];
        commands.extend(
            WidgetExample::ALL
                .map(TuiExample::new)
                .map(|command| Box::new(command) as Box<dyn PluginCommand<Plugin = Self>>),
        );
        commands
    }
}
