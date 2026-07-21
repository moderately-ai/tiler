use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Zero};

use crate::semantic::{F32, ResolvedValueType};
use crate::shape::{Extent, Shape};

use super::error::invalid_handle;
use super::handles::{BuilderId, next_builder_id};
use super::model::{
    AccessData, BoundsProof, DimensionData, IndexExprData, IndexNode, LinearTermData, OutputData,
    ScalarExprData, ScalarNode, TensorData, VerifiedAccessData, VerifiedIndexRegionData,
    WriteOwnershipProof,
};
use super::{
    AccessMode, BoundsWitnessId, CanonicalIndexRegionIdentity, ContributorOrder, DimensionId,
    DomainRole, IndexBuildError, IndexEntityKind, IndexExprClass, IndexExprId, IndexInteger,
    IndexRegionBuildError, IndexRegionDiagnostic, MAX_ACCESS_CANONICAL_BYTES,
    MAX_BOUNDARY_CANONICAL_BYTES, MAX_BOUNDARY_TENSORS, MAX_DOMAIN_DIMENSIONS,
    MAX_EXHAUSTIVE_PROOF_BYTES, MAX_EXHAUSTIVE_PROOF_CELLS, MAX_INDEX_CANONICAL_BYTES,
    MAX_INDEX_EXPRESSION_OPERANDS, MAX_INDEX_EXPRESSIONS, MAX_INDEX_REGION_IDENTITY_BYTES,
    MAX_OUTPUT_ROOTS, MAX_SCALAR_CANONICAL_BYTES, MAX_SCALAR_EXPRESSION_DEPTH,
    MAX_SCALAR_EXPRESSIONS, MAX_TENSOR_ACCESSES, MAX_TENSOR_RANK, ProofResource, ScalarExprId,
    SemanticRegionIdentity, TensorAccessId, TensorId, TensorRole, VerifiedIndexRegion,
    WriteOwnershipWitnessId,
};

/// Mutable transactional construction of one target-independent index region.
///
/// This builder verifies structural relations, exact static bounds, reduction
/// binding, and ordinary write ownership. It deliberately does not prove that
/// the relation implements the correlated semantic region.
#[derive(Debug)]
pub struct IndexRegionBuilder {
    owner: BuilderId,
    semantic_region: SemanticRegionIdentity,
    dimensions: Vec<DimensionData>,
    tensors: Vec<TensorData>,
    boundary_canonical_bytes: usize,
    expressions: Vec<IndexExprData>,
    expression_intern: BTreeMap<Vec<u8>, u32>,
    index_canonical_bytes: usize,
    accesses: Vec<AccessData>,
    access_canonical_bytes: usize,
    scalars: Vec<ScalarExprData>,
    scalar_intern: BTreeMap<Vec<u8>, u32>,
    scalar_canonical_bytes: usize,
    outputs: Vec<OutputData>,
}

impl IndexRegionBuilder {
    /// Creates a builder correlated with one canonical semantic region.
    ///
    /// # Errors
    ///
    /// Returns [`IndexBuildError::BuilderIdentityExhausted`] if process-local
    /// draft ownership cannot be allocated.
    pub fn new(semantic_region: SemanticRegionIdentity) -> Result<Self, IndexBuildError> {
        let owner = next_builder_id().ok_or(IndexBuildError::BuilderIdentityExhausted)?;
        Ok(Self {
            owner,
            semantic_region,
            dimensions: Vec::new(),
            tensors: Vec::new(),
            boundary_canonical_bytes: 0,
            expressions: Vec::new(),
            expression_intern: BTreeMap::new(),
            index_canonical_bytes: 0,
            accesses: Vec::new(),
            access_canonical_bytes: 0,
            scalars: Vec::new(),
            scalar_intern: BTreeMap::new(),
            scalar_canonical_bytes: 0,
            outputs: Vec::new(),
        })
    }

    /// Declares one ordered tensor boundary.
    ///
    /// # Errors
    ///
    /// Returns a typed structural-limit error without mutation.
    pub fn tensor(
        &mut self,
        role: TensorRole,
        value_type: ResolvedValueType,
        shape: Shape,
    ) -> Result<TensorId, IndexBuildError> {
        Self::check_capacity(
            self.tensors.len(),
            MAX_BOUNDARY_TENSORS,
            IndexEntityKind::Tensor,
        )?;
        if shape.rank() > MAX_TENSOR_RANK {
            return Err(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::Tensor,
                limit: MAX_TENSOR_RANK,
            });
        }
        let next_boundary_bytes = value_type
            .canonical_encoding()
            .as_bytes()
            .len()
            .checked_add(shape.rank().saturating_mul(8))
            .and_then(|bytes| bytes.checked_add(32))
            .and_then(|bytes| self.boundary_canonical_bytes.checked_add(bytes))
            .filter(|bytes| *bytes <= MAX_BOUNDARY_CANONICAL_BYTES)
            .ok_or(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::Tensor,
                limit: MAX_BOUNDARY_CANONICAL_BYTES,
            })?;
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
        self.boundary_canonical_bytes = next_boundary_bytes;
        Ok(id)
    }

    /// Adds a static half-open iteration dimension.
    ///
    /// Zero is a valid extent and denotes an empty point set.
    ///
    /// # Errors
    ///
    /// Returns a typed structural-limit error without mutation.
    pub fn dimension(
        &mut self,
        role: DomainRole,
        extent: Extent,
    ) -> Result<DimensionId, IndexBuildError> {
        Self::check_capacity(
            self.dimensions.len(),
            MAX_DOMAIN_DIMENSIONS,
            IndexEntityKind::Dimension,
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
    /// Returns a typed expression-budget error without mutation.
    pub fn constant(&mut self, value: IndexInteger) -> Result<IndexExprId, IndexBuildError> {
        self.intern_index(
            IndexNode::Constant(value.clone()),
            BTreeSet::new(),
            IndexExprClass::Affine,
            Some((value.0.clone(), value.0)),
        )
    }

    /// Creates or reuses the expression naming a domain dimension.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or expression-budget error without mutation.
    pub fn iteration(&mut self, dimension: DimensionId) -> Result<IndexExprId, IndexBuildError> {
        let index = self.check_dimension(dimension)?;
        let extent = self.dimensions[index].extent;
        let interval = (extent != 0).then(|| (BigInt::zero(), BigInt::from(extent - 1)));
        self.intern_index(
            IndexNode::Dimension(dimension.index),
            BTreeSet::from([dimension.index]),
            IndexExprClass::Affine,
            interval,
        )
    }

    /// Canonically adds exact index expressions.
    ///
    /// Additive constants and like coefficients are combined. Construction is
    /// transactional even when normalization or a governed budget fails.
    ///
    /// # Errors
    ///
    /// Returns a typed handle, operand-count, or byte-budget error.
    pub fn add(
        &mut self,
        terms: impl IntoIterator<Item = IndexExprId>,
    ) -> Result<IndexExprId, IndexBuildError> {
        let mut constant = BigInt::zero();
        let mut bases = BTreeMap::<Vec<u8>, (BigInt, u32)>::new();
        let mut normalization_bytes = 0_usize;
        for (count, term) in terms.into_iter().enumerate() {
            if count == MAX_INDEX_EXPRESSION_OPERANDS {
                return Err(IndexBuildError::StructuralLimit {
                    entity: IndexEntityKind::IndexExpression,
                    limit: MAX_INDEX_EXPRESSION_OPERANDS,
                });
            }
            let index = self.check_index_expr(term)?;
            normalization_bytes = normalization_bytes
                .checked_add(self.expressions[index].canonical.len())
                .filter(|bytes| *bytes <= MAX_INDEX_CANONICAL_BYTES)
                .ok_or(IndexBuildError::StructuralLimit {
                    entity: IndexEntityKind::IndexExpression,
                    limit: MAX_INDEX_CANONICAL_BYTES,
                })?;
            self.accumulate_linear(index, &BigInt::one(), &mut constant, &mut bases);
        }
        self.finish_linear(constant, bases)
    }

    /// Canonically negates one exact index expression.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or expression-budget error without mutation.
    pub fn negate(&mut self, value: IndexExprId) -> Result<IndexExprId, IndexBuildError> {
        self.scale(IndexInteger::from_i128(-1), value)
    }

    /// Multiplies an index expression by an exact constant.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or expression-budget error without mutation.
    pub fn scale(
        &mut self,
        coefficient: impl Into<IndexInteger>,
        value: IndexExprId,
    ) -> Result<IndexExprId, IndexBuildError> {
        let index = self.check_index_expr(value)?;
        let coefficient = coefficient.into();
        let mut constant = BigInt::zero();
        let mut bases = BTreeMap::new();
        self.accumulate_linear(index, &coefficient.0, &mut constant, &mut bases);
        self.finish_linear(constant, bases)
    }

    /// Applies Euclidean floor division by a positive constant.
    ///
    /// # Errors
    ///
    /// Returns a typed error for zero or an invalid dividend handle.
    pub fn floor_div(
        &mut self,
        dividend: IndexExprId,
        divisor: u64,
    ) -> Result<IndexExprId, IndexBuildError> {
        self.division_like(dividend, divisor, false)
    }

    /// Applies Euclidean modulo by a positive constant.
    ///
    /// # Errors
    ///
    /// Returns a typed error for zero or an invalid dividend handle.
    pub fn modulo(
        &mut self,
        dividend: IndexExprId,
        divisor: u64,
    ) -> Result<IndexExprId, IndexBuildError> {
        self.division_like(dividend, divisor, true)
    }

    /// Adds a logical read with an explicit lexical evaluation domain.
    ///
    /// # Errors
    ///
    /// Returns a typed tensor-role, domain, rank, handle, or limit error.
    pub fn read(
        &mut self,
        tensor: TensorId,
        domain: impl IntoIterator<Item = DimensionId>,
        coordinates: impl IntoIterator<Item = IndexExprId>,
    ) -> Result<TensorAccessId, IndexBuildError> {
        self.access(tensor, AccessMode::Read, domain, coordinates)
    }

    /// Adds an ordinary logical output write over the parallel domain.
    ///
    /// # Errors
    ///
    /// Returns a typed tensor-role, rank, handle, or limit error.
    pub fn write(
        &mut self,
        tensor: TensorId,
        coordinates: impl IntoIterator<Item = IndexExprId>,
    ) -> Result<TensorAccessId, IndexBuildError> {
        let parallel: Vec<_> = self
            .dimensions
            .iter()
            .enumerate()
            .filter_map(|(index, data)| {
                if data.role != DomainRole::Parallel {
                    return None;
                }
                Some(DimensionId {
                    owner: self.owner,
                    index: u32::try_from(index).ok()?,
                })
            })
            .collect();
        self.access(tensor, AccessMode::Write, parallel, coordinates)
    }

    /// Loads an F32 scalar through a logical read.
    ///
    /// # Errors
    ///
    /// Returns a typed access-mode, tensor-type, or budget error without mutation.
    pub fn load(&mut self, access: TensorAccessId) -> Result<ScalarExprId, IndexBuildError> {
        let index = self.check_access(access)?;
        let data = &self.accesses[index];
        if data.mode != AccessMode::Read {
            return Err(IndexBuildError::LoadFromWrite);
        }
        if self.tensors[data.tensor as usize].value_type != F32::resolved_type() {
            return Err(IndexBuildError::ScalarTypeMismatch);
        }
        let free_dimensions = data.domain.iter().copied().collect();
        self.intern_scalar(
            ScalarNode::Load {
                access: access.index,
            },
            free_dimensions,
        )
    }

    /// Creates an exact IEEE binary32 scalar constant in an evaluation domain.
    ///
    /// # Errors
    ///
    /// Returns a typed scalar budget error without mutation.
    pub fn f32_constant(
        &mut self,
        domain: impl IntoIterator<Item = DimensionId>,
        bits: u32,
    ) -> Result<ScalarExprId, IndexBuildError> {
        let mut dimensions = BTreeSet::new();
        for dimension in domain {
            self.check_dimension(dimension)?;
            if !dimensions.insert(dimension.index) {
                return Err(IndexBuildError::DuplicateAccessDimension { dimension });
            }
        }
        self.intern_scalar(ScalarNode::F32Constant { bits }, dimensions)
    }

    /// Creates an ordered binary32 multiplication.
    ///
    /// # Errors
    ///
    /// Returns a typed handle, depth, or scalar budget error without mutation.
    pub fn f32_multiply(
        &mut self,
        left: ScalarExprId,
        right: ScalarExprId,
    ) -> Result<ScalarExprId, IndexBuildError> {
        self.binary_f32(left, right, true)
    }

    /// Creates an ordered binary32 addition.
    ///
    /// # Errors
    ///
    /// Returns a typed handle, depth, or scalar budget error without mutation.
    pub fn f32_add(
        &mut self,
        left: ScalarExprId,
        right: ScalarExprId,
    ) -> Result<ScalarExprId, IndexBuildError> {
        self.binary_f32(left, right, false)
    }

    /// Lexically binds reduction dimensions to an exact serial F32 sum.
    ///
    /// # Errors
    ///
    /// Returns a typed handle, role, binding, depth, or budget error.
    pub fn strict_serial_f32_sum(
        &mut self,
        dimensions: impl IntoIterator<Item = DimensionId>,
        value: ScalarExprId,
    ) -> Result<ScalarExprId, IndexBuildError> {
        let value_index = self.check_scalar(value)?;
        let mut bound = Vec::new();
        let mut seen = BTreeSet::new();
        for dimension in dimensions {
            let index = self.check_dimension(dimension)?;
            if self.dimensions[index].role != DomainRole::Reduction {
                return Err(IndexBuildError::ExpectedReductionDimension { dimension });
            }
            if !seen.insert(dimension.index) {
                return Err(IndexBuildError::DuplicateReductionDimension { dimension });
            }
            bound.push(dimension.index);
        }
        let mut free = self.scalars[value_index].free_dimensions.clone();
        for dimension in &bound {
            free.remove(dimension);
        }
        self.intern_scalar(
            ScalarNode::StrictSerialF32Sum {
                value: value.index,
                dimensions: bound,
                order: ContributorOrder::AxisLexicographic,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            free,
        )
    }

    /// Binds one output write to its pointwise scalar expression.
    ///
    /// # Errors
    ///
    /// Returns a typed mode, tensor-type, handle, or limit error without mutation.
    pub fn output(
        &mut self,
        access: TensorAccessId,
        value: ScalarExprId,
    ) -> Result<(), IndexBuildError> {
        let access_index = self.check_access(access)?;
        self.check_scalar(value)?;
        if self.accesses[access_index].mode != AccessMode::Write {
            return Err(IndexBuildError::OutputUsesRead);
        }
        let tensor = self.accesses[access_index].tensor as usize;
        if self.tensors[tensor].value_type != F32::resolved_type() {
            return Err(IndexBuildError::ScalarTypeMismatch);
        }
        Self::check_capacity(
            self.outputs.len(),
            MAX_OUTPUT_ROOTS,
            IndexEntityKind::OutputRoot,
        )?;
        self.outputs.push(OutputData {
            access: access.index,
            value: value.index,
        });
        Ok(())
    }

    /// Verifies the complete region and produces immutable, compact IR.
    ///
    /// # Errors
    ///
    /// Returns all deterministic diagnostics together with the intact builder.
    pub fn build(self) -> Result<VerifiedIndexRegion, IndexRegionBuildError> {
        match self.verify_and_compact() {
            Ok(data) => Ok(VerifiedIndexRegion {
                data: Arc::new(data),
            }),
            Err(diagnostics) => Err(IndexRegionBuildError {
                builder: Box::new(self),
                diagnostics,
            }),
        }
    }

    fn access(
        &mut self,
        tensor: TensorId,
        mode: AccessMode,
        domain: impl IntoIterator<Item = DimensionId>,
        coordinates: impl IntoIterator<Item = IndexExprId>,
    ) -> Result<TensorAccessId, IndexBuildError> {
        let tensor_index = self.check_tensor(tensor)?;
        match (mode, self.tensors[tensor_index].role) {
            (AccessMode::Read, TensorRole::Output) => return Err(IndexBuildError::ReadFromOutput),
            (AccessMode::Write, TensorRole::Input) => return Err(IndexBuildError::WriteToInput),
            _ => {}
        }
        Self::check_capacity(
            self.accesses.len(),
            MAX_TENSOR_ACCESSES,
            IndexEntityKind::TensorAccess,
        )?;

        let mut domain_indices = Vec::new();
        let mut seen = BTreeSet::new();
        for dimension in domain {
            self.check_dimension(dimension)?;
            if !seen.insert(dimension.index) {
                return Err(IndexBuildError::DuplicateAccessDimension { dimension });
            }
            domain_indices.push(dimension.index);
        }
        domain_indices.sort_unstable();
        if mode == AccessMode::Write && domain_indices != self.parallel_dimensions() {
            return Err(IndexBuildError::InvalidWriteDomain);
        }

        let expected = self.tensors[tensor_index].shape.rank();
        let mut coordinates_bounded = Vec::with_capacity(expected);
        for coordinate in coordinates {
            if coordinates_bounded.len() == expected {
                return Err(IndexBuildError::AccessRank {
                    expected,
                    actual: expected.saturating_add(1),
                });
            }
            coordinates_bounded.push(coordinate);
        }
        if coordinates_bounded.len() != expected {
            return Err(IndexBuildError::AccessRank {
                expected,
                actual: coordinates_bounded.len(),
            });
        }
        let coordinate_indices: Vec<_> = coordinates_bounded
            .iter()
            .copied()
            .map(|coordinate| {
                self.check_index_expr(coordinate)?;
                Ok(coordinate.index)
            })
            .collect::<Result<_, IndexBuildError>>()?;
        for coordinate in &coordinate_indices {
            if !self.expressions[*coordinate as usize]
                .dimensions
                .iter()
                .all(|dimension| seen.contains(dimension))
            {
                return Err(IndexBuildError::CoordinateOutsideAccessDomain);
            }
        }

        let canonical = self.encode_access(
            u32::try_from(tensor_index).expect("bounded tensor index fits u32"),
            mode,
            &domain_indices,
            &coordinate_indices,
        )?;
        let next_access_bytes = self
            .access_canonical_bytes
            .checked_add(canonical.len())
            .filter(|bytes| *bytes <= MAX_ACCESS_CANONICAL_BYTES)
            .ok_or(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::TensorAccess,
                limit: MAX_ACCESS_CANONICAL_BYTES,
            })?;
        let id = TensorAccessId::from_len(self.owner, self.accesses.len()).ok_or(
            IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::TensorAccess,
            },
        )?;
        self.accesses.push(AccessData {
            tensor: u32::try_from(tensor_index).expect("bounded tensor index fits u32"),
            mode,
            domain: domain_indices,
            coordinates: coordinate_indices,
            canonical,
        });
        self.access_canonical_bytes = next_access_bytes;
        Ok(id)
    }

    fn binary_f32(
        &mut self,
        left: ScalarExprId,
        right: ScalarExprId,
        multiply: bool,
    ) -> Result<ScalarExprId, IndexBuildError> {
        let left_index = self.check_scalar(left)?;
        let right_index = self.check_scalar(right)?;
        let free_dimensions = self.scalars[left_index]
            .free_dimensions
            .union(&self.scalars[right_index].free_dimensions)
            .copied()
            .collect();
        let node = if multiply {
            ScalarNode::F32Multiply {
                left: left.index,
                right: right.index,
            }
        } else {
            ScalarNode::F32Add {
                left: left.index,
                right: right.index,
            }
        };
        self.intern_scalar(node, free_dimensions)
    }

    fn intern_scalar(
        &mut self,
        node: ScalarNode,
        free_dimensions: BTreeSet<u32>,
    ) -> Result<ScalarExprId, IndexBuildError> {
        let depth = match node {
            ScalarNode::Load { .. } | ScalarNode::F32Constant { .. } => 1,
            ScalarNode::F32Multiply { left, right } | ScalarNode::F32Add { left, right } => {
                1 + self.scalars[left as usize]
                    .depth
                    .max(self.scalars[right as usize].depth)
            }
            ScalarNode::StrictSerialF32Sum { value, .. } => 1 + self.scalars[value as usize].depth,
        };
        if depth > MAX_SCALAR_EXPRESSION_DEPTH {
            return Err(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::ScalarExpression,
                limit: MAX_SCALAR_EXPRESSION_DEPTH,
            });
        }
        let canonical = self.encode_scalar_node(&node, &free_dimensions)?;
        if let Some(index) = self.scalar_intern.get(&canonical) {
            return Ok(ScalarExprId {
                owner: self.owner,
                index: *index,
            });
        }
        Self::check_capacity(
            self.scalars.len(),
            MAX_SCALAR_EXPRESSIONS,
            IndexEntityKind::ScalarExpression,
        )?;
        let next_bytes = self
            .scalar_canonical_bytes
            .checked_add(canonical.len())
            .filter(|bytes| *bytes <= MAX_SCALAR_CANONICAL_BYTES)
            .ok_or(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::ScalarExpression,
                limit: MAX_SCALAR_CANONICAL_BYTES,
            })?;
        let id = ScalarExprId::from_len(self.owner, self.scalars.len()).ok_or(
            IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::ScalarExpression,
            },
        )?;
        self.scalars.push(ScalarExprData {
            node,
            free_dimensions,
            depth,
            canonical: canonical.clone(),
        });
        self.scalar_intern.insert(canonical, id.index);
        self.scalar_canonical_bytes = next_bytes;
        Ok(id)
    }

    fn encode_scalar_node(
        &self,
        node: &ScalarNode,
        free_dimensions: &BTreeSet<u32>,
    ) -> Result<Vec<u8>, IndexBuildError> {
        let components: Vec<&[u8]> = match node {
            ScalarNode::Load { access } => vec![&self.accesses[*access as usize].canonical],
            ScalarNode::F32Constant { .. } => Vec::new(),
            ScalarNode::F32Multiply { left, right } | ScalarNode::F32Add { left, right } => vec![
                &self.scalars[*left as usize].canonical,
                &self.scalars[*right as usize].canonical,
            ],
            ScalarNode::StrictSerialF32Sum { value, .. } => {
                vec![&self.scalars[*value as usize].canonical]
            }
        };
        let component_bytes = components
            .iter()
            .try_fold(0_usize, |total, bytes| total.checked_add(bytes.len()));
        let fixed = 40_usize
            .checked_add(free_dimensions.len().saturating_mul(4))
            .ok_or(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::ScalarExpression,
                limit: MAX_SCALAR_CANONICAL_BYTES,
            })?;
        let required = component_bytes
            .and_then(|value| value.checked_add(fixed))
            .ok_or(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::ScalarExpression,
                limit: MAX_SCALAR_CANONICAL_BYTES,
            })?;
        if required > MAX_SCALAR_CANONICAL_BYTES.saturating_sub(self.scalar_canonical_bytes) {
            return Err(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::ScalarExpression,
                limit: MAX_SCALAR_CANONICAL_BYTES,
            });
        }
        let mut output = Vec::with_capacity(required);
        match node {
            ScalarNode::Load { .. } => output.push(1),
            ScalarNode::F32Constant { bits } => {
                output.push(2);
                output.extend_from_slice(&bits.to_be_bytes());
            }
            ScalarNode::F32Multiply { .. } => output.push(3),
            ScalarNode::F32Add { .. } => output.push(4),
            ScalarNode::StrictSerialF32Sum {
                dimensions,
                order: ContributorOrder::AxisLexicographic,
                empty_identity_bits,
                ..
            } => {
                output.push(5);
                encode_u32_slice(&mut output, dimensions);
                output.push(1);
                output.extend_from_slice(&empty_identity_bits.to_be_bytes());
            }
        }
        for bytes in components {
            encode_bytes(&mut output, bytes);
        }
        encode_len(&mut output, free_dimensions.len());
        for dimension in free_dimensions {
            output.extend_from_slice(&dimension.to_be_bytes());
        }
        Ok(output)
    }

    fn accumulate_linear(
        &self,
        index: usize,
        multiplier: &BigInt,
        constant: &mut BigInt,
        bases: &mut BTreeMap<Vec<u8>, (BigInt, u32)>,
    ) {
        match &self.expressions[index].node {
            IndexNode::Constant(value) => *constant += multiplier * &value.0,
            IndexNode::LinearCombination {
                constant: inner,
                terms,
            } => {
                *constant += multiplier * &inner.0;
                for term in terms {
                    let coefficient = multiplier * &term.coefficient.0;
                    let key = self.expressions[term.value as usize].canonical.clone();
                    bases
                        .entry(key)
                        .and_modify(|(existing, _)| *existing += &coefficient)
                        .or_insert((coefficient, term.value));
                }
            }
            _ => {
                let key = self.expressions[index].canonical.clone();
                bases
                    .entry(key)
                    .and_modify(|(existing, _)| *existing += multiplier)
                    .or_insert((
                        multiplier.clone(),
                        u32::try_from(index).expect("bounded index"),
                    ));
            }
        }
    }

    fn finish_linear(
        &mut self,
        constant: BigInt,
        bases: BTreeMap<Vec<u8>, (BigInt, u32)>,
    ) -> Result<IndexExprId, IndexBuildError> {
        let terms: Vec<_> = bases
            .into_values()
            .filter_map(|(coefficient, value)| {
                (!coefficient.is_zero()).then_some(LinearTermData {
                    coefficient: IndexInteger(coefficient),
                    value,
                })
            })
            .collect();
        if terms.is_empty() {
            return self.constant(IndexInteger(constant));
        }
        if constant.is_zero() && terms.len() == 1 && terms[0].coefficient.0.is_one() {
            return Ok(IndexExprId {
                owner: self.owner,
                index: terms[0].value,
            });
        }
        let mut dimensions = BTreeSet::new();
        let mut class = IndexExprClass::Affine;
        let mut interval = Some((constant.clone(), constant.clone()));
        for term in &terms {
            let expression = &self.expressions[term.value as usize];
            dimensions.extend(expression.dimensions.iter().copied());
            class = class.max(expression.class);
            interval = match (interval, &expression.interval) {
                (Some((min, max)), Some((other_min, other_max))) => {
                    let first = &term.coefficient.0 * other_min;
                    let second = &term.coefficient.0 * other_max;
                    let (term_min, term_max) = if first <= second {
                        (first, second)
                    } else {
                        (second, first)
                    };
                    Some((min + term_min, max + term_max))
                }
                _ => None,
            };
        }
        self.intern_index(
            IndexNode::LinearCombination {
                constant: IndexInteger(constant),
                terms,
            },
            dimensions,
            class,
            interval,
        )
    }

    fn division_like(
        &mut self,
        dividend: IndexExprId,
        divisor: u64,
        modulo: bool,
    ) -> Result<IndexExprId, IndexBuildError> {
        if divisor == 0 {
            return Err(IndexBuildError::NonPositiveDivisor);
        }
        let index = self.check_index_expr(dividend)?;
        if divisor == 1 {
            return if modulo {
                self.constant(IndexInteger::from_u64(0))
            } else {
                Ok(dividend)
            };
        }
        if let IndexNode::Constant(value) = &self.expressions[index].node {
            let divisor = BigInt::from(divisor);
            let value = if modulo {
                value.0.mod_floor(&divisor)
            } else {
                value.0.div_floor(&divisor)
            };
            return self.constant(IndexInteger(value));
        }
        let expression = &self.expressions[index];
        let interval = expression.interval.as_ref().map(|(min, max)| {
            let divisor = BigInt::from(divisor);
            if modulo {
                (BigInt::zero(), &divisor - BigInt::one())
            } else {
                (min.div_floor(&divisor), max.div_floor(&divisor))
            }
        });
        let node = if modulo {
            IndexNode::Modulo {
                dividend: dividend.index,
                divisor,
            }
        } else {
            IndexNode::FloorDiv {
                dividend: dividend.index,
                divisor,
            }
        };
        self.intern_index(
            node,
            expression.dimensions.clone(),
            IndexExprClass::QuasiAffine,
            interval,
        )
    }

    fn intern_index(
        &mut self,
        node: IndexNode,
        dimensions: BTreeSet<u32>,
        class: IndexExprClass,
        interval: Option<(BigInt, BigInt)>,
    ) -> Result<IndexExprId, IndexBuildError> {
        let canonical = self.encode_index_node(&node)?;
        if let Some(index) = self.expression_intern.get(&canonical) {
            return Ok(IndexExprId {
                owner: self.owner,
                index: *index,
            });
        }
        Self::check_capacity(
            self.expressions.len(),
            MAX_INDEX_EXPRESSIONS,
            IndexEntityKind::IndexExpression,
        )?;
        let next_bytes = self
            .index_canonical_bytes
            .checked_add(canonical.len())
            .filter(|bytes| *bytes <= MAX_INDEX_CANONICAL_BYTES)
            .ok_or(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::IndexExpression,
                limit: MAX_INDEX_CANONICAL_BYTES,
            })?;
        let id = IndexExprId::from_len(self.owner, self.expressions.len()).ok_or(
            IndexBuildError::TooManyEntities {
                entity: IndexEntityKind::IndexExpression,
            },
        )?;
        self.expressions.push(IndexExprData {
            node,
            canonical: canonical.clone(),
            dimensions,
            class,
            interval,
        });
        self.expression_intern.insert(canonical, id.index);
        self.index_canonical_bytes = next_bytes;
        Ok(id)
    }

    fn encode_index_node(&self, node: &IndexNode) -> Result<Vec<u8>, IndexBuildError> {
        let required = match node {
            IndexNode::Constant(value) => 1 + encoded_integer_len(value),
            IndexNode::Dimension(_) => 5,
            IndexNode::LinearCombination { constant, terms } => {
                let mut bytes = 1_usize + encoded_integer_len(constant) + 8;
                for term in terms {
                    bytes = bytes
                        .checked_add(encoded_integer_len(&term.coefficient))
                        .and_then(|value| {
                            value.checked_add(
                                self.expressions[term.value as usize].canonical.len() + 8,
                            )
                        })
                        .ok_or(IndexBuildError::StructuralLimit {
                            entity: IndexEntityKind::IndexExpression,
                            limit: MAX_INDEX_CANONICAL_BYTES,
                        })?;
                }
                bytes
            }
            IndexNode::FloorDiv { dividend, .. } | IndexNode::Modulo { dividend, .. } => 17_usize
                .checked_add(self.expressions[*dividend as usize].canonical.len())
                .ok_or(IndexBuildError::StructuralLimit {
                    entity: IndexEntityKind::IndexExpression,
                    limit: MAX_INDEX_CANONICAL_BYTES,
                })?,
        };
        if required > MAX_INDEX_CANONICAL_BYTES.saturating_sub(self.index_canonical_bytes) {
            return Err(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::IndexExpression,
                limit: MAX_INDEX_CANONICAL_BYTES,
            });
        }
        let mut output = Vec::with_capacity(required);
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
                        &self.expressions[term.value as usize].canonical,
                    );
                }
            }
            IndexNode::FloorDiv { dividend, divisor } => {
                output.push(4);
                encode_bytes(&mut output, &self.expressions[*dividend as usize].canonical);
                output.extend_from_slice(&divisor.to_be_bytes());
            }
            IndexNode::Modulo { dividend, divisor } => {
                output.push(5);
                encode_bytes(&mut output, &self.expressions[*dividend as usize].canonical);
                output.extend_from_slice(&divisor.to_be_bytes());
            }
        }
        Ok(output)
    }

    fn encode_access(
        &self,
        tensor: u32,
        mode: AccessMode,
        domain: &[u32],
        coordinates: &[u32],
    ) -> Result<Vec<u8>, IndexBuildError> {
        let coordinate_bytes = coordinates.iter().try_fold(0_usize, |total, coordinate| {
            total
                .checked_add(self.expressions[*coordinate as usize].canonical.len())
                .and_then(|value| value.checked_add(8))
        });
        let required = coordinate_bytes
            .and_then(|value| value.checked_add(32 + domain.len() * 4))
            .ok_or(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::TensorAccess,
                limit: MAX_INDEX_CANONICAL_BYTES,
            })?;
        if required > MAX_ACCESS_CANONICAL_BYTES.saturating_sub(self.access_canonical_bytes) {
            return Err(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::TensorAccess,
                limit: MAX_ACCESS_CANONICAL_BYTES,
            });
        }
        let mut output = Vec::with_capacity(required);
        output.extend_from_slice(b"tiler.access.v1\0");
        output.extend_from_slice(&tensor.to_be_bytes());
        output.push(match mode {
            AccessMode::Read => 1,
            AccessMode::Write => 2,
        });
        encode_u32_slice(&mut output, domain);
        encode_len(&mut output, coordinates.len());
        for coordinate in coordinates {
            encode_bytes(
                &mut output,
                &self.expressions[*coordinate as usize].canonical,
            );
        }
        Ok(output)
    }

    fn verify_and_compact(&self) -> Result<VerifiedIndexRegionData, Vec<IndexRegionDiagnostic>> {
        let mut diagnostics = Vec::new();
        if self.outputs.is_empty() {
            diagnostics.push(IndexRegionDiagnostic::NoOutputs);
        }

        let reachable_scalars = self.reachable_scalars();
        let mut reachable_accesses: BTreeSet<u32> =
            self.outputs.iter().map(|output| output.access).collect();
        let mut bound_reductions = BTreeSet::new();
        for scalar in &reachable_scalars {
            match &self.scalars[*scalar as usize].node {
                ScalarNode::Load { access } => {
                    reachable_accesses.insert(*access);
                }
                ScalarNode::StrictSerialF32Sum { dimensions, .. } => {
                    bound_reductions.extend(dimensions.iter().copied());
                }
                _ => {}
            }
        }
        for (index, dimension) in self.dimensions.iter().enumerate() {
            let index = u32::try_from(index).expect("bounded dimension index fits u32");
            if dimension.role == DomainRole::Reduction && !bound_reductions.contains(&index) {
                diagnostics.push(IndexRegionDiagnostic::UnusedDomainDimension {
                    dimension: self.dimension_handle(index),
                });
            }
        }
        for scalar in &reachable_scalars {
            if let ScalarNode::StrictSerialF32Sum {
                value, dimensions, ..
            } = &self.scalars[*scalar as usize].node
            {
                for dimension in dimensions {
                    if !self.scalars[*value as usize]
                        .free_dimensions
                        .contains(dimension)
                    {
                        diagnostics.push(IndexRegionDiagnostic::UnusedReductionDimension {
                            expression: self.scalar_handle(*scalar),
                            dimension: self.dimension_handle(*dimension),
                        });
                    }
                }
            }
        }
        for output in &self.outputs {
            for dimension in &self.scalars[output.value as usize].free_dimensions {
                if self.dimensions[*dimension as usize].role == DomainRole::Reduction {
                    diagnostics.push(IndexRegionDiagnostic::FreeReductionDimension {
                        expression: self.scalar_handle(output.value),
                        dimension: self.dimension_handle(*dimension),
                    });
                }
            }
        }

        let mut access_proofs = BTreeMap::new();
        for access in &reachable_accesses {
            match self.verify_bounds(*access) {
                Ok(proof) => {
                    access_proofs.insert(*access, proof);
                }
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }

        let mut seen_output_tensors = BTreeSet::new();
        let mut ownership_proofs = BTreeMap::new();
        for output in &self.outputs {
            let tensor = self.accesses[output.access as usize].tensor;
            if !seen_output_tensors.insert(tensor) {
                diagnostics.push(IndexRegionDiagnostic::DuplicateOutputTensor {
                    tensor: TensorId {
                        owner: self.owner,
                        index: tensor,
                    },
                });
            }
            if !access_proofs.contains_key(&output.access) {
                continue;
            }
            match self.verify_write_ownership(output.access) {
                Ok(proof) => {
                    ownership_proofs.insert(output.access, proof);
                }
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }
        diagnostics.extend(self.missing_output_diagnostics(&seen_output_tensors));

        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        self.compact(
            reachable_scalars,
            reachable_accesses,
            &access_proofs,
            &ownership_proofs,
        )
    }

    fn missing_output_diagnostics(&self, written: &BTreeSet<u32>) -> Vec<IndexRegionDiagnostic> {
        self.tensors
            .iter()
            .enumerate()
            .filter_map(|(index, tensor)| {
                let index = u32::try_from(index).expect("bounded tensor index fits u32");
                if tensor.role == TensorRole::Output && !written.contains(&index) {
                    Some(IndexRegionDiagnostic::MissingOutputTensor {
                        tensor: TensorId {
                            owner: self.owner,
                            index,
                        },
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn reachable_scalars(&self) -> BTreeSet<u32> {
        let mut reached = BTreeSet::new();
        let mut work: Vec<_> = self.outputs.iter().map(|output| output.value).collect();
        while let Some(index) = work.pop() {
            if !reached.insert(index) {
                continue;
            }
            match self.scalars[index as usize].node {
                ScalarNode::F32Multiply { left, right } | ScalarNode::F32Add { left, right } => {
                    work.push(left);
                    work.push(right);
                }
                ScalarNode::StrictSerialF32Sum { value, .. } => work.push(value),
                ScalarNode::Load { .. } | ScalarNode::F32Constant { .. } => {}
            }
        }
        reached
    }

    fn verify_bounds(&self, access_index: u32) -> Result<BoundsProof, IndexRegionDiagnostic> {
        let access = &self.accesses[access_index as usize];
        if access
            .domain
            .iter()
            .any(|dimension| self.dimensions[*dimension as usize].extent == 0)
        {
            return Ok(BoundsProof::VacuousEmptyDomain);
        }
        let shape = &self.tensors[access.tensor as usize].shape;
        let mut interval_unknown = false;
        for (coordinate, extent) in access.coordinates.iter().zip(shape.extents()) {
            match &self.expressions[*coordinate as usize].interval {
                Some((min, max)) if min >= &BigInt::zero() && max < &BigInt::from(extent.get()) => {
                }
                Some((min, max)) if max < &BigInt::zero() || min >= &BigInt::from(extent.get()) => {
                    return Err(IndexRegionDiagnostic::CoordinateOutOfBounds {
                        access: self.access_handle(access_index),
                    });
                }
                Some(_) | None => interval_unknown = true,
            }
        }
        if !interval_unknown {
            return Ok(BoundsProof::Interval);
        }
        let points = self.domain_points(&access.domain);
        let plan = self.evaluation_plan(&access.coordinates);
        Self::check_proof_resources(
            points,
            access.coordinates.len().saturating_add(plan.order.len()),
            plan.estimated_integer_bytes(self),
        )?;
        let mut evaluated = 0_u64;
        let result = self.for_each_point(&access.domain, |point| {
            evaluated += 1;
            plan.evaluate(self, point)
                .into_iter()
                .zip(shape.extents())
                .all(|(value, extent)| {
                    value >= BigInt::zero() && value < BigInt::from(extent.get())
                })
        });
        if result {
            Ok(BoundsProof::Exhaustive { points: evaluated })
        } else {
            Err(IndexRegionDiagnostic::CoordinateOutOfBounds {
                access: self.access_handle(access_index),
            })
        }
    }

    fn verify_write_ownership(
        &self,
        access_index: u32,
    ) -> Result<WriteOwnershipProof, IndexRegionDiagnostic> {
        let access = &self.accesses[access_index as usize];
        let parallel = self.parallel_dimensions();
        let shape = &self.tensors[access.tensor as usize].shape;
        if access.domain == parallel && access.coordinates.len() == parallel.len() {
            let mut used = BTreeSet::new();
            let permutation = access.coordinates.iter().zip(shape.extents()).all(
                |(coordinate, extent)| match self.expressions[*coordinate as usize].node {
                    IndexNode::Dimension(dimension)
                        if self.dimensions[dimension as usize].role == DomainRole::Parallel
                            && self.dimensions[dimension as usize].extent == extent.get() =>
                    {
                        used.insert(dimension)
                    }
                    _ => false,
                },
            );
            if permutation && used.len() == parallel.len() {
                return Ok(WriteOwnershipProof::CoordinatePermutation);
            }
        }
        let points = self.domain_points(&parallel);
        let elements = shape.extents().iter().fold(1_u128, |total, extent| {
            total.saturating_mul(u128::from(extent.get()))
        });
        if points != elements {
            return Err(IndexRegionDiagnostic::WriteOwnershipNotProven {
                access: self.access_handle(access_index),
            });
        }
        let plan = self.evaluation_plan(&access.coordinates);
        Self::check_proof_resources(
            points,
            shape.rank().saturating_add(plan.order.len()),
            plan.estimated_integer_bytes(self),
        )?;
        let mut images = BTreeSet::new();
        let valid = self.for_each_point(&parallel, |point| {
            let coordinates = plan.evaluate(self, point);
            images.insert(coordinates)
        });
        if valid && u128::try_from(images.len()).ok() == Some(points) {
            Ok(WriteOwnershipProof::Exhaustive {
                points: u64::try_from(points).expect("proof cap fits u64"),
            })
        } else {
            Err(IndexRegionDiagnostic::WriteOwnershipNotProven {
                access: self.access_handle(access_index),
            })
        }
    }

    fn check_proof_resources(
        points: u128,
        width: usize,
        integer_bytes_per_point: u128,
    ) -> Result<(), IndexRegionDiagnostic> {
        let cells = points.saturating_mul(u128::try_from(width.max(1)).expect("usize fits u128"));
        if cells > u128::from(MAX_EXHAUSTIVE_PROOF_CELLS) {
            Err(IndexRegionDiagnostic::ProofResourceLimit {
                resource: ProofResource::Cells,
                required: cells,
                limit: MAX_EXHAUSTIVE_PROOF_CELLS,
            })
        } else {
            let bytes = points.saturating_mul(integer_bytes_per_point.max(1));
            if bytes > u128::from(MAX_EXHAUSTIVE_PROOF_BYTES) {
                Err(IndexRegionDiagnostic::ProofResourceLimit {
                    resource: ProofResource::IntegerBytes,
                    required: bytes,
                    limit: MAX_EXHAUSTIVE_PROOF_BYTES,
                })
            } else {
                Ok(())
            }
        }
    }

    fn domain_points(&self, domain: &[u32]) -> u128 {
        domain.iter().fold(1_u128, |points, dimension| {
            points.saturating_mul(u128::from(self.dimensions[*dimension as usize].extent))
        })
    }

    fn for_each_point(&self, domain: &[u32], mut predicate: impl FnMut(&[u64]) -> bool) -> bool {
        if domain
            .iter()
            .any(|dimension| self.dimensions[*dimension as usize].extent == 0)
        {
            return true;
        }
        let mut point = vec![0_u64; self.dimensions.len()];
        loop {
            if !predicate(&point) {
                return false;
            }
            let mut advanced = false;
            for dimension in domain.iter().rev() {
                let index = *dimension as usize;
                point[index] += 1;
                if point[index] < self.dimensions[index].extent {
                    advanced = true;
                    break;
                }
                point[index] = 0;
            }
            if !advanced {
                return true;
            }
        }
    }

    fn evaluation_plan(&self, roots: &[u32]) -> EvaluationPlan {
        let mut reached = BTreeSet::new();
        let mut work = roots.to_vec();
        while let Some(index) = work.pop() {
            if !reached.insert(index) {
                continue;
            }
            match &self.expressions[index as usize].node {
                IndexNode::LinearCombination { terms, .. } => {
                    work.extend(terms.iter().map(|term| term.value));
                }
                IndexNode::FloorDiv { dividend, .. } | IndexNode::Modulo { dividend, .. } => {
                    work.push(*dividend);
                }
                IndexNode::Constant(_) | IndexNode::Dimension(_) => {}
            }
        }
        let order: Vec<_> = reached.into_iter().collect();
        let positions: BTreeMap<u32, usize> = order
            .iter()
            .enumerate()
            .map(|(position, index)| (*index, position))
            .collect();
        let roots = roots.iter().map(|root| positions[root]).collect();
        EvaluationPlan {
            order,
            positions,
            roots,
        }
    }

    fn compact(
        &self,
        scalar_set: BTreeSet<u32>,
        access_set: BTreeSet<u32>,
        access_proofs: &BTreeMap<u32, BoundsProof>,
        ownership_proofs: &BTreeMap<u32, WriteOwnershipProof>,
    ) -> Result<VerifiedIndexRegionData, Vec<IndexRegionDiagnostic>> {
        let mut expression_set = BTreeSet::new();
        let mut expression_work = Vec::new();
        for access in &access_set {
            expression_work.extend(self.accesses[*access as usize].coordinates.iter().copied());
        }
        while let Some(index) = expression_work.pop() {
            if !expression_set.insert(index) {
                continue;
            }
            match &self.expressions[index as usize].node {
                IndexNode::LinearCombination { terms, .. } => {
                    expression_work.extend(terms.iter().map(|term| term.value));
                }
                IndexNode::FloorDiv { dividend, .. } | IndexNode::Modulo { dividend, .. } => {
                    expression_work.push(*dividend);
                }
                IndexNode::Constant(_) | IndexNode::Dimension(_) => {}
            }
        }

        let (expression_order, access_order, scalar_order) =
            self.canonical_orders(expression_set, access_set, scalar_set);

        let expression_map = ordered_ordinal_map(&expression_order);
        let access_map = ordered_ordinal_map(&access_order);
        let scalar_map = ordered_ordinal_map(&scalar_order);

        let expressions: Vec<_> = expression_order
            .iter()
            .map(|old| {
                let mut data = self.expressions[*old as usize].clone();
                remap_index_node(&mut data.node, &expression_map);
                data
            })
            .collect();
        let accesses: Vec<_> = access_order
            .iter()
            .enumerate()
            .map(|(new, old)| {
                let data = &self.accesses[*old as usize];
                VerifiedAccessData {
                    tensor: data.tensor,
                    mode: data.mode,
                    domain: data.domain.clone(),
                    coordinates: data
                        .coordinates
                        .iter()
                        .map(|value| expression_map[value])
                        .collect(),
                    bounds: BoundsWitnessId(u32::try_from(new).expect("bounded access index")),
                    bounds_proof: access_proofs[old],
                    ownership: ownership_proofs.get(old).copied().map(|proof| {
                        (
                            WriteOwnershipWitnessId(
                                u32::try_from(new).expect("bounded access index"),
                            ),
                            proof,
                        )
                    }),
                }
            })
            .collect();
        let scalars: Vec<_> = scalar_order
            .iter()
            .map(|old| {
                let mut data = self.scalars[*old as usize].clone();
                remap_scalar_node(&mut data.node, &access_map, &scalar_map);
                data
            })
            .collect();
        let outputs: Vec<_> = self
            .outputs
            .iter()
            .map(|output| OutputData {
                access: access_map[&output.access],
                value: scalar_map[&output.value],
            })
            .collect();

        let identity_bytes =
            self.encode_region_identity(&expressions, &accesses, &scalars, &outputs);
        if identity_bytes.len() > MAX_INDEX_REGION_IDENTITY_BYTES {
            return Err(vec![IndexRegionDiagnostic::CanonicalIdentityLimit {
                bytes: identity_bytes.len(),
                limit: MAX_INDEX_REGION_IDENTITY_BYTES,
            }]);
        }
        let identity = CanonicalIndexRegionIdentity(identity_bytes);
        Ok(VerifiedIndexRegionData {
            semantic_region: self.semantic_region.clone(),
            dimensions: self.dimensions.clone(),
            tensors: self.tensors.clone(),
            expressions,
            accesses,
            scalars,
            outputs,
            identity,
        })
    }

    fn canonical_orders(
        &self,
        expression_set: BTreeSet<u32>,
        access_set: BTreeSet<u32>,
        scalar_set: BTreeSet<u32>,
    ) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
        let mut expressions: Vec<_> = expression_set.into_iter().collect();
        expressions.sort_by(|left, right| {
            let left = &self.expressions[*left as usize].canonical;
            let right = &self.expressions[*right as usize].canonical;
            left.len().cmp(&right.len()).then_with(|| left.cmp(right))
        });
        let mut accesses: Vec<_> = access_set.into_iter().collect();
        accesses.sort_by(|left, right| {
            self.accesses[*left as usize]
                .canonical
                .cmp(&self.accesses[*right as usize].canonical)
        });
        let mut scalars: Vec<_> = scalar_set.into_iter().collect();
        scalars.sort_by(|left, right| {
            let left_data = &self.scalars[*left as usize];
            let right_data = &self.scalars[*right as usize];
            left_data
                .depth
                .cmp(&right_data.depth)
                .then_with(|| left_data.canonical.cmp(&right_data.canonical))
        });
        (expressions, accesses, scalars)
    }

    fn encode_region_identity(
        &self,
        expressions: &[IndexExprData],
        accesses: &[VerifiedAccessData],
        scalars: &[ScalarExprData],
        outputs: &[OutputData],
    ) -> Vec<u8> {
        let mut output = b"tiler.index-region.v2\0".to_vec();
        encode_bytes(&mut output, self.semantic_region.as_bytes());
        encode_len(&mut output, self.dimensions.len());
        for dimension in &self.dimensions {
            output.push(match dimension.role {
                DomainRole::Parallel => 1,
                DomainRole::Reduction => 2,
            });
            output.extend_from_slice(&dimension.extent.to_be_bytes());
        }
        encode_len(&mut output, self.tensors.len());
        for tensor in &self.tensors {
            output.push(match tensor.role {
                TensorRole::Input => 1,
                TensorRole::Output => 2,
            });
            encode_bytes(
                &mut output,
                tensor.value_type.canonical_encoding().as_bytes(),
            );
            encode_len(&mut output, tensor.shape.rank());
            for extent in tensor.shape.extents() {
                output.extend_from_slice(&extent.get().to_be_bytes());
            }
        }
        encode_len(&mut output, expressions.len());
        for expression in expressions {
            encode_compact_index_node(&mut output, &expression.node);
        }
        encode_len(&mut output, accesses.len());
        for access in accesses {
            output.extend_from_slice(&access.tensor.to_be_bytes());
            output.push(match access.mode {
                AccessMode::Read => 1,
                AccessMode::Write => 2,
            });
            encode_u32_slice(&mut output, &access.domain);
            encode_u32_slice(&mut output, &access.coordinates);
        }
        encode_len(&mut output, scalars.len());
        for scalar in scalars {
            encode_compact_scalar_node(&mut output, &scalar.node);
            encode_len(&mut output, scalar.free_dimensions.len());
            for dimension in &scalar.free_dimensions {
                output.extend_from_slice(&dimension.to_be_bytes());
            }
        }
        encode_len(&mut output, outputs.len());
        for root in outputs {
            output.extend_from_slice(&root.access.to_be_bytes());
            output.extend_from_slice(&root.value.to_be_bytes());
        }
        output
    }

    fn parallel_dimensions(&self) -> Vec<u32> {
        self.dimensions
            .iter()
            .enumerate()
            .filter_map(|(index, data)| {
                if data.role == DomainRole::Parallel {
                    u32::try_from(index).ok()
                } else {
                    None
                }
            })
            .collect()
    }

    fn check_capacity(
        current: usize,
        limit: usize,
        entity: IndexEntityKind,
    ) -> Result<(), IndexBuildError> {
        if current >= limit {
            Err(IndexBuildError::StructuralLimit { entity, limit })
        } else {
            Ok(())
        }
    }

    fn check_dimension(&self, value: DimensionId) -> Result<usize, IndexBuildError> {
        self.check_handle(
            value.owner,
            value.as_usize(),
            self.dimensions.len(),
            IndexEntityKind::Dimension,
        )
    }

    fn check_tensor(&self, value: TensorId) -> Result<usize, IndexBuildError> {
        self.check_handle(
            value.owner,
            value.as_usize(),
            self.tensors.len(),
            IndexEntityKind::Tensor,
        )
    }

    fn check_index_expr(&self, value: IndexExprId) -> Result<usize, IndexBuildError> {
        self.check_handle(
            value.owner,
            value.as_usize(),
            self.expressions.len(),
            IndexEntityKind::IndexExpression,
        )
    }

    fn check_access(&self, value: TensorAccessId) -> Result<usize, IndexBuildError> {
        self.check_handle(
            value.owner,
            value.as_usize(),
            self.accesses.len(),
            IndexEntityKind::TensorAccess,
        )
    }

    fn check_scalar(&self, value: ScalarExprId) -> Result<usize, IndexBuildError> {
        self.check_handle(
            value.owner,
            value.as_usize(),
            self.scalars.len(),
            IndexEntityKind::ScalarExpression,
        )
    }

    fn check_handle(
        &self,
        owner: BuilderId,
        index: usize,
        len: usize,
        entity: IndexEntityKind,
    ) -> Result<usize, IndexBuildError> {
        if owner != self.owner {
            Err(invalid_handle(entity, true))
        } else if index >= len {
            Err(invalid_handle(entity, false))
        } else {
            Ok(index)
        }
    }

    fn dimension_handle(&self, index: u32) -> DimensionId {
        DimensionId {
            owner: self.owner,
            index,
        }
    }

    fn access_handle(&self, index: u32) -> TensorAccessId {
        TensorAccessId {
            owner: self.owner,
            index,
        }
    }

    fn scalar_handle(&self, index: u32) -> ScalarExprId {
        ScalarExprId {
            owner: self.owner,
            index,
        }
    }
}

struct EvaluationPlan {
    order: Vec<u32>,
    positions: BTreeMap<u32, usize>,
    roots: Vec<usize>,
}

impl EvaluationPlan {
    fn estimated_integer_bytes(&self, builder: &IndexRegionBuilder) -> u128 {
        self.order
            .iter()
            .fold(0_u128, |total, index| {
                let bytes = builder.expressions[*index as usize]
                    .interval
                    .as_ref()
                    .map_or(u128::from(MAX_EXHAUSTIVE_PROOF_BYTES), |(min, max)| {
                        let (_, min_bytes) = min.to_bytes_be();
                        let (_, max_bytes) = max.to_bytes_be();
                        u128::try_from(min_bytes.len().max(max_bytes.len()).saturating_add(16))
                            .expect("usize fits u128")
                    });
                total.saturating_add(bytes)
            })
            .saturating_mul(2)
    }

    fn evaluate(&self, builder: &IndexRegionBuilder, point: &[u64]) -> Vec<BigInt> {
        let mut values: Vec<BigInt> = Vec::with_capacity(self.order.len());
        for index in &self.order {
            let value = match &builder.expressions[*index as usize].node {
                IndexNode::Constant(value) => value.0.clone(),
                IndexNode::Dimension(dimension) => BigInt::from(point[*dimension as usize]),
                IndexNode::LinearCombination { constant, terms } => {
                    terms.iter().fold(constant.0.clone(), |value, term| {
                        value + &term.coefficient.0 * &values[self.positions[&term.value]]
                    })
                }
                IndexNode::FloorDiv { dividend, divisor } => {
                    values[self.positions[dividend]].div_floor(&BigInt::from(*divisor))
                }
                IndexNode::Modulo { dividend, divisor } => {
                    values[self.positions[dividend]].mod_floor(&BigInt::from(*divisor))
                }
            };
            values.push(value);
        }
        self.roots
            .iter()
            .map(|root| values[*root].clone())
            .collect()
    }
}

fn ordered_ordinal_map(values: &[u32]) -> BTreeMap<u32, u32> {
    values
        .iter()
        .enumerate()
        .map(|(new, old)| {
            (
                *old,
                u32::try_from(new).expect("bounded arena index fits u32"),
            )
        })
        .collect()
}

fn remap_index_node(node: &mut IndexNode, map: &BTreeMap<u32, u32>) {
    match node {
        IndexNode::LinearCombination { terms, .. } => {
            for term in terms {
                term.value = map[&term.value];
            }
        }
        IndexNode::FloorDiv { dividend, .. } | IndexNode::Modulo { dividend, .. } => {
            *dividend = map[dividend];
        }
        IndexNode::Constant(_) | IndexNode::Dimension(_) => {}
    }
}

fn remap_scalar_node(
    node: &mut ScalarNode,
    access_map: &BTreeMap<u32, u32>,
    scalar_map: &BTreeMap<u32, u32>,
) {
    match node {
        ScalarNode::Load { access } => *access = access_map[access],
        ScalarNode::F32Multiply { left, right } | ScalarNode::F32Add { left, right } => {
            *left = scalar_map[left];
            *right = scalar_map[right];
        }
        ScalarNode::StrictSerialF32Sum { value, .. } => *value = scalar_map[value],
        ScalarNode::F32Constant { .. } => {}
    }
}

fn encode_compact_index_node(output: &mut Vec<u8>, node: &IndexNode) {
    match node {
        IndexNode::Constant(value) => {
            output.push(1);
            value.encode(output);
        }
        IndexNode::Dimension(value) => {
            output.push(2);
            output.extend_from_slice(&value.to_be_bytes());
        }
        IndexNode::LinearCombination { constant, terms } => {
            output.push(3);
            constant.encode(output);
            encode_len(output, terms.len());
            for term in terms {
                term.coefficient.encode(output);
                output.extend_from_slice(&term.value.to_be_bytes());
            }
        }
        IndexNode::FloorDiv { dividend, divisor } => {
            output.push(4);
            output.extend_from_slice(&dividend.to_be_bytes());
            output.extend_from_slice(&divisor.to_be_bytes());
        }
        IndexNode::Modulo { dividend, divisor } => {
            output.push(5);
            output.extend_from_slice(&dividend.to_be_bytes());
            output.extend_from_slice(&divisor.to_be_bytes());
        }
    }
}

fn encode_compact_scalar_node(output: &mut Vec<u8>, node: &ScalarNode) {
    match node {
        ScalarNode::Load { access } => {
            output.push(1);
            output.extend_from_slice(&access.to_be_bytes());
        }
        ScalarNode::F32Constant { bits } => {
            output.push(2);
            output.extend_from_slice(&bits.to_be_bytes());
        }
        ScalarNode::F32Multiply { left, right } => {
            output.push(3);
            output.extend_from_slice(&left.to_be_bytes());
            output.extend_from_slice(&right.to_be_bytes());
        }
        ScalarNode::F32Add { left, right } => {
            output.push(4);
            output.extend_from_slice(&left.to_be_bytes());
            output.extend_from_slice(&right.to_be_bytes());
        }
        ScalarNode::StrictSerialF32Sum {
            value,
            dimensions,
            order: ContributorOrder::AxisLexicographic,
            empty_identity_bits,
        } => {
            output.push(5);
            output.extend_from_slice(&value.to_be_bytes());
            encode_u32_slice(output, dimensions);
            output.push(1);
            output.extend_from_slice(&empty_identity_bits.to_be_bytes());
        }
    }
}

fn encoded_integer_len(value: &IndexInteger) -> usize {
    let (_, bytes) = value.0.to_bytes_be();
    9 + bytes.len()
}

fn encode_len(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(
        &u64::try_from(value)
            .expect("bounded length fits u64")
            .to_be_bytes(),
    );
}

fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) {
    encode_len(output, value.len());
    output.extend_from_slice(value);
}

fn encode_u32_slice(output: &mut Vec<u8>, values: &[u32]) {
    encode_len(output, values.len());
    for value in values {
        output.extend_from_slice(&value.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use crate::semantic::{F32, InputKey, OutputKey, SemanticProgramBuilder};

    use super::*;

    fn builder() -> IndexRegionBuilder {
        let mut semantic = SemanticProgramBuilder::try_standard().unwrap();
        let input = semantic
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([]))
            .unwrap();
        semantic
            .output(OutputKey::new("output").unwrap(), input)
            .unwrap();
        let semantic = semantic.build().unwrap();
        IndexRegionBuilder::new(SemanticRegionIdentity::for_program(&semantic)).unwrap()
    }

    #[test]
    fn aggregate_boundary_budget_fails_before_tensor_mutation() {
        let mut builder = builder();
        builder.boundary_canonical_bytes = MAX_BOUNDARY_CANONICAL_BYTES - 1;

        assert!(matches!(
            builder.tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([]),
            ),
            Err(IndexBuildError::StructuralLimit {
                entity: IndexEntityKind::Tensor,
                limit: MAX_BOUNDARY_CANONICAL_BYTES,
            })
        ));
        assert!(builder.tensors.is_empty());
        assert_eq!(
            builder.boundary_canonical_bytes,
            MAX_BOUNDARY_CANONICAL_BYTES - 1
        );
    }
}
