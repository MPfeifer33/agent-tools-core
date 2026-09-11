//! Shared plumbing for the agent CLI tools.
//!
//! Everything here is composition-only: small free functions and plain
//! structs. Each tool keeps its own error enum, its own clap `Cli`, and its
//! own JSON contract; this crate supplies the pieces they had each
//! re-implemented and let drift:
//!
//! - [`repo`]: `.git` walk-up detection and the shared `--repo` override
//!   resolution (explicit path > `AGENT_REPO` env > detected root > cwd).
//! - [`format`]: the `--format json|text` clap value enum.
//! - [`output`]: pretty JSON printing, the optional response envelope, and
//!   the stderr error report every tool emits on failure.
//! - [`exit`]: the exit-code table latch documents plus the `doctor --strict`
//!   gate codes, and `exit_with`.

pub mod exit;
pub mod format;
pub mod output;
pub mod repo;

pub use exit::{exit_with, ExitCode};
pub use format::Format;
pub use output::{
    error_value, print_json, print_raw_json, report_error, report_error_envelope, Envelope,
    ErrorEntry,
};
pub use repo::{find_repo_root, resolve_repo, RepoError, REPO_ENV};
