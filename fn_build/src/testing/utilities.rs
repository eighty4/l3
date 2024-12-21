use std::{path::Path, process::Command};

#[cfg(target_os = "windows")]
pub fn unzip(archive_path: &Path, unzip_path: &Path) {
    std::fs::create_dir_all(unzip_path).unwrap();
    let output = Command::new("tar")
        .arg("-xf")
        .arg(archive_path)
        .arg("-C")
        .arg(unzip_path)
        .output()
        .unwrap();
    if !output.status.success() {
        dbg!(output);
        panic!();
    }
}

#[cfg(not(target_os = "windows"))]
pub fn unzip(archive_path: &Path, unzip_path: &Path) {
    let output = Command::new("unzip")
        .arg(archive_path.to_string_lossy().to_string())
        .arg("-d")
        .arg(unzip_path.to_string_lossy().to_string())
        .output()
        .unwrap();
    if !output.status.success() {
        dbg!(output);
        panic!();
    }
}
