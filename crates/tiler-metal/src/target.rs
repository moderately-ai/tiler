//! Explicit Metal target facts consumed by deterministic source emission.
//!
//! Every fact that can change emitted source is a required field. This module
//! deliberately provides no `Default`: a caller states the language standard,
//! artifact family, deployment minimum, and binding capacity it is emitting
//! for, exactly as `tiler-metal-aot` requires every output-affecting compiler
//! input to be stated. Source-level choices are carried separately by
//! [`crate::target::MetalEmissionRealization`].
//!
//! These are *compile-time* target facts. Prepared-pipeline facts such as
//! `maxTotalThreadsPerThreadgroup`, `threadExecutionWidth`, and
//! `staticThreadgroupMemoryLength` are deliberately absent: the Metal backend
//! contract keys them by device, bundle, entry point, and pipeline descriptor,
//! so a pure emitter cannot and must not decide them.
//!
//! Every public item here is an accepted boundary: Tom accepted the crate's
//! exact public facade on 2026-08-18 under ADR 0075, with the provenance
//! recorded in `tickets/decide-the-tiler-metal-public-facade-surface.md`. That
//! acceptance made the backend spelling helpers crate-private — the launch
//! index's attribute and declared-type text, and the `as_str` of the
//! arithmetic-type and subnormal-behaviour vocabularies — because they expose
//! implementation text or duplicate `Display` without enabling a distinct
//! supported caller.
//! [`MetalSubnormalArithmetic::subnormal_mode`](crate::target::MetalSubnormalArithmetic::subnormal_mode)
//! was separately ratified earlier as the owner-side total projection and is
//! unchanged. Accepted is not stabilized — ADR 0075's pre-alpha posture keeps
//! a later source break cheap, explicit, and reviewed.
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
//! also carries a per-dtype subnormal-arithmetic record and a binding capacity,
//! while [`crate::target::MetalEmissionRealization`] carries source-level lowering choices;
//! none of those facts has any use in a compiler invocation. The
//! driver's `MetalTarget` derives an `AppleSdk`, which selects `xcrun --sdk`,
//! and builds the `air64-apple-*` triple — tool-discovery knowledge this crate
//! must never acquire. Neither record subsumes the other; they overlap in
//! exactly the three facts above.
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

use tiler_ir::schedule::{FlushedZeroSign, SubnormalMode};

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
    /// MSL 1.0.
    Metal1_0,
    /// MSL 1.1.
    Metal1_1,
    /// MSL 1.2.
    Metal1_2,
    /// MSL 2.0.
    Metal2_0,
    /// MSL 2.1.
    Metal2_1,
    /// MSL 2.2.
    Metal2_2,
    /// MSL 2.3.
    Metal2_3,
    /// MSL 2.4.
    Metal2_4,
    /// MSL 3.0, spelled `-std=metal3.0`.
    Metal3_0,
    /// MSL 3.1, spelled `-std=metal3.1`.
    Metal3_1,
    /// MSL 3.2.
    Metal3_2,
    /// MSL 4.0.
    Metal4_0,
}

impl MslLanguageVersion {
    /// Every semantic MSL revision this emission vocabulary names.
    ///
    /// The declared length is `variant_count`, so a standard added to the enum
    /// and not to this list is an array-length error at this declaration. The
    /// exhaustive matches below close themselves, but an array is a
    /// hand-written list and `rustc` has nothing to say about a short one.
    pub const ALL: [Self; core::mem::variant_count::<Self>()] = [
        Self::Metal1_0,
        Self::Metal1_1,
        Self::Metal1_2,
        Self::Metal2_0,
        Self::Metal2_1,
        Self::Metal2_2,
        Self::Metal2_3,
        Self::Metal2_4,
        Self::Metal3_0,
        Self::Metal3_1,
        Self::Metal3_2,
        Self::Metal4_0,
    ];

    /// How many semantic MSL revisions this emission vocabulary names.
    pub const COUNT: usize = Self::ALL.len();

    /// Returns the numeric semantic revision without a compiler-token prefix.
    #[must_use]
    pub const fn revision(self) -> &'static str {
        match self {
            Self::Metal1_0 => "1.0",
            Self::Metal1_1 => "1.1",
            Self::Metal1_2 => "1.2",
            Self::Metal2_0 => "2.0",
            Self::Metal2_1 => "2.1",
            Self::Metal2_2 => "2.2",
            Self::Metal2_3 => "2.3",
            Self::Metal2_4 => "2.4",
            Self::Metal3_0 => "3.0",
            Self::Metal3_1 => "3.1",
            Self::Metal3_2 => "3.2",
            Self::Metal4_0 => "4.0",
        }
    }

    /// Returns the platform-independent semantic spelling.
    #[must_use]
    pub const fn semantic_name(self) -> &'static str {
        match self {
            Self::Metal1_0 => "metal1.0",
            Self::Metal1_1 => "metal1.1",
            Self::Metal1_2 => "metal1.2",
            Self::Metal2_0 => "metal2.0",
            Self::Metal2_1 => "metal2.1",
            Self::Metal2_2 => "metal2.2",
            Self::Metal2_3 => "metal2.3",
            Self::Metal2_4 => "metal2.4",
            Self::Metal3_0 => "metal3.0",
            Self::Metal3_1 => "metal3.1",
            Self::Metal3_2 => "metal3.2",
            Self::Metal4_0 => "metal4.0",
        }
    }
}

impl fmt::Display for MslLanguageVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.semantic_name())
    }
}

/// The Apple artifact family this translation unit is emitted for.
///
/// Family is a compile guarantee, never a live-device fact. Device, simulator,
/// Catalyst, and desktop artifacts remain distinct even where they share an SDK.
///
/// The driver spells the same families as
/// `tiler_metal_aot::input::ApplePlatform`, with the same stable identifiers.
/// It does *not* spell the SDK that selects one; `AppleSdk` is the driver's
/// tool-discovery vocabulary and has no counterpart here.
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
    /// Mac Catalyst.
    MacCatalyst,
    /// tvOS device.
    TvOsDevice,
    /// tvOS simulator.
    TvOsSimulator,
    /// visionOS device.
    VisionOsDevice,
    /// visionOS simulator.
    VisionOsSimulator,
    /// watchOS device.
    WatchOsDevice,
    /// watchOS simulator.
    WatchOsSimulator,
}

impl MetalPlatform {
    /// Every artifact family this emission vocabulary names.
    ///
    /// The declared length is `variant_count`, so a family added to the enum
    /// and not to this list is an array-length error at this declaration
    /// rather than a correspondence test that fails somewhere else.
    pub const ALL: [Self; core::mem::variant_count::<Self>()] = [
        Self::MacOs,
        Self::IOsDevice,
        Self::IOsSimulator,
        Self::MacCatalyst,
        Self::TvOsDevice,
        Self::TvOsSimulator,
        Self::VisionOsDevice,
        Self::VisionOsSimulator,
        Self::WatchOsDevice,
        Self::WatchOsSimulator,
    ];

    /// How many artifact families this emission vocabulary names.
    pub const COUNT: usize = Self::ALL.len();

    /// Returns a stable lowercase identifier for this family.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MacOs => "macos",
            Self::IOsDevice => "ios-device",
            Self::IOsSimulator => "ios-simulator",
            Self::MacCatalyst => "mac-catalyst",
            Self::TvOsDevice => "tvos-device",
            Self::TvOsSimulator => "tvos-simulator",
            Self::VisionOsDevice => "visionos-device",
            Self::VisionOsSimulator => "visionos-simulator",
            Self::WatchOsDevice => "watchos-device",
            Self::WatchOsSimulator => "watchos-simulator",
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

/// One selected MSL realization of the governed global launch index.
///
/// The structured kernel IR types a launch builtin as its governed index role,
/// an unsigned 64-bit integer. MSL 4.0 Table 5.8 permits
/// `[[thread_position_in_grid]]` to be declared as either `ushort` or `uint`
/// scalar/vector forms. The backend selects one admitted declaration here,
/// separately from target facts, and any width difference becomes an explicit
/// widening in emitted source.
///
/// **Measurement.** On Metal 32023.883 (`air64-apple-macos13.0`,
/// `-std=metal3.1`), declaring `[[thread_position_in_grid]]` as `ulong` is
/// rejected: `type 'ulong' (aka 'unsigned long') is not valid for attribute
/// 'thread_position_in_grid'`. Declaring it as `uint` compiles.
///
/// The variant is the whole external vocabulary: a caller selects a
/// realization and reads the selection back structurally, and the emitted
/// source is where the spelling appears. The MSL attribute and declared-type
/// text are crate-private under the accepted facade, so an external caller
/// cannot mint a second spelling authority:
///
/// ```compile_fail,E0624
/// use tiler_metal::target::LaunchIndexRealization;
///
/// let _ = LaunchIndexRealization::ThreadPositionInGridUInt.attribute();
/// ```
///
/// ```compile_fail,E0624
/// use tiler_metal::target::LaunchIndexRealization;
///
/// let _ = LaunchIndexRealization::ThreadPositionInGridUInt.declared_type();
/// ```
///
/// The structured route stays open — the selection is matchable, with the
/// wildcard this `#[non_exhaustive]` vocabulary requires:
///
/// ```
/// use tiler_metal::target::{LaunchIndexRealization, MetalEmissionRealization};
///
/// let emission =
///     MetalEmissionRealization::new(LaunchIndexRealization::ThreadPositionInGridUInt);
/// assert!(matches!(
///     emission.launch_index,
///     LaunchIndexRealization::ThreadPositionInGridUInt,
/// ));
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LaunchIndexRealization {
    /// `[[thread_position_in_grid]]` declared as MSL `uint`.
    ThreadPositionInGridUInt,
}

impl LaunchIndexRealization {
    /// Returns the MSL attribute spelling this realization uses.
    ///
    /// Crate-private under the accepted facade: the spelling is emission's
    /// authority, and a caller that could read it apart from emitted source
    /// could also restate it.
    #[must_use]
    pub(crate) const fn attribute(self) -> &'static str {
        match self {
            Self::ThreadPositionInGridUInt => "thread_position_in_grid",
        }
    }

    /// Returns the MSL type the attributed parameter is declared with.
    ///
    /// Crate-private for the same reason as [`Self::attribute`].
    #[must_use]
    pub(crate) const fn declared_type(self) -> &'static str {
        match self {
            Self::ThreadPositionInGridUInt => "uint",
        }
    }
}

/// Source-level choices used to emit one Metal translation unit.
///
/// These are realizations selected by the backend, not capabilities inferred
/// from an Apple artifact family or a device. Keeping them separate from
/// [`MetalTargetFacts`] prevents a selected launch-parameter spelling from
/// being mistaken for proof of integer arithmetic width, device-address width,
/// or concrete launch capacity.
///
/// This is a caller-constructed input record, so it deliberately exposes its
/// field and is not `#[non_exhaustive]`: adding another required realization is
/// a construction-site change either way.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MetalEmissionRealization {
    /// The selected declaration of the governed global launch index.
    pub launch_index: LaunchIndexRealization,
}

impl MetalEmissionRealization {
    /// Assembles the complete source-emission realization.
    #[must_use]
    pub const fn new(launch_index: LaunchIndexRealization) -> Self {
        Self { launch_index }
    }
}

/// One floating-point arithmetic type whose subnormal behaviour is a fact of
/// its own.
///
/// Subnormal behaviour is *not* a single property of a target. On the measured
/// Apple row `f32` arithmetic flushes and `f16` arithmetic preserves, on the
/// same GPU, in the same math modes, from modules that declare
/// `air.compile.denorms_disable` identically — so one dtype-free declaration is
/// false for one of the two whichever way it is set. This enum is the key of
/// [`MetalSubnormalArithmeticFacts`], which states one behaviour per type and
/// answers `Unknown` for a type nothing has measured.
///
/// The set is deliberately open. `f64` where a target has it, and every integer
/// and quantized format, are unmeasured; dtypes disagreeing establishes that the
/// flush *depends on* the dtype and establishes nothing about which dtypes
/// flush. Adding a variant is a build error at this type's private `index` map
/// and at its crate-private `ALL` inventory, never a silent inheritance of
/// another type's fact.
///
/// These are MSL arithmetic types, not [`tiler_ir::kernel::KernelType`] values.
/// The structured kernel IR resolves one floating-point element type today;
/// this vocabulary is the target's, and it is what the *measurements* are
/// indexed by.
///
/// The `ALL`/`COUNT` inventory and the `as_str` text are crate-private under
/// the accepted facade — the census sizes this crate's own fact record, and
/// the stable text is what [`fmt::Display`] already renders:
///
/// ```compile_fail,E0624
/// use tiler_metal::target::MetalFloatArithmeticType;
///
/// let _ = MetalFloatArithmeticType::F32.as_str();
/// ```
///
/// ```compile_fail,E0624
/// use tiler_metal::target::MetalFloatArithmeticType;
///
/// let _ = MetalFloatArithmeticType::ALL;
/// ```
///
/// ```compile_fail,E0624
/// use tiler_metal::target::MetalFloatArithmeticType;
///
/// let _ = MetalFloatArithmeticType::COUNT;
/// ```
///
/// ```
/// use tiler_metal::target::MetalFloatArithmeticType;
///
/// assert_eq!(MetalFloatArithmeticType::Bf16.to_string(), "bf16");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MetalFloatArithmeticType {
    /// MSL `float`, IEEE-754 binary32.
    F32,
    /// MSL `half`, IEEE-754 binary16.
    F16,
    /// MSL `bfloat`, `bfloat16` — `f32` truncated to its high 16 bits.
    ///
    /// Named third because it is what distinguished the two mechanisms `f16`
    /// alone could not: every `bf16` subnormal *is* an `f32` subnormal, where
    /// every `f16` subnormal is an `f32` normal.
    Bf16,
}

impl MetalFloatArithmeticType {
    /// Every arithmetic type this vocabulary names, in canonical order.
    ///
    /// The order is the derived one, so it agrees with the `Ord` a `BTreeSet`
    /// of these uses and with the order emission reports them in.
    ///
    /// The declared length is `variant_count`, so a type added to the enum and
    /// not to this list is an array-length error at this declaration. That
    /// matters beyond the list itself: [`Self::COUNT`] sizes the
    /// [`MetalSubnormalArithmeticFacts`] slot array, so a short `ALL` would
    /// leave the new type with no slot to state a fact in.
    pub(crate) const ALL: [Self; core::mem::variant_count::<Self>()] =
        [Self::F32, Self::F16, Self::Bf16];

    /// How many arithmetic types this vocabulary names.
    pub(crate) const COUNT: usize = Self::ALL.len();

    /// Returns a stable lowercase identifier for this arithmetic type.
    ///
    /// Crate-private under the accepted facade; `Display` renders this text.
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::F32 => "f32",
            Self::F16 => "f16",
            Self::Bf16 => "bf16",
        }
    }

    /// Returns this type's position in [`Self::ALL`].
    ///
    /// Written as an exhaustive match rather than read from the discriminant,
    /// so adding or reordering a variant is a build error here instead of a
    /// silent re-keying of every stated fact.
    /// `every_arithmetic_type_indexes_to_its_own_slot` proves the map is a
    /// bijection onto `0..COUNT`.
    const fn index(self) -> usize {
        match self {
            Self::F32 => 0,
            Self::F16 => 1,
            Self::Bf16 => 2,
        }
    }
}

impl fmt::Display for MetalFloatArithmeticType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query for a subnormal-arithmetic fact the target has not stated.
///
/// This is the `Unknown` class, and it is neither of the two behaviours. A
/// consumer that holds a fact for one arithmetic type and needs another must
/// fail closed on this rather than read the fact it does have: the two measured
/// types disagree, so substituting one for the other is a guess wearing a
/// measurement's clothes.
///
/// Kept as a reason type rather than folded into a behaviour variant for the
/// same purpose [`MetalSubnormalArithmetic`] separates a flush from
/// preservation — an unmeasured type has no behaviour to report, and an
/// `Unknown` that could be pattern-matched as a behaviour would be honoured by
/// something.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MetalUnstatedSubnormalArithmetic {
    arithmetic_type: MetalFloatArithmeticType,
}

impl MetalUnstatedSubnormalArithmetic {
    pub(crate) const fn for_type(arithmetic_type: MetalFloatArithmeticType) -> Self {
        Self { arithmetic_type }
    }

    /// Returns the arithmetic type no fact was stated for.
    #[must_use]
    pub const fn arithmetic_type(self) -> MetalFloatArithmeticType {
        self.arithmetic_type
    }

    /// Returns the stable rule identifier for this unstated fact.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        "unstated-subnormal-arithmetic"
    }
}

impl fmt::Display for MetalUnstatedSubnormalArithmetic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.rule(), self.arithmetic_type)
    }
}

/// How one arithmetic type treats subnormal operands and results on a target.
///
/// This is a hard feasibility fact about the target, not a compiler-flag
/// choice, so it belongs beside the binding capacity rather than in the
/// numerical requirement set. A realization that demands subnormal preservation
/// is unrealizable on a target that flushes *that arithmetic type*, for any
/// kernel that performs arithmetic in it, and emission reports that as a
/// [`MetalNumericalGap`](crate::record::MetalNumericalGap) instead of quietly
/// naming a compiler flag that does not deliver it.
///
/// The behaviour carries no dtype of its own: it is the *value* of a
/// [`MetalSubnormalArithmeticFacts`] entry, and the entry's
/// [`MetalFloatArithmeticType`] key is the dtype. See that record for the
/// per-type measurements.
///
/// The `as_str` text is crate-private under the accepted facade; the
/// structured variants and the accepted [`Self::subnormal_mode`] projection
/// are the consumption routes, and [`fmt::Display`] renders the same stable
/// text:
///
/// ```compile_fail,E0624
/// use tiler_metal::target::MetalSubnormalArithmetic;
///
/// let _ = MetalSubnormalArithmetic::PreservesSubnormals.as_str();
/// ```
///
/// ```
/// use tiler_metal::target::MetalSubnormalArithmetic;
///
/// assert_eq!(
///     MetalSubnormalArithmetic::PreservesSubnormals.to_string(),
///     "preserves-subnormals",
/// );
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MetalSubnormalArithmetic {
    /// Arithmetic in this type flushes subnormal operands and subnormal
    /// results to the stated zero.
    ///
    /// The zero is a field rather than an implied `+0.0` because a flush that
    /// does not say which zero it produces cannot establish a declared
    /// `SubnormalMode::FlushToZero`, which always names one. Stating it is what
    /// turns a sign-matching flush into a positive conformance claim and leaves
    /// only a sign *mismatch* as a gap.
    FlushesToZero {
        /// The zero this target's flush produces.
        zero_sign: MetalFlushedZeroSign,
    },
    /// Arithmetic in this type preserves subnormal operands and results
    /// exactly.
    PreservesSubnormals,
}

impl MetalSubnormalArithmetic {
    /// Projects this target fact into the shared numerical-behaviour vocabulary.
    ///
    /// This owner-side projection is total even though the target vocabulary is
    /// deliberately `#[non_exhaustive]` to downstream crates. Consumers can
    /// therefore declare compiler honourability without a wildcard that would
    /// guess what a future Metal behaviour means.
    #[must_use]
    pub const fn subnormal_mode(self) -> SubnormalMode {
        match self {
            Self::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::PreservesSign,
            } => SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            Self::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::AlwaysPositive,
            } => SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
            Self::PreservesSubnormals => SubnormalMode::Preserve,
        }
    }

    /// Returns a stable lowercase identifier for this behaviour.
    ///
    /// Crate-private under the accepted facade; `Display` renders this text.
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::PreservesSign,
            } => "flushes-to-zero-preserving-sign",
            Self::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::AlwaysPositive,
            } => "flushes-to-zero-always-positive",
            Self::PreservesSubnormals => "preserves-subnormals",
        }
    }
}

/// The target's subnormal-arithmetic behaviour, stated per arithmetic type.
///
/// A type with no entry is `Unknown`, which is a third class beside the two
/// behaviours and never a default for either. [`Self::behaviour`] returns
/// [`MetalUnstatedSubnormalArithmetic`] for it, and emission carries that
/// through to a fail-closed conformance rejection rather than reading a
/// neighbouring type's fact.
///
/// # Measurement — `f32`
///
/// On an Apple M4 Max under macOS 27.0 (build 26A5388g) with Metal 32023.883,
/// an emitted `x * 2.0f` returns `0x00000000` for the operand `0x00400000`, and
/// an emitted `x * 0.5f` returns `0x00000000` for the operand `0x00800000`.
/// Both hold for every `-fmetal-math-mode` (`safe`, `relaxed`, `fast`), every
/// `-O` level (`0`, `1`, `2`, `3`, `s`), and through both the offline `xcrun
/// metal` driver and runtime `MTLCompileOptions` compilation. A load/store
/// round trip with no arithmetic returns every subnormal input unchanged, so
/// the flush is a property of arithmetic, not of materialization.
///
/// # Measurement — `f16`
///
/// On the same host, GPU, and toolchain row, every dimension the `f32` kernels
/// isolate returns the **preserved** value when the identical kernel is spelled
/// at `f16` width: `multiply_two_f16` returns `0400` for `0200`,
/// `multiply_half_f16` returns `0200` for `0400`, `add_smallest_normal_f16`
/// returns `0200` for `8200`, and `divide_by_three_f16` returns `0155` for
/// `0400`. Each holds under `safe`, `relaxed`, and `fast`, on both compilation
/// paths, and on both dispatchable families (`MacOs`, `IOsSimulator`). Each
/// `f16` kernel carries an execution witness on a non-subnormal operand that
/// reports `executed` in every configuration, which matters here in a way it
/// does not at `f32` width: `preserved` is also what a kernel whose arithmetic
/// was optimized away would report, so without the witness the observation and
/// the trap are the same word. Finding 21 of the [Apple numerical behaviour
/// record](../../../docs/research/apple-targets/numerical-behaviour.md) owns
/// both rows.
///
/// # Fact — the modules do not predict this
///
/// `air.compile.denorms_disable` is emitted for the `f16` kernels and the `f32`
/// kernels alike, under all three math modes and for all three families. The
/// declaration is a compile-side fact about what was *requested*; only a
/// witnessed dispatch says what was delivered. Nothing readable on the compile
/// side would have caught the divergence.
///
/// # Measurement — `bf16`, and it is macOS-only
///
/// On the same host, GPU, and toolchain row, every flush dimension returns the
/// **flushed** value at `bfloat16` width, in all three math modes, at `-O0` and
/// `-O2`, and on both compilation paths: `0040` → `0000` and `0080` → `0000`,
/// and the sign rows `8040` → `8000` twice over, which is what makes the zero a
/// measured `PreservesSign` rather than an assumed `+0.0`. Each verdict carries
/// an execution witness reporting `executed`, and `materialize_bf16` returns all
/// eight operands unchanged, so the zeros are the output of arithmetic that ran
/// rather than of a buffer round trip that normalized. Finding 24 of the [Apple
/// numerical behaviour
/// record](../../../docs/research/apple-targets/numerical-behaviour.md) owns it.
///
/// **`bf16` is `Unknown` for both iOS families, for two different reasons.** The
/// iOS Simulator compiles and links every `bfloat` module and then refuses to
/// create a pipeline for it — `XPC_ERROR_CONNECTION_INTERRUPTED`, on both the
/// offline and runtime paths, and for an arithmetic-free `materialize_bf16` too,
/// so the refusal is about the format and not about an operation. `IOsDevice`
/// was never asked. Finding 26 owns the refusal and does not diagnose its cause.
/// So a caller states this row only for a macOS target, which is why the
/// platform dimension stays on [`MetalTargetFacts`] rather than being duplicated
/// inside each dtype row.
///
/// # Fact — what three dtypes establish
///
/// That the flush depends on the dtype. Not which dtypes flush. `f64` and every
/// integer and quantized format are unmeasured, and a fourth dtype could agree
/// with any measured one, so they are `Unknown` here rather than assumed. The
/// `IOsDevice` family's arithmetic is unmeasured for all three types alike — its
/// compile side agrees and nothing dispatched it — so a caller stating facts for
/// that family states them from the same inference it already makes for `f32`.
///
/// **The mechanism is narrowed and not settled.** One explanation — narrow
/// arithmetic evaluated at `f32` precision and rounded once — predicts all three
/// measured dtypes with no free parameter, and the competing native-support
/// explanation survives only weakened to a per-format claim this record has no
/// independent evidence for. It is *not* separated from native `bfloat16`
/// arithmetic flushing at its own boundary, and no single operation can separate
/// them: a value is `bf16`-subnormal exactly when it is `f32`-subnormal, and
/// `f32`'s 24-bit significand exceeds the 18 bits that would make a second
/// rounding to `bfloat16`'s 8-bit significand innocuous. A two-operation chain
/// with a rounding-sensitive intermediate would; none has been measured. Nothing
/// here may be read as a rule for a dtype nobody measured.
///
/// # Fact — which way a wrong reading errs
///
/// Reading the `f32` fact for `f16` arithmetic **over-rejects**: it reports a
/// [`SubnormalFlushInArithmetic`](crate::record::MetalNumericalGap::SubnormalFlushInArithmetic)
/// gap against a subnormal the device carries exactly, refusing a plan that is
/// correct. It becomes a *wrong tensor* only where a reference evaluation
/// flushes to match a device that does not — so the reference-side reading is
/// the dangerous one, and this record's job is to stop either side inheriting a
/// fact it was not given.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MetalSubnormalArithmeticFacts {
    stated: [Option<MetalSubnormalArithmetic>; MetalFloatArithmeticType::COUNT],
}

impl MetalSubnormalArithmeticFacts {
    /// A record that states nothing: every arithmetic type is `Unknown`.
    ///
    /// This is the starting point for [`Self::stating`], and it is also the
    /// honest record for a target whose subnormal behaviour nobody has
    /// measured. Emission refuses to claim conformance for arithmetic in any
    /// type it does not find here.
    #[must_use]
    pub const fn unmeasured() -> Self {
        Self {
            stated: [None; MetalFloatArithmeticType::COUNT],
        }
    }

    /// Returns this record with one arithmetic type's measured behaviour added.
    ///
    /// # Panics
    ///
    /// Panics if `arithmetic_type` was already stated. Two statements about one
    /// type are two claims, and silently keeping either would drop a
    /// measurement; a caller assembling a target profile from more than one
    /// source must reconcile them before stating. In a `const` context this is
    /// a compile error.
    #[must_use]
    pub const fn stating(
        mut self,
        arithmetic_type: MetalFloatArithmeticType,
        behaviour: MetalSubnormalArithmetic,
    ) -> Self {
        let index = arithmetic_type.index();
        assert!(
            self.stated[index].is_none(),
            "a subnormal-arithmetic fact was stated twice for one arithmetic type"
        );
        self.stated[index] = Some(behaviour);
        self
    }

    /// Returns the stated behaviour of one arithmetic type.
    ///
    /// # Errors
    ///
    /// Returns [`MetalUnstatedSubnormalArithmetic`] when this record states
    /// nothing about `arithmetic_type`. That is `Unknown`, not a behaviour: the
    /// measured types disagree, so there is no neighbouring fact a caller may
    /// substitute and no direction that fails safe by default.
    pub const fn behaviour(
        self,
        arithmetic_type: MetalFloatArithmeticType,
    ) -> Result<MetalSubnormalArithmetic, MetalUnstatedSubnormalArithmetic> {
        match self.stated[arithmetic_type.index()] {
            Some(behaviour) => Ok(behaviour),
            None => Err(MetalUnstatedSubnormalArithmetic { arithmetic_type }),
        }
    }
}

/// Which zero a target's subnormal flush produces.
///
/// The counterpart of `tiler_ir::schedule::FlushedZeroSign` on the target side.
/// They are separate types because one is a *declaration a program makes* and
/// the other a *fact a target states*, and `tiler-metal` must be able to
/// compare them rather than assume they agree.
///
/// **Measurement.** On an Apple M4 Max under macOS 27.0 (build 26A5388g) with
/// Metal 32023.883, an emitted `x * 2.0f` returns `0x80000000` for the operand
/// `0x80400000`, not `0x00000000`. The governed Apple flush is therefore
/// sign-preserving, and a program that asks for `AlwaysPositive` is *not*
/// honoured by it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MetalFlushedZeroSign {
    /// The produced zero carries the sign of the value it replaced.
    PreservesSign,
    /// Every flushed value produces positive zero regardless of its own sign.
    AlwaysPositive,
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
    /// How the target's arithmetic treats subnormals, stated per floating-point
    /// type.
    ///
    /// One value per arithmetic type rather than one for the target, because
    /// the measured Apple row flushes in `f32` and preserves in `f16`. A type
    /// with no entry is `Unknown` and is rejected at the conformance claim
    /// rather than defaulted to either behaviour.
    pub subnormal_arithmetic: MetalSubnormalArithmeticFacts,
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
        subnormal_arithmetic: MetalSubnormalArithmeticFacts,
        buffer_binding_limit: u32,
    ) -> Self {
        Self {
            language,
            platform,
            deployment_minimum,
            subnormal_arithmetic,
            buffer_binding_limit,
        }
    }
}
