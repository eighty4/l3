use std::path::Path;

use anyhow::anyhow;

use crate::code::source::SourceFile;

pub(crate) mod es_module;

#[cfg(test)]
mod es_module_test;
#[cfg(test)]
mod parse_test;
#[cfg(test)]
mod ts_module_test;

pub fn parse_source_file(path: &Path, project_dir: &Path) -> Result<SourceFile, anyhow::Error> {
    debug_assert!(path.is_relative());
    debug_assert!(path.extension().is_some());
    debug_assert!(project_dir.is_absolute());
    debug_assert!(project_dir.is_dir());
    debug_assert!(project_dir.join(path).is_file());
    let extension = path.extension().unwrap().to_string_lossy().to_string();
    if extension == "js" || extension == "mjs" {
        es_module::parse(path.to_path_buf(), project_dir)
    } else {
        Err(anyhow!(
            "{extension} is not a supported file type for source file {}",
            path.file_name().unwrap().to_string_lossy()
        ))
    }
}
