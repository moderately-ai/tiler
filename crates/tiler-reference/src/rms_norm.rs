//! The exact binary32 root-mean-square normalization reference.
//!
//! # Why the reciprocal square root is certified rather than trusted
//!
//! Every step but one is an exact host operation: the squares, the fold, the
//! division by the extent, the `eps` addition, and the two multiplies are all
//! binary32 round-to-nearest ties-to-even, which ADR 0024 fixes and which the
//! host provides exactly. The reciprocal square root is the one step a host
//! library computes to *its own* accuracy, and a reference that took
//! `f64::sqrt(..).recip() as f32` on trust would be comparing one approximation
//! against another and calling the answer a reference.
//!
//! So the host value is a **candidate**, never the answer. [`certified_rsqrt_f32`]
//! brackets `1/sqrt(t)` with the crate's exact-rational
//! [`rsqrt_enclosure`](crate::accuracy::rsqrt_enclosure) and admits the candidate
//! only when the bracket lies strictly inside the candidate's own
//! round-to-nearest interval. When the bracket straddles a rounding boundary the
//! reference refuses with
//! [`ReferenceOperationError::UndecidedTranscendentalReference`] rather than
//! picking the nearer side.
//!
//! # Why the strict tie refusal is sound, which needs an argument `exp` did not
//!
//! `exp(t)` is irrational at every nonzero representable `t`, so its reference
//! can never sit exactly on a rounding midpoint and a strict refusal costs
//! nothing. The reciprocal square root is different: `1/sqrt(t)` *is* rational at
//! infinitely many binary32 arguments, so the case has to be decided rather than
//! excluded by irrationality.
//!
//! **It is decided, and the answer is that a midpoint is still unreachable.**
//! Write a finite positive binary32 `t` in lowest terms as `m · 2^k` with `m` an
//! odd integer. `1/sqrt(t)` is a *dyadic* rational — the only kind a binary32
//! value or a binary32 midpoint can be — exactly when `sqrt(m)` is an integer and
//! that integer is one, that is when `m = 1` and `k` is even; then `t = 2^k` and
//! `1/sqrt(t) = 2^(-k/2)`, a power of two. When `m > 1` is an odd perfect square
//! the reference is rational but has the odd factor `sqrt(m)` in its denominator,
//! so it is not dyadic and is neither representable nor a midpoint; when `m` is
//! not a perfect square or `k` is odd the reference is irrational. A binary32
//! rounding midpoint carries exactly twenty-five significant bits, and a power of
//! two carries one, so the reachable rational references are all *representable*
//! and none is a midpoint. The strict comparison is therefore sound and the
//! refusal it protects is unreachable on this domain — and
//! [`certified_rsqrt_f32`] still refuses rather than resolving, because the
//! argument above is a proof about the reference and not about the width of any
//! particular enclosure.
//!
//! # What this reference is, against the accuracy contract
//!
//! It produces the **canonical** member of the operation's admitted result set:
//! the composition in which the subordinate reciprocal square root is correctly
//! rounded. `tiler::rms-norm-f32@1` admits a *set* — its reciprocal square root
//! carries a `Faithful` contract, so either binary32 neighbour of the exact
//! reference is legal — and a conformance decision compares a candidate against
//! [`crate::accuracy::decide_contract`] with the registered contract rather than
//! against this value bit for bit.
//!
//! **Measurement — the workload's own reference implementation does not satisfy
//! that contract at one measured argument, and this records it rather than
//! widening to fit.** The [retained reference-semantics
//! probe](../../../spikes/numerics/transformer_reference_semantics/README.md)
//! records `torch.rsqrt(1e-6f32)` as `0x4479ffff`. The exact reference is
//! `1000.00000126…`, whose correctly rounded binary32 value is `0x447a0000` and
//! whose bracketing pair is `(0x447a0000, 0x447a0001)`; `0x4479ffff` is one step
//! below that pair, about `1.02` ULP from the exact value, so it is outside the
//! faithful result set. It is exactly what the two-rounding `1 / sqrt(t)`
//! composition delivers at that argument, which is the substitution the pinned
//! formula's choice of `rsqrt` exists to exclude. The consequence is a named
//! divergence between this reference and the workload's, not a defect on either
//! side, and it propagates: the probe's `rms_subnormal_vector` row is
//! `0x02081cb9` where this reference gives `0x02081cba`, one step apart for the
//! same reason.

use std::sync::Arc;

use tiler_ir::semantic::accuracy::ExactRational;
use tiler_ir::semantic::{
    CanonicalValueView, F32, RMS_NORM_EPS_BITS_ATTRIBUTE, RMS_NORM_REDUCED_AXES_ATTRIBUTE, TypeKey,
};
use tiler_ir::shape::{Axis, Shape};

use super::accuracy::{CertifiedEnclosure, EnclosurePrecision, rsqrt_enclosure};
use super::canonicalize_arithmetic_f32;
use super::error::ReferenceOperationError;
use super::evaluate::{RowGeometry, decode_f32, f32_element, f32_elements};
use super::registry::{ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs};
use super::tensor::{ReferenceElement, Tensor};

/// Registers the governed RMS normalization reference implementation.
pub(crate) struct RmsNormF32Reference;

impl ReferenceOperation for RmsNormF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let [input, weight] = operands else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let shape = input.shape();
        if weight.shape() != shape {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let attributes = request.attributes();
        if attributes.fields().len() != 2 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let axis = normalized_axis(&request, shape.rank())?;
        let eps_payload = eps_bits(&request)?;

        let values = decoded(f32_elements(input)?)?;
        let weights = decoded(f32_elements(weight)?)?;
        let count = shape
            .element_count()
            .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        if values.len() != count || weights.len() != count {
            return Err(ReferenceOperationError::InvalidApplication);
        }

        let mapped = normalize_dense(shape, axis, eps_payload, &values, &weights)?
            .into_iter()
            .map(f32_element)
            .collect::<Result<Vec<ReferenceElement>, ReferenceOperationError>>()?;
        let tensor = Tensor::dense(F32::resolved_type(), shape.clone(), mapped)
            .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
        outputs.push(tensor)
    }
}

impl RowGeometry {
    /// Returns `Rsqrt(mean-of-squares + eps)` for one normalized row.
    fn row_scale(
        &self,
        values: &[f32],
        row: usize,
        eps: f32,
    ) -> Result<f32, ReferenceOperationError> {
        // An empty normalized axis makes the fold empty, whose identity is the
        // `+0.0` the strict serial sum declares. It is stated rather than
        // discovered: the extent is an attribute of the operand's shape and not a
        // proof, so a zero-extent occurrence is decided here.
        let mut accumulator: Option<f32> = None;
        for position in 0..self.extent {
            let value = values[self.element_index(row, position)];
            // One rounding per square, then one per combine, in the canonical
            // contributor order — the strict left fold seeded at the first
            // contributor, which is what `tiler::strict-serial-sum-f32@1` also
            // declares. Seeding with `+0.0` instead would lose the sign of a
            // single-contributor row, and while a square is never a negative
            // zero, the seeding rule is the reduction's and not this operation's
            // to vary.
            let square = value * value;
            accumulator = Some(match accumulator {
                None => square,
                Some(total) => total + square,
            });
        }
        let total = accumulator.unwrap_or(0.0);
        #[allow(
            clippy::cast_precision_loss,
            reason = "the extent is a bounded tensor dimension; a widening to f32 is exact for every extent this profile admits, and an extent beyond 2^24 is refused by the checked conversion above rather than silently rounded here"
        )]
        let extent = self.extent as f32;
        // A division, deliberately, and never a multiplication by `1 / N`. The
        // two agree exactly at a power-of-two extent and round twice at every
        // other one, so the substitution would make the operation's meaning
        // depend on the extent.
        let mean = if self.extent == 0 {
            total
        } else {
            total / extent
        };
        certified_rsqrt_f32(mean + eps)
    }
}

/// Returns the provably correctly rounded binary32 value of `1 / sqrt(argument)`.
///
/// The four arguments decided before any enclosure are each a consequence of the
/// pinned formula rather than a repair of it:
///
/// - a NaN argument has a NaN reference;
/// - `+inf` gives `+0.0`. This is the route a squaring overflow takes, and it is
///   the whole of decision **D-3**'s mechanism: the mean of squares is infinite,
///   the scale is exactly zero, and every output of the row is a signed zero;
/// - `+0.0` gives `+inf` and `-0.0` gives `-inf`, the infinite-reference rule.
///   Neither is reachable while `eps` is positive, and both are stated because
///   the function is defined independently of which of its inputs this workload
///   happens to reach;
/// - a negative argument is a domain error and gives a NaN.
///
/// # Errors
///
/// Returns [`ReferenceOperationError::UndecidedTranscendentalReference`] when the
/// certified enclosure cannot prove which binary32 value the reference rounds to.
/// This module's header proves the case is unreachable on the admitted domain;
/// the refusal remains because that is a proof about the reference rather than
/// about any particular enclosure's width.
pub fn certified_rsqrt_f32(argument: f32) -> Result<f32, ReferenceOperationError> {
    if argument.is_nan() {
        return Ok(f32::NAN);
    }
    if argument == f32::INFINITY {
        return Ok(0.0);
    }
    if argument == 0.0 {
        return Ok(if argument.is_sign_negative() {
            f32::NEG_INFINITY
        } else {
            f32::INFINITY
        });
    }
    if argument < 0.0 {
        return Ok(f32::NAN);
    }
    let exact = ExactRational::from_f32(argument).ok_or(undecided())?;
    let enclosure =
        rsqrt_enclosure(&exact, EnclosurePrecision::binary32_corpus()).map_err(|_| undecided())?;

    // The candidate is the host library's answer and carries no authority; the
    // enclosure decides whether it is the correctly rounded one. The reciprocal
    // is taken at `f64`, where every binary32 argument's square root and its
    // reciprocal are far from the format's own boundaries, so the narrowing cast
    // is the only rounding the candidate carries — and a candidate the cast
    // pushed anywhere wrong simply fails `rounds_to` below.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the narrowed value is a candidate the certified enclosure then accepts or refuses, never a result taken on trust"
    )]
    let candidate = f64::from(argument).sqrt().recip() as f32;
    if !candidate.is_finite() || candidate <= 0.0 {
        return Err(undecided());
    }
    if rounds_to(&enclosure, candidate) {
        return Ok(candidate);
    }
    // The host was not exact. Its immediate neighbours are the only other values
    // the enclosure can admit, because the enclosure is far narrower than one
    // binary32 gap over this whole range.
    for neighbour in [previous_f32(candidate), next_f32(candidate)] {
        if neighbour.is_finite() && neighbour > 0.0 && rounds_to(&enclosure, neighbour) {
            return Ok(neighbour);
        }
    }
    Err(undecided())
}

/// Returns whether the bracketed reference provably rounds to `candidate`.
///
/// Strict on both sides, so an enclosure touching a rounding midpoint is refused
/// rather than resolved by the ties-to-even rule. This module's header proves
/// that no reachable reference sits on a midpoint, so the strictness costs
/// nothing here; admitting the boundary would only widen what an inexact
/// enclosure could claim.
fn rounds_to(enclosure: &CertifiedEnclosure, candidate: f32) -> bool {
    let Some(value) = ExactRational::from_f32(candidate) else {
        return false;
    };
    let predecessor = previous_f32(candidate);
    if !predecessor.is_finite() || predecessor < 0.0 {
        return false;
    }
    let successor = next_f32(candidate);
    if !successor.is_finite() {
        return false;
    }
    let lower_midpoint = midpoint(&value, predecessor);
    let upper_midpoint = midpoint(&value, successor);
    *enclosure.lower() > lower_midpoint && *enclosure.upper() < upper_midpoint
}

fn midpoint(value: &ExactRational, neighbour: f32) -> ExactRational {
    let neighbour =
        ExactRational::from_f32(neighbour).unwrap_or_else(|| unreachable!("a finite neighbour"));
    value.add(&neighbour).scale_by_power_of_two(-1)
}

/// Returns the next binary32 value above a finite positive `value`.
fn next_f32(value: f32) -> f32 {
    f32::from_bits(value.to_bits() + 1)
}

/// Returns the next binary32 value below a finite positive `value`.
fn previous_f32(value: f32) -> f32 {
    f32::from_bits(value.to_bits() - 1)
}

const fn undecided() -> ReferenceOperationError {
    ReferenceOperationError::UndecidedTranscendentalReference
}

/// Resolves the single normalized axis from the occurrence's attributes.
fn normalized_axis(
    request: &ReferenceEvaluationRequest<'_>,
    rank: usize,
) -> Result<Axis, ReferenceOperationError> {
    let Some(CanonicalValueView::Sequence(values)) = request
        .attributes()
        .get(RMS_NORM_REDUCED_AXES_ATTRIBUTE)
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
    if usize::try_from(axis.get()).map_or(true, |position| position >= rank) {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    Ok(axis)
}

/// Resolves the exact `eps` payload from the occurrence's attributes.
fn eps_bits(request: &ReferenceEvaluationRequest<'_>) -> Result<u32, ReferenceOperationError> {
    let Some(CanonicalValueView::FloatBits(payload)) = request
        .attributes()
        .get(RMS_NORM_EPS_BITS_ATTRIBUTE)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    let governed =
        TypeKey::new("tiler", "f32", 1).map_err(|_| ReferenceOperationError::InvalidApplication)?;
    if payload.format() != &governed {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let bytes = <[u8; 4]>::try_from(payload.bits())
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
    Ok(u32::from_be_bytes(bytes))
}

/// Returns the governed RMS normalization reference for registration.
pub(crate) fn rms_norm_reference() -> Arc<dyn ReferenceOperation> {
    Arc::new(RmsNormF32Reference)
}

/// Returns the reference values of one dense binary32 normalization.
///
/// The evaluator's own arithmetic, reachable without assembling a tensor and a
/// request, so a corpus row states its inputs as slices — and it is the *same*
/// function the registered evaluator calls rather than a second copy of the
/// formula, because two copies would let a corpus pass against arithmetic the
/// registry never runs.
///
/// # Errors
///
/// Returns [`ReferenceOperationError`] for a non-positive or non-finite `eps`,
/// for an axis outside the shape, for a values/weights length that disagrees
/// with the shape, or for an undecided reciprocal square root.
pub fn rms_norm_f32(
    shape: &Shape,
    axis: Axis,
    eps_bits: u32,
    values: &[f32],
    weights: &[f32],
) -> Result<Vec<f32>, ReferenceOperationError> {
    let count = shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    if values.len() != count || weights.len() != count {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    normalize_dense(shape, axis, eps_bits, values, weights)
}

/// Evaluates the pinned formula over a dense row-major tensor.
fn normalize_dense(
    shape: &Shape,
    axis: Axis,
    eps_bits: u32,
    values: &[f32],
    weights: &[f32],
) -> Result<Vec<f32>, ReferenceOperationError> {
    let eps = f32::from_bits(eps_bits);
    // The refusal the semantic inferencer already states, restated here because
    // this function is reachable without the registry and a non-positive `eps`
    // would otherwise change the operation rather than fail.
    if eps <= 0.0 || !eps.is_finite() {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let geometry = RowGeometry::derive(shape, axis)?;
    let mut mapped = vec![0.0_f32; values.len()];
    for row in 0..geometry.rows {
        let scale = geometry.row_scale(values, row, eps)?;
        for position in 0..geometry.extent {
            let index = geometry.element_index(row, position);
            // Normalize first, then weight: the reference applies the weight
            // *after* the conversion back to the input dtype, which is an
            // identity at binary32 and a rounding boundary at a narrower one.
            // Writing the two multiplies in this order is what carries that
            // ordering into a profile where it becomes observable.
            let normalized = values[index] * scale;
            mapped[index] = canonicalize_arithmetic_f32(weights[index] * normalized);
        }
    }
    Ok(mapped)
}

/// Decodes a dense binary32 payload into host values.
fn decoded(elements: &[ReferenceElement]) -> Result<Vec<f32>, ReferenceOperationError> {
    elements.iter().map(decode_f32).collect()
}

#[cfg(test)]
mod tests;
