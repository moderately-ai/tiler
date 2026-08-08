//! The governed `Concatenate` family: extending a tensor along one named axis.
//!
//! **What a `Concatenate` is.** An ordered join of two or more operands along one
//! axis, producing a new value whose extent on that axis is the sum of theirs.
//! Every result element is an operand element unchanged, so the family computes
//! nothing: it is structural in exactly the sense [`super::reindex`] and
//! [`super::broadcast`] are, and it shares their bit-preservation obligation.
//!
//! **It makes no storage claim.** Registering an occurrence says that the
//! *logical* values were joined. It does not claim that bytes were copied, that a
//! destination was preallocated, or that anything was left alone. Whether the
//! join costs a dispatch, becomes a windowed write into a retained allocation, or
//! is composed into a consumer's access map is a physical-planning outcome, and
//! this definition deliberately fixes none of it. In particular, a contiguous
//! byte window exists only for the slowest-varying axis under a row-major layout,
//! so a concatenation along an inner axis writes a strided destination — which is
//! an applicability predicate over a physical candidate, not a second semantic
//! identity.
//!
//! **One general family, not a narrow sequence-extend key.** A key that fixed the
//! axis at zero and the arity at two would owe exactly the obligations the general
//! form owes: the same extent agreement on the non-concatenated axes, the same
//! additive result extent, the same partitioned write, the same ownership proof.
//! Specializing buys nothing and guarantees a second family later.
//!
//! # The extent-domain boundary, and why it is a refusal
//!
//! The result extent on the concatenated axis is the *exact* sum of the operands'
//! extents there. [`ExtentRelation::AdditiveEquality`](crate::shape::ExtentRelation::AdditiveEquality)
//! can retain that relationship for sourced extents without turning
//! [`ExtentTerm`](crate::shape::ExtentTerm) into an expression tree. Semantic
//! value facts can carry sourced extents, but this family requires every operand
//! to pass [`OperationInferenceRequest::static_operand_shape`] before
//! [`concatenate_result_shape`] can compute the relationship directly. Its
//! operand collection stops at the first refusal, so a sourced operand never
//! reaches the literal-shape helper.
//!
//! Because only literal operands reach that helper, the sum is computable there;
//! the contraction's extent agreement likewise explains its family-specific
//! boundary. When the exact sum leaves that domain the family has nothing left
//! to return that the operands determine: saturating at `u64::MAX`, wrapping, or
//! choosing any other value would bind a result extent unrelated to the operands
//! — the static-extent spelling of binding a fresh unconstrained symbol. It is
//! refused under
//! [`ConcatenateError::ResultExtentUnrelatable`] instead. The additive relation
//! does not widen the `u64` extent domain, so this refusal remains necessary and
//! is written here rather than assumed.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::shape::{Axis, Extent, Shape};

use super::registry::standard_conformance;
use super::{
    AttributeFieldId, CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind,
    CanonicalValueView, F32, NormativeDefinitionRef, OpKey, OperationArity,
    OperationAttributeSchema, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, ProviderDiagnosticCode, RegistryError,
    SemanticRegistryRegistrar, ValueFact,
};

/// Fewest operands one concatenation may join.
///
/// Two, because a one-operand concatenation returns its operand and would give one
/// program two identities — the canonicality rule
/// [`super::ReindexFormError::IdentityMapping`] states for the same reason.
pub const MIN_CONCATENATE_OPERANDS: u32 = 2;

/// Most operands one concatenation may join.
///
/// The bound is what the reference provider enumerates, and that is not a
/// coincidence. A reference capability is keyed by an operation together with an
/// *exact* resolved signature, so every admitted arity needs its own registered
/// evaluator; a family whose schema admitted an arity the reference could not
/// evaluate would admit at construction an occurrence nothing can answer for,
/// which is the failure the "reject rather than normalize" rule exists to
/// prevent. The only stated consumer — one decode step's KV append — joins two.
/// Widening this is one constant here and one further registered signature there.
pub const MAX_CONCATENATE_OPERANDS: u32 = 8;

/// Stable field ID carrying the concatenated axis on the occurrence.
pub const CONCATENATE_AXIS_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// Fact field naming what a concatenation does to values.
///
/// The six fields below are the family's semantic signature. Every one is
/// unconditional on this definition: absence is a malformed record, never a
/// default. None of them is numerical, because a concatenation performs no
/// arithmetic — which is itself the fact a reader needs.
pub const CONCATENATE_FACT_VALUE_BEHAVIOUR: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming whether operand order is semantic.
pub const CONCATENATE_FACT_OPERAND_ORDER: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming how the result extent relates to the operands'.
pub const CONCATENATE_FACT_RESULT_EXTENT: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field naming what a zero-extent operand does.
pub const CONCATENATE_FACT_EMPTY_OPERAND: AttributeFieldId = AttributeFieldId::new(4);
/// Fact field naming the dtype permission this family grants.
pub const CONCATENATE_FACT_TYPE_PROMOTION: AttributeFieldId = AttributeFieldId::new(5);
/// Fact field naming what this family claims about storage.
pub const CONCATENATE_FACT_STORAGE_CLAIM: AttributeFieldId = AttributeFieldId::new(6);

/// Returns the governed binary32 concatenation operation key.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn concatenate_f32_op() -> OpKey {
    OpKey::new("tiler", "concatenate-f32", 1).expect("the governed concatenate key is valid")
}

/// Builds the canonical axis attribute one concatenation occurrence carries.
#[must_use]
pub fn concatenate_f32_axis_attribute(axis: Axis) -> CanonicalValue {
    CanonicalValue::unsigned_u32(axis.get())
}

/// A typed refusal of one concatenation occurrence.
///
/// Every variant is one named admission rule. A malformed occurrence is never a
/// generic invalidity and never a value that reaches identity, planning, explain
/// output, or a cache subject: [`concatenate_result_shape`] is the only path to a
/// result, so holding a result is evidence that every rule below was decided.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConcatenateError {
    /// The occurrence joined fewer or more operands than the family admits.
    OperandCount {
        /// Operands the occurrence supplied.
        operands: usize,
        /// Inclusive minimum this family admits.
        minimum: u32,
        /// Inclusive maximum this family admits.
        maximum: u32,
    },
    /// An operand's rank differs from the first operand's.
    ///
    /// Refused rather than padded: this family inserts no implicit unit axis, so
    /// a rank disagreement is a different program rather than a shape to repair.
    RankDisagreement {
        /// Zero-based position of the disagreeing operand.
        operand: usize,
        /// That operand's rank.
        rank: usize,
        /// Operand zero's rank, which every operand must match.
        first: usize,
    },
    /// The named axis does not exist on the operands.
    AxisOutOfRange {
        /// The named axis.
        axis: Axis,
        /// The operands' shared rank.
        rank: usize,
    },
    /// Two operands disagree on an axis other than the concatenated one.
    ///
    /// Both observed extents are reported, because equality does not erase source
    /// identity and reporting one of them would tell a caller half of what it
    /// needs to fix the program. No extent-one operand is stretched to match:
    /// widening belongs to [`broadcast_f32_op`](super::broadcast_f32_op) and is
    /// stated explicitly by an occurrence a caller writes.
    ExtentDisagreement {
        /// The axis on which the two operands disagree.
        axis: Axis,
        /// Zero-based position of the disagreeing operand.
        operand: usize,
        /// That operand's extent on the axis.
        extent: u64,
        /// Operand zero's extent on the same axis.
        first: u64,
    },
    /// The exact result extent leaves the extent domain, so nothing relates it to
    /// the operands'.
    ///
    /// The result extent is the exact sum of the operands' extents on the
    /// concatenated axis. An additive relation can retain a representable sum,
    /// but it cannot widen the extent domain. When the sum is not
    /// computable, every value this family could return would be one the operands
    /// do not determine — so it returns none. Saturating or wrapping here is the
    /// static-extent spelling of binding a fresh unconstrained symbol, and it
    /// would make an occurrence verify while meaning something else.
    ResultExtentUnrelatable {
        /// The concatenated axis.
        axis: Axis,
        /// The exact sum of the extents accumulated before this operand.
        accumulated: u64,
        /// Zero-based position of the operand whose extent overflows the sum.
        operand: usize,
        /// That operand's extent on the concatenated axis.
        extent: u64,
    },
    /// The attribute was not a canonical unsigned 32-bit axis.
    MalformedAxisAttribute,
    /// The result shape exceeded the governed rank profile.
    ResultShape(crate::shape::ShapeError),
}

impl ConcatenateError {
    /// Returns the stable provider diagnostic code naming this refusal.
    ///
    /// Each rule has its own code, so a caller reads *which* rule refused from the
    /// code rather than by matching on a message.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::OperandCount { .. } => "concatenate.operands.arity",
            Self::RankDisagreement { .. } => "concatenate.operands.rank-disagreement",
            Self::AxisOutOfRange { .. } => "concatenate.axis.out-of-range",
            Self::ExtentDisagreement { .. } => "concatenate.operands.extent-disagreement",
            Self::ResultExtentUnrelatable { .. } => "concatenate.axis.result-extent-unrelatable",
            Self::MalformedAxisAttribute => "concatenate.axis.malformed-attribute",
            Self::ResultShape(_) => "concatenate.result-shape",
        }
    }
}

impl fmt::Display for ConcatenateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperandCount {
                operands,
                minimum,
                maximum,
            } => write!(
                formatter,
                "a concatenation joins between {minimum} and {maximum} operands and {operands} were supplied"
            ),
            Self::RankDisagreement {
                operand,
                rank,
                first,
            } => write!(
                formatter,
                "operand {operand} has rank {rank} and operand 0 has rank {first}; a concatenation pads no axis"
            ),
            Self::AxisOutOfRange { axis, rank } => write!(
                formatter,
                "axis {} does not exist on operands of rank {rank}",
                axis.get()
            ),
            Self::ExtentDisagreement {
                axis,
                operand,
                extent,
                first,
            } => write!(
                formatter,
                "operand {operand} has extent {extent} on axis {} and operand 0 has extent {first}; a concatenation requires equal extents on every axis except the concatenated one, and stretches no extent-one axis",
                axis.get()
            ),
            Self::ResultExtentUnrelatable {
                axis,
                accumulated,
                operand,
                extent,
            } => write!(
                formatter,
                "the concatenated extent on axis {} leaves the extent domain: {accumulated} plus operand {operand}'s {extent} is not representable; an additive extent relation does not widen that domain, so no result extent this family could bind would be related to its operands'",
                axis.get()
            ),
            Self::MalformedAxisAttribute => {
                formatter.write_str("the concatenated axis attribute is malformed")
            }
            Self::ResultShape(source) => {
                write!(formatter, "the result shape is not admitted: {source}")
            }
        }
    }
}

impl Error for ConcatenateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ResultShape(source) => Some(source),
            _ => None,
        }
    }
}

/// Decodes the concatenated axis exactly as an occurrence carries it.
///
/// # Errors
///
/// Returns [`ConcatenateError::MalformedAxisAttribute`] when the attribute is not
/// a canonical unsigned 32-bit value. Whether the axis exists is a question about
/// the operands and is decided by [`concatenate_result_shape`].
pub fn concatenate_axis(value: &CanonicalValue) -> Result<Axis, ConcatenateError> {
    let CanonicalValueView::Unsigned {
        width: CanonicalIntegerWidth::Bits32,
        bits,
    } = value.view()
    else {
        return Err(ConcatenateError::MalformedAxisAttribute);
    };
    u32::try_from(bits)
        .map(Axis::new)
        .map_err(|_| ConcatenateError::MalformedAxisAttribute)
}

/// Decides one concatenation against its operands' shapes and derives the result.
///
/// Extent agreement runs through the accepted three-outcome path. Concatenate's
/// inferencer requires every operand to pass
/// [`OperationInferenceRequest::static_operand_shape`] before calling this
/// literal-shape helper; its collection stops at the first refusal, so a sourced
/// operand never reaches equality here. The family has no symbolic equality or
/// unresolved-requirement rule yet; a disproof names both observed extents.
///
/// **A zero-extent operand is admitted and contributes no coordinate.** It is not
/// skipped and not special-cased: it must still agree on rank, and on every axis
/// except the concatenated one, exactly as any other operand must. Concatenating
/// `[8, 0, 128]` with `[8, T, 128]` on axis 1 therefore yields `[8, T, 128]`,
/// whose elements are the second operand's bit for bit. This is stated rather than
/// inherited because prefill binds an empty cache, so it is the pinned occurrence
/// rather than a hypothetical one. An operand empty on some *other* axis forces
/// every operand to be empty there, and the result then has no elements at all;
/// that too is admitted rather than refused.
///
/// # Errors
///
/// Returns [`ConcatenateError`] naming the violated rule.
pub fn concatenate_result_shape(
    axis: Axis,
    operands: &[&Shape],
) -> Result<Shape, ConcatenateError> {
    let admitted = usize::try_from(MIN_CONCATENATE_OPERANDS).unwrap_or(usize::MAX)
        ..=usize::try_from(MAX_CONCATENATE_OPERANDS).unwrap_or(usize::MAX);
    if !admitted.contains(&operands.len()) {
        return Err(ConcatenateError::OperandCount {
            operands: operands.len(),
            minimum: MIN_CONCATENATE_OPERANDS,
            maximum: MAX_CONCATENATE_OPERANDS,
        });
    }
    let first = operands[0];
    let rank = first.rank();
    for (operand, shape) in operands.iter().enumerate().skip(1) {
        if shape.rank() != rank {
            return Err(ConcatenateError::RankDisagreement {
                operand,
                rank: shape.rank(),
                first: rank,
            });
        }
    }
    // Decided after rank agreement, so the axis is checked against a rank every
    // operand shares rather than against whichever operand was consulted first.
    let position = usize::try_from(axis.get()).unwrap_or(usize::MAX);
    if position >= rank {
        return Err(ConcatenateError::AxisOutOfRange { axis, rank });
    }
    for (operand, shape) in operands.iter().enumerate().skip(1) {
        for (index, (extent, expected)) in shape.extents().iter().zip(first.extents()).enumerate() {
            if index != position && extent != expected {
                return Err(ConcatenateError::ExtentDisagreement {
                    axis: Axis::new(u32::try_from(index).unwrap_or(u32::MAX)),
                    operand,
                    extent: extent.get(),
                    first: expected.get(),
                });
            }
        }
    }
    let mut joined = 0_u64;
    for (operand, shape) in operands.iter().enumerate() {
        let extent = shape.extents()[position];
        joined =
            joined
                .checked_add(extent.get())
                .ok_or(ConcatenateError::ResultExtentUnrelatable {
                    axis,
                    accumulated: joined,
                    operand,
                    extent: extent.get(),
                })?;
    }
    let mut result = first.extents().to_vec();
    result[position] = Extent::new(joined);
    Shape::try_new(result).map_err(ConcatenateError::ResultShape)
}

/// Registers the governed concatenation family.
pub(super) fn register_standard_concatenate(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        concatenate_f32_op(),
        OperationSchema::new(
            OperationArity::inclusive(MIN_CONCATENATE_OPERANDS, MAX_CONCATENATE_OPERANDS)
                .expect("the governed concatenate arity range is ordered"),
            OperationArity::exact(1),
            [OperationAttributeSchema::required(
                CONCATENATE_AXIS_ATTRIBUTE,
                CanonicalValueKind::Unsigned,
            )],
        )
        .expect("the governed concatenate schema is valid"),
        NormativeDefinitionRef::new(CONCATENATE_F32_NORMATIVE_DEFINITION)?,
        OperationDefinitionFacts::new(concatenate_facts()),
        standard_conformance("concatenate-f32"),
        OperationEffect::Pure,
        Arc::new(ConcatenateF32),
    ))
    // No algebraic capability is declared, deliberately. A concatenation performs
    // no arithmetic, so it has no associativity or commutativity *of rounding* to
    // declare; that joining is associative over the operand sequence is a
    // structural identity rather than a numerical permission, and a missing
    // declaration reads as unknown rather than as the inverse law.
}

/// The complete normative definition of `tiler::concatenate-f32@1`.
///
/// Held as a constant rather than written inline because it is where the
/// zero-extent rule and the extent-domain contract are stated, and a reader
/// looking for either should find it under a name rather than inside a
/// registration call.
const CONCATENATE_F32_NORMATIVE_DEFINITION: &str = concat!(
    "tiler::concatenate-f32@1; a bit-preserving binary32 join of two or more operands along one named ",
    "axis. Every result element is an operand element unchanged: no value is computed, converted, ",
    "rounded, or canonicalized, so an exceptional payload — a non-canonical NaN, a signalling NaN, a ",
    "signed zero, a subnormal — arrives at the result exactly as it left its operand. ",
    "Operand order is semantic: result coordinates along the concatenated axis run through operand 0's ",
    "coordinates, then operand 1's, and so on, so two occurrences differing only in operand order are ",
    "different computations. ",
    "Admission: every operand has the same rank, the same resolved value type tiler::f32@1, and the same ",
    "extent on every axis except the concatenated one. Each disagreement is refused at construction ",
    "naming the axis and both observed extents. No rank is padded and no extent-one axis is stretched; ",
    "widening is tiler::broadcast-f32@1's and is written as its own occurrence. This family grants no ",
    "dtype promotion, no weak-scalar rule, and no numerical permission. ",
    "The result extent on the concatenated axis is the exact sum of the operands' extents on that axis, ",
    "derived from the operands and never declared by a caller. An occurrence whose exact sum leaves the ",
    "extent domain is refused rather than saturated or wrapped: an additive extent relation does not ",
    "widen that domain, so a result extent this family could not compute would not be related to its ",
    "operands' at all. ",
    "A zero-extent operand is admitted and contributes no coordinate. It is not skipped: it must still ",
    "agree on rank, on resolved value type, and on every axis except the concatenated one. Joining a ",
    "zero-extent operand with one other therefore yields that other operand's extent on the concatenated ",
    "axis, and every result element is that operand's, bit for bit. An operand empty on another axis ",
    "makes every operand empty on that axis and the result has no elements; that is admitted rather than ",
    "refused. ",
    "This operation makes no claim that storage was copied, moved, preallocated, or left alone: it ",
    "states a logical join, and every physical realization of it remains a planning outcome. Whether the ",
    "join has a contiguous byte window depends on the axis and the layout and is an applicability ",
    "predicate over a physical candidate rather than part of this identity.",
);

fn concatenate_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            CONCATENATE_FACT_VALUE_BEHAVIOUR,
            fact("none-every-result-element-is-an-operand-element-unchanged"),
        ),
        CanonicalField::new(
            CONCATENATE_FACT_OPERAND_ORDER,
            fact("semantic-result-coordinates-run-through-the-operands-in-order"),
        ),
        CanonicalField::new(
            CONCATENATE_FACT_RESULT_EXTENT,
            fact("exact-sum-of-the-operand-extents-on-the-concatenated-axis-or-refusal"),
        ),
        CanonicalField::new(
            CONCATENATE_FACT_EMPTY_OPERAND,
            fact("admitted-contributes-no-coordinate-and-still-agrees-on-every-other-axis"),
        ),
        CanonicalField::new(
            CONCATENATE_FACT_TYPE_PROMOTION,
            fact("none-every-operand-is-already-tiler-f32-1"),
        ),
        CanonicalField::new(
            CONCATENATE_FACT_STORAGE_CLAIM,
            fact("none-no-copy-move-or-materialization-is-claimed"),
        ),
    ])
    .expect("the governed concatenate facts are canonical")
}

fn fact(value: &str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("a governed concatenate fact is bounded")
}

struct ConcatenateF32;

impl OperationInferencer for ConcatenateF32 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "concatenate.attributes",
                "a concatenation requires exactly the concatenated-axis attribute".to_owned(),
            ));
        }
        let Some(value) = attributes.get(CONCATENATE_AXIS_ATTRIBUTE) else {
            return Err(op_error(
                "concatenate.attributes",
                "a concatenation requires exactly the concatenated-axis attribute".to_owned(),
            ));
        };
        // The attribute's own rule is decided before anything about the operands,
        // so a malformed axis is refused under its own name rather than under
        // whichever shape check happened to notice first.
        let axis = concatenate_axis(value).map_err(|error| rejection(&error))?;
        let expected = F32::resolved_type();
        for (position, operand) in operands.iter().enumerate() {
            if operand.resolved_type() != &expected {
                return Err(op_error(
                    "concatenate.f32.implicit-promotion",
                    format!(
                        "operand {position} is not tiler::f32@1; the binary32 concatenation admits no \
                         implicit promotion and converts no operand"
                    ),
                ));
            }
        }
        // Concatenation *sums* the extents on its axis and requires equality on
        // every other, so it needs arithmetic over extents rather than a proof
        // of equality. `SourcedExtent` is deliberately not an expression tree —
        // a composed extent is a relation in the environment — so this family
        // has no rule for a symbolic operand yet and declines by name.
        let shapes: Vec<&Shape> = (0..operands.len())
            .map(|position| request.static_operand_shape(position))
            .collect::<Result<_, _>>()?;
        let shape = concatenate_result_shape(axis, &shapes).map_err(|error| rejection(&error))?;
        outputs.try_push(ValueFact::new(expected, shape))
    }
}

fn rejection(error: &ConcatenateError) -> OperationInferenceError {
    op_error(error.diagnostic_code(), error.to_string())
}

fn op_error(code: &str, message: String) -> OperationInferenceError {
    OperationInferenceError::new(
        ProviderDiagnosticCode::new(code).expect("a governed diagnostic code is canonical"),
        message,
    )
    .expect("a governed diagnostic message is canonical")
}

#[cfg(test)]
mod tests;
