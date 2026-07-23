//! Typed fail-closed diagnostics for the offline Metal driver.
//!
//! Every [`DriverError`](crate::diagnostic::DriverError) variant means the same
//! thing: no `metallib` was
//! produced. The driver never returns partial or best-effort bytes.

use core::fmt;

/// Which offline tool invocation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileStage {
    /// The `metal` front-end compilation of MSL to AIR.
    Metal,
    /// The `metallib` link of AIR into a Metal library.
    Metallib,
}

impl CompileStage {
    /// Returns the offline tool name for this stage.
    #[must_use]
    pub const fn tool(self) -> &'static str {
        match self {
            Self::Metal => "metal",
            Self::Metallib => "metallib",
        }
    }
}

impl fmt::Display for CompileStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tool())
    }
}

/// A typed offline-compilation failure.
///
/// The driver fails closed: each variant is a hard rejection, not a fallback to
/// compiler defaults or to an unqualified toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverError {
    /// The Apple toolchain could not be resolved or did not report its identity.
    ///
    /// This is the expected result on a host without a qualified Metal
    /// toolchain, including a non-macOS host where `xcrun` cannot be run.
    ToolchainUnavailable {
        /// The tool whose resolution or version query failed.
        tool: String,
        /// A human-readable explanation.
        detail: String,
    },
    /// The selected SDK could not be resolved.
    SdkUnavailable {
        /// The `--sdk` selector that failed.
        sdk: String,
        /// A human-readable explanation.
        detail: String,
    },
    /// A tool ran to completion but reported a nonzero status.
    ToolFailure {
        /// The failing stage.
        stage: CompileStage,
        /// The reported exit status.
        status: String,
        /// The captured standard-error text.
        stderr: String,
    },
    /// A host filesystem or process operation failed.
    Host {
        /// A human-readable explanation.
        detail: String,
    },
    /// A tool succeeded but produced no usable Metal library.
    EmptyArtifact {
        /// A human-readable explanation.
        detail: String,
    },
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolchainUnavailable { tool, detail } => {
                write!(f, "Apple Metal toolchain unavailable ({tool}): {detail}")
            }
            Self::SdkUnavailable { sdk, detail } => {
                write!(f, "Apple SDK unavailable ({sdk}): {detail}")
            }
            Self::ToolFailure {
                stage,
                status,
                stderr,
            } => {
                write!(f, "offline {stage} failed ({status}): {stderr}")
            }
            Self::Host { detail } => write!(f, "host operation failed: {detail}"),
            Self::EmptyArtifact { detail } => {
                write!(
                    f,
                    "offline compilation produced no usable metallib: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for DriverError {}

#[cfg(test)]
mod tests {
    use super::{CompileStage, DriverError};

    #[test]
    fn toolchain_unavailable_names_the_tool() {
        let error = DriverError::ToolchainUnavailable {
            tool: "metallib".to_owned(),
            detail: "could not run xcrun".to_owned(),
        };
        let rendered = error.to_string();
        assert!(rendered.contains("metallib"));
        assert!(rendered.contains("could not run xcrun"));
    }

    #[test]
    fn tool_failure_names_the_stage() {
        let error = DriverError::ToolFailure {
            stage: CompileStage::Metal,
            status: "exit status: 1".to_owned(),
            stderr: "bad kernel".to_owned(),
        };
        assert!(error.to_string().starts_with("offline metal failed"));
    }
}
