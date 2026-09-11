//! Repository-root detection and `--repo` override resolution.

use std::path::{Path, PathBuf};

/// Environment variable consulted when no explicit `--repo` is given.
pub const REPO_ENV: &str = "AGENT_REPO";

/// Why a repo path could not be resolved.
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    /// The process has no readable current directory.
    #[error("cannot determine current directory: {0}")]
    CurrentDir(#[source] std::io::Error),
    /// The candidate exists but could not be canonicalized (permissions, dangling symlink chain).
    #[error("cannot canonicalize repo path {}: {source}", path.display())]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl From<RepoError> for std::io::Error {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::CurrentDir(source) => source,
            RepoError::Canonicalize { path, source } => std::io::Error::new(
                source.kind(),
                format!("cannot canonicalize repo path {}: {source}", path.display()),
            ),
        }
    }
}

/// Walk up from `start` and return the first ancestor (including `start`)
/// that contains a `.git` entry. A `.git` file (worktrees, submodules) counts
/// the same as a `.git` directory.
pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Resolve the repository the tool should operate on.
///
/// Precedence: `explicit` (the `--repo` flag) > `AGENT_REPO` env var >
/// detected root via [`find_repo_root`] from the current directory > the
/// current directory itself.
///
/// Paths that exist are returned canonicalized. A path that does not exist
/// is returned as given so callers can report it (for example a `doctor`
/// command describing a missing repo) instead of failing here.
pub fn resolve_repo(explicit: Option<&Path>) -> Result<PathBuf, RepoError> {
    if let Some(path) = explicit {
        return canonical_if_exists(path.to_path_buf());
    }
    if let Some(path) = std::env::var_os(REPO_ENV).filter(|value| !value.is_empty()) {
        return canonical_if_exists(PathBuf::from(path));
    }
    let cwd = std::env::current_dir().map_err(RepoError::CurrentDir)?;
    let root = find_repo_root(&cwd).unwrap_or(cwd);
    canonical_if_exists(root)
}

fn canonical_if_exists(path: PathBuf) -> Result<PathBuf, RepoError> {
    if !path.exists() {
        return Ok(path);
    }
    path.canonicalize()
        .map_err(|source| RepoError::Canonicalize { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_git_dir_from_nested_start() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        fs::create_dir_all(root.join(".git")).unwrap();
        let nested = root.join("src/deep");
        fs::create_dir_all(&nested).unwrap();
        assert_eq!(find_repo_root(&nested), Some(root));
    }

    #[test]
    fn git_file_counts_as_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("worktree");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(".git"), "gitdir: /elsewhere\n").unwrap();
        assert_eq!(find_repo_root(&root), Some(root));
    }

    #[test]
    fn nearest_root_wins_for_nested_repos() {
        let tmp = tempfile::tempdir().unwrap();
        let outer = tmp.path().join("outer");
        let inner = outer.join("inner");
        fs::create_dir_all(outer.join(".git")).unwrap();
        fs::create_dir_all(inner.join(".git")).unwrap();
        assert_eq!(find_repo_root(&inner.join("x")), Some(inner));
    }

    #[test]
    fn no_git_means_none() {
        let tmp = tempfile::tempdir().unwrap();
        let plain = tmp.path().join("plain");
        fs::create_dir_all(&plain).unwrap();
        // The tempdir itself must not sit inside a repo for this to be meaningful.
        if find_repo_root(tmp.path()).is_none() {
            assert_eq!(find_repo_root(&plain), None);
        }
    }

    #[test]
    fn explicit_existing_path_is_canonical() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let resolved = resolve_repo(Some(&repo)).unwrap();
        assert_eq!(resolved, repo.canonicalize().unwrap());
    }

    #[test]
    fn explicit_missing_path_passes_through() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("does-not-exist");
        assert_eq!(resolve_repo(Some(&missing)).unwrap(), missing);
    }

    #[test]
    fn repo_error_converts_to_io_error() {
        let err = RepoError::Canonicalize {
            path: PathBuf::from("/nope"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "gone"),
        };
        let io: std::io::Error = err.into();
        assert_eq!(io.kind(), std::io::ErrorKind::NotFound);
        assert!(io.to_string().contains("/nope"));
    }
}
