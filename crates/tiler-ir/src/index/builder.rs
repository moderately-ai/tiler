use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use std::sync::Arc;

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Signed, ToPrimitive, Zero};

use crate::semantic::ResolvedValueType;
use crate::shape::{Extent, Shape};

use super::error::invalid_handle;
use super::handles::{BuilderId, next_builder_id};
use super::model::{
    AccessData, BoundsProof, DimensionData, IndexExprData, IndexNode, LinearTermData, OutputData,
    ReducerBodyOperationData, ReducerBodyValueData, ReducerBodyValueSource, ScalarOperationData,
    ScalarOperationKindData, ScalarReducerBodyData, ScalarValueData, ScalarValueDefinition,
    TensorData, VerifiedAccessData, VerifiedIndexRegionData, WriteOwnershipProof,
};
use super::scalar::{
    ScalarApplyError, ScalarInferenceCapacity, ScalarInferenceHostFailure, encode_bytes,
    encode_canonical, encode_key, encode_len,
};
use super::{
    AccessMode, CanonicalIndexRegionIdentity, DimensionId, DomainRole, FrozenScalarRegistry,
    IndexBuildError, IndexEntityKind, IndexExprClass, IndexExprId, IndexInteger, IndexLimitKind,
    IndexRegionBuildError, IndexRegionDiagnostic, MAX_ACCESS_CANONICAL_BYTES,
    MAX_BOUNDARY_CANONICAL_BYTES, MAX_BOUNDARY_TENSORS, MAX_DOMAIN_DIMENSIONS,
    MAX_EXHAUSTIVE_PROOF_BYTES, MAX_EXHAUSTIVE_PROOF_CELLS, MAX_INDEX_CANONICAL_BYTES,
    MAX_INDEX_EXPRESSION_DEPTH, MAX_INDEX_EXPRESSION_OPERANDS, MAX_INDEX_EXPRESSIONS,
    MAX_INDEX_INTEGER_BYTES, MAX_INDEX_REGION_IDENTITY_BYTES, MAX_OUTPUT_ROOTS,
    MAX_SCALAR_CANONICAL_BYTES, MAX_SCALAR_EXPRESSION_DEPTH, MAX_SCALAR_EXPRESSIONS,
    MAX_SCALAR_OPERANDS, MAX_TENSOR_ACCESSES, MAX_TENSOR_RANK, ProofResource, ReductionTraversal,
    ScalarAttributes, ScalarOpKey, ScalarOperationId, ScalarResultIndex, ScalarValueId,
    TensorAccessId, TensorId, TensorRole, VerifiedIndexRegion,
};

#[derive(Clone, Debug)]
struct DraftIndexExpr {
    node: Arc<IndexNode>,
    structural_key: Arc<Vec<u8>>,
    dimensions: BTreeSet<u32>,
    class: IndexExprClass,
    interval: Option<(BigInt, BigInt)>,
    depth: u32,
}

#[derive(Clone, Debug)]
struct DraftScalarValue {
    data: ScalarValueData,
    structural_key: Arc<Vec<u8>>,
}
impl Deref for DraftScalarValue {
    type Target = ScalarValueData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Clone, Debug)]
struct DraftScalarOperation {
    data: ScalarOperationData,
}

struct StagedReducerResults {
    values: Vec<ReducerBodyValueData>,
    keys: Vec<Arc<Vec<u8>>>,
    indices: Vec<u32>,
    ids: Vec<ReducerScalarValueId>,
}

struct CompactionOrder {
    dimensions: Vec<u32>,
    dimension_map: BTreeMap<u32, u32>,
    tensors: Vec<u32>,
    tensor_map: BTreeMap<u32, u32>,
    expressions: Vec<u32>,
    expression_map: BTreeMap<u32, u32>,
    accesses: Vec<u32>,
    access_map: BTreeMap<u32, u32>,
    operations: Vec<u32>,
    operation_map: BTreeMap<u32, u32>,
    values: Vec<u32>,
    value_map: BTreeMap<u32, u32>,
}

struct CompactedRegion {
    dimensions: Vec<DimensionData>,
    tensors: Vec<TensorData>,
    expressions: Vec<IndexExprData>,
    accesses: Vec<VerifiedAccessData>,
    operations: Vec<ScalarOperationData>,
    values: Vec<ScalarValueData>,
    outputs: Vec<OutputData>,
}

struct ReductionInputs {
    bound: BTreeSet<u32>,
    init: Vec<ScalarValueData>,
    contributors: Vec<ScalarValueData>,
    free: BTreeSet<u32>,
    body_budget: ReducerBodyBudget,
}

#[derive(Clone, Copy, Debug)]
struct ReducerBodyBudget {
    parent_bytes_without_body: usize,
    body_multiplier: usize,
    maximum_encoded_bytes: usize,
}

impl ReducerBodyBudget {
    fn parent_bytes(self, encoded_body_bytes: usize) -> usize {
        self.parent_bytes_without_body
            .saturating_add(encoded_body_bytes.saturating_mul(self.body_multiplier))
    }

    fn admit(self, encoded_body_bytes: usize) -> Result<(), IndexBuildError> {
        if encoded_body_bytes > self.maximum_encoded_bytes {
            return Err(IndexBuildError::StructuralLimit {
                resource: IndexLimitKind::ScalarCanonicalBytes,
                actual: self.parent_bytes(encoded_body_bytes) as u128,
                limit: MAX_SCALAR_CANONICAL_BYTES as u128,
            });
        }
        Ok(())
    }
}

fn admit_reducer_body_append<T>(
    budget: ReducerBodyBudget,
    encoded_body_bytes: usize,
    commit: impl FnOnce() -> T,
) -> Result<T, IndexBuildError> {
    budget.admit(encoded_body_bytes)?;
    Ok(commit())
}
impl Deref for DraftScalarOperation {
    type Target = ScalarOperationData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Ordered scalar results returned by one operation occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarResults(Vec<ScalarValueId>);
impl ScalarResults {
    /// Returns the number of inferred results.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Returns whether no result exists.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Returns one result.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<ScalarValueId> {
        self.0.get(index).copied()
    }
    /// Iterates ordered results.
    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = ScalarValueId> + '_ {
        self.0.iter().copied()
    }
}

/// Nested, capture-free typed SSA builder for one reduction step.
pub struct ScalarReducerBodyBuilder<'a> {
    registry: &'a FrozenScalarRegistry,
    owner: BuilderId,
    values: Vec<ReducerBodyValueData>,
    value_keys: Vec<Arc<Vec<u8>>>,
    value_depths: Vec<u32>,
    operations: Vec<ReducerBodyOperationData>,
    operation_keys: Vec<Arc<Vec<u8>>>,
    operation_depths: Vec<u32>,
    operation_intern: BTreeMap<Arc<Vec<u8>>, u32>,
    canonical_bytes: usize,
    encoded_body_bytes: usize,
    parent_budget: ReducerBodyBudget,
    state_count: usize,
    contributor_count: usize,
    yields: Option<Vec<u32>>,
}

/// Builder-owned scalar value inside a reducer body.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReducerScalarValueId {
    owner: BuilderId,
    index: u32,
}

/// Ordered nested-body results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReducerScalarResults(Vec<ReducerScalarValueId>);
impl ReducerScalarResults {
    /// Returns one result.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<ReducerScalarValueId> {
        self.0.get(index).copied()
    }
}

impl<'a> ScalarReducerBodyBuilder<'a> {
    fn new(
        registry: &'a FrozenScalarRegistry,
        state: &[ResolvedValueType],
        contributors: &[ResolvedValueType],
        parent_budget: ReducerBodyBudget,
    ) -> Result<Self, IndexBuildError> {
        limit(
            state.len().saturating_add(contributors.len()),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarValues,
        )?;
        let parameter_bytes = state
            .iter()
            .chain(contributors)
            .try_fold(0_usize, |bytes, value_type| {
                bytes.checked_add(encoded_reducer_parameter_len(value_type))
            })
            .unwrap_or(usize::MAX);
        limit(
            parameter_bytes,
            MAX_SCALAR_CANONICAL_BYTES,
            IndexLimitKind::ScalarCanonicalBytes,
        )?;
        let owner = next_builder_id().ok_or(IndexBuildError::BuilderIdentityExhausted)?;
        let mut values = Vec::with_capacity(state.len() + contributors.len());
        let mut value_keys = Vec::with_capacity(state.len() + contributors.len());
        for (i, value_type) in state.iter().cloned().enumerate() {
            let index = u32::try_from(i).map_err(|_| IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::ScalarValue,
            })?;
            let mut key = vec![1];
            key.extend_from_slice(&index.to_be_bytes());
            encode_bytes(&mut key, value_type.canonical_encoding().as_bytes());
            value_keys.push(Arc::new(key));
            values.push(ReducerBodyValueData {
                source: ReducerBodyValueSource::StateParameter(index),
                value_type,
            });
        }
        for (i, value_type) in contributors.iter().cloned().enumerate() {
            let index = u32::try_from(i).map_err(|_| IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::ScalarValue,
            })?;
            let mut key = vec![2];
            key.extend_from_slice(&index.to_be_bytes());
            encode_bytes(&mut key, value_type.canonical_encoding().as_bytes());
            value_keys.push(Arc::new(key));
            values.push(ReducerBodyValueData {
                source: ReducerBodyValueSource::ContributorParameter(index),
                value_type,
            });
        }
        let value_count = values.len();
        let canonical_bytes = value_keys.iter().map(|key| key.len()).sum();
        limit(
            canonical_bytes,
            MAX_SCALAR_CANONICAL_BYTES,
            IndexLimitKind::ScalarCanonicalBytes,
        )?;
        let encoded_body_bytes = 24_usize.saturating_add(parameter_bytes);
        parent_budget.admit(encoded_body_bytes)?;
        Ok(Self {
            registry,
            owner,
            values,
            value_keys,
            value_depths: vec![0; value_count],
            operations: Vec::new(),
            operation_keys: Vec::new(),
            operation_depths: Vec::new(),
            operation_intern: BTreeMap::new(),
            canonical_bytes,
            encoded_body_bytes,
            parent_budget,
            state_count: state.len(),
            contributor_count: contributors.len(),
            yields: None,
        })
    }
    /// Returns one ordered accumulator-state parameter.
    #[must_use]
    pub fn state(&self, index: usize) -> Option<ReducerScalarValueId> {
        (index < self.state_count).then(|| self.id(index))
    }
    /// Returns one ordered contributor parameter.
    #[must_use]
    pub fn contributor(&self, index: usize) -> Option<ReducerScalarValueId> {
        (index < self.contributor_count).then(|| self.id(self.state_count + index))
    }
    fn id(&self, index: usize) -> ReducerScalarValueId {
        ReducerScalarValueId {
            owner: self.owner,
            index: u32::try_from(index).expect("checked reducer-body bound fits u32"),
        }
    }
    fn resolve(&self, id: ReducerScalarValueId) -> Result<&ReducerBodyValueData, IndexBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(IndexEntityKind::ScalarValue, true));
        }
        self.values
            .get(id.index as usize)
            .ok_or_else(|| invalid_handle(IndexEntityKind::ScalarValue, false))
    }
    /// Applies a registered generic scalar operation inside the reducer body.
    ///
    /// # Errors
    ///
    /// Returns an error for foreign operands, rejected inference, or exceeded limits.
    pub fn apply(
        &mut self,
        key: ScalarOpKey,
        attributes: ScalarAttributes,
        operands: &[ReducerScalarValueId],
    ) -> Result<ReducerScalarResults, IndexBuildError> {
        limit(
            operands.len(),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarOperands,
        )?;
        let operand_indices: Vec<_> = operands
            .iter()
            .map(|id| {
                self.resolve(*id)?;
                Ok(id.index)
            })
            .collect::<Result<_, IndexBuildError>>()?;
        let operand_types: Vec<_> = operands
            .iter()
            .map(|id| self.resolve(*id).map(|value| value.value_type.clone()))
            .collect::<Result<_, _>>()?;
        let attributes = self.registry.normalize_attributes(&key, attributes)?;
        let structural_key = Arc::new(nested_operation_key(
            &key,
            &attributes,
            &operand_indices,
            &self.value_keys,
        ));
        if let Some(operation) = self.operation_intern.get(&structural_key) {
            return Ok(ReducerScalarResults(
                self.operations[*operation as usize]
                    .results
                    .iter()
                    .map(|index| ReducerScalarValueId {
                        owner: self.owner,
                        index: *index,
                    })
                    .collect(),
            ));
        }
        let depth = operands
            .iter()
            .map(|operand| self.value_depths[operand.index as usize].saturating_add(1))
            .max()
            .unwrap_or(0);
        let minimum_results = self.registry.minimum_results(&key)?;
        self.preflight_operation(depth, minimum_results, structural_key.len(), operands.len())?;
        let (encoded_before_results, capacity) =
            self.inference_capacity(&key, &attributes, operands.len(), minimum_results)?;
        let result_types = self
            .registry
            .infer(&key, &operand_types, &attributes, capacity)
            .map_err(map_scalar_apply_error)?;
        let added_bytes =
            retained_operation_bytes(structural_key.len(), operands.len(), &result_types, 0);
        let encoded_result_bytes = result_types.iter().fold(0_usize, |bytes, value_type| {
            bytes.saturating_add(encoded_reducer_operation_result_increment(value_type))
        });
        let encoded_after_results = encoded_before_results.saturating_add(encoded_result_bytes);
        limit(
            self.canonical_bytes.saturating_add(added_bytes),
            MAX_SCALAR_CANONICAL_BYTES,
            IndexLimitKind::ScalarCanonicalBytes,
        )?;
        let operation =
            u32::try_from(self.operations.len()).map_err(|_| IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::ScalarOperation,
            })?;
        let staged = stage_reducer_results(
            self.owner,
            operation,
            self.values.len(),
            result_types,
            &structural_key,
        )?;
        admit_reducer_body_append(self.parent_budget, encoded_after_results, || {
            self.values.extend(staged.values);
            self.value_keys.extend(staged.keys);
            self.value_depths
                .extend(std::iter::repeat_n(depth, staged.indices.len()));
            self.operations.push(ReducerBodyOperationData {
                key,
                attributes,
                operands: operand_indices,
                results: staged.indices,
            });
            self.operation_keys.push(structural_key.clone());
            self.operation_depths.push(depth);
            self.operation_intern.insert(structural_key, operation);
            self.canonical_bytes += added_bytes;
            self.encoded_body_bytes = encoded_after_results;
            ReducerScalarResults(staged.ids)
        })
    }

    fn preflight_operation(
        &self,
        depth: u32,
        minimum_results: usize,
        key_bytes: usize,
        operand_count: usize,
    ) -> Result<(), IndexBuildError> {
        limit(
            self.operations.len().saturating_add(1),
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarOperations,
        )?;
        limit(
            self.values.len().saturating_add(minimum_results),
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarValues,
        )?;
        limit(
            depth as usize,
            MAX_SCALAR_EXPRESSION_DEPTH as usize,
            IndexLimitKind::ScalarExpressionDepth,
        )?;
        limit(
            self.canonical_bytes
                .saturating_add(minimum_retained_operation_bytes(
                    key_bytes,
                    operand_count,
                    minimum_results,
                    0,
                )),
            MAX_SCALAR_CANONICAL_BYTES,
            IndexLimitKind::ScalarCanonicalBytes,
        )
    }

    fn inference_capacity(
        &self,
        key: &ScalarOpKey,
        attributes: &ScalarAttributes,
        operand_count: usize,
        minimum_results: usize,
    ) -> Result<(usize, ScalarInferenceCapacity), IndexBuildError> {
        let fixed_encoded_bytes =
            encoded_reducer_operation_base_len(key, attributes, operand_count);
        let encoded_before_results = self.encoded_body_bytes.saturating_add(fixed_encoded_bytes);
        let minimum_fixed_result_bytes = minimum_results
            .checked_mul(encoded_reducer_operation_result_overhead())
            .and_then(|bytes| encoded_before_results.checked_add(bytes))
            .unwrap_or(usize::MAX);
        self.parent_budget.admit(minimum_fixed_result_bytes)?;
        let parent_bytes_before_results = self.parent_budget.parent_bytes(encoded_before_results);
        Ok((
            encoded_before_results,
            ScalarInferenceCapacity {
                result_slots: MAX_SCALAR_EXPRESSIONS.saturating_sub(self.values.len()),
                result_count_before: self.values.len(),
                result_limit: MAX_SCALAR_EXPRESSIONS,
                retained_bytes: MAX_SCALAR_CANONICAL_BYTES
                    .saturating_sub(parent_bytes_before_results),
                retained_bytes_before: parent_bytes_before_results,
                retained_byte_limit: MAX_SCALAR_CANONICAL_BYTES,
                per_result_overhead: encoded_reducer_operation_result_overhead(),
                byte_multiplier: self.parent_budget.body_multiplier,
            },
        ))
    }
    /// Sets the exact ordered state yielded by one reducer step.
    ///
    /// # Errors
    ///
    /// Returns an error for foreign values or a second yield declaration.
    pub fn yield_values(&mut self, values: &[ReducerScalarValueId]) -> Result<(), IndexBuildError> {
        if self.yields.is_some() {
            return Err(IndexBuildError::ReducerYieldAlreadySet);
        }
        limit(
            values.len(),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarValues,
        )?;
        let indices: Vec<_> = values
            .iter()
            .map(|id| {
                self.resolve(*id)?;
                Ok(id.index)
            })
            .collect::<Result<_, IndexBuildError>>()?;
        let encoded_body_bytes = self
            .encoded_body_bytes
            .saturating_add(indices.len().saturating_mul(4));
        admit_reducer_body_append(self.parent_budget, encoded_body_bytes, || {
            self.yields = Some(indices);
            self.encoded_body_bytes = encoded_body_bytes;
        })
    }
}

/// Mutable transactional construction of one target-independent index region.
#[derive(Debug)]
pub struct IndexRegionBuilder {
    owner: BuilderId,
    registry: FrozenScalarRegistry,
    dimensions: Vec<DimensionData>,
    tensors: Vec<TensorData>,
    boundary_bytes: usize,
    expressions: Vec<DraftIndexExpr>,
    expression_intern: BTreeMap<Arc<Vec<u8>>, u32>,
    index_bytes: usize,
    accesses: Vec<AccessData>,
    access_intern: BTreeMap<AccessData, u32>,
    access_bytes: usize,
    operations: Vec<DraftScalarOperation>,
    operation_intern: BTreeMap<Arc<Vec<u8>>, u32>,
    values: Vec<DraftScalarValue>,
    read_values: BTreeMap<u32, u32>,
    scalar_bytes: usize,
    outputs: Vec<OutputData>,
    output_tensors: BTreeSet<u32>,
}

impl IndexRegionBuilder {
    /// Creates a builder over an exact frozen scalar/type authority snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error when no fresh builder ownership identity remains.
    pub fn new(registry: FrozenScalarRegistry) -> Result<Self, IndexBuildError> {
        let owner = next_builder_id().ok_or(IndexBuildError::BuilderIdentityExhausted)?;
        Ok(Self {
            owner,
            registry,
            dimensions: Vec::new(),
            tensors: Vec::new(),
            boundary_bytes: 0,
            expressions: Vec::new(),
            expression_intern: BTreeMap::new(),
            index_bytes: 0,
            accesses: Vec::new(),
            access_intern: BTreeMap::new(),
            access_bytes: 0,
            operations: Vec::new(),
            operation_intern: BTreeMap::new(),
            values: Vec::new(),
            read_values: BTreeMap::new(),
            scalar_bytes: 0,
            outputs: Vec::new(),
            output_tensors: BTreeSet::new(),
        })
    }

    /// Declares one tensor boundary.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown type or exceeded structural limit.
    pub fn tensor(
        &mut self,
        role: TensorRole,
        value_type: ResolvedValueType,
        shape: Shape,
    ) -> Result<TensorId, IndexBuildError> {
        self.registry.validate_type(&value_type)?;
        limit(
            self.tensors.len() + 1,
            MAX_BOUNDARY_TENSORS,
            IndexLimitKind::BoundaryTensors,
        )?;
        limit(shape.rank(), MAX_TENSOR_RANK, IndexLimitKind::TensorRank)?;
        let bytes = value_type
            .canonical_encoding()
            .as_bytes()
            .len()
            .saturating_add(shape.rank().saturating_mul(8))
            .saturating_add(16);
        limit(
            self.boundary_bytes.saturating_add(bytes),
            MAX_BOUNDARY_CANONICAL_BYTES,
            IndexLimitKind::BoundaryCanonicalBytes,
        )?;
        let id = TensorId::from_len(self.owner, self.tensors.len()).ok_or(
            IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::Tensor,
            },
        )?;
        self.tensors.push(TensorData {
            role,
            value_type,
            shape,
        });
        self.boundary_bytes += bytes;
        Ok(id)
    }
    /// Adds a static half-open dimension.
    ///
    /// # Errors
    ///
    /// Returns an error when the dimension-count limit is exceeded.
    pub fn dimension(
        &mut self,
        role: DomainRole,
        extent: Extent,
    ) -> Result<DimensionId, IndexBuildError> {
        limit(
            self.dimensions.len() + 1,
            MAX_DOMAIN_DIMENSIONS,
            IndexLimitKind::DomainDimensions,
        )?;
        let id = DimensionId::from_len(self.owner, self.dimensions.len()).ok_or(
            IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::Dimension,
            },
        )?;
        self.dimensions.push(DimensionData {
            role,
            extent: extent.get(),
        });
        Ok(id)
    }
    /// Creates or reuses an exact constant expression.
    ///
    /// # Errors
    ///
    /// Returns an error when an index-expression limit is exceeded.
    pub fn constant(&mut self, value: IndexInteger) -> Result<IndexExprId, IndexBuildError> {
        check_integer(&value.0)?;
        let integer = value.0.clone();
        self.intern_index(
            IndexNode::Constant(value),
            BTreeSet::new(),
            IndexExprClass::Affine,
            Some((integer.clone(), integer)),
            0,
        )
    }
    /// Creates or reuses a dimension expression.
    ///
    /// # Errors
    ///
    /// Returns an error for a foreign dimension or exceeded expression limit.
    pub fn dimension_expr(
        &mut self,
        dimension: DimensionId,
    ) -> Result<IndexExprId, IndexBuildError> {
        let data = self.resolve_dimension(dimension)?;
        let mut dimensions = BTreeSet::new();
        dimensions.insert(dimension.index);
        self.intern_index(
            IndexNode::Dimension(dimension.index),
            dimensions,
            IndexExprClass::Affine,
            (data.extent != 0).then(|| (BigInt::zero(), BigInt::from(data.extent - 1))),
            0,
        )
    }
    /// Creates a normalized affine linear combination.
    ///
    /// # Errors
    ///
    /// Returns an error for foreign operands or exceeded expression limits.
    pub fn linear_combination(
        &mut self,
        constant: IndexInteger,
        terms: &[(IndexInteger, IndexExprId)],
    ) -> Result<IndexExprId, IndexBuildError> {
        limit(
            terms.len(),
            MAX_INDEX_EXPRESSION_OPERANDS,
            IndexLimitKind::IndexExpressionOperands,
        )?;
        check_integer(&constant.0)?;
        for (coefficient, _) in terms {
            check_integer(&coefficient.0)?;
        }
        let mut normalized_constant = constant.0;
        let mut coefficients: BTreeMap<Arc<Vec<u8>>, (u32, BigInt)> = BTreeMap::new();
        for (coefficient, id) in terms {
            self.resolve_expr(*id)?;
            accumulate_linear_term(
                &mut normalized_constant,
                &mut coefficients,
                &coefficient.0,
                id.index,
                &self.expressions,
            )?;
        }
        let mut terms: Vec<_> = coefficients
            .into_iter()
            .filter(|(_, (_, coefficient))| !coefficient.is_zero())
            .map(|(_, (value, coefficient))| LinearTermData {
                coefficient: IndexInteger(coefficient),
                value,
            })
            .collect();
        limit(
            terms.len(),
            MAX_INDEX_EXPRESSION_OPERANDS,
            IndexLimitKind::IndexExpressionOperands,
        )?;
        if terms.is_empty() {
            return self.constant(IndexInteger(normalized_constant));
        }
        terms.sort_by_key(|term| Arc::clone(&self.expressions[term.value as usize].structural_key));
        if normalized_constant.is_zero()
            && terms.len() == 1
            && terms[0].coefficient.0 == BigInt::from(1_u8)
        {
            return Ok(IndexExprId {
                owner: self.owner,
                index: terms[0].value,
            });
        }
        let mut dimensions = BTreeSet::new();
        let mut class = IndexExprClass::Affine;
        let mut depth = 0;
        for term in &terms {
            check_integer(&term.coefficient.0)?;
            let expression = &self.expressions[term.value as usize];
            dimensions.extend(&expression.dimensions);
            if expression.class == IndexExprClass::QuasiAffine {
                class = IndexExprClass::QuasiAffine;
            }
            depth = depth.max(expression.depth.saturating_add(1));
        }
        check_integer(&normalized_constant)?;
        let interval = interval_linear(&normalized_constant, &terms, &self.expressions)?;
        self.intern_index(
            IndexNode::LinearCombination {
                constant: IndexInteger(normalized_constant),
                terms,
            },
            dimensions,
            class,
            interval,
            depth,
        )
    }
    /// Creates Euclidean floor division by a positive constant.
    ///
    /// # Errors
    ///
    /// Returns an error for a zero divisor, foreign dividend, or exceeded limit.
    pub fn floor_div(
        &mut self,
        dividend: IndexExprId,
        divisor: u64,
    ) -> Result<IndexExprId, IndexBuildError> {
        self.div_mod(dividend, divisor, true)
    }
    /// Creates Euclidean modulo by a positive constant.
    ///
    /// # Errors
    ///
    /// Returns an error for a zero divisor, foreign dividend, or exceeded limit.
    pub fn modulo(
        &mut self,
        dividend: IndexExprId,
        divisor: u64,
    ) -> Result<IndexExprId, IndexBuildError> {
        self.div_mod(dividend, divisor, false)
    }
    fn div_mod(
        &mut self,
        dividend: IndexExprId,
        divisor: u64,
        div: bool,
    ) -> Result<IndexExprId, IndexBuildError> {
        if divisor == 0 {
            return Err(IndexBuildError::NonPositiveDivisor);
        }
        let data = self.resolve_expr(dividend)?.clone();
        let d = BigInt::from(divisor);
        let interval = if div {
            data.interval
                .map(|(a, b)| (a.div_floor(&d), b.div_floor(&d)))
        } else {
            Some((BigInt::zero(), BigInt::from(divisor - 1)))
        };
        let node = if div {
            IndexNode::FloorDiv {
                dividend: dividend.index,
                divisor,
            }
        } else {
            IndexNode::Modulo {
                dividend: dividend.index,
                divisor,
            }
        };
        self.intern_index(
            node,
            data.dimensions,
            IndexExprClass::QuasiAffine,
            interval,
            data.depth + 1,
        )
    }

    /// Creates or reuses a logical write access.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid access contract or exceeded limit.
    pub fn write(
        &mut self,
        tensor: TensorId,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<TensorAccessId, IndexBuildError> {
        self.access(tensor, AccessMode::Write, domain, coordinates)
    }
    /// Creates or reuses a read access and its scalar SSA value.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid access contract or exceeded limit.
    pub fn read(
        &mut self,
        tensor: TensorId,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, IndexBuildError> {
        let (data, bytes) = self.prepare_access(tensor, AccessMode::Read, domain, coordinates)?;
        if let Some(access) = self.access_intern.get(&data)
            && let Some(value) = self.read_values.get(access)
        {
            return Ok(ScalarValueId {
                owner: self.owner,
                index: *value,
            });
        }
        limit(
            self.values.len() + 1,
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarValues,
        )?;
        let tensor_data = self.resolve_tensor(tensor)?.clone();
        let free_dimensions: BTreeSet<u32> = domain.iter().map(|d| d.index).collect();
        let structural_key = Arc::new(access_read_key(&data, &self.tensors, &self.expressions));
        let retained_bytes = structural_key
            .len()
            .saturating_add(tensor_data.value_type.canonical_encoding().as_bytes().len())
            .saturating_add(free_dimensions.len().saturating_mul(4));
        limit(
            self.scalar_bytes.saturating_add(retained_bytes),
            MAX_SCALAR_CANONICAL_BYTES,
            IndexLimitKind::ScalarCanonicalBytes,
        )?;
        self.preflight_access(&data, bytes)?;
        let access = self.commit_access(data, bytes)?;
        let value = self.commit_value(
            ScalarValueDefinition::AccessRead {
                access: access.index,
            },
            tensor_data.value_type,
            free_dimensions,
            0,
            structural_key,
            retained_bytes,
        );
        self.read_values.insert(access.index, value.index);
        Ok(value)
    }
    fn access(
        &mut self,
        tensor: TensorId,
        mode: AccessMode,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<TensorAccessId, IndexBuildError> {
        let (data, bytes) = self.prepare_access(tensor, mode, domain, coordinates)?;
        self.commit_access(data, bytes)
    }

    fn prepare_access(
        &self,
        tensor: TensorId,
        mode: AccessMode,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<(AccessData, usize), IndexBuildError> {
        let tensor_data = self.resolve_tensor(tensor)?.clone();
        match (mode, tensor_data.role) {
            (AccessMode::Read, TensorRole::Output) => return Err(IndexBuildError::ReadFromOutput),
            (AccessMode::Write, TensorRole::Input) => return Err(IndexBuildError::WriteToInput),
            _ => {}
        }
        if coordinates.len() != tensor_data.shape.rank() {
            return Err(IndexBuildError::AccessRank {
                expected: tensor_data.shape.rank(),
                actual: coordinates.len(),
            });
        }
        let mut domain_set = BTreeSet::new();
        for dimension in domain {
            self.resolve_dimension(*dimension)?;
            if !domain_set.insert(dimension.index) {
                return Err(IndexBuildError::DuplicateAccessDimension {
                    dimension: *dimension,
                });
            }
        }
        if mode == AccessMode::Write && domain_set != self.parallel_dimensions() {
            return Err(IndexBuildError::InvalidWriteDomain);
        }
        let coords: Vec<_> = coordinates
            .iter()
            .map(|id| {
                let expr = self.resolve_expr(*id)?;
                if !expr.dimensions.is_subset(&domain_set) {
                    return Err(IndexBuildError::CoordinateOutsideAccessDomain);
                }
                Ok(id.index)
            })
            .collect::<Result<_, _>>()?;
        let data = AccessData {
            tensor: tensor.index,
            mode,
            domain: domain_set.iter().copied().collect(),
            coordinates: coords,
        };
        let bytes = 24 + 4 * (data.domain.len() + data.coordinates.len());
        Ok((data, bytes))
    }

    fn commit_access(
        &mut self,
        data: AccessData,
        bytes: usize,
    ) -> Result<TensorAccessId, IndexBuildError> {
        if let Some(index) = self.access_intern.get(&data) {
            return Ok(TensorAccessId {
                owner: self.owner,
                index: *index,
            });
        }
        self.preflight_access(&data, bytes)?;
        let id = TensorAccessId::from_len(self.owner, self.accesses.len())
            .ok_or_else(|| too_many(IndexEntityKind::TensorAccess))?;
        self.access_intern.insert(data.clone(), id.index);
        self.accesses.push(data);
        self.access_bytes += bytes;
        Ok(id)
    }

    fn preflight_access(&self, data: &AccessData, bytes: usize) -> Result<(), IndexBuildError> {
        if self.access_intern.contains_key(data) {
            return Ok(());
        }
        limit(
            self.accesses.len() + 1,
            MAX_TENSOR_ACCESSES,
            IndexLimitKind::TensorAccesses,
        )?;
        limit(
            self.access_bytes + bytes,
            MAX_ACCESS_CANONICAL_BYTES,
            IndexLimitKind::AccessCanonicalBytes,
        )?;
        TensorAccessId::from_len(self.owner, self.accesses.len())
            .ok_or_else(|| too_many(IndexEntityKind::TensorAccess))?;
        Ok(())
    }

    /// Applies one registered scalar operation and infers all ordered results.
    ///
    /// # Errors
    ///
    /// Returns an error for foreign operands, rejected inference, or exceeded limits.
    pub fn apply(
        &mut self,
        key: ScalarOpKey,
        attributes: ScalarAttributes,
        operands: &[ScalarValueId],
    ) -> Result<ScalarResults, IndexBuildError> {
        self.apply_in(&[], key, attributes, operands)
    }

    /// Applies a scalar operation in an explicit additional evaluation scope.
    ///
    /// Explicit dimensions are useful for nullary or broadcast values that must be evaluated at
    /// each point of a later reduction. Operand dimensions remain implicit inputs to the scope.
    ///
    /// # Errors
    ///
    /// Returns an error for foreign or duplicate dimensions, foreign operands, rejected
    /// inference, or exceeded limits.
    pub fn apply_in(
        &mut self,
        dimensions: &[DimensionId],
        key: ScalarOpKey,
        attributes: ScalarAttributes,
        operands: &[ScalarValueId],
    ) -> Result<ScalarResults, IndexBuildError> {
        limit(
            operands.len(),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarOperands,
        )?;
        limit(
            dimensions.len(),
            MAX_DOMAIN_DIMENSIONS,
            IndexLimitKind::DomainDimensions,
        )?;
        let operand_data: Vec<_> = operands
            .iter()
            .map(|id| self.resolve_value(*id).cloned())
            .collect::<Result<_, _>>()?;
        let mut free: BTreeSet<_> = operand_data
            .iter()
            .flat_map(|value| value.free_dimensions.iter().copied())
            .collect();
        for dimension in dimensions {
            self.resolve_dimension(*dimension)?;
            if !free.insert(dimension.index) {
                return Err(IndexBuildError::DuplicateEvaluationDimension {
                    dimension: *dimension,
                });
            }
        }
        let attributes = self.registry.normalize_attributes(&key, attributes)?;
        let structural_key = Arc::new(apply_operation_key(
            &key,
            &attributes,
            operands,
            &self.values,
            &free,
        ));
        if let Some(operation) = self.operation_intern.get(&structural_key) {
            return Ok(self.operation_results(*operation));
        }
        let depth = operands
            .iter()
            .map(|id| self.values[id.as_usize()].depth.saturating_add(1))
            .max()
            .unwrap_or(0);
        let minimum_results = self.registry.minimum_results(&key)?;
        limit(
            self.operations.len().saturating_add(1),
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarOperations,
        )?;
        limit(
            self.values.len().saturating_add(minimum_results),
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarValues,
        )?;
        limit(
            depth as usize,
            MAX_SCALAR_EXPRESSION_DEPTH as usize,
            IndexLimitKind::ScalarExpressionDepth,
        )?;
        limit(
            self.scalar_bytes
                .saturating_add(minimum_retained_operation_bytes(
                    structural_key.len(),
                    operands.len(),
                    minimum_results,
                    free.len(),
                )),
            MAX_SCALAR_CANONICAL_BYTES,
            IndexLimitKind::ScalarCanonicalBytes,
        )?;
        let types: Vec<_> = operand_data.iter().map(|v| v.value_type.clone()).collect();
        let capacity = inference_capacity(
            self.values.len(),
            MAX_SCALAR_EXPRESSIONS,
            self.scalar_bytes,
            MAX_SCALAR_CANONICAL_BYTES,
            structural_key.len(),
            operands.len(),
            free.len(),
        );
        let result_types = self
            .registry
            .infer(&key, &types, &attributes, capacity)
            .map_err(map_scalar_apply_error)?;
        self.push_operation(
            ScalarOperationKindData::Apply { key, attributes },
            operands,
            result_types,
            &free,
        )
    }

    /// Builds an N-state exact lexicographic left-fold reduction.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid reduction, reducer body, or exceeded limit.
    pub fn reduce<F>(
        &mut self,
        dimensions: &[DimensionId],
        init: &[ScalarValueId],
        contributors: &[ScalarValueId],
        build: F,
    ) -> Result<ScalarResults, IndexBuildError>
    where
        F: FnOnce(&mut ScalarReducerBodyBuilder<'_>) -> Result<(), IndexBuildError>,
    {
        let ReductionInputs {
            bound,
            init: init_data,
            contributors: contributor_data,
            mut free,
            body_budget,
        } = self.prepare_reduction_inputs(dimensions, init, contributors)?;
        let state_types: Vec<_> = init_data.iter().map(|v| v.value_type.clone()).collect();
        let contributor_types: Vec<_> = contributor_data
            .iter()
            .map(|v| v.value_type.clone())
            .collect();
        let mut body = ScalarReducerBodyBuilder::new(
            &self.registry,
            &state_types,
            &contributor_types,
            body_budget,
        )?;
        build(&mut body)?;
        let yields = body
            .yields
            .take()
            .ok_or(IndexBuildError::MissingReducerYield)?;
        if yields.len() != state_types.len() {
            return Err(IndexBuildError::ReducerYieldArity {
                expected: state_types.len(),
                actual: yields.len(),
            });
        }
        for (position, (yielded, expected)) in yields.iter().zip(&state_types).enumerate() {
            let actual = &body.values[*yielded as usize].value_type;
            if actual != expected {
                return Err(IndexBuildError::ReducerYieldTypeMismatch {
                    position,
                    expected: Arc::new(expected.clone()),
                    actual: Arc::new(actual.clone()),
                });
            }
        }
        for dimension in &bound {
            free.remove(dimension);
        }
        let mut operands = init.to_vec();
        operands.extend_from_slice(contributors);
        let nested = compact_reducer_body(
            &ScalarReducerBodyData {
                values: body.values,
                operations: body.operations,
                yields,
            },
            &body.operation_keys,
            &body.operation_depths,
        );
        self.push_operation(
            ScalarOperationKindData::Reduce {
                dimensions: dimensions.iter().map(|dimension| dimension.index).collect(),
                traversal: ReductionTraversal::ExactLexicographicLeftFold,
                init: init.iter().map(|value| value.index).collect(),
                contributors: contributors.iter().map(|value| value.index).collect(),
                body: nested,
            },
            &operands,
            state_types,
            &free,
        )
    }

    fn prepare_reduction_inputs(
        &self,
        dimensions: &[DimensionId],
        init: &[ScalarValueId],
        contributors: &[ScalarValueId],
    ) -> Result<ReductionInputs, IndexBuildError> {
        if dimensions.is_empty() {
            return Err(IndexBuildError::EmptyReductionDimensions);
        }
        if init.is_empty() {
            return Err(IndexBuildError::EmptyReductionState);
        }
        limit(
            dimensions.len(),
            MAX_DOMAIN_DIMENSIONS,
            IndexLimitKind::DomainDimensions,
        )?;
        limit(
            init.len(),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarValues,
        )?;
        limit(
            contributors.len(),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarOperands,
        )?;
        limit(
            init.len().saturating_add(contributors.len()),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarOperands,
        )?;
        limit(
            self.operations.len().saturating_add(1),
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarOperations,
        )?;
        limit(
            self.values.len().saturating_add(init.len()),
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarValues,
        )?;
        let mut bound = BTreeSet::new();
        for dimension in dimensions {
            let data = self.resolve_dimension(*dimension)?;
            if data.role != DomainRole::Reduction {
                return Err(IndexBuildError::ExpectedReductionDimension {
                    dimension: *dimension,
                });
            }
            if !bound.insert(dimension.index) {
                return Err(IndexBuildError::DuplicateReductionDimension {
                    dimension: *dimension,
                });
            }
        }
        let init_data: Vec<_> = init
            .iter()
            .map(|id| self.resolve_value(*id).cloned())
            .collect::<Result<_, _>>()?;
        for value in &init_data {
            if let Some(dimension) = value.free_dimensions.intersection(&bound).next() {
                return Err(IndexBuildError::PointwiseDomainContainsReductionDimension {
                    dimension: DimensionId {
                        owner: self.owner,
                        index: *dimension,
                    },
                });
            }
        }
        let contributor_data: Vec<_> = contributors
            .iter()
            .map(|id| self.resolve_value(*id).cloned())
            .collect::<Result<_, _>>()?;
        let (free, body_budget) = self.preflight_reduction_occurrence(
            dimensions,
            init,
            contributors,
            &bound,
            &init_data,
            &contributor_data,
        )?;
        Ok(ReductionInputs {
            bound,
            init: init_data,
            contributors: contributor_data,
            free,
            body_budget,
        })
    }

    fn preflight_reduction_occurrence(
        &self,
        dimensions: &[DimensionId],
        init: &[ScalarValueId],
        contributors: &[ScalarValueId],
        bound: &BTreeSet<u32>,
        init_data: &[ScalarValueData],
        contributor_data: &[ScalarValueData],
    ) -> Result<(BTreeSet<u32>, ReducerBodyBudget), IndexBuildError> {
        let mut free: BTreeSet<_> = init_data
            .iter()
            .chain(contributor_data)
            .flat_map(|value| value.free_dimensions.iter().copied())
            .collect();
        for dimension in bound {
            free.remove(dimension);
        }
        let operands = init.iter().chain(contributors).copied().collect::<Vec<_>>();
        let minimum_body = minimum_reducer_body(init_data, contributor_data);
        let minimum_body_bytes = encoded_reducer_body_len(&minimum_body);
        let minimum_kind = ScalarOperationKindData::Reduce {
            dimensions: dimensions.iter().map(|dimension| dimension.index).collect(),
            traversal: ReductionTraversal::ExactLexicographicLeftFold,
            init: init.iter().map(|value| value.index).collect(),
            contributors: contributors.iter().map(|value| value.index).collect(),
            body: minimum_body,
        };
        let key = operation_structural_key(&minimum_kind, &operands, &self.values, &free);
        let depth = operands
            .iter()
            .map(|value| self.values[value.as_usize()].depth.saturating_add(1))
            .max()
            .unwrap_or(0);
        limit(
            depth as usize,
            MAX_SCALAR_EXPRESSION_DEPTH as usize,
            IndexLimitKind::ScalarExpressionDepth,
        )?;
        let result_types = init_data
            .iter()
            .map(|value| value.value_type.clone())
            .collect::<Vec<_>>();
        let minimum_parent_bytes = self.scalar_bytes.saturating_add(retained_operation_bytes(
            key.len(),
            operands.len(),
            &result_types,
            free.len(),
        ));
        limit(
            minimum_parent_bytes,
            MAX_SCALAR_CANONICAL_BYTES,
            IndexLimitKind::ScalarCanonicalBytes,
        )?;
        let body_multiplier = init.len().saturating_add(1);
        let body_contribution = minimum_body_bytes.saturating_mul(body_multiplier);
        let parent_bytes_without_body = minimum_parent_bytes.saturating_sub(body_contribution);
        let maximum_encoded_bytes =
            MAX_SCALAR_CANONICAL_BYTES.saturating_sub(parent_bytes_without_body) / body_multiplier;
        Ok((
            free,
            ReducerBodyBudget {
                parent_bytes_without_body,
                body_multiplier,
                maximum_encoded_bytes,
            },
        ))
    }

    fn push_operation(
        &mut self,
        kind: ScalarOperationKindData,
        operands: &[ScalarValueId],
        result_types: Vec<ResolvedValueType>,
        free: &BTreeSet<u32>,
    ) -> Result<ScalarResults, IndexBuildError> {
        let structural_key = Arc::new(operation_structural_key(
            &kind,
            operands,
            &self.values,
            free,
        ));
        if let Some(operation) = self.operation_intern.get(&structural_key) {
            return Ok(self.operation_results(*operation));
        }
        limit(
            operands.len(),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarOperands,
        )?;
        limit(
            result_types.len(),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarValues,
        )?;
        limit(
            self.operations.len() + 1,
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarOperations,
        )?;
        limit(
            self.values.len().saturating_add(result_types.len()),
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarValues,
        )?;
        let bytes = retained_operation_bytes(
            structural_key.len(),
            operands.len(),
            &result_types,
            free.len(),
        );
        limit(
            self.scalar_bytes + bytes,
            MAX_SCALAR_CANONICAL_BYTES,
            IndexLimitKind::ScalarCanonicalBytes,
        )?;
        let operation = ScalarOperationId::from_len(self.owner, self.operations.len())
            .ok_or_else(|| too_many(IndexEntityKind::ScalarOperation))?;
        let depth = operands
            .iter()
            .map(|id| self.values[id.as_usize()].depth + 1)
            .max()
            .unwrap_or(0);
        limit(
            depth as usize,
            MAX_SCALAR_EXPRESSION_DEPTH as usize,
            IndexLimitKind::ScalarExpressionDepth,
        )?;
        let mut results = Vec::with_capacity(result_types.len());
        let mut result_indices = Vec::with_capacity(result_types.len());
        let mut staged_values = Vec::with_capacity(result_types.len());
        for (result, value_type) in result_types.into_iter().enumerate() {
            let result =
                ScalarResultIndex::from_usize(result).ok_or(IndexBuildError::TooManyEntities {
                    entity: IndexEntityKind::ScalarValue,
                })?;
            let index = u32::try_from(self.values.len() + staged_values.len()).map_err(|_| {
                IndexBuildError::TooManyEntities {
                    entity: IndexEntityKind::ScalarValue,
                }
            })?;
            let id = ScalarValueId {
                owner: self.owner,
                index,
            };
            let mut value_key = structural_key.as_ref().clone();
            value_key.extend_from_slice(&result.get().to_be_bytes());
            staged_values.push(DraftScalarValue {
                data: ScalarValueData {
                    definition: ScalarValueDefinition::OperationResult {
                        operation: operation.index,
                        result,
                    },
                    value_type,
                    free_dimensions: free.clone(),
                    depth,
                },
                structural_key: Arc::new(value_key),
            });
            result_indices.push(id.index);
            results.push(id);
        }
        self.values.extend(staged_values);
        self.operations.push(DraftScalarOperation {
            data: ScalarOperationData {
                kind,
                operands: operands.iter().map(|v| v.index).collect(),
                results: result_indices,
                depth,
            },
        });
        self.operation_intern
            .insert(structural_key, operation.index);
        self.scalar_bytes += bytes;
        Ok(ScalarResults(results))
    }
    fn commit_value(
        &mut self,
        definition: ScalarValueDefinition,
        value_type: ResolvedValueType,
        free_dimensions: BTreeSet<u32>,
        depth: u32,
        structural_key: Arc<Vec<u8>>,
        retained_bytes: usize,
    ) -> ScalarValueId {
        let id = ScalarValueId::from_len(self.owner, self.values.len())
            .expect("preflighted scalar-value count fits its handle");
        self.values.push(DraftScalarValue {
            data: ScalarValueData {
                definition,
                value_type,
                free_dimensions,
                depth,
            },
            structural_key,
        });
        self.scalar_bytes += retained_bytes;
        id
    }

    fn operation_results(&self, operation: u32) -> ScalarResults {
        ScalarResults(
            self.operations[operation as usize]
                .results
                .iter()
                .map(|index| ScalarValueId {
                    owner: self.owner,
                    index: *index,
                })
                .collect(),
        )
    }

    /// Adds one ordered output root.
    ///
    /// # Errors
    ///
    /// Returns an error for foreign handles, duplicate roots, or type mismatch.
    pub fn output(
        &mut self,
        access: TensorAccessId,
        value: ScalarValueId,
    ) -> Result<(), IndexBuildError> {
        limit(
            self.outputs.len() + 1,
            MAX_OUTPUT_ROOTS,
            IndexLimitKind::OutputRoots,
        )?;
        let access_data = self.resolve_access(access)?.clone();
        let value_data = self.resolve_value(value)?;
        if access_data.mode != AccessMode::Write {
            return Err(IndexBuildError::OutputUsesRead);
        }
        if self.tensors[access_data.tensor as usize].value_type != value_data.value_type {
            return Err(IndexBuildError::OutputTypeMismatch);
        }
        if !self.output_tensors.insert(access_data.tensor) {
            return Err(IndexBuildError::DuplicateOutputTensor);
        }
        self.outputs.push(OutputData {
            access: access.index,
            value: value.index,
        });
        Ok(())
    }

    /// Consumes, verifies, reachability-compacts, and canonicalizes this region.
    ///
    /// # Errors
    ///
    /// Returns the intact builder with all deterministic verification diagnostics.
    pub fn build(self) -> Result<VerifiedIndexRegion, IndexRegionBuildError> {
        match self.verify() {
            Ok(region) => Ok(region),
            Err(diagnostics) => Err(IndexRegionBuildError {
                builder: Box::new(self),
                diagnostics,
            }),
        }
    }
    fn verify(&self) -> Result<VerifiedIndexRegion, Vec<IndexRegionDiagnostic>> {
        let mut diagnostics = Vec::new();
        if self.outputs.is_empty() {
            diagnostics.push(IndexRegionDiagnostic::NoOutputs);
        }
        self.verify_output_tensors(&mut diagnostics);
        let reachable_values = self.reachable_values();
        let reachable_accesses: BTreeSet<_> = reachable_values
            .iter()
            .filter_map(|i| match self.values[*i as usize].definition {
                ScalarValueDefinition::AccessRead { access } => Some(access),
                ScalarValueDefinition::OperationResult { .. } => None,
            })
            .chain(self.outputs.iter().map(|o| o.access))
            .collect();
        self.verify_inputs_are_reachable(&reachable_accesses, &mut diagnostics);
        let reachable_operations: BTreeSet<_> = reachable_values
            .iter()
            .filter_map(|value| match self.values[*value as usize].definition {
                ScalarValueDefinition::OperationResult { operation, .. } => Some(operation),
                ScalarValueDefinition::AccessRead { .. } => None,
            })
            .collect();
        let used_reductions: BTreeSet<_> = reachable_operations
            .iter()
            .filter_map(
                |operation| match &self.operations[*operation as usize].kind {
                    ScalarOperationKindData::Reduce { dimensions, .. } => {
                        Some(dimensions.iter().copied())
                    }
                    ScalarOperationKindData::Apply { .. } => None,
                },
            )
            .flatten()
            .collect();
        for output in &self.outputs {
            for dimension in &self.values[output.value as usize].free_dimensions {
                if self.dimensions[*dimension as usize].role == DomainRole::Reduction {
                    diagnostics.push(IndexRegionDiagnostic::FreeReductionDimension {
                        value: ScalarValueId {
                            owner: self.owner,
                            index: output.value,
                        },
                        dimension: DimensionId {
                            owner: self.owner,
                            index: *dimension,
                        },
                    });
                }
            }
        }
        for (i, _dimension) in self
            .dimensions
            .iter()
            .enumerate()
            .filter(|(_, d)| d.role == DomainRole::Reduction)
        {
            let index = bounded_index(i);
            if !used_reductions.contains(&index) {
                diagnostics.push(IndexRegionDiagnostic::UnusedDomainDimension {
                    dimension: DimensionId {
                        owner: self.owner,
                        index,
                    },
                });
            }
        }
        self.verify_accesses(&reachable_accesses, &mut diagnostics);
        if !diagnostics.is_empty() {
            diagnostics.sort_by_key(|d| format!("{d:?}"));
            diagnostics.dedup();
            return Err(diagnostics);
        }
        self.compact(reachable_values, reachable_accesses)
            .map_err(|diagnostic| vec![diagnostic])
    }

    fn verify_output_tensors(&self, diagnostics: &mut Vec<IndexRegionDiagnostic>) {
        for (i, _tensor) in self
            .tensors
            .iter()
            .enumerate()
            .filter(|(_, t)| t.role == TensorRole::Output)
        {
            let index = bounded_index(i);
            if !self.output_tensors.contains(&index) {
                diagnostics.push(IndexRegionDiagnostic::MissingOutputTensor {
                    tensor: TensorId {
                        owner: self.owner,
                        index,
                    },
                });
            }
        }
    }

    fn verify_inputs_are_reachable(
        &self,
        reachable_accesses: &BTreeSet<u32>,
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) {
        for (i, _tensor) in self
            .tensors
            .iter()
            .enumerate()
            .filter(|(_, t)| t.role == TensorRole::Input)
        {
            let index = bounded_index(i);
            if !reachable_accesses
                .iter()
                .any(|a| self.accesses[*a as usize].tensor == index)
            {
                diagnostics.push(IndexRegionDiagnostic::UnusedInputTensor {
                    tensor: TensorId {
                        owner: self.owner,
                        index,
                    },
                });
            }
        }
    }
    fn verify_accesses(
        &self,
        accesses: &BTreeSet<u32>,
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) {
        let mut cells = 0_u128;
        let mut integer_bytes = 0_u128;
        for access_index in accesses {
            let access = &self.accesses[*access_index as usize];
            let shape = &self.tensors[access.tensor as usize].shape;
            let points = self.domain_points(&access.domain);
            if points == 0 {
                if access.mode == AccessMode::Write && shape.element_count() != Some(0) {
                    diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                        access: TensorAccessId {
                            owner: self.owner,
                            index: *access_index,
                        },
                    });
                }
                continue;
            }
            let mut interval_proved = true;
            let mut definitely_outside = false;
            for (coordinate, extent) in access.coordinates.iter().zip(shape.extents()) {
                let Some((min, max)) = &self.expressions[*coordinate as usize].interval else {
                    interval_proved = false;
                    continue;
                };
                let extent = BigInt::from(extent.get());
                if max < &BigInt::zero() || min >= &extent {
                    definitely_outside = true;
                }
                if min < &BigInt::zero() || max >= &extent {
                    interval_proved = false;
                }
            }
            if definitely_outside {
                diagnostics.push(IndexRegionDiagnostic::CoordinateOutOfBounds {
                    access: TensorAccessId {
                        owner: self.owner,
                        index: *access_index,
                    },
                });
                continue;
            }
            if !interval_proved
                || (access.mode == AccessMode::Write && !self.write_is_permutation(access, shape))
            {
                let (plan_len, bytes_per_point) = self.proof_plan_size(&access.coordinates);
                cells = cells.saturating_add(u128::from(points).saturating_mul(plan_len as u128));
                let coordinate_bytes = u128::from(points).saturating_mul(bytes_per_point.max(1));
                let dense_bytes = if access.mode == AccessMode::Write {
                    shape.element_count().map_or(u128::MAX, |elements| {
                        elements.div_ceil(64).saturating_mul(8) as u128
                    })
                } else {
                    0
                };
                integer_bytes =
                    integer_bytes.saturating_add(coordinate_bytes.saturating_add(dense_bytes));
            }
        }
        let admitted = with_admitted_proof_budget(
            cells,
            integer_bytes,
            MAX_EXHAUSTIVE_PROOF_CELLS,
            MAX_EXHAUSTIVE_PROOF_BYTES,
            || {
                for access_index in accesses {
                    let access = &self.accesses[*access_index as usize];
                    let shape = &self.tensors[access.tensor as usize].shape;
                    let points = self.domain_points(&access.domain);
                    if points == 0 || !self.access_needs_exhaustive_proof(access, shape) {
                        continue;
                    }
                    let mut reached = BTreeSet::new();
                    for coordinate in &access.coordinates {
                        self.mark_expr(*coordinate, &mut reached);
                    }
                    let plan = reached.into_iter().collect::<Vec<_>>();
                    self.verify_access_exhaustively(*access_index, &plan, diagnostics);
                }
            },
        );
        if let Err(excess) = admitted {
            diagnostics.push(excess.diagnostic());
        }
    }

    fn access_needs_exhaustive_proof(&self, access: &AccessData, shape: &Shape) -> bool {
        let interval_proved =
            access
                .coordinates
                .iter()
                .zip(shape.extents())
                .all(|(coordinate, extent)| {
                    self.expressions[*coordinate as usize]
                        .interval
                        .as_ref()
                        .is_some_and(|(minimum, maximum)| {
                            minimum >= &BigInt::zero() && maximum < &BigInt::from(extent.get())
                        })
                });
        !interval_proved
            || (access.mode == AccessMode::Write && !self.write_is_permutation(access, shape))
    }

    fn proof_plan_size(&self, coordinates: &[u32]) -> (usize, u128) {
        let mut visited = vec![false; self.expressions.len()];
        let mut pending = coordinates.to_vec();
        let mut count = 0_usize;
        let mut integer_bytes = 0_u128;
        while let Some(expression) = pending.pop() {
            if std::mem::replace(&mut visited[expression as usize], true) {
                continue;
            }
            count = count.saturating_add(1);
            integer_bytes = integer_bytes.saturating_add(self.expression_integer_bytes(expression));
            match &*self.expressions[expression as usize].node {
                IndexNode::LinearCombination { terms, .. } => {
                    pending.extend(terms.iter().map(|term| term.value));
                }
                IndexNode::FloorDiv { dividend, .. } | IndexNode::Modulo { dividend, .. } => {
                    pending.push(*dividend);
                }
                IndexNode::Constant(_) | IndexNode::Dimension(_) => {}
            }
        }
        (count, integer_bytes)
    }

    fn domain_points(&self, domain: &[u32]) -> u64 {
        if domain
            .iter()
            .any(|dimension| self.dimensions[*dimension as usize].extent == 0)
        {
            return 0;
        }
        domain.iter().fold(1_u64, |points, dimension| {
            points.saturating_mul(self.dimensions[*dimension as usize].extent)
        })
    }

    fn expression_integer_bytes(&self, expression: u32) -> u128 {
        let data = &self.expressions[expression as usize];
        if data.interval.is_none() {
            return u128::MAX;
        }
        let magnitude_bound = if let IndexNode::LinearCombination { constant, terms } = &*data.node
        {
            let mut bound = constant.0.abs();
            for term in terms {
                let (minimum, maximum) = self.expressions[term.value as usize]
                    .interval
                    .as_ref()
                    .expect("a linear interval requires every child interval");
                let child_bound = minimum.abs().max(maximum.abs());
                let Ok(product) = checked_index_product(&term.coefficient.0.abs(), &child_bound)
                else {
                    return u128::MAX;
                };
                if checked_index_add_assign(&mut bound, &product).is_err() {
                    return u128::MAX;
                }
            }
            bound
        } else {
            let Some((minimum, maximum)) = &data.interval else {
                return u128::MAX;
            };
            minimum.abs().max(maximum.abs())
        };
        u128::try_from(magnitude_bound.to_signed_bytes_be().len().max(1)).unwrap_or(u128::MAX)
    }

    fn verify_access_exhaustively(
        &self,
        access_index: u32,
        expression_plan: &[u32],
        diagnostics: &mut Vec<IndexRegionDiagnostic>,
    ) {
        let access = &self.accesses[access_index as usize];
        let shape = &self.tensors[access.tensor as usize].shape;
        let Some(elements) = shape.element_count() else {
            let access = TensorAccessId {
                owner: self.owner,
                index: access_index,
            };
            diagnostics.push(
                if self.accesses[access_index as usize].mode == AccessMode::Write {
                    IndexRegionDiagnostic::WriteOwnershipNotProven { access }
                } else {
                    IndexRegionDiagnostic::BoundsNotProven { access }
                },
            );
            return;
        };
        let mut seen =
            (access.mode == AccessMode::Write).then(|| vec![0_u64; elements.div_ceil(64)]);
        let extents: Vec<_> = access
            .domain
            .iter()
            .map(|d| self.dimensions[*d as usize].extent)
            .collect();
        let mut point = vec![0_u64; extents.len()];
        loop {
            let assignments: BTreeMap<_, _> = access
                .domain
                .iter()
                .copied()
                .zip(point.iter().copied())
                .collect();
            let evaluated = self.evaluate_expressions(expression_plan, &assignments);
            let mut linear = 0_usize;
            let mut in_bounds = true;
            for (coordinate, extent) in access.coordinates.iter().zip(shape.extents()) {
                let Some(value) = evaluated.get(coordinate).and_then(ToPrimitive::to_usize) else {
                    in_bounds = false;
                    break;
                };
                let Ok(axis_extent) = usize::try_from(extent.get()) else {
                    in_bounds = false;
                    break;
                };
                if value >= axis_extent {
                    in_bounds = false;
                    break;
                }
                let Some(next) = linear
                    .checked_mul(axis_extent)
                    .and_then(|base| base.checked_add(value))
                else {
                    in_bounds = false;
                    break;
                };
                linear = next;
            }
            if !in_bounds {
                diagnostics.push(IndexRegionDiagnostic::CoordinateOutOfBounds {
                    access: TensorAccessId {
                        owner: self.owner,
                        index: access_index,
                    },
                });
                return;
            }
            if let Some(bits) = &mut seen {
                let word = linear / 64;
                let mask = 1_u64 << (linear % 64);
                if bits.get(word).is_none_or(|bits| bits & mask != 0) {
                    diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                        access: TensorAccessId {
                            owner: self.owner,
                            index: access_index,
                        },
                    });
                    return;
                }
                bits[word] |= mask;
            }
            if !advance_point(&mut point, &extents) {
                break;
            }
        }
        if let Some(bits) = seen
            && (0..elements).any(|index| bits[index / 64] & (1_u64 << (index % 64)) == 0)
        {
            diagnostics.push(IndexRegionDiagnostic::WriteOwnershipNotProven {
                access: TensorAccessId {
                    owner: self.owner,
                    index: access_index,
                },
            });
        }
    }

    fn evaluate_expressions(
        &self,
        plan: &[u32],
        dimensions: &BTreeMap<u32, u64>,
    ) -> BTreeMap<u32, BigInt> {
        let mut values: BTreeMap<u32, BigInt> = BTreeMap::new();
        for index in plan {
            let expression = &self.expressions[*index as usize];
            let value = match &*expression.node {
                IndexNode::Constant(value) => value.0.clone(),
                IndexNode::Dimension(dimension) => {
                    BigInt::from(dimensions.get(dimension).copied().unwrap_or(0))
                }
                IndexNode::LinearCombination { constant, terms } => {
                    terms.iter().fold(constant.0.clone(), |sum, term| {
                        sum + &term.coefficient.0 * &values[&term.value]
                    })
                }
                IndexNode::FloorDiv { dividend, divisor } => {
                    values[dividend].div_floor(&BigInt::from(*divisor))
                }
                IndexNode::Modulo { dividend, divisor } => {
                    values[dividend].mod_floor(&BigInt::from(*divisor))
                }
            };
            values.insert(*index, value);
        }
        values
    }
    fn write_is_permutation(&self, access: &AccessData, shape: &Shape) -> bool {
        if access.coordinates.len() != access.domain.len() || shape.rank() != access.domain.len() {
            return false;
        }
        let mut seen = BTreeSet::new();
        for (coordinate, extent) in access.coordinates.iter().zip(shape.extents()) {
            let IndexNode::Dimension(d) = *self.expressions[*coordinate as usize].node else {
                return false;
            };
            if !access.domain.contains(&d)
                || self.dimensions[d as usize].extent != extent.get()
                || !seen.insert(d)
            {
                return false;
            }
        }
        true
    }

    fn compact(
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
            operations,
            values,
            outputs,
        })
    }

    fn compaction_order(
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

    fn finish_compaction(
        &self,
        compacted: CompactedRegion,
    ) -> Result<VerifiedIndexRegion, IndexRegionDiagnostic> {
        let identity_bytes = encoded_region_len(
            &compacted.dimensions,
            &compacted.tensors,
            &compacted.expressions,
            &compacted.accesses,
            &compacted.operations,
            &compacted.values,
            &compacted.outputs,
        );
        if identity_bytes > MAX_INDEX_REGION_IDENTITY_BYTES {
            return Err(IndexRegionDiagnostic::CanonicalIdentityLimit {
                bytes: identity_bytes,
                limit: MAX_INDEX_REGION_IDENTITY_BYTES,
            });
        }
        let identity = encode_region(&compacted, identity_bytes);
        let CompactedRegion {
            dimensions,
            tensors,
            expressions,
            accesses,
            operations,
            values,
            outputs,
        } = compacted;
        Ok(VerifiedIndexRegion {
            data: Arc::new(VerifiedIndexRegionData {
                owner: self.owner.verified_owner(),
                dimensions,
                tensors,
                expressions,
                accesses,
                operations,
                values,
                outputs,
                identity,
            }),
        })
    }

    fn remap_access(
        &self,
        old: u32,
        tensor_map: &BTreeMap<u32, u32>,
        expression_map: &BTreeMap<u32, u32>,
        dimension_map: &BTreeMap<u32, u32>,
    ) -> VerifiedAccessData {
        let access = &self.accesses[old as usize];
        let points = self.domain_points(&access.domain);
        let interval = points != 0
            && access
                .coordinates
                .iter()
                .zip(self.tensors[access.tensor as usize].shape.extents())
                .all(|(coordinate, extent)| {
                    self.expressions[*coordinate as usize]
                        .interval
                        .as_ref()
                        .is_some_and(|(minimum, maximum)| {
                            minimum >= &BigInt::zero() && maximum < &BigInt::from(extent.get())
                        })
                });
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
            bounds_proof: if points == 0 {
                BoundsProof::VacuousEmptyDomain
            } else if interval {
                BoundsProof::Interval
            } else {
                BoundsProof::Exhaustive { points }
            },
            ownership_proof: (access.mode == AccessMode::Write).then(|| {
                if self.write_is_permutation(access, &self.tensors[access.tensor as usize].shape) {
                    WriteOwnershipProof::CoordinatePermutation
                } else {
                    WriteOwnershipProof::Exhaustive { points }
                }
            }),
        }
    }

    fn reachable_values(&self) -> BTreeSet<u32> {
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
    fn mark_expr(&self, i: u32, reached: &mut BTreeSet<u32>) {
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
    fn alpha_dimension_order(&self) -> Vec<u32> {
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
        remaining.sort_by_key(|dimension| {
            let data = &self.dimensions[*dimension as usize];
            (data.role, data.extent)
        });
        order.extend(remaining);
        order
    }

    #[allow(clippy::too_many_arguments)]
    fn visit_value_dimensions(
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
        free.sort_by_key(|dimension| {
            let data = &self.dimensions[*dimension as usize];
            (data.role, data.extent)
        });
        for dimension in free {
            assign_dimension(dimension, order, assigned);
        }
    }

    fn visit_access_dimensions(
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
        domain.sort_by_key(|dimension| {
            let data = &self.dimensions[*dimension as usize];
            (data.role, data.extent)
        });
        for dimension in domain {
            assign_dimension(dimension, order, assigned);
        }
    }

    fn visit_expression_dimensions(
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

    fn alpha_blind_expr_key(&self, expression: u32) -> Vec<u8> {
        alpha_expr_key_impl(expression, &self.expressions, None, &self.dimensions)
    }

    fn alpha_expr_key(&self, expression: u32, dimensions: &BTreeMap<u32, u32>) -> Vec<u8> {
        alpha_expr_key_impl(
            expression,
            &self.expressions,
            Some(dimensions),
            &self.dimensions,
        )
    }

    fn alpha_operation_keys(&self, dimensions: &BTreeMap<u32, u32>) -> Vec<Vec<u8>> {
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
                    encode_len(&mut key, init.len());
                    encode_len(&mut key, contributors.len());
                    encode_reducer_body(&mut key, body);
                }
            }
            encode_len(&mut key, operation.operands.len());
            for operand in &operation.operands {
                encode_bytes(&mut key, &value_keys[*operand as usize]);
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

    fn alpha_access_key(&self, access: u32, dimensions: &BTreeMap<u32, u32>) -> Vec<u8> {
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
        encode_len(&mut key, role_ordinal);
        let mut domain: Vec<_> = data
            .domain
            .iter()
            .map(|dimension| dimensions[dimension])
            .collect();
        domain.sort_unstable();
        encode_u32s(&mut key, &domain);
        encode_len(&mut key, data.coordinates.len());
        for coordinate in &data.coordinates {
            encode_bytes(&mut key, &self.alpha_expr_key(*coordinate, dimensions));
        }
        key
    }
    fn intern_index(
        &mut self,
        node: IndexNode,
        dimensions: BTreeSet<u32>,
        class: IndexExprClass,
        interval: Option<(BigInt, BigInt)>,
        depth: u32,
    ) -> Result<IndexExprId, IndexBuildError> {
        limit(
            depth as usize,
            MAX_INDEX_EXPRESSION_DEPTH as usize,
            IndexLimitKind::IndexExpressionDepth,
        )?;
        check_index_node_integers(&node)?;
        let structural_key = Arc::new(structural_index_key(&node, &self.expressions));
        if let Some(index) = self.expression_intern.get(&structural_key) {
            return Ok(IndexExprId {
                owner: self.owner,
                index: *index,
            });
        }
        let key = Arc::new(node);
        limit(
            self.expressions.len() + 1,
            MAX_INDEX_EXPRESSIONS,
            IndexLimitKind::IndexExpressions,
        )?;
        let bytes = structural_key.len();
        limit(
            self.index_bytes + bytes,
            MAX_INDEX_CANONICAL_BYTES,
            IndexLimitKind::IndexCanonicalBytes,
        )?;
        let id = IndexExprId::from_len(self.owner, self.expressions.len()).ok_or(
            IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::IndexExpression,
            },
        )?;
        self.expression_intern
            .insert(structural_key.clone(), id.index);
        self.expressions.push(DraftIndexExpr {
            node: key,
            structural_key,
            dimensions,
            class,
            interval,
            depth,
        });
        self.index_bytes += bytes;
        Ok(id)
    }
    fn resolve_dimension(&self, id: DimensionId) -> Result<&DimensionData, IndexBuildError> {
        resolve(
            self.owner,
            id.owner,
            id.index,
            &self.dimensions,
            IndexEntityKind::Dimension,
        )
    }
    fn resolve_tensor(&self, id: TensorId) -> Result<&TensorData, IndexBuildError> {
        resolve(
            self.owner,
            id.owner,
            id.index,
            &self.tensors,
            IndexEntityKind::Tensor,
        )
    }
    fn resolve_expr(&self, id: IndexExprId) -> Result<&DraftIndexExpr, IndexBuildError> {
        resolve(
            self.owner,
            id.owner,
            id.index,
            &self.expressions,
            IndexEntityKind::IndexExpression,
        )
    }
    fn resolve_access(&self, id: TensorAccessId) -> Result<&AccessData, IndexBuildError> {
        resolve(
            self.owner,
            id.owner,
            id.index,
            &self.accesses,
            IndexEntityKind::TensorAccess,
        )
    }
    fn resolve_value(&self, id: ScalarValueId) -> Result<&ScalarValueData, IndexBuildError> {
        resolve(
            self.owner,
            id.owner,
            id.index,
            &self.values,
            IndexEntityKind::ScalarValue,
        )
        .map(|value| &value.data)
    }
    fn parallel_dimensions(&self) -> BTreeSet<u32> {
        self.dimensions
            .iter()
            .enumerate()
            .filter(|(_, dimension)| dimension.role == DomainRole::Parallel)
            .map(|(index, _)| bounded_index(index))
            .collect()
    }
}

fn resolve<T>(
    owner: BuilderId,
    actual: BuilderId,
    index: u32,
    values: &[T],
    entity: IndexEntityKind,
) -> Result<&T, IndexBuildError> {
    if owner != actual {
        return Err(invalid_handle(entity, true));
    }
    values
        .get(index as usize)
        .ok_or_else(|| invalid_handle(entity, false))
}
fn limit(actual: usize, max: usize, resource: IndexLimitKind) -> Result<(), IndexBuildError> {
    if actual > max {
        Err(IndexBuildError::StructuralLimit {
            resource,
            actual: actual as u128,
            limit: max as u128,
        })
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProofBudgetExcess {
    Cells { required: u128, limit: u64 },
    IntegerBytes { required: u128, limit: u64 },
}

impl ProofBudgetExcess {
    fn diagnostic(self) -> IndexRegionDiagnostic {
        match self {
            Self::Cells { required, limit } => IndexRegionDiagnostic::ProofResourceLimit {
                resource: ProofResource::Cells,
                required,
                limit,
            },
            Self::IntegerBytes { required, limit } => IndexRegionDiagnostic::ProofResourceLimit {
                resource: ProofResource::IntegerBytes,
                required,
                limit,
            },
        }
    }
}

fn with_admitted_proof_budget<T>(
    cells: u128,
    integer_bytes: u128,
    cell_limit: u64,
    byte_limit: u64,
    materialize: impl FnOnce() -> T,
) -> Result<T, ProofBudgetExcess> {
    if cells > u128::from(cell_limit) {
        return Err(ProofBudgetExcess::Cells {
            required: cells,
            limit: cell_limit,
        });
    }
    if integer_bytes > u128::from(byte_limit) {
        return Err(ProofBudgetExcess::IntegerBytes {
            required: integer_bytes,
            limit: byte_limit,
        });
    }
    Ok(materialize())
}
fn check_integer(value: &BigInt) -> Result<(), IndexBuildError> {
    let magnitude_bytes = usize::try_from(value.bits().div_ceil(8)).unwrap_or(usize::MAX);
    limit(
        magnitude_bytes,
        MAX_INDEX_INTEGER_BYTES,
        IndexLimitKind::IndexIntegerBytes,
    )
}
fn checked_index_product(left: &BigInt, right: &BigInt) -> Result<BigInt, IndexBuildError> {
    if left.is_zero() || right.is_zero() {
        return Ok(BigInt::zero());
    }
    let maximum_bits = (MAX_INDEX_INTEGER_BYTES as u64).saturating_mul(8);
    let upper_bits = left.bits().saturating_add(right.bits());
    if upper_bits > maximum_bits.saturating_add(1) {
        return Err(IndexBuildError::StructuralLimit {
            resource: IndexLimitKind::IndexIntegerBytes,
            actual: u128::from(upper_bits.div_ceil(8)),
            limit: MAX_INDEX_INTEGER_BYTES as u128,
        });
    }
    let product = left * right;
    check_integer(&product)?;
    Ok(product)
}

fn checked_index_add_assign(
    accumulator: &mut BigInt,
    addend: &BigInt,
) -> Result<(), IndexBuildError> {
    if addend.is_zero() {
        return Ok(());
    }
    let maximum_bits = (MAX_INDEX_INTEGER_BYTES as u64).saturating_mul(8);
    let upper_bits = accumulator.bits().max(addend.bits()).saturating_add(1);
    if accumulator.sign() == addend.sign() && upper_bits > maximum_bits.saturating_add(1) {
        return Err(IndexBuildError::StructuralLimit {
            resource: IndexLimitKind::IndexIntegerBytes,
            actual: u128::from(upper_bits.div_ceil(8)),
            limit: MAX_INDEX_INTEGER_BYTES as u128,
        });
    }
    let sum = &*accumulator + addend;
    check_integer(&sum)?;
    *accumulator = sum;
    Ok(())
}
fn check_index_node_integers(node: &IndexNode) -> Result<(), IndexBuildError> {
    match node {
        IndexNode::Constant(value) => check_integer(&value.0),
        IndexNode::LinearCombination { constant, terms } => {
            check_integer(&constant.0)?;
            for term in terms {
                check_integer(&term.coefficient.0)?;
            }
            Ok(())
        }
        IndexNode::Dimension(_) | IndexNode::FloorDiv { .. } | IndexNode::Modulo { .. } => Ok(()),
    }
}
fn too_many(entity: IndexEntityKind) -> IndexBuildError {
    IndexBuildError::TooManyEntities { entity }
}
fn retained_operation_bytes(
    key_bytes: usize,
    operand_count: usize,
    result_types: &[ResolvedValueType],
    free_dimension_count: usize,
) -> usize {
    let operation_storage = operand_count
        .saturating_add(result_types.len())
        .saturating_mul(4);
    let free_dimension_bytes = free_dimension_count.saturating_mul(4);
    key_bytes
        .saturating_add(operation_storage)
        .saturating_add(result_types.iter().fold(0_usize, |bytes, value_type| {
            bytes
                .saturating_add(key_bytes)
                .saturating_add(4)
                .saturating_add(value_type.canonical_encoding().as_bytes().len())
                .saturating_add(free_dimension_bytes)
        }))
}
fn minimum_retained_operation_bytes(
    key_bytes: usize,
    operand_count: usize,
    minimum_results: usize,
    free_dimension_count: usize,
) -> usize {
    operand_count
        .saturating_add(minimum_results)
        .saturating_mul(4)
        .saturating_add(key_bytes)
        .saturating_add(
            minimum_results.saturating_mul(
                key_bytes
                    .saturating_add(4)
                    .saturating_add(free_dimension_count.saturating_mul(4)),
            ),
        )
}
fn inference_capacity(
    value_count: usize,
    value_limit: usize,
    retained_bytes: usize,
    retained_byte_limit: usize,
    key_bytes: usize,
    operand_count: usize,
    free_dimension_count: usize,
) -> ScalarInferenceCapacity {
    let fixed_bytes = key_bytes.saturating_add(operand_count.saturating_mul(4));
    ScalarInferenceCapacity {
        result_slots: value_limit.saturating_sub(value_count),
        result_count_before: value_count,
        result_limit: value_limit,
        retained_bytes: retained_byte_limit
            .saturating_sub(retained_bytes)
            .saturating_sub(fixed_bytes),
        retained_bytes_before: retained_bytes.saturating_add(fixed_bytes),
        retained_byte_limit,
        per_result_overhead: key_bytes
            .saturating_add(8)
            .saturating_add(free_dimension_count.saturating_mul(4)),
        byte_multiplier: 1,
    }
}
fn map_scalar_apply_error(error: ScalarApplyError) -> IndexBuildError {
    match error {
        ScalarApplyError::Authority(error) => IndexBuildError::from(error),
        ScalarApplyError::Host(ScalarInferenceHostFailure::ResultSlots { actual, limit }) => {
            IndexBuildError::StructuralLimit {
                resource: IndexLimitKind::ScalarValues,
                actual: actual as u128,
                limit: limit as u128,
            }
        }
        ScalarApplyError::Host(ScalarInferenceHostFailure::CanonicalBytes { actual, limit }) => {
            IndexBuildError::StructuralLimit {
                resource: IndexLimitKind::ScalarCanonicalBytes,
                actual: actual as u128,
                limit: limit as u128,
            }
        }
    }
}
fn map_order(order: &[u32]) -> BTreeMap<u32, u32> {
    order
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, bounded_index(new)))
        .collect()
}
fn bounded_index(index: usize) -> u32 {
    u32::try_from(index).expect("governed region limits fit u32")
}
fn stage_reducer_results(
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
fn advance_point(point: &mut [u64], extents: &[u64]) -> bool {
    for axis in (0..point.len()).rev() {
        point[axis] += 1;
        if point[axis] < extents[axis] {
            return true;
        }
        point[axis] = 0;
    }
    false
}
fn accumulate_linear_term(
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
    match &*expression.node {
        IndexNode::Constant(inner) => {
            let product = checked_index_product(coefficient, &inner.0)?;
            checked_index_add_assign(constant, &product)?;
        }
        IndexNode::LinearCombination {
            constant: inner_constant,
            terms,
        } => {
            let product = checked_index_product(coefficient, &inner_constant.0)?;
            checked_index_add_assign(constant, &product)?;
            for term in terms {
                let nested_coefficient = checked_index_product(coefficient, &term.coefficient.0)?;
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
fn compact_reducer_body(
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
fn minimum_reducer_body(
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
fn interval_linear(
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
fn structural_index_key(node: &IndexNode, expressions: &[DraftIndexExpr]) -> Vec<u8> {
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
            encode_len(&mut output, terms.len());
            for term in terms {
                term.coefficient.encode(&mut output);
                encode_bytes(
                    &mut output,
                    &expressions[term.value as usize].structural_key,
                );
            }
        }
        IndexNode::FloorDiv { dividend, divisor } => {
            output.push(4);
            encode_bytes(&mut output, &expressions[*dividend as usize].structural_key);
            output.extend_from_slice(&divisor.to_be_bytes());
        }
        IndexNode::Modulo { dividend, divisor } => {
            output.push(5);
            encode_bytes(&mut output, &expressions[*dividend as usize].structural_key);
            output.extend_from_slice(&divisor.to_be_bytes());
        }
    }
    output
}
fn access_read_key(
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
    encode_len(&mut output, role_ordinal);
    encode_u32s(&mut output, &data.domain);
    encode_len(&mut output, data.coordinates.len());
    for coordinate in &data.coordinates {
        encode_bytes(
            &mut output,
            &expressions[*coordinate as usize].structural_key,
        );
    }
    output
}
fn nested_operation_key(
    key: &ScalarOpKey,
    attributes: &ScalarAttributes,
    operands: &[u32],
    value_keys: &[Arc<Vec<u8>>],
) -> Vec<u8> {
    let mut output = b"tiler.index.reducer-apply.v2\0".to_vec();
    encode_key(&mut output, key);
    encode_canonical(&mut output, attributes.value());
    encode_len(&mut output, operands.len());
    for operand in operands {
        encode_bytes(&mut output, &value_keys[*operand as usize]);
    }
    output
}
fn apply_operation_key(
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
    encode_len(&mut output, operands.len());
    for operand in operands {
        encode_bytes(&mut output, &values[operand.as_usize()].structural_key);
    }
    encode_u32s(
        &mut output,
        &free_dimensions.iter().copied().collect::<Vec<_>>(),
    );
    output
}
fn operation_structural_key(
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
            encode_len(&mut output, init.len());
            encode_len(&mut output, contributors.len());
            encode_reducer_body(&mut output, body);
        }
    }
    encode_len(&mut output, operands.len());
    for operand in operands {
        encode_bytes(&mut output, &values[operand.as_usize()].structural_key);
    }
    encode_u32s(
        &mut output,
        &free_dimensions.iter().copied().collect::<Vec<_>>(),
    );
    output
}
fn encode_reducer_body(output: &mut Vec<u8>, body: &ScalarReducerBodyData) {
    encode_len(output, body.values.len());
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
        encode_bytes(output, value.value_type.canonical_encoding().as_bytes());
    }
    encode_len(output, body.operations.len());
    for operation in &body.operations {
        encode_key(output, &operation.key);
        encode_canonical(output, operation.attributes.value());
        encode_u32s(output, &operation.operands);
        encode_u32s(output, &operation.results);
    }
    encode_u32s(output, &body.yields);
}
fn assign_dimension(dimension: u32, order: &mut Vec<u32>, assigned: &mut BTreeSet<u32>) {
    if assigned.insert(dimension) {
        order.push(dimension);
    }
}

fn alpha_expr_key_impl(
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
                output.extend_from_slice(&data.extent.to_be_bytes());
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
                    encode_bytes(
                        &mut encoded,
                        &alpha_expr_key_impl(term.value, expressions, dimension_map, dimensions),
                    );
                    encoded
                })
                .collect::<Vec<_>>();
            encoded_terms.sort();
            encode_len(&mut output, encoded_terms.len());
            for term in encoded_terms {
                encode_bytes(&mut output, &term);
            }
        }
        IndexNode::FloorDiv { dividend, divisor } => {
            output.push(4);
            encode_bytes(
                &mut output,
                &alpha_expr_key_impl(*dividend, expressions, dimension_map, dimensions),
            );
            output.extend_from_slice(&divisor.to_be_bytes());
        }
        IndexNode::Modulo { dividend, divisor } => {
            output.push(5);
            encode_bytes(
                &mut output,
                &alpha_expr_key_impl(*dividend, expressions, dimension_map, dimensions),
            );
            output.extend_from_slice(&divisor.to_be_bytes());
        }
    }
    output
}

fn remap_node(
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
fn remap_operation(
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

fn encode_region(
    compacted: &CompactedRegion,
    exact_capacity: usize,
) -> CanonicalIndexRegionIdentity {
    let CompactedRegion {
        dimensions,
        tensors,
        expressions,
        accesses,
        operations,
        values,
        outputs,
    } = compacted;
    let mut out = Vec::with_capacity(exact_capacity);
    out.extend_from_slice(b"tiler.index-region.v4\0");
    encode_len(&mut out, dimensions.len());
    for d in dimensions {
        out.push(match d.role {
            DomainRole::Parallel => 1,
            DomainRole::Reduction => 2,
        });
        out.extend_from_slice(&d.extent.to_be_bytes());
    }
    encode_len(&mut out, tensors.len());
    for t in tensors {
        out.push(match t.role {
            TensorRole::Input => 1,
            TensorRole::Output => 2,
        });
        encode_bytes(&mut out, t.value_type.canonical_encoding().as_bytes());
        encode_len(&mut out, t.shape.rank());
        for e in t.shape.extents() {
            out.extend_from_slice(&e.get().to_be_bytes());
        }
    }
    encode_len(&mut out, expressions.len());
    for e in expressions {
        encode_index_node(&mut out, &e.node);
    }
    encode_len(&mut out, accesses.len());
    for a in accesses {
        out.push(match a.mode {
            AccessMode::Read => 1,
            AccessMode::Write => 2,
        });
        out.extend_from_slice(&a.tensor.to_be_bytes());
        encode_u32s(&mut out, &a.domain);
        encode_u32s(&mut out, &a.coordinates);
    }
    encode_len(&mut out, operations.len());
    for op in operations {
        encode_operation_kind(&mut out, &op.kind);
        encode_u32s(&mut out, &op.operands);
        encode_u32s(&mut out, &op.results);
    }
    encode_len(&mut out, values.len());
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
        encode_bytes(&mut out, v.value_type.canonical_encoding().as_bytes());
        encode_u32s(
            &mut out,
            &v.free_dimensions.iter().copied().collect::<Vec<_>>(),
        );
    }
    encode_len(&mut out, outputs.len());
    for o in outputs {
        out.extend_from_slice(&o.access.to_be_bytes());
        out.extend_from_slice(&o.value.to_be_bytes());
    }
    debug_assert_eq!(out.len(), exact_capacity);
    CanonicalIndexRegionIdentity(out)
}

fn encoded_region_len(
    dimensions: &[DimensionData],
    tensors: &[TensorData],
    expressions: &[IndexExprData],
    accesses: &[VerifiedAccessData],
    operations: &[ScalarOperationData],
    values: &[ScalarValueData],
    outputs: &[OutputData],
) -> usize {
    let mut bytes = b"tiler.index-region.v4\0".len() + 8;
    bytes = bytes.saturating_add(dimensions.len().saturating_mul(9));
    bytes = bytes.saturating_add(8);
    for tensor in tensors {
        bytes = bytes
            .saturating_add(1)
            .saturating_add(encoded_bytes_len(
                tensor.value_type.canonical_encoding().as_bytes().len(),
            ))
            .saturating_add(8)
            .saturating_add(tensor.shape.rank().saturating_mul(8));
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

fn encoded_index_node_len(node: &IndexNode) -> usize {
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

fn encoded_integer_len(value: &IndexInteger) -> usize {
    let magnitude = usize::try_from(value.0.bits().div_ceil(8)).unwrap_or(usize::MAX);
    9_usize.saturating_add(magnitude)
}

fn encoded_operation_kind_len(kind: &ScalarOperationKindData) -> usize {
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

fn encoded_reducer_body_len(body: &ScalarReducerBodyData) -> usize {
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

fn encoded_reducer_parameter_len(value_type: &ResolvedValueType) -> usize {
    encoded_reducer_parameter_source_len().saturating_add(encoded_bytes_len(
        value_type.canonical_encoding().as_bytes().len(),
    ))
}

fn encoded_reducer_value_len(value: &ReducerBodyValueData) -> usize {
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

fn encoded_reducer_operation_base_len(
    key: &ScalarOpKey,
    attributes: &ScalarAttributes,
    operand_count: usize,
) -> usize {
    encoded_key_len(key)
        .saturating_add(attributes.value().encoded_len())
        .saturating_add(encoded_u32s_len(operand_count))
        .saturating_add(8)
}

fn encoded_reducer_operation_len(operation: &ReducerBodyOperationData) -> usize {
    encoded_reducer_operation_base_len(
        &operation.key,
        &operation.attributes,
        operation.operands.len(),
    )
    .saturating_add(operation.results.len().saturating_mul(4))
}

const fn encoded_reducer_operation_result_overhead() -> usize {
    encoded_reducer_operation_result_source_len()
        .saturating_add(8) // Encoded-byte length prefix.
        .saturating_add(4) // Result-list index.
}

const fn encoded_reducer_parameter_source_len() -> usize {
    5
}

const fn encoded_reducer_operation_result_source_len() -> usize {
    9
}

fn encoded_reducer_operation_result_increment(value_type: &ResolvedValueType) -> usize {
    value_type
        .canonical_encoding()
        .as_bytes()
        .len()
        .saturating_add(encoded_reducer_operation_result_overhead())
}

fn encoded_key_len(key: &ScalarOpKey) -> usize {
    encoded_bytes_len(key.namespace().len())
        .saturating_add(encoded_bytes_len(key.name().len()))
        .saturating_add(4)
}

const fn encoded_bytes_len(bytes: usize) -> usize {
    8_usize.saturating_add(bytes)
}

const fn encoded_u32s_len(values: usize) -> usize {
    8_usize.saturating_add(values.saturating_mul(4))
}

fn encode_index_node(out: &mut Vec<u8>, node: &IndexNode) {
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
            encode_len(out, terms.len());
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
fn encode_operation_kind(out: &mut Vec<u8>, kind: &ScalarOperationKindData) {
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
            encode_len(out, body.values.len());
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
                encode_bytes(out, value.value_type.canonical_encoding().as_bytes());
            }
            encode_len(out, body.operations.len());
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

fn encode_u32s(out: &mut Vec<u8>, values: &[u32]) {
    encode_len(out, values.len());
    for v in values {
        out.extend_from_slice(&v.to_be_bytes());
    }
}

#[cfg(test)]
mod resource_order_tests {
    use std::cell::Cell;
    use std::error::Error as _;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{
        ProofBudgetExcess, ReducerBodyBudget, admit_reducer_body_append, encode_reducer_body,
        encoded_reducer_body_len, encoded_reducer_operation_base_len,
        encoded_reducer_operation_result_increment, encoded_reducer_operation_result_overhead,
        encoded_reducer_parameter_len, map_scalar_apply_error, minimum_reducer_body,
        with_admitted_proof_budget,
    };
    use crate::index::model::{
        ReducerBodyOperationData, ReducerBodyValueData, ReducerBodyValueSource,
        ScalarReducerBodyData,
    };
    use crate::index::scalar::{ScalarApplyError, ScalarInferenceHostFailure};
    use crate::index::{
        DomainRole, FrozenScalarRegistry, IndexBuildError, IndexLimitKind, IndexRegionBuilder,
        MAX_SCALAR_CANONICAL_BYTES, ScalarArity, ScalarAttributeSchema, ScalarAttributes,
        ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs, ScalarInferenceRequest,
        ScalarOpKey, ScalarOperationContract, ScalarOperationDefinition, ScalarOperationInferencer,
        ScalarRegistryBuilder, ScalarResultIndex, TensorRole,
    };
    use crate::semantic::{
        CanonicalValue, NormativeDefinitionRef, ProviderIdentity, RegistryError, ResolvedValueType,
        SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
        TypeDefinitionFacts, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
    };
    use crate::shape::{Extent, Shape};

    fn reducer_test_type() -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("test", "reducer-value", 1).unwrap())
    }

    struct ReducerTestTypes;
    impl SemanticRegistryProvider for ReducerTestTypes {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "reducer-types", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("test", "reducer-value", 1).unwrap()),
                NormativeDefinitionRef::new("urn:test:reducer-value:v1").unwrap(),
                TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            ))
        }
    }

    struct FirstOperand {
        calls: Arc<AtomicUsize>,
    }
    impl ScalarOperationInferencer for FirstOperand {
        fn infer(
            &self,
            request: ScalarInferenceRequest<'_>,
            outputs: &mut ScalarInferenceOutputs,
        ) -> Result<(), ScalarInferenceError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            outputs.try_push(request.operands()[0].clone())
        }
    }

    fn reducer_test_registry(calls: Arc<AtomicUsize>) -> FrozenScalarRegistry {
        let mut semantic = SemanticRegistryBuilder::new();
        semantic.register_provider(&ReducerTestTypes).unwrap();
        let mut scalar = ScalarRegistryBuilder::new(semantic.freeze().unwrap());
        let key = ScalarOpKey::new("test", "step", 1).unwrap();
        scalar
            .register(
                ProviderIdentity::new("test", "reducer-scalars", 1).unwrap(),
                ScalarOperationDefinition::new(
                    key,
                    NormativeDefinitionRef::new("urn:test:step:v1").unwrap(),
                    ScalarOperationContract::new(
                        ScalarAttributeSchema::empty(),
                        ScalarArity::exact(1).unwrap(),
                        ScalarArity::exact(1).unwrap(),
                        ScalarEffect::Pure,
                        CanonicalValue::record([]).unwrap(),
                        CanonicalValue::record([]).unwrap(),
                    ),
                    Arc::new(FirstOperand { calls }),
                ),
            )
            .unwrap();
        scalar.freeze()
    }

    #[test]
    fn proof_materialization_runs_only_after_aggregate_admission() {
        let materializations = Cell::new(0_u32);
        let rejected = with_admitted_proof_budget(11, 8, 10, 10, || {
            materializations.set(materializations.get() + 1);
        });
        assert_eq!(
            rejected,
            Err(ProofBudgetExcess::Cells {
                required: 11,
                limit: 10,
            })
        );
        assert_eq!(materializations.get(), 0);

        with_admitted_proof_budget(10, 10, 10, 10, || {
            materializations.set(materializations.get() + 1);
        })
        .unwrap();
        assert_eq!(materializations.get(), 1);
    }

    #[test]
    fn parent_reducer_budget_rejects_before_retaining_the_append() {
        let commits = Cell::new(0_u32);
        let budget = ReducerBodyBudget {
            parent_bytes_without_body: MAX_SCALAR_CANONICAL_BYTES - 20,
            body_multiplier: 2,
            maximum_encoded_bytes: 10,
        };
        let error =
            admit_reducer_body_append(budget, 11, || commits.set(commits.get() + 1)).unwrap_err();
        assert_eq!(commits.get(), 0);
        assert_eq!(
            error,
            IndexBuildError::StructuralLimit {
                resource: IndexLimitKind::ScalarCanonicalBytes,
                actual: (MAX_SCALAR_CANONICAL_BYTES + 2) as u128,
                limit: MAX_SCALAR_CANONICAL_BYTES as u128,
            }
        );

        admit_reducer_body_append(budget, 10, || commits.set(commits.get() + 1)).unwrap();
        assert_eq!(commits.get(), 1);
    }

    #[test]
    fn enclosing_capacity_errors_have_no_provider_source() {
        for error in [
            ScalarApplyError::Host(ScalarInferenceHostFailure::ResultSlots {
                actual: 65_537,
                limit: 65_536,
            }),
            ScalarApplyError::Host(ScalarInferenceHostFailure::CanonicalBytes {
                actual: MAX_SCALAR_CANONICAL_BYTES + 1,
                limit: MAX_SCALAR_CANONICAL_BYTES,
            }),
        ] {
            let mapped = map_scalar_apply_error(error);
            assert!(matches!(mapped, IndexBuildError::StructuralLimit { .. }));
            assert!(mapped.source().is_none());
        }
    }

    #[test]
    fn incremental_reducer_accounting_matches_the_final_encoder() {
        let value_type = reducer_test_type();
        let key = ScalarOpKey::new("test", "step", 1).unwrap();
        let attributes = ScalarAttributes::empty();
        let body = ScalarReducerBodyData {
            values: vec![
                ReducerBodyValueData {
                    source: ReducerBodyValueSource::StateParameter(0),
                    value_type: value_type.clone(),
                },
                ReducerBodyValueData {
                    source: ReducerBodyValueSource::ContributorParameter(0),
                    value_type: value_type.clone(),
                },
                ReducerBodyValueData {
                    source: ReducerBodyValueSource::OperationResult {
                        operation: 0,
                        result: ScalarResultIndex::from_usize(0).unwrap(),
                    },
                    value_type: value_type.clone(),
                },
            ],
            operations: vec![ReducerBodyOperationData {
                key: key.clone(),
                attributes: attributes.clone(),
                operands: vec![0, 1],
                results: vec![2],
            }],
            yields: vec![2],
        };
        let incrementally_accounted = 24_usize
            .saturating_add(encoded_reducer_parameter_len(&value_type).saturating_mul(2))
            .saturating_add(encoded_reducer_operation_base_len(&key, &attributes, 2))
            .saturating_add(encoded_reducer_operation_result_increment(&value_type))
            .saturating_add(4);
        let mut encoded = Vec::new();
        encode_reducer_body(&mut encoded, &body);
        assert_eq!(incrementally_accounted, encoded_reducer_body_len(&body));
        assert_eq!(incrementally_accounted, encoded.len());
    }

    #[test]
    fn near_parent_limit_reducer_failure_leaves_the_outer_builder_unchanged() {
        let inference_calls = Arc::new(AtomicUsize::new(0));
        let mut builder =
            IndexRegionBuilder::new(reducer_test_registry(Arc::clone(&inference_calls))).unwrap();
        let reduction = builder
            .dimension(DomainRole::Reduction, Extent::new(2))
            .unwrap();
        let coordinate = builder.dimension_expr(reduction).unwrap();
        let input = builder
            .tensor(
                TensorRole::Input,
                reducer_test_type(),
                Shape::new([Extent::new(2)]),
            )
            .unwrap();
        let contributor = builder.read(input, &[reduction], &[coordinate]).unwrap();
        let init_input = builder
            .tensor(TensorRole::Input, reducer_test_type(), Shape::new([]))
            .unwrap();
        let init = builder.read(init_input, &[], &[]).unwrap();
        let inputs = builder
            .prepare_reduction_inputs(&[reduction], &[init], &[contributor])
            .unwrap();
        let minimum_body_bytes =
            encoded_reducer_body_len(&minimum_reducer_body(&inputs.init, &inputs.contributors));
        let operation_base = encoded_reducer_operation_base_len(
            &ScalarOpKey::new("test", "step", 1).unwrap(),
            &ScalarAttributes::empty(),
            1,
        );
        let encoded_before_results = minimum_body_bytes
            .checked_sub(4)
            .unwrap()
            .checked_add(operation_base)
            .unwrap();
        let minimum_fixed_results = encoded_before_results
            .checked_add(encoded_reducer_operation_result_overhead())
            .unwrap();
        let target_capacity = minimum_fixed_results - 1;
        assert!(target_capacity >= minimum_body_bytes);
        assert!(target_capacity > encoded_before_results);
        assert!(inputs.body_budget.maximum_encoded_bytes >= target_capacity);
        let removable_headroom = inputs
            .body_budget
            .maximum_encoded_bytes
            .checked_sub(target_capacity)
            .unwrap();
        builder.scalar_bytes = builder
            .scalar_bytes
            .saturating_add(removable_headroom.saturating_mul(inputs.body_budget.body_multiplier));
        let before = (
            builder.operations.len(),
            builder.values.len(),
            builder.scalar_bytes,
        );
        let callback_calls = Cell::new(0_u32);
        let error = builder
            .reduce(&[reduction], &[init], &[contributor], |body| {
                callback_calls.set(callback_calls.get() + 1);
                let state = body.state(0).unwrap();
                body.apply(
                    ScalarOpKey::new("test", "step", 1).unwrap(),
                    ScalarAttributes::empty(),
                    &[state],
                )?;
                unreachable!("the parent budget must reject the nested append")
            })
            .unwrap_err();
        assert!(matches!(
            error,
            IndexBuildError::StructuralLimit {
                resource: IndexLimitKind::ScalarCanonicalBytes,
                ..
            }
        ));
        assert_eq!(callback_calls.get(), 1);
        assert_eq!(inference_calls.load(Ordering::Relaxed), 0);
        assert_eq!(
            before,
            (
                builder.operations.len(),
                builder.values.len(),
                builder.scalar_bytes,
            )
        );
    }
}
