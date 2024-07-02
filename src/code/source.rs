use std::path::PathBuf;

pub enum Language {
    JavaScript,
    #[allow(unused)]
    TypeScript,
    #[allow(unused)]
    Python,
}

pub struct SourceFile {
    pub exported_fns: Vec<String>,
    #[allow(unused)]
    pub imports: Vec<PathBuf>,
    #[allow(unused)]
    pub language: Language,
    pub path: PathBuf,
}
