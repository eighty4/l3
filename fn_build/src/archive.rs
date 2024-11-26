use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use zip::write::FileOptions;
use zip::ZipWriter;

// todo archive build output from memory vs writing to disk then reading to archive
pub fn write_archive(archive_file: &Path, build_dir: &Path) -> Result<(), anyhow::Error> {
    debug_assert!(build_dir.is_absolute());
    debug_assert!(build_dir.is_dir());
    debug_assert!(archive_file.extension().unwrap().to_string_lossy() == "zip");
    assert!(archive_file.is_absolute());
    assert!(!archive_file.is_file());
    let _ = fs::create_dir_all(archive_file.parent().unwrap());
    let zip_file = File::create(archive_file)?;
    let mut zip_writer = ZipWriter::new(zip_file);
    let compress_options: FileOptions<'static, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut buf = Vec::new();
    for abs in l3_api_base::collect_files(build_dir) {
        File::open(build_dir.join(&abs))?.read_to_end(&mut buf)?;
        let rel = abs.strip_prefix(build_dir)?;
        zip_writer.start_file(rel.to_string_lossy(), compress_options)?;
        zip_writer.write_all(buf.as_ref())?;
        buf.clear();
    }
    zip_writer.finish()?;
    Ok(())
}
