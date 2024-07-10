use std::path::Path;

use anyhow::anyhow;

use crate::code::source::{Language, SourceFile};

mod swc;

#[cfg(test)]
mod parse_test;
#[cfg(test)]
mod swc_test;

pub fn parse_source_file(path: &Path, project_dir: &Path) -> Result<SourceFile, anyhow::Error> {
    debug_assert!(path.is_relative());
    debug_assert!(path.extension().is_some());
    debug_assert!(project_dir.is_absolute());
    debug_assert!(project_dir.is_dir());
    debug_assert!(project_dir.join(path).is_file());
    let language = match Language::from_extension(path) {
        None => {
            return Err(anyhow!(
                "{} is not a supported file type",
                path.file_name().unwrap().to_string_lossy()
            ))
        }
        Some(language) => language,
    };
    match language {
        Language::JavaScript | Language::TypeScript => {
            swc::parse_source_file(language, path.to_path_buf(), project_dir)
        }
        Language::Python => panic!(),
    }
}
