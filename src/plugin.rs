use nu_plugin::{Plugin, PluginCommand};

use crate::{
  run::Tui,
  widget::{TuiWidget, WidgetKind},
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
    let mut commands: Vec<Box<dyn PluginCommand<Plugin = Self>>> = vec![Box::new(Tui)];
    commands.extend(
      WidgetKind::ALL
        .map(TuiWidget::new)
        .map(|command| Box::new(command) as Box<dyn PluginCommand<Plugin = Self>>),
    );
    commands
  }
}

#[cfg(test)]
mod tests {
  use nu_plugin::Plugin;

  use super::TuiPlugin;

  /// Verifies that the application runs at the namespace root without a run subcommand.
  #[test]
  fn exports_root_application_command() {
    let names = TuiPlugin
      .commands()
      .into_iter()
      .map(|command| command.name().to_owned())
      .collect::<Vec<_>>();

    assert!(names.iter().any(|name| name == "tui"));
    assert!(!names.iter().any(|name| name == "tui run"));
  }
}
