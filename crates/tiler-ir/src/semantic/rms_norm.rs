//! The governed `f32` root-mean-square normalization, and its reciprocal
//! square root's accuracy contract.
//!
//! **Why one atomic key rather than a composition.** The operation is eight
//! elementary steps — square, sum, divide by the extent, add `eps`, reciprocal
//! square root, scale, and weight — and the graph admits none of `Rsqrt`, a
//! general mean, or a sum carrying a prologue as a semantic key. Registering the
//! normalization as one key is what lets its *identity* carry the exact `eps`
//! bits, the reduced axis, and a resolved ADR 0042 accuracy contract for the one
//! inexact step. A composition would leave all three to whoever assembled it.
//!
//! **Three decisions the usual spelling hides, and this key pins each.** From
//! `Qwen3RMSNorm.forward` at lines 74–76 of the pinned `modeling_qwen3.py`
//! (digest `704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2`):
//! nothing is subtracted, so this is root-mean-square normalization and **not**
//! layer normalization; `eps` is added to the mean of squares *inside* the
//! reciprocal square root's argument rather than to the root outside it; and the
//! operation is `rsqrt`, not `1 / sqrt`, which are different binary32 functions.
//! [`rms_norm_f32_reference_semantics`] states all three, and
//! [`RMS_NORM_F32_FACT_EVALUATION_ORDER`] states the rounding boundaries.
//!
//! **`eps` is a semantic term and not a guard, so it is part of identity.** The
//! workload's `rms_norm_eps` is `1e-06`, whose binary32 rounding is
//! [`RMS_NORM_F32_REFERENCE_EPS_BITS`] and which is not exactly representable.
//! Measurement from the [reference-semantics
//! probe](../../../../spikes/numerics/transformer_reference_semantics/README.md):
//! removing `eps` turns a zero row into NaNs and a subnormal row into
//! infinities, and it changes the result at an ordinary input as well — so it
//! perturbs every output rather than activating near zero. Two normalizations
//! differing only in that constant are therefore different operations, and the
//! attribute carries the exact bits so they cannot share an identity, a cache
//! subject, or a golden.
//!
//! **What the accuracy contract covers.** [`rms_norm_f32_rsqrt_accuracy_contract`]
//! resolves the accuracy of the *subordinate reciprocal square root only*. Every
//! other step — the squares, the fold, the division by the extent, the `eps`
//! addition, and the two multiplies — is binary32 arithmetic whose result ADR
//! 0024 already fixes exactly, so the reciprocal square root is the composition's
//! one inexact element and the only step with a tolerance to state.
//!
//! **The division by the extent is written as a division.** Both of the
//! workload's extents are powers of two, so multiplying by `1 / N` would be an
//! exact scaling of the exponent *there*; at a non-power-of-two extent `1 / N` is
//! itself rounded and the product rounds again, so a contract that pinned the
//! reciprocal form would silently acquire a rounding the division does not have.
//! The exactness at 1024 and 128 is a derived property of those extents, not part
//! of the definition, and [`RMS_NORM_F32_FACT_EXTENT_DIVISION`] says so.

use std::sync::Arc;

use super::accuracy::{
    AccuracyContract, AccuracyContractForm, DomainBound, DomainErrorRule, DomainInterval,
    ExactRational, ExceptionalValueContract, FiniteOverflowRule, InfiniteReferenceRule,
    NanReferenceRule, OperandOrdinal,
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

/// Attribute carrying the single normalized axis, as a one-element sequence.
///
/// A sequence rather than a bare integer, and the redundancy is deliberate: the
/// family normalizes over exactly one axis, and spelling the attribute as a
/// sequence is what lets the inferencer refuse an *absent* axis, a *duplicated*
/// axis, and a *second* axis by three different named rules instead of making
/// two of the three unstatable. A bare integer would make "duplicated" a shape
/// the attribute could not express and therefore a refusal nothing could reach.
pub const RMS_NORM_REDUCED_AXES_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// Attribute carrying the exact binary32 payload of `eps`.
///
/// `FloatBits` rather than an unsigned integer, so the attribute states the
/// format its payload is read under. The value is part of the operation's
/// identity: see this module's header for why two normalizations differing only
/// here are different operations.
pub const RMS_NORM_EPS_BITS_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(2);

/// Exact binary32 payload of the pinned workload's `rms_norm_eps`.
///
/// `1e-06` is not exactly representable in binary32; this is its rounding,
/// approximately `9.999999975e-07`. Written as a bit pattern because the
/// operation's identity carries the payload rather than the decimal literal that
/// produced it, and because a reader comparing two occurrences compares these
/// bits.
///
/// It is a *constant of the workload*, not a default: the schema has no default
/// and [`rms_norm_f32_eps_attribute`] takes the payload from its caller.
pub const RMS_NORM_F32_REFERENCE_EPS_BITS: u32 = 0x3586_37bd;

/// Exact binary32 payload above which squaring overflows to infinity.
///
/// The largest binary32 whose square is finite. **Measurement**, from the
/// reference-semantics probe: `sqrt(f32::MAX)` rounds to this value, and a row
/// whose elements reach it produces a mean of squares of `+inf`, a reciprocal
/// square root of `+0.0`, and a row of signed zeros. That is the silent-wrongness
/// case decision **D-3** names, and [`RMS_NORM_F32_FACT_SQUARING_OVERFLOW`]
/// records what this family decided about it.
pub const RMS_NORM_F32_SQUARING_OVERFLOW_BITS: u32 = 0x5f7f_ffff;

/// Fact field naming the type this operation's arithmetic is performed at.
pub const RMS_NORM_F32_FACT_COMPUTATION_TYPE: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming the result value type.
pub const RMS_NORM_F32_FACT_RESULT_TYPE: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming the exact spelling and its rounding boundaries.
///
/// The *order* is part of the operation. Three orderings a reader would call the
/// same normalization are different binary32 functions: `eps` outside the root
/// rather than inside its argument, `1 / sqrt` rather than `rsqrt`, and the
/// weight multiply before the conversion back rather than after it.
pub const RMS_NORM_F32_FACT_EVALUATION_ORDER: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field carrying the complete resolved accuracy contract of the reciprocal square root.
///
/// The canonical value of [`rms_norm_f32_rsqrt_accuracy_contract`], written into
/// the definition's facts so that ADR 0016's requirement — transcendental
/// accuracy participates in semantic, plan, artifact, reference, and explain
/// identity — is satisfied by the registry's own definition projection rather
/// than by a second authority beside it.
pub const RMS_NORM_F32_FACT_RSQRT_ACCURACY_CONTRACT: AttributeFieldId = AttributeFieldId::new(4);
/// Fact field naming the embedded reduction's contributor order.
pub const RMS_NORM_F32_FACT_FOLD_ORDER: AttributeFieldId = AttributeFieldId::new(5);
/// Fact field naming the type the embedded reduction accumulates at.
///
/// Explicit, and **not** inherited from the element type. Criterion 3 of
/// [`implement-parallel-reduction-strategies`](../../../../tickets/implement-parallel-reduction-strategies.md)
/// owns decision **D-5** and states the obligation this fact discharges: "the
/// accumulation dtype is an explicit part of the strategy, not inherited
/// silently from the element dtype, and a strategy that would accumulate at a
/// narrower width than the contract allows is rejected with a typed reason".
/// This family consumes that authority rather than re-deciding it — the value is
/// `tiler::f32@1` because F32 is what reproduces the pinned reference, and
/// whether a longer context justifies widening stays that ticket's question.
pub const RMS_NORM_F32_FACT_ACCUMULATOR_TYPE: AttributeFieldId = AttributeFieldId::new(6);
/// Fact field naming how the mean divides by the normalized extent.
pub const RMS_NORM_F32_FACT_EXTENT_DIVISION: AttributeFieldId = AttributeFieldId::new(7);
/// Fact field naming the operation's behaviour on binary32 subnormals.
///
/// **Measurement — the reference and the qualified Metal row disagree here, and
/// the fact records the disagreement rather than resolving it.** A row of `1e-40`
/// squares to exactly `+0.0` on a preserving host, so the mean of squares is zero
/// and the result is the input scaled by `rsqrt(eps)` — a *normal* value from
/// subnormal inputs. On a target that flushes input subnormals the same row
/// reaches the squaring as zeros and normalizes to zeros. Under ADR 0076 that is
/// a declared realization difference, not a defect to be tuned away.
pub const RMS_NORM_F32_FACT_SUBNORMALS: AttributeFieldId = AttributeFieldId::new(8);
/// Fact field naming the operation's behaviour on signed zero.
pub const RMS_NORM_F32_FACT_SIGNED_ZERO: AttributeFieldId = AttributeFieldId::new(9);
/// Fact field naming the operation's NaN behaviour.
pub const RMS_NORM_F32_FACT_NAN_BEHAVIOUR: AttributeFieldId = AttributeFieldId::new(10);
/// Fact field carrying the canonical arithmetic-NaN payload this operation installs.
pub const RMS_NORM_F32_FACT_CANONICAL_NAN_BITS: AttributeFieldId = AttributeFieldId::new(11);
/// Fact field naming what a squaring overflow produces, which is decision **D-3**.
///
/// **D-3 is settled here as *define*, not refuse, and the elimination is the
/// justification.** A row whose elements reach
/// [`RMS_NORM_F32_SQUARING_OVERFLOW_BITS`] produces an infinite mean of squares,
/// a reciprocal square root of `+0.0`, and a row of signed zeros — finite,
/// plausible, and wrong, with no NaN or infinity to reveal it. Three candidate
/// refusals were tested against what this graph can actually establish and each
/// fails:
///
/// - **Refuse at construction.** Construction sees shapes and attributes, never
///   element values, so the precondition is not a fact the graph holds. There is
///   nothing to check.
/// - **Refuse on a proved value domain.** This would need an upper bound on
///   `|x|` carried by the operand. `ExceptionalValueAssumption` and
///   `ValueDomainProvenance::CompilerProven` name that class of evidence and no
///   program input supplies it, so the refusal would be unreachable rather than
///   conservative.
/// - **Refuse after a runtime scan.** A scan is a *costed operation*, not a free
///   guard: it reads every element of every normalized row a second time — a
///   full extra pass over the 144,384·`T` contributors of one forward pass — and
///   it must then be acted on, which needs either a host readback per occurrence
///   or a device-side validation mechanism. Neither exists in the bounded profile
///   and inventing one is a runtime-validation authority rather than a rule.
///
/// So refusal is not available at any cost this ticket can pay, and defining the
/// behaviour is what the pinned formula already means. The operation reproduces
/// the reference exactly; a divergence would be the defect. The threshold is
/// named, the conformance corpus carries a row above it, and the deferred
/// capability is
/// [`scope-a-value-domain-precondition-for-squaring-overflow`](../../../../tickets/scope-a-value-domain-precondition-for-squaring-overflow.md).
pub const RMS_NORM_F32_FACT_SQUARING_OVERFLOW: AttributeFieldId = AttributeFieldId::new(12);
/// Fact field stating whether ADR 0015's arithmetic contraction is permitted.
pub const RMS_NORM_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(13);
/// Fact field stating whether the extent division may become a reciprocal multiplication.
///
/// `false`, and it is a *withheld permission* rather than an absent one. See the
/// module header: the substitution is exact at a power-of-two extent and rounds
/// twice at every other one, so granting it would make the operation's meaning
/// depend on the extent.
pub const RMS_NORM_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(14);
/// Fact field stating whether an approximate intrinsic may realize the reciprocal square root.
///
/// `false`. Metal's fast-math table gives `rsqrt` `<= 2 ulp` where its ordinary
/// table gives correctly rounded, so `air.fast_rsqrt.f32` satisfies a different
/// contract rather than realizing this one more cheaply.
pub const RMS_NORM_F32_FACT_APPROXIMATE_INTRINSIC_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(15);

/// Returns the governed binary32 root-mean-square normalization key.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn rms_norm_f32_op() -> OpKey {
    OpKey::new("tiler", "rms-norm-f32", 1)
        .expect("the governed RMS normalization operation key is valid")
}

/// Returns the immutable reference semantics this key pins.
///
/// # Panics
///
/// Panics only if this crate's own compile-time reference text violates the
/// canonical bound registration would reject it under.
#[must_use]
pub fn rms_norm_f32_reference_semantics() -> NormativeDefinitionRef {
    NormativeDefinitionRef::new(
        "tiler::rms-norm-f32@1; over the single normalized axis of extent N, in this exact order: \
         q_i = x_i * x_i, then a = the strict left fold of q over the canonical contributor \
         sequence seeded at the first contributor, then u = a / N as a division and never a \
         multiplication by 1/N, then t = u + eps with eps inside the reciprocal square root's \
         argument, then r = Rsqrt(t) and deliberately not 1 / Sqrt(t), then y_i = w_i * (x_i * r) \
         with the weight applied after the identity conversion back to tiler::f32@1; nothing is \
         subtracted, so this is not layer normalization",
    )
    .expect("the governed RMS normalization reference semantics are canonical")
}

/// Returns the immutable reference semantics of the subordinate reciprocal square root.
///
/// A separate reference from the normalization's own, because the accuracy
/// contract resolves this one step and a contract whose reference named the whole
/// normalization would be stating a tolerance over a composition whose every
/// other step ADR 0024 already fixes exactly.
///
/// # Panics
///
/// Panics only if this crate's own compile-time reference text violates the
/// canonical bound.
#[must_use]
pub fn rms_norm_f32_rsqrt_reference_semantics() -> NormativeDefinitionRef {
    NormativeDefinitionRef::new(
        "the real reciprocal square root 1/sqrt(t) at t > 0, evaluated as the one inexact \
         subordinate step of tiler::rms-norm-f32@1; this is not a registered general Rsqrt \
         operation and mints no key of its own",
    )
    .expect("the governed subordinate reciprocal square root reference is canonical")
}

/// Returns the resolved ADR 0042 accuracy contract of the reciprocal square root.
///
/// # The form is `Faithful`, and that is a derivation rather than a fallback
///
/// **What the specification states.** Table 8.1 of the retained Metal Shading
/// Language Specification gives `rsqrt` at F32 as *correctly rounded* under the
/// precise math selection — a stronger guarantee than the `<= 4 ulp` it gives
/// `exp`, and the entry that made this family's contract shaped differently from
/// `tiler::silu-f32@1`'s.
///
/// **Why it cannot be written as `CorrectlyRounded`.** §8.2 states, in its
/// entirety, that "either round ties to even or round toward zero rounding mode
/// may be supported", and [`AccuracyContractForm::CorrectlyRounded`] carries a
/// required [`ReferenceRoundingRule`] whose only variant is
/// `NearestTiesToEven`. Writing that form would claim a choice the specification
/// deliberately leaves open — Gap 4 of the [Metal elementary-function accuracy
/// guarantee](../../../../docs/research/numerics/metal-elementary-function-accuracy.md),
/// which records that the omission is deliberate because Apple names the mode
/// wherever it means to fix one.
///
/// **Why `Faithful` is the exact statement and not a conservative one.** A
/// correctly rounded result under round-to-nearest is the nearer of the two
/// binary32 values bracketing the exact reference; under round-toward-zero it is
/// the one of smaller magnitude. Both are members of the faithful result set, so
/// the union over the two admitted modes is contained in it. The containment is
/// an equality rather than a strict inclusion: at any argument whose exact
/// reciprocal square root lies above the midpoint of its bracketing pair,
/// round-to-nearest returns the upper neighbour and round-toward-zero the lower,
/// so both members are reachable and no narrower form of this vocabulary
/// contains the union.
///
/// **The consequence for the ULP metric, which is the part worth stating.**
/// `Faithful` names a result set directly and measures nothing, so the
/// cross-metric reconciliation that `tiler::silu-f32@1`'s exponential needed —
/// Apple's `ulp` against `tiler::ulp-reference-gap@1`, a registered
/// `ScaledMetric` implication carrying a factor of three — **does not bind this
/// family at all**. Gap 1 and Gap 4 bind disjoint halves of Table 8.1, and this
/// entry is on the other half from the exponential's. This vertical therefore
/// registers no second cross-metric row, and a contract stating
/// `Ulp(tiler::ulp-reference-gap@1, 1)` here would be *weaker* than what the
/// specification supports as well as needing a translation it does not need: the
/// vocabulary's own `faithful-satisfies-ulp` row already carries that implication
/// in the one direction it holds, and nothing in this family consumes it.
///
/// # Why the contract states no accuracy domain, and what carries the region
///
/// `Faithful` names a result set for every argument at which the reference is
/// defined, so — unlike [`AccuracyContractForm::BoundedPiecewise`] — it takes no
/// [`AccuracyDomain`] and there is no per-region tolerance to place. The region
/// is therefore carried by [`rms_norm_f32_rsqrt_ordinary_domain`] beside the
/// contract rather than inside it, and the reference's behaviour outside that
/// region is carried by the four independent rules of
/// [`rms_norm_f32_rsqrt_exceptional_contract`]. Two properties make that split
/// total rather than partial over the reachable inputs. The argument is
/// `t = u + eps` where `u` is a sum of squares — never negative — and `eps` is
/// refused at construction unless it is positive and finite, so `t` is positive
/// at every input whose mean of squares is finite. And the reference stays inside
/// binary32's finite normal range across the whole ordinary domain: at the least
/// positive subnormal argument `2^-149` the reciprocal square root is about
/// `2^74.5`, and at `f32::MAX` it is about `2^-63.5`, both far from the format's
/// overflow and subnormal boundaries. So the faithful decision is defined
/// everywhere the ordinary domain reaches, and no ordinary-domain input is
/// carried by an exceptional rule instead.
///
/// # Panics
///
/// Panics only if this crate's own compile-time contract violates the grammar its
/// own vocabulary defines, which registration would reject as well.
///
/// [`ReferenceRoundingRule`]: super::accuracy::ReferenceRoundingRule
/// [`AccuracyDomain`]: super::accuracy::AccuracyDomain
#[must_use]
pub fn rms_norm_f32_rsqrt_accuracy_contract() -> AccuracyContract {
    AccuracyContract::new(
        rms_norm_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        rms_norm_f32_rsqrt_reference_semantics(),
        AccuracyContractForm::Faithful,
        rms_norm_f32_rsqrt_exceptional_contract(),
    )
}

/// Returns the region over which the reciprocal square root's reference is defined.
///
/// Open at zero and unbounded above, over operand ordinal zero. It is stated as a
/// value rather than only in prose because
/// [`rms_norm_f32_rsqrt_accuracy_contract`] is `Faithful` and therefore carries
/// no [`AccuracyDomain`] of its own: without this, the boundary between the
/// accuracy form and the exceptional rules would live only in a comment, and a
/// consumer deciding which of the two governs an argument would have to
/// rediscover it.
///
/// The lower bound is *open*. Zero is not an ordinary argument: `1/sqrt(+0)` is
/// the infinite reference [`InfiniteReferenceRule`] governs, and admitting it
/// here would place a finite-result obligation on an argument with no finite
/// result.
///
/// [`AccuracyDomain`]: super::accuracy::AccuracyDomain
///
/// # Panics
///
/// Panics only if this crate's own compile-time interval violates the domain
/// language's grammar.
#[must_use]
pub fn rms_norm_f32_rsqrt_ordinary_domain() -> DomainInterval {
    DomainInterval::new(
        OperandOrdinal::new(0),
        DomainBound::Open(ExactRational::zero()),
        DomainBound::Unbounded,
    )
    .expect("the governed reciprocal square root domain admits every positive argument")
}

/// Returns the reciprocal square root's independent exceptional-value contract.
///
/// Stated separately from the accuracy form and from the normalization's own
/// exceptional behaviour, because ADR 0042 makes those three different claims and
/// `refines` refuses outright when two contracts state different ones — so a
/// realization must reproduce this record exactly rather than approximate it.
///
/// - a NaN argument has a NaN reference, and the operation installs its canonical
///   arithmetic NaN;
/// - an infinite reference is the infinity of its own sign, which is the `+inf`
///   of `1/sqrt(+0)`; the case is unreachable while `eps` is positive, and the
///   rule is stated because the contract is a claim about the function rather
///   than about which of its inputs this workload happens to reach;
/// - a negative argument is a *domain error* and yields the canonical NaN. It is
///   likewise unreachable — a sum of squares is never negative — and stated for
///   the same reason;
/// - a finite reference above binary32's finite range yields the infinity of its
///   sign. The header's range derivation shows this case cannot arise on the
///   admitted domain, which makes the rule vacuous rather than absent: a contract
///   that omitted it would not be comparable by `refines` against one that stated
///   it.
#[must_use]
pub const fn rms_norm_f32_rsqrt_exceptional_contract() -> ExceptionalValueContract {
    ExceptionalValueContract::new(
        NanReferenceRule::CanonicalNan,
        InfiniteReferenceRule::SignedInfinity,
        DomainErrorRule::CanonicalNan,
        FiniteOverflowRule::SignedInfinity,
    )
}

/// Returns the exact fact record the governed normalization definition carries.
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
pub fn rms_norm_f32_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(RMS_NORM_F32_FACT_COMPUTATION_TYPE, f32_value_type()),
        CanonicalField::new(RMS_NORM_F32_FACT_RESULT_TYPE, f32_value_type()),
        CanonicalField::new(
            RMS_NORM_F32_FACT_EVALUATION_ORDER,
            fact(
                "square-then-fold-then-divide-by-the-extent-then-add-eps-then-rsqrt-then-scale-\
                 then-weight; eps-inside-the-rsqrt-argument; rsqrt-and-not-one-over-sqrt; \
                 weight-multiply-after-the-f32-identity-conversion; every-step-rounds-once-under-\
                 ties-to-even-except-the-rsqrt",
            ),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_RSQRT_ACCURACY_CONTRACT,
            rms_norm_f32_rsqrt_accuracy_contract()
                .to_canonical_value()
                .expect("the governed RMS normalization accuracy contract is canonical"),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_FOLD_ORDER,
            fact("strict-left-fold-over-the-canonical-contributor-sequence-seeded-at-the-first-contributor"),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_ACCUMULATOR_TYPE,
            fact("tiler::f32@1"),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_EXTENT_DIVISION,
            fact("divide-by-the-static-extent-never-multiply-by-its-reciprocal"),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_SUBNORMALS,
            fact("preserved-by-this-contract-and-flushed-on-a-declared-flushing-realization-a-recorded-divergence"),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_SIGNED_ZERO,
            fact("ieee-754-signed-zero-rules-a-negative-zero-element-normalizes-to-a-negative-zero"),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_NAN_BEHAVIOUR,
            fact("quiet-nan-propagates-through-the-fold-and-every-arithmetic-nan-result-is-canonicalized"),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_CANONICAL_NAN_BITS,
            super::registry::canonical_f32_bits(super::CANONICAL_F32_ARITHMETIC_NAN_BITS),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_SQUARING_OVERFLOW,
            fact("defined-and-not-refused-an-infinite-mean-of-squares-gives-a-zero-rsqrt-and-a-row-of-signed-zeros"),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            RMS_NORM_F32_FACT_APPROXIMATE_INTRINSIC_PERMITTED,
            CanonicalValue::boolean(false),
        ),
    ])
    .expect("the governed RMS normalization facts are canonical")
}

/// Returns the reduced-axis attribute value for one normalized axis.
///
/// # Panics
///
/// Panics only if a one-element sequence violates the canonical value grammar.
#[must_use]
pub fn rms_norm_f32_axis_attribute(axis: Axis) -> CanonicalValue {
    CanonicalValue::sequence([CanonicalValue::unsigned_u32(axis.get())])
        .expect("a one-element axis sequence is canonical")
}

/// Returns the `eps` attribute value for one exact binary32 payload.
///
/// Takes the payload rather than an `f32` so that a caller cannot reach the
/// attribute through a decimal literal whose rounding it never looked at. The
/// admissibility of the payload is decided by the inferencer, not here.
///
/// # Panics
///
/// Panics only if the governed binary32 format key violates its own grammar.
#[must_use]
pub fn rms_norm_f32_eps_attribute(bits: u32) -> CanonicalValue {
    super::registry::canonical_f32_bits(bits)
}

pub(super) fn register_standard_rms_norm(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        rms_norm_f32_op(),
        OperationSchema::new(
            OperationArity::exact(2),
            OperationArity::exact(1),
            [
                OperationAttributeSchema::required(
                    RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                    CanonicalValueKind::Sequence,
                ),
                OperationAttributeSchema::required(
                    RMS_NORM_EPS_BITS_ATTRIBUTE,
                    CanonicalValueKind::FloatBits,
                ),
            ],
        )
        .expect("the governed RMS normalization operation schema is valid"),
        rms_norm_f32_reference_semantics(),
        OperationDefinitionFacts::new(rms_norm_f32_facts()),
        standard_conformance("rms-norm-f32"),
        OperationEffect::Pure,
        Arc::new(RmsNormF32),
    ))
    // No algebraic capability is declared, and the absence is derived rather than
    // deferred. The normalization is neither associative nor commutative in
    // either operand, and the ordered-associativity law `tiler::add-f32@1`
    // declares is about the *embedded* fold rather than about this operation.
    // Declaring nothing reads as unknown rather than as the inverse law.
}

struct RmsNormF32;

impl OperationInferencer for RmsNormF32 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let [input, weight] = operands else {
            return Err(op_error(
                "rms-norm.f32.arity",
                "the binary32 RMS normalization requires exactly two operands, the normalized \
                 value and its weight",
            ));
        };
        let expected = F32::resolved_type();
        if input.resolved_type() != &expected || weight.resolved_type() != &expected {
            return Err(op_error(
                "rms-norm.f32.implicit-promotion",
                "the binary32 RMS normalization admits no implicit promotion; an operand of \
                 another type is not converted to tiler::f32@1",
            ));
        }
        // `docs/ir.md` admits no implicit broadcasting, and the rank-zero scalar
        // admission does not cover a per-channel weight. The workload's weight is
        // `[N]` against `[T, N]`, so the program spells that widening as a
        // `tiler::broadcast-f32@1` node and this key receives two operands of one
        // shape. Accepting the narrow shape here would be implicit broadcasting
        // under another name, and it would put the broadcast's own access
        // relation inside an operation whose identity does not carry one.
        if weight.shape() != input.shape() {
            return Err(op_error(
                "rms-norm.f32.weight-shape",
                "the binary32 RMS normalization admits no implicit broadcasting; the weight \
                 operand must already have the normalized value's shape, which a \
                 tiler::broadcast-f32@1 occurrence produces",
            ));
        }
        let attributes = request.attributes();
        if attributes.fields().len() != 2 {
            return Err(op_error(
                "rms-norm.f32.attributes",
                "the binary32 RMS normalization requires exactly the reduced-axis and eps \
                 attributes",
            ));
        }
        let axis = reduced_axis(&request, input.shape().rank())?;
        eps_payload(&request)?;
        // Shape-preserving: the normalization divides each element of a row by
        // that row's root mean square, so the reduced axis is folded over and
        // then restored. A contract that dropped the axis would be a mean, which
        // this family deliberately does not admit.
        let _ = axis;
        outputs.try_push(ValueFact::new(expected, input.shape().clone()))
    }
}

/// Resolves the single normalized axis, naming the violated rule on refusal.
fn reduced_axis(
    request: &OperationInferenceRequest<'_>,
    rank: usize,
) -> Result<Axis, OperationInferenceError> {
    let Some(CanonicalValueView::Sequence(values)) = request
        .attributes()
        .get(RMS_NORM_REDUCED_AXES_ATTRIBUTE)
        .map(CanonicalValue::view)
    else {
        return Err(op_error(
            "rms-norm.f32.axis.kind",
            "the RMS normalization's reduced-axis attribute must be a sequence of u32 axes",
        ));
    };
    if values.is_empty() {
        return Err(op_error(
            "rms-norm.f32.axis.absent",
            "the RMS normalization requires a normalized axis; the reduced-axis sequence is empty",
        ));
    }
    let mut named: Vec<Axis> = Vec::with_capacity(values.len());
    for value in values {
        let CanonicalValueView::Unsigned { width, bits } = value.view() else {
            return Err(op_error(
                "rms-norm.f32.axis.type",
                "an RMS normalization reduced axis must be an unsigned integer",
            ));
        };
        if width != super::CanonicalIntegerWidth::Bits32 {
            return Err(op_error(
                "rms-norm.f32.axis.width",
                "an RMS normalization reduced axis must use u32",
            ));
        }
        let axis = Axis::new(u32::try_from(bits).map_err(|_| {
            op_error(
                "rms-norm.f32.axis.width",
                "an RMS normalization reduced axis exceeds u32",
            )
        })?);
        if named.contains(&axis) {
            return Err(op_error(
                "rms-norm.f32.axis.duplicated",
                "an RMS normalization reduced axis is named more than once; the reduced-axis \
                 sequence must be strictly ascending",
            ));
        }
        named.push(axis);
    }
    let [only] = named.as_slice() else {
        return Err(op_error(
            "rms-norm.f32.axis.rank",
            "the RMS normalization family normalizes over exactly one axis; a multi-axis \
             normalization is a different operation and is not admitted here",
        ));
    };
    if usize::try_from(only.get()).map_or(true, |position| position >= rank) {
        return Err(op_error(
            "rms-norm.f32.axis.range",
            "the RMS normalization's reduced axis is out of range for the operand's rank",
        ));
    }
    Ok(*only)
}

/// Resolves the exact `eps` payload, naming the violated rule on refusal.
///
/// A zero `eps` is refused rather than accepted as a degenerate parameter,
/// because it is a *different operation with a different domain*: with `eps`, a
/// zero row normalizes to zeros and a subnormal row to a normal value; without
/// it, the same rows give NaNs and infinities.
fn eps_payload(request: &OperationInferenceRequest<'_>) -> Result<u32, OperationInferenceError> {
    let Some(CanonicalValueView::FloatBits(payload)) = request
        .attributes()
        .get(RMS_NORM_EPS_BITS_ATTRIBUTE)
        .map(CanonicalValue::view)
    else {
        return Err(op_error(
            "rms-norm.f32.eps.kind",
            "the RMS normalization's eps attribute must be exact binary32 bits with an explicit \
             format",
        ));
    };
    let governed =
        super::TypeKey::new("tiler", "f32", 1).expect("the governed binary32 format key is valid");
    if payload.format() != &governed {
        return Err(op_error(
            "rms-norm.f32.eps.format",
            "the RMS normalization's eps attribute must name tiler::f32@1 as its format",
        ));
    }
    let bytes = <[u8; 4]>::try_from(payload.bits()).map_err(|_| {
        op_error(
            "rms-norm.f32.eps.width",
            "the RMS normalization's eps attribute must carry exactly four payload bytes",
        )
    })?;
    let bits = u32::from_be_bytes(bytes);
    let value = f32::from_bits(bits);
    if value.is_nan() {
        return Err(op_error(
            "rms-norm.f32.eps.nan",
            "the RMS normalization's eps must be a number; a NaN eps makes every output a NaN",
        ));
    }
    if !value.is_finite() {
        return Err(op_error(
            "rms-norm.f32.eps.non-finite",
            "the RMS normalization's eps must be finite; an infinite eps makes every reciprocal \
             square root zero",
        ));
    }
    // `value > 0.0` is false for both zeros and for every negative, which is the
    // whole admissibility rule in one comparison — and the two refusals are
    // separated so the diagnostic names which one was violated.
    if value == 0.0 {
        return Err(op_error(
            "rms-norm.f32.eps.zero",
            "the RMS normalization's eps must be strictly positive; a zero eps is not a \
             degenerate parameter but a different operation whose domain excludes the zero row",
        ));
    }
    if value < 0.0 {
        return Err(op_error(
            "rms-norm.f32.eps.negative",
            "the RMS normalization's eps must be strictly positive; a negative eps admits a \
             negative reciprocal square root argument and therefore a NaN result at an ordinary \
             input",
        ));
    }
    Ok(bits)
}

fn f32_value_type() -> CanonicalValue {
    CanonicalValue::value_type(F32::resolved_type())
}

fn fact(value: &'static str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("the governed RMS normalization fact is bounded")
}

fn op_error(code: &str, message: &str) -> OperationInferenceError {
    OperationInferenceError::new(
        ProviderDiagnosticCode::new(code)
            .expect("the governed RMS normalization diagnostic code is canonical"),
        message,
    )
    .expect("the governed RMS normalization diagnostic message is canonical")
}

#[cfg(test)]
mod tests;
