//! A separately authored physical-implementation provider for Tiler's Metal
//! vertical, written against the public surface only.
//!
//! This crate is the spike's stand-in for a third party that wants to
//! contribute **one** specialized Metal implementation of a region Tiler
//! already implements, without forking `tiler-compiler` and without replacing
//! `tiler-metal`. It deliberately depends on `tiler-compiler` and `tiler-ir`
//! exactly as an out-of-tree crate would: by path, with no feature flag, no
//! `#[path]` include, and no access to any private module.
//!
//! # What it can express
//!
//! Everything a proposal *body* is made of is publicly constructible from
//! `tiler_ir::schedule`, so the specialized implementation below is a real
//! [`ScheduledRegion`] and not a sketch. Its specialization is the workgroup
//! width: [`SPECIALIZED_THREADS_PER_WORKGROUP`] threads per workgroup instead
//! of the governed provider's one
//! (`crates/tiler-compiler/src/physical.rs:495`, `linear_schedule`). That axis
//! is free under the intrinsic verifier, which requires only that
//! `schedule.threads_per_workgroup` equal `schedule.launch.threads_per_workgroup`
//! and be non-zero (`crates/tiler-ir/src/schedule/builder.rs:288`), and it is
//! folded into `CanonicalScheduledRegionIdentity`
//! (`crates/tiler-ir/src/schedule/model.rs:892`), so the two implementations of
//! one region are distinct rather than duplicates.
//!
//! # What it cannot express
//!
//! It cannot implement `tiler_compiler::frontier::PhysicalImplementationProvider`,
//! because `frontier` is a private module and every item a provider would name
//! is `pub(crate)` inside it. Nor could an implementation be installed if the
//! trait were public: `tiler_compiler::session::CompileRequest` has no
//! physical-provider field, and the provider array is a hardcoded one-element
//! literal at `crates/tiler-compiler/src/pipeline/planning.rs:171`. The
//! `probe` crate holds the compile-fail evidence for both statements.
//!
//! So this crate stops at the proposal body, its identity, and its cost — the
//! three things the trait's `propose` would return — and the probe drives them
//! as far as the public surface reaches.

use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ExceptionalValueAssumption,
    ExecutionBinding, FlushedZeroSign, IndexRegion, InputOrdinal, KernelSchedule, LaunchPlan,
    LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof, OwnershipProofKind,
    OwnershipWitnessId, PointwiseF32Expression, PointwiseF32ExpressionBuilder, ReductionTopology,
    RegionId, ScalarProgram, ScheduledRegion, SubnormalMode, TailPolicy, TensorRole,
};
use tiler_ir::semantic::ProviderIdentity;
use tiler_ir::shape::Shape;

/// Namespace of this separately authored provider.
///
/// Deliberately not `tiler`: provider provenance is a versioned identity
/// separated from semantic meaning (ADR 0072), so a third party's proposals
/// must be attributable to the third party.
pub const NAMESPACE: &str = "acme";

/// Name of this separately authored provider.
pub const NAME: &str = "simdgroup-pointwise-metal";

/// Output-affecting revision of this provider.
///
/// Output-affecting in the literal sense the identity contract means: bumping
/// it must accompany a change to the bytes this crate proposes. It is `3`
/// rather than `1` so a reader cannot mistake it for the governed provider's
/// revision (`crates/tiler-compiler/src/frontier.rs:2029`) by coincidence.
pub const REVISION: u32 = 3;

/// The workgroup width this provider specializes on.
///
/// One Apple SIMD group. The governed provider emits one thread per workgroup
/// unconditionally, which is the launch geometry a Metal backend would most
/// obviously want to improve on, and it is the narrowest specialization that is
/// a genuine physical difference rather than a re-spelling.
pub const SPECIALIZED_THREADS_PER_WORKGROUP: u32 = 32;

/// The numerical realization key this provider proposes under.
const REALIZATION_KEY: &str = "acme.spike.flush-subnormals-f32";

/// The canonical arithmetic NaN bit pattern the realization pins.
const CANONICAL_NAN_BITS: u32 = 0x7fc0_0000;

/// Returns this provider's identity.
///
/// # Panics
///
/// Panics only if the compile-time components above violate the canonical
/// provider-identity grammar, which no reachable input can cause.
#[must_use]
pub fn identity() -> ProviderIdentity {
    ProviderIdentity::new(NAMESPACE, NAME, REVISION)
        .expect("the spike provider identity is well formed")
}

/// The pointwise region subject this provider offers an implementation of.
///
/// It mirrors what `ImplementationContext::request()` would expose: the
/// iteration extent and the scale/bias constants of the normalized pointwise
/// program. The real context type is `pub(crate)` in a private module, so the
/// spike restates the three values rather than pretending to receive it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointwiseSubject {
    /// Element count of the one-dimensional iteration domain.
    pub elements: u64,
    /// Bit pattern of the `f32` scale constant.
    pub scale_bits: u32,
    /// Bit pattern of the `f32` bias constant.
    pub bias_bits: u32,
}

impl PointwiseSubject {
    /// The subject the probe drives, matching the bounded profile's shape.
    #[must_use]
    pub const fn spike_default() -> Self {
        Self {
            elements: 256,
            scale_bits: 0x3f80_0000,
            bias_bits: 0,
        }
    }
}

/// Builds the implementation the governed provider would offer for `subject`.
///
/// Present as the *contrast*, not as a copy for its own sake: a specialization
/// claim is only meaningful beside the baseline it specializes, and the two
/// must differ in exactly one axis for the identity comparison in the probe to
/// mean what it says.
#[must_use]
pub fn governed_shaped_region(subject: PointwiseSubject) -> ScheduledRegion {
    region(subject, 1, subject.elements)
}

/// Builds this provider's specialized implementation of `subject`.
///
/// Identical index region, identical scalar program, identical numerical
/// realization; a wider workgroup. That is the whole difference, and it is the
/// difference the request-subject binding permits: the binding compares the
/// region id, iteration shape, scalar program, semantic members, and access map
/// (`crates/tiler-compiler/src/physical.rs:700`) and says nothing about
/// `KernelSchedule`.
#[must_use]
pub fn specialized_region(subject: PointwiseSubject) -> ScheduledRegion {
    region(subject, SPECIALIZED_THREADS_PER_WORKGROUP, subject.elements)
}

/// Builds a deliberately malformed variant of the specialized implementation.
///
/// The perturbation is a launch plan that does not cover the iteration domain:
/// `launch.grid_threads` is one short. Chosen over a malformed *body* the
/// builder could not even hold, because the interesting question is whether a
/// provider that returns a structurally plausible region is believed, and the
/// answer must be that the host's verifier rejects it.
#[must_use]
pub fn malformed_specialized_region(subject: PointwiseSubject) -> ScheduledRegion {
    region(
        subject,
        SPECIALIZED_THREADS_PER_WORKGROUP,
        subject.elements.saturating_sub(1),
    )
}

/// The structural cost estimate this provider would declare.
///
/// Returned as plain numbers because `PhysicalCostEstimate` is `pub(crate)` in
/// a private module, and the governed cost-model key it must be attributed to
/// (`tiler.cost.structural.v1`) is a private constant with no public spelling:
/// a proposal attributing its estimate to any other key is a hard
/// `FrontierError::MalformedCostProvenance`
/// (`crates/tiler-compiler/src/frontier.rs:1470`).
#[must_use]
pub const fn declared_cost(subject: PointwiseSubject) -> (u32, u64, u64) {
    (1, subject.elements, 0)
}

fn region(
    subject: PointwiseSubject,
    threads_per_workgroup: u32,
    grid_threads: u64,
) -> ScheduledRegion {
    ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: Shape::from_dims([subject.elements]),
            accesses: vec![
                Access {
                    tensor: TensorRole::Input {
                        ordinal: InputOrdinal::FIRST,
                    },
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(1),
                    ownership: Some(OwnershipWitnessId::new(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(0),
                    tensor: TensorRole::Input {
                        ordinal: InputOrdinal::FIRST,
                    },
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: subject.elements,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: TensorRole::Output,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: subject.elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: subject.elements,
                },
            },
            scalar_program: ScalarProgram::PointwiseF32(scale_bias_expression(
                subject.scale_bits,
                subject.bias_bits,
            )),
            numerical: realization(),
        },
        schedule: KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: subject.elements,
            threads_per_workgroup,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads,
                threads_per_workgroup,
                zero_work_skips_dispatch: true,
            },
        },
    }
}

/// The scale-then-bias expression the normalized pointwise subject denotes.
///
/// Spelled the same way the compiler spells it
/// (`crates/tiler-compiler/src/physical.rs:517`), because the request-subject
/// binding compares the expression for structural equality against its own
/// construction and rejects an algebraically similar one.
fn scale_bias_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression
        .input(InputOrdinal::FIRST)
        .expect("the fixed expression has exactly one input");
    let scale = expression
        .constant(scale_bits)
        .expect("the fixed expression is within the node limit");
    let product = expression
        .multiply(input, scale)
        .expect("the fixed expression is within the node limit");
    let bias = expression
        .constant(bias_bits)
        .expect("the fixed expression is within the node limit");
    let root = expression
        .add(product, bias)
        .expect("the fixed expression is within the node limit");
    expression
        .build(root)
        .expect("the fixed expression is well formed")
}

/// The numerical realization this provider proposes under.
///
/// Flushing on both subnormal dimensions and every reshaping freedom refused,
/// which is what Apple hardware measurably delivers for `f32`. The compiler
/// would compare this against the request's resolved contract
/// (`crates/tiler-compiler/src/physical.rs:655`); that comparison is one of the
/// gates this spike cannot reach, so the realization is stated here rather than
/// derived from a request.
fn realization() -> NumericalRealization {
    NumericalRealization::new(
        REALIZATION_KEY,
        CANONICAL_NAN_BITS,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}
