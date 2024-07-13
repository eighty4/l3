use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::code::archiver::Archiver;
use crate::code::build::BuildMode;
use crate::code::source::path::SourcePath;
use crate::code::source::FunctionBuildDir;
use crate::testing::{ProjectTest, TestSource};

#[test]
fn test_archiver_write_zipped_source_file() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = () => {}"),
        )
        .build();
    let build_dir = FunctionBuildDir::new("API".to_string(), BuildMode::Debug, "FN".to_string());
    let archive_path = Archiver::new(
        &project_test.project_dir,
        &build_dir,
        vec![project_test.source_path("routes/data/lambda.js")],
    )
    .write()
    .unwrap();
    assert_eq!(
        archive_path,
        build_dir.abs(&project_test.project_dir).join("code.zip")
    );

    unzip(&archive_path, &project_test.project_dir.join("result")).unwrap();
    assert_eq!(
        fs::read_to_string(project_test.path("routes/data/lambda.js")).unwrap(),
        fs::read_to_string(project_test.path("result/routes/data/lambda.js")).unwrap(),
    );
}

#[test]
fn test_archiver_write_zipped_build_output() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = () => {}"),
        )
        .build();
    let build_dir = FunctionBuildDir::new("API".to_string(), BuildMode::Debug, "FN".to_string());
    let source_path = project_test.source_path("routes/data/lambda.js");
    let built_source_path =
        SourcePath::from_rel(&project_test.project_dir, PathBuf::from("src/data.js"))
            .to_build_dir(build_dir.clone(), &project_test.project_dir);
    _ = fs::create_dir_all(&built_source_path.abs.parent().unwrap());
    fs::write(&built_source_path.abs, "hooty hoo").unwrap();
    let archive_path = Archiver::new(
        &project_test.project_dir,
        &build_dir,
        vec![source_path, built_source_path],
    )
    .write()
    .unwrap();

    unzip(&archive_path, &project_test.project_dir.join("result")).unwrap();
    assert_eq!(
        fs::read_to_string(project_test.path("routes/data/lambda.js")).unwrap(),
        fs::read_to_string(project_test.path("result/routes/data/lambda.js")).unwrap(),
    );
    assert_eq!(
        "hooty hoo",
        fs::read_to_string(project_test.path("result/src/data.js")).unwrap(),
    );
}

#[test]
fn test_archiver_write_does_not_append_to_existing_archive() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = () => {}"),
        )
        .build();
    let build_dir = FunctionBuildDir::new("API".to_string(), BuildMode::Debug, "FN".to_string());
    Archiver::new(
        &project_test.project_dir,
        &build_dir,
        vec![project_test.source_path("routes/data/lambda.js")],
    )
    .write()
    .unwrap();
    let archive_path = Archiver::new(
        &project_test.project_dir,
        &build_dir,
        vec![project_test.source_path("routes/data/lambda.js")],
    )
    .write()
    .unwrap();
    println!();

    assert_eq!(
        archive_path,
        build_dir.abs(&project_test.project_dir).join("code.zip")
    );

    unzip(&archive_path, &project_test.project_dir.join("result")).unwrap();
    assert_eq!(
        fs::read_to_string(project_test.path("routes/data/lambda.js")).unwrap(),
        fs::read_to_string(project_test.path("result/routes/data/lambda.js")).unwrap(),
    );
}

fn unzip(archive_path: &Path, unzip_path: &Path) -> Result<(), anyhow::Error> {
    let exit_code = Command::new("unzip")
        .arg(archive_path.to_string_lossy().to_string())
        .arg("-d")
        .arg(unzip_path.to_string_lossy().to_string())
        .spawn()?
        .wait()?
        .code()
        .unwrap();
    assert_eq!(0, exit_code);
    Ok(())
}
