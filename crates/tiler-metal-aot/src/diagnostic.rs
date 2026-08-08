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
    /// Every stage one offline compilation runs, in execution order.
    ///
    /// A caller that names both stages — retaining what each wrote, say — reads
    /// them from here rather than writing the pair out again, so the order and
    /// the membership are stated once.
    ///
    /// The declared length is `variant_count`, so a stage added to the enum and
    /// not to this list is an array-length error at this declaration. The
    /// length is what has to carry that guarantee: an exhaustive `match` wrapped
    /// around the literal does go non-exhaustive when a stage lands, but it
    /// constrains the pattern rather than the array, so widening the alternation
    /// silences it while the short literal still compiles and a caller iterating
    /// `ALL` still never visits the new stage.
    pub const ALL: [Self; core::mem::variant_count::<Self>()] = [Self::Metal, Self::Metallib];

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

/// Which query about the toolchain failed.
///
/// Discovery and version probing are separate because their remedies are:
/// discovery failing means no such tool is installed for the selected SDK,
/// while a version probe failing means the tool exists and did not identify
/// itself — one is an installation problem and the other is a qualification
/// problem, and a caller that could not tell them apart could act on neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolchainPhase {
    /// `xcrun --find` could not locate the tool for the selected SDK.
    Discovery,
    /// The located tool did not report a usable version banner.
    VersionProbe,
}

impl fmt::Display for ToolchainPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Discovery => "discovery",
            Self::VersionProbe => "version probe",
        })
    }
}

/// How one offline tool process ended.
///
/// Typed rather than the formatted `ExitStatus`, so a caller can branch on a
/// specific exit code — the Metal front end distinguishes them — without
/// parsing a message whose text is the host's to choose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    /// The tool exited with this code.
    Code(i32),
    /// The tool was terminated by this signal.
    Signal(i32),
    /// The host reported neither a code nor a signal.
    ///
    /// Kept rather than folded into `Code(-1)`, which would be a code the tool
    /// never returned, presented as one it did.
    Unreported,
}

impl fmt::Display for ToolStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(code) => write!(f, "exit code {code}"),
            Self::Signal(signal) => write!(f, "signal {signal}"),
            Self::Unreported => f.write_str("no reported status"),
        }
    }
}

/// Largest number of captured output bytes one diagnostic retains.
///
/// A failing Metal compilation can emit megabytes; a diagnostic that carried
/// all of it would make the error as expensive to move as the artifact that
/// failed. Truncation is recorded rather than hidden, so a reader is never
/// shown a prefix as if it were the whole.
pub const MAX_RETAINED_OUTPUT_BYTES: usize = 16 * 1024;

/// Captured tool output, retained as bytes and bounded.
///
/// One type for both outcomes: a stage that fails carries it in
/// [`DriverError::ToolFailure`] and a stage that succeeds carries it in
/// [`StageOutputs`](crate::record::StageOutputs). A second success-path capture
/// would be a second bound to keep in step with this one, and the two would
/// disagree the first time either moved.
///
/// **Bytes, not `String`.** A compiler's diagnostics are whatever it wrote, and
/// `String::from_utf8_lossy` at the capture site would replace invalid
/// sequences with `U+FFFD` and leave nothing able to tell that from a tool that
/// really emitted a replacement character. The bytes are kept; rendering is a
/// view of them, and [`Self::is_valid_utf8`] says which case a reader is in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutput {
    retained: Vec<u8>,
    total: usize,
}

impl ToolOutput {
    /// Retains up to [`MAX_RETAINED_OUTPUT_BYTES`] of what a tool wrote.
    #[must_use]
    pub fn capture(bytes: &[u8]) -> Self {
        Self {
            retained: bytes
                .iter()
                .take(MAX_RETAINED_OUTPUT_BYTES)
                .copied()
                .collect(),
            total: bytes.len(),
        }
    }

    /// The retained bytes exactly as the tool wrote them.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.retained
    }

    /// How many bytes the tool wrote, including any not retained.
    #[must_use]
    pub const fn total_bytes(&self) -> usize {
        self.total
    }

    /// Whether bytes were dropped to stay within the bound.
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.total > self.retained.len()
    }

    /// Whether the retained bytes are valid UTF-8.
    ///
    /// A reader that renders regardless should still know, because a lossy
    /// rendering of invalid bytes is not what the tool said.
    #[must_use]
    pub fn is_valid_utf8(&self) -> bool {
        std::str::from_utf8(&self.retained).is_ok()
    }

    /// Whether the tool wrote nothing at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.total == 0
    }
}

impl fmt::Display for ToolOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.retained).trim())?;
        if !self.is_valid_utf8() {
            f.write_str(" [output was not valid UTF-8]")?;
        }
        if self.is_truncated() {
            write!(
                f,
                " [truncated: {} of {} bytes retained]",
                self.retained.len(),
                self.total
            )?;
        }
        Ok(())
    }
}

/// A typed offline-compilation failure.
///
/// The driver fails closed: each variant is a hard rejection, not a fallback to
/// compiler defaults or to an unqualified toolchain.
///
/// **Do not mark this `#[non_exhaustive]`.** `tiler-metal` recognizes it out of
/// crate to decide whether a failed toolchain resolution means an absent Apple
/// toolchain — in which case its compiling tests self-skip — or a defect they
/// must report. That is ADR 0074 convention 5c: a wildcard arm there would be
/// correct today and wrong the moment a variant lands that must not be read as
/// an absent toolchain, and the mistake would compile silently and turn a
/// defect into a skipped test. The compile error is what keeps the
/// classification complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverError {
    /// The Apple toolchain could not be resolved or did not report its identity.
    ///
    /// This is the expected result on a host without a qualified Metal
    /// toolchain, including a non-macOS host where `xcrun` cannot be run.
    ToolchainUnavailable {
        /// The tool whose resolution or version query failed.
        tool: String,
        /// Which query failed, so a caller need not read `detail` to tell an
        /// absent tool from an unqualified one.
        phase: ToolchainPhase,
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
        /// The binary that actually ran.
        ///
        /// The resolved path rather than the stage's bare tool name: those are
        /// two different observations, and the whole point of executing the
        /// resolved tool is that a diagnostic names the one that produced the
        /// failure rather than the one a second selection would find.
        executable: std::path::PathBuf,
        /// How the process ended.
        status: ToolStatus,
        /// The captured standard-error output, bounded and byte-preserving.
        stderr: ToolOutput,
    },
    /// A host filesystem or process operation failed.
    Host {
        /// A human-readable explanation.
        detail: String,
    },
    /// A tool reported success but its output does not begin with the `MTLB`
    /// magic, so no `metallib`-shaped file was produced.
    ///
    /// The condition is a shape check on the linker's output, not a
    /// compatibility verdict: this crate never loads the library, so it cannot
    /// and does not report whether a device could.
    EmptyArtifact {
        /// A human-readable explanation.
        detail: String,
    },
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolchainUnavailable {
                tool,
                phase,
                detail,
            } => {
                write!(
                    f,
                    "Apple Metal toolchain unavailable ({tool}, {phase}): {detail}"
                )
            }
            Self::SdkUnavailable { sdk, detail } => {
                write!(f, "Apple SDK unavailable ({sdk}): {detail}")
            }
            Self::ToolFailure {
                stage,
                executable,
                status,
                stderr,
            } => {
                write!(
                    f,
                    "offline {stage} failed [{}] ({status}): {stderr}",
                    executable.display()
                )
            }
            Self::Host { detail } => write!(f, "host operation failed: {detail}"),
            Self::EmptyArtifact { detail } => {
                write!(
                    f,
                    "offline compilation produced no metallib-shaped output: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for DriverError {}

#[cfg(test)]
mod tests {
    use super::{
        CompileStage, DriverError, MAX_RETAINED_OUTPUT_BYTES, ToolOutput, ToolStatus,
        ToolchainPhase,
    };

    #[test]
    fn toolchain_unavailable_names_the_tool_and_the_query() {
        let error = DriverError::ToolchainUnavailable {
            tool: "metallib".to_owned(),
            phase: ToolchainPhase::VersionProbe,
            detail: "could not run xcrun".to_owned(),
        };
        let rendered = error.to_string();
        assert!(rendered.contains("metallib"));
        assert!(rendered.contains("version probe"));
        assert!(rendered.contains("could not run xcrun"));
    }

    /// The two toolchain queries stay distinct, because their remedies differ.
    #[test]
    fn discovery_and_version_probe_are_different_failures() {
        let of = |phase| DriverError::ToolchainUnavailable {
            tool: "metal".to_owned(),
            phase,
            detail: "same detail".to_owned(),
        };
        assert_ne!(
            of(ToolchainPhase::Discovery),
            of(ToolchainPhase::VersionProbe),
            "an absent tool and an unqualified one are not one failure",
        );
    }

    #[test]
    fn tool_failure_names_the_stage_the_executable_and_the_status() {
        let error = DriverError::ToolFailure {
            stage: CompileStage::Metal,
            executable: std::path::PathBuf::from("/usr/bin/metal"),
            status: ToolStatus::Code(1),
            stderr: ToolOutput::capture(b"bad kernel"),
        };
        let rendered = error.to_string();
        assert!(rendered.starts_with("offline metal failed"));
        assert!(rendered.contains("/usr/bin/metal"));
        assert!(rendered.contains("exit code 1"));
        assert!(rendered.contains("bad kernel"));
    }

    /// A signal is not an exit code, and neither is the absence of both.
    #[test]
    fn every_way_a_process_can_end_stays_distinct() {
        assert_ne!(ToolStatus::Code(9), ToolStatus::Signal(9));
        assert_ne!(ToolStatus::Code(-1), ToolStatus::Unreported);
        assert_eq!(ToolStatus::Signal(9).to_string(), "signal 9");
        assert_eq!(ToolStatus::Unreported.to_string(), "no reported status");
    }

    /// Output is retained as bytes, so a non-UTF-8 diagnostic stays honest.
    #[test]
    fn non_utf8_output_is_preserved_and_declared() {
        let raw = b"error: \xff\xfe not text";
        let output = ToolOutput::capture(raw);
        assert_eq!(output.as_bytes(), raw, "the tool's bytes are kept exactly");
        assert!(!output.is_valid_utf8());
        assert!(!output.is_truncated());
        assert!(
            output.to_string().contains("not valid UTF-8"),
            "a lossy rendering must say it is one: {output}",
        );
    }

    /// Truncation is reported rather than presented as the whole output.
    #[test]
    fn truncated_output_reports_what_it_dropped() {
        let raw = vec![b'x'; MAX_RETAINED_OUTPUT_BYTES + 64];
        let output = ToolOutput::capture(&raw);
        assert_eq!(output.as_bytes().len(), MAX_RETAINED_OUTPUT_BYTES);
        assert_eq!(output.total_bytes(), raw.len());
        assert!(output.is_truncated());
        let rendered = output.to_string();
        assert!(
            rendered.contains("truncated") && rendered.contains(&raw.len().to_string()),
            "a prefix must not be shown as the whole: {rendered}",
        );

        // The exact-fit neighbour: retaining everything is not a truncation.
        let exact = ToolOutput::capture(&vec![b'x'; MAX_RETAINED_OUTPUT_BYTES]);
        assert!(!exact.is_truncated());
        assert!(!exact.to_string().contains("truncated"));
    }
}
