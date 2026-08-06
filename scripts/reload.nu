cargo build --locked
plugin rm ratatui
plugin add ($env.FILE_PWD)/../target/debug/nu_plugin_ratatui
plugin use ratatui
