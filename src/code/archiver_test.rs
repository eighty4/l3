use std::fs;
use std::path::PathBuf;
use std::process::Command;

use temp_dir::TempDir;

use crate::code::archiver::Archiver;

#[test]
fn test_archiver_write_abs_file_path() {
    let temp_test_dir = TempDir::new().unwrap();
    let project_dir = temp_test_dir.path().to_path_buf();
    fs::create_dir_all(project_dir.join("routes/data")).unwrap();
    let src_path = project_dir.join("routes/data/lambda.js");
    fs::write(&src_path, "export const GET = () => {}").unwrap();
    let archive_path = Archiver::new(
        project_dir.clone(),
        PathBuf::from("code.zip"),
        vec![src_path],
    )
    .write()
    .unwrap();
    assert_eq!(&archive_path, &project_dir.join("code.zip"));

    assert_eq!(
        0,
        Command::new("unzip")
            .arg(archive_path.to_string_lossy().to_string())
            .arg("-d")
            .arg(&project_dir.join("result").to_string_lossy().to_string())
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .code()
            .unwrap()
    );

    assert_eq!(
        fs::read_to_string(&project_dir.join("routes/data/lambda.js")).unwrap(),
        fs::read_to_string(&project_dir.join("result/routes/data/lambda.js")).unwrap(),
    );
}

#[test]
fn test_archiver_write_rel_file_path() {
    let temp_test_dir = TempDir::new().unwrap();
    let project_dir = temp_test_dir.path().to_path_buf();
    fs::create_dir_all(project_dir.join("routes/data")).unwrap();
    let src_path = PathBuf::from("routes/data/lambda.js");
    fs::write(&project_dir.join(&src_path), "export const GET = () => {}").unwrap();
    let archive_path = Archiver::new(
        project_dir.clone(),
        PathBuf::from("code.zip"),
        vec![src_path],
    )
    .write()
    .unwrap();
    assert_eq!(&archive_path, &project_dir.join("code.zip"));

    assert_eq!(
        0,
        Command::new("unzip")
            .arg(archive_path.to_string_lossy().to_string())
            .arg("-d")
            .arg(&project_dir.join("result").to_string_lossy().to_string())
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .code()
            .unwrap()
    );

    assert_eq!(
        fs::read_to_string(&project_dir.join("routes/data/lambda.js")).unwrap(),
        fs::read_to_string(&project_dir.join("result/routes/data/lambda.js")).unwrap(),
    );
}
