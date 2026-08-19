//! Structural occurrences recognized as mapped reads.
//!
//! A reindex or a broadcast computes nothing: the value it produces is the value
//! it read, addressed differently. So it contributes an access relation to the
//! reading region rather than a node to the expression, and this module derives
//! that relation's per-operand-axis decodes from the family's own attribute
//! record. The derivation is a restatement of the coordinate relation the index
//! vocabulary emits, and the two are held together by the compiled result being
//! bit-compared against the reference evaluator.

use super::*;

/// Recognizes one structural occurrence as a mapped read of a leaf tensor.
///
/// Returns the leaf value and the access relation the occurrence denotes, or
/// `None` when the operation is not a structural family at all — which is the
/// caller's signal to try the elementwise projection instead. An operation that
/// *is* structural but cannot be admitted returns a typed refusal rather than
/// `None`, so a reindex this profile cannot bind never falls through to be
/// reported as an unrecognized operation set.
///
/// **The operand must already be a value this walk reads rather than computes.**
/// A direct mapped-only occurrence over a value another region would materialize
/// does not discover that boundary: on the first walk the producer result is not
/// yet a staged leaf, so it refuses under `structural-operand`. If another dense
/// occurrence first discovers the boundary, replay does make the producer result
/// a staged leaf and this function recognizes the mapped read, but
/// [`record_leaf`] then refuses it as a second read of the unordinalled
/// [`TensorRole::Intermediate`] under `structural-access-conflict`. Thus neither
/// path currently admits a structural read of a staged operand; materializing a
/// same-region computed value would additionally introduce an observable rounding
/// boundary the structural family's admission deliberately excludes.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the property that was
/// not recognized: `structural-arity`, `structural-operand` for an operand this
/// walk does not read, `structural-attributes` for a malformed or missing
/// form record, `structural-shape` for a result at another domain, and
/// `structural-relation` when the derived map is not one the region vocabulary
/// admits. A reindex over a symbolic extent is still
/// [`RequestError::UnsupportedSymbolicExtent`]. A sourced broadcast is the
/// accepted [`LogicalAccess::ParametricBroadcast`] carrier, not a folded
/// concrete neighbour.
pub(super) fn recognize_structural_read(
    program: &SemanticProgram,
    operation: &tiler_ir::semantic::OperationRef<'_>,
    leaves: &ElementwiseLeaves<'_>,
    domain: &SourcedShape,
) -> Result<Option<(ValueId, LogicalAccess)>, RequestError> {
    let reindex = operation.key() == &reindex_f32_op();
    if !reindex && operation.key() != &broadcast_f32_op() {
        return Ok(None);
    }
    let operands: Vec<ValueId> = operation.operands().collect();
    let [operand] = operands.as_slice() else {
        return mismatch("structural-arity");
    };
    if !leaves.is_leaf(*operand) {
        return mismatch("structural-operand");
    }
    let Some(operand_sourced) = sourced_shape_ref(program, *operand) else {
        return mismatch("structural-operand");
    };
    // The occurrence's result is what the region iterates, so a result at any
    // other domain would make every derived divisor address the wrong window.
    let results: Vec<ValueId> = operation.results().collect();
    let [result] = results.as_slice() else {
        return mismatch("structural-arity");
    };
    let Some(result_sourced) = sourced_shape_ref(program, *result) else {
        return mismatch("structural-shape");
    };
    if result_sourced != domain {
        return mismatch("structural-shape");
    }
    if reindex {
        let Some(operand_shape) = operand_sourced.as_static() else {
            return Err(unsupported_symbolic_extent(program, *operand, domain));
        };
        let Some(shape) = domain.as_static() else {
            return Err(unsupported_symbolic_extent(program, *result, domain));
        };
        let Some(value) = operation.attributes().get(REINDEX_MAPPING_ATTRIBUTE) else {
            return mismatch("structural-attributes");
        };
        let Ok(form) = ReindexForm::from_canonical_value(value) else {
            return mismatch("structural-attributes");
        };
        // Re-derived rather than trusted: the form must produce exactly this
        // result from exactly this operand, or the region would realize a
        // different occurrence than the one requested — the same check the
        // governed index-access lowering makes for the same reason.
        if form.result_shape(operand_shape).ok().as_ref() != Some(shape) {
            return mismatch("structural-shape");
        }
        let Some(axes) = reindex_axis_decodes(&form, operand_shape, shape) else {
            return mismatch("structural-relation");
        };
        if !tiler_ir::schedule::reindex_decodes_are_bijective(operand_shape, shape, &axes) {
            return mismatch("structural-relation");
        }
        return Ok(Some((
            *operand,
            LogicalAccess::ReindexBijection {
                operand_shape: operand_shape.clone(),
                result_shape: shape.clone(),
                axes,
            },
        )));
    }
    let Some(value) = operation.attributes().get(BROADCAST_AXIS_MAPPING_ATTRIBUTE) else {
        return mismatch("structural-attributes");
    };
    let Ok(mapping) = BroadcastAxisMapping::from_canonical_value(value) else {
        return mismatch("structural-attributes");
    };
    let declared: Vec<SourcedExtent> = mapping.result_extents().to_vec();
    let observed: Vec<SourcedExtent> = result_sourced.extents().collect();
    if declared != observed {
        return mismatch("structural-shape");
    }
    if mapping_names_a_symbol(operand_sourced, &mapping) {
        let Some(sources) = program.extent_sources() else {
            return mismatch("structural-relation");
        };
        let map = LogicalAccess::ParametricBroadcast {
            operand_shape: operand_sourced.clone(),
            mapping,
            environment: sources.environment_identity().clone(),
        };
        if !parametric_broadcast_read_is_admissible(&map, domain.rank()) {
            return mismatch("structural-relation");
        }
        if interpret_parametric_broadcast(&map, sources.environment()).is_err() {
            return mismatch("structural-relation");
        }
        return Ok(Some((*operand, map)));
    }
    let Some(operand_shape) = operand_sourced.as_static() else {
        return Err(unsupported_symbolic_extent(program, *operand, domain));
    };
    let Some(shape) = domain.as_static() else {
        return Err(unsupported_symbolic_extent(program, *result, domain));
    };
    if mapping.result_shape(operand_shape).ok().as_ref() != Some(shape) {
        return mismatch("structural-shape");
    }
    let Some(axes) = broadcast_axis_decodes(&mapping, operand_shape, shape) else {
        return mismatch("structural-relation");
    };
    // The region verifier will refuse a map that fails its admission rule, but
    // refusing here reports the *program* property rather than letting a region
    // be assembled that cannot be built. A broadcast that widens nothing lands
    // here, which is the one case a well-formed semantic mapping can reach.
    if !tiler_ir::schedule::broadcast_decodes_are_replicating(operand_shape, shape, &axes) {
        return mismatch("structural-relation");
    }
    Ok(Some((
        *operand,
        LogicalAccess::BroadcastReplication {
            operand_shape: operand_shape.clone(),
            result_shape: shape.clone(),
            axes,
        },
    )))
}

/// Returns the row-major suffix products of `shape`, one per axis.
///
/// Entry `k` is the product of every extent after axis `k`, which is the divisor
/// that extracts axis `k`'s coordinate from a row-major linear index. `None` on
/// overflow, so a derived divisor is never a wrapped one.
fn shape_suffix_products(shape: &Shape) -> Option<Vec<u64>> {
    let extents = shape.extents();
    let mut products = vec![1_u64; extents.len()];
    let mut running = 1_u64;
    for (position, extent) in extents.iter().enumerate().rev() {
        products[position] = running;
        running = running.checked_mul(extent.get())?;
    }
    Some(products)
}

/// Builds one operand axis's decode, canonicalizing an extent-one axis.
///
/// An extent-one axis has exactly one coordinate, so its divisor and mirroring
/// are unobservable — and [`AxisDecode::is_canonical`] requires them to be the
/// canonical pair, because admitting any other spelling would give one access
/// relation many identities. Routing every construction through here is what
/// makes that a property of the derivation rather than a rule each form has to
/// remember.
fn axis_decode(divisor: u64, extent: u64, mirrored: bool) -> AxisDecode {
    if extent == 1 {
        return AxisDecode::fixed();
    }
    AxisDecode {
        divisor,
        modulus: extent,
        mirrored,
    }
}

/// Derives the per-operand-axis decodes one reindex form realizes.
///
/// The physical restatement of the same coordinate relation
/// `reindex_operand_coordinates` emits into the index vocabulary, and it is a
/// restatement rather than a second derivation for a reason worth stating: the
/// index-region half is what occurrence refinement proves realizes the
/// occurrence, and this half is what the region's identity and its kernel offset
/// are built from. They are checked against each other by the compiled result
/// being bit-compared with the reference evaluator, which is the only place the
/// two can disagree and be caught.
///
/// Every form reduces to one decode per operand axis. Returns `None` when the
/// form and the shapes disagree, which the caller turns into a typed refusal
/// rather than a nearest-fit map.
fn reindex_axis_decodes(
    form: &ReindexForm,
    operand: &Shape,
    result: &Shape,
) -> Option<Vec<AxisDecode>> {
    use std::cmp::Ordering;

    let suffix = shape_suffix_products(result)?;
    let extents: Vec<u64> = operand
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    let rank = extents.len();
    let position_of = |axis: &Axis| {
        usize::try_from(axis.get())
            .ok()
            .filter(|index| *index < rank)
    };
    let mut decodes = Vec::with_capacity(rank);
    match form.kind() {
        // Result axis `k` reads operand axis `order[k]`, so operand axis
        // `order[k]` takes the window of result axis `k`. Written as a scatter
        // for the reason the index-region half is: that is the direction the
        // attribute states.
        ReindexFormKind::PermuteAxes => {
            let order = form.axes();
            if order.len() != rank {
                return None;
            }
            let mut slots: Vec<Option<AxisDecode>> = vec![None; rank];
            for (position, axis) in order.iter().enumerate() {
                let index = position_of(axis)?;
                let decode = axis_decode(*suffix.get(position)?, extents[index], false);
                if slots.get_mut(index)?.replace(decode).is_some() {
                    return None;
                }
            }
            slots.into_iter().collect()
        }
        // The split's result axes are a *contiguous* run, and contiguous
        // row-major axes linearize as one window: the run's combined coordinate
        // is the window ending at its last axis, whose extent is the operand
        // axis's own. That is why a split needs no multi-term sum here.
        ReindexFormKind::SplitAxis => {
            let axis = position_of(form.axes().first()?)?;
            let factors = form.factors().len();
            let last = axis.checked_add(factors)?.checked_sub(1)?;
            for (position, extent) in extents.iter().enumerate() {
                let divisor = match position.cmp(&axis) {
                    Ordering::Less => *suffix.get(position)?,
                    Ordering::Equal => *suffix.get(last)?,
                    Ordering::Greater => {
                        *suffix.get(position.checked_add(factors)?.checked_sub(1)?)?
                    }
                };
                decodes.push(axis_decode(divisor, *extent, false));
            }
            Some(decodes)
        }
        // The merge decodes one result coordinate back into the merged run. The
        // two-level decode collapses into one window per operand axis: the outer
        // wrap is redundant because the merged result axis's extent is the
        // product of the run, so the part the outer wrap would discard is
        // already a multiple of each inner modulus.
        ReindexFormKind::MergeAxes => {
            let merged = form.axes();
            let first = position_of(merged.first()?)?;
            let count = merged.len();
            let base = *suffix.get(first)?;
            let mut inner = vec![1_u64; count];
            let mut running = 1_u64;
            for offset in (0..count).rev() {
                inner[offset] = running;
                running = running.checked_mul(*extents.get(first.checked_add(offset)?)?)?;
            }
            for (position, extent) in extents.iter().enumerate() {
                let divisor = if position < first {
                    *suffix.get(position)?
                } else if position < first.checked_add(count)? {
                    base.checked_mul(inner[position.checked_sub(first)?])?
                } else {
                    *suffix.get(position.checked_sub(count)?.checked_add(1)?)?
                };
                decodes.push(axis_decode(divisor, *extent, false));
            }
            Some(decodes)
        }
        // The inserted result axis has extent one and no operand axis behind it,
        // so every operand axis reads the result axis one position later from
        // the insertion point onward.
        ReindexFormKind::InsertUnitAxis => {
            let inserted = usize::try_from(form.axes().first()?.get()).ok()?;
            for (position, extent) in extents.iter().enumerate() {
                let source = if position < inserted {
                    position
                } else {
                    position.checked_add(1)?
                };
                decodes.push(axis_decode(*suffix.get(source)?, *extent, false));
            }
            Some(decodes)
        }
        // The removed operand axis has extent one, so its only coordinate is
        // zero and it reads no result axis at all.
        ReindexFormKind::RemoveUnitAxis => {
            let removed = position_of(form.axes().first()?)?;
            for (position, extent) in extents.iter().enumerate() {
                let decode = match position.cmp(&removed) {
                    Ordering::Equal => AxisDecode::fixed(),
                    Ordering::Less => axis_decode(*suffix.get(position)?, *extent, false),
                    Ordering::Greater => {
                        axis_decode(*suffix.get(position.checked_sub(1)?)?, *extent, false)
                    }
                };
                decodes.push(decode);
            }
            Some(decodes)
        }
        // The shape is preserved, so every operand axis reads its own result
        // axis; the reversed one reads it mirrored. This is the one form the
        // mirror flag exists for.
        ReindexFormKind::ReverseAxis => {
            let reversed = position_of(form.axes().first()?)?;
            for (position, extent) in extents.iter().enumerate() {
                decodes.push(axis_decode(
                    *suffix.get(position)?,
                    *extent,
                    position == reversed,
                ));
            }
            Some(decodes)
        }
    }
}

/// Derives the per-operand-axis decodes one broadcast mapping realizes.
///
/// Each entry of the mapping names a *result* axis. A `FromOperand` entry gives
/// its operand axis that result axis's window; a `StretchUnit` entry names an
/// extent-one operand axis, whose decode is the canonical fixed one; and a
/// `Replicate` entry names no operand axis, which is exactly what leaves the
/// read invariant in that result axis.
fn broadcast_axis_decodes(
    mapping: &BroadcastAxisMapping,
    operand: &Shape,
    result: &Shape,
) -> Option<Vec<AxisDecode>> {
    let suffix = shape_suffix_products(result)?;
    let extents: Vec<u64> = operand
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    let sources = mapping.sources();
    if sources.len() != result.rank() {
        return None;
    }
    let mut slots: Vec<Option<AxisDecode>> = vec![None; extents.len()];
    for (position, source) in sources.iter().enumerate() {
        let Some(axis) = source.operand_axis() else {
            continue;
        };
        let index = usize::try_from(axis.get())
            .ok()
            .filter(|index| *index < extents.len())?;
        let decode = axis_decode(*suffix.get(position)?, extents[index], false);
        if slots.get_mut(index)?.replace(decode).is_some() {
            return None;
        }
    }
    slots.into_iter().collect()
}

pub(super) fn is_structural_family(key: &OpKey) -> bool {
    *key == reindex_f32_op() || *key == broadcast_f32_op()
}
