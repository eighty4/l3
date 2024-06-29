use std::io;
use std::io::{stderr, Write};
use std::process;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, ClearType};
use crossterm::{event, QueueableCommand};

pub fn confirm(prompt: &str) -> Result<bool, anyhow::Error> {
    let mut stdout = io::stdout();
    stdout.write_all(format!("{} ", prompt).as_bytes())?;
    stdout.flush()?;
    enable_raw_mode()?;
    match event::read()? {
        Event::Key(KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('c'),
            ..
        }) => exit(0),
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        }) => {
            let confirmed = c == 'y' || c == 'Y';
            if confirmed {
                stdout.queue(crossterm::terminal::Clear(ClearType::CurrentLine))?;
            } else {
                stdout.write_all(format!("{c}\n").as_bytes())?;
            }
            stdout.queue(crossterm::cursor::MoveToColumn(0))?;
            stdout.flush()?;
            disable_raw_mode()?;
            Ok(confirmed)
        }
        _ => err_exit("when the fuck does this happen!"),
    }
}

fn err_exit(msg: &str) -> ! {
    stderr()
        .write_all(format!("\nfatal: {msg}").as_bytes())
        .unwrap();
    exit(1);
}

fn exit(code: i32) -> ! {
    disable_raw_mode().expect("failed disabling raw mode");
    process::exit(code);
}
