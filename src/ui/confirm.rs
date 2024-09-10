use crossterm::{event, terminal, QueueableCommand};
use std::io;
use std::io::Write;

use crate::ui::exit::exit;

pub fn confirm(prompt: &str) -> bool {
    terminal::enable_raw_mode().unwrap();
    match confirm_inner(prompt) {
        Ok(confirm) => {
            terminal::disable_raw_mode().unwrap();
            confirm
        }
        Err(err) => {
            terminal::disable_raw_mode().unwrap();
            panic!("{}", err);
        }
    }
}

fn confirm_inner(prompt: &str) -> Result<bool, anyhow::Error> {
    let mut stdout = io::stdout();
    stdout.write_all(format!("{} ", prompt).as_bytes())?;
    stdout.flush()?;
    let confirmed;
    loop {
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
                confirmed = c == 'y' || c == 'Y';
                break;
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Enter,
                ..
            }) => {
                confirmed = true;
                break;
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Esc,
                ..
            }) => {
                confirmed = false;
                break;
            }
            _ => {}
        }
    }
    if confirmed {
        stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
    }
    stdout.queue(crossterm::cursor::MoveToColumn(0))?;
    stdout.flush()?;
    Ok(confirmed)
}
