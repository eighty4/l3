use crate::code::checksum::ChecksumTree;
use crate::code::source::SourceFile;
use crate::lambda::{LambdaFn, RouteKey};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

pub enum SourceUpdate {
    #[allow(unused)]
    UpdateLambdaCode,
    #[allow(unused)]
    UpdateLambdaEnv,
    #[allow(unused)]
    UpdateLambdaDeps,
    #[allow(unused)]
    AddLambda,
    #[allow(unused)]
    RemoveLambda,
}

pub struct SourceTracker {
    #[allow(unused)]
    checksums: HashMap<RouteKey, ChecksumTree>,
    /// Map of SourceFile for Lambda Functions by RouteKey
    #[allow(unused)]
    lambdas: HashMap<RouteKey, LambdaFn>,
    #[allow(unused)]
    project_dir: PathBuf,
    /// Map of SourceFile by relative PathBuf
    #[allow(unused)]
    sources: HashMap<PathBuf, SourceFile>,
    #[allow(unused)]
    update_sender: Sender<SourceUpdate>,
}

impl SourceTracker {
    pub fn new(project_dir: PathBuf, update_sender: Sender<SourceUpdate>) -> Self {
        debug_assert!(project_dir.is_dir());
        Self {
            checksums: HashMap::new(),
            lambdas: HashMap::new(),
            project_dir,
            sources: HashMap::new(),
            update_sender,
        }
    }

    pub fn content_changed(&mut self, path: PathBuf) {
        debug_assert!(path.is_absolute());
        println!(
            "content changed {}",
            path.strip_prefix(&self.project_dir)
                .unwrap()
                .to_string_lossy()
        );
    }

    pub fn file_created(&mut self, path: PathBuf) {
        debug_assert!(path.is_absolute());
        println!(
            "file created {}",
            path.strip_prefix(&self.project_dir)
                .unwrap()
                .to_string_lossy()
        );
    }

    pub fn file_removed(&mut self, path: PathBuf) {
        debug_assert!(path.is_absolute());
        println!(
            "file removed {}",
            path.strip_prefix(&self.project_dir)
                .unwrap()
                .to_string_lossy()
        );
    }
}
