//! `Source/utils/console.cpp`: text output to the console the game was started from.
//! The Windows version attaches to the parent console's stderr; the port's console-subsystem
//! executable writes to stderr directly.

use std::io::Write;

/// Original: `devilution::printInConsole` (utils/console.cpp).
// @port utils/console.cpp|devilution::printInConsole(string_view str) sha=f8087b9bb81b
pub fn print_in_console(s: &str) {
    let _ = std::io::stderr().write_all(s.as_bytes());
}

/// Original: `devilution::printNewlineInConsole` (utils/console.cpp).
// @port utils/console.cpp|devilution::printNewlineInConsole() sha=cc6a4a831642
pub fn print_newline_in_console() {
    let _ = std::io::stderr().write_all(b"\r\n");
}
