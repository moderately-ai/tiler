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
use super::scalar::{encode_bytes, encode_canonical, encode_key, encode_len};
use super::{
    AccessMode, CanonicalIndexRegionIdentity, DimensionId, DomainRole, FrozenScalarRegistry,
    IndexBuildError, IndexEntityKind, IndexExprClass, IndexExprId, IndexInteger, IndexLimitKind,
    IndexRegionBuildError, IndexRegionDiagnostic, MAX_ACCESS_CANONICAL_BYTES,
    MAX_BOUNDARY_CANONICAL_BYTES, MAX_BOUNDARY_TENSORS, MAX_DOMAIN_DIMENSIONS,
    MAX_EXHAUSTIVE_PROOF_BYTES, MAX_EXHAUSTIVE_PROOF_CELLS, MAX_INDEX_CANONICAL_BYTES,
    MAX_INDEX_EXPRESSION_OPERANDS, MAX_INDEX_EXPRESSIONS, MAX_INDEX_REGION_IDENTITY_BYTES,
    MAX_OUTPUT_ROOTS, MAX_SCALAR_CANONICAL_BYTES, MAX_SCALAR_EXPRESSION_DEPTH,
    MAX_SCALAR_EXPRESSIONS, MAX_SCALAR_OPERANDS, MAX_TENSOR_ACCESSES, MAX_TENSOR_RANK,
    ProofResource, ReductionTraversal, ScalarAttributes, ScalarOpKey, ScalarOperationId,
    ScalarResultIndex, ScalarValueId, TensorAccessId, TensorId, TensorRole, VerifiedIndexRegion,
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
    structural_key: Arc<Vec<u8>>,
}

struct StagedReducerResults {
    values: Vec<ReducerBodyValueData>,
    keys: Vec<Arc<Vec<u8>>>,
    indices: Vec<u32>,
    ids: Vec<ReducerScalarValueId>,
}

struct CompactionOrder {
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
    ) -> Result<Self, IndexBuildError> {
        let owner = next_builder_id().ok_or(IndexBuildError::BuilderIdentityExhausted)?;
        limit(
            state.len().saturating_add(contributors.len()),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarValues,
        )?;
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
        let result_types = self.registry.infer(&key, &operand_types, &attributes)?;
        let depth = operands
            .iter()
            .map(|operand| self.value_depths[operand.index as usize].saturating_add(1))
            .max()
            .unwrap_or(0);
        limit(
            operands.len(),
            MAX_SCALAR_OPERANDS,
            IndexLimitKind::ScalarOperands,
        )?;
        limit(
            self.operations.len().saturating_add(1),
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarOperations,
        )?;
        limit(
            self.values.len().saturating_add(result_types.len()),
            MAX_SCALAR_EXPRESSIONS,
            IndexLimitKind::ScalarValues,
        )?;
        limit(
            depth as usize,
            MAX_SCALAR_EXPRESSION_DEPTH as usize,
            IndexLimitKind::ScalarValues,
        )?;
        let added_bytes =
            retained_operation_bytes(structural_key.len(), operands.len(), &result_types, 0);
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
        Ok(ReducerScalarResults(staged.ids))
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
        let indices: Vec<_> = values
            .iter()
            .map(|id| {
                self.resolve(*id)?;
                Ok(id.index)
            })
            .collect::<Result<_, IndexBuildError>>()?;
        self.yields = Some(indices);
        Ok(())
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
            );
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
            let expression = &self.expressions[term.value as usize];
            dimensions.extend(&expression.dimensions);
            if expression.class == IndexExprClass::QuasiAffine {
                class = IndexExprClass::QuasiAffine;
            }
            depth = depth.max(expression.depth.saturating_add(1));
        }
        let interval = interval_linear(&normalized_constant, &terms, &self.expressions);
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
        let types: Vec<_> = operand_data.iter().map(|v| v.value_type.clone()).collect();
        let result_types = self.registry.infer(&key, &types, &attributes)?;
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
        if dimensions.is_empty() {
            return Err(IndexBuildError::EmptyReductionDimensions);
        }
        if init.is_empty() {
            return Err(IndexBuildError::EmptyReductionState);
        }
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
        let state_types: Vec<_> = init_data.iter().map(|v| v.value_type.clone()).collect();
        let contributor_types: Vec<_> = contributor_data
            .iter()
            .map(|v| v.value_type.clone())
            .collect();
        let mut body =
            ScalarReducerBodyBuilder::new(&self.registry, &state_types, &contributor_types)?;
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
        let mut free: BTreeSet<_> = init_data
            .iter()
            .chain(&contributor_data)
            .flat_map(|v| v.free_dimensions.iter().copied())
            .collect();
        for d in &bound {
            free.remove(d);
        }
        let mut operands = init.to_vec();
        operands.extend_from_slice(contributors);
        let operation_keys = body.operation_keys;
        let operation_depths = body.operation_depths;
        let uncompact_body = ScalarReducerBodyData {
            values: body.values,
            operations: body.operations,
            yields,
        };
        let nested = compact_reducer_body(&uncompact_body, &operation_keys, &operation_depths);
        self.push_operation(
            ScalarOperationKindData::Reduce {
                dimensions: dimensions.iter().map(|d| d.index).collect(),
                traversal: ReductionTraversal::ExactLexicographicLeftFold,
                init: init.iter().map(|v| v.index).collect(),
                contributors: contributors.iter().map(|v| v.index).collect(),
                body: nested,
            },
            &operands,
            state_types,
            &free,
        )
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
            IndexLimitKind::ScalarValues,
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
            structural_key: structural_key.clone(),
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
        let mut exhaustive = Vec::new();
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
                let mut reached = BTreeSet::new();
                for coordinate in &access.coordinates {
                    self.mark_expr(*coordinate, &mut reached);
                }
                exhaustive.push((
                    *access_index,
                    points,
                    reached.into_iter().collect::<Vec<_>>(),
                ));
            }
        }
        let cells: u128 = exhaustive.iter().fold(0_u128, |total, (_, points, plan)| {
            total.saturating_add(u128::from(*points).saturating_mul(plan.len() as u128))
        });
        let integer_bytes: u128 =
            exhaustive
                .iter()
                .fold(0_u128, |total, (access, points, plan)| {
                    let bytes_per_point = plan.iter().fold(0_u128, |bytes, expression| {
                        bytes.saturating_add(self.expression_integer_bytes(*expression))
                    });
                    let coordinate_bytes =
                        u128::from(*points).saturating_mul(bytes_per_point.max(1));
                    let dense_bytes = if self.accesses[*access as usize].mode == AccessMode::Write {
                        self.tensors[self.accesses[*access as usize].tensor as usize]
                            .shape
                            .element_count()
                            .map_or(u128::MAX, |elements| {
                                elements.div_ceil(64).saturating_mul(8) as u128
                            })
                    } else {
                        0
                    };
                    total.saturating_add(coordinate_bytes.saturating_add(dense_bytes))
                });
        if cells > u128::from(MAX_EXHAUSTIVE_PROOF_CELLS) {
            diagnostics.push(IndexRegionDiagnostic::ProofResourceLimit {
                resource: ProofResource::Cells,
                required: cells,
                limit: MAX_EXHAUSTIVE_PROOF_CELLS,
            });
            return;
        }
        if integer_bytes > u128::from(MAX_EXHAUSTIVE_PROOF_BYTES) {
            diagnostics.push(IndexRegionDiagnostic::ProofResourceLimit {
                resource: ProofResource::IntegerBytes,
                required: integer_bytes,
                limit: MAX_EXHAUSTIVE_PROOF_BYTES,
            });
            return;
        }
        for (access, _, plan) in exhaustive {
            self.verify_access_exhaustively(access, &plan, diagnostics);
        }
    }

    fn domain_points(&self, domain: &[u32]) -> u64 {
        domain
            .iter()
            .try_fold(1_u64, |points, dimension| {
                points.checked_mul(self.dimensions[*dimension as usize].extent)
            })
            .unwrap_or(u64::MAX)
    }

    fn expression_integer_bytes(&self, expression: u32) -> u128 {
        let data = &self.expressions[expression as usize];
        if data.interval.is_none() {
            return u128::MAX;
        }
        let magnitude_bound = if let IndexNode::LinearCombination { constant, terms } = &*data.node
        {
            terms.iter().fold(constant.0.abs(), |bound, term| {
                let (minimum, maximum) = self.expressions[term.value as usize]
                    .interval
                    .as_ref()
                    .expect("a linear interval requires every child interval");
                let child_bound = minimum.abs().max(maximum.abs());
                bound + term.coefficient.0.abs() * child_bound
            })
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
                    node: remap_node(&data.node, &expr_map),
                    class: data.class,
                }
            })
            .collect();
        let accesses: Vec<_> = access_order
            .iter()
            .map(|old| self.remap_access(*old, &tensor_map, &expr_map))
            .collect();
        let operations: Vec<_> = op_order
            .iter()
            .map(|old| remap_operation(&self.operations[*old as usize].data, &value_map, &op_map))
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
                    free_dimensions: value.free_dimensions.clone(),
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
        self.finish_compaction(tensors, expressions, accesses, operations, values, outputs)
    }

    fn compaction_order(
        &self,
        reachable_values: BTreeSet<u32>,
        reachable_accesses: BTreeSet<u32>,
    ) -> CompactionOrder {
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
        expr_order.sort_by_key(|i| (self.expressions[*i as usize].depth, self.expr_key(*i)));
        let expr_map = map_order(&expr_order);
        let mut access_order: Vec<_> = reachable_accesses.into_iter().collect();
        access_order.sort_by_key(|i| {
            let a = &self.accesses[*i as usize];
            (
                tensor_map[&a.tensor],
                a.mode,
                a.domain.clone(),
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
        let mut op_order: Vec<_> = reachable_ops.into_iter().collect();
        op_order.sort_by_key(|i| {
            (
                self.operations[*i as usize].depth,
                self.operations[*i as usize].structural_key.clone(),
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
        tensors: Vec<TensorData>,
        expressions: Vec<IndexExprData>,
        accesses: Vec<VerifiedAccessData>,
        operations: Vec<ScalarOperationData>,
        values: Vec<ScalarValueData>,
        outputs: Vec<OutputData>,
    ) -> Result<VerifiedIndexRegion, IndexRegionDiagnostic> {
        let dimensions = self.dimensions.clone();
        let identity = encode_region(
            &dimensions,
            &tensors,
            &expressions,
            &accesses,
            &operations,
            &values,
            &outputs,
        );
        if identity.as_bytes().len() > MAX_INDEX_REGION_IDENTITY_BYTES {
            return Err(IndexRegionDiagnostic::CanonicalIdentityLimit {
                bytes: identity.as_bytes().len(),
                limit: MAX_INDEX_REGION_IDENTITY_BYTES,
            });
        }
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
            domain: access.domain.clone(),
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
        if !reached.insert(i) {
            return;
        }
        match &*self.expressions[i as usize].node {
            IndexNode::LinearCombination { terms, .. } => {
                for t in terms {
                    self.mark_expr(t.value, reached);
                }
            }
            IndexNode::FloorDiv { dividend, .. } | IndexNode::Modulo { dividend, .. } => {
                self.mark_expr(*dividend, reached);
            }
            _ => {}
        }
    }
    fn expr_key(&self, i: u32) -> Vec<u8> {
        self.expressions[i as usize].structural_key.as_ref().clone()
    }
    fn intern_index(
        &mut self,
        node: IndexNode,
        dimensions: BTreeSet<u32>,
        class: IndexExprClass,
        interval: Option<(BigInt, BigInt)>,
        depth: u32,
    ) -> Result<IndexExprId, IndexBuildError> {
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
) {
    if coefficient.is_zero() {
        return;
    }
    let expression = &expressions[value as usize];
    match &*expression.node {
        IndexNode::Constant(inner) => *constant += coefficient * &inner.0,
        IndexNode::LinearCombination {
            constant: inner_constant,
            terms,
        } => {
            *constant += coefficient * &inner_constant.0;
            for term in terms {
                accumulate_linear_term(
                    constant,
                    coefficients,
                    &(coefficient * &term.coefficient.0),
                    term.value,
                    expressions,
                );
            }
        }
        _ => {
            let entry = coefficients
                .entry(Arc::clone(&expression.structural_key))
                .or_insert_with(|| (value, BigInt::zero()));
            entry.1 += coefficient;
        }
    }
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
fn interval_linear(
    constant: &BigInt,
    terms: &[LinearTermData],
    expressions: &[DraftIndexExpr],
) -> Option<(BigInt, BigInt)> {
    let (mut minimum, mut maximum) = (constant.clone(), constant.clone());
    for term in terms {
        let (child_minimum, child_maximum) = expressions[term.value as usize].interval.clone()?;
        let (term_minimum, term_maximum) = if term.coefficient.0.sign() == num_bigint::Sign::Minus {
            (
                &term.coefficient.0 * &child_maximum,
                &term.coefficient.0 * &child_minimum,
            )
        } else {
            (
                &term.coefficient.0 * &child_minimum,
                &term.coefficient.0 * &child_maximum,
            )
        };
        minimum += term_minimum;
        maximum += term_maximum;
    }
    Some((minimum, maximum))
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
fn remap_node(node: &IndexNode, map: &BTreeMap<u32, u32>) -> IndexNode {
    match node {
        IndexNode::Constant(v) => IndexNode::Constant(v.clone()),
        IndexNode::Dimension(d) => IndexNode::Dimension(*d),
        IndexNode::LinearCombination { constant, terms } => IndexNode::LinearCombination {
            constant: constant.clone(),
            terms: terms
                .iter()
                .map(|t| LinearTermData {
                    coefficient: t.coefficient.clone(),
                    value: map[&t.value],
                })
                .collect(),
        },
        IndexNode::FloorDiv { dividend, divisor } => IndexNode::FloorDiv {
            dividend: map[dividend],
            divisor: *divisor,
        },
        IndexNode::Modulo { dividend, divisor } => IndexNode::Modulo {
            dividend: map[dividend],
            divisor: *divisor,
        },
    }
}
fn remap_operation(
    op: &ScalarOperationData,
    values: &BTreeMap<u32, u32>,
    _ops: &BTreeMap<u32, u32>,
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
                dimensions: dimensions.clone(),
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
    dimensions: &[DimensionData],
    tensors: &[TensorData],
    expressions: &[IndexExprData],
    accesses: &[VerifiedAccessData],
    operations: &[ScalarOperationData],
    values: &[ScalarValueData],
    outputs: &[OutputData],
) -> CanonicalIndexRegionIdentity {
    let mut out = b"tiler.index-region.v3\0".to_vec();
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
    CanonicalIndexRegionIdentity(out)
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
