use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::code::archiver::Archiver;
use crate::testing::{ProjectTest, TestSource};

#[test]
fn test_archiver_write_abs_file_path() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = () => {}"),
        )
        .build();
    let archive_path = Archiver::new(
        project_test.project_dir.clone(),
        PathBuf::from("code.zip"),
        vec![project_test.path("routes/data/lambda.js")],
    )
    .write()
    .unwrap();
    assert_eq!(&archive_path, &project_test.path("code.zip"));

    assert_eq!(
        0,
        Command::new("unzip")
            .arg(archive_path.to_string_lossy().to_string())
            .arg("-d")
            .arg(project_test.path_string("result"))
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .code()
            .unwrap()
    );

    assert_eq!(
        fs::read_to_string(project_test.path("routes/data/lambda.js")).unwrap(),
        fs::read_to_string(project_test.path("result/routes/data/lambda.js")).unwrap(),
    );
}

#[test]
fn test_archiver_write_rel_file_path() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = () => {}"),
        )
        .build();
    let src_path = PathBuf::from("routes/data/lambda.js");
    let archive_path = Archiver::new(
        project_test.project_dir.clone(),
        PathBuf::from("code.zip"),
        vec![src_path],
    )
    .write()
    .unwrap();
    assert_eq!(&archive_path, &project_test.path("code.zip"));

    assert_eq!(
        0,
        Command::new("unzip")
            .arg(archive_path.to_string_lossy().to_string())
            .arg("-d")
            .arg(project_test.path_string("result"))
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .code()
            .unwrap()
    );

    assert_eq!(
        fs::read_to_string(project_test.path("routes/data/lambda.js")).unwrap(),
        fs::read_to_string(project_test.path("result/routes/data/lambda.js")).unwrap(),
    );
}
