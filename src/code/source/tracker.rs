use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Interval;

use crate::code::source::tree::SourceTree;
use crate::code::source::update::SourceUpdate;
use crate::code::source::watcher::{FileUpdate, FileWatcher};
use crate::project::Lx3ProjectDeets;

enum SourceTrackerMessage {}

struct SourceTrackerEventLoop {
    fs_interval: Interval,
    fs_rx: Receiver<FileUpdate>,
    fs_tx: Sender<FileUpdate>,
    fs_updates: Vec<FileUpdate>,
    msg_rx: Receiver<SourceTrackerMessage>,
    project_details: Arc<Lx3ProjectDeets>,
    src_tree: SourceTree,
    src_tx: Sender<SourceUpdate>,
    watches: HashMap<PathBuf, FileWatcher>,
}

pub struct SourceTracker {
    msg_tx: Sender<SourceTrackerMessage>,
}

impl SourceTrackerEventLoop {
    fn new(
        project_details: Arc<Lx3ProjectDeets>,
        src_tree: SourceTree,
        src_tx: Sender<SourceUpdate>,
    ) -> Result<(Self, Sender<SourceTrackerMessage>), anyhow::Error> {
        let (fs_tx, fs_rx) = mpsc::channel::<FileUpdate>(10);
        let (msg_tx, msg_rx) = mpsc::channel::<SourceTrackerMessage>(10);
        let event_loop = Self {
            fs_interval: tokio::time::interval(Duration::from_secs(1)),
            fs_rx,
            fs_tx,
            fs_updates: Vec::new(),
            msg_rx,
            project_details,
            src_tree,
            src_tx,
            watches: HashMap::new(),
        };
        Ok((event_loop, msg_tx))
    }

    fn initialize(&mut self) -> Result<(), anyhow::Error> {
        self.add_file_watch(PathBuf::from("routes"))?;
        // for p in self
        //     .src_tree
        //     .sources
        //     .keys()
        //     .cloned()
        //     .collect::<Vec<PathBuf>>()
        // {
        //     self.add_file_watch(p)?;
        // }
        Ok(())
    }

    async fn start(&mut self) {
        loop {
            tokio::select! {
                tracker_message_opt = self.msg_rx.recv() => {
                    if let Some(tracker_message) = tracker_message_opt {
                        self.handle_tracker_message(tracker_message);
                    }
                }
                file_update_opt = self.fs_rx.recv() => {
                    if let Some(file_update) = file_update_opt {
                        self.fs_updates.push(file_update)
                    }
                }
                _ = self.fs_interval.tick() => if !self.fs_updates.is_empty() {
                    self.sync_file_updates()
                }
            }
        }
    }

    fn handle_tracker_message(&mut self, msg: SourceTrackerMessage) {
        match msg {}
    }

    fn sync_file_updates(&mut self) {}

    fn add_file_watch(&mut self, p: PathBuf) -> Result<(), anyhow::Error> {
        self.watches
            .insert(p.clone(), FileWatcher::for_file(self.fs_tx.clone(), p)?);
        Ok(())
    }
}

impl SourceTracker {
    pub fn new(
        project_details: Arc<Lx3ProjectDeets>,
        src_tree: SourceTree,
    ) -> Result<(Self, Receiver<SourceUpdate>), anyhow::Error> {
        let (src_tx, src_rx) = mpsc::channel::<SourceUpdate>(10);
        let (mut event_loop, msg_tx) =
            SourceTrackerEventLoop::new(project_details, src_tree, src_tx)?;
        event_loop.initialize()?;
        tokio::spawn(async move { event_loop.start().await });
        Ok((Self { msg_tx }, src_rx))
    }

    // pub fn watch_dir(&self, p: PathBuf) -> Result<(), anyhow::Error> {
    //     self.msg_tx.try_send(WatchDir(p))?;
    //     Ok(())
    // }
    //
    // pub fn watch_file(&self, p: PathBuf) -> Result<(), anyhow::Error> {
    //     self.msg_tx.try_send(WatchFile(p))?;
    //     Ok(())
    // }
}
