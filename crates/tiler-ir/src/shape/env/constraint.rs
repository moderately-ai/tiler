//! The constraint half of the `ShapeEnv` authority: typed relations over
//! declared extent symbols, and the decision procedure that rejects a
//! contradictory set.
//!
//! # The supported arithmetic fragment
//!
//! `docs/ir.md` leaves "the solver algorithm and exact supported arithmetic
//! fragment" an implementation choice, so this module chooses one and states
//! it. The choice is driven by a single requirement: the contract says
//! "contradictory semantic constraints reject the graph", and a procedure that
//! missed some contradictions would answer *satisfiable* for a set the contract
//! calls invalid. That is the silently weaker answer the contract forbids, so
//! the fragment is narrowed until the procedure is **complete** on it, and
//! everything outside is refused with a typed error rather than admitted and
//! under-decided.
//!
//! The fragment is: **bounded interval–congruence constraints over equality
//! classes of declared symbols, closed under non-strict comparison, plus
//! factorizations with at most one undetermined term and binary additive
//! equalities for which the procedure can exhibit a model.** Concretely, after
//! equality classes are formed, every admitted relation contributes one of
//!
//! - a merge of two classes ([`ExtentRelation::Equal`] between symbols),
//! - a constant pin on a class,
//! - a closed interval bound on a class ([`ExtentRelation::Interval`], and the
//!   constant-sided forms of [`ExtentRelation::NonNegativeDifference`]),
//! - a congruence `m | x` on a class ([`ExtentRelation::Divisible`]), or
//! - a zero-weight difference edge `x >= y` between two classes, or
//! - a fixed-arity equality `s == left + right`.
//!
//! Every extent is a `u64`, so every class starts at `[0, u64::MAX]` and the
//! *unary* nonnegativity of an extent is a tautology. The nonnegativity kind is
//! therefore carried as a **difference**: `minuend - subtrahend >= 0`, which is
//! what a slice, pad, or window precondition actually asserts and is the only
//! form of it that constrains anything.
//!
//! ## Why the procedure is complete on that fragment
//!
//! A model is exhibited, not merely not-refuted. Equality classes are merged
//! first, and so are the strongly connected components of the `>=` graph, since
//! a `>=` cycle forces equality; what remains is a DAG. Each class carries an
//! interval and a modulus, both exact meets (intersection and least common
//! multiple). Lower bounds are then propagated forward along the DAG and raised
//! to the next multiple of the class modulus. Every raise is implied by the
//! constraints, so a class whose lower bound passes its upper bound is
//! genuinely unsatisfiable. Conversely, when no class fails, assigning every
//! class its propagated lower bound is a model: it satisfies each congruence by
//! construction, each interval because the bounds crossed nowhere, and each
//! edge `c >= d` because propagation established `lower(c) >= lower(d)` and only
//! raised it afterwards.
//!
//! ## What is outside, and why it is refused rather than approximated
//!
//! A factorization `p == f0 * f1 * ...` with two or more undetermined terms is
//! nonlinear. Admitting it would put the environment in a fragment whose
//! satisfiability no interval–congruence propagation decides, and answering
//! "no contradiction found" there would be indistinguishable, to every caller,
//! from a decided *satisfiable*. It is [`FragmentViolation::UnderdeterminedFactorization`]
//! instead. A term counts as **determined** when its equality class holds a
//! constant — from a literal, from an `Equal` against a literal, or from a
//! [`Static`](super::BindingSource::Static) root binding. Determination is deliberately
//! *not* read off a narrowed interval, so fragment membership is a syntactic,
//! order-free property of the environment rather than a consequence of how far
//! propagation happened to get.
//!
//! An additive equality is deliberately one relation over three terms, not an
//! arithmetic node nested inside [`ExtentTerm`]. When all but at most one term
//! are determined, the remaining term is solved exactly. With more free terms,
//! the ordinary interval/congruence procedure first constructs its canonical
//! lower-bound model and the additive equality is admitted only when that same
//! model satisfies it. Otherwise the relation is refused as
//! [`FragmentViolation::UnderdeterminedAdditiveEquality`]. This is conservative
//! but complete on what it admits: every successful build has an exhibited
//! model, while the common runtime-bound `S == C + T` case is admitted by the
//! all-zero model and retained so a later launch-preflight validator can
//! evaluate it against observed bindings.
//!
//! # Semantic input constraints and variant guards are not one list
//!
//! The contract states the two are "not interchangeable": a semantic input
//! constraint is required for the expression to be defined and its failure is an
//! invalid-input diagnostic, while a variant guard is required only for one
//! optimization and its failure selects another plan. They are separate types
//! here, with different provenance vocabularies — [`SemanticInputConstraint`]
//! carries [`FactProvenance`], and [`VariantGuard`] carries the contract's guard
//! vocabulary, [`GuardApplicability`] — so neither can be passed where the other
//! is expected and no accessor blurs them. Their outcomes differ accordingly: a
//! contradictory semantic constraint fails `build`, while a contradictory guard
//! builds and is reported by `ShapeEnv::unsatisfiable_guards`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::num::NonZeroU64;

use super::{FactProvenance, RootBinding, ShapeEnvError, ShapeSymbol};

/// One past the largest representable extent.
///
/// Used as a saturation point: a bound or modulus at or above this admits only
/// zero within the extent domain, which is exactly what the true unsaturated
/// value would admit, so saturating here loses no distinction the decision
/// depends on.
const IMPOSSIBLE: u128 = 1 << 64;

/// The largest representable extent, as the arithmetic width the solver uses.
const MAX_EXTENT: u128 = IMPOSSIBLE - 1;

/// One side of a constraint relation.
///
/// A term is a declared symbol or a literal extent. It is deliberately not an
/// arbitrary expression tree: the fragment this module decides is closed under
/// the relations below applied to these terms, and admitting nested arithmetic
/// would widen the fragment past what the procedure decides.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExtentTerm {
    /// A declared extent symbol.
    Symbol(ShapeSymbol),
    /// A literal extent.
    Constant(u64),
}

impl ExtentTerm {
    /// Returns the governed tag of this term kind, exhaustively.
    const fn tag(&self) -> u8 {
        match self {
            Self::Symbol(_) => 0x01,
            Self::Constant(_) => 0x02,
        }
    }

    /// Returns the symbol this term names, if it names one.
    #[must_use]
    pub const fn symbol(&self) -> Option<&ShapeSymbol> {
        match self {
            Self::Symbol(symbol) => Some(symbol),
            Self::Constant(_) => None,
        }
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        match self {
            Self::Symbol(symbol) => symbol.encode(bytes),
            Self::Constant(value) => bytes.extend_from_slice(&value.to_be_bytes()),
        }
    }
}

impl fmt::Display for ExtentTerm {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbol(symbol) => write!(formatter, "{symbol}"),
            Self::Constant(value) => write!(formatter, "{value}"),
        }
    }
}

/// One relation over extent terms.
///
/// The variants realize the kinds `docs/ir.md` names for the constraint
/// environment: ordinary and fixed-additive extent equalities, divisibility,
/// nonnegativity, intervals, and factorization relationships.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExtentRelation {
    /// `left == right`.
    Equal {
        /// Left side.
        left: ExtentTerm,
        /// Right side.
        right: ExtentTerm,
    },
    /// `sum == left + right` in mathematical nonnegative-integer arithmetic.
    ///
    /// Fixed at two addends so admitting it does not turn [`ExtentTerm`] into a
    /// recursively nestable expression language. A longer sum needs its own
    /// governed relation rather than an unbound intermediate symbol.
    AdditiveEquality {
        /// The extent equal to the two-addend sum.
        sum: ExtentTerm,
        /// First addend.
        left: ExtentTerm,
        /// Second addend.
        right: ExtentTerm,
    },
    /// `divisor` divides `dividend` exactly.
    Divisible {
        /// The term being divided.
        dividend: ExtentTerm,
        /// The divisor. Nonzero, because divisibility by zero is not a relation.
        divisor: NonZeroU64,
    },
    /// `minuend - subtrahend >= 0`.
    ///
    /// The nonnegativity kind. A bare extent is unsigned and so trivially
    /// nonnegative; the difference form is the one that constrains anything.
    NonNegativeDifference {
        /// The term the difference is taken from.
        minuend: ExtentTerm,
        /// The term subtracted.
        subtrahend: ExtentTerm,
    },
    /// `lower <= term <= upper`, inclusive at both ends.
    Interval {
        /// The bounded term.
        term: ExtentTerm,
        /// Inclusive lower bound.
        lower: u64,
        /// Inclusive upper bound.
        upper: u64,
    },
    /// `product == factors[0] * factors[1] * ...`.
    Factorization {
        /// The composed extent.
        product: ExtentTerm,
        /// Its factors, in the order the composition names them.
        factors: Vec<ExtentTerm>,
    },
}

impl ExtentRelation {
    /// Returns the canonical spelling used by environment storage and identity.
    fn canonicalized(self) -> Self {
        match self {
            Self::AdditiveEquality { sum, left, right } => {
                Self::additive_equality(sum, left, right)
            }
            relation => relation,
        }
    }

    /// Asserts that two terms are equal.
    #[must_use]
    pub const fn equal(left: ExtentTerm, right: ExtentTerm) -> Self {
        Self::Equal { left, right }
    }

    /// Asserts that `sum == left + right`.
    ///
    /// Addends are sorted because mathematical addition is commutative; the two
    /// authoring orders therefore produce one canonical relation and identity.
    #[must_use]
    pub fn additive_equality(sum: ExtentTerm, left: ExtentTerm, right: ExtentTerm) -> Self {
        let (left, right) = if left <= right {
            (left, right)
        } else {
            (right, left)
        };
        Self::AdditiveEquality { sum, left, right }
    }

    /// Asserts that `divisor` divides `dividend`.
    #[must_use]
    pub const fn divisible(dividend: ExtentTerm, divisor: NonZeroU64) -> Self {
        Self::Divisible { dividend, divisor }
    }

    /// Asserts that `minuend - subtrahend` is nonnegative.
    #[must_use]
    pub const fn non_negative_difference(minuend: ExtentTerm, subtrahend: ExtentTerm) -> Self {
        Self::NonNegativeDifference {
            minuend,
            subtrahend,
        }
    }

    /// Bounds a term to an inclusive interval.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::EmptyInterval`] when `lower` exceeds `upper`.
    /// An inverted interval is a malformed relation rather than a contradictory
    /// one — no environment is needed to see it is unwritable — so it is refused
    /// where it is written instead of being carried to the decision procedure.
    pub fn interval(term: ExtentTerm, lower: u64, upper: u64) -> Result<Self, ShapeEnvError> {
        if lower > upper {
            return Err(ShapeEnvError::EmptyInterval { lower, upper });
        }
        Ok(Self::Interval { term, lower, upper })
    }

    /// Asserts that `product` is the product of `factors`.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeEnvError::DegenerateFactorization`] for fewer than two
    /// factors. A one-factor composition is an equality and a zero-factor one
    /// asserts the product is `1`; spelling either as a factorization would let
    /// two different assertions share the kind that carries composition
    /// structure into identity.
    pub fn factorization(
        product: ExtentTerm,
        factors: Vec<ExtentTerm>,
    ) -> Result<Self, ShapeEnvError> {
        if factors.len() < 2 {
            return Err(ShapeEnvError::DegenerateFactorization {
                factors: factors.len(),
            });
        }
        Ok(Self::Factorization { product, factors })
    }

    /// Returns the governed tag of this relation kind, exhaustively.
    const fn tag(&self) -> u8 {
        match self {
            Self::Equal { .. } => 0x01,
            Self::Divisible { .. } => 0x02,
            Self::NonNegativeDifference { .. } => 0x03,
            Self::Interval { .. } => 0x04,
            Self::Factorization { .. } => 0x05,
            Self::AdditiveEquality { .. } => 0x06,
        }
    }

    /// Visits every term this relation mentions.
    fn for_each_term(&self, mut visit: impl FnMut(&ExtentTerm)) {
        match self {
            Self::Equal { left, right } => {
                visit(left);
                visit(right);
            }
            Self::AdditiveEquality { sum, left, right } => {
                visit(sum);
                visit(left);
                visit(right);
            }
            Self::Divisible { dividend, .. } => visit(dividend),
            Self::NonNegativeDifference {
                minuend,
                subtrahend,
            } => {
                visit(minuend);
                visit(subtrahend);
            }
            Self::Interval { term, .. } => visit(term),
            Self::Factorization { product, factors } => {
                visit(product);
                for factor in factors {
                    visit(factor);
                }
            }
        }
    }

    /// Visits every declared symbol this relation mentions.
    pub fn for_each_symbol(&self, mut visit: impl FnMut(&ShapeSymbol)) {
        self.for_each_term(|term| {
            if let Some(symbol) = term.symbol() {
                visit(symbol);
            }
        });
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.tag());
        match self {
            Self::Equal { left, right } => {
                left.encode(bytes);
                right.encode(bytes);
            }
            Self::AdditiveEquality { sum, left, right } => {
                sum.encode(bytes);
                left.encode(bytes);
                right.encode(bytes);
            }
            Self::Divisible { dividend, divisor } => {
                dividend.encode(bytes);
                bytes.extend_from_slice(&divisor.get().to_be_bytes());
            }
            Self::NonNegativeDifference {
                minuend,
                subtrahend,
            } => {
                minuend.encode(bytes);
                subtrahend.encode(bytes);
            }
            Self::Interval { term, lower, upper } => {
                term.encode(bytes);
                bytes.extend_from_slice(&lower.to_be_bytes());
                bytes.extend_from_slice(&upper.to_be_bytes());
            }
            Self::Factorization { product, factors } => {
                product.encode(bytes);
                crate::identity::push_len(bytes, factors.len());
                for factor in factors {
                    factor.encode(bytes);
                }
            }
        }
    }
}

impl fmt::Display for ExtentRelation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equal { left, right } => write!(formatter, "{left} == {right}"),
            Self::AdditiveEquality { sum, left, right } => {
                write!(formatter, "{sum} == {left} + {right}")
            }
            Self::Divisible { dividend, divisor } => write!(formatter, "{divisor} | {dividend}"),
            Self::NonNegativeDifference {
                minuend,
                subtrahend,
            } => write!(formatter, "{minuend} - {subtrahend} >= 0"),
            Self::Interval { term, lower, upper } => {
                write!(formatter, "{lower} <= {term} <= {upper}")
            }
            Self::Factorization { product, factors } => {
                write!(formatter, "{product} ==")?;
                for (position, factor) in factors.iter().enumerate() {
                    if position == 0 {
                        write!(formatter, " {factor}")?;
                    } else {
                        write!(formatter, " * {factor}")?;
                    }
                }
                Ok(())
            }
        }
    }
}

/// One constraint required for the program's expressions to be defined.
///
/// Failure of one of these is an invalid-input diagnostic, which is why a
/// contradictory set fails `ShapeEnvBuilder::build` rather than being reported
/// as an unavailable optimization. It carries [`FactProvenance`] because the
/// contract requires facts to record how they were established.
///
/// There is no constructor, setter, or conversion that re-records an existing
/// constraint under a different provenance. That absence is what enforces
/// "inferred or proven facts may not silently become additional
/// frontend-required semantics": a [`FactProvenance::StaticallyProven`]
/// constraint has no path to becoming a [`FactProvenance::FrontendRequired`]
/// one, and two assertions of the same relation under different provenance stay
/// two constraints rather than being merged into whichever was recorded first.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticInputConstraint {
    relation: ExtentRelation,
    provenance: FactProvenance,
}

impl SemanticInputConstraint {
    /// Records one semantic input constraint with its provenance.
    ///
    /// This is the authoritative storage boundary for a semantic relation, so
    /// it canonicalizes even a directly constructed public enum variant.
    #[must_use]
    pub fn new(relation: ExtentRelation, provenance: FactProvenance) -> Self {
        Self {
            relation: relation.canonicalized(),
            provenance,
        }
    }

    /// Returns the relation asserted.
    #[must_use]
    pub const fn relation(&self) -> &ExtentRelation {
        &self.relation
    }

    /// Returns how this constraint was established.
    #[must_use]
    pub const fn provenance(&self) -> FactProvenance {
        self.provenance
    }

    pub(super) fn encode(&self, bytes: &mut Vec<u8>) {
        self.relation.encode(bytes);
        bytes.push(self.provenance.tag());
    }
}

/// Which planning decision a variant guard qualifies.
///
/// `docs/ir.md`: "Later guards also record provenance as storage-applicability,
/// schedule-applicability, target-compatibility, or dispatch-safety
/// predicates." This is that vocabulary. It stands where a semantic input
/// constraint carries [`FactProvenance`], which is what makes the two kinds
/// structurally non-interchangeable rather than two flags on one record.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GuardApplicability {
    /// The guard qualifies a storage choice.
    Storage,
    /// The guard qualifies a schedule choice.
    Schedule,
    /// The guard qualifies target compatibility.
    TargetCompatibility,
    /// The guard qualifies dispatch safety.
    DispatchSafety,
}

/// One predicate required by a particular optimization, not by the program.
///
/// Failure selects another valid plan or fallback, so an unsatisfiable guard
/// does not reject the environment. An *undecidable* guard does: a relation
/// outside the supported fragment leaves the variant's selectability unknown,
/// and treating unknown as satisfiable would be the silently weaker answer the
/// contract forbids.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantGuard {
    relation: ExtentRelation,
    applicability: GuardApplicability,
}

impl VariantGuard {
    /// Records one variant guard against the planning decision it qualifies.
    ///
    /// This is the authoritative storage boundary for a guard relation, so it
    /// canonicalizes even a directly constructed public enum variant.
    #[must_use]
    pub fn new(relation: ExtentRelation, applicability: GuardApplicability) -> Self {
        Self {
            relation: relation.canonicalized(),
            applicability,
        }
    }

    /// Returns the relation the guard requires.
    #[must_use]
    pub const fn relation(&self) -> &ExtentRelation {
        &self.relation
    }

    /// Returns which planning decision this guard qualifies.
    #[must_use]
    pub const fn applicability(&self) -> GuardApplicability {
        self.applicability
    }
}

/// Why a relation lies outside the supported arithmetic fragment.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FragmentViolation {
    /// A factorization left more than one of its terms undetermined.
    ///
    /// `p == a * b` with both `a` and `b` unknown is nonlinear integer
    /// arithmetic, which the interval–congruence procedure does not decide.
    UnderdeterminedFactorization {
        /// How many of the relation's terms had no determined constant.
        undetermined: usize,
    },
    /// The canonical model did not satisfy an additive equality with multiple
    /// undetermined terms.
    UnderdeterminedAdditiveEquality {
        /// How many term positions were not pinned to constants.
        undetermined: usize,
    },
}

impl fmt::Display for FragmentViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnderdeterminedFactorization { undetermined } => write!(
                formatter,
                "{undetermined} undetermined terms in one factorization; the supported fragment admits at most one"
            ),
            Self::UnderdeterminedAdditiveEquality { undetermined } => write!(
                formatter,
                "{undetermined} undetermined terms in one additive equality and the canonical model does not satisfy it"
            ),
        }
    }
}

/// The explained reason one constraint set is contradictory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConstraintConflict {
    /// One equality class was pinned to two different constants.
    ConflictingConstants {
        /// Canonically least symbol of the class.
        symbol: ShapeSymbol,
        /// The constant established first.
        first: u64,
        /// The constant that conflicted with it.
        second: u64,
    },
    /// A relation over literals alone is false.
    FalseGroundRelation {
        /// The false relation.
        relation: ExtentRelation,
    },
    /// A fully observed additive equality has unequal sides.
    AdditiveEqualityMismatch {
        /// The rejected three-term relation.
        relation: ExtentRelation,
        /// Observed value of the sum term.
        sum: u128,
        /// Exact mathematical sum of the two observed addends.
        addends: u128,
    },
    /// One observed addend exceeds the observed sum, forcing a negative extent.
    AddendExceedsSum {
        /// The rejected three-term relation.
        relation: ExtentRelation,
        /// Observed value of the sum term.
        sum: u128,
        /// Observed value of the addend that already exceeds the sum.
        addend: u128,
        /// The undetermined addend that would have to be negative.
        remaining: ExtentTerm,
    },
    /// A factorization has no integer solution for its undetermined term.
    IndivisibleFactorization {
        /// The factorization.
        relation: ExtentRelation,
        /// The product the determined terms must reach.
        product: u128,
        /// The product of the determined factors.
        determined: u128,
    },
    /// A class admits no value: no multiple of its modulus lies in its interval.
    EmptyDomain {
        /// Canonically least symbol of the class.
        symbol: ShapeSymbol,
        /// The class's implied lower bound after propagation.
        lower: u128,
        /// The class's implied upper bound.
        upper: u128,
        /// The class's implied modulus.
        modulus: u128,
    },
}

impl fmt::Display for ConstraintConflict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingConstants {
                symbol,
                first,
                second,
            } => write!(
                formatter,
                "{symbol} is required to be both {first} and {second}"
            ),
            Self::FalseGroundRelation { relation } => write!(formatter, "`{relation}` is false"),
            Self::AdditiveEqualityMismatch {
                relation,
                sum,
                addends,
            } => write!(
                formatter,
                "`{relation}` is false: the sum term is {sum} and the two addends total {addends}"
            ),
            Self::AddendExceedsSum {
                relation,
                sum,
                addend,
                remaining,
            } => write!(
                formatter,
                "`{relation}` has no nonnegative solution: addend {addend} exceeds sum {sum}, so {remaining} would have to be negative"
            ),
            Self::IndivisibleFactorization {
                relation,
                product,
                determined,
            } => write!(
                formatter,
                "`{relation}` has no integer solution: {determined} does not divide {product}"
            ),
            Self::EmptyDomain {
                symbol,
                lower,
                upper,
                modulus,
            } => write!(
                formatter,
                "{symbol} admits no value: no multiple of {modulus} lies in [{lower}, {upper}]"
            ),
        }
    }
}

/// Disjoint-set forest over symbol slots.
struct Classes {
    parent: Vec<usize>,
}

impl Classes {
    fn new(len: usize) -> Self {
        Self {
            parent: (0..len).collect(),
        }
    }

    fn find(&mut self, slot: usize) -> usize {
        let mut root = slot;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut walk = slot;
        while self.parent[walk] != root {
            let next = self.parent[walk];
            self.parent[walk] = root;
            walk = next;
        }
        root
    }

    fn union(&mut self, left: usize, right: usize) {
        let (left, right) = (self.find(left), self.find(right));
        if left != right {
            // Bias toward the lower slot so the canonically least symbol of a
            // class stays reachable as its representative-independent identity.
            let (keep, absorb) = if left < right {
                (left, right)
            } else {
                (right, left)
            };
            self.parent[absorb] = keep;
        }
    }
}

/// The per-class interval-and-congruence domain the procedure meets into.
struct Domains {
    lower: Vec<u128>,
    upper: Vec<u128>,
    modulus: Vec<u128>,
}

fn gcd(left: u128, right: u128) -> u128 {
    let (mut left, mut right) = (left, right);
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

/// Least common multiple, saturating at [`IMPOSSIBLE`].
///
/// A modulus above the extent domain admits only zero within it, exactly as the
/// unsaturated value would, so saturation preserves the decision.
fn lcm(left: u128, right: u128) -> u128 {
    match (left / gcd(left, right)).checked_mul(right) {
        Some(value) if value <= MAX_EXTENT => value,
        _ => IMPOSSIBLE,
    }
}

/// Rounds `value` up to the next multiple of `modulus`, clamped at [`IMPOSSIBLE`].
fn raise_to_multiple(value: u128, modulus: u128) -> u128 {
    match value.div_ceil(modulus).checked_mul(modulus) {
        Some(raised) if raised <= MAX_EXTENT => raised,
        _ => IMPOSSIBLE,
    }
}

/// Multiplies extents, saturating at [`IMPOSSIBLE`].
///
/// A single zero factor makes the product zero regardless of the rest, so it is
/// answered first; every remaining factor is at least one, which makes the fold
/// monotonically non-decreasing and saturation exact.
fn extent_product(factors: &[u64]) -> u128 {
    if factors.contains(&0) {
        return 0;
    }
    let mut product: u128 = 1;
    for factor in factors {
        match product.checked_mul(u128::from(*factor)) {
            Some(next) if next <= MAX_EXTENT => product = next,
            _ => return IMPOSSIBLE,
        }
    }
    product
}

/// Decides whether one relation set is satisfiable over the bound environment.
///
/// The entries are the environment's symbols with their root bindings, in the
/// canonical order `build` established; a [`Static`](super::BindingSource::Static) binding
/// participates as a constant pin, so a constraint that contradicts a statically
/// bound extent is caught here rather than by a later consumer.
///
/// # Errors
///
/// Returns [`ShapeEnvError::UnsupportedRelation`] for a relation outside the
/// supported fragment, [`ShapeEnvError::ContradictoryConstraints`] for a
/// contradictory set, and [`ShapeEnvError::ConstraintOnUndeclaredSymbol`] if a
/// relation names a symbol the environment does not declare.
pub(super) fn decide(
    entries: &[(ShapeSymbol, RootBinding)],
    relations: &[&ExtentRelation],
) -> Result<(), ShapeEnvError> {
    solve(entries, relations).map(|_| ())
}

/// The implied closed interval one symbol's extent is confined to.
///
/// Every admitted value of the symbol lies inside it: the lower bound is only
/// ever raised by an implied step and the upper bound is only ever met with an
/// asserted one. It is therefore sound to prove a bound against, and it is
/// deliberately not a claim that every value inside it is admissible — a
/// congruence can exclude interior values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtentInterval {
    /// Smallest value any model may assign.
    pub lower: u64,
    /// Largest value any model may assign.
    pub upper: u64,
}

impl ExtentInterval {
    /// Returns whether the environment admits every representable extent.
    ///
    /// **A symbol nothing constrains is not absent from a solution; it is
    /// present with the whole extent domain.** The decision procedure seeds
    /// every symbol at the full extent domain and narrows from there, so "the
    /// environment says
    /// nothing about this extent" reads as an upper bound still sitting at the
    /// domain's ceiling rather than as a missing interval. A caller that tested
    /// for a missing interval instead would never fire, because one is only
    /// returned when a class bound *exceeds* the domain.
    ///
    /// This is the condition a frontend can act on: an extent bounded nowhere
    /// above cannot be proved against any axis, and the remedy is to state a
    /// constraint rather than to retry.
    #[must_use]
    pub const fn states_no_upper_bound(&self) -> bool {
        // `MAX_EXTENT` is `IMPOSSIBLE - 1` and `IMPOSSIBLE` is `1 << 64`, so the
        // domain ceiling is exactly `u64::MAX`; asserted rather than assumed so
        // widening the domain fails here instead of silently disabling this.
        const { assert!(MAX_EXTENT == u64::MAX as u128) };
        self.upper == u64::MAX
    }
}

/// The per-symbol result of one satisfiable constraint set.
///
/// Recomputed by every caller rather than stored on the environment: the
/// contract excludes "derived solver caches" from canonical identity, and
/// deriving nothing that could be stored is the simplest way to hold that.
pub(super) struct Solution {
    classes: Classes,
    domains: Domains,
}

impl Solution {
    /// Returns the implied interval of the symbol at `slot`.
    ///
    /// `None` when the class bound exceeds the extent domain, which carries no
    /// information a consumer can prove anything against.
    pub(super) fn interval(&mut self, slot: usize) -> Option<ExtentInterval> {
        let root = self.classes.find(slot);
        let lower = u64::try_from(self.domains.lower[root]).ok()?;
        let upper = u64::try_from(self.domains.upper[root]).ok()?;
        Some(ExtentInterval { lower, upper })
    }

    /// Returns whether two slots were forced into one equality class.
    ///
    /// Sound as a proof of equality because a class is only ever merged by
    /// something that forces it: [`merge_equalities`] unions on an asserted
    /// `left == right` between two symbols, and [`merge_comparison_cycles`]
    /// unions a `>=` cycle, which the module documentation records as forcing
    /// equality. Nothing merges on a coincidence of bounds.
    ///
    /// The converse does not hold and no caller may assume it: two symbols in
    /// different classes may still be equal in every model — pinned to the same
    /// constant, for instance — so `false` means *not proved here*.
    pub(super) fn same_class(&mut self, left: usize, right: usize) -> bool {
        self.classes.find(left) == self.classes.find(right)
    }
}

/// Decides one relation set and retains the per-class domains it established.
pub(super) fn solve(
    entries: &[(ShapeSymbol, RootBinding)],
    relations: &[&ExtentRelation],
) -> Result<Solution, ShapeEnvError> {
    let index: BTreeMap<&ShapeSymbol, usize> = entries
        .iter()
        .enumerate()
        .map(|(slot, (symbol, _))| (symbol, slot))
        .collect();
    let mut classes = Classes::new(entries.len());

    merge_equalities(&mut classes, &index, relations)?;
    merge_comparison_cycles(&mut classes, &index, relations)?;
    let constants = resolve_constants(&mut classes, &index, entries, relations)?;
    check_fragment(&mut classes, &index, &constants, relations)?;

    let mut domains = seed_domains(&mut classes, &constants, entries.len());
    let edges = apply_relations(&mut classes, &index, &constants, relations, &mut domains)?;
    propagate(&mut classes, entries.len(), &edges, &mut domains);
    report_empty_domain(&mut classes, entries, &domains)?;
    check_additive_model(&mut classes, &index, &constants, relations, &domains)?;
    Ok(Solution { classes, domains })
}

/// Resolves one symbol to its slot, failing closed on an undeclared one.
fn slot(
    index: &BTreeMap<&ShapeSymbol, usize>,
    symbol: &ShapeSymbol,
) -> Result<usize, ShapeEnvError> {
    index
        .get(symbol)
        .copied()
        .ok_or_else(|| ShapeEnvError::ConstraintOnUndeclaredSymbol {
            symbol: symbol.clone(),
        })
}

fn contradiction(conflict: ConstraintConflict) -> ShapeEnvError {
    ShapeEnvError::ContradictoryConstraints {
        conflict: Box::new(conflict),
    }
}

/// Returns the canonically least symbol of `slot`'s class.
///
/// Entries arrive sorted, so the first member found is the least one, and the
/// diagnostic names the same symbol however the class was assembled.
fn class_symbol(
    classes: &mut Classes,
    entries: &[(ShapeSymbol, RootBinding)],
    slot: usize,
) -> ShapeSymbol {
    let root = classes.find(slot);
    for (position, (symbol, _)) in entries.iter().enumerate() {
        if classes.find(position) == root {
            return symbol.clone();
        }
    }
    entries[slot].0.clone()
}

/// Merges the classes named by symbol equalities and refutes false ground ones.
fn merge_equalities(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    relations: &[&ExtentRelation],
) -> Result<(), ShapeEnvError> {
    for relation in relations {
        let ExtentRelation::Equal { left, right } = relation else {
            continue;
        };
        match (left, right) {
            (ExtentTerm::Symbol(left), ExtentTerm::Symbol(right)) => {
                let (left, right) = (slot(index, left)?, slot(index, right)?);
                classes.union(left, right);
            }
            (ExtentTerm::Constant(left), ExtentTerm::Constant(right)) if left != right => {
                return Err(contradiction(ConstraintConflict::FalseGroundRelation {
                    relation: (*relation).clone(),
                }));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Merges every strongly connected component of the `>=` graph.
///
/// `a >= b` together with `b >= a` forces `a == b`, so collapsing the component
/// is sound; it is also what leaves an acyclic graph, which is what makes the
/// bounded propagation below both terminating and complete.
fn merge_comparison_cycles(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    relations: &[&ExtentRelation],
) -> Result<(), ShapeEnvError> {
    loop {
        let edges = comparison_edges(classes, index, relations)?;
        let mut adjacency: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
        for &(greater, lesser) in &edges {
            adjacency.entry(greater).or_default().insert(lesser);
        }

        let mut merged = false;
        for &(greater, lesser) in &edges {
            if classes.find(greater) != classes.find(lesser) && reaches(&adjacency, lesser, greater)
            {
                classes.union(greater, lesser);
                merged = true;
            }
        }
        if !merged {
            return Ok(());
        }
    }
}

/// Collects the symbol-to-symbol `>=` edges between distinct current classes.
fn comparison_edges(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    relations: &[&ExtentRelation],
) -> Result<Vec<(usize, usize)>, ShapeEnvError> {
    let mut edges = Vec::new();
    for relation in relations {
        let ExtentRelation::NonNegativeDifference {
            minuend: ExtentTerm::Symbol(minuend),
            subtrahend: ExtentTerm::Symbol(subtrahend),
        } = relation
        else {
            continue;
        };
        let greater = classes.find(slot(index, minuend)?);
        let lesser = classes.find(slot(index, subtrahend)?);
        if greater != lesser {
            edges.push((greater, lesser));
        }
    }
    Ok(edges)
}

fn reaches(adjacency: &BTreeMap<usize, BTreeSet<usize>>, from: usize, target: usize) -> bool {
    let mut seen = BTreeSet::new();
    let mut stack = vec![from];
    while let Some(node) = stack.pop() {
        if node == target {
            return true;
        }
        if !seen.insert(node) {
            continue;
        }
        if let Some(next) = adjacency.get(&node) {
            stack.extend(next.iter().copied());
        }
    }
    false
}

/// Determines each class's constant from static bindings and literal equalities.
fn resolve_constants(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    entries: &[(ShapeSymbol, RootBinding)],
    relations: &[&ExtentRelation],
) -> Result<Vec<Option<u64>>, ShapeEnvError> {
    let mut constants = vec![None; entries.len()];

    for (position, (_, binding)) in entries.iter().enumerate() {
        if let Some(extent) = binding.source().static_extent() {
            pin(classes, entries, &mut constants, position, extent.get())?;
        }
    }

    for relation in relations {
        let ExtentRelation::Equal { left, right } = relation else {
            continue;
        };
        let pinned = match (left, right) {
            (ExtentTerm::Symbol(symbol), ExtentTerm::Constant(value))
            | (ExtentTerm::Constant(value), ExtentTerm::Symbol(symbol)) => {
                Some((slot(index, symbol)?, *value))
            }
            _ => None,
        };
        if let Some((position, value)) = pinned {
            pin(classes, entries, &mut constants, position, value)?;
        }
    }

    Ok(constants)
}

fn pin(
    classes: &mut Classes,
    entries: &[(ShapeSymbol, RootBinding)],
    constants: &mut [Option<u64>],
    position: usize,
    value: u64,
) -> Result<(), ShapeEnvError> {
    let root = classes.find(position);
    match constants[root] {
        None => {
            constants[root] = Some(value);
            Ok(())
        }
        Some(existing) if existing == value => Ok(()),
        Some(existing) => Err(contradiction(ConstraintConflict::ConflictingConstants {
            symbol: class_symbol(classes, entries, position),
            first: existing,
            second: value,
        })),
    }
}

/// A term seen through the equality classes: either a known extent or a symbol
/// whose value the environment does not determine.
///
/// Total by construction, so a caller never has to assert that a term it
/// already classified is a symbol.
enum Resolved<'a> {
    /// A literal, or a symbol whose class holds a constant.
    Known(u64),
    /// A symbol whose class holds no constant.
    Free(&'a ShapeSymbol),
}

/// Resolves a term against the equality classes.
fn resolve<'term>(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    constants: &[Option<u64>],
    term: &'term ExtentTerm,
) -> Result<Resolved<'term>, ShapeEnvError> {
    match term {
        ExtentTerm::Constant(value) => Ok(Resolved::Known(*value)),
        ExtentTerm::Symbol(symbol) => {
            let root = classes.find(slot(index, symbol)?);
            Ok(constants[root].map_or(Resolved::Free(symbol), Resolved::Known))
        }
    }
}

/// Refuses every relation outside the supported fragment.
fn check_fragment(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    constants: &[Option<u64>],
    relations: &[&ExtentRelation],
) -> Result<(), ShapeEnvError> {
    for relation in relations {
        let ExtentRelation::Factorization { product, factors } = relation else {
            continue;
        };
        let mut undetermined = 0_usize;
        let mut count = |term: &ExtentTerm| -> Result<(), ShapeEnvError> {
            if matches!(resolve(classes, index, constants, term)?, Resolved::Free(_)) {
                undetermined += 1;
            }
            Ok(())
        };
        count(product)?;
        for factor in factors {
            count(factor)?;
        }
        if undetermined > 1 {
            return Err(ShapeEnvError::UnsupportedRelation {
                relation: Box::new((*relation).clone()),
                violation: FragmentViolation::UnderdeterminedFactorization { undetermined },
            });
        }
    }
    Ok(())
}

/// Opens every class at the full extent domain, narrowed by its constant.
fn seed_domains(classes: &mut Classes, constants: &[Option<u64>], len: usize) -> Domains {
    let mut domains = Domains {
        lower: vec![0; len],
        upper: vec![MAX_EXTENT; len],
        modulus: vec![1; len],
    };
    for position in 0..len {
        let root = classes.find(position);
        if let Some(value) = constants[root] {
            domains.lower[root] = u128::from(value);
            domains.upper[root] = u128::from(value);
        }
    }
    domains
}

/// Meets every relation into the per-class domains, returning the `>=` edges.
fn apply_relations(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    constants: &[Option<u64>],
    relations: &[&ExtentRelation],
    domains: &mut Domains,
) -> Result<Vec<(usize, usize)>, ShapeEnvError> {
    let mut edges = Vec::new();
    for relation in relations {
        match relation {
            // Merged and pinned already; a literal-only equality was refuted in
            // `merge_equalities` when false and asserts nothing when true.
            ExtentRelation::Equal { .. } => {}
            ExtentRelation::AdditiveEquality { .. } => {
                apply_additive_equality(classes, index, constants, relation, domains)?;
            }
            ExtentRelation::Interval { term, lower, upper } => match term {
                ExtentTerm::Symbol(symbol) => {
                    let root = classes.find(slot(index, symbol)?);
                    domains.lower[root] = domains.lower[root].max(u128::from(*lower));
                    domains.upper[root] = domains.upper[root].min(u128::from(*upper));
                }
                ExtentTerm::Constant(value) => {
                    if value < lower || value > upper {
                        return Err(contradiction(ConstraintConflict::FalseGroundRelation {
                            relation: (*relation).clone(),
                        }));
                    }
                }
            },
            ExtentRelation::Divisible { dividend, divisor } => match dividend {
                ExtentTerm::Symbol(symbol) => {
                    let root = classes.find(slot(index, symbol)?);
                    domains.modulus[root] = lcm(domains.modulus[root], u128::from(divisor.get()));
                }
                ExtentTerm::Constant(value) => {
                    if value % divisor.get() != 0 {
                        return Err(contradiction(ConstraintConflict::FalseGroundRelation {
                            relation: (*relation).clone(),
                        }));
                    }
                }
            },
            ExtentRelation::NonNegativeDifference {
                minuend,
                subtrahend,
            } => {
                apply_comparison(
                    classes, index, relation, minuend, subtrahend, domains, &mut edges,
                )?;
            }
            ExtentRelation::Factorization { product, factors } => {
                apply_factorization(
                    classes, index, constants, relation, product, factors, domains,
                )?;
            }
        }
    }
    Ok(edges)
}

/// Solves an additive equality with at most one free term and defers the
/// multi-free case to [`check_additive_model`].
fn apply_additive_equality(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    constants: &[Option<u64>],
    relation: &ExtentRelation,
    domains: &mut Domains,
) -> Result<(), ShapeEnvError> {
    let ExtentRelation::AdditiveEquality { sum, left, right } = relation else {
        return Ok(());
    };
    let (sum_value, left_value, right_value) = (
        resolve(classes, index, constants, sum)?,
        resolve(classes, index, constants, left)?,
        resolve(classes, index, constants, right)?,
    );
    match (sum_value, left_value, right_value) {
        (Resolved::Known(sum_value), Resolved::Known(left), Resolved::Known(right)) => {
            let addends = u128::from(left) + u128::from(right);
            if u128::from(sum_value) != addends {
                return Err(additive_mismatch(relation, u128::from(sum_value), addends));
            }
        }
        (Resolved::Free(symbol), Resolved::Known(left), Resolved::Known(right)) => {
            pin_domain(
                classes,
                index,
                symbol,
                u128::from(left) + u128::from(right),
                domains,
            )?;
        }
        (Resolved::Known(sum_value), Resolved::Free(symbol), Resolved::Known(right)) => {
            let sum_value = u128::from(sum_value);
            let right = u128::from(right);
            let Some(left) = sum_value.checked_sub(right) else {
                return Err(addend_exceeds_sum(relation, sum_value, right, symbol));
            };
            pin_domain(classes, index, symbol, left, domains)?;
        }
        (Resolved::Known(sum_value), Resolved::Known(left), Resolved::Free(symbol)) => {
            let sum_value = u128::from(sum_value);
            let left = u128::from(left);
            let Some(right) = sum_value.checked_sub(left) else {
                return Err(addend_exceeds_sum(relation, sum_value, left, symbol));
            };
            pin_domain(classes, index, symbol, right, domains)?;
        }
        _ => {}
    }
    Ok(())
}

fn additive_mismatch(relation: &ExtentRelation, sum: u128, addends: u128) -> ShapeEnvError {
    contradiction(ConstraintConflict::AdditiveEqualityMismatch {
        relation: relation.clone(),
        sum,
        addends,
    })
}

fn addend_exceeds_sum(
    relation: &ExtentRelation,
    sum: u128,
    addend: u128,
    remaining: &ShapeSymbol,
) -> ShapeEnvError {
    contradiction(ConstraintConflict::AddendExceedsSum {
        relation: relation.clone(),
        sum,
        addend,
        remaining: ExtentTerm::Symbol(remaining.clone()),
    })
}

/// Proves the remaining additive equalities by the exact model the ordinary
/// interval/congruence solver already constructed.
fn check_additive_model(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    constants: &[Option<u64>],
    relations: &[&ExtentRelation],
    domains: &Domains,
) -> Result<(), ShapeEnvError> {
    for relation in relations {
        let ExtentRelation::AdditiveEquality { sum, left, right } = relation else {
            continue;
        };
        let terms = [sum, left, right];
        let mut undetermined = 0;
        let mut values = [0_u128; 3];
        for (position, term) in terms.into_iter().enumerate() {
            values[position] = match resolve(classes, index, constants, term)? {
                Resolved::Known(value) => u128::from(value),
                Resolved::Free(symbol) => {
                    undetermined += 1;
                    let root = classes.find(slot(index, symbol)?);
                    domains.lower[root]
                }
            };
        }
        if values[0] != values[1] + values[2] {
            if undetermined == 0 {
                return Err(additive_mismatch(
                    relation,
                    values[0],
                    values[1] + values[2],
                ));
            }
            return Err(ShapeEnvError::UnsupportedRelation {
                relation: Box::new((*relation).clone()),
                violation: FragmentViolation::UnderdeterminedAdditiveEquality { undetermined },
            });
        }
    }
    Ok(())
}

fn apply_comparison(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    relation: &ExtentRelation,
    minuend: &ExtentTerm,
    subtrahend: &ExtentTerm,
    domains: &mut Domains,
    edges: &mut Vec<(usize, usize)>,
) -> Result<(), ShapeEnvError> {
    match (minuend, subtrahend) {
        (ExtentTerm::Constant(greater), ExtentTerm::Constant(lesser)) => {
            if greater < lesser {
                return Err(contradiction(ConstraintConflict::FalseGroundRelation {
                    relation: relation.clone(),
                }));
            }
        }
        (ExtentTerm::Symbol(symbol), ExtentTerm::Constant(floor)) => {
            let root = classes.find(slot(index, symbol)?);
            domains.lower[root] = domains.lower[root].max(u128::from(*floor));
        }
        (ExtentTerm::Constant(ceiling), ExtentTerm::Symbol(symbol)) => {
            let root = classes.find(slot(index, symbol)?);
            domains.upper[root] = domains.upper[root].min(u128::from(*ceiling));
        }
        (ExtentTerm::Symbol(greater), ExtentTerm::Symbol(lesser)) => {
            let greater = classes.find(slot(index, greater)?);
            let lesser = classes.find(slot(index, lesser)?);
            if greater != lesser {
                edges.push((greater, lesser));
            }
        }
    }
    Ok(())
}

/// Meets one in-fragment factorization, which has at most one undetermined term.
fn apply_factorization(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    constants: &[Option<u64>],
    relation: &ExtentRelation,
    product: &ExtentTerm,
    factors: &[ExtentTerm],
    domains: &mut Domains,
) -> Result<(), ShapeEnvError> {
    let product_value = resolve(classes, index, constants, product)?;
    let mut known = Vec::with_capacity(factors.len());
    let mut free_factor = None;
    for factor in factors {
        match resolve(classes, index, constants, factor)? {
            Resolved::Known(value) => known.push(value),
            Resolved::Free(symbol) => free_factor = Some(symbol),
        }
    }
    let determined_product = extent_product(&known);

    match (product_value, free_factor) {
        // Everything is known: the relation is a ground check.
        (Resolved::Known(value), None) => {
            if u128::from(value) != determined_product {
                return Err(contradiction(ConstraintConflict::FalseGroundRelation {
                    relation: relation.clone(),
                }));
            }
        }
        // The composed extent is the one unknown: it is pinned to the product.
        (Resolved::Free(symbol), None) => {
            pin_domain(classes, index, symbol, determined_product, domains)?;
        }
        // One factor is the unknown: it is pinned to the exact quotient, or the
        // relation has no integer solution at all.
        (Resolved::Known(value), Some(symbol)) => {
            let target = u128::from(value);
            let indivisible = || {
                contradiction(ConstraintConflict::IndivisibleFactorization {
                    relation: relation.clone(),
                    product: target,
                    determined: determined_product,
                })
            };
            let Some(quotient) = target.checked_div(determined_product) else {
                // A zero determined factor makes the product zero whatever the
                // free factor is: the relation holds exactly when the target is
                // zero, and then constrains the free factor not at all.
                if target == 0 {
                    return Ok(());
                }
                return Err(indivisible());
            };
            // Exact division rather than a remainder test: `quotient` is the
            // floor, so the product cannot overflow the target.
            if quotient * determined_product != target {
                return Err(indivisible());
            }
            pin_domain(classes, index, symbol, quotient, domains)?;
        }
        // Two unknowns is outside the fragment and was refused before this, so
        // this arm asserts nothing rather than approximating the relation.
        (Resolved::Free(_), Some(_)) => {}
    }
    Ok(())
}

/// Narrows one symbol's class to a single value.
fn pin_domain(
    classes: &mut Classes,
    index: &BTreeMap<&ShapeSymbol, usize>,
    symbol: &ShapeSymbol,
    value: u128,
    domains: &mut Domains,
) -> Result<(), ShapeEnvError> {
    let root = classes.find(slot(index, symbol)?);
    domains.lower[root] = domains.lower[root].max(value);
    domains.upper[root] = domains.upper[root].min(value);
    Ok(())
}

/// Propagates lower bounds along the acyclic `>=` graph to a fixpoint.
///
/// Cycles were collapsed into equality classes, so the graph is a DAG and one
/// round advances every bound by at least one edge. The number of classes is
/// therefore a sufficient round bound; the early exit is a shortcut, not the
/// termination argument.
fn propagate(classes: &mut Classes, len: usize, edges: &[(usize, usize)], domains: &mut Domains) {
    let roots: Vec<usize> = (0..len)
        .filter(|slot| classes.find(*slot) == *slot)
        .collect();

    for root in &roots {
        domains.lower[*root] = raise_to_multiple(domains.lower[*root], domains.modulus[*root]);
    }

    for _ in 0..roots.len() {
        let mut changed = false;
        for &(greater, lesser) in edges {
            let floor = domains.lower[lesser];
            if floor > domains.lower[greater] {
                domains.lower[greater] = raise_to_multiple(floor, domains.modulus[greater]);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}

/// Reports the canonically least class that admits no value.
fn report_empty_domain(
    classes: &mut Classes,
    entries: &[(ShapeSymbol, RootBinding)],
    domains: &Domains,
) -> Result<(), ShapeEnvError> {
    for position in 0..entries.len() {
        let root = classes.find(position);
        if domains.lower[root] > domains.upper[root] {
            return Err(contradiction(ConstraintConflict::EmptyDomain {
                symbol: class_symbol(classes, entries, position),
                lower: domains.lower[root],
                upper: domains.upper[root],
                modulus: domains.modulus[root],
            }));
        }
    }
    Ok(())
}
