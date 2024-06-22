use std::fs;
use std::io::Write;

use temp_dir::TempDir;

use crate::code::sha256::make_checksum;

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
