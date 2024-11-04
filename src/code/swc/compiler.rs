use anyhow::anyhow;
use std::sync::Arc;
use swc::{try_with_handler, Compiler};
use swc_common::errors::Handler;
use swc_common::{SourceMap, GLOBALS};

#[allow(unused)]
pub fn with_swc_compiler_without_diagnostics<F, R>(f: F) -> Result<R, anyhow::Error>
where
    F: FnOnce(&Compiler, &Handler, Arc<SourceMap>) -> Result<R, anyhow::Error>,
{
    let source_map = Arc::<SourceMap>::default();
    let compiler = Compiler::new(source_map.clone());
    GLOBALS
        .set(&Default::default(), || {
            try_with_handler(source_map.clone(), Default::default(), |handler| {
                f(&compiler, handler, source_map)
            })
        })
        .map_err(|err| anyhow!("swc compiler error: {}", err.to_string()))
}
