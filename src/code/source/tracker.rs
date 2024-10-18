use crate::code::runtime::RuntimeConfigApi;
use crate::code::source::tree::SourcesApi;
use crate::code::source::watcher::{FileUpdate, FileUpdateKind, FileWatcher, SpecialFile};
use crate::project::Lx3Project;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::{interval, Interval};

struct SourceTrackerEventLoop {
    file_rx: Receiver<FileUpdate>,
    _file_watcher: Arc<Mutex<FileWatcher>>,
    /// Interval used to debounce FileWatcher updates
    update_interval: Interval,
    _project: Arc<Lx3Project>,
    runtime_config_api: Arc<RuntimeConfigApi>,
    _sources_api: Arc<SourcesApi>,
    /// FileUpdate events queued until the next syncing interval
    update_queue: Vec<FileUpdate>,
}

impl SourceTrackerEventLoop {
    fn new(
        file_rx: Receiver<FileUpdate>,
        _file_watcher: Arc<Mutex<FileWatcher>>,
        _project: Arc<Lx3Project>,
        runtime_config_api: Arc<RuntimeConfigApi>,
        _sources_api: Arc<SourcesApi>,
    ) -> Self {
        Self {
            file_rx,
            _file_watcher,
            update_interval: interval(Duration::from_secs(1)),
            _project,
            runtime_config_api,
            _sources_api,
            update_queue: Vec::new(),
        }
    }

    async fn start(&mut self) {
        loop {
            select! {
                opt = self.file_rx.recv() => self.update_queue.push(opt.unwrap()),
                _ = self.update_interval.tick() => self.sync_file_updates(),
            }
        }
    }

    fn sync_file_updates(&mut self) {
        if !self.update_queue.is_empty() {
            for file_update in &self.update_queue {
                match &file_update.file {
                    None => match &file_update.kind {
                        FileUpdateKind::ContentChanged => {
                            println!("{} content changed", file_update.path.to_string_lossy())
                        }
                        FileUpdateKind::FileCreated => {
                            println!("{} file created", file_update.path.to_string_lossy())
                        }
                        FileUpdateKind::FileRemoved => {
                            println!("{} file removed", file_update.path.to_string_lossy())
                        }
                        FileUpdateKind::FileRenamed => {
                            println!("{} file renamed", file_update.path.to_string_lossy())
                        }
                    },
                    Some(file) => match file {
                        SpecialFile::PackageJson => self.runtime_config_api.refresh_node_config(),
                        SpecialFile::TypeScriptConfig => {
                            self.runtime_config_api.refresh_typescript_config()
                        }
                    },
                }
            }
        }
    }
}

pub struct SourceTracker {
    _file_watcher: Arc<Mutex<FileWatcher>>,
    _sources_api: Arc<SourcesApi>,
}

impl SourceTracker {
    pub fn new(
        project: Arc<Lx3Project>,
        runtime_config_api: Arc<RuntimeConfigApi>,
        sources_api: Arc<SourcesApi>,
    ) -> Self {
        let (tx, rx) = channel(10);
        let file_watcher = Arc::new(Mutex::new(FileWatcher::new(project.dir.clone(), tx)));
        let mut event_loop = SourceTrackerEventLoop::new(
            rx,
            file_watcher.clone(),
            project,
            runtime_config_api,
            sources_api.clone(),
        );
        tokio::spawn(async move { event_loop.start().await });
        {
            let mut file_watcher = file_watcher.lock().unwrap();
            file_watcher
                .add_non_recursive(PathBuf::from("."))
                .expect("wtf");
            // file_watcher.add_recursive(PathBuf::from("routes")).expect("wtf");
        }
        Self {
            _file_watcher: file_watcher,
            _sources_api: sources_api,
        }
    }
}
