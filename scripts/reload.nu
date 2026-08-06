cargo build --locked
try { plugin rm ratatui }
plugin add ($env.FILE_PWD)/../target/debug/nu_plugin_ratatui
plugin use ratatui
