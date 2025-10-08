mod ecmascript;
mod python;
pub mod swc;

#[cfg(test)]
mod ecmascript_test;
#[cfg(test)]
mod python_test;

use std::{io, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum AstParseError {
    #[error("unsupported file type")]
    InvalidFileType,
    #[error("io error")]
    IO(#[from] io::Error),
    #[error("syntax error")]
    Syntax,
}

pub type AstParseResult<T> = Result<T, AstParseError>;

pub fn collect_exported_fns(path: &Path) -> AstParseResult<Vec<String>> {
    debug_assert!(path.is_absolute());
    debug_assert!(path.is_file());
    match path.extension().and_then(|ext| ext.to_str()) {
        None => Err(AstParseError::InvalidFileType),
        Some(ext) => match ext {
            "js" | "mjs" | "ts" => Ok(ecmascript::collect_exported_fns(path)?),
            "py" => Ok(python::collect_exported_fns(path)?),
            _ => Err(AstParseError::InvalidFileType),
        },
    }
}
