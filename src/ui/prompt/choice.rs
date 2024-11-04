use crate::ui::exit::clean_exit;
use crate::ui::prompt::terminal::{clear_lines, match_prompt_result};
use crossterm::style::{StyledContent, Stylize};
use crossterm::terminal::enable_raw_mode;
use crossterm::{event, QueueableCommand};
use std::io;
use std::io::{Stdout, Write};

const CURSOR_OFF: &str = "○";
const CURSOR_ON: &str = "●";

pub fn prompt_for_choice(line_prefix: &str, prompt: &str, choices: Vec<String>) -> String {
    debug_assert!(choices.len() < 4);
    enable_raw_mode().unwrap();
    match_prompt_result(choice_inner(line_prefix, prompt, choices))
}

fn choice_inner(
    line_prefix: &str,
    prompt: &str,
    choices: Vec<String>,
) -> Result<String, anyhow::Error> {
    let mut stdout = io::stdout();
    stdout.write_all(format!("{line_prefix}{}\n", prompt).as_bytes())?;
    stdout.queue(crossterm::cursor::MoveToColumn(0))?;
    stdout.flush()?;
    let mut cursor = 0;
    write_choices(&mut stdout, cursor, &choices, line_prefix)?;
    loop {
        match event::read()? {
            event::Event::Key(event::KeyEvent {
                modifiers: event::KeyModifiers::CONTROL,
                code: event::KeyCode::Char('c'),
                ..
            })
            | event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Esc,
                ..
            }) => clean_exit(),
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Up,
                ..
            }) => {
                if cursor > 0 {
                    cursor -= 1;
                    rewrite_choices(&mut stdout, cursor, &choices, line_prefix)?;
                }
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Down,
                ..
            }) => {
                if cursor < choices.len() - 1 {
                    cursor += 1;
                    rewrite_choices(&mut stdout, cursor, &choices, line_prefix)?;
                }
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Enter,
                ..
            }) => {
                rewrite_choices(&mut stdout, cursor, &choices, line_prefix)?;
                break;
            }
            _ => {}
        }
    }
    let choice = choices.get(cursor).cloned().unwrap();
    clear_lines(&mut stdout, choices.len())?;
    write_choice(&mut stdout, "✔".green(), &choice, line_prefix)?;
    stdout.flush()?;
    Ok(choice)
}

fn write_choice(
    stdout: &mut Stdout,
    cursor: StyledContent<&str>,
    choice: &str,
    line_prefix: &str,
) -> Result<(), anyhow::Error> {
    let _ = stdout.write(format!("{line_prefix}{cursor} {choice}\n").as_bytes())?;
    stdout.queue(crossterm::cursor::MoveToColumn(0))?;
    Ok(())
}

fn write_choices(
    stdout: &mut Stdout,
    cursor: usize,
    choices: &[String],
    line_prefix: &str,
) -> Result<(), anyhow::Error> {
    for (n, choice) in choices.iter().enumerate() {
        write_choice(
            stdout,
            if n == cursor {
                CURSOR_ON.stylize()
            } else {
                CURSOR_OFF.stylize()
            },
            choice,
            line_prefix,
        )?;
    }
    stdout.flush()?;
    Ok(())
}

fn rewrite_choices(
    stdout: &mut Stdout,
    cursor: usize,
    choices: &[String],
    line_prefix: &str,
) -> Result<(), anyhow::Error> {
    clear_lines(stdout, choices.len())?;
    write_choices(stdout, cursor, choices, line_prefix)
}
