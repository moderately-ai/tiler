//! The versioned ULP metric key, and the dtype capability it requires.
//!
//! ADR 0042 fixes `tiler::ulp-reference-gap@1` as the initial ULP metric and
//! spells out its rules, because "even the term ULP has competing definitions"
//! and a bound quoted under one definition is not a bound under another. The key
//! is versioned for exactly that reason: a second definition is a second key, not
//! a reinterpretation of stored programs.
//!
//! # What the metric says
//!
//! For a real reference `r` and a candidate `z`, `Ulp(metric, t)` asserts
//! `|z - r| / ulp(r) <= t`, where `ulp(r)` is:
//!
//! - the gap `b - a` when `r` lies strictly between numerically distinct
//!   consecutive finite values `a < b`;
//! - **the smaller of the predecessor and successor gaps** when `r` is itself
//!   representable and the two differ, so binary `ulp(2^e)` uses the predecessor
//!   gap while the scale increases immediately above that value;
//! - the smallest positive finite representable value at `r = 0` — the minimum
//!   positive subnormal for a gradual-underflow format, the minimum positive
//!   normal for a format without subnormals.
//!
//! It is **defined only** when `r` and `z` are finite and `r` lies within the
//! result format's finite numerical range. `tiler::ulp-reference-gap@1`
//! deliberately does *not* inherit OpenCL's additional hypothetical-successor
//! overflow allowance: a reference above the largest finite value leaves the
//! metric's domain and is refused by [`UlpMetricError::ReferenceOutOfFiniteRange`]
//! rather than measured against a value the format does not have.
//!
//! # Why compatibility is a check and not an assumption
//!
//! The metric needs "an ordered set of numerically distinct finite values and
//! adjacent-value behavior", and ADR 0042 requires a dtype/metric pair lacking
//! that capability to be **rejected rather than guessed**. A predicate dtype, an
//! integer, a decimal cohort format, a complex pair, and a microscaling block
//! scheme each fail that requirement for a different reason, and none of them
//! would fail loudly if the metric simply assumed a binary interchange layout.
//!
//! The rule table this module holds is therefore an explicit list of the classes
//! whose adjacent-value behaviour a *stated normative rule* fixes completely from
//! the descriptor's own fields, and [`ulp_metric_format_rules`] exposes it. A class
//! absent from it is refused by name. The table is what makes this check able to
//! say no: it names its population, and a new dtype class is unsupported until a
//! row is added with its basis.

use std::error::Error;
use std::fmt;

use crate::identity::push_slice;
use crate::semantic::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueView, SCALAR_TYPE_FACT_CLASS,
    SCALAR_TYPE_FACT_EXPONENT_BITS, SCALAR_TYPE_FACT_HAS_SUBNORMALS, SCALAR_TYPE_FACT_SIGN_BITS,
    SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS, TypeIdentityError, TypeKey,
};

use super::error::{AccuracyAttributeSubject, AccuracyContractError, malformed};
use super::rational::ExactRational;

/// A canonical accuracy-metric identity, distinct from every other key kind.
///
/// A newtype rather than a reused [`TypeKey`] for the reason ADR 0042's
/// refinement rule makes a correctness requirement: "a distinct metric key is not
/// a name to match on". A metric key names a *definition of distance*, and using
/// the type that names a dtype would let one be passed where the other belongs.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AccuracyMetricKey(TypeKey);

impl AccuracyMetricKey {
    /// Creates a validated, versioned accuracy-metric key.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] for an invalid component or version.
    pub fn new(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        semantic_version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::new(namespace, name, semantic_version).map(Self)
    }

    /// Returns the canonical namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        self.0.namespace()
    }

    /// Returns the name within the namespace.
    #[must_use]
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// Returns the nonzero semantic version.
    #[must_use]
    pub const fn semantic_version(&self) -> u32 {
        self.0.semantic_version()
    }

    /// Returns whether this key is the metric this module defines.
    #[must_use]
    pub fn is_ulp_reference_gap(&self) -> bool {
        self.namespace() == "tiler"
            && self.name() == "ulp-reference-gap"
            && self.semantic_version() == 1
    }

    pub(crate) fn encode(&self, output: &mut Vec<u8>) {
        push_slice(output, self.namespace().as_bytes());
        push_slice(output, self.name().as_bytes());
        output.extend_from_slice(&self.semantic_version().to_be_bytes());
    }

    /// Returns the canonical attribute value carrying this key.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::CanonicalBound`] when the key exceeds a
    /// canonical bound.
    pub fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        Ok(CanonicalValue::record([
            CanonicalField::new(
                METRIC_KEY_NAMESPACE,
                CanonicalValue::utf8(self.namespace())?,
            ),
            CanonicalField::new(METRIC_KEY_NAME, CanonicalValue::utf8(self.name())?),
            CanonicalField::new(
                METRIC_KEY_VERSION,
                CanonicalValue::unsigned_u32(self.semantic_version()),
            ),
        ])?)
    }

    /// Decodes one metric key exactly as an attribute carries it.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] for a malformed record or an invalid
    /// identity component.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        let malformed = || malformed(AccuracyAttributeSubject::MetricKey);
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(malformed());
        };
        let [namespace, name, version] = fields else {
            return Err(malformed());
        };
        if namespace.id() != METRIC_KEY_NAMESPACE
            || name.id() != METRIC_KEY_NAME
            || version.id() != METRIC_KEY_VERSION
        {
            return Err(malformed());
        }
        let (
            CanonicalValueView::Utf8(namespace),
            CanonicalValueView::Utf8(name),
            CanonicalValueView::Unsigned { bits, .. },
        ) = (
            namespace.value().view(),
            name.value().view(),
            version.value().view(),
        )
        else {
            return Err(malformed());
        };
        Ok(Self::new(
            namespace,
            name,
            u32::try_from(bits).map_err(|_| malformed())?,
        )?)
    }
}

/// Metric-key record field carrying the authority namespace.
pub(super) const METRIC_KEY_NAMESPACE: AttributeFieldId = AttributeFieldId::new(1);
/// Metric-key record field carrying the name within that namespace.
pub(super) const METRIC_KEY_NAME: AttributeFieldId = AttributeFieldId::new(2);
/// Metric-key record field carrying the nonzero semantic version.
pub(super) const METRIC_KEY_VERSION: AttributeFieldId = AttributeFieldId::new(3);

impl fmt::Display for AccuracyMetricKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Returns the initial governed ULP metric key, `tiler::ulp-reference-gap@1`.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn ulp_reference_gap_metric_key() -> AccuracyMetricKey {
    AccuracyMetricKey::new("tiler", "ulp-reference-gap", 1)
        .expect("the governed ULP metric key is valid")
}

/// One descriptor class whose adjacent-value behaviour a stated rule fixes.
///
/// The `basis` is the whole of the claim. A row asserts that the named authority
/// determines the complete finite value set from the descriptor fields this
/// module reads, and nothing else. Adding a row is a positive claim about a
/// format family, not a convenience.
struct UlpFormatRule {
    /// The `SCALAR_TYPE_FACT_CLASS` value this rule interprets.
    class: &'static str,
    /// The authority that fixes the parameterization, stated exactly.
    basis: &'static str,
}

/// Every descriptor class `tiler::ulp-reference-gap@1` can interpret.
///
/// Both rows share one parameterization — `p = t + 1`, `emax = 2^(w-1) - 1`,
/// `emin = 1 - emax` — because both authorities define it that way, and they are
/// separate rows rather than one because they are separate authorities: the
/// bfloat row's basis is the RISC-V operand format, not IEEE 754, and a change to
/// either must be traceable to its own document.
///
/// **Absent deliberately.** `ieee-decimal` carries a coefficient digit count and
/// no exponent range, and its cohorts mean several encodings share one value, so
/// this descriptor does not fix adjacent-value behaviour. `ocp-binary-element`
/// rows are finite-only or NaN-only value sets whose top-of-range behaviour
/// differs per format and is not derivable from the four fields read here.
/// `ocp-exponent-scale` has no zero, no sign, and no significand. Integer,
/// predicate, complex, and microscaling-block classes are not floating-point
/// value sets at all. Each is refused by [`UlpFormatError::UnrecognizedClass`]
/// rather than approximated.
const ULP_FORMAT_RULES: &[UlpFormatRule] = &[
    UlpFormatRule {
        class: "ieee-binary",
        basis: "IEEE 754-2019 binary interchange format: precision is the trailing significand width plus the implicit leading bit, emax is 2^(exponent bits - 1) - 1, and emin is 1 - emax",
    },
    UlpFormatRule {
        class: "bfloat",
        basis: "RISC-V Unprivileged ISA BF16 operand format, parameterized identically to an IEEE 754 binary interchange format: precision is the trailing significand width plus the implicit leading bit, emax is 2^(exponent bits - 1) - 1, and emin is 1 - emax",
    },
];

/// Returns every descriptor class `tiler::ulp-reference-gap@1` interprets, with its basis.
///
/// The population, named and countable. A check over dtypes that reports a
/// uniform answer without knowing how many classes exist is the signature this
/// repository distrusts; this is what lets a caller — or a test — assert that the
/// set is exactly the one that was intended rather than whatever the table
/// happens to hold.
#[must_use]
pub fn ulp_metric_format_rules() -> impl ExactSizeIterator<Item = (&'static str, &'static str)> {
    ULP_FORMAT_RULES.iter().map(|rule| (rule.class, rule.basis))
}

/// Why one dtype descriptor cannot carry `tiler::ulp-reference-gap@1`.
///
/// Every variant is a refusal that ADR 0042 requires in place of a guess. A
/// descriptor that reaches none of them has an ordered set of numerically
/// distinct finite values with derivable adjacent-value behaviour, which is
/// exactly the metric's stated compatibility requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UlpFormatError {
    /// The definition's facts were not a canonical record.
    MalformedDescriptor,
    /// A required descriptor field was absent.
    ///
    /// Absent is not a default: ADR 0034's catalog states every field a format
    /// has, so an absent one means Tiler's evidence does not fix it.
    MissingFact {
        /// The name of the absent field.
        field: &'static str,
    },
    /// A required descriptor field carried the wrong canonical kind.
    MalformedFact {
        /// The name of the malformed field.
        field: &'static str,
    },
    /// No registered rule interprets the descriptor's class.
    UnrecognizedClass {
        /// The class the descriptor declared.
        class: String,
    },
    /// The format has no sign field, so it is not a signed arithmetic value set.
    UnsignedFormat {
        /// The declared sign-field width.
        sign_bits: u32,
    },
    /// The declared exponent or significand widths do not describe a usable value set.
    DegenerateParameters {
        /// Declared exponent width.
        exponent_bits: u32,
        /// Declared trailing significand width.
        trailing_significand_bits: u32,
    },
}

impl UlpFormatError {
    /// Returns the stable provider diagnostic code naming this refusal.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::MalformedDescriptor => "accuracy.metric.malformed-descriptor",
            Self::MissingFact { .. } => "accuracy.metric.missing-descriptor-fact",
            Self::MalformedFact { .. } => "accuracy.metric.malformed-descriptor-fact",
            Self::UnrecognizedClass { .. } => "accuracy.metric.incompatible-dtype",
            Self::UnsignedFormat { .. } => "accuracy.metric.unsigned-format",
            Self::DegenerateParameters { .. } => "accuracy.metric.degenerate-format",
        }
    }
}

impl fmt::Display for UlpFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedDescriptor => {
                formatter.write_str("the value-type definition's facts are not a canonical record")
            }
            Self::MissingFact { field } => write!(
                formatter,
                "the descriptor does not state {field}, so its adjacent-value behaviour is not fixed"
            ),
            Self::MalformedFact { field } => {
                write!(
                    formatter,
                    "the descriptor's {field} has the wrong canonical kind"
                )
            }
            Self::UnrecognizedClass { class } => write!(
                formatter,
                "no registered rule interprets descriptor class {class:?}, so tiler::ulp-reference-gap@1 is rejected for this dtype rather than guessed"
            ),
            Self::UnsignedFormat { sign_bits } => write!(
                formatter,
                "the format declares {sign_bits} sign bits, so it is not a signed arithmetic value set"
            ),
            Self::DegenerateParameters {
                exponent_bits,
                trailing_significand_bits,
            } => write!(
                formatter,
                "an exponent width of {exponent_bits} and a trailing significand width of {trailing_significand_bits} do not describe a usable value set"
            ),
        }
    }
}

impl Error for UlpFormatError {}

/// Why the metric is undefined for one reference value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UlpMetricError {
    /// The reference lies outside the result format's finite numerical range.
    ///
    /// This is where `tiler::ulp-reference-gap@1` parts company with `OpenCL`,
    /// deliberately. `OpenCL` admits an additional allowance measured against a
    /// *hypothetical* successor of the largest finite value; this metric does not
    /// inherit it, so a reference above that value has no scale rather than a
    /// generously imagined one.
    ReferenceOutOfFiniteRange {
        /// The largest finite magnitude the format represents.
        largest_finite: ExactRational,
    },
}

impl UlpMetricError {
    /// Returns the stable provider diagnostic code naming this refusal.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::ReferenceOutOfFiniteRange { .. } => {
                "accuracy.metric.reference-out-of-finite-range"
            }
        }
    }
}

impl fmt::Display for UlpMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReferenceOutOfFiniteRange { largest_finite } => write!(
                formatter,
                "the reference lies outside the finite range bounded by {largest_finite}, where tiler::ulp-reference-gap@1 is undefined"
            ),
        }
    }
}

impl Error for UlpMetricError {}

/// The finite value set of one metric-compatible result dtype.
///
/// Derived from a registered value-type definition's own descriptor facts, never
/// from a hard-coded table of dtypes: the catalog is the authority on what a
/// format is, and a second copy here would be a second place for it to be wrong.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UlpFormat {
    class: &'static str,
    basis: &'static str,
    precision: u32,
    min_exponent: i32,
    max_exponent: i32,
    has_subnormals: bool,
}

impl UlpFormat {
    /// Derives the finite value set from one value-type definition's facts.
    ///
    /// # Errors
    ///
    /// Returns [`UlpFormatError`] when the descriptor does not expose an ordered
    /// set of numerically distinct finite values with derivable adjacent-value
    /// behaviour. That refusal is ADR 0042's "rejected rather than guessed".
    pub fn from_value_type_facts(facts: &CanonicalValue) -> Result<Self, UlpFormatError> {
        let CanonicalValueView::Record(fields) = facts.view() else {
            return Err(UlpFormatError::MalformedDescriptor);
        };
        let find = |id| {
            fields
                .iter()
                .find(|field| field.id() == id)
                .map(CanonicalField::value)
        };

        let class_value = find(SCALAR_TYPE_FACT_CLASS).ok_or(UlpFormatError::MissingFact {
            field: "the descriptor class",
        })?;
        let CanonicalValueView::Utf8(class) = class_value.view() else {
            return Err(UlpFormatError::MalformedFact {
                field: "the descriptor class",
            });
        };
        let rule = ULP_FORMAT_RULES
            .iter()
            .find(|rule| rule.class == class)
            .ok_or_else(|| UlpFormatError::UnrecognizedClass {
                class: class.to_owned(),
            })?;

        let sign_bits = unsigned_fact(find(SCALAR_TYPE_FACT_SIGN_BITS), "the sign-field width")?;
        if sign_bits != 1 {
            return Err(UlpFormatError::UnsignedFormat { sign_bits });
        }
        let exponent_bits =
            unsigned_fact(find(SCALAR_TYPE_FACT_EXPONENT_BITS), "the exponent width")?;
        let trailing_significand_bits = unsigned_fact(
            find(SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS),
            "the trailing significand width",
        )?;
        let has_subnormals_value =
            find(SCALAR_TYPE_FACT_HAS_SUBNORMALS).ok_or(UlpFormatError::MissingFact {
                field: "the subnormal-presence flag",
            })?;
        let CanonicalValueView::Bool(has_subnormals) = has_subnormals_value.view() else {
            return Err(UlpFormatError::MalformedFact {
                field: "the subnormal-presence flag",
            });
        };

        // Two exponent bits are the minimum that leave a normal range once the
        // all-zero and all-ones codes are reserved, and the bound above keeps
        // `2^(w - 1) - 1` inside `i32`.
        if !(2..=30).contains(&exponent_bits) || trailing_significand_bits == 0 {
            return Err(UlpFormatError::DegenerateParameters {
                exponent_bits,
                trailing_significand_bits,
            });
        }

        let max_exponent = (1_i32 << (exponent_bits - 1)) - 1;
        Ok(Self {
            class: rule.class,
            basis: rule.basis,
            precision: trailing_significand_bits + 1,
            min_exponent: 1 - max_exponent,
            max_exponent,
            has_subnormals,
        })
    }

    /// Returns the descriptor class this format was derived from.
    #[must_use]
    pub const fn class(&self) -> &'static str {
        self.class
    }

    /// Returns the authority whose rule fixed this format's parameterization.
    ///
    /// Carried on the derived format rather than left in the table, so a
    /// consumer that acts on a ULP scale can state *why* that scale is what it
    /// is without re-deriving it.
    #[must_use]
    pub const fn normative_basis(&self) -> &'static str {
        self.basis
    }

    /// Returns the significand precision, including any implicit leading bit.
    #[must_use]
    pub const fn precision(&self) -> u32 {
        self.precision
    }

    /// Returns the least exponent of a normal value.
    #[must_use]
    pub const fn min_exponent(&self) -> i32 {
        self.min_exponent
    }

    /// Returns the greatest exponent of a finite value.
    #[must_use]
    pub const fn max_exponent(&self) -> i32 {
        self.max_exponent
    }

    /// Returns whether the format has subnormal values.
    #[must_use]
    pub const fn has_subnormals(&self) -> bool {
        self.has_subnormals
    }

    /// Returns the smallest positive finite representable value.
    ///
    /// The minimum positive subnormal for a gradual-underflow format and the
    /// minimum positive normal otherwise, exactly as ADR 0042's zero rule states.
    #[must_use]
    pub fn smallest_positive_finite(&self) -> ExactRational {
        ExactRational::power_of_two(if self.has_subnormals {
            self.subnormal_gap_exponent()
        } else {
            self.min_exponent
        })
    }

    /// Returns the largest finite representable magnitude.
    ///
    /// # Panics
    ///
    /// Panics if the derived precision leaves `i32`, which
    /// [`Self::from_value_type_facts`] makes unreachable by bounding the exponent
    /// and significand widths before this value exists.
    #[must_use]
    pub fn largest_finite(&self) -> ExactRational {
        // (2^p - 1) * 2^(emax - p + 1), written exactly rather than as
        // `2^emax * (2 - 2^(1-p))`, which would need a subtraction of an inexact
        // intermediate in any representation that was not exact.
        let significand = ExactRational::power_of_two(
            i32::try_from(self.precision).expect("a bounded precision fits i32"),
        )
        .subtract(&ExactRational::one());
        significand.scale_by_power_of_two(
            self.max_exponent
                - i32::try_from(self.precision).expect("a bounded precision fits i32")
                + 1,
        )
    }

    /// Returns the exponent of the gap between adjacent subnormal values.
    const fn subnormal_gap_exponent(&self) -> i32 {
        // Exact rather than wrapping: `from_value_type_facts` bounds the
        // precision far below `i32::MAX`, and `cast_signed` is the const-usable
        // spelling of that fact.
        self.min_exponent - (self.precision.cast_signed() - 1)
    }

    /// Returns the spacing of the binade containing `value`.
    ///
    /// **Not** [`Self::ulp_scale`], and the difference is exactly ADR 0042's
    /// representable-case rule. At a power of two the metric selects the *smaller*
    /// of the two gaps, which is the predecessor's; rounding needs the spacing of
    /// the interval the value actually lies in, which is the successor's. Two
    /// questions, two functions — a single one would have to answer one of them
    /// wrongly at every power of two.
    fn binade_spacing(&self, magnitude: &ExactRational) -> ExactRational {
        let precision = i32::try_from(self.precision).expect("a bounded precision fits i32");
        let least_normal = ExactRational::power_of_two(self.min_exponent);
        if *magnitude < least_normal {
            return ExactRational::power_of_two(if self.has_subnormals {
                self.subnormal_gap_exponent()
            } else {
                self.min_exponent
            });
        }
        let binade = magnitude
            .floor_log2_abs()
            .expect("a magnitude at or above the least normal is nonzero");
        ExactRational::power_of_two(binade - precision + 1)
    }

    /// Returns the two representable values bracketing an exact reference.
    ///
    /// `(below, above)` with `below <= value <= above`, equal when the value is
    /// itself representable. This is the faithful-rounding result set, written
    /// once so that a faithful contract and a one-ULP bound cannot be checked by
    /// the same code path and thereby become the same obligation.
    ///
    /// # Errors
    ///
    /// Returns [`UlpMetricError::ReferenceOutOfFiniteRange`] when a bracket would
    /// leave the format's finite range.
    ///
    /// # Panics
    ///
    /// Panics under the same unreachable precision condition as
    /// [`Self::largest_finite`].
    pub fn bracketing(
        &self,
        value: &ExactRational,
    ) -> Result<(ExactRational, ExactRational), UlpMetricError> {
        let magnitude = value.abs();
        let largest_finite = self.largest_finite();
        if magnitude > largest_finite {
            return Err(UlpMetricError::ReferenceOutOfFiniteRange { largest_finite });
        }
        let spacing = self.binade_spacing(&magnitude);
        let quotient = magnitude
            .divide(&spacing)
            .unwrap_or_else(|_| unreachable!("a spacing is a positive power of two"));
        let below = quotient.floor_to_binary_grid(0).multiply(&spacing);
        let above = quotient.ceil_to_binary_grid(0).multiply(&spacing);
        if above > largest_finite {
            return Err(UlpMetricError::ReferenceOutOfFiniteRange { largest_finite });
        }
        Ok(if value.is_negative() {
            (above.negate(), below.negate())
        } else {
            (below, above)
        })
    }

    /// Rounds an exact value to the nearest representable value, ties to even.
    ///
    /// Returns the *exact rational value* of the rounded result rather than a host
    /// float, because the whole point of this vocabulary is that the comparison
    /// happens in exact arithmetic. ADR 0024 fixes this direction as the initial
    /// arithmetic rounding.
    ///
    /// # Errors
    ///
    /// Returns [`UlpMetricError::ReferenceOutOfFiniteRange`] when the rounded
    /// result would leave the format's finite range, which is the finite-overflow
    /// contract's subject rather than the accuracy contract's.
    ///
    /// # Panics
    ///
    /// Panics under the same unreachable precision condition as
    /// [`Self::largest_finite`].
    pub fn round_to_nearest_ties_even(
        &self,
        value: &ExactRational,
    ) -> Result<ExactRational, UlpMetricError> {
        let magnitude = value.abs();
        let largest_finite = self.largest_finite();
        let spacing = self.binade_spacing(&magnitude);
        let quotient = magnitude
            .divide(&spacing)
            .unwrap_or_else(|_| unreachable!("a spacing is a positive power of two"));
        let below = quotient.floor_to_binary_grid(0);
        let fraction = quotient.subtract(&below);
        let half = ExactRational::power_of_two(-1);
        let rounded = match fraction.cmp(&half) {
            std::cmp::Ordering::Less => below,
            std::cmp::Ordering::Greater => below.add(&ExactRational::one()),
            // The tie: keep the even multiple of the spacing, which is what
            // "ties to even" means once the value is expressed in units of that
            // spacing.
            std::cmp::Ordering::Equal => {
                if below.scale_by_power_of_two(-1).floor_to_binary_grid(0)
                    == below.scale_by_power_of_two(-1)
                {
                    below
                } else {
                    below.add(&ExactRational::one())
                }
            }
        };
        let result = rounded.multiply(&spacing);
        if result > largest_finite {
            return Err(UlpMetricError::ReferenceOutOfFiniteRange { largest_finite });
        }
        Ok(if value.is_negative() {
            result.negate()
        } else {
            result
        })
    }

    /// Returns `ulp(reference)` under `tiler::ulp-reference-gap@1`.
    ///
    /// # Errors
    ///
    /// Returns [`UlpMetricError::ReferenceOutOfFiniteRange`] when the reference
    /// leaves the format's finite range, where the metric is undefined.
    ///
    /// # Panics
    ///
    /// Panics under the same unreachable precision condition as
    /// [`Self::largest_finite`].
    pub fn ulp_scale(&self, reference: &ExactRational) -> Result<ExactRational, UlpMetricError> {
        if reference.is_zero() {
            return Ok(self.smallest_positive_finite());
        }
        let magnitude = reference.abs();
        let largest_finite = self.largest_finite();
        if magnitude > largest_finite {
            return Err(UlpMetricError::ReferenceOutOfFiniteRange { largest_finite });
        }
        let precision = i32::try_from(self.precision).expect("a bounded precision fits i32");
        if magnitude < ExactRational::power_of_two(self.min_exponent) {
            // Below the least normal there is one gap: the subnormal spacing for
            // a gradual-underflow format, and the whole distance from zero to the
            // least normal for a format without subnormals.
            return Ok(ExactRational::power_of_two(if self.has_subnormals {
                self.subnormal_gap_exponent()
            } else {
                self.min_exponent
            }));
        }
        let binade = magnitude
            .floor_log2_abs()
            .expect("a nonzero magnitude has a binade");
        if magnitude.is_power_of_two_abs() && binade > self.min_exponent {
            // The representable case where the two gaps differ. ADR 0042 selects
            // the smaller, which is the predecessor's: the scale increases
            // immediately *above* a power of two, not at it.
            return Ok(ExactRational::power_of_two(binade - precision));
        }
        Ok(ExactRational::power_of_two(binade - precision + 1))
    }
}

fn unsigned_fact(
    value: Option<&CanonicalValue>,
    field: &'static str,
) -> Result<u32, UlpFormatError> {
    let value = value.ok_or(UlpFormatError::MissingFact { field })?;
    let CanonicalValueView::Unsigned { bits, .. } = value.view() else {
        return Err(UlpFormatError::MalformedFact { field });
    };
    u32::try_from(bits).map_err(|_| UlpFormatError::MalformedFact { field })
}
