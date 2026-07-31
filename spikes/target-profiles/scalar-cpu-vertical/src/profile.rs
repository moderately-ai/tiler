//! The bounded scalar CPU target profile this spike declares, and the host
//! facts it is declared *about*.
//!
//! # What a declared profile is here
//!
//! `tiler_compiler::target::TargetProfileBuilder` builds a **sparse** profile:
//! "every omitted quantitative axis remains unknown". That property is what
//! makes a bounded CPU declaration expressible at all. Every axis this module
//! declares is a positive claim with a named authority; every axis it does not
//! declare is `Unknown`, not "absent" and not "zero" — and vector width, mask
//! and tail support, scalable-vector length, thread count, task granularity, and
//! oversubscription are all in the second group, because this spike measured
//! none of them and the profile vocabulary has no axis for them either.
//!
//! # The two facts this profile states that no CPU has
//!
//! `WorkgroupThreads` and `LocalMemoryBytes` are GPU axes. A scalar CPU
//! realization has no workgroup and no explicitly staged local memory, and the
//! profile still has to say something about both, because a kernel's derived
//! `ResourceRequirements` carries a `threads_per_workgroup` and a
//! `local_memory_bytes` that feasibility compares against a profile bound. The
//! honest reading of the neutral vocabulary is that a scalar host executes each
//! invocation independently — one invocation per "workgroup" — and stages no
//! local memory, so the declarations below are `1` and `0`. They are recorded
//! here as the exact place the neutral axis set is narrower than the targets it
//! claims to be neutral over; `README.md` carries the finding.

use tiler_compiler::target::{
    DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
    TargetProfile, TargetProfileBuildError, TargetProfileKey,
};
use tiler_ir::schedule::{
    ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission, SubnormalMode,
};
use tiler_ir::semantic::F32;

/// Governed profile key of this spike's declared scalar CPU target.
///
/// Distinct from `tiler.target.governed-prototype` by construction: a host
/// stating this key against an artifact packaged for the governed Metal-shaped
/// profile is refused as a `ProfileKeyMismatch` rather than as a descriptor
/// revision, which is the distinction ADR 0043 draws and the spike perturbs.
pub const PROFILE_KEY: &str = "tiler.target.cpu-scalar-host-aarch64-darwin";

/// Governed backend family key of the scalar CPU backend.
pub const BACKEND_KEY: &str = "tiler.cpu.scalar";

/// Governed executable-representation key of its carried payload.
pub const REPRESENTATION_KEY: &str = "tiler.cpu.scalar-image-v1";

/// Governed representation key of the retained payload "source".
///
/// The scalar image is translated from verified structured KIR rather than from
/// text, so the retained source is the canonical kernel identity the image was
/// translated from. Naming it as its own representation keeps the payload's
/// compilation subject honest: there is no `.c` file behind this payload and
/// claiming one would be provenance nobody can check.
pub const SOURCE_REPRESENTATION_KEY: &str = "tiler.cpu.kernel-identity-list-v1";

/// The declared target triple this profile is about.
///
/// A string rather than a typed axis, because the profile vocabulary has no
/// target-triple axis: `CapabilityAxis` covers grid threads, workgroup threads,
/// buffer bindings, index arithmetic, device address width, device address
/// space, and local memory bytes, and nothing else. The triple therefore
/// survives only inside the profile *key* and inside the payload provenance,
/// which is recorded as a finding rather than worked around.
pub const TARGET_TRIPLE: &str = "aarch64-apple-darwin";

/// The declared C ABI and data layout this profile is about.
pub const DATA_LAYOUT: &str = "AAPCS64/Darwin: f32 4-byte aligned little-endian, 64-bit pointers";

/// Declared address width of the scalar CPU address model.
pub const ADDRESS_WIDTH: DeviceAddressWidth = DeviceAddressWidth::Bits64;

/// Threads per workgroup this scalar execution model admits.
///
/// One. A scalar host runs one invocation at a time and shares nothing between
/// invocations, which is exactly the neutral meaning of a workgroup of one.
pub const WORKGROUP_THREADS: u32 = 1;

/// Explicitly staged local memory this scalar execution model admits.
pub const LOCAL_MEMORY_BYTES: u64 = 0;

/// Buffer bindings one scalar entry admits.
///
/// Two, matching the governed profile: every kernel this bounded profile can
/// realize destructures to one read buffer and one write buffer. A wider
/// signature is refused by feasibility rather than executed.
pub const BUFFER_BINDINGS: u32 = 2;

/// Grid extent one scalar launch admits along its single axis.
///
/// `1 << 24` invocations, chosen as a bound this spike can state rather than a
/// hardware maximum: a scalar interpreter's grid is a `for` loop over `u64`, so
/// nothing physical caps it, and declaring `u64::MAX` would be claiming a
/// launch nobody has run. It bounds this spike's workload by a wide margin.
pub const GRID_AXIS_THREADS: u64 = 1 << 24;

/// The producer identity every fact in this profile is attributed to.
fn producer() -> Result<TargetFactProducerIdentity, TargetProfileBuildError> {
    TargetFactProducerIdentity::new("tiler.spike.scalar-cpu-vertical".to_owned(), 1)
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)
}

/// The normative reference the arithmetic facts are guaranteed against.
///
/// IEEE 754-2019 binary32, which is what Rust's `f32` operations are specified
/// to realize. This is an *external guarantee*, deliberately not a measurement:
/// the spike separately measures the running host and refuses a disagreement,
/// so the two evidence classes stay apart.
fn reference() -> Result<TargetNormativeReferenceIdentity, TargetProfileBuildError> {
    TargetNormativeReferenceIdentity::new("ieee.754.2019.binary32".to_owned(), 1)
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)
}

fn source() -> Result<TargetFactSource, TargetProfileBuildError> {
    Ok(TargetFactSource::external_guarantee(
        producer()?,
        reference()?,
    ))
}

/// Builds this spike's bounded scalar CPU target profile.
///
/// # Errors
///
/// Returns the first typed declaration or freeze diagnostic. Nothing here is
/// expected to fail, and it is returned rather than unwrapped so a widening of
/// the profile vocabulary shows up as a refusal the caller reports instead of a
/// panic inside a spike's setup.
pub fn scalar_cpu_profile() -> Result<TargetProfile, TargetProfileBuildError> {
    let key = TargetProfileKey::new(PROFILE_KEY.to_owned())
        .map_err(|_| TargetProfileBuildError::InvalidProducerClaim)?;
    let mut builder = tiler_compiler::target::TargetProfileBuilder::new(key);
    let source = source()?;

    builder.declare_max_threads_per_grid_axis(GRID_AXIS_THREADS, source.clone())?;
    // An available fact, not a deferred query. This is the first structural
    // difference from the governed Metal-shaped profile, which defers the
    // workgroup bound to a prepared-pipeline query because only a built
    // pipeline knows its own register pressure. A scalar interpreter's answer
    // is one, known at declaration time, so the CPU vertical routes through the
    // device-free `preflight` path and never mints a deferred predicate.
    builder.declare_max_threads_per_workgroup(WORKGROUP_THREADS, source.clone())?;
    builder.declare_max_buffer_bindings_per_entry(BUFFER_BINDINGS, source.clone())?;
    builder.declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())?;
    builder.declare_device_address_width(ADDRESS_WIDTH, source.clone())?;
    // "An explicitly addressable memory space exists" — on a scalar CPU that is
    // the process heap. The axis asks whether one exists, not whether it is
    // separate from the host's, so `true` is the accurate answer and the
    // absence of a *distinct* device domain is a placement fact this profile
    // does not carry.
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

/// Declares the exact numerical behaviours a scalar CPU interpreter honours.
///
/// The table is deliberately narrower than the governed profile's. Every
/// reshaping freedom is declared `Unsupported`, because this backend evaluates
/// the structured body operation by operation in the order the kernel states
/// it: it never contracts a multiply and an add, never reassociates a
/// reduction, never permutes operands, and never eliminates a signed zero. A
/// contract that requires any of those is therefore refused at compile time,
/// which the spike perturbs and observes.
fn declare_numerics(
    builder: &mut tiler_compiler::target::TargetProfileBuilder,
    source: &TargetFactSource,
) -> Result<(), TargetProfileBuildError> {
    let f32_subject = ScalarArithmetic::f32();

    // Subnormals: preserved exactly, and both flushing realizations explicitly
    // unsupported. This is the opposite of the measured Apple row, and it is
    // the whole reason a second backend is worth building: the two targets
    // cannot both be described by one implicit default.
    // The three-row table is stated per dimension rather than looped over a
    // function pointer, so each row reads as the claim it is.
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
