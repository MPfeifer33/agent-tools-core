//! Process exit codes shared across the tools.

/// The exit-code table latch documents, plus the `doctor --strict` gate
/// codes. Tools that predate the table keep any historical codes they still
/// document (for example `io error` = 2 outside latch); nothing here forces a
/// renumbering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// Command completed.
    Success = 0,
    /// Bad input, bad flags, malformed data.
    Validation = 1,
    /// A claim on the requested path is held by someone else.
    ClaimConflict = 2,
    /// The referenced record does not exist.
    NotFound = 3,
    /// Database, filesystem, or serialization failure while persisting.
    Storage = 4,
    /// `doctor --strict`: routine action needed before proceeding
    /// (initialize / coordinate / refresh).
    GateAct = 10,
    /// `doctor --strict`: the change needs scrutiny (review / validate).
    GateReview = 20,
    /// `doctor --strict`: stop; the tool considers the state unsafe to build on.
    GateStop = 30,
}

impl ExitCode {
    pub const fn code(self) -> i32 {
        self as i32
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code.code()
    }
}

/// Flush stdout and exit the process with `code`.
pub fn exit_with(code: impl Into<i32>) -> ! {
    use std::io::Write;
    let _ = std::io::stdout().flush();
    std::process::exit(code.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_matches_latch_docs() {
        assert_eq!(ExitCode::Success.code(), 0);
        assert_eq!(ExitCode::Validation.code(), 1);
        assert_eq!(ExitCode::ClaimConflict.code(), 2);
        assert_eq!(ExitCode::NotFound.code(), 3);
        assert_eq!(ExitCode::Storage.code(), 4);
        assert_eq!(ExitCode::GateAct.code(), 10);
        assert_eq!(ExitCode::GateReview.code(), 20);
        assert_eq!(ExitCode::GateStop.code(), 30);
        let as_i32: i32 = ExitCode::NotFound.into();
        assert_eq!(as_i32, 3);
    }
}
