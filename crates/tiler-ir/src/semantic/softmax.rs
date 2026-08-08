//! The governed `f32` softmax, its two reductions, and its exponential's
//! accuracy contract.
//!
//! **Why one atomic key rather than a composition.** The operation is six
//! elementary steps — an extrema reduction, a subtraction, an exponential, a sum
//! reduction, a reciprocal, and a multiply — and the graph admits none of a
//! `Maximum` reduction, a general `Exp`, or a general `Divide` as a semantic key.
//! Registering the softmax as one key is what lets its *identity* carry the
//! maximum subtraction, the extrema family, the normalization form, the reduced
//! axis, and a resolved ADR 0042 accuracy contract for the one inexact step. A
//! composition would leave all five to whoever assembled it, and the derivation
//! measures three of them as observable in binary32.
//!
//! **Three decisions the usual spelling hides, and this key pins each.** From
//! `eager_attention_forward` at lines 157–162 of the pinned `modeling_qwen3.py`
//! (digest `704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2`):
//! the row maximum is subtracted before the exponential, which is finite against
//! NaN rather than a tolerance — `softmax([1000, 1000])` is a half each where the
//! naive quotient of exponentials is NaN; the normalization *multiplies by the
//! denominator's reciprocal* rather than dividing; and the reduced axis is the
//! last one, whose extent is the workload's only growing extent.
//!
//! **The normalization form is the pinned formula, not a permission.**
//! [`SOFTMAX_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED`] is `false` in the opposite
//! direction from its siblings': `tiler::silu-f32@1` and `tiler::rms-norm-f32@1`
//! pin a division and *withhold* the permission to turn it into a reciprocal
//! multiplication, while this key pins the multiplication itself and withholds
//! the permission to turn it back into `e_i / d`. Both spellings are conventional
//! and both look correct; the retained probe counts every discriminating element
//! at row widths two and three matching the reciprocal form and none matching the
//! division.
//!
//! **What the accuracy contract covers.** [`softmax_f32_exponential_accuracy_contract`]
//! resolves the accuracy of the *subordinate exponential only*. The maximum is a
//! selection rather than an approximation and is exact by construction; the
//! subtraction, the fold's adds, the division, and the multiplies each round once
//! under ADR 0024. So the exponential is the composition's one inexact element
//! and the only step with a tolerance to state. Its bound and its metric
//! translation are `tiler::silu-f32@1`'s, applied to a *narrower domain* — see
//! [`SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS`].
//!
//! # Decision D-2 — the extrema family, settled as `Maximum`
//!
//! **The two candidates are observationally indistinguishable through this
//! operation, and that is a theorem rather than a corpus result.** `Maximum`
//! propagates NaN and `MaximumNumber` prefers the numeric operand, so they differ
//! only on a row containing a NaN. On such a row, under *either* family, `e_i` at
//! the NaN position is `Exp(NaN - m)` and therefore NaN; the sum's `Add`
//! propagates that NaN unconditionally, so `d` is NaN, `c = 1.0 / d` is NaN, and
//! every `r_i = e_i * c` is NaN. The whole row is NaN either way, and no input
//! separates the two. `the_two_extrema_families_are_indistinguishable_through_the_pinned_formula`
//! in `crates/tiler-reference/src/softmax/tests.rs` is that argument, executed.
//!
//! **So the choice is made on grounds outside observable behaviour, and they are
//! stated rather than left implicit.** `Maximum` is pinned for three reasons.
//! First, it is the family whose NaN rule *agrees with the operation's own*: a
//! fact naming `MaximumNumber` would be true of the reduction in isolation while
//! inviting the reading that a NaN score does not poison its row, which is false.
//! Second, `MaximumNumber`'s equivalence here is a property of the *epilogue*
//! rather than of the reduction, so it would have to be re-derived by anything
//! that reused the construct without a downstream sum — the online single-pass
//! form and a log-sum-exp sibling are both such things. Third, ADR 0023 makes the
//! families separate operations and requires a reduction to name its scalar
//! family; between two families that cost the same here, the one that never
//! discards an input's NaN-ness is the fail-closed one.
//!
//! **Measurement — the reference's own maximum propagates, and it is now readable
//! from the retained record rather than re-measured.** In the retained probe's
//! pinned environment (`torch` 2.6.0, `transformers` 4.51.0, CPU, F32), `torch.max`
//! over `[1.0, NaN, 3.0]` is `0x7fc00000` and `softmax` of that row is three
//! canonical NaNs. The 2026-08-01 record carries both as
//! `torch_max_of_row_with_a_nan_score` and `softmax_row_with_a_nan_score` in
//! `spikes/numerics/transformer_reference_semantics/results/2026-08-01-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`,
//! so the NaN half of this measurement is read from the record.
//!
//! **Measurement — the reference's maximum does *not* implement the signed-zero
//! ordering, and the difference is an order dependence rather than a rule.** In
//! the same record, `torch.max` over `[+0.0, -0.0]` is `-0.0` (`0x80000000`) and
//! over `[-0.0, +0.0]` is `+0.0`, while `torch.amax` answers the other way on both
//! — the four `torch_max_of_signed_zeros_*` and `torch_amax_of_signed_zeros_*`
//! rows. Each spelling returns a fixed *position* rather than a fixed value, so
//! neither implements the `-0.0 < +0.0` total ordering that both Tiler families
//! share and that ADR 0023 requires. **Nothing in this decision rests on it**: the
//! three grounds above are about NaN, and the zero ordering is Tiler's own choice
//! rather than a reproduction of the reference model's — which is also why the
//! Metal lowering below needs a fixup for the same reason.
//!
//! **Neither family lowers to `air.fmax.f32`.** [Numerical
//! semantics](../../../../docs/numerical-semantics.md) records that Metal's
//! `fmax` is number-preferring *and* order-dependent in its signed-zero result,
//! so it implements neither Tiler family, and ADR 0023 requires an exact fixup.
//! `crates/tiler-metal/src/emit.rs` emits one, built from ordered comparisons
//! alone; see [`super::super::kernel::BinaryOp::F32Maximum`].
//!
//! # Decision D-1 — the fully masked row, settled as *no special case*
//!
//! **The question is about the mask convention, not about this key, and the
//! elimination is what shows it.** Under the workload's finite fill a fully
//! masked row is a row of equal finite values, which the pinned formula maps to a
//! uniform distribution with no branch taken; under a `-inf` fill it is a row of
//! `-inf`, whose `s_i - m` is `-inf - -inf` and therefore NaN, which the pinned
//! formula maps to a row of NaNs, also with no branch taken. Both are the *same*
//! formula on different inputs.
//!
//! Making either one a *rule* would mean detecting the other and repairing it,
//! and every detection route is eliminated:
//!
//! - **Refuse or repair at construction.** Construction sees shapes and
//!   attributes, never element values, so the predicate is not a fact the graph
//!   holds — the route decision **D-3** eliminated for the same reason.
//! - **Refuse on a proved value domain.** No program input carries an upper or
//!   lower bound on its scores, so the refusal would be unreachable rather than
//!   conservative.
//! - **Refuse after a runtime scan.** A scan is a costed second pass over
//!   448·`T`·`S` contributors per forward pass plus a validation mechanism the
//!   bounded profile does not have.
//! - **Name the fill value.** This route is specific to D-1 and it is the
//!   decisive one: the mask is an *external F32 program input* added upstream by
//!   a `tiler::add-f32@1` occurrence, so by the time the scores reach this key
//!   they are ordinary numbers and "fully masked" is not a predicate over this
//!   operation's operands at all. A refusal would require the key's identity to
//!   carry a constant that is not part of the operation.
//!
//! So the family states that it applies **no fully-masked repair** — which is a
//! decision rather than an omission, because the reference model *contains* such
//! a repair (`AttentionMaskConverter._unmask_unattended`) that is guarded to
//! `sdpa` on `cuda`/`xpu` and does not run on the path this workload takes.
//! Neither answer is order-dependent, so nothing here is refused for
//! order-dependence. Both are pinned in the conformance corpus, because the
//! measurement the L4 design added says the C1 row cannot falsify either.
//!
//! # What the two reductions may legally have done to them, which is not the same
//!
//! `Maximum` is associative and commutative on *every* binary32 input — NaN is
//! absorbing and the `-0.0 < +0.0` ordering is total — so any tree over the same
//! contributors gives the same bits and the maximum pass may be reassociated and
//! permuted with no permission at all. The sum is neither associative nor
//! commutative in binary32, so its pass moves only under the separately resolved
//! reassociation and permutation permissions. A legality check that let one
//! permission answer for both passes would be wrong in exactly one direction, and
//! [`SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY`] and
//! [`SOFTMAX_F32_FACT_SUM_FOLD_ORDER`] are two facts rather than one for that
//! reason.
//!
//! **The online single-pass form is *not* a reassociation, and the difference is
//! the whole legality question.** Rescaling a running sum whenever the maximum
//! changes yields a Horner nesting rather than a re-parenthesized sum: unrolled,
//! its contributors are `exp(x_j - m_j) * prod_{k>j} exp(m_{k-1} - m_k)`, which
//! share no binary32 value with the two-pass fold's `exp(x_j - m_V)`. No
//! contributor sequence has both folds as groupings and neither permutes to the
//! other, so **no reassociation permission and no permutation permission reaches
//! this rewrite** — the sum-fold permissions above answer a question it never
//! asks. What reaching the nesting from the sum does consume is *distributivity*,
//! because each rescale factor multiplies through a partial sum: [ADR
//! 0080](../../../../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md)
//! names that dimension and [ADR
//! 0095](../../../../docs/decisions/0095-decline-a-distributivity-permission.md)
//! declines a permission for it. The telescoping step
//! `exp(x_j - m_j) * exp(m_j - m_V) = exp(x_j - m_V)` consumes a second freedom,
//! the exponential's own functional equation, which is false in binary32 and which
//! no declared dimension names at all. [The certified-bounds
//! record](../../../../docs/research/numerics/certified-bounds-as-rewrite-permissions.md)
//! derives both, and [the elementary-identity
//! record](../../../../docs/research/numerics/elementary-identity-rewrite-dimension.md)
//! is where the second one is named.
//!
//! [`SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM`] states **both** freedoms, so that a
//! scheduler reaching for the form cannot spend a permission that does not reach
//! it. Naming one of two missing freedoms would imply that granting that one
//! suffices, which is the false inference ADR 0080 item 5 exists to prevent.

use std::sync::Arc;

use super::accuracy::{
    AccuracyContract, AccuracyContractForm, AccuracyDomain, AccuracyDomainClause,
    AccuracyPredicate, DomainBound, DomainErrorRule, DomainInterval, ExactRational, ExactTolerance,
    ExceptionalValueContract, FiniteOverflowRule, InfiniteReferenceRule, NanReferenceRule,
    OperandOrdinal, ReferenceResultClass, ReferenceResultConstraint, ulp_reference_gap_metric_key,
};
use super::registry::standard_conformance;
use super::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueKind, CanonicalValueView, F32,
    NormativeDefinitionRef, OpKey, OperationArity, OperationAttributeSchema, OperationDefinition,
    OperationDefinitionFacts, OperationEffect, OperationInferenceError, OperationInferenceOutputs,
    OperationInferenceRequest, OperationInferencer, OperationSchema, ProviderDiagnosticCode,
    RegistryError, SemanticRegistryRegistrar, ValueFact,
};
use crate::shape::Axis;

/// Attribute carrying the single reduced axis, as a one-element sequence.
///
/// A sequence rather than a bare integer, for the reason
/// [`super::RMS_NORM_REDUCED_AXES_ATTRIBUTE`] states and this family inherits:
/// spelling the attribute as a sequence is what lets the inferencer refuse an
/// *absent* axis, a *duplicated* axis, and a *second* axis by three different
/// named rules instead of making two of the three unstatable.
pub const SOFTMAX_REDUCED_AXES_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// Exact binary32 payload closing the subordinate exponential's ordinary domain.
///
/// **`+0.0`, and the value is the maximum subtraction's whole payoff written as a
/// contract.** Every argument the exponential receives is `s_i - m` where `m` is
/// the maximum of the same row, so the exact difference is never positive and
/// round-to-nearest of a non-positive real is non-positive. The domain therefore
/// closes at zero, one binary32 argument above the largest the operation can
/// reach, and the [`FiniteOverflowRule`] below it is *vacuous* rather than
/// absent: `e^t <= 1` on the whole admitted domain, so no finite reference of
/// this exponential can leave binary32's finite range.
///
/// This is the one field in which this family's exponential contract differs from
/// `tiler::silu-f32@1`'s, whose ceiling is
/// [`super::SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS`] because that activation
/// genuinely reaches the overflow band. Same metric, same tolerance, same
/// registered cross-metric implication, narrower region.
pub const SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS: u32 = 0x0000_0000;

/// ULP tolerance the subordinate exponential's resolved contract states.
///
/// **Twelve, and it is the same *derivation* as `tiler::silu-f32@1`'s rather than
/// a citation of that constant.** Metal's Table 8.1 gives `exp <= 4 ulp` under
/// Apple's own definition of `ulp`, and the conservative factor covering both
/// admissible readings of Apple's ambiguous second clause is three, so the
/// translated bound is `4 * 3 = 12`. The bound is a property of *Metal's
/// exponential*, not of either operation that calls one.
///
/// It is declared here rather than read off the activation's constant because the
/// two are different operations' contracts, and one moving for a reason of its
/// own must not silently move the other.
/// `the_two_exponential_tolerances_agree_because_the_derivation_is_one` asserts
/// the equality, so a divergence has to be deliberate rather than accidental.
pub const SOFTMAX_F32_EXPONENTIAL_ULP_TOLERANCE: u64 = 12;

/// Fact field naming the type this operation's arithmetic is performed at.
pub const SOFTMAX_F32_FACT_COMPUTATION_TYPE: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming the result value type.
pub const SOFTMAX_F32_FACT_RESULT_TYPE: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming the exact spelling and its rounding boundaries.
///
/// The *order* is part of the operation. Two orderings a reader would call the
/// same softmax are different binary32 functions: omitting the maximum
/// subtraction turns an overflowing row from a finite distribution into NaNs, and
/// dividing by the denominator instead of multiplying by its reciprocal disagrees
/// at every discriminating element the retained probe counted.
pub const SOFTMAX_F32_FACT_EVALUATION_ORDER: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field carrying the complete resolved accuracy contract of the exponential.
///
/// The canonical value of [`softmax_f32_exponential_accuracy_contract`], written
/// into the definition's facts so that ADR 0016's requirement — transcendental
/// accuracy participates in semantic, plan, artifact, reference, and explain
/// identity — is satisfied by the registry's own definition projection rather
/// than by a second authority beside it.
pub const SOFTMAX_F32_FACT_EXPONENTIAL_ACCURACY_CONTRACT: AttributeFieldId =
    AttributeFieldId::new(4);
/// Fact field naming the extrema family of the row maximum, which is decision **D-2**.
///
/// `Maximum`, the NaN-propagating IEEE 754-2019 family with the deterministic
/// `-0.0 < +0.0` ordering both Tiler extrema families share. The module header
/// carries the elimination and the theorem behind it: the alternative is
/// observationally indistinguishable through this operation, so the choice rests
/// on the reduction's own statability rather than on a corpus.
pub const SOFTMAX_F32_FACT_MAXIMUM_EXTREMA_FAMILY: AttributeFieldId = AttributeFieldId::new(5);
/// Fact field naming what a schedule may legally do to the maximum pass.
///
/// Free reassociation and free permutation, consuming *no* permission, because
/// the pinned extrema family is associative and commutative on every binary32
/// input. Stated as its own fact beside [`SOFTMAX_F32_FACT_SUM_FOLD_ORDER`]
/// because the two passes answer differently and one permission must not be read
/// as covering both.
pub const SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY: AttributeFieldId = AttributeFieldId::new(6);
/// Fact field naming the denominator sum's contributor order.
///
/// The strict left fold over the canonical contributor sequence, seeded at the
/// first contributor — the same order `tiler::strict-serial-sum-f32@1` declares.
/// Neither associative nor commutative in binary32, so this pass moves only under
/// the separately resolved reassociation and permutation permissions.
pub const SOFTMAX_F32_FACT_SUM_FOLD_ORDER: AttributeFieldId = AttributeFieldId::new(7);
/// Fact field naming the type the denominator sum accumulates at.
///
/// Explicit, and **not** inherited from the element type, discharging decision
/// **D-5** for the second of the two sums the L3′ derivation identified. The
/// value is `tiler::f32@1` because F32 is what reproduces the pinned reference;
/// whether a growing context justifies widening stays
/// [`implement-parallel-reduction-strategies`](../../../../tickets/implement-parallel-reduction-strategies.md)'s
/// question. The maximum reduction carries no accumulator declaration of its own
/// because it performs no arithmetic: it selects one of its contributors' bit
/// patterns, so there is no width at which it could accumulate differently.
pub const SOFTMAX_F32_FACT_ACCUMULATOR_TYPE: AttributeFieldId = AttributeFieldId::new(8);
/// Fact field naming how the row is normalized.
///
/// A reciprocal multiplication, never a division, which is the reverse of what
/// the siblings pin. See the module header.
pub const SOFTMAX_F32_FACT_NORMALIZATION_FORM: AttributeFieldId = AttributeFieldId::new(9);
/// Fact field naming what the outputs of one row sum to.
///
/// **They do not sum to exactly one, and no check may assert that they do.**
/// `softmax([0.0, 2.0])` is `0x3df420a8` and `0x3f617bea`, whose strict sum is
/// `0x3f7fffff`; `softmax([0.0, 1.0, 0.0])` sums to `0x3f800001`. The deviation
/// goes in both directions, so even a one-sided check would be wrong.
pub const SOFTMAX_F32_FACT_ROW_SUM: AttributeFieldId = AttributeFieldId::new(10);
/// Fact field naming the behaviour of a row every element of which is a mask fill,
/// which is decision **D-1**.
///
/// No special case and no repair: the pinned formula decides, and what it decides
/// depends on the caller's mask convention rather than on this key. The module
/// header carries the four-route elimination.
pub const SOFTMAX_F32_FACT_FULLY_MASKED_ROW: AttributeFieldId = AttributeFieldId::new(11);
/// Fact field naming what a zero-length reduced axis produces.
///
/// A zero-length output, with no scalar softmax evaluated. Softmax is
/// shape-preserving rather than shape-reducing, so the reduction contract's
/// empty-domain rules do not reach it — and the fact says so rather than letting
/// them appear to apply. The distinction is load-bearing here in a way it is not
/// for `tiler::rms-norm-f32@1`: the embedded maximum is an *identity-less*
/// reduction, so an empty contributor domain would have no value to commit, and
/// the shape rule is what makes that case unreachable rather than merely
/// undefined.
pub const SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS: AttributeFieldId = AttributeFieldId::new(12);
/// Fact field naming what the online single-pass realization consumes.
///
/// Distributivity and the subordinate exponential's own functional equation —
/// never reassociation. Fusing the maximum and the sum into one pass over the row
/// means rescaling a running sum whenever the maximum changes, and the result is a
/// Horner nesting whose contributors share no binary32 value with the two-pass
/// fold's, so neither the reassociation nor the permutation permission reaches it.
/// Both consumed freedoms are named, because naming one of two would imply that
/// granting that one admits the rewrite. The module header carries the derivation
/// and the two records it is read from.
pub const SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM: AttributeFieldId = AttributeFieldId::new(13);
/// Fact field naming the operation's behaviour on binary32 subnormals.
///
/// **Measurement — this operation reaches a subnormal on an ordinary input, and
/// the reference and the qualified Metal row disagree there.** A contributor
/// about 87 below the row maximum has an exponential of `0x00b33687`, a subnormal,
/// and one about 104 below has exactly `+0.0`. On a target that flushes results
/// the subnormal band collapses to zero. Under ADR 0076 that is a declared
/// realization difference, not a defect to be tuned away.
pub const SOFTMAX_F32_FACT_SUBNORMALS: AttributeFieldId = AttributeFieldId::new(14);
/// Fact field naming the operation's behaviour on signed zero.
pub const SOFTMAX_F32_FACT_SIGNED_ZERO: AttributeFieldId = AttributeFieldId::new(15);
/// Fact field naming the operation's NaN behaviour.
pub const SOFTMAX_F32_FACT_NAN_BEHAVIOUR: AttributeFieldId = AttributeFieldId::new(16);
/// Fact field carrying the canonical arithmetic-NaN payload this operation installs.
pub const SOFTMAX_F32_FACT_CANONICAL_NAN_BITS: AttributeFieldId = AttributeFieldId::new(17);
/// Fact field stating whether ADR 0015's arithmetic contraction is permitted.
///
/// `false`. The only multiply-add adjacency in the whole operation is the
/// maximum subtraction, whose multiply is an exact sign flip — so fusing it would
/// remove a rounding that never happened. The permission is withheld rather than
/// granted-because-inert, because a granted permission is a statement a target
/// would be asked about.
pub const SOFTMAX_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(18);
/// Fact field stating whether the reciprocal multiplication may become a division.
///
/// `false`, and it is a *withheld permission* rather than an absent one, in the
/// opposite direction from the siblings'. `e_i * (1 / d)` rounds the reciprocal
/// once and the product once; `e_i / d` rounds once in total, and the retained
/// probe measures the two disagreeing at every discriminating element of a
/// width-two or width-three row.
pub const SOFTMAX_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(19);
/// Fact field stating whether an approximate elementary intrinsic may realize the exponential.
///
/// `false`. The resolved accuracy contract is what the exponential must satisfy,
/// and an approximate intrinsic is admissible only through a stated envelope with
/// conformance evidence that it refines that contract.
pub const SOFTMAX_F32_FACT_APPROXIMATE_INTRINSIC_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(20);

/// Returns the governed binary32 softmax operation key.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn softmax_f32_op() -> OpKey {
    OpKey::new("tiler", "softmax-f32", 1).expect("the governed softmax operation key is valid")
}

/// Returns the immutable reference semantics this key pins.
///
/// Names the extrema family, the maximum subtraction, the fold order, and the
/// reciprocal multiplication explicitly, because a reference that said only "the
/// softmax" would admit the division spelling and the no-subtraction spelling
/// that both differ from it observably.
///
/// # Panics
///
/// Panics only if this crate's own compile-time reference text violates the
/// canonical bound registration would reject it under.
#[must_use]
pub fn softmax_f32_reference_semantics() -> NormativeDefinitionRef {
    NormativeDefinitionRef::new(
        "tiler::softmax-f32@1; over the single reduced axis, in this exact order: m = the strict \
         left fold of the NaN-propagating Maximum extrema family over the canonical contributor \
         sequence seeded at the first contributor, ordering -0.0 < +0.0 and deliberately not \
         MaximumNumber; then e_i = Exp(s_i - m), the subtraction rounding once under \
         round-to-nearest ties-to-even (ADR 0024) and the exponential the one inexact step; then \
         d = the strict left fold sum of e over the same sequence seeded at the first contributor; \
         then c = 1.0 / d as one division of one by the denominator; then r_i = e_i * c as a \
         multiplication by that reciprocal and deliberately not e_i / d, which is a different \
         binary32 function; the operation is shape-preserving, so a zero-length reduced axis \
         yields a zero-length output and evaluates no scalar softmax; the outputs of a row do not \
         sum to exactly 1.0",
    )
    .expect("the governed softmax reference semantics are canonical")
}

/// Returns the immutable reference semantics of the subordinate exponential.
///
/// A separate reference from the softmax's own, because the accuracy contract
/// resolves this one step and a contract whose reference named the whole softmax
/// would be stating a tolerance over a composition whose every other step ADR
/// 0024 already fixes exactly.
///
/// It also names the *argument's* provenance, which the activation's does not
/// need to: the maximum subtraction is what confines the argument to `(-inf, 0]`
/// and therefore what makes this contract's domain narrower than
/// `tiler::silu-f32@1`'s.
///
/// # Panics
///
/// Panics only if this crate's own compile-time reference text violates the
/// canonical bound.
#[must_use]
pub fn softmax_f32_exponential_reference_semantics() -> NormativeDefinitionRef {
    NormativeDefinitionRef::new(
        "the natural exponential e^t on the reals, evaluated at t = s_i - m where m is the row \
         maximum, as the one inexact subordinate step of tiler::softmax-f32@1; the maximum \
         subtraction confines t to the non-positive reals; this is not a registered general Exp \
         operation and mints no key of its own",
    )
    .expect("the governed subordinate exponential reference is canonical")
}

/// Returns the resolved ADR 0042 accuracy contract of the subordinate exponential.
///
/// # What this reuses, and what it necessarily does not
///
/// **The bound, the metric, and the cross-metric translation are shared with
/// `tiler::silu-f32@1`; the contract instance is not, and cannot be.** An
/// [`AccuracyContract`] carries the [`OpKey`] it speaks about, and
/// `assess_elementary_accuracy` matches an installed realization against a
/// requirement by that key before it compares anything else — so a contract
/// naming the activation would be an identity for a different operation and would
/// simply never be consulted for this one. What *is* reused is the machinery: one
/// `apple::msl-ulp@1` metric key, one registered
/// `RegisteredImplication::ScaledMetric` with factor three, and one certified
/// exponential in `tiler-reference`. This vertical installs no second implication
/// row, and `the_softmax_needs_no_second_registered_implication` proves it by
/// admitting this contract against a registry carrying exactly the activation's
/// row.
///
/// # The one field that differs, and why it is a derivation
///
/// The domain closes at [`SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS`], which
/// is `+0.0`, where the activation's closes at the last argument whose
/// exponential is finite. The narrowing is not conservatism: it is the region the
/// operation can actually reach, and stating a wider one would place an
/// obligation on arguments the maximum subtraction makes unreachable.
///
/// # Panics
///
/// Panics only if this crate's own compile-time contract violates the grammar its
/// own vocabulary defines, which registration would reject as well.
#[must_use]
pub fn softmax_f32_exponential_accuracy_contract() -> AccuracyContract {
    let ceiling = ExactRational::from_f32(f32::from_bits(
        SOFTMAX_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS,
    ))
    .expect("the governed exponential ceiling is a finite binary32 value");
    let ordinary = DomainInterval::new(
        OperandOrdinal::new(0),
        DomainBound::Unbounded,
        DomainBound::Closed(ceiling),
    )
    .expect("the governed exponential domain admits every non-positive argument");
    let clause = AccuracyDomainClause::new(
        [(OperandOrdinal::new(0), ordinary.clone())],
        ReferenceResultConstraint::new(
            [ReferenceResultClass::Positive],
            None,
            Some(
                NormativeDefinitionRef::new(
                    "e^t is strictly positive at every real t, so the reference result is never \
                     zero and never negative on this clause's whole region",
                )
                .expect("the governed positivity justification is canonical"),
            ),
        )
        .expect("the governed reference-result constraint is canonical"),
        AccuracyPredicate::ulp(
            ulp_reference_gap_metric_key(),
            ExactTolerance::from_integer(SOFTMAX_F32_EXPONENTIAL_ULP_TOLERANCE),
        ),
    )
    .expect("the governed exponential clause is canonical");
    AccuracyContract::new(
        softmax_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        softmax_f32_exponential_reference_semantics(),
        AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new([ordinary], [clause])
                .expect("the governed exponential domain is canonical"),
        ),
        softmax_f32_exponential_exceptional_contract(),
    )
}

/// Returns the subordinate exponential's independent exceptional-value contract.
///
/// Stated separately from the error metric and from the softmax's own exceptional
/// behaviour, because ADR 0042 makes those three different claims and `refines`
/// refuses outright when two contracts state different ones — so a realization
/// must reproduce this record exactly rather than approximate it.
///
/// - a NaN argument has a NaN reference, and the operation installs its canonical
///   arithmetic NaN. This is the rule a poisoned row travels: `s_i - m` is NaN
///   whenever either operand is;
/// - an infinite reference is the infinity of its own sign. Unreachable on the
///   admitted domain, where `e^t <= 1`, and stated because the contract is a claim
///   about the function rather than about which of its inputs this workload
///   reaches;
/// - the admitted ordinary domain is bounded above at zero, and ADR 0042 routes an
///   argument beyond it to the *finite overflow* rule rather than to
///   [`DomainErrorRule`], so the domain rule governs only arguments the operand
///   type cannot produce;
/// - a finite reference above binary32's finite range yields the infinity of its
///   sign. Likewise vacuous on this domain, and stated rather than omitted so that
///   `refines` can compare this contract against a realization that states it.
#[must_use]
pub const fn softmax_f32_exponential_exceptional_contract() -> ExceptionalValueContract {
    ExceptionalValueContract::new(
        NanReferenceRule::CanonicalNan,
        InfiniteReferenceRule::SignedInfinity,
        DomainErrorRule::CanonicalNan,
        FiniteOverflowRule::SignedInfinity,
    )
}

/// Returns the exact fact record the governed softmax definition carries.
///
/// Built by the same constructor the registration uses rather than restated, so a
/// consumer parameterizing itself on the declared record and the registered
/// definition cannot disagree about what was declared.
///
/// # Panics
///
/// Panics only if this crate's own compile-time fact record violates the
/// canonical value grammar, which registration would reject as well.
#[must_use]
pub fn softmax_f32_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(SOFTMAX_F32_FACT_COMPUTATION_TYPE, f32_value_type()),
        CanonicalField::new(SOFTMAX_F32_FACT_RESULT_TYPE, f32_value_type()),
        CanonicalField::new(
            SOFTMAX_F32_FACT_EVALUATION_ORDER,
            fact(
                "maximum-then-subtract-then-exp-then-sum-then-reciprocal-then-multiply; \
                 the-subtraction-and-every-combine-and-the-reciprocal-and-every-scale-round-once-\
                 under-ties-to-even-and-the-exponential-is-the-one-inexact-step",
            ),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_EXPONENTIAL_ACCURACY_CONTRACT,
            softmax_f32_exponential_accuracy_contract()
                .to_canonical_value()
                .expect("the governed softmax accuracy contract is canonical"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_MAXIMUM_EXTREMA_FAMILY,
            fact("maximum-nan-propagating-with-negative-zero-below-positive-zero-not-maximum-number"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY,
            fact("reassociation-and-permutation-free-and-consuming-no-permission-because-the-family-is-associative-and-commutative-on-every-input"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_SUM_FOLD_ORDER,
            fact("strict-left-fold-over-the-canonical-contributor-sequence-seeded-at-the-first-contributor-moving-only-under-the-separate-reassociation-and-permutation-permissions"),
        ),
        CanonicalField::new(SOFTMAX_F32_FACT_ACCUMULATOR_TYPE, fact("tiler::f32@1")),
        CanonicalField::new(
            SOFTMAX_F32_FACT_NORMALIZATION_FORM,
            fact("multiply-by-the-denominators-reciprocal-never-divide-by-the-denominator"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_ROW_SUM,
            fact("not-exactly-one-and-deviating-in-both-directions-no-check-may-assert-a-unit-row-sum"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_FULLY_MASKED_ROW,
            fact("no-special-case-and-no-repair-the-pinned-formula-decides-uniform-under-a-finite-fill-and-nan-under-an-infinite-one"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS,
            fact("zero-length-output-with-no-scalar-softmax-evaluated-outside-the-reduction-empty-domain-rules"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM,
            fact(
                "not-a-reassociation-of-the-sum-but-a-horner-nesting-consuming-distributivity-\
                 which-no-permission-grants-and-the-subordinate-exponentials-elementary-function-\
                 identity-which-no-declared-dimension-names-so-no-reassociation-or-permutation-\
                 permission-reaches-it",
            ),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_SUBNORMALS,
            fact("preserved-by-this-contract-and-reached-on-an-ordinary-row-and-flushed-on-a-declared-flushing-realization-a-recorded-divergence"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_SIGNED_ZERO,
            fact("ieee-754-signed-zero-rules-with-the-maximum-ordering-negative-zero-below-positive-zero"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_NAN_BEHAVIOUR,
            fact("quiet-nan-propagates-through-both-folds-and-poisons-the-whole-row-and-every-arithmetic-nan-result-is-canonicalized"),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_CANONICAL_NAN_BITS,
            super::registry::canonical_f32_bits(super::CANONICAL_F32_ARITHMETIC_NAN_BITS),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            SOFTMAX_F32_FACT_APPROXIMATE_INTRINSIC_PERMITTED,
            CanonicalValue::boolean(false),
        ),
    ])
    .expect("the governed softmax facts are canonical")
}

/// Returns the reduced-axis attribute value for one softmax axis.
///
/// # Panics
///
/// Panics only if a one-element sequence violates the canonical value grammar.
#[must_use]
pub fn softmax_f32_axis_attribute(axis: Axis) -> CanonicalValue {
    CanonicalValue::sequence([CanonicalValue::unsigned_u32(axis.get())])
        .expect("a one-element axis sequence is canonical")
}

pub(super) fn register_standard_softmax(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        softmax_f32_op(),
        OperationSchema::new(
            OperationArity::exact(1),
            OperationArity::exact(1),
            [OperationAttributeSchema::required(
                SOFTMAX_REDUCED_AXES_ATTRIBUTE,
                CanonicalValueKind::Sequence,
            )],
        )
        .expect("the governed softmax operation schema is valid"),
        softmax_f32_reference_semantics(),
        OperationDefinitionFacts::new(softmax_f32_facts()),
        standard_conformance("softmax-f32"),
        OperationEffect::Pure,
        Arc::new(SoftmaxF32),
    ))
    // No algebraic capability is declared, and the absence is derived rather than
    // deferred. The softmax is neither associative nor commutative — it is unary
    // over a whole row — and the ordered-associativity law `tiler::add-f32@1`
    // declares is about the *embedded* denominator sum rather than about this
    // operation. The embedded maximum's own associativity is stated in the
    // definition facts, where a legality reasoner reads it, rather than as an
    // algebraic capability of the softmax itself. Declaring nothing here reads as
    // unknown rather than as the inverse law.
}

struct SoftmaxF32;

impl OperationInferencer for SoftmaxF32 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let [input] = operands else {
            return Err(op_error(
                "softmax.f32.arity",
                "the binary32 softmax requires exactly one operand, the score tensor",
            ));
        };
        let expected = F32::resolved_type();
        if input.resolved_type() != &expected {
            return Err(op_error(
                "softmax.f32.implicit-promotion",
                "the binary32 softmax admits no implicit promotion; an operand of another type is \
                 not converted to tiler::f32@1",
            ));
        }
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "softmax.f32.attributes",
                "the binary32 softmax requires exactly the reduced-axis attribute",
            ));
        }
        // The axis is resolved for its refusals; the result shape does not depend
        // on which axis was named, because the operation is shape-preserving: the
        // reduced axis is folded over twice and then restored. A contract that
        // dropped the axis would be a reduction, which this family deliberately
        // does not admit.
        //
        // A symbolic operand is declined by name rather than carried through.
        // The *shape* rule would survive one — the result is the operand's own
        // boundary, whatever its extents are — but this family's normative
        // definition, its reference evaluation, and its numerical conformance
        // are all stated over a fixed reduced extent, and admitting a boundary
        // none of them can evaluate would move the refusal downstream to a place
        // that has no name for it.
        let input = request.static_operand_shape(0)?;
        let _axis = reduced_axis(&request, input.rank())?;
        outputs.try_push(ValueFact::new(expected, input.clone()))
    }
}

/// Resolves the single reduced axis, naming the violated rule on refusal.
fn reduced_axis(
    request: &OperationInferenceRequest<'_>,
    rank: usize,
) -> Result<Axis, OperationInferenceError> {
    let Some(CanonicalValueView::Sequence(values)) = request
        .attributes()
        .get(SOFTMAX_REDUCED_AXES_ATTRIBUTE)
        .map(CanonicalValue::view)
    else {
        return Err(op_error(
            "softmax.f32.axis.kind",
            "the softmax's reduced-axis attribute must be a sequence of u32 axes",
        ));
    };
    if values.is_empty() {
        return Err(op_error(
            "softmax.f32.axis.absent",
            "the softmax requires a reduced axis; the reduced-axis sequence is empty",
        ));
    }
    let mut named: Vec<Axis> = Vec::with_capacity(values.len());
    for value in values {
        let CanonicalValueView::Unsigned { width, bits } = value.view() else {
            return Err(op_error(
                "softmax.f32.axis.type",
                "a softmax reduced axis must be an unsigned integer",
            ));
        };
        if width != super::CanonicalIntegerWidth::Bits32 {
            return Err(op_error(
                "softmax.f32.axis.width",
                "a softmax reduced axis must use u32",
            ));
        }
        let axis = Axis::new(u32::try_from(bits).map_err(|_| {
            op_error(
                "softmax.f32.axis.width",
                "a softmax reduced axis exceeds u32",
            )
        })?);
        if named.contains(&axis) {
            return Err(op_error(
                "softmax.f32.axis.duplicated",
                "a softmax reduced axis is named more than once; the reduced-axis sequence must be \
                 strictly ascending",
            ));
        }
        named.push(axis);
    }
    let [only] = named.as_slice() else {
        return Err(op_error(
            "softmax.f32.axis.rank",
            "the softmax family reduces over exactly one axis; a multi-axis softmax is a different \
             operation and is not admitted here",
        ));
    };
    if usize::try_from(only.get()).is_ok_and(|position| position < rank) {
        Ok(*only)
    } else {
        Err(op_error(
            "softmax.f32.axis.range",
            "the softmax's reduced axis is out of range for the operand's rank",
        ))
    }
}

fn f32_value_type() -> CanonicalValue {
    CanonicalValue::value_type(F32::resolved_type())
}

fn fact(value: &'static str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("the governed softmax fact is bounded")
}

fn op_error(code: &str, message: &str) -> OperationInferenceError {
    OperationInferenceError::new(
        ProviderDiagnosticCode::new(code)
            .expect("the governed softmax diagnostic code is canonical"),
        message,
    )
    .expect("the governed softmax diagnostic message is canonical")
}

#[cfg(test)]
mod tests;
