//! The generic bounded accuracy predicates and their normalization.
//!
//! ADR 0042's initial generic predicates, over the infinitely precise reference
//! `r` and the mathematical value `z` of the finite result-dtype candidate
//! selected *before* result-subnormal and signed-zero mapping:
//!
//! ```text
//! Absolute(t):            |z - r| <= t
//! Relative(t):            |z - r| / |r| <= t, with the clause excluding r = 0
//! AbsoluteRelative(a, q): |z - r| <= a + q * |r|
//! Ulp(metric_key, t):     |z - r| / ulp_result_dtype(r) <= t
//! AllOf(predicates):      every member predicate is satisfied
//! AnyOf(predicates):      at least one member predicate is satisfied
//! ```
//!
//! **Why the Boolean forms exist.** `AbsoluteRelative` is not `AnyOf([Absolute,
//! Relative])`: an additive absolute-plus-relative tolerance is a *different*
//! bound from an absolute-or-relative guarantee, and the two disagree on real
//! specifications. ADR 0042 keeps all three so that neither has to be
//! approximated by the other.
//!
//! # Normalization, and why decode refuses instead of renormalizing
//!
//! A constructed collection is flattened over same-kind members, sorted by
//! canonical encoding, deduplicated, and bounded in depth and cardinality; an
//! empty collection is invalid and a singleton canonicalizes to its member. So a
//! held [`AccuracyPredicate`] is normalized by construction, and its encoding is
//! its identity.
//!
//! That makes the *decode* path different from the construct path on purpose.
//! [`AccuracyPredicate::from_canonical_value`] refuses a non-flattened, unsorted,
//! duplicated, or singleton collection rather than normalizing it, exactly as
//! [`crate::semantic::ContractionStructureError::NonCanonicalNumbering`] refuses a
//! non-canonical index numbering. Renormalizing on decode would admit several
//! encodings of one predicate, and since the encoding reaches identity, one
//! predicate would have several identities — the collision the normalization
//! exists to prevent.
//!
//! # The definedness rule is recursive
//!
//! `Relative` divides by `|r|`, so it is undefined at `r = 0` and its enclosing
//! clause domain must exclude that reference.
//! [`AccuracyPredicate::requires_nonzero_reference`]
//! walks the whole tree, and `AnyOf` propagates it exactly as `AllOf` does: a
//! disjunction containing an undefined member is undefined, not satisfied by its
//! other member. ADR 0042 states this directly — "`AnyOf` cannot hide an
//! undefined relative predicate at reference zero" — and it is the one place the
//! obvious Boolean reading is the wrong one.

use std::cmp::Ordering;
use std::fmt;

use crate::identity::{push_len, push_slice};
use crate::semantic::{AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueView};

use super::error::{AccuracyAttributeSubject, AccuracyContractError, malformed};
use super::metric::AccuracyMetricKey;
use super::rational::ExactTolerance;

/// Maximum members one Boolean accuracy predicate may name.
pub const MAX_ACCURACY_PREDICATE_MEMBERS: usize = 16;
/// Maximum nesting depth of one accuracy predicate.
pub const MAX_ACCURACY_PREDICATE_DEPTH: usize = 8;
/// Maximum total nodes in one accuracy predicate.
pub const MAX_ACCURACY_PREDICATE_NODES: usize = 64;

/// Predicate-record field carrying the predicate kind.
const PREDICATE_KIND: AttributeFieldId = AttributeFieldId::new(1);
/// Predicate-record field carrying the absolute tolerance, where the kind has one.
const PREDICATE_ABSOLUTE: AttributeFieldId = AttributeFieldId::new(2);
/// Predicate-record field carrying the relative tolerance, where the kind has one.
const PREDICATE_RELATIVE: AttributeFieldId = AttributeFieldId::new(3);
/// Predicate-record field carrying the metric key, where the kind has one.
const PREDICATE_METRIC: AttributeFieldId = AttributeFieldId::new(4);
/// Predicate-record field carrying the ordered member sequence, where the kind has one.
const PREDICATE_MEMBERS: AttributeFieldId = AttributeFieldId::new(5);

/// Domain separator of a canonical accuracy-predicate encoding.
const ACCURACY_PREDICATE_DOMAIN: &[u8] = b"tiler.accuracy-predicate.v1\0";

/// Which Boolean combinator one collection is.
///
/// Not `#[non_exhaustive]`, and that is load-bearing rather than incidental: every
/// consumer that encodes a predicate into canonical identity or decides an
/// implication over one matches this exhaustively, so widening the vocabulary is a
/// build error at each such site instead of a silently unhandled combinator (ADR
/// 0074 convention 5b). The same reasoning applies to
/// [`AccuracyPredicateView`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BooleanPredicateKind {
    /// Every member predicate is satisfied.
    AllOf,
    /// At least one member predicate is satisfied.
    AnyOf,
}

impl BooleanPredicateKind {
    const fn spelling(self) -> &'static str {
        match self {
            Self::AllOf => "all-of",
            Self::AnyOf => "any-of",
        }
    }
}

impl fmt::Display for BooleanPredicateKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.spelling())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum PredicateData {
    Absolute(ExactTolerance),
    Relative(ExactTolerance),
    AbsoluteRelative {
        absolute: ExactTolerance,
        relative: ExactTolerance,
    },
    Ulp {
        metric: AccuracyMetricKey,
        tolerance: ExactTolerance,
    },
    Boolean {
        kind: BooleanPredicateKind,
        members: Vec<AccuracyPredicate>,
    },
}

/// Borrowed inspection of one normalized accuracy predicate.
///
/// Closed for the reason [`BooleanPredicateKind`] records: a new predicate shape
/// must break every consumer that decides, encodes, or refines over this
/// vocabulary rather than falling into a wildcard that silently ignores it.
#[derive(Clone, Copy, Debug)]
pub enum AccuracyPredicateView<'a> {
    /// `|z - r| <= t`.
    Absolute {
        /// The exact tolerance `t`.
        tolerance: &'a ExactTolerance,
    },
    /// `|z - r| / |r| <= t`, requiring the clause domain to exclude `r = 0`.
    Relative {
        /// The exact tolerance `t`.
        tolerance: &'a ExactTolerance,
    },
    /// `|z - r| <= a + q * |r|`.
    AbsoluteRelative {
        /// The exact additive term `a`.
        absolute: &'a ExactTolerance,
        /// The exact proportional term `q`.
        relative: &'a ExactTolerance,
    },
    /// `|z - r| / ulp(r) <= t` under a named versioned metric.
    Ulp {
        /// The metric defining `ulp`.
        metric: &'a AccuracyMetricKey,
        /// The exact tolerance `t`.
        tolerance: &'a ExactTolerance,
    },
    /// A normalized Boolean combination.
    Boolean {
        /// Which combinator this is.
        kind: BooleanPredicateKind,
        /// The flattened, sorted, deduplicated members.
        members: &'a [AccuracyPredicate],
    },
}

/// A normalized generic bounded accuracy predicate.
///
/// There is no unchecked constructor, so holding one is evidence that every
/// normalization rule was decided.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AccuracyPredicate(PredicateData);

impl AccuracyPredicate {
    /// Builds `|z - r| <= tolerance`.
    #[must_use]
    pub const fn absolute(tolerance: ExactTolerance) -> Self {
        Self(PredicateData::Absolute(tolerance))
    }

    /// Builds `|z - r| / |r| <= tolerance`.
    ///
    /// The enclosing clause must exclude `r = 0`; that obligation is checked
    /// where the clause is assembled, not here, because a predicate alone does
    /// not know its domain.
    #[must_use]
    pub const fn relative(tolerance: ExactTolerance) -> Self {
        Self(PredicateData::Relative(tolerance))
    }

    /// Builds `|z - r| <= absolute + relative * |r|`.
    #[must_use]
    pub const fn absolute_relative(absolute: ExactTolerance, relative: ExactTolerance) -> Self {
        Self(PredicateData::AbsoluteRelative { absolute, relative })
    }

    /// Builds `|z - r| / ulp(r) <= tolerance` under one named versioned metric.
    #[must_use]
    pub const fn ulp(metric: AccuracyMetricKey, tolerance: ExactTolerance) -> Self {
        Self(PredicateData::Ulp { metric, tolerance })
    }

    /// Builds the normalized conjunction of the supplied predicates.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] for an empty collection or an exceeded
    /// member, depth, or node bound.
    pub fn all_of(members: impl IntoIterator<Item = Self>) -> Result<Self, AccuracyContractError> {
        Self::boolean(BooleanPredicateKind::AllOf, members)
    }

    /// Builds the normalized disjunction of the supplied predicates.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] for an empty collection or an exceeded
    /// member, depth, or node bound.
    pub fn any_of(members: impl IntoIterator<Item = Self>) -> Result<Self, AccuracyContractError> {
        Self::boolean(BooleanPredicateKind::AnyOf, members)
    }

    fn boolean(
        kind: BooleanPredicateKind,
        members: impl IntoIterator<Item = Self>,
    ) -> Result<Self, AccuracyContractError> {
        let mut flattened: Vec<Self> = Vec::new();
        for member in members
            .into_iter()
            .take(MAX_ACCURACY_PREDICATE_MEMBERS.saturating_add(1))
        {
            // Flattening is what makes `AllOf([a, AllOf([b, c])])` and
            // `AllOf([a, b, c])` one predicate rather than two spellings of one
            // meaning. It happens before the bound so that a flattened result
            // over the bound is reported as over the bound.
            match member.0 {
                PredicateData::Boolean {
                    kind: member_kind,
                    members: nested,
                } if member_kind == kind => flattened.extend(nested),
                _ => flattened.push(member),
            }
            if flattened.len() > MAX_ACCURACY_PREDICATE_MEMBERS {
                return Err(AccuracyContractError::TooManyPredicateMembers {
                    members: flattened.len(),
                    limit: MAX_ACCURACY_PREDICATE_MEMBERS,
                });
            }
        }
        flattened.sort();
        flattened.dedup();
        Self::finish(kind, flattened)
    }

    fn finish(
        kind: BooleanPredicateKind,
        members: Vec<Self>,
    ) -> Result<Self, AccuracyContractError> {
        if members.is_empty() {
            return Err(AccuracyContractError::EmptyPredicateCollection { kind });
        }
        if members.len() > MAX_ACCURACY_PREDICATE_MEMBERS {
            return Err(AccuracyContractError::TooManyPredicateMembers {
                members: members.len(),
                limit: MAX_ACCURACY_PREDICATE_MEMBERS,
            });
        }
        if let [single] = members.as_slice() {
            return Ok(single.clone());
        }
        let candidate = Self(PredicateData::Boolean { kind, members });
        let depth = candidate.depth();
        if depth > MAX_ACCURACY_PREDICATE_DEPTH {
            return Err(AccuracyContractError::PredicateTooDeep {
                depth,
                limit: MAX_ACCURACY_PREDICATE_DEPTH,
            });
        }
        let nodes = candidate.nodes();
        if nodes > MAX_ACCURACY_PREDICATE_NODES {
            return Err(AccuracyContractError::TooManyPredicateNodes {
                nodes,
                limit: MAX_ACCURACY_PREDICATE_NODES,
            });
        }
        Ok(candidate)
    }

    /// Returns a borrowed, exhaustively tagged view of this predicate.
    #[must_use]
    pub fn view(&self) -> AccuracyPredicateView<'_> {
        match &self.0 {
            PredicateData::Absolute(tolerance) => AccuracyPredicateView::Absolute { tolerance },
            PredicateData::Relative(tolerance) => AccuracyPredicateView::Relative { tolerance },
            PredicateData::AbsoluteRelative { absolute, relative } => {
                AccuracyPredicateView::AbsoluteRelative { absolute, relative }
            }
            PredicateData::Ulp { metric, tolerance } => {
                AccuracyPredicateView::Ulp { metric, tolerance }
            }
            PredicateData::Boolean { kind, members } => AccuracyPredicateView::Boolean {
                kind: *kind,
                members,
            },
        }
    }

    /// Returns whether this predicate is undefined at a zero reference.
    ///
    /// `true` exactly when a `Relative` appears anywhere in the tree. Both
    /// Boolean kinds propagate, which is the recursion ADR 0042 requires: a
    /// disjunction whose other member happens to hold at zero does not make the
    /// relative member defined there.
    #[must_use]
    pub fn requires_nonzero_reference(&self) -> bool {
        match &self.0 {
            PredicateData::Relative(_) => true,
            PredicateData::Absolute(_)
            | PredicateData::AbsoluteRelative { .. }
            | PredicateData::Ulp { .. } => false,
            PredicateData::Boolean { members, .. } => {
                members.iter().any(Self::requires_nonzero_reference)
            }
        }
    }

    /// Returns every metric key this predicate measures under, in canonical order.
    #[must_use]
    pub fn metrics(&self) -> Vec<AccuracyMetricKey> {
        let mut collected = Vec::new();
        self.collect_metrics(&mut collected);
        collected.sort();
        collected.dedup();
        collected
    }

    fn collect_metrics(&self, output: &mut Vec<AccuracyMetricKey>) {
        match &self.0 {
            PredicateData::Ulp { metric, .. } => output.push(metric.clone()),
            PredicateData::Boolean { members, .. } => {
                for member in members {
                    member.collect_metrics(output);
                }
            }
            PredicateData::Absolute(_)
            | PredicateData::Relative(_)
            | PredicateData::AbsoluteRelative { .. } => {}
        }
    }

    fn depth(&self) -> usize {
        match &self.0 {
            PredicateData::Boolean { members, .. } => {
                1 + members.iter().map(Self::depth).max().unwrap_or(0)
            }
            _ => 1,
        }
    }

    fn nodes(&self) -> usize {
        match &self.0 {
            PredicateData::Boolean { members, .. } => {
                1 + members.iter().map(Self::nodes).sum::<usize>()
            }
            _ => 1,
        }
    }

    /// Returns the domain-separated canonical encoding of this predicate.
    #[must_use]
    pub fn canonical_encoding(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, ACCURACY_PREDICATE_DOMAIN);
        self.encode(&mut bytes);
        bytes
    }

    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        match &self.0 {
            PredicateData::Absolute(tolerance) => {
                output.push(1);
                tolerance.encode(output);
            }
            PredicateData::Relative(tolerance) => {
                output.push(2);
                tolerance.encode(output);
            }
            PredicateData::AbsoluteRelative { absolute, relative } => {
                output.push(3);
                absolute.encode(output);
                relative.encode(output);
            }
            PredicateData::Ulp { metric, tolerance } => {
                output.push(4);
                metric.encode(output);
                tolerance.encode(output);
            }
            PredicateData::Boolean { kind, members } => {
                output.push(match kind {
                    BooleanPredicateKind::AllOf => 5,
                    BooleanPredicateKind::AnyOf => 6,
                });
                push_len(output, members.len());
                for member in members {
                    member.encode(output);
                }
            }
        }
    }

    /// Returns the canonical attribute value carrying this predicate.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] when the predicate exceeds a canonical
    /// structural bound.
    pub fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        let mut fields = vec![CanonicalField::new(
            PREDICATE_KIND,
            CanonicalValue::utf8(self.kind_spelling())?,
        )];
        match &self.0 {
            PredicateData::Absolute(tolerance) => fields.push(CanonicalField::new(
                PREDICATE_ABSOLUTE,
                tolerance.to_canonical_value()?,
            )),
            PredicateData::Relative(tolerance) => fields.push(CanonicalField::new(
                PREDICATE_RELATIVE,
                tolerance.to_canonical_value()?,
            )),
            PredicateData::AbsoluteRelative { absolute, relative } => {
                fields.push(CanonicalField::new(
                    PREDICATE_ABSOLUTE,
                    absolute.to_canonical_value()?,
                ));
                fields.push(CanonicalField::new(
                    PREDICATE_RELATIVE,
                    relative.to_canonical_value()?,
                ));
            }
            PredicateData::Ulp { metric, tolerance } => {
                fields.push(CanonicalField::new(
                    PREDICATE_METRIC,
                    metric.to_canonical_value()?,
                ));
                fields.push(CanonicalField::new(
                    PREDICATE_ABSOLUTE,
                    tolerance.to_canonical_value()?,
                ));
            }
            PredicateData::Boolean { members, .. } => {
                let encoded: Result<Vec<_>, _> =
                    members.iter().map(Self::to_canonical_value).collect();
                fields.push(CanonicalField::new(
                    PREDICATE_MEMBERS,
                    CanonicalValue::sequence(encoded?)?,
                ));
            }
        }
        Ok(CanonicalValue::record(fields)?)
    }

    const fn kind_spelling(&self) -> &'static str {
        match &self.0 {
            PredicateData::Absolute(_) => "absolute",
            PredicateData::Relative(_) => "relative",
            PredicateData::AbsoluteRelative { .. } => "absolute-relative",
            PredicateData::Ulp { .. } => "ulp",
            PredicateData::Boolean { kind, .. } => kind.spelling(),
        }
    }

    /// Decodes one predicate exactly as an attribute carries it.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] for a malformed record, and for every
    /// non-canonical Boolean spelling: unflattened same-kind nesting, unsorted
    /// members, a repeated member, an empty collection, and a singleton. Those
    /// are refusals rather than renormalizations because the encoding is the
    /// identity.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(malformed(AccuracyAttributeSubject::PredicateRecord));
        };
        let find = |id| {
            fields
                .iter()
                .find(|field| field.id() == id)
                .map(CanonicalField::value)
        };
        let kind_value = find(PREDICATE_KIND)
            .ok_or_else(|| malformed(AccuracyAttributeSubject::PredicateKind))?;
        let CanonicalValueView::Utf8(kind) = kind_value.view() else {
            return Err(malformed(AccuracyAttributeSubject::PredicateKind));
        };
        let tolerance = |id| {
            find(id)
                .ok_or_else(|| malformed(AccuracyAttributeSubject::PredicateRecord))
                .and_then(ExactTolerance::from_canonical_value)
        };
        match kind {
            "absolute" => {
                expect_fields(fields.len(), 2)?;
                Ok(Self::absolute(tolerance(PREDICATE_ABSOLUTE)?))
            }
            "relative" => {
                expect_fields(fields.len(), 2)?;
                Ok(Self::relative(tolerance(PREDICATE_RELATIVE)?))
            }
            "absolute-relative" => {
                expect_fields(fields.len(), 3)?;
                Ok(Self::absolute_relative(
                    tolerance(PREDICATE_ABSOLUTE)?,
                    tolerance(PREDICATE_RELATIVE)?,
                ))
            }
            "ulp" => {
                expect_fields(fields.len(), 3)?;
                let metric = AccuracyMetricKey::from_canonical_value(
                    find(PREDICATE_METRIC)
                        .ok_or_else(|| malformed(AccuracyAttributeSubject::MetricKey))?,
                )?;
                Ok(Self::ulp(metric, tolerance(PREDICATE_ABSOLUTE)?))
            }
            "all-of" | "any-of" => {
                expect_fields(fields.len(), 2)?;
                let kind = if kind == "all-of" {
                    BooleanPredicateKind::AllOf
                } else {
                    BooleanPredicateKind::AnyOf
                };
                let members_value = find(PREDICATE_MEMBERS)
                    .ok_or_else(|| malformed(AccuracyAttributeSubject::PredicateMembers))?;
                let CanonicalValueView::Sequence(members) = members_value.view() else {
                    return Err(malformed(AccuracyAttributeSubject::PredicateMembers));
                };
                if members.len() > MAX_ACCURACY_PREDICATE_MEMBERS {
                    return Err(AccuracyContractError::TooManyPredicateMembers {
                        members: members.len(),
                        limit: MAX_ACCURACY_PREDICATE_MEMBERS,
                    });
                }
                if members.is_empty() {
                    return Err(AccuracyContractError::EmptyPredicateCollection { kind });
                }
                if members.len() == 1 {
                    return Err(AccuracyContractError::NonCanonicalPredicateSingleton { kind });
                }
                let mut decoded = Vec::with_capacity(members.len());
                for member in members {
                    let member = Self::from_canonical_value(member)?;
                    if matches!(
                        &member.0,
                        PredicateData::Boolean { kind: nested, .. } if *nested == kind
                    ) {
                        return Err(AccuracyContractError::NonCanonicalPredicateNesting { kind });
                    }
                    decoded.push(member);
                }
                for pair in decoded.windows(2) {
                    match pair[0].cmp(&pair[1]) {
                        Ordering::Less => {}
                        Ordering::Equal => {
                            return Err(AccuracyContractError::DuplicatePredicateMember { kind });
                        }
                        Ordering::Greater => {
                            return Err(AccuracyContractError::NonCanonicalPredicateOrder { kind });
                        }
                    }
                }
                Self::finish(kind, decoded)
            }
            _ => Err(malformed(AccuracyAttributeSubject::PredicateKind)),
        }
    }
}

fn expect_fields(actual: usize, expected: usize) -> Result<(), AccuracyContractError> {
    if actual == expected {
        Ok(())
    } else {
        Err(malformed(AccuracyAttributeSubject::PredicateRecord))
    }
}

impl Ord for AccuracyPredicate {
    /// Orders by canonical encoding, which is the order normalization sorts by.
    ///
    /// Derived from the encoding rather than from variant declaration order, so
    /// "sorted by canonical encoding" is true by construction instead of being a
    /// second rule that has to be kept agreeing with the first.
    fn cmp(&self, other: &Self) -> Ordering {
        let mut left = Vec::new();
        let mut right = Vec::new();
        self.encode(&mut left);
        other.encode(&mut right);
        left.cmp(&right)
    }
}

impl PartialOrd for AccuracyPredicate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
