# PROJECT.md — agent-tools-core

**What:** Shared plumbing library for the agent CLI tools (latch, witness,
sieve, rivet, sentinel, quarry): repo-root detection and `--repo` resolution,
the `--format json|text` value enum, JSON printing plus the optional response
envelope, the exit-code table with `doctor --strict` gate codes, and the
stderr error report.

**Status:** v0.1.0, published to GitHub 2026-09-11 (public, tag `v0.1.0`). The six tools (latch, witness, sieve, rivet, sentinel, quarry) depend on it by git tag.
(`../agent-tools-core`). Only quarry is on the JSON envelope; the other five
keep their pinned contracts. Not published; publishing (crates.io or git dep)
is Mark's call.

**Tech:** Rust 2021 library crate, clap 4 (derive, for `ValueEnum`),
serde/serde_json, thiserror. Dev: tempfile.

## Module Ownership

| Module | Owner | Status |
|--------|-------|--------|
| repo.rs | Nix | Done |
| format.rs | Nix | Done |
| output.rs | Nix | Done |
| exit.rs | Nix | Done |
| README.md | Nix | Done |

## Build

```sh
cargo test              # 14 unit + 1 integration
cargo clippy --all-targets
```

## Key Design Choices

- Composition over inheritance: free functions and plain structs; no traits
  for tools to implement. Each tool keeps its own error enum and converts
  `RepoError` with a four-line `From` impl.
- `resolve_repo` returns a missing explicit path as given rather than erroring,
  because `doctor` commands report on missing repos.
- Existing paths come back canonicalized (matches what
  `git rev-parse --show-toplevel` used to give the tools).
- The envelope is opt-in per tool; a pinned contract is never rewrapped.
- Exit codes are named constants, not a renumbering: tools with documented
  historical codes keep them.

## Last Updated

2026-09-11 — Initial crate; latch, witness, sieve, rivet, sentinel, quarry
refactored onto it. `cargo test` passes with 15 tests; clippy clean.
