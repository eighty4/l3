use crossterm::terminal;
use std::io::{stderr, Write};
use std::process;

pub fn err_exit(msg: &str) -> ! {
    stderr()
        .write_all(format!("\nfatal: {msg}").as_bytes())
        .unwrap();
    exit(1);
}

pub fn exit(code: i32) -> ! {
    terminal::disable_raw_mode().expect("failed disabling raw mode");
    process::exit(code);
}
