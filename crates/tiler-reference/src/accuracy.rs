//! Certified reference enclosures, and the accuracy decision built on them.
//!
//! ADR 0042 requires the inclusive accuracy comparison to be "evaluated exactly
//! or with certified bounds rather than by rounded floating-point division". For
//! an algebraic reference that is easy: the reference value is rational and the
//! comparison is exact. For a transcendental it is the whole problem — `exp(x)`
//! is irrational at every nonzero representable `x`, so there is no exact
//! rational reference to compare against, and evaluating the reference in host
//! floating point would compare a candidate against another approximation and
//! call the answer a conformance decision.
//!
//! # What a certified enclosure is, and what it refuses to be
//!
//! [`CertifiedEnclosure`] is a pair of exact rationals that provably brackets the
//! reference. It is never an approximation and it is never assumed tight: a
//! caller narrows it by asking for more precision and may not assume any width.
//! Every operation below widens outward, so the bracket property survives
//! composition.
//!
//! The consequence is the point of the module. When the enclosure is too wide to
//! separate `|z - r|` from the tolerance, [`decide_predicate`] returns
//! [`ConformanceDecision::Undecided`] — it does **not** pick the nearer side. A
//! conformance check that resolved its own uncertainty toward "conforms" would be
//! a check that cannot fail, which is the failure mode this repository distrusts
//! most; one that resolved toward "violates" would reject correct
//! implementations. Both are answers the evidence does not support.
//!
//! # Why the arithmetic is exact integers
//!
//! Every value here is a [`ExactRational`], so nothing in this module depends on
//! host floating-point behaviour, rounding mode, optimization level, or profile.
//! The release-profile numerical tests and the dev-profile ones therefore compute
//! the same bracket by construction rather than by coincidence.
//!
//! # Sizing
//!
//! Deliberately sized to a **bounded corpus**. The three L3′ verticals ask for
//! bounded conformance evidence, not an exhaustive sweep of the 2^32 binary32
//! inputs, and this evaluates one argument per call at a precision the caller
//! chooses. An exhaustive sweep is a different claim
//! ([`tiler_ir::semantic::accuracy::ConformanceEvidenceClass::ExhaustiveFinite`])
//! and would need its own harness and its own budget.

use std::error::Error;
use std::fmt;

use tiler_ir::semantic::accuracy::{
    AccuracyContract, AccuracyContractForm, AccuracyPredicate, AccuracyPredicateView,
    BooleanPredicateKind, ExactRational, UlpFormat,
};

/// Maximum series terms one enclosure may accumulate.
///
/// A cap rather than a convergence loop, because a loop whose bound is only its
/// own convergence test cannot report that it failed to converge. Reaching it is
/// [`EnclosureError::PrecisionUnreachable`].
const MAX_SERIES_TERMS: u32 = 512;

/// Maximum halvings the argument reduction may apply.
const MAX_ARGUMENT_HALVINGS: u32 = 24;

/// The width of the binary grid an enclosure rounds outward onto.
///
/// Exact rational arithmetic is exact, and its magnitudes grow without limit if
/// nothing bounds them: squaring an enclosure `s` times squares its denominator
/// `s` times. Rounding each intermediate *outward* onto a fixed grid keeps every
/// magnitude bounded while preserving the bracket, so this is a resource bound
/// rather than an accuracy compromise — a coarser grid produces a wider, still
/// certified, enclosure and therefore more [`ConformanceDecision::Undecided`]
/// answers, never a wrong one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EnclosurePrecision(u32);

impl EnclosurePrecision {
    /// The precision the bounded binary32 corpora use.
    ///
    /// Chosen so that the enclosure is far narrower than one binary32 ULP over
    /// the whole ordinary exponential range: the grid is absolute, so at the
    /// smallest magnitude the corpus reaches — around `2^-150`, where binary32
    /// itself underflows — the relative width is still about `2^-100`.
    #[must_use]
    pub const fn binary32_corpus() -> Self {
        Self(256)
    }

    /// States a grid width in fraction bits.
    ///
    /// A degraded precision is a legitimate request, not an error: it is how a
    /// caller — or a test — observes the enclosure widening and the decision
    /// falling back to `Unknown` rather than guessing.
    #[must_use]
    pub const fn new(fraction_bits: u32) -> Self {
        Self(fraction_bits)
    }

    /// Returns the grid width in fraction bits.
    #[must_use]
    pub const fn fraction_bits(self) -> u32 {
        self.0
    }
}

/// Why one certified enclosure could not be produced.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EnclosureError {
    /// The argument reduction would need more halvings than the governed bound.
    ArgumentTooLarge {
        /// The rejected argument.
        argument: ExactRational,
    },
    /// The series did not reach the requested precision within the term bound.
    PrecisionUnreachable {
        /// Terms accumulated before the bound was reached.
        terms: u32,
    },
    /// The requested grid is too coarse to bracket the result away from zero.
    ///
    /// Reached when a reciprocal would divide by an enclosure endpoint that
    /// rounded down to zero. Refused rather than clamped: a clamp would invent a
    /// bracket the arithmetic did not establish.
    PrecisionTooCoarse,
    /// The argument is outside the function's domain.
    OutsideDomain {
        /// The rejected argument.
        argument: ExactRational,
    },
}

impl EnclosureError {
    /// Returns the stable provider diagnostic code naming this refusal.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::ArgumentTooLarge { .. } => "reference.enclosure.argument-too-large",
            Self::PrecisionUnreachable { .. } => "reference.enclosure.precision-unreachable",
            Self::PrecisionTooCoarse => "reference.enclosure.precision-too-coarse",
            Self::OutsideDomain { .. } => "reference.enclosure.outside-domain",
        }
    }
}

impl fmt::Display for EnclosureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArgumentTooLarge { argument } => write!(
                formatter,
                "reducing {argument} would need more than {MAX_ARGUMENT_HALVINGS} halvings"
            ),
            Self::PrecisionUnreachable { terms } => write!(
                formatter,
                "the series did not reach the requested precision within {terms} terms"
            ),
            Self::PrecisionTooCoarse => formatter
                .write_str("the requested grid is too coarse to bracket the result away from zero"),
            Self::OutsideDomain { argument } => {
                write!(formatter, "{argument} is outside the function's domain")
            }
        }
    }
}

impl Error for EnclosureError {}

/// A pair of exact rationals that provably brackets one real reference value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertifiedEnclosure {
    lower: ExactRational,
    upper: ExactRational,
}

impl CertifiedEnclosure {
    /// States an enclosure whose bracket the caller has established.
    ///
    /// # Panics
    ///
    /// Panics when `lower` exceeds `upper`, which would be an inverted bracket
    /// rather than a wide one — a caller that produced it has a defect, and
    /// carrying it would let every downstream comparison answer nonsense.
    #[must_use]
    pub fn new(lower: ExactRational, upper: ExactRational) -> Self {
        assert!(lower <= upper, "a certified enclosure cannot be inverted");
        Self { lower, upper }
    }

    /// States the degenerate enclosure of one exactly known value.
    #[must_use]
    pub fn exact(value: ExactRational) -> Self {
        Self {
            lower: value.clone(),
            upper: value,
        }
    }

    /// Returns the greatest lower bound this enclosure establishes.
    #[must_use]
    pub const fn lower(&self) -> &ExactRational {
        &self.lower
    }

    /// Returns the least upper bound this enclosure establishes.
    #[must_use]
    pub const fn upper(&self) -> &ExactRational {
        &self.upper
    }

    /// Returns whether this enclosure pins one exact value.
    #[must_use]
    pub fn is_exact(&self) -> bool {
        self.lower == self.upper
    }

    /// Returns whether the bracketed value is provably nonzero.
    #[must_use]
    pub fn excludes_zero(&self) -> bool {
        self.lower.sign() == self.upper.sign() && !self.lower.is_zero()
    }

    /// Returns the width of the bracket.
    #[must_use]
    pub fn width(&self) -> ExactRational {
        self.upper.subtract(&self.lower)
    }

    /// Returns the enclosure of `|value|`.
    #[must_use]
    pub fn magnitude(&self) -> Self {
        if !self.lower.is_negative() {
            return self.clone();
        }
        if self.upper.is_negative() {
            return Self {
                lower: self.upper.abs(),
                upper: self.lower.abs(),
            };
        }
        // The bracket straddles zero, so the magnitude's least value is zero.
        let left = self.lower.abs();
        let right = self.upper.abs();
        Self {
            lower: ExactRational::zero(),
            upper: if left > right { left } else { right },
        }
    }

    /// Returns the enclosure of the product, both factors bracketed.
    ///
    /// # Panics
    ///
    /// Panics only if the four corner products were empty, which the fixed-size
    /// array makes unreachable.
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        let products = [
            self.lower.multiply(&other.lower),
            self.lower.multiply(&other.upper),
            self.upper.multiply(&other.lower),
            self.upper.multiply(&other.upper),
        ];
        let lower = products
            .iter()
            .min()
            .expect("four products are nonempty")
            .clone();
        let upper = products
            .iter()
            .max()
            .expect("four products are nonempty")
            .clone();
        Self { lower, upper }
    }

    /// Returns the enclosure of the reciprocal.
    ///
    /// # Errors
    ///
    /// Returns [`EnclosureError::PrecisionTooCoarse`] when the bracket includes
    /// zero, where the reciprocal is unbounded.
    pub fn reciprocal(&self) -> Result<Self, EnclosureError> {
        if !self.excludes_zero() {
            return Err(EnclosureError::PrecisionTooCoarse);
        }
        let lower = self
            .upper
            .reciprocal()
            .map_err(|_| EnclosureError::PrecisionTooCoarse)?;
        let upper = self
            .lower
            .reciprocal()
            .map_err(|_| EnclosureError::PrecisionTooCoarse)?;
        Ok(Self { lower, upper })
    }

    /// Widens this enclosure outward onto the precision's binary grid.
    #[must_use]
    pub fn coarsen(&self, precision: EnclosurePrecision) -> Self {
        Self {
            lower: self.lower.floor_to_binary_grid(precision.fraction_bits()),
            upper: self.upper.ceil_to_binary_grid(precision.fraction_bits()),
        }
    }
}

/// Returns a certified enclosure of `exp(argument)`.
///
/// # The bracket, and why each step preserves it
///
/// 1. **Negative arguments invert.** `exp(x) = 1 / exp(-x)`, and inverting a
///    positive bracket reverses its endpoints; the reduction runs on the positive
///    side, where the series is well conditioned.
/// 2. **Argument reduction squares.** `exp(y) = exp(y / 2^s)^(2^s)`. `s` is
///    chosen so the reduced argument is at most one half, which is what bounds
///    the tail below.
/// 3. **The series has a rigorous tail bound.** For `0 <= y <= 1/2`, the
///    remainder after the term `T_i = y^i / i!` is `sum_{j >= i} y^j / j! <=
///    T_i / (1 - y) <= 2 * T_i`. The enclosure is `[S, S + 2 * T_i]`, and both
///    ends are exact rationals.
/// 4. **Squaring preserves the bracket** because the reduced value is positive
///    and squaring is monotone there.
///
/// Each intermediate is widened outward onto the precision's grid, so magnitudes
/// stay bounded and the bracket survives.
///
/// # Errors
///
/// Returns [`EnclosureError`] when the argument needs more halvings than the
/// governed bound, when the series cannot reach the requested precision within
/// the term bound, or when the final reciprocal cannot be bracketed away from
/// zero.
///
/// # Panics
///
/// Panics only if the governed halving bound or a grid width bounded by the
/// caller's own [`EnclosurePrecision`] leaves `i32`, which those bounds make
/// unreachable.
pub fn exp_enclosure(
    argument: &ExactRational,
    precision: EnclosurePrecision,
) -> Result<CertifiedEnclosure, EnclosureError> {
    if argument.is_negative() {
        return exp_enclosure(&argument.negate(), precision)?.reciprocal();
    }
    if argument.is_zero() {
        return Ok(CertifiedEnclosure::exact(ExactRational::one()));
    }

    // `argument` lies in `[2^k, 2^(k+1))`, so halving it `k + 2` times puts it at
    // or below one half. A value already below one half needs none.
    let binade = argument
        .floor_log2_abs()
        .expect("a nonzero argument has a binade");
    let halvings = u32::try_from(binade + 2).unwrap_or(0);
    if halvings > MAX_ARGUMENT_HALVINGS {
        return Err(EnclosureError::ArgumentTooLarge {
            argument: argument.clone(),
        });
    }
    let reduced = argument
        .scale_by_power_of_two(-i32::try_from(halvings).expect("a bounded halving count fits i32"));

    // Accumulate until the next term is below the grid, then bound the whole
    // remaining tail by twice that term. The threshold is two grid steps below
    // the grid itself so that the tail cannot be lost to the outward rounding.
    let threshold = ExactRational::power_of_two(
        -i32::try_from(precision.fraction_bits().saturating_add(2))
            .expect("a bounded grid width fits i32"),
    );
    let mut sum = ExactRational::zero();
    let mut term = ExactRational::one();
    let mut terms = 0_u32;
    loop {
        sum = sum.add(&term);
        term = term
            .multiply(&reduced)
            .divide(&ExactRational::from_integer(i128::from(terms) + 1))
            .unwrap_or_else(|_| unreachable!("the divisor is a positive integer"));
        terms += 1;
        if term <= threshold {
            break;
        }
        if terms >= MAX_SERIES_TERMS {
            return Err(EnclosureError::PrecisionUnreachable { terms });
        }
    }
    let mut enclosure =
        CertifiedEnclosure::new(sum.clone(), sum.add(&term.scale_by_power_of_two(1)))
            .coarsen(precision);

    for _ in 0..halvings {
        enclosure = enclosure.multiply(&enclosure.clone()).coarsen(precision);
    }
    Ok(enclosure)
}

/// Returns a certified enclosure of `1 / sqrt(argument)`.
///
/// The square root is bracketed by an exact integer square root on the
/// precision's grid — `lower^2 <= argument < upper^2` holds by construction — and
/// the reciprocal reverses the endpoints. No floating point is involved at any
/// step.
///
/// # Errors
///
/// Returns [`EnclosureError::OutsideDomain`] for a non-positive argument, and
/// [`EnclosureError::PrecisionTooCoarse`] when the grid cannot bracket the root
/// away from zero.
pub fn rsqrt_enclosure(
    argument: &ExactRational,
    precision: EnclosurePrecision,
) -> Result<CertifiedEnclosure, EnclosureError> {
    if argument.is_negative() || argument.is_zero() {
        return Err(EnclosureError::OutsideDomain {
            argument: argument.clone(),
        });
    }
    let (lower, upper) = argument
        .sqrt_enclosure(precision.fraction_bits())
        .map_err(|_| EnclosureError::OutsideDomain {
            argument: argument.clone(),
        })?;
    CertifiedEnclosure::new(lower, upper).reciprocal()
}

/// Why a conformance decision could not be reached.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UndecidedConformance {
    /// The enclosure is too wide to separate the error from the tolerance.
    ///
    /// The honest answer, and the reason this is not a boolean: narrowing the
    /// enclosure decides it, and guessing would make the check unable to fail.
    EnclosureTooWide,
    /// A relative predicate's divisor is not provably nonzero.
    ReferenceNotProvablyNonzero,
    /// The metric is undefined somewhere in the enclosure.
    MetricUndefined,
    /// The predicate measures under a metric this evaluator does not define.
    UnsupportedMetric,
    /// The candidate is a NaN or an infinity, which has no exact value.
    CandidateNotFinite,
    /// The contract's form is defined by an external descriptor this cannot read.
    ///
    /// A named-elementary profile's result set lives in its descriptor, and this
    /// evaluator holds a digest rather than the descriptor's content. `Unknown` is
    /// what it can honestly report.
    NamedProfileNotInterpretable,
    /// No clause of the contract applies at the supplied input point.
    NoApplicableClause,
}

impl UndecidedConformance {
    /// Returns the stable provider diagnostic code naming this outcome.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::EnclosureTooWide => "reference.conformance.enclosure-too-wide",
            Self::ReferenceNotProvablyNonzero => {
                "reference.conformance.reference-not-provably-nonzero"
            }
            Self::MetricUndefined => "reference.conformance.metric-undefined",
            Self::UnsupportedMetric => "reference.conformance.unsupported-metric",
            Self::CandidateNotFinite => "reference.conformance.candidate-not-finite",
            Self::NamedProfileNotInterpretable => {
                "reference.conformance.named-profile-not-interpretable"
            }
            Self::NoApplicableClause => "reference.conformance.no-applicable-clause",
        }
    }
}

impl fmt::Display for UndecidedConformance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EnclosureTooWide => {
                "the reference enclosure is too wide to separate the error from the tolerance"
            }
            Self::ReferenceNotProvablyNonzero => {
                "a relative predicate divides by a reference the enclosure does not prove nonzero"
            }
            Self::MetricUndefined => "the metric is undefined somewhere in the enclosure",
            Self::UnsupportedMetric => {
                "the predicate measures under a metric this evaluator does not define"
            }
            Self::CandidateNotFinite => "a NaN or infinite candidate has no exact value",
            Self::NamedProfileNotInterpretable => {
                "a named-elementary profile's result set is defined by its descriptor, which this evaluator does not hold"
            }
            Self::NoApplicableClause => {
                "no clause of the contract applies at the supplied input point"
            }
        })
    }
}

impl Error for UndecidedConformance {}

/// Whether a candidate satisfies an accuracy obligation at one reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConformanceDecision {
    /// The candidate provably satisfies the obligation.
    Conforms,
    /// The candidate provably violates the obligation.
    Violates,
    /// The evidence does not decide, and this says so rather than guessing.
    Undecided {
        /// Why, naming what would close it.
        reason: UndecidedConformance,
    },
}

impl ConformanceDecision {
    /// Returns whether the obligation was proved satisfied.
    ///
    /// `false` for both [`Self::Violates`] and [`Self::Undecided`], which is the
    /// fail-closed reading: an undecided comparison is not a pass.
    #[must_use]
    pub const fn conforms(&self) -> bool {
        matches!(self, Self::Conforms)
    }
}

/// Decides one accuracy predicate against a bracketed reference and a candidate.
///
/// `candidate` is the exact mathematical value `z` of the finite result-dtype
/// candidate, taken *before* the result-subnormal and signed-zero mapping — step
/// three of ADR 0042's composition, not step four.
///
/// # The three-way comparison
///
/// Both sides of `lhs <= tolerance` are bracketed. The decision is `Conforms`
/// when the bracket's upper end is within the tolerance, `Violates` when its
/// lower end is beyond it, and `Undecided` when the bracket straddles the
/// tolerance. Narrowing the enclosure moves an `Undecided` to one of the other
/// two; nothing else does.
#[must_use]
pub fn decide_predicate(
    predicate: &AccuracyPredicate,
    format: &UlpFormat,
    reference: &CertifiedEnclosure,
    candidate: &ExactRational,
) -> ConformanceDecision {
    match predicate.view() {
        AccuracyPredicateView::Absolute { tolerance } => {
            compare(&absolute_error(reference, candidate), tolerance.value())
        }
        AccuracyPredicateView::Relative { tolerance } => {
            let magnitude = reference.magnitude();
            if !magnitude.excludes_zero() {
                return undecided(UndecidedConformance::ReferenceNotProvablyNonzero);
            }
            let Ok(ratio) = absolute_error(reference, candidate).divide_by(&magnitude) else {
                return undecided(UndecidedConformance::ReferenceNotProvablyNonzero);
            };
            compare(&ratio, tolerance.value())
        }
        AccuracyPredicateView::AbsoluteRelative { absolute, relative } => {
            // `a + q * |r|` is itself bracketed, so the comparison is between two
            // enclosures rather than against a constant.
            let bound = CertifiedEnclosure::exact(absolute.value().clone()).add(
                &CertifiedEnclosure::exact(relative.value().clone())
                    .multiply(&reference.magnitude()),
            );
            compare_enclosures(&absolute_error(reference, candidate), &bound)
        }
        AccuracyPredicateView::Ulp { metric, tolerance } => {
            if !metric.is_ulp_reference_gap() {
                return undecided(UndecidedConformance::UnsupportedMetric);
            }
            // `ulp` is monotone nondecreasing in the magnitude, so bracketing the
            // magnitude brackets the scale.
            let magnitude = reference.magnitude();
            let (Ok(lower_scale), Ok(upper_scale)) = (
                format.ulp_scale(magnitude.lower()),
                format.ulp_scale(magnitude.upper()),
            ) else {
                return undecided(UndecidedConformance::MetricUndefined);
            };
            let scale = CertifiedEnclosure::new(lower_scale, upper_scale);
            let Ok(ratio) = absolute_error(reference, candidate).divide_by(&scale) else {
                return undecided(UndecidedConformance::EnclosureTooWide);
            };
            compare(&ratio, tolerance.value())
        }
        AccuracyPredicateView::Boolean { kind, members } => {
            let decisions: Vec<_> = members
                .iter()
                .map(|member| decide_predicate(member, format, reference, candidate))
                .collect();
            combine(kind, decisions)
        }
    }
}

/// Decides one complete accuracy contract at an exact input point.
///
/// Selects the applicable clauses by ADR 0042's intersection semantics — every
/// matching clause applies, so the obligation is their conjunction — and decides
/// the conjunction. The correctly rounded and faithful forms are decided directly
/// against the format's own rounding rather than through a ULP bound, because
/// equating them with one is exactly what ADR 0042 forbids.
#[must_use]
pub fn decide_contract(
    contract: &AccuracyContract,
    format: &UlpFormat,
    inputs: &[ExactRational],
    reference: &CertifiedEnclosure,
    candidate: &ExactRational,
) -> ConformanceDecision {
    match contract.form() {
        AccuracyContractForm::CorrectlyRounded { .. } => {
            let (Ok(lower), Ok(upper)) = (
                format.round_to_nearest_ties_even(reference.lower()),
                format.round_to_nearest_ties_even(reference.upper()),
            ) else {
                return undecided(UndecidedConformance::MetricUndefined);
            };
            if lower != upper {
                // The bracket spans a rounding boundary, so the correctly rounded
                // result is not determined by this enclosure.
                return undecided(UndecidedConformance::EnclosureTooWide);
            }
            if *candidate == lower {
                ConformanceDecision::Conforms
            } else {
                ConformanceDecision::Violates
            }
        }
        AccuracyContractForm::Faithful => {
            let (Ok(lower), Ok(upper)) = (
                format.bracketing(reference.lower()),
                format.bracketing(reference.upper()),
            ) else {
                return undecided(UndecidedConformance::MetricUndefined);
            };
            if lower != upper {
                return undecided(UndecidedConformance::EnclosureTooWide);
            }
            if *candidate == lower.0 || *candidate == lower.1 {
                ConformanceDecision::Conforms
            } else {
                ConformanceDecision::Violates
            }
        }
        AccuracyContractForm::NamedElementary { .. } => {
            undecided(UndecidedConformance::NamedProfileNotInterpretable)
        }
        AccuracyContractForm::BoundedPiecewise(domain) => {
            let applicable: Vec<_> = domain
                .clauses()
                .iter()
                .filter(|clause| clause.applies_at(inputs))
                .collect();
            if applicable.is_empty() {
                return undecided(UndecidedConformance::NoApplicableClause);
            }
            combine(
                BooleanPredicateKind::AllOf,
                applicable
                    .into_iter()
                    .map(|clause| {
                        decide_predicate(clause.predicate(), format, reference, candidate)
                    })
                    .collect(),
            )
        }
    }
}

fn combine(kind: BooleanPredicateKind, decisions: Vec<ConformanceDecision>) -> ConformanceDecision {
    match kind {
        BooleanPredicateKind::AllOf => {
            if let Some(violation) = decisions
                .iter()
                .find(|decision| **decision == ConformanceDecision::Violates)
            {
                return violation.clone();
            }
            decisions
                .into_iter()
                .find(|decision| !decision.conforms())
                .unwrap_or(ConformanceDecision::Conforms)
        }
        BooleanPredicateKind::AnyOf => {
            if decisions.iter().any(ConformanceDecision::conforms) {
                return ConformanceDecision::Conforms;
            }
            decisions
                .into_iter()
                .find(|decision| *decision != ConformanceDecision::Violates)
                .unwrap_or(ConformanceDecision::Violates)
        }
    }
}

const fn undecided(reason: UndecidedConformance) -> ConformanceDecision {
    ConformanceDecision::Undecided { reason }
}

/// Brackets `|z - r|` from a bracketed `r` and an exact `z`.
fn absolute_error(reference: &CertifiedEnclosure, candidate: &ExactRational) -> CertifiedEnclosure {
    CertifiedEnclosure::new(
        candidate.subtract(reference.upper()),
        candidate.subtract(reference.lower()),
    )
    .magnitude()
}

fn compare(value: &CertifiedEnclosure, tolerance: &ExactRational) -> ConformanceDecision {
    if value.upper() <= tolerance {
        ConformanceDecision::Conforms
    } else if value.lower() > tolerance {
        ConformanceDecision::Violates
    } else {
        undecided(UndecidedConformance::EnclosureTooWide)
    }
}

fn compare_enclosures(
    value: &CertifiedEnclosure,
    bound: &CertifiedEnclosure,
) -> ConformanceDecision {
    if value.upper() <= bound.lower() {
        ConformanceDecision::Conforms
    } else if value.lower() > bound.upper() {
        ConformanceDecision::Violates
    } else {
        undecided(UndecidedConformance::EnclosureTooWide)
    }
}

impl CertifiedEnclosure {
    /// Returns the enclosure of the sum.
    #[must_use]
    fn add(&self, other: &Self) -> Self {
        Self {
            lower: self.lower.add(&other.lower),
            upper: self.upper.add(&other.upper),
        }
    }

    /// Returns the enclosure of the quotient by a divisor that excludes zero.
    fn divide_by(&self, divisor: &Self) -> Result<Self, EnclosureError> {
        Ok(self.multiply(&divisor.reciprocal()?))
    }
}

/// Returns the exact value of one finite binary32 candidate.
///
/// The boundary between a host value and the exact arithmetic every decision
/// above performs. `None` for a NaN or an infinity, which have no exact value and
/// are the exceptional-value contract's subject rather than the accuracy
/// contract's.
#[must_use]
pub fn exact_binary32_candidate(candidate: f32) -> Option<ExactRational> {
    ExactRational::from_f32(candidate)
}

#[cfg(test)]
mod tests;
