use std::{collections::HashMap, fs, io, path::Path};

use crate::parse_vars::parse_env_file_content;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum EnvFileParseError {
    #[error("io error reading {file_name} env file: {kind}")]
    IoError {
        file_name: String,
        kind: std::io::ErrorKind,
    },
    #[error("error parsing env file {file_name}: {cause}")]
    SyntaxError { cause: String, file_name: String },
}

pub fn parse_env_file(p: &Path) -> Result<HashMap<String, String>, EnvFileParseError> {
    debug_assert!(p.is_absolute());
    let file_name = p
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or("UNNAMED ENV FILE".into());
    match fs::read_to_string(p) {
        Ok(content) => {
            parse_env_file_content(content.as_str()).map_err(|err| EnvFileParseError::SyntaxError {
                cause: err.to_string(),
                file_name,
            })
        }
        Err(io_err) => {
            debug_assert!(io_err.kind() == io::ErrorKind::NotFound);
            Err(EnvFileParseError::IoError {
                file_name,
                kind: io_err.kind(),
            })
        }
    }
}
