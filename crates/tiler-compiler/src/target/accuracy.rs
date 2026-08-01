#![allow(
    dead_code,
    reason = "the authority is complete and its only consumer today is its own conformance tests: reaching it from the compile path needs a whole-program recognizer that admits an elementary operation, which `reach-a-verified-kernel-through-the-structural-families` owns; wiring it to an unreachable call site would assert a routing this build does not perform"
)]

//! Which elementary-function accuracy contracts a target realization refines.
//!
//! # The question this authority answers, and the one it does not
//!
//! A registered transcendental operation carries a *resolved* accuracy contract:
//! `tiler::silu-f32@1` states a twelve-ULP bound on its subordinate exponential
//! under `tiler::ulp-reference-gap@1`, together with four independent
//! exceptional-value rules. A target realization is legal only when its own
//! stated contract **provably refines** that one — ADR 0042: "An implementation
//! is legal only when its allowed result set refines that semantic result set."
//!
//! This module decides that, and only that. It does not choose an
//! implementation, does not rank contracts, and never narrows the required
//! contract to make a target fit; ADR 0076 item 5 forbids all three. When no
//! installed realization refines the requirement the answer is a typed refusal
//! naming the declaring profile and the refusing fact's measurement boundary,
//! which is the shape ADR 0043 makes hard feasibility rather than cost.
//!
//! # Two families, two contract forms, and only one of them needs a metric
//!
//! `tiler::silu-f32@1`'s exponential is a ULP bound and reaches the
//! reconciliation below. `tiler::rms-norm-f32@1`'s reciprocal square root is
//! `Faithful` and does not: Table 8.1 states `rsqrt` *correctly rounded*, §8.2
//! leaves the rounding mode open between ties-to-even and toward-zero, and the
//! union of the two admitted modes is exactly the faithful result set. So the
//! four gaps the Metal accuracy record names bind disjoint halves of that table —
//! the metric reconciliation (Gap 1) binds the ULP entries and the rounding-mode
//! question (Gap 4) binds the correctly rounded ones — and this module registers
//! **one** cross-metric row rather than two, because the second family needs
//! none.
//!
//! # Why the Metal declaration is not simply `Ulp(tiler::ulp-reference-gap@1, 4)`
//!
//! Metal's Table 8.1 states `exp <= 4 ulp` under **Apple's** definition of `ulp`,
//! and ADR 0042 forbids translating a bound across metric definitions by name:
//! "a distinct metric key is not a name to match on". So the declaration states
//! its bound under [`apple_msl_ulp_metric_key`], and the translation is a
//! *registered* [`RegisteredImplication::ScaledMetric`] carrying its derivation —
//! which is the mechanism the accuracy vocabulary supplies for exactly this and
//! whose `standard()` registry deliberately supplies no row for.
//!
//! [`RegisteredImplication::ScaledMetric`]: tiler_ir::semantic::accuracy::RegisteredImplication::ScaledMetric
//!
//! # Two evidence records, because they are two different claims
//!
//! [`metal_f32_exponential_bound_evidence`] is a `NormativeGuarantee`: a quoted
//! entry of a retained specification at a verified digest, which
//! `ConformanceEvidenceClass::discharges_hard_requirement` admits.
//!
//! [`metal_f32_exceptional_value_evidence`] is an `EmpiricalQualification`, and
//! that is the honest class rather than a conservative one. The specification
//! supplies **no** exceptional-value contract for `exp`: chapter 8 has no
//! edge-case table for math functions, §8.3 disables floating-point exceptions,
//! and §8.1's "may be flushed to zero" is permissive and therefore licenses
//! neither declaration. What exists is a bounded corpus on one host row. Under
//! ADR 0042 an empirical record "detects regressions and characterizes
//! implementations but does not prove an unmeasured worst-case bound", so it
//! cannot discharge a hard requirement — and [`ElementaryRealization::discharge`]
//! reports that rather than borrowing the bound the other half established.

use std::sync::Arc;

use tiler_ir::semantic::accuracy::{
    AccuracyContract, AccuracyContractForm, AccuracyDomain, AccuracyDomainClause,
    AccuracyMetricKey, AccuracyPredicate, ConformanceEvidence, ConformanceEvidenceClass,
    ConformanceEvidenceError, DomainBound, DomainInterval, ExactRational, ExactTolerance,
    OperandOrdinal, ReferenceResultClass, ReferenceResultConstraint, RefinementBasis,
    RefinementOutcome, RefinementUnknown, RegisteredImplication, RegisteredImplicationKey,
    RegisteredImplicationRegistry, ulp_reference_gap_metric_key,
};
use tiler_ir::semantic::{
    F32, NormativeDefinitionRef, OpKey, SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS,
    rms_norm_f32_op, rms_norm_f32_rsqrt_exceptional_contract,
    rms_norm_f32_rsqrt_reference_semantics, silu_f32_exponential_exceptional_contract,
    silu_f32_exponential_reference_semantics, silu_f32_op,
};

use super::honourability::FactSourceProvenance;

/// Exact factor relating Apple's ULP scale to `tiler::ulp-reference-gap@1`'s.
///
/// **Three, and the derivation is a claim about two readings rather than a
/// choice between them.** Apple's second clause — "otherwise `ulp(x)` is the
/// distance between the two nonequal finite floating-point numbers nearest x" —
/// can be read as the nearest adjacent pair of distinct representable values
/// (consistent with the first clause) or as predecessor-to-successor (the more
/// literal parse). Against Tiler's metric, which takes the *smaller* adjacent
/// gap, the largest ratio over the whole finite domain is two under the first
/// reading and three under the second. Nothing in the retained specification
/// chooses, so a conversion covering both takes three.
pub(crate) const APPLE_ULP_TRANSLATION_FACTOR: u64 = 3;

/// Apple's stated single-precision `exp` bound, in Apple's own ULPs.
pub(crate) const APPLE_MSL_EXP_F32_ULP_BOUND: u64 = 4;

/// Returns the metric key carrying Metal's own definition of `ulp`.
///
/// A second key rather than a reinterpretation of Tiler's, because the two
/// definitions differ at a representable value, at a power of two, at zero, and
/// at NaN. Minting it is what makes the difference statable; registering the
/// implication below is what makes it crossable.
///
/// # Panics
///
/// Panics only if this crate's compile-time key violates the canonical identity
/// grammar.
#[must_use]
pub(crate) fn apple_msl_ulp_metric_key() -> AccuracyMetricKey {
    AccuracyMetricKey::new("apple", "msl-ulp", 1).expect("the Apple MSL ULP metric key is valid")
}

/// Returns the implication registry this build decides elementary refinement under.
///
/// The vocabulary's own three rows plus the one cross-metric row this vertical
/// derives. `RegisteredImplicationRegistry::standard` deliberately ships no
/// cross-metric row — adopting a vendor's ULP bound needs that vendor's own
/// definition read and reconciled, which is evidence work rather than a default —
/// and this is that work, registered where the target realization is declared
/// rather than inside the target-neutral vocabulary.
///
/// # Panics
///
/// Panics only if this crate's compile-time keys or exact rationals violate
/// their own grammar.
#[must_use]
pub(crate) fn installed_implication_registry() -> RegisteredImplicationRegistry {
    let mut registry =
        RegisteredImplicationRegistry::standard().expect("the governed implications are valid");
    registry.register(
        RegisteredImplicationKey::new("tiler", "apple-msl-ulp-to-reference-gap", 1)
            .expect("the cross-metric implication key is valid"),
        RegisteredImplication::ScaledMetric {
            from: apple_msl_ulp_metric_key(),
            to: ulp_reference_gap_metric_key(),
            factor: ExactTolerance::from_integer(APPLE_ULP_TRANSLATION_FACTOR),
        },
        NormativeDefinitionRef::new(
            "Metal Shading Language Specification v4.1 (2026-06-04) section 8.4 defines ulp(x) as \
             |b - a| between the consecutive finite values bracketing a non-representable x, and \
             otherwise as the distance between the two nonequal finite values nearest x; the \
             second clause admits an adjacent-pair reading and a predecessor-to-successor reading, \
             whose largest ratios to tiler::ulp-reference-gap@1's smaller-adjacent-gap rule over \
             the whole finite domain are two and three respectively, so three is the conservative \
             factor covering both readings and no domain restriction is claimed",
        )
        .expect("the cross-metric derivation is canonical"),
    );
    registry
}

/// Returns the accuracy contract the Metal realization of the activation states.
///
/// Identical to `tiler::silu-f32@1`'s requirement in operation, dtype signature,
/// reference semantics, exceptional-value contract, and admitted domain — all
/// five of which `refines` compares before it reaches the bound, and any of which
/// differing is an outright `Unknown` rather than a looser bound. The one
/// difference is the metric the bound is stated under, which is the whole subject
/// of the registered implication above.
///
/// # Panics
///
/// Panics only if this crate's compile-time contract violates the grammar the
/// accuracy vocabulary defines.
#[must_use]
pub(crate) fn metal_f32_exponential_contract() -> AccuracyContract {
    let ceiling =
        ExactRational::from_f32(f32::from_bits(SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS))
            .expect("the governed exponential ceiling is a finite binary32 value");
    let ordinary = DomainInterval::new(
        OperandOrdinal::new(0),
        DomainBound::Unbounded,
        DomainBound::Closed(ceiling),
    )
    .expect("the declared domain admits every argument at or below its ceiling");
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
                .expect("the positivity justification is canonical"),
            ),
        )
        .expect("the reference-result constraint is canonical"),
        AccuracyPredicate::ulp(
            apple_msl_ulp_metric_key(),
            ExactTolerance::from_integer(APPLE_MSL_EXP_F32_ULP_BOUND),
        ),
    )
    .expect("the declared clause is canonical");
    AccuracyContract::new(
        silu_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        silu_f32_exponential_reference_semantics(),
        AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new([ordinary], [clause]).expect("the declared domain is canonical"),
        ),
        silu_f32_exponential_exceptional_contract(),
    )
}

/// Returns the accuracy contract the Metal realization of the normalization states.
///
/// **Identical to `tiler::rms-norm-f32@1`'s requirement, and the identity is the
/// whole content of the declaration rather than a coincidence.** Table 8.1 states
/// `rsqrt` correctly rounded and §8.2 admits either ties-to-even or toward-zero,
/// so what Metal *promises* is a value drawn from the two-element faithful set —
/// which is what the requirement states. `refines` therefore admits it on
/// `RefinementBasis::IdenticalNormalizedContract` rather than through a
/// registered implication, and that is the honest outcome: there is no
/// translation to perform.
///
/// **Declaring `CorrectlyRounded { NearestTiesToEven }` here would be the
/// substitution to avoid**, in the direction that looks conservative and is not.
/// It would be a *stronger* claim than the specification supports, and because
/// `refines` proves correctly-rounded-satisfies-faithful along a registered row,
/// it would be admitted — so the over-claim would pass rather than fail. The
/// check that catches it is
/// `the_metal_normalization_declaration_is_not_stronger_than_the_specification`.
///
/// # Panics
///
/// Panics only if this crate's compile-time contract violates the grammar the
/// accuracy vocabulary defines.
#[must_use]
pub(crate) fn metal_f32_reciprocal_square_root_contract() -> AccuracyContract {
    AccuracyContract::new(
        rms_norm_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        rms_norm_f32_rsqrt_reference_semantics(),
        AccuracyContractForm::Faithful,
        rms_norm_f32_rsqrt_exceptional_contract(),
    )
}

/// Returns the normative record behind the Metal reciprocal square root's form.
///
/// A `NormativeGuarantee`, like the exponential's bound and for the same reason —
/// a quoted entry of a retained specification at a verified digest — but its
/// scope names the *rounding-mode* qualification rather than a metric one,
/// because that is the gap this entry has to cross.
///
/// # Errors
///
/// Returns [`ConformanceEvidenceError`] only if this crate's own compile-time
/// record violates the record's stated obligations.
pub(crate) fn metal_f32_reciprocal_square_root_bound_evidence()
-> Result<ConformanceEvidence, ConformanceEvidenceError> {
    let reference = |text: &str| {
        NormativeDefinitionRef::new(text).expect("a compile-time evidence field is canonical")
    };
    ConformanceEvidence::new(
        ConformanceEvidenceClass::NormativeGuarantee,
        reference(
            "the ordinary-domain accuracy of single-precision rsqrt under Metal's precise math              selection, stated as a faithful result set; qualified by two readings the              specification requires rather than one. MSL 4.1 Table 8.1 gives rsqrt as correctly              rounded, and section 8.2 states that either round ties to even or round toward zero              may be supported, so the promised set is the union of the two correctly rounded              results, which is the faithful pair. Also qualified by the applicability inference              that section 1.6.3's equivalence -fno-fast-math =              -fmetal-math-fp32-functions=precise -fmetal-math-mode=safe makes Table 8.1 rather              than Table 8.2 the governing table, which the specification never states directly",
        ),
        reference("Metal shading language, single precision, precise math selection"),
        reference("air.rsqrt.f32, selected by the precise::rsqrt namespace under -std=metal4.0"),
        reference(
            "Metal Shading Language Specification version 4.1 dated 2026-06-04, and version 4              dated 2025-10-23, whose section 8.4 is byte-identical after footer normalization",
        ),
        None,
        None,
        None,
        b"sha256:41538b30d2f1140a5b2a0c84ce0a9f7b67bf0c707e224cfea0bfe5a44aa26cf5",
    )
}

/// Returns the empirical record behind the normalization's exceptional behaviour.
///
/// An `EmpiricalQualification` for the same reason the activation's is: chapter 8
/// has no edge-case table for math functions, and §8.1's "may be flushed to zero"
/// is permissive and therefore licenses neither declaration. What exists is a
/// bounded corpus, and it does not discharge a hard requirement.
///
/// # Errors
///
/// Returns [`ConformanceEvidenceError`] only if this crate's own compile-time
/// record violates the record's stated obligations.
pub(crate) fn metal_f32_normalization_exceptional_value_evidence()
-> Result<ConformanceEvidence, ConformanceEvidenceError> {
    let reference = |text: &str| {
        NormativeDefinitionRef::new(text).expect("a compile-time evidence field is canonical")
    };
    ConformanceEvidence::new(
        ConformanceEvidenceClass::EmpiricalQualification,
        reference(
            "the exceptional-value, signed-zero, and subnormal behaviour of              tiler::rms-norm-f32@1 at binary32; the Metal specification supplies no edge-case              contract for rsqrt, and section 8.1's flush permission licenses neither a flushing              nor a preserving declaration, so nothing normative covers this half",
        ),
        reference("Metal shading language on one measured Apple GPU row"),
        reference("air.rsqrt.f32 with the operator division, under the governed flag set"),
        reference("Apple metal version 32023.883, macOS 27.0, -std=metal4.0"),
        Some(reference("Apple M4 Max")),
        Some(reference(
            "crates/tiler-reference rms_norm_f32, whose reciprocal square root is certified              against an exact rational enclosure rather than a host library",
        )),
        Some(reference(
            "the bounded corpus of crates/tiler-reference/src/rms_norm/tests.rs: the retained              worked example, a zero row, a signed-zero row, a subnormal row, a row above the              squaring-overflow threshold, both workload extent classes at 1024 and 128, and a              contiguous 512-argument sweep of the reciprocal square root",
        )),
        b"corpus:rms-norm-f32-boundary-v1",
    )
}

/// Returns the normative record behind the Metal exponential's ULP bound.
///
/// # Errors
///
/// Returns [`ConformanceEvidenceError`] only if this crate's own compile-time
/// record violates the record's stated obligations.
pub(crate) fn metal_f32_exponential_bound_evidence()
-> Result<ConformanceEvidence, ConformanceEvidenceError> {
    let reference = |text: &str| {
        NormativeDefinitionRef::new(text).expect("a compile-time evidence field is canonical")
    };
    ConformanceEvidence::new(
        ConformanceEvidenceClass::NormativeGuarantee,
        reference(
            "the ordinary-domain accuracy of single-precision exp under Metal's precise math \
             selection; qualified by the applicability inference that MSL 4.1 section 1.6.3's \
             equivalence -fno-fast-math = -fmetal-math-fp32-functions=precise \
             -fmetal-math-mode=safe makes Table 8.1 rather than Table 8.2 the governing table, \
             which the specification never states directly",
        ),
        reference("Metal shading language, single precision, precise math selection"),
        reference("air.exp.f32, selected by the precise::exp namespace under -std=metal4.0"),
        reference(
            "Metal Shading Language Specification version 4.1 dated 2026-06-04, and version 4 \
             dated 2025-10-23, whose section 8.4 is byte-identical after footer normalization",
        ),
        None,
        None,
        None,
        b"sha256:41538b30d2f1140a5b2a0c84ce0a9f7b67bf0c707e224cfea0bfe5a44aa26cf5",
    )
}

/// Returns the empirical record behind the activation's exceptional-value behaviour.
///
/// # Errors
///
/// Returns [`ConformanceEvidenceError`] only if this crate's own compile-time
/// record violates the record's stated obligations.
pub(crate) fn metal_f32_exceptional_value_evidence()
-> Result<ConformanceEvidence, ConformanceEvidenceError> {
    let reference = |text: &str| {
        NormativeDefinitionRef::new(text).expect("a compile-time evidence field is canonical")
    };
    ConformanceEvidence::new(
        ConformanceEvidenceClass::EmpiricalQualification,
        reference(
            "the exceptional-value, signed-zero, and subnormal behaviour of tiler::silu-f32@1 at \
             binary32; the Metal specification supplies no edge-case contract for exp, so nothing \
             normative covers this half",
        ),
        reference("Metal shading language on one measured Apple GPU row"),
        reference("air.exp.f32 with the operator division, under the governed flag set"),
        reference("Apple metal version 32023.883, macOS 27.0, -std=metal4.0"),
        Some(reference("Apple M4 Max")),
        Some(reference(
            "crates/tiler-reference silu_f32, whose exponential is certified against an exact \
             rational enclosure rather than a host library",
        )),
        Some(reference(
            "the boundary corpus of crates/tiler-reference/src/silu/tests.rs: fourteen enumerated \
             binary32 arguments plus a contiguous 4,096-argument walk across the overflow band",
        )),
        b"corpus:silu-f32-boundary-v1",
    )
}

/// One installed target realization of one registered elementary operation.
#[derive(Clone, Debug)]
pub(crate) struct ElementaryRealization {
    operation: OpKey,
    contract: AccuracyContract,
    bound_evidence: ConformanceEvidence,
    exceptional_evidence: ConformanceEvidence,
    source: Arc<FactSourceProvenance>,
}

impl ElementaryRealization {
    /// States one realization, its contract, and the evidence behind each half.
    pub(crate) const fn new(
        operation: OpKey,
        contract: AccuracyContract,
        bound_evidence: ConformanceEvidence,
        exceptional_evidence: ConformanceEvidence,
        source: Arc<FactSourceProvenance>,
    ) -> Self {
        Self {
            operation,
            contract,
            bound_evidence,
            exceptional_evidence,
            source,
        }
    }

    /// The operation this realization speaks about.
    pub(crate) const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// The contract this realization states.
    pub(crate) const fn contract(&self) -> &AccuracyContract {
        &self.contract
    }

    /// The provenance of the declaring profile.
    pub(crate) fn source(&self) -> &FactSourceProvenance {
        &self.source
    }

    /// The record behind the accuracy bound.
    pub(crate) const fn bound_evidence(&self) -> &ConformanceEvidence {
        &self.bound_evidence
    }

    /// The record behind the exceptional-value behaviour.
    pub(crate) const fn exceptional_evidence(&self) -> &ConformanceEvidence {
        &self.exceptional_evidence
    }

    /// Reports which half of this realization discharges a hard requirement.
    ///
    /// Both halves are asked, and they answer differently on purpose. Reporting
    /// one summary boolean would have to pick which half to believe.
    pub(crate) fn discharge(&self) -> ElementaryDischarge {
        ElementaryDischarge {
            bound: self.bound_evidence.discharge().is_ok(),
            exceptional: self.exceptional_evidence.discharge().is_ok(),
            exceptional_class: self.exceptional_evidence.class(),
        }
    }
}

/// What each half of one realization's evidence establishes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ElementaryDischarge {
    bound: bool,
    exceptional: bool,
    exceptional_class: ConformanceEvidenceClass,
}

impl ElementaryDischarge {
    /// Whether the accuracy bound rests on evidence that may discharge a hard requirement.
    pub(crate) const fn bound_is_discharged(self) -> bool {
        self.bound
    }

    /// Whether the exceptional-value behaviour does.
    pub(crate) const fn exceptional_is_discharged(self) -> bool {
        self.exceptional
    }

    /// The class the exceptional-value record carries.
    pub(crate) const fn exceptional_class(self) -> ConformanceEvidenceClass {
        self.exceptional_class
    }
}

/// Why no installed realization satisfies one resolved accuracy contract.
///
/// Every field is one of the things ADR 0076 item 5 requires a rejection to name:
/// which operation, what the contract required, the declaring profile's versioned
/// identity, and the boundary of the fact that refused. A generic
/// unsupported-operation error carries none of them.
#[derive(Clone, Debug)]
pub(crate) struct ElementaryAccuracyRefusal {
    operation: OpKey,
    reason: ElementaryRefusalReason,
}

impl ElementaryAccuracyRefusal {
    /// The operation whose contract went unsatisfied.
    pub(crate) const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Why, in the shape the refusing authority reported it.
    pub(crate) const fn reason(&self) -> &ElementaryRefusalReason {
        &self.reason
    }

    /// The stable provider diagnostic code naming this refusal.
    pub(crate) const fn diagnostic_code(&self) -> &'static str {
        match self.reason {
            ElementaryRefusalReason::NoInstalledRealization => {
                "accuracy.elementary.no-installed-realization"
            }
            ElementaryRefusalReason::Unrefined { .. } => {
                "accuracy.elementary.unrefined-realization"
            }
        }
    }
}

/// The two ways an elementary accuracy requirement goes unmet.
#[derive(Clone, Debug)]
pub(crate) enum ElementaryRefusalReason {
    /// No installed realization speaks about the operation at all.
    ///
    /// ADR 0043's `Unknown` in its exact sense: not a disproved predicate but no
    /// admissible proof path, which fails closed rather than defaulting to
    /// honoured.
    NoInstalledRealization,
    /// A realization was installed and could not be proved to refine the contract.
    Unrefined {
        /// The declaring profile's versioned identity.
        declaring_profile: Arc<FactSourceProvenance>,
        /// The unproved-refinement reason, from the conservative proof relation.
        unknown: RefinementUnknown,
    },
}

/// The proof that one installed realization satisfies one resolved contract.
///
/// There is no constructor other than [`assess_elementary_accuracy`], so holding
/// one is evidence that the refinement relation proved the containment.
#[derive(Clone, Debug)]
pub(crate) struct ElementaryAccuracyAdmission {
    basis: RefinementBasis,
    discharge: ElementaryDischarge,
}

impl ElementaryAccuracyAdmission {
    /// What established the refinement.
    pub(crate) const fn basis(&self) -> &RefinementBasis {
        &self.basis
    }

    /// What each half of the realization's evidence establishes.
    pub(crate) const fn discharge(&self) -> ElementaryDischarge {
        self.discharge
    }
}

/// Decides whether some installed realization provably refines `required`.
///
/// Conservative in one direction only, exactly as `refines` is: an admission is a
/// proof, and a refusal may be a limitation of the closed algebra rather than a
/// counterexample. That asymmetry can reject a legal implementation and can never
/// admit an illegal one.
///
/// # Errors
///
/// Returns [`ElementaryAccuracyRefusal`] naming the operation, the declaring
/// profile, and the refusing reason. It is boxed because a refusal carries the
/// declaring profile's whole provenance record — every measurement context, every
/// compiler build identity — and that completeness is the point: a rejection that
/// named less would not be reproducible.
pub(crate) fn assess_elementary_accuracy(
    required: &AccuracyContract,
    installed: &[ElementaryRealization],
    registry: &RegisteredImplicationRegistry,
) -> Result<ElementaryAccuracyAdmission, Box<ElementaryAccuracyRefusal>> {
    let mut refusal = None;
    for realization in installed {
        if realization.operation() != required.operation() {
            continue;
        }
        match tiler_ir::semantic::accuracy::refines(realization.contract(), required, registry) {
            RefinementOutcome::Refines { basis } => {
                return Ok(ElementaryAccuracyAdmission {
                    basis,
                    discharge: realization.discharge(),
                });
            }
            RefinementOutcome::Unknown { reason } => {
                // The first refusal is reported, so the cause is a function of the
                // installed order rather than of which candidate happened to be
                // examined last.
                refusal.get_or_insert_with(|| ElementaryAccuracyRefusal {
                    operation: required.operation().clone(),
                    reason: ElementaryRefusalReason::Unrefined {
                        declaring_profile: Arc::clone(&realization.source),
                        unknown: reason.clone(),
                    },
                });
            }
        }
    }
    Err(Box::new(refusal.unwrap_or_else(|| {
        ElementaryAccuracyRefusal {
            operation: required.operation().clone(),
            reason: ElementaryRefusalReason::NoInstalledRealization,
        }
    })))
}

/// Returns the elementary realizations this build installs.
///
/// Two rows, one per registered family. Each Metal declaration is caller-vouched
/// in exactly the sense ADR 0076's `tiler-build` projection is: the accuracy half
/// rests on a quoted specification and the exceptional half on a bounded corpus,
/// and neither is authenticated here.
///
/// The two rows state their accuracy in *different contract forms*, which is the
/// point rather than an inconsistency: the exponential's is a ULP bound needing a
/// registered cross-metric implication, and the reciprocal square root's is a
/// faithful result set needing none.
///
/// # Panics
///
/// Panics only if this crate's own compile-time evidence records violate their
/// stated obligations.
#[must_use]
pub(crate) fn installed_elementary_realizations() -> Vec<ElementaryRealization> {
    vec![
        ElementaryRealization::new(
            silu_f32_op(),
            metal_f32_exponential_contract(),
            metal_f32_exponential_bound_evidence().expect("the normative record is well formed"),
            metal_f32_exceptional_value_evidence().expect("the empirical record is well formed"),
            super::honourability::governed_profile_source(),
        ),
        ElementaryRealization::new(
            rms_norm_f32_op(),
            metal_f32_reciprocal_square_root_contract(),
            metal_f32_reciprocal_square_root_bound_evidence()
                .expect("the normative record is well formed"),
            metal_f32_normalization_exceptional_value_evidence()
                .expect("the empirical record is well formed"),
            super::honourability::governed_profile_source(),
        ),
    ]
}

#[cfg(test)]
mod tests;
