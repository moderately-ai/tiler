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
use crate::index::predicate::{
    encode_index_domain_subject_predicate, encode_index_domain_unknown_reason,
};

#[derive(Clone, Copy)]
enum IndexDomainAssessment {
    Discharged(DischargedIndexDomainPredicate),
    Unknown(UnknownIndexDomainPredicate),
}

/// Propagates an interval through a linear combination, or declines.
///
/// `Ok(None)` is a refusal to state a bound. A child whose own interval is
/// unknown is one cause; a symbolic coefficient the region's environment bounds
/// nowhere above is the other. An interval nothing proved would be worse than
/// `None`, because `None` falls through to a proof that either closes another
/// way or is retained as an explicit obligation under
/// `IndexDomainUnknownReason::InsufficientFacts`, whereas a fabricated bound
/// would be believed.
///
/// # Why a symbolic coefficient is bounded here rather than declined
///
/// **A proof may read this region's shape environment; a rewrite may not.**
/// The environment is part of the program this region denotes: `encode_region`
/// folds `ExtentSources::environment_identity` into the region's canonical
/// bytes, and that identity covers the environment's symbol declarations,
/// root-binding provenance, and semantic constraints. Two regions spelled
/// identically over differently constrained environments therefore have
/// different identities, and a fact read from the environment is a fact about
/// *this* region and about no other. It is not a fact about a runtime binding:
/// a `ShapeEnv` holds no values, only declarations, typed root bindings naming
/// where a value will come from, and constraints over the symbols.
///
/// What the sourced boundary keeps apart is the other operation — writing an
/// environment-derived *value* into a node. Normalization declines to fold `S *
/// x` at `S == 1`, and `SourcedIndexInteger::as_literal` answers `None` for a
/// pinned symbol, because the node's bytes must keep naming the symbol: a
/// region scaled by `b` adapts to its caller and one scaled by `4` does not,
/// and collapsing the first into the second would collapse graph identity into
/// specialized identity. Reading the symbol's declared interval leaves the node
/// untouched, so it crosses nothing.
///
/// That rule is the one the rest of this crate already runs on, and it is
/// applied here so both halves of the admitted symbolic vocabulary end on it.
/// A dimension expression's interval comes from `extent_upper_bound`, a
/// boundary axis is compared through `extent_interval`, a quotient's interval
/// comes from `determined` on its divisor, `plan_divisors` resolves a divisor
/// for enumeration, and `extents_proved_equal` decides a permutation — every
/// one of them an environment read inside a proof. The coefficient was the sole
/// exception.
///
/// # Soundness of the product
///
/// A symbolic coefficient resolves through a `ShapeSymbol`, which names an
/// extent, so its interval is nonnegative — but the *child* interval need not
/// be, and the extrema of `a * x` over a rectangle sit at its corners whatever
/// the signs. All four are computed and reduced rather than the sign of a
/// literal being read, because with `a` ranging there is no single sign to
/// read.
pub(super) fn interval_linear(
    constant: &BigInt,
    terms: &[LinearTermData],
    expressions: &[DraftIndexExpr],
    sources: Option<&ExtentSources>,
) -> Result<Option<(BigInt, BigInt)>, IndexBuildError> {
    let (mut minimum, mut maximum) = (constant.clone(), constant.clone());
    for term in terms {
        let Some((child_minimum, child_maximum)) =
            expressions[term.value as usize].interval.clone()
        else {
            return Ok(None);
        };
        let (term_minimum, term_maximum) = match &term.coefficient {
            SourcedIndexInteger::Literal(coefficient) => {
                if coefficient.0.sign() == num_bigint::Sign::Minus {
                    (
                        checked_index_product(&coefficient.0, &child_maximum)?,
                        checked_index_product(&coefficient.0, &child_minimum)?,
                    )
                } else {
                    (
                        checked_index_product(&coefficient.0, &child_minimum)?,
                        checked_index_product(&coefficient.0, &child_maximum)?,
                    )
                }
            }
            SourcedIndexInteger::Symbol(symbol) => {
                // A symbolic coefficient with no environment is unresolvable
                // rather than unconstrained. No constructor can produce one —
                // `admit_index_scalar` refuses the symbol as undeclared — so
                // this is a fail-closed floor rather than a reachable path.
                let Some(sources) = sources else {
                    return Ok(None);
                };
                let Some(bound) = sources.interval(&SourcedExtent::Symbol(symbol.clone())) else {
                    return Ok(None);
                };
                let (lower, upper) = (BigInt::from(bound.lower), BigInt::from(bound.upper));
                let low_min = checked_index_product(&lower, &child_minimum)?;
                let low_max = checked_index_product(&lower, &child_maximum)?;
                let high_min = checked_index_product(&upper, &child_minimum)?;
                let high_max = checked_index_product(&upper, &child_maximum)?;
                (
                    low_min
                        .clone()
                        .min(low_max.clone())
                        .min(high_min.clone())
                        .min(high_max.clone()),
                    low_min.max(low_max).max(high_min).max(high_max),
                )
            }
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
            for LinearTermData { coefficient, value } in terms {
                coefficient.encode(&mut output);
                push_slice(&mut output, &expressions[*value as usize].structural_key);
            }
        }
        IndexNode::FloorDiv { dividend, divisor } => {
            output.push(4);
            push_slice(&mut output, &expressions[*dividend as usize].structural_key);
            divisor.encode(&mut output);
        }
        IndexNode::Modulo { dividend, divisor } => {
            output.push(5);
            push_slice(&mut output, &expressions[*dividend as usize].structural_key);
            divisor.encode(&mut output);
        }
    }
    output
}
pub(super) fn access_read_key(
    data: &AccessData,
    tensors: &[TensorData],
    expressions: &[DraftIndexExpr],
) -> Vec<u8> {
    // Destructured with no rest pattern, here and in every encoder below, for
    // the reason `crate::numerics` gives for the same convention: a field added
    // to a framed record is then a build error at the encoder that would
    // otherwise have silently omitted it, leaving a narrower identity that
    // nothing fails on. A field bound to `_` is a field this encoder has
    // decided not to write; the comment beside it says who writes it instead.
    let boundary = |output: &mut Vec<u8>, ordinal: u32| {
        // The value type and the shape belong to the region's own tensor list,
        // which encodes them once; an access names only which boundary it
        // crosses and in what role order.
        let TensorData {
            role,
            value_type: _,
            shape: _,
        } = &tensors[ordinal as usize];
        let role_ordinal = tensors[..ordinal as usize]
            .iter()
            .filter(|candidate| candidate.role == *role)
            .count();
        output.push(match role {
            TensorRole::Input => 1,
            TensorRole::Output => 2,
        });
        push_len(output, role_ordinal);
    };
    let coordinates = |output: &mut Vec<u8>, run: &[u32]| {
        push_len(output, run.len());
        for coordinate in run {
            push_slice(output, &expressions[*coordinate as usize].structural_key);
        }
    };
    // The direct arm writes exactly the bytes it always did under exactly its
    // former domain literal, so every existing scalar value's structural key —
    // and therefore its interning and its retained byte budget — is unchanged.
    // The gather arm opens with a domain literal of its own, which is what
    // keeps a gather's F32 result from interning against a direct read of the
    // same source at the same coordinates.
    match data {
        AccessData::Direct(DirectAccessData {
            tensor,
            // Every construction site of this key passes `AccessMode::Read`;
            // the mode is fixed by the caller rather than written here.
            mode: _,
            domain,
            coordinates: run,
        }) => {
            let mut output = b"tiler.index.access-read.v1\0".to_vec();
            boundary(&mut output, *tensor);
            encode_u32s(&mut output, domain);
            coordinates(&mut output, run);
            output
        }
        AccessData::GatherRead(GatherReadAccessData {
            source,
            index,
            axis,
            domain,
            source_coordinates,
            index_coordinates,
        }) => {
            let mut output = b"tiler.index.access-gather-read.v1\0".to_vec();
            boundary(&mut output, *source);
            boundary(&mut output, *index);
            output.extend_from_slice(&axis.to_be_bytes());
            encode_u32s(&mut output, domain);
            coordinates(&mut output, source_coordinates);
            coordinates(&mut output, index_coordinates);
            output
        }
    }
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
    let ScalarReducerBodyData {
        values,
        operations,
        yields,
    } = body;
    push_len(output, values.len());
    for ReducerBodyValueData { source, value_type } in values {
        match source {
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
        push_slice(output, value_type.canonical_encoding().as_bytes());
    }
    push_len(output, operations.len());
    for ReducerBodyOperationData {
        key,
        attributes,
        operands,
        results,
    } in operations
    {
        encode_key(output, key);
        encode_canonical(output, attributes.value());
        encode_u32s(output, operands);
        encode_u32s(output, results);
    }
    encode_u32s(output, yields);
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
                let DimensionData { role, extent } = &dimensions[*dimension as usize];
                output.push(match role {
                    DomainRole::Parallel => 1,
                    DomainRole::Reduction => 2,
                });
                extent.encode(&mut output);
            }
        }
        IndexNode::LinearCombination { constant, terms } => {
            output.push(3);
            constant.encode(&mut output);
            let mut encoded_terms = terms
                .iter()
                .map(|LinearTermData { coefficient, value }| {
                    let mut encoded = Vec::new();
                    coefficient.encode(&mut encoded);
                    push_slice(
                        &mut encoded,
                        &alpha_expr_key_impl(*value, expressions, dimension_map, dimensions),
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
            divisor.encode(&mut output);
        }
        IndexNode::Modulo { dividend, divisor } => {
            output.push(5);
            push_slice(
                &mut output,
                &alpha_expr_key_impl(*dividend, expressions, dimension_map, dimensions),
            );
            divisor.encode(&mut output);
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
            divisor: divisor.clone(),
        },
        IndexNode::Modulo { dividend, divisor } => IndexNode::Modulo {
            dividend: expression_map[dividend],
            divisor: divisor.clone(),
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
        unknown_index_domain_predicates,
        operations,
        values,
        outputs,
    } = compacted;
    let mut out = Vec::with_capacity(exact_capacity);
    // `v8`: discharged and unresolved index-domain predicates share one
    // canonical assessment sequence. `v7` encoded only discharged evidence,
    // so a region's residual semantic obligations were absent from identity.
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
    for DimensionData { role, extent } in dimensions {
        out.push(match role {
            DomainRole::Parallel => 1,
            DomainRole::Reduction => 2,
        });
        extent.encode(&mut out);
    }
    push_len(&mut out, tensors.len());
    for TensorData {
        role,
        value_type,
        shape,
    } in tensors
    {
        out.push(match role {
            TensorRole::Input => 1,
            TensorRole::Output => 2,
        });
        push_slice(&mut out, value_type.canonical_encoding().as_bytes());
        shape.encode(&mut out);
    }
    push_len(&mut out, expressions.len());
    // `class` is derived from the node graph and the environment a symbolic
    // divisor or coefficient resolves against — `join`ed up from the children
    // in `assemble_linear_combination` and `div_mod` — and both of those are
    // already in these bytes, so it is not independent content.
    for IndexExprData { node, class: _ } in expressions {
        encode_index_node(&mut out, node);
    }
    push_len(&mut out, accesses.len());
    for a in accesses {
        match a {
            // `0x01` and `0x02` keep their tags and their exact field layouts,
            // so no previously encodable region's bytes move and
            // `tiler.index-region.v11` deliberately does not step.
            CompactedAccess::Direct(VerifiedDirectAccessData {
                tensor,
                mode,
                domain,
                coordinates,
                // Retained analysis metadata rather than region content: the
                // discharged and unresolved predicates are encoded once, in
                // the assessment sequence below, and
                // `retained_ownership_proof_metadata_does_not_enter_region_identity`
                // pins that these three stay out of the bytes.
                bounds_proof: _,
                bounds_facts: _,
                ownership_proof: _,
            }) => {
                out.push(match mode {
                    AccessMode::Read => 1,
                    AccessMode::Write => 2,
                });
                out.extend_from_slice(&tensor.to_be_bytes());
                encode_u32s(&mut out, domain);
                encode_u32s(&mut out, coordinates);
            }
            // Appended at the next free value. A reader that reaches `0x03` is
            // reading an access the earlier vocabulary could not express, never
            // an earlier access under a new interpretation. The two tensor
            // ordinals and the two coordinate runs are written in a fixed
            // order, each run length-framed, so swapping a source for an index
            // or dropping the axis frame changes the bytes.
            CompactedAccess::GatherRead(gather) => {
                let CompactedGatherReadAccess {
                    source,
                    index,
                    axis,
                    domain,
                    source_coordinates,
                    index_coordinates,
                } = &**gather;
                out.push(3);
                out.extend_from_slice(&source.to_be_bytes());
                out.extend_from_slice(&index.to_be_bytes());
                out.extend_from_slice(&axis.to_be_bytes());
                encode_u32s(&mut out, domain);
                encode_u32s(&mut out, source_coordinates);
                encode_u32s(&mut out, index_coordinates);
            }
        }
    }
    let mut assessments = index_domain_evidence
        .iter()
        .copied()
        .map(IndexDomainAssessment::Discharged)
        .chain(
            unknown_index_domain_predicates
                .iter()
                .copied()
                .map(IndexDomainAssessment::Unknown),
        )
        .collect::<Vec<_>>();
    assessments.sort_by_key(index_domain_assessment_key);
    push_len(&mut out, assessments.len());
    for assessment in assessments {
        encode_index_domain_assessment(&mut out, assessment);
    }
    push_len(&mut out, operations.len());
    // `depth`, here and on a value below, is `max(operand depth) + 1`, kept so
    // the builder can enforce `MAX_SCALAR_EXPRESSION_DEPTH` without a walk. It
    // is a function of the operand graph these bytes already carry.
    for ScalarOperationData {
        kind,
        operands,
        results,
        depth: _,
    } in operations
    {
        encode_operation_kind(&mut out, kind);
        encode_u32s(&mut out, operands);
        encode_u32s(&mut out, results);
    }
    push_len(&mut out, values.len());
    for ScalarValueData {
        definition,
        value_type,
        free_dimensions,
        depth: _,
    } in values
    {
        match definition {
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
        push_slice(&mut out, value_type.canonical_encoding().as_bytes());
        encode_u32s(
            &mut out,
            &free_dimensions.iter().copied().collect::<Vec<_>>(),
        );
    }
    push_len(&mut out, outputs.len());
    for OutputData { access, value } in outputs {
        out.extend_from_slice(&access.to_be_bytes());
        out.extend_from_slice(&value.to_be_bytes());
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
        unknown_index_domain_predicates,
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
    // Every loop below destructures the same record its `encode_region` twin
    // frames, and binds the same fields to `_`, so the two halves of the pair
    // are forced to change together: a field added to one of these records is
    // a build error in both functions, not just the one that writes bytes.
    bytes = bytes.saturating_add(8);
    for DimensionData { role: _, extent } in dimensions {
        // The role is the one tag byte.
        bytes = bytes.saturating_add(1).saturating_add(extent.encoded_len());
    }
    bytes = bytes.saturating_add(8);
    for TensorData {
        role: _,
        value_type,
        shape,
    } in tensors
    {
        bytes = bytes
            .saturating_add(1)
            .saturating_add(encoded_bytes_len(
                value_type.canonical_encoding().as_bytes().len(),
            ))
            .saturating_add(shape.encoded_len());
    }
    bytes = bytes.saturating_add(8);
    for IndexExprData { node, class: _ } in expressions {
        bytes = bytes.saturating_add(encoded_index_node_len(node));
    }
    bytes = bytes.saturating_add(8);
    for access in accesses {
        bytes = match access {
            // The mode tag and the tensor ordinal are the fixed five.
            CompactedAccess::Direct(VerifiedDirectAccessData {
                tensor: _,
                mode: _,
                domain,
                coordinates,
                bounds_proof: _,
                bounds_facts: _,
                ownership_proof: _,
            }) => bytes
                .saturating_add(5)
                .saturating_add(encoded_u32s_len(domain.len()))
                .saturating_add(encoded_u32s_len(coordinates.len())),
            // Tag, two tensor ordinals, and the axis, then three framed runs.
            CompactedAccess::GatherRead(gather) => {
                let CompactedGatherReadAccess {
                    source: _,
                    index: _,
                    axis: _,
                    domain,
                    source_coordinates,
                    index_coordinates,
                } = &**gather;
                bytes
                    .saturating_add(13)
                    .saturating_add(encoded_u32s_len(domain.len()))
                    .saturating_add(encoded_u32s_len(source_coordinates.len()))
                    .saturating_add(encoded_u32s_len(index_coordinates.len()))
            }
        };
    }
    bytes = bytes.saturating_add(8).saturating_add(
        index_domain_evidence
            .iter()
            .copied()
            .map(IndexDomainAssessment::Discharged)
            .chain(
                unknown_index_domain_predicates
                    .iter()
                    .copied()
                    .map(IndexDomainAssessment::Unknown),
            )
            .map(encoded_index_domain_assessment_len)
            .fold(0_usize, usize::saturating_add),
    );
    bytes = bytes.saturating_add(8);
    for ScalarOperationData {
        kind,
        operands,
        results,
        depth: _,
    } in operations
    {
        bytes = bytes
            .saturating_add(encoded_operation_kind_len(kind))
            .saturating_add(encoded_u32s_len(operands.len()))
            .saturating_add(encoded_u32s_len(results.len()));
    }
    bytes = bytes.saturating_add(8);
    for ScalarValueData {
        definition,
        value_type,
        free_dimensions,
        depth: _,
    } in values
    {
        bytes = bytes
            .saturating_add(match definition {
                // Tag plus the access ordinal; tag, operation ordinal and
                // result index.
                ScalarValueDefinition::AccessRead { access: _ } => 5,
                ScalarValueDefinition::OperationResult {
                    operation: _,
                    result: _,
                } => 9,
            })
            .saturating_add(encoded_bytes_len(
                value_type.canonical_encoding().as_bytes().len(),
            ))
            .saturating_add(encoded_u32s_len(free_dimensions.len()));
    }
    // Walked rather than multiplied by a hand-written width, so that a field
    // added to `OutputData` is a build error on this side of the pair too.
    outputs
        .iter()
        .map(
            |OutputData {
                 access: _,
                 value: _,
             }| 8_usize,
        )
        .fold(bytes.saturating_add(8), usize::saturating_add)
}

fn index_domain_assessment_key(
    assessment: &IndexDomainAssessment,
) -> (VerifiedTensorAccessId, IndexDomainPredicate) {
    // The sort key is the subject and the predicate, which together name the
    // obligation; the evidence and the reason say how it was answered and must
    // not reorder it. Bound explicitly so a widened record has to say here
    // whether it participates in the canonical order.
    match assessment {
        IndexDomainAssessment::Discharged(DischargedIndexDomainPredicate {
            subject,
            predicate,
            evidence: _,
            facts: _,
        })
        | IndexDomainAssessment::Unknown(UnknownIndexDomainPredicate {
            subject,
            predicate,
            reason: _,
        }) => (*subject, *predicate),
    }
}

fn encode_index_domain_assessment(output: &mut Vec<u8>, assessment: IndexDomainAssessment) {
    match assessment {
        IndexDomainAssessment::Discharged(DischargedIndexDomainPredicate {
            subject,
            predicate,
            evidence,
            facts,
        }) => {
            encode_index_domain_subject_predicate(output, subject, predicate);
            output.push(1);
            match evidence {
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
            // Which facts the argument above was allowed to read, encoded for
            // every discharged record rather than only the environment-reading
            // ones, so the slot is fixed-width and a reader never has to know
            // the evidence to know how many bytes follow it.
            output.push(facts.tag());
        }
        IndexDomainAssessment::Unknown(UnknownIndexDomainPredicate {
            subject,
            predicate,
            reason,
        }) => {
            encode_index_domain_subject_predicate(output, subject, predicate);
            output.push(2);
            encode_index_domain_unknown_reason(output, reason);
        }
    }
}

fn encoded_index_domain_subject_predicate_len(predicate: IndexDomainPredicate) -> usize {
    // A tag byte plus the constrained expression's four, then the extent
    // reference's own tag and the one or two ids under it. Bound with no rest
    // pattern for the same reason as the records above: a widened variant must
    // be a build error on the sizing side, not a count that silently stops
    // matching what `encode_index_domain_subject_predicate` writes.
    let predicate = match predicate {
        IndexDomainPredicate::NonNegative { expression: _ } => 5,
        IndexDomainPredicate::LessThanExtent {
            expression: _,
            extent,
        } => match extent {
            IndexExtentRef::Dimension(_) => 10,
            IndexExtentRef::TensorAxis { tensor: _, axis: _ } => 14,
        },
    };
    4_usize.saturating_add(predicate)
}

fn encoded_index_domain_assessment_len(assessment: IndexDomainAssessment) -> usize {
    // The subject is the fixed four bytes inside
    // `encoded_index_domain_subject_predicate_len`, and the fact-source tag is
    // the trailing one; both are accounted for without being read here.
    match assessment {
        IndexDomainAssessment::Discharged(DischargedIndexDomainPredicate {
            subject: _,
            predicate,
            evidence,
            facts: _,
        }) => {
            let evidence = match evidence {
                IndexDomainEvidence::SoundProof(_) => 2,
                IndexDomainEvidence::ExhaustiveFinite { points: _ } => 9,
                IndexDomainEvidence::Empirical | IndexDomainEvidence::Unknown => 1,
            };
            encoded_index_domain_subject_predicate_len(predicate)
                .saturating_add(1)
                .saturating_add(evidence)
                // The fact-source tag `encode_index_domain_assessment` appends.
                .saturating_add(1)
        }
        IndexDomainAssessment::Unknown(UnknownIndexDomainPredicate {
            subject: _,
            predicate,
            reason,
        }) => {
            let reason = match reason {
                IndexDomainUnknownReason::InsufficientFacts
                | IndexDomainUnknownReason::UnsupportedFragment => 1,
                IndexDomainUnknownReason::ResourceLimit {
                    resource: _,
                    required: _,
                    limit: _,
                } => 26,
            };
            encoded_index_domain_subject_predicate_len(predicate)
                .saturating_add(1)
                .saturating_add(reason)
        }
    }
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
                    // Read from the coefficient rather than fixed at an
                    // integer's width, for the reason the divisor's own length
                    // gives: a symbolic one carries its scope and name.
                    .map(
                        |LinearTermData {
                             coefficient,
                             value: _,
                         }| { coefficient.encoded_len().saturating_add(4) },
                    )
                    .fold(0_usize, usize::saturating_add),
            ),
        // One tag byte, the dividend's index, and the divisor's own tagged
        // encoding — read from the divisor rather than fixed at eight bytes,
        // because a symbolic one carries its scope and name.
        IndexNode::FloorDiv {
            dividend: _,
            divisor,
        }
        | IndexNode::Modulo {
            dividend: _,
            divisor,
        } => 5_usize.saturating_add(divisor.encoded_len()),
    }
}

pub(super) fn encoded_integer_len(value: &IndexInteger) -> usize {
    value.encoded_len()
}

pub(super) fn encoded_operation_kind_len(kind: &ScalarOperationKindData) -> usize {
    match kind {
        ScalarOperationKindData::Apply { key, attributes } => 1_usize
            .saturating_add(encoded_key_len(key))
            .saturating_add(attributes.value().encoded_len()),
        // The tag and the traversal tag are the fixed two.
        ScalarOperationKindData::Reduce {
            dimensions,
            traversal: _,
            init,
            contributors,
            body,
        } => 2_usize
            .saturating_add(encoded_u32s_len(dimensions.len()))
            .saturating_add(encoded_u32s_len(init.len()))
            .saturating_add(encoded_u32s_len(contributors.len()))
            .saturating_add(encoded_reducer_body_len(body)),
    }
}

pub(super) fn encoded_reducer_body_len(body: &ScalarReducerBodyData) -> usize {
    let ScalarReducerBodyData {
        values,
        operations,
        yields,
    } = body;
    let mut bytes = 8_usize;
    for value in values {
        bytes = bytes.saturating_add(encoded_reducer_value_len(value));
    }
    bytes = bytes.saturating_add(8);
    for operation in operations {
        bytes = bytes.saturating_add(encoded_reducer_operation_len(operation));
    }
    bytes.saturating_add(encoded_u32s_len(yields.len()))
}

pub(super) fn encoded_reducer_parameter_len(value_type: &ResolvedValueType) -> usize {
    encoded_reducer_parameter_source_len().saturating_add(encoded_bytes_len(
        value_type.canonical_encoding().as_bytes().len(),
    ))
}

pub(super) fn encoded_reducer_value_len(value: &ReducerBodyValueData) -> usize {
    let ReducerBodyValueData { source, value_type } = value;
    let source_bytes: usize = match source {
        ReducerBodyValueSource::StateParameter(_)
        | ReducerBodyValueSource::ContributorParameter(_) => encoded_reducer_parameter_source_len(),
        ReducerBodyValueSource::OperationResult {
            operation: _,
            result: _,
        } => encoded_reducer_operation_result_source_len(),
    };
    source_bytes.saturating_add(encoded_bytes_len(
        value_type.canonical_encoding().as_bytes().len(),
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
    let ReducerBodyOperationData {
        key,
        attributes,
        operands,
        results,
    } = operation;
    encoded_reducer_operation_base_len(key, attributes, operands.len())
        .saturating_add(results.len().saturating_mul(4))
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
            for LinearTermData { coefficient, value } in terms {
                coefficient.encode(out);
                out.extend_from_slice(&value.to_be_bytes());
            }
        }
        IndexNode::FloorDiv { dividend, divisor } => {
            out.push(4);
            out.extend_from_slice(&dividend.to_be_bytes());
            divisor.encode(out);
        }
        IndexNode::Modulo { dividend, divisor } => {
            out.push(5);
            out.extend_from_slice(&dividend.to_be_bytes());
            divisor.encode(out);
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
            let ScalarReducerBodyData {
                values,
                operations,
                yields,
            } = body;
            push_len(out, values.len());
            for ReducerBodyValueData { source, value_type } in values {
                match source {
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
                push_slice(out, value_type.canonical_encoding().as_bytes());
            }
            push_len(out, operations.len());
            for ReducerBodyOperationData {
                key,
                attributes,
                operands,
                results,
            } in operations
            {
                encode_key(out, key);
                encode_canonical(out, attributes.value());
                encode_u32s(out, operands);
                encode_u32s(out, results);
            }
            encode_u32s(out, yields);
        }
    }
}

pub(super) fn encode_u32s(out: &mut Vec<u8>, values: &[u32]) {
    push_len(out, values.len());
    for v in values {
        out.extend_from_slice(&v.to_be_bytes());
    }
}
