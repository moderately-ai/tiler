//! The scalar-host target profile this backend declares, and its governed keys.
//!
//! Nothing here is Apple's and nothing here is `tiler-build`'s. The profile is
//! declared through `tiler_compiler::target::TargetProfileBuilder` by a caller
//! outside every crate in the workspace, in the shape the bounded scalar CPU
//! vertical measured — which is the point: a second backend must be able to
//! state its own target facts without a production edit, and the artifact this
//! suite assembles must carry *these* facts rather than the Metal path's.
//!
//! # The two facts a scalar host has to state and no scalar host has
//!
//! `WorkgroupThreads` and `LocalMemoryBytes` are GPU axes, and feasibility
//! compares every kernel's derived requirements against them, so omitting them
//! leaves them `Unknown` — a different claim from "one" and "zero". A scalar
//! host runs each invocation independently and stages no local memory, so the
//! honest neutral readings are `1` and `0`. They are also why this profile mints
//! **no deferred prepared-entry predicate**: the workgroup bound is a
//! compile-time fact here, where Metal can only learn it from a built pipeline.
//! The assembled artifact carrying zero deferred predicates is that difference
//! reaching the artifact, and it exercises a branch the Metal path never takes.

use tiler_compiler::target::{
    DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
    TargetProfile, TargetProfileBuildError, TargetProfileBuilder, TargetProfileKey,
};
use tiler_ir::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
    SubnormalMode,
};
use tiler_ir::semantic::F32;

/// Governed profile key of this backend's declared target.
pub const PROFILE_KEY: &str = "tiler.test.scalar-host-aarch64-darwin";

/// Governed backend family key this backend's payloads declare.
pub const BACKEND_KEY: &str = "tiler.test.scalar-host";

/// Governed executable-representation key of its carried payload.
pub const REPRESENTATION_KEY: &str = "tiler.test.scalar-host-image-v1";

/// Governed representation key of the source the payload retains.
///
/// The image is translated from verified structured kernels rather than from
/// text, so the retained source is the canonical kernel identity list it was
/// translated from. Naming it as its own representation keeps the compilation
/// subject honest: there is no source file behind this payload, and claiming one
/// would be provenance nobody can check.
pub const SOURCE_REPRESENTATION_KEY: &str = "tiler.test.scalar-host-kernel-identity-list-v1";

/// Governed toolchain identity of this backend's in-process translator.
pub const TOOLCHAIN_KEY: &str = "tiler.test.scalar-host-translator";

/// Declared target triple of this profile's macOS artifact family.
///
/// A string rather than a typed axis, because the profile vocabulary has no
/// target-triple axis; it survives only inside the profile key and the payload
/// provenance. ADR 0090 item 14 names that gap.
pub const MACOS_TARGET_TRIPLE: &str = "aarch64-apple-darwin";

/// Declared target triple of this profile's iOS artifact family.
///
/// **Two build targets under one compiler profile, which is what makes a second
/// delivery position expressible here.** Every axis this profile declares —
/// grid extent, workgroup threads, buffer bindings, index arithmetic, address
/// width, device and local memory, the f32 numerical rows, and dtype
/// dispatchability — is a property of the aarch64 Darwin scalar execution model
/// and is identical for both triples, so the two families compile under one
/// profile key and one byte-identical canonical descriptor. What differs is the
/// build target a consumer's `cfg` resolves to, which the profile vocabulary
/// cannot express and which reaches the artifact only through the payload
/// provenance and the emitted image.
///
/// It is `-ios` rather than a second architecture because the key above names
/// `aarch64-darwin`, and iOS is Darwin on aarch64: a second architecture would
/// be a second profile and could not share a delivery run at all. That is the
/// same shape the standard Metal path has, where the authority ledger records
/// `MetalTargetFacts::platform` as backend-only.
///
/// **This states no numerical measurement about iOS, and could not.** The
/// scalar-host backend runs no target compiler: its payload is an image its own
/// in-process translator wrote, so the only iOS-specific claim this fixture makes
/// is the triple string in the provenance and the target-mangled entry-point
/// symbols derived from it. The standard Metal path is the opposite case — a second
/// Apple family there needs its own measured rows, which is why
/// `first-authoritative-ios-metal-compile-declaration` is a measurement ticket
/// and why no fixture may wear the macOS rows under another platform.
pub const IOS_TARGET_TRIPLE: &str = "aarch64-apple-ios";

/// Threads per workgroup this scalar execution model admits.
pub const WORKGROUP_THREADS: u32 = 1;

/// Explicitly staged local memory this scalar execution model admits.
pub const LOCAL_MEMORY_BYTES: u64 = 0;

/// Buffer bindings one scalar entry admits.
pub const BUFFER_BINDINGS: u32 = 2;

/// Grid extent one scalar launch admits along its single axis.
pub const GRID_AXIS_THREADS: u64 = 1 << 24;

fn source() -> Result<TargetFactSource, TargetProfileBuildError> {
    let producer = TargetFactProducerIdentity::new("tiler.test.scalar-host-backend".to_owned(), 1)
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    // An external guarantee rather than a measurement: this suite declares what
    // IEEE 754-2019 binary32 requires and does not measure the running host, so
    // the evidence class the profile carries is the one it can support.
    let reference = TargetNormativeReferenceIdentity::new("ieee.754.2019.binary32".to_owned(), 1)
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    Ok(TargetFactSource::external_guarantee(producer, reference))
}

/// Builds the scalar-host target profile this backend compiles against.
///
/// # Errors
///
/// Returns the first typed declaration or freeze diagnostic. Nothing here is
/// expected to fail; it is returned rather than unwrapped so that a widening of
/// the profile vocabulary surfaces as a refusal a test reports rather than a
/// panic inside setup.
pub fn scalar_host_profile() -> Result<TargetProfile, TargetProfileBuildError> {
    let key = TargetProfileKey::new(PROFILE_KEY.to_owned())
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let mut builder = TargetProfileBuilder::new(key);
    let source = source()?;

    builder.declare_max_threads_per_grid_axis(GRID_AXIS_THREADS, source.clone())?;
    builder.declare_max_threads_per_workgroup(WORKGROUP_THREADS, source.clone())?;
    builder.declare_max_buffer_bindings_per_entry(BUFFER_BINDINGS, source.clone())?;
    builder.declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())?;
    builder.declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())?;
    // "An addressable memory space exists" — on a scalar host that is the
    // process heap. The axis asks whether one exists, not whether it is distinct
    // from the host's.
    builder.declare_device_memory(true, source.clone())?;
    builder.declare_local_memory_bytes(LOCAL_MEMORY_BYTES, source.clone())?;

    declare_numerics(&mut builder, &source)?;

    builder.declare_dtype_dispatchability(
        F32::resolved_type(),
        DTypeDispatchability::Dispatchable,
        source,
    )?;
    builder.build()
}

/// Declares the exact numerical behaviours this backend's interpreter honours.
///
/// Deliberately narrower than the governed Metal-shaped profile's, and the
/// opposite of the measured Apple row on subnormals: this backend evaluates the
/// structured body operation by operation in the order the kernel states, so it
/// never contracts, reassociates, permutes, or eliminates a signed zero, and a
/// contract requiring any of those is refused at compile time rather than
/// approximated at run time.
fn declare_numerics(
    builder: &mut TargetProfileBuilder,
    source: &TargetFactSource,
) -> Result<(), TargetProfileBuildError> {
    let f32_subject = ScalarArithmetic::f32();

    let rows = [
        (SubnormalMode::Preserve, ScalarSupport::Exact),
        (
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            ScalarSupport::Unsupported,
        ),
        (
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
            ScalarSupport::Unsupported,
        ),
    ];
    for (behaviour, support) in rows {
        builder.declare_input_subnormals(
            f32_subject.clone(),
            behaviour,
            support,
            source.clone(),
        )?;
        builder.declare_result_subnormals(
            f32_subject.clone(),
            behaviour,
            support,
            source.clone(),
        )?;
    }

    builder.declare_contraction(
        f32_subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_contraction(
        f32_subject.clone(),
        NumericalPermission::Permitted,
        ScalarSupport::Unsupported,
        source.clone(),
    )?;
    builder.declare_reassociation(
        f32_subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_reassociation(
        f32_subject.clone(),
        NumericalPermission::Permitted,
        ScalarSupport::Unsupported,
        source.clone(),
    )?;
    builder.declare_permutation(
        f32_subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_signed_zero(
        f32_subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_reciprocal_transform(
        f32_subject.clone(),
        NumericalPermission::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_approximate_intrinsics(
        f32_subject.clone(),
        ApproximationEnvelope::Forbidden,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_nan_assumptions(
        f32_subject.clone(),
        ExceptionalValueAssumption::MakeNoAssumption,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    builder.declare_infinity_assumptions(
        f32_subject,
        ExceptionalValueAssumption::MakeNoAssumption,
        ScalarSupport::Exact,
        source.clone(),
    )?;
    Ok(())
}
