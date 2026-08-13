//! Reference semantics for the four governed structural families.
//!
//! `tiler::reindex-f32@1`, `tiler::broadcast-f32@2`, `tiler::slice-f32@1`, and
//! `tiler::concatenate-f32@1` move elements and compute nothing, so these
//! evaluators decode no floating-point value and produce none: each result
//! element is the operand element its coordinate map — or, for the concatenation,
//! its operand block — names, cloned byte for byte. The crate-wide
//! [`canonicalize_arithmetic_f32`] rule is deliberately **not** applied — it
//! exists for arithmetic that *produces* a result, and applying it here would
//! rewrite a non-canonical NaN a program only transported, which is the one thing
//! a structural family must never do.
//!
//! [`canonicalize_arithmetic_f32`]: crate::canonicalize_arithmetic_f32
//!
//! **The declared numerical conformance is not read here for that same reason,
//! and the omission is the answer rather than an oversight.** Both subnormal
//! dimensions are functions on an arithmetic site: the input dimension replaces a
//! subnormal *operand before an operation*, and the result dimension replaces a
//! subnormal value an operation *produced*. These four families perform no
//! operation, so a transported subnormal is neither. Applying a flush here would
//! make a permissive contract silently rewrite payloads a program only moved,
//! and it would do so for values no arithmetic ever touched — which is a stronger
//! claim than any target's arithmetic makes about its own registers.
//!
//! All four evaluators recompute the family's own shape rule from the attribute
//! rather than trusting the operand and result shapes the graph carries. The
//! semantic registry already refused a malformed mapping at construction, so a
//! disagreement here is invalid state rather than a caller error, and it is
//! reported as an invalid application instead of being resolved in favour of
//! either side. That recomputation is what makes the selection's bounds hold
//! here too: an occurrence whose window left its axis never reaches an operand
//! read, because the shape rule refuses before the walk begins.

use std::cmp::Ordering;

use tiler_ir::semantic::{
    BROADCAST_AXIS_MAPPING_ATTRIBUTE, BroadcastAxisMapping, BroadcastAxisSource,
    CONCATENATE_AXIS_ATTRIBUTE, F32, GATHER_AXIS_ATTRIBUTE, GatherError, OperationAttributes,
    REINDEX_MAPPING_ATTRIBUTE, ReindexForm, ReindexFormKind, SLICE_SELECTION_ATTRIBUTE,
    SliceSelection, concatenate_axis, concatenate_result_shape, decide_gather_index, gather_axis,
    gather_index_resolved_type, gather_result_shape,
};
use tiler_ir::shape::{Extent, Shape};

use super::error::{ReferenceOperationError, dense_result_error};
use super::evaluate::{decode_coordinate, f32_elements, preflight_f32_output, row_major_strides};
use super::registry::{ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs};
use super::tensor::{Tensor, TensorPayloadView};

pub(crate) struct ReindexF32Reference;

impl ReferenceOperation for ReindexF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [input] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let form = reindex_form(request.attributes())?;
        let result_shape = form
            .result_shape(input.shape())
            .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        gather(input, &result_shape, |result, operand| {
            reindex_operand_coordinate(&form, input.shape(), result, operand)
        })
        .and_then(|tensor| outputs.push(tensor))
    }
}

pub(crate) struct BroadcastF32Reference;

impl ReferenceOperation for BroadcastF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [input] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let mapping = broadcast_mapping(request.attributes())?;
        let result_shape = mapping
            .result_shape(input.shape())
            .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        gather(input, &result_shape, |result, operand| {
            for (result_axis, source) in mapping.sources().iter().enumerate() {
                let coordinate = match source {
                    // A stretched operand axis has one coordinate; a replicated
                    // result axis has no operand axis to write to at all.
                    BroadcastAxisSource::FromOperand(_) => *result
                        .get(result_axis)
                        .ok_or(ReferenceOperationError::InvalidApplication)?,
                    BroadcastAxisSource::StretchUnit(_) => 0,
                    BroadcastAxisSource::Replicate => continue,
                };
                let axis = source
                    .operand_axis()
                    .ok_or(ReferenceOperationError::InvalidApplication)?;
                *index_mut(operand, axis.get())? = coordinate;
            }
            Ok(())
        })
        .and_then(|tensor| outputs.push(tensor))
    }
}

pub(crate) struct SliceF32Reference;

impl ReferenceOperation for SliceF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [input] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let selection = slice_selection(request.attributes())?;
        let result_shape = selection
            .result_shape(input.shape())
            .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        gather(input, &result_shape, |result, operand| {
            for (axis, entry) in selection.axes().iter().enumerate() {
                // A whole axis reads its own coordinate and a window reads that
                // coordinate shifted by the offset; `offset` reports zero for the
                // former, so both are the same addition rather than two paths.
                let coordinate = *result
                    .get(axis)
                    .ok_or(ReferenceOperationError::InvalidApplication)?;
                let offset = usize::try_from(entry.offset())
                    .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
                *operand
                    .get_mut(axis)
                    .ok_or(ReferenceOperationError::InvalidApplication)? = coordinate
                    .checked_add(offset)
                    .ok_or(ReferenceOperationError::ShapeTooLarge)?;
            }
            Ok(())
        })
        .and_then(|tensor| outputs.push(tensor))
    }
}

pub(crate) struct ConcatenateF32Reference;

impl ReferenceOperation for ConcatenateF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let Some(value) = attributes.get(CONCATENATE_AXIS_ATTRIBUTE) else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let axis =
            concatenate_axis(value).map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let shapes: Vec<&Shape> = operands.iter().map(|operand| operand.shape()).collect();
        let result_shape = concatenate_result_shape(axis, &shapes)
            .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let count = result_shape
            .element_count()
            .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        preflight_f32_output(count)?;
        let position =
            usize::try_from(axis.get()).map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let extents = result_shape.extents();
        // Row-major splits the result into `outer` independent slabs, inside each
        // of which the operands' rows sit end to end. Copying whole blocks rather
        // than decoding a coordinate per element is what keeps the transport
        // exact: an element is cloned, never decoded and re-encoded, so an
        // exceptional payload arrives at the result as it left its operand.
        let outer = dense_product(extents.get(..position).unwrap_or_default())?;
        let inner = dense_product(
            extents
                .get(position.saturating_add(1)..)
                .unwrap_or_default(),
        )?;
        let mut sources = Vec::with_capacity(operands.len());
        for operand in operands.iter().copied() {
            sources.push(f32_elements(operand)?);
        }
        let mut joined = Vec::with_capacity(count);
        for slab in 0..outer {
            for (elements, shape) in sources.iter().zip(&shapes) {
                let extent = usize::try_from(shape.extents()[position].get())
                    .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
                let block = extent
                    .checked_mul(inner)
                    .ok_or(ReferenceOperationError::ShapeTooLarge)?;
                let start = slab
                    .checked_mul(block)
                    .ok_or(ReferenceOperationError::ShapeTooLarge)?;
                let end = start
                    .checked_add(block)
                    .ok_or(ReferenceOperationError::ShapeTooLarge)?;
                let chunk = elements
                    .get(start..end)
                    .ok_or(ReferenceOperationError::InvalidApplication)?;
                joined.extend(chunk.iter().cloned());
            }
        }
        Tensor::dense(F32::resolved_type(), result_shape.clone(), joined)
            .map_err(|source| dense_result_error(&source))
            .and_then(|tensor| outputs.push(tensor))
    }
}

/// Multiplies a run of extents into a host element count.
fn dense_product(extents: &[Extent]) -> Result<usize, ReferenceOperationError> {
    extents.iter().try_fold(1_usize, |product, extent| {
        usize::try_from(extent.get())
            .ok()
            .and_then(|extent| product.checked_mul(extent))
            .ok_or(ReferenceOperationError::ShapeTooLarge)
    })
}

/// Builds a result tensor by reading one operand element per result coordinate.
///
/// `map` writes the operand coordinate for one result coordinate. Every element
/// is cloned rather than decoded and re-encoded, so an exceptional payload — a
/// non-canonical NaN, a signalling NaN, a signed zero, a subnormal — arrives at
/// the result exactly as it left the operand.
fn gather(
    input: &Tensor,
    result_shape: &Shape,
    mut map: impl FnMut(&[usize], &mut [usize]) -> Result<(), ReferenceOperationError>,
) -> Result<Tensor, ReferenceOperationError> {
    let elements = f32_elements(input)?;
    let count = result_shape
        .element_count()
        .ok_or(ReferenceOperationError::ShapeTooLarge)?;
    // Refused here rather than in the broadcast evaluator alone, because this is
    // where the result is reserved and filled: `Tensor::dense` below rejects the
    // same count under the same name, but only after this function has cloned an
    // element per result coordinate. The check is defensive for the reindex
    // family, whose result is a permutation of an operand already inside the
    // bound, and load-bearing for the broadcast family, whose replicated axes
    // make the result larger than the operand it reads.
    preflight_f32_output(count)?;
    let input_strides = row_major_strides(input.shape())?;
    let result_strides = row_major_strides(result_shape)?;
    let mut result_coordinate = vec![0_usize; result_shape.rank()];
    let mut operand_coordinate = vec![0_usize; input.shape().rank()];
    let mut gathered = Vec::with_capacity(count);
    for linear in 0..count {
        decode_coordinate(
            linear,
            result_shape,
            &result_strides,
            &mut result_coordinate,
        )?;
        operand_coordinate.fill(0);
        map(&result_coordinate, &mut operand_coordinate)?;
        let mut offset = 0_usize;
        for (coordinate, stride) in operand_coordinate.iter().zip(&input_strides) {
            offset = offset
                .checked_add(
                    coordinate
                        .checked_mul(*stride)
                        .ok_or(ReferenceOperationError::ShapeTooLarge)?,
                )
                .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        }
        let element = elements
            .get(offset)
            .ok_or(ReferenceOperationError::InvalidApplication)?;
        gathered.push(element.clone());
    }
    Tensor::dense(F32::resolved_type(), result_shape.clone(), gathered)
        .map_err(|source| dense_result_error(&source))
}

/// Writes the operand coordinate one result coordinate reads, per admitted form.
fn reindex_operand_coordinate(
    form: &ReindexForm,
    input_shape: &Shape,
    result: &[usize],
    operand: &mut [usize],
) -> Result<(), ReferenceOperationError> {
    let extents = input_shape.extents();
    let at = |position: usize| -> Result<usize, ReferenceOperationError> {
        result
            .get(position)
            .copied()
            .ok_or(ReferenceOperationError::InvalidApplication)
    };
    let subject = || -> Result<usize, ReferenceOperationError> {
        form.axes()
            .first()
            .map(|axis| usize::try_from(axis.get()))
            .transpose()
            .ok()
            .flatten()
            .ok_or(ReferenceOperationError::InvalidApplication)
    };
    match form.kind() {
        ReindexFormKind::PermuteAxes => {
            for (position, axis) in form.axes().iter().enumerate() {
                *index_mut(operand, axis.get())? = at(position)?;
            }
        }
        ReindexFormKind::SplitAxis => {
            let axis = subject()?;
            let factors = form.factors();
            let mut linearized = 0_usize;
            for (position, factor) in factors.iter().enumerate() {
                let extent = usize::try_from(factor.get())
                    .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
                let coordinate = at(axis.saturating_add(position))?;
                linearized = linearized
                    .checked_mul(extent)
                    .and_then(|scaled| scaled.checked_add(coordinate))
                    .ok_or(ReferenceOperationError::ShapeTooLarge)?;
            }
            for (position, slot) in operand.iter_mut().enumerate() {
                *slot = match position.cmp(&axis) {
                    Ordering::Less => at(position)?,
                    Ordering::Equal => linearized,
                    Ordering::Greater => {
                        at(position.saturating_add(factors.len()).saturating_sub(1))?
                    }
                };
            }
        }
        ReindexFormKind::MergeAxes => {
            let first = subject()?;
            let count = form.axes().len();
            let mut linear = at(first)?;
            // Decoded innermost first, so each step peels one merged axis off the
            // low end of the linear coordinate.
            for position in (0..count).rev() {
                let extent = usize::try_from(
                    extents
                        .get(first.saturating_add(position))
                        .ok_or(ReferenceOperationError::InvalidApplication)?
                        .get(),
                )
                .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
                if extent == 0 {
                    return Err(ReferenceOperationError::InvalidApplication);
                }
                operand[first.saturating_add(position)] = linear % extent;
                linear /= extent;
            }
            let last = first.saturating_add(count);
            for (position, slot) in operand.iter_mut().enumerate() {
                if position < first {
                    *slot = at(position)?;
                } else if position >= last {
                    *slot = at(position.saturating_sub(count).saturating_add(1))?;
                }
            }
        }
        ReindexFormKind::InsertUnitAxis => {
            let inserted = subject()?;
            for (position, slot) in operand.iter_mut().enumerate() {
                *slot = if position < inserted {
                    at(position)?
                } else {
                    at(position.saturating_add(1))?
                };
            }
        }
        ReindexFormKind::RemoveUnitAxis => {
            let removed = subject()?;
            for (position, slot) in operand.iter_mut().enumerate() {
                *slot = match position.cmp(&removed) {
                    Ordering::Less => at(position)?,
                    Ordering::Equal => 0,
                    Ordering::Greater => at(position.saturating_sub(1))?,
                };
            }
        }
        ReindexFormKind::ReverseAxis => {
            let reversed = subject()?;
            let extent = usize::try_from(
                extents
                    .get(reversed)
                    .ok_or(ReferenceOperationError::InvalidApplication)?
                    .get(),
            )
            .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
            for (position, slot) in operand.iter_mut().enumerate() {
                let coordinate = at(position)?;
                *slot = if position == reversed {
                    extent
                        .checked_sub(1)
                        .and_then(|last| last.checked_sub(coordinate))
                        .ok_or(ReferenceOperationError::InvalidApplication)?
                } else {
                    coordinate
                };
            }
        }
    }
    Ok(())
}

fn index_mut(operand: &mut [usize], axis: u32) -> Result<&mut usize, ReferenceOperationError> {
    usize::try_from(axis)
        .ok()
        .and_then(|axis| operand.get_mut(axis))
        .ok_or(ReferenceOperationError::InvalidApplication)
}

fn reindex_form(attributes: &OperationAttributes) -> Result<ReindexForm, ReferenceOperationError> {
    let Some(value) = attributes.get(REINDEX_MAPPING_ATTRIBUTE) else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    if attributes.fields().len() != 1 {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    ReindexForm::from_canonical_value(value)
        .map_err(|_| ReferenceOperationError::InvalidApplication)
}

fn slice_selection(
    attributes: &OperationAttributes,
) -> Result<SliceSelection, ReferenceOperationError> {
    let Some(value) = attributes.get(SLICE_SELECTION_ATTRIBUTE) else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    if attributes.fields().len() != 1 {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    SliceSelection::from_canonical_value(value)
        .map_err(|_| ReferenceOperationError::InvalidApplication)
}

fn broadcast_mapping(
    attributes: &OperationAttributes,
) -> Result<BroadcastAxisMapping, ReferenceOperationError> {
    let Some(value) = attributes.get(BROADCAST_AXIS_MAPPING_ATTRIBUTE) else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    if attributes.fields().len() != 1 {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    BroadcastAxisMapping::from_canonical_value(value)
        .map_err(|_| ReferenceOperationError::InvalidApplication)
}

/// Reference semantics for `tiler::gather-f32@1`.
///
/// Deliberately in this module beside the four families that move elements
/// without computing, because that is the obligation it shares with them: every
/// result element is a source element *cloned*, so an exceptional payload crosses
/// a gather exactly as it left the source, and neither
/// [`canonicalize_arithmetic_f32`](crate::canonicalize_arithmetic_f32) nor the
/// declared numerical conformance is read, for the reasons this module's header
/// states once for all five.
///
/// **What it does not share with them is the whole reason the family exists**, and
/// it is visible in this evaluator's shape. The four above recompute their
/// coordinate map from an *attribute* and are total by construction, so their
/// bounds hold before a single element is read. This one reads its coordinate
/// from an operand, so it is the named enforcement boundary for a bound nothing
/// upstream can decide: each index element is checked against the gathered axis
/// as it is used, and an out-of-range value refuses under
/// [`ReferenceOperationError::GatherIndexOutOfBounds`] naming the position, the
/// value, and the extent. It is never clamped and never wrapped.
///
/// The shape rule is recomputed from the attribute here as it is for the other
/// four, so an occurrence whose axis or ranks disagree with the graph refuses
/// before any operand read begins.
pub(crate) struct GatherF32Reference;

impl ReferenceOperation for GatherF32Reference {
    fn evaluate(
        &self,
        request: ReferenceEvaluationRequest<'_>,
        outputs: &mut ReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [source, index] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let Some(value) = attributes.get(GATHER_AXIS_ATTRIBUTE) else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let axis = gather_axis(value).map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let (gathered, result_shape) = gather_result_shape(axis, source.shape(), index.shape())
            .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        let position = gathered.position();
        let source_extents = source.shape().extents();
        let gathered_extent = *source_extents
            .get(position)
            .ok_or(ReferenceOperationError::InvalidApplication)?;

        let source_elements = f32_elements(source)?;
        let coordinates = u32_elements(index)?;
        let count = result_shape
            .element_count()
            .ok_or(ReferenceOperationError::ShapeTooLarge)?;
        preflight_f32_output(count)?;

        // The result is laid out as [outer | index | inner] in row-major order,
        // where `outer` is the product of the source axes before the gathered one
        // and `inner` the product of those after it. Walking those three nested
        // runs directly is what keeps the transport exact: a source element is
        // cloned into place, never decoded and re-encoded, and the arithmetic
        // below only ever computes *offsets*.
        let outer = dense_product(source_extents.get(..position).unwrap_or_default())?;
        let inner = dense_product(
            source_extents
                .get(position.saturating_add(1)..)
                .unwrap_or_default(),
        )?;
        let row = dense_product(&[gathered_extent])?
            .checked_mul(inner)
            .ok_or(ReferenceOperationError::ShapeTooLarge)?;

        let mut gathered_elements = Vec::with_capacity(count);
        for slab in 0..outer {
            let slab_start = slab
                .checked_mul(row)
                .ok_or(ReferenceOperationError::ShapeTooLarge)?;
            for (element_position, coordinate) in coordinates.iter().enumerate() {
                // The bound is decided by the semantic layer's own rule rather
                // than restated here, so a second enforcement boundary refuses
                // under one definition instead of a second copy of it.
                let selected =
                    decide_gather_index(element_position, u64::from(*coordinate), gathered_extent)
                        .map_err(|error| match error {
                            GatherError::IndexOutOfBounds {
                                position,
                                value,
                                extent,
                            } => ReferenceOperationError::GatherIndexOutOfBounds {
                                position,
                                value,
                                extent,
                            },
                            _ => ReferenceOperationError::InvalidApplication,
                        })?;
                let start = slab_start
                    .checked_add(
                        selected
                            .checked_mul(inner)
                            .ok_or(ReferenceOperationError::ShapeTooLarge)?,
                    )
                    .ok_or(ReferenceOperationError::ShapeTooLarge)?;
                let end = start
                    .checked_add(inner)
                    .ok_or(ReferenceOperationError::ShapeTooLarge)?;
                let chunk = source_elements
                    .get(start..end)
                    .ok_or(ReferenceOperationError::InvalidApplication)?;
                gathered_elements.extend(chunk.iter().cloned());
            }
        }
        Tensor::dense(F32::resolved_type(), result_shape, gathered_elements)
            .map_err(|source| dense_result_error(&source))
            .and_then(|tensor| outputs.push(tensor))
    }
}

/// Decodes an index operand's dense `tiler::u32@1` elements.
///
/// The width is checked here as well as by the registered value validator,
/// because this function reads the bytes and a validator that had not run would
/// otherwise let a short element be interpreted as a coordinate.
fn u32_elements(tensor: &Tensor) -> Result<Vec<u32>, ReferenceOperationError> {
    if tensor.resolved_type() != &gather_index_resolved_type() {
        return Err(ReferenceOperationError::InvalidApplication);
    }
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    elements
        .iter()
        .map(|element| {
            <[u8; 4]>::try_from(element.as_bytes())
                .map(u32::from_be_bytes)
                .map_err(|_| ReferenceOperationError::InvalidApplication)
        })
        .collect()
}
