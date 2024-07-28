use std::path::PathBuf;

use anyhow::anyhow;
use notify::event::{DataChange, ModifyKind, RenameMode};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::Sender;

// todo FileUpdate as struct with FileUpdateKind so enum doesn't have to be matched to access path
pub enum FileUpdate {
    ContentChanged(PathBuf),
    FileCreated(PathBuf),
    FileRemoved(PathBuf),
    FileRenamed(PathBuf),
}

pub struct FileWatcher {
    paths: Vec<PathBuf>,
    w: RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(s: Sender<FileUpdate>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            paths: Vec::new(),
            w: RecommendedWatcher::new(
                move |result: notify::Result<Event>| match result {
                    Ok(event) => {
                        let path = event.paths.first().cloned().unwrap();
                        let maybe_update = match event.kind {
                            EventKind::Create(_) => Some(FileUpdate::FileCreated(path)),
                            EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                                Some(FileUpdate::ContentChanged(path))
                            }
                            EventKind::Modify(ModifyKind::Name(RenameMode::Any)) => {
                                Some(FileUpdate::FileRenamed(path))
                            }
                            EventKind::Modify(ModifyKind::Name(rename_mode)) => {
                                panic!("unexpected EventKind::Modify(ModifyKind::Name(RenameMode::{:?}))", rename_mode);
                            }
                            EventKind::Remove(_) => Some(FileUpdate::FileRemoved(path)),
                            _ => None,
                        };
                        match maybe_update {
                            None => println!("{:?} {:?}", event.kind, event.paths),
                            Some(update) => _ = s.try_send(update),
                        };
                    }
                    Err(e) => eprintln!("{e:?}"),
                },
                Default::default(),
            )?,
        })
    }

    pub fn for_file(s: Sender<FileUpdate>, p: PathBuf) -> Result<Self, anyhow::Error> {
        let mut w = Self::new(s)?;
        w.add_path(p)?;
        Ok(w)
    }

    pub fn add_path(&mut self, path: PathBuf) -> Result<(), anyhow::Error> {
        if !path.exists() {
            return Err(anyhow!(
                "cannot watch {} does not exist",
                path.to_string_lossy()
            ));
        }
        self.w.watch(path.as_path(), RecursiveMode::Recursive)?;
        self.paths.push(path);
        Ok(())
    }

    #[allow(unused)]
    pub fn rm_path(&mut self, path: PathBuf) -> Result<(), anyhow::Error> {
        self.w.unwatch(path.as_path())?;
        self.paths
            .iter()
            .position(|p| p == &path)
            .map(|i| self.paths.remove(i));
        Ok(())
    }
}
