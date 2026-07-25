//! Explicit Metal target facts consumed by deterministic source emission.
//!
//! Every fact that can change emitted source is a required field. This module
//! deliberately provides no `Default`: a caller states the language standard,
//! artifact family, deployment minimum, launch-index realization, and binding
//! capacity it is emitting for, exactly as `tiler-metal-aot` requires every
//! output-affecting compiler input to be stated.
//!
//! These are *compile-time* target facts. Prepared-pipeline facts such as
//! `maxTotalThreadsPerThreadgroup`, `threadExecutionWidth`, and
//! `staticThreadgroupMemoryLength` are deliberately absent: the Metal backend
//! contract keys them by device, bundle, entry point, and pipeline descriptor,
//! so a pure emitter cannot and must not decide them.
//!
//! Every public item here is a reviewed *draft* boundary (ADR 0074 §7): the
//! surface is built and tested at full fidelity while the facade is under
//! review, and it says so rather than pretending to be accepted.
//!
//! # The Apple target vocabulary is deliberately owned twice
//!
//! [`MslLanguageVersion`](crate::target::MslLanguageVersion),
//! [`MetalPlatform`](crate::target::MetalPlatform), and
//! [`MetalDeploymentMinimum`](crate::target::MetalDeploymentMinimum)
//! have counterparts in `tiler_metal_aot::input`: `MslVersion`,
//! `ApplePlatform`, and `DeploymentMinimum`. They describe the same three facts
//! about the same targets. That duplication is a decision recorded by
//! `choose-one-owner-for-apple-target-vocabulary`, not an accident to be
//! consolidated by the next reader who notices it.
//!
//! **Why neither crate owns both copies.** `tiler-metal-aot` has an empty
//! dependency closure on purpose: it spawns `xcrun` and its entire value is
//! being a small shim whose exact compiler invocation can be audited in
//! isolation. `tiler-metal` depends on `tiler-ir` and `tiler-artifact`, so
//! giving the driver this crate's vocabulary would pull the whole lowering
//! stack into the build graph of the component that runs the compiler. Pointing
//! the edge the other way — a normal `tiler-metal` → `tiler-metal-aot`
//! dependency — puts Apple tool discovery into every consumer's build graph,
//! and Cargo's cycle rule would then forbid the eventual `tiler-metal-aot` →
//! `tiler-metal` production direction outright. A third crate owning
//! three types would leave both crates still owning the rest of their target
//! vocabulary and buy no invariant the checked correspondence does not already
//! give.
//!
//! **Why the two records are not one type in disguise.**
//! [`MetalTargetFacts`](crate::target::MetalTargetFacts)
//! also carries a launch-index realization, a subnormal-arithmetic fact, and a
//! binding capacity, none of which a compiler invocation has any use for. The
//! driver's `MetalTarget` carries an `AppleSdk`, which selects `xcrun --sdk` and
//! builds the `air64-apple-*` triple — tool-discovery knowledge this crate must
//! never acquire. Neither record subsumes the other; they overlap in exactly the
//! three facts above.
//!
//! **What keeps them from drifting.** `crate::target_correspondence` maps each
//! vocabulary onto the other *totally*, so a language standard or an artifact
//! family added to either crate fails this crate's build until the other gains
//! it. That check can only live here: the driver cannot see this crate, so this
//! crate's development dependency on the driver is the sole edge in the
//! workspace over which both vocabularies are visible at once. It is also the
//! reason the correspondence is a test rather than a conversion function — a
//! production `MetalTargetFacts` → `MetalTarget` translation would need a normal
//! dependency in one direction or the other, so it belongs to whichever
//! component eventually orchestrates emission and compilation together.

use core::fmt;

/// The selected Metal Shading Language standard.
///
/// MSL 3.1 is the standard the Apple artifact-compatibility probe measured. The
/// set is a bounded-profile placeholder and will grow.
///
/// This is the standard the *emitted source declares it was written against*.
/// The driver's `tiler_metal_aot::input::MslVersion` is the standard a
/// compilation is *invoked* with, and the two must name the same set; see this
/// module's documentation for why they are separate types and
/// `crate::target_correspondence` for the check that keeps them in step.
///
/// **This is an ADR 0074 convention 5b type and is deliberately exhaustive.**
/// It was 5a — and carried `#[non_exhaustive]` — only while no consumer outside
/// this crate mapped it. The bundle assembler now derives a `MetalTarget` from
/// [`MetalTargetFacts`] out of crate, and a wildcard arm there could only invent
/// a `-std` token the variant alone determines, which would let a bundle's
/// provenance header and its actual compilation disagree about what it is. 5b
/// resolves that against convention 3 by keeping the enum exhaustive, so adding
/// a standard is a build failure at every out-of-crate map rather than a silent
/// mistranslation. Adding a variant here is source-breaking for those maps by
/// design; `cargo check` enumerates them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MslLanguageVersion {
    /// MSL 3.0, spelled `-std=metal3.0`.
    Metal3_0,
    /// MSL 3.1, spelled `-std=metal3.1`.
    Metal3_1,
}

impl MslLanguageVersion {
    /// Returns the `-std` value token for this language version.
    ///
    /// The token is emitted into the translation unit's provenance header so a
    /// reader of the source can see the standard it was written against.
    #[must_use]
    pub const fn std_token(self) -> &'static str {
        match self {
            Self::Metal3_0 => "metal3.0",
            Self::Metal3_1 => "metal3.1",
        }
    }
}

impl fmt::Display for MslLanguageVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.std_token())
    }
}

/// The Apple artifact family this translation unit is emitted for.
///
/// Family is a compile guarantee, never a live-device fact, and macOS, iOS
/// device, and iOS simulator remain distinct measured families.
///
/// The driver spells the same three families as
/// `tiler_metal_aot::input::ApplePlatform`, with the same stable identifiers.
/// It does *not* spell the SDK that selects one; `AppleSdk` is the driver's
/// tool-discovery vocabulary and has no counterpart here. Mac Catalyst is a
/// deferred fourth family in both crates and must be added to both at once —
/// `crate::target_correspondence` fails the build otherwise.
///
/// The convention 5b classification and the reasoning are the same as
/// [`MslLanguageVersion`]'s: an out-of-crate wildcard could only invent an
/// `AppleSdk`, so this enum is deliberately exhaustive rather than
/// `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MetalPlatform {
    /// macOS.
    MacOs,
    /// iOS device.
    IOsDevice,
    /// iOS simulator.
    IOsSimulator,
}

impl MetalPlatform {
    /// Returns a stable lowercase identifier for this family.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MacOs => "macos",
            Self::IOsDevice => "ios-device",
            Self::IOsSimulator => "ios-simulator",
        }
    }
}

impl fmt::Display for MetalPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A declared lower runtime boundary as a `<major>.<minor>` deployment minimum.
///
/// This is the requested application deployment minimum, recorded in the
/// emitted provenance header. It is not evidence that the compiled library runs
/// on every operating-system version at or above it.
///
/// The driver's `tiler_metal_aot::input::DeploymentMinimum` holds the same two
/// components and renders them the same way, but reaches a different output:
/// this one is written into the emitted header, that one into the
/// `air64-apple-*` target triple. `crate::target_correspondence` asserts the
/// components and the rendering agree, which is what makes the header's claim
/// about the compilation true rather than decorative.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MetalDeploymentMinimum {
    major: u16,
    minor: u16,
}

impl MetalDeploymentMinimum {
    /// Creates a deployment minimum from an explicit major and minor version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the major version component.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor version component.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

impl fmt::Display for MetalDeploymentMinimum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// How the target delivers the governed global launch index to a kernel.
///
/// The structured kernel IR types a launch builtin as its governed index role,
/// an unsigned 64-bit integer. Metal delivers the launch position through an
/// attributed parameter whose declared type is fixed by the language, so the
/// realization is a target fact rather than a lowering choice, and any width
/// difference becomes an explicit widening in emitted source.
///
/// **Measurement.** On Metal 32023.883 (`air64-apple-macos13.0`,
/// `-std=metal3.1`), declaring `[[thread_position_in_grid]]` as `ulong` is
/// rejected: `type 'ulong' (aka 'unsigned long') is not valid for attribute
/// 'thread_position_in_grid'`. Declaring it as `uint` compiles.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LaunchIndexRealization {
    /// `[[thread_position_in_grid]]` declared as MSL `uint`.
    ThreadPositionInGridUInt,
}

impl LaunchIndexRealization {
    /// Returns the MSL attribute spelling this realization uses.
    #[must_use]
    pub const fn attribute(self) -> &'static str {
        match self {
            Self::ThreadPositionInGridUInt => "thread_position_in_grid",
        }
    }

    /// Returns the MSL type the attributed parameter is declared with.
    #[must_use]
    pub const fn declared_type(self) -> &'static str {
        match self {
            Self::ThreadPositionInGridUInt => "uint",
        }
    }

    /// Returns the largest launch index this realization can deliver.
    ///
    /// A dispatch whose grid exceeds this bound cannot address every invocation
    /// through this builtin. Launch geometry is not part of a verified kernel,
    /// so this bound is stated in the emitted provenance header as a launch
    /// precondition rather than checked here.
    #[must_use]
    pub const fn maximum_index(self) -> u64 {
        match self {
            Self::ThreadPositionInGridUInt => u32::MAX as u64,
        }
    }
}

/// How the target's `f32` arithmetic treats subnormal operands and results.
///
/// This is a hard feasibility fact about the target, not a compiler-flag
/// choice, so it belongs beside the binding capacity rather than in the
/// numerical requirement set. A realization that demands subnormal preservation
/// is unrealizable on a flushing target for any kernel that performs `f32`
/// arithmetic, and emission reports that as a
/// [`MetalNumericalGap`](crate::record::MetalNumericalGap) instead of quietly
/// naming a compiler flag that does not deliver it.
///
/// **Measurement.** On an Apple M4 Max under macOS 27.0 (build 26A5388g) with
/// Metal 32023.883, an emitted `x * 1.0` returns `0x00000000` for the operand
/// `0x00000001`, and an emitted `x * 0.5` returns `0x00000000` for the operand
/// `0x00800000`. Both hold for every `-fmetal-math-mode` (`safe`, `relaxed`,
/// `fast`), every `-O` level (`0`, `1`, `2`, `3`, `s`), and through both the
/// offline `xcrun metal` driver and runtime `MTLCompileOptions` compilation. A
/// load/store round trip with no arithmetic returns every subnormal input
/// unchanged, so the flush is a property of arithmetic, not of materialization.
/// The front end emits `air.compile.denorms_disable` under every one of those
/// flag combinations, and no `metal` driver flag was found that clears it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MetalSubnormalArithmetic {
    /// `f32` arithmetic flushes subnormal operands and subnormal results to
    /// zero. This is the measured behaviour of every governed Apple family.
    FlushesToZero,
    /// `f32` arithmetic preserves subnormal operands and results exactly.
    ///
    /// No governed Apple family has been measured to do this. The variant
    /// exists so the flushing fact is a stated target property that emission
    /// checks, rather than an assumption compiled into the backend.
    PreservesSubnormals,
}

impl MetalSubnormalArithmetic {
    /// Returns a stable lowercase identifier for this behaviour.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FlushesToZero => "flushes-to-zero",
            Self::PreservesSubnormals => "preserves-subnormals",
        }
    }
}

impl fmt::Display for MetalSubnormalArithmetic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The complete set of Metal target facts one translation unit is emitted for.
///
/// This is a caller-constructed input record, so it exposes `pub` fields and is
/// deliberately not `#[non_exhaustive]`: growing it is a construction-site
/// change either way, and a caller must be able to write the literal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MetalTargetFacts {
    /// The selected Metal Shading Language standard.
    pub language: MslLanguageVersion,
    /// The Apple artifact family the translation unit targets.
    pub platform: MetalPlatform,
    /// The requested deployment minimum.
    pub deployment_minimum: MetalDeploymentMinimum,
    /// How the target delivers the governed global launch index.
    pub launch_index: LaunchIndexRealization,
    /// How the target's `f32` arithmetic treats subnormals.
    pub subnormal_arithmetic: MetalSubnormalArithmetic,
    /// Buffer argument-table entries this emission may address.
    ///
    /// Apple's feature tables state 31 entries per compute function for every
    /// current family. Tiler does not derive the value; a caller states the
    /// capacity of the profile it is emitting for, and a signature needing more
    /// bindings is rejected rather than emitted with an unaddressable
    /// `[[buffer(N)]]` attribute.
    pub buffer_binding_limit: u32,
}

impl MetalTargetFacts {
    /// Assembles the complete target facts for one translation unit.
    #[must_use]
    pub const fn new(
        language: MslLanguageVersion,
        platform: MetalPlatform,
        deployment_minimum: MetalDeploymentMinimum,
        launch_index: LaunchIndexRealization,
        subnormal_arithmetic: MetalSubnormalArithmetic,
        buffer_binding_limit: u32,
    ) -> Self {
        Self {
            language,
            platform,
            deployment_minimum,
            launch_index,
            subnormal_arithmetic,
            buffer_binding_limit,
        }
    }
}
