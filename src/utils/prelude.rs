use git2::Repository;
use std::path::{Path, PathBuf};
pub fn is_git_repo(path: &Path) -> bool {
    Repository::discover(path).is_ok()
}
pub fn path_exists(path: &Path) -> bool {
    Path::new(path).exists()
}
pub fn path_exists_or_none(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    if !path_exists(path) {
        return None;
    }
    Some(path.to_path_buf())
}
