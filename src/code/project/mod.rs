use std::path::{Path, PathBuf};

use crate::code::project::javascript::JavaScriptDeets;
use crate::code::project::python::PythonDeets;
use crate::code::project::typescript::TypeScriptDeets;

pub(crate) mod javascript;
pub(crate) mod python;
pub(crate) mod typescript;

#[cfg(test)]
mod javascript_test;

#[derive(Clone, Default)]
pub struct ProjectDetails {
    #[allow(unused)]
    pub javascript: JavaScriptDeets,
    pub project_dir: PathBuf,
    #[allow(unused)]
    pub python: PythonDeets,
    #[allow(unused)]
    pub typescript: TypeScriptDeets,
}

impl ProjectDetails {
    pub fn read_details(project_dir: &Path) -> Result<ProjectDetails, anyhow::Error> {
        Ok(Self {
            javascript: JavaScriptDeets::read_details(project_dir)?,
            project_dir: project_dir.to_path_buf(),
            python: PythonDeets::read_details()?,
            typescript: TypeScriptDeets::read_details(project_dir)?,
        })
    }
}
