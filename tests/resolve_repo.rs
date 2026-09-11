//! Integration tests for `resolve_repo` that need a controlled cwd/env.
//! Environment and cwd are process-wide, so everything lives in one test.

use agent_tools_core::{find_repo_root, resolve_repo, REPO_ENV};
use std::fs;

#[test]
fn precedence_explicit_env_detected_cwd() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().canonicalize().unwrap();

    let repo = base.join("repo");
    fs::create_dir_all(repo.join(".git")).unwrap();
    let nested = repo.join("src/inner");
    fs::create_dir_all(&nested).unwrap();

    let env_repo = base.join("env-repo");
    fs::create_dir_all(&env_repo).unwrap();

    let explicit = base.join("explicit");
    fs::create_dir_all(&explicit).unwrap();

    let plain = base.join("plain");
    fs::create_dir_all(&plain).unwrap();

    std::env::set_current_dir(&nested).unwrap();

    // detected root from cwd
    std::env::remove_var(REPO_ENV);
    assert_eq!(resolve_repo(None).unwrap(), repo);

    // env beats detection
    std::env::set_var(REPO_ENV, &env_repo);
    assert_eq!(resolve_repo(None).unwrap(), env_repo);

    // explicit beats env
    assert_eq!(resolve_repo(Some(&explicit)).unwrap(), explicit);

    // empty env is ignored
    std::env::set_var(REPO_ENV, "");
    assert_eq!(resolve_repo(None).unwrap(), repo);
    std::env::remove_var(REPO_ENV);

    // cwd fallback when no .git above (only meaningful if tmp is outside a repo)
    if find_repo_root(&base).is_none() {
        std::env::set_current_dir(&plain).unwrap();
        assert_eq!(resolve_repo(None).unwrap(), plain);
    }
}
