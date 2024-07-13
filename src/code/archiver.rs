use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zip::write::FileOptions;
use zip::ZipWriter;

use crate::code::source::path::SourcePath;
use crate::code::source::FunctionBuildDir;

pub struct Archiver {
    /// Absolute path to archive result
    dest: PathBuf,
    source_paths: Vec<SourcePath>,
}

impl Archiver {
    pub fn new(
        project_dir: &Path,
        build_dir: &FunctionBuildDir,
        source_paths: Vec<SourcePath>,
    ) -> Self {
        debug_assert!(project_dir.is_absolute());
        debug_assert!(source_paths
            .iter()
            .all(|p| p.abs.is_dir() || p.abs.is_file()));
        Self {
            dest: build_dir.abs(project_dir).join("code.zip"),
            source_paths,
        }
    }

    pub fn write(self) -> Result<PathBuf, anyhow::Error> {
        _ = fs::create_dir_all(self.dest.parent().unwrap());
        _ = fs::remove_file(&self.dest);
        let zip_file = File::create(&self.dest)?;
        let mut zip_writer = ZipWriter::new(zip_file);
        let compress_options: FileOptions<'static, ()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let mut buf = Vec::new();
        for path in &self.source_paths {
            File::open(&path.abs)?.read_to_end(&mut buf)?;
            zip_writer.start_file(path.rel.to_string_lossy().to_string(), compress_options)?;
            zip_writer.write_all(buf.as_ref())?;
            buf.clear();
        }
        zip_writer.finish()?;
        Ok(self.dest)
    }
}
