use std::path::{Path, PathBuf};

use crate::code::project::javascript::JavaScriptDeets;
use crate::code::project::python::PythonDeets;
use crate::code::project::typescript::TypeScriptDeets;

pub(crate) mod javascript;
pub(crate) mod python;
pub(crate) mod typescript;

#[cfg(test)]
mod javascript_test;

#[cfg(test)]
mod typescript_test;

#[derive(Clone, Default)]
pub struct ProjectDetails {
    #[allow(unused)]
    pub javascript: JavaScriptDeets,
    pub dir: PathBuf,
    pub name: String,
    #[allow(unused)]
    pub python: PythonDeets,
    #[allow(unused)]
    pub typescript: TypeScriptDeets,
}

impl ProjectDetails {
    pub fn read_details(dir: &Path, name: String) -> Result<ProjectDetails, anyhow::Error> {
        Ok(Self {
            javascript: JavaScriptDeets::read_details(dir)?,
            dir: dir.to_path_buf(),
            name,
            python: PythonDeets::read_details()?,
            typescript: TypeScriptDeets::read_details(dir)?,
        })
    }
}
