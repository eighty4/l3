use crate::checksum::Checksum;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_checksum_try_from_path() {
    let p = PathBuf::from("fixtures/python/http_routes/get_fn/routes/data/lambda.py");
    assert_eq!(
        Checksum::try_from(p.as_path()).unwrap().as_str(),
        "FKgftnCHl4EcZ+rr8zwnBa1+bneocM2lS9PvMqRblGk="
    );
}

#[test]
fn test_checksum_try_from_string() {
    let p = PathBuf::from("fixtures/python/http_routes/get_fn/routes/data/lambda.py");
    let s = fs::read_to_string(p).unwrap();
    assert_eq!(
        Checksum::try_from(s.as_str()).unwrap().as_str(),
        "FKgftnCHl4EcZ+rr8zwnBa1+bneocM2lS9PvMqRblGk="
    );
}
