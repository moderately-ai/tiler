use std::collections::BTreeSet;
use std::sync::Arc;

use crate::semantic::ResolvedValueType;
use crate::shape::{ExtentSources, SourcedExtent, SourcedShape};

use super::handles::VerifiedRegionOwner;
use super::sourced::SourcedIndexInteger;
use super::{
    DischargedIndexDomainPredicate, IndexDomainFactSource, IndexDomainPredicate, IndexEntityKind,
    IndexExtentRef, IndexInteger, ScalarAttributes, ScalarOpKey, ScalarResultIndex,
    UnknownIndexDomainPredicate, VerifiedDimensionId, VerifiedIndexExprId,
    VerifiedIndexHandleError, VerifiedReducerBodyOperationId, VerifiedReducerBodyValueId,
    VerifiedScalarOperationId, VerifiedScalarValueId, VerifiedTensorAccessId, VerifiedTensorId,
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
///
/// The class describes the expression's *form*, never what a particular
/// environment happens to resolve it to. A division by a symbol the environment
/// pins to a single value is still [`Self::SemiAffine`]: the region's canonical
/// bytes name the symbol, two environments can bind it differently, and a class
/// that moved with the binding would describe the environment rather than the
/// expression. A pass that can use the pinned value reads it explicitly through
/// [`ExtentSources::determined`](crate::shape::ExtentSources::determined).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexExprClass {
    /// Integer-affine expression.
    Affine,
    /// Affine expression extended with constant floor division or modulo.
    QuasiAffine,
    /// Affine expression extended with a symbolic divisor, coefficient, or
    /// addend.
    ///
    /// ADR 0046 admits both halves — "symbolic coefficients **or**
    /// proven-positive symbolic divisors" — and one class covers them because
    /// each makes the expression nonlinear in the environment's variables for
    /// the same reason: a value the region names but does not fix participates
    /// in the arithmetic. The two halves differ only in their admission
    /// predicate. A divisor must be proved positive, because `x floordiv 0` is
    /// undefined; a coefficient or addend must not, because it may be any
    /// magnitude the environment admits, zero included, and every one of those
    /// denotes a coordinate.
    ///
    /// ADR 0046 permits a pass to "conservatively decline semi-affine maps they
    /// cannot analyze", and declining is what every pass here does: a refusal
    /// carries a typed reason, where approximating would return a coordinate
    /// nobody proved.
    SemiAffine,
}

impl IndexExprClass {
    /// Returns the weakest class that admits both operands.
    ///
    /// Exhaustive by construction rather than a comparison on the derived
    /// order, so a new class is a build error here instead of silently sorting
    /// itself into a lattice position nobody chose.
    #[must_use]
    pub const fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::SemiAffine, _) | (_, Self::SemiAffine) => Self::SemiAffine,
            (Self::QuasiAffine, _) | (_, Self::QuasiAffine) => Self::QuasiAffine,
            (Self::Affine, Self::Affine) => Self::Affine,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct LinearTermData {
    pub coefficient: SourcedIndexInteger,
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
        divisor: SourcedExtent,
    },
    Modulo {
        dividend: u32,
        divisor: SourcedExtent,
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
    pub extent: SourcedExtent,
}
#[derive(Clone, Debug)]
pub(super) struct TensorData {
    pub role: TensorRole,
    pub value_type: ResolvedValueType,
    pub shape: SourcedShape,
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
    ProvedExtentEquality,
    Exhaustive { points: u64 },
}
#[derive(Clone, Copy, Debug)]
pub(super) enum WriteOwnershipProof {
    CoordinatePermutation,
    Exhaustive { points: u64 },
    PartitionMember { joint: JointPartitionProof },
}
#[derive(Clone, Copy, Debug)]
pub(super) enum JointPartitionProof {
    Interval,
    Exhaustive { points: u64 },
}
#[derive(Clone, Debug)]
pub(super) struct VerifiedAccessData {
    pub tensor: u32,
    pub mode: AccessMode,
    pub domain: Vec<u32>,
    pub coordinates: Vec<u32>,
    pub bounds_proof: Option<BoundsProof>,
    /// Which facts this access's bounds obligation was allowed to read.
    ///
    /// Held beside the proof kind rather than inside it because the two are
    /// independent axes — see [`IndexDomainFactSource`] — and stored even when
    /// `bounds_proof` is `None`, so that pairing them is one place's job rather
    /// than a rule two optional fields would have to keep between them.
    pub bounds_facts: IndexDomainFactSource,
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
    /// The environment every symbolic extent in this region resolves against.
    ///
    /// `None` for a region built without one. Retained rather than consulted
    /// once and dropped, because a consumer that reads a symbolic extent needs
    /// the same environment the verifier used, and because the region's
    /// identity names that environment's identity.
    pub sources: Option<ExtentSources>,
    pub dimensions: Vec<DimensionData>,
    pub tensors: Vec<TensorData>,
    pub expressions: Vec<IndexExprData>,
    pub accesses: Vec<VerifiedAccessData>,
    pub index_domain_evidence: Vec<DischargedIndexDomainPredicate>,
    pub unknown_index_domain_predicates: Vec<UnknownIndexDomainPredicate>,
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
    /// Returns the environment this region's symbolic extents resolve against.
    ///
    /// `None` for a region built without one, which is every region whose
    /// extents and divisors are all literals: an environment nothing resolves
    /// against would still enter this region's identity, so it is absent rather
    /// than empty.
    #[must_use]
    pub fn extent_sources(&self) -> Option<&ExtentSources> {
        self.data.sources.as_ref()
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
    /// Returns every discharged index-domain predicate in canonical order.
    ///
    /// Each record binds an exact predicate to the access whose iteration
    /// domain it was proved over. Unknown obligations are absent because an
    /// unknown is not evidence and cannot discharge a predicate.
    #[must_use]
    pub fn discharged_index_domain_predicates(
        &self,
    ) -> impl ExactSizeIterator<Item = DischargedIndexDomainPredicate> + '_ {
        self.data.index_domain_evidence.iter().copied()
    }
    /// Looks up retained evidence for one exact access/predicate pair.
    ///
    /// # Errors
    ///
    /// Returns an error when the subject or any predicate handle belongs to a
    /// different region, names no retained entity, or names a tensor axis that
    /// does not exist.
    pub fn index_domain_evidence(
        &self,
        subject: VerifiedTensorAccessId,
        predicate: IndexDomainPredicate,
    ) -> Result<Option<DischargedIndexDomainPredicate>, VerifiedIndexHandleError> {
        self.validate_index_domain_predicate(subject, predicate)?;
        Ok(self
            .data
            .index_domain_evidence
            .iter()
            .copied()
            .find(|record| record.subject == subject && record.predicate == predicate))
    }
    /// Returns every unresolved index-domain predicate in canonical order.
    ///
    /// These are semantic obligations, not physical guards. A consumer must
    /// discharge every record before program work.
    #[must_use]
    pub fn unknown_index_domain_predicates(
        &self,
    ) -> impl ExactSizeIterator<Item = UnknownIndexDomainPredicate> + '_ {
        self.data.unknown_index_domain_predicates.iter().copied()
    }
    /// Looks up one exact unresolved access/predicate pair.
    ///
    /// # Errors
    ///
    /// Returns an error when the subject or any predicate handle belongs to a
    /// different region, names no retained entity, or names a tensor axis that
    /// does not exist.
    pub fn index_domain_unknown(
        &self,
        subject: VerifiedTensorAccessId,
        predicate: IndexDomainPredicate,
    ) -> Result<Option<UnknownIndexDomainPredicate>, VerifiedIndexHandleError> {
        self.validate_index_domain_predicate(subject, predicate)?;
        Ok(self
            .data
            .unknown_index_domain_predicates
            .iter()
            .copied()
            .find(|record| record.subject == subject && record.predicate == predicate))
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
    fn validate_index_domain_predicate(
        &self,
        subject: VerifiedTensorAccessId,
        predicate: IndexDomainPredicate,
    ) -> Result<(), VerifiedIndexHandleError> {
        let access = self.access(subject)?;
        let expression = match predicate {
            IndexDomainPredicate::NonNegative { expression }
            | IndexDomainPredicate::LessThanExtent { expression, .. } => expression,
        };
        self.index_expression(expression)?;
        let expression_index =
            u32::try_from(expression.as_usize()).expect("verified handles fit u32");
        if !access.data.coordinates.contains(&expression_index) {
            return Err(VerifiedIndexHandleError::InvalidHandle {
                entity: IndexEntityKind::IndexExpression,
            });
        }
        if let IndexDomainPredicate::LessThanExtent { extent, .. } = predicate {
            match extent {
                IndexExtentRef::Dimension(dimension) => {
                    self.dimension(dimension)?;
                    let dimension_index =
                        u32::try_from(dimension.as_usize()).expect("verified handles fit u32");
                    if !access.data.domain.contains(&dimension_index) {
                        return Err(VerifiedIndexHandleError::InvalidHandle {
                            entity: IndexEntityKind::Dimension,
                        });
                    }
                }
                IndexExtentRef::TensorAxis { tensor, axis } => {
                    let tensor = self.tensor(tensor)?;
                    if tensor.id.as_usize() != access.data.tensor as usize
                        || usize::try_from(axis).ok().is_none_or(|axis| {
                            axis >= tensor.data.shape.rank()
                                || access.data.coordinates.get(axis) != Some(&expression_index)
                        })
                    {
                        return Err(VerifiedIndexHandleError::InvalidHandle {
                            entity: IndexEntityKind::Tensor,
                        });
                    }
                }
            }
        }
        Ok(())
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
impl<'a> DomainDimensionRef<'a> {
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
    /// Returns the half-open extent together with where its value comes from.
    ///
    /// **One total view rather than a pair of optional accessors.** An earlier
    /// draft answered this question twice — a `static_extent()` returning
    /// `Option<Extent>` beside a symbol accessor returning
    /// `Option<&ShapeSymbol>` — with the rule that exactly one of them is
    /// `Some` held only by a test. That rule is unenforceable: a third source
    /// kind would make both `None` for a real dimension, and every consumer
    /// that had encoded "not static, therefore symbolic" would be silently
    /// wrong. [`SourcedExtent`] is total by construction, so a consumer matches
    /// it exhaustively and a new kind is a build error at every site.
    ///
    /// A consumer that only handles literals calls
    /// [`SourcedExtent::as_static`] and refuses the rest with its own typed
    /// reason. Resolve a symbol through
    /// [`VerifiedIndexRegion::extent_sources`], which is the one environment
    /// this region's symbols are declared in.
    #[must_use]
    pub const fn extent(self) -> &'a SourcedExtent {
        &self.data.extent
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
    /// Returns the boundary's extents together with where each one comes from.
    ///
    /// The boundary counterpart of [`DomainDimensionRef::extent`], and one total
    /// view for the same reason: [`SourcedShape::as_static`] answers the literal
    /// case from the boundary itself, so no caller has to hold the rule that a
    /// non-static boundary is therefore symbolic. A wholly literal boundary
    /// normalizes to [`SourcedShape::Static`] whichever constructor authored it,
    /// so `shape().as_static()` is a fact about the boundary rather than about
    /// the call that made it.
    #[must_use]
    pub const fn shape(self) -> &'a SourcedShape {
        &self.data.shape
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
    /// Normalized affine or semi-affine sum.
    LinearCombination {
        /// Additive constant, always exact.
        ///
        /// A caller may *write* a symbolic addend —
        /// [`IndexRegionBuilder::sourced_linear_combination`](super::IndexRegionBuilder::sourced_linear_combination)
        /// takes a [`SourcedIndexInteger`] here — but normalization carries it into the
        /// term list as `symbol * 1` rather than storing it, so this slot keeps
        /// one meaning. That is what preserves folding: a literal constant
        /// reached through any operand still accumulates here, so `S + 2*3` and
        /// `S + 6*1` remain one region, where a slot that could hold either a
        /// symbol or an integer would have had nowhere to fold them and would
        /// have given one program two identities.
        constant: &'a IndexInteger,
        /// Ordered terms, combined only where exact arithmetic could combine
        /// them — see [`LinearTermRef::coefficient`].
        terms: LinearTerms<'a>,
    },
    /// Euclidean floor division by a proven-positive extent.
    FloorDiv {
        /// Dividend expression.
        dividend: VerifiedIndexExprId,
        /// The divisor, and where its value comes from.
        ///
        /// One vocabulary for both cases rather than a static and a symbolic
        /// variant: an affine-only consumer calls
        /// [`SourcedExtent::as_static`] once and refuses a `None` with its own
        /// typed reason, which is cheaper than matching two variants and cannot
        /// forget one of them. Positivity was proved when the expression was
        /// authored, so no consumer re-decides it.
        divisor: &'a SourcedExtent,
    },
    /// Euclidean modulo by a proven-positive extent.
    Modulo {
        /// Dividend expression.
        dividend: VerifiedIndexExprId,
        /// The divisor, and where its value comes from. See
        /// [`Self::FloorDiv`].
        divisor: &'a SourcedExtent,
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
    /// Returns the coefficient, and where its value comes from.
    ///
    /// **Draft surface, not yet accepted.** This return type widened from
    /// `&IndexInteger` to carry a declared symbol; the widening is a concrete
    /// draft pending Tom's acceptance, and [`SourcedIndexInteger`] carries the
    /// full label.
    ///
    /// # What normalization did and did not combine
    ///
    /// A term whose coefficient is [`SourcedIndexInteger::Literal`] was folded
    /// by exact arithmetic: it is nonzero, it names an operand no other literal
    /// term names, and no nested sum survives under it.
    ///
    /// A term whose coefficient is [`SourcedIndexInteger::Symbol`] was retained
    /// verbatim — never merged with another term, dropped, distributed over a
    /// nested sum, or unwrapped — because none of those rewrites is available
    /// without a value the environment need not pin, and performing them
    /// *when* it happens to pin one would make canonicalization a function of
    /// the environment rather than of the program. Two symbolic terms over one
    /// operand therefore both appear, and `S * x` is a term even when the
    /// environment fixes `S == 1`.
    #[must_use]
    pub const fn coefficient(self) -> &'a SourcedIndexInteger {
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
                divisor,
            },
            IndexNode::Modulo { dividend, divisor } => IndexExprView::Modulo {
                dividend: VerifiedIndexExprId::from_verified(self.id.owner, *dividend),
                divisor,
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
    ///
    /// Each form carries the facts it rested on, so a caller reads *how* the
    /// access was proved and *what the proof was allowed to consult* from one
    /// value. A second optional accessor beside this one would have made their
    /// complementarity a rule only a test could hold, which is the defect the
    /// total extent and boundary views already replaced.
    #[must_use]
    pub fn bounds_proof(self) -> Option<BoundsProofView> {
        let facts = self.data.bounds_facts;
        self.data.bounds_proof.map(|proof| match proof {
            BoundsProof::VacuousEmptyDomain => BoundsProofView::VacuousEmptyDomain { facts },
            BoundsProof::Interval => BoundsProofView::Interval { facts },
            BoundsProof::ProvedExtentEquality => BoundsProofView::ProvedExtentEquality { facts },
            BoundsProof::Exhaustive { points } => BoundsProofView::Exhaustive { points, facts },
        })
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
            WriteOwnershipProof::PartitionMember { joint } => {
                WriteOwnershipProofView::PartitionMember {
                    joint: match joint {
                        JointPartitionProof::Interval => JointPartitionProofView::Interval,
                        JointPartitionProof::Exhaustive { points } => {
                            JointPartitionProofView::Exhaustive { points }
                        }
                    },
                }
            }
        })
    }
}

/// Public view of one sound bounds proof.
///
/// Every form carries an [`IndexDomainFactSource`], because the argument and
/// the premises it read are independent: each of the four below can run over a
/// wholly literal region or over one whose extents, divisors, and coefficients
/// are declared symbols. The field is repeated on each variant rather than
/// hoisted beside the enum so that a new form has to decide the question rather
/// than inherit an answer, and [`Self::facts`] reads it without a match.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BoundsProofView {
    /// The iteration domain is empty, so bounds hold vacuously.
    VacuousEmptyDomain {
        /// Facts the emptiness rested on.
        facts: IndexDomainFactSource,
    },
    /// Exact interval propagation proved every coordinate in bounds.
    Interval {
        /// Facts the propagated intervals rested on.
        facts: IndexDomainFactSource,
    },
    /// Every coordinate *is* a domain dimension whose extent the environment
    /// proves equal to the axis it indexes.
    ///
    /// A structural argument rather than a numeric one, and it is what a
    /// caller-sized program rests on: a coordinate that is `Dimension(d)`
    /// ranges over `[0, extent(d))` by construction, so when the environment
    /// proves `extent(d)` and the axis are one extent, the coordinate is in
    /// bounds in *every* model — with no bound known on either. Interval
    /// propagation cannot express this, because a wholly undetermined symbol's
    /// interval is the entire extent domain and no comparison against it
    /// closes.
    ProvedExtentEquality {
        /// Facts the proved equality rested on.
        facts: IndexDomainFactSource,
    },
    /// Finite enumeration proved every coordinate in bounds.
    Exhaustive {
        /// Enumerated domain points.
        points: u64,
        /// Facts the walked extents, divisors, and coefficients rested on.
        facts: IndexDomainFactSource,
    },
}

impl BoundsProofView {
    /// Returns which facts this proof rested on.
    ///
    /// **Draft surface, not yet accepted**; [`IndexDomainFactSource`] carries
    /// the full label.
    #[must_use]
    pub const fn facts(self) -> IndexDomainFactSource {
        match self {
            Self::VacuousEmptyDomain { facts }
            | Self::Interval { facts }
            | Self::ProvedExtentEquality { facts }
            | Self::Exhaustive { facts, .. } => facts,
        }
    }
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
    /// This root is total and injective over its own partition of an output
    /// that several roots jointly own.
    ///
    /// A partition member proves strictly less on its own than the two forms
    /// above do: it covers its declared partition rather than the whole
    /// boundary. What makes the *output* owned is the joint obligation across
    /// the root set — pairwise disjoint partitions whose union is the boundary
    /// exactly — so the mechanism that discharged it is carried here rather
    /// than left for a consumer to assume. A root carrying this form and no
    /// sibling is unrepresentable: the verifier records it only for a boundary
    /// whose roots it decided together.
    PartitionMember {
        /// How the joint obligation across this output's roots was discharged.
        joint: JointPartitionProofView,
    },
}

/// Public view of the mechanism that discharged one output's joint partition
/// obligation.
///
/// Recorded rather than derived, because the two mechanisms decide different
/// populations and a consumer that must re-derive the obligation needs to know
/// which one answered. [`Self::Interval`] closes over the ranges themselves and
/// says nothing about any individual element; [`Self::Exhaustive`] visited every
/// element of the boundary and is available only where every extent is
/// determined.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum JointPartitionProofView {
    /// Interval reasoning over contiguous coordinate ranges decided the set.
    ///
    /// Each root's partition is a rectangle of static ranges, pairwise
    /// disjointness was decided by finding a separating axis for every pair,
    /// and coverage followed from the disjoint volumes summing to the
    /// boundary's element count. Nothing was enumerated.
    Interval,
    /// Finite enumeration over the boundary decided the set.
    ///
    /// One shared bitset across every root: a second write to one element is a
    /// refusal and an element no root reached is a refusal.
    Exhaustive {
        /// Domain points enumerated across every root of this output.
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
