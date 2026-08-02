//! The exact binary32 `SiLU` reference, `y = x / (1 + Exp(-x))`.
//!
//! # Why the exponential is certified rather than trusted
//!
//! Three of the four steps are exact host operations: the negation is IEEE sign
//! manipulation, and the addition and the division are binary32 round-to-nearest
//! ties-to-even, which ADR 0024 fixes and which the host provides exactly. The
//! exponential is the one step a host library computes to *its own* accuracy, and
//! a reference that took `f64::exp(..) as f32` on trust would be comparing one
//! approximation against another and calling the answer a reference.
//!
//! So the host value is a **candidate**, never the answer. [`certified_exp_f32`]
//! brackets `e^t` with the crate's exact-rational
//! [`exp_enclosure`](crate::accuracy::exp_enclosure) and admits the candidate only
//! when the bracket lies strictly inside the candidate's own round-to-nearest
//! interval — which proves the candidate *is* the correctly rounded binary32
//! value. When the bracket straddles a rounding boundary the reference refuses
//! with [`ReferenceOperationError::UndecidedTranscendentalReference`] rather than
//! picking the nearer side. A reference that resolved its own uncertainty could
//! not fail, and the whole point of this module is that it can.
//!
//! # What this reference is, against the accuracy contract
//!
//! It produces the **canonical** member of the operation's admitted result set:
//! the composition in which the subordinate exponential is correctly rounded.
//! `tiler::silu-f32@1` admits a *set* — its exponential carries a twelve-ULP
//! bound — so a conformance decision compares a candidate against
//! [`crate::accuracy::decide_contract`] with the registered contract, not against
//! this value bit for bit. This value is what a bit-exact comparison uses when the
//! implementation under test is itself correctly rounded, and it is what the
//! boundary corpus pins.
//!
//! # The two shortcuts, and why each is exact rather than approximate
//!
//! [`exp_enclosure`](crate::accuracy::exp_enclosure) reduces its argument by
//! halving and refuses beyond a governed halving bound, so it does not cover every
//! finite binary32 argument. Two guards close that gap, and both are exact
//! inequalities rather than tolerances:
//!
//! - `t > 89` implies `e^t > e^88.723 > f32::MAX`, because `ln(f32::MAX)` is
//!   `88.7228391...`; the finite-overflow rule gives `+inf`.
//! - `t < -104` implies `e^t < 2^-150`, because `-150 * ln 2` is `-103.972...`;
//!   `2^-150` is exactly half the least positive subnormal, so round-to-nearest
//!   gives `+0.0` — and ties-to-even would give `+0.0` at the midpoint itself.
//!
//! Everything in between has `|t| <= 104 < 2^7`, which the halving bound covers
//! with room to spare.

use std::sync::Arc;

use tiler_ir::semantic::accuracy::ExactRational;

use super::accuracy::{CertifiedEnclosure, EnclosurePrecision, exp_enclosure};
use super::error::{ReferenceOperationError, dense_result_error};
use super::evaluate::{decode_f32, f32_element, f32_elements};
use super::registry::{ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs};
use super::tensor::Tensor;
use super::{canonicalize_arithmetic_f32, tensor::ReferenceElement};

use tiler_ir::semantic::F32;

/// Argument above which the exponential provably overflows binary32.
const EXPONENTIAL_OVERFLOW_GUARD: f32 = 89.0;

/// Argument below which the exponential provably rounds to positive zero.
const EXPONENTIAL_UNDERFLOW_GUARD: f32 = -104.0;

/// Registers the governed `SiLU` reference implementation.
pub(crate) struct SiluF32Reference;

impl ReferenceOperation for SiluF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let [input] = operands else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if !request.attributes().fields().is_empty() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let elements = f32_elements(input)?;
        let shape = input.shape();
        let count = shape
            .element_count()
            .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        let mapped = (0..count)
            .map(|index| {
                let value = decode_f32(&elements[index])?;
                f32_element(canonicalize_arithmetic_f32(silu_f32(value)?))
            })
            .collect::<Result<Vec<ReferenceElement>, ReferenceOperationError>>()?;
        let tensor = Tensor::dense(F32::resolved_type(), shape.clone(), mapped)
            .map_err(|source| dense_result_error(&source))?;
        outputs.push(tensor)
    }
}

/// Returns the canonical binary32 `SiLU` reference value of one argument.
///
/// The three exceptional arguments are decided before any arithmetic, and each is
/// a consequence of the pinned formula rather than a repair of it:
///
/// - `NaN` propagates, and the caller canonicalizes the payload;
/// - `+inf` gives `exp(-inf) = +0.0`, so the divisor is `1.0` and the result is
///   `+inf`;
/// - `-inf` gives `exp(+inf) = +inf`, so the divisor is `+inf` and the result is
///   `-inf / +inf`, which IEEE-754 makes a **NaN**. The reference is not total on
///   the extended reals, and this records that rather than repairing it.
///
/// Both signed zeros fall out of the ordinary path: `exp(∓0.0)` is exactly `1.0`,
/// the divisor is `2.0`, and `±0.0 / 2.0` preserves the sign.
///
/// # Errors
///
/// Returns [`ReferenceOperationError::UndecidedTranscendentalReference`] when the
/// certified enclosure cannot prove which binary32 value the exponential rounds
/// to.
pub fn silu_f32(argument: f32) -> Result<f32, ReferenceOperationError> {
    if argument.is_nan() {
        return Ok(f32::NAN);
    }
    if argument == f32::INFINITY {
        return Ok(f32::INFINITY);
    }
    if argument == f32::NEG_INFINITY {
        return Ok(f32::NAN);
    }
    // Exact: negating a finite binary32 value flips the sign bit and changes
    // nothing else, so this introduces no rounding of its own.
    let exponent_argument = -argument;
    let exponential = certified_exp_f32(exponent_argument)?;
    // Both remaining steps are binary32 round-to-nearest ties-to-even, which is
    // what ADR 0024 fixes and what the host performs exactly.
    let divisor = 1.0_f32 + exponential;
    Ok(argument / divisor)
}

/// Returns the provably correctly rounded binary32 value of `e^argument`.
///
/// # Errors
///
/// Returns [`ReferenceOperationError::UndecidedTranscendentalReference`] when the
/// enclosure straddles a rounding boundary or cannot be produced. Both are
/// refusals rather than a nearest-side guess.
pub fn certified_exp_f32(argument: f32) -> Result<f32, ReferenceOperationError> {
    if argument.is_nan() {
        return Ok(f32::NAN);
    }
    if argument >= EXPONENTIAL_OVERFLOW_GUARD {
        return Ok(f32::INFINITY);
    }
    if argument <= EXPONENTIAL_UNDERFLOW_GUARD {
        return Ok(0.0);
    }
    if argument == 0.0 {
        // Exactly representable, and the only argument at which the reference is
        // rational; the enclosure would be exact here but stating it directly
        // keeps the tie analysis below restricted to irrational references.
        return Ok(1.0);
    }
    let exact = ExactRational::from_f32(argument).ok_or(undecided())?;
    let enclosure =
        exp_enclosure(&exact, EnclosurePrecision::binary32_corpus()).map_err(|_| undecided())?;

    // Round-to-nearest sends every reference at or above this threshold to
    // infinity: it is the midpoint between `f32::MAX` and the first value the
    // format cannot represent, `2^128`.
    let overflow_threshold =
        ExactRational::power_of_two(128).subtract(&ExactRational::power_of_two(103));
    if *enclosure.lower() >= overflow_threshold {
        return Ok(f32::INFINITY);
    }
    if *enclosure.upper() >= overflow_threshold {
        return Err(undecided());
    }

    // The candidate is the host library's answer and carries no authority; the
    // enclosure decides whether it is the correctly rounded one. The narrowing
    // cast is the point rather than a hazard: what this function must decide is
    // which *binary32* value the reference rounds to, and a candidate the cast
    // pushed to an infinity or a zero simply fails `rounds_to` below.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the narrowed value is a candidate the certified enclosure then accepts or refuses, never a result taken on trust"
    )]
    let candidate = f64::from(argument).exp() as f32;
    if !candidate.is_finite() || candidate < 0.0 {
        return Err(undecided());
    }
    if rounds_to(&enclosure, candidate) {
        return Ok(candidate);
    }
    // The host was not exact. Its immediate neighbours are the only other values
    // the enclosure can admit, because the enclosure is far narrower than one
    // binary32 gap over this whole range.
    for neighbour in [previous_f32(candidate), next_f32(candidate)] {
        if neighbour.is_finite() && neighbour >= 0.0 && rounds_to(&enclosure, neighbour) {
            return Ok(neighbour);
        }
    }
    Err(undecided())
}

/// Returns whether the bracketed reference provably rounds to `candidate`.
///
/// Strict on both sides, so an enclosure touching a rounding midpoint is refused
/// rather than resolved by the ties-to-even rule. That refusal costs nothing here
/// and would be unsound in general: `e^t` is irrational at every nonzero
/// representable `t`, so a reference *at* a midpoint cannot occur, and admitting
/// the boundary would only widen what an inexact enclosure could claim.
fn rounds_to(enclosure: &CertifiedEnclosure, candidate: f32) -> bool {
    let Some(value) = ExactRational::from_f32(candidate) else {
        return false;
    };
    let lower_midpoint = if candidate == 0.0 {
        // The reference is a positive exponential, so nothing below zero is
        // reachable and zero itself is the effective floor.
        ExactRational::zero()
    } else {
        midpoint(&value, previous_f32(candidate))
    };
    let successor = next_f32(candidate);
    if !successor.is_finite() {
        return false;
    }
    let upper_midpoint = midpoint(&value, successor);
    *enclosure.lower() > lower_midpoint && *enclosure.upper() < upper_midpoint
}

fn midpoint(value: &ExactRational, neighbour: f32) -> ExactRational {
    let neighbour =
        ExactRational::from_f32(neighbour).unwrap_or_else(|| unreachable!("a finite neighbour"));
    value.add(&neighbour).scale_by_power_of_two(-1)
}

/// Returns the next binary32 value above a finite non-negative `value`.
fn next_f32(value: f32) -> f32 {
    if value == 0.0 {
        return f32::from_bits(1);
    }
    f32::from_bits(value.to_bits() + 1)
}

/// Returns the next binary32 value below a finite positive `value`.
fn previous_f32(value: f32) -> f32 {
    if value == 0.0 {
        return -f32::from_bits(1);
    }
    f32::from_bits(value.to_bits() - 1)
}

const fn undecided() -> ReferenceOperationError {
    ReferenceOperationError::UndecidedTranscendentalReference
}

/// Returns the governed `SiLU` reference implementation for registration.
pub(crate) fn silu_reference() -> Arc<dyn ReferenceOperation> {
    Arc::new(SiluF32Reference)
}

#[cfg(test)]
mod tests;
