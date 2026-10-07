use git2::{ErrorCode, IndexAddOption, Oid, Repository, Signature, StatusOptions};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::AppConfig;

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

/// Resolves the absolute path and relative path within the repository's working directory.
fn resolve_repo_paths(repo: &Repository, path: &Path) -> Result<(PathBuf, PathBuf), git2::Error> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| git2::Error::from_str("Repository does not have a working directory"))?;

    let (full_path, rel_path) = if path.is_absolute() {
        let rel = match path.strip_prefix(workdir) {
            Ok(stripped) => stripped.to_path_buf(),
            Err(_) => {
                // If direct strip fails (e.g. symlinks), try canonicalizing
                let canon_workdir = workdir.canonicalize().ok();
                let canon_path = path.canonicalize().ok().or_else(|| {
                    path.parent()
                        .and_then(|p| p.canonicalize().ok())
                        .map(|mut p| {
                            if let Some(name) = path.file_name() {
                                p.push(name);
                            }
                            p
                        })
                });

                if let (Some(cw), Some(cp)) = (canon_workdir, canon_path) {
                    cp.strip_prefix(&cw)
                        .map_err(|_| {
                            git2::Error::from_str(
                                "Path is outside the repository working directory",
                            )
                        })?
                        .to_path_buf()
                } else {
                    return Err(git2::Error::from_str(
                        "Path is outside the repository working directory",
                    ));
                }
            }
        };
        (path.to_path_buf(), rel)
    } else {
        (workdir.join(path), path.to_path_buf())
    };

    Ok((full_path, rel_path))
}

/// Commits the current index state to HEAD.
fn commit_index(
    repo: &Repository,
    index: &mut git2::Index,
    message: &str,
) -> Result<Oid, git2::Error> {
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let message = parse_message(message);
    // Get author and committer signature from git config, with fallback
    let sig = repo
        .signature()
        .or_else(|_| Signature::now("dockeep", "dockeep@local"))?;

    // Get parent commit if HEAD exists (empty repo / unborn branch has no parents)
    let parent_commit = match repo.head() {
        Ok(head) => head.peel_to_commit().ok(),
        Err(_) => None,
    };
    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents)
}
fn parse_message(message: &str) -> String {
    let app_config = AppConfig::load();
    if app_config.git_setting.use_custom_language {
        return t!(
            message,
            locales = app_config.git_setting.language.name_short()
        )
        .to_string();
    }
    t!(message, locales = app_config.language.name_short()).to_string()
}
/// Adds a path to the git index and creates a commit with the specified message.
///
/// - `repo_path`: Path to the repository or any directory inside it.
/// - `path`: Path to add (file or directory). Can be absolute or relative to the repository working directory.
/// - `message`: The commit message.
///
/// Returns the commit `Oid` on success.
pub fn add_and_commit(repo_path: &Path, path: &Path, message: &str) -> Result<Oid, git2::Error> {
    let repo = Repository::discover(repo_path)?;
    let (full_path, rel_path) = resolve_repo_paths(&repo, path)?;
    let mut index = repo.index()?;

    if full_path.is_dir() {
        let pattern = if rel_path.as_os_str().is_empty() || rel_path == Path::new(".") {
            "*".to_string()
        } else {
            let s = rel_path.to_string_lossy();
            if s.ends_with('/') {
                format!("{}*", s)
            } else {
                format!("{}/*", s)
            }
        };
        index.add_all([pattern], IndexAddOption::DEFAULT, None)?;
    } else if full_path.exists() {
        index.add_path(&rel_path)?;
    } else {
        // If file was deleted from disk, remove it from the index
        let _ = index.remove_path(&rel_path);
    }

    commit_index(&repo, &mut index, message)
}

/// Convenience alias for `add_and_commit`.
pub fn add_path_and_commit(
    repo_path: &Path,
    path: &Path,
    message: &str,
) -> Result<Oid, git2::Error> {
    add_and_commit(repo_path, path, message)
}

/// Adds a path to the index and creates a commit, discovering the repository from `path`.
pub fn commit_path(path: &Path, message: &str) -> Result<Oid, git2::Error> {
    add_and_commit(path, path, message)
}

/// Stages the removal/deletion of a path and creates a commit with the specified message.
/// If the file or directory still exists on disk, it is removed from the filesystem as well.
///
/// - `repo_path`: Path to the repository or any directory inside it.
/// - `path`: Path to remove (file or directory). Can be absolute or relative to the repository working directory.
/// - `message`: The commit message.
///
/// Returns the commit `Oid` on success.
pub fn remove_and_commit(repo_path: &Path, path: &Path, message: &str) -> Result<Oid, git2::Error> {
    let repo = Repository::discover(repo_path)?;
    let (full_path, rel_path) = resolve_repo_paths(&repo, path)?;
    let mut index = repo.index()?;

    // Remove from index
    if index.remove_path(&rel_path).is_err() {
        // If remove_path failed, try pattern removal for directories/wildcards
        let s = rel_path.to_string_lossy();
        let pattern = if s.is_empty() || rel_path == Path::new(".") {
            "*".to_string()
        } else if s.ends_with('/') {
            format!("{}*", s)
        } else {
            format!("{}/*", s)
        };
        index.remove_all([pattern], None)?;
    }

    // Remove from disk if still present
    if full_path.is_dir() {
        let _ = fs::remove_dir_all(&full_path);
    } else if full_path.exists() {
        let _ = fs::remove_file(&full_path);
    }

    commit_index(&repo, &mut index, message)
}

/// Convenience alias for `super::remove_and_commit`.
pub fn remove_path_and_commit(
    repo_path: &Path,
    path: &Path,
    message: &str,
) -> Result<Oid, git2::Error> {
    remove_and_commit(repo_path, path, message)
}

/// Convenience alias for `remove_and_commit`.
pub fn delete_and_commit(repo_path: &Path, path: &Path, message: &str) -> Result<Oid, git2::Error> {
    remove_and_commit(repo_path, path, message)
}

/// Stages the removal of a path and creates a commit with the specified message.
pub fn commit_removal(repo_path: &Path, path: &Path, message: &str) -> Result<Oid, git2::Error> {
    remove_and_commit(repo_path, path, message)
}

/// Stages the removal of a path discovering the repository from `path`.
pub fn commit_path_removal(path: &Path, message: &str) -> Result<Oid, git2::Error> {
    remove_and_commit(path, path, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_commit() {
        let temp_dir = std::env::temp_dir().join(format!(
            "dockeep_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let repo = Repository::init(&temp_dir).unwrap();

        // Test initial commit on unborn branch with relative path
        let file_path = temp_dir.join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let oid1 = add_and_commit(&temp_dir, Path::new("test.txt"), "Initial commit").unwrap();
        let commit1 = repo.find_commit(oid1).unwrap();
        assert_eq!(commit1.message().unwrap(), "Initial commit");
        assert_eq!(commit1.parent_count(), 0);

        // Test second commit with absolute path
        let file_path2 = temp_dir.join("sub").join("test2.txt");
        fs::create_dir_all(file_path2.parent().unwrap()).unwrap();
        fs::write(&file_path2, "second file").unwrap();

        let oid2 = add_and_commit(&temp_dir, &file_path2, "Second commit").unwrap();
        let commit2 = repo.find_commit(oid2).unwrap();
        assert_eq!(commit2.message().unwrap(), "Second commit");
        assert_eq!(commit2.parent_count(), 1);
        assert_eq!(commit2.parent_id(0).unwrap(), oid1);

        // Test commit_path convenience function discovering repo from path
        let file_path3 = temp_dir.join("test3.txt");
        fs::write(&file_path3, "third file").unwrap();
        let oid3 = commit_path(&file_path3, "Third commit").unwrap();
        let commit3 = repo.find_commit(oid3).unwrap();
        assert_eq!(commit3.message().unwrap(), "Third commit");
        assert_eq!(commit3.parent_count(), 1);
        assert_eq!(commit3.parent_id(0).unwrap(), oid2);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_remove_and_commit() {
        let temp_dir = std::env::temp_dir().join(format!(
            "dockeep_test_rm_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let repo = Repository::init(&temp_dir).unwrap();

        // Create initial files
        let file1 = temp_dir.join("file1.txt");
        let file2 = temp_dir.join("file2.txt");
        fs::write(&file1, "content 1").unwrap();
        fs::write(&file2, "content 2").unwrap();

        add_and_commit(&temp_dir, Path::new("file1.txt"), "Add file 1").unwrap();
        let oid2 = add_and_commit(&temp_dir, Path::new("file2.txt"), "Add file 2").unwrap();

        // Case 1: File already deleted on disk before calling remove_and_commit
        fs::remove_file(&file1).unwrap();
        let oid3 = remove_and_commit(&temp_dir, Path::new("file1.txt"), "Remove file 1").unwrap();
        let commit3 = repo.find_commit(oid3).unwrap();
        assert_eq!(commit3.message().unwrap(), "Remove file 1");
        assert_eq!(commit3.parent_id(0).unwrap(), oid2);

        // Verify file1 is not in the tree of commit3
        let tree3 = commit3.tree().unwrap();
        assert!(tree3.get_name("file1.txt").is_none());
        assert!(tree3.get_name("file2.txt").is_some());

        // Case 2: File still on disk - remove_and_commit removes it from disk and commits
        let oid4 = remove_and_commit(&temp_dir, &file2, "Remove file 2").unwrap();
        let commit4 = repo.find_commit(oid4).unwrap();
        assert_eq!(commit4.message().unwrap(), "Remove file 2");
        assert!(!file2.exists());

        let tree4 = commit4.tree().unwrap();
        assert!(tree4.get_name("file2.txt").is_none());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
