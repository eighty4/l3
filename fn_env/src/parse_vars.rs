use std::collections::HashMap;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum EnvVarsParseError {
    #[error("{0}")]
    SyntaxError(String),
}

// todo correct parsing .env file shell script syntax
pub fn parse_env_file_content(
    env_file_content: &str,
) -> Result<HashMap<String, String>, EnvVarsParseError> {
    let mut vars: HashMap<String, String> = HashMap::new();
    for line in env_file_content.lines() {
        let trimmed_comment = match line.find('#') {
            None => line,
            Some(i) => &line[0..i],
        }
        .trim();
        if trimmed_comment.is_empty() {
            continue;
        }
        let mut parts = trimmed_comment.splitn(2, '=');
        let key = parts.next().unwrap().trim().to_string();
        if key.is_empty() {
            return Err(EnvVarsParseError::SyntaxError(
                "env var does not have a name".into(),
            ));
        }
        let val = match parts.next() {
            None => {
                return Err(EnvVarsParseError::SyntaxError(
                    "env var is incomplete".into(),
                ))
            }
            Some(val) => val.trim().to_string(),
        };
        vars.insert(
            key,
            if val.starts_with('\'') {
                val.trim_matches('\'').into()
            } else if val.starts_with('\"') {
                val.trim_matches('\"').into()
            } else {
                val
            },
        );
    }
    Ok(vars)
}
