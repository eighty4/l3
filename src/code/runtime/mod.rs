use std::path::Path;

use crate::code::runtime::javascript::JavaScriptDeets;
use crate::code::runtime::python::PythonDeets;
use crate::code::runtime::typescript::TypeScriptDeets;

pub(crate) mod javascript;
pub(crate) mod python;
pub(crate) mod typescript;

#[cfg(test)]
mod javascript_test;

#[cfg(test)]
mod typescript_test;

#[derive(Clone, Default)]
pub struct SourcesRuntimeDeets {
    #[allow(unused)]
    pub javascript: JavaScriptDeets,
    #[allow(unused)]
    pub python: PythonDeets,
    #[allow(unused)]
    pub typescript: TypeScriptDeets,
}

impl SourcesRuntimeDeets {
    pub fn read_details(dir: &Path) -> Result<SourcesRuntimeDeets, anyhow::Error> {
        Ok(Self {
            javascript: JavaScriptDeets::read_details(dir)?,
            python: PythonDeets::read_details()?,
            typescript: TypeScriptDeets::read_details(dir)?,
        })
    }
}
