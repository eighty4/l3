use rustpython_parser::ast::Stmt;
use rustpython_parser::{ast, Parse, ParseError};
use std::{fs, path::Path};

use crate::{AstParseError, AstParseResult};

pub fn collect_exported_fns(path: &Path) -> AstParseResult<Vec<String>> {
    let python_code = fs::read_to_string(path)?;
    let ast = ast::Suite::parse(&python_code, &path.to_string_lossy())?;
    let mut handlers: Vec<String> = Vec::new();
    for stmt in ast {
        match stmt {
            Stmt::FunctionDef(function) => handlers.push(function.name.to_string()),
            Stmt::AsyncFunctionDef(_) => todo!("to support python async functions as handlers, build_python_fn will have to generate code for a non async handler that launches the async python function with the python async runtime"),
            _ => {}
        }
    }
    Ok(handlers)
}

impl From<ParseError> for AstParseError {
    fn from(err: ParseError) -> Self {
        // todo map rustpython_parser::ParseError diagnostics to a public API type
        dbg!(err);
        AstParseError::Syntax
    }
}
