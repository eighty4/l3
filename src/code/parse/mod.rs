use anyhow::anyhow;

use crate::code::runtime::SourcesRuntimeDeets;
use crate::code::source::path::SourcePath;
use crate::code::source::{Language, SourceFile};

mod swc;

#[cfg(test)]
mod parse_test;
#[cfg(test)]
mod swc_test;

pub fn parse_source_file(
    path: SourcePath,
    project_details: &SourcesRuntimeDeets,
) -> Result<SourceFile, anyhow::Error> {
    debug_assert!(path.rel.extension().is_some());
    debug_assert!(path.abs.is_file());
    let language = match path.language() {
        None => return Err(anyhow!("{} is not a supported file type", path.file_name())),
        Some(language) => language,
    };
    match language {
        Language::JavaScript | Language::TypeScript => {
            swc::parse_source_file(language, path, project_details)
        }
        Language::Python => panic!(),
    }
}
