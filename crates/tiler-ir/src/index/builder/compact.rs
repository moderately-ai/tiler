#![allow(
    clippy::wildcard_imports,
    reason = "a private child of `builder`, not a separate concept: every name it uses is \
defined in that parent, and enumerating them would restate the parent's imports and have to \
be restated on every change"
)]

//! Canonical compaction: dropping unreachable drafts and renumbering.
//!
//! The invariant is alpha-equivalence. Two regions that differ only in the
//! order their dimensions and expressions were authored must compact to the
//! same shape, so the ordering here is derived from content keys and never
//! from draft position.

use super::*;

impl IndexRegionBuilder {
    pub(super) fn compact(
        &self,
        reachable_values: BTreeSet<u32>,
        reachable_accesses: BTreeSet<u32>,
    ) -> Result<VerifiedIndexRegion, IndexRegionDiagnostic> {
        let order = self.compaction_order(reachable_values, reachable_accesses);
        let CompactionOrder {
            dimensions: dimension_order,
            dimension_map,
            tensors: tensor_order,
            tensor_map,
            expressions: expr_order,
            expression_map: expr_map,
            accesses: access_order,
            access_map,
            operations: op_order,
            operation_map: op_map,
            values: value_order,
            value_map,
        } = order;
        let expressions: Vec<_> = expr_order
            .iter()
            .map(|old| {
                let data = &self.expressions[*old as usize];
                IndexExprData {
                    node: remap_node(&data.node, &expr_map, &dimension_map),
                    class: data.class,
                }
            })
            .collect();
        let accesses: Vec<_> = access_order
            .iter()
            .map(|old| self.remap_access(*old, &tensor_map, &expr_map, &dimension_map))
            .collect();
        let owner = self.owner.verified_owner();
        let index_domain_evidence = accesses
            .iter()
            .enumerate()
            .flat_map(|(index, access)| {
                Self::retain_access_index_domain_evidence(owner, bounded_index(index), access)
            })
            .collect();
        let operations: Vec<_> = op_order
            .iter()
            .map(|old| {
                remap_operation(
                    &self.operations[*old as usize].data,
                    &value_map,
                    &dimension_map,
                )
            })
            .collect();
        let values: Vec<_> = value_order
            .iter()
            .map(|old| {
                let value = &self.values[*old as usize];
                ScalarValueData {
                    definition: match value.definition {
                        ScalarValueDefinition::AccessRead { access } => {
                            ScalarValueDefinition::AccessRead {
                                access: access_map[&access],
                            }
                        }
                        ScalarValueDefinition::OperationResult { operation, result } => {
                            ScalarValueDefinition::OperationResult {
                                operation: op_map[&operation],
                                result,
                            }
                        }
                    },
                    value_type: value.value_type.clone(),
                    free_dimensions: value
                        .free_dimensions
                        .iter()
                        .map(|dimension| dimension_map[dimension])
                        .collect(),
                    depth: value.depth,
                }
            })
            .collect();
        let outputs = self
            .outputs
            .iter()
            .map(|output| OutputData {
                access: access_map[&output.access],
                value: value_map[&output.value],
            })
            .collect::<Vec<_>>();
        let tensors = tensor_order
            .iter()
            .map(|index| self.tensors[*index as usize].clone())
            .collect::<Vec<_>>();
        let dimensions = dimension_order
            .iter()
            .map(|index| self.dimensions[*index as usize].clone())
            .collect();
        self.finish_compaction(CompactedRegion {
            dimensions,
            tensors,
            expressions,
            accesses,
            index_domain_evidence,
            operations,
            values,
            outputs,
        })
    }

    pub(super) fn compaction_order(
        &self,
        reachable_values: BTreeSet<u32>,
        reachable_accesses: BTreeSet<u32>,
    ) -> CompactionOrder {
        let dimension_order = self.alpha_dimension_order();
        let dimension_map = map_order(&dimension_order);
        let mut tensor_order: Vec<_> = (0..bounded_index(self.tensors.len())).collect();
        tensor_order.sort_by_key(|i| (self.tensors[*i as usize].role, *i));
        let tensor_map = map_order(&tensor_order);
        let mut expr_reached = BTreeSet::new();
        for access in &reachable_accesses {
            for expr in &self.accesses[*access as usize].coordinates {
                self.mark_expr(*expr, &mut expr_reached);
            }
        }
        let mut expr_order: Vec<_> = expr_reached.into_iter().collect();
        expr_order.sort_by_key(|i| {
            (
                self.expressions[*i as usize].depth,
                self.alpha_expr_key(*i, &dimension_map),
            )
        });
        let expr_map = map_order(&expr_order);
        let mut access_order: Vec<_> = reachable_accesses.into_iter().collect();
        access_order.sort_by_key(|i| {
            let a = &self.accesses[*i as usize];
            (
                tensor_map[&a.tensor],
                a.mode,
                {
                    let mut domain = a
                        .domain
                        .iter()
                        .map(|dimension| dimension_map[dimension])
                        .collect::<Vec<_>>();
                    domain.sort_unstable();
                    domain
                },
                a.coordinates
                    .iter()
                    .map(|e| expr_map[e])
                    .collect::<Vec<_>>(),
            )
        });
        let access_map = map_order(&access_order);
        let reachable_ops: BTreeSet<_> = reachable_values
            .iter()
            .filter_map(|v| match self.values[*v as usize].definition {
                ScalarValueDefinition::OperationResult { operation, .. } => Some(operation),
                ScalarValueDefinition::AccessRead { .. } => None,
            })
            .collect();
        let alpha_operation_keys = self.alpha_operation_keys(&dimension_map);
        let mut op_order: Vec<_> = reachable_ops.into_iter().collect();
        op_order.sort_by_key(|i| {
            (
                self.operations[*i as usize].depth,
                alpha_operation_keys[*i as usize].clone(),
            )
        });
        let op_map = map_order(&op_order);
        let mut value_order: Vec<_> = reachable_values.into_iter().collect();
        value_order.sort_by_key(|i| {
            let v = &self.values[*i as usize];
            (
                v.depth,
                match v.definition {
                    ScalarValueDefinition::AccessRead { access } => (0, access_map[&access], 0),
                    ScalarValueDefinition::OperationResult { operation, result } => {
                        (1, op_map[&operation], result.get())
                    }
                },
            )
        });
        let value_map = map_order(&value_order);
        CompactionOrder {
            dimensions: dimension_order,
            dimension_map,
            tensors: tensor_order,
            tensor_map,
            expressions: expr_order,
            expression_map: expr_map,
            accesses: access_order,
            access_map,
            operations: op_order,
            operation_map: op_map,
            values: value_order,
            value_map,
        }
    }

    pub(super) fn finish_compaction(
        &self,
        compacted: CompactedRegion,
    ) -> Result<VerifiedIndexRegion, IndexRegionDiagnostic> {
        let identity_bytes = encoded_region_len(&compacted, self.sources.as_ref());
        if identity_bytes > MAX_INDEX_REGION_IDENTITY_BYTES {
            return Err(IndexRegionDiagnostic::CanonicalIdentityLimit {
                bytes: identity_bytes,
                limit: MAX_INDEX_REGION_IDENTITY_BYTES,
            });
        }
        let identity = encode_region(&compacted, self.sources.as_ref(), identity_bytes);
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
        Ok(VerifiedIndexRegion {
            data: Arc::new(VerifiedIndexRegionData {
                owner: self.owner.verified_owner(),
                sources: self.sources.clone(),
                dimensions,
                tensors,
                expressions,
                accesses,
                index_domain_evidence,
                operations,
                values,
                outputs,
                identity,
            }),
        })
    }

    pub(super) fn remap_access(
        &self,
        old: u32,
        tensor_map: &BTreeMap<u32, u32>,
        expression_map: &BTreeMap<u32, u32>,
        dimension_map: &BTreeMap<u32, u32>,
    ) -> VerifiedAccessData {
        let access = &self.accesses[old as usize];
        let shape = &self.tensors[access.tensor as usize].shape;
        let points = self.domain_points(&access.domain);
        // The retained evidence names how the access was *actually* proved, so
        // it reads the same predicates the verifier admitted it on rather than
        // a second copy of them. A copy that drifted would record an interval
        // proof for an access the verifier enumerated, or the reverse. The
        // precedence is the verifier's: interval first, the structural equality
        // argument second, enumeration last.
        let visited = points != Some(0);
        let interval = visited && self.interval_verdict(access, shape).interval_proved;
        let extent_equality =
            visited && !interval && self.coordinates_are_bounded_dimensions(access, shape);
        VerifiedAccessData {
            tensor: tensor_map[&access.tensor],
            mode: access.mode,
            domain: {
                let mut domain = access
                    .domain
                    .iter()
                    .map(|dimension| dimension_map[dimension])
                    .collect::<Vec<_>>();
                domain.sort_unstable();
                domain
            },
            coordinates: access
                .coordinates
                .iter()
                .map(|expression| expression_map[expression])
                .collect(),
            bounds_proof: if points == Some(0) {
                BoundsProof::VacuousEmptyDomain
            } else if interval {
                BoundsProof::Interval
            } else if extent_equality {
                BoundsProof::ProvedExtentEquality
            } else {
                BoundsProof::Exhaustive {
                    points: enumerated_points(points),
                }
            },
            ownership_proof: (access.mode == AccessMode::Write).then(|| {
                if self.write_is_permutation(access, shape) {
                    WriteOwnershipProof::CoordinatePermutation
                } else {
                    WriteOwnershipProof::Exhaustive {
                        points: enumerated_points(points),
                    }
                }
            }),
        }
    }

    fn retain_access_index_domain_evidence(
        owner: super::super::handles::VerifiedRegionOwner,
        access_index: u32,
        access: &VerifiedAccessData,
    ) -> Vec<DischargedIndexDomainPredicate> {
        let subject = VerifiedTensorAccessId::from_verified(owner, access_index);
        let tensor = VerifiedTensorId::from_verified(owner, access.tensor);
        let evidence = match access.bounds_proof {
            BoundsProof::VacuousEmptyDomain => {
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::VacuousEmptyDomain)
            }
            BoundsProof::Interval => {
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval)
            }
            BoundsProof::ProvedExtentEquality => {
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::ProvedExtentEquality)
            }
            BoundsProof::Exhaustive { points } => IndexDomainEvidence::ExhaustiveFinite { points },
        };
        let mut records = Vec::with_capacity(access.coordinates.len().saturating_mul(2));
        for (axis, expression) in access.coordinates.iter().copied().enumerate() {
            let expression = VerifiedIndexExprId::from_verified(owner, expression);
            for predicate in [
                IndexDomainPredicate::NonNegative { expression },
                IndexDomainPredicate::LessThanExtent {
                    expression,
                    extent: IndexExtentRef::TensorAxis {
                        tensor,
                        axis: bounded_index(axis),
                    },
                },
            ] {
                let record =
                    DischargedIndexDomainPredicate::checked(owner, subject, predicate, evidence)
                        .expect("compaction mints every evidence handle from one owner")
                        .expect("verified accesses retain only discharging evidence");
                records.push(record);
            }
        }
        records
    }

    pub(super) fn reachable_values(&self) -> BTreeSet<u32> {
        let mut reached = BTreeSet::new();
        let mut stack: Vec<_> = self.outputs.iter().map(|o| o.value).collect();
        while let Some(v) = stack.pop() {
            if !reached.insert(v) {
                continue;
            }
            if let ScalarValueDefinition::OperationResult { operation, .. } =
                self.values[v as usize].definition
            {
                let occurrence = &self.operations[operation as usize];
                stack.extend(&occurrence.operands);
                stack.extend(&occurrence.results);
            }
        }
        reached
    }
    pub(super) fn mark_expr(&self, i: u32, reached: &mut BTreeSet<u32>) {
        let mut pending = vec![i];
        while let Some(index) = pending.pop() {
            if !reached.insert(index) {
                continue;
            }
            match &*self.expressions[index as usize].node {
                IndexNode::LinearCombination { terms, .. } => {
                    pending.extend(terms.iter().map(|term| term.value));
                }
                IndexNode::FloorDiv { dividend, .. } | IndexNode::Modulo { dividend, .. } => {
                    pending.push(*dividend);
                }
                IndexNode::Constant(_) | IndexNode::Dimension(_) => {}
            }
        }
    }
    pub(super) fn alpha_dimension_order(&self) -> Vec<u32> {
        let mut order = Vec::new();
        let mut assigned = BTreeSet::new();
        let mut visited_values = BTreeSet::new();
        let mut visited_operations = BTreeSet::new();
        let mut visited_accesses = BTreeSet::new();
        let mut visited_expressions = BTreeSet::new();
        for output in &self.outputs {
            self.visit_access_dimensions(
                output.access,
                &mut order,
                &mut assigned,
                &mut visited_accesses,
                &mut visited_expressions,
            );
            self.visit_value_dimensions(
                output.value,
                &mut order,
                &mut assigned,
                &mut visited_values,
                &mut visited_operations,
                &mut visited_accesses,
                &mut visited_expressions,
            );
        }
        let mut remaining: Vec<_> = (0..bounded_index(self.dimensions.len()))
            .filter(|dimension| !assigned.contains(dimension))
            .collect();
        remaining.sort_by(|left, right| {
            let left = &self.dimensions[*left as usize];
            let right = &self.dimensions[*right as usize];
            (left.role, &left.extent).cmp(&(right.role, &right.extent))
        });
        order.extend(remaining);
        order
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn visit_value_dimensions(
        &self,
        value: u32,
        order: &mut Vec<u32>,
        assigned: &mut BTreeSet<u32>,
        visited_values: &mut BTreeSet<u32>,
        visited_operations: &mut BTreeSet<u32>,
        visited_accesses: &mut BTreeSet<u32>,
        visited_expressions: &mut BTreeSet<u32>,
    ) {
        if !visited_values.insert(value) {
            return;
        }
        let data = &self.values[value as usize];
        match data.definition {
            ScalarValueDefinition::AccessRead { access } => self.visit_access_dimensions(
                access,
                order,
                assigned,
                visited_accesses,
                visited_expressions,
            ),
            ScalarValueDefinition::OperationResult { operation, .. } => {
                if visited_operations.insert(operation) {
                    let occurrence = &self.operations[operation as usize];
                    if let ScalarOperationKindData::Reduce { dimensions, .. } = &occurrence.kind {
                        for dimension in dimensions {
                            assign_dimension(*dimension, order, assigned);
                        }
                    }
                    for operand in &occurrence.operands {
                        self.visit_value_dimensions(
                            *operand,
                            order,
                            assigned,
                            visited_values,
                            visited_operations,
                            visited_accesses,
                            visited_expressions,
                        );
                    }
                }
            }
        }
        let mut free: Vec<_> = data.free_dimensions.iter().copied().collect();
        free.sort_by(|left, right| {
            let left = &self.dimensions[*left as usize];
            let right = &self.dimensions[*right as usize];
            (left.role, &left.extent).cmp(&(right.role, &right.extent))
        });
        for dimension in free {
            assign_dimension(dimension, order, assigned);
        }
    }

    pub(super) fn visit_access_dimensions(
        &self,
        access: u32,
        order: &mut Vec<u32>,
        assigned: &mut BTreeSet<u32>,
        visited_accesses: &mut BTreeSet<u32>,
        visited_expressions: &mut BTreeSet<u32>,
    ) {
        if !visited_accesses.insert(access) {
            return;
        }
        let access = &self.accesses[access as usize];
        for coordinate in &access.coordinates {
            self.visit_expression_dimensions(*coordinate, order, assigned, visited_expressions);
        }
        let mut domain = access.domain.clone();
        domain.sort_by(|left, right| {
            let left = &self.dimensions[*left as usize];
            let right = &self.dimensions[*right as usize];
            (left.role, &left.extent).cmp(&(right.role, &right.extent))
        });
        for dimension in domain {
            assign_dimension(dimension, order, assigned);
        }
    }

    pub(super) fn visit_expression_dimensions(
        &self,
        expression: u32,
        order: &mut Vec<u32>,
        assigned: &mut BTreeSet<u32>,
        visited: &mut BTreeSet<u32>,
    ) {
        if !visited.insert(expression) {
            return;
        }
        match &*self.expressions[expression as usize].node {
            IndexNode::Dimension(dimension) => assign_dimension(*dimension, order, assigned),
            IndexNode::LinearCombination { terms, .. } => {
                let mut terms: Vec<_> = terms.iter().collect();
                terms.sort_by_key(|term| {
                    (
                        term.coefficient.clone(),
                        self.alpha_blind_expr_key(term.value),
                    )
                });
                for term in terms {
                    self.visit_expression_dimensions(term.value, order, assigned, visited);
                }
            }
            IndexNode::FloorDiv { dividend, .. } | IndexNode::Modulo { dividend, .. } => {
                self.visit_expression_dimensions(*dividend, order, assigned, visited);
            }
            IndexNode::Constant(_) => {}
        }
    }

    pub(super) fn alpha_blind_expr_key(&self, expression: u32) -> Vec<u8> {
        alpha_expr_key_impl(expression, &self.expressions, None, &self.dimensions)
    }

    pub(super) fn alpha_expr_key(
        &self,
        expression: u32,
        dimensions: &BTreeMap<u32, u32>,
    ) -> Vec<u8> {
        alpha_expr_key_impl(
            expression,
            &self.expressions,
            Some(dimensions),
            &self.dimensions,
        )
    }

    pub(super) fn alpha_operation_keys(&self, dimensions: &BTreeMap<u32, u32>) -> Vec<Vec<u8>> {
        let mut value_keys = vec![Vec::new(); self.values.len()];
        for (index, value) in self.values.iter().enumerate() {
            if let ScalarValueDefinition::AccessRead { access } = value.definition {
                value_keys[index] = self.alpha_access_key(access, dimensions);
            }
        }
        let mut operation_keys = Vec::with_capacity(self.operations.len());
        for operation in &self.operations {
            let mut key = b"tiler.index.scalar-operation.alpha.v1\0".to_vec();
            match &operation.kind {
                ScalarOperationKindData::Apply {
                    key: operation_key,
                    attributes,
                } => {
                    key.push(1);
                    encode_key(&mut key, operation_key);
                    encode_canonical(&mut key, attributes.value());
                }
                ScalarOperationKindData::Reduce {
                    dimensions: reduction_dimensions,
                    traversal,
                    init,
                    contributors,
                    body,
                } => {
                    key.push(2);
                    encode_u32s(
                        &mut key,
                        &reduction_dimensions
                            .iter()
                            .map(|dimension| dimensions[dimension])
                            .collect::<Vec<_>>(),
                    );
                    key.push(match traversal {
                        ReductionTraversal::ExactLexicographicLeftFold => 1,
                    });
                    push_len(&mut key, init.len());
                    push_len(&mut key, contributors.len());
                    encode_reducer_body(&mut key, body);
                }
            }
            push_len(&mut key, operation.operands.len());
            for operand in &operation.operands {
                push_slice(&mut key, &value_keys[*operand as usize]);
            }
            let free_dimensions: BTreeSet<_> =
                operation
                    .results
                    .first()
                    .map_or_else(BTreeSet::new, |result| {
                        self.values[*result as usize]
                            .free_dimensions
                            .iter()
                            .map(|dimension| dimensions[dimension])
                            .collect()
                    });
            encode_u32s(
                &mut key,
                &free_dimensions.iter().copied().collect::<Vec<_>>(),
            );
            for result in &operation.results {
                let ScalarValueDefinition::OperationResult {
                    result: result_index,
                    ..
                } = self.values[*result as usize].definition
                else {
                    unreachable!("operation results have operation definitions")
                };
                let mut value_key = key.clone();
                value_key.extend_from_slice(&result_index.get().to_be_bytes());
                value_keys[*result as usize] = value_key;
            }
            operation_keys.push(key);
        }
        operation_keys
    }

    pub(super) fn alpha_access_key(&self, access: u32, dimensions: &BTreeMap<u32, u32>) -> Vec<u8> {
        let data = &self.accesses[access as usize];
        let tensor = &self.tensors[data.tensor as usize];
        let role_ordinal = self.tensors[..data.tensor as usize]
            .iter()
            .filter(|candidate| candidate.role == tensor.role)
            .count();
        let mut key = b"tiler.index.access-read.alpha.v1\0".to_vec();
        key.push(match tensor.role {
            TensorRole::Input => 1,
            TensorRole::Output => 2,
        });
        push_len(&mut key, role_ordinal);
        let mut domain: Vec<_> = data
            .domain
            .iter()
            .map(|dimension| dimensions[dimension])
            .collect();
        domain.sort_unstable();
        encode_u32s(&mut key, &domain);
        push_len(&mut key, data.coordinates.len());
        for coordinate in &data.coordinates {
            push_slice(&mut key, &self.alpha_expr_key(*coordinate, dimensions));
        }
        key
    }
}
