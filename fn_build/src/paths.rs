use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};

/// Joins file paths and rewrites `.` and `..` segments from result.
pub fn join_file_paths(base: &Path, relative: &Path) -> PathBuf {
    debug_assert!(!base.is_dir());
    debug_assert!(relative.to_string_lossy().starts_with('.'));
    rewrite_current_and_parent_path_segments(base.parent().unwrap().join(relative))
}

fn rewrite_current_and_parent_path_segments(p: PathBuf) -> PathBuf {
    let mut stack: Vec<String> = Vec::new();
    let mut changed = false;
    for path_component in p.to_string_lossy().split(MAIN_SEPARATOR_STR) {
        match path_component {
            "." => changed = true,
            ".." => {
                if stack.pop().is_none() {
                    panic!(
                        "rewrote fs path to outside project for {}",
                        p.to_string_lossy()
                    );
                }
                changed = true;
            }
            _ => stack.push(match path_component.strip_prefix("./") {
                None => path_component.to_string(),
                Some(path_component_sans_pwd_prefix) => {
                    changed = true;
                    path_component_sans_pwd_prefix.to_string()
                }
            }),
        }
    }
    if changed {
        PathBuf::from(stack.join(MAIN_SEPARATOR_STR))
    } else {
        p
    }
}
