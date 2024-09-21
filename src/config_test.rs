use crate::config::is_valid_project_name;

#[test]
fn test_is_valid_project_name() {
    // valid
    assert!(is_valid_project_name("asd"));
    assert!(is_valid_project_name("asdf-asdf-asdf0"));
    assert!(is_valid_project_name("asdf-asdf"));
    assert!(is_valid_project_name("asdf_asdf"));
    // not valid
    assert!(!is_valid_project_name("1asdf_asdf"));
    assert!(!is_valid_project_name("_asdf_asdf"));
    assert!(!is_valid_project_name("asdf asdf"));
    assert!(!is_valid_project_name("asdf(asdf)"));
    assert!(!is_valid_project_name("asdf-asdf-asdf-asdf"));
    assert!(!is_valid_project_name("as"));
    assert!(!is_valid_project_name("asdf-asdf-asdf_"));
    assert!(!is_valid_project_name("asdf-asdf-asdf-"));
}
