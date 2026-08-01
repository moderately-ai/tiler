//! The typed transcendental accuracy-contract vocabulary of ADR 0042.
//!
//! ADR 0016 requires every transcendental operation to carry a resolved,
//! operation-specific accuracy contract and leaves the vocabulary open; ADR 0042
//! fixes it. This module is that vocabulary. It registers no operation, admits no
//! key, and selects no profile tuple — it is the carrier those three things need,
//! and Milestone 1 makes it a precondition of *registration* rather than of
//! execution, because a key admitted without its accuracy contract would not have
//! a partial identity but a wrong one.
//!
//! # Reading order
//!
//! | module | subject |
//! | --- | --- |
//! | [`rational`](crate::semantic::accuracy::rational) | exact rational arithmetic and the nonnegative tolerance |
//! | [`metric`](crate::semantic::accuracy::metric) | the versioned ULP metric key and the dtype capability it needs |
//! | [`predicate`](crate::semantic::accuracy::predicate) | the generic bounded predicates and their normalization |
//! | [`domain`](crate::semantic::accuracy::domain) | the accuracy-domain clause language, coverage, and intersection |
//! | [`contract`](crate::semantic::accuracy::contract) | the four contract forms and the five-step composition |
//! | [`refinement`](crate::semantic::accuracy::refinement) | refinement as a conservative proof relation |
//! | [`evidence`](crate::semantic::accuracy::evidence) | classified conformance evidence and what it may discharge |
//! | [`error`](crate::semantic::accuracy::error) | the one typed refusal every rule reports under |
//!
//! # What holds this together
//!
//! **Nothing here is a name.** A bare `fast`, `approximate`, or `precise` label
//! is what ADR 0042 exists to replace, and the replacement is not a richer
//! vocabulary of labels: every claim in this module is either an exact number, a
//! versioned key with a stated definition, or a decided predicate. Where a claim
//! cannot be decided — a dtype whose adjacent-value behaviour is not derivable, a
//! cross-metric implication nobody derived, a coverage question too large for the
//! budget — the answer is a typed refusal or `Unknown`, never a default.
//!
//! **Four maturity claims stay distinct.** This module is a *tested
//! implementation* of the vocabulary. It is not an accepted public boundary
//! (that is Tom's), it does not make any operation admissible, and it establishes
//! no fact about any target: the only thing it can say about a backend is whether
//! a stated guarantee provably refines a stated contract, and it answers
//! `Unknown` whenever it cannot prove one.
//!
//! # A worked example
//!
//! The shape the three L3′ verticals need: a constant ULP bound on an
//! exponential's whole ordinary domain, with the reference proved strictly
//! positive so a relative clause would be defined there.
//!
//! ```
//! use tiler_ir::semantic::accuracy::{
//!     AccuracyContract, AccuracyContractForm, AccuracyDomain, AccuracyDomainClause,
//!     AccuracyPredicate, DomainBound, DomainInterval, DomainErrorRule, ExactTolerance,
//!     ExceptionalValueContract, FiniteOverflowRule, InfiniteReferenceRule, NanReferenceRule,
//!     OperandOrdinal, ReferenceResultClass, ReferenceResultConstraint,
//!     ulp_reference_gap_metric_key,
//! };
//! use tiler_ir::semantic::{F32, NormativeDefinitionRef, OpKey, builtin_scalar_value_type_facts};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let whole_domain = DomainInterval::unbounded();
//! let clause = AccuracyDomainClause::new(
//!     [(OperandOrdinal::new(0), whole_domain.clone())],
//!     ReferenceResultConstraint::new(
//!         [ReferenceResultClass::Positive],
//!         None,
//!         Some(NormativeDefinitionRef::new("exp is strictly positive on the reals")?),
//!     )?,
//!     AccuracyPredicate::ulp(ulp_reference_gap_metric_key(), ExactTolerance::from_integer(4)),
//! )?;
//! let contract = AccuracyContract::new(
//!     OpKey::new("example", "exp-f32", 1)?,
//!     vec![F32::resolved_type()],
//!     F32::resolved_type(),
//!     NormativeDefinitionRef::new("the exponential function on the reals")?,
//!     AccuracyContractForm::BoundedPiecewise(AccuracyDomain::new([whole_domain], [clause])?),
//!     ExceptionalValueContract::new(
//!         NanReferenceRule::CanonicalNan,
//!         InfiniteReferenceRule::SignedInfinity,
//!         DomainErrorRule::CanonicalNan,
//!         FiniteOverflowRule::SignedInfinity,
//!     ),
//! );
//!
//! // Verification decides coverage, definedness, metric compatibility, and that
//! // the composed result set is nonempty at every admitted input.
//! let facts = builtin_scalar_value_type_facts(&F32::resolved_type())
//!     .expect("f32 is a governed built-in scalar");
//! let verified = contract.verify(&facts)?;
//! assert_eq!(verified.contract().canonical_encoding(), contract.canonical_encoding());
//! # Ok(())
//! # }
//! ```

/// The four discriminated contract forms and the five-step composition.
pub mod contract;
/// The accuracy-domain clause language, its coverage rules, and intersection semantics.
pub mod domain;
/// The one typed refusal every accuracy-contract rule reports under.
pub mod error;
/// Classified conformance evidence and what each class may discharge.
pub mod evidence;
/// The versioned ULP metric key and the dtype capability it requires.
pub mod metric;
/// The generic bounded accuracy predicates and their normalization.
pub mod predicate;
/// Exact rational arithmetic and the nonnegative tolerance built on it.
pub mod rational;
/// Refinement as a conservative proof relation.
pub mod refinement;

pub use contract::{
    AccuracyContract, AccuracyContractForm, CanonicalAccuracyContract, CompositionStep,
    DomainErrorRule, ExceptionalValueContract, FiniteOverflowRule, InfiniteReferenceRule,
    MAX_NAMED_ELEMENTARY_DIGEST_BYTES, NamedElementaryDescriptorDigest, NamedElementaryProfileKey,
    NanReferenceRule, ReferenceRoundingRule, ResultSetEstablishment, VerifiedAccuracyContract,
    correctly_rounded_ulp_bound,
};
pub use domain::{
    AccuracyDomain, AccuracyDomainClause, CoveredCell, DomainBound, DomainInterval,
    MAX_ACCURACY_DOMAIN_CLAUSES, MAX_ACCURACY_DOMAIN_COVERAGE_CELLS, MAX_ACCURACY_DOMAIN_OPERANDS,
    OperandOrdinal, ReferenceResultClass, ReferenceResultConstraint,
};
pub use error::{AccuracyAttributeSubject, AccuracyContractError, UnestablishedResultSet};
pub use evidence::{
    ConformanceEvidence, ConformanceEvidenceClass, ConformanceEvidenceError, HardAccuracyDischarge,
    MAX_CONFORMANCE_EVIDENCE_DIGEST_BYTES,
};
pub use metric::{
    AccuracyMetricKey, UlpFormat, UlpFormatError, UlpMetricError, ulp_metric_format_rules,
    ulp_reference_gap_metric_key,
};
pub use predicate::{
    AccuracyPredicate, AccuracyPredicateView, BooleanPredicateKind, MAX_ACCURACY_PREDICATE_DEPTH,
    MAX_ACCURACY_PREDICATE_MEMBERS, MAX_ACCURACY_PREDICATE_NODES,
};
pub use rational::{
    ExactRational, ExactRationalError, ExactSign, ExactTolerance,
    MAX_EXACT_RATIONAL_MAGNITUDE_BYTES,
};
pub use refinement::{
    RefinementBasis, RefinementOutcome, RefinementUnknown, RegisteredImplication,
    RegisteredImplicationKey, RegisteredImplicationRegistry, refines,
};

#[cfg(test)]
mod tests;
