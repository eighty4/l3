use std::io;
use std::io::Write;

use crossterm::{event, terminal, QueueableCommand};

use crate::ui::exit::{err_exit, exit};

pub fn confirm(prompt: &str) -> Result<bool, anyhow::Error> {
    let mut stdout = io::stdout();
    stdout.write_all(format!("{} ", prompt).as_bytes())?;
    stdout.flush()?;
    terminal::enable_raw_mode()?;
    match event::read()? {
        event::Event::Key(event::KeyEvent {
            modifiers: event::KeyModifiers::CONTROL,
            code: event::KeyCode::Char('c'),
            ..
        }) => exit(0),
        event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Char(c),
            ..
        }) => {
            let confirmed = c == 'y' || c == 'Y';
            if confirmed {
                stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
            } else {
                stdout.write_all(format!("{c}\n").as_bytes())?;
            }
            stdout.queue(crossterm::cursor::MoveToColumn(0))?;
            stdout.flush()?;
            terminal::disable_raw_mode()?;
            Ok(confirmed)
        }
        _ => err_exit("when the fuck does this happen!"),
    }
}
