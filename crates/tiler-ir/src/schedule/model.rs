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
    ContributorArrival, CooperativePhase, CooperativeTile, LocalCoordinates, ParticipantRange,
    ParticipantSpace, StagedRead, StagedSpan, StagedWrite, WorkgroupStaging,
};
use super::error::{ContributorError, ElementCountOverflow, VectorLaneCountError};
use super::handles::{AccessOrdinal, BoundsWitnessId, OwnershipWitnessId, RegionId};
use super::numerics::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    NumericalPermission, NumericalRealization, SubnormalFreedom, SubnormalMode,
    ValueDomainProvenance,
};
use super::pointwise::{PointwiseF32Expression, PointwiseF32Node};
use super::pointwise_bf16::{PointwiseBf16Expression, PointwiseBf16Node};
use super::subgroup::SubgroupRealizationSubject;
use super::synchronization::{
    SynchronizationPlacement, SynchronizationPoint, SynchronizationSubject, required_subject,
};

/// The role a boundary tensor plays for one scheduled region.
///
/// This role classifies an access as input, intermediate, or output. Identity
/// among several accesses with the same role belongs to their ordered position
/// or an explicit [`AccessOrdinal`], not to the category itself.
///
/// **Do not add `#[non_exhaustive]`.** This is an ADR 0074 convention 5b type
/// for the same reason [`AccessMode`] is: `push_tensor_role` in
/// `tiler-compiler`'s `selection.rs` and `frontier.rs` maps it *totally* onto
/// identity tags from outside this crate with no wildcard arm. A wildcard there would have to
/// invent a tag the variant alone determines, so a variant added later would
/// encode under some other variant's bytes instead of failing the build.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TensorRole {
    /// An input boundary consumed by the region.
    ///
    /// Association with a named program input belongs to the compiler's checked
    /// request subject and the program stage, not to this category.
    Input,
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

/// How one operand axis's coordinate is decoded from a linear iteration
/// coordinate.
///
/// The single decode form both structural access relations are written in:
/// `mirrored` selects between `c` and `modulus − 1 − c`, where
/// `c = (linear / divisor) % modulus`. One struct rather than two because the
/// *arithmetic* is the same in both relations and only the admission rule
/// differs — [`LogicalAccess::ReindexBijection`] requires the decodes to tile
/// the iteration domain exactly, and [`LogicalAccess::BroadcastReplication`]
/// requires them to name distinct result axes and leave at least one uncovered.
/// Sharing the arithmetic is what keeps a single linearization in the kernel
/// lowering; keeping the rules apart is what keeps a bijection and a replication
/// from being one concept.
///
/// **Why this covers all six registered reindex mapping forms.** Every form is a
/// single decode per operand axis, which is not obvious and is the reason the
/// vocabulary is this small rather than one variant per form:
///
/// - `PermuteAxes`, `InsertUnitAxis`, and `RemoveUnitAxis` read one result axis
///   per operand axis, so `divisor` is that axis's suffix product.
/// - `SplitAxis` replaces one operand axis by a *contiguous* run of result axes,
///   and contiguous row-major axes linearize as one window: the run's combined
///   coordinate is `(linear / suffix[last]) % operand_extent`.
/// - `MergeAxes` decodes one result coordinate into a run of operand axes. The
///   two-level decode collapses because the outer wrap is redundant — for
///   `E = result_extent[k]` divisible by `s * m`,
///   `((linear / R) % E) / s % m == (linear / (R * s)) % m`, since the discarded
///   high part is a multiple of `E / s`, itself a multiple of `m`.
/// - `ReverseAxis` is the same decode with `mirrored` set, which is exactly the
///   affine `i -> extent − 1 − i` D-10 admits and the one form that needs the
///   flag at all.
///
/// **The form is not closed under composition, and a chain is refused rather
/// than approximated.** Every bullet above is one *occurrence*; composing them
/// can produce a bijection this vocabulary cannot spell. Splitting `[4]` into
/// `[2, 2]`, permuting by `[1, 0]`, then merging back to `[4]` sends
/// `0, 1, 2, 3` to `0, 2, 1, 3`, and no single `(linear / divisor) % modulus`,
/// mirrored or not, produces that permutation over an operand axis of extent
/// four. Nothing composes such a chain today: the request boundary's
/// `recognize_structural_read` admits a structural operand only when it is a
/// declared input or the staged value, so a reindex whose operand is another
/// reindex's result refuses under `structural-operand` before any decode is
/// derived. Widening that admission needs a form that can state the composed
/// map, not a looser reading of this one.
///
/// **Do not add `#[non_exhaustive]`, and do not give it a default.** A decode is
/// three independent facts and every one of them participates in canonical
/// identity, so a field a producer could omit would be a map two regions could
/// disagree about while sharing bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AxisDecode {
    /// Divisor applied to the linear iteration coordinate before the wrap.
    pub divisor: u64,
    /// Wrap modulus, which is this operand axis's own extent.
    pub modulus: u64,
    /// Whether the decoded coordinate is mirrored to `modulus − 1 − c`.
    pub mirrored: bool,
}

impl AxisDecode {
    /// Returns the decode reading one whole axis whose suffix product is
    /// `divisor` and whose extent is `modulus`.
    #[must_use]
    pub const fn read(divisor: u64, modulus: u64) -> Self {
        Self {
            divisor,
            modulus,
            mirrored: false,
        }
    }

    /// Returns the decode of an extent-one axis, whose only coordinate is zero.
    ///
    /// Canonical by construction: an extent-one axis's coordinate does not
    /// depend on the divisor or the mirroring, so admitting any other spelling
    /// would give one access relation many identities.
    #[must_use]
    pub const fn fixed() -> Self {
        Self {
            divisor: 1,
            modulus: 1,
            mirrored: false,
        }
    }

    /// Returns whether this decode is spelled canonically.
    ///
    /// The one rule: an extent-one axis reads no coordinate, so its divisor and
    /// mirroring are unobservable and must be the canonical ones.
    #[must_use]
    pub const fn is_canonical(self) -> bool {
        if self.modulus == 1 {
            return self.divisor == 1 && !self.mirrored;
        }
        self.modulus != 0 && self.divisor != 0
    }
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
    /// A structural rearrangement: each iteration coordinate reads one operand
    /// element, and each operand element is read exactly once.
    ///
    /// The access relation of the registered `tiler::reindex-f32@1` family. It
    /// computes nothing — the value written is the value read — so it never
    /// appears as a region's scalar program; it appears as the *read map* of a
    /// region whose arithmetic comes from a fused neighbour.
    ///
    /// **The bijectivity is proven, not declared.**
    /// [`reindex_decodes_are_bijective`] requires the decodes to tile the
    /// iteration domain's linear coordinate exactly once, which is what makes
    /// "every operand element read exactly once" a checked fact rather than a
    /// producer's claim — and it is what separates this from
    /// [`Self::BroadcastReplication`], whose decodes deliberately leave part of
    /// the domain uncovered.
    ///
    /// Deliberately **not** a [`Self::BroadcastReplication`] whose replication
    /// happens to be empty. The two relations differ in what a consumer may
    /// conclude: a bijection lets a fusion authority treat the read as consuming
    /// its operand once, and a replication does not. Collapsing them would make
    /// that distinction a shape computation at every consumer rather than a
    /// variant the compiler checks.
    ReindexBijection {
        /// Shape of the operand being read.
        operand_shape: Shape,
        /// Shape of the region's result, whose linear coordinate is decoded.
        result_shape: Shape,
        /// One decode per operand axis, in axis order.
        axes: Vec<AxisDecode>,
    },
    /// A widening replication: several iteration coordinates read one operand
    /// element.
    ///
    /// The access relation of the registered `tiler::broadcast-f32@2` family
    /// when it actually widens. It is **not** [`Self::ScalarBroadcast`], and the
    /// difference is the whole reason it exists: `ScalarBroadcast` is a
    /// rank-zero operand element read by every invocation, so it carries no
    /// shape and no axis correspondence, while this relation carries both — a
    /// `[1024]` weight read across a `[T, 1024]` activation reads a *different*
    /// operand element per column and the same one per row, which
    /// `ScalarBroadcast` cannot express at all.
    ///
    /// Each operand axis either reads one result axis of equal extent or is a
    /// stretched extent-one axis whose coordinate is zero; a result axis no
    /// operand axis names is replicated, and the read is invariant in it.
    /// [`broadcast_decodes_are_replicating`] requires at least one such
    /// uncovered coordinate, so a "broadcast" that widens nothing is refused
    /// rather than admitted as a second spelling of [`Self::LinearIdentity`] —
    /// the canonicality rule the reindex form vocabulary states for its own
    /// identity mappings.
    BroadcastReplication {
        /// Shape of the operand being read.
        operand_shape: Shape,
        /// Shape of the region's result, whose linear coordinate is decoded.
        result_shape: Shape,
        /// One decode per operand axis, in axis order.
        axes: Vec<AxisDecode>,
    },
    /// A sourced broadcast relation over its whole symbolic domain.
    ///
    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-parametric-broadcast-access-surface`].
    /// Dependents may treat this variant as accepted vocabulary.
    ///
    /// [`accept-the-parametric-broadcast-access-surface`]: ../../../../../tickets/accept-the-parametric-broadcast-access-surface.md
    ///
    /// The access relation of a sourced `tiler::broadcast-f32@2` mapping,
    /// including the bijective binding at one. It is **not**
    /// [`Self::BroadcastReplication`] and **not** [`Self::ReindexBijection`].
    /// Those remain exact over their concrete subjects. Consumers must match
    /// this carrier explicitly. Replication-only fusion and costing are
    /// admitted only when the named environment proves actual widening.
    ///
    /// The payload is the authored operand, the sourced mapping, and the
    /// environment identity needed to interpret the symbols. It does not bind
    /// an extent and does not select a concrete neighbour.
    ParametricBroadcast {
        /// Sourced shape of the operand being read.
        operand_shape: crate::shape::SourcedShape,
        /// The sourced operand/result mapping, including symbolic pads.
        mapping: crate::semantic::BroadcastAxisMapping,
        /// Exact identity of the environment that interprets the mapping.
        environment: crate::shape::ShapeEnvIdentity,
    },
    /// The one live-inner-loop **source**: this input read addresses its tensor
    /// by the live row-major relation and marks its `inner_axis` as the
    /// region's single runtime extent operand. One invocation owns a static
    /// outer coordinate and loops the live inner extent.
    ///
    /// **Accepted public surface.** Tom accepted this exact fieldless-marker
    /// spelling on 2026-08-18 under
    /// [`decide-the-source-bound-live-row-major-access-surface`], replacing the
    /// 2026-08-13 contextual `LiveRowMajor { inner_axis }` relation whole: the
    /// source and every consumer are disjoint explicit variants, and the
    /// retired relation's tag `0x09` is never reinterpreted.
    ///
    /// [`decide-the-source-bound-live-row-major-access-surface`]: ../../../../../tickets/decide-the-source-bound-live-row-major-access-surface.md
    ///
    /// The inner axis is an
    /// [`crate::program::abi::AbiRoot::InputExtent`] consumed in the payload
    /// rather than specialized into the schedule. The iteration domain is the
    /// static outer product; the live inner extent is not a schedule identity
    /// value. Intrinsic verification requires exactly one source per live
    /// pointwise region, on a [`TensorRole::Input`] read, and requires every
    /// other pointwise read and the final write to carry the fieldless
    /// [`Self::LiveRowMajor`] consumer marker — the four
    /// [`super::LiveRowMajorSourceRule`]s are that relation's own diagnostics.
    LiveRowMajorSource {
        /// Axis of this tensor whose live extent is the inner stride and loop
        /// bound.
        inner_axis: Axis,
    },
    /// One live-inner-loop **consumer**: this access is driven by the loop the
    /// region's unique [`Self::LiveRowMajorSource`] marker declares.
    ///
    /// **Accepted public surface** (2026-08-18, the same record as
    /// [`Self::LiveRowMajorSource`]). Fieldless deliberately: in the admitted
    /// exact same-shape rank-one population the unique verified marker's axis
    /// is the only axis a consumer could name, so a repeated `inner_axis` or a
    /// `source_access` handle here would be a second authority that can
    /// disagree — the axis-mismatch and dangling-reference failure states are
    /// unrepresentable rather than representable-and-refused. Interpreting a
    /// consumer therefore requires the containing verified region's unique
    /// marker; a constructed region carrying consumers and no marker reports
    /// [`super::LiveRowMajorSourceRule::Missing`].
    ///
    /// A unit variant cannot acquire an `inner_axis` or `source_access` field
    /// without a reviewed public decision, and each spelling below must stay a
    /// build error (`E0559`: the variant has no such field):
    ///
    /// ```compile_fail,E0559
    /// use tiler_ir::schedule::LogicalAccess;
    /// use tiler_ir::shape::Axis;
    /// let _ = LogicalAccess::LiveRowMajor { inner_axis: Axis::new(0) };
    /// ```
    ///
    /// ```compile_fail,E0559
    /// use tiler_ir::schedule::{AccessOrdinal, LogicalAccess};
    /// let _ = LogicalAccess::LiveRowMajor { source_access: AccessOrdinal::FIRST };
    /// ```
    LiveRowMajor,
    /// One partitioned-copy read: the whole source, addressed by the copy's
    /// derived member rectangle.
    ///
    /// **Accepted public surface.** Tom accepted this exact fieldless spelling
    /// on 2026-08-18 under
    /// [`decide-the-partitioned-copy-scheduled-region-public-surface`].
    ///
    /// [`decide-the-partitioned-copy-scheduled-region-public-surface`]: ../../../../../tickets/decide-the-partitioned-copy-scheduled-region-public-surface.md
    ///
    /// Fieldless deliberately: which members read this source, at which derived
    /// destination offsets, over which derived source shape are all total
    /// functions of the region's [`PartitionedCopyProgram`], so a field here
    /// would be a second spelling two regions could disagree in — the rule the
    /// cooperative tile's underived visibility edges already state. The map is
    /// admissible only on a [`RegionProgram::PartitionedCopy`] region's reads;
    /// every other program family's admission refuses it by name.
    PartitionedCopySource,
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
///         ScalarProgram::PointwiseBf16(_) => false,
///         ScalarProgram::StrictAffineU4Dequantize { .. } => false,
///         ScalarProgram::StrictSerialSum { .. }
///         | ScalarProgram::SquaredSerialSum { .. }
///         | ScalarProgram::SquaredSerialSumThenEpilogue { .. }
///         | ScalarProgram::FusedMultiplyAddSerialSum { .. }
///         | ScalarProgram::StrictTensorContraction { .. }
///         | ScalarProgram::StrictSerialMaximum { .. } => true,
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
    /// An exact physical `bf16` pointwise expression.
    ///
    /// A variant of its own rather than a width field on [`Self::PointwiseF32`],
    /// and the separation is the same one
    /// [`PointwiseBf16Expression`] states: the
    /// two vocabularies are different node sets, their constants are different
    /// widths, and the arithmetic each names belongs to a different registered
    /// operation family. Every consumer that classifies a region by dtype —
    /// the boundary carrier, the kernel signature, the emitted arithmetic —
    /// therefore decides on the variant rather than on a field it could forget
    /// to read.
    ///
    /// The region's own
    /// [`NumericalRealization::canonical_arithmetic_nan_bits`](super::NumericalRealization::canonical_arithmetic_nan_bits)
    /// must carry the `bf16` canonical arithmetic NaN payload zero-extended into
    /// the 32-bit field, which the intrinsic verifier requires rather than
    /// assumes.
    PointwiseBf16(PointwiseBf16Expression),
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
    /// epilogue field: this variant is the fold *alone*, and a family that
    /// transforms the folded value before committing it carries
    /// [`Self::SquaredSerialSumThenEpilogue`] instead. Which of the two a program
    /// needs is decided by where the transform belongs, and that is the operation's
    /// question rather than a schedule's — `tiler::rms-norm-f32@1` computes its
    /// scale once per folded row and reads it once per point, so its epilogue is
    /// inside the fold's region; a family whose transform is genuinely per point
    /// leaves it to the pass that consumes this reduction's result.
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
    /// A squared serial fold whose value a scalar epilogue then transforms.
    ///
    /// The producing stage of a staged elementary family: one fold per kept
    /// coordinate, then a chain of governed scalar operations applied to *the
    /// fold's value*, and the chain's result is what the region commits.
    /// `tiler::rms-norm-f32@1`'s
    /// [`IndexRealizationLaw::StagedRootMeanSquareScaleF32`](crate::index::IndexRealizationLaw::StagedRootMeanSquareScaleF32)
    /// is the shipped instance, whose epilogue is `Rsqrt(a / N + eps)`.
    ///
    /// **Accepted public surface** — by Tom on 2026-08-06 at the live session's
    /// decision round, as-is with no exclusion;
    /// [`accept-the-fold-with-epilogue-scheduled-region`](../../../../tickets/accept-the-fold-with-epilogue-scheduled-region.md)
    /// records the provenance. Acceptance is not stabilization: this is accepted
    /// pre-alpha vocabulary, not a published API with compatibility obligations.
    ///
    /// **Why the epilogue is inside this region and not in the consuming pass.**
    /// The accepted law's own derivation: the folded row's scale is computed once
    /// per *row* and read once per *point*, so publishing the bare fold and putting
    /// the transform in the pointwise pass evaluates it `N` times per row. That is
    /// a different scalar program, not a different schedule for this one — the
    /// arithmetic count differs — so the vocabulary must be able to state which
    /// one a region performs. It is not two iteration domains in one region: the
    /// fold's contributor loop and the epilogue are both per output position of
    /// *this* region's domain, exactly as a prologue is per contributor.
    ///
    /// **The epilogue is general and the fold is not, deliberately.** The epilogue
    /// is a whole verified [`PointwiseF32Expression`] over one input — the fold's
    /// value, read as ordinal zero — so any chain the physical `f32` vocabulary
    /// spells is expressible without a further variant: a mean is `a / N`, this
    /// family's scale is `Rsqrt(a / N + eps)`, and a reciprocal-sum normalizer
    /// would be `c / a`. The *fold* stays one variant per (prologue, combiner)
    /// pair, which is the grain [`Self::SquaredSerialSum`] and
    /// [`Self::StrictSerialMaximum`] already set. The consequence is named rather
    /// than hidden: the softmax's shifting stage folds a *maximum* and would need
    /// its own sibling here, exactly as `StrictSerialMaximum` is a sibling of
    /// `Self::StrictSerialSum`; what it inherits unchanged is this epilogue field
    /// and every derivation threaded for it — the verifier's rules, the identity
    /// payload, the lowering's epilogue hook, and the split refusal below.
    ///
    /// **The epilogue must transform something.** An epilogue whose root is its own
    /// input leaf computes nothing, and admitting it would give one program two
    /// spellings — this variant and [`Self::SquaredSerialSum`] — which is the
    /// canonicality rule [`broadcast_decodes_are_replicating`] states for its own
    /// degenerate case. The schedule verifier refuses it.
    ///
    /// **No parallel topology may split it**, and the refusal is the family's
    /// algebra rather than caution: the epilogue applies to the *complete* fold, so
    /// a partial pass that applied it would transform a fragment and one that did
    /// not would be computing [`Self::SquaredSerialSum`] under this variant's name.
    /// A split of this family therefore needs a *pair* of programs rather than a
    /// partition of one, which no cover states; `split_family` classifies this
    /// variant as serial-only and every parallel topology is refused.
    ///
    /// `empty_identity_bits` is the value the *fold* commits over an empty
    /// contributor domain, and the epilogue transforms that value like any other:
    /// the program is "fold, then epilogue", so the empty case differs only in
    /// where the fold's value came from. Nothing in the shipped law reaches it —
    /// `rms-scale-empty-fold` refuses an empty fold a layer up — and stating it
    /// here is what keeps the variant's meaning total rather than conditional on
    /// its one producer.
    SquaredSerialSumThenEpilogue {
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Canonical arithmetic NaN bit pattern.
        canonical_nan_bits: u32,
        /// Empty-reduction identity bit pattern the fold commits.
        empty_identity_bits: u32,
        /// The chain applied to the fold's value before it is committed.
        ///
        /// A one-input expression whose sole leaf, ordinal zero, *is* the folded
        /// value. The ordinal names no boundary tensor here — this region reads
        /// exactly one, its contributor domain — so the schedule verifier requires
        /// the expression to hold exactly one leaf and the lowering supplies the
        /// accumulator for it. A second leaf would name a buffer no reduction
        /// region binds.
        epilogue: PointwiseF32Expression,
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
    /// A serial reduction under the NaN-propagating `Maximum` extrema family.
    ///
    /// The first fold `tiler::softmax-f32@1` embeds, and this vocabulary's first
    /// reduction whose combiner is not `Add`. It is a *new reducer* rather than a
    /// prologue or epilogue over an existing one, which is the difference from
    /// [`Self::SquaredSerialSum`]: no choice of per-contributor expression turns a
    /// sum into a maximum.
    ///
    /// **There is deliberately no empty-domain identity, and the omission is the
    /// contract rather than an oversight.** What a maximum over *no* contributors
    /// means is a declaration the operation makes, never a schedule or a backend
    /// ([ADR 0022](../../../../docs/decisions/0022-reduction-identities-and-initial-values.md)),
    /// and no registered operation embedding this fold declares one — so a field
    /// here would commit a semantic answer nothing has given, which is the same
    /// reasoning [`Self::StrictTensorContraction`] states for its unseeded fold.
    /// [Numerical semantics](../../../../docs/numerical-semantics.md) records the
    /// standing obligation such a reduction carries — it "is valid only with an
    /// explicit initial value or a proven/runtime-validated non-empty domain" —
    /// and this family discharges it with the domain: the schedule verifier
    /// refuses a reduced domain with no contributors, and the lowering refuses it
    /// again where it could still emit.
    ///
    /// **That refusal is not the claim that no binary32 value is neutral for this
    /// fold, and running the two together is what makes the family look
    /// unpaddable.** `0xff80_0000` (`-inf`) is a two-sided identity of the pinned
    /// family: it is the order's minimum, so every finite value, both infinities,
    /// and — under the `-0.0 < +0.0` ordering — both zeros compare above it and
    /// come back with their own bits; a NaN operand propagates, and the fold's
    /// per-combine canonicalization then commits the same bits an unpadded fold
    /// over that NaN commits. `-inf` is therefore a *padding* value proved
    /// observably neutral, which
    /// [ADR 0025](../../../../docs/decisions/0025-reduction-empty-results-and-padding.md)
    /// keeps separate from an empty-domain result in both directions: proving one
    /// neither supplies nor weakens the other, and
    /// [ADR 0100](../../../../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md)
    /// decision 7 owns the walk and the admission. The padding identity does not
    /// sit on this program: exact coverage still carries none, and an
    /// identity-padded split states one on [`ContributorCoverage`]. [`TailPolicy`]
    /// remains iteration-domain launch coverage and still admits `Exact` alone.
    /// [`ContributorPartition::covers`] keeps its exact meaning.
    ///
    /// **The `-0.0 < +0.0` ordering makes this fold order-insensitive**, which is
    /// what separates its legality from every sum in this vocabulary: the pinned
    /// family is associative and commutative on *every* binary32 input, so any
    /// tree over the same contributors gives the same bits. Every reduction
    /// topology this vocabulary states is therefore admitted for it — the serial
    /// fold, the [`ReductionTopology::MultiPass`] split, and the
    /// [`ReductionTopology::CooperativeWorkgroup`] tile — and admitted *under a
    /// strict contract*, because a split of this family spends no reassociation
    /// permission. The missing empty-domain value reaches the parallel forms as
    /// the non-emptiness precondition rather than as a staged `has_value` flag,
    /// and the argument is an *exactly covering* split's: that contract makes
    /// every partition's contributor count a nonzero factor of a nonzero product,
    /// so each staged partial is a real maximum. A split covering a padded
    /// sequence would need the padding identity above instead, which is why the
    /// two facts are stated apart.
    ///
    /// It carries `canonical_nan_bits` like every other reduction: a maximum
    /// selects bit patterns rather than computing values, but a NaN it selects is
    /// still an arithmetic reduction's result and follows the result-boundary
    /// canonicalization rule.
    StrictSerialMaximum {
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Canonical arithmetic NaN bit pattern.
        canonical_nan_bits: u32,
    },
}

/// The computation class one scheduled region performs, and the state that
/// class carries.
///
/// **Accepted public surface.** Tom accepted this exact spelling on 2026-08-18
/// under [`decide-the-partitioned-copy-scheduled-region-public-surface`]: a
/// field-replacing sum on [`IndexRegion`], so a region carrying a copy program
/// plus a numerical realization, an arithmetic region without one, and an
/// unclassified empty state are all unrepresentable rather than
/// representable-and-refused.
///
/// [`decide-the-partitioned-copy-scheduled-region-public-surface`]: ../../../../../tickets/decide-the-partitioned-copy-scheduled-region-public-surface.md
///
/// **Do not add `#[non_exhaustive]`.** An ADR 0074 convention 5b type for the
/// reason [`ScalarProgram`] states at its own definition: `tiler-compiler`'s
/// `physical.rs` and `frontier.rs` map the program totally from outside this
/// crate, so a third computation class must stop those builds rather than reach
/// a wildcard that answers for a program it was never checked against.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegionProgram {
    /// An arithmetic region: a scalar program evaluated per output under a
    /// declared numerical realization.
    Numerical {
        /// Scalar program evaluated per output.
        scalar: ScalarProgram,
        /// Preserved numerical realization.
        numerical: NumericalRealization,
    },
    /// A partitioned bit-preserving copy: no arithmetic, no realization.
    ///
    /// The variant carries no [`NumericalRealization`] deliberately — a copy
    /// performs no arithmetic, so a realization here would mint an identity for
    /// arithmetic no kernel performs, and an optional one would conflate
    /// missing with proved inapplicability (both eliminated by the accepted
    /// 2026-08-12 concatenate decision).
    PartitionedCopy(PartitionedCopyProgram),
}

impl RegionProgram {
    /// Returns the declared numerical realization, when this program class
    /// carries one.
    ///
    /// The total replacement for every former `region.index.numerical` read:
    /// `None` is the copy arm's proved absence of a realization, not a missing
    /// field.
    #[must_use]
    pub const fn numerical(&self) -> Option<&NumericalRealization> {
        match self {
            Self::Numerical { numerical, .. } => Some(numerical),
            Self::PartitionedCopy(_) => None,
        }
    }
}

/// One partitioned bit-preserving copy over a single concatenated axis.
///
/// **Accepted public surface** (2026-08-18, same record as [`RegionProgram`]).
/// Ordered members are semantic identity — `members[k]` is concatenate operand
/// `k`, never deduplicated, so `concat(x, x)` is two members over one
/// deduplicated read. Destination offsets, member source shapes, and
/// destination rectangles are **derived, never stored**: a field a producer
/// could set beside its derivation is a second spelling two regions could
/// disagree in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartitionedCopyProgram {
    /// The one bit-preserved element format.
    pub element: CopyElement,
    /// The concatenated axis of the iteration domain.
    pub axis: Axis,
    /// Ordered operand members, one per concatenate operand.
    pub members: Vec<CopyMember>,
}

impl PartitionedCopyProgram {
    /// Returns the exclusive prefix sums of the member extents, or `None` when
    /// a sum overflows `u64`.
    ///
    /// Entry `k` is member `k`'s destination offset on the copy axis. Derived
    /// on demand rather than stored, so intervals `[offset_k, offset_k +
    /// extent_k)` are adjacent by construction and the only representable
    /// coverage defect is a wrong extent total.
    #[must_use]
    pub fn member_offsets(&self) -> Option<Vec<u64>> {
        let mut offsets = Vec::with_capacity(self.members.len());
        let mut running = 0_u64;
        for member in &self.members {
            offsets.push(running);
            running = running.checked_add(member.extent)?;
        }
        Some(offsets)
    }

    /// Returns member `k`'s whole-source shape: the iteration shape with the
    /// copy axis's extent replaced by the member's extent.
    ///
    /// `None` when the member index or the copy axis is out of range.
    #[must_use]
    pub fn member_source_shape(&self, iteration_shape: &Shape, member: usize) -> Option<Shape> {
        let member = self.members.get(member)?;
        let axis = usize::try_from(self.axis.get()).ok()?;
        if axis >= iteration_shape.rank() {
            return None;
        }
        let extents = iteration_shape
            .extents()
            .iter()
            .enumerate()
            .map(|(position, extent)| {
                if position == axis {
                    crate::shape::Extent::new(member.extent)
                } else {
                    *extent
                }
            });
        Shape::try_new(extents).ok()
    }
}

/// One ordered member of a partitioned copy: which read it copies from, and
/// how much of the copy axis it owns.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CopyMember {
    /// The read access this member copies from, by ordinal into the region's
    /// access list.
    pub source: AccessOrdinal,
    /// This member's extent on the copy axis.
    pub extent: u64,
}

/// The element format a partitioned copy bit-preserves.
///
/// Closed at one variant deliberately: widening to another dtype is an
/// identity-visible act — a new variant, a new tag, and build errors at every
/// encoder and total match — never a field a producer could vary. **Do not add
/// `#[non_exhaustive]`** (ADR 0074 convention 5b, same reason as
/// [`RegionProgram`]).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CopyElement {
    /// IEEE-754 binary32 bit patterns, moved unchanged.
    F32,
}

impl CopyElement {
    /// Returns the canonical identity tag naming this element format.
    ///
    /// Written by an exhaustive match rather than read from the discriminant,
    /// so adding or reordering a variant is a build error here instead of a
    /// silent change to every identity ever produced (ADR 0074 convention 3).
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::F32 => 0x01,
        }
    }

    /// Returns the storage width of one element, in bytes.
    ///
    /// The load/store width the derivation KIR reads.
    #[must_use]
    pub const fn storage_bytes(self) -> u64 {
        match self {
            Self::F32 => 4,
        }
    }
}

/// The bounded index region a schedule maps onto a target machine.
///
/// This carries the iteration domain, logical accesses, bounds and ownership
/// proofs, and the region program. It deliberately does not carry any
/// semantic-graph correlation; binding a region to semantic occurrences is a
/// separate compiler-owned refinement (ADR 0070).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRegion {
    /// Planning ordinal, excluded from canonical identity.
    pub id: RegionId,
    /// Parallel iteration domain of the region.
    pub iteration_shape: Shape,
    /// Logical accesses: the region's reads, followed by its one owning write.
    ///
    /// The read count is the region program's rather than a constant — one per
    /// expression leaf for a pointwise family, exactly one for a serial
    /// reduction, two for a contraction, three for the affine `u4` dequantize,
    /// one per distinct source boundary for a partitioned copy — and
    /// `verify_intrinsic` refuses a list whose length disagrees with the
    /// family the region declares.
    pub accesses: Vec<Access>,
    /// Bounds proofs, one per access.
    pub bounds_proofs: Vec<BoundsProof>,
    /// The single write-ownership proof.
    pub ownership_proof: OwnershipProof,
    /// The computation class and its state.
    pub program: RegionProgram,
}

/// A literal fixed-vector lane count.
///
/// **Accepted public surface.** Tom accepted this exact spelling on 2026-08-12
/// under [`admit-vector-lane-bindings-into-the-schedule-vocabulary`]: a checked
/// `u64` constructor requiring at least two lanes and a `get` reader, and
/// nothing else. There is deliberately no power-of-two rule, architecture
/// preset, default, or independent lane-count cap.
///
/// [`admit-vector-lane-bindings-into-the-schedule-vocabulary`]: ../../../../../tickets/admit-vector-lane-bindings-into-the-schedule-vocabulary.md
///
/// The two refused widths are refused for different reasons and named
/// separately. Zero lanes name no packet, so the width is invalid outright.
/// One lane is the existing scalar map — packet `p` lane `0` owning output
/// `p` *is* [`ExecutionBinding::GlobalLinearInvocation`] — and admitting it
/// would give one schedule two spellings and two identities, which is the
/// canonicality rule [`broadcast_decodes_are_replicating`] states for its own
/// degenerate case.
///
/// The invariant is established at construction rather than re-checked by the
/// intrinsic verifier: the field is private and every constructor path refuses
/// the two widths, so a verifier check for them could never fail — the same
/// reasoning [`super::ParticipantSpace`] applies to its rank bound.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VectorLaneCount(u64);

impl VectorLaneCount {
    /// Constructs a literal lane count of at least two.
    ///
    /// # Errors
    ///
    /// Returns [`VectorLaneCountError::Zero`] for zero lanes and
    /// [`VectorLaneCountError::ScalarSpelling`] for one, each by name.
    pub const fn new(lanes: u64) -> Result<Self, VectorLaneCountError> {
        match lanes {
            0 => Err(VectorLaneCountError::Zero),
            1 => Err(VectorLaneCountError::ScalarSpelling),
            lanes => Ok(Self(lanes)),
        }
    }

    /// The lane count, always at least two.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// How a region binds execution coordinates to iteration coordinates.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a new execution binding lands additively. Verified by
/// marking it and compiling the workspace — no consumer broke. Total maps
/// *inside* `tiler-ir` are unaffected, because the attribute has no effect
/// within the defining crate, which is what keeps them breaking.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExecutionBinding {
    /// One global linear invocation per iteration coordinate.
    GlobalLinearInvocation,
    /// Hardware workgroup and local coordinates map once onto the contraction's
    /// logical output coordinates.
    ///
    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under
    /// [`accept-the-blocked-workgroup-and-cooperative-contraction-surface`].
    /// The 2026-08-11 model acceptance is the earlier packet: an explicit
    /// blocked-workgroup binding, required rather than defaulted, whose
    /// verifier supplies the bijection
    /// [`OwnershipProofKind::OneGlobalInvocationPerOutput`] states.
    ///
    /// [`accept-the-blocked-workgroup-and-cooperative-contraction-surface`]: ../../../../../tickets/accept-the-blocked-workgroup-and-cooperative-contraction-surface.md
    ///
    /// Invocation at logical workgroup `w` and local `l` owns output coordinate
    /// `w[d] * block[d] + l[d]` on each output axis `d`. The binding is the
    /// layer [ADR 0007](../../../../docs/decisions/0007-first-class-kernel-schedules.md)
    /// assigns hardware-to-logical mapping to, so both operand reads and the
    /// owning write consult it once. `Copy` is dropped because the map carries
    /// shapes; `GlobalLinearInvocation` keeps tag `0x01` and every earlier
    /// region's bytes.
    BlockedWorkgroup {
        /// Per-axis local extents, in the output shape's axis order.
        block: Shape,
        /// Per-axis workgroup-grid extents, in the same order.
        workgroups: Shape,
    },
    /// One fixed-width vector packet per `lanes` consecutive iteration
    /// coordinates of a map-parallel region.
    ///
    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-12 under
    /// [`admit-vector-lane-bindings-into-the-schedule-vocabulary`], the first
    /// schedule construct of [ADR 0093](../../../../docs/decisions/0093-bind-vector-lanes-to-the-map-or-the-contributor-partition.md):
    /// a lane over the *map*, never the contributor partition, a horizontal
    /// reduction, a scalable vector, a worker thread, or a backend instruction
    /// choice. The name states the axis it binds.
    ///
    /// [`admit-vector-lane-bindings-into-the-schedule-vocabulary`]: ../../../../../tickets/admit-vector-lane-bindings-into-the-schedule-vocabulary.md
    ///
    /// Packet `p`, lane `l` owns scalar output `p * lanes + l`. The launch
    /// identity follows the accepted correction: [`KernelSchedule::work_items`]
    /// stays the `N` logical scalar outputs, and [`LaunchPlan::grid_threads`]
    /// is the exact packet population `N / lanes`, because `grid_threads`'
    /// invariant is the number of executing invocations. No implementation may
    /// keep `grid_threads = N` and ask an emitter to reinterpret the builtin —
    /// that would execute `N * lanes` lane positions under a launch identity
    /// claiming only `N` outputs — and the intrinsic verifier refuses exactly
    /// that spelling by its own packet-population rule.
    ///
    /// The first admitted combination is deliberately narrow: [`TailPolicy::Exact`]
    /// alone, over [`ReductionTopology::None`] and [`ReductionTopology::Serial`]
    /// — the pointwise map and the strict serial fold across independent
    /// outputs — with checked `N mod lanes == 0`. Grouping independent outputs
    /// into packets changes no operand, rounding site, or contributor order
    /// (ADR 0093 decision 2), so the admission consumes no numerical
    /// permission and the verifier never reads one to decide it. Predicated
    /// tails, scalar epilogues, contributor partitions, and scalable maps are
    /// separate accepted successors and do not reinterpret this variant.
    ///
    /// The carrier is not yet executable. Lane-shaped KIR, exact target
    /// requirements, provider-versioned execution and numerical evidence,
    /// artifact delivery, host qualification, and a real native `tiler-cpu` /
    /// `tiler-cpu-runtime` approach are all absent, so the kernel lowering and
    /// the refinement gate refuse this binding by
    /// `unlowered-execution-binding` rather than scalarizing it. Absence of an
    /// emission leaves the plan non-executable; it never authorizes a
    /// fallback.
    FixedVectorMap {
        /// Literal lane count, checked at construction to be at least two.
        lanes: VectorLaneCount,
    },
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
    /// The launch may be a strict superset of the logical iteration domain.
    ///
    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-12 under
    /// [`admit-guarded-output-tails-for-cooperative-contraction`]. The first
    /// admitted composition is the blocked-workgroup cooperative F32
    /// contraction. Active coordinates are derived from that binding; the
    /// variant carries no predicate payload, mask, or padding identity.
    ///
    /// [`admit-guarded-output-tails-for-cooperative-contraction`]: ../../../../../tickets/admit-guarded-output-tails-for-cooperative-contraction.md
    ///
    /// It does not mean contributor padding, inactive subgroup lanes, scalar
    /// peeling, or a backend-chosen mask. `Exact` keeps tag `0x01` and every
    /// earlier region's bytes.
    Predicated,
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
        /// Whether this pass covers its contributor sequence exactly or extends
        /// it by proved identity values.
        ///
        /// **Accepted public surface.** Tom accepted this exact spelling on
        /// 2026-08-13 under [`accept-the-contributor-coverage-and-padding-identity-surface`].
        /// The 2026-08-11 model acceptance is the earlier packet.
        ///
        /// [`accept-the-contributor-coverage-and-padding-identity-surface`]: ../../../../../tickets/accept-the-contributor-coverage-and-padding-identity-surface.md
        coverage: ContributorCoverage,
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
        /// this is true — for every family whose fold order is observable. The
        /// one exception is [`ScalarProgram::StrictSerialMaximum`], where every
        /// tree over the same contributors returns the same bits, so the split
        /// changes nothing the permission governs and spends none of it. The
        /// field is still required to agree with the region's declared
        /// realization in either case.
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
    /// A contraction whose contracted extent is a live input-axis operand.
    ///
    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    /// Dependents may treat this variant as accepted vocabulary.
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// The contracted trip count is the named input axis, not a specialized
    /// `Shape`. Output shape and free indices stay static; only the contracted
    /// extent is live.
    LiveContraction {
        /// Input whose axis supplies the contracted trip count.
        live_access: AccessOrdinal,
        /// Axis of that input whose live extent is the contracted bound.
        live_axis: Axis,
        /// Contributor combination order within the contracted space.
        order: ContributorOrder,
        /// Whether the contract permits reassociation.
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
    /// [`CooperativeTile::visibility_edges`] states, and which the tile's own
    /// [`SynchronizationPoint`]s authorize: the schedule verifier requires
    /// exactly one point to discharge each edge, and the structured-kernel
    /// verifier separately proves the emitted body puts that point's barrier
    /// between the staged write and the staged read. A tile whose phases repeat
    /// carries a second class, [`super::AntiDependencyEdge`], under the same
    /// rule.
    ///
    /// What no schedule authorizes is the *machine*: whether a target can
    /// perform the realization those points require is a feasibility question
    /// composed against a target profile's own declaration, and a schedule
    /// proving its own ordering is not a claim that any device offers it.
    ///
    /// The split is a reassociation of the declared contributor sequence and
    /// never a permutation of it, for the reason [`ContributorPartition`]
    /// records: participant `p` combines the contiguous contributor range that
    /// partition owns, and the staged partials are combined in ascending `p`.
    CooperativeWorkgroup {
        /// How one output's contributor sequence is split across participants,
        /// per round, and whether that split covers the real sequence or a
        /// suffix-padded one.
        ///
        /// **Accepted public surface.** Tom accepted this exact spelling on
        /// 2026-08-13 under [`accept-the-contributor-coverage-and-padding-identity-surface`].
        /// The 2026-08-11 model acceptance is the earlier packet.
        ///
        /// [`accept-the-contributor-coverage-and-padding-identity-surface`]: ../../../../../tickets/accept-the-contributor-coverage-and-padding-identity-surface.md
        ///
        /// `contributors_per_partition` is what one participant folds on *one*
        /// round, so the sequence an exact split covers is
        /// `partitions * contributors_per_partition * tile.rounds`. On a
        /// single-round tile that is the plain product and the field means
        /// exactly what it does for [`Self::MultiPass`]; on a loop-carried one,
        /// participant `p` of round `r` owns the contiguous range at index
        /// `r * partitions + p`, which is why the coverage stays ascending and
        /// the strategy still consumes reassociation alone. An identity-padded
        /// split covers that same capacity, with the verifier deriving the
        /// padding count as capacity minus the real contributor count.
        coverage: ContributorCoverage,
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
        /// The permission this strategy consumes, and the verifier admits the
        /// topology only when it holds — except over
        /// [`ScalarProgram::StrictSerialMaximum`], whose fold is order-insensitive
        /// on every binary32 input, so a tile over it consumes nothing and a
        /// strict contract admits one. Required to agree with the region's
        /// declared realization either way.
        permits_reassociation: bool,
        /// Whether the contract permits contributor permutation.
        ///
        /// Recorded because the region preserves the declared realization
        /// whole. Whether it is *consumed* depends on `arrival`, which is why
        /// the two are separate fields rather than one summary.
        permits_permutation: bool,
        /// Order in which the staged partials reach the combining participant.
        ///
        /// The field that decides whether this strategy consumes permutation in
        /// addition to reassociation: the admitted
        /// [`ContributorArrival::AscendingParticipant`] fixes the combine order
        /// in the program and consumes reassociation alone, while an arrival the
        /// program does not fix reorders the contributors themselves. Stated on
        /// the topology so the composition is checkable rather than inferred
        /// from whatever body a backend happens to emit.
        arrival: ContributorArrival,
    },
    /// One workgroup's invocations each own an output position and cooperate by
    /// staging shared operand tiles.
    ///
    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under
    /// [`accept-the-blocked-workgroup-and-cooperative-contraction-surface`].
    /// The 2026-08-11 model acceptance is the earlier packet: a sibling of
    /// [`Self::CooperativeWorkgroup`] with its own semantic, commit, coverage,
    /// and shape verifier, reusing the [`CooperativeTile`] dataflow record. The
    /// one-committer theorem on [`Self::CooperativeWorkgroup`] is unchanged.
    ///
    /// [`accept-the-blocked-workgroup-and-cooperative-contraction-surface`]: ../../../../../tickets/accept-the-blocked-workgroup-and-cooperative-contraction-surface.md
    ///
    /// The inverse relation: `commit` names every participant, the iteration
    /// domain *is* the output (no trailing participant axis), and
    /// `owned_output_positions` equals the work-item count. No helper may infer
    /// the one-committer ownership theorem from the mere presence of a tile.
    /// The topology requires [`ExecutionBinding::BlockedWorkgroup`]; it is
    /// never defaulted from [`ExecutionBinding::GlobalLinearInvocation`].
    CooperativeContraction {
        /// The cross-invocation operand staging that tile requires.
        tile: CooperativeTile,
        /// Row-major shape of the contracted iteration space, in ascending
        /// canonical contracted-index order.
        contracted_shape: Shape,
        /// Exact tile of that contracted space. Every extent must divide the
        /// matching contracted extent; the quotient product is `tile.rounds`.
        contracted_tile: Shape,
        /// Contributor combination order within the contracted space.
        order: ContributorOrder,
        /// Width every combining step is performed at.
        accumulation: ArithmeticType,
        /// Whether the contract permits reassociation.
        ///
        /// Tiling the contracted space regroups the declared contributor
        /// sequence, so the verifier admits this topology only when this is
        /// true.
        permits_reassociation: bool,
        /// Whether the contract permits contributor permutation.
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

/// Whether a reduction topology covers its contributor sequence exactly or
/// extends it by proved identity values.
///
/// **Accepted public surface.** Tom accepted this exact spelling on 2026-08-13
/// under [`accept-the-contributor-coverage-and-padding-identity-surface`]. The
/// 2026-08-11 model acceptance is the earlier packet; this label is the
/// included/excluded Rust surface. A required tagged coverage whose exact arm
/// carries no identity and whose padded arm cannot omit one.
///
/// [`accept-the-contributor-coverage-and-padding-identity-surface`]: ../../../../../tickets/accept-the-contributor-coverage-and-padding-identity-surface.md
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: every out-of-crate
/// consumer constructs a variant or reads a field, and none classifies this
/// type by exhaustive match, so a new coverage mode lands additively. Total
/// maps *inside* `tiler-ir` are unaffected, because the attribute has no
/// effect within the defining crate, which is what keeps them breaking.
///
/// Exact coverage is [`ContributorPartition::covers`] (and that product times
/// the tile's round count, for a cooperative split). Identity-padded coverage
/// states a [`ReductionPaddingIdentity`] the intrinsic verifier must prove
/// two-sided-neutral; it never falls back to a family's empty-domain bits.
///
/// ```compile_fail,E0004
/// use tiler_ir::schedule::ContributorCoverage;
/// fn classify(coverage: ContributorCoverage) -> u8 {
///     match coverage {
///         ContributorCoverage::Exact(_) => 0,
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ContributorCoverage {
    /// The split covers the real contributor sequence exactly once each.
    Exact(ContributorPartition),
    /// The split covers a suffix-padded sequence whose extra positions hold
    /// the stated identity.
    IdentityPadded {
        /// The physical split whose capacity is the padded length.
        partition: ContributorPartition,
        /// The value every padded position holds.
        identity: ReductionPaddingIdentity,
    },
}

impl ContributorCoverage {
    /// Returns the physical split this coverage states.
    #[must_use]
    pub const fn partition(self) -> ContributorPartition {
        match self {
            Self::Exact(partition) | Self::IdentityPadded { partition, .. } => partition,
        }
    }
}

/// An exact arithmetic value a padded contributor position holds.
///
/// **Accepted public surface.** Tom accepted this exact spelling on 2026-08-13
/// under [`accept-the-contributor-coverage-and-padding-identity-surface`]. The
/// 2026-08-11 model acceptance is the earlier packet. An opaque or
/// width-discriminated exact-bit carrier whose format and bit width cannot
/// disagree, proved two-sided-neutral by intrinsic verification rather than
/// trusted as a statement.
///
/// [`accept-the-contributor-coverage-and-padding-identity-surface`]: ../../../../../tickets/accept-the-contributor-coverage-and-padding-identity-surface.md
///
/// Each variant is the format's own width. A raw `u32` field would freeze an
/// `f32`-only boundary and let a `bf16` payload occupy four bytes; pairing a
/// separate [`ArithmeticType`] with a widest-width integer would let the two
/// disagree. The verifier still checks the variant against the region's own
/// arithmetic, so a well-typed `bf16` identity on an `f32` fold is a named
/// mismatch rather than an unrepresentable one.
///
/// Not `#[non_exhaustive]`: the encoder maps every variant onto a tag and a
/// fixed-width payload, so widening this set is a build error there instead of
/// a silent identity collision (ADR 0074 convention 5b).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReductionPaddingIdentity {
    /// IEEE-754 binary16 bits.
    F16(u16),
    /// `bf16` bits.
    Bf16(u16),
    /// IEEE-754 binary32 bits.
    F32(u32),
    /// IEEE-754 binary64 bits.
    F64(u64),
}

impl ReductionPaddingIdentity {
    /// Returns the arithmetic type this identity is spelled in.
    #[must_use]
    pub const fn arithmetic_type(self) -> ArithmeticType {
        match self {
            Self::F16(_) => ArithmeticType::F16,
            Self::Bf16(_) => ArithmeticType::Bf16,
            Self::F32(_) => ArithmeticType::F32,
            Self::F64(_) => ArithmeticType::F64,
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

/// Live input-axis extents this region requires a kernel to consume.
///
/// Derived from the region's own access maps and reduction topology rather
/// than stored beside them, so a region that names no live extent stays the
/// same subject it was. Canonical order is `(access ordinal, axis)`.
///
/// A live pointwise region contributes exactly its unique
/// [`LogicalAccess::LiveRowMajorSource`] marker's coordinate; the fieldless
/// [`LogicalAccess::LiveRowMajor`] consumers deliberately contribute nothing,
/// because the marker is the region's one runtime extent authority and a
/// per-consumer operand would re-derive three operands for one runtime fact —
/// the `UnusedInputExtent` state the accepted source surface exists to remove.
///
/// # Panics
///
/// Panics only if a directly constructed, unverified region contains more than
/// `u32::MAX` accesses. Verified regions are bounded far below that population.
#[must_use]
pub fn live_input_extents(schedule: &ScheduledRegion) -> Vec<(AccessOrdinal, Axis)> {
    let mut extents = Vec::new();
    for (position, access) in schedule.index.accesses.iter().enumerate() {
        if let LogicalAccess::LiveRowMajorSource { inner_axis } = &access.map
            && matches!(access.tensor, TensorRole::Input)
        {
            let position = u32::try_from(position).expect("verified access count is bounded");
            extents.push((AccessOrdinal::new(position), *inner_axis));
        }
    }
    if let ReductionTopology::LiveContraction {
        live_access,
        live_axis,
        ..
    } = &schedule.schedule.reduction
    {
        extents.push((*live_access, *live_axis));
    }
    extents.sort_by_key(|(access, axis)| (access.get(), axis.get()));
    extents.dedup();
    extents
}

/// The unique live-row-major source marker's inner axis, when the region
/// carries one on an input read.
///
/// The one derivation the fieldless [`LogicalAccess::LiveRowMajor`] consumer is
/// interpreted through: a consumer's stride, loop bound, and element offset are
/// the marker's, so every reader takes the checked containing region rather
/// than a detached map. `None` is a region with no admissible marker — the
/// intrinsic verifier refuses a live pointwise region in that state, so a
/// `None` beside a consumer never reaches a verified product, and consumers of
/// this helper fail closed on it rather than defaulting an axis.
///
/// Deliberately first-match over input reads: intrinsic verification proves at
/// most one marker exists, and an unverified region carrying two is refused
/// under `live-row-major-source-multiple` before any identity or lowering
/// exists for this derivation to mislead.
pub(crate) fn live_source_axis(schedule: &ScheduledRegion) -> Option<Axis> {
    schedule.index.accesses.iter().find_map(|access| {
        if let LogicalAccess::LiveRowMajorSource { inner_axis } = &access.map
            && matches!(access.tensor, TensorRole::Input)
            && matches!(access.mode, AccessMode::Read)
        {
            Some(*inner_axis)
        } else {
            None
        }
    })
}

/// The index arithmetic one region's coordinate computation requires of a target.
///
/// **Nominal, not a width.** A raw `64` would state that a target can *spell* a
/// 64-bit integer, and a spellable type is a language fact:
/// `separate-metal-launch-index-from-index-and-address-width` already eliminated
/// deriving arithmetic support from one. What a region actually needs is
/// complete *operation* support over the governed KIR index family — the
/// addition, multiplication, division, and modulo its coordinate maps perform —
/// and that is a capability a target either has or does not. Naming it leaves no
/// room for a consumer to read a width and conclude a capability.
///
/// **An ADR 0074 convention 5b type, deliberately exhaustive.** It carries no
/// `#[non_exhaustive]`: a backend maps every variant to its own live-device
/// vocabulary as a total function, so a variant added here must stop that
/// backend's build rather than reach a wildcard that would answer for an
/// arithmetic it has never been asked about. Answering "supported" for an
/// unrecognized requirement is the silently-wrong fast path.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexArithmetic {
    /// Complete support for the governed unsigned-64 KIR index operation family.
    CompleteU64,
}

/// The numerical requirement one region's computation class states.
///
/// **Accepted public surface** (2026-08-18, the same record as
/// [`RegionProgram`]): the former ten flat floating-point fields of
/// [`ResourceRequirements`] become one required sum, so a copy region's
/// requirements cannot fabricate a floating-point row and an arithmetic
/// region's cannot omit one.
///
/// **Do not add `#[non_exhaustive]`.** An ADR 0074 convention 5b type: the
/// kernel-identity encoder, the artifact wire codec, and the compiler's
/// feasibility projection each map this totally from outside this crate, so a
/// third arm must stop those builds rather than reach a wildcard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionNumericalRequirements {
    /// The region performs floating-point arithmetic and requires each declared
    /// dimension honoured, carried forward per dimension rather than as one
    /// summary bit. A single `requires_strict_f32` boolean cannot name which
    /// dimension a target failed to honour, and the boolean these replaced was
    /// derived from contraction and reassociation alone — so a
    /// subnormal-preserving contract that permitted both transforms reported no
    /// strict-`f32` requirement at all (ADR 0076 item 3).
    ///
    /// The realization's `profile_key` and canonical NaN bits are deliberately
    /// not repeated here: they name the governing contract and a produced value
    /// rather than a behaviour a target profile declares honourability for, and
    /// they remain on the region's [`NumericalRealization`].
    FloatingPoint {
        /// Subnormal input handling the region's declared realization requires.
        input_subnormals: SubnormalMode,
        /// Subnormal result handling the region's declared realization requires.
        result_subnormals: SubnormalMode,
        /// Whether the region's declared realization permits contraction.
        contraction: NumericalPermission,
        /// Whether the region's declared realization permits reassociation.
        reassociation: NumericalPermission,
        /// Whether the region's declared realization permits contributor
        /// permutation.
        permutation: NumericalPermission,
        /// Whether the region's declared realization permits signed-zero
        /// elimination.
        signed_zero: NumericalPermission,
        /// Whether the region's declared realization permits reciprocal
        /// replacement.
        reciprocal_transform: NumericalPermission,
        /// The approximate-intrinsic envelope the region's declared realization
        /// authorizes.
        approximate_intrinsics: ApproximationEnvelope,
        /// The region's declared NaN-absence assumption.
        nan_assumptions: ExceptionalValueAssumption,
        /// The region's declared infinity-absence assumption.
        infinity_assumptions: ExceptionalValueAssumption,
    },
    /// The region moves bits and performs no floating-point arithmetic, so it
    /// states **no** floating-point requirement — a proved absence, not target
    /// silence. Consumption as proved absence is owned by
    /// `derive-target-numerical-feasibility-from-reached-arithmetic-only`.
    BitPreservingCopy,
}

/// Exact or proven resource requirements derived from a verified schedule.
///
/// These feed a separate phased target-feasibility assessment; deriving them is
/// part of intrinsic verification and never a target decision (ADR 0007).
///
/// The structural fields stay unconditional — a copy still binds buffers,
/// launches threads, and computes coordinates — while the numerical requirement
/// is the [`RegionNumericalRequirements`] sum the region's computation class
/// determines.
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
    /// The index arithmetic the region's coordinate computation requires.
    ///
    /// Not an `Option`, and the contrast with [`Self::synchronization`] beside
    /// it is the whole reason. A region with no synchronization point is
    /// reachable and derives `None`, so that absence is a fact some region
    /// states. Every scheduled region computes coordinates — an
    /// [`IndexRegion`] is a bounded coordinate space and its accesses are
    /// coordinate maps — so a region requiring no index arithmetic does not
    /// exist, and an `Option` would add an absence nothing can derive. It would
    /// also let a producer encode that absence and skip the device comparison
    /// entirely, which is the fast path this record exists to close.
    ///
    /// One value per region rather than one per operation: a target either
    /// supports the governed index family completely or does not, so counting
    /// the additions a region performs would be the barrier-count capacity
    /// `replace-or-justify-the-barrier-count-axis` retired.
    pub index_arithmetic: IndexArithmetic,
    /// The synchronization realization the region's schedule requires, if any.
    ///
    /// `None` is the canonical absence a schedule with no synchronization point
    /// derives, and it is not a zero: it emits no requirement, no target query,
    /// no explain row, and no artifact field, so a target that declares nothing
    /// about synchronization is *feasible* for such a region rather than merely
    /// untested. A `Some` is the complete
    /// [`SynchronizationSubject`] one atomic
    /// target fact must equal; it is deliberately not five independent
    /// dimensions, because each of them is separately true of some realization
    /// and their conjunction is what the region actually needs.
    ///
    /// One value rather than one per point: every point of a region is checked
    /// against the same derivation, so a region requires one realization however
    /// many times it performs it. A count of points would be the barrier-count
    /// capacity `replace-or-justify-the-barrier-count-axis` retired.
    pub synchronization: Option<SynchronizationSubject>,
    /// The subgroup realization the region's schedule requires, if any.
    ///
    /// `None` is the canonical absence a schedule with no subgroup combine
    /// derives, and it is not a default width: it emits no target requirement,
    /// query, explain row, or subgroup block in the artifact resource tail, so
    /// a target that declares nothing about subgroups is *feasible* for such a
    /// region rather than merely untested. A `Some` is the complete
    /// [`SubgroupRealizationSubject`] one atomic target fact must equal, and the
    /// artifact carrier preserves that whole subject without decomposing it.
    ///
    /// This ticket does not derive a `Some` from any admitted topology —
    /// subgroup KIR emission is a separate ticket — so every region produced
    /// here carries `None`.
    pub subgroup: Option<SubgroupRealizationSubject>,
    /// The numerical requirement the region's computation class states.
    ///
    /// Required rather than optional: an arithmetic region always states its
    /// floating-point rows and a copy region always states their proved
    /// absence, so no producer can encode a silence a feasibility authority
    /// would then have to interpret.
    pub numerical: RegionNumericalRequirements,
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

    /// **Draft surface, not yet accepted.**
    ///
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
        subnormal_freedom_of(&self.region.index.program)
    }
}

/// Classifies one region program's subnormal freedom.
///
/// The single definition both [`VerifiedScheduledRegion::subnormal_freedom`]
/// and the structured-kernel lowering read, so a kernel's answer cannot drift
/// from the region's.
pub(crate) const fn subnormal_freedom_of(program: &RegionProgram) -> SubnormalFreedom {
    let scalar = match program {
        RegionProgram::Numerical { scalar, .. } => scalar,
        // A copy performs no arithmetic and so produces no new subnormal — but
        // it moves bit patterns a flushing target could still alter at some
        // other site, and nothing here proves it cannot, so `Unproven` is the
        // fail-closed answer. A copy-specific freedom claim would need its own
        // KIR-level bit-preservation evidence and belongs to the feasibility
        // ticket if ever needed.
        RegionProgram::PartitionedCopy(_) => return SubnormalFreedom::Unproven,
    };
    match scalar {
        ScalarProgram::StrictAffineU4Dequantize { .. } => {
            SubnormalFreedom::StrictAffineNormalScaleDecode
        }
        ScalarProgram::PointwiseF32(_)
        // Nothing bounds a dense `bf16` payload away from the subnormal range
        // either, and the one freedom this vocabulary states is explicitly
        // `f32`-only: `SubnormalFreedom::discharges` answers `false` for
        // `ArithmeticType::Bf16`, because the decode derivation rests on `f32`'s
        // exponent range. `Unproven` is therefore the only honest answer here.
        | ScalarProgram::PointwiseBf16(_)
        | ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        // An epilogue over the fold's value bounds nothing either: its own
        // arithmetic is `f32` over a dense payload, and a reciprocal square root
        // reaches the subnormal range from below like any other division.
        | ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. }
        | ScalarProgram::StrictTensorContraction { .. }
        // A maximum performs no arithmetic, so it produces no *new* subnormal —
        // but it selects between operands a flushing target would have already
        // changed, so nothing here is proved and `Unproven` is the honest answer
        // rather than a discharge this variant has not earned.
        | ScalarProgram::StrictSerialMaximum { .. } => SubnormalFreedom::Unproven,
    }
}

/// The arithmetic type one scalar program's own operations are performed at.
///
/// The **one** derivation of a region's width, read by the schedule verifier's
/// accumulation gates and by the structured-kernel signature. Both used to
/// compare against `F32` directly, which was correct only while every program was
/// `f32`; a hard-coded width is exactly the check that keeps passing for the
/// wrong reason once a second one exists.
///
/// It answers for the *arithmetic*, not for every value a region touches: the
/// strict-affine decode loads `u8` codes and a `u8` zero point, and the only
/// arithmetic it performs after the exact widening is the `f32` multiply by its
/// scale. That is why it is not a boundary-carrier derivation — `boundary_carrier`
/// in the compiler is, and it refuses the decode rather than answering for it.
///
/// Exhaustive, so a new scalar program states its own width here instead of
/// inheriting whichever one it resembles. The copy arm answers for its element
/// format — a copy computes nothing, but the bits it moves are one format's,
/// and that format is what a signature or carrier derivation needs.
pub(super) const fn region_arithmetic_type(program: &RegionProgram) -> ArithmeticType {
    match program {
        RegionProgram::Numerical { scalar, .. } => scalar_arithmetic_type(scalar),
        RegionProgram::PartitionedCopy(copy) => match copy.element {
            CopyElement::F32 => ArithmeticType::F32,
        },
    }
}

/// The arithmetic type one scalar program's own operations are performed at.
///
/// The scalar half of [`region_arithmetic_type`], split out because the
/// accumulation-width and padding-identity gates compare against a *fold's*
/// width, and a fold exists only on the arithmetic arm.
pub(super) const fn scalar_arithmetic_type(program: &ScalarProgram) -> ArithmeticType {
    match program {
        ScalarProgram::PointwiseBf16(_) => ArithmeticType::Bf16,
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        | ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. }
        | ScalarProgram::StrictTensorContraction { .. }
        | ScalarProgram::StrictSerialMaximum { .. } => ArithmeticType::F32,
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
/// The accessor for a consumer that needs the tile alone, so it cannot
/// accidentally read a tile from some other variant. It is not the only match on
/// the topology: four sites in this crate destructure `CooperativeWorkgroup`
/// whole because they need its other fields in the same breath — `push_schedule`
/// below, `verify_cooperative_semantics` in the schedule builder, and
/// `verify_reduction` and `cooperative_plan` in `crate::kernel`.
#[must_use]
pub fn cooperative_tile(reduction: &ReductionTopology) -> Option<&CooperativeTile> {
    match reduction {
        ReductionTopology::CooperativeWorkgroup { tile, .. }
        | ReductionTopology::CooperativeContraction { tile, .. } => Some(tile),
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::MultiPass { .. }
        | ReductionTopology::Contraction { .. }
        | ReductionTopology::LiveContraction { .. } => None,
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

/// Returns the synchronization realization one reduction topology requires.
///
/// `None` for every topology that stages nothing across invocations, which is
/// the canonical absence: no requirement, no target query, no explain row, no
/// artifact field. A cooperative tile's answer is derived from the visibility
/// edges its own phases and staged accesses determine, so a producer cannot
/// widen or narrow it by editing the points it declared — the intrinsic verifier
/// requires every declared point to state exactly this subject.
#[must_use]
pub fn cooperative_synchronization_requirement(
    reduction: &ReductionTopology,
) -> Option<SynchronizationSubject> {
    let tile = cooperative_tile(reduction)?;
    required_subject(&tile.visibility_edges())
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

/// Returns the row-major suffix products of `shape`, one per axis.
///
/// Entry `k` is the product of every extent after axis `k`, which is the divisor
/// that extracts axis `k`'s coordinate from a row-major linear index. Returns
/// `None` when a product overflows `u64`, so a caller never silently compares
/// against a wrapped divisor.
fn suffix_products(shape: &Shape) -> Option<Vec<u64>> {
    let extents = shape.extents();
    let mut products = vec![1_u64; extents.len()];
    let mut running = 1_u64;
    for (position, extent) in extents.iter().enumerate().rev() {
        products[position] = running;
        running = running.checked_mul(extent.get())?;
    }
    Some(products)
}

/// Returns whether the decodes state a bijection of `result_shape`'s linear
/// coordinates onto `operand_shape`'s elements.
///
/// This is the bounds-proof obligation [`LogicalAccess::ReindexBijection`] owes,
/// and it discharges *both* halves at once. Every decoded coordinate is in range
/// structurally, because each is a `% modulus` whose modulus is that axis's own
/// extent; and every operand element is reached exactly once, because the
/// decodes with a nontrivial modulus are required to *tile* the linear
/// coordinate — sorted by descending divisor they must telescope, so the linear
/// index decomposes uniquely into them, exactly as a mixed-radix numeral does.
///
/// The telescoping check is what makes bijectivity a proven fact rather than a
/// producer's declaration. Mirroring is irrelevant to it: `c -> modulus − 1 − c`
/// is a bijection of any axis onto itself, so a mirrored decode tiles exactly
/// what its unmirrored twin does.
///
/// Extent-one operand axes are excluded from the tiling and carry no
/// information, which is correct — their only coordinate is zero — and canonical,
/// because [`AxisDecode::is_canonical`] pins their spelling.
#[must_use]
pub fn reindex_decodes_are_bijective(
    operand_shape: &Shape,
    result_shape: &Shape,
    axes: &[AxisDecode],
) -> bool {
    let extents = operand_shape.extents();
    if axes.len() != extents.len() {
        return false;
    }
    if axes
        .iter()
        .zip(extents)
        .any(|(decode, extent)| !decode.is_canonical() || decode.modulus != extent.get())
    {
        return false;
    }
    let (Ok(operand_elements), Ok(result_elements)) =
        (element_count(operand_shape), element_count(result_shape))
    else {
        return false;
    };
    // A bijection is onto, so the two domains have the same size. Checked before
    // the tiling because the tiling is stated against this total.
    if operand_elements != result_elements {
        return false;
    }
    let mut carrying: Vec<AxisDecode> = axes
        .iter()
        .copied()
        .filter(|decode| decode.modulus > 1)
        .collect();
    carrying.sort_unstable_by_key(|decode| std::cmp::Reverse(decode.divisor));
    let Some(first) = carrying.first() else {
        // No axis carries a coordinate, so the map is a bijection exactly when
        // both domains are the single element the empty product names.
        return operand_elements == 1;
    };
    if first.divisor.checked_mul(first.modulus) != Some(operand_elements) {
        return false;
    }
    for pair in carrying.windows(2) {
        let [higher, lower] = pair else {
            return false;
        };
        // Telescoping: the lower digit's window must end exactly where the
        // higher one begins. A gap would leave linear coordinates unreachable
        // and an overlap would make two of them collide.
        if lower.divisor.checked_mul(lower.modulus) != Some(higher.divisor) {
            return false;
        }
    }
    carrying.last().is_some_and(|decode| decode.divisor == 1)
}

/// Returns whether the decodes state a widening replication of `operand_shape`
/// across `result_shape`.
///
/// The bounds-proof obligation [`LogicalAccess::BroadcastReplication`] owes.
/// Every decoded coordinate is in range for the reason a reindex's is — the
/// modulus is the axis's own extent — but the covering requirement is
/// deliberately the opposite one: each operand axis reads *one whole result
/// axis* of equal extent, and the result axes no operand axis names are the
/// replicated ones the read is invariant in.
///
/// Three rules beyond that, each refusing a map that would otherwise be a second
/// spelling of a relation this vocabulary already has:
///
/// - **No two operand axes may read one result axis.** That would read one
///   result coordinate into two operand coordinates, which is a reindex-style
///   decode and not a broadcast.
/// - **No mirroring.** A broadcast replicates; reversing an axis is the reindex
///   family's business, and admitting it here would let one composition be
///   spelled two ways.
/// - **It must actually widen.** A replication covering the whole result domain
///   is [`LogicalAccess::LinearIdentity`], and admitting it here would give one
///   region two identities.
#[must_use]
pub fn broadcast_decodes_are_replicating(
    operand_shape: &Shape,
    result_shape: &Shape,
    axes: &[AxisDecode],
) -> bool {
    let extents = operand_shape.extents();
    if axes.len() != extents.len() {
        return false;
    }
    if axes.iter().zip(extents).any(|(decode, extent)| {
        !decode.is_canonical() || decode.modulus != extent.get() || decode.mirrored
    }) {
        return false;
    }
    let (Ok(operand_elements), Ok(result_elements)) =
        (element_count(operand_shape), element_count(result_shape))
    else {
        return false;
    };
    // Widening is what separates this relation from an identity read, and it is
    // checked on element counts rather than ranks: a rank that grew by an
    // extent-one axis widens nothing.
    if result_elements <= operand_elements {
        return false;
    }
    let Some(result_suffix) = suffix_products(result_shape) else {
        return false;
    };
    let result_extents = result_shape.extents();
    let mut claimed = vec![false; result_extents.len()];
    for decode in axes.iter().filter(|decode| decode.modulus > 1) {
        // The decode must name one whole result axis: its divisor is that axis's
        // suffix product and its modulus that axis's extent. Anything else is a
        // partial window, which this relation does not admit. Among axes of
        // extent above one the suffix products are strictly decreasing, so the
        // pair identifies exactly one axis.
        let named = result_suffix
            .iter()
            .zip(result_extents)
            .position(|(divisor, extent)| {
                *divisor == decode.divisor && extent.get() == decode.modulus
            });
        let Some(position) = named else {
            return false;
        };
        if std::mem::replace(&mut claimed[position], true) {
            return false;
        }
    }
    true
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

/// The index arithmetic every scheduled region's coordinate space requires.
///
/// A constant of *this* IR level rather than a value read from the kernel it
/// lowers to, because the fact belongs here: a [`ScheduledRegion`]'s coordinate
/// space is unsigned-64 throughout — [`Shape`] extents, [`element_count`],
/// [`KernelSchedule::work_items`], and [`LaunchPlan::grid_threads`] are all
/// `u64` — so every region's coordinate maps compute over the whole `u64` range
/// and none of them can require less. Reading the requirement out of
/// [`crate::kernel::KernelType`] instead would make the schedule layer depend on
/// its own lowering to state a property of its own coordinates.
///
/// The two levels are held together where they meet rather than by inspection:
/// `crate::kernel::model` asserts at compile time that this constant is exactly
/// what its crate-private KIR classifier derives for the governed index role,
/// so a lowering that changed the index type without changing this would stop
/// the build.
pub(crate) const REGION_INDEX_ARITHMETIC: IndexArithmetic = IndexArithmetic::CompleteU64;

/// Derives the resource requirements of a verified region.
///
/// Bindings follow the region's access count; the launch fixes the thread
/// count; local memory is the workgroup storage a cooperative tile allocates and
/// zero for every topology that stages nothing. The synchronization requirement
/// is derived from the *visibility edges* a cooperative tile carries rather than
/// from the points a producer declared, so it is one derivation the intrinsic
/// verifier has already proved every declared point equal to — and a region with
/// no edges derives `None`, which is an absence rather than a zero. The numerical
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
    // The numerical requirement is the region program's own class: an
    // arithmetic region carries every declared dimension forward, and a copy
    // region states the proved absence of a floating-point requirement rather
    // than fabricating strict rows for arithmetic no kernel performs.
    let numerical = match &region.index.program {
        RegionProgram::Numerical { numerical, .. } => RegionNumericalRequirements::FloatingPoint {
            input_subnormals: numerical.input_subnormals,
            result_subnormals: numerical.result_subnormals,
            contraction: numerical.contraction,
            reassociation: numerical.reassociation,
            permutation: numerical.permutation,
            signed_zero: numerical.signed_zero,
            reciprocal_transform: numerical.reciprocal_transform,
            approximate_intrinsics: numerical.approximate_intrinsics,
            nan_assumptions: numerical.nan_assumptions,
            infinity_assumptions: numerical.infinity_assumptions,
        },
        RegionProgram::PartitionedCopy(_) => RegionNumericalRequirements::BitPreservingCopy,
    };
    ResourceRequirements {
        buffer_bindings,
        threads_per_workgroup: region.schedule.threads_per_workgroup,
        local_memory_bytes: cooperative_local_memory_bytes(&region.schedule.reduction).unwrap_or(0),
        requires_device_memory: true,
        index_arithmetic: REGION_INDEX_ARITHMETIC,
        synchronization: cooperative_synchronization_requirement(&region.schedule.reduction),
        subgroup: None,
        numerical,
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
/// Logical-access tag of a reindex's bijective coordinate map.
///
/// Appended exactly as `0x05` was, and with the same injectivity argument:
/// `0x01` through `0x05` keep their tags and their field layouts, so no
/// previously encodable region's bytes move and the schedule identity domain
/// deliberately does not step. A reader that reaches `0x06` is reading an access
/// the earlier vocabulary could not express, never an earlier access under a new
/// interpretation.
const TAG_REINDEX_BIJECTION: u8 = 0x06;
/// Logical-access tag of a widening broadcast's replication map.
///
/// Appended for the same reason and with the same consequence as `0x06`. It is a
/// tag of its own rather than a field on `0x06` because the two relations differ
/// in what they permit a consumer to conclude, and a shared tag would make a
/// bijection and a replication differ only in bytes a reader has to interpret.
const TAG_BROADCAST_REPLICATION: u8 = 0x07;
/// Logical-access tag of a sourced parametric broadcast.
///
/// Appended for the same reason and with the same consequence as `0x07`. It is
/// a tag of its own rather than a field on `0x07` because a parametric
/// relation that may bind to one is not a concrete replication, and a shared
/// tag would make that distinction a payload interpretation. `0x01` through
/// `0x07` keep their tags and their field layouts, so no previously encodable
/// region's bytes move and the schedule identity domain deliberately does not
/// step.
const TAG_PARAMETRIC_BROADCAST: u8 = 0x08;
/// Logical-access tag of the live-inner-extent row-major **source** marker.
///
/// `0x09` was the retired contextual `LiveRowMajor { inner_axis }` relation the
/// accepted 2026-08-18 source-bound surface
/// (`decide-the-source-bound-live-row-major-access-surface`) replaced whole,
/// and it stays permanently unassigned: reusing it — for this marker, for the
/// consumer, or for any later map — would reinterpret every retained all-live
/// identity value under a changed payload. The source takes the fresh `0x0A`
/// that decision reserved for it, so a reader that reaches `0x0A` is reading
/// an access the earlier vocabulary could not express, never a retired access
/// under a new interpretation. Every live schedule identity value moves with
/// this replacement — the old five-byte `0x09` run becomes `0x0A` plus the
/// axis on the source and the bare `0x0B` on each consumer — while every
/// static region keeps its exact bytes, so the schedule identity domain
/// deliberately does not step.
const TAG_LIVE_ROW_MAJOR_SOURCE: u8 = 0x0A;
/// Logical-access tag of the fieldless live-inner-loop **consumer** marker.
///
/// The second fresh value the accepted 2026-08-18 source-bound surface
/// reserved. A tag of its own rather than a payload flag on `0x0A`, because
/// the source and a consumer are different relations — one declares the
/// region's runtime extent operand and the other consumes it — and one tag
/// would make that distinction a payload interpretation. The tag alone is the
/// whole encoding: the consumer is deliberately fieldless, so everything a
/// reader could ask of it is a derivation from the containing region's unique
/// `0x0A` marker. Sharing `0x0A` here instead would make a source and a
/// consumer indistinguishable at the byte level and is exactly the collision
/// the whole-vocabulary injectivity test refuses.
const TAG_LIVE_ROW_MAJOR_CONSUMER: u8 = 0x0B;
/// Logical-access tag of a partitioned copy's fieldless source map.
///
/// Appended at the next **free** value rather than at `0x0A`, and the gap is a
/// reconciliation across three same-day accepted decisions, not an oversight.
/// The accepted source-bound live-row-major surface
/// (`decide-the-source-bound-live-row-major-access-surface`, 2026-08-18)
/// assigned `0x0A` to `LiveRowMajorSource` and `0x0B` to its fieldless
/// consumer marker — both now written above — while retiring `0x09`; the
/// accepted data-dependent-index surface
/// (`decide-the-data-dependent-index-representation-public-surface`,
/// 2026-08-18) reserves `0x0C` for `GatherSource` and itself records that
/// "`0x0A` and `0x0B` remain reserved by the earlier live-row-major decision
/// packet … a gap is preferable to colliding reviewed identities". The
/// partitioned-copy packet, drafted before those reservations were visible to
/// it, named `0x0A`; taking it would collide two reviewed identity
/// assignments, so this tag takes `0x0D`, the next value no accepted record
/// claims. The injectivity argument is unchanged: `0x01`–`0x08`, `0x0A`, and
/// `0x0B` are the only bytes a `tiler.schedule.v7` region now writes at the
/// access-map position, `0x09` is retired-and-never-reused, `0x0C` is
/// reserved-and-unwritten at this base, so a reader that reaches `0x0D` is
/// reading an access the earlier vocabulary could not express and no
/// previously encodable region's bytes move.
const TAG_PARTITIONED_COPY_SOURCE: u8 = 0x0D;
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
/// Scalar-program tag of the strict serial `Maximum` reduction.
///
/// Appended for the same reason and with the same consequence as `0x26` and
/// `0x27`: `0x22` through `0x27` keep their meanings and their field positions,
/// so no previously encodable region's bytes move and the schedule identity
/// domain does not step. A reader that reaches `0x28` is reading a region the
/// earlier vocabulary could not express.
const TAG_SCALAR_SERIAL_MAXIMUM: u8 = 0x28;
/// Scalar-program tag of the physical `bf16` pointwise expression.
///
/// Appended for the same reason and with the same consequence as `0x26` through
/// `0x28`: `0x22` through `0x28` keep their meanings and their field positions,
/// so no previously encodable region's bytes move and the schedule identity
/// domain does not step. A reader that reaches `0x29` is reading a region the
/// earlier vocabulary could not express — no `f32` region can carry this tag,
/// because the variant it names holds a `bf16` expression.
///
/// Its node payloads are written by [`push_pointwise_bf16_node`], a separate
/// encoder from the `f32` one. The two node tag spaces overlap deliberately and
/// harmlessly: a node run is only ever read inside the scalar-program variant
/// that framed it, so `0x03` under `0x29` and `0x03` under `0x24` are never
/// reachable from one another. Sharing one encoder would instead couple two
/// vocabularies whose widenings are independent.
const TAG_SCALAR_POINTWISE_BF16: u8 = 0x29;
/// Scalar-program tag of the squared fold carrying a scalar epilogue.
///
/// Appended for the same reason and with the same consequence as `0x26` through
/// `0x29`: `0x22` through `0x29` keep their meanings and their field positions,
/// so no previously encodable region's bytes move and the schedule identity
/// domain does not step.
///
/// **Injectivity at this tag, in both directions.** The leading byte
/// discriminates, so no earlier program's bytes can be read as this one. Within
/// the tag the payload is the four fields `0x26` writes — a framed axis run, a
/// tag-per-order byte, and two fixed-width payloads — followed by the epilogue
/// written exactly as `TAG_SCALAR_POINTWISE_F32` writes an expression: a framed
/// node count, that many self-delimiting nodes, and the framed root ordinal. Each
/// field is therefore recoverable at a position the frames determine, so two
/// programs differing in *any* of them — including two epilogues differing only
/// in their root, or in one node's operand order — differ in these bytes. And
/// nothing else reaches them, so two programs equal in meaning encode
/// identically: the expression's own canonicalization gives one node order per
/// meaning, which is what makes the second direction hold rather than be hoped
/// for.
///
/// The node run cannot be confused with `0x24`'s: a run is read only inside the
/// scalar-program variant that framed it, exactly as the `bf16` node space is.
const TAG_SCALAR_SQUARED_SUM_EPILOGUE: u8 = 0x2A;
/// Program-position tag of the partitioned bit-preserving copy.
///
/// Appended after `TAG_SCALAR_SQUARED_SUM_EPILOGUE` (`0x2A`), and the schedule
/// identity domain deliberately does not step — the argument re-derived at
/// `tiler.schedule.v7` rather than carried from the packet's `v6` base:
///
/// 1. *Cross-arm discrimination.* Every field before the program position —
///    domain, framed shape, framed access list, framed proof list, fixed-width
///    ownership record — is framed and self-delimiting, so two encodings parse
///    to the same program-byte offset. At that offset every `v7`-encodable
///    region writes one of `0x22`–`0x2A`; `0x2B` is a byte no earlier region
///    can carry, so no old identity can equal a new one and no reader
///    reinterprets old bytes.
/// 2. *Within-arm recoverability.* The element tag and the four-byte axis are
///    fixed-width at fixed positions; the member run is length-framed with
///    fixed-width twelve-byte records, so every source ordinal and extent is
///    recoverable at a frame-determined position. Two copy programs differing
///    in element, axis, member count, any member's source, or any member's
///    extent differ in these bytes.
/// 3. *Equal meanings encode equally.* Offsets, source shapes, and destination
///    rectangles are derived and never written; the `partitioned-copy-
///    source-order` rule pins one read order per meaning; member order is
///    itself semantic. One program meaning therefore has exactly one encoding.
///
/// No numerical record follows this arm — the copy carries none — and the
/// schedule record follows directly, separated by the frame the member run
/// closes.
const TAG_REGION_PARTITIONED_COPY: u8 = 0x2B;
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
/// Reduction-topology tag of the operand-sharing cooperative contraction.
///
/// Appended exactly as `0x35` was. `0x36` is reserved for the accepted
/// [`ReductionTopology`] spelling `CooperativeContractionSplit` owned by
/// `decide-the-fixed-strided-contributor-membership-vocabulary` and is not
/// consumed here. A reader that reaches `0x37` is reading a region the earlier
/// vocabulary could not express; every earlier topology keeps its tag and field
/// positions, so no previously encodable region's bytes move and the schedule
/// identity domain does not step.
const TAG_REDUCTION_COOPERATIVE_CONTRACTION: u8 = 0x37;
/// Reduction-topology tag of a contraction whose contracted extent is live.
///
/// Appended after `0x37`. `0x36` stays reserved for
/// `CooperativeContractionSplit`; this tag is the next free slot. `0x01`
/// through `0x37` keep their tags and field positions, so no previously
/// encodable region's bytes move and the schedule identity domain does not
/// step. A reader that reaches `0x38` is reading a region the earlier
/// vocabulary could not express.
const TAG_REDUCTION_LIVE_CONTRACTION: u8 = 0x38;
/// Local coverage tag of an identity-padded contributor split.
///
/// Written only in the padded arm, after every field the earlier topology
/// payload already wrote. Exact coverage writes nothing here, so a previously
/// encodable exact split keeps its bytes. A reader that reaches `0x01` is
/// reading a padded split the earlier vocabulary could not express.
const TAG_COVERAGE_PADDED: u8 = 0x01;

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
/// Access identity is carried by the owning ordered access list and explicit
/// access references. This encoder records only the role category.
fn push_tensor_role(bytes: &mut Vec<u8>, role: TensorRole) {
    match role {
        TensorRole::Input => bytes.push(0x01),
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
        // Appended tags, whose payloads are framed exactly as the earlier
        // shape-carrying arms are: two framed shapes, then a framed run of
        // fixed-width decodes. Injective in both directions — every field of
        // every decode reaches the bytes, so two maps differing in meaning
        // differ here; and nothing but those fields does, so two maps equal in
        // meaning encode identically.
        LogicalAccess::ReindexBijection {
            operand_shape,
            result_shape,
            axes,
        } => {
            bytes.push(TAG_REINDEX_BIJECTION);
            push_shape(bytes, operand_shape);
            push_shape(bytes, result_shape);
            push_axis_decodes(bytes, axes);
        }
        LogicalAccess::BroadcastReplication {
            operand_shape,
            result_shape,
            axes,
        } => {
            bytes.push(TAG_BROADCAST_REPLICATION);
            push_shape(bytes, operand_shape);
            push_shape(bytes, result_shape);
            push_axis_decodes(bytes, axes);
        }
        LogicalAccess::ParametricBroadcast {
            operand_shape,
            mapping,
            environment,
        } => {
            bytes.push(TAG_PARAMETRIC_BROADCAST);
            operand_shape.encode(bytes);
            push_slice(bytes, mapping.canonical_encoding().as_bytes());
            push_slice(bytes, environment.as_bytes());
        }
        LogicalAccess::LiveRowMajorSource { inner_axis } => {
            bytes.push(TAG_LIVE_ROW_MAJOR_SOURCE);
            bytes.extend_from_slice(&inner_axis.get().to_be_bytes());
        }
        // Fieldless: the tag alone, because everything a reader could ask of a
        // consumer is a derivation from the region's unique `0x0A` marker. See
        // `TAG_LIVE_ROW_MAJOR_CONSUMER` for why it is not a payload of `0x0A`.
        LogicalAccess::LiveRowMajor => bytes.push(TAG_LIVE_ROW_MAJOR_CONSUMER),
        // Fieldless: the tag alone, because everything a reader could ask of
        // this map is a derivation from the region's copy program. See
        // `TAG_PARTITIONED_COPY_SOURCE` for the tag-value reconciliation.
        LogicalAccess::PartitionedCopySource => bytes.push(TAG_PARTITIONED_COPY_SOURCE),
    }
}

#[cfg(test)]
pub(super) fn push_logical_access_for_test(bytes: &mut Vec<u8>, access: &LogicalAccess) {
    push_logical_access(bytes, access);
}

/// Encodes one framed run of operand-axis coordinate decodes.
///
/// The count leads through [`push_len`] and exactly that many seventeen-byte
/// records follow, so the run's end is determined before it is read. Every field
/// is written — including the mirror flag, because a mirrored and an unmirrored
/// decode of the same axis are different access relations that must not share
/// identity.
fn push_axis_decodes(bytes: &mut Vec<u8>, axes: &[AxisDecode]) {
    push_len(bytes, axes.len());
    for decode in axes {
        bytes.extend_from_slice(&decode.divisor.to_be_bytes());
        bytes.extend_from_slice(&decode.modulus.to_be_bytes());
        bytes.push(u8::from(decode.mirrored));
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

/// Encodes one approximate-intrinsic accuracy envelope.
///
/// The match is exhaustive over a non-`#[non_exhaustive]` enum, so admitting a
/// third envelope is a build error here rather than an identity collision
/// between two regions that differ only in the approximation they authorize
/// (ADR 0076 item 6). Written locally rather than through
/// [`ApproximationEnvelope::tag`] to follow this record's convention: every
/// numerical field is written by its own local exhaustive encoder, so a change
/// to the vocabulary's own tag table cannot silently move schedule identity.
fn push_approximation_envelope(bytes: &mut Vec<u8>, envelope: ApproximationEnvelope) {
    bytes.push(match envelope {
        ApproximationEnvelope::Forbidden => 0x01,
        ApproximationEnvelope::BackendElementary => 0x02,
    });
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
    // Destructured exhaustively rather than field-accessed, so a widened
    // realization is a build error at this encoder instead of two semantically
    // different regions sharing one identity (ADR 0076 items 1 and 6).
    let NumericalRealization {
        profile_key,
        canonical_arithmetic_nan_bits,
        input_subnormals,
        result_subnormals,
        contraction,
        reassociation,
        permutation,
        signed_zero,
        reciprocal_transform,
        approximate_intrinsics,
        nan_assumptions,
        infinity_assumptions,
    } = *numerical;
    push_slice(bytes, profile_key.as_bytes());
    bytes.extend_from_slice(&canonical_arithmetic_nan_bits.to_be_bytes());
    push_subnormal(bytes, input_subnormals);
    push_subnormal(bytes, result_subnormals);
    push_permission(bytes, contraction);
    push_permission(bytes, reassociation);
    push_permission(bytes, permutation);
    push_permission(bytes, signed_zero);
    // The two elementary dimensions sit between the transform permissions and
    // the exceptional-value assumptions, in canonical dimension order. They are
    // written unconditionally: an optional row would let a strict region and a
    // region that never stated the dimension share bytes, which is the
    // compatibility default the accepted decision refuses.
    push_permission(bytes, reciprocal_transform);
    push_approximation_envelope(bytes, approximate_intrinsics);
    push_exceptional_assumption(bytes, nan_assumptions);
    push_exceptional_assumption(bytes, infinity_assumptions);
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
        // Appended for the reason `TAG_SCALAR_POINTWISE_BF16` records: the tag is
        // one no earlier region could carry, and the node run it frames is read
        // only inside it.
        ScalarProgram::PointwiseBf16(expression) => {
            bytes.push(TAG_SCALAR_POINTWISE_BF16);
            push_len(bytes, expression.nodes().len());
            for node in expression.nodes() {
                push_pointwise_bf16_node(bytes, node);
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
        // The fold's four fields exactly as `0x26` writes them, then the
        // epilogue in the framing `TAG_SCALAR_POINTWISE_F32` uses. See
        // `TAG_SCALAR_SQUARED_SUM_EPILOGUE` for the injectivity argument.
        ScalarProgram::SquaredSerialSumThenEpilogue {
            axes,
            order,
            canonical_nan_bits,
            empty_identity_bits,
            epilogue,
        } => {
            bytes.push(TAG_SCALAR_SQUARED_SUM_EPILOGUE);
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.extend_from_slice(&canonical_nan_bits.to_be_bytes());
            bytes.extend_from_slice(&empty_identity_bits.to_be_bytes());
            push_len(bytes, epilogue.nodes().len());
            for node in epilogue.nodes() {
                push_pointwise_f32_node(bytes, node);
            }
            push_slice(bytes, &epilogue.root().index().to_be_bytes());
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
        // Appended for the same reason as `0x26` and `0x27`, and — like the
        // contraction and unlike every sum — with no empty-domain identity to
        // encode: the extrema family has none, so the verifier refuses an empty
        // reduced domain rather than committing a value there.
        ScalarProgram::StrictSerialMaximum {
            axes,
            order,
            canonical_nan_bits,
        } => {
            bytes.push(TAG_SCALAR_SERIAL_MAXIMUM);
            push_axes(bytes, axes);
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
        // that field means: an access ordinal, a constant payload, or one
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
        PointwiseF32Node::Input { access } => {
            push_slice(bytes, &[0x01]);
            push_slice(bytes, &access.get().to_be_bytes());
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

/// Encodes one node of a physical `bf16` pointwise expression.
///
/// A **second, independent encoder** rather than a widened
/// [`push_pointwise_f32_node`], for the reason `push_tensor_role` is duplicated
/// between the schedule and kernel identity domains: the two node vocabularies
/// grow independently, and one encoder would make either vocabulary's widening a
/// change to the other's bytes. The tag values restart at `0x01` because a `bf16`
/// node run is only ever read inside the `TAG_SCALAR_POINTWISE_BF16` variant that
/// framed it, so no reader can confuse the two spaces.
///
/// The constant payload is two bytes, which is the whole reason a shared node
/// type was rejected: it is the `bf16` format's own width, so an over-wide
/// payload is unrepresentable rather than encodable-and-refused.
fn push_pointwise_bf16_node(bytes: &mut Vec<u8>, node: &PointwiseBf16Node) {
    const LENGTH_BYTES: usize = size_of::<u64>();
    const TAG_FIELD_BYTES: usize = LENGTH_BYTES + size_of::<u8>();
    const U16_FIELD_BYTES: usize = LENGTH_BYTES + size_of::<u16>();
    const U32_FIELD_BYTES: usize = LENGTH_BYTES + size_of::<u32>();

    let encoded_len = match node {
        PointwiseBf16Node::Input { .. } => TAG_FIELD_BYTES + U32_FIELD_BYTES,
        PointwiseBf16Node::Constant { .. } => TAG_FIELD_BYTES + U16_FIELD_BYTES,
        PointwiseBf16Node::Add { .. } | PointwiseBf16Node::Multiply { .. } => {
            TAG_FIELD_BYTES + 2 * U32_FIELD_BYTES
        }
    };
    push_len(bytes, encoded_len);
    let start = bytes.len();
    match node {
        PointwiseBf16Node::Input { access } => {
            push_slice(bytes, &[0x01]);
            push_slice(bytes, &access.get().to_be_bytes());
        }
        PointwiseBf16Node::Constant { bits } => {
            push_slice(bytes, &[0x02]);
            push_slice(bytes, &bits.to_be_bytes());
        }
        PointwiseBf16Node::Add { lhs, rhs } => {
            push_slice(bytes, &[0x03]);
            push_slice(bytes, &lhs.index().to_be_bytes());
            push_slice(bytes, &rhs.index().to_be_bytes());
        }
        PointwiseBf16Node::Multiply { lhs, rhs } => {
            push_slice(bytes, &[0x04]);
            push_slice(bytes, &lhs.index().to_be_bytes());
            push_slice(bytes, &rhs.index().to_be_bytes());
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

/// Encodes the shape one cooperative tile's participants occupy.
///
/// The rank is framed through [`push_len`] and exactly `rank` extents follow, so
/// the run's end is determined before it is read.
///
/// **The inline array's unused tail is deliberately not written.** Both this
/// encoder and [`push_staged_span`] frame the rank and the *used* elements
/// alone, which is what keeps identity injective on meaning in both directions:
/// two spaces that differ in meaning differ in these bytes, because the rank and
/// every used extent are all recoverable; and two spaces *equal* in meaning
/// encode identically, because nothing beyond the rank reaches the bytes. The
/// second direction is what a fixed-rank array would otherwise put at risk — a
/// rank-one space whose tail held stale values would encode differently from an
/// identical one whose tail held zeroes. `ParticipantSpace::new` zeroes the tail
/// so the derived `Eq` and `Hash` agree with this encoding as well, and the two
/// guarantees are separate: the constructor makes equality agree, and this
/// encoder makes identity agree, and neither substitutes for the other.
fn push_participant_space(bytes: &mut Vec<u8>, space: ParticipantSpace) {
    push_len(bytes, space.rank());
    for extent in space.extents() {
        bytes.extend_from_slice(&extent.to_be_bytes());
    }
}

/// Encodes the slots one participant addresses in one phase.
///
/// The strides lead, framed by their own rank, so a reader that has the
/// participant space already knows the run length before it reads one; `offset`
/// and `count` follow at positions the frame determines. The array's unused tail
/// is not written, for the reason [`push_participant_space`] states in full.
fn push_staged_span(bytes: &mut Vec<u8>, span: StagedSpan) {
    push_len(bytes, span.rank());
    for stride in span.strides() {
        bytes.extend_from_slice(&stride.to_be_bytes());
    }
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

/// Encodes the complete realization one synchronization point requires.
///
/// Six bytes: four tags and the two fence flags, in the field order
/// [`SynchronizationSubject`] declares. Every enumeration goes through its own
/// `tag` method, so widening one is a build error at that method rather than a
/// silent renumbering here.
fn push_synchronization_subject(bytes: &mut Vec<u8>, subject: SynchronizationSubject) {
    bytes.push(subject.kind.tag());
    bytes.push(subject.execution_scope.tag());
    bytes.push(subject.visibility_scope.tag());
    bytes.push(u8::from(subject.fenced_spaces.workgroup));
    bytes.push(u8::from(subject.fenced_spaces.device));
    bytes.push(subject.ordering.tag());
}

/// Encodes where in a cooperative tile one point sits.
///
/// A tag byte, then the ordinals the placement carries — two for a phase
/// boundary and none for a round boundary, whose separated phases are the tile's
/// own last and first. Written as an exhaustive match over a
/// non-`#[non_exhaustive]` enum so a widened placement vocabulary is a build
/// error here rather than a position encoded under another's bytes.
fn push_synchronization_placement(bytes: &mut Vec<u8>, placement: SynchronizationPlacement) {
    bytes.push(placement.tag());
    match placement {
        SynchronizationPlacement::PhaseBoundary {
            preceding,
            following,
        } => {
            bytes.extend_from_slice(&preceding.get().to_be_bytes());
            bytes.extend_from_slice(&following.get().to_be_bytes());
        }
        SynchronizationPlacement::RoundBoundary => {}
    }
}

/// Encodes one synchronization point of a cooperative tile.
///
/// The discharged edge set is deliberately *not* encoded, for the reason the
/// visibility edges are not: it is a total function of the placement and the
/// phases already written, so encoding it would add bytes no two distinguishable
/// tiles differ in. That covers both evidence classes — the anti-dependencies a
/// point discharges follow from the same placement and phases.
fn push_synchronization_point(bytes: &mut Vec<u8>, point: &SynchronizationPoint) {
    bytes.extend_from_slice(&point.id.get().to_be_bytes());
    push_synchronization_subject(bytes, point.subject);
    push_synchronization_placement(bytes, point.placement);
    push_participant_range(bytes, point.participants);
    bytes.push(point.convergence.tag());
}

/// Encodes one cooperative tile's complete cross-invocation dataflow.
///
/// The visibility edges are deliberately *not* encoded: they are a total
/// function of the phases and staged accesses already written here, so encoding
/// them would add bytes no two distinguishable tiles differ in — and would give
/// a producer a second place to state a fact the verifier derives.
///
/// The synchronization points *are* encoded, and the distinction is the whole
/// difference between a derivation and an authority: which boundary a producer
/// chose to order a handoff at is a physical decision with more than one legal
/// answer, so two tiles differing only in it are two schedules.
///
/// The round count is written beside the coordinates, before any staging or
/// phase record, because it scopes every one of them: a phase runs `rounds`
/// times and an allocation's declared lifetime is a within-round one. That
/// placement *inserts* into the payload rather than appending to it, and it is
/// what `tiler.schedule.v4` existed for — see [`encode_identity`] for why an
/// append there would not have been safe either, and for why the participant
/// space and stride vector this now writes forced `v5`.
fn push_cooperative_tile(bytes: &mut Vec<u8>, tile: &CooperativeTile) {
    let LocalCoordinates {
        source,
        participants,
    } = tile.coordinates;
    bytes.push(source.tag());
    push_participant_space(bytes, participants);
    bytes.extend_from_slice(&tile.rounds.to_be_bytes());
    push_len(bytes, tile.staging.len());
    for staging in &tile.staging {
        push_workgroup_staging(bytes, staging);
    }
    push_len(bytes, tile.phases.len());
    for phase in &tile.phases {
        push_cooperative_phase(bytes, phase);
    }
    push_len(bytes, tile.synchronization.len());
    for point in &tile.synchronization {
        push_synchronization_point(bytes, point);
    }
    push_participant_range(bytes, tile.commit);
}

/// Encodes the coverage suffix of one reduction topology.
///
/// Exact coverage is the implicit default of the earlier payload, so it writes
/// nothing. The padded arm appends a local tag and then the identity, and only
/// the identity: the partition has already been written in the position the
/// earlier encoding used.
fn push_coverage_suffix(bytes: &mut Vec<u8>, coverage: ContributorCoverage) {
    match coverage {
        ContributorCoverage::Exact(_) => {}
        ContributorCoverage::IdentityPadded { identity, .. } => {
            bytes.push(TAG_COVERAGE_PADDED);
            push_padding_identity(bytes, identity);
        }
    }
}

/// Encodes one padding identity as its arithmetic-type tag plus exact-width bits.
///
/// The variant determines both the tag and the payload width, so a format and
/// a bit width cannot disagree in these bytes. The tags are
/// [`ArithmeticType::tag`], which is why a `bf16` identity and an `f32` identity
/// cannot collide even when their low bits agree.
fn push_padding_identity(bytes: &mut Vec<u8>, identity: ReductionPaddingIdentity) {
    bytes.push(identity.arithmetic_type().tag());
    match identity {
        ReductionPaddingIdentity::F16(bits) | ReductionPaddingIdentity::Bf16(bits) => {
            bytes.extend_from_slice(&bits.to_be_bytes());
        }
        ReductionPaddingIdentity::F32(bits) => bytes.extend_from_slice(&bits.to_be_bytes()),
        ReductionPaddingIdentity::F64(bits) => bytes.extend_from_slice(&bits.to_be_bytes()),
    }
}

fn push_schedule(bytes: &mut Vec<u8>, schedule: &KernelSchedule) {
    match &schedule.binding {
        ExecutionBinding::GlobalLinearInvocation => bytes.push(0x01),
        // Appended binding tag. `0x01` keeps its meaning and every earlier
        // field keeps its position, so a region that still carries
        // `GlobalLinearInvocation` encodes the same bytes it did before this
        // arm existed. A reader that reaches `0x02` is reading a binding the
        // earlier vocabulary could not express.
        ExecutionBinding::BlockedWorkgroup { block, workgroups } => {
            bytes.push(0x02);
            push_shape(bytes, block);
            push_shape(bytes, workgroups);
        }
        // Appended binding tag, re-derived at `tiler.schedule.v6`: `0x01` and
        // `0x02` keep their meanings and every earlier field keeps its
        // position, so a region carrying either earlier binding encodes the
        // same bytes it did before this arm existed and the schedule identity
        // domain deliberately does not step. A reader that reaches `0x03` is
        // reading a binding the earlier vocabulary could not express, never an
        // earlier binding reinterpreted — old encodings carry only `0x01` or
        // `0x02` at this decode-determined position. The lane count is a
        // fixed-width big-endian `u64`, so the fields after the binding stay
        // at determined positions and two maps differing only in width differ
        // in these bytes; the `v4` counterexample (an append absorbed by a
        // following length-framed field) does not arise, because nothing
        // variable-length precedes the payload within this arm.
        ExecutionBinding::FixedVectorMap { lanes } => {
            bytes.push(0x03);
            bytes.extend_from_slice(&lanes.get().to_be_bytes());
        }
    }
    bytes.extend_from_slice(&schedule.work_items.to_be_bytes());
    bytes.extend_from_slice(&schedule.threads_per_workgroup.to_be_bytes());
    match schedule.tail {
        TailPolicy::Exact => bytes.push(0x01),
        // Appended tail tag. `0x01` keeps its meaning and every earlier field
        // keeps its position, so a region that still carries `Exact` encodes
        // the same bytes it did before this arm existed.
        TailPolicy::Predicated => bytes.push(0x02),
    }
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
            coverage,
            axes,
            order,
            accumulation,
            permits_reassociation,
            permits_permutation,
        } => {
            bytes.push(TAG_REDUCTION_MULTI_PASS);
            bytes.push(pass.tag());
            let partition = coverage.partition();
            bytes.extend_from_slice(&partition.partitions.to_be_bytes());
            bytes.extend_from_slice(&partition.contributors_per_partition.to_be_bytes());
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.push(accumulation.tag());
            bytes.push(u8::from(*permits_reassociation));
            bytes.push(u8::from(*permits_permutation));
            // Appended after every field the earlier `0x33` arm wrote, so an
            // exact-coverage region encodes the same bytes it did before this
            // suffix existed. The padded arm is a sequence no earlier region
            // could carry: the local tag and the identity sit where an earlier
            // reader would already have moved on to the launch record.
            push_coverage_suffix(bytes, *coverage);
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
        ReductionTopology::LiveContraction {
            live_access,
            live_axis,
            order,
            permits_reassociation,
            permits_permutation,
        } => {
            bytes.push(TAG_REDUCTION_LIVE_CONTRACTION);
            bytes.extend_from_slice(&live_access.get().to_be_bytes());
            bytes.extend_from_slice(&live_axis.get().to_be_bytes());
            push_order(bytes, *order);
            bytes.push(u8::from(*permits_reassociation));
            bytes.push(u8::from(*permits_permutation));
        }
        ReductionTopology::CooperativeWorkgroup {
            coverage,
            tile,
            axes,
            order,
            accumulation,
            permits_reassociation,
            permits_permutation,
            arrival,
        } => {
            bytes.push(TAG_REDUCTION_COOPERATIVE_WORKGROUP);
            let partition = coverage.partition();
            bytes.extend_from_slice(&partition.partitions.to_be_bytes());
            bytes.extend_from_slice(&partition.contributors_per_partition.to_be_bytes());
            push_cooperative_tile(bytes, tile);
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.push(accumulation.tag());
            bytes.push(u8::from(*permits_reassociation));
            bytes.push(u8::from(*permits_permutation));
            // At the end of the `0x35` arm rather than beside `accumulation`,
            // where it belongs by meaning. It was appended there while the arm
            // was young enough for that to move nothing a reader had; the `v4`
            // step has since made the position free, and it is left alone
            // because moving it would churn bytes for no gain.
            bytes.push(arrival.tag());
            // Coverage suffix after `arrival`, for the same reason the
            // multi-pass arm appends after its last earlier field: exact
            // encodings keep every previously written byte and the padded arm
            // is a sequence no earlier `0x35` region could carry.
            push_coverage_suffix(bytes, *coverage);
        }
        ReductionTopology::CooperativeContraction {
            tile,
            contracted_shape,
            contracted_tile,
            order,
            accumulation,
            permits_reassociation,
            permits_permutation,
        } => {
            bytes.push(TAG_REDUCTION_COOPERATIVE_CONTRACTION);
            push_cooperative_tile(bytes, tile);
            push_shape(bytes, contracted_shape);
            push_shape(bytes, contracted_tile);
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
/// # Why this is a `v7` step
///
/// `v7` gives the numerical-realization record its two elementary dimensions:
/// the reciprocal-transform permission and the approximate-intrinsic envelope,
/// written between the signed-zero permission and the exceptional-value
/// assumptions in canonical dimension order. The two bytes land *inside* the
/// realization record, which every following field of the region encoding
/// trails, so every region ever encoded maps to different bytes now — and an
/// append would not have been safe even at the record's end, because the
/// schedule payload continues after `push_numerical` and a `v6` reader handed
/// `v7` bytes would consume the reciprocal byte as the NaN-assumption tag and
/// lose framing for everything after it. Two regions that differ only in an
/// elementary dimension were also *one subject* under `v6`, which is exactly
/// the collision ADR 0076 item 6 exists to refuse: a cache or artifact holding
/// a `v6` identity must miss rather than match a region whose elementary
/// freedoms the earlier record could not state.
///
/// # Why this was a `v5` step
///
/// `v5` widens the cooperative staging relation to two dimensions (ADR 0097). A
/// tile's participants occupy a stated [`ParticipantSpace`] rather than a
/// contiguous range, and a [`StagedSpan`] carries one stride per participant
/// dimension rather than a single stride over a linear coordinate. Both changes
/// land *inside* records that repeat — the coordinates of every tile, and every
/// staged write and staged read of every phase — and both replace an unframed
/// fixed-width run with a length-framed one, so every cooperative region's bytes
/// move and no earlier reader keeps framing.
///
/// **An append was not available, and this time the encoding says so on its own
/// terms rather than by the argument `v4` had to make.** A stride vector is not
/// an added field beside a stride; it is a different relation in the same
/// position, so there is no position to append to. The framing is what makes the
/// widened form injective: the rank leads through `push_len`, exactly `rank`
/// eight-byte elements follow, and `offset` and `count` sit at positions the
/// frame determines — so no two spans differing in rank, strides, offset, or
/// count share bytes, and the inline array's unused tail never enters the
/// encoding at all, which is what keeps two spans *equal* in meaning from
/// differing in identity.
///
/// # Why this was a `v4` step
///
/// `v4` gave [`CooperativeTile`] its round count, so a tile can state that its
/// phase sequence repeats and that its staging is rewritten between rounds. The
/// field lands *inside* the `0x35` topology payload, ahead of the staging and
/// phase records, so every cooperative region's bytes move.
///
/// **The append that would have avoided it was not available, and the reason is
/// worth stating because the arm's earlier extensions did rely on it.** Both
/// `TAG_REDUCTION_COOPERATIVE_WORKGROUP` and the `arrival` byte after it were
/// justified by "no cooperative region has ever reached a retained identity", a
/// premise the single-workgroup tree strategy has since expired: a cooperative
/// region now lowers to a verified kernel, emits a checked-in Metal golden, and
/// folds into an artifact identity and a cache subject. Once bytes are retained,
/// the question stops being "does anything hold the old bytes" and becomes "can
/// an old identity equal a new one", and adding eight bytes anywhere inside this
/// arm does not answer it: the arm ends in `axes`, whose own length prefix and
/// four-byte elements can absorb the shift, so an old region with axes
/// `[0, 1, 2]` encodes the same bytes a new region with axes `[2]` and three
/// rounds does. Nothing in the *encoding* separates them; only the verifier's
/// requirement that the topology's axes repeat the access's does, and an
/// identity encoder that leans on a verifier invariant has stopped being
/// injective on its own terms. Stepping the domain is what restores that, and it
/// costs the retained corpus a miss rather than a wrong hit.
///
/// `v3` gave [`TensorRole::Input`] an ordinal and [`PointwiseF32Node::Input`]
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
    bytes.extend_from_slice(b"tiler.schedule.v7\0");
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
    // The program position: the `Numerical` arm writes byte-for-byte what the
    // two former required fields wrote — the scalar program, then the numerical
    // record — so every previously encodable region keeps its exact bytes; the
    // copy arm writes a sequence no earlier region could carry. See
    // `TAG_REGION_PARTITIONED_COPY` for the appended-tag injectivity argument.
    match &region.index.program {
        RegionProgram::Numerical { scalar, numerical } => {
            push_scalar_program(&mut bytes, scalar);
            push_numerical(&mut bytes, numerical);
        }
        RegionProgram::PartitionedCopy(program) => {
            push_partitioned_copy(&mut bytes, program);
        }
    }
    push_schedule(&mut bytes, &region.schedule);
    CanonicalScheduledRegionIdentity(bytes)
}

/// Encodes the partitioned-copy program arm.
///
/// In order: the program-position tag; one element tag byte via
/// [`CopyElement::tag`]; the copy axis as four big-endian bytes; one framed
/// member run — a [`push_len`] count, then exactly that many fixed-width
/// twelve-byte records of four-byte source ordinal plus eight-byte extent,
/// both big-endian. Derived quantities — offsets, source shapes, destination
/// rectangles — are deliberately never written: nothing beyond the semantic
/// fields reaches the bytes, so two programs equal in meaning cannot differ in
/// identity.
fn push_partitioned_copy(bytes: &mut Vec<u8>, program: &PartitionedCopyProgram) {
    bytes.push(TAG_REGION_PARTITIONED_COPY);
    bytes.push(program.element.tag());
    bytes.extend_from_slice(&program.axis.get().to_be_bytes());
    push_len(bytes, program.members.len());
    for member in &program.members {
        bytes.extend_from_slice(&member.source.get().to_be_bytes());
        bytes.extend_from_slice(&member.extent.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use crate::exhaustive_injectivity::{
        EXCEPTIONAL_ASSUMPTIONS, PERMISSIONS, SUBJECT_POPULATION, SUBNORMAL_MODES,
        assert_injective, assert_injective_fixed_width, every_synchronization_subject,
    };
    use crate::schedule::ExceptionalValueAssumption;

    use super::{
        ContributorCoverage, ContributorOrder, ContributorPartition, ReductionPaddingIdentity,
        TAG_COVERAGE_PADDED, push_coverage_suffix, push_exceptional_assumption, push_order,
        push_padding_identity, push_permission, push_subnormal, push_synchronization_subject,
    };

    /// The subject encoder is injective over its entire 648-value domain.
    ///
    /// **Exhaustive finite evidence, not a sample.** The domain is the product
    /// of five closed vocabularies — 6 construct kinds, 3 arrival scopes, 3
    /// publication scopes, 4 fences, 3 orderings — each enumerated in
    /// [`crate::exhaustive_injectivity`] from its own variant count, so "all
    /// 209 628 pairs are distinguished" is established by counting rather than
    /// argued from the encoder's shape.
    ///
    /// The subject is the atomic unit a target feasibility fact ranges over, so
    /// a collision here would let an authority answer for a realization it never
    /// attested to. That is a wrong plan, not a cache miss.
    #[test]
    fn the_synchronization_subject_encoding_is_injective_over_its_whole_domain() {
        let subjects = every_synchronization_subject();
        assert_eq!(
            SUBJECT_POPULATION, 648,
            "the subject domain changed size; the exhaustive claim this test makes is about \
             whatever it is now, so restate it deliberately"
        );
        assert_eq!(subjects.len(), SUBJECT_POPULATION);
        // Six bytes: four tags and the two fence flags. Fixed, so the following
        // field of a synchronization point cannot be shifted by the subject.
        assert_injective_fixed_width(&subjects, 6, push_synchronization_subject);
    }

    /// The subnormal encoder is injective over all three of its inhabitants.
    ///
    /// Exhaustive finite evidence. `SubnormalMode` has two variants, one
    /// carrying a two-inhabitant zero sign, so the domain is `1 + 2`. The flush
    /// sign is in the domain because two flushes producing different zeros are
    /// different realizations.
    #[test]
    fn the_subnormal_encoding_is_injective_over_its_whole_domain() {
        assert_eq!(SUBNORMAL_MODES.len(), 3);
        assert_injective_fixed_width(&SUBNORMAL_MODES, 1, push_subnormal);
    }

    /// The permission encoder is injective over both of its inhabitants.
    ///
    /// Exhaustive finite evidence, and the property that makes the encoder a
    /// tagged value rather than the derived `permits_*` boolean it used to be: a
    /// projection cannot fail closed when the projected enum grows, and this
    /// counts the tags rather than trusting that it will not.
    #[test]
    fn the_permission_encoding_is_injective_over_its_whole_domain() {
        assert_eq!(PERMISSIONS.len(), 2);
        assert_injective_fixed_width(&PERMISSIONS, 1, push_permission);
    }

    /// The exceptional-assumption encoder is injective over all four inhabitants.
    ///
    /// Exhaustive finite evidence over `1 + 3` values. The width is deliberately
    /// *not* fixed — `MakeNoAssumption` is one byte and `AssumeAbsent` is two —
    /// so the widths are asserted per variant instead. What makes the variable
    /// width safe inside the composite is that the one-byte encoding is a prefix
    /// of no two-byte one, which follows from the distinct leading tags this
    /// checks.
    #[test]
    fn the_exceptional_assumption_encoding_is_injective_over_its_whole_domain() {
        assert_eq!(EXCEPTIONAL_ASSUMPTIONS.len(), 4);
        for assumption in EXCEPTIONAL_ASSUMPTIONS {
            let mut bytes = Vec::new();
            push_exceptional_assumption(&mut bytes, assumption);
            let expected = match assumption {
                ExceptionalValueAssumption::MakeNoAssumption => 1,
                ExceptionalValueAssumption::AssumeAbsent { .. } => 2,
            };
            assert_eq!(bytes.len(), expected, "{assumption:?} changed width");
        }
        assert_injective(&EXCEPTIONAL_ASSUMPTIONS, push_exceptional_assumption);
    }

    /// The copy-element tag is injective over its one inhabitant.
    ///
    /// Exhaustive finite evidence over a domain of size one, exactly as the
    /// contributor-order claim below: the population assertion is what fails —
    /// deliberately — on the day a second element format lands and its tag and
    /// storage width must be stated rather than inherited.
    #[test]
    fn the_copy_element_tag_is_injective_over_its_whole_domain() {
        const ELEMENTS: [super::CopyElement; std::mem::variant_count::<super::CopyElement>()] =
            [super::CopyElement::F32];

        assert_eq!(ELEMENTS.len(), 1);
        assert_injective_fixed_width(&ELEMENTS, 1, |bytes, element: super::CopyElement| {
            bytes.push(element.tag());
        });
        assert_eq!(super::CopyElement::F32.tag(), 0x01);
        assert_eq!(super::CopyElement::F32.storage_bytes(), 4);
    }

    /// The contributor-order encoder is injective over its one inhabitant.
    ///
    /// Exhaustive finite evidence over a domain of size one, which is a real if
    /// unexciting claim: a one-value domain cannot collide, and the assertion
    /// that the population is one is what will fail — deliberately — on the day
    /// a second contributor order lands and the constant `0x01` stops being a
    /// function of the value.
    #[test]
    fn the_contributor_order_encoding_is_injective_over_its_whole_domain() {
        const ORDERS: [ContributorOrder; std::mem::variant_count::<ContributorOrder>()] =
            [ContributorOrder::OriginalAxisLexicographic];

        assert_eq!(ORDERS.len(), 1);
        assert_injective_fixed_width(&ORDERS, 1, push_order);
    }

    /// Exact coverage writes no suffix; the padded arm is an appended local tag
    /// plus a width-discriminated identity.
    #[test]
    fn exact_coverage_writes_no_suffix_and_padded_encodings_are_injective() {
        let split = ContributorPartition {
            partitions: 3,
            contributors_per_partition: 2,
        };
        let mut exact = Vec::new();
        push_coverage_suffix(&mut exact, ContributorCoverage::Exact(split));
        assert_eq!(exact, [], "exact coverage is the implicit default");

        let identities = [
            ReductionPaddingIdentity::F16(0x8000),
            ReductionPaddingIdentity::Bf16(0x8000),
            ReductionPaddingIdentity::F32(0x8000_0000),
            ReductionPaddingIdentity::F32(0x0000_0000),
            ReductionPaddingIdentity::F64(0x8000_0000_0000_0000),
        ];
        let mut seen = Vec::new();
        for identity in identities {
            let mut bytes = Vec::new();
            push_coverage_suffix(
                &mut bytes,
                ContributorCoverage::IdentityPadded {
                    partition: split,
                    identity,
                },
            );
            assert_eq!(bytes[0], TAG_COVERAGE_PADDED);
            assert!(
                !seen.contains(&bytes),
                "{identity:?} collided with an earlier padding identity"
            );
            seen.push(bytes);
        }
        assert_injective(&identities, push_padding_identity);
    }
}
