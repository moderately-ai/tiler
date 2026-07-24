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

/// One selected Apple SDK and the artifact family it produces.
///
/// Mac Catalyst (`ios` + `macabi`) is a deferred fourth family and is not yet
/// representable here; the Apple artifact-compatibility research keeps it
/// explicitly deferred rather than relabelling a macOS or iOS artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppleSdk {
    /// The `macosx` SDK: the macOS artifact family.
    MacOs,
    /// The `iphoneos` SDK: the iOS device artifact family.
    IPhoneOs,
    /// The `iphonesimulator` SDK: the iOS simulator artifact family.
    IPhoneSimulator,
}

impl AppleSdk {
    /// Returns the canonical `xcrun --sdk` selector for this SDK.
    #[must_use]
    pub const fn selector(self) -> &'static str {
        match self {
            Self::MacOs => "macosx",
            Self::IPhoneOs => "iphoneos",
            Self::IPhoneSimulator => "iphonesimulator",
        }
    }

    /// Returns the artifact platform family this SDK targets.
    #[must_use]
    pub const fn platform(self) -> ApplePlatform {
        match self {
            Self::MacOs => ApplePlatform::MacOs,
            Self::IPhoneOs => ApplePlatform::IOsDevice,
            Self::IPhoneSimulator => ApplePlatform::IOsSimulator,
        }
    }

    /// Returns the `air64-apple-<os>` operating-system token for the triple.
    const fn triple_os(self) -> &'static str {
        match self {
            Self::MacOs => "macos",
            Self::IPhoneOs | Self::IPhoneSimulator => "ios",
        }
    }

    /// Returns the triple environment suffix; the simulator adds `-simulator`.
    const fn triple_suffix(self) -> &'static str {
        match self {
            Self::MacOs | Self::IPhoneOs => "",
            Self::IPhoneSimulator => "-simulator",
        }
    }
}

/// The measured Apple artifact family a compiled `metallib` belongs to.
///
/// This is the family a compilation *produces*, and
/// `tiler_metal::target::MetalPlatform` is the family emitted source
/// *declares*, with the same three variants and the same stable identifiers.
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
}

impl ApplePlatform {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// MSL 3.0, spelled `-std=metal3.0`.
    Metal3_0,
    /// MSL 3.1, spelled `-std=metal3.1`.
    Metal3_1,
}

impl MslVersion {
    /// Returns the `-std` value token for this language version.
    #[must_use]
    pub const fn std_token(self) -> &'static str {
        match self {
            Self::Metal3_0 => "metal3.0",
            Self::Metal3_1 => "metal3.1",
        }
    }
}

/// The selected Metal optimization level.
///
/// Optimization is output-affecting and therefore an explicit input. `-O2` is
/// the level the Apple artifact-compatibility probe measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

impl FpContract {
    /// Returns the `-ffp-contract` value token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::On => "on",
            Self::Fast => "fast",
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
            Self::Fast => true,
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
    /// full IEEE-754 binary32 conformance; see [`Self::preserves_subnormals`].
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
    pub const fn preserves_subnormals(self) -> bool {
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

/// One fully specified Apple Metal compilation target.
///
/// The SDK selects the family and simulator environment, the deployment minimum
/// supplies the version, and the two together fully determine the normalized
/// target triple, so no inconsistent SDK/triple pairing is representable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetalTarget {
    /// The single selected SDK and artifact family.
    pub sdk: AppleSdk,
    /// The requested deployment minimum.
    pub deployment_minimum: DeploymentMinimum,
    /// The selected MSL standard.
    pub msl_version: MslVersion,
}

impl MetalTarget {
    /// Constructs a fully specified target.
    #[must_use]
    pub const fn new(
        sdk: AppleSdk,
        deployment_minimum: DeploymentMinimum,
        msl_version: MslVersion,
    ) -> Self {
        Self {
            sdk,
            deployment_minimum,
            msl_version,
        }
    }

    /// Returns the artifact platform family this target produces.
    #[must_use]
    pub const fn platform(self) -> ApplePlatform {
        self.sdk.platform()
    }

    /// Returns the normalized `air64-apple-*` target triple, for example
    /// `air64-apple-macos13.0` or `air64-apple-ios16.0-simulator`.
    #[must_use]
    pub fn triple(self) -> String {
        format!(
            "air64-apple-{}{}{}",
            self.sdk.triple_os(),
            self.deployment_minimum,
            self.sdk.triple_suffix(),
        )
    }
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
            format!("-std={}", self.target.msl_version.std_token()),
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
        MathMode, MetalTarget, MslVersion, NumericalRealization, OptimizationLevel,
    };

    fn baseline_request(sdk: AppleSdk, minimum: DeploymentMinimum) -> CompileRequest {
        CompileRequest::new(
            "// source",
            MetalTarget::new(sdk, minimum, MslVersion::Metal3_1),
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        )
    }

    #[test]
    fn macos_triple_is_normalized() {
        let target = MetalTarget::new(
            AppleSdk::MacOs,
            DeploymentMinimum::new(13, 0),
            MslVersion::Metal3_1,
        );
        assert_eq!(target.triple(), "air64-apple-macos13.0");
        assert_eq!(target.platform(), ApplePlatform::MacOs);
    }

    #[test]
    fn ios_device_triple_has_no_environment_suffix() {
        let target = MetalTarget::new(
            AppleSdk::IPhoneOs,
            DeploymentMinimum::new(16, 0),
            MslVersion::Metal3_1,
        );
        assert_eq!(target.triple(), "air64-apple-ios16.0");
        assert_eq!(target.platform(), ApplePlatform::IOsDevice);
    }

    #[test]
    fn ios_simulator_triple_carries_the_simulator_environment() {
        let target = MetalTarget::new(
            AppleSdk::IPhoneSimulator,
            DeploymentMinimum::new(17, 0),
            MslVersion::Metal3_1,
        );
        assert_eq!(target.triple(), "air64-apple-ios17.0-simulator");
        assert_eq!(target.platform(), ApplePlatform::IOsSimulator);
        assert_eq!(target.sdk.selector(), "iphonesimulator");
    }

    /// Every artifact family names the SDK selector that produces it.
    ///
    /// [`AppleSdk::platform`] is total in one direction: every SDK yields a
    /// family. The direction that can rot is the other one — a family added to
    /// [`ApplePlatform`] with no SDK producing it would be recordable in
    /// provenance and impossible to compile for. The match below is exhaustive
    /// over [`ApplePlatform`], so a new family fails to compile here until it
    /// names its selector.
    #[test]
    fn every_artifact_family_names_the_sdk_that_selects_it() {
        for family in [
            ApplePlatform::MacOs,
            ApplePlatform::IOsDevice,
            ApplePlatform::IOsSimulator,
        ] {
            let (sdk, selector) = match family {
                ApplePlatform::MacOs => (AppleSdk::MacOs, "macosx"),
                ApplePlatform::IOsDevice => (AppleSdk::IPhoneOs, "iphoneos"),
                ApplePlatform::IOsSimulator => (AppleSdk::IPhoneSimulator, "iphonesimulator"),
            };
            assert_eq!(sdk.platform(), family, "{}", family.as_str());
            assert_eq!(sdk.selector(), selector, "{}", family.as_str());
        }
    }

    #[test]
    fn compile_flags_are_exact_and_ordered() {
        let request = baseline_request(AppleSdk::MacOs, DeploymentMinimum::new(14, 0));
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
        assert!(!strict.preserves_subnormals());

        for mode in [MathMode::Relaxed, MathMode::Fast] {
            let relaxed = NumericalRealization::new(mode, Fp32Functions::Precise, FpContract::Off);
            assert!(!relaxed.preserves_signed_zero(), "{mode:?}");
            assert!(relaxed.permits_reassociation(), "{mode:?}");
            assert!(!relaxed.preserves_subnormals(), "{mode:?}");
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
        let request = baseline_request(AppleSdk::MacOs, DeploymentMinimum::new(13, 0));
        assert!(request.link_flags().is_empty());
    }
}
