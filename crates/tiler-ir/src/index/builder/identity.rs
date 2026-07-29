#![allow(
    clippy::wildcard_imports,
    reason = "a private child of `builder`, not a separate concept: every name it uses is \
defined in that parent, and enumerating them would restate the parent's imports and have to \
be restated on every change"
)]

//! Canonical identity formation for a compacted index region.
//!
//! Structural keys, alpha-equivalence keys, node remapping, and the region
//! encoder with its exact-length companions. The invariant is the pairing:
//! `encode_region` and `encoded_region_len` must agree, and the encoder
//! asserts it, so a field added to one without the other fails there rather
//! than silently moving an identity.

use super::*;

pub(super) fn interval_linear(
    constant: &BigInt,
    terms: &[LinearTermData],
    expressions: &[DraftIndexExpr],
) -> Result<Option<(BigInt, BigInt)>, IndexBuildError> {
    let (mut minimum, mut maximum) = (constant.clone(), constant.clone());
    for term in terms {
        let Some((child_minimum, child_maximum)) =
            expressions[term.value as usize].interval.clone()
        else {
            return Ok(None);
        };
        let (term_minimum, term_maximum) = if term.coefficient.0.sign() == num_bigint::Sign::Minus {
            (
                checked_index_product(&term.coefficient.0, &child_maximum)?,
                checked_index_product(&term.coefficient.0, &child_minimum)?,
            )
        } else {
            (
                checked_index_product(&term.coefficient.0, &child_minimum)?,
                checked_index_product(&term.coefficient.0, &child_maximum)?,
            )
        };
        checked_index_add_assign(&mut minimum, &term_minimum)?;
        checked_index_add_assign(&mut maximum, &term_maximum)?;
    }
    Ok(Some((minimum, maximum)))
}
pub(super) fn structural_index_key(node: &IndexNode, expressions: &[DraftIndexExpr]) -> Vec<u8> {
    let mut output = Vec::new();
    match node {
        IndexNode::Constant(value) => {
            output.push(1);
            value.encode(&mut output);
        }
        IndexNode::Dimension(dimension) => {
            output.push(2);
            output.extend_from_slice(&dimension.to_be_bytes());
        }
        IndexNode::LinearCombination { constant, terms } => {
            output.push(3);
            constant.encode(&mut output);
            push_len(&mut output, terms.len());
            for term in terms {
                term.coefficient.encode(&mut output);
                push_slice(
                    &mut output,
                    &expressions[term.value as usize].structural_key,
                );
            }
        }
        IndexNode::FloorDiv { dividend, divisor } => {
            output.push(4);
            push_slice(&mut output, &expressions[*dividend as usize].structural_key);
            output.extend_from_slice(&divisor.to_be_bytes());
        }
        IndexNode::Modulo { dividend, divisor } => {
            output.push(5);
            push_slice(&mut output, &expressions[*dividend as usize].structural_key);
            output.extend_from_slice(&divisor.to_be_bytes());
        }
    }
    output
}
pub(super) fn access_read_key(
    data: &AccessData,
    tensors: &[TensorData],
    expressions: &[DraftIndexExpr],
) -> Vec<u8> {
    let tensor = &tensors[data.tensor as usize];
    let role_ordinal = tensors[..data.tensor as usize]
        .iter()
        .filter(|candidate| candidate.role == tensor.role)
        .count();
    let mut output = b"tiler.index.access-read.v1\0".to_vec();
    output.push(match tensor.role {
        TensorRole::Input => 1,
        TensorRole::Output => 2,
    });
    push_len(&mut output, role_ordinal);
    encode_u32s(&mut output, &data.domain);
    push_len(&mut output, data.coordinates.len());
    for coordinate in &data.coordinates {
        push_slice(
            &mut output,
            &expressions[*coordinate as usize].structural_key,
        );
    }
    output
}
pub(super) fn nested_operation_key(
    key: &ScalarOpKey,
    attributes: &ScalarAttributes,
    operands: &[u32],
    value_keys: &[Arc<Vec<u8>>],
) -> Vec<u8> {
    let mut output = b"tiler.index.reducer-apply.v2\0".to_vec();
    encode_key(&mut output, key);
    encode_canonical(&mut output, attributes.value());
    push_len(&mut output, operands.len());
    for operand in operands {
        push_slice(&mut output, &value_keys[*operand as usize]);
    }
    output
}
pub(super) fn apply_operation_key(
    key: &ScalarOpKey,
    attributes: &ScalarAttributes,
    operands: &[ScalarValueId],
    values: &[DraftScalarValue],
    free_dimensions: &BTreeSet<u32>,
) -> Vec<u8> {
    let mut output = b"tiler.index.scalar-operation.v2\0".to_vec();
    output.push(1);
    encode_key(&mut output, key);
    encode_canonical(&mut output, attributes.value());
    push_len(&mut output, operands.len());
    for operand in operands {
        push_slice(&mut output, &values[operand.as_usize()].structural_key);
    }
    encode_u32s(
        &mut output,
        &free_dimensions.iter().copied().collect::<Vec<_>>(),
    );
    output
}
pub(super) fn operation_structural_key(
    kind: &ScalarOperationKindData,
    operands: &[ScalarValueId],
    values: &[DraftScalarValue],
    free_dimensions: &BTreeSet<u32>,
) -> Vec<u8> {
    let mut output = b"tiler.index.scalar-operation.v2\0".to_vec();
    match kind {
        ScalarOperationKindData::Apply { key, attributes } => {
            output.push(1);
            encode_key(&mut output, key);
            encode_canonical(&mut output, attributes.value());
        }
        ScalarOperationKindData::Reduce {
            dimensions,
            traversal,
            body,
            init,
            contributors,
        } => {
            output.push(2);
            encode_u32s(&mut output, dimensions);
            output.push(match traversal {
                ReductionTraversal::ExactLexicographicLeftFold => 1,
            });
            push_len(&mut output, init.len());
            push_len(&mut output, contributors.len());
            encode_reducer_body(&mut output, body);
        }
    }
    push_len(&mut output, operands.len());
    for operand in operands {
        push_slice(&mut output, &values[operand.as_usize()].structural_key);
    }
    encode_u32s(
        &mut output,
        &free_dimensions.iter().copied().collect::<Vec<_>>(),
    );
    output
}
pub(super) fn encode_reducer_body(output: &mut Vec<u8>, body: &ScalarReducerBodyData) {
    push_len(output, body.values.len());
    for value in &body.values {
        match value.source {
            ReducerBodyValueSource::StateParameter(index) => {
                output.push(1);
                output.extend_from_slice(&index.to_be_bytes());
            }
            ReducerBodyValueSource::ContributorParameter(index) => {
                output.push(2);
                output.extend_from_slice(&index.to_be_bytes());
            }
            ReducerBodyValueSource::OperationResult { operation, result } => {
                output.push(3);
                output.extend_from_slice(&operation.to_be_bytes());
                output.extend_from_slice(&result.get().to_be_bytes());
            }
        }
        push_slice(output, value.value_type.canonical_encoding().as_bytes());
    }
    push_len(output, body.operations.len());
    for operation in &body.operations {
        encode_key(output, &operation.key);
        encode_canonical(output, operation.attributes.value());
        encode_u32s(output, &operation.operands);
        encode_u32s(output, &operation.results);
    }
    encode_u32s(output, &body.yields);
}
pub(super) fn assign_dimension(dimension: u32, order: &mut Vec<u32>, assigned: &mut BTreeSet<u32>) {
    if assigned.insert(dimension) {
        order.push(dimension);
    }
}

pub(super) fn alpha_expr_key_impl(
    expression: u32,
    expressions: &[DraftIndexExpr],
    dimension_map: Option<&BTreeMap<u32, u32>>,
    dimensions: &[DimensionData],
) -> Vec<u8> {
    let mut output = Vec::new();
    match &*expressions[expression as usize].node {
        IndexNode::Constant(value) => {
            output.push(1);
            value.encode(&mut output);
        }
        IndexNode::Dimension(dimension) => {
            output.push(2);
            if let Some(dimension_map) = dimension_map {
                output.extend_from_slice(&dimension_map[dimension].to_be_bytes());
            } else {
                let data = &dimensions[*dimension as usize];
                output.push(match data.role {
                    DomainRole::Parallel => 1,
                    DomainRole::Reduction => 2,
                });
                data.extent.encode(&mut output);
            }
        }
        IndexNode::LinearCombination { constant, terms } => {
            output.push(3);
            constant.encode(&mut output);
            let mut encoded_terms = terms
                .iter()
                .map(|term| {
                    let mut encoded = Vec::new();
                    term.coefficient.encode(&mut encoded);
                    push_slice(
                        &mut encoded,
                        &alpha_expr_key_impl(term.value, expressions, dimension_map, dimensions),
                    );
                    encoded
                })
                .collect::<Vec<_>>();
            encoded_terms.sort();
            push_len(&mut output, encoded_terms.len());
            for term in encoded_terms {
                push_slice(&mut output, &term);
            }
        }
        IndexNode::FloorDiv { dividend, divisor } => {
            output.push(4);
            push_slice(
                &mut output,
                &alpha_expr_key_impl(*dividend, expressions, dimension_map, dimensions),
            );
            output.extend_from_slice(&divisor.to_be_bytes());
        }
        IndexNode::Modulo { dividend, divisor } => {
            output.push(5);
            push_slice(
                &mut output,
                &alpha_expr_key_impl(*dividend, expressions, dimension_map, dimensions),
            );
            output.extend_from_slice(&divisor.to_be_bytes());
        }
    }
    output
}

pub(super) fn remap_node(
    node: &IndexNode,
    expression_map: &BTreeMap<u32, u32>,
    dimension_map: &BTreeMap<u32, u32>,
) -> IndexNode {
    match node {
        IndexNode::Constant(v) => IndexNode::Constant(v.clone()),
        IndexNode::Dimension(d) => IndexNode::Dimension(dimension_map[d]),
        IndexNode::LinearCombination { constant, terms } => IndexNode::LinearCombination {
            constant: constant.clone(),
            terms: {
                let mut terms = terms
                    .iter()
                    .map(|t| LinearTermData {
                        coefficient: t.coefficient.clone(),
                        value: expression_map[&t.value],
                    })
                    .collect::<Vec<_>>();
                terms.sort();
                terms
            },
        },
        IndexNode::FloorDiv { dividend, divisor } => IndexNode::FloorDiv {
            dividend: expression_map[dividend],
            divisor: *divisor,
        },
        IndexNode::Modulo { dividend, divisor } => IndexNode::Modulo {
            dividend: expression_map[dividend],
            divisor: *divisor,
        },
    }
}
pub(super) fn remap_operation(
    op: &ScalarOperationData,
    values: &BTreeMap<u32, u32>,
    dimension_map: &BTreeMap<u32, u32>,
) -> ScalarOperationData {
    ScalarOperationData {
        kind: match &op.kind {
            ScalarOperationKindData::Apply { key, attributes } => ScalarOperationKindData::Apply {
                key: key.clone(),
                attributes: attributes.clone(),
            },
            ScalarOperationKindData::Reduce {
                dimensions,
                traversal,
                init,
                contributors,
                body,
            } => ScalarOperationKindData::Reduce {
                dimensions: dimensions
                    .iter()
                    .map(|dimension| dimension_map[dimension])
                    .collect(),
                traversal: *traversal,
                init: init.iter().map(|v| values[v]).collect(),
                contributors: contributors.iter().map(|v| values[v]).collect(),
                body: body.clone(),
            },
        },
        operands: op.operands.iter().map(|v| values[v]).collect(),
        results: op.results.iter().map(|v| values[v]).collect(),
        depth: op.depth,
    }
}

pub(super) fn encode_region(
    compacted: &CompactedRegion,
    sources: Option<&ExtentSources>,
    exact_capacity: usize,
) -> CanonicalIndexRegionIdentity {
    let CompactedRegion {
        dimensions,
        tensors,
        expressions,
        accesses,
        index_domain_evidence,
        operations,
        values,
        outputs,
    } = compacted;
    let mut out = Vec::with_capacity(exact_capacity);
    // `v7`: every discharged index-domain predicate now enters identity with
    // its exact access subject and evidence. `v6` retained one access-wide
    // proof summary beside identity, so two consumers could rely on different
    // predicate evidence while comparing the same bytes.
    out.extend_from_slice(INDEX_REGION_DOMAIN);
    // The environment a symbolic extent resolves against is part of what this
    // region is. Two regions spelling the same symbol against differently bound
    // environments are different regions, and folding the environment's own
    // identity is what keeps them apart without re-encoding its content here.
    match sources {
        Some(sources) => {
            out.push(1);
            push_slice(&mut out, sources.environment_identity().as_bytes());
        }
        None => out.push(0),
    }
    push_len(&mut out, dimensions.len());
    for d in dimensions {
        out.push(match d.role {
            DomainRole::Parallel => 1,
            DomainRole::Reduction => 2,
        });
        d.extent.encode(&mut out);
    }
    push_len(&mut out, tensors.len());
    for t in tensors {
        out.push(match t.role {
            TensorRole::Input => 1,
            TensorRole::Output => 2,
        });
        push_slice(&mut out, t.value_type.canonical_encoding().as_bytes());
        t.shape.encode(&mut out);
    }
    push_len(&mut out, expressions.len());
    for e in expressions {
        encode_index_node(&mut out, &e.node);
    }
    push_len(&mut out, accesses.len());
    for a in accesses {
        out.push(match a.mode {
            AccessMode::Read => 1,
            AccessMode::Write => 2,
        });
        out.extend_from_slice(&a.tensor.to_be_bytes());
        encode_u32s(&mut out, &a.domain);
        encode_u32s(&mut out, &a.coordinates);
    }
    push_len(&mut out, index_domain_evidence.len());
    for evidence in index_domain_evidence {
        encode_index_domain_evidence(&mut out, *evidence);
    }
    push_len(&mut out, operations.len());
    for op in operations {
        encode_operation_kind(&mut out, &op.kind);
        encode_u32s(&mut out, &op.operands);
        encode_u32s(&mut out, &op.results);
    }
    push_len(&mut out, values.len());
    for v in values {
        match v.definition {
            ScalarValueDefinition::AccessRead { access } => {
                out.push(1);
                out.extend_from_slice(&access.to_be_bytes());
            }
            ScalarValueDefinition::OperationResult { operation, result } => {
                out.push(2);
                out.extend_from_slice(&operation.to_be_bytes());
                out.extend_from_slice(&result.get().to_be_bytes());
            }
        }
        push_slice(&mut out, v.value_type.canonical_encoding().as_bytes());
        encode_u32s(
            &mut out,
            &v.free_dimensions.iter().copied().collect::<Vec<_>>(),
        );
    }
    push_len(&mut out, outputs.len());
    for o in outputs {
        out.extend_from_slice(&o.access.to_be_bytes());
        out.extend_from_slice(&o.value.to_be_bytes());
    }
    debug_assert_eq!(out.len(), exact_capacity);
    CanonicalIndexRegionIdentity(out)
}

pub(super) fn encoded_region_len(
    compacted: &CompactedRegion,
    sources: Option<&ExtentSources>,
) -> usize {
    let CompactedRegion {
        dimensions,
        tensors,
        expressions,
        accesses,
        index_domain_evidence,
        operations,
        values,
        outputs,
    } = compacted;
    let mut bytes = INDEX_REGION_DOMAIN.len() + 1;
    if let Some(sources) = sources {
        bytes = bytes.saturating_add(encoded_bytes_len(
            sources.environment_identity().as_bytes().len(),
        ));
    }
    bytes = bytes.saturating_add(8);
    for dimension in dimensions {
        bytes = bytes
            .saturating_add(1)
            .saturating_add(dimension.extent.encoded_len());
    }
    bytes = bytes.saturating_add(8);
    for tensor in tensors {
        bytes = bytes
            .saturating_add(1)
            .saturating_add(encoded_bytes_len(
                tensor.value_type.canonical_encoding().as_bytes().len(),
            ))
            .saturating_add(tensor.shape.encoded_len());
    }
    bytes = bytes.saturating_add(8);
    for expression in expressions {
        bytes = bytes.saturating_add(encoded_index_node_len(&expression.node));
    }
    bytes = bytes.saturating_add(8);
    for access in accesses {
        bytes = bytes
            .saturating_add(5)
            .saturating_add(encoded_u32s_len(access.domain.len()))
            .saturating_add(encoded_u32s_len(access.coordinates.len()));
    }
    bytes = bytes.saturating_add(8).saturating_add(
        index_domain_evidence
            .iter()
            .map(|evidence| encoded_index_domain_evidence_len(*evidence))
            .fold(0_usize, usize::saturating_add),
    );
    bytes = bytes.saturating_add(8);
    for operation in operations {
        bytes = bytes
            .saturating_add(encoded_operation_kind_len(&operation.kind))
            .saturating_add(encoded_u32s_len(operation.operands.len()))
            .saturating_add(encoded_u32s_len(operation.results.len()));
    }
    bytes = bytes.saturating_add(8);
    for value in values {
        bytes = bytes
            .saturating_add(match value.definition {
                ScalarValueDefinition::AccessRead { .. } => 5,
                ScalarValueDefinition::OperationResult { .. } => 9,
            })
            .saturating_add(encoded_bytes_len(
                value.value_type.canonical_encoding().as_bytes().len(),
            ))
            .saturating_add(encoded_u32s_len(value.free_dimensions.len()));
    }
    bytes
        .saturating_add(8)
        .saturating_add(outputs.len().saturating_mul(8))
}

fn encode_index_domain_evidence(output: &mut Vec<u8>, record: DischargedIndexDomainPredicate) {
    output.extend_from_slice(&record.subject.index.to_be_bytes());
    match record.predicate {
        IndexDomainPredicate::NonNegative { expression } => {
            output.push(1);
            output.extend_from_slice(&expression.index.to_be_bytes());
        }
        IndexDomainPredicate::LessThanExtent { expression, extent } => {
            output.push(2);
            output.extend_from_slice(&expression.index.to_be_bytes());
            match extent {
                IndexExtentRef::Dimension(dimension) => {
                    output.push(1);
                    output.extend_from_slice(&dimension.index.to_be_bytes());
                }
                IndexExtentRef::TensorAxis { tensor, axis } => {
                    output.push(2);
                    output.extend_from_slice(&tensor.index.to_be_bytes());
                    output.extend_from_slice(&axis.to_be_bytes());
                }
            }
        }
    }
    match record.evidence {
        IndexDomainEvidence::SoundProof(proof) => {
            output.push(1);
            output.push(match proof {
                IndexDomainSoundProof::VacuousEmptyDomain => 1,
                IndexDomainSoundProof::Interval => 2,
                IndexDomainSoundProof::ProvedExtentEquality => 3,
            });
        }
        IndexDomainEvidence::ExhaustiveFinite { points } => {
            output.push(2);
            output.extend_from_slice(&points.to_be_bytes());
        }
        IndexDomainEvidence::Empirical => output.push(3),
        IndexDomainEvidence::Unknown => output.push(4),
    }
}

fn encoded_index_domain_evidence_len(record: DischargedIndexDomainPredicate) -> usize {
    let predicate = match record.predicate {
        IndexDomainPredicate::NonNegative { .. } => 5,
        IndexDomainPredicate::LessThanExtent { extent, .. } => match extent {
            IndexExtentRef::Dimension(_) => 10,
            IndexExtentRef::TensorAxis { .. } => 14,
        },
    };
    let evidence = match record.evidence {
        IndexDomainEvidence::SoundProof(_) => 2,
        IndexDomainEvidence::ExhaustiveFinite { .. } => 9,
        IndexDomainEvidence::Empirical | IndexDomainEvidence::Unknown => 1,
    };
    4_usize.saturating_add(predicate).saturating_add(evidence)
}

pub(super) fn encoded_index_node_len(node: &IndexNode) -> usize {
    match node {
        IndexNode::Constant(value) => 1 + encoded_integer_len(value),
        IndexNode::Dimension(_) => 5,
        IndexNode::LinearCombination { constant, terms } => 1_usize
            .saturating_add(encoded_integer_len(constant))
            .saturating_add(8)
            .saturating_add(
                terms
                    .iter()
                    .map(|term| encoded_integer_len(&term.coefficient).saturating_add(4))
                    .fold(0_usize, usize::saturating_add),
            ),
        IndexNode::FloorDiv { .. } | IndexNode::Modulo { .. } => 13,
    }
}

pub(super) fn encoded_integer_len(value: &IndexInteger) -> usize {
    let magnitude = usize::try_from(value.0.bits().div_ceil(8)).unwrap_or(usize::MAX);
    9_usize.saturating_add(magnitude)
}

pub(super) fn encoded_operation_kind_len(kind: &ScalarOperationKindData) -> usize {
    match kind {
        ScalarOperationKindData::Apply { key, attributes } => 1_usize
            .saturating_add(encoded_key_len(key))
            .saturating_add(attributes.value().encoded_len()),
        ScalarOperationKindData::Reduce {
            dimensions,
            init,
            contributors,
            body,
            ..
        } => 2_usize
            .saturating_add(encoded_u32s_len(dimensions.len()))
            .saturating_add(encoded_u32s_len(init.len()))
            .saturating_add(encoded_u32s_len(contributors.len()))
            .saturating_add(encoded_reducer_body_len(body)),
    }
}

pub(super) fn encoded_reducer_body_len(body: &ScalarReducerBodyData) -> usize {
    let mut bytes = 8_usize;
    for value in &body.values {
        bytes = bytes.saturating_add(encoded_reducer_value_len(value));
    }
    bytes = bytes.saturating_add(8);
    for operation in &body.operations {
        bytes = bytes.saturating_add(encoded_reducer_operation_len(operation));
    }
    bytes.saturating_add(encoded_u32s_len(body.yields.len()))
}

pub(super) fn encoded_reducer_parameter_len(value_type: &ResolvedValueType) -> usize {
    encoded_reducer_parameter_source_len().saturating_add(encoded_bytes_len(
        value_type.canonical_encoding().as_bytes().len(),
    ))
}

pub(super) fn encoded_reducer_value_len(value: &ReducerBodyValueData) -> usize {
    let source_bytes: usize = match value.source {
        ReducerBodyValueSource::StateParameter(_)
        | ReducerBodyValueSource::ContributorParameter(_) => encoded_reducer_parameter_source_len(),
        ReducerBodyValueSource::OperationResult { .. } => {
            encoded_reducer_operation_result_source_len()
        }
    };
    source_bytes.saturating_add(encoded_bytes_len(
        value.value_type.canonical_encoding().as_bytes().len(),
    ))
}

pub(super) fn encoded_reducer_operation_base_len(
    key: &ScalarOpKey,
    attributes: &ScalarAttributes,
    operand_count: usize,
) -> usize {
    encoded_key_len(key)
        .saturating_add(attributes.value().encoded_len())
        .saturating_add(encoded_u32s_len(operand_count))
        .saturating_add(8)
}

pub(super) fn encoded_reducer_operation_len(operation: &ReducerBodyOperationData) -> usize {
    encoded_reducer_operation_base_len(
        &operation.key,
        &operation.attributes,
        operation.operands.len(),
    )
    .saturating_add(operation.results.len().saturating_mul(4))
}

pub(super) const fn encoded_reducer_operation_result_overhead() -> usize {
    encoded_reducer_operation_result_source_len()
        .saturating_add(8) // Encoded-byte length prefix.
        .saturating_add(4) // Result-list index.
}

pub(super) const fn encoded_reducer_parameter_source_len() -> usize {
    5
}

pub(super) const fn encoded_reducer_operation_result_source_len() -> usize {
    9
}

pub(super) fn encoded_reducer_operation_result_increment(value_type: &ResolvedValueType) -> usize {
    value_type
        .canonical_encoding()
        .as_bytes()
        .len()
        .saturating_add(encoded_reducer_operation_result_overhead())
}

pub(super) fn encoded_key_len(key: &ScalarOpKey) -> usize {
    encoded_bytes_len(key.namespace().len())
        .saturating_add(encoded_bytes_len(key.name().len()))
        .saturating_add(4)
}

pub(super) const fn encoded_bytes_len(bytes: usize) -> usize {
    8_usize.saturating_add(bytes)
}

pub(super) const fn encoded_u32s_len(values: usize) -> usize {
    8_usize.saturating_add(values.saturating_mul(4))
}

pub(super) fn encode_index_node(out: &mut Vec<u8>, node: &IndexNode) {
    match node {
        IndexNode::Constant(v) => {
            out.push(1);
            v.encode(out);
        }
        IndexNode::Dimension(d) => {
            out.push(2);
            out.extend_from_slice(&d.to_be_bytes());
        }
        IndexNode::LinearCombination { constant, terms } => {
            out.push(3);
            constant.encode(out);
            push_len(out, terms.len());
            for t in terms {
                t.coefficient.encode(out);
                out.extend_from_slice(&t.value.to_be_bytes());
            }
        }
        IndexNode::FloorDiv { dividend, divisor } => {
            out.push(4);
            out.extend_from_slice(&dividend.to_be_bytes());
            out.extend_from_slice(&divisor.to_be_bytes());
        }
        IndexNode::Modulo { dividend, divisor } => {
            out.push(5);
            out.extend_from_slice(&dividend.to_be_bytes());
            out.extend_from_slice(&divisor.to_be_bytes());
        }
    }
}
pub(super) fn encode_operation_kind(out: &mut Vec<u8>, kind: &ScalarOperationKindData) {
    match kind {
        ScalarOperationKindData::Apply { key, attributes } => {
            out.push(1);
            encode_key(out, key);
            encode_canonical(out, attributes.value());
        }
        ScalarOperationKindData::Reduce {
            dimensions,
            traversal,
            init,
            contributors,
            body,
        } => {
            out.push(2);
            encode_u32s(out, dimensions);
            out.push(match traversal {
                ReductionTraversal::ExactLexicographicLeftFold => 1,
            });
            encode_u32s(out, init);
            encode_u32s(out, contributors);
            push_len(out, body.values.len());
            for value in &body.values {
                match value.source {
                    ReducerBodyValueSource::StateParameter(i) => {
                        out.push(1);
                        out.extend_from_slice(&i.to_be_bytes());
                    }
                    ReducerBodyValueSource::ContributorParameter(i) => {
                        out.push(2);
                        out.extend_from_slice(&i.to_be_bytes());
                    }
                    ReducerBodyValueSource::OperationResult { operation, result } => {
                        out.push(3);
                        out.extend_from_slice(&operation.to_be_bytes());
                        out.extend_from_slice(&result.get().to_be_bytes());
                    }
                }
                push_slice(out, value.value_type.canonical_encoding().as_bytes());
            }
            push_len(out, body.operations.len());
            for op in &body.operations {
                encode_key(out, &op.key);
                encode_canonical(out, op.attributes.value());
                encode_u32s(out, &op.operands);
                encode_u32s(out, &op.results);
            }
            encode_u32s(out, &body.yields);
        }
    }
}

pub(super) fn encode_u32s(out: &mut Vec<u8>, values: &[u32]) {
    push_len(out, values.len());
    for v in values {
        out.extend_from_slice(&v.to_be_bytes());
    }
}
