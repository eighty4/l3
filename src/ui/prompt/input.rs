use crate::ui::prompt::terminal::match_prompt_result;
use crossterm::style::Stylize;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::QueueableCommand;
use std::io;
use std::io::{stdin, Stdout, Write};

pub struct InputPromptConfig<'a> {
    pub _autocomplete_value: Option<&'a str>,
    pub help_text: Option<&'a str>,
    pub line_padding: &'a str,
    pub prompt_text: &'a str,
    pub validation: Option<fn(&str) -> ValidationResult>,
}

pub enum ValidationResult {
    Valid,
    Invalid(String),
}

pub fn prompt_for_input(config: InputPromptConfig) -> String {
    match_prompt_result(input_inner(config))
}

fn input_inner(config: InputPromptConfig) -> Result<String, anyhow::Error> {
    let mut stdout = io::stdout();
    loop {
        let input = prompt_and_read_line(&mut stdout, &config)?;
        let validation_result = match &config.validation {
            None => {
                if input.is_empty() {
                    ValidationResult::Valid
                } else {
                    ValidationResult::Invalid("Value is required.".to_string())
                }
            }
            Some(validation) => validation(&input),
        };
        match validation_result {
            ValidationResult::Valid => {
                rewrite_result_indicator(&mut stdout, config.line_padding, true)?;
                return Ok(input);
            }
            ValidationResult::Invalid(msg) => {
                rewrite_result_indicator(&mut stdout, config.line_padding, false)?;
                println!("\n{}{} {msg}\n", config.line_padding, "!".red());
            }
        }
    }
}

fn prompt_and_read_line(
    stdout: &mut Stdout,
    config: &InputPromptConfig,
) -> Result<String, anyhow::Error> {
    enable_raw_mode()?;
    let prompt = match config.help_text {
        None => format!("{}{}\n", &config.line_padding, &config.prompt_text),
        Some(help_text) => format!(
            "{}{}   {}\n",
            &config.line_padding,
            &config.prompt_text,
            format!("({help_text})").grey(),
        ),
    };
    stdout.write_all(prompt.as_bytes())?;
    stdout.queue(crossterm::cursor::MoveToColumn(0))?;
    stdout.write_all(format!("{}> ", &config.line_padding).as_bytes())?;
    stdout.flush()?;

    disable_raw_mode()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn rewrite_result_indicator(
    stdout: &mut Stdout,
    line_padding: &str,
    valid: bool,
) -> Result<(), anyhow::Error> {
    enable_raw_mode()?;
    stdout.queue(crossterm::cursor::MoveToPreviousLine(1))?;
    let x = line_padding.len();
    stdout.queue(crossterm::cursor::MoveToColumn(x as u16))?;
    stdout.write_all(format!("{}", if valid { "✔".green() } else { ">".red() }).as_bytes())?;
    stdout.queue(crossterm::cursor::MoveToNextLine(1))?;
    disable_raw_mode()?;
    Ok(())
}
