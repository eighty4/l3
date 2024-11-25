use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;
use std::{fs, io};

/// Sha256 checksum.
#[derive(Clone, Deserialize)]
pub struct Checksum(String);

impl Checksum {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Create a Checksum of a file's content.
impl TryFrom<&Path> for Checksum {
    type Error = io::Error;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let mut hasher = Sha256::new();
        let mut file = fs::File::open(p)?;
        io::copy(&mut file, &mut hasher)?;
        Ok(Checksum(
            base64::engine::general_purpose::STANDARD.encode(hasher.finalize()),
        ))
    }
}

/// Create a Checksum of a string's content.
impl TryFrom<&str> for Checksum {
    type Error = anyhow::Error;

    fn try_from(p: &str) -> Result<Self, Self::Error> {
        let mut hasher = Sha256::new();
        hasher.write_all(p.as_bytes())?;
        Ok(Checksum(
            base64::engine::general_purpose::STANDARD.encode(hasher.finalize()),
        ))
    }
}
