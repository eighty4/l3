use crate::code::build::archive::write_archive;
use crate::code::source::path::{FunctionBuildDir, SourcePath};
use crate::testing::project::ProjectTest;
use crate::testing::source::TestSource;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[tokio::test]
async fn test_archiver_write_zipped_source_file() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = () => {}"),
        )
        .build();
    let build_dir = FunctionBuildDir::new(&project_test.project, &"l3-get-data-fn".to_string());
    let archive_path = write_archive(
        build_dir.abs.clone(),
        vec![project_test.source_path("routes/data/lambda.js")],
    )
    .unwrap();
    assert_eq!(archive_path, build_dir.abs.join("code.zip"));

    unzip(&archive_path, &project_test.project_dir.join("result")).unwrap();
    assert_eq!(
        fs::read_to_string(project_test.path("routes/data/lambda.js")).unwrap(),
        fs::read_to_string(project_test.path("result/routes/data/lambda.js")).unwrap(),
    );
}

#[tokio::test]
async fn test_archiver_write_zipped_build_output() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = () => {}"),
        )
        .build();
    let build_dir = FunctionBuildDir::new(&project_test.project, &"l3-get-data-fn".to_string());
    let source_path = project_test.source_path("routes/data/lambda.js");
    let built_source_path =
        SourcePath::from_rel(&project_test.project_dir, PathBuf::from("src/data.js"))
            .to_build_dir(build_dir.clone());
    let _ = fs::create_dir_all(&built_source_path.abs.parent().unwrap());
    fs::write(&built_source_path.abs, "hooty hoo").unwrap();
    let archive_path =
        write_archive(build_dir.abs.clone(), vec![source_path, built_source_path]).unwrap();

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

#[tokio::test]
async fn test_archiver_write_does_not_append_to_existing_archive() {
    let project_test = ProjectTest::builder()
        .with_source(
            TestSource::with_path("routes/data/lambda.js").content("export const GET = () => {}"),
        )
        .build();
    let build_dir = FunctionBuildDir::new(&project_test.project, &"l3-get-data-fn".to_string());
    write_archive(
        build_dir.abs.clone(),
        vec![project_test.source_path("routes/data/lambda.js")],
    )
    .unwrap();
    let archive_path = write_archive(
        build_dir.abs.clone(),
        vec![project_test.source_path("routes/data/lambda.js")],
    )
    .unwrap();

    assert_eq!(archive_path, build_dir.abs.join("code.zip"));

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
