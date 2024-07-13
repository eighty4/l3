use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::anyhow;
use notify::event::{DataChange, ModifyKind};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinSet;

use crate::aws::config::load_sdk_config;
use crate::code::source::tracker::{SourceTracker, SourceUpdate};
use crate::config::{project_name, read_api_id_from_data_dir};

/*

TODO
    LambdaSources
        pushes updates to SourceTree instance for each LambdaFn/RouteKey
    SourceTree
        manages sources for a single LambdaFn/RouteKey
        pushes events to TaskExecutor
        delegates to a CodeBuilder trait to build LambdaFn
        requires_update() -> bool fn for sync and dev fs events
    TaskExecutor
        queues updates by RouteKey
        pushes events to DevInterface
        uses SourceTree for tasks to build code
    DevInterface
        shows status of LambdaFns and Tasks as updated by TaskExecutor

 */

pub struct DevOptions {}

enum FileUpdate {
    ContentChanged(PathBuf),
    FileCreated(PathBuf),
    FileRemoved(PathBuf),
}

struct FileWatcher {
    w: Debouncer<RecommendedWatcher, FileIdMap>,
}

pub async fn develop_project(_dev_options: DevOptions) -> Result<(), anyhow::Error> {
    let sdk_config = load_sdk_config().await;
    let project_dir = env::current_dir()?;
    let project_name = project_name()?.unwrap();
    let api_id = read_api_id_from_data_dir()?.unwrap();

    println!("λλλ dev");
    println!("  project: {project_name}");
    println!("  region: {}", sdk_config.region().unwrap().to_owned());
    println!("  api id: {api_id}");
    println!();

    let (fs_sender, fs_receiver) = mpsc::channel::<FileUpdate>(100);
    let (src_sender, src_receiver) = mpsc::channel::<SourceUpdate>(100);
    let sources = Arc::new(Mutex::new(SourceTracker::new(
        project_dir.clone(),
        src_sender,
    )));
    let mut watcher = FileWatcher::new(fs_sender)?;
    watcher.add_path(PathBuf::from("routes"))?;
    _ = watcher.add_path(PathBuf::from("src"));

    let mut join_set = JoinSet::new();
    join_set.spawn(handle_fs_events(fs_receiver, sources.clone()));
    join_set.spawn(handle_src_events(src_receiver));
    while join_set.join_next().await.is_some() {
        println!("join_next awaited");
    }

    Ok(())
}

async fn handle_fs_events(
    mut fs_receiver: Receiver<FileUpdate>,
    sources: Arc<Mutex<SourceTracker>>,
) {
    while let Some(update) = fs_receiver.recv().await {
        match update {
            FileUpdate::ContentChanged(p) => sources.lock().unwrap().content_changed(p),
            FileUpdate::FileCreated(p) => sources.lock().unwrap().file_created(p),
            FileUpdate::FileRemoved(p) => sources.lock().unwrap().file_removed(p),
        }
    }
}

async fn handle_src_events(mut src_receiver: Receiver<SourceUpdate>) {
    while let Some(update) = src_receiver.recv().await {
        // todo send to TaskExecutor
        match update {
            SourceUpdate::UpdateLambdaCode => println!("update lambda code"),
            SourceUpdate::UpdateLambdaEnv => println!("update lambda env"),
            SourceUpdate::UpdateLambdaDeps => println!("update lambda deps"),
            SourceUpdate::AddLambda => println!("add lambda"),
            SourceUpdate::RemoveLambda => println!("remove lambda"),
        }
    }
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

    fn add_path(&mut self, path: PathBuf) -> Result<(), anyhow::Error> {
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
    fn rm_path(&mut self, path: PathBuf) -> Result<(), anyhow::Error> {
        self.w.watcher().unwatch(path.as_path())?;
        self.w.cache().remove_root(path.as_path());
        Ok(())
    }
}
