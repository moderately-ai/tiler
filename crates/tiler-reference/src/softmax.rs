//! The exact binary32 softmax reference, and its two folds.
//!
//! # What is exact here, and what is certified
//!
//! Every step but one is an exact host operation. The row maximum performs no
//! arithmetic at all — it *selects* one of its contributors' bit patterns — and
//! the subtraction, the fold's adds, the reciprocal division, and the per-element
//! multiplies are binary32 round-to-nearest ties-to-even, which ADR 0024 fixes
//! and which the host provides exactly. The exponential is the one step a host
//! library computes to *its own* accuracy.
//!
//! **So the exponential is certified rather than trusted, and this module reuses
//! the activation's certification rather than repeating it.**
//! [`certified_exp_f32`](crate::certified_exp_f32) brackets `e^t` with the
//! crate's exact-rational enclosure and refuses when the bracket straddles a
//! rounding boundary. It is *one* implementation shared by
//! `tiler::silu-f32@1` and `tiler::softmax-f32@1`, not two copies: the two keys
//! carry different [`AccuracyContract`](tiler_ir::semantic::accuracy::AccuracyContract)
//! instances because a contract names the operation it speaks about, but the
//! exponential is the same function and two copies of it would let this corpus
//! pass against arithmetic the activation never runs.
//!
//! # The extrema family this reference implements, and why it is not `f32::max`
//!
//! [`maximum_f32`] is the NaN-propagating IEEE 754-2019 `maximum` with the
//! deterministic `-0.0 < +0.0` ordering, which is decision **D-2** as
//! `tiler::softmax-f32@1` settles it. Rust's `f32::max` is the *other* family —
//! it is `maxNum`, returning the numeric operand when exactly one is NaN — so
//! writing it here would have silently installed `MaximumNumber`. That is the
//! substitution this module exists to avoid, and
//! `the_reference_maximum_is_not_the_host_maximum` is the check.
//!
//! # The normalization form, which is the reverse of the siblings'
//!
//! `r_i = e_i * (1.0 / d)`: one division of one by the denominator, then one
//! multiply per element. Deliberately **not** `e_i / d`, which rounds once where
//! this rounds twice and is a different binary32 function. The retained probe
//! counts every discriminating element of a width-two or width-three row matching
//! this form and none matching the division.
//!
//! # Measurement — the reference model sums the same contributors in another order
//!
//! **On rows of four or more contributors, `torch.nn.functional.softmax` matches
//! neither pinned spelling's bits, and the reason is the denominator's contributor
//! order rather than an approximation.** Measured in the retained probe's own
//! pinned environment (`torch` 2.6.0, `transformers` 4.51.0, CPU, F32): at row
//! widths two and three the reference agrees with this form at *every* element —
//! 40,000 and 60,000 elements, zero disagreements — and from width four upward it
//! disagrees with both `e_i * (1/d)` and `e_i / d` by up to four ULP. At the L3′
//! record's own worked example the whole difference is one constant: the
//! reference's implied constant is `0x3f2a4d3a` where the correctly rounded
//! `1.0 / d` over the strict left fold's `d = 0x3fc06957` is `0x3f2a4d3b`. That
//! constant is not an approximate reciprocal — it is the **correctly rounded**
//! reciprocal of `0x3fc06958`, which this row's own four exponentials reach under
//! the contributor order `(e₀, e₂, e₁, e₃)`. So the reference is evaluating *this*
//! formula over a permuted contributor sequence: the exponential, the reciprocal,
//! and the per-element multiply are each correctly rounded there, and only the
//! denominator's contributor order differs.
//!
//! **That reading is the stronger one, and it is the family's own order contract
//! observed rather than a numerical defect.** `SOFTMAX_F32_FACT_SUM_FOLD_ORDER`
//! pins the strict left fold and moves only under the separately resolved
//! reassociation and permutation permissions, which is exactly the freedom the
//! reference model is taking here — so *matching* the reference's bits at these
//! widths would mean performing an unpermitted reassociation, not passing a
//! conformance check. It also means the reference model cannot settle the legality
//! question: a schedule that permuted the sum without the permission would agree
//! with it and would still be illegal.
//!
//! **The form question is settled at every width by the same measurement, and the
//! order attribution carries a boundary.** Over 20,000 rows per width the
//! reference's output row is exactly one scalar multiple of these exponentials at
//! every element, at all five measured widths — and a division by a denominator is
//! not a single-constant multiply, which is why this supports the reciprocal form
//! more broadly than the width-two and width-three element counts do. At width
//! four, where the summation orders are enumerable, 19,895 of 20,000 implied
//! constants are the correctly rounded reciprocal of a denominator those
//! exponentials reach under some strict left fold or the balanced tree; the
//! enumeration is not every legal grouping, so that count is a lower bound on
//! reachability. Widths eight and eighteen are not enumerable and stay open.
//! `the_retained_worked_example_reproduces_the_pinned_formula` carries both
//! denominators and both reciprocals as bits.

use std::sync::Arc;

use tiler_ir::semantic::{CanonicalValueView, F32, SOFTMAX_REDUCED_AXES_ATTRIBUTE};
use tiler_ir::shape::{Axis, Shape};

use super::canonicalize_arithmetic_f32;
use super::error::{ReferenceOperationError, dense_result_error};
use super::evaluate::{RowGeometry, decode_f32, f32_element, f32_elements};
use super::registry::{ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs};
use super::silu::certified_exp_f32;
use super::tensor::{ReferenceElement, Tensor};

/// Registers the governed softmax reference implementation.
pub(crate) struct SoftmaxF32Reference;

impl ReferenceOperation for SoftmaxF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let [input] = operands else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let shape = input.shape();
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let axis = reduced_axis(&request, shape.rank())?;

        let values: Vec<f32> = f32_elements(input)?
            .iter()
            .map(decode_f32)
            .collect::<Result<Vec<f32>, ReferenceOperationError>>()?;
        let count = shape
            .element_count()
            .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        if values.len() != count {
            return Err(ReferenceOperationError::InvalidApplication);
        }

        let mapped = softmax_dense(shape, axis, &values)?
            .into_iter()
            .map(f32_element)
            .collect::<Result<Vec<ReferenceElement>, ReferenceOperationError>>()?;
        let tensor = Tensor::dense(F32::resolved_type(), shape.clone(), mapped)
            .map_err(|source| dense_result_error(&source))?;
        outputs.push(tensor)
    }
}

/// Returns the IEEE 754-2019 `maximum` of two binary32 values.
///
/// The NaN-propagating extrema family with the deterministic `-0.0 < +0.0`
/// ordering, which is what `tiler::softmax-f32@1` pins and what ADR 0023 keeps
/// separate from `maximumNumber`.
///
/// Three cases, and the middle one is the one a naive comparison gets wrong:
///
/// - either operand NaN: NaN, so a single NaN score poisons its row's maximum;
/// - equal operands: the bitwise **and** of the two payloads. Two binary32 values
///   compare equal only when their bit patterns agree *or* they are opposite
///   zeros, and the `and` clears the sign bit in exactly the second case — which
///   is `-0.0 < +0.0` written as one operation rather than as a branch;
/// - otherwise: the greater.
///
/// The `and` is deliberately not `if a.is_sign_negative() { b } else { a }`, which
/// would be equivalent here but would state the zero rule as a sign test rather
/// than as the ordering it is.
#[must_use]
pub(crate) fn maximum_f32(left: f32, right: f32) -> f32 {
    if left.is_nan() || right.is_nan() {
        return f32::NAN;
    }
    // The equality is the *point* rather than a tolerance question: IEEE-754
    // equality is exactly the predicate that is true for identical values and
    // for the opposite-zero pair, which is the case this arm exists to decide.
    // A margin comparison would merge distinct neighbours and would still not
    // separate the zeros.
    #[allow(
        clippy::float_cmp,
        reason = "the extrema family is defined by exact IEEE-754 comparison; a margin would merge distinct values and would not decide the opposite-zero pair this arm exists for"
    )]
    let equal = left == right;
    if equal {
        return f32::from_bits(left.to_bits() & right.to_bits());
    }
    if left > right { left } else { right }
}

/// Returns the reference values of one dense binary32 softmax.
///
/// The evaluator's own arithmetic, reachable without assembling a tensor and a
/// request, so a corpus row states its inputs as a slice — and it is the *same*
/// function the registered evaluator calls rather than a second copy of the
/// formula, because two copies would let a corpus pass against arithmetic the
/// registry never runs.
///
/// # Errors
///
/// Returns [`ReferenceOperationError`] for an axis outside the shape, for a
/// values length that disagrees with the shape, or for an undecided exponential.
pub fn softmax_f32(
    shape: &Shape,
    axis: Axis,
    values: &[f32],
) -> Result<Vec<f32>, ReferenceOperationError> {
    let count = shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    if values.len() != count {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    softmax_dense(shape, axis, values)
}

/// Evaluates the pinned formula over a dense row-major tensor.
fn softmax_dense(
    shape: &Shape,
    axis: Axis,
    values: &[f32],
) -> Result<Vec<f32>, ReferenceOperationError> {
    let geometry = RowGeometry::derive(shape, axis)?;
    let mut mapped = vec![0.0_f32; values.len()];
    // A zero-length reduced axis makes every row empty, so no scalar softmax is
    // evaluated and the shape-preserving result is empty too. It is decided here
    // rather than discovered, and it is what keeps the identity-less maximum from
    // ever facing an empty contributor domain: with no contributor there is no
    // admissible seed, so the row loop must not run rather than run and pick one.
    if geometry.extent == 0 {
        return Ok(mapped);
    }
    for row in 0..geometry.rows {
        row_softmax(&geometry, values, row, &mut mapped)?;
    }
    Ok(mapped)
}

/// Evaluates the pinned formula over one row, writing its outputs in place.
fn row_softmax(
    geometry: &RowGeometry,
    values: &[f32],
    row: usize,
    mapped: &mut [f32],
) -> Result<(), ReferenceOperationError> {
    // The first fold: the strict left fold of the `Maximum` family over the
    // canonical contributor sequence, seeded at the first contributor. Seeding is
    // not a choice here the way it is for a sum — the family has no identity, so
    // the first contributor is the only admissible seed and an empty row has no
    // maximum at all.
    let mut maximum = values[geometry.element_index(row, 0)];
    for position in 1..geometry.extent {
        maximum = maximum_f32(maximum, values[geometry.element_index(row, position)]);
    }

    // The prologue: one subtraction rounding once, then the one inexact step.
    // Storing the exponentials is what makes the second fold a fold over *them*
    // rather than a recomputation, which is the contributor sequence the identity
    // names.
    let mut exponentials = Vec::with_capacity(geometry.extent);
    for position in 0..geometry.extent {
        let score = values[geometry.element_index(row, position)];
        exponentials.push(certified_exp_f32(score - maximum)?);
    }

    // The second fold: the strict left fold sum over the same sequence, seeded at
    // the first contributor — the order `tiler::strict-serial-sum-f32@1` declares.
    // Seeding with `+0.0` instead would lose the sign of a single-contributor
    // row; an exponential is never a negative zero, but the seeding rule is the
    // reduction's and not this operation's to vary.
    let mut denominator = exponentials[0];
    for value in &exponentials[1..] {
        denominator += *value;
    }

    // One division of one by the denominator, then one multiply per element.
    // Written in this order because that *is* the pinned formula: `e_i / d` is a
    // different binary32 function, and it is the spelling this line exists to
    // exclude.
    let reciprocal = 1.0_f32 / denominator;
    for (position, exponential) in exponentials.iter().enumerate() {
        let index = geometry.element_index(row, position);
        mapped[index] = canonicalize_arithmetic_f32(exponential * reciprocal);
    }
    Ok(())
}

/// Resolves the single reduced axis from the occurrence's attributes.
fn reduced_axis(
    request: &ReferenceEvaluationRequest<'_>,
    rank: usize,
) -> Result<Axis, ReferenceOperationError> {
    let Some(CanonicalValueView::Sequence(values)) = request
        .attributes()
        .get(SOFTMAX_REDUCED_AXES_ATTRIBUTE)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    let [only] = values else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    let CanonicalValueView::Unsigned { width, bits } = only.view() else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    if width != tiler_ir::semantic::CanonicalIntegerWidth::Bits32 {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let axis =
        Axis::new(u32::try_from(bits).map_err(|_| ReferenceOperationError::InvalidApplication)?);
    if usize::try_from(axis.get()).is_ok_and(|position| position < rank) {
        Ok(axis)
    } else {
        Err(ReferenceOperationError::InvalidApplication)
    }
}

/// Returns the governed softmax reference for registration.
pub(crate) fn softmax_reference() -> Arc<dyn ReferenceOperation> {
    Arc::new(SoftmaxF32Reference)
}

#[cfg(test)]
mod tests;
