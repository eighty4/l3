use crossterm::terminal::disable_raw_mode;
use crossterm::{terminal, QueueableCommand};
use std::io::Stdout;

pub fn match_prompt_result<T>(r: Result<T, anyhow::Error>) -> T {
    match r {
        Ok(v) => {
            disable_raw_mode().unwrap();
            v
        }
        Err(e) => {
            let _ = disable_raw_mode();
            panic!("{}", e);
        }
    }
}

pub fn clear_lines(stdout: &mut Stdout, n: usize) -> Result<(), anyhow::Error> {
    stdout.queue(crossterm::cursor::MoveToPreviousLine(n as u16))?;
    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
    Ok(())
}
