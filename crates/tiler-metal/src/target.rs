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

use core::fmt;

/// The selected Metal Shading Language standard.
///
/// MSL 3.1 is the standard the Apple artifact-compatibility probe measured. The
/// set is a bounded-profile placeholder and will grow.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
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
        buffer_binding_limit: u32,
    ) -> Self {
        Self {
            language,
            platform,
            deployment_minimum,
            launch_index,
            buffer_binding_limit,
        }
    }
}
