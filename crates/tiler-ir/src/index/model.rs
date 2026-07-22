use std::collections::BTreeSet;
use std::sync::Arc;

use crate::semantic::ResolvedValueType;
use crate::shape::{Extent, Shape};

use super::handles::VerifiedRegionOwner;
use super::{
    IndexEntityKind, IndexInteger, ScalarAttributes, ScalarOpKey, ScalarResultIndex,
    VerifiedDimensionId, VerifiedIndexExprId, VerifiedIndexHandleError,
    VerifiedReducerBodyOperationId, VerifiedReducerBodyValueId, VerifiedScalarOperationId,
    VerifiedScalarValueId, VerifiedTensorAccessId, VerifiedTensorId,
};

/// Whether one boundary tensor is consumed or produced.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TensorRole {
    /// Caller-provided tensor boundary.
    Input,
    /// Region-produced tensor boundary.
    Output,
}
/// Whether a dimension names output points or reduction contributors.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DomainRole {
    /// Dimension that remains free across output elements.
    Parallel,
    /// Dimension consumed by an explicit reduction.
    Reduction,
}
/// Logical tensor access mode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AccessMode {
    /// Logical tensor read.
    Read,
    /// Logical tensor write.
    Write,
}
/// Exact reduction traversal contract.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ReductionTraversal {
    /// Visit the Cartesian domain in dimension order, folding strictly left.
    ExactLexicographicLeftFold,
}
/// Index expression classification.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexExprClass {
    /// Integer-affine expression.
    Affine,
    /// Affine expression extended with constant floor division or modulo.
    QuasiAffine,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct LinearTermData {
    pub coefficient: IndexInteger,
    pub value: u32,
}
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum IndexNode {
    Constant(IndexInteger),
    Dimension(u32),
    LinearCombination {
        constant: IndexInteger,
        terms: Vec<LinearTermData>,
    },
    FloorDiv {
        dividend: u32,
        divisor: u64,
    },
    Modulo {
        dividend: u32,
        divisor: u64,
    },
}
#[derive(Clone, Debug)]
pub(super) struct IndexExprData {
    pub node: IndexNode,
    pub class: IndexExprClass,
}
#[derive(Clone, Debug)]
pub(super) struct DimensionData {
    pub role: DomainRole,
    pub extent: u64,
}
#[derive(Clone, Debug)]
pub(super) struct TensorData {
    pub role: TensorRole,
    pub value_type: ResolvedValueType,
    pub shape: Shape,
}
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct AccessData {
    pub tensor: u32,
    pub mode: AccessMode,
    pub domain: Vec<u32>,
    pub coordinates: Vec<u32>,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum BoundsProof {
    VacuousEmptyDomain,
    Interval,
    Exhaustive { points: u64 },
}
#[derive(Clone, Copy, Debug)]
pub(super) enum WriteOwnershipProof {
    CoordinatePermutation,
    Exhaustive { points: u64 },
}
#[derive(Clone, Debug)]
pub(super) struct VerifiedAccessData {
    pub tensor: u32,
    pub mode: AccessMode,
    pub domain: Vec<u32>,
    pub coordinates: Vec<u32>,
    pub bounds_proof: BoundsProof,
    pub ownership_proof: Option<WriteOwnershipProof>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum ScalarValueDefinition {
    AccessRead {
        access: u32,
    },
    OperationResult {
        operation: u32,
        result: ScalarResultIndex,
    },
}
#[derive(Clone, Debug)]
pub(super) struct ScalarValueData {
    pub definition: ScalarValueDefinition,
    pub value_type: ResolvedValueType,
    pub free_dimensions: BTreeSet<u32>,
    pub depth: u32,
}

#[derive(Clone, Debug)]
pub(super) enum ReducerBodyValueSource {
    StateParameter(u32),
    ContributorParameter(u32),
    OperationResult {
        operation: u32,
        result: ScalarResultIndex,
    },
}
#[derive(Clone, Debug)]
pub(super) struct ReducerBodyValueData {
    pub source: ReducerBodyValueSource,
    pub value_type: ResolvedValueType,
}
#[derive(Clone, Debug)]
pub(super) struct ReducerBodyOperationData {
    pub key: ScalarOpKey,
    pub attributes: ScalarAttributes,
    pub operands: Vec<u32>,
    pub results: Vec<u32>,
}
#[derive(Clone, Debug)]
pub(super) struct ScalarReducerBodyData {
    pub values: Vec<ReducerBodyValueData>,
    pub operations: Vec<ReducerBodyOperationData>,
    pub yields: Vec<u32>,
}

#[derive(Clone, Debug)]
pub(super) enum ScalarOperationKindData {
    Apply {
        key: ScalarOpKey,
        attributes: ScalarAttributes,
    },
    Reduce {
        dimensions: Vec<u32>,
        traversal: ReductionTraversal,
        init: Vec<u32>,
        contributors: Vec<u32>,
        body: ScalarReducerBodyData,
    },
}
#[derive(Clone, Debug)]
pub(super) struct ScalarOperationData {
    pub kind: ScalarOperationKindData,
    pub operands: Vec<u32>,
    pub results: Vec<u32>,
    pub depth: u32,
}
#[derive(Clone, Debug)]
pub(super) struct OutputData {
    pub access: u32,
    pub value: u32,
}

/// Opaque canonical bytes for one verified region.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalIndexRegionIdentity(pub(super) Vec<u8>);
impl CanonicalIndexRegionIdentity {
    /// Returns canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Immutable, compacted and verified index region.
#[derive(Clone, Debug)]
pub struct VerifiedIndexRegion {
    pub(super) data: Arc<VerifiedIndexRegionData>,
}
#[derive(Clone, Debug)]
pub(super) struct VerifiedIndexRegionData {
    pub owner: VerifiedRegionOwner,
    pub dimensions: Vec<DimensionData>,
    pub tensors: Vec<TensorData>,
    pub expressions: Vec<IndexExprData>,
    pub accesses: Vec<VerifiedAccessData>,
    pub operations: Vec<ScalarOperationData>,
    pub values: Vec<ScalarValueData>,
    pub outputs: Vec<OutputData>,
    pub identity: CanonicalIndexRegionIdentity,
}

impl VerifiedIndexRegion {
    /// Returns canonical structural identity.
    #[must_use]
    pub fn canonical_identity(&self) -> &CanonicalIndexRegionIdentity {
        &self.data.identity
    }
    /// Returns dimensions in canonical order.
    #[must_use]
    pub fn dimensions(&self) -> impl ExactSizeIterator<Item = DomainDimensionRef<'_>> {
        self.data
            .dimensions
            .iter()
            .enumerate()
            .map(|(i, data)| DomainDimensionRef {
                id: self.dimension_id(i),
                data,
            })
    }
    /// Returns boundary tensors, inputs before outputs.
    #[must_use]
    pub fn tensors(&self) -> impl ExactSizeIterator<Item = TensorRef<'_>> {
        self.data
            .tensors
            .iter()
            .enumerate()
            .map(|(i, data)| TensorRef {
                id: self.tensor_id(i),
                data,
            })
    }
    /// Returns index expressions.
    #[must_use]
    pub fn index_expressions(&self) -> impl ExactSizeIterator<Item = IndexExprRef<'_>> {
        self.data
            .expressions
            .iter()
            .enumerate()
            .map(|(i, data)| IndexExprRef {
                id: self.expr_id(i),
                data,
            })
    }
    /// Returns logical accesses.
    #[must_use]
    pub fn accesses(&self) -> impl ExactSizeIterator<Item = TensorAccessRef<'_>> {
        self.data
            .accesses
            .iter()
            .enumerate()
            .map(|(i, data)| TensorAccessRef {
                id: self.access_id(i),
                data,
                region: self,
            })
    }
    /// Returns scalar operation occurrences.
    #[must_use]
    pub fn scalar_operations(&self) -> impl ExactSizeIterator<Item = ScalarOperationRef<'_>> {
        self.data
            .operations
            .iter()
            .enumerate()
            .map(|(i, data)| ScalarOperationRef {
                id: self.operation_id(i),
                data,
                region: self,
            })
    }
    /// Returns scalar SSA values.
    #[must_use]
    pub fn scalar_values(&self) -> impl ExactSizeIterator<Item = ScalarValueRef<'_>> {
        self.data
            .values
            .iter()
            .enumerate()
            .map(|(i, data)| ScalarValueRef {
                id: self.value_id(i),
                data,
                region: self,
            })
    }
    /// Returns ordered graph outputs.
    #[must_use]
    pub fn outputs(&self) -> impl ExactSizeIterator<Item = OutputRef<'_>> {
        self.data
            .outputs
            .iter()
            .map(|data| OutputRef { data, region: self })
    }

    fn dimension_id(&self, i: usize) -> VerifiedDimensionId {
        VerifiedDimensionId::from_verified(self.data.owner, u32::try_from(i).expect("bounded"))
    }
    fn tensor_id(&self, i: usize) -> VerifiedTensorId {
        VerifiedTensorId::from_verified(self.data.owner, u32::try_from(i).expect("bounded"))
    }
    fn expr_id(&self, i: usize) -> VerifiedIndexExprId {
        VerifiedIndexExprId::from_verified(self.data.owner, u32::try_from(i).expect("bounded"))
    }
    fn access_id(&self, i: usize) -> VerifiedTensorAccessId {
        VerifiedTensorAccessId::from_verified(self.data.owner, u32::try_from(i).expect("bounded"))
    }
    fn operation_id(&self, i: usize) -> VerifiedScalarOperationId {
        VerifiedScalarOperationId::from_verified(
            self.data.owner,
            u32::try_from(i).expect("bounded"),
        )
    }
    fn value_id(&self, i: usize) -> VerifiedScalarValueId {
        VerifiedScalarValueId::from_verified(self.data.owner, u32::try_from(i).expect("bounded"))
    }

    /// Resolves a verified scalar value handle.
    ///
    /// # Errors
    ///
    /// Returns an error for a foreign-region or invalid handle.
    pub fn scalar_value(
        &self,
        id: VerifiedScalarValueId,
    ) -> Result<ScalarValueRef<'_>, VerifiedIndexHandleError> {
        if id.owner != self.data.owner {
            return Err(VerifiedIndexHandleError::ForeignRegion {
                entity: IndexEntityKind::ScalarValue,
            });
        }
        let data =
            self.data
                .values
                .get(id.as_usize())
                .ok_or(VerifiedIndexHandleError::InvalidHandle {
                    entity: IndexEntityKind::ScalarValue,
                })?;
        Ok(ScalarValueRef {
            id,
            data,
            region: self,
        })
    }
    /// Resolves a domain dimension in constant time.
    ///
    /// # Errors
    ///
    /// Returns an error for a foreign-region or invalid handle.
    pub fn dimension(
        &self,
        id: VerifiedDimensionId,
    ) -> Result<DomainDimensionRef<'_>, VerifiedIndexHandleError> {
        self.check_owner(id.owner, IndexEntityKind::Dimension)?;
        let data = self.data.dimensions.get(id.as_usize()).ok_or(
            VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::Dimension,
            },
        )?;
        Ok(DomainDimensionRef { id, data })
    }
    /// Resolves a boundary tensor in constant time.
    ///
    /// # Errors
    ///
    /// Returns an error for a foreign-region or invalid handle.
    pub fn tensor(&self, id: VerifiedTensorId) -> Result<TensorRef<'_>, VerifiedIndexHandleError> {
        self.check_owner(id.owner, IndexEntityKind::Tensor)?;
        let data = self.data.tensors.get(id.as_usize()).ok_or(
            VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::Tensor,
            },
        )?;
        Ok(TensorRef { id, data })
    }
    /// Resolves an index expression in constant time.
    ///
    /// # Errors
    ///
    /// Returns an error for a foreign-region or invalid handle.
    pub fn index_expression(
        &self,
        id: VerifiedIndexExprId,
    ) -> Result<IndexExprRef<'_>, VerifiedIndexHandleError> {
        self.check_owner(id.owner, IndexEntityKind::IndexExpression)?;
        let data = self.data.expressions.get(id.as_usize()).ok_or(
            VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::IndexExpression,
            },
        )?;
        Ok(IndexExprRef { id, data })
    }
    /// Resolves a logical access in constant time.
    ///
    /// # Errors
    ///
    /// Returns an error for a foreign-region or invalid handle.
    pub fn access(
        &self,
        id: VerifiedTensorAccessId,
    ) -> Result<TensorAccessRef<'_>, VerifiedIndexHandleError> {
        self.check_owner(id.owner, IndexEntityKind::TensorAccess)?;
        let data = self.data.accesses.get(id.as_usize()).ok_or(
            VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::TensorAccess,
            },
        )?;
        Ok(TensorAccessRef {
            id,
            data,
            region: self,
        })
    }
    /// Resolves a scalar operation in constant time.
    ///
    /// # Errors
    ///
    /// Returns an error for a foreign-region or invalid handle.
    pub fn scalar_operation(
        &self,
        id: VerifiedScalarOperationId,
    ) -> Result<ScalarOperationRef<'_>, VerifiedIndexHandleError> {
        self.check_owner(id.owner, IndexEntityKind::ScalarOperation)?;
        let data = self.data.operations.get(id.as_usize()).ok_or(
            VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::ScalarOperation,
            },
        )?;
        Ok(ScalarOperationRef {
            id,
            data,
            region: self,
        })
    }
    fn check_owner(
        &self,
        owner: VerifiedRegionOwner,
        entity: IndexEntityKind,
    ) -> Result<(), VerifiedIndexHandleError> {
        if owner == self.data.owner {
            Ok(())
        } else {
            Err(VerifiedIndexHandleError::ForeignRegion { entity })
        }
    }
    /// Resolves a reducer-body value in constant time.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid region, reduction, or local index.
    pub fn reducer_body_value(
        &self,
        id: VerifiedReducerBodyValueId,
    ) -> Result<ReducerBodyValueRef<'_>, VerifiedIndexHandleError> {
        self.check_owner(id.owner, IndexEntityKind::ScalarValue)?;
        let operation = self.data.operations.get(id.reduction as usize).ok_or(
            VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::ScalarOperation,
            },
        )?;
        let ScalarOperationKindData::Reduce { body, .. } = &operation.kind else {
            return Err(VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::ScalarOperation,
            });
        };
        let data =
            body.values
                .get(id.index as usize)
                .ok_or(VerifiedIndexHandleError::InvalidHandle {
                    entity: IndexEntityKind::ScalarValue,
                })?;
        Ok(ReducerBodyValueRef { id, data })
    }
    /// Resolves a reducer-body operation in constant time.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid region, reduction, or local index.
    pub fn reducer_body_operation(
        &self,
        id: VerifiedReducerBodyOperationId,
    ) -> Result<ReducerBodyOperationRef<'_>, VerifiedIndexHandleError> {
        self.check_owner(id.owner, IndexEntityKind::ScalarOperation)?;
        let operation = self.data.operations.get(id.reduction as usize).ok_or(
            VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::ScalarOperation,
            },
        )?;
        let ScalarOperationKindData::Reduce { body, .. } = &operation.kind else {
            return Err(VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::ScalarOperation,
            });
        };
        let data = body.operations.get(id.index as usize).ok_or(
            VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::ScalarOperation,
            },
        )?;
        Ok(ReducerBodyOperationRef { id, data })
    }
}

/// Borrowed dimension inspection.
#[derive(Clone, Copy, Debug)]
pub struct DomainDimensionRef<'a> {
    id: VerifiedDimensionId,
    data: &'a DimensionData,
}
impl DomainDimensionRef<'_> {
    /// Returns the verified dimension identity.
    #[must_use]
    pub const fn id(self) -> VerifiedDimensionId {
        self.id
    }
    /// Returns the semantic dimension role.
    #[must_use]
    pub const fn role(self) -> DomainRole {
        self.data.role
    }
    /// Returns the static half-open extent in this bounded profile.
    ///
    /// A future symbolic dimension returns `None` here and exposes its expression separately.
    #[must_use]
    pub const fn static_extent(self) -> Option<Extent> {
        Some(Extent::new(self.data.extent))
    }
}
/// Borrowed tensor inspection.
#[derive(Clone, Copy, Debug)]
pub struct TensorRef<'a> {
    id: VerifiedTensorId,
    data: &'a TensorData,
}
impl<'a> TensorRef<'a> {
    /// Returns the verified tensor identity.
    #[must_use]
    pub const fn id(self) -> VerifiedTensorId {
        self.id
    }
    /// Returns the boundary role.
    #[must_use]
    pub const fn role(self) -> TensorRole {
        self.data.role
    }
    /// Returns the complete semantic element type.
    #[must_use]
    pub const fn value_type(self) -> &'a ResolvedValueType {
        &self.data.value_type
    }
    /// Returns the exact static tensor shape in this bounded profile.
    ///
    /// A future symbolic boundary returns `None` here and exposes shape expressions separately.
    #[must_use]
    pub const fn static_shape(self) -> Option<&'a Shape> {
        Some(&self.data.shape)
    }
}

/// Borrowed index expression inspection.
#[derive(Clone, Copy, Debug)]
pub struct IndexExprRef<'a> {
    id: VerifiedIndexExprId,
    data: &'a IndexExprData,
}
/// One index expression view.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum IndexExprView<'a> {
    /// Exact integer constant.
    Constant(&'a IndexInteger),
    /// Reference to a domain dimension.
    Dimension(VerifiedDimensionId),
    /// Normalized affine sum.
    LinearCombination {
        /// Additive constant.
        constant: &'a IndexInteger,
        /// Ordered, combined nonzero terms.
        terms: LinearTerms<'a>,
    },
    /// Euclidean floor division by a positive constant.
    FloorDiv {
        /// Dividend expression.
        dividend: VerifiedIndexExprId,
        /// Positive divisor.
        divisor: u64,
    },
    /// Euclidean modulo by a positive constant.
    Modulo {
        /// Dividend expression.
        dividend: VerifiedIndexExprId,
        /// Positive divisor.
        divisor: u64,
    },
}
/// Iterator over ordered normalized linear terms.
#[derive(Clone, Debug)]
pub struct LinearTerms<'a> {
    inner: std::slice::Iter<'a, LinearTermData>,
    owner: VerifiedRegionOwner,
}
impl<'a> Iterator for LinearTerms<'a> {
    type Item = LinearTermRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|data| LinearTermRef {
            data,
            owner: self.owner,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl ExactSizeIterator for LinearTerms<'_> {}
/// Borrowed normalized linear term.
#[derive(Clone, Copy, Debug)]
pub struct LinearTermRef<'a> {
    data: &'a LinearTermData,
    owner: VerifiedRegionOwner,
}
impl<'a> LinearTermRef<'a> {
    /// Returns the exact coefficient.
    #[must_use]
    pub const fn coefficient(self) -> &'a IndexInteger {
        &self.data.coefficient
    }
    /// Returns the referenced child expression.
    #[must_use]
    pub fn value(self) -> VerifiedIndexExprId {
        VerifiedIndexExprId::from_verified(self.owner, self.data.value)
    }
}
impl<'a> IndexExprRef<'a> {
    /// Returns the verified expression identity.
    #[must_use]
    pub const fn id(self) -> VerifiedIndexExprId {
        self.id
    }
    /// Returns the strongest implemented expression class.
    #[must_use]
    pub const fn class(self) -> IndexExprClass {
        self.data.class
    }
    /// Returns the typed structural view.
    #[must_use]
    pub fn view(self) -> IndexExprView<'a> {
        match &self.data.node {
            IndexNode::Constant(v) => IndexExprView::Constant(v),
            IndexNode::Dimension(i) => {
                IndexExprView::Dimension(VerifiedDimensionId::from_verified(self.id.owner, *i))
            }
            IndexNode::LinearCombination { constant, terms } => IndexExprView::LinearCombination {
                constant,
                terms: LinearTerms {
                    inner: terms.iter(),
                    owner: self.id.owner,
                },
            },
            IndexNode::FloorDiv { dividend, divisor } => IndexExprView::FloorDiv {
                dividend: VerifiedIndexExprId::from_verified(self.id.owner, *dividend),
                divisor: *divisor,
            },
            IndexNode::Modulo { dividend, divisor } => IndexExprView::Modulo {
                dividend: VerifiedIndexExprId::from_verified(self.id.owner, *dividend),
                divisor: *divisor,
            },
        }
    }
}

/// Borrowed tensor-access inspection.
#[derive(Clone, Copy, Debug)]
pub struct TensorAccessRef<'a> {
    id: VerifiedTensorAccessId,
    data: &'a VerifiedAccessData,
    region: &'a VerifiedIndexRegion,
}
impl<'a> TensorAccessRef<'a> {
    /// Returns the verified access identity.
    #[must_use]
    pub const fn id(self) -> VerifiedTensorAccessId {
        self.id
    }
    /// Returns whether this access reads or writes.
    #[must_use]
    pub const fn mode(self) -> AccessMode {
        self.data.mode
    }
    /// Returns the referenced tensor boundary.
    #[must_use]
    pub fn tensor(self) -> VerifiedTensorId {
        self.region.tensor_id(self.data.tensor as usize)
    }
    /// Returns the canonical in-scope dimension set.
    #[must_use]
    pub fn domain(self) -> impl ExactSizeIterator<Item = VerifiedDimensionId> + 'a {
        let owner = self.region.data.owner;
        self.data
            .domain
            .iter()
            .copied()
            .map(move |index| VerifiedDimensionId::from_verified(owner, index))
    }
    /// Returns ordered tensor-coordinate expressions.
    #[must_use]
    pub fn coordinates(self) -> impl ExactSizeIterator<Item = VerifiedIndexExprId> + 'a {
        let owner = self.region.data.owner;
        self.data
            .coordinates
            .iter()
            .copied()
            .map(move |index| VerifiedIndexExprId::from_verified(owner, index))
    }
    /// Returns retained bounds evidence.
    #[must_use]
    pub fn bounds_proof(self) -> BoundsProofView {
        match self.data.bounds_proof {
            BoundsProof::VacuousEmptyDomain => BoundsProofView::VacuousEmptyDomain,
            BoundsProof::Interval => BoundsProofView::Interval,
            BoundsProof::Exhaustive { points } => BoundsProofView::Exhaustive { points },
        }
    }
    /// Returns retained complete-write evidence when this is a write.
    #[must_use]
    pub fn write_ownership_proof(self) -> Option<WriteOwnershipProofView> {
        self.data.ownership_proof.map(|proof| match proof {
            WriteOwnershipProof::CoordinatePermutation => {
                WriteOwnershipProofView::CoordinatePermutation
            }
            WriteOwnershipProof::Exhaustive { points } => {
                WriteOwnershipProofView::Exhaustive { points }
            }
        })
    }
}

/// Public view of one sound bounds proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BoundsProofView {
    /// The iteration domain is empty, so bounds hold vacuously.
    VacuousEmptyDomain,
    /// Exact interval propagation proved every coordinate in bounds.
    Interval,
    /// Finite enumeration proved every coordinate in bounds.
    Exhaustive {
        /// Enumerated domain points.
        points: u64,
    },
}
/// Public view of one sound complete-write proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WriteOwnershipProofView {
    /// Coordinates are a dimension permutation matching output shape.
    CoordinatePermutation,
    /// Finite enumeration proved total, injective ownership.
    Exhaustive {
        /// Enumerated domain points.
        points: u64,
    },
}

/// One scalar value definition.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum ScalarValueDefinitionView {
    /// Value loaded by a logical read access.
    AccessRead(VerifiedTensorAccessId),
    /// Ordered result of a scalar operation occurrence.
    OperationResult {
        /// Defining operation.
        operation: VerifiedScalarOperationId,
        /// Result position.
        result: ScalarResultIndex,
    },
}
/// Borrowed scalar SSA value.
#[derive(Clone, Copy, Debug)]
pub struct ScalarValueRef<'a> {
    id: VerifiedScalarValueId,
    data: &'a ScalarValueData,
    region: &'a VerifiedIndexRegion,
}
impl<'a> ScalarValueRef<'a> {
    /// Returns the verified scalar value identity.
    #[must_use]
    pub const fn id(self) -> VerifiedScalarValueId {
        self.id
    }
    /// Returns the complete inferred semantic type.
    #[must_use]
    pub const fn value_type(self) -> &'a ResolvedValueType {
        &self.data.value_type
    }
    /// Returns free iteration dimensions in canonical order.
    #[must_use]
    pub fn free_dimensions(self) -> impl ExactSizeIterator<Item = VerifiedDimensionId> + 'a {
        let owner = self.region.data.owner;
        self.data
            .free_dimensions
            .iter()
            .copied()
            .map(move |i| VerifiedDimensionId::from_verified(owner, i))
    }
    /// Returns the SSA definition.
    #[must_use]
    pub fn definition(self) -> ScalarValueDefinitionView {
        match self.data.definition {
            ScalarValueDefinition::AccessRead { access } => {
                ScalarValueDefinitionView::AccessRead(self.region.access_id(access as usize))
            }
            ScalarValueDefinition::OperationResult { operation, result } => {
                ScalarValueDefinitionView::OperationResult {
                    operation: self.region.operation_id(operation as usize),
                    result,
                }
            }
        }
    }
}

/// One scalar operation kind.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum ScalarOperationKindRef<'a> {
    /// One registered pointwise scalar application.
    Apply {
        /// Stable operation identity.
        key: &'a ScalarOpKey,
        /// Checked canonical attributes.
        attributes: &'a ScalarAttributes,
    },
    /// One exact N-state reduction occurrence.
    Reduce(ScalarReductionRef<'a>),
}
/// Borrowed inspection of one exact reduction occurrence.
#[derive(Clone, Copy, Debug)]
pub struct ScalarReductionRef<'a> {
    operation: u32,
    dimensions: &'a [u32],
    traversal: ReductionTraversal,
    init: &'a [u32],
    contributors: &'a [u32],
    body: &'a ScalarReducerBodyData,
    region: &'a VerifiedIndexRegion,
}
impl<'a> ScalarReductionRef<'a> {
    /// Returns ordered lexicographic reduction dimensions.
    #[must_use]
    pub fn dimensions(self) -> impl ExactSizeIterator<Item = VerifiedDimensionId> + 'a {
        let owner = self.region.data.owner;
        self.dimensions
            .iter()
            .copied()
            .map(move |index| VerifiedDimensionId::from_verified(owner, index))
    }
    /// Returns the exact traversal contract.
    #[must_use]
    pub const fn traversal(self) -> ReductionTraversal {
        self.traversal
    }
    /// Returns ordered initial state values.
    #[must_use]
    pub fn init(self) -> impl ExactSizeIterator<Item = VerifiedScalarValueId> + 'a {
        self.init
            .iter()
            .copied()
            .map(move |index| self.region.value_id(index as usize))
    }
    /// Returns ordered contributor values.
    #[must_use]
    pub fn contributors(self) -> impl ExactSizeIterator<Item = VerifiedScalarValueId> + 'a {
        self.contributors
            .iter()
            .copied()
            .map(move |index| self.region.value_id(index as usize))
    }
    /// Returns the closed reducer-body SSA region.
    #[must_use]
    pub const fn body(self) -> ScalarReducerBodyRef<'a> {
        ScalarReducerBodyRef {
            data: self.body,
            reduction: self.operation,
            region: self.region,
        }
    }
}
/// Borrowed inspection of one closed reducer-body SSA region.
#[derive(Clone, Copy, Debug)]
pub struct ScalarReducerBodyRef<'a> {
    data: &'a ScalarReducerBodyData,
    reduction: u32,
    region: &'a VerifiedIndexRegion,
}
impl<'a> ScalarReducerBodyRef<'a> {
    /// Returns all body-local values in canonical order.
    #[must_use]
    pub fn values(self) -> impl ExactSizeIterator<Item = ReducerBodyValueRef<'a>> {
        self.data
            .values
            .iter()
            .zip(0..verified_count(self.data.values.len()))
            .map(move |(data, index)| ReducerBodyValueRef {
                id: VerifiedReducerBodyValueId {
                    owner: self.region.data.owner,
                    reduction: self.reduction,
                    index,
                },
                data,
            })
    }
    /// Returns generic scalar applications retained in the reachable body.
    #[must_use]
    pub fn operations(self) -> impl ExactSizeIterator<Item = ReducerBodyOperationRef<'a>> {
        self.data
            .operations
            .iter()
            .zip(0..verified_count(self.data.operations.len()))
            .map(move |(data, index)| ReducerBodyOperationRef {
                id: VerifiedReducerBodyOperationId {
                    owner: self.region.data.owner,
                    reduction: self.reduction,
                    index,
                },
                data,
            })
    }
    /// Returns the ordered yielded state values.
    #[must_use]
    pub fn yields(self) -> impl ExactSizeIterator<Item = VerifiedReducerBodyValueId> + 'a {
        self.data
            .yields
            .iter()
            .copied()
            .map(move |index| VerifiedReducerBodyValueId {
                owner: self.region.data.owner,
                reduction: self.reduction,
                index,
            })
    }
}
/// Definition of one reducer-body SSA value.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum ReducerBodyValueDefinitionView {
    /// Ordered accumulator state parameter.
    StateParameter(u32),
    /// Ordered contributor parameter.
    ContributorParameter(u32),
    /// Result of one body-local scalar operation.
    OperationResult {
        /// Defining body-local operation.
        operation: VerifiedReducerBodyOperationId,
        /// Result position.
        result: ScalarResultIndex,
    },
}
/// Borrowed reducer-body value inspection.
#[derive(Clone, Copy, Debug)]
pub struct ReducerBodyValueRef<'a> {
    id: VerifiedReducerBodyValueId,
    data: &'a ReducerBodyValueData,
}
impl<'a> ReducerBodyValueRef<'a> {
    /// Returns the owner-checked local value identity.
    #[must_use]
    pub const fn id(self) -> VerifiedReducerBodyValueId {
        self.id
    }
    /// Returns the complete inferred semantic type.
    #[must_use]
    pub const fn value_type(self) -> &'a ResolvedValueType {
        &self.data.value_type
    }
    /// Returns the SSA definition.
    #[must_use]
    pub fn definition(self) -> ReducerBodyValueDefinitionView {
        match self.data.source {
            ReducerBodyValueSource::StateParameter(index) => {
                ReducerBodyValueDefinitionView::StateParameter(index)
            }
            ReducerBodyValueSource::ContributorParameter(index) => {
                ReducerBodyValueDefinitionView::ContributorParameter(index)
            }
            ReducerBodyValueSource::OperationResult { operation, result } => {
                ReducerBodyValueDefinitionView::OperationResult {
                    operation: VerifiedReducerBodyOperationId {
                        owner: self.id.owner,
                        reduction: self.id.reduction,
                        index: operation,
                    },
                    result,
                }
            }
        }
    }
}
/// Borrowed inspection of one generic application in a reducer body.
#[derive(Clone, Copy, Debug)]
pub struct ReducerBodyOperationRef<'a> {
    id: VerifiedReducerBodyOperationId,
    data: &'a ReducerBodyOperationData,
}
impl<'a> ReducerBodyOperationRef<'a> {
    /// Returns the owner-checked local operation identity.
    #[must_use]
    pub const fn id(self) -> VerifiedReducerBodyOperationId {
        self.id
    }
    /// Returns the registered scalar operation identity.
    #[must_use]
    pub const fn key(self) -> &'a ScalarOpKey {
        &self.data.key
    }
    /// Returns checked canonical attributes.
    #[must_use]
    pub const fn attributes(self) -> &'a ScalarAttributes {
        &self.data.attributes
    }
    /// Returns ordered operands.
    #[must_use]
    pub fn operands(self) -> impl ExactSizeIterator<Item = VerifiedReducerBodyValueId> + 'a {
        self.data
            .operands
            .iter()
            .copied()
            .map(move |index| VerifiedReducerBodyValueId {
                owner: self.id.owner,
                reduction: self.id.reduction,
                index,
            })
    }
    /// Returns ordered inferred results.
    #[must_use]
    pub fn results(self) -> impl ExactSizeIterator<Item = VerifiedReducerBodyValueId> + 'a {
        self.data
            .results
            .iter()
            .copied()
            .map(move |index| VerifiedReducerBodyValueId {
                owner: self.id.owner,
                reduction: self.id.reduction,
                index,
            })
    }
}
/// Borrowed scalar operation occurrence.
#[derive(Clone, Copy, Debug)]
pub struct ScalarOperationRef<'a> {
    id: VerifiedScalarOperationId,
    data: &'a ScalarOperationData,
    region: &'a VerifiedIndexRegion,
}
impl<'a> ScalarOperationRef<'a> {
    /// Returns the verified operation identity.
    #[must_use]
    pub const fn id(self) -> VerifiedScalarOperationId {
        self.id
    }
    /// Returns ordered operand values.
    #[must_use]
    pub fn operands(self) -> impl ExactSizeIterator<Item = VerifiedScalarValueId> + 'a {
        self.data
            .operands
            .iter()
            .copied()
            .map(move |i| self.region.value_id(i as usize))
    }
    /// Returns ordered result values.
    #[must_use]
    pub fn results(self) -> impl ExactSizeIterator<Item = VerifiedScalarValueId> + 'a {
        self.data
            .results
            .iter()
            .copied()
            .map(move |i| self.region.value_id(i as usize))
    }
    /// Returns the typed operation-kind view.
    #[must_use]
    pub fn kind(self) -> ScalarOperationKindRef<'a> {
        match &self.data.kind {
            ScalarOperationKindData::Apply { key, attributes } => {
                ScalarOperationKindRef::Apply { key, attributes }
            }
            ScalarOperationKindData::Reduce {
                dimensions,
                traversal,
                body,
                init,
                contributors,
            } => ScalarOperationKindRef::Reduce(ScalarReductionRef {
                operation: self.id.index,
                dimensions,
                traversal: *traversal,
                init,
                contributors,
                body,
                region: self.region,
            }),
        }
    }
}

/// Borrowed output root.
#[derive(Clone, Copy, Debug)]
pub struct OutputRef<'a> {
    data: &'a OutputData,
    region: &'a VerifiedIndexRegion,
}
impl OutputRef<'_> {
    /// Returns the logical write access bound to this root.
    #[must_use]
    pub fn access(self) -> VerifiedTensorAccessId {
        self.region.access_id(self.data.access as usize)
    }
    /// Returns the scalar value written by this root.
    #[must_use]
    pub fn value(self) -> VerifiedScalarValueId {
        self.region.value_id(self.data.value as usize)
    }
}

fn verified_count(count: usize) -> u32 {
    u32::try_from(count).expect("verified region entity counts fit u32")
}
