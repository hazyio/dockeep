use git2::{Repository, StatusOptions};
use std::path::PathBuf;
#[derive(Clone)]

pub struct GitRepoInfo {
    pub branch: String,
    pub dirty: bool,
}
pub fn is_git_repo(path: &PathBuf) -> bool {
    Repository::discover(path).is_ok()
}
pub fn get_git_repo_info(path: &PathBuf) -> Option<GitRepoInfo> {
    let repo = Repository::discover(path).ok()?;

    let head = repo.head().ok()?;
    let Ok(branch) = head.shorthand() else {
        return None;
    };

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    let statuses = repo.statuses(Some(&mut opts)).ok()?;
    let dirty = statuses.is_empty();

    Some(GitRepoInfo {
        branch: branch.to_string(),
        dirty,
    })
}
pub fn get_git_branch(path: &PathBuf) -> Option<String> {
    let repo = Repository::discover(path).ok()?;
    let Ok(branch) = repo.head() else {
        return None;
    };
    let Ok(branch) = branch.name() else {
        return None;
    };
    let branch = branch.to_string();

    Some(branch)
}
 