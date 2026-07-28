//! Explicit compilation inputs for the offline Apple Metal driver.
//!
//! Every output-affecting choice is a strongly typed field. The driver never
//! inherits the Metal compiler's math or optimization defaults; the numerical
//! realization flags are required inputs, so
//! [`NumericalRealization`](crate::input::NumericalRealization) deliberately has
//! no `Default` implementation.
//!
//! # The Apple target vocabulary is deliberately owned twice
//!
//! [`MslVersion`](crate::input::MslVersion),
//! [`ApplePlatform`](crate::input::ApplePlatform), and
//! [`DeploymentMinimum`](crate::input::DeploymentMinimum) have
//! counterparts in `tiler_metal::target`: `MslLanguageVersion`,
//! `MetalPlatform`, and `MetalDeploymentMinimum`. They describe the same three
//! facts about the same targets. That duplication is a decision recorded by
//! `choose-one-owner-for-apple-target-vocabulary`, not an accident to be
//! consolidated by the next reader who notices it.
//!
//! **Why this crate keeps its own copy.** This crate has an empty dependency
//! closure on purpose. It spawns `xcrun` and its whole value is being a small
//! shim whose exact compiler invocation can be read and audited without the
//! lowering stack behind it. `tiler-metal` depends on `tiler-ir` and
//! `tiler-artifact`, so importing its vocabulary would pull both into the build
//! graph of the component that runs the compiler, and would do so to obtain
//! three enums. Reversing the edge is worse: a normal `tiler-metal` →
//! `tiler-metal-aot` dependency puts Apple tool discovery into every consumer's
//! build graph, and Cargo's cycle rule would then forbid the
//! `tiler-metal-aot` → `tiler-metal` direction that the existing
//! development-only edge exists to keep available.
//!
//! **These types are matched across the crate boundary, so they stay
//! exhaustive.** `tiler-metal` maps [`MslVersion`](crate::input::MslVersion)
//! and [`ApplePlatform`](crate::input::ApplePlatform) onto
//! its own vocabulary *totally*, in a test module reached through its
//! development dependency on this crate. Under ADR 0074 convention 5b that
//! makes both 5b types: marking either `#[non_exhaustive]` would force a
//! wildcard arm into a map whose every arm must produce the counterpart the
//! variant alone determines, and a wildcard there could only invent a family or
//! a language standard. A variant added to either enum is meant to fail
//! `tiler-metal`'s build until the emitter gains the matching one.
//!
//! Nothing else in this module is constrained that way. A caller outside this
//! crate constructs [`AppleSdk`](crate::input::AppleSdk),
//! [`OptimizationLevel`](crate::input::OptimizationLevel), and the input
//! records without matching them, which is ADR 0074 convention 5a.

use core::fmt;

/// One selected Apple SDK used to discover the offline Metal tools.
///
/// An SDK is not an artifact-family authority: macOS and Mac Catalyst both use
/// `macosx` while producing different target triples. [`ApplePlatform`] owns
/// the artifact family and derives its SDK selector.
/// # Growth
///
/// `#[non_exhaustive]`, because every out-of-crate use of this type names a
/// variant to pass in — `tiler-metal`'s golden compilation and both serial-sum
/// prototypes — and none classifies one by an exhaustive match. Admitting an SDK
/// is therefore additive for a consumer and a compile error for every total map
/// *inside* this crate, which is the asymmetry ADR 0074 asks for.
///
/// An out-of-crate match must carry a wildcard:
///
/// ```compile_fail,E0004
/// use tiler_metal_aot::input::AppleSdk;
/// fn selector(sdk: AppleSdk) -> &'static str {
///     match sdk {
///         AppleSdk::MacOs => "macosx",
///         AppleSdk::IPhoneOs => "iphoneos",
///         AppleSdk::IPhoneSimulator => "iphonesimulator",
///     }
/// }
/// ```
///
/// while naming a variant to construct one still compiles:
///
/// ```
/// use tiler_metal_aot::input::AppleSdk;
/// let sdk = AppleSdk::MacOs;
/// assert_eq!(sdk, AppleSdk::MacOs);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AppleSdk {
    /// The `macosx` SDK used by macOS and Mac Catalyst.
    MacOs,
    /// The `iphoneos` SDK: the iOS device artifact family.
    IPhoneOs,
    /// The `iphonesimulator` SDK: the iOS simulator artifact family.
    IPhoneSimulator,
    /// The `appletvos` SDK.
    AppleTvOs,
    /// The `appletvsimulator` SDK.
    AppleTvSimulator,
    /// The `xros` SDK.
    XrOs,
    /// The `xrsimulator` SDK.
    XrSimulator,
    /// The `watchos` SDK.
    WatchOs,
    /// The `watchsimulator` SDK.
    WatchSimulator,
}

impl AppleSdk {
    /// Every SDK selector this toolchain vocabulary currently names.
    pub const ALL: [Self; 9] = [
        Self::MacOs,
        Self::IPhoneOs,
        Self::IPhoneSimulator,
        Self::AppleTvOs,
        Self::AppleTvSimulator,
        Self::XrOs,
        Self::XrSimulator,
        Self::WatchOs,
        Self::WatchSimulator,
    ];

    /// How many SDK selectors this toolchain vocabulary currently names.
    pub const COUNT: usize = Self::ALL.len();

    #[cfg(test)]
    const fn index(self) -> usize {
        match self {
            Self::MacOs => 0,
            Self::IPhoneOs => 1,
            Self::IPhoneSimulator => 2,
            Self::AppleTvOs => 3,
            Self::AppleTvSimulator => 4,
            Self::XrOs => 5,
            Self::XrSimulator => 6,
            Self::WatchOs => 7,
            Self::WatchSimulator => 8,
        }
    }

    /// Returns the canonical `xcrun --sdk` selector for this SDK.
    #[must_use]
    pub const fn selector(self) -> &'static str {
        match self {
            Self::MacOs => "macosx",
            Self::IPhoneOs => "iphoneos",
            Self::IPhoneSimulator => "iphonesimulator",
            Self::AppleTvOs => "appletvos",
            Self::AppleTvSimulator => "appletvsimulator",
            Self::XrOs => "xros",
            Self::XrSimulator => "xrsimulator",
            Self::WatchOs => "watchos",
            Self::WatchSimulator => "watchsimulator",
        }
    }
}

/// The Apple artifact family a compiled `metallib` belongs to.
///
/// This is the family a compilation *produces*, and
/// `tiler_metal::target::MetalPlatform` is the family emitted source
/// *declares*, with the same variants and the same stable identifiers.
/// Adding a family here without adding it there fails `tiler-metal`'s build.
/// Do not mark this `#[non_exhaustive]`: see this module's documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApplePlatform {
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

impl ApplePlatform {
    /// Every artifact family this compiler-target vocabulary names.
    pub const ALL: [Self; 10] = [
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

    /// How many artifact families this compiler-target vocabulary names.
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

    /// Returns the SDK selector used to compile this artifact family.
    #[must_use]
    pub const fn sdk(self) -> AppleSdk {
        match self {
            Self::MacOs | Self::MacCatalyst => AppleSdk::MacOs,
            Self::IOsDevice => AppleSdk::IPhoneOs,
            Self::IOsSimulator => AppleSdk::IPhoneSimulator,
            Self::TvOsDevice => AppleSdk::AppleTvOs,
            Self::TvOsSimulator => AppleSdk::AppleTvSimulator,
            Self::VisionOsDevice => AppleSdk::XrOs,
            Self::VisionOsSimulator => AppleSdk::XrSimulator,
            Self::WatchOsDevice => AppleSdk::WatchOs,
            Self::WatchOsSimulator => AppleSdk::WatchSimulator,
        }
    }

    const fn triple_os(self) -> &'static str {
        match self {
            Self::MacOs => "macos",
            Self::MacCatalyst | Self::IOsDevice | Self::IOsSimulator => "ios",
            Self::TvOsDevice | Self::TvOsSimulator => "tvos",
            Self::VisionOsDevice | Self::VisionOsSimulator => "xros",
            Self::WatchOsDevice | Self::WatchOsSimulator => "watchos",
        }
    }

    const fn triple_suffix(self) -> &'static str {
        match self {
            Self::MacCatalyst => "-macabi",
            Self::IOsSimulator
            | Self::TvOsSimulator
            | Self::VisionOsSimulator
            | Self::WatchOsSimulator => "-simulator",
            Self::MacOs
            | Self::IOsDevice
            | Self::TvOsDevice
            | Self::VisionOsDevice
            | Self::WatchOsDevice => "",
        }
    }
}

/// A declared lower runtime boundary as a `<major>.<minor>` deployment minimum.
///
/// This is the requested application deployment minimum and a compiler input. It
/// is not evidence that a produced `metallib` runs on every OS at or above it;
/// deployment minimum, platform family, language features, and live GPU
/// capabilities remain independent compatibility dimensions.
///
/// It reaches the `air64-apple-*` target triple through [`MetalTarget::triple`].
/// `tiler_metal::target::MetalDeploymentMinimum` holds the same two components
/// and renders them the same way, but reaches an emitted provenance header
/// instead. Both spellings are asserted to agree from `tiler-metal`, which is
/// what makes that header's claim about this compilation true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeploymentMinimum {
    major: u16,
    minor: u16,
}

impl DeploymentMinimum {
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

impl fmt::Display for DeploymentMinimum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// The selected Metal Shading Language standard.
///
/// MSL 3.1 is the standard the Apple artifact-compatibility probe measured.
///
/// This is the standard a compilation is *invoked* with;
/// `tiler_metal::target::MslLanguageVersion` is the standard emitted source
/// *declares it was written against*, with the same variants and the same
/// `-std` tokens. Adding a standard here without adding it there fails
/// `tiler-metal`'s build. Do not mark this `#[non_exhaustive]`: see this
/// module's documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MslVersion {
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

impl MslVersion {
    /// Every semantic MSL revision this toolchain vocabulary names.
    pub const ALL: [Self; 12] = [
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

    /// How many semantic MSL revisions this toolchain vocabulary names.
    pub const COUNT: usize = Self::ALL.len();

    /// Returns the platform-independent semantic revision.
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

    /// Returns the stable platform-independent semantic spelling.
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

/// The selected Metal optimization level.
///
/// Optimization is output-affecting and therefore an explicit input. `-O2` is
/// the level the Apple artifact-compatibility probe measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OptimizationLevel {
    /// `-O0`: no optimization.
    None,
    /// `-O1`: limited optimization.
    Less,
    /// `-O2`: default optimization.
    Default,
    /// `-O3`: aggressive optimization.
    Aggressive,
    /// `-Os`: optimize for size.
    Size,
}

impl OptimizationLevel {
    /// Returns the `-O` flag token for this level.
    #[must_use]
    pub const fn flag(self) -> &'static str {
        match self {
            Self::None => "-O0",
            Self::Less => "-O1",
            Self::Default => "-O2",
            Self::Aggressive => "-O3",
            Self::Size => "-Os",
        }
    }
}

/// The `-fmetal-math-mode` selection.
///
/// **Measurement.** `xcrun --sdk macosx metal --help` on Metal 32023.883 states
/// "Default is 'fast'". Omitting the flag therefore selects the most relaxed
/// mode, which is why this is a required input with no `Default` rather than
/// something the driver may leave unstated.
///
/// **Measurement.** Compiling with `-S -emit-llvm` on that toolchain shows the
/// licences each mode applies to every emitted `f32` operation: `safe` applies
/// none, `relaxed` applies `reassoc nsz arcp afn`, and `fast` applies
/// `reassoc nnan ninf nsz arcp afn`. `relaxed` and `fast` also differ in the
/// recorded `air.compile_options`: `fast_math_disable` against
/// `fast_math_enable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathMode {
    /// `safe`: no numerically unsafe relaxations.
    Safe,
    /// `relaxed`: permit relaxed math transforms.
    Relaxed,
    /// `fast`: permit fast, numerically unsafe math.
    Fast,
}

impl MathMode {
    /// Returns the `-fmetal-math-mode` value token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Relaxed => "relaxed",
            Self::Fast => "fast",
        }
    }

    /// Returns whether this mode applies LLVM's `reassoc`, `nsz`, `arcp`, and
    /// `afn` licences to emitted `f32` operations.
    #[must_use]
    pub const fn relaxes_floating_point(self) -> bool {
        match self {
            Self::Safe => false,
            Self::Relaxed | Self::Fast => true,
        }
    }

    /// Returns whether this mode additionally assumes operands and results are
    /// neither NaN nor infinite (`nnan` and `ninf`).
    ///
    /// Under that assumption an arithmetic result that is a NaN has no defined
    /// value, so no emitted operation can canonicalize it. The driver's
    /// documented `relaxed` behaviour — "preserves infs and nans" — is why this
    /// is a narrower question than [`Self::relaxes_floating_point`].
    #[must_use]
    pub const fn assumes_finite(self) -> bool {
        match self {
            Self::Safe | Self::Relaxed => false,
            Self::Fast => true,
        }
    }
}

/// The `-fmetal-math-fp32-functions` selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fp32Functions {
    /// `precise`: precise 32-bit floating-point library functions.
    Precise,
    /// `fast`: fast, lower-accuracy 32-bit floating-point library functions.
    Fast,
}

impl Fp32Functions {
    /// Returns the `-fmetal-math-fp32-functions` value token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Precise => "precise",
            Self::Fast => "fast",
        }
    }
}

/// The `-ffp-contract` selection controlling floating-point contraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FpContract {
    /// `off`: never form fused multiply-add contractions.
    Off,
    /// `on`: contract within a single language statement.
    On,
    /// `fast`: contract freely across statements.
    Fast,
    /// `fast-honor-pragmas`: contract freely except where source pragmas forbid it.
    FastHonorPragmas,
}

impl FpContract {
    /// Returns the `-ffp-contract` value token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::On => "on",
            Self::Fast => "fast",
            Self::FastHonorPragmas => "fast-honor-pragmas",
        }
    }

    /// Returns whether this selection may fuse a multiply and an add that are
    /// written as two separate statements.
    ///
    /// This is the question a source emitter can act on: writing every
    /// arithmetic operation as its own statement defends against `on` but not
    /// against `fast`.
    ///
    /// **Measurement.** On an Apple M4 Max under macOS 27.0 (build 26A5388g)
    /// with Metal 32023.883, `v5 = v3 * 1.5f; v8 = v5 + 1.0f;` written as two
    /// statements returns the separately rounded `0x3fc58f9e` for the operand
    /// `0x3eb97ef9` under `off` and `on`, and the fused `0x3fc58f9d` under
    /// `fast`.
    #[must_use]
    pub const fn contracts_across_statements(self) -> bool {
        match self {
            Self::Off | Self::On => false,
            Self::Fast | Self::FastHonorPragmas => true,
        }
    }
}

/// The explicit numerical realization flags for one compilation.
///
/// These flags are required inputs. The driver never falls back to the Metal
/// compiler's math defaults; an unavailable realization rejects the request
/// rather than silently producing differently rounded arithmetic. Reassociation,
/// approximate functions, and contraction stay independent permissions rather
/// than one `fast` bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NumericalRealization {
    /// The `-fmetal-math-mode` selection.
    pub math_mode: MathMode,
    /// The `-fmetal-math-fp32-functions` selection.
    pub fp32_functions: Fp32Functions,
    /// The `-ffp-contract` selection.
    pub fp_contract: FpContract,
}

impl NumericalRealization {
    /// Constructs an explicit numerical realization from its three permissions.
    #[must_use]
    pub const fn new(
        math_mode: MathMode,
        fp32_functions: Fp32Functions,
        fp_contract: FpContract,
    ) -> Self {
        Self {
            math_mode,
            fp32_functions,
            fp_contract,
        }
    }

    /// The strict baseline governed for the local Metal 32023.883 toolchain row:
    /// `safe` math mode, `precise` 32-bit functions, and disabled contraction.
    ///
    /// This is a named explicit choice, not an inherited default: the compiler's
    /// own defaults are `fast` math mode and statement-scoped contraction.
    ///
    /// It is the strictest realization the offline driver can select. It is not
    /// full IEEE-754 binary32 conformance; see [`Self::preserves_f32_subnormals`].
    #[must_use]
    pub const fn strict_baseline() -> Self {
        Self::new(MathMode::Safe, Fp32Functions::Precise, FpContract::Off)
    }

    /// Returns whether this realization preserves the sign of a zero result.
    ///
    /// **Measurement.** On an Apple M4 Max under macOS 27.0 (build 26A5388g)
    /// with Metal 32023.883, an emitted `x * 1.0f` followed by `+ 0.0f` returns
    /// `0x00000000` for the operand `0x80000000` under `-fmetal-math-mode=safe`
    /// and `0x80000000` under both `relaxed` and `fast`. IEEE-754
    /// round-to-nearest requires `0x00000000`. The divergence holds at `-O0`,
    /// `-O1`, `-O2`, `-O3`, and `-Os`, and is independent of
    /// `-fmetal-math-fp32-functions`.
    #[must_use]
    pub const fn preserves_signed_zero(self) -> bool {
        !self.math_mode.relaxes_floating_point()
    }

    /// Returns whether this realization permits reordering a reduction.
    ///
    /// Kept separate from [`Self::preserves_signed_zero`] even though today's
    /// three modes answer both with one bit: `nsz` and `reassoc` are distinct
    /// LLVM licences that a future mode could select independently, and the
    /// numerical contract asks the two questions separately.
    #[must_use]
    pub const fn permits_reassociation(self) -> bool {
        self.math_mode.relaxes_floating_point()
    }

    /// Returns whether an arithmetic NaN keeps a defined value under this
    /// realization.
    ///
    /// When it does not, a NaN-canonicalizing operation has nothing well
    /// defined to map, however the source spells the NaN test.
    #[must_use]
    pub const fn preserves_nan_results(self) -> bool {
        !self.math_mode.assumes_finite()
    }

    /// Returns whether this realization preserves subnormal `f32` operands and
    /// results through arithmetic. It never does.
    ///
    /// This is a hard feasibility limit of the Apple GPU families, not a flag
    /// choice, so no selection makes it true. It is stated as a method rather
    /// than left implicit so a caller checking a declared numerical contract
    /// against this realization gets a definite answer instead of assuming the
    /// strictest flags imply IEEE-754 subnormal behaviour.
    ///
    /// **The `f32` in the name is load-bearing and is not a spelling of
    /// "floating-point".** The same hardware, in the same math modes, from
    /// modules declaring `air.compile.denorms_disable` identically, *preserves*
    /// subnormals through `f16` arithmetic — finding 21 of the [Apple numerical
    /// behaviour record](../../../docs/research/apple-targets/numerical-behaviour.md)
    /// measures it with an execution witness on both dispatchable families.
    /// There is deliberately no `f16` counterpart here and no dtype-free
    /// predicate: a caller needing another width's answer must state which
    /// width, and the driver has no measurement to give it for `bf16` or any
    /// other unmeasured format.
    ///
    /// **Measurement.** On an Apple M4 Max under macOS 27.0 (build 26A5388g)
    /// with Metal 32023.883, `x * 1.0f` returns `0x00000000` for the operand
    /// `0x00000001` and `x * 0.5f` returns `0x00000000` for the operand
    /// `0x00800000`, under every `-fmetal-math-mode`, every `-O` level, and
    /// through both this offline driver and runtime `MTLCompileOptions`
    /// compilation. The front end records `air.compile.denorms_disable` under
    /// every one of those combinations, and neither `-fdenormal-fp-math=ieee`
    /// nor any other `metal` driver flag was found to clear it. A load/store
    /// round trip with no arithmetic returns every subnormal bit pattern
    /// unchanged, so materialization is unaffected.
    #[must_use]
    pub const fn preserves_f32_subnormals(self) -> bool {
        false
    }

    /// Returns the exact ordered numerical compiler flags for this realization.
    #[must_use]
    pub fn flags(self) -> [String; 3] {
        [
            format!("-fmetal-math-mode={}", self.math_mode.token()),
            format!(
                "-fmetal-math-fp32-functions={}",
                self.fp32_functions.token()
            ),
            format!("-ffp-contract={}", self.fp_contract.token()),
        ]
    }
}

/// A platform, language revision, and deployment minimum do not form a valid target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetalTargetError {
    /// The language revision has no governed spelling for this platform.
    LanguageUnavailable {
        /// Requested artifact family.
        platform: ApplePlatform,
        /// Requested semantic MSL revision.
        language: MslVersion,
    },
    /// The requested deployment minimum predates the language revision.
    DeploymentMinimumTooLow {
        /// Requested artifact family.
        platform: ApplePlatform,
        /// Requested semantic MSL revision.
        language: MslVersion,
        /// Requested minimum.
        requested: DeploymentMinimum,
        /// Earliest permitted minimum.
        required: DeploymentMinimum,
    },
}

impl fmt::Display for MetalTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LanguageUnavailable { platform, language } => write!(
                formatter,
                "MSL {} is not admitted for {}",
                language.revision(),
                platform.as_str()
            ),
            Self::DeploymentMinimumTooLow {
                platform,
                language,
                requested,
                required,
            } => write!(
                formatter,
                "MSL {} on {} requires deployment minimum {required}, got {requested}",
                language.revision(),
                platform.as_str()
            ),
        }
    }
}

impl std::error::Error for MetalTargetError {}

/// One validated Apple Metal compilation target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetalTarget {
    platform: ApplePlatform,
    deployment_minimum: DeploymentMinimum,
    msl_version: MslVersion,
    std_token: &'static str,
}

impl MetalTarget {
    /// Constructs a target only when the platform, revision, and minimum agree.
    ///
    /// # Errors
    ///
    /// Returns a typed rejection for an unavailable revision/platform pair or a deployment minimum below its governed floor.
    ///
    /// The MSL specification supplies the macOS, iOS, tvOS, and visionOS floors. Metal 32023.883 compile-and-link measurements supply the MSL 4.0 floor for Mac Catalyst and watchOS; those rows are compilation evidence, not runtime qualification.
    pub fn new(
        platform: ApplePlatform,
        deployment_minimum: DeploymentMinimum,
        msl_version: MslVersion,
    ) -> Result<Self, MetalTargetError> {
        let Some((required, std_token)) = target_language(platform, msl_version) else {
            return Err(MetalTargetError::LanguageUnavailable {
                platform,
                language: msl_version,
            });
        };
        if deployment_minimum < required {
            return Err(MetalTargetError::DeploymentMinimumTooLow {
                platform,
                language: msl_version,
                requested: deployment_minimum,
                required,
            });
        }
        Ok(Self {
            platform,
            deployment_minimum,
            msl_version,
            std_token,
        })
    }

    /// Returns the artifact platform family this target produces.
    #[must_use]
    pub const fn platform(self) -> ApplePlatform {
        self.platform
    }

    /// Returns the SDK used to discover this platform's tools.
    #[must_use]
    pub const fn sdk(self) -> AppleSdk {
        self.platform.sdk()
    }

    /// Returns the requested deployment minimum.
    #[must_use]
    pub const fn deployment_minimum(self) -> DeploymentMinimum {
        self.deployment_minimum
    }

    /// Returns the semantic MSL revision.
    #[must_use]
    pub const fn msl_version(self) -> MslVersion {
        self.msl_version
    }

    /// Returns the exact platform-qualified compiler token.
    #[must_use]
    pub const fn std_token(self) -> &'static str {
        self.std_token
    }

    /// Returns the normalized `air64-apple-*` target triple, for example
    /// `air64-apple-macos14.0` or `air64-apple-ios17.0-simulator`.
    #[must_use]
    pub fn triple(self) -> String {
        format!(
            "air64-apple-{}{}{}",
            self.platform.triple_os(),
            self.deployment_minimum,
            self.platform.triple_suffix(),
        )
    }
}

const fn target_language(
    platform: ApplePlatform,
    language: MslVersion,
) -> Option<(DeploymentMinimum, &'static str)> {
    use ApplePlatform::{
        IOsDevice, IOsSimulator, MacCatalyst, MacOs, TvOsDevice, TvOsSimulator, VisionOsDevice,
        VisionOsSimulator, WatchOsDevice, WatchOsSimulator,
    };
    use MslVersion::{
        Metal1_0, Metal1_1, Metal1_2, Metal2_0, Metal2_1, Metal2_2, Metal2_3, Metal2_4, Metal3_0,
        Metal3_1, Metal3_2, Metal4_0,
    };
    let pair = match (platform, language) {
        (IOsDevice | IOsSimulator, Metal1_0) => (8, 0, "ios-metal1.0"),
        (IOsDevice | IOsSimulator, Metal1_1) => (9, 0, "ios-metal1.1"),
        (IOsDevice | IOsSimulator, Metal1_2) => (10, 0, "ios-metal1.2"),
        (IOsDevice | IOsSimulator, Metal2_0) => (11, 0, "ios-metal2.0"),
        (IOsDevice | IOsSimulator, Metal2_1) => (12, 0, "ios-metal2.1"),
        (IOsDevice | IOsSimulator, Metal2_2) => (13, 0, "ios-metal2.2"),
        (IOsDevice | IOsSimulator, Metal2_3) => (14, 0, "ios-metal2.3"),
        (IOsDevice | IOsSimulator, Metal2_4) => (15, 0, "ios-metal2.4"),
        (MacOs, Metal1_1) => (10, 11, "macos-metal1.1"),
        (MacOs, Metal1_2) => (10, 12, "macos-metal1.2"),
        (MacOs, Metal2_0) => (10, 13, "macos-metal2.0"),
        (MacOs, Metal2_1) => (10, 14, "macos-metal2.1"),
        (MacOs, Metal2_2) => (10, 15, "macos-metal2.2"),
        (MacOs, Metal2_3) => (11, 0, "macos-metal2.3"),
        (MacOs, Metal2_4) => (12, 0, "macos-metal2.4"),
        (MacOs, Metal3_0) => (13, 0, "metal3.0"),
        (MacOs, Metal3_1) => (14, 0, "metal3.1"),
        (MacOs, Metal3_2) => (15, 0, "metal3.2"),
        (IOsDevice | IOsSimulator | TvOsDevice | TvOsSimulator, Metal3_0) => (16, 0, "metal3.0"),
        (IOsDevice | IOsSimulator | TvOsDevice | TvOsSimulator, Metal3_1) => (17, 0, "metal3.1"),
        (IOsDevice | IOsSimulator | TvOsDevice | TvOsSimulator, Metal3_2) => (18, 0, "metal3.2"),
        (VisionOsDevice | VisionOsSimulator, Metal3_1) => (1, 0, "metal3.1"),
        (VisionOsDevice | VisionOsSimulator, Metal3_2) => (2, 0, "metal3.2"),
        (
            MacOs | MacCatalyst | IOsDevice | IOsSimulator | TvOsDevice | TvOsSimulator
            | VisionOsDevice | VisionOsSimulator | WatchOsDevice | WatchOsSimulator,
            Metal4_0,
        ) => (26, 0, "metal4.0"),
        _ => return None,
    };
    Some((DeploymentMinimum::new(pair.0, pair.1), pair.2))
}

/// A complete offline Metal compilation request.
///
/// It carries the MSL source and every explicit output-affecting choice. The
/// driver reads MSL as input; emitting MSL is a separate concern owned by
/// `tiler-metal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileRequest {
    /// The Metal Shading Language source text to compile.
    pub source: String,
    /// The fully specified target.
    pub target: MetalTarget,
    /// The selected optimization level.
    pub optimization: OptimizationLevel,
    /// The explicit numerical realization flags.
    pub numerical: NumericalRealization,
}

impl CompileRequest {
    /// Constructs a compilation request from source and explicit choices.
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        target: MetalTarget,
        optimization: OptimizationLevel,
        numerical: NumericalRealization,
    ) -> Self {
        Self {
            source: source.into(),
            target,
            optimization,
            numerical,
        }
    }

    /// Returns the exact ordered `metal` compiler flags this request compiles
    /// with, excluding the source and output file paths.
    ///
    /// The order is stable: target triple, language standard, optimization
    /// level, then the three numerical realization flags.
    #[must_use]
    pub fn compile_flags(&self) -> Vec<String> {
        let mut flags = vec![
            "-target".to_owned(),
            self.target.triple(),
            format!("-std={}", self.target.std_token()),
            self.optimization.flag().to_owned(),
        ];
        flags.extend(self.numerical.flags());
        flags
    }

    /// Returns the exact ordered `metallib` linker flags this request links
    /// with, excluding the input and output file paths.
    ///
    /// The bounded driver links with no extra flags; the vector is the reserved
    /// seam for future linker options that participate in identity.
    #[must_use]
    pub fn link_flags(&self) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApplePlatform, AppleSdk, CompileRequest, DeploymentMinimum, Fp32Functions, FpContract,
        MathMode, MetalTarget, MetalTargetError, MslVersion, NumericalRealization,
        OptimizationLevel,
    };

    fn baseline_request(platform: ApplePlatform, minimum: DeploymentMinimum) -> CompileRequest {
        CompileRequest::new(
            "// source",
            MetalTarget::new(platform, minimum, MslVersion::Metal3_1)
                .expect("the fixture target is valid"),
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        )
    }

    #[test]
    fn macos_triple_is_normalized() {
        let target = MetalTarget::new(
            ApplePlatform::MacOs,
            DeploymentMinimum::new(14, 0),
            MslVersion::Metal3_1,
        )
        .expect("MSL 3.1 is admitted from macOS 14");
        assert_eq!(target.triple(), "air64-apple-macos14.0");
        assert_eq!(target.platform(), ApplePlatform::MacOs);
    }

    #[test]
    fn ios_device_triple_has_no_environment_suffix() {
        let target = MetalTarget::new(
            ApplePlatform::IOsDevice,
            DeploymentMinimum::new(17, 0),
            MslVersion::Metal3_1,
        )
        .expect("MSL 3.1 is admitted from iOS 17");
        assert_eq!(target.triple(), "air64-apple-ios17.0");
        assert_eq!(target.platform(), ApplePlatform::IOsDevice);
    }

    #[test]
    fn ios_simulator_triple_carries_the_simulator_environment() {
        let target = MetalTarget::new(
            ApplePlatform::IOsSimulator,
            DeploymentMinimum::new(17, 0),
            MslVersion::Metal3_1,
        )
        .expect("MSL 3.1 is admitted from iOS 17");
        assert_eq!(target.triple(), "air64-apple-ios17.0-simulator");
        assert_eq!(target.platform(), ApplePlatform::IOsSimulator);
        assert_eq!(target.sdk().selector(), "iphonesimulator");
    }

    /// Every artifact family names its stable identifier, SDK, and triple shape.
    ///
    /// The match is exhaustive so adding a family cannot silently inherit
    /// another family's compiler routing.
    #[test]
    fn every_artifact_family_has_complete_compiler_routing() {
        for (family, name, sdk, selector, triple) in [
            (
                ApplePlatform::MacOs,
                "macos",
                AppleSdk::MacOs,
                "macosx",
                "air64-apple-macos26.0",
            ),
            (
                ApplePlatform::MacCatalyst,
                "mac-catalyst",
                AppleSdk::MacOs,
                "macosx",
                "air64-apple-ios26.0-macabi",
            ),
            (
                ApplePlatform::IOsDevice,
                "ios-device",
                AppleSdk::IPhoneOs,
                "iphoneos",
                "air64-apple-ios26.0",
            ),
            (
                ApplePlatform::IOsSimulator,
                "ios-simulator",
                AppleSdk::IPhoneSimulator,
                "iphonesimulator",
                "air64-apple-ios26.0-simulator",
            ),
            (
                ApplePlatform::TvOsDevice,
                "tvos-device",
                AppleSdk::AppleTvOs,
                "appletvos",
                "air64-apple-tvos26.0",
            ),
            (
                ApplePlatform::TvOsSimulator,
                "tvos-simulator",
                AppleSdk::AppleTvSimulator,
                "appletvsimulator",
                "air64-apple-tvos26.0-simulator",
            ),
            (
                ApplePlatform::VisionOsDevice,
                "visionos-device",
                AppleSdk::XrOs,
                "xros",
                "air64-apple-xros26.0",
            ),
            (
                ApplePlatform::VisionOsSimulator,
                "visionos-simulator",
                AppleSdk::XrSimulator,
                "xrsimulator",
                "air64-apple-xros26.0-simulator",
            ),
            (
                ApplePlatform::WatchOsDevice,
                "watchos-device",
                AppleSdk::WatchOs,
                "watchos",
                "air64-apple-watchos26.0",
            ),
            (
                ApplePlatform::WatchOsSimulator,
                "watchos-simulator",
                AppleSdk::WatchSimulator,
                "watchsimulator",
                "air64-apple-watchos26.0-simulator",
            ),
        ] {
            let target =
                MetalTarget::new(family, DeploymentMinimum::new(26, 0), MslVersion::Metal4_0)
                    .expect("MSL 4.0 is admitted for every represented family");
            let expected = match family {
                ApplePlatform::MacOs => ("macos", AppleSdk::MacOs),
                ApplePlatform::MacCatalyst => ("mac-catalyst", AppleSdk::MacOs),
                ApplePlatform::IOsDevice => ("ios-device", AppleSdk::IPhoneOs),
                ApplePlatform::IOsSimulator => ("ios-simulator", AppleSdk::IPhoneSimulator),
                ApplePlatform::TvOsDevice => ("tvos-device", AppleSdk::AppleTvOs),
                ApplePlatform::TvOsSimulator => ("tvos-simulator", AppleSdk::AppleTvSimulator),
                ApplePlatform::VisionOsDevice => ("visionos-device", AppleSdk::XrOs),
                ApplePlatform::VisionOsSimulator => ("visionos-simulator", AppleSdk::XrSimulator),
                ApplePlatform::WatchOsDevice => ("watchos-device", AppleSdk::WatchOs),
                ApplePlatform::WatchOsSimulator => ("watchos-simulator", AppleSdk::WatchSimulator),
            };
            assert_eq!((name, sdk), expected);
            assert_eq!(family.as_str(), name);
            assert_eq!(target.sdk(), sdk);
            assert_eq!(target.sdk().selector(), selector);
            assert_eq!(target.triple(), triple);
            assert_eq!(target.std_token(), "metal4.0");
        }
    }

    #[test]
    fn every_sdk_selector_appears_once_in_the_canonical_inventory() {
        let mut seen = [false; AppleSdk::COUNT];
        for sdk in AppleSdk::ALL {
            let index = sdk.index();
            assert!(!seen[index], "{} appears more than once", sdk.selector());
            seen[index] = true;
        }
        assert!(
            seen.into_iter().all(|present| present),
            "AppleSdk::ALL omits a selector"
        );
    }

    #[test]
    fn every_semantic_language_revision_has_stable_spellings() {
        for (language, revision, semantic_name) in [
            (MslVersion::Metal1_0, "1.0", "metal1.0"),
            (MslVersion::Metal1_1, "1.1", "metal1.1"),
            (MslVersion::Metal1_2, "1.2", "metal1.2"),
            (MslVersion::Metal2_0, "2.0", "metal2.0"),
            (MslVersion::Metal2_1, "2.1", "metal2.1"),
            (MslVersion::Metal2_2, "2.2", "metal2.2"),
            (MslVersion::Metal2_3, "2.3", "metal2.3"),
            (MslVersion::Metal2_4, "2.4", "metal2.4"),
            (MslVersion::Metal3_0, "3.0", "metal3.0"),
            (MslVersion::Metal3_1, "3.1", "metal3.1"),
            (MslVersion::Metal3_2, "3.2", "metal3.2"),
            (MslVersion::Metal4_0, "4.0", "metal4.0"),
        ] {
            let expected = match language {
                MslVersion::Metal1_0 => ("1.0", "metal1.0"),
                MslVersion::Metal1_1 => ("1.1", "metal1.1"),
                MslVersion::Metal1_2 => ("1.2", "metal1.2"),
                MslVersion::Metal2_0 => ("2.0", "metal2.0"),
                MslVersion::Metal2_1 => ("2.1", "metal2.1"),
                MslVersion::Metal2_2 => ("2.2", "metal2.2"),
                MslVersion::Metal2_3 => ("2.3", "metal2.3"),
                MslVersion::Metal2_4 => ("2.4", "metal2.4"),
                MslVersion::Metal3_0 => ("3.0", "metal3.0"),
                MslVersion::Metal3_1 => ("3.1", "metal3.1"),
                MslVersion::Metal3_2 => ("3.2", "metal3.2"),
                MslVersion::Metal4_0 => ("4.0", "metal4.0"),
            };
            assert_eq!((revision, semantic_name), expected);
            assert_eq!(language.revision(), revision);
            assert_eq!(language.semantic_name(), semantic_name);
        }
    }

    #[test]
    fn governed_legacy_and_unified_tokens_use_the_specification_floors() {
        let assert_target = |platform, language, major, minor, token| {
            let target = MetalTarget::new(platform, DeploymentMinimum::new(major, minor), language)
                .expect("the exact specification floor is admitted");
            assert_eq!(target.std_token(), token);
        };

        for platform in [ApplePlatform::IOsDevice, ApplePlatform::IOsSimulator] {
            for (language, major, token) in [
                (MslVersion::Metal1_0, 8, "ios-metal1.0"),
                (MslVersion::Metal1_1, 9, "ios-metal1.1"),
                (MslVersion::Metal1_2, 10, "ios-metal1.2"),
                (MslVersion::Metal2_0, 11, "ios-metal2.0"),
                (MslVersion::Metal2_1, 12, "ios-metal2.1"),
                (MslVersion::Metal2_2, 13, "ios-metal2.2"),
                (MslVersion::Metal2_3, 14, "ios-metal2.3"),
                (MslVersion::Metal2_4, 15, "ios-metal2.4"),
                (MslVersion::Metal3_0, 16, "metal3.0"),
                (MslVersion::Metal3_1, 17, "metal3.1"),
                (MslVersion::Metal3_2, 18, "metal3.2"),
                (MslVersion::Metal4_0, 26, "metal4.0"),
            ] {
                assert_target(platform, language, major, 0, token);
            }
        }
        for (language, major, minor, token) in [
            (MslVersion::Metal1_1, 10, 11, "macos-metal1.1"),
            (MslVersion::Metal1_2, 10, 12, "macos-metal1.2"),
            (MslVersion::Metal2_0, 10, 13, "macos-metal2.0"),
            (MslVersion::Metal2_1, 10, 14, "macos-metal2.1"),
            (MslVersion::Metal2_2, 10, 15, "macos-metal2.2"),
            (MslVersion::Metal2_3, 11, 0, "macos-metal2.3"),
            (MslVersion::Metal2_4, 12, 0, "macos-metal2.4"),
            (MslVersion::Metal3_0, 13, 0, "metal3.0"),
            (MslVersion::Metal3_1, 14, 0, "metal3.1"),
            (MslVersion::Metal3_2, 15, 0, "metal3.2"),
            (MslVersion::Metal4_0, 26, 0, "metal4.0"),
        ] {
            assert_target(ApplePlatform::MacOs, language, major, minor, token);
        }
        for platform in [ApplePlatform::TvOsDevice, ApplePlatform::TvOsSimulator] {
            for (language, major) in [
                (MslVersion::Metal3_0, 16),
                (MslVersion::Metal3_1, 17),
                (MslVersion::Metal3_2, 18),
                (MslVersion::Metal4_0, 26),
            ] {
                assert_target(platform, language, major, 0, language.semantic_name());
            }
        }
        for platform in [
            ApplePlatform::VisionOsDevice,
            ApplePlatform::VisionOsSimulator,
        ] {
            for (language, major) in [
                (MslVersion::Metal3_1, 1),
                (MslVersion::Metal3_2, 2),
                (MslVersion::Metal4_0, 26),
            ] {
                assert_target(platform, language, major, 0, language.semantic_name());
            }
        }
    }

    #[test]
    fn target_construction_rejects_below_specification_floors() {
        assert_eq!(
            MetalTarget::new(
                ApplePlatform::MacOs,
                DeploymentMinimum::new(13, 0),
                MslVersion::Metal3_1,
            ),
            Err(MetalTargetError::DeploymentMinimumTooLow {
                platform: ApplePlatform::MacOs,
                language: MslVersion::Metal3_1,
                requested: DeploymentMinimum::new(13, 0),
                required: DeploymentMinimum::new(14, 0),
            }),
        );
        assert_eq!(
            MetalTarget::new(
                ApplePlatform::IOsDevice,
                DeploymentMinimum::new(16, 0),
                MslVersion::Metal3_1,
            ),
            Err(MetalTargetError::DeploymentMinimumTooLow {
                platform: ApplePlatform::IOsDevice,
                language: MslVersion::Metal3_1,
                requested: DeploymentMinimum::new(16, 0),
                required: DeploymentMinimum::new(17, 0),
            }),
        );
    }

    #[test]
    fn target_construction_rejects_an_unavailable_language_platform_pair() {
        assert_eq!(
            MetalTarget::new(
                ApplePlatform::MacOs,
                DeploymentMinimum::new(10, 0),
                MslVersion::Metal1_0,
            ),
            Err(MetalTargetError::LanguageUnavailable {
                platform: ApplePlatform::MacOs,
                language: MslVersion::Metal1_0,
            }),
        );
    }

    #[test]
    fn compile_flags_are_exact_and_ordered() {
        let request = baseline_request(ApplePlatform::MacOs, DeploymentMinimum::new(14, 0));
        assert_eq!(
            request.compile_flags(),
            [
                "-target",
                "air64-apple-macos14.0",
                "-std=metal3.1",
                "-O2",
                "-fmetal-math-mode=safe",
                "-fmetal-math-fp32-functions=precise",
                "-ffp-contract=off",
            ],
        );
    }

    #[test]
    fn numerical_permissions_are_independent() {
        let realization =
            NumericalRealization::new(MathMode::Relaxed, Fp32Functions::Fast, FpContract::Fast);
        assert_eq!(
            realization.flags(),
            [
                "-fmetal-math-mode=relaxed",
                "-fmetal-math-fp32-functions=fast",
                "-ffp-contract=fast",
            ],
        );
    }

    /// Pins the measured semantics each math mode actually delivers.
    ///
    /// These predicates are what a caller checks a declared numerical contract
    /// against. Getting one wrong would silently approve a realization that
    /// changes results, so each is asserted for every mode rather than only for
    /// the strict baseline.
    #[test]
    fn each_math_mode_states_the_licences_it_applies() {
        for (mode, relaxes, finite) in [
            (MathMode::Safe, false, false),
            (MathMode::Relaxed, true, false),
            (MathMode::Fast, true, true),
        ] {
            assert_eq!(mode.relaxes_floating_point(), relaxes, "{mode:?}");
            assert_eq!(mode.assumes_finite(), finite, "{mode:?}");
        }
    }

    #[test]
    fn only_fast_contraction_fuses_across_statements() {
        assert!(!FpContract::Off.contracts_across_statements());
        assert!(!FpContract::On.contracts_across_statements());
        assert!(FpContract::Fast.contracts_across_statements());
        assert!(FpContract::FastHonorPragmas.contracts_across_statements());
        assert_eq!(FpContract::FastHonorPragmas.token(), "fast-honor-pragmas");
    }

    /// The strictest selectable realization is still not IEEE-754 binary32.
    ///
    /// Signed zero, reassociation, and NaN results are all recovered by the
    /// strict baseline. Subnormal preservation is not recoverable by any
    /// selection, so a caller must not read "strict flags" as "conforming".
    #[test]
    fn the_strict_baseline_recovers_everything_except_subnormals() {
        let strict = NumericalRealization::strict_baseline();
        assert!(strict.preserves_signed_zero());
        assert!(!strict.permits_reassociation());
        assert!(strict.preserves_nan_results());
        assert!(!strict.preserves_f32_subnormals());

        for mode in [MathMode::Relaxed, MathMode::Fast] {
            let relaxed = NumericalRealization::new(mode, Fp32Functions::Precise, FpContract::Off);
            assert!(!relaxed.preserves_signed_zero(), "{mode:?}");
            assert!(relaxed.permits_reassociation(), "{mode:?}");
            assert!(!relaxed.preserves_f32_subnormals(), "{mode:?}");
        }
        assert!(
            NumericalRealization::new(MathMode::Relaxed, Fp32Functions::Precise, FpContract::Off)
                .preserves_nan_results()
        );
        assert!(
            !NumericalRealization::new(MathMode::Fast, Fp32Functions::Precise, FpContract::Off)
                .preserves_nan_results()
        );
    }

    #[test]
    fn link_flags_are_empty_for_the_bounded_driver() {
        let request = baseline_request(ApplePlatform::MacOs, DeploymentMinimum::new(14, 0));
        assert!(request.link_flags().is_empty());
    }
}
