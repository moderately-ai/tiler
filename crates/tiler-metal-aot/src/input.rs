//! Explicit compilation inputs for the offline Apple Metal driver.
//!
//! Every output-affecting choice is a strongly typed field. The driver never
//! inherits the Metal compiler's math or optimization defaults; the numerical
//! realization flags are required inputs, so
//! [`NumericalRealization`](crate::input::NumericalRealization) deliberately has
//! no `Default` implementation.

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
    /// This is a named explicit choice, not an inherited default.
    #[must_use]
    pub const fn strict_baseline() -> Self {
        Self::new(MathMode::Safe, Fp32Functions::Precise, FpContract::Off)
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

    #[test]
    fn link_flags_are_empty_for_the_bounded_driver() {
        let request = baseline_request(AppleSdk::MacOs, DeploymentMinimum::new(13, 0));
        assert!(request.link_flags().is_empty());
    }
}
