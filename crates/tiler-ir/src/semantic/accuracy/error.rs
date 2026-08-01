//! The one typed refusal every accuracy-contract rule reports under.
//!
//! One enum rather than one per submodule, following the precedent
//! [`crate::semantic::ContractionStructureError`] sets: a caller reads *which
//! rule* refused from a stable code rather than by matching on a message, and a
//! rule that has no variant here has no way to refuse — which is the property
//! that makes the vocabulary's refusals enumerable.
//!
//! Exact-arithmetic and dtype-capability failures keep their own types
//! ([`ExactRationalError`], [`UlpFormatError`], [`UlpMetricError`]) and are
//! carried as sources, because they are refusals of a *different subject*: a
//! tolerance that is not a number and a dtype that has no adjacent-value
//! behaviour are not contract-shape rules, and collapsing them would lose the
//! distinction a reader needs to know where to look.

use std::error::Error;
use std::fmt;

use crate::semantic::TypeIdentityError;

use super::domain::{OperandOrdinal, ReferenceResultClass};
use super::metric::{AccuracyMetricKey, UlpFormatError, UlpMetricError};
use super::predicate::BooleanPredicateKind;
use super::rational::{ExactRational, ExactRationalError};

/// Which part of a malformed accuracy attribute was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AccuracyAttributeSubject {
    /// The attribute was not the accuracy-contract record.
    ContractRecord,
    /// The contract's form discriminator was absent or not one of the four forms.
    ContractForm,
    /// The attribute was not an exact-rational record.
    ExactRational,
    /// The attribute was not a metric-key record.
    MetricKey,
    /// The attribute was not a predicate record.
    PredicateRecord,
    /// The predicate's kind discriminator was absent or unrecognized.
    PredicateKind,
    /// The predicate's member sequence was malformed.
    PredicateMembers,
    /// The attribute was not a domain-clause record.
    DomainClauseRecord,
    /// A domain interval endpoint was malformed.
    DomainBound,
    /// A reference-result class was absent or unrecognized.
    ReferenceResultClass,
    /// The exceptional-value contract record was malformed.
    ExceptionalValueContract,
    /// A named-elementary profile descriptor digest was malformed.
    NamedElementaryDescriptor,
    /// The rounding rule was absent or unrecognized.
    RoundingRule,
}

impl fmt::Display for AccuracyAttributeSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ContractRecord => "accuracy-contract record",
            Self::ContractForm => "contract form",
            Self::ExactRational => "exact-rational record",
            Self::MetricKey => "metric-key record",
            Self::PredicateRecord => "predicate record",
            Self::PredicateKind => "predicate kind",
            Self::PredicateMembers => "predicate member sequence",
            Self::DomainClauseRecord => "domain-clause record",
            Self::DomainBound => "domain interval bound",
            Self::ReferenceResultClass => "reference-result class",
            Self::ExceptionalValueContract => "exceptional-value contract record",
            Self::NamedElementaryDescriptor => "named-elementary descriptor digest",
            Self::RoundingRule => "rounding rule",
        })
    }
}

/// Why the verifier could not establish that a contract admits any result.
///
/// ADR 0042 requires verification to reject a contract "when it cannot establish"
/// that the composed result set is nonempty for every admitted input. These are
/// the reasons it cannot, each naming what evidence would close it — which is
/// what separates "this contract is unsatisfiable" from "this checker is too
/// weak", two conclusions a caller acts on differently.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UnestablishedResultSet {
    /// A ULP tolerance below one half admits no candidate at every reference.
    ///
    /// Round-to-nearest minimizes `|z - r|` over the representable values, and it
    /// attains exactly `ulp(r)/2` at a midpoint. A ULP bound below one half is
    /// therefore violated by *every* candidate at some admitted reference — this
    /// is a property of the contract, not a limitation of the check.
    UlpToleranceBelowRoundingFloor {
        /// The tolerance the clause stated.
        tolerance: ExactRational,
    },
    /// An absolute bound needs a bound on the reference magnitude and the clause states none.
    ///
    /// Closes with a justified reference-result interval on the clause: an
    /// absolute tolerance is satisfiable only where the reference is small enough
    /// that the format's own spacing fits inside it, and nothing can decide that
    /// without knowing how large the reference gets.
    AbsoluteBoundWithoutReferenceMagnitude {
        /// The tolerance the clause stated.
        tolerance: ExactRational,
    },
    /// An absolute bound is narrower than half the spacing at the largest admitted reference.
    AbsoluteBoundBelowSpacing {
        /// The tolerance the clause stated.
        tolerance: ExactRational,
        /// Half the format's spacing at the largest admitted reference magnitude.
        required: ExactRational,
    },
    /// A relative bound needs a lower bound on the reference magnitude and the clause states none.
    RelativeBoundWithoutReferenceMagnitude {
        /// The tolerance the clause stated.
        tolerance: ExactRational,
    },
    /// A relative bound is narrower than the format's worst-case rounding ratio.
    RelativeBoundBelowRoundingRatio {
        /// The tolerance the clause stated.
        tolerance: ExactRational,
        /// The largest ratio round-to-nearest can produce over the admitted references.
        required: ExactRational,
    },
    /// Every member of a disjunction failed for its own reason.
    NoDisjunctFinishedEstablished,
    /// The clause's predicate names a metric this build does not define.
    UnregisteredMetric {
        /// The metric the clause named.
        metric: AccuracyMetricKey,
    },
}

impl UnestablishedResultSet {
    /// Returns the stable provider diagnostic code naming this reason.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::UlpToleranceBelowRoundingFloor { .. } => {
                "accuracy.result-set.ulp-tolerance-below-rounding-floor"
            }
            Self::AbsoluteBoundWithoutReferenceMagnitude { .. } => {
                "accuracy.result-set.absolute-bound-without-reference-magnitude"
            }
            Self::AbsoluteBoundBelowSpacing { .. } => {
                "accuracy.result-set.absolute-bound-below-spacing"
            }
            Self::RelativeBoundWithoutReferenceMagnitude { .. } => {
                "accuracy.result-set.relative-bound-without-reference-magnitude"
            }
            Self::RelativeBoundBelowRoundingRatio { .. } => {
                "accuracy.result-set.relative-bound-below-rounding-ratio"
            }
            Self::NoDisjunctFinishedEstablished => "accuracy.result-set.no-disjunct-established",
            Self::UnregisteredMetric { .. } => "accuracy.result-set.unregistered-metric",
        }
    }
}

impl fmt::Display for UnestablishedResultSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UlpToleranceBelowRoundingFloor { tolerance } => write!(
                formatter,
                "a ULP tolerance of {tolerance} is below the one-half floor round-to-nearest attains, so no candidate satisfies it at every admitted reference"
            ),
            Self::AbsoluteBoundWithoutReferenceMagnitude { tolerance } => write!(
                formatter,
                "an absolute tolerance of {tolerance} needs a justified upper bound on the reference magnitude and the clause states none"
            ),
            Self::AbsoluteBoundBelowSpacing {
                tolerance,
                required,
            } => write!(
                formatter,
                "an absolute tolerance of {tolerance} is below the {required} half-spacing at the largest admitted reference"
            ),
            Self::RelativeBoundWithoutReferenceMagnitude { tolerance } => write!(
                formatter,
                "a relative tolerance of {tolerance} needs a justified lower bound on the reference magnitude and the clause states none"
            ),
            Self::RelativeBoundBelowRoundingRatio {
                tolerance,
                required,
            } => write!(
                formatter,
                "a relative tolerance of {tolerance} is below the {required} worst-case rounding ratio over the admitted references"
            ),
            Self::NoDisjunctFinishedEstablished => formatter.write_str(
                "no member of the disjunction admits a candidate at every admitted reference",
            ),
            Self::UnregisteredMetric { metric } => write!(
                formatter,
                "the clause measures under {metric}, which this build does not define"
            ),
        }
    }
}

impl Error for UnestablishedResultSet {}

/// A typed refusal of one accuracy contract, predicate, or domain.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AccuracyContractError {
    /// A Boolean predicate collection was empty.
    EmptyPredicateCollection {
        /// The Boolean kind that was empty.
        kind: BooleanPredicateKind,
    },
    /// A Boolean predicate collection exceeded the canonical member bound.
    TooManyPredicateMembers {
        /// First rejected member count.
        members: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// A predicate exceeded the canonical nesting bound.
    PredicateTooDeep {
        /// Rejected depth.
        depth: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// A predicate exceeded the canonical total-node bound.
    TooManyPredicateNodes {
        /// Rejected node count.
        nodes: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// A decoded Boolean collection directly contained a collection of its own kind.
    ///
    /// Refused rather than flattened on the decode path, following the
    /// [`crate::semantic::ContractionStructureError::NonCanonicalNumbering`]
    /// precedent: the encoding is the identity, so admitting a second spelling of
    /// one normalized predicate would give one predicate two identities.
    NonCanonicalPredicateNesting {
        /// The Boolean kind whose members were not flattened.
        kind: BooleanPredicateKind,
    },
    /// A decoded Boolean collection's members were not in canonical encoding order.
    NonCanonicalPredicateOrder {
        /// The Boolean kind whose members were unsorted.
        kind: BooleanPredicateKind,
    },
    /// A decoded Boolean collection repeated a member.
    DuplicatePredicateMember {
        /// The Boolean kind that repeated a member.
        kind: BooleanPredicateKind,
    },
    /// A decoded Boolean collection carried exactly one member.
    ///
    /// A singleton canonicalizes to its member, so an encoded one is a second
    /// spelling of that member rather than a distinct predicate.
    NonCanonicalPredicateSingleton {
        /// The Boolean kind that carried one member.
        kind: BooleanPredicateKind,
    },
    /// A relative predicate is undefined at the zero reference its clause admits.
    ///
    /// The definedness rule applies **recursively**, so a `Relative` buried under
    /// an `AnyOf` reaches this refusal exactly as a bare one does. ADR 0042 is
    /// explicit that `AnyOf` "cannot hide an undefined relative predicate at
    /// reference zero"; there is no hidden epsilon and no silently-true
    /// disjunct.
    UndefinedRelativePredicateAtZeroReference,
    /// A domain interval admits no value.
    EmptyDomainInterval {
        /// The operand whose interval was empty.
        operand: OperandOrdinal,
    },
    /// A bounded contract supplied no domain clauses.
    EmptyClauseSet,
    /// A bounded contract exceeded the canonical clause bound.
    TooManyDomainClauses {
        /// First rejected clause count.
        clauses: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// A clause asserted a reference-result class with no operation-specific proof.
    ///
    /// ADR 0042: "An input-domain predicate may justify a reference-result class
    /// only through an operation-specific proof." An unjustified class would let a
    /// contract assume its own reference is nonzero, which is exactly the
    /// assumption the relative predicate's definedness rule exists to check.
    UnjustifiedReferenceResultClass {
        /// The class the clause asserted.
        class: ReferenceResultClass,
    },
    /// The clauses do not cover the operation's admitted ordinary input domain.
    IncompleteDomainCoverage {
        /// A witness point the clauses leave uncovered.
        witness: Vec<ExactRational>,
    },
    /// Coverage could not be decided within the governed budget.
    ///
    /// An unverifiable gap, in ADR 0042's sense: the checker does not know the
    /// domain is covered, and "does not know" fails closed rather than passing.
    CoverageNotVerifiable {
        /// Cells the decomposition would have required.
        cells: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// The composed observable result set could not be established as nonempty.
    EmptyComposedResultSet {
        /// The exact reason, naming what evidence would close it.
        reason: UnestablishedResultSet,
    },
    /// The result dtype cannot carry the metric the contract measures under.
    IncompatibleResultDtype(UlpFormatError),
    /// The metric is undefined at a value the contract admits.
    UndefinedMetric(UlpMetricError),
    /// An exact number was not a valid exact number.
    ExactNumber(ExactRationalError),
    /// The attribute was not a well-formed accuracy record.
    MalformedAttribute {
        /// The rejected part.
        subject: AccuracyAttributeSubject,
    },
    /// The contract exceeded a canonical structural bound.
    CanonicalBound(TypeIdentityError),
}

impl AccuracyContractError {
    /// Returns the stable provider diagnostic code naming this refusal.
    ///
    /// Each rule has its own code, so a caller reads which rule refused from the
    /// code rather than by matching on a message.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::EmptyPredicateCollection { .. } => "accuracy.predicate.empty-collection",
            Self::TooManyPredicateMembers { .. } => "accuracy.predicate.too-many-members",
            Self::PredicateTooDeep { .. } => "accuracy.predicate.too-deep",
            Self::TooManyPredicateNodes { .. } => "accuracy.predicate.too-many-nodes",
            Self::NonCanonicalPredicateNesting { .. } => "accuracy.predicate.non-canonical-nesting",
            Self::NonCanonicalPredicateOrder { .. } => "accuracy.predicate.non-canonical-order",
            Self::DuplicatePredicateMember { .. } => "accuracy.predicate.duplicate-member",
            Self::NonCanonicalPredicateSingleton { .. } => {
                "accuracy.predicate.non-canonical-singleton"
            }
            Self::UndefinedRelativePredicateAtZeroReference => {
                "accuracy.predicate.undefined-relative-at-zero-reference"
            }
            Self::EmptyDomainInterval { .. } => "accuracy.domain.empty-interval",
            Self::EmptyClauseSet => "accuracy.domain.empty-clause-set",
            Self::TooManyDomainClauses { .. } => "accuracy.domain.too-many-clauses",
            Self::UnjustifiedReferenceResultClass { .. } => {
                "accuracy.domain.unjustified-reference-result-class"
            }
            Self::IncompleteDomainCoverage { .. } => "accuracy.domain.incomplete-coverage",
            Self::CoverageNotVerifiable { .. } => "accuracy.domain.coverage-not-verifiable",
            Self::EmptyComposedResultSet { .. } => "accuracy.contract.empty-composed-result-set",
            Self::IncompatibleResultDtype(_) => "accuracy.contract.incompatible-result-dtype",
            Self::UndefinedMetric(_) => "accuracy.contract.undefined-metric",
            Self::ExactNumber(_) => "accuracy.contract.invalid-exact-number",
            Self::MalformedAttribute { .. } => "accuracy.contract.malformed-attribute",
            Self::CanonicalBound(_) => "accuracy.contract.canonical-bound",
        }
    }
}

impl fmt::Display for AccuracyContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPredicateCollection { kind } => {
                write!(formatter, "an empty {kind} collection is invalid")
            }
            Self::TooManyPredicateMembers { members, limit } => write!(
                formatter,
                "a Boolean predicate names {members} members, exceeding {limit}"
            ),
            Self::PredicateTooDeep { depth, limit } => write!(
                formatter,
                "a predicate nests {depth} levels deep, exceeding {limit}"
            ),
            Self::TooManyPredicateNodes { nodes, limit } => write!(
                formatter,
                "a predicate carries {nodes} nodes, exceeding {limit}"
            ),
            Self::NonCanonicalPredicateNesting { kind } => write!(
                formatter,
                "a {kind} directly contains a {kind}, which normalization flattens, so the encoding is not canonical"
            ),
            Self::NonCanonicalPredicateOrder { kind } => write!(
                formatter,
                "a {kind}'s members are not sorted by canonical encoding"
            ),
            Self::DuplicatePredicateMember { kind } => {
                write!(formatter, "a {kind} repeats a member")
            }
            Self::NonCanonicalPredicateSingleton { kind } => write!(
                formatter,
                "a {kind} carries one member, which canonicalizes to that member"
            ),
            Self::UndefinedRelativePredicateAtZeroReference => formatter.write_str(
                "a relative predicate is undefined at a zero reference and its clause does not exclude one",
            ),
            Self::EmptyDomainInterval { operand } => {
                write!(formatter, "the interval for operand {operand} admits no value")
            }
            Self::EmptyClauseSet => {
                formatter.write_str("a bounded piecewise contract requires at least one clause")
            }
            Self::TooManyDomainClauses { clauses, limit } => write!(
                formatter,
                "the contract states {clauses} clauses, exceeding {limit}"
            ),
            Self::UnjustifiedReferenceResultClass { class } => write!(
                formatter,
                "the clause asserts the {class} reference-result class without an operation-specific proof"
            ),
            Self::IncompleteDomainCoverage { witness } => {
                formatter.write_str("the clauses leave the admitted input domain uncovered at (")?;
                for (position, coordinate) in witness.iter().enumerate() {
                    if position > 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{coordinate}")?;
                }
                formatter.write_str(")")
            }
            Self::CoverageNotVerifiable { cells, limit } => write!(
                formatter,
                "deciding coverage would need {cells} cells, exceeding {limit}, so the gap is unverifiable rather than absent"
            ),
            Self::EmptyComposedResultSet { reason } => write!(
                formatter,
                "the composed observable result set was not established as nonempty: {reason}"
            ),
            Self::IncompatibleResultDtype(source) => {
                write!(formatter, "the result dtype cannot carry the metric: {source}")
            }
            Self::UndefinedMetric(source) => {
                write!(formatter, "the metric is undefined on this contract: {source}")
            }
            Self::ExactNumber(source) => write!(formatter, "an exact number is invalid: {source}"),
            Self::MalformedAttribute { subject } => {
                write!(formatter, "the {subject} is malformed")
            }
            Self::CanonicalBound(source) => {
                write!(formatter, "the contract exceeds a canonical bound: {source}")
            }
        }
    }
}

impl Error for AccuracyContractError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EmptyComposedResultSet { reason } => Some(reason),
            Self::IncompatibleResultDtype(source) => Some(source),
            Self::UndefinedMetric(source) => Some(source),
            Self::ExactNumber(source) => Some(source),
            Self::CanonicalBound(source) => Some(source),
            _ => None,
        }
    }
}

impl From<ExactRationalError> for AccuracyContractError {
    fn from(value: ExactRationalError) -> Self {
        Self::ExactNumber(value)
    }
}

impl From<TypeIdentityError> for AccuracyContractError {
    fn from(value: TypeIdentityError) -> Self {
        Self::CanonicalBound(value)
    }
}

impl From<UlpFormatError> for AccuracyContractError {
    fn from(value: UlpFormatError) -> Self {
        Self::IncompatibleResultDtype(value)
    }
}

impl From<UlpMetricError> for AccuracyContractError {
    fn from(value: UlpMetricError) -> Self {
        Self::UndefinedMetric(value)
    }
}

pub(super) fn malformed(subject: AccuracyAttributeSubject) -> AccuracyContractError {
    AccuracyContractError::MalformedAttribute { subject }
}
