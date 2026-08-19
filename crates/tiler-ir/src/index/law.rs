//! Closed, typed semantic laws for canonical logical index realization.
//!
//! A law is registered by the same semantic-provider transaction that defines
//! an operation. It is data, not a verdict callback: the verifier interprets it
//! without exposing the candidate, builds the expected canonical region
//! *sequence*, and compares the two only after both have passed ordinary
//! structural checks.
//!
//! The compared value is the sequence identity, not one region's. A law whose
//! realization is a single region answers a one-stage sequence whose identity is
//! that region's identity byte for byte, so the comparison this module drives is
//! the same one it always drove for those laws; a law whose realization is a
//! chain is compared whole, and stages that are individually correct but ordered
//! or wired differently render a different sequence identity.

use core::fmt;
use std::error::Error;

use crate::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, F32NumericalContractKey,
    MaterializationRounding, NumericalPermission, SubnormalMode,
};
use crate::semantic::{
    AttributeFieldId, BF16_CONSTANT_BITS_ATTRIBUTE, BROADCAST_AXIS_MAPPING_ATTRIBUTE,
    BroadcastAxisMapping, BroadcastAxisSource, CONCATENATE_AXIS_ATTRIBUTE,
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalField, CanonicalIntegerWidth, CanonicalValue,
    CanonicalValueView, ContractionIndex, ContractionIndexStructure, EncodedComponentRole,
    F32_CONSTANT_BITS_ATTRIBUTE, OperationAttributes, REDUCTION_AXES_ATTRIBUTE,
    REINDEX_MAPPING_ATTRIBUTE, RMS_NORM_EPS_BITS_ATTRIBUTE, RMS_NORM_REDUCED_AXES_ATTRIBUTE,
    ReindexForm, ReindexFormKind, ResolvedValueType, SLICE_SELECTION_ATTRIBUTE,
    SOFTMAX_REDUCED_AXES_ATTRIBUTE, STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE,
    STRICT_AFFINE_ZERO_POINT_ROLE, SliceAxisSelection, SliceSelection, StrictAffineU4, TypeKey,
    concatenate_axis, concatenate_result_shape,
};
use crate::shape::{Axis, Extent, ExtentSources, Shape, SourcedExtent};

use super::{
    DimensionId, DomainRole, FrozenScalarRegistry, IndexBuildError, IndexExprId, IndexInteger,
    IndexRefinementBoundary, IndexRefinementSubject, IndexRegionBuildError, IndexRegionBuilder,
    IndexRegionSequenceError, ScalarAttributes, ScalarOpKey, ScalarReducerBodyBuilder,
    ScalarValueId, SourcedIndexInteger, StagedInputSource, SymbolicExtentError, TensorAccessId,
    TensorId, TensorRole, VerifiedIndexRegion, VerifiedIndexRegionSequence, add_bf16_scalar_op,
    add_f32_scalar_op, canonicalize_nan_f32_scalar_op, constant_bf16_scalar_op,
    constant_f32_scalar_op, divide_f32_scalar_op, exp_f32_scalar_op, maximum_f32_scalar_op,
    multiply_bf16_scalar_op, multiply_f32_scalar_op, rsqrt_f32_scalar_op,
    strict_affine_u4_dequantize_scalar_op,
};

/// Exact binary32 payload of `1.0`, the numerator of the softmax's reciprocal.
const F32_ONE_BITS: u32 = 0x3f80_0000;
/// Exact binary32 payload of `-1.0`, the softmax shift's exact sign flip.
const F32_NEGATIVE_ONE_BITS: u32 = 0xbf80_0000;

/// A bounded semantic template for one canonical logical index realization.
///
/// This is deliberately not a universal IR. Each variant is an atomic template
/// whose complete interpretation is owned here; semantics outside this set are
/// unsupported and therefore cannot mint refinement evidence. Verification
/// requires the candidate region's exact canonical identity to equal the region
/// this law constructs. A semantically equivalent alternate logical index form
/// is deliberately refused; physical alternatives belong to later planning.
///
/// # Stating a refusal no construction path reaches
///
/// **A law may state a refusal rule no current producer can reach, when a
/// subject could present the refused form.** A law is interpreted against an
/// [`IndexRefinementSubject`], never against the inferencer that produced one.
/// [`IndexRefinementSubject::derive`] is that subject's only producer today, and
/// it builds one from the family's own inferencer — so a family rule refusing a
/// malformed occurrence refuses it before any subject exists. That is a fact
/// about the current producer rather than about the vocabulary a law reads: the
/// refused form is expressible in the subject's own types, so a subject re-read
/// from durable bytes, hand-built by a later producer, or derived for a family
/// registered afterwards can present it, and the law is what answers. Stating
/// the rule is what makes that answer a named refusal instead of a realization
/// of something this law does not mean. That is a **reinterpretation boundary**,
/// and it is the ground such a rule stands on.
///
/// **It does not extend to a refusal nothing can reach.** Where the checked
/// value comes from a total function, no subject however built denotes the
/// refused state, so the check can never be watched failing and stating it
/// claims a maturity no evidence supports. The rejected case is the mixed-width
/// refusal proposed for the `bf16` reference: `region_arithmetic_type` maps
/// every `ScalarProgram` to one `ArithmeticType`, so no constructible program
/// could ever fire it.
///
/// One question separates the two: **can a subject reach this rule by any route,
/// including one no current producer takes?** If yes, state it. If no, it is not
/// a check and does not belong in a law.
///
/// **Nothing here relaxes the discipline for a *reachable* refusal**, which is
/// still watched failing before it is trusted. The exception is this one class,
/// and each member of it says at its own site that it is unreachable and why it
/// is stated anyway, so the reason travels with the rule. The four members are
/// `softmax-reduced-axis-rank` in `realize_softmax`, and
/// `concatenate-result-arity`, `concatenate-result-shape`, and
/// `concatenate-operand-binding` in `realize_concatenate`,
/// `ConcatenatePlan::derive`, and `emit_partitioned_concatenate`.
///
/// The convention is stated for the realization-law vocabulary. Whether it
/// reaches any other vocabulary is undecided, and nothing here decides it.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum IndexRealizationLaw {
    /// Rank-zero constant from one float-bits semantic attribute.
    ConstantFromFloatBits {
        /// Attribute holding the exact payload.
        attribute: AttributeFieldId,
        /// Scalar constant operation receiving that payload.
        scalar: ScalarOpKey,
    },
    /// Binary pointwise scalar application with rank-zero broadcasting only.
    PointwiseBinary {
        /// Scalar operation applied to the two ordered operands.
        scalar: ScalarOpKey,
    },
    /// The governed precise `x / (1 + exp(-x))` scalar chain.
    PreciseSiluF32,
    /// Lexicographic left fold over axes named by one semantic attribute.
    StrictSerialSumF32 {
        /// Attribute containing the ordered reduction axes.
        axes_attribute: AttributeFieldId,
    },
    /// Payload-preserving coordinate transform from a reindex attribute.
    Reindex {
        /// Attribute containing the canonical reindex form.
        mapping_attribute: AttributeFieldId,
    },
    /// Payload-preserving coordinate map from a broadcast attribute.
    Broadcast {
        /// Attribute containing the canonical broadcast map.
        mapping_attribute: AttributeFieldId,
    },
    /// Payload-preserving literal-offset sub-tensor selection.
    ///
    /// One result dimension exists for every operand dimension. A whole-axis
    /// selection reads operand coordinate `d`; a literal window reads `d +
    /// offset`, where `offset` is the exact `u64` payload in the named semantic
    /// attribute. The write is identity over the result domain. This law
    /// therefore states the access relation only: it chooses neither a view nor
    /// a copy and reaches no scalar operation.
    ///
    /// **Included and excluded surface.** The variant reads the complete
    /// [`SliceSelection`] grammar admitted today: [`SliceAxisSelection::WholeAxis`]
    /// and [`SliceAxisSelection::Window`] with a literal [`SourcedExtent::Static`]
    /// offset and extent. It deliberately excludes strided windows, source-bearing
    /// symbolic offsets, scheduling, and backend realization. A symbolic window
    /// offset is a typed unsupported subject here; semantic construction may
    /// admit it, but this law does not refine it.
    ///
    /// **Accepted public surface.** Tom accepted the exact variant, its `const`
    /// constructor, append-only tag-13 encoding, and standard
    /// `tiler::slice-f32@1` revision-1 registration on 2026-08-11 under
    /// [`accept-the-literal-offset-slice-realization-law`]. The acceptance is
    /// limited to the included and excluded surface above; it grants no strided
    /// or symbolic selection, storage choice, schedule, or backend realization.
    ///
    /// [`accept-the-literal-offset-slice-realization-law`]: ../../../../tickets/accept-the-literal-offset-slice-realization-law.md
    Slice {
        /// Attribute containing the canonical literal selection.
        selection_attribute: AttributeFieldId,
    },
    /// Strict contraction over an explicit index-structure attribute.
    StrictTensorContractionF32 {
        /// Attribute containing the canonical contraction structure.
        structure_attribute: AttributeFieldId,
    },
    /// A strict serial sum whose materialized result a pointwise pass consumes.
    ///
    /// The first law form whose realization is an ordered *sequence* of regions
    /// rather than one region. Stage zero folds operand zero over the axes the
    /// named attribute carries and publishes the fold; stage one applies `scalar`
    /// to operand one and that published value, pointwise over the result shape.
    ///
    /// **Why the sequence is the law's shape and not a physical choice.** The
    /// intermediate is *read more than once* — every point of stage one consumes
    /// the same fold — so a single-region spelling would either recompute the
    /// fold per point, which is a different scalar program, or need a value with
    /// no region-local definition. The materialization is therefore part of what
    /// the realization means, and the region sequence is where it is stated.
    /// Which memory the intermediate occupies, and whether a target can keep it
    /// in registers, remain physical planning's questions.
    ///
    /// This is the reduction-then-elementwise *shape*, and it is deliberately not
    /// the normalization's own law. `tiler::rms-norm-f32@1` folds a square rather
    /// than the operand's own elements, transforms the published fold three times
    /// before the pass consumes it, and reads three values in that pass; this form
    /// expresses none of the three, so the normalization carries
    /// [`Self::StagedRootMeanSquareScaleF32`] instead. What the two share is the
    /// staged emitters below, not a template.
    StagedStrictSerialSumThenPointwiseF32 {
        /// Attribute containing the ordered axes stage zero folds over.
        axes_attribute: AttributeFieldId,
        /// Scalar operation stage one applies to operand one and the fold.
        scalar: ScalarOpKey,
    },
    /// The governed root-mean-square scale: a squared fold, an epilogue, a weight.
    ///
    /// Stage zero folds the *square* of operand zero over the axes the named
    /// attribute carries, divides that fold by the folded contributor count,
    /// adds the exact payload the second named attribute carries, applies the
    /// reciprocal square root, and publishes the result. Stage one reads operand
    /// zero, operand one, and that published value, and writes
    /// `weight * (value * published)` pointwise over the result shape.
    ///
    /// **Why the chain is fixed and only the two attributes are law data.** Every
    /// other shape was eliminated against this vocabulary's own contract. Carrying
    /// the epilogue and the pass as *data* would need a scalar-program language
    /// inside a law, which is the universal IR this module's header refuses; and a
    /// template with five independently settable scalar keys is one whose complete
    /// interpretation is no longer owned here, because most of its 5-tuples denote
    /// programs nothing means. Fixing the chain and naming the attributes is the
    /// same split [`Self::ConstantFromFloatBits`] draws: what a second row of one
    /// template varies is its record-local field identifiers, which are
    /// record-local precisely because two families number them alike.
    ///
    /// **What generalizes instead is the emission machinery.** A fold carrying a
    /// per-contributor square, a fold whose value an epilogue transforms inside
    /// the producing region, and a pass reading a reduced-rank published value at
    /// its kept coordinates are three capabilities the staged vocabulary did not
    /// have. They are stated as reusable emitters, so the next staged family
    /// instantiates them rather than this template.
    StagedRootMeanSquareScaleF32 {
        /// Attribute containing the ordered axes stage zero folds over.
        axes_attribute: AttributeFieldId,
        /// Attribute containing the exact bias added inside the root's argument.
        ///
        /// Named rather than assumed, and checked against the occurrence's exact
        /// declared field set: the payload is part of the semantic operation's
        /// identity, so a realization that read the axes and ignored this would be
        /// a different operation wearing this one's law.
        eps_attribute: AttributeFieldId,
    },
    /// The governed softmax: an extrema fold, a shifted exponential, a sum fold,
    /// and a reciprocal scale, over one reduced axis.
    ///
    /// Stage zero folds operand zero with the NaN-propagating maximum family over
    /// the axis the named attribute carries and publishes the row maximum `m`.
    /// Stage one reads operand zero at its own coordinates and `m` at the kept
    /// coordinates and publishes the exponentials `e_i = Exp(s_i - m)`. Stage two
    /// folds `e` with the governed addition and publishes the denominator `d`.
    /// Stage three reads `e` **again**, at its own coordinates, and `d` at the
    /// kept coordinates, and writes `e_i * (1.0 / d)` — one division per folded
    /// row and one multiplication per point.
    ///
    /// **Why four stages and not fewer.** `e` is read by stage two and by stage
    /// three, and a value with more than one reader is exactly what the region
    /// sequence's retention contract expresses; folding the denominator inside
    /// stage three would recompute `e` once per point of its own row, which is a
    /// different scalar program. Fusing the two folds into one pass is the online
    /// single-pass form, which
    /// [`SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM`](crate::semantic::SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM)
    /// records as consuming distributivity and the exponential's own functional
    /// equation, neither of which any permission grants.
    ///
    /// **Why the chain is fixed and only the axes attribute is law data.** The
    /// same split [`Self::StagedRootMeanSquareScaleF32`] draws, for the same
    /// reason: carrying the shift, the exponential, the reciprocal, and the scale
    /// as *data* would need a scalar-program language inside a law, which this
    /// module's header refuses. This is the second family whose chain the template
    /// names rather than carries, and the emission machinery is where the
    /// generality went — an identity-less fold parameterized by its combiner, and
    /// a stage that reads a reduced-rank published value at its kept coordinates
    /// and runs arbitrary scalar work between the read and the write, are stated
    /// as reusable emitters that this template and the normalization's both
    /// instantiate.
    ///
    /// **The empty reduced axis is refused rather than realized.** The extrema
    /// fold has no identity and is seeded at the first contributor, which a
    /// zero-length axis does not have. The operation is shape-preserving, so its
    /// own semantics evaluate no scalar softmax there
    /// ([`SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS`](crate::semantic::SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS)),
    /// while this staged shape would still have to commit one row maximum per
    /// kept coordinate — so the realization refuses by name instead of inventing a
    /// seed the reference does not have.
    ///
    /// **Accepted public surface.** Tom accepted the exact variant, its `const`
    /// constructor, append-only tag-11 encoding, and standard
    /// `tiler::softmax-f32@1` registration on 2026-08-07 under
    /// [`accept-the-softmax-realization-law`]. The acceptance is limited to this
    /// staged law; the three general emitters stay private, and it grants no
    /// governed lowering or scheduled-stage spelling.
    ///
    /// [`accept-the-softmax-realization-law`]: ../../../../tickets/accept-the-softmax-realization-law.md
    StagedSoftmaxF32 {
        /// Attribute containing the single reduced axis, as a one-element sequence.
        axes_attribute: AttributeFieldId,
    },
    /// Payload-preserving join of an ordered operand sequence along one axis,
    /// realized as a partitioned write into one output.
    ///
    /// One write root per operand over the single output value. Root *k* iterates
    /// its own dimension on the concatenated axis — extent the operand's own —
    /// together with the dimensions the non-concatenated axes share, reads its
    /// operand at those coordinates unchanged, and writes at `t + offset_k` on
    /// the concatenated axis, where `offset_k` is the sum of the preceding
    /// operands' extents there. The roots' rectangles tile the output, which the
    /// joint partition obligation decides by interval reasoning.
    ///
    /// **Why several iteration domains and not one partitioned by coordinate.**
    /// The write-domain contract admits a write domain that is any *subset* of
    /// the region's parallel dimensions, and eliminated the sub-range annotation
    /// that would have let one shared domain be cut by coordinate; the region's
    /// parallel set is the union of its roots' domains. A concatenation of
    /// unequally sized operands has no spelling under one shared domain at all,
    /// because every root of one domain owns the same element count — which is
    /// exactly why the pinned `[8, 0, 128]`-with-`[8, T, 128]` occurrence forced
    /// that relaxation. This template therefore states the answer that contract
    /// fixed rather than inventing a second one.
    ///
    /// **Why the non-concatenated axes share one dimension each.** The family
    /// admits an occurrence only when every operand agrees on those axes, so one
    /// dimension per such axis is the region's own statement of that agreement.
    /// Declaring a private copy per root would put `n · (rank − 1)` dimensions
    /// into the canonical identity that are pairwise equal by construction, which
    /// is one meaning under several spellings.
    ///
    /// **The emitted scalar program is empty, deliberately.** Every result
    /// element is an operand element unchanged, so the value written is the value
    /// read and no scalar authority is reached — the same reason
    /// [`Self::Reindex`] applies no scalar operation.
    ///
    /// **A zero-extent operand is a member with an empty rectangle, not a
    /// coverage hole.** Its own dimension has extent zero, so its root visits no
    /// point, writes no element, and contributes zero volume — which is what the
    /// joint obligation's volume identity says about a root over an empty domain.
    ///
    /// **Accepted public surface.** Tom accepted the exact variant, its `const`
    /// constructor, append-only tag-12 encoding, and standard
    /// `tiler::concatenate-f32@1` registration on 2026-08-07 under
    /// [`accept-the-partitioned-concatenate-realization-law`]. The acceptance is
    /// limited to this single-region law; the emission helpers stay private, and
    /// it grants no scheduled-region spelling, kernel construct, or backend
    /// realization.
    ///
    /// [`accept-the-partitioned-concatenate-realization-law`]: ../../../../tickets/accept-the-partitioned-concatenate-realization-law.md
    PartitionedConcatenate {
        /// Attribute containing the concatenated axis.
        axis_attribute: AttributeFieldId,
    },
    /// Per-point decode of one governed compound strict-affine U4 value.
    StrictAffineU4Dequantize {
        /// Ordered logical codes component role.
        codes_role: EncodedComponentRole,
        /// Ordered positive-normal scale component role.
        scale_role: EncodedComponentRole,
        /// Ordered logical zero-point component role.
        zero_point_role: EncodedComponentRole,
        /// Atomic scalar operation implementing the widened strict decode.
        scalar: ScalarOpKey,
    },
}

impl IndexRealizationLaw {
    pub(crate) fn accepts_numerical_contract(&self, subject: &IndexRefinementSubject) -> bool {
        match self {
            Self::StrictAffineU4Dequantize { .. } => {
                let strict = F32NumericalContractKey::new(
                    SubnormalMode::Preserve,
                    SubnormalMode::Preserve,
                    NumericalPermission::Forbidden,
                    NumericalPermission::Forbidden,
                    NumericalPermission::Forbidden,
                    NumericalPermission::Forbidden,
                    NumericalPermission::Forbidden,
                    ApproximationEnvelope::Forbidden,
                    ExceptionalValueAssumption::MakeNoAssumption,
                    ExceptionalValueAssumption::MakeNoAssumption,
                    MaterializationRounding::NearestTiesToEven,
                )
                .expect("the governed strict F32 contract is coherent");
                subject.numerical_contract().as_str() == strict.as_str()
            }
            Self::ConstantFromFloatBits { .. }
            | Self::PointwiseBinary { .. }
            | Self::PreciseSiluF32
            | Self::StrictSerialSumF32 { .. }
            | Self::Reindex { .. }
            | Self::Broadcast { .. }
            | Self::Slice { .. }
            | Self::StagedStrictSerialSumThenPointwiseF32 { .. }
            | Self::StagedRootMeanSquareScaleF32 { .. }
            | Self::StagedSoftmaxF32 { .. }
            | Self::PartitionedConcatenate { .. }
            | Self::StrictTensorContractionF32 { .. } => governs_result_arithmetic(subject),
        }
    }

    /// Standard constant-f32 law.
    #[must_use]
    pub fn constant_f32() -> Self {
        Self::ConstantFromFloatBits {
            attribute: F32_CONSTANT_BITS_ATTRIBUTE,
            scalar: constant_f32_scalar_op(),
        }
    }

    /// Standard multiply-f32 law.
    #[must_use]
    pub fn multiply_f32() -> Self {
        Self::PointwiseBinary {
            scalar: multiply_f32_scalar_op(),
        }
    }

    /// Standard add-f32 law.
    #[must_use]
    pub fn add_f32() -> Self {
        Self::PointwiseBinary {
            scalar: add_f32_scalar_op(),
        }
    }

    /// Standard constant-bf16 law.
    ///
    /// The same template the `f32` constant uses, carrying this family's own
    /// attribute and its own scalar. Neither the template nor its encoding tag
    /// is new: what distinguishes the two rows is the payload they carry, which
    /// is exactly the distinction the semantic keys already draw.
    #[must_use]
    pub fn constant_bf16() -> Self {
        Self::ConstantFromFloatBits {
            attribute: BF16_CONSTANT_BITS_ATTRIBUTE,
            scalar: constant_bf16_scalar_op(),
        }
    }

    /// Standard multiply-bf16 law.
    #[must_use]
    pub fn multiply_bf16() -> Self {
        Self::PointwiseBinary {
            scalar: multiply_bf16_scalar_op(),
        }
    }

    /// Standard add-bf16 law.
    #[must_use]
    pub fn add_bf16() -> Self {
        Self::PointwiseBinary {
            scalar: add_bf16_scalar_op(),
        }
    }

    /// Standard strict-serial-sum-f32 law.
    #[must_use]
    pub const fn strict_serial_sum_f32() -> Self {
        Self::StrictSerialSumF32 {
            axes_attribute: REDUCTION_AXES_ATTRIBUTE,
        }
    }

    /// Standard reindex-f32 law.
    #[must_use]
    pub const fn reindex_f32() -> Self {
        Self::Reindex {
            mapping_attribute: REINDEX_MAPPING_ATTRIBUTE,
        }
    }

    /// Standard broadcast-f32 law.
    #[must_use]
    pub const fn broadcast_f32() -> Self {
        Self::Broadcast {
            mapping_attribute: BROADCAST_AXIS_MAPPING_ATTRIBUTE,
        }
    }

    /// Standard slice-f32 law.
    ///
    /// Realizes a literal window as `d + offset` and a source-bearing window as
    /// `t + C` through the sourced addend vocabulary. The encoded row is still
    /// tag 13 over the record-local selection field; only the interpretation
    /// grew.
    #[must_use]
    pub const fn slice_f32() -> Self {
        Self::Slice {
            selection_attribute: SLICE_SELECTION_ATTRIBUTE,
        }
    }

    /// Standard concatenate-f32 law, as `tiler::concatenate-f32@1` registers it.
    ///
    /// Names that family's own axis identifier. It is record-local — the reindex
    /// and broadcast records number their own single field the same way — so this
    /// constructor is what ties the general template to the one family whose
    /// record means this field.
    #[must_use]
    pub const fn concatenate_f32() -> Self {
        Self::PartitionedConcatenate {
            axis_attribute: CONCATENATE_AXIS_ATTRIBUTE,
        }
    }

    /// Standard tensor-contraction-f32 law.
    ///
    /// The realization it states is the strict ascending-lexicographic left
    /// fold — the successor key's sole registered legal realization, and the
    /// exact answer of its strict request cell. The variant name spells the
    /// realization, not the retired key.
    #[must_use]
    pub const fn tensor_contraction_f32() -> Self {
        Self::StrictTensorContractionF32 {
            structure_attribute: CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
        }
    }

    /// Standard staged strict-serial-sum-then-multiply-f32 law.
    ///
    /// A constructor for the governed spelling of the staged form, one of the
    /// three of this law's thirteen variants whose realization is a region
    /// *sequence*; the other ten are single-region, and
    /// `realizes_region_sequence` decides which is which in one match over the
    /// closed enum. No standard operation carries this row: the
    /// normalization, which is the family this shape was derived for, needs a fold
    /// prologue, an epilogue, and a ternary pass that this form does not express
    /// and carries [`Self::staged_root_mean_square_scale_f32`] instead. So the
    /// law-registry sidecar is unchanged by this row's existence.
    #[must_use]
    pub fn staged_strict_serial_sum_then_multiply_f32() -> Self {
        Self::StagedStrictSerialSumThenPointwiseF32 {
            axes_attribute: REDUCTION_AXES_ATTRIBUTE,
            scalar: multiply_f32_scalar_op(),
        }
    }

    /// Standard root-mean-square scale law, as `tiler::rms-norm-f32@1` registers it.
    ///
    /// Names that family's own two attribute identifiers. They are record-local,
    /// so this constructor is what ties the general template to the one family
    /// whose record numbers its axes field one and its `eps` field two.
    #[must_use]
    pub const fn staged_root_mean_square_scale_f32() -> Self {
        Self::StagedRootMeanSquareScaleF32 {
            axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
            eps_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
        }
    }

    /// Standard softmax law, as `tiler::softmax-f32@1` registers it.
    ///
    /// Names that family's own reduced-axes identifier. It is record-local — the
    /// normalization numbers its own axes field the same way — so this
    /// constructor is what ties the general template to the one family whose
    /// record means this field.
    #[must_use]
    pub const fn staged_softmax_f32() -> Self {
        Self::StagedSoftmaxF32 {
            axes_attribute: SOFTMAX_REDUCED_AXES_ATTRIBUTE,
        }
    }

    /// Standard strict-affine U4-to-F32 decode law.
    #[must_use]
    pub fn strict_affine_u4_dequantize() -> Self {
        Self::StrictAffineU4Dequantize {
            codes_role: STRICT_AFFINE_CODES_ROLE,
            scale_role: STRICT_AFFINE_SCALE_ROLE,
            zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            scalar: strict_affine_u4_dequantize_scalar_op(),
        }
    }

    /// Whether this law's realization is an ordered sequence of regions.
    ///
    /// A total match over the closed variant set, so the answer is decided by
    /// the variant alone and needs no subject, no scalar authority, and no
    /// realization.
    ///
    /// [`ResolvedIndexRealization::verify`](super::ResolvedIndexRealization::verify)
    /// asks it *before* any interface checking, so a caller offering one region
    /// for a staged law is told that rather than being told its lone region's
    /// boundaries disagree with the occurrence — which is true, but names the
    /// symptom instead of the mismatch. That ordering is `verify`'s alone:
    /// [`ResolvedIndexRealization::verify_sequence`](super::ResolvedIndexRealization::verify_sequence)
    /// never asks it, because a sequence candidate needs no such pre-check — a
    /// stage-count disagreement is what the whole-realization comparison is for.
    ///
    /// The predicate's other callers are the two public queries that project it,
    /// [`FrozenIndexRealizationLawRegistry::family_realizes_region_sequence`](super::FrozenIndexRealizationLawRegistry::family_realizes_region_sequence)
    /// and
    /// [`ResolvedIndexRealization::realizes_region_sequence`](super::ResolvedIndexRealization::realizes_region_sequence).
    /// Those check no interfaces at all: they answer the registered law's shape
    /// so a consumer can classify an occurrence before paying for a realization.
    pub(crate) const fn realizes_region_sequence(&self) -> bool {
        matches!(
            self,
            Self::StagedStrictSerialSumThenPointwiseF32 { .. }
                | Self::StagedRootMeanSquareScaleF32 { .. }
                | Self::StagedSoftmaxF32 { .. }
        )
    }

    /// Builds the exact canonical region sequence required by this law.
    ///
    /// The candidate is intentionally absent from this API. A law can describe
    /// expected work but cannot inspect or approve provider output.
    ///
    /// Every single-region template answers a one-stage sequence, whose identity
    /// is its region's identity byte for byte, so the sequence vocabulary changes
    /// nothing a single-region law ever produced.
    pub(crate) fn realize_sequence(
        &self,
        subject: &IndexRefinementSubject,
        scalars: &FrozenScalarRegistry,
    ) -> Result<VerifiedIndexRegionSequence, IndexRealizationLawError> {
        match self {
            Self::StagedStrictSerialSumThenPointwiseF32 {
                axes_attribute,
                scalar,
            } => {
                realize_staged_sum_then_pointwise(subject, scalars, *axes_attribute, scalar.clone())
            }
            Self::StagedRootMeanSquareScaleF32 {
                axes_attribute,
                eps_attribute,
            } => realize_root_mean_square_scale(subject, scalars, *axes_attribute, *eps_attribute),
            Self::StagedSoftmaxF32 { axes_attribute } => {
                realize_softmax(subject, scalars, *axes_attribute)
            }
            _ => Ok(VerifiedIndexRegionSequence::single(
                self.realize(subject, scalars)?,
            )),
        }
    }

    /// Builds the exact canonical logical region required by a one-region law.
    ///
    /// # Errors
    ///
    /// A law whose realization is a region sequence refuses here rather than
    /// answering one of its stages, which would be a truncated realization
    /// wearing the shape of a complete one.
    pub(crate) fn realize(
        &self,
        subject: &IndexRefinementSubject,
        scalars: &FrozenScalarRegistry,
    ) -> Result<VerifiedIndexRegion, IndexRealizationLawError> {
        let mut builder = match subject.shape_environment() {
            Some(environment)
                if broadcast_subject_is_parametric(self, subject)
                    || slice_subject_is_source_bearing(self, subject)
                    || subject_boundaries_name_a_symbol(subject) =>
            {
                IndexRegionBuilder::new_with_shape_environment(
                    scalars.clone(),
                    std::sync::Arc::clone(environment),
                )?
            }
            _ => IndexRegionBuilder::new(scalars.clone())?,
        };
        {
            let mut context = LawContext {
                builder: &mut builder,
                subject,
            };
            match self {
                Self::ConstantFromFloatBits { attribute, scalar } => {
                    realize_constant(&mut context, *attribute, scalar.clone())?;
                }
                Self::PointwiseBinary { scalar } => {
                    realize_pointwise(&mut context, scalar.clone())?;
                }
                Self::PreciseSiluF32 => realize_silu(&mut context)?,
                Self::StrictSerialSumF32 { axes_attribute } => {
                    realize_serial_sum(&mut context, *axes_attribute)?;
                }
                Self::Reindex { mapping_attribute } => {
                    realize_reindex(&mut context, *mapping_attribute)?;
                }
                Self::Broadcast { mapping_attribute } => {
                    realize_broadcast(&mut context, *mapping_attribute)?;
                }
                Self::Slice {
                    selection_attribute,
                } => realize_slice(&mut context, *selection_attribute)?,
                Self::PartitionedConcatenate { axis_attribute } => {
                    realize_concatenate(&mut context, *axis_attribute)?;
                }
                Self::StrictTensorContractionF32 {
                    structure_attribute,
                } => realize_contraction(&mut context, *structure_attribute)?,
                Self::StrictAffineU4Dequantize {
                    codes_role,
                    scale_role,
                    zero_point_role,
                    scalar,
                } => realize_strict_affine_u4_dequantize(
                    &mut context,
                    [*codes_role, *scale_role, *zero_point_role],
                    scalar.clone(),
                )?,
                Self::StagedStrictSerialSumThenPointwiseF32 { .. }
                | Self::StagedRootMeanSquareScaleF32 { .. }
                | Self::StagedSoftmaxF32 { .. } => {
                    return Err(unsupported("staged-law-requires-region-sequence"));
                }
            }
        }
        builder.build().map_err(IndexRealizationLawError::Build)
    }

    pub(crate) fn encode(&self, output: &mut Vec<u8>) {
        match self {
            Self::ConstantFromFloatBits { attribute, scalar } => {
                output.push(1);
                output.extend_from_slice(&attribute.get().to_be_bytes());
                encode_scalar(output, scalar);
            }
            Self::PointwiseBinary { scalar } => {
                output.push(2);
                encode_scalar(output, scalar);
            }
            Self::PreciseSiluF32 => output.push(3),
            Self::StrictSerialSumF32 { axes_attribute } => {
                output.push(4);
                output.extend_from_slice(&axes_attribute.get().to_be_bytes());
            }
            Self::Reindex { mapping_attribute } => {
                output.push(5);
                output.extend_from_slice(&mapping_attribute.get().to_be_bytes());
            }
            Self::Broadcast { mapping_attribute } => {
                output.push(6);
                output.extend_from_slice(&mapping_attribute.get().to_be_bytes());
            }
            Self::StrictTensorContractionF32 {
                structure_attribute,
            } => {
                output.push(7);
                output.extend_from_slice(&structure_attribute.get().to_be_bytes());
            }
            Self::StrictAffineU4Dequantize {
                codes_role,
                scale_role,
                zero_point_role,
                scalar,
            } => {
                // Tag 8 is append-only. Tags 1..=7 and their payloads are
                // unchanged. A row is self-delimiting through the canonical
                // operation and provider encodings, fixed-width revision, and
                // the tagged law payload, so this form cannot equal an old row.
                output.push(8);
                for role in [codes_role, scale_role, zero_point_role] {
                    output.extend_from_slice(&role.get().to_be_bytes());
                }
                encode_scalar(output, scalar);
            }
            Self::StagedStrictSerialSumThenPointwiseF32 {
                axes_attribute,
                scalar,
            } => {
                // Tag 9 is append-only. Tags 1..=8 and their payloads are
                // unchanged, and no registered row carries this tag, so every
                // sidecar byte a law registry has ever encoded is untouched.
                //
                // Injectivity at this site: the first byte discriminates, and the
                // payload that follows is a fixed-width attribute identifier
                // followed by the self-delimiting scalar encoding — the same
                // shape tag 4 writes for its attribute and tag 2 for its scalar,
                // but no other tag writes both, so this form is reachable from
                // exactly one variant.
                output.push(9);
                output.extend_from_slice(&axes_attribute.get().to_be_bytes());
                encode_scalar(output, scalar);
            }
            Self::StagedRootMeanSquareScaleF32 {
                axes_attribute,
                eps_attribute,
            } => {
                // Tag 10 is append-only. Tags 1..=9 and their payloads are
                // unchanged, so every sidecar byte a law registry has ever
                // encoded is byte-identical under this addition; only the row
                // this variant newly occupies is added.
                //
                // Injectivity at this site. The first byte discriminates, so no
                // other variant's encoding can be read as this one whatever
                // follows. Within this tag the payload is two fixed-width
                // attribute identifiers written in a fixed order and nothing
                // else, so the map from `(axes_attribute, eps_attribute)` to
                // bytes is a pair of injections on disjoint fixed offsets and is
                // therefore itself injective: two rows differing in either field
                // differ in the four bytes that field owns. The pair is ordered
                // rather than a set, so the transposed row — axes and `eps`
                // exchanged — encodes distinctly, which matters because that
                // transposition is a real construction error rather than a
                // hypothetical one and `realize_root_mean_square_scale` must be
                // able to refuse it as a different law rather than the same one.
                output.push(10);
                output.extend_from_slice(&axes_attribute.get().to_be_bytes());
                output.extend_from_slice(&eps_attribute.get().to_be_bytes());
            }
            Self::StagedSoftmaxF32 { axes_attribute } => {
                // Tag 11 is append-only. Tags 1..=10 and their payloads are
                // unchanged, so every sidecar byte a law registry has ever
                // encoded is byte-identical under this addition; only the row
                // this variant newly occupies is added.
                //
                // Injectivity at this site. The first byte discriminates, so no
                // other variant's encoding can be read as this one whatever
                // follows — which is the whole of the separation from tags 4, 5,
                // 6, and 7, each of which writes the same shape of payload this
                // one does: one fixed-width attribute identifier and nothing
                // else. Within this tag that payload is a single injection on a
                // fixed offset, so two rows differing in the axes identifier
                // differ in the four bytes it owns. There is no second field, so
                // no ordering question arises here of the kind tag 10 has to
                // answer for its pair.
                output.push(11);
                output.extend_from_slice(&axes_attribute.get().to_be_bytes());
            }
            Self::PartitionedConcatenate { axis_attribute } => {
                // Tag 12 is append-only. Tags 1..=11 and their payloads are
                // unchanged, so every sidecar byte a law registry has ever
                // encoded is byte-identical under this addition; only the row
                // this variant newly occupies is added.
                //
                // Injectivity at this site. The first byte discriminates, so no
                // other variant's encoding can be read as this one whatever
                // follows — which is the whole of the separation from tags 4, 5,
                // 6, 7, and 11, each of which writes the payload shape this one
                // does: one fixed-width attribute identifier and nothing else.
                // Within this tag that payload is a single injection on a fixed
                // offset, so two rows differing in the axis identifier differ in
                // the four bytes it owns. There is no second field, so no
                // ordering question arises here of the kind tag 10 has to answer
                // for its pair.
                output.push(12);
                output.extend_from_slice(&axis_attribute.get().to_be_bytes());
            }
            Self::Slice {
                selection_attribute,
            } => {
                // Tag 13 is append-only. Tags 1..=12 and their payloads are
                // unchanged, so every pre-existing row remains byte-identical;
                // only the newly registered slice row enters the sidecar.
                //
                // Injectivity at this site: the first byte discriminates this
                // variant from the six older one-attribute payloads, and the
                // remaining four bytes injectively encode the record-local
                // selection field identifier.
                output.push(13);
                output.extend_from_slice(&selection_attribute.get().to_be_bytes());
            }
        }
    }
}

/// Whether the contract is stated for the arithmetic this law's result carries.
///
/// **Derived from the verified semantic subject, never declared on the law.**
/// Every template that reaches here builds its output tensor with the subject's
/// own result type, so that type *is* the arithmetic the expected region will
/// emit; restating it as law data would be a second authority over one fact, and
/// the two could disagree. The comparison goes through
/// [`ArithmeticType::canonical_type_key`], which is the single durable spelling
/// of a dtype identity, so a contract stated for a width the result is not
/// produced in — an `f32` program under a `bf16` contract, or the reverse — is
/// refused rather than governed by a contract about another format's subnormals
/// and rounding.
///
/// A result type that is not nominal names no arithmetic and is refused, which
/// is strictly tighter than the `f32`-only test this replaced.
///
/// [`ArithmeticType::canonical_type_key`]: crate::schedule::ArithmeticType::canonical_type_key
fn governs_result_arithmetic(subject: &IndexRefinementSubject) -> bool {
    let [result] = subject.results() else {
        // Not a single-result subject. Every template here refuses one by its
        // own arity rule, which names the defect precisely; answering `false`
        // would replace that diagnostic with a contract complaint about a
        // subject no law was ever going to accept.
        return true;
    };
    let expected = subject
        .numerical_contract()
        .arithmetic()
        .canonical_type_key();
    result
        .value_type()
        .nominal_key()
        .is_some_and(|key| key.to_string() == expected)
}

fn encode_scalar(output: &mut Vec<u8>, scalar: &ScalarOpKey) {
    for component in [scalar.namespace(), scalar.name()] {
        output.extend_from_slice(&(component.len() as u64).to_be_bytes());
        output.extend_from_slice(component.as_bytes());
    }
    output.extend_from_slice(&scalar.semantic_version().to_be_bytes());
}

/// Failure to interpret a registered logical realization law.
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum IndexRealizationLawError {
    /// The controlled canonical builder rejected an emission.
    Emit(IndexBuildError),
    /// A sourced extent was unavailable in this static law profile.
    Extent(SymbolicExtentError),
    /// The semantic occurrence is outside the law's exact supported form.
    Unsupported {
        /// Stable rule naming the refused fact.
        rule: &'static str,
    },
    /// Whole-region verification rejected the expected construction.
    Build(IndexRegionBuildError),
    /// The staged regions this law built do not compose into a chain.
    ///
    /// Reaching this is a defect in the law's own construction rather than in
    /// any candidate: the sequence is built from the law's regions alone.
    Sequence(IndexRegionSequenceError),
}

impl fmt::Display for IndexRealizationLawError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Emit(source) => write!(formatter, "law emission failed: {source}"),
            Self::Extent(source) => write!(formatter, "law extent failed: {source}"),
            Self::Unsupported { rule } => write!(formatter, "law does not support {rule}"),
            Self::Build(source) => write!(formatter, "law region failed verification: {source}"),
            Self::Sequence(source) => write!(formatter, "law sequence is not chained: {source}"),
        }
    }
}

impl Error for IndexRealizationLawError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Emit(source) => Some(source),
            Self::Extent(source) => Some(source),
            Self::Build(source) => Some(source),
            Self::Sequence(source) => Some(source),
            Self::Unsupported { .. } => None,
        }
    }
}

impl IndexRealizationLawError {
    pub(crate) const fn rule(&self) -> &'static str {
        match self {
            Self::Emit(_) => "canonical-emission",
            Self::Extent(_) => "extent-authority",
            Self::Unsupported { rule } => rule,
            Self::Build(_) => "expected-region-verification",
            Self::Sequence(_) => "expected-sequence-chaining",
        }
    }
}

impl From<IndexBuildError> for IndexRealizationLawError {
    fn from(source: IndexBuildError) -> Self {
        Self::Emit(source)
    }
}

impl From<SymbolicExtentError> for IndexRealizationLawError {
    fn from(source: SymbolicExtentError) -> Self {
        Self::Extent(source)
    }
}

fn unsupported(rule: &'static str) -> IndexRealizationLawError {
    IndexRealizationLawError::Unsupported { rule }
}

struct LawContext<'a> {
    builder: &'a mut IndexRegionBuilder,
    subject: &'a IndexRefinementSubject,
}

impl LawContext<'_> {
    fn dimension(
        &mut self,
        role: DomainRole,
        extent: Extent,
    ) -> Result<DimensionId, IndexRealizationLawError> {
        Ok(self.builder.dimension(role, extent)?)
    }
    fn sourced_dimension(
        &mut self,
        role: DomainRole,
        extent: SourcedExtent,
    ) -> Result<DimensionId, IndexRealizationLawError> {
        match extent {
            SourcedExtent::Static(extent) => self.dimension(role, extent),
            SourcedExtent::Symbol(symbol) => Ok(self.builder.symbolic_dimension(role, symbol)?),
        }
    }
    fn tensor(
        &mut self,
        role: TensorRole,
        value_type: ResolvedValueType,
        shape: Shape,
    ) -> Result<TensorId, IndexRealizationLawError> {
        Ok(self.builder.tensor(role, value_type, shape)?)
    }
    fn sourced_tensor(
        &mut self,
        role: TensorRole,
        value_type: ResolvedValueType,
        shape: &crate::shape::SourcedShape,
    ) -> Result<TensorId, IndexRealizationLawError> {
        Ok(self
            .builder
            .sourced_tensor(role, value_type, shape.extents().collect())?)
    }
    fn constant(&mut self, value: IndexInteger) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.constant(value)?)
    }
    fn dimension_expr(
        &mut self,
        dimension: DimensionId,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.dimension_expr(dimension)?)
    }
    fn linear_combination(
        &mut self,
        constant: IndexInteger,
        terms: &[(IndexInteger, IndexExprId)],
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.linear_combination(constant, terms)?)
    }
    fn sourced_linear_combination(
        &mut self,
        constant: SourcedIndexInteger,
        terms: &[(SourcedIndexInteger, IndexExprId)],
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.sourced_linear_combination(constant, terms)?)
    }
    fn floor_div(
        &mut self,
        value: IndexExprId,
        divisor: SourcedExtent,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.floor_div(value, divisor)?)
    }
    fn modulo(
        &mut self,
        value: IndexExprId,
        divisor: SourcedExtent,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.modulo(value, divisor)?)
    }
    fn read(
        &mut self,
        tensor: TensorId,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        Ok(self.builder.read(tensor, domain, coordinates)?)
    }
    fn write(
        &mut self,
        tensor: TensorId,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<TensorAccessId, IndexRealizationLawError> {
        Ok(self.builder.write(tensor, domain, coordinates)?)
    }
    fn apply(
        &mut self,
        key: ScalarOpKey,
        attributes: ScalarAttributes,
        operands: &[ScalarValueId],
    ) -> Result<super::ScalarResults, IndexRealizationLawError> {
        Ok(self.builder.apply(key, attributes, operands)?)
    }
    fn reduce<F>(
        &mut self,
        dimensions: &[DimensionId],
        init: &[ScalarValueId],
        contributors: &[ScalarValueId],
        build: F,
    ) -> Result<super::ScalarResults, IndexRealizationLawError>
    where
        F: FnOnce(&mut ScalarReducerBodyBuilder<'_>) -> Result<(), IndexBuildError>,
    {
        Ok(self.builder.reduce(dimensions, init, contributors, build)?)
    }
    fn output(
        &mut self,
        access: TensorAccessId,
        value: ScalarValueId,
    ) -> Result<(), IndexRealizationLawError> {
        Ok(self.builder.output(access, value)?)
    }
}

fn realize_constant(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
    scalar: ScalarOpKey,
) -> Result<(), IndexRealizationLawError> {
    if !context.subject.inputs().is_empty() {
        return Err(unsupported("constant-operand-arity"));
    }
    let [result] = context.subject.results() else {
        return Err(unsupported("constant-result-arity"));
    };
    if result.shape().rank() != 0 {
        return Err(unsupported("constant-result-rank"));
    }
    let [field] = context.subject.attributes().fields() else {
        return Err(unsupported("constant-attributes"));
    };
    if field.id() != attribute {
        return Err(unsupported("constant-attribute-key"));
    }
    let attributes = scalar_attributes(attribute, field.value().clone())?;
    let output = context.tensor(
        TensorRole::Output,
        result.value_type().clone(),
        result.shape().clone(),
    )?;
    let value = single_result(&context.apply(scalar, attributes, &[])?, "constant")?;
    let write = context.write(output, &[], &[])?;
    context.output(write, value)
}

fn realize_pointwise(
    context: &mut LawContext<'_>,
    scalar: ScalarOpKey,
) -> Result<(), IndexRealizationLawError> {
    let [result] = context.subject.results() else {
        return Err(unsupported("pointwise-result-arity"));
    };
    let result = (
        (*result).value_type().clone(),
        (*result).sourced_shape().clone(),
    );
    let inputs = context
        .subject
        .inputs()
        .iter()
        .map(|input| (input.value_type().clone(), input.sourced_shape().clone()))
        .collect::<Vec<_>>();
    let operands = context.subject.operands().to_vec();
    emit_pointwise(context, scalar, &inputs, &operands, &result)
}

/// Emits `result[i] = scalar(operands[0][i], operands[1][i])` over explicit
/// boundaries.
///
/// The boundaries are parameters rather than reads of the subject because a
/// staged realization's later stage consumes a value that is not an occurrence
/// input at all: the fold an earlier stage published. Deriving them from the
/// subject would make that stage unstatable.
fn emit_pointwise(
    context: &mut LawContext<'_>,
    scalar: ScalarOpKey,
    inputs: &[(ResolvedValueType, crate::shape::SourcedShape)],
    operands: &[usize],
    result: &(ResolvedValueType, crate::shape::SourcedShape),
) -> Result<(), IndexRealizationLawError> {
    if operands.len() != 2 {
        return Err(unsupported("pointwise-operand-arity"));
    }
    // The authored sourced boundary throughout: a same-shape symbolic domain
    // keeps its symbols as written — the sourced dimension below is the
    // declared symbol, never a bound value — and the exact per-boundary
    // comparison consults no environment, so two differently spelled
    // proved-equal symbols stay two different boundaries.
    let shape = result.1.clone();
    let mut dimensions = Vec::with_capacity(shape.rank());
    for extent in shape.extents() {
        dimensions.push(context.sourced_dimension(DomainRole::Parallel, extent)?);
    }
    let coordinates = dimension_expressions(context, &dimensions)?;
    let mut tensors = Vec::with_capacity(inputs.len());
    for (value_type, input_shape) in inputs {
        tensors.push(context.sourced_tensor(TensorRole::Input, value_type.clone(), input_shape)?);
    }
    let mut values = Vec::with_capacity(2);
    for position in operands.iter().copied() {
        let (_, boundary) = inputs
            .get(position)
            .ok_or_else(|| unsupported("pointwise-operand-binding"))?;
        let tensor = tensors[position];
        let value = if boundary == &shape {
            context.read(tensor, &dimensions, &coordinates)?
        } else if boundary.rank() == 0 {
            context.read(tensor, &[], &[])?
        } else {
            return Err(unsupported("pointwise-broadcast"));
        };
        values.push(value);
    }
    let value = single_result(
        &context.apply(scalar, ScalarAttributes::empty(), &values)?,
        "pointwise",
    )?;
    let output = context.sourced_tensor(TensorRole::Output, result.0.clone(), &shape)?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}

fn realize_strict_affine_u4_dequantize(
    context: &mut LawContext<'_>,
    roles: [EncodedComponentRole; 3],
    scalar: ScalarOpKey,
) -> Result<(), IndexRealizationLawError> {
    let ([input], [result]) = (context.subject.inputs(), context.subject.results()) else {
        return Err(unsupported("strict-affine-arity"));
    };
    if context.subject.operands() != [0] {
        return Err(unsupported("strict-affine-operand-binding"));
    }
    if !context.subject.attributes().fields().is_empty() {
        return Err(unsupported("strict-affine-attributes"));
    }
    if input.value_type() != &StrictAffineU4::resolved_type() {
        return Err(unsupported("strict-affine-encoded-contract"));
    }
    if result.value_type() != &crate::semantic::F32::resolved_type() {
        return Err(unsupported("strict-affine-result-type"));
    }
    if input.shape() != result.shape() {
        return Err(unsupported("strict-affine-result-shape"));
    }
    let (_, contract) = input
        .value_type()
        .encoded_numeric_parts()
        .ok_or_else(|| unsupported("strict-affine-encoded-contract"))?;
    let components = contract.components();
    if components.len() != roles.len()
        || components
            .iter()
            .zip(roles)
            .any(|(component, role)| component.role() != role)
    {
        return Err(unsupported("strict-affine-component-roles"));
    }

    let shape = result.shape().clone();
    let dimensions = declare_parallel_domain(context, &shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let mut tensors = Vec::with_capacity(components.len());
    for component in components {
        tensors.push(context.tensor(
            TensorRole::Input,
            component.resolved_type().clone(),
            component.shape_relation().component_shape(input.shape()),
        )?);
    }
    let codes = context.read(tensors[0], &dimensions, &coordinates)?;
    let scale = context.read(tensors[1], &[], &[])?;
    let zero_point = context.read(tensors[2], &[], &[])?;
    let value = apply_one(context, scalar, &[codes, scale, zero_point])?;
    let output = context.tensor(TensorRole::Output, result.value_type().clone(), shape)?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}

fn realize_silu(context: &mut LawContext<'_>) -> Result<(), IndexRealizationLawError> {
    let [result] = context.subject.results() else {
        return Err(unsupported("silu-result-arity"));
    };
    if context.subject.operands() != [0] || context.subject.inputs().len() != 1 {
        return Err(unsupported("silu-operand-arity"));
    }
    let input = context.subject.inputs()[0].clone();
    if input.shape() != result.shape() {
        return Err(unsupported("silu-shape"));
    }
    let shape = result.shape().clone();
    let dimensions = declare_parallel_domain(context, &shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let tensor = context.tensor(TensorRole::Input, input.value_type().clone(), shape.clone())?;
    let argument = context.read(tensor, &dimensions, &coordinates)?;
    let negative_one = scalar_constant(context, F32_NEGATIVE_ONE_BITS)?;
    let negated = apply_one(context, multiply_f32_scalar_op(), &[argument, negative_one])?;
    let exponential = apply_one(context, exp_f32_scalar_op(), &[negated])?;
    let one = scalar_constant(context, F32_ONE_BITS)?;
    let divisor = apply_one(context, add_f32_scalar_op(), &[one, exponential])?;
    let value = apply_one(context, divide_f32_scalar_op(), &[argument, divisor])?;
    let output = context.tensor(TensorRole::Output, result.value_type().clone(), shape)?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}

fn scalar_constant(
    context: &mut LawContext<'_>,
    bits: u32,
) -> Result<ScalarValueId, IndexRealizationLawError> {
    let value = CanonicalValue::float_bits(
        TypeKey::new("tiler", "f32", 1).expect("the governed f32 key is valid"),
        bits.to_be_bytes(),
    )
    .map_err(|_| unsupported("scalar-constant"))?;
    let values = context.apply(
        constant_f32_scalar_op(),
        scalar_attributes(F32_CONSTANT_BITS_ATTRIBUTE, value)?,
        &[],
    )?;
    single_result(&values, "scalar-constant")
}

/// Wraps one exact payload in the attribute record its scalar constant declares.
///
/// The field is a parameter rather than the `f32` constant's own identifier
/// because attribute field IDs are record-local: the `f32` and `bf16` constants
/// number their payload field alike, and a writer hard-coding one of them would
/// build the other family's record correctly only by that coincidence.
fn scalar_attributes(
    field: AttributeFieldId,
    bits: CanonicalValue,
) -> Result<ScalarAttributes, IndexRealizationLawError> {
    let record = CanonicalValue::record([CanonicalField::new(field, bits)])
        .map_err(|_| unsupported("scalar-constant-attributes"))?;
    ScalarAttributes::new(record).map_err(|_| unsupported("scalar-constant-attributes"))
}

fn apply_one(
    context: &mut LawContext<'_>,
    key: ScalarOpKey,
    operands: &[ScalarValueId],
) -> Result<ScalarValueId, IndexRealizationLawError> {
    let values = context.apply(key, ScalarAttributes::empty(), operands)?;
    single_result(&values, "scalar-application")
}

fn single_result(
    values: &super::ScalarResults,
    rule: &'static str,
) -> Result<ScalarValueId, IndexRealizationLawError> {
    if values.len() != 1 {
        return Err(unsupported(rule));
    }
    values.get(0).ok_or_else(|| unsupported(rule))
}

fn declare_parallel_domain(
    context: &mut LawContext<'_>,
    shape: &Shape,
) -> Result<Vec<DimensionId>, IndexRealizationLawError> {
    shape
        .extents()
        .iter()
        .copied()
        .map(|extent| context.dimension(DomainRole::Parallel, extent))
        .collect()
}

fn dimension_expressions(
    context: &mut LawContext<'_>,
    dimensions: &[DimensionId],
) -> Result<Vec<IndexExprId>, IndexRealizationLawError> {
    dimensions
        .iter()
        .copied()
        .map(|dimension| context.dimension_expr(dimension))
        .collect()
}

// The remaining non-pointwise templates are filled below; keeping their entry
// points explicit makes unsupported support impossible to confuse with a law.
fn realize_serial_sum(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    let plan = FoldPlan::derive(context.subject, attribute)?;
    emit_fold_region(context, &plan, |_, total| Ok(total))
}

/// Emits one whole folding region: the fold, an epilogue, and the write.
///
/// **The epilogue is a parameter because a staged realization transforms its
/// fold inside the producing region.** The normalization divides, biases, and
/// takes the reciprocal square root of its fold before anything is written; the
/// softmax's two folds write theirs unchanged. A fold emitter that could only
/// write its own result would force the normalization's epilogue into the
/// consuming stage, where it would run once per point instead of once per folded
/// row — a different scalar program, not a different placement.
///
/// The plain fold passes the identity epilogue, which emits nothing, so every
/// region this emitter produced before the parameter existed is unchanged.
fn emit_fold_region<F>(
    context: &mut LawContext<'_>,
    plan: &FoldPlan,
    epilogue: F,
) -> Result<(), IndexRealizationLawError>
where
    F: FnOnce(
        &mut LawContext<'_>,
        ScalarValueId,
    ) -> Result<ScalarValueId, IndexRealizationLawError>,
{
    let kept = plan.declare_kept_domain(context)?;
    let kept_coordinates = dimension_expressions(context, &kept)?;
    let input = context.tensor(
        TensorRole::Input,
        plan.value_type.clone(),
        plan.input_shape.clone(),
    )?;
    let output = context.tensor(
        TensorRole::Output,
        plan.value_type.clone(),
        plan.output_shape.clone(),
    )?;
    let total = plan.fold(context, input, &kept, &kept_coordinates)?;
    let published = epilogue(context, total)?;
    let write = context.write(output, &kept, &kept_coordinates)?;
    context.output(write, published)
}

/// How one input boundary of a [`emit_row_broadcast_stage`] stage is addressed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StageAccess {
    /// At the stage's own point coordinates. The boundary is the result shape.
    Point,
    /// At the kept coordinates alone. The boundary is the result shape without
    /// the folded axes, so one value serves every point of a folded row.
    ///
    /// The rank-zero case — every axis folded — is this one degenerate, and it is
    /// deliberately not spelled separately: a read at an empty domain with empty
    /// coordinates is what both mean.
    FoldedRow,
}

/// Emits one stage that pairs per-point values with per-folded-row values.
///
/// The stage declares a parallel domain over `result`'s shape, reads each input
/// boundary at either its own point coordinates or the kept coordinates of the
/// folded axes, hands the read values to `body`, and writes what `body` answers.
///
/// **Three capabilities live here that the binary pointwise emitter does not
/// have.** It admits *any* number of input boundaries rather than exactly two; a
/// boundary of reduced rank is read at the kept coordinates rather than refused
/// as an unstated broadcast; and `body` is arbitrary scalar work between the
/// reads and the write rather than one scalar application. The third is what
/// makes a *per-row prologue* expressible without a rule of its own: a value
/// computed from a folded-row read alone carries only the kept dimensions as its
/// free dimensions, so `1.0 / d` is evaluated once per row and `e_i * c` once per
/// point, and the region model says so rather than a comment.
///
/// `reduced` marks, per axis of the result shape, whether the folded axes cover
/// it. It is the [`FoldPlan`]'s own mask, so the stage that consumes a fold and
/// the fold itself cannot disagree about which axes were removed.
fn emit_row_broadcast_stage<F>(
    context: &mut LawContext<'_>,
    reduced: &[bool],
    inputs: &[(ResolvedValueType, Shape, StageAccess)],
    result: &(ResolvedValueType, Shape),
    body: F,
) -> Result<(), IndexRealizationLawError>
where
    F: FnOnce(
        &mut LawContext<'_>,
        &[ScalarValueId],
    ) -> Result<ScalarValueId, IndexRealizationLawError>,
{
    let shape = result.1.clone();
    if reduced.len() != shape.rank() {
        return Err(unsupported("row-broadcast-reduced-rank"));
    }
    let row_shape = Shape::try_new(
        shape
            .extents()
            .iter()
            .zip(reduced)
            .filter(|(_, reduced)| !**reduced)
            .map(|(extent, _)| *extent)
            .collect::<Vec<_>>(),
    )
    .map_err(|_| unsupported("row-broadcast-row-shape"))?;
    let dimensions = declare_parallel_domain(context, &shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let kept = dimensions
        .iter()
        .zip(reduced)
        .filter(|(_, reduced)| !**reduced)
        .map(|(dimension, _)| *dimension)
        .collect::<Vec<_>>();
    let kept_coordinates = coordinates
        .iter()
        .zip(reduced)
        .filter(|(_, reduced)| !**reduced)
        .map(|(coordinate, _)| *coordinate)
        .collect::<Vec<_>>();
    let mut tensors = Vec::with_capacity(inputs.len());
    for (value_type, boundary, _) in inputs {
        tensors.push(context.tensor(TensorRole::Input, value_type.clone(), boundary.clone())?);
    }
    let mut values = Vec::with_capacity(inputs.len());
    for ((_, boundary, access), tensor) in inputs.iter().zip(&tensors) {
        let value = match access {
            StageAccess::Point => {
                if boundary != &shape {
                    return Err(unsupported("row-broadcast-point-boundary"));
                }
                context.read(*tensor, &dimensions, &coordinates)?
            }
            StageAccess::FoldedRow => {
                if boundary != &row_shape {
                    return Err(unsupported("row-broadcast-row-boundary"));
                }
                context.read(*tensor, &kept, &kept_coordinates)?
            }
        };
        values.push(value);
    }
    let value = body(context, &values)?;
    let output = context.tensor(TensorRole::Output, result.0.clone(), shape)?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}
/// Builds the two-stage fold-then-pointwise realization.
///
/// The occurrence is `(operand 0, operand 1) -> result`: stage zero folds operand
/// zero over the attribute's axes into a value that is no occurrence boundary,
/// and stage one applies the scalar to operand one and that value.
///
/// Stage one declares its input boundaries in the order `(operand 1,
/// intermediate)`, which is the order its sources report and therefore the order
/// operand binding walks. The pointwise emitter's own broadcast rule is what
/// decides whether the fold is legible from stage one: a fold that removed every
/// axis is rank zero and read once per point, and a fold whose result is exactly
/// the result shape is read pointwise. Any other reduced shape refuses with
/// `pointwise-broadcast` rather than being stretched by a rule this law has not
/// stated.
fn realize_staged_sum_then_pointwise(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    axes_attribute: AttributeFieldId,
    scalar: ScalarOpKey,
) -> Result<VerifiedIndexRegionSequence, IndexRealizationLawError> {
    let ([folded, elementwise], [result]) = (subject.inputs(), subject.results()) else {
        return Err(unsupported("staged-sum-pointwise-arity"));
    };
    if subject.operands() != [0, 1] {
        return Err(unsupported("staged-sum-pointwise-operand-binding"));
    }
    let axes = reduction_axes(subject.attributes(), axes_attribute)?;
    let intermediate_shape = folded.shape().without_axes(&axes);
    let plan = FoldPlan::for_boundaries(
        folded.value_type(),
        folded.shape(),
        &intermediate_shape,
        &axes,
    )?;

    let mut fold = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut fold,
            subject,
        };
        emit_fold_region(&mut context, &plan, |_, total| Ok(total))?;
    }
    let fold = fold.build().map_err(IndexRealizationLawError::Build)?;

    let mut apply = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut apply,
            subject,
        };
        emit_pointwise(
            &mut context,
            scalar,
            &[
                (
                    elementwise.value_type().clone(),
                    elementwise.shape().clone().into(),
                ),
                (folded.value_type().clone(), intermediate_shape.into()),
            ],
            &[0, 1],
            &(result.value_type().clone(), result.shape().clone().into()),
        )?;
    }
    let apply = apply.build().map_err(IndexRealizationLawError::Build)?;

    VerifiedIndexRegionSequence::try_new(
        vec![fold, apply],
        vec![
            vec![StagedInputSource::Occurrence(0)],
            vec![
                StagedInputSource::Occurrence(1),
                StagedInputSource::Intermediate(0),
            ],
        ],
    )
    .map_err(IndexRealizationLawError::Sequence)
}

/// Builds the two-stage root-mean-square scale realization.
///
/// The occurrence is `(operand 0 = value, operand 1 = weight) -> result`, and the
/// realization is the pinned reference in order: `q_i = square(x_i)`, `a` the
/// strict left fold of `q`, `u = a / N`, `t = u + eps`, `r = Rsqrt(t)`, and
/// `y_i = w_i * (x_i * r)`.
///
/// **Where the split falls, and why there.** `r` is read once per *point* and
/// computed once per *folded row*, so stage zero carries the whole epilogue and
/// publishes `r` rather than publishing `a` and leaving the epilogue to stage
/// one. Publishing `a` would put the division, the bias, and the reciprocal
/// square root inside the pointwise pass, evaluating each `N` times per row: a
/// different scalar program, not a different schedule for this one.
///
/// **Every declared attribute is consumed by name.** The occurrence's field set
/// is required to be exactly the two this law names, so the tolerance of
/// [`reduction_axes`] for a record carrying more fields than it reads cannot
/// silently drop the `eps` payload here — which is the whole reason the plain
/// staged template is not this family's law.
fn realize_root_mean_square_scale(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    axes_attribute: AttributeFieldId,
    eps_attribute: AttributeFieldId,
) -> Result<VerifiedIndexRegionSequence, IndexRealizationLawError> {
    let ([value, weight], [result]) = (subject.inputs(), subject.results()) else {
        return Err(unsupported("rms-scale-arity"));
    };
    if subject.operands() != [0, 1] {
        return Err(unsupported("rms-scale-operand-binding"));
    }
    let expected = crate::semantic::F32::resolved_type();
    if value.value_type() != &expected
        || weight.value_type() != &expected
        || result.value_type() != &expected
    {
        return Err(unsupported("rms-scale-value-type"));
    }
    if value.shape() != result.shape() || weight.shape() != result.shape() {
        return Err(unsupported("rms-scale-shape"));
    }

    // Two distinct named identifiers, and a record of exactly two fields both of
    // which are named, means each is present exactly once. Aliased identifiers
    // would satisfy the count while leaving one of the two unread, so they are
    // refused first rather than left to make the inference false.
    if axes_attribute == eps_attribute {
        return Err(unsupported("rms-scale-attribute-aliasing"));
    }
    let fields = subject.attributes().fields();
    if fields.len() != 2
        || !fields
            .iter()
            .all(|field| field.id() == axes_attribute || field.id() == eps_attribute)
    {
        return Err(unsupported("rms-scale-attributes"));
    }
    let eps = subject
        .attributes()
        .get(eps_attribute)
        .ok_or_else(|| unsupported("rms-scale-eps-missing"))?
        .clone();
    if !matches!(eps.view(), CanonicalValueView::FloatBits(_)) {
        return Err(unsupported("rms-scale-eps-kind"));
    }
    let axes = reduction_axes(subject.attributes(), axes_attribute)?;

    let intermediate_shape = value.shape().without_axes(&axes);
    let plan = FoldPlan::for_boundaries(
        value.value_type(),
        value.shape(),
        &intermediate_shape,
        &axes,
    )?
    .squaring_contributors(multiply_f32_scalar_op());
    let extent_bits = folded_extent_bits(plan.reduced_points)?;

    let mut fold = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut fold,
            subject,
        };
        emit_fold_region(
            &mut context,
            &plan,
            |context: &mut LawContext<'_>, total: ScalarValueId| {
                let extent = scalar_constant(context, extent_bits)?;
                let mean = apply_one(context, divide_f32_scalar_op(), &[total, extent])?;
                let bias = single_result(
                    &context.apply(
                        constant_f32_scalar_op(),
                        scalar_attributes(F32_CONSTANT_BITS_ATTRIBUTE, eps)?,
                        &[],
                    )?,
                    "rms-scale-eps-constant",
                )?;
                let biased = apply_one(context, add_f32_scalar_op(), &[mean, bias])?;
                apply_one(context, rsqrt_f32_scalar_op(), &[biased])
            },
        )?;
    }
    let fold = fold.build().map_err(IndexRealizationLawError::Build)?;

    let mut scale = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut scale,
            subject,
        };
        let shape = result.shape().clone();
        // The published value is one per folded row, so it is read at the kept
        // coordinates of this stage's own point domain — the `FoldedRow` access
        // below. That is neither the rank-zero nor the whole-shape case the
        // binary pointwise emitter admits, which is one of the three things this
        // family needs that the staged template cannot state.
        emit_row_broadcast_stage(
            &mut context,
            &plan.reduced,
            &[
                (expected.clone(), shape.clone(), StageAccess::Point),
                (expected.clone(), shape.clone(), StageAccess::Point),
                (expected.clone(), intermediate_shape, StageAccess::FoldedRow),
            ],
            &(expected, shape),
            |context: &mut LawContext<'_>, values: &[ScalarValueId]| {
                let [element, weight_element, root] = values else {
                    return Err(unsupported("rms-scale-pass-operands"));
                };
                let scaled = apply_one(context, multiply_f32_scalar_op(), &[*element, *root])?;
                apply_one(
                    context,
                    multiply_f32_scalar_op(),
                    &[*weight_element, scaled],
                )
            },
        )?;
    }
    let scale = scale.build().map_err(IndexRealizationLawError::Build)?;

    VerifiedIndexRegionSequence::try_new(
        vec![fold, scale],
        vec![
            vec![StagedInputSource::Occurrence(0)],
            vec![
                StagedInputSource::Occurrence(0),
                StagedInputSource::Occurrence(1),
                StagedInputSource::Intermediate(0),
            ],
        ],
    )
    .map_err(IndexRealizationLawError::Sequence)
}

/// Builds the four-stage softmax realization.
///
/// The occurrence is `(operand 0 = scores) -> result`, and the realization is
/// `softmax_f32_reference_semantics` in order: `m` the strict left fold of the
/// NaN-propagating maximum seeded at the first contributor, `e_i = Exp(s_i - m)`,
/// `d` the strict left fold sum of `e` seeded at the first contributor,
/// `c = 1.0 / d` as one division of one by the denominator, and `r_i = e_i * c`
/// as a multiplication by that reciprocal.
///
/// **Where the three splits fall.** `m` is read once per point and computed once
/// per row, so it is published rather than recomputed. `e` is read by the
/// denominator's fold *and* by the final pass, which is why it is published as
/// well and why this chain is four stages: recomputing it in the final pass would
/// evaluate one exponential per point twice, and a chain that copied it through
/// the folding stage would put a materialization no part of the operation means
/// into a region's canonical identity. `c` is computed inside the final stage
/// from a folded-row read alone, so its free dimensions are the kept ones and it
/// is evaluated once per row — the reference's single division, not one per
/// point.
///
/// **The subtraction is spelled as an exact sign flip and one rounded add.**
/// There is no subtraction scalar key, and negating a binary32 value is exact, so
/// `s_i + (-m)` rounds exactly where `s_i - m` does and is the same function.
/// `SOFTMAX_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED` names this adjacency as
/// the operation's only multiply-add pair and withholds contraction over it,
/// which is a statement about this spelling rather than about a rewrite of it.
///
/// **Every declared attribute is consumed by name.** The occurrence's field set
/// is required to be exactly the one this law names, so the tolerance of
/// [`reduction_axes`] for a record carrying more fields than it reads cannot let
/// a payload go unread here.
fn realize_softmax(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    axes_attribute: AttributeFieldId,
) -> Result<VerifiedIndexRegionSequence, IndexRealizationLawError> {
    let ([scores], [result]) = (subject.inputs(), subject.results()) else {
        return Err(unsupported("softmax-arity"));
    };
    if subject.operands() != [0] {
        return Err(unsupported("softmax-operand-binding"));
    }
    let expected = crate::semantic::F32::resolved_type();
    if scores.value_type() != &expected || result.value_type() != &expected {
        return Err(unsupported("softmax-value-type"));
    }
    // Shape-preserving: the reduced axis is folded over twice and then restored,
    // so an occurrence whose result dropped it is a reduction wearing this law.
    if scores.shape() != result.shape() {
        return Err(unsupported("softmax-shape"));
    }
    let [field] = subject.attributes().fields() else {
        return Err(unsupported("softmax-attributes"));
    };
    if field.id() != axes_attribute {
        return Err(unsupported("softmax-attributes"));
    }
    let axes = reduction_axes(subject.attributes(), axes_attribute)?;
    if axes.len() != 1 {
        // Unreachable from a verified occurrence, and stated under the
        // unreachable-refusal convention at `IndexRealizationLaw`.
        // `tiler::softmax-f32@1`'s own inferencer refuses an absent axis, a
        // duplicated one, and any count other than one — `softmax.f32.axis`'s
        // `absent`, `duplicated`, and `rank` codes — before a subject exists,
        // and no other registered family reaches this line: the only other one
        // whose single record field is a `u32` axis sequence is the strict
        // serial sum, whose result drops those axes and is refused by
        // `softmax-shape` above. A subject carrying a two-axis sequence is
        // nonetheless expressible, and the reference pins the formula over *the
        // single reduced axis* — a two-axis fold would be a different operation
        // realized under this law's name.
        return Err(unsupported("softmax-reduced-axis-rank"));
    }
    let shape = result.shape().clone();
    let row_shape = shape.without_axes(&axes);

    // Both folds are seeded at the first contributor, which is what the reference
    // pins for each of them, so neither carries an empty-domain identity. The
    // maximum has none to carry; the sum has one and deliberately does not use it
    // here, because a zero-length reduced axis evaluates no scalar softmax at all
    // and this shape would still have to commit one value per kept coordinate.
    let extrema_plan = FoldPlan::for_boundaries(&expected, &shape, &row_shape, &axes)?
        .combining(maximum_f32_scalar_op(), None);
    let denominator_plan = FoldPlan::for_boundaries(&expected, &shape, &row_shape, &axes)?
        .combining(add_f32_scalar_op(), None);

    let mut extrema = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut extrema,
            subject,
        };
        emit_fold_region(&mut context, &extrema_plan, |_, total| Ok(total))?;
    }
    let extrema = extrema.build().map_err(IndexRealizationLawError::Build)?;

    let mut exponentials = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut exponentials,
            subject,
        };
        emit_row_broadcast_stage(
            &mut context,
            &extrema_plan.reduced,
            &[
                (expected.clone(), shape.clone(), StageAccess::Point),
                (expected.clone(), row_shape.clone(), StageAccess::FoldedRow),
            ],
            &(expected.clone(), shape.clone()),
            |context: &mut LawContext<'_>, values: &[ScalarValueId]| {
                let [score, maximum] = values else {
                    return Err(unsupported("softmax-exponential-operands"));
                };
                let negative_one = scalar_constant(context, F32_NEGATIVE_ONE_BITS)?;
                let negated =
                    apply_one(context, multiply_f32_scalar_op(), &[*maximum, negative_one])?;
                let shifted = apply_one(context, add_f32_scalar_op(), &[*score, negated])?;
                apply_one(context, exp_f32_scalar_op(), &[shifted])
            },
        )?;
    }
    let exponentials = exponentials
        .build()
        .map_err(IndexRealizationLawError::Build)?;

    let mut denominator = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut denominator,
            subject,
        };
        emit_fold_region(&mut context, &denominator_plan, |_, total| Ok(total))?;
    }
    let denominator = denominator
        .build()
        .map_err(IndexRealizationLawError::Build)?;

    let mut normalize = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut normalize,
            subject,
        };
        emit_row_broadcast_stage(
            &mut context,
            &extrema_plan.reduced,
            &[
                (expected.clone(), shape.clone(), StageAccess::Point),
                (expected.clone(), row_shape, StageAccess::FoldedRow),
            ],
            &(expected, shape),
            |context: &mut LawContext<'_>, values: &[ScalarValueId]| {
                let [exponential, total] = values else {
                    return Err(unsupported("softmax-normalization-operands"));
                };
                let one = scalar_constant(context, F32_ONE_BITS)?;
                let reciprocal = apply_one(context, divide_f32_scalar_op(), &[one, *total])?;
                apply_one(
                    context,
                    multiply_f32_scalar_op(),
                    &[*exponential, reciprocal],
                )
            },
        )?;
    }
    let normalize = normalize.build().map_err(IndexRealizationLawError::Build)?;

    VerifiedIndexRegionSequence::try_new(
        vec![extrema, exponentials, denominator, normalize],
        vec![
            vec![StagedInputSource::Occurrence(0)],
            vec![
                StagedInputSource::Occurrence(0),
                StagedInputSource::Intermediate(0),
            ],
            vec![StagedInputSource::Intermediate(1)],
            vec![
                StagedInputSource::Intermediate(1),
                StagedInputSource::Intermediate(2),
            ],
        ],
    )
    .map_err(IndexRealizationLawError::Sequence)
}

/// Returns the exact binary32 payload of the folded contributor count.
///
/// The reference divides by the extent itself — never by a reciprocal, and
/// therefore never by a divisor that is merely close to it. Above the binary32
/// significand's width the integers are not all representable, so a count whose
/// nearest binary32 is not the count would make the emitted division a different
/// function from the one the operation pins; it is refused rather than rounded.
///
/// The representability test is integer-only, so it does not depend on the
/// rounding it exists to detect: an integer is a binary32 value exactly when its
/// odd part fits in the twenty-four-bit significand.
fn folded_extent_bits(points: u64) -> Result<u32, IndexRealizationLawError> {
    if points == 0 {
        // An empty fold has no first contributor to seed at, so the reference's
        // own fold is undefined here before the division by zero is reached.
        return Err(unsupported("rms-scale-empty-fold"));
    }
    if points >> points.trailing_zeros() >= 1 << 24 {
        return Err(unsupported("rms-scale-extent-not-exact"));
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "the representability test above proves this conversion is exact"
    )]
    let extent = points as f32;
    Ok(extent.to_bits())
}

fn realize_reindex(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    let ([input], [result]) = (context.subject.inputs(), context.subject.results()) else {
        return Err(unsupported("reindex-arity"));
    };
    if context.subject.operands() != [0] {
        return Err(unsupported("reindex-operand-binding"));
    }
    let value = context
        .subject
        .attributes()
        .get(attribute)
        .ok_or_else(|| unsupported("reindex-form-missing"))?;
    let form = ReindexForm::from_canonical_value(value).map_err(|_| unsupported("reindex-form"))?;
    let input_shape = input.shape().clone();
    let result_shape = result.shape().clone();
    if form
        .result_shape(&input_shape)
        .map_err(|_| unsupported("reindex-form"))?
        != result_shape
    {
        return Err(unsupported("reindex-result-shape"));
    }
    let dimensions = declare_parallel_domain(context, &result_shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let operand_coordinates =
        reindex_operand_coordinates(context, &form, &input_shape, &coordinates)?;
    let domain = match form.kind() {
        ReindexFormKind::InsertUnitAxis => {
            let inserted = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            dimensions
                .iter()
                .enumerate()
                .filter(|(position, _)| *position != inserted)
                .map(|(_, dimension)| *dimension)
                .collect()
        }
        _ => dimensions.clone(),
    };
    let tensor = context.tensor(TensorRole::Input, input.value_type().clone(), input_shape)?;
    let value = context.read(tensor, &domain, &operand_coordinates)?;
    let output = context.tensor(
        TensorRole::Output,
        result.value_type().clone(),
        result_shape,
    )?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}
/// Returns whether one occurrence's own boundaries name a declared symbol.
///
/// The third condition that opens the environment-carrying builder, beside the
/// parametric broadcast and the source-bearing slice: a same-shape symbolic
/// pointwise occurrence declares its domain through symbolic dimensions, which
/// the builder admits only against the program's own environment. A neighbour
/// occurrence that merely lives in a program *with* an environment names no
/// symbol on its own boundaries and keeps the environment-free builder, so its
/// identity does not move — the caveat the earlier two conditions already
/// state.
pub(crate) fn subject_boundaries_name_a_symbol(subject: &IndexRefinementSubject) -> bool {
    subject
        .inputs()
        .iter()
        .any(|boundary| boundary.sourced_shape().as_static().is_none())
        || subject
            .results()
            .iter()
            .any(|boundary| boundary.sourced_shape().as_static().is_none())
}

fn slice_subject_is_source_bearing(
    law: &IndexRealizationLaw,
    subject: &IndexRefinementSubject,
) -> bool {
    let IndexRealizationLaw::Slice {
        selection_attribute,
    } = law
    else {
        return false;
    };
    let Some(value) = subject.attributes().get(*selection_attribute) else {
        return false;
    };
    let Ok(selection) = SliceSelection::from_canonical_value(value) else {
        return false;
    };
    selection.names_a_symbol()
}

fn broadcast_subject_is_parametric(
    law: &IndexRealizationLaw,
    subject: &IndexRefinementSubject,
) -> bool {
    let IndexRealizationLaw::Broadcast { mapping_attribute } = law else {
        return false;
    };
    let Some(value) = subject.attributes().get(*mapping_attribute) else {
        return false;
    };
    let Ok(mapping) = BroadcastAxisMapping::from_canonical_value(value) else {
        return false;
    };
    let Some(input) = subject.inputs().first() else {
        return false;
    };
    crate::schedule::mapping_names_a_symbol(input.sourced_shape(), &mapping)
}

fn realize_broadcast(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    let ([input], [result]) = (context.subject.inputs(), context.subject.results()) else {
        return Err(unsupported("broadcast-arity"));
    };
    if context.subject.operands() != [0] {
        return Err(unsupported("broadcast-operand-binding"));
    }
    let value = context
        .subject
        .attributes()
        .get(attribute)
        .ok_or_else(|| unsupported("broadcast-mapping-missing"))?;
    let mapping = BroadcastAxisMapping::from_canonical_value(value)
        .map_err(|_| unsupported("broadcast-mapping"))?;
    if crate::schedule::mapping_names_a_symbol(input.sourced_shape(), &mapping) {
        return realize_parametric_broadcast(context, &mapping, input, result);
    }
    let input_shape = input.shape().clone();
    let result_shape = result.shape().clone();
    if mapping
        .result_shape(&input_shape)
        .map_err(|_| unsupported("broadcast-mapping"))?
        != result_shape
    {
        return Err(unsupported("broadcast-result-shape"));
    }
    let dimensions = declare_parallel_domain(context, &result_shape)?;
    emit_broadcast_coordinates(
        context,
        &mapping,
        input_shape.rank(),
        &dimensions,
        |context| context.tensor(TensorRole::Input, input.value_type().clone(), input_shape),
        |context| {
            context.tensor(
                TensorRole::Output,
                result.value_type().clone(),
                result_shape,
            )
        },
    )
}

fn realize_parametric_broadcast(
    context: &mut LawContext<'_>,
    mapping: &BroadcastAxisMapping,
    input: &IndexRefinementBoundary,
    result: &IndexRefinementBoundary,
) -> Result<(), IndexRealizationLawError> {
    let Some(environment) = context.subject.shape_environment() else {
        return Err(unsupported("broadcast-environment"));
    };
    let sources = ExtentSources::new(std::sync::Arc::clone(environment));
    let applied = mapping
        .apply(input.sourced_shape(), Some(&sources))
        .map_err(|_| unsupported("broadcast-mapping"))?;
    if &applied != result.sourced_shape() {
        return Err(unsupported("broadcast-result-shape"));
    }
    let dimensions = mapping
        .result_extents()
        .iter()
        .cloned()
        .map(|extent| context.sourced_dimension(DomainRole::Parallel, extent))
        .collect::<Result<Vec<_>, _>>()?;
    emit_broadcast_coordinates(
        context,
        mapping,
        input.sourced_shape().rank(),
        &dimensions,
        |context| {
            context.sourced_tensor(
                TensorRole::Input,
                input.value_type().clone(),
                input.sourced_shape(),
            )
        },
        |context| {
            context.sourced_tensor(
                TensorRole::Output,
                result.value_type().clone(),
                result.sourced_shape(),
            )
        },
    )
}

fn emit_broadcast_coordinates(
    context: &mut LawContext<'_>,
    mapping: &BroadcastAxisMapping,
    operand_rank: usize,
    dimensions: &[DimensionId],
    input: impl FnOnce(&mut LawContext<'_>) -> Result<TensorId, IndexRealizationLawError>,
    output: impl FnOnce(&mut LawContext<'_>) -> Result<TensorId, IndexRealizationLawError>,
) -> Result<(), IndexRealizationLawError> {
    let coordinates = dimension_expressions(context, dimensions)?;
    let zero = context.constant(IndexInteger::from_u64(0))?;
    let domain = mapping
        .sources()
        .iter()
        .zip(dimensions)
        .filter(|(source, _)| matches!(source, BroadcastAxisSource::FromOperand(_)))
        .map(|(_, dimension)| *dimension)
        .collect::<Vec<_>>();
    let mut operand_coordinates = vec![None; operand_rank];
    for (result_axis, source) in mapping.sources().iter().enumerate() {
        let Some(axis) = source.operand_axis() else {
            continue;
        };
        let index = axis_position(axis)?;
        let slot = operand_coordinates
            .get_mut(index)
            .ok_or_else(|| unsupported("broadcast-axis"))?;
        let coordinate = match source {
            BroadcastAxisSource::FromOperand(_) => *coordinates
                .get(result_axis)
                .ok_or_else(|| unsupported("broadcast-coordinate"))?,
            BroadcastAxisSource::StretchUnit(_) => zero,
            BroadcastAxisSource::Replicate => unreachable!("replication has no operand axis"),
        };
        if slot.replace(coordinate).is_some() {
            return Err(unsupported("broadcast-axis-repeated"));
        }
    }
    let operand_coordinates = operand_coordinates
        .into_iter()
        .map(|coordinate| coordinate.ok_or_else(|| unsupported("broadcast-axis-unmapped")))
        .collect::<Result<Vec<_>, _>>()?;
    let tensor = input(context)?;
    let value = context.read(tensor, &domain, &operand_coordinates)?;
    let output = output(context)?;
    let write = context.write(output, dimensions, &coordinates)?;
    context.output(write, value)
}

/// Emits the total selection relation of one slice occurrence.
///
/// The occurrence is re-derived rather than trusted: the named field must be the
/// complete attribute record and its selection must derive the declared result
/// shape from the declared operand. That makes every coordinate below a function
/// of semantic facts this law actually read.
///
/// A source-bearing window is realized as `t + C` through
/// [`IndexRegionBuilder::sourced_linear_combination`]: the addend is the
/// selection's own symbol, not a resolved value and not a second cursor input.
/// A literal window stays on the environment-free `d + offset` path so its
/// region bytes do not move.
fn realize_slice(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    let ([input], [result]) = (context.subject.inputs(), context.subject.results()) else {
        return Err(unsupported("slice-arity"));
    };
    if context.subject.operands() != [0] {
        return Err(unsupported("slice-operand-binding"));
    }
    let [field] = context.subject.attributes().fields() else {
        return Err(unsupported("slice-attributes"));
    };
    if field.id() != attribute {
        return Err(unsupported("slice-attribute-key"));
    }
    let selection = SliceSelection::from_canonical_value(field.value())
        .map_err(|_| unsupported("slice-selection"))?;
    if selection.names_a_symbol() {
        return realize_source_bearing_slice(context, &selection, input, result);
    }
    let input_shape = input.shape().clone();
    let result_shape = result.shape().clone();
    if selection
        .result_shape(&input_shape)
        .map_err(|_| unsupported("slice-selection"))?
        != result_shape
    {
        return Err(unsupported("slice-result-shape"));
    }

    let dimensions = declare_parallel_domain(context, &result_shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let mut operand_coordinates = Vec::with_capacity(selection.axes().len());
    for (selection, coordinate) in selection.axes().iter().zip(&coordinates) {
        operand_coordinates.push(match selection {
            SliceAxisSelection::WholeAxis => *coordinate,
            SliceAxisSelection::Window { offset, .. } => {
                let Some(literal) = offset.as_static() else {
                    return Err(unsupported("slice-symbolic-offset"));
                };
                context.linear_combination(
                    IndexInteger::from_u64(literal.get()),
                    &[(IndexInteger::from_u64(1), *coordinate)],
                )?
            }
        });
    }

    let tensor = context.tensor(TensorRole::Input, input.value_type().clone(), input_shape)?;
    let value = context.read(tensor, &dimensions, &operand_coordinates)?;
    let output = context.tensor(
        TensorRole::Output,
        result.value_type().clone(),
        result_shape,
    )?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}

fn realize_source_bearing_slice(
    context: &mut LawContext<'_>,
    selection: &SliceSelection,
    input: &IndexRefinementBoundary,
    result: &IndexRefinementBoundary,
) -> Result<(), IndexRealizationLawError> {
    let Some(environment) = context.subject.shape_environment() else {
        return Err(unsupported("slice-environment"));
    };
    let sources = ExtentSources::new(std::sync::Arc::clone(environment));
    let applied = selection
        .apply(input.sourced_shape(), Some(&sources))
        .map_err(|_| unsupported("slice-selection"))?;
    if &applied != result.sourced_shape() {
        return Err(unsupported("slice-result-shape"));
    }

    let dimensions = applied
        .extents()
        .map(|extent| context.sourced_dimension(DomainRole::Parallel, extent))
        .collect::<Result<Vec<_>, _>>()?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let mut operand_coordinates = Vec::with_capacity(selection.axes().len());
    for (axis, coordinate) in selection.axes().iter().zip(&coordinates) {
        operand_coordinates.push(match axis {
            SliceAxisSelection::WholeAxis => *coordinate,
            SliceAxisSelection::Window { offset, .. } => context.sourced_linear_combination(
                offset.clone().into(),
                &[(SourcedIndexInteger::from(1_u64), *coordinate)],
            )?,
        });
    }

    let tensor = context.sourced_tensor(
        TensorRole::Input,
        input.value_type().clone(),
        input.sourced_shape(),
    )?;
    let value = context.read(tensor, &dimensions, &operand_coordinates)?;
    let output = context.sourced_tensor(
        TensorRole::Output,
        result.value_type().clone(),
        result.sourced_shape(),
    )?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}

/// Emits the partitioned write realizing one concatenation.
///
/// The occurrence is re-derived rather than trusted: the axis attribute must
/// produce exactly this result from exactly these operands, so a subject whose
/// declared result disagrees with the family's own derivation is refused instead
/// of realized as a different join.
fn realize_concatenate(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    // Unreachable from a verified occurrence, and stated under the
    // unreachable-refusal convention at `IndexRealizationLaw`: every registered
    // family declares exactly one result, so no subject `derive` can build today
    // carries a second one or none. The graph admits up to
    // `MAX_OPERATION_RESULTS` of them, so the first multi-result family to be
    // registered makes this reachable — and a join that realized the first of
    // several results as though it were the whole occurrence would be exactly
    // the silent wrongness this refusal exists to prevent.
    let [result] = context.subject.results() else {
        return Err(unsupported("concatenate-result-arity"));
    };
    let result_type = result.value_type().clone();
    let result_shape = result.shape().clone();
    // Exactly the one field the law names, as the two staged templates demand of
    // their own records: an occurrence carrying more than the axis would be
    // realized while part of its identity went unread.
    let [field] = context.subject.attributes().fields() else {
        return Err(unsupported("concatenate-attributes"));
    };
    if field.id() != attribute {
        return Err(unsupported("concatenate-attribute-key"));
    }
    let axis = concatenate_axis(field.value()).map_err(|_| unsupported("concatenate-axis"))?;
    let inputs = context
        .subject
        .inputs()
        .iter()
        .map(|input| (input.value_type().clone(), input.shape().clone()))
        .collect::<Vec<_>>();
    let operands = context.subject.operands().to_vec();
    let plan = ConcatenatePlan::derive(axis, &inputs, &operands, &result_shape)?;
    emit_partitioned_concatenate(
        context,
        &plan,
        &inputs,
        &operands,
        result_type,
        result_shape,
    )
}

/// The per-operand partition of one concatenation's concatenated axis.
struct ConcatenatePlan {
    /// Position of the concatenated axis in the result's axis order.
    position: usize,
    /// One `(extent, offset)` pair per *operand*, in operand order.
    ///
    /// Keyed by operand rather than by distinct input because operand order is
    /// semantic and one input may be joined to itself: `concat(x, x)` has one
    /// boundary and two partition members at two different offsets.
    members: Vec<(Extent, u64)>,
}

impl ConcatenatePlan {
    /// Re-derives the partition from the axis and the operands' own shapes.
    ///
    /// **Both of this function's refusals are unreachable from a verified
    /// occurrence, and both are stated under the unreachable-refusal convention
    /// at [`IndexRealizationLaw`].**
    ///
    /// `concatenate-operand-binding` refuses an operand position outside the
    /// input boundaries. [`IndexRefinementSubject::derive`] builds those
    /// positions *as* indices into the boundary list it collects — one boundary
    /// per distinct operand value — so no subject it produces can carry an
    /// out-of-range one. The subject's own types express one, and a later
    /// producer or a re-read is what would present it; a law that indexed
    /// blindly would then read some other operand's shape as this member's.
    ///
    /// `concatenate-result-shape` refuses a re-derived result the declared one
    /// disagrees with. `tiler::concatenate-f32@1`'s inferencer derives the
    /// declared result with this same `concatenate_result_shape` call over the
    /// same ordered operand shapes, so for a verified occurrence the two agree
    /// by construction. The rule's three further sites are downstream of that
    /// agreement rather than independent checks: the derivation just proved the
    /// axis within every operand's shared rank and the exact prefix sum
    /// representable, so neither the extent lookup nor the accumulation below
    /// can fail once it has succeeded.
    fn derive(
        axis: Axis,
        inputs: &[(ResolvedValueType, Shape)],
        operands: &[usize],
        result_shape: &Shape,
    ) -> Result<Self, IndexRealizationLawError> {
        let mut shapes = Vec::with_capacity(operands.len());
        for operand in operands {
            let (_, shape) = inputs
                .get(*operand)
                .ok_or_else(|| unsupported("concatenate-operand-binding"))?;
            shapes.push(shape);
        }
        if concatenate_result_shape(axis, &shapes)
            .map_err(|_| unsupported("concatenate-result-shape"))?
            != *result_shape
        {
            return Err(unsupported("concatenate-result-shape"));
        }
        let position = axis_position(axis)?;
        // Every prefix is bounded by the result extent the derivation above just
        // proved representable, so this accumulation refuses under that same rule
        // rather than under one of its own.
        let mut offset = 0_u64;
        let mut members = Vec::with_capacity(shapes.len());
        for shape in shapes {
            let extent = *shape
                .extents()
                .get(position)
                .ok_or_else(|| unsupported("concatenate-result-shape"))?;
            members.push((extent, offset));
            offset = offset
                .checked_add(extent.get())
                .ok_or_else(|| unsupported("concatenate-result-shape"))?;
        }
        Ok(Self { position, members })
    }
}

fn emit_partitioned_concatenate(
    context: &mut LawContext<'_>,
    plan: &ConcatenatePlan,
    inputs: &[(ResolvedValueType, Shape)],
    operands: &[usize],
    result_type: ResolvedValueType,
    result_shape: Shape,
) -> Result<(), IndexRealizationLawError> {
    let shared = declare_shared_concatenate_domain(context, &result_shape, plan.position)?;
    let mut tensors = Vec::with_capacity(inputs.len());
    for (value_type, shape) in inputs {
        tensors.push(context.tensor(TensorRole::Input, value_type.clone(), shape.clone())?);
    }
    let output = context.tensor(TensorRole::Output, result_type, result_shape)?;
    for (operand, (extent, offset)) in operands.iter().zip(&plan.members) {
        let own = context.dimension(DomainRole::Parallel, *extent)?;
        let mut domain = Vec::with_capacity(shared.len());
        let mut read_coordinates = Vec::with_capacity(shared.len());
        for slot in &shared {
            let dimension = slot.unwrap_or(own);
            domain.push(dimension);
            read_coordinates.push(context.dimension_expr(dimension)?);
        }
        let displaced = context.linear_combination(
            IndexInteger::from_u64(*offset),
            &[(IndexInteger::from_u64(1), read_coordinates[plan.position])],
        )?;
        let mut write_coordinates = read_coordinates.clone();
        write_coordinates[plan.position] = displaced;
        // The second site of the same unreachable rule, for the same reason
        // `ConcatenatePlan::derive` states: a subject's operand positions are
        // built as indices into its own boundary list, and this tensor list is
        // one entry per boundary in that order.
        let tensor = *tensors
            .get(*operand)
            .ok_or_else(|| unsupported("concatenate-operand-binding"))?;
        let value = context.read(tensor, &domain, &read_coordinates)?;
        let write = context.write(output, &domain, &write_coordinates)?;
        context.output(write, value)?;
    }
    Ok(())
}

/// Declares one parallel dimension per *non*-concatenated axis, in axis order.
///
/// The concatenated axis's slot is left empty because each root supplies its own
/// dimension there; every other slot is shared by every root, which is the
/// region's statement of the extent agreement the family admits an occurrence on.
fn declare_shared_concatenate_domain(
    context: &mut LawContext<'_>,
    result_shape: &Shape,
    position: usize,
) -> Result<Vec<Option<DimensionId>>, IndexRealizationLawError> {
    let mut shared = Vec::with_capacity(result_shape.rank());
    for (index, extent) in result_shape.extents().iter().enumerate() {
        shared.push(if index == position {
            None
        } else {
            Some(context.dimension(DomainRole::Parallel, *extent)?)
        });
    }
    Ok(shared)
}

fn realize_contraction(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    let plan = ContractionPlan::derive(context.subject, attribute)?;
    let output = declare_parallel_domain(context, &plan.output_shape)?;
    let output_coordinates = dimension_expressions(context, &output)?;
    let mut tensors = Vec::with_capacity(plan.operand_shapes.len());
    for shape in &plan.operand_shapes {
        tensors.push(context.tensor(TensorRole::Input, plan.value_type.clone(), shape.clone())?);
    }
    let result = context.tensor(
        TensorRole::Output,
        plan.value_type.clone(),
        plan.output_shape.clone(),
    )?;
    let seed = plan.product(context, &tensors, &output, &output_coordinates, None)?;
    let total = if plan.contracted_points == 1 {
        seed
    } else {
        let tail = context.dimension(
            DomainRole::Reduction,
            Extent::new(plan.contracted_points - 1),
        )?;
        let contributor =
            plan.product(context, &tensors, &output, &output_coordinates, Some(tail))?;
        let folded = context.reduce(&[tail], &[seed], &[contributor], |body| {
            combine_with(body, add_f32_scalar_op())
        })?;
        single_result(&folded, "contraction")?
    };
    let write = context.write(result, &output, &output_coordinates)?;
    context.output(write, total)
}

fn axis_position(axis: Axis) -> Result<usize, IndexRealizationLawError> {
    usize::try_from(axis.get()).map_err(|_| unsupported("axis-width"))
}

/// One strict lexicographic left fold over a rectangular reduced sub-domain.
///
/// **Parameterized by its combiner and its seeding rule**, because the two folds
/// this vocabulary must emit differ in exactly those two things and in nothing
/// else. The sum combines with the governed addition and has `0.0` to seed an
/// empty contributor domain with; the softmax's row maximum combines with the
/// NaN-propagating extrema family, which has *no* identity at all and is
/// therefore seeded at the first contributor with an empty domain refused rather
/// than invented. Everything else — the kept domain, the reduced linearization,
/// the one-contributor case, the tail reduction — is one emitter.
struct FoldPlan {
    value_type: ResolvedValueType,
    input_shape: Shape,
    output_shape: Shape,
    reduced: Vec<bool>,
    reduced_strides: Vec<u64>,
    reduced_extents: Vec<u64>,
    reduced_points: u64,
    /// Scalar each contributor is squared with before the fold combines it.
    ///
    /// `None` folds the operand's own elements, which is every plain strict
    /// serial sum. `Some(scalar)` applies `scalar(v, v)` to each contributor
    /// first, which is the only per-contributor transform the registered scalar
    /// vocabulary can spell as one application to one read value — a wider
    /// prologue would need a scalar-program language in law data, which this
    /// module deliberately does not have.
    contributor_square: Option<ScalarOpKey>,
    /// Binary scalar the reducer body combines state and contributor with.
    combiner: ScalarOpKey,
    /// Exact binary32 payload seeding an empty contributor domain.
    ///
    /// `Some(bits)` is a combiner with an identity, and it is the *only* value a
    /// fold over no contributors may commit. `None` is a combiner with none, so
    /// an empty domain has no first contributor to seed at and no identity to
    /// stand in for one; the fold refuses rather than choosing a seed the
    /// reference it realizes does not have.
    empty_identity: Option<u32>,
}

impl FoldPlan {
    fn derive(
        subject: &IndexRefinementSubject,
        attribute: AttributeFieldId,
    ) -> Result<Self, IndexRealizationLawError> {
        let ([input], [result]) = (subject.inputs(), subject.results()) else {
            return Err(unsupported("sum-arity"));
        };
        if subject.operands() != [0] {
            return Err(unsupported("sum-operand-binding"));
        }
        let axes = reduction_axes(subject.attributes(), attribute)?;
        Self::for_boundaries(input.value_type(), input.shape(), result.shape(), &axes)
    }

    /// Derives the fold plan from explicit boundaries rather than the subject.
    ///
    /// A staged realization's fold publishes an intermediate, which is no
    /// occurrence result, so the output shape it must produce is a parameter.
    fn for_boundaries(
        value_type: &ResolvedValueType,
        input_shape: &Shape,
        output_shape: &Shape,
        axes: &[Axis],
    ) -> Result<Self, IndexRealizationLawError> {
        let input_shape = input_shape.clone();
        let mut reduced = vec![false; input_shape.rank()];
        for axis in axes {
            let index = axis_position(*axis)?;
            let slot = reduced
                .get_mut(index)
                .ok_or_else(|| unsupported("sum-axis-range"))?;
            if std::mem::replace(slot, true) {
                return Err(unsupported("sum-axis-duplicate"));
            }
        }
        if &input_shape.without_axes(axes) != output_shape {
            return Err(unsupported("sum-result-shape"));
        }
        let reduced_extents = input_shape
            .extents()
            .iter()
            .zip(&reduced)
            .filter(|(_, reduced)| **reduced)
            .map(|(extent, _)| extent.get())
            .collect::<Vec<_>>();
        let mut reduced_strides = vec![0_u64; reduced_extents.len()];
        let mut stride = 1_u64;
        for (position, extent) in reduced_extents.iter().enumerate().rev() {
            reduced_strides[position] = stride;
            stride = stride
                .checked_mul(*extent)
                .ok_or_else(|| unsupported("sum-reduced-extent-overflow"))?;
        }
        Ok(Self {
            value_type: value_type.clone(),
            input_shape,
            output_shape: output_shape.clone(),
            reduced,
            reduced_strides,
            reduced_extents,
            reduced_points: stride,
            contributor_square: None,
            // The governed addition and its identity, which is what every fold
            // this emitter produced before the combiner was a parameter used.
            combiner: add_f32_scalar_op(),
            empty_identity: Some(0.0_f32.to_bits()),
        })
    }

    /// Folds `scalar(v, v)` per contributor rather than the contributor itself.
    fn squaring_contributors(mut self, scalar: ScalarOpKey) -> Self {
        self.contributor_square = Some(scalar);
        self
    }

    /// Combines with `combiner`, seeding an empty domain from `empty_identity`.
    ///
    /// Both are set together because they are one decision: an identity is a
    /// property of the combiner, so a plan carrying one combiner's identity
    /// beside another's combiner would fold an empty domain to a value that
    /// operation never produces.
    fn combining(mut self, combiner: ScalarOpKey, empty_identity: Option<u32>) -> Self {
        self.combiner = combiner;
        self.empty_identity = empty_identity;
        self
    }

    /// Emits the complete fold and returns its value, writing nothing.
    ///
    /// Three cases, and the split is over the *contributor population* rather
    /// than over the combiner: no contributor needs the identity this plan may
    /// not have, one contributor needs no combine at all and reaches the
    /// reduction's result boundary through the canonicalization the numerical
    /// contract places there, and two or more are the seeded tail fold.
    fn fold(
        &self,
        context: &mut LawContext<'_>,
        input: TensorId,
        kept: &[DimensionId],
        kept_coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        if self.reduced_points == 0 {
            return self.fold_empty(context, input, kept, kept_coordinates);
        }
        let seed = self.read_contributor(context, input, kept, kept_coordinates, None)?;
        if self.reduced_points == 1 {
            apply_one(context, canonicalize_nan_f32_scalar_op(), &[seed])
        } else {
            self.fold_tail(context, input, kept, kept_coordinates, seed)
        }
    }

    /// Applies the per-contributor square, when this plan carries one.
    fn square(
        &self,
        context: &mut LawContext<'_>,
        contributor: ScalarValueId,
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        match &self.contributor_square {
            Some(scalar) => apply_one(context, scalar.clone(), &[contributor, contributor]),
            None => Ok(contributor),
        }
    }

    fn declare_kept_domain(
        &self,
        context: &mut LawContext<'_>,
    ) -> Result<Vec<DimensionId>, IndexRealizationLawError> {
        self.input_shape
            .extents()
            .iter()
            .zip(&self.reduced)
            .filter(|(_, reduced)| !**reduced)
            .map(|(extent, _)| context.dimension(DomainRole::Parallel, *extent))
            .collect()
    }

    fn read_contributor(
        &self,
        context: &mut LawContext<'_>,
        input: TensorId,
        kept: &[DimensionId],
        kept_coordinates: &[IndexExprId],
        tail: Option<DimensionId>,
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        let offset = match tail {
            Some(tail) => {
                let induction = context.dimension_expr(tail)?;
                let one = IndexInteger::from_u64(1);
                Some(context.linear_combination(one.clone(), &[(one, induction)])?)
            }
            None => None,
        };
        let zero = context.constant(IndexInteger::from_u64(0))?;
        let mut coordinates = Vec::with_capacity(self.input_shape.rank());
        let mut kept_position = 0;
        let mut reduced_position = 0;
        for reduced in &self.reduced {
            if *reduced {
                coordinates.push(match offset {
                    Some(offset) => self.decode_reduced(context, offset, reduced_position)?,
                    None => zero,
                });
                reduced_position += 1;
            } else {
                coordinates.push(kept_coordinates[kept_position]);
                kept_position += 1;
            }
        }
        let mut domain = kept.to_vec();
        domain.extend(tail);
        let contributor = context.read(input, &domain, &coordinates)?;
        self.square(context, contributor)
    }

    fn decode_reduced(
        &self,
        context: &mut LawContext<'_>,
        offset: IndexExprId,
        position: usize,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        let stride = self.reduced_strides[position];
        let extent = self.reduced_extents[position];
        let wrapped = if position == 0 {
            offset
        } else {
            let modulus = stride
                .checked_mul(extent)
                .ok_or_else(|| unsupported("sum-reduced-extent-overflow"))?;
            context.modulo(offset, SourcedExtent::Static(Extent::new(modulus)))?
        };
        if stride == 1 {
            Ok(wrapped)
        } else {
            context.floor_div(wrapped, SourcedExtent::Static(Extent::new(stride)))
        }
    }

    fn fold_empty(
        &self,
        context: &mut LawContext<'_>,
        input: TensorId,
        kept: &[DimensionId],
        kept_coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        let Some(identity_bits) = self.empty_identity else {
            return Err(unsupported("fold-empty-domain-without-identity"));
        };
        let mut reduced_dimensions = Vec::new();
        let mut coordinates = Vec::with_capacity(self.input_shape.rank());
        let mut kept_position = 0;
        for (extent, reduced) in self.input_shape.extents().iter().zip(&self.reduced) {
            if *reduced {
                let dimension = context.dimension(DomainRole::Reduction, *extent)?;
                coordinates.push(context.dimension_expr(dimension)?);
                reduced_dimensions.push(dimension);
            } else {
                coordinates.push(kept_coordinates[kept_position]);
                kept_position += 1;
            }
        }
        let mut domain = kept.to_vec();
        domain.extend(reduced_dimensions.iter().copied());
        let contributor = context.read(input, &domain, &coordinates)?;
        let contributor = self.square(context, contributor)?;
        let identity = scalar_constant(context, identity_bits)?;
        let folded = context.reduce(&reduced_dimensions, &[identity], &[contributor], |body| {
            combine_with(body, self.combiner.clone())
        })?;
        single_result(&folded, "reduction")
    }

    fn fold_tail(
        &self,
        context: &mut LawContext<'_>,
        input: TensorId,
        kept: &[DimensionId],
        kept_coordinates: &[IndexExprId],
        seed: ScalarValueId,
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        let tail = context.dimension(
            DomainRole::Reduction,
            Extent::new(self.reduced_points.saturating_sub(1)),
        )?;
        let contributor =
            self.read_contributor(context, input, kept, kept_coordinates, Some(tail))?;
        let folded = context.reduce(&[tail], &[seed], &[contributor], |body| {
            combine_with(body, self.combiner.clone())
        })?;
        single_result(&folded, "reduction")
    }
}

/// Fills one reducer body with the single-state combine `state = key(state, v)`.
///
/// The combiner is a parameter rather than the governed addition because the
/// extrema fold this vocabulary now emits is the same body shape with a
/// different scalar; nothing else about a one-state left fold varies between
/// them.
fn combine_with(
    body: &mut ScalarReducerBodyBuilder<'_>,
    key: ScalarOpKey,
) -> Result<(), IndexBuildError> {
    let state = body.state(0).expect("one state");
    let contributor = body.contributor(0).expect("one contributor");
    let accumulated = body
        .apply(key, ScalarAttributes::empty(), &[state, contributor])?
        .get(0)
        .expect("a governed binary combiner has one result");
    body.yield_values(&[accumulated])
}

fn reduction_axes(
    attributes: &OperationAttributes,
    attribute: AttributeFieldId,
) -> Result<Vec<Axis>, IndexRealizationLawError> {
    let value = attributes
        .get(attribute)
        .ok_or_else(|| unsupported("sum-axes-missing"))?;
    let CanonicalValueView::Sequence(values) = value.view() else {
        return Err(unsupported("sum-axes-kind"));
    };
    values
        .iter()
        .map(|value| {
            let CanonicalValueView::Unsigned {
                width: CanonicalIntegerWidth::Bits32,
                bits,
            } = value.view()
            else {
                return Err(unsupported("sum-axes-element"));
            };
            Ok(Axis::new(
                u32::try_from(bits).map_err(|_| unsupported("sum-axes-width"))?,
            ))
        })
        .collect()
}

struct ContractionPlan {
    value_type: ResolvedValueType,
    operand_shapes: Vec<Shape>,
    output_shape: Shape,
    sources: Vec<Vec<AxisSource>>,
    contracted_strides: Vec<u64>,
    contracted_extents: Vec<u64>,
    contracted_points: u64,
}

#[derive(Clone, Copy)]
enum AxisSource {
    Output(usize),
    Contracted(usize),
}

impl ContractionPlan {
    fn derive(
        subject: &IndexRefinementSubject,
        attribute: AttributeFieldId,
    ) -> Result<Self, IndexRealizationLawError> {
        let ([left, right], [result]) = (subject.inputs(), subject.results()) else {
            return Err(unsupported("contraction-arity"));
        };
        if subject.operands() != [0, 1] {
            return Err(unsupported("contraction-operand-binding"));
        }
        let [field] = subject.attributes().fields() else {
            return Err(unsupported("contraction-attributes"));
        };
        if field.id() != attribute {
            return Err(unsupported("contraction-attributes"));
        }
        let structure = ContractionIndexStructure::from_canonical_value(field.value())
            .map_err(|_| unsupported("contraction-structure"))?;
        let boundaries = [left, right];
        if structure.operand_count() != boundaries.len() {
            return Err(unsupported("contraction-operand-count"));
        }
        let mut extents: Vec<(ContractionIndex, Extent)> = Vec::new();
        let mut operand_shapes = Vec::with_capacity(boundaries.len());
        for (tuple, boundary) in structure.operands().zip(boundaries) {
            let shape = boundary.shape().clone();
            if shape.rank() != tuple.len() {
                return Err(unsupported("contraction-rank"));
            }
            for (axis, index) in tuple.iter().enumerate() {
                let extent = shape.extents()[axis];
                match extents.iter().find(|(bound, _)| bound == index) {
                    Some((_, bound)) if *bound != extent => {
                        return Err(unsupported("contraction-extent"));
                    }
                    Some(_) => {}
                    None => extents.push((*index, extent)),
                }
            }
            operand_shapes.push(shape);
        }
        let shape_over = |indices: &[ContractionIndex]| {
            Shape::try_new(
                indices
                    .iter()
                    .map(|index| {
                        extents
                            .iter()
                            .find(|(bound, _)| bound == index)
                            .map(|(_, extent)| *extent)
                            .ok_or_else(|| unsupported("contraction-extent"))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .map_err(|_| unsupported("contraction-shape"))
        };
        let output_shape = shape_over(structure.output())?;
        if &output_shape != result.shape() {
            return Err(unsupported("contraction-result-shape"));
        }
        let contracted_shape = shape_over(structure.contracted())?;
        let mut sources = Vec::with_capacity(structure.operand_count());
        for tuple in structure.operands() {
            let mut operand = Vec::with_capacity(tuple.len());
            for index in tuple {
                if let Some(position) = structure
                    .output()
                    .iter()
                    .position(|candidate| candidate == index)
                {
                    operand.push(AxisSource::Output(position));
                } else if let Some(position) = structure
                    .contracted()
                    .iter()
                    .position(|candidate| candidate == index)
                {
                    operand.push(AxisSource::Contracted(position));
                } else {
                    return Err(unsupported("contraction-index"));
                }
            }
            sources.push(operand);
        }
        let contracted_extents = contracted_shape
            .extents()
            .iter()
            .map(|extent| extent.get())
            .collect::<Vec<_>>();
        let mut contracted_strides = vec![0_u64; contracted_extents.len()];
        let mut stride = 1_u64;
        for (position, extent) in contracted_extents.iter().enumerate().rev() {
            contracted_strides[position] = stride;
            stride = stride
                .checked_mul(*extent)
                .ok_or_else(|| unsupported("contraction-extent-overflow"))?;
        }
        if stride == 0 {
            return Err(unsupported("contraction-empty-domain"));
        }
        Ok(Self {
            value_type: result.value_type().clone(),
            operand_shapes,
            output_shape,
            sources,
            contracted_strides,
            contracted_extents,
            contracted_points: stride,
        })
    }

    fn product(
        &self,
        context: &mut LawContext<'_>,
        tensors: &[TensorId],
        output: &[DimensionId],
        output_coordinates: &[IndexExprId],
        tail: Option<DimensionId>,
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        let offset = match tail {
            Some(tail) => {
                let induction = context.dimension_expr(tail)?;
                let one = IndexInteger::from_u64(1);
                Some(context.linear_combination(one.clone(), &[(one, induction)])?)
            }
            None => None,
        };
        let zero = context.constant(IndexInteger::from_u64(0))?;
        let mut domain = output.to_vec();
        domain.extend(tail);
        let mut values = Vec::with_capacity(tensors.len());
        for (position, tensor) in tensors.iter().enumerate() {
            let coordinates = self.sources[position]
                .iter()
                .map(|source| match source {
                    AxisSource::Output(axis) => output_coordinates
                        .get(*axis)
                        .copied()
                        .ok_or_else(|| unsupported("contraction-coordinate")),
                    AxisSource::Contracted(axis) => match offset {
                        Some(offset) => self.decode_contracted(context, offset, *axis),
                        None => Ok(zero),
                    },
                })
                .collect::<Result<Vec<_>, _>>()?;
            values.push(context.read(*tensor, &domain, &coordinates)?);
        }
        apply_one(context, multiply_f32_scalar_op(), &values)
    }

    fn decode_contracted(
        &self,
        context: &mut LawContext<'_>,
        offset: IndexExprId,
        position: usize,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        let stride = self.contracted_strides[position];
        let extent = self.contracted_extents[position];
        let wrapped = if position == 0 {
            offset
        } else {
            let modulus = stride
                .checked_mul(extent)
                .ok_or_else(|| unsupported("contraction-extent-overflow"))?;
            context.modulo(offset, SourcedExtent::Static(Extent::new(modulus)))?
        };
        if stride == 1 {
            Ok(wrapped)
        } else {
            context.floor_div(wrapped, SourcedExtent::Static(Extent::new(stride)))
        }
    }
}

fn reindex_operand_coordinates(
    context: &mut LawContext<'_>,
    form: &ReindexForm,
    input_shape: &Shape,
    coordinates: &[IndexExprId],
) -> Result<Vec<IndexExprId>, IndexRealizationLawError> {
    let extents = input_shape.extents();
    let at = |position: usize| {
        coordinates
            .get(position)
            .copied()
            .ok_or_else(|| unsupported("reindex-coordinate"))
    };
    match form.kind() {
        ReindexFormKind::PermuteAxes => {
            let mut operand = vec![None; extents.len()];
            for (position, axis) in form.axes().iter().enumerate() {
                let slot = operand
                    .get_mut(axis_position(*axis)?)
                    .ok_or_else(|| unsupported("reindex-axis"))?;
                if slot.replace(at(position)?).is_some() {
                    return Err(unsupported("reindex-permutation"));
                }
            }
            operand
                .into_iter()
                .map(|value| value.ok_or_else(|| unsupported("reindex-permutation")))
                .collect()
        }
        ReindexFormKind::SplitAxis => {
            let axis = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            let factors = form.factors();
            let mut strides = vec![1_u64; factors.len()];
            let mut stride = 1_u64;
            for (position, factor) in factors.iter().enumerate().rev() {
                strides[position] = stride;
                stride = stride
                    .checked_mul(factor.get())
                    .ok_or_else(|| unsupported("reindex-split-overflow"))?;
            }
            let terms = strides
                .iter()
                .enumerate()
                .map(|(position, stride)| {
                    Ok((IndexInteger::from_u64(*stride), at(axis + position)?))
                })
                .collect::<Result<Vec<_>, IndexRealizationLawError>>()?;
            let linearized = context.linear_combination(IndexInteger::from_u64(0), &terms)?;
            (0..extents.len())
                .map(|position| match position.cmp(&axis) {
                    std::cmp::Ordering::Less => at(position),
                    std::cmp::Ordering::Equal => Ok(linearized),
                    std::cmp::Ordering::Greater => at(position + factors.len() - 1),
                })
                .collect()
        }
        ReindexFormKind::MergeAxes => {
            let axes = form.axes();
            let first = axis_position(*axes.first().ok_or_else(|| unsupported("reindex-axis"))?)?;
            let merged = axes
                .iter()
                .map(|axis| {
                    extents
                        .get(axis_position(*axis)?)
                        .map(|extent| extent.get())
                        .ok_or_else(|| unsupported("reindex-axis"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut strides = vec![1_u64; merged.len()];
            let mut stride = 1_u64;
            for (position, extent) in merged.iter().enumerate().rev() {
                strides[position] = stride;
                stride = stride
                    .checked_mul(*extent)
                    .ok_or_else(|| unsupported("reindex-merge-overflow"))?;
            }
            let linear = at(first)?;
            let mut decoded = Vec::with_capacity(merged.len());
            for position in 0..merged.len() {
                let wrapped = if position == 0 {
                    linear
                } else {
                    let modulus = strides[position]
                        .checked_mul(merged[position])
                        .ok_or_else(|| unsupported("reindex-merge-overflow"))?;
                    context.modulo(linear, SourcedExtent::Static(Extent::new(modulus)))?
                };
                decoded.push(if strides[position] == 1 {
                    wrapped
                } else {
                    context.floor_div(
                        wrapped,
                        SourcedExtent::Static(Extent::new(strides[position])),
                    )?
                });
            }
            (0..extents.len())
                .map(|position| {
                    if position < first {
                        at(position)
                    } else if position < first + merged.len() {
                        Ok(decoded[position - first])
                    } else {
                        at(position - merged.len() + 1)
                    }
                })
                .collect()
        }
        ReindexFormKind::InsertUnitAxis => {
            let inserted = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            (0..extents.len())
                .map(|axis| at(if axis < inserted { axis } else { axis + 1 }))
                .collect()
        }
        ReindexFormKind::RemoveUnitAxis => {
            let removed = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            let zero = context.constant(IndexInteger::from_u64(0))?;
            (0..extents.len())
                .map(|axis| match axis.cmp(&removed) {
                    std::cmp::Ordering::Less => at(axis),
                    std::cmp::Ordering::Equal => Ok(zero),
                    std::cmp::Ordering::Greater => at(axis - 1),
                })
                .collect()
        }
        ReindexFormKind::ReverseAxis => {
            let reversed = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            let last = i128::from(
                extents
                    .get(reversed)
                    .ok_or_else(|| unsupported("reindex-axis"))?
                    .get(),
            )
            .checked_sub(1)
            .ok_or_else(|| unsupported("reindex-reverse-extent"))?;
            let mirrored = context.linear_combination(
                IndexInteger::from_i128(last),
                &[(IndexInteger::from_i128(-1), at(reversed)?)],
            )?;
            (0..extents.len())
                .map(|axis| {
                    if axis == reversed {
                        Ok(mirrored)
                    } else {
                        at(axis)
                    }
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{
        AccessMode, BoundsProofView, FrozenScalarRegistry, IndexDomainFactSource,
        IndexDomainUnknownReason, IndexExprView, IndexRegionBuildError, IndexRegionDiagnostic,
        JointPartitionProofView, NumericalContractIdentity, ScalarOperationKindRef,
        ScalarOperationRef, ScalarValueDefinitionView, VerifiedScalarOperationId,
        VerifiedScalarValueId, WriteOwnershipProofView,
    };
    use crate::semantic::{
        F32, F32Slice, InputKey, OperationAttributes, OutputKey, RMS_NORM_F32_REFERENCE_EPS_BITS,
        SemanticProgramBuilder, SliceAxisSelection, SliceSelection, StrictAffineU8,
        concatenate_f32_axis_attribute, concatenate_f32_op, dequantize_strict_affine_op,
        rms_norm_f32_axis_attribute, rms_norm_f32_eps_attribute, rms_norm_f32_op,
        softmax_f32_axis_attribute, softmax_f32_op,
    };
    use crate::shape::{
        BindingSource, EXTENT_PHASE_CEILING, Extent, ExtentRelation, ExtentTerm, FactProvenance,
        RootBinding, SemanticInputConstraint, ShapeEnv, ShapeEnvBuilder, ShapeSymbol,
        SourcedExtent, SymbolScope,
    };

    /// Domain separating this file's identity pins from every governed digest.
    ///
    /// A pin needs a stable pre-image and nothing else; it is deliberately not
    /// one of the identity domains the workspace publishes, so a fixture digest
    /// can never be mistaken for evidence about a subject the compiler derives.
    const SEQUENCE_IDENTITY_PIN_DOMAIN: &[u8] = b"tiler.test.index-region-sequence-identity-pin\0";

    fn strict_contract() -> NumericalContractIdentity {
        NumericalContractIdentity::from(
            F32NumericalContractKey::new(
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
                MaterializationRounding::NearestTiesToEven,
            )
            .unwrap(),
        )
    }

    fn subject(value_type: ResolvedValueType) -> IndexRefinementSubject {
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let input = program
            .input_resolved(
                InputKey::new("encoded").unwrap(),
                Shape::from_dims([5]),
                value_type,
            )
            .unwrap();
        let result = program
            .apply(
                dequantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[input],
            )
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap()
    }

    /// Every one-reader chain's identity is unchanged, byte for byte.
    ///
    /// **The pin is over exact bytes, taken before either widening landed.** The
    /// three digests below were captured on base commit `dd9def76` — that is,
    /// before `tiler.scalar::maximum-f32@1` was registered and before
    /// [`VerifiedIndexRegionSequence::try_new`](crate::index::VerifiedIndexRegionSequence::try_new)
    /// admitted a value with more than one reader — and they are asserted here
    /// unchanged. The length is pinned beside the digest so a chain that moved
    /// reports *how* rather than only that it did.
    ///
    /// **One pin covers both widenings, and that is the point of pinning here
    /// rather than in `sequence.rs`.** A realized law's sequence identity is
    /// built from its regions' canonical identities, and a region's identity
    /// carries the projection of the scalar definitions it *reaches*. So a new
    /// scalar key that no existing law emits must leave every byte alone, and a
    /// wider admitted chain shape that no existing law spells must leave every
    /// byte alone; both claims fail here if either is false.
    ///
    /// The rows are the two live instances plus the plain staged template: the
    /// normalization's own law at rank two and at rank one, and the
    /// fold-then-pointwise form no registered operation carries.
    ///
    /// **Rebaselined twice, and each time for a reason the invariant above
    /// predicts rather than an exception to it.**
    ///
    /// At the `tiler.index-region.v10` step,
    /// `admit-symbolic-index-expression-coefficients` made a linear
    /// combination's coefficient a tagged `SourcedIndexInteger` where `v9`
    /// wrote a bare integer, so every chain carrying one grew by a byte per
    /// coefficient — three here at rank one, three at rank two.
    ///
    /// At the `tiler.index-region.v11` step,
    /// `bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`
    /// gave every discharged index-domain assessment a fact-source tag, so each
    /// chain grew by exactly one byte per discharged predicate: 24 for the rank
    /// two normalization, 10 for its rank one instance, and 8 for the staged
    /// template. Every one of these chains is wholly literal, so every new tag
    /// reads `Program` — which is itself the claim that the tag reports what a
    /// proof read rather than changing what any of them proved.
    ///
    /// Both are encoding changes to fields these chains *do* spell, which is
    /// exactly the class of move this pin exists to report; the claim it defends
    /// is that a widening no existing law spells changes nothing. Every digest
    /// was recomputed on the tree that landed its step.
    #[test]
    fn the_landed_one_reader_chain_identities_are_unchanged_byte_for_byte() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        for (name, sequence, bytes, digest) in [
            (
                "rms-norm-3x4-axis1",
                IndexRealizationLaw::staged_root_mean_square_scale_f32()
                    .realize_sequence(
                        &rms_norm_subject(&[3, 4], 1, RMS_NORM_F32_REFERENCE_EPS_BITS),
                        &scalars,
                    )
                    .unwrap(),
                4099_usize,
                "76251765b9ab0938a554c914c346bce51dea05e134cacf7853307fe53c679a29",
            ),
            (
                "rms-norm-rank1-4-axis0",
                IndexRealizationLaw::staged_root_mean_square_scale_f32()
                    .realize_sequence(
                        &rms_norm_subject(&[4], 0, RMS_NORM_F32_REFERENCE_EPS_BITS),
                        &scalars,
                    )
                    .unwrap(),
                3662,
                "f0e20e547666d2a62d906d8e87af97e957896da1d08141091eea8a12f26daea6",
            ),
            (
                "staged-template-rank1-4-axis0",
                IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
                    axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                    scalar: multiply_f32_scalar_op(),
                }
                .realize_sequence(
                    &rms_norm_subject(&[4], 0, RMS_NORM_F32_REFERENCE_EPS_BITS),
                    &scalars,
                )
                .unwrap(),
                2034,
                "08353da8f68dd1bbd894a35fa9b6f284dac0f291a7603df8691c7e434b1b3d6c",
            ),
        ] {
            let identity = sequence.identity().as_bytes();
            assert_eq!(identity.len(), bytes, "{name} changed length");
            assert_eq!(
                tiler_digest::DigestAlgorithm::GOVERNED
                    .digest(SEQUENCE_IDENTITY_PIN_DOMAIN, identity)
                    .label(),
                digest,
                "{name} changed bytes"
            );
        }
    }

    #[test]
    fn strict_affine_law_refuses_wrong_contract_role_and_scalar_independently() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let u8_subject = subject(StrictAffineU8::resolved_type());
        let error = IndexRealizationLaw::strict_affine_u4_dequantize()
            .realize(&u8_subject, &scalars)
            .unwrap_err();
        assert_eq!(error.rule(), "strict-affine-encoded-contract");

        let subject = subject(StrictAffineU4::resolved_type());
        let wrong_role = IndexRealizationLaw::StrictAffineU4Dequantize {
            codes_role: STRICT_AFFINE_SCALE_ROLE,
            scale_role: STRICT_AFFINE_CODES_ROLE,
            zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            scalar: strict_affine_u4_dequantize_scalar_op(),
        };
        assert_eq!(
            wrong_role.realize(&subject, &scalars).unwrap_err().rule(),
            "strict-affine-component-roles"
        );

        let wrong_scalar = IndexRealizationLaw::StrictAffineU4Dequantize {
            codes_role: STRICT_AFFINE_CODES_ROLE,
            scale_role: STRICT_AFFINE_SCALE_ROLE,
            zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            scalar: add_f32_scalar_op(),
        };
        assert_eq!(
            wrong_scalar.realize(&subject, &scalars).unwrap_err().rule(),
            "canonical-emission"
        );
    }

    #[test]
    fn strict_affine_law_tag_is_append_only_and_distinct() {
        let mut strict = Vec::new();
        IndexRealizationLaw::strict_affine_u4_dequantize().encode(&mut strict);
        assert_eq!(strict.first(), Some(&8));
        for old in [
            IndexRealizationLaw::constant_f32(),
            IndexRealizationLaw::multiply_f32(),
            IndexRealizationLaw::PreciseSiluF32,
            IndexRealizationLaw::strict_serial_sum_f32(),
            IndexRealizationLaw::reindex_f32(),
            IndexRealizationLaw::broadcast_f32(),
            IndexRealizationLaw::tensor_contraction_f32(),
        ] {
            let mut encoded = Vec::new();
            old.encode(&mut encoded);
            assert_ne!(encoded, strict);
            assert!((1..=7).contains(encoded.first().unwrap()));
        }
    }

    #[test]
    fn the_staged_law_tag_is_append_only_and_distinct() {
        let mut staged = Vec::new();
        IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32().encode(&mut staged);
        assert_eq!(staged.first(), Some(&9));
        // The staged form names an attribute like tag 4 and a scalar like tag 2.
        // Neither prefix can be mistaken for it, because the discriminating byte
        // comes first and no earlier tag writes both payloads.
        for old in [
            IndexRealizationLaw::constant_f32(),
            IndexRealizationLaw::constant_bf16(),
            IndexRealizationLaw::multiply_f32(),
            IndexRealizationLaw::add_f32(),
            IndexRealizationLaw::multiply_bf16(),
            IndexRealizationLaw::add_bf16(),
            IndexRealizationLaw::PreciseSiluF32,
            IndexRealizationLaw::strict_serial_sum_f32(),
            IndexRealizationLaw::reindex_f32(),
            IndexRealizationLaw::broadcast_f32(),
            IndexRealizationLaw::tensor_contraction_f32(),
            IndexRealizationLaw::strict_affine_u4_dequantize(),
        ] {
            let mut encoded = Vec::new();
            old.encode(&mut encoded);
            assert_ne!(encoded, staged);
            assert!((1..=8).contains(encoded.first().unwrap()));
        }
        // Its own payload separates two staged rows that differ only in the
        // scalar their pass applies, or only in the attribute their fold reads.
        let mut other_scalar = Vec::new();
        IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
            axes_attribute: REDUCTION_AXES_ATTRIBUTE,
            scalar: add_f32_scalar_op(),
        }
        .encode(&mut other_scalar);
        assert_ne!(other_scalar, staged);
        let mut other_attribute = Vec::new();
        IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
            axes_attribute: AttributeFieldId::new(REDUCTION_AXES_ATTRIBUTE.get() + 1),
            scalar: multiply_f32_scalar_op(),
        }
        .encode(&mut other_attribute);
        assert_ne!(other_attribute, staged);
    }

    /// A staged law cannot answer the single-region realization API.
    ///
    /// Answering one of its stages would be a truncated realization wearing the
    /// shape of a complete one, and the single-region `verify` path would then
    /// compare a candidate against a fragment. `verify` is not the path the
    /// compiler takes — `refine_index_region` in `tiler-compiler` drives
    /// `verify_sequence`, and the only calls to `verify` in that crate are in
    /// its own tests — but it is public, so a consumer can reach it and this
    /// refusal is what it meets.
    #[test]
    fn a_staged_law_refuses_the_single_region_realization() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let subject = subject(StrictAffineU4::resolved_type());
        assert_eq!(
            IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32()
                .realize(&subject, &scalars)
                .unwrap_err()
                .rule(),
            "staged-law-requires-region-sequence"
        );
    }

    /// Derives one `tiler::rms-norm-f32@1` occurrence's refinement subject.
    fn rms_norm_subject(dims: &[u64], axis: u32, eps_bits: u32) -> IndexRefinementSubject {
        let shape = Shape::try_new(dims.iter().copied().map(Extent::new).collect::<Vec<_>>())
            .expect("the test shape is canonical");
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let value = program
            .input_resolved(
                InputKey::new("value").unwrap(),
                shape.clone(),
                F32::resolved_type(),
            )
            .unwrap();
        let weight = program
            .input_resolved(
                InputKey::new("weight").unwrap(),
                shape,
                F32::resolved_type(),
            )
            .unwrap();
        let attributes = OperationAttributes::new([
            CanonicalField::new(
                RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                rms_norm_f32_axis_attribute(Axis::new(axis)),
            ),
            CanonicalField::new(
                RMS_NORM_EPS_BITS_ATTRIBUTE,
                rms_norm_f32_eps_attribute(eps_bits),
            ),
        ])
        .unwrap();
        let result = program
            .apply(rms_norm_f32_op(), attributes, &[value, weight])
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("normalized").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap()
    }

    /// Names one stage's scalar steps in emission order.
    fn stage_steps(stage: &VerifiedIndexRegion) -> Vec<String> {
        stage
            .scalar_operations()
            .map(|operation| match operation.kind() {
                ScalarOperationKindRef::Apply { key, .. } => key.name().to_owned(),
                ScalarOperationKindRef::Reduce(_) => "reduce".to_owned(),
            })
            .collect()
    }

    /// Returns the operation defining one value, or `None` when it is a read.
    fn defined_by(
        region: &VerifiedIndexRegion,
        value: VerifiedScalarValueId,
    ) -> Option<VerifiedScalarOperationId> {
        match region.scalar_value(value).unwrap().definition() {
            ScalarValueDefinitionView::OperationResult { operation, .. } => Some(operation),
            ScalarValueDefinitionView::AccessRead(_) => None,
        }
    }

    fn operand_definitions(
        region: &VerifiedIndexRegion,
        operation: ScalarOperationRef<'_>,
    ) -> Vec<Option<VerifiedScalarOperationId>> {
        operation
            .operands()
            .map(|value| defined_by(region, value))
            .collect()
    }

    /// Returns the boundary one read value comes from, or `None` for a computed value.
    fn read_boundary(
        region: &VerifiedIndexRegion,
        value: VerifiedScalarValueId,
    ) -> Option<super::super::VerifiedTensorId> {
        match region.scalar_value(value).unwrap().definition() {
            ScalarValueDefinitionView::AccessRead(access) => {
                Some(region.access(access).unwrap().tensor())
            }
            ScalarValueDefinitionView::OperationResult { .. } => None,
        }
    }

    fn by_id(
        region: &VerifiedIndexRegion,
        id: VerifiedScalarOperationId,
    ) -> ScalarOperationRef<'_> {
        region.scalar_operation(id).unwrap()
    }

    /// Returns the operation defining one value, refusing a read.
    fn operation_defining(
        region: &VerifiedIndexRegion,
        value: VerifiedScalarValueId,
    ) -> ScalarOperationRef<'_> {
        by_id(
            region,
            defined_by(region, value).expect("this value is computed rather than read"),
        )
    }

    fn applied_key(operation: ScalarOperationRef<'_>) -> String {
        match operation.kind() {
            ScalarOperationKindRef::Apply { key, .. } => key.name().to_owned(),
            ScalarOperationKindRef::Reduce(_) => "reduce".to_owned(),
        }
    }

    fn applied_attributes(operation: ScalarOperationRef<'_>) -> &CanonicalValue {
        match operation.kind() {
            ScalarOperationKindRef::Apply { attributes, .. } => attributes.value(),
            ScalarOperationKindRef::Reduce(_) => panic!("a reduction carries no attribute record"),
        }
    }

    fn f32_bits_record(bits: u32) -> CanonicalValue {
        CanonicalValue::record([CanonicalField::new(
            F32_CONSTANT_BITS_ATTRIBUTE,
            CanonicalValue::float_bits(
                TypeKey::new("tiler", "f32", 1).unwrap(),
                bits.to_be_bytes(),
            )
            .unwrap(),
        )])
        .unwrap()
    }

    /// The realization is the pinned reference, in the pinned order.
    ///
    /// `rms_norm_f32_reference_semantics` fixes `q_i = x_i * x_i`, `a = fold(q)`,
    /// `u = a / N`, `t = u + eps`, `r = Rsqrt(t)`, `y_i = w_i * (x_i * r)`. This
    /// walks the two realized stages and pins each of those six steps: the scalar
    /// applied, the values it consumes, and — for the two constants, which are the
    /// only places an exact payload enters — the payload itself.
    #[test]
    fn the_normalization_law_realizes_the_pinned_reference_step_for_step() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let subject = rms_norm_subject(&[3, 4], 1, RMS_NORM_F32_REFERENCE_EPS_BITS);
        let realization = IndexRealizationLaw::staged_root_mean_square_scale_f32()
            .realize_sequence(&subject, &scalars)
            .expect("the normalization's law realizes its own occurrence");

        assert_eq!(realization.stage_count(), 2);
        let [intermediate] = realization.intermediates() else {
            panic!("the normalization hands exactly one value on")
        };
        // One published value per folded row, not one per point and not one per
        // program: `r` is rank one over the kept axis of a rank-two operand.
        assert_eq!(intermediate.shape(), &Shape::from_dims([3]));
        assert_eq!(
            realization.stage_sources(1),
            Some(
                [
                    StagedInputSource::Occurrence(0),
                    StagedInputSource::Occurrence(1),
                    StagedInputSource::Intermediate(0),
                ]
                .as_slice()
            ),
            "the scale stage reads the value again, the weight, and the fold"
        );

        let fold = realization.stage(0).expect("the folding stage");
        // The population first, so a walk that missed a step could not pass: a
        // verified region orders its operations canonically rather than in
        // emission order, which is why the walk below navigates by definition
        // rather than by position.
        assert_eq!(
            stage_steps(fold),
            [
                "constant-f32",
                "constant-f32",
                "multiply-f32",
                "multiply-f32",
                "reduce",
                "divide-f32",
                "add-f32",
                "rsqrt-f32",
            ]
        );

        // r = Rsqrt(t) is exactly what the stage publishes.
        let root = operation_defining(fold, fold.outputs().next().unwrap().value());
        assert_eq!(applied_key(root), "rsqrt-f32");
        // t = u + eps, with the exact declared payload and no other.
        let [biased] = operand_definitions(fold, root)[..] else {
            panic!("the reciprocal square root takes one argument")
        };
        let biased = by_id(fold, biased.expect("t is computed"));
        assert_eq!(applied_key(biased), "add-f32");
        let [mean, bias] = operand_definitions(fold, biased)[..] else {
            panic!("the bias is an addition")
        };
        let bias = by_id(fold, bias.expect("eps enters as a constant"));
        assert_eq!(
            applied_attributes(bias),
            &f32_bits_record(RMS_NORM_F32_REFERENCE_EPS_BITS)
        );
        // u = a / N, a division by the extent itself and never by a reciprocal.
        let mean = by_id(fold, mean.expect("u is computed"));
        assert_eq!(applied_key(mean), "divide-f32");
        let [total, extent] = operand_definitions(fold, mean)[..] else {
            panic!("the mean is a division")
        };
        let extent = by_id(fold, extent.expect("the extent enters as a constant"));
        assert_eq!(
            applied_attributes(extent),
            &f32_bits_record(4.0_f32.to_bits())
        );
        // a = the strict left fold of q seeded at the first contributor: the
        // reduction's state starts at a squared seed and combines squared tail
        // contributors, so nothing unsquared reaches the accumulator.
        let total = by_id(fold, total.expect("a is computed"));
        let ScalarOperationKindRef::Reduce(reduction) = total.kind() else {
            panic!("the fold is a reduction")
        };
        let seed = reduction
            .init()
            .map(|value| operation_defining(fold, value))
            .collect::<Vec<_>>();
        let contributors = reduction
            .contributors()
            .map(|value| operation_defining(fold, value))
            .collect::<Vec<_>>();
        assert_eq!(seed.len(), 1);
        assert_eq!(contributors.len(), 1);
        // q_i = x_i * x_i: one read squared against itself, at both the seed and
        // the tail. A square of two different reads would be a product.
        for square in [seed[0], contributors[0]] {
            assert_eq!(applied_key(square), "multiply-f32");
            let operands = square.operands().collect::<Vec<_>>();
            assert_eq!(operands.len(), 2);
            assert_eq!(operands[0], operands[1]);
            assert_eq!(defined_by(fold, operands[0]), None);
        }
        assert_ne!(
            seed[0].id(),
            contributors[0].id(),
            "the seed and the tail read different contributors"
        );

        let scale = realization.final_stage();
        assert_eq!(stage_steps(scale), ["multiply-f32", "multiply-f32"]);
        let scale_steps = scale.scalar_operations().collect::<Vec<_>>();
        let boundaries = scale
            .tensors()
            .filter(|tensor| tensor.role() == TensorRole::Input)
            .map(super::super::model::TensorRef::id)
            .collect::<Vec<_>>();
        // y_i = w_i * (x_i * r): the inner product is the value and the published
        // root — in that order — and the outer applies the weight to it. The
        // boundary each read comes from is what separates the value from the
        // weight, which agree on element type and shape.
        let inner = scale_steps[0].operands().collect::<Vec<_>>();
        assert_eq!(
            inner
                .iter()
                .map(|value| read_boundary(scale, *value))
                .collect::<Vec<_>>(),
            vec![Some(boundaries[0]), Some(boundaries[2])]
        );
        let outer = scale_steps[1].operands().collect::<Vec<_>>();
        assert_eq!(read_boundary(scale, outer[0]), Some(boundaries[1]));
        assert_eq!(defined_by(scale, outer[1]), Some(scale_steps[0].id()));
        assert_eq!(
            defined_by(scale, scale.outputs().next().unwrap().value()),
            Some(scale_steps[1].id())
        );
    }

    /// The `eps` payload is consumed, and the template that would drop it is not.
    ///
    /// **The hazard is watched here rather than described.** `reduction_axes`
    /// reads its attribute by field identifier and tolerates a record carrying
    /// more, so the plain staged template realizes this very occurrence while
    /// never looking at `eps` — the first assertion below is that silent success.
    /// `eps` is part of the operation's identity, so a law that did that would be
    /// realizing a different operation. The normalization's own law therefore
    /// pins the exact declared field set, and the remaining assertions watch that
    /// refusal fire for every way the pair can fail to name the record.
    #[test]
    fn the_normalization_law_consumes_eps_where_the_staged_template_drops_it() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        // Rank one so the plain template's own broadcast rule does not refuse
        // first: the fold removes the only axis, so what it publishes is rank
        // zero and legible to a binary pointwise pass.
        let subject = rms_norm_subject(&[4], 0, RMS_NORM_F32_REFERENCE_EPS_BITS);
        assert!(
            IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
                axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                scalar: multiply_f32_scalar_op(),
            }
            .realize_sequence(&subject, &scalars)
            .is_ok(),
            "the hazard this law exists to close: the staged template realizes the \
             normalization's occurrence without ever reading its eps attribute"
        );

        // Every way of failing to name the declared record refuses, by name.
        for (law, rule) in [
            (
                IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
                    axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                    eps_attribute: AttributeFieldId::new(RMS_NORM_EPS_BITS_ATTRIBUTE.get() + 1),
                },
                "rms-scale-attributes",
            ),
            (
                IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
                    axes_attribute: AttributeFieldId::new(
                        RMS_NORM_REDUCED_AXES_ATTRIBUTE.get() + 8,
                    ),
                    eps_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
                },
                "rms-scale-attributes",
            ),
            (
                // Aliased identifiers would satisfy a field count while leaving
                // one of the two declared payloads unread.
                IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
                    axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                    eps_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                },
                "rms-scale-attribute-aliasing",
            ),
            (
                // The transposed pair names both declared fields, so the field-set
                // check passes and the payloads are read as each other's.
                IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
                    axes_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
                    eps_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                },
                "rms-scale-eps-kind",
            ),
        ] {
            assert_eq!(
                law.realize_sequence(&subject, &scalars)
                    .expect_err("a law that does not name this record's fields realizes nothing")
                    .rule(),
                rule
            );
        }
    }

    /// The division is by the extent, so an extent that is not one is refused.
    #[test]
    fn a_folded_extent_no_binary32_value_equals_is_refused_rather_than_rounded() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        // 2^24 + 1 is odd and above the significand's width, so its nearest
        // binary32 is 2^24 — a divisor that is not the extent.
        let subject = rms_norm_subject(&[16_777_217], 0, RMS_NORM_F32_REFERENCE_EPS_BITS);
        assert_eq!(
            IndexRealizationLaw::staged_root_mean_square_scale_f32()
                .realize_sequence(&subject, &scalars)
                .unwrap_err()
                .rule(),
            "rms-scale-extent-not-exact"
        );
        // The neighbouring even extent is exactly representable and realizes.
        let subject = rms_norm_subject(&[16_777_216], 0, RMS_NORM_F32_REFERENCE_EPS_BITS);
        assert!(
            IndexRealizationLaw::staged_root_mean_square_scale_f32()
                .realize_sequence(&subject, &scalars)
                .is_ok(),
            "the bound is representability, not size"
        );
    }

    #[test]
    fn the_root_mean_square_law_tag_is_append_only_and_distinct() {
        let mut normalization = Vec::new();
        IndexRealizationLaw::staged_root_mean_square_scale_f32().encode(&mut normalization);
        assert_eq!(normalization.first(), Some(&10));
        for old in [
            IndexRealizationLaw::constant_f32(),
            IndexRealizationLaw::constant_bf16(),
            IndexRealizationLaw::multiply_f32(),
            IndexRealizationLaw::add_f32(),
            IndexRealizationLaw::multiply_bf16(),
            IndexRealizationLaw::add_bf16(),
            IndexRealizationLaw::PreciseSiluF32,
            IndexRealizationLaw::strict_serial_sum_f32(),
            IndexRealizationLaw::reindex_f32(),
            IndexRealizationLaw::broadcast_f32(),
            IndexRealizationLaw::tensor_contraction_f32(),
            IndexRealizationLaw::strict_affine_u4_dequantize(),
            IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32(),
        ] {
            let mut encoded = Vec::new();
            old.encode(&mut encoded);
            assert_ne!(encoded, normalization);
            assert!((1..=9).contains(encoded.first().unwrap()));
        }
        // Within the tag, each field owns four fixed bytes, so a row differing in
        // either one encodes distinctly — and the pair is ordered, so the
        // transposition encodes as a third distinct row rather than the same one.
        let mut moved_axes = Vec::new();
        IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
            axes_attribute: AttributeFieldId::new(RMS_NORM_REDUCED_AXES_ATTRIBUTE.get() + 1),
            eps_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
        }
        .encode(&mut moved_axes);
        let mut moved_eps = Vec::new();
        IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
            axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
            eps_attribute: AttributeFieldId::new(RMS_NORM_EPS_BITS_ATTRIBUTE.get() + 1),
        }
        .encode(&mut moved_eps);
        let mut transposed = Vec::new();
        IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
            axes_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
            eps_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
        }
        .encode(&mut transposed);
        let rows = [&normalization, &moved_axes, &moved_eps, &transposed];
        for (position, row) in rows.iter().enumerate() {
            for other in &rows[position + 1..] {
                assert_ne!(row, other);
            }
        }
    }

    /// A staged law cannot answer the single-region realization API.
    #[test]
    fn the_root_mean_square_law_refuses_the_single_region_realization() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let subject = rms_norm_subject(&[3, 4], 1, RMS_NORM_F32_REFERENCE_EPS_BITS);
        assert_eq!(
            IndexRealizationLaw::staged_root_mean_square_scale_f32()
                .realize(&subject, &scalars)
                .unwrap_err()
                .rule(),
            "staged-law-requires-region-sequence"
        );
    }

    /// A resolution answers the realization's shape from its own authorities.
    ///
    /// The two accessors exist so a consumer that needs a law's stage structure
    /// — region formation is the caller — reads it from the resolution rather
    /// than deriving a second account of the law. Three claims: the staged
    /// family answers its two-stage sequence, a single-region family answers
    /// one stage behind a false predicate, and a subject the law refuses maps
    /// to the same typed refusal refinement reports, carrying the law's own
    /// rule.
    #[test]
    fn a_resolved_realization_exposes_its_laws_sequence_shape() {
        use crate::index::{FrozenIndexRealizationLawRegistry, IndexRefinementVerificationError};
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let semantic = crate::semantic::FrozenSemanticRegistry::standard().unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars.clone()).unwrap();

        let staged = laws
            .resolve(&rms_norm_subject(
                &[3, 4],
                1,
                RMS_NORM_F32_REFERENCE_EPS_BITS,
            ))
            .unwrap();
        assert!(staged.realizes_region_sequence());
        let sequence = staged.realize_sequence().unwrap();
        assert_eq!(sequence.stage_count(), 2);
        assert_eq!(sequence.intermediates().len(), 1);

        let single = laws
            .resolve(&subject(StrictAffineU4::resolved_type()))
            .unwrap();
        assert!(!single.realizes_region_sequence());
        let sequence = single.realize_sequence().unwrap();
        assert_eq!(sequence.stage_count(), 1);
        assert!(sequence.intermediates().is_empty());

        // The refusal is the law's, in refinement's vocabulary: an extent no
        // binary32 value equals refuses the realization, not the resolution.
        let refused = laws
            .resolve(&rms_norm_subject(
                &[16_777_217],
                0,
                RMS_NORM_F32_REFERENCE_EPS_BITS,
            ))
            .unwrap();
        assert!(matches!(
            refused.realize_sequence().unwrap_err(),
            IndexRefinementVerificationError::SemanticRealizationLawRefused { rule, .. }
                if rule == "rms-scale-extent-not-exact"
        ));
    }

    /// Derives one `tiler::softmax-f32@1` occurrence's refinement subject.
    fn softmax_subject(dims: &[u64], axis: u32) -> IndexRefinementSubject {
        let shape = Shape::try_new(dims.iter().copied().map(Extent::new).collect::<Vec<_>>())
            .expect("the test shape is canonical");
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let scores = program
            .input_resolved(
                InputKey::new("scores").unwrap(),
                shape,
                F32::resolved_type(),
            )
            .unwrap();
        let attributes = OperationAttributes::new([CanonicalField::new(
            SOFTMAX_REDUCED_AXES_ATTRIBUTE,
            softmax_f32_axis_attribute(Axis::new(axis)),
        )])
        .unwrap();
        let result = program
            .apply(softmax_f32_op(), attributes, &[scores])
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("weights").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap()
    }

    /// Names the scalar keys one reduction's body applies, in canonical order.
    fn reducer_body_steps(operation: ScalarOperationRef<'_>) -> Vec<String> {
        let ScalarOperationKindRef::Reduce(reduction) = operation.kind() else {
            panic!("this operation is not a reduction")
        };
        reduction
            .body()
            .operations()
            .map(|body| body.key().name().to_owned())
            .collect()
    }

    /// The realization is the pinned reference, in the pinned order.
    ///
    /// `softmax_f32_reference_semantics` fixes, over the single reduced axis:
    /// `m` = the strict left fold of the NaN-propagating `Maximum` seeded at the
    /// first contributor; `e_i = Exp(s_i - m)`; `d` = the strict left fold sum of
    /// `e` seeded at the first contributor; `c = 1.0 / d` as one division of one
    /// by the denominator; `r_i = e_i * c` and deliberately not `e_i / d`. This
    /// walks the four realized stages and pins each of those five steps: the
    /// combiner each fold applies, what each fold is seeded at, the values every
    /// application consumes, and — for the two constants, which are the only
    /// places an exact payload enters — the payload itself.
    #[test]
    fn the_softmax_law_realizes_the_pinned_reference_step_for_step() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let subject = softmax_subject(&[3, 4], 1);
        let realization = IndexRealizationLaw::staged_softmax_f32()
            .realize_sequence(&subject, &scalars)
            .expect("the softmax's law realizes its own occurrence");

        assert_eq!(realization.stage_count(), 4);
        assert_eq!(
            (0..4)
                .map(|stage| realization
                    .stage_sources(stage)
                    .expect("every stage exists")
                    .to_vec())
                .collect::<Vec<_>>(),
            vec![
                vec![StagedInputSource::Occurrence(0)],
                vec![
                    StagedInputSource::Occurrence(0),
                    StagedInputSource::Intermediate(0),
                ],
                vec![StagedInputSource::Intermediate(1)],
                vec![
                    StagedInputSource::Intermediate(1),
                    StagedInputSource::Intermediate(2),
                ],
            ],
            "the scores are read twice, and the exponentials are read by the \
             denominator's fold and again by the normalizing pass"
        );
        // Four reads of three published values. `e` is the one that stays live
        // across a stage that publishes something else, which is the whole reason
        // this chain is four stages rather than three.
        assert_eq!(
            realization
                .intermediates()
                .iter()
                .map(|read| (
                    read.producer(),
                    read.consumer(),
                    read.retained_through(),
                    read.shape().clone()
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, 1, 1, Shape::from_dims([3])),
                (1, 2, 3, Shape::from_dims([3, 4])),
                (1, 3, 3, Shape::from_dims([3, 4])),
                (2, 3, 3, Shape::from_dims([3])),
            ]
        );

        // m: the strict left fold of Maximum seeded at the first contributor.
        let extrema = realization.stage(0).expect("the extrema stage");
        assert_eq!(stage_steps(extrema), ["reduce"]);
        let maximum = operation_defining(extrema, extrema.outputs().next().unwrap().value());
        assert_eq!(reducer_body_steps(maximum), ["maximum-f32"]);
        let ScalarOperationKindRef::Reduce(fold) = maximum.kind() else {
            panic!("the row maximum is a reduction")
        };
        let seed = fold.init().collect::<Vec<_>>();
        let contributors = fold.contributors().collect::<Vec<_>>();
        assert_eq!(seed.len(), 1);
        assert_eq!(contributors.len(), 1);
        // Seeded at the *first contributor*: a read of the operand, not a
        // constant identity — which this extrema family does not have.
        assert_eq!(defined_by(extrema, seed[0]), None);
        assert_eq!(defined_by(extrema, contributors[0]), None);
        assert_ne!(seed[0], contributors[0], "the tail reads a later element");

        // e_i = Exp(s_i - m), with the subtraction spelled as an exact sign flip
        // and one rounded addition.
        let exponentials = realization.stage(1).expect("the exponential stage");
        assert_eq!(
            stage_steps(exponentials),
            ["constant-f32", "multiply-f32", "add-f32", "exp-f32"]
        );
        let boundaries = exponentials
            .tensors()
            .filter(|tensor| tensor.role() == TensorRole::Input)
            .map(super::super::model::TensorRef::id)
            .collect::<Vec<_>>();
        let exponential =
            operation_defining(exponentials, exponentials.outputs().next().unwrap().value());
        assert_eq!(applied_key(exponential), "exp-f32");
        let [shifted] = operand_definitions(exponentials, exponential)[..] else {
            panic!("the exponential takes one argument")
        };
        let shifted = by_id(exponentials, shifted.expect("the shift is computed"));
        assert_eq!(applied_key(shifted), "add-f32");
        let shift_operands = shifted.operands().collect::<Vec<_>>();
        assert_eq!(
            read_boundary(exponentials, shift_operands[0]),
            Some(boundaries[0]),
            "the left operand is the score read at its own coordinates"
        );
        let negated = operation_defining(exponentials, shift_operands[1]);
        assert_eq!(applied_key(negated), "multiply-f32");
        let negation_operands = negated.operands().collect::<Vec<_>>();
        assert_eq!(
            read_boundary(exponentials, negation_operands[0]),
            Some(boundaries[1]),
            "the negated value is the published row maximum"
        );
        assert_eq!(
            applied_attributes(operation_defining(exponentials, negation_operands[1])),
            &f32_bits_record(F32_NEGATIVE_ONE_BITS),
            "the sign flip is exact, so the addition carries the pinned rounding"
        );

        // d: the strict left fold sum of e, seeded at the first contributor.
        let denominator = realization.stage(2).expect("the denominator stage");
        assert_eq!(stage_steps(denominator), ["reduce"]);
        let total = operation_defining(denominator, denominator.outputs().next().unwrap().value());
        assert_eq!(reducer_body_steps(total), ["add-f32"]);
        let ScalarOperationKindRef::Reduce(sum) = total.kind() else {
            panic!("the denominator is a reduction")
        };
        let seed = sum.init().collect::<Vec<_>>();
        assert_eq!(seed.len(), 1);
        assert_eq!(
            defined_by(denominator, seed[0]),
            None,
            "the sum is seeded at the first exponential, not at a zero identity"
        );

        // c = 1.0 / d once per row, then r_i = e_i * c once per point.
        let normalize = realization.final_stage();
        assert_eq!(
            stage_steps(normalize),
            ["constant-f32", "divide-f32", "multiply-f32"]
        );
        let boundaries = normalize
            .tensors()
            .filter(|tensor| tensor.role() == TensorRole::Input)
            .map(super::super::model::TensorRef::id)
            .collect::<Vec<_>>();
        let scaled = operation_defining(normalize, normalize.outputs().next().unwrap().value());
        assert_eq!(applied_key(scaled), "multiply-f32");
        let scale_operands = scaled.operands().collect::<Vec<_>>();
        assert_eq!(
            read_boundary(normalize, scale_operands[0]),
            Some(boundaries[0]),
            "the scaled value is the published exponential"
        );
        let reciprocal = operation_defining(normalize, scale_operands[1]);
        assert_eq!(
            applied_key(reciprocal),
            "divide-f32",
            "the normalization multiplies by a reciprocal and never divides e_i by d"
        );
        let reciprocal_operands = reciprocal.operands().collect::<Vec<_>>();
        assert_eq!(
            applied_attributes(operation_defining(normalize, reciprocal_operands[0])),
            &f32_bits_record(F32_ONE_BITS),
            "the numerator is exactly one"
        );
        assert_eq!(
            read_boundary(normalize, reciprocal_operands[1]),
            Some(boundaries[1]),
            "the divisor is the published denominator"
        );
        // One division per folded row rather than one per point: the reciprocal
        // is computed from a read at the kept coordinates alone, so the region
        // model carries only those dimensions as its free dimensions.
        assert_eq!(
            normalize
                .scalar_value(scale_operands[1])
                .unwrap()
                .free_dimensions()
                .count(),
            1,
            "the reciprocal varies along the kept axis alone"
        );
        assert_eq!(
            normalize
                .scalar_value(scale_operands[0])
                .unwrap()
                .free_dimensions()
                .count(),
            2,
            "the exponential it scales varies along both"
        );
    }

    /// A zero-length reduced axis has no first contributor and no identity.
    ///
    /// **Watched in both directions.** The operation is shape-preserving, so its
    /// own semantics evaluate no scalar softmax over a zero-length axis
    /// (`SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS`); this staged shape would still have
    /// to commit one row maximum per kept coordinate, and the extrema family has
    /// no identity to commit. The refusal is the general fold emitter's rather
    /// than this family's, which is why the neighbouring extent-one axis — whose
    /// fold has exactly one contributor and needs no identity either — realizes.
    #[test]
    fn a_zero_length_reduced_axis_is_refused_rather_than_seeded() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        assert_eq!(
            IndexRealizationLaw::staged_softmax_f32()
                .realize_sequence(&softmax_subject(&[3, 0], 1), &scalars)
                .unwrap_err()
                .rule(),
            "fold-empty-domain-without-identity"
        );
        assert!(
            IndexRealizationLaw::staged_softmax_f32()
                .realize_sequence(&softmax_subject(&[3, 1], 1), &scalars)
                .is_ok(),
            "the bound is an empty contributor domain, not a short one"
        );
        // The sum's identity is still there for the law that uses it: the plain
        // strict serial sum folds an empty axis to zero rather than refusing, so
        // the refusal above is the seeding rule this family chose and not a
        // capability the emitter lost.
        assert!(
            IndexRealizationLaw::strict_serial_sum_f32()
                .realize(&serial_sum_subject(&[3, 0], 1), &scalars)
                .is_ok(),
            "the addition has an identity and an empty fold commits it"
        );
    }

    /// The seeding rule is the fold emitter's, and it is watched there.
    ///
    /// **Separate from the family-level refusal above, because that one is
    /// satisfied by *either* of the softmax's two folds refusing.** This builds
    /// two plans over the same empty boundaries and separates them: a combiner
    /// with an identity folds an empty contributor domain to that identity, and a
    /// combiner without one refuses. Nothing else about the two plans differs, so
    /// the seeding rule is the only thing the outcome can be attributed to.
    #[test]
    fn an_identity_less_fold_refuses_the_empty_contributor_domain() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let subject = softmax_subject(&[3, 0], 1);
        let boundaries = || {
            FoldPlan::for_boundaries(
                &F32::resolved_type(),
                &Shape::from_dims([3, 0]),
                &Shape::from_dims([3]),
                &[Axis::new(1)],
            )
            .expect("the empty-axis boundaries are coherent")
        };
        let emit = |plan: &FoldPlan| -> Result<(), IndexRealizationLawError> {
            let mut builder = IndexRegionBuilder::new(scalars.clone())?;
            let mut context = LawContext {
                builder: &mut builder,
                subject: &subject,
            };
            emit_fold_region(&mut context, plan, |_, total| Ok(total))
        };
        assert_eq!(boundaries().reduced_points, 0);
        assert!(
            emit(&boundaries()).is_ok(),
            "the governed addition has an identity and an empty fold commits it"
        );
        assert_eq!(
            emit(&boundaries().combining(maximum_f32_scalar_op(), None))
                .unwrap_err()
                .rule(),
            "fold-empty-domain-without-identity"
        );
    }

    /// Every occurrence this law does not name is refused, by name.
    #[test]
    fn the_softmax_law_refuses_the_occurrences_it_does_not_name() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let law = IndexRealizationLaw::staged_softmax_f32();
        // Two operands: the normalization's occurrence, not this one's.
        assert_eq!(
            law.realize_sequence(
                &rms_norm_subject(&[3, 4], 1, RMS_NORM_F32_REFERENCE_EPS_BITS),
                &scalars
            )
            .unwrap_err()
            .rule(),
            "softmax-arity"
        );
        // One operand of another element type.
        assert_eq!(
            law.realize_sequence(&subject(StrictAffineU4::resolved_type()), &scalars)
                .unwrap_err()
                .rule(),
            "softmax-value-type"
        );
        // One `f32` operand whose result drops the reduced axis: a reduction.
        assert_eq!(
            law.realize_sequence(&serial_sum_subject(&[3, 4], 1), &scalars)
                .unwrap_err()
                .rule(),
            "softmax-shape"
        );
        // A law naming a field the occurrence's record does not carry.
        assert_eq!(
            IndexRealizationLaw::StagedSoftmaxF32 {
                axes_attribute: AttributeFieldId::new(SOFTMAX_REDUCED_AXES_ATTRIBUTE.get() + 1),
            }
            .realize_sequence(&softmax_subject(&[3, 4], 1), &scalars)
            .unwrap_err()
            .rule(),
            "softmax-attributes"
        );
    }

    /// The registered row answers for the family, with the chain it must realize.
    ///
    /// This is what the ticket's user-visible outcome names:
    /// `FrozenIndexRealizationLawRegistry::resolve` stops answering
    /// `MissingRealizationLaw` for `tiler::softmax-f32@1`, and what it answers
    /// realizes the four-stage chain rather than merely resolving.
    #[test]
    fn the_softmax_family_resolves_to_its_registered_law() {
        use crate::index::FrozenIndexRealizationLawRegistry;
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let semantic = crate::semantic::FrozenSemanticRegistry::standard().unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars.clone()).unwrap();
        assert_eq!(
            laws.family_realization_law(&softmax_f32_op()),
            Some(&IndexRealizationLaw::staged_softmax_f32())
        );
        let resolved = laws.resolve(&softmax_subject(&[3, 4], 1)).unwrap();
        assert!(resolved.realizes_region_sequence());
        let sequence = resolved.realize_sequence().unwrap();
        assert_eq!(sequence.stage_count(), 4);
        assert_eq!(sequence.intermediates().len(), 4);
    }

    /// The realized chain's identity, pinned over exact bytes.
    ///
    /// **A new pin rather than a preservation claim.** The digests below were
    /// computed on the tree that first registered this law, so what they defend
    /// is future drift: refinement compares a provider's emitted chain against
    /// this one byte for byte, so a step, a boundary order, or a source list that
    /// moved silently would change which emissions verify. The length is pinned
    /// beside the digest so a chain that moved reports *how* rather than only
    /// that it did. Recompute both on the tree the change lands in, and put the
    /// cause in that commit.
    ///
    /// **Recomputed twice, and neither time did the softmax law itself move.**
    /// At the `tiler.index-region.v10` step a linear combination's coefficient
    /// became a tagged `SourcedIndexInteger`, so each of this chain's
    /// coefficients grew by its tag byte — six at rank two, six at rank one. At
    /// the `tiler.index-region.v11` step every discharged index-domain
    /// assessment gained a fact-source tag, so the chain grew by one byte per
    /// discharged predicate — 40 at rank two, 16 at rank one. This chain is
    /// wholly literal, so every new tag reads `Program`.
    #[test]
    fn the_softmax_chain_identity_is_pinned() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        for (name, dims, axis, bytes, digest) in [
            (
                "softmax-3x4-axis1",
                [3, 4].as_slice(),
                1_u32,
                5635_usize,
                "5ae4e45344efb4eceeb9436347aa2ab2ea37d1d2b246811026d38e861f57c22a",
            ),
            (
                "softmax-rank1-4-axis0",
                [4].as_slice(),
                0,
                4909,
                "c596a5cefa760cd69ccf44078df097f553c62880a8a7610b3550af6473d9b011",
            ),
        ] {
            let identity = IndexRealizationLaw::staged_softmax_f32()
                .realize_sequence(&softmax_subject(dims, axis), &scalars)
                .unwrap();
            let identity = identity.identity().as_bytes();
            assert_eq!(identity.len(), bytes, "{name} changed length");
            assert_eq!(
                tiler_digest::DigestAlgorithm::GOVERNED
                    .digest(SEQUENCE_IDENTITY_PIN_DOMAIN, identity)
                    .label(),
                digest,
                "{name} changed bytes"
            );
        }
    }

    #[test]
    fn the_softmax_law_tag_is_append_only_and_distinct() {
        let mut softmax = Vec::new();
        IndexRealizationLaw::staged_softmax_f32().encode(&mut softmax);
        assert_eq!(softmax.first(), Some(&11));
        // Tags 4, 5, 6, and 7 write the same payload shape this one does — one
        // fixed-width attribute identifier — so the discriminating first byte is
        // the whole of the separation, and it is asserted rather than argued.
        for old in [
            IndexRealizationLaw::constant_f32(),
            IndexRealizationLaw::constant_bf16(),
            IndexRealizationLaw::multiply_f32(),
            IndexRealizationLaw::add_f32(),
            IndexRealizationLaw::multiply_bf16(),
            IndexRealizationLaw::add_bf16(),
            IndexRealizationLaw::PreciseSiluF32,
            IndexRealizationLaw::strict_serial_sum_f32(),
            IndexRealizationLaw::reindex_f32(),
            IndexRealizationLaw::broadcast_f32(),
            IndexRealizationLaw::tensor_contraction_f32(),
            IndexRealizationLaw::strict_affine_u4_dequantize(),
            IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32(),
            IndexRealizationLaw::staged_root_mean_square_scale_f32(),
        ] {
            let mut encoded = Vec::new();
            old.encode(&mut encoded);
            assert_ne!(encoded, softmax);
            assert!((1..=10).contains(encoded.first().unwrap()));
        }
        // Its own payload separates two softmax rows differing in the one field
        // the template carries.
        let mut moved_axes = Vec::new();
        IndexRealizationLaw::StagedSoftmaxF32 {
            axes_attribute: AttributeFieldId::new(SOFTMAX_REDUCED_AXES_ATTRIBUTE.get() + 1),
        }
        .encode(&mut moved_axes);
        assert_ne!(moved_axes, softmax);
        // The reduced-axes identifier the normalization's record uses happens to
        // be a different number, and that coincidence is not what separates the
        // two rows: the tag is.
        let mut normalization = Vec::new();
        IndexRealizationLaw::staged_root_mean_square_scale_f32().encode(&mut normalization);
        assert_ne!(normalization.first(), softmax.first());
    }

    /// A staged law cannot answer the single-region realization API.
    #[test]
    fn the_softmax_law_refuses_the_single_region_realization() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        assert_eq!(
            IndexRealizationLaw::staged_softmax_f32()
                .realize(&softmax_subject(&[3, 4], 1), &scalars)
                .unwrap_err()
                .rule(),
            "staged-law-requires-region-sequence"
        );
    }

    /// Derives one `tiler::strict-serial-sum-f32@1` occurrence's subject.
    ///
    /// A one-operand `f32` occurrence whose result *drops* the reduced axis,
    /// which is what makes it the shape-rule counterexample the softmax law must
    /// refuse — and the family whose fold has an identity to seed an empty
    /// domain with.
    fn serial_sum_subject(dims: &[u64], axis: u32) -> IndexRefinementSubject {
        let shape = Shape::try_new(dims.iter().copied().map(Extent::new).collect::<Vec<_>>())
            .expect("the test shape is canonical");
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let value = program
            .input_resolved(InputKey::new("value").unwrap(), shape, F32::resolved_type())
            .unwrap();
        let attributes = OperationAttributes::new([CanonicalField::new(
            REDUCTION_AXES_ATTRIBUTE,
            CanonicalValue::sequence([CanonicalValue::unsigned_u32(axis)]).unwrap(),
        )])
        .unwrap();
        let result = program
            .apply(
                crate::semantic::strict_serial_sum_f32_op(),
                attributes,
                &[value],
            )
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("total").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap()
    }

    /// Derives one `tiler::concatenate-f32@1` occurrence's subject.
    fn concatenate_subject(operands: &[&[u64]], axis: u32) -> IndexRefinementSubject {
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let values = operands
            .iter()
            .enumerate()
            .map(|(position, dims)| {
                let shape =
                    Shape::try_new(dims.iter().copied().map(Extent::new).collect::<Vec<_>>())
                        .expect("the test shape is canonical");
                program
                    .input_resolved(
                        InputKey::new(format!("operand-{position}")).unwrap(),
                        shape,
                        F32::resolved_type(),
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let attributes = OperationAttributes::new([CanonicalField::new(
            CONCATENATE_AXIS_ATTRIBUTE,
            concatenate_f32_axis_attribute(Axis::new(axis)),
        )])
        .unwrap();
        let result = program
            .apply(concatenate_f32_op(), attributes, &values)
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("joined").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap()
    }

    /// The law's region is one output written by one root per operand.
    ///
    /// The middle operand is empty on the concatenated axis, which is the pinned
    /// prefill shape's own case: it is a partition member with an empty rectangle
    /// rather than a skipped operand, and the coverage arithmetic that admits the
    /// set is the volume identity with a zero term in it.
    #[test]
    fn the_concatenate_law_realizes_one_root_per_operand_over_one_output() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let region = IndexRealizationLaw::concatenate_f32()
            .realize(
                &concatenate_subject(&[&[2, 3, 4], &[2, 0, 4], &[2, 5, 4]], 1),
                &scalars,
            )
            .unwrap();

        assert_eq!(region.outputs().len(), 3, "one write root per operand");
        let written = region
            .outputs()
            .map(|root| region.access(root.access()).unwrap().tensor())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(written.len(), 1, "the three roots partition one output");
        for root in region.outputs() {
            assert!(
                matches!(
                    region
                        .access(root.access())
                        .unwrap()
                        .write_ownership_proof(),
                    Some(WriteOwnershipProofView::PartitionMember {
                        joint: JointPartitionProofView::Interval {
                            facts: IndexDomainFactSource::Program
                        }
                    })
                ),
                "every member owns its partition by interval reasoning"
            );
        }
        assert_eq!(
            region.scalar_operations().len(),
            0,
            "a concatenation applies no scalar operation, which is why its \
             declared emitted set is empty"
        );
        assert_eq!(
            region.dimensions().len(),
            5,
            "one shared dimension per non-concatenated axis plus one per operand"
        );
        assert_eq!(
            region
                .dimensions()
                .filter(|dimension| dimension.extent().as_static() == Some(Extent::new(0)))
                .count(),
            1,
            "the empty operand's own dimension, and only it, has extent zero"
        );
        assert_eq!(
            region
                .tensors()
                .filter(|tensor| tensor.role() == TensorRole::Input)
                .count(),
            3
        );
    }

    /// A value joined to itself is two members over one boundary.
    ///
    /// Operand order is semantic and the partition is keyed by operand rather
    /// than by distinct input, so this is one input tensor read twice at two
    /// different offsets — not one root, and not two boundaries.
    #[test]
    fn the_concatenate_law_partitions_by_operand_rather_than_by_input() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let value = program
            .input_resolved(
                InputKey::new("operand").unwrap(),
                Shape::from_dims([3]),
                F32::resolved_type(),
            )
            .unwrap();
        let attributes = OperationAttributes::new([CanonicalField::new(
            CONCATENATE_AXIS_ATTRIBUTE,
            concatenate_f32_axis_attribute(Axis::new(0)),
        )])
        .unwrap();
        let result = program
            .apply(concatenate_f32_op(), attributes, &[value, value])
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("joined").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            strict_contract(),
        )
        .unwrap();

        let region = IndexRealizationLaw::concatenate_f32()
            .realize(&subject, &scalars)
            .unwrap();
        assert_eq!(region.outputs().len(), 2);
        assert_eq!(
            region
                .tensors()
                .filter(|tensor| tensor.role() == TensorRole::Input)
                .count(),
            1,
            "one distinct boundary, joined to itself"
        );
    }

    /// The law refuses an occurrence outside its exact supported form.
    ///
    /// Three refusals, each on a subject a caller can actually build. The three
    /// remaining rules — a re-derived result shape disagreeing with the declared
    /// one, an operand position outside the input boundaries, and a subject
    /// carrying other than one result — are unreachable from a *verified*
    /// occurrence, and each says so at its own site under the
    /// unreachable-refusal convention at [`IndexRealizationLaw`].
    #[test]
    fn the_concatenate_law_refuses_occurrences_outside_its_form() {
        let scalars = FrozenScalarRegistry::standard().unwrap();

        // An occurrence carrying no attribute record at all.
        assert_eq!(
            IndexRealizationLaw::concatenate_f32()
                .realize(&subject(StrictAffineU4::resolved_type()), &scalars)
                .unwrap_err()
                .rule(),
            "concatenate-attributes"
        );

        // A law naming a field this record does not carry. Attribute identifiers
        // are record-local, so this is the mistake the constructor exists to
        // prevent rather than a hypothetical one.
        assert_eq!(
            IndexRealizationLaw::PartitionedConcatenate {
                axis_attribute: AttributeFieldId::new(CONCATENATE_AXIS_ATTRIBUTE.get() + 1),
            }
            .realize(&concatenate_subject(&[&[3], &[5]], 0), &scalars)
            .unwrap_err()
            .rule(),
            "concatenate-attribute-key"
        );

        // A record whose single field is numbered alike and means something else:
        // the softmax's reduced-axes sequence is not a canonical `u32` axis.
        assert_eq!(
            IndexRealizationLaw::concatenate_f32()
                .realize(&softmax_subject(&[3, 4], 1), &scalars)
                .unwrap_err()
                .rule(),
            "concatenate-axis"
        );
    }

    #[test]
    fn the_concatenate_law_tag_is_append_only_and_distinct() {
        let mut concatenate = Vec::new();
        IndexRealizationLaw::concatenate_f32().encode(&mut concatenate);
        assert_eq!(concatenate.first(), Some(&12));
        // Tags 4, 5, 6, 7, and 11 write the same payload shape this one does —
        // one fixed-width attribute identifier — so the discriminating first byte
        // is the whole of the separation, and it is asserted rather than argued.
        for old in [
            IndexRealizationLaw::constant_f32(),
            IndexRealizationLaw::constant_bf16(),
            IndexRealizationLaw::multiply_f32(),
            IndexRealizationLaw::add_f32(),
            IndexRealizationLaw::multiply_bf16(),
            IndexRealizationLaw::add_bf16(),
            IndexRealizationLaw::PreciseSiluF32,
            IndexRealizationLaw::strict_serial_sum_f32(),
            IndexRealizationLaw::reindex_f32(),
            IndexRealizationLaw::broadcast_f32(),
            IndexRealizationLaw::tensor_contraction_f32(),
            IndexRealizationLaw::strict_affine_u4_dequantize(),
            IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32(),
            IndexRealizationLaw::staged_root_mean_square_scale_f32(),
            IndexRealizationLaw::staged_softmax_f32(),
        ] {
            let mut encoded = Vec::new();
            old.encode(&mut encoded);
            assert_ne!(encoded, concatenate);
            assert!((1..=11).contains(encoded.first().unwrap()));
        }
        // Its own payload separates two rows differing in the one field the
        // template carries.
        let mut moved_axis = Vec::new();
        IndexRealizationLaw::PartitionedConcatenate {
            axis_attribute: AttributeFieldId::new(CONCATENATE_AXIS_ATTRIBUTE.get() + 1),
        }
        .encode(&mut moved_axis);
        assert_ne!(moved_axis, concatenate);
        // The reindex's mapping identifier happens to be the same number, and
        // that coincidence is not what separates the two rows: the tag is.
        let mut reindex = Vec::new();
        IndexRealizationLaw::reindex_f32().encode(&mut reindex);
        assert_eq!(
            REINDEX_MAPPING_ATTRIBUTE.get(),
            CONCATENATE_AXIS_ATTRIBUTE.get(),
            "the coincidence this assertion is about"
        );
        assert_ne!(reindex.first(), concatenate.first());
    }

    /// A single-region law answers the sequence API as a one-stage sequence.
    #[test]
    fn the_concatenate_law_realizes_a_one_stage_sequence() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let subject = concatenate_subject(&[&[3], &[5]], 0);
        let sequence = IndexRealizationLaw::concatenate_f32()
            .realize_sequence(&subject, &scalars)
            .unwrap();
        assert!(sequence.is_single_stage());
        assert_eq!(
            sequence.final_stage().canonical_identity(),
            IndexRealizationLaw::concatenate_f32()
                .realize(&subject, &scalars)
                .unwrap()
                .canonical_identity()
        );
    }

    /// Every law variant has one distinct leading tag, sized from the type.
    ///
    /// The array length is `variant_count`, so adding a variant without adding a
    /// representative is a build error at this census. The slice is the
    /// append-only thirteenth row: every earlier representative retains its old
    /// tag, and changing only the slice's record-local field changes only its
    /// four-byte payload.
    #[test]
    fn every_law_variant_has_one_append_only_encoding_tag() {
        let laws: [IndexRealizationLaw; std::mem::variant_count::<IndexRealizationLaw>()] = [
            IndexRealizationLaw::constant_f32(),
            IndexRealizationLaw::multiply_f32(),
            IndexRealizationLaw::PreciseSiluF32,
            IndexRealizationLaw::strict_serial_sum_f32(),
            IndexRealizationLaw::reindex_f32(),
            IndexRealizationLaw::broadcast_f32(),
            IndexRealizationLaw::tensor_contraction_f32(),
            IndexRealizationLaw::strict_affine_u4_dequantize(),
            IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32(),
            IndexRealizationLaw::staged_root_mean_square_scale_f32(),
            IndexRealizationLaw::staged_softmax_f32(),
            IndexRealizationLaw::concatenate_f32(),
            IndexRealizationLaw::slice_f32(),
        ];
        let encodings = laws
            .iter()
            .map(|law| {
                let mut encoded = Vec::new();
                law.encode(&mut encoded);
                encoded
            })
            .collect::<Vec<_>>();
        assert_eq!(
            encodings
                .iter()
                .map(|encoded| encoded[0])
                .collect::<Vec<_>>(),
            (1_u8..=13).collect::<Vec<_>>()
        );

        let slice = encodings.last().unwrap();
        let mut moved_selection = Vec::new();
        IndexRealizationLaw::Slice {
            selection_attribute: AttributeFieldId::new(SLICE_SELECTION_ATTRIBUTE.get() + 1),
        }
        .encode(&mut moved_selection);
        assert_eq!(slice.first(), Some(&13));
        assert_eq!(moved_selection.first(), Some(&13));
        assert_ne!(slice, &moved_selection);
        assert!(encodings[..12].iter().all(|encoded| encoded[0] < 13));
    }

    #[test]
    fn an_existing_law_payload_is_unchanged_by_the_appended_tag() {
        let mut encoded = Vec::new();
        IndexRealizationLaw::multiply_f32().encode(&mut encoded);
        let expected = [
            vec![2],
            12_u64.to_be_bytes().to_vec(),
            b"tiler.scalar".to_vec(),
            12_u64.to_be_bytes().to_vec(),
            b"multiply-f32".to_vec(),
            1_u32.to_be_bytes().to_vec(),
        ]
        .concat();
        assert_eq!(encoded, expected);
    }

    fn parametric_broadcast_subject(
        t_lower: u64,
        t_upper: u64,
    ) -> (
        IndexRefinementSubject,
        std::sync::Arc<crate::shape::ShapeEnv>,
    ) {
        use crate::semantic::{BroadcastAxisMapping, BroadcastAxisSource, F32Broadcast};
        use crate::shape::{
            BindingSource, EXTENT_PHASE_CEILING, ExtentRelation, ExtentTerm, FactProvenance,
            RootBinding, SemanticInputConstraint, ShapeEnvBuilder, ShapeSymbol, SymbolScope,
        };

        let scope = SymbolScope::new("law-broadcast/0").unwrap();
        let t = ShapeSymbol::new(scope.clone(), "t").unwrap();
        let n = ShapeSymbol::new(scope, "n").unwrap();
        let mut draft = ShapeEnvBuilder::new();
        draft.declare(t.clone()).unwrap();
        draft.declare(n.clone()).unwrap();
        draft
            .bind(
                &t,
                RootBinding::new(
                    BindingSource::InputDimension {
                        input: InputKey::new("operand").unwrap(),
                        axis: Axis::new(0),
                    },
                    EXTENT_PHASE_CEILING,
                    FactProvenance::RuntimeValidated,
                )
                .unwrap(),
            )
            .unwrap();
        draft
            .bind(
                &n,
                RootBinding::new(
                    BindingSource::InputDimension {
                        input: InputKey::new("operand").unwrap(),
                        axis: Axis::new(1),
                    },
                    EXTENT_PHASE_CEILING,
                    FactProvenance::RuntimeValidated,
                )
                .unwrap(),
            )
            .unwrap();
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(ExtentTerm::Symbol(t.clone()), t_lower, t_upper).unwrap(),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(ExtentTerm::Symbol(n.clone()), 2, 64).unwrap(),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        let environment = std::sync::Arc::new(draft.build().unwrap());
        let mut program =
            SemanticProgramBuilder::try_standard_with_shape_environment(environment.clone())
                .unwrap();
        let operand = program
            .input_sourced::<F32>(
                InputKey::new("operand").unwrap(),
                vec![SourcedExtent::Symbol(n.clone())],
            )
            .unwrap();
        let mapping = BroadcastAxisMapping::new(
            vec![SourcedExtent::Symbol(t), SourcedExtent::Symbol(n)],
            [
                BroadcastAxisSource::Replicate,
                BroadcastAxisSource::FromOperand(Axis::new(0)),
            ],
        )
        .unwrap();
        let widened = F32Broadcast::apply(&mut program, &mapping, operand).unwrap();
        program
            .output(OutputKey::new("widened").unwrap(), widened)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        (
            IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap(),
            environment,
        )
    }

    #[test]
    fn the_broadcast_law_realizes_the_same_parametric_carrier_across_bindings() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        for (lower, upper) in [
            (1, 32_768),
            (2, 32_768),
            (1, 1),
            (2, 2),
            (10, 10),
            (32_768, 32_768),
        ] {
            let (subject, _) = parametric_broadcast_subject(lower, upper);
            let region = IndexRealizationLaw::broadcast_f32()
                .realize(&subject, &scalars)
                .unwrap_or_else(|error| panic!("[{lower}, {upper}]: {error}"));
            assert_eq!(region.tensors().count(), 2);
            assert!(
                region.extent_sources().is_some(),
                "the parametric realization retains the program environment"
            );
        }
    }

    #[test]
    fn a_literal_broadcast_realization_does_not_attach_an_environment() {
        use crate::semantic::{BroadcastAxisMapping, BroadcastAxisSource, F32Broadcast};

        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let operand = program
            .input::<F32>(InputKey::new("operand").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let mapping = BroadcastAxisMapping::new(
            [Extent::new(8), Extent::new(4)],
            [
                BroadcastAxisSource::Replicate,
                BroadcastAxisSource::FromOperand(Axis::new(0)),
            ],
        )
        .unwrap();
        let widened = F32Broadcast::apply(&mut program, &mapping, operand).unwrap();
        program
            .output(OutputKey::new("widened").unwrap(), widened)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        let subject =
            IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap();
        let region = IndexRealizationLaw::broadcast_f32()
            .realize(&subject, &FrozenScalarRegistry::standard().unwrap())
            .unwrap();
        assert!(
            region.extent_sources().is_none(),
            "a literal mapping must keep the environment-free realization path"
        );
    }

    fn slice_symbol(name: &str) -> ShapeSymbol {
        ShapeSymbol::new(SymbolScope::new("slice/0").unwrap(), name).unwrap()
    }

    fn slice_cursor_binding() -> RootBinding {
        RootBinding::new(
            BindingSource::InputDimension {
                input: InputKey::new("tokens").unwrap(),
                axis: Axis::new(0),
            },
            EXTENT_PHASE_CEILING,
            FactProvenance::RuntimeValidated,
        )
        .unwrap()
    }

    fn slice_environment(lower: u64, upper: u64) -> std::sync::Arc<ShapeEnv> {
        let cursor = slice_symbol("c");
        let mut draft = ShapeEnvBuilder::new();
        draft.declare(cursor.clone()).unwrap();
        draft.bind(&cursor, slice_cursor_binding()).unwrap();
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(ExtentTerm::Symbol(cursor), lower, upper).unwrap(),
                FactProvenance::FrontendRequired,
            ))
            .unwrap();
        std::sync::Arc::new(draft.build().unwrap())
    }

    fn source_bearing_slice_subject(
        lower: u64,
        upper: u64,
    ) -> (
        IndexRefinementSubject,
        std::sync::Arc<ShapeEnv>,
        ShapeSymbol,
    ) {
        let environment = slice_environment(lower, upper);
        let cursor = slice_symbol("c");
        let mut program =
            SemanticProgramBuilder::try_standard_with_shape_environment(environment.clone())
                .unwrap();
        let table = program
            .input::<F32>(InputKey::new("table").unwrap(), Shape::from_dims([64, 128]))
            .unwrap();
        let selection = SliceSelection::new([
            SliceAxisSelection::Window {
                offset: SourcedExtent::Symbol(cursor.clone()),
                extent: Extent::new(6),
            },
            SliceAxisSelection::WholeAxis,
        ])
        .unwrap();
        let rows = F32Slice::apply(&mut program, &selection, table).unwrap();
        program
            .output(OutputKey::new("rows").unwrap(), rows)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        (
            IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap(),
            environment,
            cursor,
        )
    }

    fn slice_window_region(
        environment: Option<std::sync::Arc<ShapeEnv>>,
        offset: SourcedIndexInteger,
    ) -> Result<Result<VerifiedIndexRegion, IndexRegionBuildError>, SymbolicExtentError> {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let mut builder = match environment {
            Some(environment) => {
                IndexRegionBuilder::new_with_shape_environment(scalars, environment).unwrap()
            }
            None => IndexRegionBuilder::new(scalars).unwrap(),
        };
        let input = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([64]),
            )
            .unwrap();
        let output = builder
            .tensor(
                TensorRole::Output,
                F32::resolved_type(),
                Shape::from_dims([6]),
            )
            .unwrap();
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(6))
            .unwrap();
        let cursor = builder.dimension_expr(dimension).unwrap();
        let displaced = builder
            .sourced_linear_combination(offset, &[(SourcedIndexInteger::from(1_u64), cursor)])?;
        let value = builder.read(input, &[dimension], &[displaced]).unwrap();
        let write = builder.write(output, &[dimension], &[cursor]).unwrap();
        builder.output(write, value).unwrap();
        Ok(builder.build())
    }

    #[test]
    fn the_slice_law_realizes_t_plus_c_without_duplicating_the_cursor() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let (subject, _, cursor) = source_bearing_slice_subject(0, 58);
        let region = IndexRealizationLaw::slice_f32()
            .realize(&subject, &scalars)
            .unwrap();
        assert!(
            region.extent_sources().is_some(),
            "a source-bearing realization retains the program environment"
        );
        assert_eq!(
            region
                .tensors()
                .filter(|tensor| tensor.role() == TensorRole::Input)
                .count(),
            1,
            "the cursor is the selection symbol, not a second operand"
        );

        let combination = region
            .index_expressions()
            .find(|expression| matches!(expression.view(), IndexExprView::LinearCombination { .. }))
            .expect("the restricted axis is t + C");
        let IndexExprView::LinearCombination { constant, terms } = combination.view() else {
            unreachable!("matched above")
        };
        assert_eq!(
            constant.to_string(),
            "0",
            "a symbolic addend is not stored in the constant slot"
        );
        let coefficients: Vec<_> = terms.map(|term| term.coefficient().clone()).collect();
        assert!(
            coefficients.contains(&SourcedIndexInteger::Symbol(cursor)),
            "the canonical relation contains the source-bearing C * 1 term: {coefficients:?}"
        );
        assert!(
            coefficients.contains(&SourcedIndexInteger::from(1_u64)),
            "the cursor remains a coefficient-one term: {coefficients:?}"
        );
        assert_eq!(coefficients.len(), 2);

        assert!(
            region
                .accesses()
                .filter(|access| access.mode() == AccessMode::Read)
                .all(|access| access.bounds_proof()
                    == Some(BoundsProofView::Interval {
                        facts: IndexDomainFactSource::ShapeEnvironment,
                    })),
            "total access is discharged from the retained environment, not by specializing C"
        );
        assert_eq!(region.unknown_index_domain_predicates().count(), 0);
    }

    #[test]
    fn a_literal_slice_realization_does_not_attach_an_environment() {
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let table = program
            .input::<F32>(InputKey::new("table").unwrap(), Shape::from_dims([64, 128]))
            .unwrap();
        let selection = SliceSelection::new([
            SliceAxisSelection::static_window(4, Extent::new(6)),
            SliceAxisSelection::WholeAxis,
        ])
        .unwrap();
        let rows = F32Slice::apply(&mut program, &selection, table).unwrap();
        program
            .output(OutputKey::new("rows").unwrap(), rows)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        let subject =
            IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap();
        let region = IndexRealizationLaw::slice_f32()
            .realize(&subject, &FrozenScalarRegistry::standard().unwrap())
            .unwrap();
        assert!(
            region.extent_sources().is_none(),
            "a literal window must keep the environment-free realization path"
        );
    }

    #[test]
    fn a_foreign_environment_refuses_the_source_bearing_offset() {
        let foreign = slice_environment(0, 58);
        let error = slice_window_region(
            Some(foreign),
            SourcedIndexInteger::Symbol(slice_symbol("ghost")),
        )
        .expect_err("a symbol another environment declared is undeclared here");
        assert_eq!(
            error.to_string(),
            "sourced-extent.undeclared-symbol: slice/0::ghost is not declared by this program's shape environment"
        );
    }

    #[test]
    fn a_wrong_symbol_is_undeclared_in_this_environment() {
        let environment = slice_environment(0, 58);
        let other = ShapeSymbol::new(SymbolScope::new("slice/0").unwrap(), "d").unwrap();
        let error = slice_window_region(Some(environment), SourcedIndexInteger::Symbol(other))
            .expect_err("c's environment does not declare d");
        assert_eq!(
            error.to_string(),
            "sourced-extent.undeclared-symbol: slice/0::d is not declared by this program's shape environment"
        );
    }

    #[test]
    fn a_missing_source_environment_refuses_the_symbol() {
        let error = slice_window_region(None, SourcedIndexInteger::Symbol(slice_symbol("c")))
            .expect_err("no environment can declare the cursor");
        assert_eq!(
            error.to_string(),
            "sourced-extent.undeclared-symbol: slice/0::c is not declared by this program's shape environment"
        );
    }

    #[test]
    fn an_insufficient_interval_is_retained_as_unknown() {
        let region = slice_window_region(
            Some(slice_environment(0, 64)),
            SourcedIndexInteger::Symbol(slice_symbol("c")),
        )
        .expect("c is declared")
        .expect("an unproved read bound is an obligation, not a construction failure");
        let unknown: Vec<_> = region.unknown_index_domain_predicates().collect();
        assert_eq!(unknown.len(), 1);
        assert_eq!(
            unknown[0].reason(),
            IndexDomainUnknownReason::InsufficientFacts,
            "C in [0, 64] does not prove t + C stays inside 64: {unknown:?}"
        );
    }

    #[test]
    fn an_overflowing_window_is_refused_as_out_of_bounds() {
        let error = slice_window_region(
            Some(slice_environment(64, 64)),
            SourcedIndexInteger::Symbol(slice_symbol("c")),
        )
        .expect("c is declared")
        .expect_err("C = 64 against a 64-extent axis is outside every point");
        assert!(
            error.diagnostics().iter().any(|diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::CoordinateOutOfBounds { .. }
            )),
            "the overflowing window is a refutation: {error}"
        );
    }

    #[test]
    fn removing_the_bound_check_would_mint_an_unproved_region() {
        let region = slice_window_region(
            Some(slice_environment(0, 100)),
            SourcedIndexInteger::Symbol(slice_symbol("c")),
        )
        .expect("c is declared")
        .expect("syntax alone is not a proof");
        assert!(
            region.unknown_index_domain_predicates().next().is_some(),
            "an intentionally missing interval proof must leave the access unproved: {:?}",
            region.unknown_index_domain_predicates().collect::<Vec<_>>()
        );
        assert!(
            region
                .accesses()
                .filter(|access| access.mode() == AccessMode::Read)
                .all(|access| access.bounds_proof().is_none()),
            "no bounds proof may be recorded when the check cannot close"
        );
    }
}
