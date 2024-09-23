use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use zip::write::FileOptions;
use zip::ZipWriter;

use crate::code::source::path::SourcePath;

pub struct Archive {
    /// Absolute path to archive result
    pub path: PathBuf,
    pub sources: Vec<SourcePath>,
}

pub struct Archiver {
    /// Absolute path to archive result
    dest: PathBuf,
    sources: Vec<SourcePath>,
}

impl Archiver {
    pub fn new(build_dir: PathBuf, sources: Vec<SourcePath>) -> Self {
        debug_assert!(build_dir.is_absolute());
        debug_assert!(sources.iter().all(|p| p.abs.is_dir() || p.abs.is_file()));
        Self {
            dest: build_dir.join("code.zip"),
            sources,
        }
    }

    pub fn write(self) -> Result<Archive, anyhow::Error> {
        let _ = fs::create_dir_all(self.dest.parent().unwrap());
        let _ = fs::remove_file(&self.dest);
        let zip_file = File::create(&self.dest)?;
        let mut zip_writer = ZipWriter::new(zip_file);
        let compress_options: FileOptions<'static, ()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let mut buf = Vec::new();
        for path in &self.sources {
            File::open(&path.abs)?.read_to_end(&mut buf)?;
            zip_writer.start_file(path.rel.to_string_lossy().to_string(), compress_options)?;
            zip_writer.write_all(buf.as_ref())?;
            buf.clear();
        }
        zip_writer.finish()?;
        Ok(Archive {
            path: self.dest,
            sources: self.sources,
        })
    }
}
