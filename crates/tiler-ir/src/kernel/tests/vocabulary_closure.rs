use super::super::{ExecutionScope, MemoryScope};
use super::support::{cooperative_barrier, pointwise_region};
use crate::schedule::{
    ExecutionBinding, LogicalAccess, ReductionPass, ReductionTopology, RegionId, RegionProgram,
    ScalarProgram, TailPolicy,
};
use crate::shape::Shape;

/// Compile-time tripwire for `revisit-kernel-body-single-spelling-gate`.
///
/// The refinement gate re-derives the canonical body with
/// `lower::derive_canonical` — a deterministic function of the scheduled region
/// — and requires structural equality, so the profile admits **exactly one
/// spelling** of a legal kernel. That is correct only while the surface is
/// narrow enough that no two genuinely different bodies are both legal for one
/// region. Past that point derive-and-compare starts rejecting *valid* kernels.
///
/// The ticket names that widening as its trigger for reconsideration, and a
/// trigger nobody is told about is a trigger nobody notices. These matches are
/// exhaustive with no wildcard arm, so adding a variant to any of the closed
/// vocabularies that decide a body's shape is a **compile error here**. Whoever
/// hits it should read that ticket before adding an arm: the fix may be to widen
/// the gate rather than to widen this match.
///
/// Deliberately a spelling check, not a semantic one — it cannot tell that a
/// widened vocabulary admits two bodies, only that the vocabulary widened, which
/// is the point at which a human has to look.
fn body_shaping_vocabulary_is_closed(
    binding: &ExecutionBinding,
    tail: TailPolicy,
    access: &LogicalAccess,
    topology: &ReductionTopology,
    program: &ScalarProgram,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    (
        match binding {
            ExecutionBinding::GlobalLinearInvocation => "global-linear-invocation",
            ExecutionBinding::BlockedWorkgroup { .. } => "blocked-workgroup",
            // Widens the vocabulary without widening the *lowered* profile:
            // `plan` refuses this binding as `unlowered-execution-binding`
            // before any body is derived, so no scheduled region carrying it
            // has even one legal body, let alone two. Trigger re-checked and
            // not fired; see `revisit-kernel-body-single-spelling-gate`'s
            // 2026-08-18 log entry.
            ExecutionBinding::FixedVectorMap { .. } => "fixed-vector-map",
        },
        match tail {
            TailPolicy::Exact => "exact",
            TailPolicy::Predicated => "predicated",
        },
        match access {
            LogicalAccess::LinearIdentity => "linear-identity",
            LogicalAccess::ScalarBroadcast => "scalar-broadcast",
            LogicalAccess::PackedU4LsbZeroTail { .. } => "packed-u4-lsb-zero-tail",
            LogicalAccess::ReductionContributor { .. } => "reduction-contributor",
            LogicalAccess::ContractionOperand { .. } => "contraction-operand",
            LogicalAccess::ReindexBijection { .. } => "reindex-bijection",
            LogicalAccess::BroadcastReplication { .. } => "broadcast-replication",
            LogicalAccess::ParametricBroadcast { .. } => "parametric-broadcast",
            LogicalAccess::LiveRowMajorSource { .. } => "live-row-major-source",
            LogicalAccess::LiveRowMajor => "live-row-major",
            LogicalAccess::PartitionedCopySource => "partitioned-copy-source",
            LogicalAccess::GatherSource { .. } => "gather-source",
        },
        match topology {
            ReductionTopology::None => "none",
            ReductionTopology::Serial { .. } => "serial",
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                ..
            } => "multi-pass-partial",
            ReductionTopology::MultiPass {
                pass: ReductionPass::Final,
                ..
            } => "multi-pass-final",
            ReductionTopology::Contraction { .. } => "contraction",
            ReductionTopology::LiveContraction { .. } => "live-contraction",
            ReductionTopology::CooperativeWorkgroup { .. } => "cooperative-workgroup",
            ReductionTopology::CooperativeContraction { .. } => "cooperative-contraction",
        },
        match program {
            ScalarProgram::PointwiseF32(_) => "pointwise-f32",
            ScalarProgram::PointwiseBf16(_) => "pointwise-bf16",
            ScalarProgram::StrictAffineU4Dequantize { .. } => "strict-affine-u4-dequantize",
            ScalarProgram::StrictSerialSum { .. } => "strict-serial-sum",
            ScalarProgram::FusedMultiplyAddSerialSum { .. } => "fused-multiply-add-serial-sum",
            ScalarProgram::SquaredSerialSum { .. } => "squared-serial-sum",
            ScalarProgram::SquaredSerialSumThenEpilogue { .. } => "squared-serial-sum-epilogue",
            ScalarProgram::StrictTensorContraction { .. } => "strict-tensor-contraction",
            ScalarProgram::StrictSerialMaximum { .. } => "strict-serial-maximum",
        },
    )
}

/// The single-spelling gate's precondition still holds.
///
/// One execution binding and one tail policy is the substance of it: with a
/// single way to bind invocations to coordinates and no tail to handle, a
/// scheduled region's body has no legal degree of freedom for a producer to
/// spell differently. See [`body_shaping_vocabulary_is_closed`].
#[test]
fn the_single_spelling_profile_is_still_narrow_enough_for_derive_and_compare() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let region = scheduled.region();
    let names = body_shaping_vocabulary_is_closed(
        &region.schedule.binding,
        region.schedule.tail,
        &region.index.accesses[0].map,
        &region.schedule.reduction,
        match &region.index.program {
            RegionProgram::Numerical { scalar, .. } => scalar,
            RegionProgram::PartitionedCopy(_) => panic!("the fixture region is arithmetic"),
        },
    );
    assert_eq!(
        names,
        (
            "global-linear-invocation",
            "exact",
            "linear-identity",
            "none",
            "pointwise-f32",
        )
    );
}

/// Compile-time tripwire for `add-subgroup-memory-scope-when-collectives-land`.
///
/// A barrier states its execution scope and its memory scope as two separate
/// vocabularies (ADR 0048), and the pair is asymmetric today: [`ExecutionScope`]
/// names a subgroup and [`MemoryScope`] cannot, so subgroup-level visibility is
/// inexpressible and `tiler-metal` refuses every subgroup barrier rather than
/// widening its claim to workgroup visibility. That deferred ticket owns closing
/// the asymmetry.
///
/// Widening either enum is *already* a build error inside this crate: `tag` on
/// each enum and `verify::barrier_subject` are exhaustive, and
/// `#[non_exhaustive]` has no effect on a match in the defining crate. What
/// neither of those errors says is what happens downstream. `barrier_call` in
/// `crates/tiler-metal/src/emit.rs` matches both scopes with wildcard arms, so a
/// widened scope compiles there and every barrier naming it keeps being rejected
/// at run time with a typed `UnsupportedBarrier`. Those wildcards are correct and
/// stay — out of crate `#[non_exhaustive]` requires one, and they are what makes
/// an unhandled scope a typed rejection rather than a panic. This match is the
/// break that carries the instructions: whoever hits it should read that ticket
/// before adding an arm.
///
/// The two scope vocabularies only. [`BarrierOrdering`] and [`AddressSpace`] are
/// wildcarded in the same emitter, but no ticket owns widening either, and a
/// tripwire that names no owner is a build error with nothing to say.
///
/// Deliberately a spelling check and not a semantic one, exactly as
/// [`body_shaping_vocabulary_is_closed`] states: it cannot tell that a widened
/// vocabulary admits a new barrier, only that the vocabulary widened, which is
/// the point at which a human has to look. It cites constructs rather than
/// lines, because every line citation this tripwire inherited had drifted by the
/// time it was read.
fn barrier_scope_vocabulary_is_closed(
    execution: ExecutionScope,
    memory: MemoryScope,
) -> (&'static str, &'static str) {
    (
        match execution {
            ExecutionScope::Subgroup => "subgroup",
            ExecutionScope::Workgroup => "workgroup",
        },
        match memory {
            MemoryScope::Workgroup => "workgroup",
            MemoryScope::Device => "device",
        },
    )
}

/// The barrier scope vocabularies are still the pair the backends were built for.
///
/// Consumes the spelling of a real cooperative handoff rather than literals, so
/// the tripwire is anchored to a barrier the profile actually emits.
#[test]
fn the_barrier_scope_vocabularies_are_still_closed() {
    let spec = cooperative_barrier();
    assert_eq!(
        barrier_scope_vocabulary_is_closed(spec.execution_scope, spec.memory_scope),
        ("workgroup", "workgroup")
    );
}
