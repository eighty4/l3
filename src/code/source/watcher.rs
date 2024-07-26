use std::path::PathBuf;
use std::time::Duration;

use anyhow::anyhow;
use notify::event::{DataChange, ModifyKind};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use tokio::sync::mpsc::Sender;

pub enum FileUpdate {
    ContentChanged(PathBuf),
    FileCreated(PathBuf),
    FileRemoved(PathBuf),
}

pub struct FileWatcher {
    w: Debouncer<RecommendedWatcher, FileIdMap>,
}

impl FileWatcher {
    pub fn new(s: Sender<FileUpdate>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            w: new_debouncer(
                Duration::from_secs(1),
                None,
                move |result: DebounceEventResult| match result {
                    Ok(events) => {
                        for event in events {
                            let path = event.paths.first().cloned().unwrap();
                            if path.is_dir() {
                                continue;
                            }
                            let maybe_update = match event.event.kind {
                                EventKind::Create(_) => Some(FileUpdate::FileCreated(path)),
                                EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                                    Some(FileUpdate::ContentChanged(path))
                                }
                                EventKind::Remove(_) => Some(FileUpdate::FileRemoved(path)),
                                _ => None,
                            };
                            match maybe_update {
                                None => println!("{:?} {:?}", event.event.kind, event.event.paths),
                                Some(update) => _ = s.try_send(update),
                            };
                        }
                    }
                    Err(errors) => errors.iter().for_each(|e| eprintln!("{e:?}")),
                },
            )?,
        })
    }

    pub fn add_path(&mut self, path: PathBuf) -> Result<(), anyhow::Error> {
        if !path.exists() {
            return Err(anyhow!(
                "cannot watch {} does not exist",
                path.to_string_lossy()
            ));
        }
        self.w
            .watcher()
            .watch(path.as_path(), RecursiveMode::Recursive)?;
        // todo research behavior diff bw cache path and root
        self.w
            .cache()
            .add_root(path.as_path(), RecursiveMode::Recursive);
        Ok(())
    }

    #[allow(unused)]
    pub fn rm_path(&mut self, path: PathBuf) -> Result<(), anyhow::Error> {
        self.w.watcher().unwatch(path.as_path())?;
        self.w.cache().remove_root(path.as_path());
        Ok(())
    }
}
