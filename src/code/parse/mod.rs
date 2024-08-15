use crate::code::source::path::SourcePath;
use crate::code::source::SourceFile;

pub(crate) mod imports;
pub(crate) mod swc;

#[cfg(test)]
mod swc_test;

pub trait SourceParser: Send + Sync {
    fn parse(&self, path: SourcePath) -> Result<SourceFile, anyhow::Error>;
}
