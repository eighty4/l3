use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::paths::collect_files;
use crate::FnManifest;
use zip::write::FileOptions;
use zip::ZipWriter;

pub enum ArchiveInclusion {
    Directory(PathBuf),
}

// todo build_fn that archives build output from memory vs writing to disk then reading to archive
pub fn write_archive(
    build_dir: &Path,
    manifest: &FnManifest,
    archive_path: &Path,
    inclusions: Vec<ArchiveInclusion>,
) -> Result<(), anyhow::Error> {
    debug_assert!(build_dir.is_absolute());
    debug_assert!(manifest
        .sources
        .iter()
        .all(|source| source.path.is_relative()));
    debug_assert!(manifest
        .sources
        .iter()
        .map(|source| build_dir.join(&source.path))
        .all(|path| path.is_dir() || path.is_file()));
    assert!(archive_path.is_absolute());
    assert!(!archive_path.is_file());
    let _ = fs::create_dir_all(archive_path.parent().unwrap());
    let zip_file = File::create(archive_path)?;
    let mut zip_writer = ZipWriter::new(zip_file);
    let compress_options: FileOptions<'static, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut buf = Vec::new();
    for source in &manifest.sources {
        File::open(build_dir.join(&source.path))?.read_to_end(&mut buf)?;
        zip_writer.start_file(source.path.to_string_lossy(), compress_options)?;
        zip_writer.write_all(buf.as_ref())?;
        buf.clear();
    }
    for inclusion in inclusions {
        match inclusion {
            ArchiveInclusion::Directory(dir) => {
                debug_assert!(dir.is_relative());
                for abs in collect_files(&build_dir.join(&dir)) {
                    let rel = abs.strip_prefix(build_dir)?.to_path_buf();
                    File::open(abs)?.read_to_end(&mut buf)?;
                    zip_writer.start_file(rel.to_string_lossy(), compress_options)?;
                    zip_writer.write_all(buf.as_ref())?;
                    buf.clear();
                }
            }
        }
    }
    zip_writer.finish()?;
    Ok(())
}
