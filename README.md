# agent-tools-core

Shared plumbing for the agent CLI tools (`latch`, `witness`, `sieve`, `rivet`,
`sentinel`, `quarry`). Composition only: small free functions and plain
structs. Each tool keeps its own clap `Cli`, its own error enum, and its own
JSON contract; this crate replaces the pieces they had each copied and let
drift.

## What it provides

| Module | Item | Purpose |
| ------ | ---- | ------- |
| `repo` | `find_repo_root(start) -> Option<PathBuf>` | Walk up to the first ancestor containing `.git` (dir or file). |
| `repo` | `resolve_repo(explicit) -> Result<PathBuf, RepoError>` | `--repo` > `AGENT_REPO` env > detected root > cwd. Existing paths are canonicalized; a missing path is returned as given so `doctor`-style commands can report it. |
| `repo` | `RepoError`, `REPO_ENV` | Clear error type (`From<RepoError> for std::io::Error` for tools whose error enum wraps io); the env var name. |
| `format` | `Format { Json, Text }` | clap `ValueEnum` spelled `--format json\|text`; `is_json()`, `Default = Text`. |
| `output` | `Envelope<T>`, `ErrorEntry` | `{"tool", "schema_version", "ok", "data", "errors": [{"code","message"}]}`. |
| `output` | `print_json(&Envelope<T>)` | Pretty-print an envelope to stdout. |
| `output` | `print_raw_json(&impl Serialize)` | Pretty-print any value, unwrapped, for tools with a pinned contract. |
| `output` | `report_error(is_json, code, message)` | The stderr report every tool prints on failure: `{"ok": false, "error": {code, message}}` or `error: <message>`. |
| `output` | `report_error_envelope(is_json, tool, schema_version, code, message)` | Same, as a failed envelope, for tools on the envelope. |
| `exit` | `ExitCode` | `Success 0`, `Validation 1`, `ClaimConflict 2`, `NotFound 3`, `Storage 4`, `GateAct 10`, `GateReview 20`, `GateStop 30`. |
| `exit` | `exit_with(code)` | Flush stdout and `std::process::exit`. |

## Who is on the envelope

| Tool | JSON output | Why |
| ---- | ----------- | --- |
| quarry | **envelope** (`schema_version` 1) | Had no pinned contract: no SPEC, no JSON tests, no consumers parsing it. |
| latch | own contract (docs/SPEC.md, integration tests) | Preserved exactly; uses `print_raw_json` + `report_error`. |
| witness | own contract (docs/SPEC.md, integration tests, `witness.doctor.v1`) | Preserved exactly. |
| sieve | own contract (docs/SPEC.md, integration tests) | Preserved exactly. |
| rivet | own contract (docs/SPEC.md, integration tests) | Preserved exactly. |
| sentinel | own contract (per-command `schema_version` strings, unit tests) | Preserved exactly. |

Moving another tool onto the envelope is a contract change for its consumers;
do it deliberately, bump that tool's docs, and update this table.

## Exit codes

The table is what latch documents. Tools that predate it keep documented
historical codes where they differ (witness, sieve, rivet, sentinel, quarry
still exit 2 on io errors, not "claim conflict"); nothing here renumbers them.
`doctor --strict` gate codes: 10 act (initialize / coordinate / refresh),
20 review / validate, 30 stop.

## Using it

Path dependency for now, beside the tool repo:

```toml
[dependencies]
agent-tools-core = { path = "../agent-tools-core" }
```

```rust
pub use agent_tools_core::Format as OutputFormat;

pub fn resolve_repo(&self) -> Result<PathBuf, MyError> {
    Ok(agent_tools_core::resolve_repo(self.repo.as_deref())?)
}

// main
Err(e) => {
    agent_tools_core::report_error(cli.is_json(), e.error_code(), &e.to_string());
    agent_tools_core::exit_with(e.exit_code());
}
```

Publishing to crates.io or switching the tools to a git dependency is Mark's
decision; until then a standalone clone of any tool needs this repo checked
out next to it.

## Build

```sh
cargo test
cargo clippy --all-targets
```

## License

Apache-2.0. See LICENSE and NOTICE.
