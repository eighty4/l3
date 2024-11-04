use crate::ui::exit::clean_exit;
use crate::ui::prompt::terminal::match_prompt_result;
use crossterm::terminal::enable_raw_mode;
use crossterm::{event, terminal, QueueableCommand};
use std::io;
use std::io::Write;

pub fn prompt_for_confirmation(prompt: &str) -> bool {
    enable_raw_mode().unwrap();
    match_prompt_result(confirm_inner(prompt))
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
            }) => clean_exit(),
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
