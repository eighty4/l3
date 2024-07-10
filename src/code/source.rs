use std::path::{Path, PathBuf};

// todo PartialEq is only used for a debug_assert
#[derive(PartialEq)]
pub enum Language {
    JavaScript,
    TypeScript,
    Python,
}

impl Language {
    pub fn from_extension(path: &Path) -> Option<Language> {
        match path.extension().unwrap().to_string_lossy().as_ref() {
            "js" | "mjs" => Some(Language::JavaScript),
            "py" => Some(Language::Python),
            "ts" => Some(Language::TypeScript),
            &_ => None,
        }
    }
}

pub struct SourceFile {
    pub exported_fns: Vec<String>,
    #[allow(unused)]
    pub imports: Vec<PathBuf>,
    #[allow(unused)]
    pub language: Language,
    pub path: PathBuf,
}
