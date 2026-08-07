//! The measured half of a conformance run, and the boundary it is bounded to.
//!
//! # A host that cannot measure says so
//!
//! A conformance run has two halves that fail for unrelated reasons. The
//! deterministic half — the corpus, the semantic program, the oracle, the
//! scheduled region, the lowering, and `bfloat` emission — needs nothing but a
//! Rust toolchain and runs on every host. The measured half needs an Apple
//! offline toolchain and a Metal device, and it is bounded to the exact
//! environment row that ran it.
//!
//! A host that offers the second runs both. A host that does not runs the first
//! and reports the measured half as [`MeasuredHalf::Unavailable`], **naming
//! what was missing**. It never skips silently, because a silent skip makes an
//! unmeasured host indistinguishable from a green one and the gate's verdict
//! comes to depend on which machine ran it with nothing saying which; and it
//! never reports a pass it did not observe, because that manufactures evidence
//! for a device that was never reached.
//!
//! [`MeasuredHalf::Failed`] is a third outcome and is not a boundary. It is
//! what a host that *has* the environment reports when a stage it reached said
//! no: the toolchain resolved and the compilation failed, or the device
//! prepared a pipeline and the submission did not complete. Collapsing it into
//! `Unavailable` would let a real defect wear the shape of an absent machine.
//!
//! # Setting `TILER_REQUIRE_METAL_CONFORMANCE`
//!
//! The one supported ambient input, and it can only make the run stricter: with
//! it set, an `Unavailable` outcome is a failure rather than a reported
//! boundary. Nothing here lets an environment variable weaken a check.

use crate::bf16_vertical::{EmittedVertical, OperandStride};

/// The ambient input that turns an unavailable measured half into a failure.
pub(crate) const REQUIRE_MEASUREMENT: &str = "TILER_REQUIRE_METAL_CONFORMANCE";

/// The exact environment row one measured result is bounded to.
///
/// Every field is observed on the host that ran, never transcribed from a
/// record. A reader who is not on this row must treat the measurement as unmade
/// rather than as evidence: nothing here generalizes to another Apple family,
/// another OS or SDK build, another offline compiler, another deployment
/// minimum, or any iOS family.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MeasurementBoundary {
    /// Operating-system family, as the host reports it.
    pub(crate) os_family: String,
    /// Operating-system marketing version.
    pub(crate) os_version: String,
    /// Operating-system build identifier.
    pub(crate) os_build: String,
    /// Host architecture, in the spelling the retained records use.
    pub(crate) architecture: String,
    /// The name the Metal device reports for itself.
    pub(crate) device_name: String,
    /// The highest Apple GPU family the device claims, or why it was not asked.
    pub(crate) gpu_family: String,
    /// The resolved offline `metal` compiler's version banner.
    pub(crate) metal_compiler: String,
    /// The resolved offline `metallib` linker's version banner.
    pub(crate) metallib_linker: String,
    /// The resolved SDK, by canonical name, version, and build.
    pub(crate) sdk: String,
    /// The authoritative target profile the unit was emitted against.
    pub(crate) profile_key: String,
    /// The AOT target triple the unit was compiled for.
    pub(crate) aot_target: String,
    /// The language standard that compilation selected.
    pub(crate) language_standard: String,
    /// Bytes the linked `metallib` occupies.
    pub(crate) metallib_bytes: usize,
}

impl std::fmt::Display for MeasurementBoundary {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            formatter,
            "  host:      {} {} build {} on {}",
            self.os_family, self.os_version, self.os_build, self.architecture,
        )?;
        writeln!(
            formatter,
            "  device:    {}, GPU family {}",
            self.device_name, self.gpu_family,
        )?;
        writeln!(
            formatter,
            "  toolchain: metal {} / metallib {} / SDK {}",
            self.metal_compiler.lines().next().unwrap_or_default(),
            self.metallib_linker.lines().next().unwrap_or_default(),
            self.sdk,
        )?;
        write!(
            formatter,
            "  compiled:  {} for {} under {} ({} metallib byte(s))",
            self.profile_key, self.aot_target, self.language_standard, self.metallib_bytes,
        )
    }
}

/// What one host could establish about the measured half.
///
/// The vocabulary is the same on every host, and that is the point: a
/// non-Apple host states an outcome from this enum rather than having the
/// question compiled away.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "`Ran` and `Failed` are constructible only under cfg(target_os = \"macos\"), where the device and the Apple toolchain exist. They are declared unconditionally so a host without either answers in the same vocabulary — reporting `Unavailable` with its reason — rather than reporting nothing; narrowing the enum by cfg would make the two hosts' outcomes incomparable, which is the silent-skip failure this module exists to prevent."
)]
pub(crate) enum MeasuredHalf {
    /// The device ran the emitted entry point over the corpus.
    Ran {
        /// The environment row this result is bounded to.
        boundary: Box<MeasurementBoundary>,
        /// The BF16 encodings the device wrote back, in corpus order.
        observed: Vec<u16>,
    },
    /// The environment this run needs is not present, and here is what is missing.
    Unavailable(String),
    /// A stage this host reached refused, which is a defect rather than a boundary.
    Failed(String),
}

#[cfg(target_os = "macos")]
mod apple {
    use std::process::Command;

    use metal::Device;
    use tiler_metal_aot::diagnostic::DriverError;
    use tiler_metal_aot::driver::Toolchain;
    use tiler_metal_aot::input::{AppleSdk, CompileRequest, OptimizationLevel};

    use super::{MeasuredHalf, MeasurementBoundary};
    use crate::bf16_vertical::{EmittedVertical, OperandStride, operands, pack, unpack};
    use crate::dispatch::{Launch, Storage, probe_apple_families, run_entry_point};

    /// Reads one `sw_vers` field, or nothing when the tool does not answer.
    ///
    /// A tool that is missing, fails, or prints nothing leaves the field
    /// unobserved rather than supplying a placeholder: a measurement boundary
    /// carrying an invented OS build would be worse than one that says it does
    /// not know.
    fn sw_vers(field: &str) -> String {
        let Ok(output) = Command::new("/usr/bin/sw_vers").arg(field).output() else {
            return "unobserved".to_owned();
        };
        if !output.status.success() {
            return "unobserved".to_owned();
        }
        String::from_utf8(output.stdout).map_or_else(
            |_| "unobserved".to_owned(),
            |value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    "unobserved".to_owned()
                } else {
                    trimmed.to_owned()
                }
            },
        )
    }

    /// Normalizes a Rust architecture name into the spelling the records use.
    ///
    /// Exactly one spelling is mapped; everything else passes through, so an
    /// architecture nobody measured is reported by its own name rather than
    /// renamed into the one the boundary would rather have.
    fn normalized_architecture(arch: &str) -> &str {
        if arch == "aarch64" { "arm64" } else { arch }
    }

    /// Runs the measured half on this host.
    pub(super) fn run(emitted: &EmittedVertical, stride: OperandStride) -> MeasuredHalf {
        let toolchain = Toolchain::system();
        let resolved = match toolchain.resolve(AppleSdk::MacOs) {
            Ok(resolved) => resolved,
            // The narrow absence: only these two mean "no toolchain here".
            Err(
                error @ (DriverError::ToolchainUnavailable { .. }
                | DriverError::SdkUnavailable { .. }),
            ) => {
                return MeasuredHalf::Unavailable(format!(
                    "no qualified Apple Metal toolchain resolved: {error}"
                ));
            }
            // Every other variant means the driver reached the tools and
            // something else went wrong, which is a defect rather than a
            // boundary. Matched exhaustively so a new variant is classified
            // deliberately instead of defaulting into a skip.
            Err(
                error @ (DriverError::ToolFailure { .. }
                | DriverError::Host { .. }
                | DriverError::EmptyArtifact { .. }),
            ) => {
                return MeasuredHalf::Failed(format!(
                    "toolchain resolution failed for a reason that is not an absent toolchain: \
                     {error}"
                ));
            }
        };

        let Some(device) = Device::system_default() else {
            return MeasuredHalf::Unavailable(
                "this host resolves an Apple Metal toolchain and offers no default Metal device"
                    .to_owned(),
            );
        };

        let declaration = &emitted.declaration;
        let request = CompileRequest::new(
            emitted.unit.source(),
            declaration.aot_target(),
            OptimizationLevel::Default,
            declaration.numerical_realization(),
        );
        let compiled = match toolchain.compile(&request) {
            Ok(compiled) => compiled,
            Err(error) => {
                return MeasuredHalf::Failed(format!(
                    "the emitted bfloat unit did not compile and link: {error}"
                ));
            }
        };

        let Some(entry) = emitted.unit.entry_points().first() else {
            return MeasuredHalf::Failed("the emitted unit declares no entry point".to_owned());
        };

        let element_count =
            usize::try_from(emitted.element_count).expect("a corpus element count fits a usize");
        // The result side always uses the carrier's declared width. Only the
        // operand side takes `stride`, because a width error applied to both
        // sides cancels — which is exactly why a layer-local test cannot catch
        // one.
        let declared_bytes = OperandStride::Declared.bytes();
        let storage = Storage {
            operand_bytes: pack(&operands(), stride.bytes()),
            operand_index: emitted.operand_index,
            result_capacity: element_count * declared_bytes,
            result_index: emitted.result_index,
        };
        let launch = Launch {
            grid_threads: emitted.grid_threads,
            threads_per_workgroup: u64::from(emitted.threads_per_workgroup),
        };
        let result_bytes = match run_entry_point(
            &device,
            &compiled.metallib,
            entry.symbol(),
            &storage,
            launch,
        ) {
            Ok(bytes) => bytes,
            Err(error) => {
                return MeasuredHalf::Failed(format!("the dispatch did not complete: {error}"));
            }
        };

        let boundary = MeasurementBoundary {
            os_family: std::env::consts::OS.to_owned(),
            os_version: sw_vers("-productVersion"),
            os_build: sw_vers("-buildVersion"),
            architecture: normalized_architecture(std::env::consts::ARCH).to_owned(),
            device_name: device.name().to_owned(),
            gpu_family: probe_apple_families(&device).to_string(),
            metal_compiler: resolved.metal.version.clone(),
            metallib_linker: resolved.metallib.version.clone(),
            sdk: format!(
                "{} {} build {}",
                resolved.sdk.canonical_name, resolved.sdk.version, resolved.sdk.build,
            ),
            profile_key: declaration.profile().profile_key().as_str().to_owned(),
            aot_target: declaration.aot_target().triple(),
            language_standard: declaration.aot_target().std_token().to_owned(),
            metallib_bytes: compiled.metallib.len(),
        };
        MeasuredHalf::Ran {
            boundary: Box::new(boundary),
            observed: unpack(&result_bytes, declared_bytes, element_count),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod apple {
    use super::MeasuredHalf;
    use crate::bf16_vertical::{EmittedVertical, OperandStride};

    /// Reports the measured half as unavailable, naming why it cannot exist here.
    ///
    /// Not a skip and not a `#[cfg]`-erased test: the deterministic half above
    /// still ran, and this is the outcome the run states about the other one.
    /// The `metal` dependency itself is selected by
    /// `cfg(target_os = "macos")` in this crate's manifest, which is what makes
    /// a non-Apple host able to build and run the first half at all.
    pub(super) fn run(_emitted: &EmittedVertical, _stride: OperandStride) -> MeasuredHalf {
        MeasuredHalf::Unavailable(format!(
            "this host is {}; the Metal binding and the Apple offline toolchain are selected by \
             cfg(target_os = \"macos\"), so no device row exists here to measure",
            std::env::consts::OS,
        ))
    }
}

/// Runs the measured half, or states why this host cannot.
pub(crate) fn measured_half(emitted: &EmittedVertical, stride: OperandStride) -> MeasuredHalf {
    apple::run(emitted, stride)
}
