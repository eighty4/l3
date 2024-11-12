use std::{path::Path, process::Command};

pub fn unzip(archive_path: &Path, unzip_path: &Path) {
    let exit_code = Command::new("unzip")
        .arg(archive_path.to_string_lossy().to_string())
        .arg("-d")
        .arg(unzip_path.to_string_lossy().to_string())
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .code()
        .unwrap();
    assert_eq!(0, exit_code);
}
