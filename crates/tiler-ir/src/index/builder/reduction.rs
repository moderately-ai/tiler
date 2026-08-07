#![allow(
    clippy::wildcard_imports,
    reason = "a private child of `builder`, not a separate concept: every name it uses is \
defined in that parent, and enumerating them would restate the parent's imports and have to \
be restated on every change"
)]

//! Scalar reduction: staging reducer results and compacting a reducer body.
//!
//! The invariant is that a reducer body is canonicalized *independently of
//! the region that contains it*, so two regions whose bodies differ only by
//! draft ordering compact to the same bytes.

use super::*;

pub(super) fn stage_reducer_results(
    owner: BuilderId,
    operation: u32,
    value_count: usize,
    result_types: Vec<ResolvedValueType>,
    structural_key: &[u8],
) -> Result<StagedReducerResults, IndexBuildError> {
    let mut staged = StagedReducerResults {
        values: Vec::with_capacity(result_types.len()),
        keys: Vec::with_capacity(result_types.len()),
        indices: Vec::with_capacity(result_types.len()),
        ids: Vec::with_capacity(result_types.len()),
    };
    for (result, value_type) in result_types.into_iter().enumerate() {
        let index = u32::try_from(value_count + staged.values.len()).map_err(|_| {
            IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::ScalarValue,
            }
        })?;
        let result =
            ScalarResultIndex::from_usize(result).ok_or(IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::ScalarValue,
            })?;
        staged.values.push(ReducerBodyValueData {
            source: ReducerBodyValueSource::OperationResult { operation, result },
            value_type,
        });
        let mut value_key = structural_key.to_vec();
        value_key.extend_from_slice(&result.get().to_be_bytes());
        staged.keys.push(Arc::new(value_key));
        staged.indices.push(index);
        staged.ids.push(ReducerScalarValueId { owner, index });
    }
    Ok(staged)
}
pub(super) fn advance_point(point: &mut [u64], extents: &[u64]) -> bool {
    for axis in (0..point.len()).rev() {
        point[axis] += 1;
        if point[axis] < extents[axis] {
            return true;
        }
        point[axis] = 0;
    }
    false
}
/// Folds one literally scaled operand into a constant and a coefficient map.
///
/// The coefficient is exact by construction: only
/// `IndexRegionBuilder::assemble_linear_combination`'s literal arm calls this,
/// and a symbolic coefficient is retained verbatim there instead.
///
/// **Distribution over a nested sum declines when that sum carries a symbolic
/// term.** Scaling `S * x` by `c` would need the coefficient `c * S`, which the
/// sourced vocabulary does not represent — it holds one literal or one declared
/// symbol, deliberately, because a composed magnitude is a relation in the
/// environment's constraint set rather than arithmetic the index layer
/// re-derives. Such an operand therefore stays opaque and takes the same path a
/// quotient does: it becomes one term scaled by `c`, which is exact, and two
/// mentions of it still merge. Nothing is approximated and nothing is lost —
/// the sum is simply not flattened.
pub(super) fn accumulate_linear_term(
    constant: &mut BigInt,
    coefficients: &mut BTreeMap<Arc<Vec<u8>>, (u32, BigInt)>,
    coefficient: &BigInt,
    value: u32,
    expressions: &[DraftIndexExpr],
) -> Result<(), IndexBuildError> {
    if coefficient.is_zero() {
        return Ok(());
    }
    let expression = &expressions[value as usize];
    let distributable = match &*expression.node {
        IndexNode::LinearCombination { terms, .. } => terms
            .iter()
            .all(|term| term.coefficient.as_literal().is_some()),
        _ => false,
    };
    match &*expression.node {
        IndexNode::Constant(inner) => {
            let product = checked_index_product(coefficient, &inner.0)?;
            checked_index_add_assign(constant, &product)?;
        }
        IndexNode::LinearCombination {
            constant: inner_constant,
            terms,
        } if distributable => {
            let product = checked_index_product(coefficient, &inner_constant.0)?;
            checked_index_add_assign(constant, &product)?;
            for term in terms {
                let inner = term
                    .coefficient
                    .as_literal()
                    .expect("a distributable sum carries only literal coefficients");
                let nested_coefficient = checked_index_product(coefficient, &inner.0)?;
                accumulate_linear_term(
                    constant,
                    coefficients,
                    &nested_coefficient,
                    term.value,
                    expressions,
                )?;
            }
        }
        _ => {
            let entry = coefficients
                .entry(Arc::clone(&expression.structural_key))
                .or_insert_with(|| (value, BigInt::zero()));
            checked_index_add_assign(&mut entry.1, coefficient)?;
        }
    }
    Ok(())
}
pub(super) fn compact_reducer_body(
    body: &ScalarReducerBodyData,
    operation_keys: &[Arc<Vec<u8>>],
    operation_depths: &[u32],
) -> ScalarReducerBodyData {
    let mut reached_values = BTreeSet::new();
    let mut reached_operations = BTreeSet::new();
    let mut stack = body.yields.clone();
    while let Some(value) = stack.pop() {
        if !reached_values.insert(value) {
            continue;
        }
        if let ReducerBodyValueSource::OperationResult { operation, .. } =
            body.values[value as usize].source
            && reached_operations.insert(operation)
        {
            let occurrence = &body.operations[operation as usize];
            stack.extend(&occurrence.operands);
            stack.extend(&occurrence.results);
        }
    }
    // Parameters are part of the body interface even when a particular body ignores them.
    for (index, value) in body.values.iter().enumerate() {
        if matches!(
            value.source,
            ReducerBodyValueSource::StateParameter(_)
                | ReducerBodyValueSource::ContributorParameter(_)
        ) {
            reached_values.insert(u32::try_from(index).expect("bounded reducer body"));
        }
    }
    let mut operation_order: Vec<_> = reached_operations.into_iter().collect();
    operation_order.sort_by_key(|operation| {
        (
            operation_depths[*operation as usize],
            operation_keys[*operation as usize].clone(),
        )
    });
    let operation_map = map_order(&operation_order);
    let mut value_order: Vec<_> = reached_values.into_iter().collect();
    value_order.sort_by_key(|value| match body.values[*value as usize].source {
        ReducerBodyValueSource::StateParameter(index) => (0, index, 0),
        ReducerBodyValueSource::ContributorParameter(index) => (1, index, 0),
        ReducerBodyValueSource::OperationResult { operation, result } => {
            (2, operation_map[&operation], result.get())
        }
    });
    let value_map = map_order(&value_order);
    let values = value_order
        .iter()
        .map(|old| {
            let value = &body.values[*old as usize];
            ReducerBodyValueData {
                source: match value.source {
                    ReducerBodyValueSource::StateParameter(index) => {
                        ReducerBodyValueSource::StateParameter(index)
                    }
                    ReducerBodyValueSource::ContributorParameter(index) => {
                        ReducerBodyValueSource::ContributorParameter(index)
                    }
                    ReducerBodyValueSource::OperationResult { operation, result } => {
                        ReducerBodyValueSource::OperationResult {
                            operation: operation_map[&operation],
                            result,
                        }
                    }
                },
                value_type: value.value_type.clone(),
            }
        })
        .collect();
    let operations = operation_order
        .iter()
        .map(|old| {
            let operation = &body.operations[*old as usize];
            ReducerBodyOperationData {
                key: operation.key.clone(),
                attributes: operation.attributes.clone(),
                operands: operation
                    .operands
                    .iter()
                    .map(|value| value_map[value])
                    .collect(),
                results: operation
                    .results
                    .iter()
                    .map(|value| value_map[value])
                    .collect(),
            }
        })
        .collect();
    ScalarReducerBodyData {
        values,
        operations,
        yields: body.yields.iter().map(|value| value_map[value]).collect(),
    }
}
pub(super) fn minimum_reducer_body(
    state: &[ScalarValueData],
    contributors: &[ScalarValueData],
) -> ScalarReducerBodyData {
    let mut values = Vec::with_capacity(state.len().saturating_add(contributors.len()));
    values.extend(
        state
            .iter()
            .enumerate()
            .map(|(index, value)| ReducerBodyValueData {
                source: ReducerBodyValueSource::StateParameter(
                    u32::try_from(index).expect("governed state count fits u32"),
                ),
                value_type: value.value_type.clone(),
            }),
    );
    values.extend(
        contributors
            .iter()
            .enumerate()
            .map(|(index, value)| ReducerBodyValueData {
                source: ReducerBodyValueSource::ContributorParameter(
                    u32::try_from(index).expect("governed contributor count fits u32"),
                ),
                value_type: value.value_type.clone(),
            }),
    );
    ScalarReducerBodyData {
        values,
        operations: Vec::new(),
        yields: (0..u32::try_from(state.len()).expect("governed state count fits u32")).collect(),
    }
}
