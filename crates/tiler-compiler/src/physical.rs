use std::error::Error;
use std::fmt;

use tiler_ir::index::IndexRealizationLaw;
use tiler_ir::semantic::{AttributeFieldId, CanonicalIntegerWidth, CanonicalValueView, F32};
use tiler_ir::shape::{Axis, Shape};

// The target-neutral scheduled-region IR and the backend-consumable structured
// kernel IR, with their intrinsic verifiers and canonical identities, live in
// `tiler_ir::schedule` and `tiler_ir::kernel` (ADR 0070). This module owns only
// the compiler-specific refinements layered on top of a verified region:
// semantic-occurrence binding, request-subject binding, and target feasibility.
// The shared vocabulary is re-exported so existing `crate::physical::*`
// importers continue to resolve.
pub(crate) use tiler_ir::kernel::VerifiedKernel;
pub(crate) use tiler_ir::schedule::{
    Access, AccessMode, AxisDecode, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContractionAxisSource, ContributorOrder, ContributorPartition, ExecutionBinding,
    IndexArithmetic, IndexRegion, InputOrdinal, KernelSchedule, LaunchPlan, LogicalAccess,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    PointwiseF32Expression, PointwiseF32Node, ReductionTopology, RegionId, ResourceRequirements,
    ScalarProgram, ScheduledRegion, TailPolicy, TensorRole,
};
use tiler_ir::schedule::{
    ArithmeticType, ScheduledRegionBuildError, ScheduledRegionBuilder, ScheduledRegionDiagnostic,
};

use crate::region::SemanticStage;
use crate::request::{
    BoundaryRead, NormalizedContraction, NormalizedEpilogue, NormalizedOutput,
    NormalizedOutputSubject, NormalizedSerialSum, NormalizedStaged, NumericalPermission,
    RecognizedPointwise, StrictF32NumericalContract, TargetProfile, VerifiedRequestSubject,
    VerifiedTargetRequest,
};
use crate::target::feasibility::UnrealizableSynchronization;
use crate::target::feasibility::{
    AvailabilityPhase, AxisRequirement, CapabilityAxis, DeferredSet, FeasibilityError,
    FeasibilityOutcome, FeasibilityProposal, ProvenEvidence, RejectionCause,
};
use crate::target::honourability::{
    DimensionBehaviour, NumericalDimension, NumericalRequirement, UnhonouredDimension,
};

/// The boundary tensor one recognized fold's contributor read binds.
///
/// **The recognized ordinal, not the first declared input.** A fold with a
/// prologue reads the intermediate that prologue region materialized; one
/// without reads *the declared input its own contributor names*, which is the
/// first only when the program declares one input for it to be. While every
/// elementwise walk had to read every declared input, a prologue-less fold's
/// walk read exactly one tensor and the program therefore declared exactly one,
/// so `Input { ordinal: 0 }` and the recognized ordinal could not differ.
/// `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs` made
/// them differ: `sum(b)` beside an independent `a * a` folds declared input `1`,
/// and a region binding ordinal `0` for it would fold the wrong caller tensor
/// under an intrinsically valid region.
///
/// It is the compiler side of the obligation `tiler-ir`'s schedule verifier
/// states as `ContributorTensor::DeclaredDomain`: the verifier admits either
/// tensor, and *which* one a region binds is a fact about the recognized
/// program.
///
/// The split's *final* pass deliberately does not ask: it folds partials its own
/// partial pass staged, so its read is the intermediate whatever the fold's
/// contributor domain is.
const fn declared_contributor_tensor(contributor_input: Option<u32>) -> TensorRole {
    match contributor_input {
        Some(ordinal) => TensorRole::Input {
            ordinal: InputOrdinal::new(ordinal),
        },
        None => TensorRole::Intermediate,
    }
}

/// The contributor tensor one recognized fold's regions bind.
///
/// Deriving it once here is what stops the serial region, the split's partial
/// pass, and the cooperative tile from disagreeing about where the contributors
/// live — three spellings of one fold, which the program assembler would then
/// bind to three different buffers.
const fn contributor_tensor(serial: &NormalizedSerialSum) -> TensorRole {
    declared_contributor_tensor(serial.contributor_input)
}

/// The declared input a *fused* region's contributor read binds, or `None` when
/// the recognized prologue has no fused spelling at all.
///
/// **The fused region reads the tensor the prologue read.** It carries the
/// prologue inside its own scalar program rather than staging it, so its single
/// contributor access binds the declared input the prologue's read named — and
/// that is the prologue's *sole* read rather than the first declared input,
/// for the reason [`declared_contributor_tensor`] states.
///
/// The pattern is the fused vocabulary's own precondition rather than a
/// narrowing. [`ScalarProgram::FusedMultiplyAddSerialSum`] applies `scale * x +
/// bias` to each contributor of one tensor, so a prologue it can spell is a
/// one-leaf expression and therefore carries exactly one read; and that read
/// must be dense, because the fused region addresses its input through a
/// [`LogicalAccess::ReductionContributor`] relation with nowhere to put a
/// structural one. `sum(permute(a) * 2.0 + 1.0)` would otherwise fold `a` where
/// the program said `permute(a)`.
///
/// **And the ordinal must be the first, which is a wall a crate down rather
/// than a property of the fusion.** `tiler_ir::schedule`'s
/// `verify_access_and_semantics` requires a `FusedMultiplyAddSerialSum` region's
/// contributor read to be `FIRST_INPUT` exactly, so a fused region over
/// `sum(b * 2.0 + 1.0)` beside an independent output reading `a` would be
/// proposed and then rejected as invalid compiler output. Declining it here
/// loses a *candidate* and never a program — the materialized prologue-and-fold
/// pair realizes that prologue, reading declared input `1` from a region whose
/// own vocabulary admits it —
/// and `admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary`
/// is what makes the candidate reachable again.
fn fused_contributor_tensor(prologue_reads: &[(u32, LogicalAccess)]) -> Option<TensorRole> {
    let [(ordinal, LogicalAccess::LinearIdentity)] = prologue_reads else {
        return None;
    };
    let tensor = TensorRole::Input {
        ordinal: InputOrdinal::new(*ordinal),
    };
    (*ordinal == InputOrdinal::FIRST.get()).then_some(tensor)
}

/// Recovers the scale and bias a fused serial sum's scalar program can spell.
///
/// [`ScalarProgram::FusedMultiplyAddSerialSum`] applies exactly `scale * x +
/// bias` to each contributor, so the fused single-region alternative exists
/// exactly when the recognized prologue *is* that expression over the sole
/// declared input. This decides that by node topology, ordered operands, and the
/// explicit root rather than by the constants alone: an algebraically similar
/// expression with a different association is a different binary32 function, and
/// admitting one here would bind an unproved reassociation to the request's
/// occurrences.
///
/// **And the prologue's read list must be one dense read, which is a correctness
/// condition rather than a narrowing.** The fused region has no prologue read of
/// its own: it addresses the declared input through a
/// [`LogicalAccess::ReductionContributor`] relation and applies the affine body
/// to each contributor. A prologue whose read carries a structural relation —
/// `sum(permute(a) * 2.0 + 1.0)` — has that relation nowhere to go in the fused
/// spelling, so fusing would silently fold `a` where the program said
/// `permute(a)` and return a wrong tensor. The expression alone cannot report
/// it: `permute(a) * 2.0 + 1.0` and `a * 2.0 + 1.0` are the same
/// [`PointwiseF32Expression`], and only the read list separates them.
/// [`fused_contributor_tensor`] is that gate, and it answers the second half of
/// the same question — *which* declared input the fused region reads.
///
/// Returning `None` loses a candidate and never a program — the materialized
/// two-region plan realizes every recognized prologue, and it is what a general
/// prologue compiles through.
///
/// A fold with *no* prologue answers `None` too, and loses nothing at all: there
/// is no prologue to fuse into it, its member partition has one part, and the
/// single reduction region reading the declared input already is the whole
/// program. Spelling it as `x * 1.0 + 0.0` would not be the same computation —
/// that expression maps `-0.0` to `+0.0` — so the affine vocabulary is genuinely
/// absent here rather than merely unused.
///
/// This is the single authority the whole compilation asks: the region builder,
/// the request-subject binding, and the whole-program numerical proof all reach
/// it, so "a fused alternative exists" and "the fused equivalence proof is
/// claimed" cannot disagree.
pub(crate) fn fused_prologue_constants(output: &NormalizedOutput) -> Option<(u32, u32)> {
    let serial = output.try_serial_sum()?;
    fused_contributor_tensor(&serial.prologue_reads)?;
    affine_prologue(serial.prologue.as_ref()?)
}

/// Recovers the scale and bias one recognized expression spells, or declines.
fn affine_prologue(expression: &PointwiseF32Expression) -> Option<(u32, u32)> {
    let [
        PointwiseF32Node::Input {
            ordinal: InputOrdinal::FIRST,
        },
        PointwiseF32Node::Constant { bits: scale },
        PointwiseF32Node::Multiply {
            lhs: multiply_lhs,
            rhs: multiply_rhs,
        },
        PointwiseF32Node::Constant { bits: bias },
        PointwiseF32Node::Add {
            lhs: add_lhs,
            rhs: add_rhs,
        },
    ] = expression.nodes()
    else {
        return None;
    };
    (multiply_lhs.index() == 0
        && multiply_rhs.index() == 1
        && add_lhs.index() == 2
        && add_rhs.index() == 3
        && expression.root().index() == 4)
        .then_some((*scale, *bias))
}

/// The physical shape of one staged family's realization, derived from its law.
///
/// **Derived from the *law*, never from the family.** The occurrence's operation
/// key appears nowhere here: what says which axes stage zero folds, what its
/// epilogue computes, and what its consuming pass evaluates is the closed typed
/// [`IndexRealizationLaw`] the registry carries for the family, so a second family
/// registering one of these laws is spelled by the same arm and a family
/// registered tomorrow with a law this vocabulary has no arm for is refused by
/// name rather than mis-spelled. Nothing else could serve: the shapes do not
/// determine the axes — a `[2, 2]` operand handed a `[2]` value names two
/// different reductions — and the attribute record does not interpret itself.
///
/// Every field is a *physical* statement about a scheduled region, and the
/// derivation is the one place the law's meaning is translated into this
/// vocabulary. What the law's own realized region sequence proves about the same
/// occurrence stays `tiler-ir`'s: refinement compares a provider's emission
/// against it byte for byte, and this plan is checked against nothing — it is
/// checked *by* being resubmitted through the ordinary verification path, whose
/// request-subject binding re-derives this plan and requires the region to match
/// it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StagedPlan {
    /// Reduced axes of the producing stage, canonical ascending.
    axes: Vec<Axis>,
    /// Boundary tensor the producing stage folds.
    contributor: TensorRole,
    /// Shape of the operand that stage folds.
    input_shape: Shape,
    /// Shape of the value the producing stage hands on: the folded operand's
    /// shape without the folded axes.
    handed_shape: Shape,
    /// Element count of that handed value, which is the producing stage's
    /// iteration count.
    handed_elements: u64,
    /// The chain the producing stage applies to its fold's value.
    fold_epilogue: PointwiseF32Expression,
    /// The consuming stage's reads, in access order.
    pass_reads: Vec<(TensorRole, LogicalAccess)>,
    /// The consuming stage's per-point expression.
    pass_expression: PointwiseF32Expression,
}

/// Derives the physical plan of one staged occurrence, or declines.
///
/// `None` is "this vocabulary has no scheduled region for this law", which the
/// caller reports as [`RegionVocabularyWall::StagedFamilyUnspellable`]. It is the
/// answer for a law with no arm here *and* for an occurrence whose own facts the
/// arm refuses — a folded extent no binary32 value equals, an empty fold, an
/// attribute record that does not decode — so the refusal set is exactly the
/// law's own, restated where a region would otherwise be built from facts the law
/// would not have realized.
pub(crate) fn staged_plan(normalized: &NormalizedStaged) -> Option<StagedPlan> {
    match &normalized.law {
        IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
            axes_attribute,
            eps_attribute,
        } => root_mean_square_scale_plan(normalized, *axes_attribute, *eps_attribute),
        // Fail-closed over a `#[non_exhaustive]` vocabulary: a law this profile
        // has no arm for is unspellable rather than approximated by whichever arm
        // it resembles. The single-region laws reach here too and are refused for
        // the same reason — an occurrence realized by one region is not a staged
        // stage, and no cover reaches this derivation for one.
        _ => None,
    }
}

/// Derives the plan of one root-mean-square scale occurrence.
///
/// The law's two stages, in physical terms:
///
/// - **Stage zero** folds `x_i * x_i` over the named axes into one value per kept
///   coordinate, then applies `Rsqrt(a / N + eps)` to it and hands the result on.
///   `N` is the folded contributor count as an exact binary32 payload, and the
///   epilogue divides by it rather than multiplying by a reciprocal, because the
///   two round a different number of times and the reference divides.
/// - **Stage one** reads the two operands and that handed value and writes
///   `w * (x * r)` pointwise, with the handed value read at its kept coordinates.
///
/// Both are the law's own steps in the law's own order, which is what makes the
/// region a realization of the occurrence rather than an algebraically equal
/// alternative.
///
/// Every refusal below is one the law itself makes, restated here because this
/// derivation runs *before* any realization is built:
///
/// - a folded extent no binary32 value equals (`rms-scale-extent-not-exact`),
///   because the emitted division would then be a different function;
/// - an empty fold (`rms-scale-empty-fold`), which has no first contributor to
///   seed at;
/// - operand or result shapes that disagree, an attribute record that does not
///   decode, or axes that are not a canonical in-range set.
///
/// And one refusal that is this layer's rather than the law's: **the two operands
/// must be distinct declared inputs.** `rms_norm(x, x)` is a legal occurrence
/// whose consuming pass would read one declared input twice, densely, which
/// `tiler_ir::schedule`'s own read-ordering rule refuses as two spellings of one
/// computation. Declining here loses that program rather than proposing a region
/// the verifier would reject as invalid compiler output.
///
/// **The same refusal covers an operand supplied by a materialization edge**,
/// which the recognizer now admits (`rms_norm(matmul(a, b), a)`) and this
/// vocabulary still cannot spell: the consuming pass would read that edge *and*
/// the value the producing stage handed it, and `TensorRole::Intermediate`
/// carries no ordinal, so nothing says which of the two each access binds. It is
/// declined by pattern rather than by a separate test — the destructuring below
/// admits two [`BoundaryRead::Input`] operands and nothing else — so a widened
/// staged operand vocabulary is a compile error here rather than a region built
/// from an ordinal the recognizer did not supply.
/// [`admit-a-scheduled-region-that-reads-two-materialization-edges`](../../../tickets/admit-a-scheduled-region-that-reads-two-materialization-edges.md)
/// owns the widening, which is `tiler-ir`'s before it is this layer's.
fn root_mean_square_scale_plan(
    normalized: &NormalizedStaged,
    axes_attribute: AttributeFieldId,
    eps_attribute: AttributeFieldId,
) -> Option<StagedPlan> {
    let [value_shape, weight_shape] = normalized.operand_shapes.as_slice() else {
        return None;
    };
    let [
        BoundaryRead::Input(value_input),
        BoundaryRead::Input(weight_input),
    ] = normalized.operand_reads.as_slice()
    else {
        return None;
    };
    if value_shape != &normalized.output_shape
        || weight_shape != &normalized.output_shape
        || value_input == weight_input
    {
        return None;
    }
    let axes = canonical_axes(&normalized.attribute_record, axes_attribute, value_shape)?;
    let eps_bits = float_bits(&normalized.attribute_record, eps_attribute)?;
    let handed_shape = value_shape.without_axes(&axes);
    let handed_elements = tiler_ir::schedule::element_count(&handed_shape).ok()?;
    let extent_bits = folded_extent_bits(value_shape, &axes)?;

    let mut fold = tiler_ir::schedule::PointwiseF32ExpressionBuilder::new();
    let total = fold.input(InputOrdinal::FIRST).ok()?;
    let extent = fold.constant(extent_bits).ok()?;
    let mean = fold.divide(total, extent).ok()?;
    let bias = fold.constant(eps_bits).ok()?;
    let biased = fold.add(mean, bias).ok()?;
    let root = fold.rsqrt(biased).ok()?;
    let fold_epilogue = fold.build(root).ok()?;

    // The reads in the canonical order a pointwise region requires: declared
    // inputs by ascending ordinal, then the handed value. Which of the two
    // operands leads is therefore the caller's declaration order rather than the
    // law's operand order, and the expression below binds each leaf to the read
    // that actually serves it.
    let value_leads = value_input < weight_input;
    let handed_map = if handed_shape == normalized.output_shape {
        // A fold over no axis hands one value per point, read densely. Not a
        // replication at all — `broadcast_decodes_are_replicating` refuses a map
        // that widens nothing, which is the canonicality rule that keeps one read
        // from having two spellings.
        LogicalAccess::LinearIdentity
    } else {
        LogicalAccess::BroadcastReplication {
            operand_shape: handed_shape.clone(),
            result_shape: normalized.output_shape.clone(),
            axes: kept_axis_decodes(&normalized.output_shape, &axes)?,
        }
    };
    let input_read = |ordinal: u32| {
        (
            TensorRole::Input {
                ordinal: InputOrdinal::new(ordinal),
            },
            LogicalAccess::LinearIdentity,
        )
    };
    let pass_reads = vec![
        input_read(if value_leads {
            *value_input
        } else {
            *weight_input
        }),
        input_read(if value_leads {
            *weight_input
        } else {
            *value_input
        }),
        (TensorRole::Intermediate, handed_map),
    ];

    let mut pass = tiler_ir::schedule::PointwiseF32ExpressionBuilder::new();
    let value_leaf = pass
        .input(InputOrdinal::new(u32::from(!value_leads)))
        .ok()?;
    let weight_leaf = pass.input(InputOrdinal::new(u32::from(value_leads))).ok()?;
    let root_leaf = pass.input(InputOrdinal::new(2)).ok()?;
    let scaled = pass.multiply(value_leaf, root_leaf).ok()?;
    let weighted = pass.multiply(weight_leaf, scaled).ok()?;
    let pass_expression = pass.build(weighted).ok()?;

    Some(StagedPlan {
        axes,
        contributor: TensorRole::Input {
            ordinal: InputOrdinal::new(*value_input),
        },
        input_shape: value_shape.clone(),
        handed_shape,
        handed_elements,
        fold_epilogue,
        pass_reads,
        pass_expression,
    })
}

/// Decodes one attribute's axis sequence, or declines.
///
/// Canonical means what the scheduled-region vocabulary means by it: strictly
/// ascending and in range for the shape being folded. The semantic inferencer has
/// its own rules, and this does not rely on them — a region built from a
/// non-canonical axis list would be refused by the schedule verifier as invalid
/// compiler output rather than declined as an unspellable one.
fn canonical_axes(
    attributes: &tiler_ir::semantic::OperationAttributes,
    attribute: AttributeFieldId,
    shape: &Shape,
) -> Option<Vec<Axis>> {
    let CanonicalValueView::Sequence(values) = attributes.get(attribute)?.view() else {
        return None;
    };
    let axes = values
        .iter()
        .map(|value| match value.view() {
            CanonicalValueView::Unsigned {
                width: CanonicalIntegerWidth::Bits32,
                bits,
            } => u32::try_from(bits).ok().map(Axis::new),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    tiler_ir::schedule::axes_are_canonical(&axes, shape.rank()).then_some(axes)
}

/// Decodes one attribute's exact binary32 payload, or declines.
///
/// The declared format is checked rather than assumed: the payload is carried as
/// bytes beside a nominal format key, so a record naming another width would
/// otherwise be reinterpreted as four `f32` bytes and reach the epilogue as a
/// different constant.
fn float_bits(
    attributes: &tiler_ir::semantic::OperationAttributes,
    attribute: AttributeFieldId,
) -> Option<u32> {
    let CanonicalValueView::FloatBits(payload) = attributes.get(attribute)?.view() else {
        return None;
    };
    if Some(payload.format()) != F32::resolved_type().nominal_key() {
        return None;
    }
    payload.bits().try_into().ok().map(u32::from_be_bytes)
}

/// Returns the exact binary32 payload of one fold's contributor count.
///
/// The law's own `rms-scale-extent-not-exact` and `rms-scale-empty-fold`
/// refusals, restated: the reference divides by the extent itself, so a count
/// whose nearest binary32 is not the count would make the emitted division a
/// different function; and an empty fold has no first contributor to seed at. The
/// representability test is integer-only, so it does not depend on the rounding
/// it exists to detect.
fn folded_extent_bits(shape: &Shape, axes: &[Axis]) -> Option<u32> {
    let points = axes
        .iter()
        .map(|axis| {
            usize::try_from(axis.get())
                .ok()
                .and_then(|position| shape.extents().get(position))
                .map(|extent| extent.get())
        })
        .try_fold(1_u64, |points, extent| {
            extent.and_then(|extent| points.checked_mul(extent))
        })?;
    if points == 0 || points >> points.trailing_zeros() >= 1 << 24 {
        return None;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "the odd-part test above proved this count is an exact binary32 value"
    )]
    Some((points as f32).to_bits())
}

/// Returns the decode of each kept axis against the published result shape.
///
/// One decode per axis of the handed value, in axis order: the kept axis at
/// result position `p` reads that whole result axis, so its divisor is `p`'s
/// row-major suffix product and its modulus that axis's extent. The folded result
/// axes are named by no decode at all, which is exactly what makes the read a
/// replication invariant in them.
fn kept_axis_decodes(result_shape: &Shape, axes: &[Axis]) -> Option<Vec<AxisDecode>> {
    let extents = result_shape.extents();
    let mut suffix = vec![1_u64; extents.len()];
    let mut running = 1_u64;
    for position in (0..extents.len()).rev() {
        suffix[position] = running;
        running = running.checked_mul(extents[position].get())?;
    }
    Some(
        (0..extents.len())
            .filter(|position| {
                u32::try_from(*position).is_ok_and(|axis| !axes.contains(&Axis::new(axis)))
            })
            .map(|position| {
                let extent = extents[position].get();
                if extent == 1 {
                    // An extent-one axis reads no coordinate, so its divisor and
                    // mirroring are unobservable and must carry the canonical
                    // spelling `AxisDecode::is_canonical` pins.
                    AxisDecode::fixed()
                } else {
                    AxisDecode::read(suffix[position], extent)
                }
            })
            .collect(),
    )
}

/// Stable name of the serial-or-direct baseline every cover region is offered.
///
/// Named beside the two parallel reduction strategies because it is withheld
/// the same way they are: a region the schedule vocabulary cannot spell earns a
/// [`crate::frontier::DeclinedStrategy`] against this name, so "no baseline for
/// this region" is a statement in the trace rather than an absence in it.
pub(crate) const SERIAL_BASELINE_STRATEGY: &str = "tiler.region.serial-baseline";

/// Which tensor one cover region's owning write targets.
///
/// **The cover decides this, and only the cover can.** A region writes a
/// declared program output when the cover assigns it one, and a materialized
/// intermediate when one of the cover's materialization edges names it as
/// producer. Asking the *request* instead — whether its whole program was
/// recognized as an elementwise one or as a reduction prologue — answers a
/// question about the program where a question about the region belongs, and
/// gives every region a cover places the same answer.
///
/// **A cover may assign both, and that is a region of two dispatches rather than
/// a write of two tensors.** [`tiler_ir::program::ValueRole`] is exclusive and a
/// dispatch owns one write, so a region whose value is published *and* consumed
/// stages the value its consumer reads across and publishes a copy of it from a
/// second dispatch — structurally a split reduction's final pass, one fold up.
/// [`Self::tensor`] therefore answers for the region's *first* dispatch;
/// [`publishing_copy_region`] builds the second, and nothing else may read the
/// third variant as if it named one write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionWrite {
    /// The cover assigns this region a declared program output.
    ProgramOutput,
    /// A materialization edge names this region as the producer of a value
    /// another region reads.
    Materialized,
    /// A materialization edge names this region as producer *and* the cover
    /// assigns it a declared program output: the value is both published and
    /// consumed.
    MaterializedAndPublished,
}

impl RegionWrite {
    /// Returns the boundary tensor role the region's *first* dispatch writes.
    ///
    /// A published-and-consumed region's first dispatch stages the value, which
    /// is why it answers the same as [`Self::Materialized`] here: the
    /// publication is the second dispatch's write and is built separately.
    pub(crate) const fn tensor(self) -> TensorRole {
        match self {
            Self::ProgramOutput => TensorRole::Output,
            Self::Materialized | Self::MaterializedAndPublished => TensorRole::Intermediate,
        }
    }

    /// Returns whether this region needs a second dispatch to publish a copy.
    pub(crate) const fn publishes_a_copy(self) -> bool {
        matches!(self, Self::MaterializedAndPublished)
    }
}

/// Which region of the bounded schedule vocabulary spells one cover region.
///
/// This is what [`spell_region`] decides, and it is deliberately a decision
/// about the *region* rather than about the program: the same recognized
/// program is covered many ways, and only the placed occurrences say which
/// region — if any — the vocabulary has for each part.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionSpellingKind {
    /// An elementwise pass over the declared inputs, writing the tensor the
    /// cover assigned it.
    Pointwise(RegionWrite),
    /// A strict serial fold over the materialized contributor domain.
    SerialSum,
    /// A strict tensor contraction over the declared inputs.
    Contraction,
    /// The fold and its affine prologue realized as one region.
    FusedSerialSum,
    /// An elementwise pass over one staged producer result and, where the
    /// expression names them, declared inputs.
    ///
    /// Distinct from [`Self::Pointwise`] although both build a
    /// [`ScalarProgram::PointwiseF32`] region, because the two are built from
    /// different facts: a pointwise region's reads are the declared inputs in
    /// declaration order, and an epilogue's are the recognized read list, whose
    /// positions and boundary roles are independent.
    Epilogue(RegionWrite),
    /// The producing stage of a staged family: a fold carrying its own epilogue,
    /// handing one value per kept coordinate to the stage that follows.
    ///
    /// It carries no [`RegionWrite`], and the absence is the claim: the value it
    /// writes exists only inside the law's realization, so no cover can publish it
    /// and [`spell_staged`] refuses a write role that says otherwise.
    StagedFold,
    /// The consuming stage of a staged family: a pointwise pass over the
    /// occurrence's operands and the value the producing stage handed it.
    ///
    /// Distinct from [`Self::Epilogue`] although both build a
    /// [`ScalarProgram::PointwiseF32`] region, for the reason that variant is
    /// distinct from [`Self::Pointwise`]: this one's reads and expression are
    /// derived from the occurrence's *law*, not from a recognized walk, so a
    /// region built by the epilogue path would be spelling a chain the program
    /// does not contain.
    StagedPass(RegionWrite),
}

/// One cover region's spelling, together with the ordered named output whose
/// recognized partition it belongs to.
///
/// **The output travels with the kind because the kind alone no longer says
/// which shapes to build from.** A program declaring several outputs carries one
/// recognized partition per output, so "this region is an elementwise pass" is
/// only half an answer — which expression, over which domain, publishing which
/// key, is the other half, and it is the resolved output that holds it. With one
/// declared output the position is always zero and nothing about the built
/// region moves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionSpelling {
    output: usize,
    kind: RegionSpellingKind,
}

impl RegionSpelling {
    const fn new(output: usize, kind: RegionSpellingKind) -> Self {
        Self { output, kind }
    }

    /// Returns the declaration position of the output this region implements.
    pub(crate) const fn output(self) -> usize {
        self.output
    }

    /// Returns which region of the bounded vocabulary spells it.
    pub(crate) const fn kind(self) -> RegionSpellingKind {
        self.kind
    }
}

/// Why the bounded schedule vocabulary cannot spell one cover region.
///
/// Every variant is a fact about the region *the cover placed*, decided from
/// its exact occurrences — never about the target, the contract, or the
/// program's admissibility. Each occurrence in such a region already resolved
/// its lowering capability and the grouping is already legal; what is missing
/// is a scheduled region that expresses those occurrences together.
///
/// The three walls are the ones
/// `docs/research/program-planning/minimum-correct-physical-realization-profile.md`
/// names as *widenings with their own owning tickets*. Each becomes an offer
/// when its widening lands, with no change to this classification's shape. The
/// profile's third named wall — a reduction folding a declared input directly —
/// never had a variant here and still has none, but for the opposite reason: it
/// was refused at the request boundary, and
/// `admit-a-reduction-over-a-declared-input-tensor` made it a region this
/// vocabulary spells. `sum(x)` resolves to [`RegionSpellingKind::SerialSum`] by
/// the ordinary member match, so there is nothing left to decline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionVocabularyWall {
    /// The region covers part of a recognized occurrence group the vocabulary
    /// spells only whole.
    ///
    /// The scalar program carries the recognizer's expression entire — node
    /// topology, ordered operands, constant bits, and explicit root — and no
    /// sub-expression of it, so a cover that split the chain has no region to
    /// build for either half.
    PartialCoverage,
    /// The region covers the reduction together with part, but not all, of its
    /// recognized prologue.
    ///
    /// Distinct from [`Self::PartialCoverage`] because the wall is a different
    /// one: [`ScalarProgram::FusedMultiplyAddSerialSum`] fuses the *whole*
    /// prologue into the fold or nothing, so a partially fused region is
    /// unspellable even where each half separately would not be.
    PartialFusedProgram,
    /// The region is the whole recognized program, and the fused reduction
    /// vocabulary cannot spell its prologue.
    ///
    /// The one wall that was already decided and already lost a candidate
    /// silently: [`fused_region`] answers `None` for every prologue that is not
    /// the affine one, and the materialized cover still realizes the program.
    FusedPrologueUnspellable,
    /// The region covers stages of one staged occurrence that no scheduled
    /// region computes together, or a law this profile has no spelling for.
    ///
    /// **A wall of its own rather than [`Self::PartialCoverage`], because the
    /// two say opposite things about the cover.** Partial coverage means the
    /// cover grouped occurrences no recognized partition owns — the cover is
    /// wrong for this program. This means the cover is *right*: the region
    /// covers exactly stages one recognized occurrence realizes as, region
    /// formation enumerated them from the family's own law, and the refusal is
    /// about the region rather than the grouping.
    ///
    /// **Two conditions raise it, and neither is the vocabulary gap this variant
    /// was introduced for.** That gap is closed: `tiler::rms-norm-f32@1`'s two
    /// stages are spelled by [`RegionSpellingKind::StagedFold`] and
    /// [`RegionSpellingKind::StagedPass`]. What remains is
    ///
    /// - **a region carrying more than one stage.** No scheduled region folds a
    ///   contributor domain *and* evaluates a per-point expression over the
    ///   fold's result, because those are two iteration domains — which is why
    ///   the law realizes such an occurrence as a sequence at all.
    /// - **a staged law with no arm in [`staged_plan`]**, or an occurrence whose
    ///   own facts that arm refuses. The wildcard there is fail-closed, so a
    ///   family registered tomorrow under a law this profile cannot spell is
    ///   declined by name rather than mis-spelled.
    StagedFamilyUnspellable,
}

impl RegionVocabularyWall {
    /// Returns the stable reason code naming this wall.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::PartialCoverage => "region-partial-coverage",
            Self::PartialFusedProgram => "region-partial-fused-program",
            // The rule `build_fused_scheduled_region` already names for the same
            // condition, so one code covers both spellings of the fact.
            Self::FusedPrologueUnspellable => "fused-prologue-unspellable",
            Self::StagedFamilyUnspellable => "region-staged-family-unspellable",
        }
    }
}

/// Decides which scheduled region spells the occurrences one cover placed.
///
/// **The subject is what is read, and the recognition is the context it is read
/// against.** `members` are the exact occurrences the cover grouped and `write`
/// is the tensor the cover assigned that group; the request supplies what those
/// occurrences *are* — the recognized expression, its shapes, and how the
/// program partitions into prologue and fold — which a region subject does not
/// and cannot carry. A derivation that read the recognition alone would answer
/// the same for every region of a program, which is exactly the defect this
/// replaces.
///
/// The answer is total: every member set either names a region this vocabulary
/// builds or a wall it hit, and there is no third state. That totality is the
/// whole point — the caller converts an `Err` into a typed decline, so silence
/// stops being an admissible answer for a region a legal cover placed.
///
/// **The search is over every recognized output, not over one whole-program
/// partition.** The recognizer produces one partition per ordered named output
/// and a region straddling two outputs' walks matches none of them, so it is
/// refused as partial coverage — the right answer, since no scheduled region
/// computes two published results.
///
/// **The first match is no longer the only match, and declaration order is the
/// decided tie-break.** `check_output_cover` admits exactly one overlap: a walk
/// that is one whole *part* of a longer walk's partition and publishes the value
/// that part hands across the boundary. Both outputs then own that member set,
/// so the scan finds two claimants. Taking the first is correct rather than
/// arbitrary, because the two claimants are recognitions of *one value over one
/// occurrence set* — the admitted overlap requires exactly that — so they spell
/// the same expression over the same domain and differ only in which
/// [`crate::request::NormalizedOutput`] arm the walk was recorded under.
/// `both_claimants_of_a_published_and_consumed_part_spell_one_region` holds it,
/// because an argument that the choice is immaterial is worth less than a check
/// that says no when it stops being.
///
/// The resolved position still matters downstream — it is what
/// [`crate::request::VerifiedTargetRequest::output_at`] resolves the region's
/// shapes from — which is exactly why the equivalence is asserted rather than
/// assumed.
pub(crate) fn spell_region(
    request: &VerifiedTargetRequest,
    members: &[SemanticStage],
    write: RegionWrite,
) -> Result<RegionSpelling, RegionVocabularyWall> {
    // Retained across the walk rather than returned immediately, so a region
    // that partially fuses one output's reduction still reports that wall even
    // when a later output is examined after it. Only a reduction can raise it,
    // and the walks are disjoint, so at most one output can set it.
    let mut partial_fused = None;
    for (position, output) in request.normalized().outputs().iter().enumerate() {
        if let Some(spelling) = spell_output(output, position, members, write, &mut partial_fused) {
            return spelling;
        }
    }
    Err(partial_fused.unwrap_or(RegionVocabularyWall::PartialCoverage))
}

/// Decides which scheduled region spells one member set within one output's
/// recognized partition.
///
/// `None` is "not this output's", which is the caller's signal to keep looking;
/// `Some` is a decision, and a decided wall is as final as a decided spelling.
///
/// Recursive over an epilogue chain, so every region of the producer's partition
/// is spelled exactly as it would be if the producer were the whole declared
/// output — the fold, its prologue, its fused form, and the walls each of them
/// raises. Only the epilogue's own part is new, and the chain *as a whole* is
/// deliberately not a part: no scheduled region computes a fold and an
/// expression over its result, so a cover grouping both is declined rather than
/// resolved here.
fn spell_output(
    output: &NormalizedOutput,
    position: usize,
    members: &[SemanticStage],
    write: RegionWrite,
    partial_fused: &mut Option<RegionVocabularyWall>,
) -> Option<Result<RegionSpelling, RegionVocabularyWall>> {
    match output {
        NormalizedOutput::Pointwise(normalized) => (members == normalized.members).then(|| {
            Ok(RegionSpelling::new(
                position,
                RegionSpellingKind::Pointwise(write),
            ))
        }),
        NormalizedOutput::Contraction(normalized) => (members == normalized.members).then(|| {
            Ok(RegionSpelling::new(
                position,
                RegionSpellingKind::Contraction,
            ))
        }),
        NormalizedOutput::SerialSum(normalized) => {
            let recognized = &normalized.members;
            // Asked through the partition rather than by comparing against the
            // prologue member list directly: a fold over a declared input has
            // no prologue part, and an empty member set matching an empty list
            // would spell a prologue region for a program that has none — which
            // `pointwise_region` then panics on rather than building.
            //
            // Defence in depth against the enumeration as it stands, stated
            // rather than presented as a live gate: `GovernedPhysicalProvider`
            // is `spell_region`'s only caller and answers an empty member set
            // with an empty offer before reaching here, so no cover the search
            // currently places drives the distinction.
            if normalized
                .prologue_members()
                .is_some_and(|prologue| members == prologue)
            {
                return Some(Ok(RegionSpelling::new(
                    position,
                    RegionSpellingKind::Pointwise(write),
                )));
            }
            if members == recognized.reduction() {
                return Some(Ok(RegionSpelling::new(
                    position,
                    RegionSpellingKind::SerialSum,
                )));
            }
            // Unreachable for a prologue-less fold, and correctly so: its two
            // parts coincide, the reduction arm above answers first, and
            // there is no prologue for a fused region to absorb.
            if members == recognized.all() {
                return Some(
                    fused_prologue_constants(output)
                        .map(|_| RegionSpelling::new(position, RegionSpellingKind::FusedSerialSum))
                        .ok_or(RegionVocabularyWall::FusedPrologueUnspellable),
                );
            }
            if members
                .iter()
                .any(|member| recognized.reduction().contains(member))
            {
                *partial_fused = Some(RegionVocabularyWall::PartialFusedProgram);
            }
            None
        }
        NormalizedOutput::Epilogue(chain) => {
            if members == chain.members {
                return Some(Ok(RegionSpelling::new(
                    position,
                    RegionSpellingKind::Epilogue(write),
                )));
            }
            spell_output(&chain.producer, position, members, write, partial_fused)
        }
        // A decision, not a fall-through, for a member set that is one of this
        // occurrence's own stages: returning `None` there would let the scan
        // continue and report partial coverage, which names the cover instead of
        // the region's own answer.
        //
        // The ownership test is asked of the recognized shape rather than
        // restated, so a widening of the staged shape — a third stage, atoms
        // spanning occurrences — moves this arm with the recognizer instead of
        // leaving the two to disagree about which regions are the occurrence's.
        // The arms above compare member lists themselves because each has to
        // know *which* part matched to name a spelling kind; a staged
        // occurrence's own partition has one part, and *which stage* is
        // [`spell_staged`]'s question, so ownership is the whole of this arm's.
        //
        // A member set the occurrence's stages do not cover falls through to the
        // producer across its staged operand's materialization edge, exactly as
        // an epilogue chain falls through to its own. That recursion is what
        // spells the contraction in `rms_norm(matmul(a, b), a)`, and it is a
        // fall-through rather than a decision because a member set outside both
        // is another output's.
        NormalizedOutput::Staged(normalized) => {
            if normalized.owns_stage_members(members) {
                return Some(spell_staged(normalized, position, members, write));
            }
            normalized.producer.as_deref().and_then(|producer| {
                spell_output(producer, position, members, write, partial_fused)
            })
        }
    }
}

/// Decides which scheduled region spells one stage of a staged occurrence.
///
/// Called only for a member set every atom of which is a stage of `normalized`,
/// so the question left is *which* stages, and there are exactly three answers:
/// the producing stage alone, the consuming stage alone, or something no
/// scheduled region computes.
///
/// **A region carrying both stages is a wall rather than a spelling**, and the
/// reason is the vocabulary's rather than the cover's: no scheduled region folds a
/// contributor domain and evaluates a per-point expression over the fold's result,
/// because those are two iteration domains. That is the same fact
/// [`RegionSpellingKind::Epilogue`]'s own chain states — a fold and an expression
/// over its result are two regions — and it is why the law realizes the occurrence
/// as a sequence in the first place.
///
/// **The producing stage must be materializing**, and the refusal is derived: the
/// value it writes is law-internal, so it is no declared output and no cover can
/// publish it. A write role saying otherwise is a cover this profile cannot
/// assemble, and naming it here is what stops a region being built for one.
fn spell_staged(
    normalized: &NormalizedStaged,
    position: usize,
    members: &[SemanticStage],
    write: RegionWrite,
) -> Result<RegionSpelling, RegionVocabularyWall> {
    if staged_plan(normalized).is_none() {
        return Err(RegionVocabularyWall::StagedFamilyUnspellable);
    }
    let fold = SemanticStage::first(normalized.member);
    let pass = fold.next_stage();
    if members == [fold] && write == RegionWrite::Materialized {
        return Ok(RegionSpelling::new(
            position,
            RegionSpellingKind::StagedFold,
        ));
    }
    if members == [pass] {
        return Ok(RegionSpelling::new(
            position,
            RegionSpellingKind::StagedPass(write),
        ));
    }
    Err(RegionVocabularyWall::StagedFamilyUnspellable)
}

/// Stable candidate identity used when assessing one scheduled region.
const REGION_PROPOSAL_CANDIDATE: &str = "tiler.prototype.scheduled-region";

/// Stable candidate identity used when resolving a numerical contract alone.
const CONTRACT_PROPOSAL_CANDIDATE: &str = "tiler.prototype.numerical-contract";

/// A verified scheduled region bound to one compilation request.
///
/// This wraps the target-neutral [`tiler_ir::schedule::VerifiedScheduledRegion`]
/// with the compiler-owned refinements the shared IR deliberately excludes: the
/// exact semantic occurrences the region covers, the target profile it was
/// assessed against, and the request subject it belongs to. The inner region is
/// intrinsically verified before any of these bindings are formed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedScheduledRegion {
    verified: tiler_ir::schedule::VerifiedScheduledRegion,
    semantic_members: Vec<SemanticStage>,
    target_profile: TargetProfile,
    request_subject: VerifiedRequestSubject,
    admission: AdmissionEvidence,
}

impl VerifiedScheduledRegion {
    pub(crate) fn region(&self) -> &ScheduledRegion {
        self.verified.region()
    }
    /// Returns the shared-IR verified region this compiler binding wraps.
    ///
    /// Structured-kernel lowering consumes the shared verified value directly,
    /// so a kernel can only ever refine an intrinsically verified schedule.
    pub(crate) const fn verified(&self) -> &tiler_ir::schedule::VerifiedScheduledRegion {
        &self.verified
    }
    pub(crate) fn requirements(&self) -> ResourceRequirements {
        self.verified.requirements()
    }
    /// Returns the canonical, transient-ordinal-independent identity of the inner
    /// verified region.
    ///
    /// This is the shared-IR identity (ADR 0070) derived purely from the
    /// normalized schedule content, so equivalent regions proposed by different
    /// physical providers share it. The implementation frontier folds it into a
    /// per-proposal identity that additionally distinguishes provider provenance.
    pub(crate) fn canonical_identity(
        &self,
    ) -> &tiler_ir::schedule::CanonicalScheduledRegionIdentity {
        self.verified.canonical_identity()
    }
    pub(crate) const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
    }
    /// Returns the exact semantic occurrences this region covers.
    ///
    /// These are graph-local operation ordinals of the verified program, not a
    /// fixed role vocabulary, so a schedule cannot claim coverage of operations
    /// the request boundary did not actually recognize.
    pub(crate) fn semantic_members(&self) -> &[SemanticStage] {
        &self.semantic_members
    }
    pub(crate) fn matches_request(&self, request: &VerifiedTargetRequest) -> bool {
        self.request_subject == *request.subject()
    }

    /// Returns the complete hard-feasibility admission for this exact region.
    pub(crate) const fn admission(&self) -> &AdmissionEvidence {
        &self.admission
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PhysicalError {
    Intrinsic {
        rule: &'static str,
        region: RegionId,
    },
    Target {
        rule: &'static str,
        region: RegionId,
        required: u64,
        available: u64,
    },
    /// A numerical dimension the target declares it cannot honour as required.
    ///
    /// A distinct variant rather than a `Target` with two numbers, because the
    /// rejection ADR 0076 item 5 requires names a dimension, a required
    /// behaviour, a declared means, the behaviour the target does honour, and
    /// the declaring profile — none of which is a quantity, and all of which the
    /// retired `strict-f32: required 1, available 0` shape discarded.
    Numerical {
        region: RegionId,
        cause: UnhonouredDimension,
    },
    /// A synchronization realization no available target fact speaks to.
    ///
    /// Distinct from [`Self::Synchronization`], which carries a fact that says
    /// *no*. This one carries no fact because there is none: the profile has
    /// never been asked about this subject. Both reject before
    /// executable-frontier admission and neither is a cost, but only one of them
    /// can name a refusing authority, and inventing one for the other would be
    /// the false attribution the atomic fact exists to prevent.
    UnrealizedSynchronization {
        region: RegionId,
        subject: tiler_ir::schedule::SynchronizationSubject,
    },
    /// A synchronization realization the target declares it cannot provide.
    ///
    /// A distinct variant for the reason [`Self::Numerical`] is one: the
    /// rejection names a complete subject — kind, arrival scope, publication
    /// scope, fenced domains, ordering — and the profile that refused it, none of
    /// which is a quantity. Reporting it as a `Target` bound would restate an
    /// atomic subject as a number and lose exactly what makes it uncomposable.
    Synchronization {
        region: RegionId,
        cause: Box<UnrealizableSynchronization>,
    },
    Refinement {
        rule: &'static str,
        region: RegionId,
    },
    ShapeProductOverflow {
        region: RegionId,
    },
}

impl fmt::Display for PhysicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Intrinsic { rule, region } => {
                write!(
                    formatter,
                    "schedule.intrinsic.{rule}: region {} rejected",
                    region.get()
                )
            }
            Self::Target {
                rule,
                region,
                required,
                available,
            } => write!(
                formatter,
                "schedule.target.{rule}: region {} requires {required}, available {available}",
                region.get()
            ),
            Self::Numerical { region, cause } => {
                write!(
                    formatter,
                    "schedule.numerics.{}: region {} requires {}, target declares {}",
                    cause.dimension().key(),
                    region.get(),
                    cause.required().key(),
                    cause.means().label(),
                )?;
                if let Some(honoured) = cause.honoured() {
                    write!(formatter, " and honours {}", honoured.key())?;
                }
                write!(formatter, " (profile {})", cause.profile().key())
            }
            Self::UnrealizedSynchronization { region, subject } => write!(
                formatter,
                "schedule.synchronization.unrealized: region {} requires {} arriving {}, \
                 publishing {}, fencing{}{}, ordered {}; no available fact declares it",
                region.get(),
                subject.kind.key(),
                subject.execution_scope.key(),
                subject.visibility_scope.key(),
                if subject.fenced_spaces.workgroup {
                    " workgroup"
                } else {
                    ""
                },
                if subject.fenced_spaces.device {
                    " device"
                } else {
                    ""
                },
                subject.ordering.key(),
            ),
            Self::Synchronization { region, cause } => {
                let subject = cause.subject();
                write!(
                    formatter,
                    "schedule.synchronization: region {} requires {} arriving {}, publishing {}, \
                     fencing{}{}, ordered {}; profile {} declares it unrealizable",
                    region.get(),
                    subject.kind.key(),
                    subject.execution_scope.key(),
                    subject.visibility_scope.key(),
                    if subject.fenced_spaces.workgroup {
                        " workgroup"
                    } else {
                        ""
                    },
                    if subject.fenced_spaces.device {
                        " device"
                    } else {
                        ""
                    },
                    subject.ordering.key(),
                    cause.fact().provenance().profile().key(),
                )
            }
            Self::Refinement { rule, region } => write!(
                formatter,
                "kernel.refinement.{rule}: kernel for region {} rejected",
                region.get()
            ),
            Self::ShapeProductOverflow { region } => write!(
                formatter,
                "schedule.shape.element-count: region {} exceeds u64",
                region.get()
            ),
        }
    }
}

impl Error for PhysicalError {}

#[allow(
    dead_code,
    reason = "canonical region constructor the governed physical provider proposes through the frontier; retained as the single definition of each recognized region and exercised by its own tests"
)]
pub(crate) fn build_scheduled_regions(
    request: &VerifiedTargetRequest,
) -> Result<Vec<VerifiedScheduledRegion>, PhysicalError> {
    // The prologue of a materialized serial sum, which every cover placing it
    // materializes for the fold that reads it.
    let output = request.sole_output();
    let (pointwise, pointwise_members) =
        pointwise_region(request, output, RegionWrite::Materialized);
    let (reduction, reduction_members) =
        reduction_region(request, output, RegionWrite::ProgramOutput);
    Ok(vec![
        verify_schedule(pointwise, pointwise_members, request)?,
        verify_schedule(reduction, reduction_members, request)?,
    ])
}

#[allow(
    dead_code,
    reason = "canonical region constructor the governed physical provider proposes through the frontier; retained as the single definition of each recognized region and exercised by its own tests"
)]
pub(crate) fn build_fused_scheduled_region(
    request: &VerifiedTargetRequest,
) -> Result<VerifiedScheduledRegion, PhysicalError> {
    let (fused, members) = fused_region(request, request.sole_output(), RegionWrite::ProgramOutput)
        .ok_or(PhysicalError::Intrinsic {
            rule: "fused-prologue-unspellable",
            region: RegionId::new(0),
        })?;
    verify_schedule(fused, members, request)
}

/// Builds the canonical elementwise scheduled region for one request.
///
/// This constructs the raw, not-yet-verified region and its recognized
/// elementwise members. It carries the recognizer's own
/// [`PointwiseF32Expression`] rather than a spelling rebuilt here, which is what
/// lets one builder serve every expression the recognizer admits instead of one
/// shape it was taught.
///
/// **`write` comes from the cover, not from the recognition.** The elementwise
/// region a cover places is a reduction prologue when a materialization edge
/// names it as producer and a whole-program region when the cover assigns it a
/// declared output, and those are the same two regions this builder always
/// built — it simply used to decide between them by asking which whole-program
/// recognizer had matched, which is a question about the program.
///
/// **`output` comes from the spelling, not from the request.** A program
/// declaring several ordered outputs carries one recognized partition per
/// output, so the expression, domain, and member set this region is built from
/// are the resolved output's rather than the request's.
///
/// # Panics
///
/// Panics when asked for an output whose recognized shape is a contraction, or
/// for a reduction that has no prologue to build a region from. Both are invalid
/// compiler output rather than caller errors: the frontier offers this region only
/// for an elementwise or reduced-elementwise subject, and [`spell_region`] resolves
/// a prologue spelling only through
/// [`crate::request::NormalizedSerialSum::prologue_members`], which a
/// prologue-less fold answers `None` for. Panicking is what keeps a cover that
/// somehow named one from silently receiving an identity copy kernel — a
/// materialization, and its rounding boundary, the caller's program never asked
/// for.
///
/// It applies no intrinsic, subject-binding, or feasibility gate. The
/// implementation frontier and its providers use it to obtain a canonical region
/// they then re-submit through the ordinary checked verification path, including
/// for a domain the governed profile cannot dispatch.
pub(crate) fn pointwise_region(
    request: &VerifiedTargetRequest,
    output: &NormalizedOutput,
    write: RegionWrite,
) -> (ScheduledRegion, Vec<SemanticStage>) {
    let (shape, elements, expression, members, recognized_reads) =
        if let Some(pointwise) = output.pointwise() {
            (
                pointwise.shape.clone(),
                pointwise.elements,
                pointwise.expression.clone(),
                pointwise.members.clone(),
                pointwise.reads.clone(),
            )
        } else {
            let serial = output.serial_sum();
            (
                serial.input_shape.clone(),
                serial.input_elements,
                // A fold's prologue is `f32` by the fold family's own key; see
                // the recognizer's `recognize_reduction`, which states the width
                // at the walk it drives.
                RecognizedPointwise::F32(
                    serial
                        .prologue
                        .clone()
                        .expect("a prologue region is spelled only for a fold that has a prologue"),
                ),
                serial.members.pointwise().to_vec(),
                serial.prologue_reads.clone(),
            )
        };
    // The recognized read list, not the declared arity: one declared input may be
    // read twice — once densely and once through a relation — so the read count
    // and the interface width are different numbers.
    let reads: Vec<(TensorRole, LogicalAccess)> = recognized_reads
        .iter()
        .map(|(ordinal, map)| {
            (
                TensorRole::Input {
                    ordinal: InputOrdinal::new(*ordinal),
                },
                map.clone(),
            )
        })
        .collect();
    let region = elementwise_region(
        request,
        RegionId::new(0),
        shape,
        elements,
        &reads,
        expression,
        write,
    );
    (region, members)
}

/// The element range one elementwise read addresses.
///
/// A structural read addresses its *operand's* range, which is a different
/// number from the region's domain — that difference is exactly what a widening
/// broadcast is. Deriving it from the relation the read carries is what stops a
/// region from binding a widened read against a domain-sized proof.
fn addressed_elements(map: &LogicalAccess, elements: u64) -> u64 {
    match map {
        LogicalAccess::ReindexBijection { operand_shape, .. }
        | LogicalAccess::BroadcastReplication { operand_shape, .. } => {
            tiler_ir::schedule::element_count(operand_shape).unwrap_or(elements)
        }
        _ => elements,
    }
}

/// Builds one elementwise scheduled region from its ordered reads.
///
/// **The read list is the parameter, because it is the only thing that differs
/// between the two elementwise regions this profile builds.** A whole-program or
/// prologue region reads every declared input in declaration order; an epilogue
/// reads one staged value and whichever declared inputs its expression names.
/// Everything else — witness numbering, the owning write, the ownership proof,
/// the launch — is the same region shape, and stating it once is what keeps the
/// two from drifting into two shapes.
fn elementwise_region(
    request: &VerifiedTargetRequest,
    id: RegionId,
    shape: Shape,
    elements: u64,
    reads: &[(TensorRole, LogicalAccess)],
    expression: RecognizedPointwise,
    write: RegionWrite,
) -> ScheduledRegion {
    let write_tensor = write.tensor();
    // Witness numbering is access numbering, so two accesses cannot end up
    // proving against one witness, and the write's witness follows the reads
    // rather than sitting at a constant.
    let write_witness = u32::try_from(reads.len()).unwrap_or(u32::MAX);
    let witness =
        |position: usize| BoundsWitnessId::new(u32::try_from(position).unwrap_or(u32::MAX));
    let mut accesses: Vec<Access> = reads
        .iter()
        .enumerate()
        .map(|(position, (tensor, map))| Access {
            tensor: *tensor,
            component_role: None,
            mode: AccessMode::Read,
            map: map.clone(),
            bounds: witness(position),
            ownership: None,
        })
        .collect();
    let mut bounds_proofs: Vec<BoundsProof> = reads
        .iter()
        .enumerate()
        .map(|(position, (tensor, map))| BoundsProof {
            id: witness(position),
            tensor: *tensor,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: addressed_elements(map, elements),
            },
        })
        .collect();
    accesses.push(Access {
        tensor: write_tensor,
        component_role: None,
        mode: AccessMode::Write,
        map: LogicalAccess::LinearIdentity,
        bounds: BoundsWitnessId::new(write_witness),
        ownership: Some(OwnershipWitnessId::new(0)),
    });
    bounds_proofs.push(BoundsProof {
        id: BoundsWitnessId::new(write_witness),
        tensor: write_tensor,
        component_role: None,
        kind: BoundsProofKind::LinearRange {
            element_count: elements,
        },
    });
    ScheduledRegion {
        index: IndexRegion {
            id,
            iteration_shape: shape,
            accesses,
            bounds_proofs,
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: write_tensor,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: elements,
                },
            },
            // The recognized width decides the scalar program, because the two
            // per-point vocabularies are different functions rather than two
            // spellings of one: a `bf16` multiply rounds to eight significand
            // bits and a binary32 one to twenty-four, and only the matching
            // variant carries the expression the recognizer proved.
            scalar_program: match expression {
                RecognizedPointwise::F32(expression) => ScalarProgram::PointwiseF32(expression),
                RecognizedPointwise::Bf16(expression) => ScalarProgram::PointwiseBf16(expression),
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: linear_schedule(elements, OwnershipWitnessId::new(0)),
    }
}

/// The region identifier every elementwise epilogue carries.
///
/// Distinct from the whole-program elementwise region's zero because the
/// request-subject binding matches on it: a region claiming the epilogue's
/// members must be the epilogue's region and not a whole-program one that
/// happens to carry the same expression.
const EPILOGUE_REGION: RegionId = RegionId::new(5);

/// Builds the canonical elementwise epilogue region for one recognized chain.
///
/// **Exactly one read binds the materialization edge the cover hands this
/// region**, which is the whole difference from [`pointwise_region`]: both build
/// from a recognized read list, and only this one's list can name a tensor that
/// is not a declared input.
///
/// Like [`pointwise_region`], this is the raw region and its recognized members;
/// every gate is applied when the frontier resubmits it through the ordinary
/// checked verification path.
pub(crate) fn epilogue_region(
    request: &VerifiedTargetRequest,
    chain: &NormalizedEpilogue,
    write: RegionWrite,
) -> (ScheduledRegion, Vec<SemanticStage>) {
    let reads: Vec<(TensorRole, LogicalAccess)> = chain
        .reads
        .iter()
        .map(|(read, map)| (read.tensor(), map.clone()))
        .collect();
    let region = elementwise_region(
        request,
        EPILOGUE_REGION,
        chain.shape.clone(),
        chain.elements,
        &reads,
        // An epilogue chain is `f32` by its producer's family key, for the reason
        // a fold's prologue is; see `pointwise_region`.
        RecognizedPointwise::F32(chain.expression.clone()),
        write,
    );
    (region, chain.members.clone())
}

/// The region identifier every staged family's producing stage carries.
///
/// Distinct for the reason [`EPILOGUE_REGION`] is distinct: the request-subject
/// binding matches on it, so a region claiming a staged occurrence's fold stage
/// must be that stage's region and not some other fold that happens to carry an
/// equal scalar program.
const STAGED_FOLD_REGION: RegionId = RegionId::new(7);

/// The region identifier every staged family's consuming stage carries.
const STAGED_PASS_REGION: RegionId = RegionId::new(8);

/// Builds the producing stage of one staged occurrence.
///
/// The reduction region's shape with two differences, both the law's: the fold
/// carries its own epilogue, so the scalar program is
/// [`ScalarProgram::SquaredSerialSumThenEpilogue`] rather than a bare sum; and its
/// iteration domain is the *handed* value's shape, which is the folded operand's
/// shape without the folded axes and is no occurrence boundary. Everything else is
/// the fold every reduction region states — one contributor read over the reduced
/// domain, one owning write, a serial topology carrying the contract's own
/// permissions.
///
/// Like the other builders this returns the raw region and its members; every gate
/// is applied when the frontier resubmits it through the ordinary checked path.
///
/// # Panics
///
/// Panics when the occurrence has no plan, which is invalid compiler output rather
/// than a caller error: [`spell_staged`] decides the plan exists before this
/// region is built.
pub(crate) fn staged_fold_region(
    request: &VerifiedTargetRequest,
    normalized: &NormalizedStaged,
    write: RegionWrite,
) -> (ScheduledRegion, Vec<SemanticStage>) {
    let plan = staged_plan(normalized).expect("a staged spelling is decided before it is built");
    let write_tensor = write.tensor();
    let region = ScheduledRegion {
        index: IndexRegion {
            id: STAGED_FOLD_REGION,
            iteration_shape: plan.handed_shape.clone(),
            accesses: vec![
                Access {
                    tensor: plan.contributor,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: plan.input_shape.clone(),
                        output_shape: plan.handed_shape.clone(),
                        axes: plan.axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: write_tensor,
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
                    tensor: plan.contributor,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: plan.input_shape.clone(),
                        output_shape: plan.handed_shape.clone(),
                        axes: plan.axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: write_tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: plan.handed_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: write_tensor,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: plan.handed_elements,
                },
            },
            scalar_program: ScalarProgram::SquaredSerialSumThenEpilogue {
                axes: plan.axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
                epilogue: plan.fold_epilogue.clone(),
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: plan.axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                permits_permutation: false,
            },
            ..linear_schedule(plan.handed_elements, OwnershipWitnessId::new(0))
        },
    };
    (region, vec![SemanticStage::first(normalized.member)])
}

/// Builds the consuming stage of one staged occurrence.
///
/// An ordinary elementwise region, built from the *law's* read list and
/// expression rather than from a recognized walk — the pass reads the occurrence's
/// operands and the value the producing stage handed it, and the handed value is
/// read at its kept coordinates, which is a replication whenever the fold removed
/// an axis.
///
/// # Panics
///
/// Panics when the occurrence has no plan, for the reason
/// [`staged_fold_region`] does.
pub(crate) fn staged_pass_region(
    request: &VerifiedTargetRequest,
    normalized: &NormalizedStaged,
    write: RegionWrite,
) -> (ScheduledRegion, Vec<SemanticStage>) {
    let plan = staged_plan(normalized).expect("a staged spelling is decided before it is built");
    let region = elementwise_region(
        request,
        STAGED_PASS_REGION,
        normalized.output_shape.clone(),
        normalized.output_elements,
        &plan.pass_reads,
        // A staged family's realization is `f32` throughout: the recognizer
        // admits one only from a registered law over an `f32` family, for the
        // reason `pointwise_region` states about a fold's prologue.
        RecognizedPointwise::F32(plan.pass_expression.clone()),
        write,
    );
    (
        region,
        vec![SemanticStage::first(normalized.member).next_stage()],
    )
}

/// The region identifier every publishing copy carries.
///
/// Distinct from the epilogue's and the whole-program region's, for the reason
/// [`EPILOGUE_REGION`] is distinct: the request-subject binding matches on it, so
/// a region claiming to be the copy must be the copy and not an epilogue that
/// happens to read one intermediate and write one output.
pub(crate) const PUBLISHING_COPY_REGION: RegionId = RegionId::new(6);

/// The identity expression one publishing copy evaluates per point.
///
/// A copy reads one tensor and writes what it read. Spelling it as an ordinary
/// [`ScalarProgram::PointwiseF32`] whose single node is the first input's leaf is
/// what keeps the copy inside the vocabulary the schedule verifier already
/// checks — it is not a new region family, it is the smallest member of one.
///
/// # Panics
///
/// Panics only if a one-node expression violates the pointwise builder's own
/// grammar, which is invalid compiler output rather than a caller error.
fn identity_expression() -> PointwiseF32Expression {
    let mut builder = tiler_ir::schedule::PointwiseF32ExpressionBuilder::new();
    let leaf = builder
        .input(InputOrdinal::FIRST)
        .expect("a single leaf is inside the governed expression limit");
    builder
        .build(leaf)
        .expect("a lone reachable input leaf is a valid pointwise expression")
}

/// Builds the second dispatch that publishes a value the first one staged.
///
/// **It covers no occurrence, and that is the point.** The dispatch before it
/// already claims every occurrence that computed the value; this one moves those
/// bytes into the [`tiler_ir::program::ValueRole::Output`] buffer the interface
/// publishes, because that role is exclusive of the temporary the value's other
/// consumer reads across. `tiler_ir::program`'s publishing-copy declaration is
/// what accounts for the resulting uncovering stage, exactly as a split's
/// declaration accounts for its final pass.
///
/// `shape` and `elements` are the *staged* value's, not the request's: the copy
/// iterates the domain the first dispatch wrote, and the two extents agreeing is
/// an obligation whole-program verification proves rather than one this builder
/// may assume.
///
/// Like every other constructor here this is the raw region; the frontier
/// resubmits it through the ordinary checked verification path.
pub(crate) fn publishing_copy_region(
    request: &VerifiedTargetRequest,
    shape: Shape,
    elements: u64,
) -> (ScheduledRegion, Vec<SemanticStage>) {
    let region = elementwise_region(
        request,
        PUBLISHING_COPY_REGION,
        shape,
        elements,
        &[(TensorRole::Intermediate, LogicalAccess::LinearIdentity)],
        // **`f32`, and no non-`f32` program can place one.** A publishing copy
        // exists only for a value that is both published and consumed, which
        // `crate::request`'s `published_and_consumed_overlap` admits only when one
        // output's walk is a whole *part* of another's recognized partition. A
        // whole-program elementwise partition has exactly one part — its own
        // complete member set — so a strict subset of it is no part and the
        // overlap is refused under `output-partition-overlap` before any cover is
        // formed. Every shape that does have several parts is a fold or a staged
        // family, and both are `f32` by their family keys.
        RecognizedPointwise::F32(identity_expression()),
        RegionWrite::ProgramOutput,
    );
    (region, Vec::new())
}

/// Derives one contraction operand's coordinate map from the index structure.
///
/// `declaration` is the *declared input ordinal*, which is what binds the
/// region's buffers, and the structure operand it reads is the one the
/// recognizer bound to it. Each operand axis takes the position of its index in
/// the output tuple, or — when the index is contracted rather than free — in the
/// ascending contracted set. Those are the two spaces a `direct` realization
/// walks, and the structure's own derivation guarantees every operand index is
/// in exactly one of them.
///
/// # Panics
///
/// Panics only if the normalized contraction and its own structure disagree
/// about the operand count or about which indices are free, which the recognizer
/// proved they do not.
fn contraction_operand_sources(
    normalized: &NormalizedContraction,
    declaration: usize,
) -> Vec<ContractionAxisSource> {
    let structure = &normalized.structure;
    let tuple = structure
        .operand(normalized.operand_positions[declaration])
        .expect("the recognizer bound every declared input to a structure operand");
    tuple
        .iter()
        .map(|index| {
            if let Some(position) = structure.output().iter().position(|free| free == index) {
                ContractionAxisSource::Output {
                    position: u32::try_from(position).expect("an output tuple is bounded"),
                }
            } else {
                let position = structure
                    .contracted()
                    .iter()
                    .position(|summed| summed == index)
                    .expect("an operand index is free or contracted by the structure's derivation");
                ContractionAxisSource::Contracted {
                    position: u32::try_from(position).expect("a contracted set is bounded"),
                }
            }
        })
        .collect()
}

/// Builds the canonical single-region contraction for one request.
///
/// The `direct` realization the L3 elimination retains: one invocation per
/// output element, each folding its own contracted sequence in ascending order
/// from the first product. Its only precondition is a nonempty contracted space,
/// which the recognizer already established — there is deliberately no tile or
/// split width to refuse against here, and a check that could never fire would
/// be worse than its absence.
///
/// Like [`pointwise_region`], this is the raw region and its recognized members;
/// every gate is applied when the frontier resubmits it through the ordinary
/// checked verification path.
pub(crate) fn contraction_region(
    request: &VerifiedTargetRequest,
    output: &NormalizedOutput,
    write: RegionWrite,
) -> (ScheduledRegion, Vec<SemanticStage>) {
    let normalized = output
        .contraction()
        .expect("a contraction region is built only for a contraction output");
    let write_tensor = write.tensor();
    // Two reads then the owning write, with witness numbering equal to access
    // numbering so two accesses cannot prove against one witness.
    let mut accesses = Vec::with_capacity(3);
    let mut bounds_proofs = Vec::with_capacity(3);
    for declaration in 0..normalized.input_keys.len() {
        let witness = u32::try_from(declaration).unwrap_or(u32::MAX);
        let tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(witness),
        };
        accesses.push(Access {
            tensor,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ContractionOperand {
                operand_shape: normalized.input_shapes[declaration].clone(),
                output_shape: normalized.output_shape.clone(),
                contracted_shape: normalized.contracted_shape.clone(),
                sources: contraction_operand_sources(normalized, declaration),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(witness),
            ownership: None,
        });
        bounds_proofs.push(BoundsProof {
            id: BoundsWitnessId::new(witness),
            tensor,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: normalized.input_elements[declaration],
            },
        });
    }
    let write_witness = u32::try_from(accesses.len()).unwrap_or(u32::MAX);
    accesses.push(Access {
        tensor: write_tensor,
        component_role: None,
        mode: AccessMode::Write,
        map: LogicalAccess::LinearIdentity,
        bounds: BoundsWitnessId::new(write_witness),
        ownership: Some(OwnershipWitnessId::new(0)),
    });
    bounds_proofs.push(BoundsProof {
        id: BoundsWitnessId::new(write_witness),
        tensor: write_tensor,
        component_role: None,
        kind: BoundsProofKind::LinearRange {
            element_count: normalized.output_elements,
        },
    });
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: normalized.output_shape.clone(),
            accesses,
            bounds_proofs,
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: write_tensor,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: normalized.output_elements,
                },
            },
            scalar_program: ScalarProgram::StrictTensorContraction {
                contracted_shape: normalized.contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Contraction {
                contracted_shape: normalized.contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                // Derived from the contract rather than hard-coded, exactly as
                // every other region here derives them: the schedule verifier
                // cross-checks both against the region's declared realization,
                // and a constant would lose this candidate under a contract that
                // permits either freedom — silently rather than wrongly, but for
                // a reason no diagnostic would name.
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                permits_permutation: request.numerical_contract().permutation
                    != NumericalPermission::Forbidden,
            },
            ..linear_schedule(normalized.output_elements, OwnershipWitnessId::new(0))
        },
    };
    (region, normalized.members.clone())
}

/// Builds the canonical reduction scheduled region for one request.
///
/// **It is the whole plan for a fold with no prologue, and the fold half of a
/// two-region plan otherwise**, and the only thing that differs between the two is
/// which tensor [`contributor_tensor`] resolves the contributor read to. The
/// program assembler binds that read to a declared input buffer or to the
/// materialization edge the cover placed, so a prologue-less fold assembles into
/// one dispatch over one buffer with no temporary at all.
///
/// Like [`pointwise_region`], this is the raw region and its recognized members;
/// every gate is applied when the frontier resubmits it through the ordinary
/// checked verification path.
pub(crate) fn reduction_region(
    request: &VerifiedTargetRequest,
    output: &NormalizedOutput,
    write: RegionWrite,
) -> (ScheduledRegion, Vec<SemanticStage>) {
    let serial = output.serial_sum();
    let contributor = contributor_tensor(serial);
    let write_tensor = write.tensor();
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(1),
            iteration_shape: serial.output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: contributor,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: serial.input_shape.clone(),
                        output_shape: serial.output_shape.clone(),
                        axes: serial.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(2),
                    ownership: None,
                },
                Access {
                    tensor: write_tensor,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(3),
                    ownership: Some(OwnershipWitnessId::new(1)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(2),
                    tensor: contributor,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: serial.input_shape.clone(),
                        output_shape: serial.output_shape.clone(),
                        axes: serial.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(3),
                    tensor: write_tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: serial.output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(1),
                tensor: write_tensor,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: serial.output_elements,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: serial.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: serial.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                permits_permutation: false,
            },
            ..linear_schedule(serial.output_elements, OwnershipWitnessId::new(1))
        },
    };
    (region, serial.members.reduction().to_vec())
}

/// Builds the canonical fused whole-program scheduled region for one request,
/// when its scalar program can spell the recognized prologue.
///
/// **The fusion is conditional on the vocabulary, not on the family.**
/// [`ScalarProgram::FusedMultiplyAddSerialSum`] applies one scale and one bias
/// per contributor, so this alternative exists exactly when
/// [`affine_prologue`] recovers those two constants from the recognized
/// expression. A general prologue — `sum((a * b) + c)`, or one over two declared
/// inputs — has no fused spelling in this vocabulary, and `None` therefore loses
/// *a candidate* rather than the program: the materialized two-region plan
/// realizes every recognized prologue, including this one.
///
/// Like [`pointwise_region`], this is the raw region and its recognized members;
/// every gate is applied when the frontier resubmits it through the ordinary
/// checked verification path.
pub(crate) fn fused_region(
    request: &VerifiedTargetRequest,
    output: &NormalizedOutput,
    write: RegionWrite,
) -> Option<(ScheduledRegion, Vec<SemanticStage>)> {
    let (scale_bits, bias_bits) = fused_prologue_constants(output)?;
    let serial = output.serial_sum();
    // The declared input the prologue read, which `fused_prologue_constants`
    // already required to exist: asking again binds the access to the same
    // derivation the alternative's existence was decided by.
    let contributor = fused_contributor_tensor(&serial.prologue_reads)?;
    let write_tensor = write.tensor();
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: serial.output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: contributor,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: serial.input_shape.clone(),
                        output_shape: serial.output_shape.clone(),
                        axes: serial.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: write_tensor,
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
                    tensor: contributor,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: serial.input_shape.clone(),
                        output_shape: serial.output_shape.clone(),
                        axes: serial.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: write_tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: serial.output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: write_tensor,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: serial.output_elements,
                },
            },
            scalar_program: ScalarProgram::FusedMultiplyAddSerialSum {
                scale_bits,
                bias_bits,
                axes: serial.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
                // Derived from the contract, exactly as the unfused pointwise and
                // reduction regions derive theirs. Hard-coding `false` here was
                // invisible while every registered contract forbade both
                // freedoms, and would have made this candidate fail the schedule
                // verifier's realization cross-check under one that permits them
                // — losing the fused plan silently rather than wrongly, but
                // losing it for a reason no diagnostic would have named.
                contraction: request.numerical_contract().contraction
                    != NumericalPermission::Forbidden,
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: serial.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                // Permutation stays refused: no contract this build registers
                // permits it, and `crate::policy::unrepresentable_dimension`
                // refuses one that tries, because no scheduled region can record
                // which resolution was chosen.
                permits_permutation: false,
            },
            ..linear_schedule(serial.output_elements, OwnershipWitnessId::new(0))
        },
    };
    Some((region, serial.members.all()))
}

/// Chooses the split a multi-pass reduction proposal offers for one extent.
///
/// The chosen contributors-per-partition is the divisor of `contributors`
/// nearest to its integer square root from below, which is the balanced exact
/// split: among splits that cover the sequence exactly once each, it keeps both
/// passes' per-invocation folds as short as one choice can.
///
/// **This is the multi-pass split's partition and no longer the tree's.** The
/// single-workgroup tree reads [`capped_tree_partition`], whose cap the
/// calibration below selected for it; nothing measured selects a constant for
/// this one.
///
/// Returns `None` when no exact split with at least two partitions and at least
/// two contributors per partition exists — every contributor count below four,
/// and every prime one. A partition holding a single contributor folds nothing,
/// so offering it would add a dispatch that does no work, and an inexact split
/// would leave a ragged final partition this profile does not lower.
///
/// # What the measurements say about this choice, and what they do not
///
/// **Measurement, 2026-08-07 — this split *is* measured now, it is beaten, and
/// no constant replaces it.** [The retained partition calibration] held the
/// shape fixed and swept every admissible partition over seven separated shapes
/// of the dispatch sweep's contour, 130 predeclared variants on the qualified
/// Apple9 macOS host. The balanced choice is outside the
/// indistinguishable-from-best plateau in **10 of the 14 shape-and-strategy
/// cells** — six of the seven shapes on this split — costing up to **1.413x**
/// here. So this value is refuted as the *best available* choice while remaining
/// a defensible default: nothing measured is worse than it everywhere.
///
/// **The reason no constant lands here is structural rather than statistical.**
/// A cap — take the largest admissible partition not exceeding it — is the rule
/// that replaced the tree's choice, and it fails on this split: the split's
/// optimum moves from 256 partitions at four rows to the minimum split of 2 at
/// 65,536 rows, because once the row count alone saturates the device extra
/// partitions add total work and stage more partials without buying
/// parallelism. Leave-one-out therefore selects a cap on six shapes that costs
/// **2.131x** on the seventh — worse than the choice it would replace — and
/// which cap it selects is not even stable across two runs of the same matrix.
/// Improving this partition means *reading the row count against a saturation
/// threshold*, which is the same machine quantity the strategy contour turns on
/// and is owned by
/// `activate-measured-reduction-selection-from-a-target-cost-row`. The split's
/// partition is not separable from its strategy selection.
///
/// **Measurement, 2026-08-07 — the crossover between the three *strategies* is
/// measured, and it said nothing about this split.** [The retained dispatch
/// sweep] timed all three alternatives over 92 shapes on the same host and
/// found a large contour: parallel plans win by up to 50.7 times where the row
/// count cannot saturate the device, and lose by up to 1.78 times where it can.
/// Every cell of it used whatever partition this function returned, so **that
/// sweep varied the shape and never the split**; it is the calibration above,
/// not this one, that bears on the value here.
///
/// [The compile-phase domain sweep] is the other half of that record: the
/// profile's grid-axis row is a measured 268,435,456 rather than the
/// conservative four it once was, so the inequality `4 <= contributors <=
/// rows * contributors <= grid_axis_bound` — this function's floor against the
/// prologue's one-invocation-per-element launch — admits a wide domain instead
/// of the single shape `(1, 4)` it closed on at a bound of four.
/// `tiler_build::metal_plan::tests::the_measured_grid_axis_admits_more_than_one_three_strategy_shape`
/// reports the domain, in `tiler-build` because that is the crate that can see
/// the profile calibration measures against.
///
/// **This function's own four is unrelated to that row and does not move with
/// it.** Four is the smallest contributor count admitting two partitions of at
/// least two, which is a property of splitting rather than of any target.
///
/// [The retained partition calibration]:
///     ../../../spikes/program-planning/reduction-partition-calibration/README.md
/// [The retained dispatch sweep]:
///     ../../../spikes/program-planning/reduction-dispatch-crossover/README.md
/// [The compile-phase domain sweep]:
///     ../../../spikes/program-planning/reduction-crossover/README.md
pub(crate) fn governed_partition(contributors: u64) -> Option<ContributorPartition> {
    if contributors < 4 {
        return None;
    }
    let mut candidate = contributors.isqrt();
    while candidate >= 2 {
        if contributors.is_multiple_of(candidate) {
            let partitions = contributors / candidate;
            if partitions >= 2 {
                return Some(ContributorPartition {
                    partitions,
                    contributors_per_partition: candidate,
                });
            }
        }
        candidate -= 1;
    }
    None
}

/// The participant count the single-workgroup tree's width is anchored on.
///
/// **It is an anchor and not a truncation**, which is the distinction
/// [`capped_tree_partition`] turns on: that rule takes the admissible count
/// *nearest* this value, so a width at or below it is the ordinary case and a
/// width up to 509 is reachable where the divisor lattice offers nothing closer
/// from below. The name is kept because every measured cell exercised this value
/// as a ceiling; the direction it does *not* bound is stated on the rule.
///
/// **Measurement, 2026-08-07, and a property of one host row rather than a
/// portable constant.** [The retained partition calibration] swept every
/// admissible partition of seven separated shapes on the qualified Apple9 macOS
/// host — one profile (`tiler.metal.macos-apple9.msl4-0.f32.v1`), one contract,
/// one program family, `f32` only, and no contributor count that is not a power
/// of two. Capping the tree's participants here is selected by leave-one-out on
/// **all seven folds**, with a held-out worst regret of **1.008** against the
/// balanced choice's **1.216**, and both 64-encode runs of the matrix agree. It
/// is not established for another Apple family, OS row, dtype, or device, and a
/// second target profile should carry its own row rather than inherit this one.
///
/// It is also comfortably inside the workgroup width the calibration ran into:
/// that sweep's eight declined variants are all at 2,048 participants and above,
/// where the prepared entry admits 1,024 threads per workgroup and
/// [`tiler_ir::schedule::workgroup_tree_tile`] has no representation past
/// [`tiler_ir::schedule::MAX_COOPERATIVE_PARTICIPANTS`]. Both bounds remain the
/// authorities that refuse a width; this constant only expresses a preference
/// among widths they admit.
///
/// [The retained partition calibration]:
///     ../../../spikes/program-planning/reduction-partition-calibration/README.md
pub(crate) const MEASURED_TREE_PARTICIPANT_CAP: u64 = 256;

/// Chooses the participant count the single-workgroup tree runs for one extent.
///
/// The rule is **the admissible participant count nearest
/// [`MEASURED_TREE_PARTICIPANT_CAP`]**, ties going to the narrower, where a count
/// is admissible under exactly the condition [`governed_partition`] searches
/// within — it splits the contributor sequence exactly, into at least two
/// partitions of at least two contributors each.
///
/// **That is one rule where the calibration's scoring stated two, and the
/// omission between them was the defect.** "The largest admissible count not
/// exceeding the cap" is *nearest from below*; "when every admissible count
/// exceeds the cap, the smallest admissible one" is *nearest from above*. Both
/// are cases of the sentence above, and both were already here. What neither did
/// was compare across the two: at 514 contributors (`2 * 257`) the only
/// admissible count at or below the cap is **2**, while **257** is admissible one
/// step above it, and a rule that truncates at the cap without looking up took
/// the 2. On a power of two the two formulations coincide, so no measured cell
/// could distinguish them.
///
/// The domain is unchanged and is exactly [`governed_partition`]'s: this returns
/// `None` for every contributor count below four and every prime one, and `Some`
/// for every count either rule admits. The tree's
/// [`WorkgroupTreeUnavailable::NoAdmissibleParticipantCount`] decline set does
/// not move with the rule.
///
/// # The direction, stated in both senses
///
/// [`MEASURED_TREE_PARTICIPANT_CAP`] bounds the width from above. This rule also
/// bounds it from below, and that bound is arithmetic rather than fitted: a
/// chosen count `s` above the cap is chosen over an admissible `l` at or below it
/// only when `s - 256 < 256 - l`, and `l >= 2` forces **`s <= 509`**. So a cost
/// preference widens the tree past the calibrated 256 by at most 253
/// participants, and never at all in the fallback branch, whose width is exactly
/// what it was — the smallest prime factor of `contributors`, at most the integer
/// square root, and so at most [`governed_partition`]'s own count.
///
/// **That ceiling is what keeps the rule a preference rather than a feasibility
/// decision.** 509 participants stage 2,036 `f32` bytes and sit inside both
/// authorities that refuse a width: [`tiler_ir::schedule::MAX_COOPERATIVE_PARTICIPANTS`]
/// is 4,096, and the widest workgroup any profile in this repository declares is
/// the qualified Apple9 entry's 1,024. No contributor count offered a tree before
/// this rule loses one because of it.
///
/// **Taking the wider of this rule and [`governed_partition`] would not have that
/// property, and that is why it is not what happens.** At 8,198 contributors
/// (`2 * 4,099`) the balanced count is 4,099, which
/// [`tiler_ir::schedule::workgroup_tree_tile`] cannot represent, so
/// [`single_workgroup_tree_region`] would report
/// [`WorkgroupTreeUnavailable::Unrepresentable`] where it offers a two-participant
/// tree today; at `2 * 65,537` the balanced count is 65,537. Exhausting the range
/// below 20,000 finds **1,065** counts where the wider-of-the-two rule exceeds
/// 4,096 and this one does not. A width preference that withdraws a legal
/// alternative has decided feasibility, which is the separation
/// [`WorkgroupTreeUnavailable`] exists to keep.
///
/// # What the evidence supports, by direction — they are not the same rung
///
/// **Upward: empirical evidence, one host.** Seven shapes, one profile, one
/// contract, one program family, `f32`, bounded on
/// [`MEASURED_TREE_PARTICIPANT_CAP`] rather than restated here.
///
/// **Downward: two claims, deliberately not sharing a sentence.** That the
/// chosen count is never more than 253 short of an admissible count within reach
/// of the cap is *arithmetic*, and the population it moves is exhaustive finite
/// evidence over a named range —
/// `pipeline::tests::the_tree_widens_toward_the_cap_rather_than_truncating_at_it`
/// enumerates every count below 4,096, reports 3,530 admitting ones, and pins the
/// 1,061 this rule widens. That a count nearer the cap is *cheaper* is `Unknown`
/// at every count the rule moves: **no measured shape has a contributor count
/// that is not a power of two**, and on a power of two the largest admissible
/// count at or below the cap is already the widest the cap admits, so no measured
/// cell exercised this at all. The direction is inferred from the calibration's
/// steepest measured span — the tree at four rows of 8,192 costs 9.53 µs at 256
/// participants against 48.15 µs at two, 5.05x — and an inference is what it
/// stays until a non-power-of-two count is measured.
///
/// **The rule does not chase that direction to the end, and the residue is
/// named rather than left to be rediscovered.** Below 20,000 contributors, 1,133
/// counts still take two participants, against 1,176 before. The smallest is
/// 1,042 (`2 * 521`), where 521 is admissible, representable, and inside the
/// qualified entry's workgroup width, and the rule still declines it because 521
/// is 265 above the cap while 2 is 254 below. Nothing measured says which of
/// those costs less, so `measure-the-tree-width-excursion-past-the-cap` carries
/// the excursion width as an open measurement rather than a threshold fitted to
/// no data.
///
/// **It does move one feasibility question, and deliberately does not answer
/// it.** The rule chooses *within* what a target admits and decides nothing
/// about legality, but the tree stages one `f32` slot per participant, so a
/// wider choice asks a profile for more workgroup memory: at 8,192 contributors
/// the capped 256 participants need 1,024 bytes where the balanced 128 needed
/// 512. A profile whose `local-memory-bytes` row falls between the two therefore
/// refuses a tree it would have admitted under the balanced choice. That refusal
/// belongs to the feasibility authority, typed and named on its axis, and
/// narrowing the width *because* a target is small is not this function's to do:
/// it would let a cost preference decide legality, which is the separation
/// [`WorkgroupTreeUnavailable`] exists to keep. The row the calibration measured
/// against declares 32,768 bytes — 8,192 participants' worth — so even the widest
/// width this rule can reach is far inside it, and no profile in this repository
/// sits in the affected band; the prototype baseline declares zero and refuses
/// every tree at every width.
pub(crate) fn capped_tree_partition(contributors: u64) -> Option<ContributorPartition> {
    if contributors < 4 {
        return None;
    }
    let partition = |participants: u64| ContributorPartition {
        partitions: participants,
        contributors_per_partition: contributors / participants,
    };
    // At least two contributors per partition, so the widest count worth
    // considering is half the sequence even when the cap is above it.
    let ceiling = MEASURED_TREE_PARTICIPANT_CAP.min(contributors / 2);
    let mut below = None;
    let mut candidate = ceiling;
    while candidate >= 2 {
        if contributors.is_multiple_of(candidate) {
            below = Some(candidate);
            break;
        }
        candidate -= 1;
    }
    let Some(below) = below else {
        // Nothing at or below the cap divides the sequence. The smallest
        // admissible count is then the smallest divisor above it, and that
        // divisor is bounded by the integer square root: a count with no divisor
        // there is prime and admits no split at all. Searching past it would walk
        // half the sequence for every prime extent.
        let limit = contributors.isqrt();
        let mut candidate = ceiling + 1;
        while candidate <= limit {
            if contributors.is_multiple_of(candidate) {
                return Some(partition(candidate));
            }
            candidate += 1;
        }
        return None;
    };
    // A count above the cap is nearer to it than `below` exactly when it is
    // under `2 * cap - below`, so that expression — never above 510 — is the
    // whole search range, and it collapses to nothing when `below` is the cap
    // itself. The `contributors / 2` guard is the same two-contributors-per
    // -partition floor as the ceiling above, and is what makes the range empty
    // for every sequence short enough that no count past the cap could split it.
    let nearer_than_below = 2 * MEASURED_TREE_PARTICIPANT_CAP - below;
    let mut candidate = MEASURED_TREE_PARTICIPANT_CAP + 1;
    while candidate < nearer_than_below && candidate <= contributors / 2 {
        if contributors.is_multiple_of(candidate) {
            return Some(partition(candidate));
        }
        candidate += 1;
    }
    Some(partition(below))
}

/// Builds the canonical partial pass of a split reduction for one request.
///
/// It splits the *materialized* strategy's reduction rather than the fused one:
/// it reads whichever tensor holds the fold's declared contributor domain and
/// writes the partial tensor, so the split replaces one dispatch with two and
/// leaves the prologue, if there is one, where it was. Fusing the prologue into
/// this pass would additionally have to reconcile the contraction permission the
/// fused scalar program carries, which is a different question from splitting a
/// contributor sequence.
///
/// A prologue-less fold is split by the same two passes, and only the partial
/// pass's read moves: the final pass folds partials this pass staged, so it reads
/// an intermediate whatever the contributor domain is. The strategy is therefore
/// offered for `sum(x)` rather than declined for it, which is what keeps the
/// widening from silently losing an alternative.
pub(crate) fn partial_reduction_region(
    request: &VerifiedTargetRequest,
    output: &NormalizedOutput,
    partition: ContributorPartition,
) -> Option<(ScheduledRegion, Vec<SemanticStage>)> {
    let subject = output.serial_sum();
    let contributor = contributor_tensor(subject);
    let partial_shape =
        tiler_ir::schedule::partial_reduction_shape(&subject.output_shape, partition)?;
    let partial_elements = subject.output_elements.checked_mul(partition.partitions)?;
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(2),
            iteration_shape: partial_shape,
            accesses: vec![
                Access {
                    tensor: contributor,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: subject.input_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: subject.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(4),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(5),
                    ownership: Some(OwnershipWitnessId::new(2)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(4),
                    tensor: contributor,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: subject.input_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: subject.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(5),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: partial_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(2),
                tensor: TensorRole::Intermediate,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: partial_elements,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: subject.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: multi_pass_topology(
                request,
                tiler_ir::schedule::ReductionPass::Partial,
                partition,
                subject.reduction_axes.clone(),
            ),
            ..linear_schedule(partial_elements, OwnershipWitnessId::new(2))
        },
    };
    Some((region, subject.members.reduction().to_vec()))
}

/// Builds the canonical final pass of a split reduction for one request.
///
/// It reduces the single partition axis of the staged partial tensor, so its
/// axes are deliberately not the request's reduction axes: those were already
/// consumed by the partial pass.
///
/// **It claims the reduction occurrence's *second* stage, and that is the fact
/// the attribution atom exists to carry.** The two passes realize one recognized
/// fold between them; the partial pass claims stage zero and this one claims the
/// stage after it, so "which part of the occurrence does this dispatch compute"
/// is stated rather than left to the shape of the chain. While the atom was a
/// bare occurrence the only expressible answer was the empty set, which says the
/// pass computes nothing at all — true of a publishing copy, false of a combine,
/// and indistinguishable from a provider that claimed nothing by mistake.
pub(crate) fn final_reduction_region(
    request: &VerifiedTargetRequest,
    output: &NormalizedOutput,
    partition: ContributorPartition,
    write: RegionWrite,
) -> Option<(ScheduledRegion, Vec<SemanticStage>)> {
    let subject = output.serial_sum();
    let write_tensor = write.tensor();
    let partial_shape =
        tiler_ir::schedule::partial_reduction_shape(&subject.output_shape, partition)?;
    let axes = vec![tiler_ir::schedule::partial_reduction_axis(
        &subject.output_shape,
    )?];
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(3),
            iteration_shape: subject.output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: partial_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(6),
                    ownership: None,
                },
                Access {
                    tensor: write_tensor,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(7),
                    ownership: Some(OwnershipWitnessId::new(3)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(6),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: partial_shape,
                        output_shape: subject.output_shape.clone(),
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(7),
                    tensor: write_tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: subject.output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(3),
                tensor: write_tensor,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: subject.output_elements,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: multi_pass_topology(
                request,
                tiler_ir::schedule::ReductionPass::Final,
                partition,
                axes,
            ),
            ..linear_schedule(subject.output_elements, OwnershipWitnessId::new(3))
        },
    };
    let combined = subject
        .members
        .reduction()
        .iter()
        .map(|atom| atom.next_stage())
        .collect();
    Some((region, combined))
}

/// The stable name of the single-workgroup tree strategy in explain output.
pub(crate) const SINGLE_WORKGROUP_TREE_STRATEGY: &str = "tiler.reduction.single-workgroup-tree";

/// Why the governed profile offers no single-workgroup tree of one request's
/// reduction.
///
/// A decline is a fact about *this request*, decided before any region exists,
/// exactly as [`SplitUnavailable`] is. Every reason a *target* cannot run the
/// strategy is deliberately absent from this vocabulary: workgroup memory,
/// workgroup width, and the synchronization realization are resolved by the
/// feasibility authority against the profile, so putting any of them here would
/// let a preference decide legality and would hide the exact refusing bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkgroupTreeUnavailable {
    /// The resolved numerical contract forbids reassociation.
    ///
    /// The tree regroups the declared contributor sequence, so this is the
    /// permission it consumes. It is the *only* numerical decline here: the
    /// admitted arrival order is fixed by the program, so the strategy consumes
    /// no contributor permutation and a contract withholding permutation
    /// forbids nothing this strategy does.
    ReassociationForbidden,
    /// No exact split of the contributor sequence across participants exists.
    ///
    /// Carries the contributor count, because "which extent admitted none" is
    /// what a reader needs. The tail policy is exact by construction — a ragged
    /// participant would stage a slot the coverage rule requires a writer for,
    /// and a masked lane would break the emitted body's soundness argument — so
    /// an inexact split is declined rather than padded.
    NoAdmissibleParticipantCount {
        /// Contributors one output position folds under this request.
        contributors: u64,
    },
    /// The tree's derived extents, shapes, or tile are not representable.
    Unrepresentable,
}

impl WorkgroupTreeUnavailable {
    /// Returns the stable reason code naming this decline.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::ReassociationForbidden => "reassociation-forbidden",
            Self::NoAdmissibleParticipantCount { .. } => "no-admissible-participant-count",
            Self::Unrepresentable => "workgroup-tree-unrepresentable",
        }
    }
}

/// Builds the single-workgroup tree reduction of one request, or states why not.
///
/// # The strategy, with every key stated
///
/// One workgroup per output position and `participants` invocations in it.
/// Level 0: every participant serially folds the contiguous contributor range
/// its partition owns and stages the partial in its own slot of workgroup
/// memory. The synchronization point. Level 1: the one committing participant
/// folds the `participants` staged slots in ascending order and performs the
/// region's owning write.
///
/// | Key | Value | Where it is stated |
/// | --- | --- | --- |
/// | topology | depth-two tree, fan-in `contributors_per_partition` then `participants` | [`ContributorPartition`] and [`tiler_ir::schedule::workgroup_tree_tile`] |
/// | active lanes | every participant, then the committing one | the tile's phase participation and its `commit` range |
/// | tail | exact, or declined | [`ContributorPartition::covers`], enforced by the schedule verifier |
/// | workgroup storage | one `f32` slot per participant | the tile's [`tiler_ir::schedule::WorkgroupStaging`] |
/// | accumulation dtype | the resolved contract's arithmetic type | the topology's `accumulation` |
/// | contributor order | original-axis lexicographic within a partition, ascending participant across them | the topology's `order` and `arrival` |
///
/// The participant count is [`capped_tree_partition`]'s: the admissible count
/// nearest [`MEASURED_TREE_PARTICIPANT_CAP`]. It is deliberately no longer the
/// balanced exact split [`governed_partition`] returns, and the two now declare
/// *different* groupings wherever their choices differ — 8,192 contributors give
/// the tree 256 participants folding 32 each where the split takes 128 of 64.
///
/// **Only the value 256 is calibrated; the rule around it is not, in one
/// direction.** Every measured shape has a power-of-two contributor count, and
/// there "nearest the cap" and "largest not exceeding the cap" are the same
/// choice. Which of the two the rule states is therefore an *inference* from the
/// measured direction rather than a measured result, and
/// [`capped_tree_partition`] carries that separation, the arithmetic bound it
/// puts on the downward direction, and the population still outside it.
///
/// **Measurement, 2026-08-07 — the cap is what the sweep over partitions
/// selected for this strategy.** [The retained partition calibration] held the
/// shape fixed and swept every admissible partition over seven separated shapes
/// on the qualified Apple9 macOS host. The balanced choice is beaten on four of
/// the seven for this strategy, worst **1.216x**; the cap is selected by
/// leave-one-out on all seven folds with held-out worst regret **1.008**, and one
/// shape's indistinguishable-from-best plateau is `{256}` alone. The bound on
/// that claim — one profile, one contract, one program family, `f32`, powers of
/// two — is carried on the constant rather than restated here.
///
/// **The split keeps [`governed_partition`], and not for want of measuring.**
/// The same sweep refuted the balanced choice for the split too, but no constant
/// replaces it: the split's optimum moves from 256 partitions at four rows to the
/// minimum split of 2 at 65,536 rows, so a cap fitted on six of its shapes costs
/// 2.131x on the seventh. That partition is not separable from the saturation
/// quantity the strategy contour turns on, and
/// `activate-measured-reduction-selection-from-a-target-cost-row` has now landed
/// that quantity as a declared cost row — **which does not close this, and the
/// distinction is the reason the ticket above stays open.** The row is consulted
/// to choose *between* strategies, at whatever width each one already declares;
/// choosing a width *within* a strategy from the same saturation quantity is a
/// separate consumer of the same number and belongs to
/// `calibrate-device-cost-models`.
///
/// **Measurement, 2026-08-07 — the tree is known to be worth having, and
/// *selection* acts on that now.** [The retained dispatch sweep] timed all three
/// strategies over 92 shapes on the same host: the serial fold is up to 50.7
/// times slower than the best parallel plan where the row count cannot saturate
/// the device, and up to 1.78 times faster where it can. The qualified profile
/// declares the machine quantity that contour turns on as a measured cost row,
/// and [`crate::measured_cost`] consults it — the same ticket above. **This
/// function still calibrates the tree's width and not its odds of being picked**,
/// and the separation matters: the width is chosen among what a target admits,
/// and the preference is chosen among complete plans a target already admitted.
///
/// [The retained partition calibration]:
///     ../../../spikes/program-planning/reduction-partition-calibration/README.md
/// [The retained dispatch sweep]:
///     ../../../spikes/program-planning/reduction-dispatch-crossover/README.md
///
/// # Errors
///
/// Returns the typed [`WorkgroupTreeUnavailable`] the frontier records as a
/// declined strategy. None of them is a compiler fault, and none of them is a
/// target decision.
pub(crate) fn single_workgroup_tree_region(
    request: &VerifiedTargetRequest,
    output: &NormalizedOutput,
    write: RegionWrite,
) -> Result<(ScheduledRegion, Vec<SemanticStage>), WorkgroupTreeUnavailable> {
    if request.numerical_contract().reassociation == NumericalPermission::Forbidden {
        return Err(WorkgroupTreeUnavailable::ReassociationForbidden);
    }
    let subject = output.serial_sum();
    // **Threaded like its siblings, and not currently observable through
    // `compile`.** A cooperative tile committing into a materialized
    // intermediate is what an epilogue over a tree-reduced fold would need, and
    // the tree is offered for a chain's fold exactly as it is for a standalone
    // one. No retained plan places it: for every shape this profile admits, the
    // portfolio's structural cost prunes the tree alternative before it reaches
    // assembly, so hard-coding `TensorRole::Output` here fails no test today. It
    // is threaded anyway because the alternative is *offered* — a region built
    // for a write the cover did not assign is refused at assembly and the
    // alternative disappears silently, which is the failure mode that has no
    // diagnostic. **Selection can prefer the tree now**, under
    // `activate-measured-reduction-selection-from-a-target-cost-row`, so the
    // observability this note describes arrives with the first *epilogue* chain
    // whose fold the measured row prefers a tree for: preference is what was
    // missing, and a cover assigning the tree a materialized write is what is
    // still needed to reach the hard-coded role.
    let write_tensor = write.tensor();
    let contributor = contributor_tensor(subject);
    let contributors =
        reduction_contributors(output).ok_or(WorkgroupTreeUnavailable::Unrepresentable)?;
    let partition = capped_tree_partition(contributors)
        .ok_or(WorkgroupTreeUnavailable::NoAdmissibleParticipantCount { contributors })?;
    let participants = partition.partitions;
    let tile = tiler_ir::schedule::workgroup_tree_tile(participants)
        .ok_or(WorkgroupTreeUnavailable::Unrepresentable)?;
    let iteration_shape =
        tiler_ir::schedule::partial_reduction_shape(&subject.output_shape, partition)
            .ok_or(WorkgroupTreeUnavailable::Unrepresentable)?;
    let work_items = subject
        .output_elements
        .checked_mul(participants)
        .ok_or(WorkgroupTreeUnavailable::Unrepresentable)?;
    let threads_per_workgroup =
        u32::try_from(participants).map_err(|_| WorkgroupTreeUnavailable::Unrepresentable)?;
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(4),
            iteration_shape,
            accesses: vec![
                Access {
                    tensor: contributor,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: subject.input_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: subject.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(8),
                    ownership: None,
                },
                Access {
                    tensor: write_tensor,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(9),
                    ownership: Some(OwnershipWitnessId::new(4)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(8),
                    tensor: contributor,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: subject.input_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: subject.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                // The owned output positions, which is one per *workgroup* and
                // not one per invocation: the tile runs `participants`
                // invocations over each of them.
                BoundsProof {
                    id: BoundsWitnessId::new(9),
                    tensor: write_tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: subject.output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(4),
                tensor: write_tensor,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: subject.output_elements,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: subject.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            threads_per_workgroup,
            reduction: ReductionTopology::CooperativeWorkgroup {
                partition,
                tile,
                axes: subject.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: request.numerical_contract().arithmetic,
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                // Reported as the contract resolves it and deliberately not
                // consulted to admit the strategy: the arrival below is fixed by
                // the program, so a build that later registers a permuting
                // contract does not start admitting trees for the wrong reason.
                permits_permutation: request.numerical_contract().permutation
                    != NumericalPermission::Forbidden,
                arrival: tiler_ir::schedule::ContributorArrival::AscendingParticipant,
            },
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup,
                zero_work_skips_dispatch: true,
            },
            ..linear_schedule(work_items, OwnershipWitnessId::new(4))
        },
    };
    Ok((region, subject.members.reduction().to_vec()))
}

/// Why the governed profile offers no multi-pass split of one request's
/// reduction.
///
/// A decline is a *fact about this request*, not a cost and not a compiler
/// fault: the strategy applies to the subject, and the reason it was not offered
/// is what a reader needs in order to know the serial alternative stands alone
/// deliberately. Each variant is therefore carried to the frontier and recorded,
/// rather than expressed as the absence of a proposal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SplitUnavailable {
    /// The resolved numerical contract forbids reassociation.
    ///
    /// A split *is* a reassociation of the declared contributor sequence, so
    /// this is the one permission it consumes. Proposing anyway would emit a
    /// region the schedule verifier rejects as malformed compiler output, which
    /// would report a caller's numerical choice as a Tiler defect.
    ReassociationForbidden,
    /// No exact split of the contributor sequence exists.
    ///
    /// Carries the exact contributor count so a reader can see *which* extent
    /// admitted no balanced split, rather than only that one did not.
    NoAdmissiblePartition {
        /// Contributors one output position folds under this request.
        contributors: u64,
    },
    /// The split's derived extents or shapes are not representable.
    Unrepresentable,
}

impl SplitUnavailable {
    /// Returns the stable reason code naming this decline.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::ReassociationForbidden => "reassociation-forbidden",
            Self::NoAdmissiblePartition { .. } => "no-admissible-partition",
            Self::Unrepresentable => "split-extent-unrepresentable",
        }
    }
}

/// The stable name of the multi-pass split strategy in explain output.
pub(crate) const MULTI_PASS_SPLIT_STRATEGY: &str = "tiler.reduction.multi-pass-split";

/// The ordered raw stages of one governed multi-pass split.
///
/// The stages are raw and not yet verified, exactly like every other constructor
/// in this module: the frontier resubmits each through the ordinary checked path
/// before any of them is admitted.
pub(crate) struct GovernedSplit {
    /// How the contributor sequence is split, retained for cost and identity.
    pub(crate) partition: ContributorPartition,
    /// The partial pass, then the final pass, each with its claimed members.
    pub(crate) stages: Vec<(ScheduledRegion, Vec<SemanticStage>)>,
}

/// Chooses and builds the governed multi-pass split of one request's reduction.
///
/// This is the single authority deciding whether a split is offered at all. It
/// runs *before* any region is constructed for the two permissions that decide
/// the question — the contract's reassociation resolution and the existence of
/// an exact partition — because both are properties of the request rather than
/// of a schedule, and a region built for a request that admits neither is a
/// region the verifier would have to reject.
///
/// # Errors
///
/// Returns the typed [`SplitUnavailable`] the frontier records as a declined
/// strategy. None of them is a compiler fault.
pub(crate) fn split_reduction_regions(
    request: &VerifiedTargetRequest,
    output: &NormalizedOutput,
    write: RegionWrite,
) -> Result<GovernedSplit, SplitUnavailable> {
    if request.numerical_contract().reassociation == NumericalPermission::Forbidden {
        return Err(SplitUnavailable::ReassociationForbidden);
    }
    let contributors = reduction_contributors(output).ok_or(SplitUnavailable::Unrepresentable)?;
    let partition = governed_partition(contributors)
        .ok_or(SplitUnavailable::NoAdmissiblePartition { contributors })?;
    let partial = partial_reduction_region(request, output, partition)
        .ok_or(SplitUnavailable::Unrepresentable)?;
    let combine = final_reduction_region(request, output, partition, write)
        .ok_or(SplitUnavailable::Unrepresentable)?;
    Ok(GovernedSplit {
        partition,
        stages: vec![partial, combine],
    })
}

/// Counts the contributors one output position of a request's reduction folds.
///
/// Derived from the reduced axes' extents rather than from
/// `input_elements / output_elements`, because that division is undefined for
/// an empty kept domain and silently wrong for an empty reduced one — both of
/// which are shapes the request boundary admits.
///
/// Returns `None` only when an axis is out of range or the product overflows,
/// neither of which a verified request can produce; the fail-closed answer is
/// still stated rather than assumed.
fn reduction_contributors(output: &NormalizedOutput) -> Option<u64> {
    let subject = output.serial_sum();
    subject
        .reduction_axes
        .iter()
        .try_fold(1_u64, |total, axis| {
            let position = usize::try_from(axis.get()).ok()?;
            let extent = subject.input_shape.extents().get(position)?;
            total.checked_mul(extent.get())
        })
}

/// Reads back the split contract one verified partial pass declares.
///
/// The program assembler needs the partition to declare its
/// [`tiler_ir::program::PartialReduction`], and reading it from the region the
/// pass actually carries — rather than re-deriving it from the request — is what
/// makes the program-scope declaration agree with the schedule that produced it
/// by construction instead of by a second derivation.
pub(crate) fn declared_partial_partition(region: &ScheduledRegion) -> Option<ContributorPartition> {
    match &region.schedule.reduction {
        ReductionTopology::MultiPass {
            pass: tiler_ir::schedule::ReductionPass::Partial,
            partition,
            ..
        } => Some(*partition),
        _ => None,
    }
}

/// Builds the reduction topology one pass of a split declares.
///
/// Both permissions are read from the resolved contract and carried
/// independently. Permutation is reported as the contract resolves it rather
/// than hardcoded, because the split neither needs nor consumes it: the schedule
/// verifier admits the topology on reassociation alone, so a build that later
/// registers a permuting contract does not silently start admitting splits for
/// the wrong reason.
fn multi_pass_topology(
    request: &VerifiedTargetRequest,
    pass: tiler_ir::schedule::ReductionPass,
    partition: ContributorPartition,
    axes: Vec<tiler_ir::shape::Axis>,
) -> ReductionTopology {
    ReductionTopology::MultiPass {
        pass,
        partition,
        axes,
        order: ContributorOrder::OriginalAxisLexicographic,
        accumulation: request.numerical_contract().arithmetic,
        permits_reassociation: request.numerical_contract().reassociation
            != NumericalPermission::Forbidden,
        permits_permutation: request.numerical_contract().permutation
            != NumericalPermission::Forbidden,
    }
}

fn linear_schedule(work_items: u64, owner: OwnershipWitnessId) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: owner,
        reduction: ReductionTopology::None,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

/// Verifies one scheduled region and binds it to a compilation request.
///
/// Intrinsic schedule verification runs first, in `tiler_ir::schedule`, and
/// proves domain coverage, ownership, race freedom, tail/launch legality,
/// bounds-proof refinement, reduction contributor/order legality, and
/// zero-domain behaviour before any feasibility query. Only then does the
/// compiler layer its request-subject binding and the single hard-feasibility
/// decision. No cost or provider callback participates.
#[allow(
    dead_code,
    reason = "the predicate-free spelling of the checked verification path; the frontier consumes the predicate-carrying form"
)]
pub(crate) fn verify_schedule(
    region: ScheduledRegion,
    semantic_members: Vec<SemanticStage>,
    request: &VerifiedTargetRequest,
) -> Result<VerifiedScheduledRegion, PhysicalError> {
    verify_schedule_with_feasibility(region, semantic_members, request)
}

/// Verifies one scheduled region and additionally surfaces the resolved
/// feasibility evidence that an admissible target assessment carries.
///
/// This runs the exact checked path [`verify_schedule`] runs — the request-subject
/// precondition, whole-region intrinsic verification, numerical-realization
/// agreement, the request-subject binding, and the single hard-feasibility
/// decision — and additionally returns either complete proof or compiler-minted
/// deferred obligations.
/// The physical implementation frontier retains it as admission evidence for an
/// enumerated proposal. A provider cannot bypass any of these checks: a
/// [`PhysicalError::Target`] or [`PhysicalError::Numerical`] means the proposal
/// is hard-infeasible (never a cost), and any other [`PhysicalError`] means the
/// provider emitted invalid IR.
pub(crate) fn verify_schedule_with_feasibility(
    region: ScheduledRegion,
    semantic_members: Vec<SemanticStage>,
    request: &VerifiedTargetRequest,
) -> Result<VerifiedScheduledRegion, PhysicalError> {
    let id = region.index.id;
    let subject = request.subject();
    if !request.reconstructs_its_authority() || !request.numerical_contract().is_governed() {
        return intrinsic("request-subject", id);
    }
    let verified = ScheduledRegionBuilder::from_region(region)
        .build()
        .map_err(|error| map_schedule_build_error(&error, id))?;
    if verified.region().index.numerical != request.numerical_contract().realization() {
        return intrinsic("numerical-realization", id);
    }
    verify_region_subject_binding(verified.region(), &semantic_members, subject)?;
    let evidence = assess_region(
        id,
        verified.requirements(),
        // The region implements this request's resolved contract — checked one
        // line above by comparing the region's realization against it — so its
        // arithmetic type is the contract's, not a value re-derived here.
        request.numerical_contract().arithmetic,
        verified.region().schedule.work_items,
        request.target_profile(),
    )?;
    Ok(VerifiedScheduledRegion {
        verified,
        semantic_members,
        target_profile: request.target_profile().clone(),
        request_subject: subject.clone(),
        admission: evidence,
    })
}

/// Maps an intrinsic schedule-verification failure onto the physical-error
/// contract.
///
/// A domain-product overflow keeps its distinct shape-overflow class; every
/// other intrinsic diagnostic carries its stable rule identifier so the explain
/// trace attributes the exact rejected rule.
fn map_schedule_build_error(error: &ScheduledRegionBuildError, region: RegionId) -> PhysicalError {
    match error.diagnostics().first() {
        Some(ScheduledRegionDiagnostic::ShapeProductOverflow) => {
            PhysicalError::ShapeProductOverflow { region }
        }
        Some(diagnostic) => PhysicalError::Intrinsic {
            rule: diagnostic.rule(),
            region,
        },
        None => PhysicalError::Intrinsic {
            rule: "schedule-verification",
            region,
        },
    }
}

/// Binds one region to the recognized output whose partition it belongs to.
///
/// **The subject is a list, so the binding is a search with no fallback.** A
/// region binds when it binds against *some* declared output's recognized
/// partition; a region that binds against none is rejected, and the reported
/// cause is the refusal of the first declared output. The request boundary
/// admits ordered multi-output programs, so a region that binds against none has
/// one refusal per declared output and the search reports the first in
/// declaration order rather than synthesizing a combined cause. Nothing here
/// weakens what a binding proves: each
/// candidate is checked whole, so admitting a region still requires it to be the
/// exact realization of one recognized partition rather than a plausible one.
fn verify_region_subject_binding(
    region: &ScheduledRegion,
    semantic_members: &[SemanticStage],
    subject: &VerifiedRequestSubject,
) -> Result<(), PhysicalError> {
    // A publishing copy claims no occurrence, so no recognized partition's
    // member comparison can admit it and every arm below would answer `false`.
    // It binds against the *interface* instead, which is what it realizes: it
    // iterates one declared output's published domain and copies into it.
    if region.index.id == PUBLISHING_COPY_REGION {
        return verify_publishing_copy_binding(region, semantic_members, subject);
    }
    let mut first = None;
    for normalized in subject.normalized().outputs() {
        match verify_region_output_binding(region, semantic_members, normalized, subject) {
            Ok(()) => return Ok(()),
            Err(error) => first.get_or_insert(error),
        };
    }
    Err(first.unwrap_or(PhysicalError::Intrinsic {
        rule: "request-binding",
        region: region.index.id,
    }))
}

/// Binds one publishing copy to the declared output it publishes.
///
/// **Every fact the copy carries is checked, and the check is against the
/// request rather than against the proposal.** The scalar program must be
/// exactly the identity — a provider substituting any other expression would be
/// computing something under a declaration that says it copies — the accesses
/// must be exactly one linear read of a materialized intermediate followed by the
/// owning write of a program output, and the iteration domain must be some
/// declared output's published domain, because publishing into a domain no
/// declared output has is publishing into a buffer the interface never named.
///
/// What it deliberately does not check is *which* output, or that the value read
/// is the one that output names. Neither is knowable from a region: the copy's
/// source is a materialization edge the cover chose, and the pairing of edge to
/// publication is program scope. `tiler_ir::program`'s publishing-copy
/// obligations prove exactly that, and the assembler's `publishing-copy-pass-
/// count` refusal is what keeps a copy from being assembled without one.
fn verify_publishing_copy_binding(
    region: &ScheduledRegion,
    semantic_members: &[SemanticStage],
    subject: &VerifiedRequestSubject,
) -> Result<(), PhysicalError> {
    let bound = semantic_members.is_empty()
        && matches!(
            &region.index.scalar_program,
            ScalarProgram::PointwiseF32(expression) if *expression == identity_expression()
        )
        && matches!(
            region.index.accesses.as_slice(),
            [
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::LinearIdentity,
                    ..
                },
                Access {
                    tensor: TensorRole::Output,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    ..
                },
            ]
        )
        && subject
            .normalized()
            .outputs()
            .iter()
            .any(|normalized| published_shape(normalized) == &region.index.iteration_shape);
    if bound {
        Ok(())
    } else {
        intrinsic("publishing-copy-binding", region.index.id)
    }
}

/// Returns the domain one recognized output publishes.
fn published_shape(normalized: &NormalizedOutputSubject) -> &Shape {
    match normalized {
        NormalizedOutputSubject::Pointwise(normalized) => &normalized.shape,
        NormalizedOutputSubject::SerialSum(normalized) => normalized.output_shape(),
        NormalizedOutputSubject::Contraction(normalized) => &normalized.output_shape,
        NormalizedOutputSubject::Epilogue(normalized) => normalized.shape(),
        NormalizedOutputSubject::Staged(normalized) => &normalized.occurrence().output_shape,
    }
}

fn verify_region_output_binding(
    region: &ScheduledRegion,
    semantic_members: &[SemanticStage],
    normalized: &NormalizedOutputSubject,
    subject: &VerifiedRequestSubject,
) -> Result<(), PhysicalError> {
    let expected = match (normalized, &region.index.scalar_program) {
        (NormalizedOutputSubject::Pointwise(normalized), scalar) => {
            // The recognized expression itself, compared whole and *in its own
            // width*. It binds node topology, ordered operands, constant bits,
            // shared reads, and the explicit root, so a provider cannot
            // substitute an algebraically similar but unproved expression for
            // it — and the pairing binds the width too, so a `bf16` region
            // cannot claim a binary32 subject whose nodes happen to correspond.
            // A mismatched pairing answers `false`, which is the fail-closed
            // direction and the reason this is one arm rather than two with a
            // fall-through.
            let carries = match (&normalized.expression, scalar) {
                (RecognizedPointwise::F32(recognized), ScalarProgram::PointwiseF32(expression)) => {
                    expression == recognized
                }
                (
                    RecognizedPointwise::Bf16(recognized),
                    ScalarProgram::PointwiseBf16(expression),
                ) => expression == recognized,
                _ => false,
            };
            carries
                && element_count(&normalized.shape, region.index.id)? == normalized.elements
                && semantic_members == normalized.members
                && region.index.id == RegionId::new(0)
                && region.index.iteration_shape == normalized.shape
                && elementwise_reads_match(&region.index.accesses, &normalized.reads)
        }
        (
            NormalizedOutputSubject::Contraction(normalized),
            ScalarProgram::StrictTensorContraction {
                contracted_shape,
                canonical_nan_bits,
                ..
            },
        ) => {
            // Every quantity the region carries is re-derived from the subject
            // and compared, including both operands' coordinate maps: a region
            // whose access relation differs from the recognized structure's
            // would compute a different contraction over the same buffers, and
            // the intrinsic verifier — which sees only the region — cannot
            // notice that.
            element_count(&normalized.output_shape, region.index.id)? == normalized.output_elements
                && element_count(&normalized.contracted_shape, region.index.id)?
                    == normalized.contracted_elements
                && semantic_members == normalized.members
                && region.index.id == RegionId::new(0)
                && region.index.iteration_shape == normalized.output_shape
                && contracted_shape == &normalized.contracted_shape
                && *canonical_nan_bits == subject.numerical_contract().canonical_arithmetic_nan_bits
                && contraction_accesses_match(&region.index.accesses, normalized)
        }
        // A staged subject binds one of exactly two regions, and *which* stage a
        // region claims is decided by its members before anything else is
        // compared: both stages belong to one occurrence, so the scalar program
        // alone would not separate a fold that claims the pass's atom from one
        // that claims its own.
        //
        // Every quantity is re-derived from the subject through the same
        // [`staged_plan`] the builder used and compared, which is what makes this
        // a binding rather than a resemblance: the epilogue chain is compared
        // whole — node topology, ordered operands, constant bits, and the explicit
        // root — so a provider cannot substitute an algebraically similar chain
        // with a different rounding count, and the pass's read list is compared
        // tensor by tensor and relation by relation, so a region reading the
        // handed value densely where the law replicates it is refused.
        //
        // An occurrence with no plan reaches here only from a forged proposal —
        // [`spell_staged`] declines it — and answers `false` through the `else`,
        // which is the fail-closed direction.
        //
        // A region claiming neither stage is a region of the producer's
        // partition across a staged operand's materialization edge, and it is
        // re-offered to that producer's own subject exactly as a chain's is. A
        // subject with no producer refuses under this function's own
        // `request-binding` rule, which is the same fail-closed direction: a
        // member set that is neither this occurrence's stages nor a producer's
        // is forged.
        (NormalizedOutputSubject::Staged(normalized), scalar) => {
            let occurrence = normalized.occurrence();
            let fold = SemanticStage::first(occurrence.member);
            if semantic_members != [fold] && semantic_members != [fold.next_stage()] {
                return match normalized.producer() {
                    Some(producer) => {
                        verify_region_output_binding(region, semantic_members, producer, subject)
                    }
                    None => intrinsic("request-binding", region.index.id),
                };
            }
            match (staged_plan(occurrence), scalar) {
                (
                    Some(plan),
                    ScalarProgram::SquaredSerialSumThenEpilogue {
                        axes,
                        canonical_nan_bits,
                        empty_identity_bits,
                        epilogue,
                        ..
                    },
                ) => {
                    semantic_members == [fold]
                        && region.index.id == STAGED_FOLD_REGION
                        && region.index.iteration_shape == plan.handed_shape
                        && element_count(&plan.handed_shape, region.index.id)?
                            == plan.handed_elements
                        && axes == &plan.axes
                        && epilogue == &plan.fold_epilogue
                        && *canonical_nan_bits
                            == subject.numerical_contract().canonical_arithmetic_nan_bits
                        && *empty_identity_bits == 0.0_f32.to_bits()
                        && staged_fold_access_matches(&region.index.accesses, &plan)
                }
                (Some(plan), ScalarProgram::PointwiseF32(expression)) => {
                    semantic_members == [fold.next_stage()]
                        && region.index.id == STAGED_PASS_REGION
                        && region.index.iteration_shape == occurrence.output_shape
                        && element_count(&occurrence.output_shape, region.index.id)?
                            == occurrence.output_elements
                        && expression == &plan.pass_expression
                        && staged_pass_reads_match(&region.index.accesses, &plan)
                }
                _ => false,
            }
        }
        // The fail-closed answer for a contraction subject paired with any other
        // scalar program: it is bound above against the one program its
        // recognizer produces, so any other pairing is forged. The pointwise
        // subject needs no companion arm — it matches every scalar program and
        // answers `false` for each pairing it does not carry, which is what lets
        // it bind two widths without either falling through to the other.
        (NormalizedOutputSubject::Contraction(_), _) => false,
        // A chain binds either its epilogue region or a region of its producer's
        // partition, and the *members* are what separate the two: an epilogue's
        // region and a fold's prologue are both `PointwiseF32` regions, so the
        // scalar program cannot tell them apart and the coverage can.
        //
        // A region claiming the epilogue's members is then checked whole here
        // and never re-offered to the producer, so a forged epilogue cannot fall
        // through and bind as something else.
        (NormalizedOutputSubject::Epilogue(normalized), scalar) => {
            let ScalarProgram::PointwiseF32(expression) = scalar else {
                return verify_region_output_binding(
                    region,
                    semantic_members,
                    normalized.producer(),
                    subject,
                );
            };
            if semantic_members != normalized.members() {
                return verify_region_output_binding(
                    region,
                    semantic_members,
                    normalized.producer(),
                    subject,
                );
            }
            element_count(normalized.shape(), region.index.id)? == normalized.elements()
                && region.index.id == EPILOGUE_REGION
                && region.index.iteration_shape == *normalized.shape()
                // The recognized epilogue expression itself, compared whole, for
                // the reason the whole-program arm compares its own: node
                // topology, ordered operands, constant bits, shared reads, and
                // the explicit root.
                && expression == normalized.expression()
                && epilogue_accesses_match(&region.index.accesses, normalized)
        }
        (NormalizedOutputSubject::SerialSum(normalized), scalar) => {
            if !tiler_ir::schedule::axes_are_canonical(
                normalized.reduction_axes(),
                normalized.input_shape().rank(),
            ) || element_count(normalized.input_shape(), region.index.id)?
                != normalized.input_elements()
                || element_count(normalized.output_shape(), region.index.id)?
                    != normalized.output_elements()
                || normalized
                    .input_shape()
                    .without_axes(normalized.reduction_axes())
                    != *normalized.output_shape()
            {
                return intrinsic("request-subject-shape", region.index.id);
            }
            // A split reduction's two passes bind to the same subject as the
            // fused region does — they realize the same occurrences by a
            // different physical route — but neither has the fused region's
            // iteration shape, so they are matched on their own terms rather
            // than by relaxing the single-dispatch rules below.
            if matches!(
                region.schedule.reduction,
                ReductionTopology::MultiPass { .. }
            ) {
                return verify_multi_pass_subject_binding(
                    region,
                    semantic_members,
                    normalized,
                    subject,
                );
            }
            // A single-workgroup tree realizes the same occurrences as the
            // materialized reduction region by a different physical route, and
            // like a partial pass it iterates the output shape once per
            // participant, so it is matched on its own terms rather than by
            // relaxing the single-dispatch rules below.
            if matches!(
                region.schedule.reduction,
                ReductionTopology::CooperativeWorkgroup { .. }
            ) {
                return verify_workgroup_tree_subject_binding(
                    region,
                    semantic_members,
                    normalized,
                    subject,
                );
            }
            match scalar {
                ScalarProgram::PointwiseF32(expression) => {
                    // The recognized prologue itself, compared whole: node
                    // topology, ordered operands, constant bits, shared reads,
                    // and the explicit root. A provider cannot substitute an
                    // algebraically similar but unproved expression for it — and
                    // a subject with no prologue binds no pointwise region at
                    // all, because `None` equals no expression.
                    normalized.prologue() == Some(expression)
                        && semantic_members == normalized.members().pointwise()
                        && region.index.id == RegionId::new(0)
                        && region.index.iteration_shape == *normalized.input_shape()
                        && elementwise_reads_match(
                            &region.index.accesses,
                            normalized.prologue_reads(),
                        )
                }
                ScalarProgram::StrictSerialSum {
                    axes,
                    canonical_nan_bits,
                    ..
                } => {
                    semantic_members == normalized.members().reduction()
                        && region.index.id == RegionId::new(1)
                        && region.index.iteration_shape == *normalized.output_shape()
                        && axes == normalized.reduction_axes()
                        && reduction_access_matches(
                            &region.index.accesses[0],
                            normalized,
                            subject_contributor_tensor(normalized),
                        )
                        && *canonical_nan_bits
                            == subject.numerical_contract().canonical_arithmetic_nan_bits
                }
                ScalarProgram::FusedMultiplyAddSerialSum {
                    scale_bits,
                    bias_bits,
                    axes,
                    canonical_nan_bits,
                    ..
                } => {
                    semantic_members == normalized.members().all()
                        && region.index.id == RegionId::new(0)
                        && region.index.iteration_shape == *normalized.output_shape()
                        // Re-derived from the recognized prologue rather than
                        // read back: the fused scalar program is admitted only
                        // for the one expression it can spell, so a prologue
                        // that is not that expression has no fused form at all.
                        && normalized.prologue().and_then(affine_prologue)
                            == Some((*scale_bits, *bias_bits))
                        && axes == normalized.reduction_axes()
                        // The declared input the recognized prologue read, not
                        // the first one: a fused region binding another tensor
                        // folds a buffer the program never named, and only the
                        // recognized read list says which it is.
                        && fused_contributor_tensor(normalized.prologue_reads()).is_some_and(
                            |contributor| {
                                reduction_access_matches(
                                    &region.index.accesses[0],
                                    normalized,
                                    contributor,
                                )
                            },
                        )
                        && *canonical_nan_bits
                            == subject.numerical_contract().canonical_arithmetic_nan_bits
                }
                // None of these is produced by the recognized whole-program
                // shapes this arm verifies. The strict-affine one is refused
                // upstream; the squaring-prologue sum belongs to
                // `tiler::rms-norm-f32@1` and the extrema fold to
                // `tiler::softmax-f32@1`, neither of which the recognizer admits;
                // and a contraction binds to its own subject variant above, so a
                // serial-sum subject claiming one is a forged pairing. Answering
                // `false` is the fail-closed answer rather than a deferral.
                //
                // The extrema fold could not bind here even if the recognizer
                // admitted it: this arm's subject carries an empty-domain
                // identity, and the identity-less family has none to compare.
                //
                // The BF16 pointwise program is refused here for a further
                // reason of its own, and the reason moved rather than
                // disappearing when the recognizer's `dtype-f32` rule did: a
                // fold is recognized only from `tiler::strict-serial-sum-f32@1`,
                // whose contributor tensor is binary32, so a program reaching
                // this subject is `f32` throughout and no
                // `NormalizedSerialSumSubject` can name a BF16 prologue at all.
                // Answering `false` states that rather than leaving a BF16 region
                // able to claim an `f32` subject.
                ScalarProgram::StrictAffineU4Dequantize { .. }
                | ScalarProgram::PointwiseBf16(_)
                | ScalarProgram::SquaredSerialSum { .. }
                // The squaring fold *with* an epilogue is refused here for the
                // sharper reason: it is the producing stage of a staged
                // occurrence, which binds against the staged subject arm above.
                // A serial-sum subject claiming one would be a recognized
                // reduction wearing another occurrence's realization.
                | ScalarProgram::SquaredSerialSumThenEpilogue { .. }
                | ScalarProgram::StrictSerialMaximum { .. }
                | ScalarProgram::StrictTensorContraction { .. } => false,
            }
        }
    };
    if !expected {
        return intrinsic("request-binding", region.index.id);
    }
    Ok(())
}

/// Re-derives one epilogue region's reads from the recognized read list.
///
/// **Position by position, both halves.** The boundary tensor says which buffer
/// the read binds and the relation says how it addresses it, and a region
/// agreeing on one but not the other computes a different program over the same
/// buffers — which the intrinsic verifier, seeing only the region, cannot
/// notice. The staged read's position is checked by this pairing too: two
/// regions whose reads bind the same tensors in a different order serve
/// different expression leaves from the same buffers.
/// Requires one whole-program or prologue region's reads to be the recognized
/// ones, position by position.
///
/// **This binds the read list itself, which the expression cannot.** A region
/// reading declared input `0` twice and one reading inputs `0` and `1` carry the
/// same three-leaf expression, and so do a dense read and a transposed one of
/// the same tensor — so a provider substituting either would compute a different
/// tensor over the same buffers while every other fact in this arm agreed. The
/// intrinsic verifier sees only the region and cannot notice, which is the same
/// argument the contraction and epilogue arms make for their own access checks.
fn elementwise_reads_match(accesses: &[Access], recognized: &[(u32, LogicalAccess)]) -> bool {
    let Some((_, reads)) = accesses.split_last() else {
        return false;
    };
    reads.len() == recognized.len()
        && reads
            .iter()
            .zip(recognized)
            .all(|(access, (ordinal, map))| {
                access.tensor
                    == TensorRole::Input {
                        ordinal: InputOrdinal::new(*ordinal),
                    }
                    && access.map == *map
            })
}

/// Returns whether one staged fold's contributor read is the plan's.
///
/// The read the law's producing stage performs: the folded operand, addressed
/// over the reduced domain the plan derived. A region folding another declared
/// input, or the same one over other axes, computes a different value under a
/// region the intrinsic verifier cannot fault — which is exactly what this
/// separates.
fn staged_fold_access_matches(accesses: &[Access], plan: &StagedPlan) -> bool {
    let [read, _write] = accesses else {
        return false;
    };
    read.tensor == plan.contributor
        && matches!(
            &read.map,
            LogicalAccess::ReductionContributor { input_shape, output_shape, axes, .. }
                if input_shape == &plan.input_shape
                    && output_shape == &plan.handed_shape
                    && axes == &plan.axes
        )
}

/// Returns whether one staged pass's reads are the plan's, in order.
fn staged_pass_reads_match(accesses: &[Access], plan: &StagedPlan) -> bool {
    let Some((_, reads)) = accesses.split_last() else {
        return false;
    };
    reads.len() == plan.pass_reads.len()
        && reads
            .iter()
            .zip(&plan.pass_reads)
            .all(|(access, (tensor, map))| access.tensor == *tensor && access.map == *map)
}

fn epilogue_accesses_match(
    accesses: &[Access],
    normalized: &crate::request::NormalizedEpilogueSubject,
) -> bool {
    let Some((_, reads)) = accesses.split_last() else {
        return false;
    };
    reads.len() == normalized.reads().len()
        && reads
            .iter()
            .zip(normalized.reads())
            .all(|(access, (read, map))| access.tensor == read.tensor() && access.map == *map)
}

/// Binds one pass of a split reduction to the request subject it refines.
///
/// The partial pass claims the reduction occurrence's first stage, exactly as
/// the materialized strategy's single reduction region does; the final pass
/// claims the stage after it. The two atoms are distinct, so the passes together
/// realize the one occurrence the region they replace realized without either
/// claiming the other's work — and the *exact* stage is required here rather
/// than merely "something later", because a chain that skipped a stage would
/// leave part of a longer realization unattributed and still bind.
fn verify_multi_pass_subject_binding(
    region: &ScheduledRegion,
    semantic_members: &[SemanticStage],
    normalized: &crate::request::NormalizedSerialSumSubject,
    subject: &VerifiedRequestSubject,
) -> Result<(), PhysicalError> {
    let ReductionTopology::MultiPass {
        pass, partition, ..
    } = &region.schedule.reduction
    else {
        return intrinsic("request-binding", region.index.id);
    };
    let expected = match pass {
        tiler_ir::schedule::ReductionPass::Partial => {
            matches!(
                &region.index.scalar_program,
                ScalarProgram::StrictSerialSum { axes, canonical_nan_bits, .. }
                    if axes == normalized.reduction_axes()
                        && *canonical_nan_bits
                            == subject.numerical_contract().canonical_arithmetic_nan_bits
            ) && semantic_members == normalized.members().reduction()
                && region.index.id == RegionId::new(2)
                && reduction_access_matches(
                    &region.index.accesses[0],
                    normalized,
                    subject_contributor_tensor(normalized),
                )
                && tiler_ir::schedule::partial_reduction_shape(
                    normalized.output_shape(),
                    *partition,
                )
                .is_some_and(|shape| shape == region.index.iteration_shape)
        }
        tiler_ir::schedule::ReductionPass::Final => {
            matches!(
                &region.index.scalar_program,
                ScalarProgram::StrictSerialSum { canonical_nan_bits, .. }
                    if *canonical_nan_bits
                        == subject.numerical_contract().canonical_arithmetic_nan_bits
            ) && semantic_members
                == normalized
                    .members()
                    .reduction()
                    .iter()
                    .map(|atom| atom.next_stage())
                    .collect::<Vec<_>>()
                && region.index.id == RegionId::new(3)
                && region.index.iteration_shape == *normalized.output_shape()
        }
        // A pass role this compilation does not construct binds to no subject.
        // Refusing rather than guessing keeps a role added later from being
        // silently accepted under the rules the two known roles were checked
        // against.
        _ => false,
    };
    if !expected {
        return intrinsic("request-binding", region.index.id);
    }
    Ok(())
}

/// Binds one single-workgroup tree region to the request subject it refines.
///
/// It claims the reduction occurrence, exactly as the materialized strategy's
/// single reduction region does and as the split's partial pass does: the tree
/// *replaces* that one region rather than adding a stage, so there is no second
/// region to leave the occurrence to.
///
/// The participant count is re-derived from the request rather than read from
/// the topology, because reading it back would make this check agree with
/// whatever the provider chose instead of with what the request admits — the one
/// thing a subject binding exists to stop.
fn verify_workgroup_tree_subject_binding(
    region: &ScheduledRegion,
    semantic_members: &[SemanticStage],
    normalized: &crate::request::NormalizedSerialSumSubject,
    subject: &VerifiedRequestSubject,
) -> Result<(), PhysicalError> {
    let ReductionTopology::CooperativeWorkgroup { partition, .. } = &region.schedule.reduction
    else {
        return intrinsic("request-binding", region.index.id);
    };
    let expected = matches!(
        &region.index.scalar_program,
        ScalarProgram::StrictSerialSum { axes, canonical_nan_bits, .. }
            if axes == normalized.reduction_axes()
                && *canonical_nan_bits
                    == subject.numerical_contract().canonical_arithmetic_nan_bits
    ) && semantic_members == normalized.members().reduction()
        && region.index.id == RegionId::new(4)
        && reduction_access_matches(
            &region.index.accesses[0],
            normalized,
            subject_contributor_tensor(normalized),
        )
        && tiler_ir::schedule::partial_reduction_shape(normalized.output_shape(), *partition)
            .is_some_and(|shape| shape == region.index.iteration_shape);
    if !expected {
        return intrinsic("request-binding", region.index.id);
    }
    Ok(())
}

/// Requires both operand reads to realize the recognized structure exactly.
///
/// Checked per declared input ordinal rather than as a set: the ordinal is the
/// buffer position, so two accesses carrying the right pair of maps in the wrong
/// order would bind each operand to the other's tensor and still look complete.
fn contraction_accesses_match(accesses: &[Access], normalized: &NormalizedContraction) -> bool {
    let Some((_, reads)) = accesses.split_last() else {
        return false;
    };
    if reads.len() != normalized.input_shapes.len() {
        return false;
    }
    reads.iter().enumerate().all(|(declaration, read)| {
        u32::try_from(declaration).is_ok_and(|ordinal| {
            read.tensor
                == TensorRole::Input {
                    ordinal: InputOrdinal::new(ordinal),
                }
        }) && read.map
            == LogicalAccess::ContractionOperand {
                operand_shape: normalized.input_shapes[declaration].clone(),
                output_shape: normalized.output_shape.clone(),
                contracted_shape: normalized.contracted_shape.clone(),
                sources: contraction_operand_sources(normalized, declaration),
                order: ContributorOrder::OriginalAxisLexicographic,
            }
    })
}

/// The contributor tensor one recognized fold's own region must bind.
///
/// The subject projection of [`contributor_tensor`], derived from the same fact —
/// whether the recognized program has a prologue — so a region and the builder
/// that produced it cannot disagree about where the contributors live. A fold with
/// a prologue reads the intermediate that prologue region materialized; one
/// without reads the declared input directly.
///
/// The *fused* region does not ask, and resolves
/// [`fused_contributor_tensor`] instead: it carries the prologue inside its own
/// scalar program, so it reads the tensor the prologue read precisely because a
/// prologue exists.
fn subject_contributor_tensor(
    normalized: &crate::request::NormalizedSerialSumSubject,
) -> TensorRole {
    declared_contributor_tensor(normalized.contributor_input())
}

/// Requires one fold's contributor read to realize the recognized reduction.
///
/// `tensor` is stated by the caller rather than derived here because the four
/// spellings do not agree on it: the materialized fold, the split's partial pass,
/// and the cooperative tile bind whichever tensor holds the *declared contributor
/// domain*, while the fused region binds the first input because its scalar
/// program contains the prologue. Checking it at all is what stops a provider
/// offering a `sum(x)` region that reads an intermediate no cover materialized —
/// which `tiler-ir` admits as an intrinsically coherent region and the program
/// assembler would then refuse for a missing edge, naming the wrong authority.
fn reduction_access_matches(
    access: &Access,
    normalized: &crate::request::NormalizedSerialSumSubject,
    tensor: TensorRole,
) -> bool {
    access.tensor == tensor
        && matches!(
            &access.map,
            LogicalAccess::ReductionContributor { input_shape, output_shape, axes, .. }
                if input_shape == normalized.input_shape()
                    && output_shape == normalized.output_shape()
                    && axes == normalized.reduction_axes()
        )
}

/// Assesses one scheduled region against the typed feasibility authority.
///
/// Why a resource assessment did not prove feasibility, **unattributed**.
///
/// The verdict without the blame. `assess_region` attributes it to a
/// `RegionId`; an opaque physical call attributes the same verdict to the call
/// that proposed it. One feasibility decision, two attributions — which is what
/// ADR 0043's single decision requires, since the *verdict* is what must be
/// shared and only the subject differs.
///
/// Carrying a `RegionId` in here instead would force any caller that is not a
/// region to invent one, and a feasibility rejection attributed to a region that
/// does not exist is worse than no attribution at all: a reader chasing it finds
/// nothing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AdmissionEvidence {
    /// Every hard predicate was resolved before planning.
    Proven(ProvenEvidence),
    /// Every unresolved predicate has a typed query path before routing commit.
    Deferred(DeferredSet),
}

impl AdmissionEvidence {
    /// The checks already proven at compile time.
    pub(crate) const fn proven(&self) -> &ProvenEvidence {
        match self {
            Self::Proven(evidence) => evidence,
            Self::Deferred(deferred) => deferred.proven(),
        }
    }

    /// The remaining compiler-minted obligations, when there are any.
    pub(crate) const fn deferred(&self) -> Option<&DeferredSet> {
        match self {
            Self::Proven(_) => None,
            Self::Deferred(deferred) => Some(deferred),
        }
    }

    /// The capability checks already resolved at compile time.
    pub(crate) fn predicates(&self) -> &[crate::target::feasibility::ResolvedPredicate] {
        self.proven().predicates()
    }

    /// The numerical dimensions already honoured at compile time.
    pub(crate) fn honoured(&self) -> &[crate::target::honourability::HonouredDimension] {
        self.proven().honoured()
    }

    /// The synchronization realization already established at compile time.
    ///
    /// `None` for a region that requires none, which is what keeps a
    /// zero-synchronization program's explanation free of a manufactured row.
    pub(crate) const fn synchronization(
        &self,
    ) -> Option<&crate::target::feasibility::RealizedSynchronization> {
        self.proven().synchronization()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the unattributed verdict; its second caller is the opaque-call admission being built"
)]
pub(crate) enum ResourceVerdict {
    /// The target profile or the proposal itself was malformed.
    Intrinsic(FeasibilityError),
    /// The target refused the proposal, with the representative cause.
    Rejected(RejectionCause),
    /// The proposal requires a synchronization realization nothing declares.
    ///
    /// Separated from [`Self::Unknown`] because the two blame different things.
    /// A capability or dimension with no path is a gap in the profile's own
    /// vocabulary, which a caller reports as an unresolved assessment. A
    /// synchronization subject nothing speaks to is a *complete, well-formed*
    /// requirement this target has simply never been asked about — the exact
    /// case `admit-the-first-typed-synchronization-point-and-atomic-target-authority`
    /// specified as "`Unknown` … before executable-frontier admission" — and
    /// reporting it as an unresolved assessment would attribute a target's
    /// silence to the provider that emitted valid IR.
    UnrealizedSynchronization(tiler_ir::schedule::SynchronizationSubject),
    /// At least one predicate has no admissible fact or query path.
    Unknown,
}

/// Assesses exact resource requirements against a target, attributing nothing.
///
/// The shared half of the feasibility decision. Every caller runs this; each
/// then maps a [`ResourceVerdict`] onto its own error vocabulary.
#[allow(
    dead_code,
    reason = "the shared feasibility core; the opaque-call admission is its second caller"
)]
pub(crate) fn assess_resources(
    requirements: ResourceRequirements,
    arithmetic: ArithmeticType,
    work_items: u64,
    target: &TargetProfile,
) -> Result<AdmissionEvidence, ResourceVerdict> {
    let proposal = region_proposal(requirements, arithmetic, work_items)
        .map_err(ResourceVerdict::Intrinsic)?;
    match target
        .checked()
        .assess(&proposal, AvailabilityPhase::CompileProfile)
    {
        FeasibilityOutcome::Proven(evidence) => Ok(AdmissionEvidence::Proven(evidence)),
        FeasibilityOutcome::Deferred(deferred) if deferred.dimensions().is_empty() => {
            Ok(AdmissionEvidence::Deferred(deferred))
        }
        // An unknown that names a synchronization subject keeps that subject.
        // The `if` order is deliberate: a candidate can be unknown on a
        // capability *and* on its synchronization, and the synchronization is
        // the more specific answer — it names a complete subject rather than a
        // missing bound, so reporting it loses nothing a reader could act on.
        FeasibilityOutcome::Unknown(unknown) if unknown.synchronization().is_some() => {
            Err(ResourceVerdict::UnrealizedSynchronization(
                unknown
                    .synchronization()
                    .expect("the guard proved the unknown names a subject")
                    .subject(),
            ))
        }
        FeasibilityOutcome::Deferred(_) | FeasibilityOutcome::Unknown(_) => {
            Err(ResourceVerdict::Unknown)
        }
        FeasibilityOutcome::Rejected(rejection) => {
            Err(ResourceVerdict::Rejected(rejection.representative()))
        }
    }
}

/// Assesses one region's resources, attributing any verdict to that region.
pub(crate) fn assess_region(
    region: RegionId,
    requirements: ResourceRequirements,
    arithmetic: ArithmeticType,
    work_items: u64,
    target: &TargetProfile,
) -> Result<AdmissionEvidence, PhysicalError> {
    assess_resources(requirements, arithmetic, work_items, target).map_err(
        |verdict| match verdict {
            ResourceVerdict::Intrinsic(error) => feasibility_intrinsic(&error, region),
            ResourceVerdict::Rejected(RejectionCause::Numerical(cause)) => {
                PhysicalError::Numerical { region, cause }
            }
            ResourceVerdict::Rejected(RejectionCause::Synchronization(cause)) => {
                PhysicalError::Synchronization {
                    region,
                    cause: Box::new(cause),
                }
            }
            ResourceVerdict::Rejected(RejectionCause::Capability(predicate)) => {
                PhysicalError::Target {
                    rule: predicate.axis().key(),
                    region,
                    required: predicate.required().value(),
                    available: predicate.available().value(),
                }
            }
            ResourceVerdict::UnrealizedSynchronization(subject) => {
                PhysicalError::UnrealizedSynchronization { region, subject }
            }
            ResourceVerdict::Unknown => PhysicalError::Intrinsic {
                rule: "target-assessment-unresolved",
                region,
            },
        },
    )
}

/// Assesses one numerical contract alone against a target's declaration.
///
/// The request boundary resolves a caller's stated preference through this: the
/// proposal carries the contract's four dimensions and *no* capability
/// requirement, because whether a target honours a contract is a fact about the
/// contract and the target, independent of any region, schedule, or cost. A
/// region is assessed again later against the same authority, which is
/// defence in depth rather than a second decision.
pub(crate) fn assess_contract(
    target: &TargetProfile,
    contract: StrictF32NumericalContract,
) -> Result<FeasibilityOutcome, FeasibilityError> {
    let proposal = FeasibilityProposal::new(
        CONTRACT_PROPOSAL_CANDIDATE,
        Vec::new(),
        contract.dimension_requirements(),
    )?;
    Ok(target
        .checked()
        .assess(&proposal, AvailabilityPhase::CompileProfile))
}

/// Returns the canonical descriptor bytes of one target profile.
///
/// Borrowed from the same immutable checked profile the feasibility assessment
/// uses, so this path never reconstructs or revalidates target facts.
///
/// This is only half of what ADR 0043 requires an artifact to record. The other
/// half — which feasibility rules compared the candidate against these facts —
/// is [`crate::target::feasibility::GOVERNED_FEASIBILITY_RULE_SET`], and it is not
/// derived per target because the rules do not vary by target.
#[cfg(test)]
pub(crate) fn target_profile_descriptor(target: &TargetProfile) -> &[u8] {
    target.canonical_descriptor()
}

/// Builds the typed candidate proposal for one scheduled region.
///
/// The candidate requires complete support for the governed unsigned-64 KIR
/// index operation family and the device address space whenever its resource
/// requirements demand it. It does not infer a device address width from that
/// arithmetic type. Its numerical requirements are the region's declared
/// realization carried forward **per dimension** rather than collapsed into one
/// summary bit — the collapse the retired `StrictF32Arithmetic` axis forced, and
/// which could neither name a failing dimension nor express emulation.
///
/// The synchronization requirement is carried the *opposite* way: forward as one
/// atomic subject rather than per dimension, because each of its dimensions is
/// separately true of some realization and their conjunction is what the region
/// needs. It is also carried **conditionally**, and that is what keeps the
/// absence canonical: a region requiring no synchronization composes no
/// requirement at all, so no predicate is resolved, no target fact is consulted,
/// and no explain row exists to be a manufactured zero — as the retired
/// barrier-count axis must never again yield `required 0`.
///
/// The index-arithmetic requirement is carried the third way: **unconditionally
/// and by value**. It is not conditional, because every region computes
/// coordinates and so no region derives its absence; and it is not re-derived
/// here, because `tiler_ir::schedule` already derived it from the region's own
/// coordinate space. `index_arithmetic_requirement` therefore classifies a
/// value this function received rather than deciding one, which is what keeps a
/// single producer authority over a fact the verified program states.
fn region_proposal(
    requirements: ResourceRequirements,
    arithmetic: ArithmeticType,
    work_items: u64,
) -> Result<FeasibilityProposal, FeasibilityError> {
    // **The subject's value identity is derived from the region's own arithmetic,
    // not written beside it.** A target declares honourability for a
    // `ScalarArithmetic`, and a requirement matches a declaration only when
    // *both* halves of the subject agree — so a `bf16` region proposing
    // `tiler::f32@1` matched no `bf16` row a profile could ever declare, every
    // dimension resolved `Unknown`, and the region was refused as
    // `target-assessment-unresolved` on a profile whose measured `bf16` rows
    // answered it exactly. `crate::policy::dimension_requirements` already
    // derived its half this way and this one did not; the two halves of one
    // subject are now built by one constructor.
    //
    // `None` only if the arithmetic vocabulary and the governed scalar catalog
    // have drifted apart, which is a malformed proposal rather than an
    // infeasible region: a requirement set that quietly emptied itself would be
    // *vacuously* feasible, proven by every profile.
    let Some(subject) = crate::policy::arithmetic_subject(arithmetic) else {
        return Err(FeasibilityError::MalformedProposal {
            rule: "region-arithmetic-subject",
        });
    };
    let numerical = |dimension, behaviour| {
        NumericalRequirement::new(
            dimension,
            subject.arithmetic(),
            subject.resolved_type().clone(),
            behaviour,
        )
    };
    FeasibilityProposal::new_with_synchronization(
        REGION_PROPOSAL_CANDIDATE,
        vec![
            AxisRequirement::new(CapabilityAxis::GridAxisThreads, work_items),
            AxisRequirement::new(
                CapabilityAxis::WorkgroupThreads,
                u64::from(requirements.threads_per_workgroup),
            ),
            AxisRequirement::new(
                CapabilityAxis::BufferBindings,
                u64::from(requirements.buffer_bindings),
            ),
            index_arithmetic_requirement(requirements.index_arithmetic),
            AxisRequirement::new(
                CapabilityAxis::DeviceAddressSpace,
                u64::from(requirements.requires_device_memory),
            ),
            AxisRequirement::new(
                CapabilityAxis::LocalMemoryBytes,
                requirements.local_memory_bytes,
            ),
        ],
        vec![
            numerical(
                NumericalDimension::InputSubnormals,
                DimensionBehaviour::Subnormals(requirements.input_subnormals),
            ),
            numerical(
                NumericalDimension::ResultSubnormals,
                DimensionBehaviour::Subnormals(requirements.result_subnormals),
            ),
            numerical(
                NumericalDimension::Contraction,
                DimensionBehaviour::Transform(requirements.contraction),
            ),
            numerical(
                NumericalDimension::Reassociation,
                DimensionBehaviour::Transform(requirements.reassociation),
            ),
        ],
        requirements.synchronization,
    )
}

/// Maps one region's derived index arithmetic onto its capability axis.
///
/// **It classifies rather than derives, and that is the change.** This used to
/// take a [`tiler_ir::kernel::KernelType`] and re-run the index-role match
/// itself, which made the
/// compiler a second producer of a fact the verified schedule already states —
/// exactly the second authority `declare-a-required-gpu-family-in-the-artifact`
/// forbids for anything derivable from the verified program. The derivation now
/// happens once, in `tiler_ir::schedule`, and reaches here as a value.
///
/// Exhaustive so a variant added to [`IndexArithmetic`] is a build error until
/// its axis bound is stated. Storage availability alone never satisfies this
/// predicate.
const fn index_arithmetic_requirement(index_arithmetic: IndexArithmetic) -> AxisRequirement {
    match index_arithmetic {
        IndexArithmetic::CompleteU64 => AxisRequirement::new(CapabilityAxis::IndexArithmeticU64, 1),
    }
}

/// Maps a feasibility intrinsic error onto the physical-error contract.
///
/// A malformed profile or proposal is a contract violation, not a feasibility
/// outcome, so it fails closed as an intrinsic scheduling error.
fn feasibility_intrinsic(error: &FeasibilityError, region: RegionId) -> PhysicalError {
    let rule = match error {
        FeasibilityError::MalformedProfile { .. } => "target-profile-malformed",
        FeasibilityError::MalformedProposal { .. } => "target-proposal-malformed",
        // A profile too large to describe is a declaration defect in the same
        // class as a malformed one: it is a fact about the profile, decided
        // before any candidate is considered, and no other plan makes it
        // describable.
        FeasibilityError::DescriptorTooLong { .. } => "target-profile-descriptor-too-long",
    };
    PhysicalError::Intrinsic { rule, region }
}

/// Lowers one verified scheduled region to its verified structured kernel.
///
/// The structured kernel IR, its canonical lowering, and its verifier live in
/// [`tiler_ir::kernel`] (ADR 0070). This compiler entry point only forwards an
/// already request-bound verified region and re-attributes a lowering failure
/// to the region for the explain trace: a rejected lowering is a compiler
/// output defect, never a feasibility outcome.
pub(crate) fn lower_structured_kernel(
    scheduled: &VerifiedScheduledRegion,
) -> Result<VerifiedKernel, PhysicalError> {
    tiler_ir::kernel::lower_scheduled_region(scheduled.verified()).map_err(|error| {
        PhysicalError::Refinement {
            rule: error.rule(),
            region: scheduled.region().index.id,
        }
    })
}

/// Counts the elements of a shape, attributing any overflow to the region.
fn element_count(shape: &Shape, region: RegionId) -> Result<u64, PhysicalError> {
    tiler_ir::schedule::element_count(shape)
        .map_err(|_| PhysicalError::ShapeProductOverflow { region })
}

fn intrinsic<T>(rule: &'static str, region: RegionId) -> Result<T, PhysicalError> {
    Err(PhysicalError::Intrinsic { rule, region })
}

#[cfg(test)]
mod tests {
    use crate::region::SemanticMemberId;

    /// A fold's contributor tensor is its recognized ordinal, not the first.
    ///
    /// **Driven directly at the derivation, because nothing else can reach its
    /// general arm today.** `crate::request`'s `sum-contributor-ordinal`
    /// refusal declines a recognized fold over any declared input but the
    /// first, so every `contributor_input` a compilation produces is `Some(0)`
    /// or `None` and a version of this function that ignored the ordinal
    /// entirely would fail no end-to-end test — measured, not assumed: replacing
    /// its `Some` arm with `InputOrdinal::FIRST` leaves the whole `tiler-compiler`
    /// suite green. The general arm is *implemented and walled*, which is a
    /// different claim from tested, and this is the test that makes the
    /// difference small enough to state.
    /// `admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary`
    /// removes the wall.
    #[test]
    fn a_folds_contributor_tensor_is_its_recognized_declared_ordinal() {
        assert_eq!(
            super::declared_contributor_tensor(None),
            TensorRole::Intermediate,
        );
        for ordinal in [0_u32, 1, 7] {
            assert_eq!(
                super::declared_contributor_tensor(Some(ordinal)),
                TensorRole::Input {
                    ordinal: InputOrdinal::new(ordinal),
                },
            );
        }
    }

    /// The fused fold reads the tensor its prologue read, and declines the rest.
    ///
    /// Three refusals and one admission, each a different reason the fused
    /// vocabulary cannot spell a prologue: a read list of any length but one has
    /// no single contributor; a structural relation has nowhere to go in a
    /// region that addresses its input through a reduction relation; and a
    /// declared ordinal other than the first is refused by `tiler_ir::schedule`
    /// itself.
    #[test]
    fn the_fused_fold_reads_its_prologues_own_dense_first_input() {
        let dense = |ordinal| (ordinal, LogicalAccess::LinearIdentity);
        assert_eq!(
            super::fused_contributor_tensor(&[dense(0)]),
            Some(TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            }),
        );
        assert_eq!(super::fused_contributor_tensor(&[dense(1)]), None);
        assert_eq!(super::fused_contributor_tensor(&[]), None);
        assert_eq!(super::fused_contributor_tensor(&[dense(0), dense(1)]), None);
        assert_eq!(
            super::fused_contributor_tensor(&[(
                0,
                LogicalAccess::ReindexBijection {
                    operand_shape: Shape::from_dims([2, 2]),
                    result_shape: Shape::from_dims([2, 2]),
                    axes: vec![
                        tiler_ir::schedule::AxisDecode::read(1, 2),
                        tiler_ir::schedule::AxisDecode::read(2, 2),
                    ],
                },
            )]),
            None,
        );
    }

    /// Builds the five-node `input * scale + bias` expression as a forgery.
    ///
    /// Test-only and deliberately not shared with the region builders: those
    /// carry whatever expression the recognizer produced, and a helper they also
    /// used could not be substituted for one of them here.
    fn test_affine_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
        let mut expression = tiler_ir::schedule::PointwiseF32ExpressionBuilder::new();
        let input = expression.input(InputOrdinal::FIRST).unwrap();
        let scale = expression.constant(scale_bits).unwrap();
        let product = expression.multiply(input, scale).unwrap();
        let bias = expression.constant(bias_bits).unwrap();
        let root = expression.add(product, bias).unwrap();
        expression.build(root).unwrap()
    }
    use std::fmt::Write as _;
    /// The governed profile's canonical descriptor, pinned byte for byte.
    ///
    /// **This is a refactor guard, not a golden for its own sake.** The
    /// descriptor is encoded into `VerifiedRequestSubject`'s canonical explain
    /// subject and carried out through `Compilation::target_profile_descriptor`
    /// into the artifact's `TargetProfileDescriptorDigest`, so one changed byte
    /// moves every artifact identity and invalidates every cache entry. The
    /// producer's two-process determinism test and the serial-sum artifact
    /// identity would both catch it, but only after a whole compile-and-package
    /// cycle and without saying which field moved.
    ///
    /// `admit-a-caller-declared-target-profile` has to turn this type from a
    /// `Copy` struct of `&'static` fields into an owned one, touching roughly
    /// thirty sites. This exists so that refactor fails here, immediately and
    /// with a diff, rather than downstream.
    ///
    /// Regenerate only when the encoding is *deliberately* changed: print
    /// `target_profile_descriptor(&TargetProfile::governed())` as hex
    /// and step whatever domain tag the change requires in the same commit.
    #[test]
    fn the_governed_descriptor_bytes_do_not_move() {
        // Rebaselined when the governed profile raised its declared
        // buffer-binding bound from two to four — the widest signature the
        // bounded profile can assemble now that a region may read several input
        // tensors. Exactly one byte of the `buffer-bindings` row moves; the
        // declaration's shape and its domain tag are unchanged, so no domain
        // steps with it.
        //
        // An earlier rebaseline recorded the complete v10 declaration after
        // separating a future prepared-entry workgroup query from
        // compile-profile facts and replacing the grid placeholder with the
        // API-backed bound four. Device-address width remains absent because no
        // current KIR operation consumes it and the governed authority does not
        // establish it.
        // Rebaselined again at the `tiler.target-profile.declaration.v11` step,
        // which appends the synchronization-realization row family. The governed
        // profile declares *no* row, and its bytes still move: the family writes
        // its own domain separator and a count, so "this target says nothing
        // about synchronization" becomes a recorded fact. That is the step's
        // purpose — a `v10` declaration could not distinguish a target that had
        // been asked from one that had not.
        // Every artifact identity and cache entry derived from it moves with it. Regenerate with `cargo nextest run -p tiler-compiler -E 'test(the_governed_descriptor_bytes_do_not_move)'` and take `left`.
        const GOVERNED: &str = "000000000000002574696c65722e7461726765742d70726f66696c652e6465636c61726174696f6e2e76313100000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e7631000000000000002574696c65722e7461726765742d70726f66696c652e666163742d736f75726365732e7634000000000000000001000000000000007400000003010101000000000000002a74696c65722e676f7665726e65642d7461726765742d70726f66696c652d617574686f726974792e76310000000101000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e76310000000100000000000000050000000000000009677269642d61786973040000000000000000000000000000000f6275666665722d62696e64696e6773040000000000000000000000000000000d6465766963652d6d656d6f727901000000000000000000000000000000126c6f63616c2d6d656d6f72792d62797465730000000000000000000000000000000014696e6465782d61726974686d657469632d75363401000000000000000000000000000000010000000000000015746872656164732d7065722d776f726b67726f7570000000000000009274696c65722e7461726765742d70726f70657274792d71756572792e763100000000000000003874696c65722e7461726765742e70726570617265642d656e7472792e6d61782d746872656164732d7065722d776f726b67726f75702e763104000000000000000574696c6572000000000000001970726570617265642d656e7472792d70726f70657274696573000000010000000000000001000000000000004303000000000000003a74696c65722e7265736f6c7665642d76616c75652d747970652e76330001000000000000000574696c6572000000000000000366333200000001000000000000000c000101010100000101020100000201010100000201020100000302010100000302020100000402010100000402020100000502010100000602010100000904010100000a04010100000000000000002e74696c65722e7461726765742d70726f66696c652e64747970652d64697370617463686162696c6974792e7632000000000000000001000000000000003a74696c65722e7265736f6c7665642d76616c75652d747970652e76330001000000000000000574696c65720000000000000003663332000000010100000000000000003474696c65722e7461726765742d70726f66696c652e73796e6368726f6e697a6174696f6e2d7265616c697a6174696f6e2e7631000000000000000000";

        let profile = crate::request::TargetProfile::governed();
        let descriptor = target_profile_descriptor(&profile);
        let mut actual = String::with_capacity(descriptor.len() * 2);
        for byte in descriptor {
            write!(actual, "{byte:02x}").expect("writing to a String cannot fail");
        }
        assert_eq!(
            actual, GOVERNED,
            "the governed target profile's canonical descriptor moved; every artifact \
             identity and cache entry derived from it moves with it",
        );
    }
    use super::*;
    use crate::request::{CompilationRequest, StrictF32NumericalContract, verify_planned_request};
    use tiler_ir::kernel::{KernelConstant, OperationRef, OperationView};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::Axis;

    /// Returns the bounded loop range of the kernel's guarded region, if any.
    fn loop_bounds(kernel: &VerifiedKernel) -> Option<(u64, u64)> {
        guarded_operations(kernel).find_map(|view| match view {
            OperationView::SerialLoop(reduction) => Some((reduction.start(), reduction.end())),
            _ => None,
        })
    }

    /// Returns the constant the kernel commits, when it stores an immediate.
    fn stored_constant(kernel: &VerifiedKernel) -> Option<KernelConstant> {
        guarded_operations(kernel).find_map(|view| match view {
            OperationView::Store { value, .. } => kernel.value_constant(value).ok().flatten(),
            _ => None,
        })
    }

    fn guarded_operations(kernel: &VerifiedKernel) -> impl Iterator<Item = OperationView<'_>> {
        kernel
            .body()
            .operations()
            .filter_map(|operation| match operation.view() {
                OperationView::Predicated { body, .. } => Some(body),
                _ => None,
            })
            .flat_map(|body| body.operations().map(OperationRef::view))
    }

    fn request(shape: Shape, axes: impl IntoIterator<Item = Axis>) -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape)
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let pointwise = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, pointwise, axes).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let program = builder.build().unwrap();
        let request = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
        request.for_target(0).unwrap()
    }

    fn pointwise_request() -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let first = F32Constant::apply(&mut builder, 1.0e20_f32.to_bits()).unwrap();
        let second = F32Constant::apply(&mut builder, (-1.0e20_f32).to_bits()).unwrap();
        let left = F32Add::apply(&mut builder, input, first).unwrap();
        let root = F32Add::apply(&mut builder, left, second).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let program = builder.build().unwrap();
        let request = verify_planned_request(CompilationRequest::governed_under(
            &program,
            StrictF32NumericalContract::governed_relaxed(),
        ))
        .unwrap();
        request.for_target(0).unwrap()
    }

    #[test]
    fn fixed_schedules_and_kernels_refine_the_two_regions() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        let pointwise = lower_structured_kernel(&regions[0]).unwrap();
        let reduction = lower_structured_kernel(&regions[1]).unwrap();

        assert_eq!(regions[0].region().schedule.work_items, 4);
        assert_eq!(regions[1].region().schedule.work_items, 2);
        // Each kernel retains the exact identity of the region it refines.
        assert_eq!(pointwise.scheduled_region(), RegionId::new(0));
        assert_eq!(reduction.scheduled_region(), RegionId::new(1));
        assert_eq!(
            pointwise.scheduled_region_identity(),
            regions[0].canonical_identity()
        );
        assert_eq!(
            reduction.scheduled_region_identity(),
            regions[1].canonical_identity()
        );
        // The reduction realizes the scheduled contributor order as an explicit
        // bounded loop; the pointwise region carries none.
        assert_eq!(loop_bounds(&reduction), Some((1, 2)));
        assert_eq!(loop_bounds(&pointwise), None);
    }

    #[test]
    fn scheduled_regions_carry_a_transient_independent_identity() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        // Equivalent normalized regions built from a fresh request share bytes.
        let rebuilt = build_scheduled_regions(&request).unwrap();
        for (first, second) in regions.iter().zip(&rebuilt) {
            assert_eq!(
                first.verified.canonical_identity().as_bytes(),
                second.verified.canonical_identity().as_bytes()
            );
        }
        // The two distinct regions of one program have distinct identities.
        assert_ne!(
            regions[0].verified.canonical_identity().as_bytes(),
            regions[1].verified.canonical_identity().as_bytes()
        );
    }

    #[test]
    fn empty_reduction_lowers_to_explicit_positive_zero_stores() {
        let request = request(Shape::from_dims([2, 0]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        let reduction = lower_structured_kernel(&regions[1]).unwrap();
        // An empty reduction commits the proved identity directly: no loop and
        // no contributor load remain for a backend to interpret.
        assert_eq!(loop_bounds(&reduction), None);
        assert_eq!(
            stored_constant(&reduction),
            Some(KernelConstant::F32Bits(0.0_f32.to_bits()))
        );
    }

    #[test]
    fn schedule_and_kernel_fail_closed_on_refinement_mismatches() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();

        let mut invalid_schedule = regions[1].region().clone();
        invalid_schedule.schedule.reduction = ReductionTopology::None;
        assert_eq!(
            verify_schedule(
                invalid_schedule,
                regions[1].semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-or-access-refinement",
                region: RegionId::new(1),
            })
        );

        let mut invalid_access = regions[0].region().clone();
        invalid_access.index.accesses[0].bounds = BoundsWitnessId::new(9);
        assert_eq!(
            verify_schedule(
                invalid_access,
                regions[0].semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "proof-reference",
                region: RegionId::new(0),
            })
        );

        let mut invalid_proof = regions[0].region().clone();
        invalid_proof.index.bounds_proofs[0].kind =
            BoundsProofKind::LinearRange { element_count: 5 };
        assert_eq!(
            verify_schedule(
                invalid_proof,
                regions[0].semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "bounds-proof",
                region: RegionId::new(0),
            })
        );

        let mut invalid_numerics = regions[0].region().clone();
        invalid_numerics
            .index
            .numerical
            .canonical_arithmetic_nan_bits ^= 1;
        assert_eq!(
            verify_schedule(
                invalid_numerics,
                regions[0].semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-realization",
                region: RegionId::new(0),
            })
        );

        // The scale and the bias exchanged: the same two constants in the same
        // two node positions, applied the other way round. It is a different
        // binary32 function, and the binding compares the whole expression
        // rather than its constant set, so it must be refused.
        let (scale, bias) =
            fused_prologue_constants(request.sole_output()).expect("the fixture is affine");
        let mut wrong_expression = regions[0].region().clone();
        wrong_expression.index.scalar_program =
            ScalarProgram::PointwiseF32(test_affine_expression(bias, scale));
        assert_eq!(
            verify_schedule(
                wrong_expression,
                regions[0].semantic_members().to_vec(),
                &request,
            ),
            Err(PhysicalError::Intrinsic {
                rule: "request-binding",
                region: RegionId::new(0),
            })
        );
    }

    #[test]
    fn pointwise_schedule_requires_exact_expression_and_complete_ordered_coverage() {
        let request = pointwise_request();
        let (raw, members) =
            pointwise_region(&request, request.sole_output(), RegionWrite::ProgramOutput);
        let region = verify_schedule(raw, members, &request).unwrap();
        let expected = [
            SemanticStage::first(SemanticMemberId(0)),
            SemanticStage::first(SemanticMemberId(1)),
            SemanticStage::first(SemanticMemberId(2)),
            SemanticStage::first(SemanticMemberId(3)),
        ];
        assert_eq!(region.semantic_members(), expected);

        let mut wrong_expression = region.region().clone();
        wrong_expression.index.scalar_program = ScalarProgram::PointwiseF32(
            test_affine_expression(2.0_f32.to_bits(), 1.0_f32.to_bits()),
        );
        assert!(matches!(
            verify_schedule(
                wrong_expression,
                region.semantic_members().to_vec(),
                &request,
            ),
            Err(PhysicalError::Intrinsic {
                rule: "request-binding",
                ..
            })
        ));

        // A read addressed through a relation the recognition did not derive.
        // The region stays *intrinsically* well formed — a whole-extent
        // transposition of `[2, 2]` is an admissible pointwise map and its bounds
        // proof still bounds the same four elements — so nothing below the
        // subject binding can notice, and the tensor it computes is a
        // transposition of the one the program declared. Only comparing the read
        // list against the recognized one refuses it, which is why that
        // comparison exists.
        let mut forged_map = region.region().clone();
        forged_map.index.accesses[0].map = LogicalAccess::ReindexBijection {
            operand_shape: Shape::from_dims([2, 2]),
            result_shape: Shape::from_dims([2, 2]),
            axes: vec![
                tiler_ir::schedule::AxisDecode::read(1, 2),
                tiler_ir::schedule::AxisDecode::read(2, 2),
            ],
        };
        assert!(matches!(
            verify_schedule(forged_map, region.semantic_members().to_vec(), &request),
            Err(PhysicalError::Intrinsic {
                rule: "request-binding",
                ..
            })
        ));

        for forged in [
            expected[..3].to_vec(),
            vec![expected[1], expected[0], expected[2], expected[3]],
            vec![
                expected[0],
                expected[1],
                expected[2],
                expected[3],
                SemanticStage::first(SemanticMemberId(4)),
            ],
        ] {
            assert!(matches!(
                verify_schedule(region.region().clone(), forged, &request),
                Err(PhysicalError::Intrinsic {
                    rule: "request-binding",
                    ..
                })
            ));
        }
    }

    /// The same program as [`request`], under a contract that admits a split.
    ///
    /// Four contributors is the smallest extent [`governed_partition`] splits,
    /// and reassociation is the one permission the split consumes.
    fn split_request() -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, product, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let program = builder.build().unwrap();
        let request = verify_planned_request(CompilationRequest::governed_under(
            &program,
            StrictF32NumericalContract::governed_relaxed(),
        ))
        .unwrap();
        request.for_target(0).unwrap()
    }

    /// A split's two passes are told apart by the stage, not by the member set.
    ///
    /// **The one observation the attribution atom exists to make.** Both passes
    /// realize the *same* recognized fold occurrence, so as member sets their
    /// claims are `{reduction}` and `{reduction}` — indistinguishable, and one
    /// of the two would have to double-cover the graph. The spelling that
    /// survived under a bare-occurrence atom was for the combine to claim the
    /// empty set, which says it computes nothing at all: true of a publishing
    /// copy, false here, and equal to what a provider that claimed nothing by
    /// mistake would have said. With the stage in the atom each pass names the
    /// part of the occurrence it computes, and every other claim the member set
    /// alone could express is refused — including the empty one this check
    /// watched pass before the change.
    #[test]
    fn a_splits_two_passes_are_distinguished_by_stage_rather_than_member_set() {
        let request = split_request();
        let output = request.sole_output();
        let split = split_reduction_regions(&request, output, RegionWrite::ProgramOutput)
            .expect("a four-contributor relaxed request admits the split");
        let [(partial, partial_claim), (combine, combine_claim)] =
            <[_; 2]>::try_from(split.stages).expect("the governed split has two passes");

        // One occurrence, two stages: the member sets agree exactly, and only
        // the ordinal separates the claims.
        assert_eq!(partial_claim.len(), 1);
        assert_eq!(combine_claim.len(), 1);
        assert_eq!(partial_claim[0].member(), combine_claim[0].member());
        assert!(partial_claim[0].is_first());
        assert_eq!(combine_claim[0], partial_claim[0].next_stage());

        // Each pass binds under its own stage.
        verify_schedule(partial.clone(), partial_claim.clone(), &request)
            .expect("the partial pass claims the occurrence's first stage");
        verify_schedule(combine.clone(), combine_claim.clone(), &request)
            .expect("the combine claims the stage after it");

        // And under nothing else. The empty claim is the pre-atom spelling of
        // the combine; the other pass's claim is the only remaining set of this
        // occurrence a member-keyed model could name.
        for (region, forged) in [
            (&combine, Vec::new()),
            (&combine, partial_claim.clone()),
            (&partial, Vec::new()),
            (&partial, combine_claim.clone()),
        ] {
            assert_eq!(
                verify_schedule(region.clone(), forged, &request),
                Err(PhysicalError::Intrinsic {
                    rule: "request-binding",
                    region: region.index.id,
                }),
                "a pass bound against a claim that is not its own stage"
            );
        }
    }

    #[test]
    fn reduction_access_and_proof_shapes_are_bound_to_the_verified_request() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        let fused = build_fused_scheduled_region(&request).unwrap();

        for (mut forged, members) in [
            (
                regions[1].region().clone(),
                regions[1].semantic_members().to_vec(),
            ),
            (fused.region().clone(), fused.semantic_members().to_vec()),
        ] {
            let region = forged.index.id;
            let LogicalAccess::ReductionContributor { input_shape, .. } =
                &mut forged.index.accesses[0].map
            else {
                panic!("expected reduction access")
            };
            *input_shape = Shape::from_dims([2, 4]);
            let BoundsProofKind::ReductionDomain { input_shape, .. } =
                &mut forged.index.bounds_proofs[0].kind
            else {
                panic!("expected reduction proof")
            };
            *input_shape = Shape::from_dims([2, 4]);

            assert_eq!(
                verify_schedule(forged, members, &request),
                Err(PhysicalError::Intrinsic {
                    rule: "request-binding",
                    region,
                })
            );
        }
    }

    #[test]
    fn fused_schedule_rejects_numerical_corruption() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let scheduled = build_fused_scheduled_region(&request).unwrap();
        let mut invalid_schedule = scheduled.region().clone();
        let ScalarProgram::FusedMultiplyAddSerialSum { contraction, .. } =
            &mut invalid_schedule.index.scalar_program
        else {
            panic!("expected fused scalar program")
        };
        *contraction = true;
        assert_eq!(
            verify_schedule(
                invalid_schedule,
                scheduled.semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-or-access-refinement",
                region: RegionId::new(0),
            })
        );
    }

    #[test]
    fn malformed_axes_zero_launch_and_late_zero_products_fail_without_panicking() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let scheduled = build_fused_scheduled_region(&request).unwrap();

        let mut zero_threads = scheduled.region().clone();
        zero_threads.schedule.threads_per_workgroup = 0;
        zero_threads.schedule.launch.threads_per_workgroup = 0;
        assert!(matches!(
            verify_schedule(
                zero_threads,
                scheduled.semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "launch-coverage",
                ..
            })
        ));

        for axes in [vec![Axis::new(1), Axis::new(1)], vec![Axis::new(99)]] {
            let mut malformed = scheduled.region().clone();
            if let ScalarProgram::FusedMultiplyAddSerialSum {
                axes: scalar_axes, ..
            } = &mut malformed.index.scalar_program
            {
                *scalar_axes = axes.clone();
            }
            if let ReductionTopology::Serial {
                axes: schedule_axes,
                ..
            } = &mut malformed.schedule.reduction
            {
                *schedule_axes = axes.clone();
            }
            if let LogicalAccess::ReductionContributor {
                axes: access_axes, ..
            } = &mut malformed.index.accesses[0].map
            {
                *access_axes = axes.clone();
            }
            if let BoundsProofKind::ReductionDomain {
                axes: proof_axes, ..
            } = &mut malformed.index.bounds_proofs[0].kind
            {
                *proof_axes = axes;
            }
            assert!(matches!(
                verify_schedule(malformed, scheduled.semantic_members().to_vec(), &request),
                Err(PhysicalError::Intrinsic { .. })
            ));
        }
    }
}
