//! The measured half of a conformance run, and the boundary it is bounded to.
//!
//! # A host that cannot measure says so
//!
//! A conformance run has two halves that fail for unrelated reasons. The
//! deterministic half — the corpus, the semantic program, the oracle, the
//! scheduled region, the lowering, and emission — needs nothing but a Rust
//! toolchain and runs on every host. The measured half needs an Apple offline
//! toolchain and a Metal device, and it is bounded to the exact environment row
//! that ran it.
//!
//! A host that offers the second runs both. A host that does not runs the first
//! and reports the measured half as [`Measured::Unavailable`], **naming what was
//! missing**. It never skips silently, because a silent skip makes an unmeasured
//! host indistinguishable from a green one and the gate's verdict comes to
//! depend on which machine ran it with nothing saying which; and it never
//! reports a pass it did not observe, because that manufactures evidence for a
//! device that was never reached.
//!
//! [`Measured::Failed`] is a third outcome and is not a boundary. It is what a
//! host that *has* the environment reports when a stage it reached said no: the
//! toolchain resolved and the compilation failed, or the device prepared a
//! pipeline and the submission did not complete. Collapsing it into
//! `Unavailable` would let a real defect wear the shape of an absent machine.
//!
//! # One vocabulary for every vertical
//!
//! [`Measured`] is generic over what a run observed rather than fixed to one
//! vertical's result type, and [`require_or_report`] is the single place the
//! three outcomes are turned into a verdict. Two verticals each spelling their
//! own three-outcome enum and their own reporting rule is exactly the shape that
//! drifts: one of them acquires a silent skip and nothing compares the two. The
//! rule is stated once and every run is held to it.
//!
//! # The caller owns what unavailability means, and the gate reads nothing
//!
//! [`Measured`] states what a host could establish about the measured half.
//! [`HostPolicy`] states what a caller does about it. [`apply_policy`] is a pure
//! function of the two and reads no ambient input at all, which is what makes
//! both policies observable in one process against a single identical outcome.
//!
//! The ambient hardening input `TILER_REQUIRE_METAL_CONFORMANCE` is **retained**,
//! and it lives in [`ambient`], whose entire content is that one read. It is
//! retained because this gate's real callers are Rust test functions, which take
//! no arguments: a policy literal at those call sites would be fixed at compile
//! time, at either `Require` — reddening the workspace gate on every host
//! without an Apple toolchain and a Metal device, when the whole point of the
//! section above is that such a host still runs the deterministic half — or
//! `Report`, which discards the ability to make an unmeasurable host a hard
//! failure. An ambient input is the only channel a human has into a test
//! function's policy. What changed is that it no longer reaches the reporting
//! path, so a policy read drifting back in is a red census rather than an
//! invisible regression.
//!
//! Nothing here lets an ambient input weaken a check: an unset variable resolves
//! to [`HostPolicy::Report`], which is already the weaker of the two.

pub(crate) mod ambient;
#[cfg(test)]
mod tests;

use crate::bf16_vertical::{EmittedVertical, OperandStride};

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
    /// Bytes the linked `metallib`s this measurement compiled occupy in total.
    ///
    /// A total rather than one number because a vertical may link more than one
    /// unit — the serial-sum portfolio links one per retained alternative — and
    /// a single-unit run is the one-element case of the same sum. **Zero is the
    /// empty case rather than a missing value**, and it is what the envelope
    /// route reports: that route loads the object bytes the artifact carries and
    /// compiles nothing of its own, so a nonzero figure there would name a unit
    /// no device ran.
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

/// Why one host could not measure, in that host's own words.
///
/// A type rather than a bare `String` so an unavailability is distinguishable
/// from every other message this gate carries, and so the outcomes that hold one
/// can state which comparisons they admit. Two unavailabilities do compare —
/// their reasons are ordinary data — but neither [`Measured`] nor [`Reported`]
/// compares at all, so no unavailability ever reaches a comparison against a
/// pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Unavailable {
    /// What this host could not supply.
    reason: String,
}

impl Unavailable {
    /// States why this host could not measure.
    pub(crate) fn new(reason: String) -> Self {
        Self { reason }
    }

    /// What was missing, as the host reported it.
    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }
}

impl std::fmt::Display for Unavailable {
    /// Renders the bare reason, with no prefix of its own.
    ///
    /// The two sentences that carry one — the reported boundary notice and the
    /// required-host failure — supply their own framing, and both are quoted
    /// verbatim as evidence on landed tickets.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.reason)
    }
}

/// What a caller does when this host could not measure.
///
/// The judgement is always the caller's: [`Measured`] reports what the host
/// could establish and never what that means to a run. Neither value is read
/// here — [`ambient::require_measurement_policy`] is the one resolver this crate
/// offers, and calling it is a caller's decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostPolicy {
    /// Unavailability fails the run, on the named authority.
    Require {
        /// What required the measured half.
        ///
        /// [`Refused::Required`] reads `<named> is set and …`, so this is the
        /// name of an ambient input rather than an arbitrary authority, and
        /// [`ambient::require_measurement_policy`] supplies the only one this
        /// crate has.
        named: &'static str,
    },
    /// Unavailability is a boundary the run reports and does not pass.
    Report,
}

/// What one host could establish about the measured half of one run.
///
/// The vocabulary is the same on every host and for every vertical, and that is
/// the point: a non-Apple host states an outcome from this enum rather than
/// having the question compiled away, and two verticals state theirs in one
/// vocabulary rather than in two that can drift apart.
///
/// **Deliberately without `PartialEq`, `Eq`, or `Default`.** An unavailable
/// outcome must have no path by which it compares equal to, or degrades into, a
/// measured one, and the cheapest way to guarantee that is a type that offers no
/// comparison and no fabricable value at all. `measurement::tests` censuses this
/// derive, so restoring one of them is a red test rather than a silent widening.
#[derive(Clone, Debug)]
#[cfg_attr(
    not(target_os = "macos"),
    allow(
        dead_code,
        reason = "`Ran` and `Failed` are constructed only under cfg(target_os = \"macos\"), where the device and the Apple toolchain exist. They are declared unconditionally so a host without either answers in the same vocabulary — reporting `Unavailable` with its reason — rather than reporting nothing; narrowing the enum by cfg would make the two hosts' outcomes incomparable, which is the silent-skip failure this module exists to prevent. Negated rather than unconditional: an Apple host does construct both, so an allow that also covered it would absorb a variant that had genuinely stopped being reachable there."
    )
)]
pub(crate) enum Measured<T> {
    /// The device ran, and here is the row the result is bounded to.
    Ran {
        /// The environment row this result is bounded to.
        boundary: Box<MeasurementBoundary>,
        /// What the run observed, in the vertical's own terms.
        observed: T,
    },
    /// The environment this run needs is not present, and here is what is missing.
    Unavailable(Unavailable),
    /// A stage this host reached refused, which is a defect rather than a boundary.
    Failed(String),
}

/// What one caller's policy made of one measured half.
///
/// **Deliberately without `PartialEq`, `Eq`, or `Default`**, and with no
/// accessor that yields an observation for a host that could not measure. An
/// unavailable outcome cannot be compared equal to a measured one because the
/// type offers no comparison at all, and [`Self::observed`] answers `None` for
/// one, so every expression that reaches a device result has destructured a
/// completion first. `measurement::tests` censuses the derive.
#[derive(Clone, Debug)]
pub(crate) enum Reported<T> {
    /// The device ran; here is the row it is bounded to and what it observed.
    Observed {
        /// The environment row this result is bounded to.
        boundary: Box<MeasurementBoundary>,
        /// What the run observed, in the vertical's own terms.
        observed: T,
    },
    /// This host could not measure, and this caller reports the boundary.
    Boundary(Unavailable),
}

impl<T> Reported<T> {
    /// What the device observed, and `None` for a host that could not measure.
    pub(crate) fn observed(self) -> Option<T> {
        match self {
            Self::Observed { observed, .. } => Some(observed),
            Self::Boundary(_) => None,
        }
    }
}

/// Why one caller's policy refused a measured half.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Refused {
    /// The caller required the measured half and this host could not supply it.
    Required {
        /// The authority that required it, as [`HostPolicy::Require`] named it.
        named: &'static str,
        /// What this host could not supply.
        unavailable: Unavailable,
    },
    /// A stage this host reached refused, which is a defect rather than a boundary.
    Defect(String),
}

impl std::fmt::Display for Refused {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required { named, unavailable } => write!(
                formatter,
                "{named} is set and the measured half is unavailable: {unavailable}",
            ),
            Self::Defect(detail) => write!(
                formatter,
                "the measured half reached its environment and refused: {detail}",
            ),
        }
    }
}

/// Applies one caller's host policy to one measured half.
///
/// Pure, total, and free of ambient input, which is what lets one identical
/// [`Measured`] value be watched under both policies in a single process with
/// nothing set. [`Measured::Failed`] refuses under either policy, because a host
/// that reached its environment and was refused has found a defect rather than a
/// boundary; collapsing it into an unavailability would let a real defect wear
/// the shape of an absent machine.
pub(crate) fn apply_policy<T>(
    policy: HostPolicy,
    outcome: Measured<T>,
) -> Result<Reported<T>, Refused> {
    match outcome {
        Measured::Ran { boundary, observed } => Ok(Reported::Observed { boundary, observed }),
        Measured::Unavailable(unavailable) => match policy {
            HostPolicy::Require { named } => Err(Refused::Required { named, unavailable }),
            HostPolicy::Report => Ok(Reported::Boundary(unavailable)),
        },
        Measured::Failed(detail) => Err(Refused::Defect(detail)),
    }
}

/// Reports one measured half under the caller's policy, or fails the run.
///
/// Returns what the run observed when the device ran. **A skip is impossible**:
/// either the boundary is printed or the reason it does not exist is, and under
/// [`HostPolicy::Require`] the second fails the run instead. A caller that
/// receives `None` has had the boundary reported for it and states nothing
/// further; it never receives a value the device did not produce.
///
/// # Panics
///
/// Panics on every [`Refused`] [`apply_policy`] returns: when the measured half
/// [`Measured::Failed`], and when the caller's policy is
/// [`HostPolicy::Require`] and the outcome is [`Measured::Unavailable`].
pub(crate) fn require_or_report<T>(
    policy: HostPolicy,
    label: &str,
    outcome: Measured<T>,
) -> Option<T> {
    match apply_policy(policy, outcome) {
        Ok(Reported::Observed { boundary, observed }) => {
            eprintln!("{label}: measured on this row:\n{boundary}");
            Some(observed)
        }
        Ok(Reported::Boundary(unavailable)) => {
            eprintln!(
                "{label}: MEASUREMENT BOUNDARY UNAVAILABLE — {unavailable}. The deterministic \
                 half above ran; nothing here claims a device result.",
            );
            None
        }
        Err(refused) => panic!("{label}: {refused}"),
    }
}

#[cfg(target_os = "macos")]
pub(crate) mod host {
    //! Resolving the Apple environment every measured half needs, once.
    //!
    //! Split out because the measured verticals need the same three things — a
    //! qualified offline toolchain, a default Metal device, and the environment
    //! row a result is bounded to — and each of them has exactly one correct
    //! answer to "is an absent toolchain a boundary or a defect?". Answering it
    //! in one place is what stops a later vertical from classifying a
    //! `ToolFailure` as an absent machine.

    use metal::Device;
    use tiler_build::BoundMetalCompileDeclaration;
    use tiler_metal_aot::diagnostic::DriverError;
    use tiler_metal_aot::driver::Toolchain;
    use tiler_metal_aot::input::AppleSdk;
    use tiler_metal_aot::record::ResolvedToolchain;

    use super::{MeasurementBoundary, Unavailable};
    use crate::applicability::{describe_probed_gpu_family, normalized_architecture, sw_vers};
    use crate::dispatch::probe_apple_families;

    /// Everything a measured half needs from this host, resolved before it runs.
    pub(crate) struct AppleHost {
        /// The offline driver, for compiling emitted units.
        pub(crate) toolchain: Toolchain,
        /// What that driver resolved: the two tools and the SDK.
        pub(crate) resolved: ResolvedToolchain,
        /// This host's default Metal device.
        pub(crate) device: Device,
    }

    /// Why a measured half could not start here.
    ///
    /// Two outcomes rather than one string, because the caller must report an
    /// absent machine as a boundary and a refused stage as a defect, and a type
    /// that could not tell them apart would let one wear the other's shape.
    pub(crate) enum Unresolved {
        /// This host does not have the environment; that is a boundary.
        Absent(Unavailable),
        /// The environment is here and a stage refused; that is a defect.
        Defect(String),
    }

    /// Resolves the Apple environment, or states why this host cannot measure.
    pub(crate) fn resolve() -> Result<AppleHost, Unresolved> {
        let toolchain = Toolchain::system();
        let resolved = match toolchain.resolve(AppleSdk::MacOs) {
            Ok(resolved) => resolved,
            // The narrow absence: only these two mean "no toolchain here".
            Err(
                error @ (DriverError::ToolchainUnavailable { .. }
                | DriverError::SdkUnavailable { .. }),
            ) => {
                return Err(Unresolved::Absent(Unavailable::new(format!(
                    "no qualified Apple Metal toolchain resolved: {error}"
                ))));
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
                return Err(Unresolved::Defect(format!(
                    "toolchain resolution failed for a reason that is not an absent toolchain: \
                     {error}"
                )));
            }
        };
        let Some(device) = Device::system_default() else {
            return Err(Unresolved::Absent(Unavailable::new(
                "this host resolves an Apple Metal toolchain and offers no default Metal device"
                    .to_owned(),
            )));
        };
        Ok(AppleHost {
            toolchain,
            resolved,
            device,
        })
    }

    /// Reads one `sw_vers` field, or records that it was not observed.
    ///
    /// A tool that is missing, fails, or prints nothing leaves the field
    /// unobserved rather than supplying a placeholder: a measurement boundary
    /// carrying an invented OS build would be worse than one that says it does
    /// not know. The same reading serves the applicability observation, where
    /// the absence is a typed unanswered predicate rather than a string.
    fn observed_field(field: &str) -> String {
        sw_vers(field).unwrap_or_else(|| "unobserved".to_owned())
    }

    /// Builds the row a result measured on this host is bounded to.
    pub(crate) fn boundary(
        host: &AppleHost,
        declaration: &BoundMetalCompileDeclaration,
        metallib_bytes: usize,
    ) -> MeasurementBoundary {
        MeasurementBoundary {
            os_family: std::env::consts::OS.to_owned(),
            os_version: observed_field("-productVersion"),
            os_build: observed_field("-buildVersion"),
            architecture: normalized_architecture(std::env::consts::ARCH).to_owned(),
            device_name: host.device.name().to_owned(),
            gpu_family: describe_probed_gpu_family(probe_apple_families(&host.device)),
            metal_compiler: host.resolved.metal.version.clone(),
            metallib_linker: host.resolved.metallib.version.clone(),
            sdk: format!(
                "{} {} build {}",
                host.resolved.sdk.canonical_name,
                host.resolved.sdk.version,
                host.resolved.sdk.build,
            ),
            profile_key: declaration.profile().profile_key().as_str().to_owned(),
            aot_target: declaration.aot_target().triple(),
            language_standard: declaration.aot_target().std_token().to_owned(),
            metallib_bytes,
        }
    }
}

#[cfg(target_os = "macos")]
mod apple {
    use tiler_metal_aot::input::{CompileRequest, OptimizationLevel};

    use super::host::{self, Unresolved};
    use super::{Measured, MeasurementBoundary};
    use crate::bf16_vertical::{EmittedVertical, OperandStride, operands, pack, unpack};
    use crate::dispatch::{Launch, Storage, run_entry_point};

    /// Runs the BF16 vertical's measured half on this host.
    pub(super) fn run(emitted: &EmittedVertical, stride: OperandStride) -> Measured<Vec<u16>> {
        let apple = match host::resolve() {
            Ok(apple) => apple,
            Err(Unresolved::Absent(reason)) => return Measured::Unavailable(reason),
            Err(Unresolved::Defect(detail)) => return Measured::Failed(detail),
        };

        let declaration = &emitted.declaration;
        let request = CompileRequest::new(
            emitted.unit.source(),
            declaration.aot_target(),
            OptimizationLevel::Default,
            declaration.numerical_realization(),
        );
        let compiled = match apple.toolchain.compile(&request) {
            Ok(compiled) => compiled,
            Err(error) => {
                return Measured::Failed(format!(
                    "the emitted bfloat unit did not compile and link: {error}"
                ));
            }
        };

        let Some(entry) = emitted.unit.entry_points().first() else {
            return Measured::Failed("the emitted unit declares no entry point".to_owned());
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
            &apple.device,
            &compiled.metallib,
            entry.symbol(),
            &storage,
            launch,
        ) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Measured::Failed(format!("the dispatch did not complete: {error}"));
            }
        };

        let boundary: MeasurementBoundary =
            host::boundary(&apple, declaration, compiled.metallib.len());
        Measured::Ran {
            boundary: Box::new(boundary),
            observed: unpack(&result_bytes, declared_bytes, element_count),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod apple {
    use super::Measured;
    use crate::bf16_vertical::{EmittedVertical, OperandStride};

    /// Reports the measured half as unavailable, naming why it cannot exist here.
    ///
    /// Not a skip and not a `#[cfg]`-erased test: the deterministic half above
    /// still ran, and this is the outcome the run states about the other one.
    /// The `metal` dependency itself is selected by
    /// `cfg(target_os = "macos")` in this crate's manifest, which is what makes
    /// a non-Apple host able to build and run the first half at all.
    pub(super) fn run(_emitted: &EmittedVertical, _stride: OperandStride) -> Measured<Vec<u16>> {
        Measured::Unavailable(crate::measurement::absent_apple_row())
    }
}

/// States why no device row exists on a host that is not Apple's.
///
/// Written once because the measured verticals report it, and a reader comparing two
/// unavailable outcomes should be reading one sentence rather than deciding
/// whether two spellings mean the same thing.
#[cfg_attr(
    target_os = "macos",
    allow(
        dead_code,
        reason = "the non-Apple unavailability sentence has no caller on macOS, where every vertical resolves a real row or reports a resolution failure instead. It is compiled on both hosts so the two spellings cannot drift."
    )
)]
pub(crate) fn absent_apple_row() -> Unavailable {
    Unavailable::new(format!(
        "this host is {}; the Metal binding and the Apple offline toolchain are selected by \
         cfg(target_os = \"macos\"), so no device row exists here to measure",
        std::env::consts::OS,
    ))
}

/// Runs the BF16 vertical's measured half, or states why this host cannot.
pub(crate) fn measured_half(
    emitted: &EmittedVertical,
    stride: OperandStride,
) -> Measured<Vec<u16>> {
    apple::run(emitted, stride)
}
