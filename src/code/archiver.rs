use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use zip::write::FileOptions;
use zip::ZipWriter;

pub struct Archiver {
    dest: PathBuf,
    file_paths: Vec<PathBuf>,
    project_dir: PathBuf,
}

impl Archiver {
    pub fn new(project_dir: PathBuf, archive_path: PathBuf, file_paths: Vec<PathBuf>) -> Self {
        debug_assert!(project_dir.is_absolute());
        debug_assert!(archive_path.is_relative());
        debug_assert!(
            archive_path.components().count() == 1
                || archive_path
                    .parent()
                    .map(|dir| dir.exists())
                    .unwrap_or(true)
        );
        Self {
            dest: project_dir.join(archive_path),
            file_paths: file_paths
                .into_iter()
                .map(|p| {
                    if p.is_absolute() {
                        p
                    } else {
                        project_dir.join(p)
                    }
                })
                .collect(),
            project_dir,
        }
    }

    pub fn write(self) -> Result<PathBuf, anyhow::Error> {
        let zip_file = File::create(&self.dest)?;
        let mut zip_writer = ZipWriter::new(zip_file);
        let compress_options: FileOptions<'static, ()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let mut buf = Vec::new();
        for path in &self.file_paths {
            File::open(path)?.read_to_end(&mut buf)?;
            zip_writer.start_file(
                path.strip_prefix(&self.project_dir)?
                    .to_string_lossy()
                    .to_string(),
                compress_options,
            )?;
            zip_writer.write_all(buf.as_ref())?;
            buf.clear();
        }
        zip_writer.finish()?;
        Ok(self.dest)
    }
}
