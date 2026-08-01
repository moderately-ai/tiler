//! The versioned accuracy-domain predicate language, and its coverage rules.
//!
//! ADR 0042: "A bounded contract's clauses use a dedicated versioned
//! accuracy-domain predicate language over all exact input operands and typed
//! reference-result classes such as finite and nonzero. […] Clauses must cover
//! the complete ordinary input domain admitted by the operation. Every matching
//! clause applies, so overlap means intersection rather than priority;
//! unverifiable gaps or a possibly empty intersection reject the contract."
//!
//! Three properties of that paragraph shape everything below.
//!
//! **Intersection, not priority.** A clause set is unordered. At any admitted
//! point the applicable clauses are *all* of the ones whose domain contains it,
//! and the accuracy obligation there is their conjunction. There is no first
//! match, no fallback, and no way for clause order to change what a contract
//! means — which is what lets a genuinely piecewise vendor specification be
//! transcribed without inventing an ordering the vendor did not state.
//!
//! **Coverage is decided, not asserted.** [`AccuracyDomain::verify_coverage`]
//! decomposes the admitted domain at every clause endpoint and tests one
//! representative per cell. That is exact rather than sampled: every clause
//! endpoint is a breakpoint, so no clause boundary falls inside a cell, so a
//! clause containing one interior point of a cell contains the whole cell. An
//! uncovered cell yields a witness point; a decomposition too large for the
//! governed budget yields [`AccuracyContractError::CoverageNotVerifiable`],
//! because an undecided gap is an unverifiable one and fails closed.
//!
//! **A reference-result class needs a proof.** ADR 0042: "An input-domain
//! predicate may justify a reference-result class only through an
//! operation-specific proof." Nothing here can derive `r != 0` from a statement
//! about `x`, because nothing here knows what the operation computes. So a clause
//! that asserts a class or a reference magnitude carries the reference to the
//! proof that established it, and one that asserts a class without a proof is
//! refused rather than believed.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::identity::{push_len, push_slice};
use crate::semantic::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueView, NormativeDefinitionRef,
};

use super::error::{AccuracyAttributeSubject, AccuracyContractError, malformed};
use super::predicate::AccuracyPredicate;
use super::rational::ExactRational;

/// Maximum clauses one bounded piecewise contract may state.
pub const MAX_ACCURACY_DOMAIN_CLAUSES: usize = 16;
/// Maximum operands one accuracy domain may constrain.
pub const MAX_ACCURACY_DOMAIN_OPERANDS: usize = 4;
/// Maximum decomposition cells coverage verification may examine.
///
/// A budget rather than an unbounded sweep, because the decomposition is a
/// product over operands and a caller supplies both the clause count and the
/// arity. Exceeding it is a refusal, never a truncated check that reports the
/// part it managed to examine.
pub const MAX_ACCURACY_DOMAIN_COVERAGE_CELLS: usize = 4_096;

/// Domain separator of a canonical accuracy-domain encoding.
const ACCURACY_DOMAIN_DOMAIN: &[u8] = b"tiler.accuracy-domain.v1\0";

const BOUND_KIND: AttributeFieldId = AttributeFieldId::new(1);
const BOUND_VALUE: AttributeFieldId = AttributeFieldId::new(2);
const INTERVAL_LOWER: AttributeFieldId = AttributeFieldId::new(1);
const INTERVAL_UPPER: AttributeFieldId = AttributeFieldId::new(2);
const OPERAND_BINDING_ORDINAL: AttributeFieldId = AttributeFieldId::new(1);
const OPERAND_BINDING_INTERVAL: AttributeFieldId = AttributeFieldId::new(2);
const REFERENCE_CLASSES: AttributeFieldId = AttributeFieldId::new(1);
const REFERENCE_MAGNITUDE: AttributeFieldId = AttributeFieldId::new(2);
const REFERENCE_JUSTIFICATION: AttributeFieldId = AttributeFieldId::new(3);
const CLAUSE_OPERANDS: AttributeFieldId = AttributeFieldId::new(1);
const CLAUSE_REFERENCE: AttributeFieldId = AttributeFieldId::new(2);
const CLAUSE_PREDICATE: AttributeFieldId = AttributeFieldId::new(3);
const DOMAIN_ADMITTED: AttributeFieldId = AttributeFieldId::new(1);
const DOMAIN_CLAUSES: AttributeFieldId = AttributeFieldId::new(2);

/// The position of one input operand in an operation's signature.
///
/// A newtype rather than a bare `usize` for the reason
/// [`crate::semantic::ContractionIndex`] records: an operand position, a
/// contraction index, an axis, and an extent are four domains whose
/// representations happen to be primitive, and mixing them is a defect the type
/// system should catch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperandOrdinal(u32);

impl OperandOrdinal {
    /// Creates an operand ordinal.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the fixed-width position.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for OperandOrdinal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// One endpoint of an exact domain interval.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DomainBound {
    /// The interval extends without limit in this direction.
    Unbounded,
    /// The endpoint is excluded.
    Open(ExactRational),
    /// The endpoint is included.
    Closed(ExactRational),
}

impl DomainBound {
    /// Returns the finite endpoint value, when the bound has one.
    #[must_use]
    pub const fn value(&self) -> Option<&ExactRational> {
        match self {
            Self::Unbounded => None,
            Self::Open(value) | Self::Closed(value) => Some(value),
        }
    }

    const fn kind(&self) -> &'static str {
        match self {
            Self::Unbounded => "unbounded",
            Self::Open(_) => "open",
            Self::Closed(_) => "closed",
        }
    }

    fn encode(&self, output: &mut Vec<u8>) {
        match self {
            Self::Unbounded => output.push(1),
            Self::Open(value) => {
                output.push(2);
                value.encode(output);
            }
            Self::Closed(value) => {
                output.push(3);
                value.encode(output);
            }
        }
    }

    fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        let mut fields = vec![CanonicalField::new(
            BOUND_KIND,
            CanonicalValue::utf8(self.kind())?,
        )];
        if let Some(value) = self.value() {
            fields.push(CanonicalField::new(
                BOUND_VALUE,
                value.to_canonical_value()?,
            ));
        }
        Ok(CanonicalValue::record(fields)?)
    }

    fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        let subject = || malformed(AccuracyAttributeSubject::DomainBound);
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(subject());
        };
        let kind = fields
            .iter()
            .find(|field| field.id() == BOUND_KIND)
            .ok_or_else(subject)?;
        let CanonicalValueView::Utf8(kind) = kind.value().view() else {
            return Err(subject());
        };
        let endpoint = fields
            .iter()
            .find(|field| field.id() == BOUND_VALUE)
            .map(CanonicalField::value);
        match (kind, endpoint) {
            ("unbounded", None) => Ok(Self::Unbounded),
            ("open", Some(value)) => Ok(Self::Open(ExactRational::from_canonical_value(value)?)),
            ("closed", Some(value)) => {
                Ok(Self::Closed(ExactRational::from_canonical_value(value)?))
            }
            _ => Err(subject()),
        }
    }
}

/// A nonempty exact interval over one operand or one reference magnitude.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DomainInterval {
    lower: DomainBound,
    upper: DomainBound,
}

impl DomainInterval {
    /// The interval admitting every exact value.
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            lower: DomainBound::Unbounded,
            upper: DomainBound::Unbounded,
        }
    }

    /// Creates one nonempty interval.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::EmptyDomainInterval`] when the bounds
    /// admit no value. An empty interval is refused rather than carried because a
    /// clause over one is vacuously satisfied, which would let a contract state
    /// an obligation that can never be checked.
    pub fn new(
        operand: OperandOrdinal,
        lower: DomainBound,
        upper: DomainBound,
    ) -> Result<Self, AccuracyContractError> {
        let candidate = Self { lower, upper };
        if candidate.is_empty() {
            return Err(AccuracyContractError::EmptyDomainInterval { operand });
        }
        Ok(candidate)
    }

    fn is_empty(&self) -> bool {
        let (Some(lower), Some(upper)) = (self.lower.value(), self.upper.value()) else {
            return false;
        };
        let closed_both = matches!(self.lower, DomainBound::Closed(_))
            && matches!(self.upper, DomainBound::Closed(_));
        if closed_both {
            lower > upper
        } else {
            lower >= upper
        }
    }

    /// Returns the lower endpoint.
    #[must_use]
    pub const fn lower(&self) -> &DomainBound {
        &self.lower
    }

    /// Returns the upper endpoint.
    #[must_use]
    pub const fn upper(&self) -> &DomainBound {
        &self.upper
    }

    /// Returns whether this interval admits the supplied exact value.
    #[must_use]
    pub fn contains(&self, value: &ExactRational) -> bool {
        let above_lower = match &self.lower {
            DomainBound::Unbounded => true,
            DomainBound::Open(bound) => value > bound,
            DomainBound::Closed(bound) => value >= bound,
        };
        let below_upper = match &self.upper {
            DomainBound::Unbounded => true,
            DomainBound::Open(bound) => value < bound,
            DomainBound::Closed(bound) => value <= bound,
        };
        above_lower && below_upper
    }

    /// Returns every finite endpoint this interval names.
    fn endpoints(&self) -> impl Iterator<Item = &ExactRational> {
        [self.lower.value(), self.upper.value()]
            .into_iter()
            .flatten()
    }

    fn encode(&self, output: &mut Vec<u8>) {
        self.lower.encode(output);
        self.upper.encode(output);
    }

    fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        Ok(CanonicalValue::record([
            CanonicalField::new(INTERVAL_LOWER, self.lower.to_canonical_value()?),
            CanonicalField::new(INTERVAL_UPPER, self.upper.to_canonical_value()?),
        ])?)
    }

    fn from_canonical_value(
        operand: OperandOrdinal,
        value: &CanonicalValue,
    ) -> Result<Self, AccuracyContractError> {
        let subject = || malformed(AccuracyAttributeSubject::DomainClauseRecord);
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(subject());
        };
        let [lower, upper] = fields else {
            return Err(subject());
        };
        if lower.id() != INTERVAL_LOWER || upper.id() != INTERVAL_UPPER {
            return Err(subject());
        }
        Self::new(
            operand,
            DomainBound::from_canonical_value(lower.value())?,
            DomainBound::from_canonical_value(upper.value())?,
        )
    }
}

/// A typed class of the exact reference result.
///
/// These describe *semantic cases*, and ADR 0042 is explicit that "they are not
/// automatically runtime guards": asserting `Nonzero` states a proved property of
/// the reference over the clause's input domain, not a check the compiler will
/// emit.
///
/// Closed rather than `#[non_exhaustive]`: a new class carries a new definedness
/// consequence, and a consumer that ignored it through a wildcard would decide
/// definedness on an incomplete reading of what was proved.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReferenceResultClass {
    /// The exact reference is finite.
    Finite,
    /// The exact reference is not zero, which is what makes a relative predicate defined.
    Nonzero,
    /// The exact reference is strictly positive.
    Positive,
    /// The exact reference is strictly negative.
    Negative,
}

impl ReferenceResultClass {
    const fn spelling(self) -> &'static str {
        match self {
            Self::Finite => "finite",
            Self::Nonzero => "nonzero",
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "finite" => Self::Finite,
            "nonzero" => Self::Nonzero,
            "positive" => Self::Positive,
            "negative" => Self::Negative,
            _ => return None,
        })
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Finite => 1,
            Self::Nonzero => 2,
            Self::Positive => 3,
            Self::Negative => 4,
        }
    }

    /// Returns whether this class implies the reference is not zero.
    #[must_use]
    pub const fn implies_nonzero(self) -> bool {
        match self {
            Self::Finite => false,
            Self::Nonzero | Self::Positive | Self::Negative => true,
        }
    }
}

impl fmt::Display for ReferenceResultClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.spelling())
    }
}

/// What one clause proves about the exact reference over its input domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceResultConstraint {
    classes: BTreeSet<ReferenceResultClass>,
    magnitude: Option<DomainInterval>,
    justification: Option<NormativeDefinitionRef>,
}

impl ReferenceResultConstraint {
    /// The constraint that proves nothing about the reference.
    #[must_use]
    pub fn unconstrained() -> Self {
        Self {
            classes: BTreeSet::new(),
            magnitude: None,
            justification: None,
        }
    }

    /// States what an operation-specific proof establishes about the reference.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::UnjustifiedReferenceResultClass`] when a
    /// class or a magnitude bound is asserted with no proof reference. ADR 0042
    /// admits a reference-result class "only through an operation-specific
    /// proof", and an unjustified assertion is exactly the self-granted
    /// permission that rule exists to stop.
    pub fn new(
        classes: impl IntoIterator<Item = ReferenceResultClass>,
        magnitude: Option<DomainInterval>,
        justification: Option<NormativeDefinitionRef>,
    ) -> Result<Self, AccuracyContractError> {
        let classes: BTreeSet<_> = classes.into_iter().collect();
        if justification.is_none()
            && let Some(class) = classes.iter().copied().next()
        {
            return Err(AccuracyContractError::UnjustifiedReferenceResultClass { class });
        }
        if justification.is_none() && magnitude.is_some() {
            return Err(AccuracyContractError::UnjustifiedReferenceResultClass {
                class: ReferenceResultClass::Finite,
            });
        }
        Ok(Self {
            classes,
            magnitude,
            justification,
        })
    }

    /// Returns the proved classes, in canonical order.
    #[must_use]
    pub fn classes(&self) -> impl ExactSizeIterator<Item = ReferenceResultClass> + '_ {
        self.classes.iter().copied()
    }

    /// Returns the proved bounds on the reference magnitude, when there are any.
    #[must_use]
    pub const fn magnitude(&self) -> Option<&DomainInterval> {
        self.magnitude.as_ref()
    }

    /// Returns the operation-specific proof, when one is stated.
    #[must_use]
    pub const fn justification(&self) -> Option<&NormativeDefinitionRef> {
        self.justification.as_ref()
    }

    /// Returns whether this constraint proves the reference is not zero.
    #[must_use]
    pub fn proves_nonzero(&self) -> bool {
        self.classes
            .iter()
            .copied()
            .any(ReferenceResultClass::implies_nonzero)
            || self
                .magnitude
                .as_ref()
                .is_some_and(|interval| !interval.contains(&ExactRational::zero()))
    }

    fn encode(&self, output: &mut Vec<u8>) {
        push_len(output, self.classes.len());
        for class in &self.classes {
            output.push(class.tag());
        }
        match &self.magnitude {
            None => output.push(0),
            Some(interval) => {
                output.push(1);
                interval.encode(output);
            }
        }
        match &self.justification {
            None => output.push(0),
            Some(reference) => {
                output.push(1);
                push_slice(output, reference.as_str().as_bytes());
            }
        }
    }

    fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        let mut fields = vec![CanonicalField::new(
            REFERENCE_CLASSES,
            CanonicalValue::sequence(
                self.classes
                    .iter()
                    .map(|class| CanonicalValue::utf8(class.spelling()))
                    .collect::<Result<Vec<_>, _>>()?,
            )?,
        )];
        if let Some(magnitude) = &self.magnitude {
            fields.push(CanonicalField::new(
                REFERENCE_MAGNITUDE,
                magnitude.to_canonical_value()?,
            ));
        }
        if let Some(justification) = &self.justification {
            fields.push(CanonicalField::new(
                REFERENCE_JUSTIFICATION,
                CanonicalValue::utf8(justification.as_str())?,
            ));
        }
        Ok(CanonicalValue::record(fields)?)
    }

    fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        let subject = || malformed(AccuracyAttributeSubject::ReferenceResultClass);
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(subject());
        };
        let find = |id| {
            fields
                .iter()
                .find(|field| field.id() == id)
                .map(CanonicalField::value)
        };
        let classes_value = find(REFERENCE_CLASSES).ok_or_else(subject)?;
        let CanonicalValueView::Sequence(entries) = classes_value.view() else {
            return Err(subject());
        };
        let mut classes = Vec::with_capacity(entries.len());
        for entry in entries {
            let CanonicalValueView::Utf8(spelling) = entry.view() else {
                return Err(subject());
            };
            classes.push(ReferenceResultClass::parse(spelling).ok_or_else(subject)?);
        }
        let magnitude = find(REFERENCE_MAGNITUDE)
            .map(|value| DomainInterval::from_canonical_value(OperandOrdinal::new(0), value))
            .transpose()?;
        let justification = match find(REFERENCE_JUSTIFICATION) {
            None => None,
            Some(value) => {
                let CanonicalValueView::Utf8(text) = value.view() else {
                    return Err(subject());
                };
                Some(NormativeDefinitionRef::new(text).map_err(|_| subject())?)
            }
        };
        Self::new(classes, magnitude, justification)
    }
}

/// One accuracy-domain clause: where it applies, what it proves, and what it requires.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccuracyDomainClause {
    operands: BTreeMap<OperandOrdinal, DomainInterval>,
    reference: ReferenceResultConstraint,
    predicate: AccuracyPredicate,
}

impl AccuracyDomainClause {
    /// Creates one clause.
    ///
    /// An operand the clause does not name is unconstrained, which is what makes
    /// a whole-domain clause — the shape a constant vendor bound takes — one
    /// entry rather than one per operand.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::TooManyDomainClauses`] when the clause
    /// names more operands than the governed bound admits.
    pub fn new(
        operands: impl IntoIterator<Item = (OperandOrdinal, DomainInterval)>,
        reference: ReferenceResultConstraint,
        predicate: AccuracyPredicate,
    ) -> Result<Self, AccuracyContractError> {
        let operands: BTreeMap<_, _> = operands
            .into_iter()
            .take(MAX_ACCURACY_DOMAIN_OPERANDS.saturating_add(1))
            .collect();
        if operands.len() > MAX_ACCURACY_DOMAIN_OPERANDS {
            return Err(AccuracyContractError::TooManyDomainClauses {
                clauses: operands.len(),
                limit: MAX_ACCURACY_DOMAIN_OPERANDS,
            });
        }
        Ok(Self {
            operands,
            reference,
            predicate,
        })
    }

    /// Returns the interval this clause constrains one operand to, when it names it.
    #[must_use]
    pub fn operand(&self, operand: OperandOrdinal) -> Option<&DomainInterval> {
        self.operands.get(&operand)
    }

    /// Returns whether two clauses constrain exactly the same input region.
    ///
    /// Compares the whole operand map rather than one position, because two
    /// clauses agreeing on operand zero and differing on operand one describe
    /// different regions — and a refinement decision that treated them as one
    /// would carry a bound from a region where it was proved into a region where
    /// it was not.
    #[must_use]
    pub fn constrains_the_same_region(&self, other: &Self) -> bool {
        self.operands == other.operands
    }

    /// Returns what this clause proves about the exact reference.
    #[must_use]
    pub const fn reference(&self) -> &ReferenceResultConstraint {
        &self.reference
    }

    /// Returns the accuracy predicate this clause requires.
    #[must_use]
    pub const fn predicate(&self) -> &AccuracyPredicate {
        &self.predicate
    }

    /// Returns whether this clause applies at the supplied exact input point.
    #[must_use]
    pub fn applies_at(&self, point: &[ExactRational]) -> bool {
        self.operands.iter().all(|(ordinal, interval)| {
            point
                .get(ordinal.get() as usize)
                .is_some_and(|value| interval.contains(value))
        })
    }

    fn encode(&self, output: &mut Vec<u8>) {
        push_len(output, self.operands.len());
        for (ordinal, interval) in &self.operands {
            output.extend_from_slice(&ordinal.get().to_be_bytes());
            interval.encode(output);
        }
        self.reference.encode(output);
        self.predicate.encode(output);
    }

    fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        let bindings: Result<Vec<_>, AccuracyContractError> = self
            .operands
            .iter()
            .map(|(ordinal, interval)| {
                Ok(CanonicalValue::record([
                    CanonicalField::new(
                        OPERAND_BINDING_ORDINAL,
                        CanonicalValue::unsigned_u32(ordinal.get()),
                    ),
                    CanonicalField::new(OPERAND_BINDING_INTERVAL, interval.to_canonical_value()?),
                ])?)
            })
            .collect();
        Ok(CanonicalValue::record([
            CanonicalField::new(CLAUSE_OPERANDS, CanonicalValue::sequence(bindings?)?),
            CanonicalField::new(CLAUSE_REFERENCE, self.reference.to_canonical_value()?),
            CanonicalField::new(CLAUSE_PREDICATE, self.predicate.to_canonical_value()?),
        ])?)
    }

    fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        let subject = || malformed(AccuracyAttributeSubject::DomainClauseRecord);
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(subject());
        };
        let [operands, reference, predicate] = fields else {
            return Err(subject());
        };
        if operands.id() != CLAUSE_OPERANDS
            || reference.id() != CLAUSE_REFERENCE
            || predicate.id() != CLAUSE_PREDICATE
        {
            return Err(subject());
        }
        let CanonicalValueView::Sequence(bindings) = operands.value().view() else {
            return Err(subject());
        };
        let mut collected = Vec::with_capacity(bindings.len());
        for binding in bindings {
            let CanonicalValueView::Record(pair) = binding.view() else {
                return Err(subject());
            };
            let [ordinal, interval] = pair else {
                return Err(subject());
            };
            if ordinal.id() != OPERAND_BINDING_ORDINAL || interval.id() != OPERAND_BINDING_INTERVAL
            {
                return Err(subject());
            }
            let CanonicalValueView::Unsigned { bits, .. } = ordinal.value().view() else {
                return Err(subject());
            };
            let ordinal = OperandOrdinal::new(u32::try_from(bits).map_err(|_| subject())?);
            collected.push((
                ordinal,
                DomainInterval::from_canonical_value(ordinal, interval.value())?,
            ));
        }
        Self::new(
            collected,
            ReferenceResultConstraint::from_canonical_value(reference.value())?,
            AccuracyPredicate::from_canonical_value(predicate.value())?,
        )
    }
}

/// The complete clause set of one bounded piecewise contract, with its admitted domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccuracyDomain {
    admitted: Vec<DomainInterval>,
    clauses: Vec<AccuracyDomainClause>,
}

impl AccuracyDomain {
    /// Creates the clause set over an operation's admitted ordinary input domain.
    ///
    /// `admitted` is one interval per operand, in signature order; it is the
    /// operation's *ordinary* domain, so exceptional inputs are the independent
    /// exceptional-value contract's subject rather than a clause's.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] for an empty clause set, an arity or
    /// clause count over the governed bound, or an interval that admits no value.
    pub fn new(
        admitted: impl IntoIterator<Item = DomainInterval>,
        clauses: impl IntoIterator<Item = AccuracyDomainClause>,
    ) -> Result<Self, AccuracyContractError> {
        let admitted: Vec<_> = admitted
            .into_iter()
            .take(MAX_ACCURACY_DOMAIN_OPERANDS.saturating_add(1))
            .collect();
        if admitted.len() > MAX_ACCURACY_DOMAIN_OPERANDS {
            return Err(AccuracyContractError::TooManyDomainClauses {
                clauses: admitted.len(),
                limit: MAX_ACCURACY_DOMAIN_OPERANDS,
            });
        }
        let clauses: Vec<_> = clauses
            .into_iter()
            .take(MAX_ACCURACY_DOMAIN_CLAUSES.saturating_add(1))
            .collect();
        if clauses.len() > MAX_ACCURACY_DOMAIN_CLAUSES {
            return Err(AccuracyContractError::TooManyDomainClauses {
                clauses: clauses.len(),
                limit: MAX_ACCURACY_DOMAIN_CLAUSES,
            });
        }
        if clauses.is_empty() {
            return Err(AccuracyContractError::EmptyClauseSet);
        }
        Ok(Self { admitted, clauses })
    }

    /// Returns the operation's admitted ordinary input domain, per operand.
    #[must_use]
    pub fn admitted(&self) -> &[DomainInterval] {
        &self.admitted
    }

    /// Returns the unordered clause set.
    #[must_use]
    pub fn clauses(&self) -> &[AccuracyDomainClause] {
        &self.clauses
    }

    /// Decides whether the clauses cover the admitted domain, exactly.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::IncompleteDomainCoverage`] with a witness
    /// point the clauses leave uncovered, or
    /// [`AccuracyContractError::CoverageNotVerifiable`] when the decomposition
    /// exceeds the governed cell budget.
    pub fn verify_coverage(&self) -> Result<Vec<CoveredCell<'_>>, AccuracyContractError> {
        let axes: Vec<Vec<ExactRational>> = self
            .admitted
            .iter()
            .enumerate()
            .map(|(position, interval)| {
                self.axis_representatives(
                    OperandOrdinal::new(u32::try_from(position).unwrap_or(u32::MAX)),
                    interval,
                )
            })
            .collect();
        let mut cells: usize = 1;
        for axis in &axes {
            cells = cells
                .checked_mul(axis.len())
                .filter(|count| *count <= MAX_ACCURACY_DOMAIN_COVERAGE_CELLS)
                .ok_or(AccuracyContractError::CoverageNotVerifiable {
                    cells: usize::MAX,
                    limit: MAX_ACCURACY_DOMAIN_COVERAGE_CELLS,
                })?;
        }

        let mut covered = Vec::with_capacity(cells);
        let mut point = vec![ExactRational::zero(); axes.len()];
        for index in 0..cells {
            let mut remainder = index;
            for (position, axis) in axes.iter().enumerate() {
                point[position] = axis[remainder % axis.len()].clone();
                remainder /= axis.len();
            }
            let applicable: Vec<&AccuracyDomainClause> = self
                .clauses
                .iter()
                .filter(|clause| clause.applies_at(&point))
                .collect();
            if applicable.is_empty() {
                return Err(AccuracyContractError::IncompleteDomainCoverage {
                    witness: point.clone(),
                });
            }
            covered.push(CoveredCell {
                representative: point.clone(),
                applicable,
            });
        }
        Ok(covered)
    }

    /// Returns one representative per elementary cell of one admitted axis.
    ///
    /// Every clause endpoint on this axis is a breakpoint, so no clause boundary
    /// falls strictly inside an open cell. A clause therefore either contains a
    /// whole cell or is disjoint from it, and one representative decides the
    /// cell — which is what makes this decomposition a decision rather than a
    /// sample.
    fn axis_representatives(
        &self,
        operand: OperandOrdinal,
        admitted: &DomainInterval,
    ) -> Vec<ExactRational> {
        let mut breaks: Vec<ExactRational> = admitted.endpoints().cloned().collect();
        for clause in &self.clauses {
            if let Some(interval) = clause.operand(operand) {
                breaks.extend(interval.endpoints().cloned());
            }
        }
        breaks.sort();
        breaks.dedup();

        let mut representatives = Vec::new();
        if breaks.is_empty() {
            representatives.push(ExactRational::zero());
            return representatives;
        }
        // Below the least breakpoint: reachable only when the admitted interval
        // itself is unbounded there, because a finite admitted endpoint is a
        // breakpoint and nothing admitted lies below it.
        if matches!(admitted.lower(), DomainBound::Unbounded) {
            representatives.push(breaks[0].subtract(&ExactRational::one()));
        }
        for (position, value) in breaks.iter().enumerate() {
            if admitted.contains(value) {
                representatives.push(value.clone());
            }
            if let Some(next) = breaks.get(position + 1) {
                let midpoint = value.add(next).scale_by_power_of_two(-1);
                if admitted.contains(&midpoint) {
                    representatives.push(midpoint);
                }
            }
        }
        if matches!(admitted.upper(), DomainBound::Unbounded) {
            representatives.push(breaks[breaks.len() - 1].add(&ExactRational::one()));
        }
        representatives
    }

    /// Returns the domain-separated canonical encoding of this clause set.
    #[must_use]
    pub fn canonical_encoding(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, ACCURACY_DOMAIN_DOMAIN);
        self.encode(&mut bytes);
        bytes
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        push_len(output, self.admitted.len());
        for interval in &self.admitted {
            interval.encode(output);
        }
        push_len(output, self.clauses.len());
        for clause in &self.clauses {
            clause.encode(output);
        }
    }

    /// Returns the canonical attribute value carrying this clause set.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] when the clause set exceeds a canonical
    /// structural bound.
    pub fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        let admitted: Result<Vec<_>, _> = self
            .admitted
            .iter()
            .map(DomainInterval::to_canonical_value)
            .collect();
        let clauses: Result<Vec<_>, _> = self
            .clauses
            .iter()
            .map(AccuracyDomainClause::to_canonical_value)
            .collect();
        Ok(CanonicalValue::record([
            CanonicalField::new(DOMAIN_ADMITTED, CanonicalValue::sequence(admitted?)?),
            CanonicalField::new(DOMAIN_CLAUSES, CanonicalValue::sequence(clauses?)?),
        ])?)
    }

    /// Decodes one clause set exactly as an attribute carries it.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] for a malformed record or a violated
    /// domain rule.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        let subject = || malformed(AccuracyAttributeSubject::DomainClauseRecord);
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(subject());
        };
        let [admitted, clauses] = fields else {
            return Err(subject());
        };
        if admitted.id() != DOMAIN_ADMITTED || clauses.id() != DOMAIN_CLAUSES {
            return Err(subject());
        }
        let (
            CanonicalValueView::Sequence(admitted_values),
            CanonicalValueView::Sequence(clause_values),
        ) = (admitted.value().view(), clauses.value().view())
        else {
            return Err(subject());
        };
        let mut admitted = Vec::with_capacity(admitted_values.len());
        for (position, value) in admitted_values.iter().enumerate() {
            admitted.push(DomainInterval::from_canonical_value(
                OperandOrdinal::new(u32::try_from(position).unwrap_or(u32::MAX)),
                value,
            )?);
        }
        let mut clauses = Vec::with_capacity(clause_values.len());
        for value in clause_values {
            clauses.push(AccuracyDomainClause::from_canonical_value(value)?);
        }
        Self::new(admitted, clauses)
    }
}

/// One elementary cell of the admitted domain and every clause that applies there.
///
/// The applicable set is the whole of ADR 0042's intersection semantics: the
/// obligation at this cell is the conjunction of these clauses' predicates, and
/// the list is unordered because no clause has priority over another.
#[derive(Clone, Debug)]
pub struct CoveredCell<'a> {
    representative: Vec<ExactRational>,
    applicable: Vec<&'a AccuracyDomainClause>,
}

impl<'a> CoveredCell<'a> {
    /// Returns the exact input point deciding this cell.
    #[must_use]
    pub fn representative(&self) -> &[ExactRational] {
        &self.representative
    }

    /// Returns every clause that applies here.
    #[must_use]
    pub fn applicable(&self) -> &[&'a AccuracyDomainClause] {
        &self.applicable
    }

    /// Returns whether the applicable clauses jointly prove the reference is nonzero.
    #[must_use]
    pub fn proves_nonzero_reference(&self) -> bool {
        self.applicable
            .iter()
            .any(|clause| clause.reference().proves_nonzero())
    }

    /// Returns the intersected bounds the applicable clauses prove on `|r|`.
    ///
    /// `None` on a side means no applicable clause bounds the reference there, so
    /// verification of a predicate that needs that side fails closed rather than
    /// assuming one.
    #[must_use]
    pub fn reference_magnitude_bounds(&self) -> (Option<ExactRational>, Option<ExactRational>) {
        let mut lower: Option<ExactRational> = None;
        let mut upper: Option<ExactRational> = None;
        for clause in &self.applicable {
            let Some(interval) = clause.reference().magnitude() else {
                continue;
            };
            if let Some(bound) = interval.lower().value() {
                lower = Some(match lower {
                    None => bound.clone(),
                    Some(current) if current < *bound => bound.clone(),
                    Some(current) => current,
                });
            }
            if let Some(bound) = interval.upper().value() {
                upper = Some(match upper {
                    None => bound.clone(),
                    Some(current) if current > *bound => bound.clone(),
                    Some(current) => current,
                });
            }
        }
        (lower, upper)
    }
}
