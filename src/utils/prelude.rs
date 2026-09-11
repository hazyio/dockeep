use git2::Repository;
use std::path::{Path, PathBuf};
pub fn is_git_repo(path: &PathBuf) -> bool {
    Repository::discover(path).is_ok()
}

pub fn path_exists_or_none(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    if !path.exists() {
        return None;
    }
    Some(path.to_path_buf())
}
