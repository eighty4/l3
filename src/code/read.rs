use std::fs;
use std::path::PathBuf;
use tokio::task::JoinSet;

struct DirRead {
    files: Vec<PathBuf>,
    sub_dirs: Vec<PathBuf>,
}

/// Path results are relative if dir_path is relative and absolute if dir_path is absolute
pub async fn recursively_read_dirs(dir_path: &PathBuf) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut dir_reads: Vec<DirRead> = vec![read_dir(dir_path)?];
    loop {
        let mut join_set: JoinSet<Result<DirRead, anyhow::Error>> = JoinSet::new();
        for mut dir_read in dir_reads {
            files.append(&mut dir_read.files);
            for sub_dir in dir_read.sub_dirs {
                join_set.spawn(async move { read_dir(&sub_dir) });
            }
        }
        dir_reads = Vec::new();
        while let Some(result) = join_set.join_next().await {
            dir_reads.push(result??);
        }
        if dir_reads.is_empty() {
            break;
        }
    }
    Ok(files)
}

fn read_dir(p: &PathBuf) -> Result<DirRead, anyhow::Error> {
    let mut files = Vec::new();
    let mut sub_dirs = Vec::new();
    for dir_entry_result in fs::read_dir(p)? {
        let dir_entry = dir_entry_result?;
        let path = dir_entry.path();
        if path.is_dir() {
            sub_dirs.push(path);
        } else {
            files.push(path);
        }
    }
    Ok(DirRead { files, sub_dirs })
}
