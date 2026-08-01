//! Target-neutral scheduled-region data model, verified wrapper, and identity.
//!
//! A [`ScheduledRegion`] pairs a bounded [`IndexRegion`] with a normalized
//! [`KernelSchedule`] (ADR 0007). The descriptor structs are read-transparent
//! value data; only [`super::ScheduledRegionBuilder::build`] can bind a region
//! into an opaque [`VerifiedScheduledRegion`] after intrinsic verification.

use crate::identity::{push_len, push_slice};
use crate::semantic::EncodedComponentRole;
use crate::shape::{Axis, Shape};

use super::cooperative::{
    CooperativePhase, CooperativeTile, LocalCoordinates, ParticipantRange, StagedRead, StagedSpan,
    StagedWrite, WorkgroupStaging,
};
use super::error::{ContributorError, ElementCountOverflow};
use super::handles::{BoundsWitnessId, InputOrdinal, OwnershipWitnessId, RegionId};
use super::numerics::{
    ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
    NumericalRealization, SubnormalFreedom, SubnormalMode, ValueDomainProvenance,
};
use super::pointwise::{PointwiseF32Expression, PointwiseF32Node};

/// The role a boundary tensor plays for one scheduled region.
///
/// A region may read several distinct input tensors, so `Input` names *which*
/// one. The ordinal lives on the role rather than beside it because the role is
/// what travels: a buffer parameter, a bounds proof, a boundary requirement, and
/// an opaque call binding each carry a `TensorRole` and nothing else that could
/// separate two reads, and two facts that must always travel together are one
/// value. The sibling [`crate::index`] region reaches the same separation by
/// computing a positional ordinal among same-role tensors when it encodes
/// identity; this states it instead of deriving it.
///
/// **Do not add `#[non_exhaustive]`.** This is an ADR 0074 convention 5b type
/// for the same reason [`AccessMode`] is: `tensor_role_tag` in `tiler-compiler`'s
/// `selection.rs` and `frontier.rs`, and `tensor_role_name` in its
/// `call_registry.rs`, map it *totally* onto identity tags and subject strings
/// from outside this crate with no wildcard arm. A wildcard there would have to
/// invent a tag the variant alone determines, so a variant added later would
/// encode under some other variant's bytes instead of failing the build.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TensorRole {
    /// One named program input consumed by the region.
    Input {
        /// Which of the region's input tensors this is.
        ordinal: InputOrdinal,
    },
    /// A materialized intermediate produced or consumed by the region.
    Intermediate,
    /// A program output produced by the region.
    Output,
}

/// Whether an access reads or writes its tensor.
///
/// **Do not add `#[non_exhaustive]`.** This is an ADR 0074 convention 5b type:
/// `access_mode_tag` in `tiler-compiler`'s `selection.rs` and `frontier.rs` map
/// it *totally* onto identity tags from outside this crate, with no wildcard
/// arm. Marking it would make those matches a cross-crate `E0004` and force the
/// wildcard back in, and a wildcard there would have to invent an identity tag
/// that the variant alone determines — so a variant added later would encode
/// under some other variant's bytes instead of failing the build. The exhaustive
/// match is the mechanism that makes adding a variant a compile error at every
/// encoder, which is exactly what the attribute would remove.
///
/// Adding a variant here is therefore expected to break both encoders, and that
/// break is the design. Give the new variant its own tag at each site rather
/// than widening an existing one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AccessMode {
    /// The access reads its tensor.
    Read,
    /// The access writes its tensor.
    Write,
}

/// Canonical order in which reduction contributors combine.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a new contributor order lands additively. Verified by
/// marking it and compiling the workspace — no consumer broke. Total maps
/// *inside* `tiler-ir` are unaffected, because the attribute has no effect
/// within the defining crate, which is what keeps them breaking.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ContributorOrder {
    /// Contributors combine in ascending original-axis lexicographic order.
    OriginalAxisLexicographic,
}

/// Which coordinate one contraction operand axis reads.
///
/// A contraction's iteration space is the product of its output shape and its
/// contracted shape, and each operand axis binds to exactly one coordinate of
/// one of the two. Stating that per axis is what makes a general binary index
/// structure addressable without an index-arithmetic vocabulary: `td,od->to`
/// gives operand 0 the sources `[Output { 0 }, Contracted { 0 }]` and operand 1
/// `[Output { 1 }, Contracted { 0 }]`, and a structure whose contracted index
/// sits at a different axis of each operand is expressed the same way.
///
/// **Do not add `#[non_exhaustive]`.** This is an ADR 0074 convention 5b type
/// for the reason [`AccessMode`] is: the identity encoder in this crate and
/// `tiler-compiler`'s region construction and subject binding map it *totally*,
/// with no wildcard arm, and a wildcard would have to invent an identity tag the
/// variant alone determines.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContractionAxisSource {
    /// The coordinate at this position of the contraction's output shape.
    Output {
        /// Position within the output shape, in axis order.
        position: u32,
    },
    /// The coordinate at this position of the contracted iteration shape.
    Contracted {
        /// Position within the contracted shape, in axis order.
        position: u32,
    },
}

/// The logical coordinate map a scheduled access realizes.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a new coordinate map lands additively. Verified by
/// marking it and compiling the workspace — no consumer broke. Total maps
/// *inside* `tiler-ir` are unaffected, because the attribute has no effect
/// within the defining crate, which is what keeps them breaking.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LogicalAccess {
    /// One iteration coordinate maps to one linear element position.
    LinearIdentity,
    /// Every invocation reads the single scalar parameter element.
    ScalarBroadcast,
    /// One logical U4 position addresses an LSB-first nibble in a U8 carrier.
    ///
    /// An odd final logical element owns the low nibble of the last carrier;
    /// the unused high nibble must be zero before dispatch.
    PackedU4LsbZeroTail {
        /// Number of logical U4 elements represented by the carriers.
        logical_elements: u64,
    },
    /// Each output coordinate reads a family of contributor coordinates.
    ReductionContributor {
        /// Shape of the reduced input.
        input_shape: Shape,
        /// Shape of the reduced output.
        output_shape: Shape,
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
    },
    /// One operand of a tensor contraction, addressed by the output and
    /// contracted coordinates its index tuple names.
    ///
    /// Deliberately not a [`Self::ReductionContributor`] with a wider shape. A
    /// reduction's contributor family is a sub-shape of *one* input, so its
    /// output shape is that input's shape with the reduced axes removed. A
    /// contraction's two operands each name a different subset of the free
    /// indices, so neither operand's shape stands in that relation to the
    /// output, and `input_shape.without_axes(axes)` — the equality the reduction
    /// bounds proof checks — has no contraction analogue.
    ContractionOperand {
        /// Shape of this operand.
        operand_shape: Shape,
        /// Shape of the contraction's output.
        output_shape: Shape,
        /// Row-major shape of the contracted iteration space, in ascending
        /// canonical contracted-index order.
        contracted_shape: Shape,
        /// Per operand axis, in axis order, which coordinate that axis reads.
        sources: Vec<ContractionAxisSource>,
        /// Contributor combination order.
        order: ContributorOrder,
    },
}

/// One logical tensor access performed by a scheduled region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Access {
    /// Boundary tensor role.
    pub tensor: TensorRole,
    /// Semantic component role, or `None` for a dense value.
    pub component_role: Option<EncodedComponentRole>,
    /// Whether the access reads or writes.
    pub mode: AccessMode,
    /// Logical coordinate map.
    pub map: LogicalAccess,
    /// Bounds proof witness attached to this access.
    pub bounds: BoundsWitnessId,
    /// Write-ownership witness, present only for owning writes.
    pub ownership: Option<OwnershipWitnessId>,
}

/// The structure a bounds proof establishes for an access domain.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a new bounds-proof structure lands additively. Verified by
/// marking it and compiling the workspace — no consumer broke. Total maps
/// *inside* `tiler-ir` are unaffected, because the attribute has no effect
/// within the defining crate, which is what keeps them breaking.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BoundsProofKind {
    /// A contiguous linear range of `element_count` positions.
    LinearRange {
        /// Number of in-range positions.
        element_count: u64,
    },
    /// A reduction domain relating input and output coordinates.
    ReductionDomain {
        /// Shape of the reduced input.
        input_shape: Shape,
        /// Shape of the reduced output.
        output_shape: Shape,
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
    },
}

/// A witnessed proof that an access stays within its tensor bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundsProof {
    /// Witness identity referenced by the proving access.
    pub id: BoundsWitnessId,
    /// Tensor the proof applies to.
    pub tensor: TensorRole,
    /// Semantic component role, or `None` for a dense value.
    pub component_role: Option<EncodedComponentRole>,
    /// Proven domain structure.
    pub kind: BoundsProofKind,
}

/// The structure a write-ownership proof establishes.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a new ownership-proof structure lands additively. Verified by
/// marking it and compiling the workspace — no consumer broke. Total maps
/// *inside* `tiler-ir` are unaffected, because the attribute has no effect
/// within the defining crate, which is what keeps them breaking.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OwnershipProofKind {
    /// Exactly one global invocation writes each of `output_count` positions.
    OneGlobalInvocationPerOutput {
        /// Number of distinct owned output positions.
        output_count: u64,
    },
}

/// A witnessed proof that writes are total and race-free.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnershipProof {
    /// Witness identity referenced by the schedule and owning write.
    pub id: OwnershipWitnessId,
    /// Tensor the proof applies to.
    pub tensor: TensorRole,
    /// Proven ownership structure.
    pub kind: OwnershipProofKind,
}

/// The scalar program a region evaluates per output.
///
/// This type is deliberately **not** `#[non_exhaustive]`.
///
/// `tiler-compiler`'s `physical.rs` maps it totally from outside this crate, so
/// marking it would force a wildcard arm there and a variant added later would
/// be silently mis-handled instead of failing the build. Verified by marking it
/// and watching that consumer fail to compile.
///
/// An out-of-crate exhaustive match therefore compiles, and must keep doing so:
///
/// ```
/// use tiler_ir::schedule::{ContributorOrder, ScalarProgram};
/// fn is_reduction(program: &ScalarProgram) -> bool {
///     match program {
///         ScalarProgram::PointwiseF32(_) => false,
///         ScalarProgram::StrictAffineU4Dequantize { .. } => false,
///         ScalarProgram::StrictSerialSum { .. }
///         | ScalarProgram::SquaredSerialSum { .. }
///         | ScalarProgram::FusedMultiplyAddSerialSum { .. }
///         | ScalarProgram::StrictTensorContraction { .. } => true,
///     }
/// }
/// let program = ScalarProgram::StrictSerialSum {
///     axes: Vec::new(),
///     order: ContributorOrder::OriginalAxisLexicographic,
///     canonical_nan_bits: 0x7FC0_0000,
///     empty_identity_bits: 0,
/// };
/// assert!(is_reduction(&program));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScalarProgram {
    /// An exact physical IEEE-754 binary32 pointwise expression.
    PointwiseF32(PointwiseF32Expression),
    /// Strict per-tensor affine U4-to-F32 dequantization.
    ///
    /// Codes use packed U4 in LSB-first, zero-tail U8 carriers; scale is an
    /// unpacked positive *normal* F32 scalar, as the governed strict-affine
    /// value contract requires; zero point is an unpacked U8 carrier whose
    /// semantic value is in `0..=15`. The normality of the scale is what
    /// [`super::VerifiedScheduledRegion::subnormal_freedom`] rests on.
    StrictAffineU4Dequantize {
        /// Role of the logical-shape code component.
        codes_role: EncodedComponentRole,
        /// Role of the per-tensor F32 scale component.
        scale_role: EncodedComponentRole,
        /// Role of the per-tensor U4 zero-point component.
        zero_point_role: EncodedComponentRole,
    },
    /// A strict serial reduction sum.
    StrictSerialSum {
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Canonical arithmetic NaN bit pattern.
        canonical_nan_bits: u32,
        /// Empty-reduction identity bit pattern.
        empty_identity_bits: u32,
    },
    /// A fused scale-bias-then-serial-sum reduction.
    FusedMultiplyAddSerialSum {
        /// Scale constant bit pattern.
        scale_bits: u32,
        /// Bias constant bit pattern.
        bias_bits: u32,
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Canonical arithmetic NaN bit pattern.
        canonical_nan_bits: u32,
        /// Empty-reduction identity bit pattern.
        empty_identity_bits: u32,
        /// Whether contraction is permitted.
        contraction: bool,
    },
    /// A serial sum whose per-contributor prologue squares the contributor.
    ///
    /// The reduction `tiler::rms-norm-f32@1` embeds, and a *prologue* rather than
    /// a new reducer: `mean(x^2)` is the ordered sum the strict serial reduction
    /// already defines, applied to an elementwise squaring of each contributor.
    /// It is a variant of its own rather than a
    /// [`Self::FusedMultiplyAddSerialSum`] with contrived constants, because
    /// `scale * x + bias` cannot express `x * x` for any pair of constants: the
    /// prologue is quadratic in the contributor and the fused form is affine.
    ///
    /// The squaring rounds once per contributor and the fold rounds once per
    /// combine, exactly as the semantic reference states. There is deliberately no
    /// epilogue field: the division by the extent, the `eps` addition, the
    /// reciprocal square root, and the two multiplies belong to the pointwise pass
    /// that consumes this reduction's result, and folding them in here would make
    /// one region carry two different iteration domains.
    SquaredSerialSum {
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Canonical arithmetic NaN bit pattern.
        canonical_nan_bits: u32,
        /// Empty-reduction identity bit pattern.
        empty_identity_bits: u32,
    },
    /// A strict tensor contraction of two operands over a shared index space.
    ///
    /// One product per contracted point, each rounded once, folded in ascending
    /// contracted order from the *first product*. It is not a
    /// [`Self::FusedMultiplyAddSerialSum`] with a tensor scale: that program
    /// applies two constants to one contributor, and this one multiplies two
    /// loaded values whose coordinates differ. It is not a
    /// [`Self::StrictSerialSum`] over a materialized product either — that
    /// materialization would round the product at an observable boundary and
    /// cost a full temporary of the iteration space's size.
    ///
    /// **There is deliberately no empty-domain identity.** The registered
    /// contraction declares `refused-an-unseeded-fold-has-no-empty-result`, so
    /// there is no value an empty contracted domain could commit; the schedule
    /// verifier refuses a contracted shape with no points instead. A field
    /// carrying an identity here would be a value that can never be correct.
    StrictTensorContraction {
        /// Row-major shape of the contracted iteration space, in ascending
        /// canonical contracted-index order.
        contracted_shape: Shape,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Canonical arithmetic NaN bit pattern.
        canonical_nan_bits: u32,
    },
}

/// The bounded index region a schedule maps onto a target machine.
///
/// This carries the iteration domain, logical accesses, bounds and ownership
/// proofs, the scalar program, and the numerical realization. It deliberately
/// does not carry any semantic-graph correlation; binding a region to semantic
/// occurrences is a separate compiler-owned refinement (ADR 0070).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRegion {
    /// Planning ordinal, excluded from canonical identity.
    pub id: RegionId,
    /// Parallel iteration domain of the region.
    pub iteration_shape: Shape,
    /// Logical accesses, one read followed by one owning write.
    pub accesses: Vec<Access>,
    /// Bounds proofs, one per access.
    pub bounds_proofs: Vec<BoundsProof>,
    /// The single write-ownership proof.
    pub ownership_proof: OwnershipProof,
    /// Scalar program evaluated per output.
    pub scalar_program: ScalarProgram,
    /// Preserved numerical realization.
    pub numerical: NumericalRealization,
}

/// How a region binds execution coordinates to iteration coordinates.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a new execution binding lands additively. Verified by
/// marking it and compiling the workspace — no consumer broke. Total maps
/// *inside* `tiler-ir` are unaffected, because the attribute has no effect
/// within the defining crate, which is what keeps them breaking.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExecutionBinding {
    /// One global linear invocation per iteration coordinate.
    GlobalLinearInvocation,
}

/// How iteration-domain tail elements are handled.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a new tail policy lands additively. Verified by
/// marking it and compiling the workspace — no consumer broke. Total maps
/// *inside* `tiler-ir` are unaffected, because the attribute has no effect
/// within the defining crate, which is what keeps them breaking.
///
/// # Additive growth, proven in both directions
///
/// An out-of-crate exhaustive match is a compile error, which is what makes the
/// seam additive rather than merely documented. The error code is named, because
/// a bare `compile_fail` passes on *any* failure: a first attempt at this
/// doctest failed with `E0599` on a variant that does not exist, and a bare
/// `compile_fail` would have recorded that as coverage.
///
/// ```compile_fail,E0004
/// use tiler_ir::schedule::TailPolicy;
/// fn classify(policy: TailPolicy) -> u8 {
///     match policy {
///         TailPolicy::Exact => 0,
///     }
/// }
/// ```
///
/// Construction and a wildcard match still compile, so the attribute cannot be
/// over-applied without this noticing:
///
/// ```
/// use tiler_ir::schedule::TailPolicy;
/// let policy = TailPolicy::Exact;
/// let named = match policy {
///     TailPolicy::Exact => "exact",
///     _ => "other",
/// };
/// assert_eq!(named, "exact");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TailPolicy {
    /// The launch geometry covers the domain exactly with no tail.
    Exact,
}

/// The reduction topology and combination legality of a schedule.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a new reduction topology lands additively. Verified by
/// marking it and compiling the workspace — no consumer broke. Total maps
/// *inside* `tiler-ir` are unaffected, because the attribute has no effect
/// within the defining crate, which is what keeps them breaking.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReductionTopology {
    /// The region performs no reduction.
    None,
    /// The region reduces serially over the given axes.
    Serial {
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Whether the contract permits reassociation.
        permits_reassociation: bool,
        /// Whether the contract permits contributor permutation.
        permits_permutation: bool,
    },
    /// The region realizes one pass of a split, multi-dispatch reduction.
    ///
    /// The split writes and reads explicit partial tensors across a dispatch
    /// boundary, so it needs no intra-workgroup barrier: the partial pass
    /// commits its values, and the dispatch dependency the kernel program
    /// declares is what makes them visible to the final pass.
    ///
    /// Every field of the split is stated rather than implied by the pass
    /// count. `partition` fixes the partial shape and storage extent, `order`
    /// fixes each pass's reduction order, `accumulation` fixes the width each
    /// combining step is performed at, and the region's scalar program fixes
    /// the empty-domain identity — a pass that inherited any of these from its
    /// sibling would be a contract two regions could disagree about.
    MultiPass {
        /// Which pass of the split this region realizes.
        pass: ReductionPass,
        /// How the contributor sequence is split across partial values.
        partition: ContributorPartition,
        /// Reduced axes of *this pass*, in canonical ascending order.
        ///
        /// The partial pass reduces the original axes; the final pass reduces
        /// the single partition axis of the partial tensor, so the two passes
        /// deliberately name different axis sets.
        axes: Vec<Axis>,
        /// Contributor combination order within this pass.
        order: ContributorOrder,
        /// Width every combining step of this pass is performed at.
        ///
        /// Carried explicitly rather than inherited from the element type: a
        /// strategy that accumulated at a narrower width than the contract
        /// admits is a different computation, and a field the region does not
        /// carry cannot be rejected for being wrong.
        accumulation: ArithmeticType,
        /// Whether the contract permits reassociation.
        ///
        /// A multi-pass split *is* a reassociation of the declared contributor
        /// sequence, so the schedule verifier admits this topology only when
        /// this is true.
        permits_reassociation: bool,
        /// Whether the contract permits contributor permutation.
        ///
        /// Recorded because the region preserves the declared realization
        /// whole, and deliberately *not* consulted to admit the split: this
        /// strategy preserves contributor order, so granting permutation never
        /// makes an otherwise illegal split legal.
        permits_permutation: bool,
    },
    /// The region folds one contraction's contracted index space per output.
    ///
    /// A topology of its own rather than a [`Self::Serial`] carrying the
    /// contracted axes, because `Serial`'s `axes` name axes of *one* read
    /// tensor and a contraction's contracted index generally sits at a
    /// different axis of each operand — `abc,b->ac` binds its summed index to
    /// axis 1 of operand 0 and axis 0 of operand 1. One `Vec<Axis>` cannot say
    /// that, so reusing `Serial` would give one field two meanings and leave
    /// the general structure unstatable.
    ///
    /// The fold itself is serial and preserves order exactly as `Serial` does,
    /// which is why both permissions appear here with the same meaning and why
    /// the contributor-loop obligation is the shared one.
    Contraction {
        /// Row-major shape of the contracted iteration space, in ascending
        /// canonical contracted-index order.
        contracted_shape: Shape,
        /// Contributor combination order within the contracted space.
        order: ContributorOrder,
        /// Whether the contract permits reassociation.
        ///
        /// Recorded and cross-checked against the region's declared
        /// realization, and deliberately not consulted to admit the topology:
        /// this fold is the declared contributor sequence itself, so it
        /// consumes no reassociation.
        permits_reassociation: bool,
        /// Whether the contract permits contributor permutation.
        permits_permutation: bool,
    },
    /// One workgroup's invocations cooperate on each output position.
    ///
    /// The sibling of [`Self::MultiPass`], and the difference between them is
    /// the whole reason this is a topology of its own. `MultiPass` splits a
    /// contributor sequence across a *dispatch* boundary: it commits its
    /// partials to a materialized tensor, and the dispatch dependency the kernel
    /// program declares is what makes them visible, so it needs no
    /// intra-workgroup ordering. This variant splits the same sequence across
    /// the invocations of *one* workgroup and stages the partials in
    /// workgroup-shared memory, so the handoff needs visibility that no dispatch
    /// boundary supplies — which is exactly what
    /// [`CooperativeTile::visibility_edges`] states and what no schedule can yet
    /// authorize.
    ///
    /// The split is a reassociation of the declared contributor sequence and
    /// never a permutation of it, for the reason [`ContributorPartition`]
    /// records: participant `p` combines the contiguous contributor range that
    /// partition owns, and the staged partials are combined in ascending `p`.
    CooperativeWorkgroup {
        /// How one output's contributor sequence is split across participants.
        partition: ContributorPartition,
        /// The cross-invocation dataflow that split requires.
        tile: CooperativeTile,
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Width every combining step is performed at.
        accumulation: ArithmeticType,
        /// Whether the contract permits reassociation.
        ///
        /// The permission this strategy consumes; the verifier admits the
        /// topology only when it holds.
        permits_reassociation: bool,
        /// Whether the contract permits contributor permutation.
        ///
        /// Recorded because the region preserves the declared realization
        /// whole, and deliberately not consulted to admit the split.
        permits_permutation: bool,
    },
}

/// Which pass of a multi-pass reduction one region realizes.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a third pass role lands additively. Total maps
/// *inside* `tiler-ir` are unaffected, because the attribute has no effect
/// within the defining crate, which is what keeps them breaking.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ReductionPass {
    /// Combines one partition's contributors into one partial value.
    ///
    /// The region writes a materialized partial tensor a later pass consumes;
    /// it never writes the reduction's own output.
    Partial,
    /// Combines every partial value of one output position into the result.
    ///
    /// Its contributors are the partial values, not the original ones, so its
    /// contributor count is the partition count.
    Final,
}

impl ReductionPass {
    /// Returns the canonical tag naming this pass in an identity encoding.
    ///
    /// Written by an exhaustive match rather than read from the discriminant,
    /// so adding or reordering a variant is a build error here instead of a
    /// silent change to every identity ever produced (ADR 0074 convention 3).
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Partial => 0x01,
            Self::Final => 0x02,
        }
    }
}

/// How a multi-pass reduction splits one output's contributor sequence.
///
/// The split is a *physical* contract, not a semantic one: it says how many
/// partial values one output position is built from and how many contributors
/// each of them combines, so the total the two passes cover is a single
/// multiplication rather than a claim a reader has to reconstruct. Requiring
/// the product to be exact is what makes "every contributor exactly once"
/// checkable — a ragged final partition would need a second extent and a
/// second trip count, and [`ContributorPartition::covers`] rejects one rather
/// than approximating it.
///
/// Contributor order is preserved: partition `p` covers the contiguous
/// contributor range `p * contributors_per_partition ..
/// (p + 1) * contributors_per_partition` of the [`ContributorOrder`] the region
/// declares, and the final pass combines the partials in ascending `p`. A
/// multi-pass split is therefore a *reassociation* of the declared contributor
/// sequence and never a permutation of it — the two permissions stay
/// independent, and this strategy consumes only the first.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ContributorPartition {
    /// Partial values produced per output position.
    pub partitions: u64,
    /// Contributors every partition combines.
    pub contributors_per_partition: u64,
}

impl ContributorPartition {
    /// Returns the contributors this split covers, or `None` when it overflows.
    #[must_use]
    pub const fn total_contributors(self) -> Option<u64> {
        self.partitions.checked_mul(self.contributors_per_partition)
    }

    /// Returns whether this split covers `contributors` exactly once each.
    ///
    /// A split with no partitions covers nothing and is rejected even for an
    /// empty reduction: the final pass would then have no contributor to
    /// combine and no place to read the empty identity from.
    #[must_use]
    pub const fn covers(self, contributors: u64) -> bool {
        if self.partitions == 0 {
            return false;
        }
        match self.total_contributors() {
            Some(total) => total == contributors,
            None => false,
        }
    }
}

/// The symbolic launch geometry a schedule dispatches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaunchPlan {
    /// Total launched grid threads.
    pub grid_threads: u64,
    /// Threads per workgroup.
    pub threads_per_workgroup: u32,
    /// Whether a zero-work domain skips dispatch.
    pub zero_work_skips_dispatch: bool,
}

/// The normalized schedule that maps a region onto a target machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelSchedule {
    /// Execution-to-iteration coordinate binding.
    pub binding: ExecutionBinding,
    /// Iteration work items covered by the launch.
    pub work_items: u64,
    /// Threads per workgroup.
    pub threads_per_workgroup: u32,
    /// Tail policy.
    pub tail: TailPolicy,
    /// Ownership witness the owning write must reference.
    pub output_owner: OwnershipWitnessId,
    /// Reduction topology.
    pub reduction: ReductionTopology,
    /// Launch geometry.
    pub launch: LaunchPlan,
}

/// A first-class scheduled region: a bounded index region plus its schedule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledRegion {
    /// The bounded index region the schedule refines.
    pub index: IndexRegion,
    /// The normalized kernel schedule.
    pub schedule: KernelSchedule,
}

/// Exact or proven resource requirements derived from a verified schedule.
///
/// These feed a separate phased target-feasibility assessment; deriving them is
/// part of intrinsic verification and never a target decision (ADR 0007).
///
/// The four numerical fields carry the region's declared realization forward
/// per dimension rather than as one summary bit. A single `requires_strict_f32`
/// boolean cannot name which dimension a target failed to honour, and the
/// boolean these replaced was derived from contraction and reassociation alone
/// — so a subnormal-preserving contract that permitted both transforms reported
/// no strict-`f32` requirement at all (ADR 0076 item 3). A feasibility
/// authority composes each dimension against what a target profile declares it
/// honours.
///
/// The realization's `profile_key` and canonical NaN bits are deliberately not
/// repeated here: they name the governing contract and a produced value rather
/// than a behaviour a target profile declares honourability for, and they
/// remain on the region's [`NumericalRealization`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceRequirements {
    /// Distinct buffer bindings required at the entry point.
    pub buffer_bindings: u32,
    /// Threads per workgroup required by the launch.
    pub threads_per_workgroup: u32,
    /// Local (threadgroup) memory bytes required.
    pub local_memory_bytes: u64,
    /// Whether the region requires a device address space.
    pub requires_device_memory: bool,
    /// Subnormal input handling the region's declared realization requires.
    pub input_subnormals: SubnormalMode,
    /// Subnormal result handling the region's declared realization requires.
    pub result_subnormals: SubnormalMode,
    /// Whether the region's declared realization permits contraction.
    pub contraction: NumericalPermission,
    /// Whether the region's declared realization permits reassociation.
    pub reassociation: NumericalPermission,
    /// Whether the region's declared realization permits contributor permutation.
    pub permutation: NumericalPermission,
    /// Whether the region's declared realization permits signed-zero elimination.
    pub signed_zero: NumericalPermission,
    /// The region's declared NaN-absence assumption.
    pub nan_assumptions: ExceptionalValueAssumption,
    /// The region's declared infinity-absence assumption.
    pub infinity_assumptions: ExceptionalValueAssumption,
}

/// Opaque canonical bytes identifying one verified scheduled region.
///
/// The identity is a pure function of the normalized schedule content and is
/// independent of the transient [`RegionId`] and of builder insertion order for
/// equivalent regions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalScheduledRegionIdentity(Vec<u8>);

impl CanonicalScheduledRegionIdentity {
    /// Returns the canonical identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An immutable, intrinsically verified scheduled region.
///
/// Only [`super::ScheduledRegionBuilder::build`] produces one. It exposes
/// read-only meaning and never mutation, thawing, or unchecked construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedScheduledRegion {
    region: ScheduledRegion,
    requirements: ResourceRequirements,
    identity: CanonicalScheduledRegionIdentity,
}

impl VerifiedScheduledRegion {
    pub(super) fn new(
        region: ScheduledRegion,
        requirements: ResourceRequirements,
        identity: CanonicalScheduledRegionIdentity,
    ) -> Self {
        Self {
            region,
            requirements,
            identity,
        }
    }

    /// Returns the normalized scheduled region.
    #[must_use]
    pub const fn region(&self) -> &ScheduledRegion {
        &self.region
    }

    /// Returns the derived resource requirements.
    #[must_use]
    pub const fn requirements(&self) -> ResourceRequirements {
        self.requirements
    }

    /// Returns the canonical structural identity.
    #[must_use]
    pub const fn canonical_identity(&self) -> &CanonicalScheduledRegionIdentity {
        &self.identity
    }

    /// Returns whether this region's arithmetic is bounded away from subnormals.
    ///
    /// Derived from the *verified* scalar program, which is why this is a method
    /// on the verified product rather than a field a producer supplies.
    /// `verify_strict_affine_u4_dequantize` already proved, before this region
    /// existed, that the decode arm names exactly the three governed
    /// strict-affine component roles of one input tensor and preserves the
    /// strict realization; the governed value contract those roles belong to
    /// declares its scale `positive-normal-f32`. The freedom follows from those
    /// two checked facts, so nothing here has to be trusted separately.
    ///
    /// Every other scalar program reads dense tensor payloads whose value
    /// domain nothing bounds, and the match is exhaustive so a new program is a
    /// build error here rather than an unproven freedom silently inheriting an
    /// answer.
    #[must_use]
    pub const fn subnormal_freedom(&self) -> SubnormalFreedom {
        subnormal_freedom_of(&self.region.index.scalar_program)
    }
}

/// Classifies one scalar program's subnormal freedom.
///
/// The single definition both [`VerifiedScheduledRegion::subnormal_freedom`]
/// and the structured-kernel lowering read, so a kernel's answer cannot drift
/// from the region's.
pub(crate) const fn subnormal_freedom_of(program: &ScalarProgram) -> SubnormalFreedom {
    match program {
        ScalarProgram::StrictAffineU4Dequantize { .. } => {
            SubnormalFreedom::StrictAffineNormalScaleDecode
        }
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. }
        | ScalarProgram::StrictTensorContraction { .. } => SubnormalFreedom::Unproven,
    }
}

/// Returns the element count of a shape, or `0` when any extent is `0`.
///
/// # Errors
///
/// Returns [`ElementCountOverflow`] when a nonzero extent product exceeds `u64`.
pub fn element_count(shape: &Shape) -> Result<u64, ElementCountOverflow> {
    if shape.extents().iter().any(|extent| extent.get() == 0) {
        return Ok(0);
    }
    shape
        .extents()
        .iter()
        .try_fold(1_u64, |count, extent| count.checked_mul(extent.get()))
        .ok_or(ElementCountOverflow)
}

/// Returns the physical shape of the partial tensor a split reduction stages.
///
/// The partition axis is appended rather than inserted, so the partial tensor
/// is the reduction's own output shape with one partial value per partition in
/// row-major order. That makes the partial pass's linear write index and its
/// global invocation index the same number, and makes the final pass an
/// ordinary reduction of the trailing axis — the reason this layout is fixed
/// here rather than chosen per producer.
///
/// Returns `None` when appending the axis would exceed the governed rank bound.
#[must_use]
pub fn partial_reduction_shape(
    output_shape: &Shape,
    partition: ContributorPartition,
) -> Option<Shape> {
    Shape::try_new(
        output_shape
            .extents()
            .iter()
            .copied()
            .chain(std::iter::once(crate::shape::Extent::new(
                partition.partitions,
            ))),
    )
    .ok()
}

/// Returns the axis of a partial tensor the final pass of a split reduces.
#[must_use]
pub fn partial_reduction_axis(output_shape: &Shape) -> Option<Axis> {
    u32::try_from(output_shape.rank()).ok().map(Axis::new)
}

/// Returns the cooperative tile one reduction topology carries, if any.
///
/// The one place the topology is matched to reach its tile, so a consumer that
/// needs the dataflow cannot accidentally read it from some other variant.
#[must_use]
pub fn cooperative_tile(reduction: &ReductionTopology) -> Option<&CooperativeTile> {
    match reduction {
        ReductionTopology::CooperativeWorkgroup { tile, .. } => Some(tile),
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::MultiPass { .. }
        | ReductionTopology::Contraction { .. } => None,
    }
}

/// Returns the workgroup memory one reduction topology requires, in bytes.
///
/// `None` distinguishes "no tile, so nothing staged" from "a tile whose
/// allocations overflow `u64`" only at the call site that cares: the intrinsic
/// verifier refuses the overflow, and every later reader sees a value it already
/// proved exact.
#[must_use]
pub fn cooperative_local_memory_bytes(reduction: &ReductionTopology) -> Option<u64> {
    cooperative_tile(reduction).and_then(CooperativeTile::local_memory_bytes)
}

/// Returns whether `axes` is a strictly ascending in-range axis set.
#[must_use]
pub fn axes_are_canonical(axes: &[Axis], rank: usize) -> bool {
    let mut previous = None;
    axes.iter().all(|axis| {
        let Ok(index) = usize::try_from(axis.get()) else {
            return false;
        };
        let canonical = index < rank && previous.is_none_or(|previous| previous < axis.get());
        previous = Some(axis.get());
        canonical
    })
}

/// Counts the reduction contributors a reduction-contributor access combines.
///
/// Returns `0` when any reduced extent is `0` (an empty reduction).
///
/// # Errors
///
/// Returns a [`ContributorError`] when the access is not a reduction access,
/// the axes are not canonical, an axis is out of range, or the contributor
/// product overflows `u64`.
pub fn contributor_count(axes: &[Axis], access: &LogicalAccess) -> Result<u64, ContributorError> {
    let LogicalAccess::ReductionContributor { input_shape, .. } = access else {
        return Err(ContributorError::NotReductionAccess);
    };
    if !axes_are_canonical(axes, input_shape.rank()) {
        return Err(ContributorError::NonCanonicalAxes);
    }
    let extents = axes
        .iter()
        .map(|axis| {
            usize::try_from(axis.get())
                .ok()
                .and_then(|index| input_shape.extents().get(index))
                .map(|extent| extent.get())
                .ok_or(ContributorError::AxisOutOfRange)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if extents.contains(&0) {
        return Ok(0);
    }
    extents
        .into_iter()
        .try_fold(1_u64, u64::checked_mul)
        .ok_or(ContributorError::Overflow)
}

/// Derives the resource requirements of a verified region.
///
/// Bindings follow the region's access count; the launch fixes the thread
/// count; local memory is the workgroup storage a cooperative tile allocates and
/// zero for every topology that stages nothing. No synchronization requirement
/// is derived, because none is authorized: a cooperative tile records the
/// visibility dependency its staging creates without asserting that anything
/// discharges it. The numerical
/// realization is carried forward whole rather than reduced to a predicate:
/// deriving one bit here would decide, inside intrinsic verification, which
/// dimensions a target is allowed to be asked about, and that decision belongs
/// to the feasibility authority that knows what the target declares.
///
/// The local-memory value is read through [`cooperative_local_memory_bytes`],
/// whose overflow case the intrinsic verifier has already refused — so this
/// never saturates a requirement a feasibility authority would then compose as
/// if it were real.
pub(super) fn derive_requirements(region: &ScheduledRegion) -> ResourceRequirements {
    let buffer_bindings = u32::try_from(region.index.accesses.len()).unwrap_or(u32::MAX);
    ResourceRequirements {
        buffer_bindings,
        threads_per_workgroup: region.schedule.threads_per_workgroup,
        local_memory_bytes: cooperative_local_memory_bytes(&region.schedule.reduction).unwrap_or(0),
        requires_device_memory: true,
        input_subnormals: region.index.numerical.input_subnormals,
        result_subnormals: region.index.numerical.result_subnormals,
        contraction: region.index.numerical.contraction,
        reassociation: region.index.numerical.reassociation,
        permutation: region.index.numerical.permutation,
        signed_zero: region.index.numerical.signed_zero,
        nan_assumptions: region.index.numerical.nan_assumptions,
        infinity_assumptions: region.index.numerical.infinity_assumptions,
    }
}

const TAG_LINEAR_IDENTITY: u8 = 0x01;
const TAG_REDUCTION_CONTRIBUTOR: u8 = 0x02;
const TAG_SCALAR_BROADCAST: u8 = 0x03;
const TAG_PACKED_U4_LSB_ZERO_TAIL: u8 = 0x04;
/// Logical-access tag of one contraction operand's coordinate map.
///
/// Appended rather than inserted, for the reason `TAG_SCALAR_SQUARED_SUM`
/// records: `0x01` through `0x04` keep their tags and their field layouts, so no
/// previously encodable region's bytes move and the schedule identity domain
/// deliberately does not step. A reader that reaches `0x05` is reading an access
/// the earlier vocabulary could not express.
const TAG_CONTRACTION_OPERAND: u8 = 0x05;
const TAG_LINEAR_RANGE: u8 = 0x11;
const TAG_REDUCTION_DOMAIN: u8 = 0x12;
const TAG_SCALAR_SERIAL_SUM: u8 = 0x22;
const TAG_SCALAR_FUSED_SUM: u8 = 0x23;
const TAG_SCALAR_POINTWISE_F32: u8 = 0x24;
const TAG_SCALAR_STRICT_AFFINE_U4_DEQUANTIZE: u8 = 0x25;
/// Scalar-program tag of the squaring-prologue serial sum.
///
/// Appended rather than inserted, and the schedule domain deliberately did not
/// step with it: every earlier scalar program keeps its tag and its field layout,
/// so a reader that reaches `0x26` is reading a region the earlier vocabulary
/// could not express, never an earlier region under a new interpretation.
const TAG_SCALAR_SQUARED_SUM: u8 = 0x26;
/// Scalar-program tag of the strict tensor contraction.
///
/// Appended for the same reason and with the same consequence as `0x26`: `0x22`
/// through `0x26` keep their meanings and their field positions, so the schedule
/// identity domain does not step.
const TAG_SCALAR_TENSOR_CONTRACTION: u8 = 0x27;
const TAG_REDUCTION_NONE: u8 = 0x31;
const TAG_REDUCTION_SERIAL: u8 = 0x32;
/// Reduction-topology tag of a split, multi-dispatch reduction pass.
///
/// Appended rather than inserted, and the schedule domain
/// deliberately did not step with it: no previously encodable region's bytes
/// move, because `None` and `Serial` keep their tags and every other field
/// keeps its position. Injectivity is what a domain separator protects, and a
/// fresh tag byte preserves it — a reader that reaches `0x33` is reading a
/// region the earlier vocabulary could not express, never an earlier region
/// under a new interpretation.
const TAG_REDUCTION_MULTI_PASS: u8 = 0x33;
/// Reduction-topology tag of a contraction's fold over its contracted space.
///
/// Appended exactly as `0x33` was, and with the same injectivity argument:
/// `None`, `Serial`, and `MultiPass` keep their tags and their field positions.
const TAG_REDUCTION_CONTRACTION: u8 = 0x34;
/// Reduction-topology tag of a cooperative workgroup tile.
///
/// Appended exactly as `0x33` and `0x34` were, and with the same injectivity
/// argument: `None`, `Serial`, `MultiPass`, and `Contraction` keep their tags
/// and their field positions, so no previously encodable region's bytes move and
/// the schedule identity domain deliberately does not step. A reader that
/// reaches `0x35` is reading a region the earlier vocabulary could not express.
const TAG_REDUCTION_COOPERATIVE_WORKGROUP: u8 = 0x35;

fn push_shape(bytes: &mut Vec<u8>, shape: &Shape) {
    push_len(bytes, shape.rank());
    for extent in shape.extents() {
        bytes.extend_from_slice(&extent.get().to_be_bytes());
    }
}

fn push_axes(bytes: &mut Vec<u8>, axes: &[Axis]) {
    push_len(bytes, axes.len());
    for axis in axes {
        bytes.extend_from_slice(&axis.get().to_be_bytes());
    }
}

fn push_order(bytes: &mut Vec<u8>, order: ContributorOrder) {
    let ContributorOrder::OriginalAxisLexicographic = order;
    bytes.push(0x01);
}

/// Encodes one boundary tensor role.
///
/// The input ordinal follows its tag rather than being folded into it, so the
/// role tags stay a three-value table and the ordinal keeps its own fixed width.
/// Two reads of two different input tensors therefore differ in these bytes,
/// which is the whole point: a region reading `a * b` and one reading `a * a`
/// are different computations and must not share identity.
fn push_tensor_role(bytes: &mut Vec<u8>, role: TensorRole) {
    match role {
        TensorRole::Input { ordinal } => {
            bytes.push(0x01);
            bytes.extend_from_slice(&ordinal.get().to_be_bytes());
        }
        TensorRole::Intermediate => bytes.push(0x02),
        TensorRole::Output => bytes.push(0x03),
    }
}

fn push_logical_access(bytes: &mut Vec<u8>, access: &LogicalAccess) {
    match access {
        LogicalAccess::LinearIdentity => bytes.push(TAG_LINEAR_IDENTITY),
        LogicalAccess::ScalarBroadcast => bytes.push(TAG_SCALAR_BROADCAST),
        LogicalAccess::PackedU4LsbZeroTail { logical_elements } => {
            bytes.push(TAG_PACKED_U4_LSB_ZERO_TAIL);
            bytes.extend_from_slice(&logical_elements.to_be_bytes());
        }
        LogicalAccess::ReductionContributor {
            input_shape,
            output_shape,
            axes,
            order,
        } => {
            bytes.push(TAG_REDUCTION_CONTRIBUTOR);
            push_shape(bytes, input_shape);
            push_shape(bytes, output_shape);
            push_axes(bytes, axes);
            push_order(bytes, *order);
        }
        LogicalAccess::ContractionOperand {
            operand_shape,
            output_shape,
            contracted_shape,
            sources,
            order,
        } => {
            bytes.push(TAG_CONTRACTION_OPERAND);
            push_shape(bytes, operand_shape);
            push_shape(bytes, output_shape);
            push_shape(bytes, contracted_shape);
            push_len(bytes, sources.len());
            for source in sources {
                push_contraction_axis_source(bytes, *source);
            }
            push_order(bytes, *order);
        }
    }
}

/// Encodes one contraction operand axis's coordinate source.
///
/// The position follows its tag rather than being folded into it, so the two
/// spaces keep a two-value tag table and the position keeps its own fixed width.
/// An operand reading output position 0 and one reading contracted position 0
/// are different access relations and must not share identity.
fn push_contraction_axis_source(bytes: &mut Vec<u8>, source: ContractionAxisSource) {
    match source {
        ContractionAxisSource::Output { position } => {
            bytes.push(0x01);
            bytes.extend_from_slice(&position.to_be_bytes());
        }
        ContractionAxisSource::Contracted { position } => {
            bytes.push(0x02);
            bytes.extend_from_slice(&position.to_be_bytes());
        }
    }
}

/// Encodes one subnormal dimension.
///
/// The match is exhaustive over a non-`#[non_exhaustive]` enum, so widening the
/// vocabulary is a build error here rather than an identity collision between
/// two regions that differ only in subnormal treatment (ADR 0076 item 6). The
/// flush arm encodes its zero sign, because the sign is part of the behaviour
/// and two flushes producing different zeros are different realizations.
fn push_subnormal(bytes: &mut Vec<u8>, mode: SubnormalMode) {
    bytes.push(match mode {
        SubnormalMode::Preserve => 0x01,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        } => 0x02,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        } => 0x03,
    });
}

/// Encodes one transform permission.
///
/// Encoded as a tagged value rather than as the derived `permits_*` boolean it
/// used to be: a boolean is a projection, and a projection cannot fail closed
/// when the projected enum grows.
fn push_permission(bytes: &mut Vec<u8>, permission: NumericalPermission) {
    bytes.push(match permission {
        NumericalPermission::Forbidden => 0x01,
        NumericalPermission::Permitted => 0x02,
    });
}

/// Encodes one exceptional-value assumption and its evidence class.
fn push_exceptional_assumption(bytes: &mut Vec<u8>, assumption: ExceptionalValueAssumption) {
    match assumption {
        ExceptionalValueAssumption::MakeNoAssumption => bytes.push(0x01),
        ExceptionalValueAssumption::AssumeAbsent { provenance } => {
            bytes.push(0x02);
            bytes.push(match provenance {
                ValueDomainProvenance::CompilerProven => 0x01,
                ValueDomainProvenance::RuntimeValidated => 0x02,
                ValueDomainProvenance::CallerDeclaredUnvalidated => 0x03,
            });
        }
    }
}

/// Encodes the complete numerical realization a region declares.
///
/// Every field is encoded, including both subnormal dimensions. `profile_key`
/// is encoded alongside them and never in place of them: a key names a contract
/// but does not carry its field values, so relying on the key to distinguish
/// two realizations would be an unstated invariant (ADR 0076 item 6).
///
/// The key is length-prefixed through [`push_slice`], the one framing form the
/// workspace uses before a variable-length run (ADR 0074 convention 3). It was
/// NUL-terminated here alone. That was unambiguous while the key is a
/// crate-chosen `&'static str` containing no NUL, but the uniform form is what
/// removes the need to re-derive that argument at each site.
fn push_numerical(bytes: &mut Vec<u8>, numerical: &NumericalRealization) {
    push_slice(bytes, numerical.profile_key.as_bytes());
    bytes.extend_from_slice(&numerical.canonical_arithmetic_nan_bits.to_be_bytes());
    push_subnormal(bytes, numerical.input_subnormals);
    push_subnormal(bytes, numerical.result_subnormals);
    push_permission(bytes, numerical.contraction);
    push_permission(bytes, numerical.reassociation);
    push_permission(bytes, numerical.permutation);
    push_permission(bytes, numerical.signed_zero);
    push_exceptional_assumption(bytes, numerical.nan_assumptions);
    push_exceptional_assumption(bytes, numerical.infinity_assumptions);
}

fn push_scalar_program(bytes: &mut Vec<u8>, program: &ScalarProgram) {
    match program {
        ScalarProgram::PointwiseF32(expression) => {
            bytes.push(TAG_SCALAR_POINTWISE_F32);
            push_len(bytes, expression.nodes().len());
            for node in expression.nodes() {
                push_pointwise_f32_node(bytes, node);
            }
            push_slice(bytes, &expression.root().index().to_be_bytes());
        }
        ScalarProgram::StrictAffineU4Dequantize {
            codes_role,
            scale_role,
            zero_point_role,
        } => {
            bytes.push(TAG_SCALAR_STRICT_AFFINE_U4_DEQUANTIZE);
            bytes.extend_from_slice(&codes_role.get().to_be_bytes());
            bytes.extend_from_slice(&scale_role.get().to_be_bytes());
            bytes.extend_from_slice(&zero_point_role.get().to_be_bytes());
        }
        ScalarProgram::StrictSerialSum {
            axes,
            order,
            canonical_nan_bits,
            empty_identity_bits,
        } => {
            bytes.push(TAG_SCALAR_SERIAL_SUM);
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.extend_from_slice(&canonical_nan_bits.to_be_bytes());
            bytes.extend_from_slice(&empty_identity_bits.to_be_bytes());
        }
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits,
            axes,
            order,
            canonical_nan_bits,
            empty_identity_bits,
            contraction,
        } => {
            bytes.push(TAG_SCALAR_FUSED_SUM);
            bytes.extend_from_slice(&scale_bits.to_be_bytes());
            bytes.extend_from_slice(&bias_bits.to_be_bytes());
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.extend_from_slice(&canonical_nan_bits.to_be_bytes());
            bytes.extend_from_slice(&empty_identity_bits.to_be_bytes());
            bytes.push(u8::from(*contraction));
        }
        // Appended tag, like `TAG_REDUCTION_MULTI_PASS`: `0x22` through `0x25`
        // keep their meanings and every field keeps its position, so no
        // previously encodable region's bytes move and the schedule identity
        // domain does not step.
        ScalarProgram::SquaredSerialSum {
            axes,
            order,
            canonical_nan_bits,
            empty_identity_bits,
        } => {
            bytes.push(TAG_SCALAR_SQUARED_SUM);
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.extend_from_slice(&canonical_nan_bits.to_be_bytes());
            bytes.extend_from_slice(&empty_identity_bits.to_be_bytes());
        }
        // Appended for the same reason as `0x26`, and with no empty-domain
        // identity to encode: the family refuses an empty contracted domain
        // rather than committing a value there.
        ScalarProgram::StrictTensorContraction {
            contracted_shape,
            order,
            canonical_nan_bits,
        } => {
            bytes.push(TAG_SCALAR_TENSOR_CONTRACTION);
            push_shape(bytes, contracted_shape);
            push_order(bytes, *order);
            bytes.extend_from_slice(&canonical_nan_bits.to_be_bytes());
        }
    }
}

fn push_pointwise_f32_node(bytes: &mut Vec<u8>, node: &PointwiseF32Node) {
    const LENGTH_BYTES: usize = size_of::<u64>();
    const TAG_FIELD_BYTES: usize = LENGTH_BYTES + size_of::<u8>();
    const U32_FIELD_BYTES: usize = LENGTH_BYTES + size_of::<u32>();

    let encoded_len = match node {
        // A tag plus one `u32` covers every node with a single field, whatever
        // that field means: an input ordinal, a constant payload, or one
        // elementary function's argument. Grouped because the *encoded width* is
        // the only thing this function decides; the tags that distinguish them
        // are pushed below.
        PointwiseF32Node::Input { .. }
        | PointwiseF32Node::Constant { .. }
        | PointwiseF32Node::Exp { .. }
        | PointwiseF32Node::Rsqrt { .. } => TAG_FIELD_BYTES + U32_FIELD_BYTES,
        PointwiseF32Node::Add { .. }
        | PointwiseF32Node::Multiply { .. }
        | PointwiseF32Node::Divide { .. } => TAG_FIELD_BYTES + 2 * U32_FIELD_BYTES,
    };
    push_len(bytes, encoded_len);
    let start = bytes.len();
    match node {
        PointwiseF32Node::Input { ordinal } => {
            push_slice(bytes, &[0x01]);
            push_slice(bytes, &ordinal.get().to_be_bytes());
        }
        PointwiseF32Node::Constant { bits } => {
            push_slice(bytes, &[0x02]);
            push_slice(bytes, &bits.to_be_bytes());
        }
        PointwiseF32Node::Add { lhs, rhs } => {
            push_slice(bytes, &[0x03]);
            push_slice(bytes, &lhs.index().to_be_bytes());
            push_slice(bytes, &rhs.index().to_be_bytes());
        }
        PointwiseF32Node::Multiply { lhs, rhs } => {
            push_slice(bytes, &[0x04]);
            push_slice(bytes, &lhs.index().to_be_bytes());
            push_slice(bytes, &rhs.index().to_be_bytes());
        }
        // Appended tags, like `TAG_REDUCTION_MULTI_PASS`: every earlier node keeps
        // its tag and its field layout, so no previously encodable region's bytes
        // move and the schedule domain deliberately does not step. A reader that
        // reaches `0x05` or `0x06` is reading a region the earlier vocabulary could
        // not express, never an earlier region reinterpreted.
        PointwiseF32Node::Divide { lhs, rhs } => {
            push_slice(bytes, &[0x05]);
            push_slice(bytes, &lhs.index().to_be_bytes());
            push_slice(bytes, &rhs.index().to_be_bytes());
        }
        PointwiseF32Node::Exp { argument } => {
            push_slice(bytes, &[0x06]);
            push_slice(bytes, &argument.index().to_be_bytes());
        }
        // Appended for the same reason and with the same consequence: `0x07` is
        // a node the earlier vocabulary could not express.
        PointwiseF32Node::Rsqrt { argument } => {
            push_slice(bytes, &[0x07]);
            push_slice(bytes, &argument.index().to_be_bytes());
        }
    }
    debug_assert_eq!(bytes.len() - start, encoded_len);
}

fn push_access(bytes: &mut Vec<u8>, access: &Access) {
    push_tensor_role(bytes, access.tensor);
    push_component_role(bytes, access.component_role);
    bytes.push(match access.mode {
        AccessMode::Read => 0x01,
        AccessMode::Write => 0x02,
    });
    push_logical_access(bytes, &access.map);
    bytes.extend_from_slice(&access.bounds.get().to_be_bytes());
    match access.ownership {
        None => bytes.push(0x00),
        Some(owner) => {
            bytes.push(0x01);
            bytes.extend_from_slice(&owner.get().to_be_bytes());
        }
    }
}

fn push_bounds_proof(bytes: &mut Vec<u8>, proof: &BoundsProof) {
    bytes.extend_from_slice(&proof.id.get().to_be_bytes());
    push_tensor_role(bytes, proof.tensor);
    push_component_role(bytes, proof.component_role);
    match &proof.kind {
        BoundsProofKind::LinearRange { element_count } => {
            bytes.push(TAG_LINEAR_RANGE);
            bytes.extend_from_slice(&element_count.to_be_bytes());
        }
        BoundsProofKind::ReductionDomain {
            input_shape,
            output_shape,
            axes,
            order,
        } => {
            bytes.push(TAG_REDUCTION_DOMAIN);
            push_shape(bytes, input_shape);
            push_shape(bytes, output_shape);
            push_axes(bytes, axes);
            push_order(bytes, *order);
        }
    }
}

fn push_component_role(bytes: &mut Vec<u8>, role: Option<EncodedComponentRole>) {
    match role {
        None => bytes.push(0x00),
        Some(role) => {
            bytes.push(0x01);
            bytes.extend_from_slice(&role.get().to_be_bytes());
        }
    }
}

/// Encodes one contiguous run of local invocation coordinates.
fn push_participant_range(bytes: &mut Vec<u8>, range: ParticipantRange) {
    bytes.extend_from_slice(&range.first.to_be_bytes());
    bytes.extend_from_slice(&range.count.to_be_bytes());
}

/// Encodes the slots one participant addresses in one phase.
fn push_staged_span(bytes: &mut Vec<u8>, span: StagedSpan) {
    bytes.extend_from_slice(&span.stride.to_be_bytes());
    bytes.extend_from_slice(&span.offset.to_be_bytes());
    bytes.extend_from_slice(&span.count.to_be_bytes());
}

/// Encodes one workgroup staging allocation and its declared lifetime.
fn push_workgroup_staging(bytes: &mut Vec<u8>, staging: &WorkgroupStaging) {
    bytes.extend_from_slice(&staging.id.get().to_be_bytes());
    bytes.push(staging.element.tag());
    bytes.extend_from_slice(&staging.slots.to_be_bytes());
    bytes.extend_from_slice(&staging.live_from.get().to_be_bytes());
    bytes.extend_from_slice(&staging.live_through.get().to_be_bytes());
}

/// Encodes one phase's reachable participants and its staged accesses.
///
/// Writes and reads are framed separately and in declaration order, so a tile
/// that writes an allocation this phase and one that reads it here differ in
/// these bytes even when the spans coincide.
fn push_cooperative_phase(bytes: &mut Vec<u8>, phase: &CooperativePhase) {
    bytes.extend_from_slice(&phase.id.get().to_be_bytes());
    push_participant_range(bytes, phase.participation);
    push_len(bytes, phase.writes.len());
    for StagedWrite { staging, span } in &phase.writes {
        bytes.extend_from_slice(&staging.get().to_be_bytes());
        push_staged_span(bytes, *span);
    }
    push_len(bytes, phase.reads.len());
    for StagedRead { staging, span } in &phase.reads {
        bytes.extend_from_slice(&staging.get().to_be_bytes());
        push_staged_span(bytes, *span);
    }
}

/// Encodes one cooperative tile's complete cross-invocation dataflow.
///
/// The visibility edges are deliberately *not* encoded: they are a total
/// function of the phases and staged accesses already written here, so encoding
/// them would add bytes no two distinguishable tiles differ in — and would give
/// a producer a second place to state a fact the verifier derives.
fn push_cooperative_tile(bytes: &mut Vec<u8>, tile: &CooperativeTile) {
    let LocalCoordinates {
        source,
        participants,
    } = tile.coordinates;
    bytes.push(source.tag());
    push_participant_range(bytes, participants);
    push_len(bytes, tile.staging.len());
    for staging in &tile.staging {
        push_workgroup_staging(bytes, staging);
    }
    push_len(bytes, tile.phases.len());
    for phase in &tile.phases {
        push_cooperative_phase(bytes, phase);
    }
    push_participant_range(bytes, tile.commit);
}

fn push_schedule(bytes: &mut Vec<u8>, schedule: &KernelSchedule) {
    let ExecutionBinding::GlobalLinearInvocation = schedule.binding;
    bytes.push(0x01);
    bytes.extend_from_slice(&schedule.work_items.to_be_bytes());
    bytes.extend_from_slice(&schedule.threads_per_workgroup.to_be_bytes());
    let TailPolicy::Exact = schedule.tail;
    bytes.push(0x01);
    bytes.extend_from_slice(&schedule.output_owner.get().to_be_bytes());
    match &schedule.reduction {
        ReductionTopology::None => bytes.push(TAG_REDUCTION_NONE),
        ReductionTopology::Serial {
            axes,
            order,
            permits_reassociation,
            permits_permutation,
        } => {
            bytes.push(TAG_REDUCTION_SERIAL);
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.push(u8::from(*permits_reassociation));
            bytes.push(u8::from(*permits_permutation));
        }
        ReductionTopology::MultiPass {
            pass,
            partition,
            axes,
            order,
            accumulation,
            permits_reassociation,
            permits_permutation,
        } => {
            bytes.push(TAG_REDUCTION_MULTI_PASS);
            bytes.push(pass.tag());
            bytes.extend_from_slice(&partition.partitions.to_be_bytes());
            bytes.extend_from_slice(&partition.contributors_per_partition.to_be_bytes());
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.push(accumulation.tag());
            bytes.push(u8::from(*permits_reassociation));
            bytes.push(u8::from(*permits_permutation));
        }
        ReductionTopology::Contraction {
            contracted_shape,
            order,
            permits_reassociation,
            permits_permutation,
        } => {
            bytes.push(TAG_REDUCTION_CONTRACTION);
            push_shape(bytes, contracted_shape);
            push_order(bytes, *order);
            bytes.push(u8::from(*permits_reassociation));
            bytes.push(u8::from(*permits_permutation));
        }
        ReductionTopology::CooperativeWorkgroup {
            partition,
            tile,
            axes,
            order,
            accumulation,
            permits_reassociation,
            permits_permutation,
        } => {
            bytes.push(TAG_REDUCTION_COOPERATIVE_WORKGROUP);
            bytes.extend_from_slice(&partition.partitions.to_be_bytes());
            bytes.extend_from_slice(&partition.contributors_per_partition.to_be_bytes());
            push_cooperative_tile(bytes, tile);
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.push(accumulation.tag());
            bytes.push(u8::from(*permits_reassociation));
            bytes.push(u8::from(*permits_permutation));
        }
    }
    bytes.extend_from_slice(&schedule.launch.grid_threads.to_be_bytes());
    bytes.extend_from_slice(&schedule.launch.threads_per_workgroup.to_be_bytes());
    bytes.push(u8::from(schedule.launch.zero_work_skips_dispatch));
}

/// Encodes the canonical identity of a normalized scheduled region.
///
/// The encoding excludes the transient [`RegionId`] so equivalent normalized
/// schedules produced by different planning histories share identity.
///
/// The domain tag is NUL-terminated, which is the workspace's one form for a
/// versioned domain separator (ADR 0074 convention 3). This encoder was the
/// only site that omitted the terminator.
///
/// # Why this is a `v3` step
///
/// `v3` gives [`TensorRole::Input`] an ordinal and [`PointwiseF32Node::Input`]
/// the ordinal it reads, so a region can name several distinct input tensors.
/// Both fields land *inside* records that repeat — every access, every bounds
/// proof, every expression node — so a `v2` reader would consume the following
/// bytes at the old offset and lose framing, and every region ever encoded maps
/// to different bytes now. That is exactly the case the domain separator exists
/// for: a cache or artifact holding a `v2` identity must miss rather than match.
/// Contrast `TAG_REDUCTION_MULTI_PASS` above, which is an appended tag byte that
/// moves no previously encodable region's bytes and deliberately did not step
/// the domain.
pub(super) fn encode_identity(region: &ScheduledRegion) -> CanonicalScheduledRegionIdentity {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.schedule.v3\0");
    push_shape(&mut bytes, &region.index.iteration_shape);
    push_len(&mut bytes, region.index.accesses.len());
    for access in &region.index.accesses {
        push_access(&mut bytes, access);
    }
    push_len(&mut bytes, region.index.bounds_proofs.len());
    for proof in &region.index.bounds_proofs {
        push_bounds_proof(&mut bytes, proof);
    }
    bytes.extend_from_slice(&region.index.ownership_proof.id.get().to_be_bytes());
    push_tensor_role(&mut bytes, region.index.ownership_proof.tensor);
    let OwnershipProofKind::OneGlobalInvocationPerOutput { output_count } =
        region.index.ownership_proof.kind;
    bytes.extend_from_slice(&output_count.to_be_bytes());
    push_scalar_program(&mut bytes, &region.index.scalar_program);
    push_numerical(&mut bytes, &region.index.numerical);
    push_schedule(&mut bytes, &region.schedule);
    CanonicalScheduledRegionIdentity(bytes)
}
