use std::path::Path;
use swc_ecma_visit::FoldWith;

use crate::swc::visitors::CollectExportedFnsVisitor;
use crate::swc::{parse_program_from_fs, CompileError};
use crate::{AstParseError, AstParseResult};

pub fn collect_exported_fns(path: &Path) -> AstParseResult<Vec<String>> {
    let program = parse_program_from_fs(path, None)?;
    let mut collecting = CollectExportedFnsVisitor::new();
    program.fold_with(&mut collecting);
    Ok(collecting.result())
}

impl From<CompileError> for AstParseError {
    fn from(err: CompileError) -> Self {
        match err {
            CompileError::ReadError(err) => AstParseError::IO(err),
            CompileError::CompilerDiagnostics(diagnostics) => {
                dbg!(diagnostics);
                panic!();
                // AstParseError::Syntax
            }
            CompileError::OperationError(err) => {
                dbg!(err);
                panic!("swc operation error");
            }
        }
    }
}
