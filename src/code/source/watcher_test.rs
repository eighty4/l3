use crate::code::source::watcher::{FileUpdate, FileUpdateKind, FileWatcher, SpecialFile};
use crate::testing::project::ProjectTest;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::{interval_at, Instant};

async fn test_file_watcher(
    path: PathBuf,
    kind: FileUpdateKind,
    with_file_update: impl Fn(FileUpdate),
) {
    debug_assert!(path.is_relative());
    let project_test = ProjectTest::builder().build();

    let absolute_path = project_test.project_dir.join(path);

    // set up a test scenario's file before creating watch
    match kind {
        FileUpdateKind::ContentChanged => fs::write(&absolute_path, "").unwrap(),
        FileUpdateKind::FileCreated => {}
        FileUpdateKind::FileRemoved => fs::write(&absolute_path, "").unwrap(),
        FileUpdateKind::FileRenamed => todo!(),
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (tx, mut rx) = mpsc::channel::<FileUpdate>(10);
    let mut file_watcher = FileWatcher::new(project_test.project_dir.clone(), tx);
    file_watcher
        .add_non_recursive(project_test.project_dir.clone())
        .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    match kind {
        FileUpdateKind::ContentChanged => {
            let mut f = OpenOptions::new().write(true).open(&absolute_path).unwrap();
            f.write("new content".as_bytes()).unwrap();
            f.flush().unwrap();
        }
        FileUpdateKind::FileCreated => fs::write(&absolute_path, "").unwrap(),
        FileUpdateKind::FileRemoved => fs::remove_file(&absolute_path).unwrap(),
        FileUpdateKind::FileRenamed => todo!(),
    }

    let mut interval = interval_at(
        Instant::now() + Duration::from_secs(2),
        Duration::from_secs(2),
    );
    select! {
        opt = rx.recv() => {
            match opt {
                None => panic!(),
                Some(file_update) => with_file_update(file_update),
            }
        },
        _ = interval.tick() => {
            panic!();
        }
    }
}

#[tokio::test]
async fn test_file_watcher_sends_file_created_file_update() {
    test_file_watcher(
        PathBuf::from("data.js"),
        FileUpdateKind::FileCreated,
        |file_update| {
            assert!(file_update.file.is_none());
            assert!(matches!(file_update.kind, FileUpdateKind::FileCreated));
            assert!(file_update.path.is_absolute());
            assert!(file_update.path.ends_with("data.js"));
        },
    )
    .await;
}

#[ignore]
#[tokio::test]
async fn test_file_watcher_sends_content_changed_file_update() {
    test_file_watcher(
        PathBuf::from("data.js"),
        FileUpdateKind::ContentChanged,
        |file_update| {
            assert!(file_update.file.is_none());
            assert!(matches!(file_update.kind, FileUpdateKind::ContentChanged));
            assert!(file_update.path.is_absolute());
            assert!(file_update.path.ends_with("data.js"));
        },
    )
    .await;
}

#[ignore]
#[tokio::test]
async fn test_file_watcher_sends_file_removed_file_update() {
    test_file_watcher(
        PathBuf::from("data.js"),
        FileUpdateKind::FileRemoved,
        |file_update| {
            assert!(file_update.file.is_none());
            assert!(matches!(file_update.kind, FileUpdateKind::FileRemoved));
            assert!(file_update.path.is_absolute());
            assert!(file_update.path.ends_with("data.js"));
        },
    )
    .await;
}

#[tokio::test]
async fn test_file_watcher_sends_package_json_file_update() {
    test_file_watcher(
        PathBuf::from("package.json"),
        FileUpdateKind::FileCreated,
        |file_update| match file_update.file {
            None => panic!(),
            Some(file) => assert!(matches!(file, SpecialFile::PackageJson)),
        },
    )
    .await;
}

#[tokio::test]
async fn test_file_watcher_sends_tsconfig_json_file_update() {
    test_file_watcher(
        PathBuf::from("tsconfig.json"),
        FileUpdateKind::FileCreated,
        |file_update| match file_update.file {
            None => panic!(),
            Some(file) => assert!(matches!(file, SpecialFile::TypeScriptConfig)),
        },
    )
    .await;
}
