use std::fs;
use std::path::PathBuf;

use temp_dir::TempDir;

use crate::code::checksum::ChecksumCached;
use crate::code::sha256::make_checksum;
use crate::code::source::SourceFile;

const SOURCE: &str = "export function GET(){}";

#[test]
fn test_source_file_new_lambda_file() {
    let project_dir = TempDir::new().unwrap();
    fs::create_dir_all(project_dir.path().join("routes/data")).unwrap();
    let source_file_abs_path = project_dir.path().join("routes/data/lambda.js");
    fs::write(&source_file_abs_path, SOURCE).unwrap();
    let source_file = SourceFile::create(
        PathBuf::from("routes/data/lambda.js"),
        project_dir.path().to_path_buf(),
    )
    .unwrap();
    assert_eq!(source_file.extension, "js");
    assert_eq!(source_file.file_name, "lambda.js");
    assert_eq!(
        source_file.hash,
        make_checksum(&source_file_abs_path).unwrap()
    );
    assert_eq!(source_file.path, PathBuf::from("routes/data/lambda.js"));
}

#[test]
fn test_source_file_new_env_file() {
    let project_dir = TempDir::new().unwrap();
    fs::create_dir_all(project_dir.path().join("routes/data")).unwrap();
    let source_file_abs_path = project_dir.path().join("routes/data/.env");
    fs::write(&source_file_abs_path, "PROP=more data").unwrap();
    let source_file = SourceFile::create(
        PathBuf::from("routes/data/.env"),
        project_dir.path().to_path_buf(),
    )
    .unwrap();
    assert_eq!(source_file.extension, "env");
    assert_eq!(source_file.file_name, ".env");
    assert_eq!(
        source_file.hash,
        make_checksum(&source_file_abs_path).unwrap()
    );
    assert_eq!(source_file.path, PathBuf::from("routes/data/.env"));
}

#[test]
fn test_source_file_checksum_cache_do_checksums_match_clean() {
    for file_name in &["lambda.js", ".env"] {
        let project_dir = TempDir::new().unwrap();
        let api_id = "abc123".to_string();
        let source_file_rel_path = PathBuf::from("routes/data").join(file_name);
        let source_file_abs_path = project_dir.path().join(&source_file_rel_path);
        fs::create_dir_all(&project_dir.path().join("routes/data")).unwrap();
        fs::write(&project_dir.path().join(&source_file_rel_path), SOURCE).unwrap();
        let source_file =
            SourceFile::create(source_file_rel_path, project_dir.path().to_path_buf()).unwrap();
        let checksum_path = project_dir
            .path()
            .join(source_file.get_relative_cached_checksum_path(&api_id));
        fs::create_dir_all(&checksum_path.parent().unwrap()).unwrap();
        fs::write(
            &checksum_path,
            make_checksum(&source_file_abs_path).unwrap(),
        )
        .unwrap();
        assert_eq!(source_file.do_checksums_match(&api_id).unwrap(), true);
    }
}

#[test]
fn test_source_file_checksum_cache_do_checksums_match_dirty() {
    for file_name in &["lambda.js", ".env"] {
        let project_dir = TempDir::new().unwrap();
        let api_id = "abc123".to_string();
        let source_file_rel_path = PathBuf::from("routes/data").join(file_name);
        fs::create_dir_all(&project_dir.path().join("routes/data")).unwrap();
        fs::write(&project_dir.path().join(&source_file_rel_path), SOURCE).unwrap();
        let source_file =
            SourceFile::create(source_file_rel_path, project_dir.path().to_path_buf()).unwrap();
        let checksum_path = project_dir
            .path()
            .join(source_file.get_relative_cached_checksum_path(&api_id));
        fs::create_dir_all(&checksum_path.parent().unwrap()).unwrap();
        fs::write(
            &checksum_path,
            "this is not the checksum you are looking for",
        )
        .unwrap();
        assert_eq!(source_file.do_checksums_match(&api_id).unwrap(), false);
    }
}

#[test]
fn test_source_file_checksum_cache_do_checksums_match_without_checksum() {
    for file_name in &["lambda.js", ".env"] {
        let project_dir = TempDir::new().unwrap();
        let api_id = "abc123".to_string();
        let source_file_rel_path = PathBuf::from("routes/data").join(file_name);
        fs::create_dir_all(&project_dir.path().join("routes/data")).unwrap();
        fs::write(&project_dir.path().join(&source_file_rel_path), "").unwrap();
        let source_file =
            SourceFile::create(source_file_rel_path, project_dir.path().to_path_buf()).unwrap();
        assert_eq!(source_file.do_checksums_match(&api_id).unwrap(), false);
        fs::create_dir(project_dir.path().join(".l3")).unwrap();
        assert_eq!(source_file.do_checksums_match(&api_id).unwrap(), false);
        fs::create_dir(project_dir.path().join(".l3").join(&api_id)).unwrap();
        assert_eq!(source_file.do_checksums_match(&api_id).unwrap(), false);
        fs::create_dir(project_dir.path().join(".l3").join(&api_id).join("routes")).unwrap();
        assert_eq!(source_file.do_checksums_match(&api_id).unwrap(), false);
        fs::create_dir(
            project_dir
                .path()
                .join(".l3")
                .join(&api_id)
                .join("routes/data"),
        )
        .unwrap();
        assert_eq!(source_file.do_checksums_match(&api_id).unwrap(), false);
    }
}

#[test]
fn test_source_file_checksum_cache_get_cached_checksum() {
    for file_name in &["lambda.js", ".env"] {
        let project_dir = TempDir::new().unwrap();
        let api_id = "abc123".to_string();
        let source_file_rel_path = PathBuf::from("routes/data").join(file_name);
        fs::create_dir_all(project_dir.path().join("routes/data")).unwrap();
        fs::write(&project_dir.path().join(&source_file_rel_path), SOURCE).unwrap();
        let source_file =
            SourceFile::create(source_file_rel_path, project_dir.path().to_path_buf()).unwrap();
        assert!(source_file.get_cached_checksum(&api_id).unwrap().is_none());
        fs::create_dir(project_dir.path().join(".l3")).unwrap();
        assert!(source_file.get_cached_checksum(&api_id).unwrap().is_none());
        fs::create_dir(project_dir.path().join(".l3").join(&api_id)).unwrap();
        assert!(source_file.get_cached_checksum(&api_id).unwrap().is_none());
        fs::create_dir(project_dir.path().join(".l3").join(&api_id).join("routes")).unwrap();
        assert!(source_file.get_cached_checksum(&api_id).unwrap().is_none());
        fs::create_dir(
            project_dir
                .path()
                .join(".l3")
                .join(&api_id)
                .join("routes/data"),
        )
        .unwrap();
        assert!(source_file.get_cached_checksum(&api_id).unwrap().is_none());
        fs::write(
            project_dir
                .path()
                .join(".l3")
                .join(&api_id)
                .join("routes/data")
                .join(format!("{file_name}.sha256")),
            "i want to be a checksum when i grow up",
        )
        .unwrap();
        assert_eq!(
            source_file.get_cached_checksum(&api_id).unwrap().unwrap(),
            "i want to be a checksum when i grow up"
        );
    }
}

#[test]
fn test_source_file_checksum_cache_get_project_dir() {
    for file_name in &["lambda.js", ".env"] {
        let project_dir = TempDir::new().unwrap();
        fs::write(&project_dir.path().join(file_name), SOURCE).unwrap();
        let source_file =
            SourceFile::create(PathBuf::from(file_name), project_dir.path().to_path_buf()).unwrap();
        assert_eq!(
            source_file.get_project_dir().to_path_buf(),
            project_dir.path().to_path_buf()
        );
    }
}

#[test]
fn test_source_file_checksum_cache_get_relative_cached_checksum_path() {
    for file_name in &["lambda.js", ".env"] {
        let project_dir = TempDir::new().unwrap();
        let api_id = "abc123".to_string();
        let source_file_rel_path = PathBuf::from("routes/data").join(file_name);
        fs::create_dir_all(&project_dir.path().join("routes/data")).unwrap();
        fs::create_dir_all(&project_dir.path().join(".l3")).unwrap();
        fs::write(&project_dir.path().join(&source_file_rel_path), SOURCE).unwrap();
        let source_file =
            SourceFile::create(source_file_rel_path, project_dir.path().to_path_buf()).unwrap();
        assert_eq!(
            source_file.get_relative_cached_checksum_path(&api_id),
            PathBuf::from(".l3")
                .join(&api_id)
                .join("routes/data")
                .join(format!("{file_name}.sha256"))
        );
    }
}

#[test]
fn test_source_file_checksum_cache_get_relative_source_path() {
    for file_name in &["lambda.js", ".env"] {
        let project_dir = TempDir::new().unwrap();
        let source_file_rel_path = PathBuf::from("routes/data").join(file_name);
        fs::create_dir_all(&project_dir.path().join("routes/data")).unwrap();
        fs::create_dir_all(&project_dir.path().join(".l3")).unwrap();
        fs::write(&project_dir.path().join(&source_file_rel_path), SOURCE).unwrap();
        let source_file =
            SourceFile::create(source_file_rel_path, project_dir.path().to_path_buf()).unwrap();
        assert_eq!(
            source_file.get_relative_source_path().to_path_buf(),
            PathBuf::from("routes/data").join(file_name)
        );
    }
}

#[test]
fn test_source_file_checksum_cache_get_source_checksum() {
    for file_name in &["lambda.js", ".env"] {
        let project_dir = TempDir::new().unwrap();
        let source_file_rel_path = PathBuf::from(file_name);
        fs::write(project_dir.path().join(file_name), SOURCE).unwrap();
        let source_file =
            SourceFile::create(source_file_rel_path, project_dir.path().to_path_buf()).unwrap();
        assert_eq!(source_file.get_source_checksum().unwrap(), source_file.hash);
    }
}

#[test]
fn test_source_file_checksum_cache_update_checksum_cache() {
    for file_name in &["lambda.js", ".env"] {
        let project_dir = TempDir::new().unwrap();
        let api_id = "abc123".to_string();
        let source_file_rel_path = PathBuf::from("routes/data").join(file_name);
        fs::create_dir_all(&project_dir.path().join("routes/data")).unwrap();
        fs::create_dir_all(&project_dir.path().join(".l3").join(&api_id)).unwrap();
        let source_file_abs_path = project_dir.path().join(&source_file_rel_path);
        fs::write(&project_dir.path().join(&source_file_rel_path), SOURCE).unwrap();
        let checksum_path = &project_dir
            .path()
            .join(".l3")
            .join(&api_id)
            .join(&source_file_rel_path.with_file_name(format!("{file_name}.sha256")));
        let mut source_file = SourceFile::create(
            source_file_rel_path.clone(),
            project_dir.path().to_path_buf(),
        )
        .unwrap();
        assert!(!checksum_path.is_file());
        source_file.update_checksum_cache(&api_id).unwrap();
        assert!(checksum_path.is_file());
        assert_eq!(
            fs::read_to_string(&checksum_path).unwrap(),
            make_checksum(&source_file_abs_path).unwrap()
        );
        fs::write(&source_file_abs_path, "export function PATCH(){}").unwrap();
        // todo source file caching hash and being immutable will not work for a dev watch mode
        source_file =
            SourceFile::create(source_file_rel_path, project_dir.path().to_path_buf()).unwrap();
        source_file.update_checksum_cache(&api_id).unwrap();
        assert_eq!(
            fs::read_to_string(&checksum_path).unwrap(),
            make_checksum(&source_file_abs_path).unwrap()
        );
    }
}
