use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

use crate::aws::config::load_sdk_config;
use crate::code::source::tracker::{SourceTracker, SourceUpdate};
use crate::code::source::watcher::{FileUpdate, FileWatcher};
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
