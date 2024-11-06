use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};

pub fn join_relative_path(base: &Path, relative: &Path) -> PathBuf {
    debug_assert!(base.is_relative());
    debug_assert!(relative.to_string_lossy().starts_with('.'));
    rewrite_current_and_parent_path_segments(
        match base.file_name() {
            None => base,
            Some(_) => base.parent().unwrap(),
        }
        .join(relative),
    )
}

/// Removes `.` and `..` segments from paths, rewriting `..` to the parent directory
/// This fn returns Ok(None) if it's a noop (path does not have any current or parent segments)
pub fn rewrite_current_and_parent_path_segments(p: PathBuf) -> PathBuf {
    let mut stack: Vec<String> = Vec::new();
    let mut changed = false;
    for path_component_os_str in &p {
        let path_component = path_component_os_str.to_string_lossy();
        match path_component.as_ref() {
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
            _ => stack.push(path_component.to_string()),
        }
    }
    if changed {
        PathBuf::from(stack.join(MAIN_SEPARATOR_STR))
    } else {
        p
    }
}
