use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{env, fs, io};

use zip::write::FileOptions;
use zip::ZipWriter;

pub fn create_archive() -> Result<PathBuf, anyhow::Error> {
    match fs::metadata(".l3") {
        Ok(metadata) => {
            if !metadata.is_dir() {
                fs::remove_file(".l3")?;
                fs::create_dir(".l3")?;
            }
        }
        Err(_) => {
            fs::create_dir(".l3")?;
        }
    }
    let zip_file = File::create(".l3/code.zip")?;
    let mut zip_writer = ZipWriter::new(zip_file);
    let canonicalized = env::current_dir()?.canonicalize()?;
    let mut paths: Vec<PathBuf> = Vec::new();
    collect_paths(&canonicalized, &mut paths)?;
    paths.sort();
    let mut buf = Vec::new();
    let options: FileOptions<'static, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for path in paths {
        let rel_path = path.strip_prefix(&canonicalized)?;
        if is_exclude_path(rel_path.to_string_lossy().as_ref()) {
            continue;
        } else if path.is_dir() {
            zip_writer.add_directory(rel_path.to_string_lossy(), options)?;
        } else {
            let mut f = File::open(&path)?;
            f.read_to_end(&mut buf)?;
            zip_writer.start_file(rel_path.to_string_lossy(), options)?;
            zip_writer.write_all(buf.as_ref())?;
            buf.clear();
        }
    }
    zip_writer.finish()?;
    Ok(PathBuf::from(".l3/code.zip"))
}

fn is_exclude_path(p: &str) -> bool {
    matches!(p, ".l3" | "l3.yml")
}

fn collect_paths(dir: &Path, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            paths.push(path.clone());
            if path.is_dir() {
                collect_paths(&path, paths)?;
            }
        }
    }
    Ok(())
}
