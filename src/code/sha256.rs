use std::path::PathBuf;
use std::{fs, io};

use anyhow::Error;
use base64::Engine;
use sha2::{Digest, Sha256};

pub fn make_checksum(path: &PathBuf) -> Result<String, Error> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(path)?;
    io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();
    Ok(base64::engine::general_purpose::STANDARD.encode(hash_bytes))
}

#[cfg(test)]
mod tests {
    use io::Write;

    use temp_dir::TempDir;

    use super::*;

    #[test]
    fn test() {
        let d = TempDir::new().expect("make temp dir");
        let p = d.path().join("file");
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&p)
            .expect("create file");
        f.write_all("content".as_bytes())
            .expect("write bytes to file");

        let result = make_checksum(&p);
        assert!(result.is_ok());
        assert_eq!(
            "7XACtDnprIRfIjV9giusFERzD722AW0+yUMil7nsn3M=",
            result.unwrap()
        );
    }
}
