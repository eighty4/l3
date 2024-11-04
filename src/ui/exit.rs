use crossterm::style::Stylize;
use crossterm::terminal;
use std::io::{stderr, Write};
use std::process;

/// Unsupported application state or an error thrown up to CLI
pub fn cmd_err_exit(msg: &str) -> ! {
    err_exit_inner("error", msg);
}

/// Unrecoverable and unexpected errors so we print and exit
pub fn err_exit(msg: &str) -> ! {
    err_exit_inner("fatal", msg);
}

pub fn clean_exit() -> ! {
    exit(0);
}

fn exit(code: i32) -> ! {
    terminal::disable_raw_mode().expect("failed disabling raw mode");
    process::exit(code);
}

fn err_exit_inner(label: &str, msg: &str) -> ! {
    stderr()
        .write_all(format!("{}{} {msg}", label.red(), ":".red()).as_bytes())
        .unwrap();
    exit(1);
}
