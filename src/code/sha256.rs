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
