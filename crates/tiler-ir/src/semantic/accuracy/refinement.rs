//! Refinement as a conservative proof relation, not a provider claim.
//!
//! ADR 0042: "Initial host rules cover identical normalized contracts, identical
//! reference/domain/metric predicates with tighter exact bounds, normalized
//! `AllOf`/`AnyOf` implications the closed algebra can establish, and explicitly
//! registered mathematical implications such as a correctly rounded result
//! satisfying a compatible looser bound. **Correctly rounded, faithful, and
//! one-ULP contracts are never equated by name.** Any other implication requires
//! a certificate accepted by a versioned trusted checker; absent such a checker it
//! is `Unknown` and physically infeasible."
//!
//! Three consequences are built into the types here.
//!
//! **`Unknown` is an answer, and it is a refusal.** [`RefinementOutcome`] has no
//! "probably" and no boolean. A backend whose guarantee this relation cannot
//! prove refines the contract is *infeasible*, and
//! [`RefinementOutcome::is_physically_feasible`] says so rather than leaving a
//! caller to interpret a `false`.
//!
//! **A distinct metric key is not a name to match on.** `Ulp(apple::msl-ulp@1, 4)`
//! does not imply `Ulp(tiler::ulp-reference-gap@1, 4)` merely because both say
//! four ULPs and both are called ULPs. The two definitions differ at a power of
//! two and at NaN, so the implication needs a *registered* one
//! ([`RegisteredImplication::ScaledMetric`]) carrying the derivation that
//! establishes it. [`RegisteredImplicationRegistry::standard`] deliberately
//! registers no cross-metric row: supplying one is the job of the evidence ticket
//! that reads the vendor's own definition, and until then the implication is
//! `Unknown`.
//!
//! **The three named forms stay apart.** `CorrectlyRounded`, `Faithful`, and
//! `Ulp(metric, 1)` are three obligations and this relation proves between them
//! only along registered rows, in the direction the registration states. There is
//! no path by which a faithful implementation satisfies a correctly rounded
//! contract.

use std::collections::BTreeMap;
use std::fmt;

use crate::semantic::{NormativeDefinitionRef, TypeKey};

use super::contract::{AccuracyContract, AccuracyContractForm};
use super::error::AccuracyContractError;
use super::metric::{AccuracyMetricKey, ulp_reference_gap_metric_key};
use super::predicate::{AccuracyPredicate, AccuracyPredicateView, BooleanPredicateKind};
use super::rational::ExactTolerance;

/// A canonical registered-implication identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RegisteredImplicationKey(TypeKey);

impl RegisteredImplicationKey {
    /// Creates a validated, versioned implication key.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::CanonicalBound`] for an invalid component
    /// or version.
    pub fn new(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        semantic_version: u32,
    ) -> Result<Self, AccuracyContractError> {
        Ok(Self(TypeKey::new(namespace, name, semantic_version)?))
    }
}

impl fmt::Display for RegisteredImplicationKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// One explicitly registered mathematical implication.
///
/// Every variant is a *theorem about the value sets*, not a convenience. A row is
/// admitted because someone derived it, and the derivation travels with it as a
/// [`NormativeDefinitionRef`] on [`RegisteredImplicationRegistry::register`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RegisteredImplication {
    /// A correctly rounded result satisfies any ULP bound at or above `floor`.
    CorrectlyRoundedSatisfiesUlp {
        /// The metric the bound is measured under.
        metric: AccuracyMetricKey,
        /// The least tolerance the implication reaches.
        floor: ExactTolerance,
    },
    /// A faithful result satisfies any ULP bound at or above `floor`.
    FaithfulSatisfiesUlp {
        /// The metric the bound is measured under.
        metric: AccuracyMetricKey,
        /// The least tolerance the implication reaches.
        floor: ExactTolerance,
    },
    /// A correctly rounded result satisfies a faithful contract.
    CorrectlyRoundedSatisfiesFaithful,
    /// A bound of `t` under `from` implies a bound of `factor * t` under `to`.
    ///
    /// The shape a cross-metric adoption takes. It is deliberately not a name
    /// match: registering one asserts that the two definitions of `ulp` agree up
    /// to `factor` **over the domain in use**, which is a derivation about two
    /// specifications rather than an observation that both are spelled "ULP".
    ScaledMetric {
        /// The metric the candidate's bound is stated under.
        from: AccuracyMetricKey,
        /// The metric the required bound is stated under.
        to: AccuracyMetricKey,
        /// The exact factor relating the two scales.
        factor: ExactTolerance,
    },
}

/// The immutable set of registered implications one refinement decision may use.
#[derive(Clone, Debug, Default)]
pub struct RegisteredImplicationRegistry {
    entries: BTreeMap<RegisteredImplicationKey, (RegisteredImplication, NormativeDefinitionRef)>,
}

impl RegisteredImplicationRegistry {
    /// Creates a registry with no implications at all.
    ///
    /// Useful precisely because it makes every non-identical refinement
    /// `Unknown`: a test that wants to watch the fail-closed path uses this
    /// rather than arranging for a lookup to miss.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns the implications Tiler itself derives and registers.
    ///
    /// Two rows, both about `tiler::ulp-reference-gap@1`, plus the
    /// correctly-rounded-implies-faithful row. **No cross-metric row**: adopting a
    /// vendor's ULP bound under Tiler's metric needs that vendor's own definition
    /// read and reconciled, which is evidence work that belongs to the record that
    /// quotes the specification, not a default this vocabulary can supply.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] only if Tiler's compile-time governed
    /// keys violate their own grammar.
    pub fn standard() -> Result<Self, AccuracyContractError> {
        let metric = ulp_reference_gap_metric_key();
        let mut registry = Self::empty();
        registry.register(
            RegisteredImplicationKey::new("tiler", "correctly-rounded-satisfies-ulp", 1)?,
            RegisteredImplication::CorrectlyRoundedSatisfiesUlp {
                metric: metric.clone(),
                floor: ExactTolerance::from_ratio(1, 2)?,
            },
            basis(
                "A single round-to-nearest of the exact reference minimizes |z - r| over the representable values and attains at most half the gap tiler::ulp-reference-gap@1 measures, so a correctly rounded result satisfies every ULP bound at or above one half",
            )?,
        );
        registry.register(
            RegisteredImplicationKey::new("tiler", "faithful-satisfies-ulp", 1)?,
            RegisteredImplication::FaithfulSatisfiesUlp {
                metric,
                floor: ExactTolerance::from_integer(1),
            },
            basis(
                "A faithful result is one of the two adjacent values bracketing the exact reference, so |z - r| is at most the gap between them, which tiler::ulp-reference-gap@1 measures as at most one ULP",
            )?,
        );
        registry.register(
            RegisteredImplicationKey::new("tiler", "correctly-rounded-satisfies-faithful", 1)?,
            RegisteredImplication::CorrectlyRoundedSatisfiesFaithful,
            basis(
                "Round-to-nearest returns the exact reference when it is representable and otherwise one of the two adjacent bracketing values, which is exactly the faithful result set",
            )?,
        );
        Ok(registry)
    }

    /// Registers one derived implication under its versioned key.
    ///
    /// Replacing an existing key is deliberate rather than refused: a registry is
    /// assembled by its owner, and a duplicate key means the owner stated the row
    /// twice. What cannot happen is an *unregistered* implication being used,
    /// which is the property the whole relation rests on.
    pub fn register(
        &mut self,
        key: RegisteredImplicationKey,
        implication: RegisteredImplication,
        basis: NormativeDefinitionRef,
    ) {
        self.entries.insert(key, (implication, basis));
    }

    /// Returns the registered rows, in canonical key order.
    #[must_use]
    pub fn entries(
        &self,
    ) -> impl ExactSizeIterator<
        Item = (
            &RegisteredImplicationKey,
            &RegisteredImplication,
            &NormativeDefinitionRef,
        ),
    > {
        self.entries
            .iter()
            .map(|(key, (implication, basis))| (key, implication, basis))
    }

    fn find(
        &self,
        matches: impl Fn(&RegisteredImplication) -> bool,
    ) -> Option<&RegisteredImplicationKey> {
        self.entries
            .iter()
            .find(|(_, (implication, _))| matches(implication))
            .map(|(key, _)| key)
    }
}

fn basis(text: &str) -> Result<NormativeDefinitionRef, AccuracyContractError> {
    NormativeDefinitionRef::new(text).map_err(|_| AccuracyContractError::MalformedAttribute {
        subject: super::error::AccuracyAttributeSubject::ContractRecord,
    })
}

/// Why one contract could not be proved to refine another.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RefinementUnknown {
    /// The two contracts are about different operations or dtype signatures.
    DifferentSignature,
    /// The two contracts name different immutable reference semantics.
    DifferentReferenceSemantics,
    /// The two contracts state different independent exceptional-value contracts.
    DifferentExceptionalValueContract,
    /// No registered implication relates the two contract forms in this direction.
    ///
    /// This is where "never equated by name" lands. A faithful candidate against a
    /// correctly rounded requirement reaches it, and no amount of tightening a
    /// tolerance changes that.
    NoImplicationBetweenForms {
        /// The candidate's form.
        candidate: &'static str,
        /// The required form.
        required: &'static str,
    },
    /// The two bounded contracts do not state the same domains.
    DifferentDomains,
    /// The candidate's bound is not tighter than the required bound.
    LooserExactBound,
    /// No registered implication relates the two metrics.
    ///
    /// The cross-metric case ADR 0042 names explicitly. Closing it needs a
    /// derivation registered as [`RegisteredImplication::ScaledMetric`], not a
    /// matching spelling.
    UnregisteredMetricImplication {
        /// The metric the candidate states its bound under.
        from: AccuracyMetricKey,
        /// The metric the requirement states its bound under.
        to: AccuracyMetricKey,
    },
    /// The closed Boolean algebra cannot establish the implication.
    ///
    /// ADR 0042 sends this case to "a certificate accepted by a versioned trusted
    /// checker"; no such checker exists, so it stays `Unknown`.
    NoClosedAlgebraProof,
}

impl RefinementUnknown {
    /// Returns the stable provider diagnostic code naming this outcome.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::DifferentSignature => "accuracy.refinement.different-signature",
            Self::DifferentReferenceSemantics => {
                "accuracy.refinement.different-reference-semantics"
            }
            Self::DifferentExceptionalValueContract => {
                "accuracy.refinement.different-exceptional-value-contract"
            }
            Self::NoImplicationBetweenForms { .. } => {
                "accuracy.refinement.no-implication-between-forms"
            }
            Self::DifferentDomains => "accuracy.refinement.different-domains",
            Self::LooserExactBound => "accuracy.refinement.looser-exact-bound",
            Self::UnregisteredMetricImplication { .. } => {
                "accuracy.refinement.unregistered-metric-implication"
            }
            Self::NoClosedAlgebraProof => "accuracy.refinement.no-closed-algebra-proof",
        }
    }
}

impl fmt::Display for RefinementUnknown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DifferentSignature => {
                formatter.write_str("the contracts resolve different operation or dtype signatures")
            }
            Self::DifferentReferenceSemantics => {
                formatter.write_str("the contracts name different immutable reference semantics")
            }
            Self::DifferentExceptionalValueContract => formatter
                .write_str("the contracts state different independent exceptional-value contracts"),
            Self::NoImplicationBetweenForms {
                candidate,
                required,
            } => write!(
                formatter,
                "no registered implication proves that a {candidate} contract satisfies a {required} one"
            ),
            Self::DifferentDomains => {
                formatter.write_str("the bounded contracts do not state the same domains")
            }
            Self::LooserExactBound => {
                formatter.write_str("the candidate's exact bound is not tighter than the requirement")
            }
            Self::UnregisteredMetricImplication { from, to } => write!(
                formatter,
                "a bound under {from} implies nothing under {to} without a registered implication, because a distinct metric key is not a name to match on"
            ),
            Self::NoClosedAlgebraProof => formatter.write_str(
                "the closed Boolean algebra cannot establish the implication, and no versioned trusted checker exists to accept a certificate for it",
            ),
        }
    }
}

/// How the refinement relation was established, when it was.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RefinementBasis {
    /// The two normalized contracts are identical.
    IdenticalNormalizedContract,
    /// Identical predicates with a tighter or equal exact bound.
    TighterExactBound,
    /// The closed `AllOf`/`AnyOf` algebra establishes the implication.
    ClosedAlgebraImplication,
    /// An explicitly registered mathematical implication establishes it.
    RegisteredImplication {
        /// The row that carries the derivation.
        key: RegisteredImplicationKey,
    },
}

/// The result of one conservative refinement decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefinementOutcome {
    /// The candidate's result set is contained in the requirement's, provably.
    Refines {
        /// What established it.
        basis: RefinementBasis,
    },
    /// The relation could not be proved, so the candidate is physically infeasible.
    Unknown {
        /// Why, naming what evidence would close it.
        reason: RefinementUnknown,
    },
}

impl RefinementOutcome {
    /// Returns whether a plan selecting the candidate is physically feasible.
    ///
    /// ADR 0042 makes `Unknown` "physically infeasible", so this is the same
    /// question as [`Self::Refines`] and is spelled out because the two readings
    /// have been conflated elsewhere: an unproved refinement is a hard
    /// feasibility failure with an explainable reason, not a cost.
    #[must_use]
    pub const fn is_physically_feasible(&self) -> bool {
        matches!(self, Self::Refines { .. })
    }
}

/// Decides whether `candidate` provably refines `required`.
///
/// Conservative in one direction only: a `Refines` outcome is a proof, and an
/// `Unknown` outcome may be a limitation of the closed algebra rather than a real
/// counterexample. That asymmetry is the safe one — it can reject a legal
/// implementation, never admit an illegal one.
#[must_use]
pub fn refines(
    candidate: &AccuracyContract,
    required: &AccuracyContract,
    registry: &RegisteredImplicationRegistry,
) -> RefinementOutcome {
    if candidate.operation() != required.operation()
        || candidate.operand_types() != required.operand_types()
        || candidate.result_type() != required.result_type()
    {
        return unknown(RefinementUnknown::DifferentSignature);
    }
    if candidate.reference_semantics().as_str() != required.reference_semantics().as_str() {
        return unknown(RefinementUnknown::DifferentReferenceSemantics);
    }
    if candidate.exceptional() != required.exceptional() {
        return unknown(RefinementUnknown::DifferentExceptionalValueContract);
    }
    if candidate.canonical_encoding() == required.canonical_encoding() {
        return RefinementOutcome::Refines {
            basis: RefinementBasis::IdenticalNormalizedContract,
        };
    }
    match (candidate.form(), required.form()) {
        (AccuracyContractForm::CorrectlyRounded { .. }, AccuracyContractForm::Faithful) => {
            match registry.find(|implication| {
                matches!(
                    implication,
                    RegisteredImplication::CorrectlyRoundedSatisfiesFaithful
                )
            }) {
                Some(key) => registered(key),
                None => unknown(RefinementUnknown::NoImplicationBetweenForms {
                    candidate: "correctly rounded",
                    required: "faithful",
                }),
            }
        }
        (
            AccuracyContractForm::CorrectlyRounded { .. } | AccuracyContractForm::Faithful,
            AccuracyContractForm::BoundedPiecewise(domain),
        ) => {
            let exact = matches!(
                candidate.form(),
                AccuracyContractForm::CorrectlyRounded { .. }
            );
            let Some((key, floor)) = registry.entries().find_map(|(key, implication, _)| {
                match (exact, implication) {
                    (
                        true,
                        RegisteredImplication::CorrectlyRoundedSatisfiesUlp { metric, floor },
                    )
                    | (false, RegisteredImplication::FaithfulSatisfiesUlp { metric, floor }) => {
                        Some((
                            key.clone(),
                            AccuracyPredicate::ulp(metric.clone(), floor.clone()),
                        ))
                    }
                    _ => None,
                }
            }) else {
                return unknown(RefinementUnknown::NoImplicationBetweenForms {
                    candidate: if exact {
                        "correctly rounded"
                    } else {
                        "faithful"
                    },
                    required: "bounded piecewise",
                });
            };
            for clause in domain.clauses() {
                if let Err(reason) = implies(&floor, clause.predicate(), registry) {
                    return unknown(reason);
                }
            }
            registered(&key)
        }
        (
            AccuracyContractForm::BoundedPiecewise(candidate_domain),
            AccuracyContractForm::BoundedPiecewise(required_domain),
        ) => {
            if candidate_domain.admitted() != required_domain.admitted()
                || candidate_domain.clauses().len() != required_domain.clauses().len()
            {
                return unknown(RefinementUnknown::DifferentDomains);
            }
            // Matched by region rather than by position, because a clause set is
            // unordered: two contracts stating the same clauses in a different
            // order are the same contract, and a pairwise comparison by index
            // would report a difference that does not exist. The whole operand
            // map decides the region, because two clauses agreeing on operand
            // zero and differing on operand one describe different regions, and
            // pairing them would carry a bound out of the region where it was
            // proved.
            let mut basis = RefinementBasis::TighterExactBound;
            for required_clause in required_domain.clauses() {
                let Some(candidate_clause) = candidate_domain
                    .clauses()
                    .iter()
                    .find(|clause| clause.constrains_the_same_region(required_clause))
                else {
                    return unknown(RefinementUnknown::DifferentDomains);
                };
                match implies(
                    candidate_clause.predicate(),
                    required_clause.predicate(),
                    registry,
                ) {
                    Ok(RefinementBasis::TighterExactBound) => {}
                    Ok(other) => basis = other,
                    Err(reason) => return unknown(reason),
                }
            }
            RefinementOutcome::Refines { basis }
        }
        (candidate_form, required_form) => unknown(RefinementUnknown::NoImplicationBetweenForms {
            candidate: form_name(candidate_form),
            required: form_name(required_form),
        }),
    }
}

const fn form_name(form: &AccuracyContractForm) -> &'static str {
    match form {
        AccuracyContractForm::CorrectlyRounded { .. } => "correctly rounded",
        AccuracyContractForm::Faithful => "faithful",
        AccuracyContractForm::BoundedPiecewise(_) => "bounded piecewise",
        AccuracyContractForm::NamedElementary { .. } => "named elementary",
    }
}

const fn unknown(reason: RefinementUnknown) -> RefinementOutcome {
    RefinementOutcome::Unknown { reason }
}

fn registered(key: &RegisteredImplicationKey) -> RefinementOutcome {
    RefinementOutcome::Refines {
        basis: RefinementBasis::RegisteredImplication { key: key.clone() },
    }
}

/// Decides whether satisfying `candidate` implies satisfying `required`.
///
/// The closed algebra, written out. Every rule is a containment of result sets:
/// a tighter tolerance of the same shape, a Boolean rule that follows from the
/// meaning of the combinator, or a registered cross-metric row. Nothing here
/// infers an implication from two predicates merely resembling each other.
fn implies(
    candidate: &AccuracyPredicate,
    required: &AccuracyPredicate,
    registry: &RegisteredImplicationRegistry,
) -> Result<RefinementBasis, RefinementUnknown> {
    // The Boolean rules come first and in this order because they decompose the
    // problem: a conjunction on the *required* side must hold in every member,
    // and a disjunction on the *candidate* side must be discharged by every
    // member, so both are stronger obligations than their duals.
    if let AccuracyPredicateView::Boolean {
        kind: BooleanPredicateKind::AllOf,
        members,
    } = required.view()
    {
        for member in members {
            implies(candidate, member, registry)?;
        }
        return Ok(RefinementBasis::ClosedAlgebraImplication);
    }
    if let AccuracyPredicateView::Boolean {
        kind: BooleanPredicateKind::AnyOf,
        members,
    } = candidate.view()
    {
        for member in members {
            implies(member, required, registry)?;
        }
        return Ok(RefinementBasis::ClosedAlgebraImplication);
    }
    if let AccuracyPredicateView::Boolean {
        kind: BooleanPredicateKind::AllOf,
        members,
    } = candidate.view()
        && members
            .iter()
            .any(|member| implies(member, required, registry).is_ok())
    {
        return Ok(RefinementBasis::ClosedAlgebraImplication);
    }
    if let AccuracyPredicateView::Boolean {
        kind: BooleanPredicateKind::AnyOf,
        members,
    } = required.view()
        && members
            .iter()
            .any(|member| implies(candidate, member, registry).is_ok())
    {
        return Ok(RefinementBasis::ClosedAlgebraImplication);
    }

    match (candidate.view(), required.view()) {
        // Four containments with one proof obligation each: a tighter tolerance
        // of the same shape. The absolute-into-additive and relative-into-additive
        // rows join them because `a + q|r|` dominates either term alone, so the
        // same comparison discharges all four.
        (
            AccuracyPredicateView::Absolute { tolerance: left },
            AccuracyPredicateView::Absolute { tolerance: right }
            | AccuracyPredicateView::AbsoluteRelative {
                absolute: right, ..
            },
        )
        | (
            AccuracyPredicateView::Relative { tolerance: left },
            AccuracyPredicateView::Relative { tolerance: right }
            | AccuracyPredicateView::AbsoluteRelative {
                relative: right, ..
            },
        ) => tighter(left, right),
        (
            AccuracyPredicateView::AbsoluteRelative {
                absolute: left_absolute,
                relative: left_relative,
            },
            AccuracyPredicateView::AbsoluteRelative {
                absolute: right_absolute,
                relative: right_relative,
            },
        ) => {
            tighter(left_absolute, right_absolute)?;
            tighter(left_relative, right_relative)
        }
        (
            AccuracyPredicateView::Ulp {
                metric: left_metric,
                tolerance: left,
            },
            AccuracyPredicateView::Ulp {
                metric: right_metric,
                tolerance: right,
            },
        ) => {
            if left_metric == right_metric {
                return tighter(left, right);
            }
            // A distinct metric key needs a registered implication, never a name
            // match. The factor converts the candidate's bound into the
            // requirement's scale before the comparison.
            let Some((key, factor)) =
                registry
                    .entries()
                    .find_map(|(key, implication, _)| match implication {
                        RegisteredImplication::ScaledMetric { from, to, factor }
                            if from == left_metric && to == right_metric =>
                        {
                            Some((key.clone(), factor.clone()))
                        }
                        _ => None,
                    })
            else {
                return Err(RefinementUnknown::UnregisteredMetricImplication {
                    from: left_metric.clone(),
                    to: right_metric.clone(),
                });
            };
            let converted = left.value().multiply(factor.value());
            if converted <= *right.value() {
                Ok(RefinementBasis::RegisteredImplication { key })
            } else {
                Err(RefinementUnknown::LooserExactBound)
            }
        }
        _ => Err(RefinementUnknown::NoClosedAlgebraProof),
    }
}

fn tighter(
    candidate: &ExactTolerance,
    required: &ExactTolerance,
) -> Result<RefinementBasis, RefinementUnknown> {
    if candidate <= required {
        Ok(RefinementBasis::TighterExactBound)
    } else {
        Err(RefinementUnknown::LooserExactBound)
    }
}
