use crate::code::source::watcher::SpecialFile::{PackageJson, TypeScriptConfig};
use anyhow::anyhow;
use notify::event::{DataChange, ModifyKind, RenameMode};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

pub struct FileUpdate {
    pub file: Option<SpecialFile>,
    pub kind: FileUpdateKind,
    pub path: PathBuf,
}

pub enum SpecialFile {
    PackageJson,
    TypeScriptConfig,
}

pub enum FileUpdateKind {
    ContentChanged,
    FileCreated,
    FileRemoved,
    FileRenamed,
}

pub struct FileWatcher {
    paths: Vec<PathBuf>,
    project_dir: PathBuf,
    w: RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(project_dir: PathBuf, s: Sender<FileUpdate>) -> Self {
        let package_json_path = project_dir.join("package.json");
        let tsconfig_json_path = project_dir.join("tsconfig.json");
        Self {
            paths: Vec::new(),
            project_dir,
            w: RecommendedWatcher::new(
                move |result: notify::Result<Event>| match result {
                    Ok(event) => {
                        let path = event.paths.first().cloned().unwrap();
                        let maybe_update: Option<FileUpdateKind> = match event.kind {
                            EventKind::Create(_) => Some(FileUpdateKind::FileCreated),
                            EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                                Some(FileUpdateKind::ContentChanged)
                            }
                            EventKind::Modify(ModifyKind::Name(RenameMode::Any)) => {
                                Some(FileUpdateKind::FileRenamed)
                            }
                            EventKind::Modify(ModifyKind::Name(rename_mode)) => {
                                panic!("unexpected EventKind::Modify(ModifyKind::Name(RenameMode::{:?}))", rename_mode);
                            }
                            EventKind::Remove(_) => Some(FileUpdateKind::FileRemoved),
                            _ => None,
                        };
                        match maybe_update {
                            None => println!("{:?} {:?}", event.kind, event.paths),
                            Some(kind) => {
                                let file = match path.file_name() {
                                    None => None,
                                    Some(file_name) => {
                                        if file_name == package_json_path.file_name().unwrap() && path == package_json_path {
                                            Some(PackageJson)
                                        } else if file_name == tsconfig_json_path.file_name().unwrap() && path == tsconfig_json_path {
                                            Some(TypeScriptConfig)
                                        } else {
                                            None
                                        }
                                    }
                                };
                                let _ = s.try_send(FileUpdate { file, kind, path });
                            },
                        };
                    }
                    Err(e) => eprintln!("{e:?}"),
                },
                Default::default(),
            ).expect("startup file watcher"),
        }
    }

    pub fn _add_recursive(&mut self, path: PathBuf) -> Result<(), anyhow::Error> {
        self.add_path(path, RecursiveMode::Recursive)
    }

    pub fn add_non_recursive(&mut self, path: PathBuf) -> Result<(), anyhow::Error> {
        self.add_path(path, RecursiveMode::NonRecursive)
    }

    fn add_path(&mut self, input: PathBuf, mode: RecursiveMode) -> Result<(), anyhow::Error> {
        let path = if input.is_relative() {
            self.project_dir.join(input)
        } else {
            input
        };
        if !path.exists() {
            return Err(anyhow!(
                "cannot watch {} does not exist",
                path.to_string_lossy()
            ));
        }
        self.w.watch(path.as_path(), mode)?;
        self.paths.push(path);
        Ok(())
    }

    #[allow(unused)]
    pub fn rm_path(&mut self, input: PathBuf) -> Result<(), anyhow::Error> {
        let path = if input.is_relative() {
            self.project_dir.join(input)
        } else {
            input
        };
        self.w.unwatch(path.as_path())?;
        self.paths
            .iter()
            .position(|p| p == &path)
            .map(|i| self.paths.remove(i));
        Ok(())
    }
}
