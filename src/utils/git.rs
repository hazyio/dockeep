use git2::{ErrorCode, Repository, StatusOptions};
use std::path::Path;

#[derive(Clone)]
pub struct GitRepoInfo {
    pub branch: String,
    pub dirty: bool,
}

pub fn is_git_repo(path: &Path) -> bool {
    Repository::discover(path).is_ok()
}

fn branch_name(repo: &Repository) -> Option<String> {
    match repo.head() {
        Ok(head) => head.shorthand().ok().map(str::to_owned),
        // No commits yet: HEAD is symbolic and points to a branch that doesn't exist
        Err(e) if e.code() == ErrorCode::UnbornBranch => {
            let head = repo.find_reference("HEAD").ok()?;
            let target = head.symbolic_target().ok()??;
            Some(
                target
                    .strip_prefix("refs/heads/")
                    .unwrap_or(target)
                    .to_owned(),
            )
        }
        Err(_) => None,
    }
}
pub fn get_git_repo_info(path: &Path) -> Option<GitRepoInfo> {
    let repo = Repository::discover(path).ok()?;
    let branch = branch_name(&repo)?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    let statuses = repo.statuses(Some(&mut opts)).ok()?;

    Some(GitRepoInfo {
        branch,
        dirty: !statuses.is_empty(),
    })
}

pub fn get_git_branch(path: &Path) -> Option<String> {
    branch_name(&Repository::discover(path).ok()?)
}
