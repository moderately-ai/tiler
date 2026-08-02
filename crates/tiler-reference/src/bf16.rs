//! Reference semantics for the governed pure-BF16 constant, multiply, and add.
//!
//! # Why the oracle is exact rational rather than host arithmetic
//!
//! Every operand is decoded to its exact mathematical value, the arithmetic is
//! performed with no precision bound and no intermediate format, and the result
//! is rounded **once**, at the observable materialization, by
//! round-to-nearest-ties-to-even over BF16's value set. That is what
//! `tiler::multiply-bf16@1` and `tiler::add-bf16@1` declare, and it is the whole
//! reason this module exists rather than a `f32` composition of the same shape.
//!
//! The tempting shortcut is real and is rejected deliberately. BF16 widens to
//! binary32 exactly, so `bf16 -> f32 -> arithmetic -> round to bf16` computes the
//! *right answer* for a single multiply or add: the classical bound for an
//! innocuous double rounding of one operation is `q >= 2p + 2`, which here is
//! `24 >= 18` and holds. Finding 24 of the retained Apple numerical record says
//! the same thing from the other side — no single operation can separate an `f32`
//! intermediate from native `bfloat` arithmetic. An oracle built on that bound
//! would therefore be correct *and* would stop being correct the moment this
//! family acquires an operation the bound does not cover: a fused multiply-add,
//! an accumulation, a contraction. An exact-rational oracle does not depend on the
//! bound at all, which is why it is the one that generalizes to this module's
//! successors.
//!
//! # Nothing here restates the format
//!
//! The encoded width, the precision, the exponent range and its bias, whether
//! subnormals are members, and the canonical arithmetic NaN payload are all read
//! from the registered `tiler::bf16@1` descriptor and from the family's own
//! declared fact record, by [`Bf16Format::governed`]. A catalog change is a typed
//! registration refusal here rather than a silent divergence, and there is no
//! second table for either to drift against.
//!
//! The rounding itself is [`UlpFormat::round_to_nearest_ties_even`], the exact
//! format-parameterized round-to-nearest this workspace already owns. Its
//! `bfloat` rule row cites the ratified RISC-V BF16 operand format, which is the
//! same authority the catalog row names, so the reference and the metric are
//! parameterized by one document rather than by two agreeing readings of it.
//!
//! # What is decided here rather than inherited
//!
//! Both zeros and their signs, subnormals preserved rather than flushed,
//! `inf * 0` and `inf - inf` as the canonical NaN, overflow at the midpoint above
//! the largest finite value, and every arithmetic NaN canonicalized to the one
//! declared payload. Each is a rule this module states and the witness corpus
//! checks, not a behaviour borrowed from a host float type — BF16 has no host
//! float type to borrow from, which is exactly why the rules had to be written
//! down.

use std::sync::Arc;

use tiler_ir::semantic::accuracy::{ExactRational, ExactSign, UlpFormat};
use tiler_ir::semantic::{
    AttributeFieldId, BF16_CONSTANT_BITS_ATTRIBUTE, BF16_FACT_CANONICAL_NAN_BITS, Bf16,
    CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueView,
    SCALAR_TYPE_FACT_EXPONENT_BIAS, SCALAR_TYPE_FACT_WIDTH_BITS, TypeKey, add_bf16_op,
    arithmetic_bf16_facts, builtin_scalar_value_type_facts, constant_bf16_op, multiply_bf16_op,
};

use super::error::{
    ReferenceOperationError, ReferenceRegistryError, ReferenceValueError,
    UnsupportedBf16Declaration, dense_result_error,
};
use super::registry::{
    ReferenceCapabilityRevision, ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs,
    ReferenceRegistryRegistrar, ReferenceSignature, ReferenceValueValidator,
};
use super::tensor::{FloatBitOrder, ReferenceElement, Tensor, TensorPayloadView};

/// The exact value one BF16 encoding denotes.
///
/// Deliberately not "an [`ExactRational`] plus a flag". An infinity is not a
/// number carrying an attribute, a NaN is not a number at all, and the sign of a
/// zero is information an unsigned rational zero cannot hold — every arithmetic
/// rule below dispatches on those distinctions, so collapsing them would mean
/// re-deriving them at each site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Bf16Value {
    /// A finite value, with the sign of a zero carried separately.
    Finite {
        /// The exact value. A zero here carries no sign; `zero_is_negative` does.
        value: ExactRational,
        /// Which zero this is, meaningful only when `value` is exactly zero.
        zero_is_negative: bool,
    },
    /// A signed infinity.
    Infinite {
        /// `true` for negative infinity.
        negative: bool,
    },
    /// Not a number. The payload is not modelled; the result is canonicalized.
    Nan,
}

/// The BF16 value set this reference computes over, read from its declarations.
///
/// Every field is derived at construction rather than at each use, so the decode,
/// the rounding, and the encode cannot disagree about what the format is.
#[derive(Clone, Debug)]
pub(crate) struct Bf16Format {
    payload_bytes: usize,
    exponent_bits: u32,
    trailing_bits: u32,
    /// The trailing significand width as a signed exponent offset.
    ///
    /// Every exponent computation below shifts by `precision - 1`, which is
    /// exactly this width; naming it once keeps that arithmetic in one type.
    significand_offset: i32,
    exponent_bias: i32,
    ulp: UlpFormat,
    format_key: TypeKey,
    /// The midpoint above the largest finite value, where overflow begins.
    overflow_threshold: ExactRational,
    canonical_nan_bits: u16,
}

impl Bf16Format {
    /// Reads the governed `tiler::bf16@1` declarations.
    ///
    /// # Errors
    ///
    /// Returns the declared term this reference does not realize.
    pub(crate) fn governed() -> Result<Self, UnsupportedBf16Declaration> {
        let resolved = Bf16::resolved_type();
        let format_key = resolved
            .nominal_key()
            .ok_or(UnsupportedBf16Declaration::MalformedFact {
                field: "the bf16 nominal identity",
            })?
            .clone();
        let facts = builtin_scalar_value_type_facts(&resolved)
            .ok_or(UnsupportedBf16Declaration::MissingDescriptor)?;
        Self::from_declarations(format_key, &facts, &arithmetic_bf16_facts())
    }

    /// Derives the value set from one descriptor and one arithmetic fact record.
    ///
    /// Separated from [`Self::governed`] for the reason the refusals exist: a
    /// refusal nothing can reach is not a check. The registered declarations are
    /// the only ones this crate ships, so a perturbed record is the only way to
    /// watch each rule below say no.
    ///
    /// # Errors
    ///
    /// Returns the declared term this reference does not realize.
    pub(crate) fn from_declarations(
        format_key: TypeKey,
        facts: &CanonicalValue,
        arithmetic_facts: &CanonicalValue,
    ) -> Result<Self, UnsupportedBf16Declaration> {
        // The metric's own format rule is the rounding this reference performs,
        // so it is also the check that the descriptor describes a value set to
        // round onto at all: class, sign field, and degenerate parameters are
        // refused there rather than re-derived here.
        let format = UlpFormat::from_value_type_facts(facts)
            .map_err(UnsupportedBf16Declaration::IncompatibleFormat)?;
        if !format.has_subnormals() {
            return Err(UnsupportedBf16Declaration::SubnormalsAbsent);
        }
        let width_bits = unsigned_fact(facts, SCALAR_TYPE_FACT_WIDTH_BITS, "the encoded width")?;
        // The element carrier below is a `u16`. A wider format needs a different
        // carrier rather than a wider mask, so it is refused by name here instead
        // of silently truncating an operand.
        if width_bits != u16::BITS {
            return Err(UnsupportedBf16Declaration::UnsupportedWidth { width_bits });
        }
        let trailing_bits = format.precision().saturating_sub(1);
        // `UlpFormat` has already refused a descriptor whose sign field is not one
        // bit, so the encoded width is the sign bit plus the exponent field plus
        // the trailing significand, and the exponent width is what remains.
        let exponent_bits = width_bits
            .checked_sub(format.precision())
            .ok_or(UnsupportedBf16Declaration::UnsupportedWidth { width_bits })?;
        let max_exponent = format.max_exponent();
        if !(2..=30).contains(&exponent_bits)
            || (1_i32 << (exponent_bits - 1)) - 1 != max_exponent
            || trailing_bits == 0
        {
            return Err(UnsupportedBf16Declaration::InconsistentExponentRange {
                exponent_bits,
                max_exponent,
            });
        }
        // The bias the exponent range implies. The catalog row states no override
        // for this format; one that did would describe a different encoding of the
        // same value set, and the decode below would misread every normal operand.
        let bias = max_exponent;
        if let Some(declared) = signed_fact(facts, SCALAR_TYPE_FACT_EXPONENT_BIAS)?
            && declared != bias
        {
            return Err(UnsupportedBf16Declaration::OverriddenExponentBias {
                declared,
                derived: bias,
            });
        }
        let payload_bytes = usize::try_from(width_bits / 8).unwrap_or(usize::MAX);
        let significand_offset = i32::try_from(trailing_bits).map_err(|_| {
            UnsupportedBf16Declaration::MalformedFact {
                field: "the trailing significand width",
            }
        })?;
        // The quantum of the top binade, halved: round-to-nearest-ties-to-even
        // sends every magnitude at or above this bound to infinity, which is the
        // rule IEEE 754 states applied to this format's own parameters.
        let overflow_threshold = format.largest_finite().add(&ExactRational::power_of_two(
            max_exponent - significand_offset - 1,
        ));
        let canonical_nan_bits =
            declared_arithmetic_nan(arithmetic_facts, &format_key, payload_bytes)?;
        let candidate = Self {
            payload_bytes,
            exponent_bits,
            trailing_bits,
            significand_offset,
            exponent_bias: bias,
            ulp: format,
            format_key,
            overflow_threshold,
            canonical_nan_bits,
        };
        // A declared payload that is not a NaN encoding would make every invalid
        // operation return a number, which is the one failure this family cannot
        // be allowed to have.
        if !matches!(candidate.decode(canonical_nan_bits), Bf16Value::Nan) {
            return Err(UnsupportedBf16Declaration::ArithmeticNanPayloadIsNotNan {
                bits: canonical_nan_bits,
            });
        }
        Ok(candidate)
    }

    /// Returns the exact payload width one BF16 element carries.
    pub(crate) const fn payload_bytes(&self) -> usize {
        self.payload_bytes
    }

    const fn exponent_mask(&self) -> u16 {
        (1 << self.exponent_bits) - 1
    }

    const fn trailing_mask(&self) -> u16 {
        (1 << self.trailing_bits) - 1
    }

    const fn infinity(&self, negative: bool) -> u16 {
        let magnitude = self.exponent_mask() << self.trailing_bits;
        if negative {
            sign_mask() | magnitude
        } else {
            magnitude
        }
    }

    /// Returns the exact value one encoding denotes.
    pub(crate) fn decode(&self, bits: u16) -> Bf16Value {
        let negative = bits & sign_mask() != 0;
        let biased = (bits >> self.trailing_bits) & self.exponent_mask();
        let trailing = bits & self.trailing_mask();
        if biased == self.exponent_mask() {
            return if trailing == 0 {
                Bf16Value::Infinite { negative }
            } else {
                Bf16Value::Nan
            };
        }
        if biased == 0 && trailing == 0 {
            return Bf16Value::Finite {
                value: ExactRational::zero(),
                zero_is_negative: negative,
            };
        }
        // A subnormal has no implicit leading bit and shares the least normal
        // exponent, which is what makes the spacing uniform down to zero; a normal
        // carries the implicit bit and biases its stored exponent.
        let (significand, exponent) = if biased == 0 {
            (
                i128::from(trailing),
                self.ulp.min_exponent() - self.significand_offset,
            )
        } else {
            (
                i128::from(trailing | (1 << self.trailing_bits)),
                i32::from(biased) - self.exponent_bias - self.significand_offset,
            )
        };
        let magnitude = ExactRational::from_integer(significand).scale_by_power_of_two(exponent);
        Bf16Value::Finite {
            value: if negative {
                magnitude.negate()
            } else {
                magnitude
            },
            zero_is_negative: false,
        }
    }

    /// Rounds one exact value into BF16 by round-to-nearest-ties-to-even.
    ///
    /// The single observable-materialization rule, applied exactly once per
    /// materialized result and never to an intermediate.
    pub(crate) fn round(&self, value: &Bf16Value) -> u16 {
        match value {
            Bf16Value::Nan => self.canonical_nan_bits,
            Bf16Value::Infinite { negative } => self.infinity(*negative),
            Bf16Value::Finite {
                value,
                zero_is_negative,
            } => {
                if value.is_zero() {
                    return if *zero_is_negative { sign_mask() } else { 0 };
                }
                let negative = value.is_negative();
                let magnitude = value.abs();
                if magnitude >= self.overflow_threshold {
                    return self.infinity(negative);
                }
                let sign = if negative { sign_mask() } else { 0 };
                let rounded = self
                    .ulp
                    .round_to_nearest_ties_even(&magnitude)
                    // Not a second overflow policy: `round_to_nearest_ties_even`
                    // refuses exactly when the rounded result would exceed the
                    // largest finite value, which under this rounding is exactly
                    // the magnitudes the threshold above already sent to infinity.
                    // The arm is therefore unreachable, and it answers with the
                    // same rule rather than inventing another.
                    .map_or_else(|_| None, |rounded| self.encode_magnitude(&rounded));
                match rounded {
                    Some(magnitude) => sign | magnitude,
                    None => self.infinity(negative),
                }
            }
        }
    }

    /// Returns the encoded exponent and significand of one representable magnitude.
    ///
    /// `None` for a value that is not a member of this format's finite value set,
    /// which is what makes the caller's overflow answer the only overflow answer.
    fn encode_magnitude(&self, magnitude: &ExactRational) -> Option<u16> {
        if magnitude.is_zero() {
            return Some(0);
        }
        if magnitude.is_negative() {
            return None;
        }
        let exponent = magnitude.floor_log2_abs()?.max(self.ulp.min_exponent());
        if exponent > self.ulp.max_exponent() {
            return None;
        }
        // A representable magnitude is a whole count of its binade's quantum, so
        // this scaling is an exact integer below `2^precision`.
        let quanta = exact_nonnegative_integer(
            &magnitude.scale_by_power_of_two(self.significand_offset.checked_sub(exponent)?),
        )?;
        let implicit = 1_u64 << self.trailing_bits;
        let (biased, significand) = if quanta < implicit {
            // Subnormal: the exponent field is zero and the quanta are the
            // trailing bits themselves.
            (0_u32, quanta)
        } else if quanta < implicit << 1 {
            (
                u32::try_from(exponent.checked_add(self.exponent_bias)?).ok()?,
                quanta - implicit,
            )
        } else {
            return None;
        };
        // The all-ones exponent field encodes the infinities and the NaNs, so a
        // finite value reaching it would be a different value class.
        if biased >= u32::from(self.exponent_mask()) {
            return None;
        }
        u16::try_from((biased << self.trailing_bits) | u32::try_from(significand).ok()?).ok()
    }
}

/// Returns the sign bit of the fixed sixteen-bit carrier every element uses.
///
/// A free function rather than a method because it is a fact about the carrier
/// [`Bf16Format::from_declarations`] already refused every other width for, not a
/// fact this instance holds.
const fn sign_mask() -> u16 {
    1 << (u16::BITS - 1)
}

/// Decodes one element's exact encoding.
fn decode_element(element: &ReferenceElement) -> Result<u16, ReferenceOperationError> {
    <[u8; 2]>::try_from(element.as_bytes())
        .map(u16::from_be_bytes)
        .map_err(|_| ReferenceOperationError::InvalidApplication)
}

/// Encodes one exact result into an element.
fn encode_element(bits: u16) -> Result<ReferenceElement, ReferenceOperationError> {
    ReferenceElement::from_float_bits(bits.to_be_bytes(), FloatBitOrder::MostSignificantByteFirst)
        .map_err(|_| ReferenceOperationError::InvalidApplication)
}

/// Returns the dense elements of one BF16 tensor.
///
/// The refusal is the one this family's keys require: an operand whose resolved
/// type is not `tiler::bf16@1` is rejected rather than reinterpreted, so a caller
/// reaching past the registry's signature dispatch cannot obtain a BF16 answer for
/// another dtype's bytes.
fn bf16_elements(tensor: &Tensor) -> Result<&[ReferenceElement], ReferenceOperationError> {
    if tensor.resolved_type() != &Bf16::resolved_type() {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    match tensor.payload() {
        TensorPayloadView::Dense(elements) => Ok(elements),
        TensorPayloadView::Compound(_) => Err(ReferenceOperationError::InvalidApplication),
    }
}

/// Returns the exact nonnegative integer this value denotes, when it is one.
fn exact_nonnegative_integer(value: &ExactRational) -> Option<u64> {
    let (sign, numerator, denominator) = value.to_sign_magnitude_ratio();
    if sign == ExactSign::Negative || denominator != [1] || numerator.len() > 8 {
        return None;
    }
    Some(
        numerator
            .iter()
            .fold(0_u64, |bits, byte| (bits << 8) | u64::from(*byte)),
    )
}

fn record_field(facts: &CanonicalValue, id: AttributeFieldId) -> Option<&CanonicalValue> {
    let CanonicalValueView::Record(fields) = facts.view() else {
        return None;
    };
    fields
        .iter()
        .find(|field| field.id() == id)
        .map(CanonicalField::value)
}

fn unsigned_fact(
    facts: &CanonicalValue,
    id: AttributeFieldId,
    field: &'static str,
) -> Result<u32, UnsupportedBf16Declaration> {
    let malformed = || UnsupportedBf16Declaration::MalformedFact { field };
    let value = record_field(facts, id).ok_or_else(malformed)?;
    let CanonicalValueView::Unsigned { bits, .. } = value.view() else {
        return Err(malformed());
    };
    u32::try_from(bits).map_err(|_| malformed())
}

/// Returns one optional signed descriptor fact.
///
/// Absent is a legitimate answer — this format's catalog row states no exponent
/// bias override — and is distinguished from a present field of the wrong kind,
/// which is malformed.
fn signed_fact(
    facts: &CanonicalValue,
    id: AttributeFieldId,
) -> Result<Option<i32>, UnsupportedBf16Declaration> {
    let malformed = || UnsupportedBf16Declaration::MalformedFact {
        field: "the declared exponent bias",
    };
    let Some(value) = record_field(facts, id) else {
        return Ok(None);
    };
    let CanonicalValueView::Signed { width, bits } = value.view() else {
        return Err(malformed());
    };
    if width != CanonicalIntegerWidth::Bits32 {
        return Err(malformed());
    }
    Ok(Some(
        u32::try_from(bits).map_err(|_| malformed())?.cast_signed(),
    ))
}

/// Returns the canonical arithmetic NaN payload the family's own facts declare.
///
/// Read from the declaration rather than restated, so this reference cannot hold
/// a second opinion about the payload `tiler::multiply-bf16@1` and
/// `tiler::add-bf16@1` promise. This crate's root owns the canonicalization
/// *rule* — every arithmetic NaN result carries one declared payload and never
/// the operand's — and this is the BF16 family's payload under it.
fn declared_arithmetic_nan(
    facts: &CanonicalValue,
    format_key: &TypeKey,
    payload_bytes: usize,
) -> Result<u16, UnsupportedBf16Declaration> {
    let field = "the declared canonical arithmetic NaN payload";
    let malformed = || UnsupportedBf16Declaration::MalformedFact { field };
    let value = record_field(facts, BF16_FACT_CANONICAL_NAN_BITS).ok_or_else(malformed)?;
    let CanonicalValueView::FloatBits(payload) = value.view() else {
        return Err(malformed());
    };
    if payload.format() != format_key || payload.bits().len() != payload_bytes {
        return Err(malformed());
    }
    <[u8; 2]>::try_from(payload.bits())
        .map(u16::from_be_bytes)
        .map_err(|_| malformed())
}

/// Adds two already-decoded exact values, exactly and without rounding.
pub(crate) fn exact_add(left: &Bf16Value, right: &Bf16Value) -> Bf16Value {
    match (left, right) {
        (Bf16Value::Nan, _) | (_, Bf16Value::Nan) => Bf16Value::Nan,
        (Bf16Value::Infinite { negative: left }, Bf16Value::Infinite { negative: right }) => {
            if left == right {
                Bf16Value::Infinite { negative: *left }
            } else {
                // Infinity minus infinity is invalid; its default result is a
                // quiet NaN, which this family then canonicalizes.
                Bf16Value::Nan
            }
        }
        (Bf16Value::Infinite { negative }, Bf16Value::Finite { .. })
        | (Bf16Value::Finite { .. }, Bf16Value::Infinite { negative }) => Bf16Value::Infinite {
            negative: *negative,
        },
        (
            Bf16Value::Finite {
                value: left,
                zero_is_negative: left_zero_negative,
            },
            Bf16Value::Finite {
                value: right,
                zero_is_negative: right_zero_negative,
            },
        ) => Bf16Value::Finite {
            value: left.add(right),
            // IEEE 754: under round-to-nearest the sum of two zeros is negative
            // only when both are, a nonzero exact sum carries its own sign, and an
            // exact-zero sum of opposite-signed nonzeros is positive zero.
            zero_is_negative: left.is_zero()
                && right.is_zero()
                && *left_zero_negative
                && *right_zero_negative,
        },
    }
}

/// Multiplies two already-decoded exact values, exactly and without rounding.
pub(crate) fn exact_multiply(left: &Bf16Value, right: &Bf16Value) -> Bf16Value {
    match (left, right) {
        (Bf16Value::Nan, _) | (_, Bf16Value::Nan) => Bf16Value::Nan,
        (Bf16Value::Infinite { negative: left }, Bf16Value::Infinite { negative: right }) => {
            Bf16Value::Infinite {
                negative: left != right,
            }
        }
        (Bf16Value::Infinite { negative }, Bf16Value::Finite { value, .. })
        | (Bf16Value::Finite { value, .. }, Bf16Value::Infinite { negative }) => {
            if value.is_zero() {
                // Infinity times zero is invalid; its default result is a quiet
                // NaN, which this family then canonicalizes.
                Bf16Value::Nan
            } else {
                Bf16Value::Infinite {
                    negative: *negative != value.is_negative(),
                }
            }
        }
        (
            Bf16Value::Finite {
                value: left,
                zero_is_negative: left_zero_negative,
            },
            Bf16Value::Finite {
                value: right,
                zero_is_negative: right_zero_negative,
            },
        ) => {
            // The product's zero sign is the exclusive-or of the operand signs,
            // which for a zero operand is carried by its zero sign rather than by
            // the unsigned rational zero.
            let left_negative = left.is_negative() || (left.is_zero() && *left_zero_negative);
            let right_negative = right.is_negative() || (right.is_zero() && *right_zero_negative);
            Bf16Value::Finite {
                value: left.multiply(right),
                zero_is_negative: left_negative != right_negative,
            }
        }
    }
}

/// Validates that a BF16 tensor carries the width its descriptor declares.
pub(crate) struct Bf16ValueValidator {
    payload_bytes: usize,
}

impl ReferenceValueValidator for Bf16ValueValidator {
    fn validate(&self, tensor: &Tensor) -> Result<(), ReferenceValueError> {
        if tensor.resolved_type() != &Bf16::resolved_type() {
            return Err(ReferenceValueError::InvalidRepresentation);
        }
        let TensorPayloadView::Dense(elements) = tensor.payload() else {
            return Err(ReferenceValueError::InvalidRepresentation);
        };
        if elements
            .iter()
            .any(|element| element.as_bytes().len() != self.payload_bytes)
        {
            return Err(ReferenceValueError::InvalidRepresentation);
        }
        Ok(())
    }
}

/// The exact BF16 constant, whose payload is preserved rather than rounded.
struct ConstantBf16Reference {
    format: Bf16Format,
}

impl ReferenceOperation for ConstantBf16Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let attributes = request.attributes();
        if !request.operands().is_empty() || attributes.fields().len() != 1 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let Some(CanonicalValueView::FloatBits(payload)) = attributes
            .get(BF16_CONSTANT_BITS_ATTRIBUTE)
            .map(CanonicalValue::view)
        else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        // Format first, then width, so a binary32 payload and a bf16-tagged
        // payload of another width fail for their own reasons.
        if payload.format() != &self.format.format_key
            || payload.bits().len() != self.format.payload_bytes()
        {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        // The declared payload is the value, exactly. `BF16_FACT_NAN_BEHAVIOUR`
        // states that a constant preserves the payload it was given, so a
        // non-canonical NaN constant is *not* canonicalized here; only an
        // arithmetic result is.
        let element = ReferenceElement::from_float_bits(
            payload.bits(),
            FloatBitOrder::MostSignificantByteFirst,
        )
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        outputs.push(
            Tensor::scalar(Bf16::resolved_type(), element)
                .map_err(|_| ReferenceOperationError::InvalidApplication)?,
        )
    }
}

/// Which exact arithmetic one binary BF16 capability performs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Bf16Arithmetic {
    /// Pure-BF16 multiplication, one rounding.
    Multiply,
    /// Pure-BF16 addition, one rounding.
    Add,
}

impl Bf16Arithmetic {
    /// Applies this arithmetic to two exact values, without rounding.
    pub(crate) fn apply(self, left: &Bf16Value, right: &Bf16Value) -> Bf16Value {
        match self {
            Self::Multiply => exact_multiply(left, right),
            Self::Add => exact_add(left, right),
        }
    }
}

/// The exact BF16 multiply and add, sharing one decode, arithmetic, and rounding.
pub(crate) struct Bf16BinaryReference {
    format: Bf16Format,
    arithmetic: Bf16Arithmetic,
}

impl Bf16BinaryReference {
    pub(crate) const fn new(format: Bf16Format, arithmetic: Bf16Arithmetic) -> Self {
        Self { format, arithmetic }
    }

    /// Evaluates the elementwise arithmetic with the scalar broadcast the
    /// operation's own inferencer admits.
    pub(crate) fn combine(
        &self,
        left: &Tensor,
        right: &Tensor,
    ) -> Result<Tensor, ReferenceOperationError> {
        let left_elements = bf16_elements(left)?;
        let right_elements = bf16_elements(right)?;
        // The same three cases `tiler::multiply-bf16@1`'s inferencer admits, and
        // the same refusal for a mismatched pair: the graph carries no implicit
        // broadcasting beyond a rank-zero operand.
        let result_shape = if left.shape().rank() == 0 {
            right.shape()
        } else if right.shape().rank() == 0 || left.shape() == right.shape() {
            left.shape()
        } else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let count = result_shape
            .element_count()
            .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        let elements = (0..count)
            .map(|index| {
                let left_bits = decode_element(
                    element_at(left_elements, left.shape().rank(), index)
                        .ok_or(ReferenceOperationError::InvalidApplication)?,
                )?;
                let right_bits = decode_element(
                    element_at(right_elements, right.shape().rank(), index)
                        .ok_or(ReferenceOperationError::InvalidApplication)?,
                )?;
                let result = self.arithmetic.apply(
                    &self.format.decode(left_bits),
                    &self.format.decode(right_bits),
                );
                encode_element(self.format.round(&result))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Tensor::dense(Bf16::resolved_type(), result_shape.clone(), elements)
            .map_err(|source| dense_result_error(&source))
    }
}

/// Returns the element one operand contributes at a result position.
///
/// A rank-zero operand contributes its single element everywhere, which is the
/// scalar broadcast the operation admits; every other operand is read positionally.
fn element_at(
    elements: &[ReferenceElement],
    rank: usize,
    index: usize,
) -> Option<&ReferenceElement> {
    elements.get(if rank == 0 { 0 } else { index })
}

impl ReferenceOperation for Bf16BinaryReference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [left, right] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if !request.attributes().fields().is_empty() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        outputs.push(self.combine(left, right)?)
    }
}

/// Registers the governed BF16 value contract and the three BF16 capabilities.
///
/// # Errors
///
/// Returns [`ReferenceRegistryError::UnsupportedBf16`] when the governed
/// declarations cannot parameterize these evaluators, and the registrar's own
/// typed errors otherwise.
pub(crate) fn register_standard_bf16(
    registrar: &mut ReferenceRegistryRegistrar<'_>,
    revision: ReferenceCapabilityRevision,
) -> Result<(), ReferenceRegistryError> {
    let format = Bf16Format::governed()
        .map_err(|source| ReferenceRegistryError::UnsupportedBf16 { source })?;
    registrar.register_value_type(
        Bf16::resolved_type(),
        revision,
        Arc::new(Bf16ValueValidator {
            payload_bytes: format.payload_bytes(),
        }),
    )?;
    registrar.register(
        constant_bf16_op(),
        ReferenceSignature::new([], [Bf16::resolved_type()])?,
        revision,
        Arc::new(ConstantBf16Reference {
            format: format.clone(),
        }),
    )?;
    let binary_signature = ReferenceSignature::new(
        [Bf16::resolved_type(), Bf16::resolved_type()],
        [Bf16::resolved_type()],
    )?;
    registrar.register(
        multiply_bf16_op(),
        binary_signature.clone(),
        revision,
        Arc::new(Bf16BinaryReference::new(
            format.clone(),
            Bf16Arithmetic::Multiply,
        )),
    )?;
    registrar.register(
        add_bf16_op(),
        binary_signature,
        revision,
        Arc::new(Bf16BinaryReference::new(format, Bf16Arithmetic::Add)),
    )
}

#[cfg(test)]
mod tests;
