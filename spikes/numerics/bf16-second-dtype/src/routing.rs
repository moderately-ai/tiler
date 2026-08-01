//! The two target-family routes, built from one mechanism.
//!
//! macOS supplies a measured **positive** BF16 dispatchability fact; the iOS
//! Simulator supplies a measured **negative** one. Both are declared through the
//! same `TargetProfileBuilder::declare_dtype_dispatchability` seam and resolved
//! through the same `dtype_dispatchability` query, so the refusal is the
//! mechanism saying no rather than a second code path.
//!
//! # What this does not do
//!
//! It does not submit BF16 program work on either family, and it does not
//! compile a `bfloat` module. The negative route is a **pre-routing** refusal:
//! it resolves before any program work, which is the whole point — the measured
//! simulator failure occurs at `PreparedKernelPreflight`, one phase *after* the
//! one-way routing commit, so a design that discovered it there would already
//! have committed. Declaring it as a compile-profile fact moves the refusal
//! before the commit.

use tiler_compiler::target::{
    DTypeDispatchability, DTypeDispatchabilityResolution, TargetFactProducerIdentity,
    TargetFactSource, TargetNormativeReferenceIdentity, TargetProfile, TargetProfileBuildError,
    TargetProfileBuilder, TargetProfileKey,
};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::semantic::{F32, ResolvedValueType};

use crate::seams::Bf16;

/// Profile key of the measured macOS row that dispatches BF16.
pub const MACOS_PROFILE_KEY: &str = "tiler.spike.metal.macos-apple9.bf16-dispatch.v1";

/// Profile key of the measured iOS-Simulator row that refuses BF16.
pub const IOS_SIMULATOR_PROFILE_KEY: &str = "tiler.spike.metal.ios-simulator.bf16-refusal.v1";

/// The exact diagnostic the measured simulator refusal carried.
///
/// Quoted from the retained record
/// `spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv`,
/// row `environment.family.ios-simulator.device_bfloat_support`.
pub const SIMULATOR_REFUSAL_DIAGNOSTIC: &str = "pipeline creation failed: Compilation failed due to an interrupted connection: \
     XPC_ERROR_CONNECTION_INTERRUPTED. This error occurred after multiple retries.";

/// The measurement identity every fact in these profiles is attributed to.
fn measured_source(reference: &str) -> Result<TargetFactSource, TargetProfileBuildError> {
    let producer = TargetFactProducerIdentity::new("tiler.spike.bf16-second-dtype".to_owned(), 1)
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let normative = TargetNormativeReferenceIdentity::new(reference.to_owned(), 1)
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    Ok(TargetFactSource::external_guarantee(producer, normative))
}

/// Builds the macOS profile that declares BF16 dispatchable.
///
/// **Measurement boundary.** The positive fact is the retained Apple record's
/// `environment.family.macos.device_bfloat_support supported` row on an Apple
/// M4 Max under macOS 27.0 build 26A5388g with Metal 32023.883. It is a fact
/// about that family on that row, not about Apple GPUs.
///
/// # Errors
///
/// Returns the first typed declaration or freeze diagnostic.
pub fn macos_profile() -> Result<TargetProfile, TargetProfileBuildError> {
    let key = TargetProfileKey::new(MACOS_PROFILE_KEY.to_owned())
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let mut builder = TargetProfileBuilder::new(key);
    let source = measured_source("tiler.apple-numerical-behaviour.v6.macos.bfloat-support")?;
    builder.declare_dtype_dispatchability(
        Bf16::resolved_type(),
        DTypeDispatchability::Dispatchable,
        source.clone(),
    )?;
    // F32 is declared beside it deliberately. A profile that spoke only about
    // BF16 would make every refusal below indistinguishable from a profile that
    // says nothing at all; the accepted neighbour is what makes the BF16 answer
    // evidence about BF16.
    builder.declare_dtype_dispatchability(
        F32::resolved_type(),
        DTypeDispatchability::Dispatchable,
        source,
    )?;
    builder.build()
}

/// Builds the iOS-Simulator profile that declares BF16 explicitly unsupported.
///
/// **Measurement boundary.** The negative fact is the same retained record's
/// `environment.family.ios-simulator.device_bfloat_support` row, and the
/// separately retained `bfloat_dispatch_probe.py` observation that the
/// arithmetic-free `materialize_bf16` is refused too — which is what makes the
/// declaration a fact about the *format* rather than about one operation. F32
/// remains dispatchable on the same family, so the refusal is not a dead profile.
///
/// # Errors
///
/// Returns the first typed declaration or freeze diagnostic.
pub fn ios_simulator_profile() -> Result<TargetProfile, TargetProfileBuildError> {
    let key = TargetProfileKey::new(IOS_SIMULATOR_PROFILE_KEY.to_owned())
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let mut builder = TargetProfileBuilder::new(key);
    let source =
        measured_source("tiler.apple-numerical-behaviour.v6.ios-simulator.bfloat-refusal")?;
    builder.declare_dtype_dispatchability(
        Bf16::resolved_type(),
        DTypeDispatchability::Unsupported,
        source.clone(),
    )?;
    builder.declare_dtype_dispatchability(
        F32::resolved_type(),
        DTypeDispatchability::Dispatchable,
        source,
    )?;
    builder.build()
}

/// Builds a profile that states nothing about BF16, for the `Unknown` control.
///
/// A third answer, distinct from both. It is what a family nobody measured must
/// resolve to, and the run asserts it rather than assuming it: `IOsDevice` is
/// exactly this case today, and a design that let it inherit either neighbour's
/// answer would be wrong in one direction or the other.
///
/// # Errors
///
/// Returns the first typed declaration or freeze diagnostic.
pub fn silent_profile() -> Result<TargetProfile, TargetProfileBuildError> {
    let key = TargetProfileKey::new("tiler.spike.metal.ios-device.unmeasured.v1".to_owned())
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let mut builder = TargetProfileBuilder::new(key);
    let source = measured_source("tiler.apple-numerical-behaviour.v6.ios-device.unmeasured")?;
    builder.declare_dtype_dispatchability(
        F32::resolved_type(),
        DTypeDispatchability::Dispatchable,
        source,
    )?;
    builder.build()
}

/// One resolved routing answer, rendered into the run narrative.
pub struct Route {
    /// Which profile was asked.
    pub profile: &'static str,
    /// Which dtype was asked about.
    pub dtype: &'static str,
    /// The resolution the profile returned.
    pub resolution: DTypeDispatchabilityResolution,
}

/// Resolves one dtype against one profile at the compile-profile phase.
#[must_use]
pub fn resolve(
    profile: &TargetProfile,
    profile_name: &'static str,
    dtype: &ResolvedValueType,
    dtype_name: &'static str,
) -> Route {
    Route {
        profile: profile_name,
        dtype: dtype_name,
        resolution: profile.dtype_dispatchability(dtype, AvailabilityPhase::CompileProfile),
    }
}

/// Resolves the complete routing matrix this spike claims.
///
/// Three profiles by two dtypes. The matrix is returned whole rather than
/// checked case by case so the run can assert its **shape** — six answers, three
/// distinct resolutions — instead of six independent facts that could all be the
/// same answer without anything noticing.
///
/// # Errors
///
/// Returns the first profile-construction diagnostic.
pub fn routing_matrix() -> Result<Vec<Route>, TargetProfileBuildError> {
    let macos = macos_profile()?;
    let simulator = ios_simulator_profile()?;
    let silent = silent_profile()?;
    let bf16 = Bf16::resolved_type();
    let f32 = F32::resolved_type();
    Ok(vec![
        resolve(&macos, "macos", &bf16, "bf16"),
        resolve(&macos, "macos", &f32, "f32"),
        resolve(&simulator, "ios-simulator", &bf16, "bf16"),
        resolve(&simulator, "ios-simulator", &f32, "f32"),
        resolve(&silent, "ios-device (unmeasured)", &bf16, "bf16"),
        resolve(&silent, "ios-device (unmeasured)", &f32, "f32"),
    ])
}
