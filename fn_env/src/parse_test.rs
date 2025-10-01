use std::{collections::HashMap, fs};

use temp_dir::TempDir;

use crate::{parse_env_file, parse_env_file_content, EnvFileParseError, EnvVarsParseError};

#[test]
fn test_parse_env_vars_accepts_a_happy_path_env_var() {
    assert_eq!(
        parse_env_file_content("GREETING=goodbye\n"),
        Ok(HashMap::from([("GREETING".into(), "goodbye".into())]))
    );
}

#[test]
fn test_parse_env_vars_accepts_empty_value() {
    assert_eq!(
        parse_env_file_content("GREETING=\n"),
        Ok(HashMap::from([("GREETING".into(), "".into())]))
    );
}

#[test]
fn test_parse_env_vars_trims_quotes() {
    assert_eq!(
        parse_env_file_content(r#"GREETING=' sup '"#),
        Ok(HashMap::from([("GREETING".into(), " sup ".into())]))
    );
    assert_eq!(
        parse_env_file_content(r#"GREETING=" sup sup ""#),
        Ok(HashMap::from([("GREETING".into(), " sup sup ".into())]))
    );
}

#[test]
fn test_parse_env_vars_accepts_multiple_equals() {
    assert_eq!(
        parse_env_file_content("GREETING=yo=yo"),
        Ok(HashMap::from([("GREETING".into(), "yo=yo".into())]))
    );
}

#[test]
fn test_parse_env_vars_cuts_empty_lines() {
    assert_eq!(parse_env_file_content("\n\n"), Ok(HashMap::new()));
}

#[test]
fn test_parse_env_vars_trims_line_comment() {
    assert_eq!(
        parse_env_file_content("GREETING=yo\n# comment\n"),
        Ok(HashMap::from([("GREETING".into(), "yo".into())]))
    );
}

#[test]
fn test_parse_env_vars_trims_line_comment_midline() {
    assert_eq!(
        parse_env_file_content("GREETING=yo # comment\n"),
        Ok(HashMap::from([("GREETING".into(), "yo".into())]))
    );
}

#[test]
fn test_parse_env_vars_trims_key_and_val() {
    assert_eq!(
        parse_env_file_content(" GREETING = yo "),
        Ok(HashMap::from([("GREETING".into(), "yo".into())]))
    );
}

#[test]
fn test_parse_env_vars_returns_syntax_errors() {
    let tests = [
        ("\n=\n", "env var does not have a name"),
        ("GREETING", "env var is incomplete"),
    ];
    for (env_file_content, cause) in tests {
        assert_eq!(
            parse_env_file_content(env_file_content),
            Err(EnvVarsParseError::SyntaxError(cause.into()))
        );
    }
}

#[test]
fn test_read_env_vars_happy_path() {
    let temp_dir = TempDir::new().unwrap();
    let p = temp_dir.child(".env");
    fs::write(&p, "GREETING=goodbye").unwrap();
    assert_eq!(
        parse_env_file(&p),
        Ok(HashMap::from([("GREETING".into(), "goodbye".into())]))
    );
}

#[test]
fn test_read_env_vars_io_error() {
    let temp_dir = TempDir::new().unwrap();
    let p = temp_dir.child(".env");
    assert_eq!(
        parse_env_file(&p),
        Err(EnvFileParseError::IoError {
            file_name: ".env".into(),
            kind: std::io::ErrorKind::NotFound,
        })
    );
}
