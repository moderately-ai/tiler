//! Construction of a verified index region, in lifecycle order.
//!
//! A caller declares tensors and dimensions, builds index expressions over
//! them, records accesses, applies scalar operations, names outputs, and calls
//! `build`. This root owns that path — the public constructors, the draft state
//! they accumulate, and the handle resolvers — so the lifecycle can be read
//! without opening anything else.
//!
//! # What happens inside `build`, and where each part lives
//!
//! The phases below run in this order and each owns one invariant. They are
//! private children of this module rather than separate concepts, so
//! `tiler_ir::index::builder` stays the single disclosure point and no public
//! path moves.
//!
//! 1. `proof` — the obligations a draft must discharge: reachability, access
//!    bounds, write permutation, and the exhaustive enumeration fallback. It
//!    **collects every diagnostic rather than returning the first**, because a
//!    caller fixing one unproved access should not have to rebuild to find the
//!    next.
//! 2. `compact` — dropping unreachable drafts and renumbering what survives.
//!    The invariant is alpha-equivalence: two regions differing only in the
//!    order their dimensions and expressions were authored must compact to the
//!    same shape, so every ordering here derives from a content key and never
//!    from draft position.
//! 3. `identity` — the canonical encoding of the compacted region. `encode_region`
//!    and `encoded_region_len` are one pair and the encoder asserts they agree,
//!    so a field added to one without the other fails there instead of silently
//!    moving an identity.
//!
//! `reduction` sits beside them rather than in the sequence: a reducer body is
//! canonicalized independently of the region containing it, so two regions whose
//! bodies differ only by draft ordering compact to the same bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use std::sync::Arc;

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Signed, ToPrimitive, Zero};

use crate::convenience::{CheckedBuildError, build_checked};
use crate::identity::{push_len, push_slice};
use crate::semantic::ResolvedValueType;
use crate::shape::{Extent, Shape};

use super::error::invalid_handle;
use super::handles::{BuilderId, next_builder_id};
use super::model::{
    AccessData, BoundsProof, DimensionData, IndexExprData, IndexNode, JointPartitionProof,
    LinearTermData, OutputData, ReducerBodyOperationData, ReducerBodyValueData,
    ReducerBodyValueSource, ScalarOperationData, ScalarOperationKindData, ScalarReducerBodyData,
    ScalarValueData, ScalarValueDefinition, TensorData, VerifiedAccessData,
    VerifiedIndexRegionData, WriteOwnershipProof,
};
use super::scalar::{
    ScalarApplyError, ScalarInferenceCapacity, ScalarInferenceHostFailure, encode_canonical,
    encode_key,
};
use super::sourced::{
    ExtentSourceError, ExtentSources, SourcedExtent, SourcedIndexInteger, SourcedShape,
    SymbolicExtentError,
};
use super::{
    AccessMode, CanonicalIndexRegionIdentity, DimensionId, DischargedIndexDomainPredicate,
    DomainRole, FrozenScalarRegistry, IndexBuildError, IndexDomainEvidence, IndexDomainPredicate,
    IndexDomainSoundProof, IndexDomainUnknownReason, IndexEntityKind, IndexExprClass, IndexExprId,
    IndexExtentRef, IndexInteger, IndexLimitKind, IndexRegionBuildError, IndexRegionDiagnostic,
    MAX_ACCESS_CANONICAL_BYTES, MAX_BOUNDARY_CANONICAL_BYTES, MAX_BOUNDARY_TENSORS,
    MAX_DOMAIN_DIMENSIONS, MAX_EXHAUSTIVE_PROOF_BYTES, MAX_EXHAUSTIVE_PROOF_CELLS,
    MAX_INDEX_CANONICAL_BYTES, MAX_INDEX_EXPRESSION_DEPTH, MAX_INDEX_EXPRESSION_OPERANDS,
    MAX_INDEX_EXPRESSIONS, MAX_INDEX_INTEGER_BYTES, MAX_INDEX_REGION_IDENTITY_BYTES,
    MAX_OUTPUT_ROOTS, MAX_SCALAR_CANONICAL_BYTES, MAX_SCALAR_EXPRESSION_DEPTH,
    MAX_SCALAR_EXPRESSIONS, MAX_SCALAR_OPERANDS, MAX_TENSOR_ACCESSES, MAX_TENSOR_RANK,
    ProofResource, ReductionTraversal, ScalarAttributes, ScalarOpKey, ScalarOperationId,
    ScalarResultIndex, ScalarValueId, TensorAccessId, TensorId, TensorRole,
    UnknownIndexDomainPredicate, VerifiedIndexExprId, VerifiedIndexRegion, VerifiedTensorAccessId,
    VerifiedTensorId,
};
use crate::shape::{ExtentInterval, ShapeEnv, ShapeSymbol};

/// The domain separator of one verified index region's canonical identity.
///
/// `v10` rather than `v9`: a linear combination's coefficient now encodes as a
/// tagged [`SourcedIndexInteger`] where `v9` wrote a bare sign-and-magnitude
/// integer, so the bytes of an *exact* coefficient changed even though its
/// meaning did not, and every region carrying a linear combination moves. That
/// is the same step the divisor took at `v9`, stated rather than left to two
/// encodings sharing one domain. The additive constant is unchanged and did not
/// move it — normalization keeps that slot an exact integer — and neither did
/// admitting the symbolic form, which adds regions rather than re-encoding
/// existing ones.
const INDEX_REGION_DOMAIN: &[u8] = b"tiler.index-region.v10\0";

/// What interval propagation concluded about one access's coordinates.
#[derive(Clone, Copy, Debug)]
struct IntervalVerdict {
    /// Every coordinate provably lies inside its axis.
    interval_proved: bool,
    /// Some coordinate provably lies outside its axis for every domain point.
    definitely_outside: bool,
}

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
    index_domain_evidence: Vec<DischargedIndexDomainPredicate>,
    unknown_index_domain_predicates: Vec<UnknownIndexDomainPredicate>,
    operations: Vec<ScalarOperationData>,
    values: Vec<ScalarValueData>,
    outputs: Vec<OutputData>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingIndexDomainBound {
    NonNegative,
    LessThanAxis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingIndexDomainDisposition {
    Discharged(IndexDomainEvidence),
    Unknown(IndexDomainUnknownReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PendingIndexDomainPredicate {
    access: u32,
    axis: u32,
    bound: PendingIndexDomainBound,
    disposition: PendingIndexDomainDisposition,
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
            push_slice(&mut key, value_type.canonical_encoding().as_bytes());
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
            push_slice(&mut key, value_type.canonical_encoding().as_bytes());
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
    /// The one environment every symbolic extent in this region resolves against.
    ///
    /// One per region rather than one per extent: a region whose extents were
    /// resolved against two environments would have an identity naming neither,
    /// and a consumer would have no way to bind its symbols.
    sources: Option<ExtentSources>,
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
    /// Every extent this builder admits is a literal. Use
    /// [`Self::new_with_shape_environment`] to author a region whose extents or
    /// divisors may name declared symbols; a static caller needs no environment
    /// and is not asked for an absent one.
    ///
    /// # Errors
    ///
    /// Returns an error when no fresh builder ownership identity remains.
    pub fn new(registry: FrozenScalarRegistry) -> Result<Self, IndexBuildError> {
        Self::open(registry, None)
    }

    /// Creates a builder whose symbolic extents resolve in one verified
    /// environment.
    ///
    /// **A constructor rather than a setter, and there is no setter.** A region
    /// has exactly one environment for the whole of its life: a second one
    /// would silently reinterpret every extent already authored against the
    /// first, and the region's identity — which folds the environment's
    /// identity — would name whichever happened to be installed last. Fixing
    /// the environment before any dimension, boundary, or divisor exists makes
    /// that replacement unrepresentable rather than merely discouraged.
    ///
    /// # Errors
    ///
    /// Returns an error when no fresh builder ownership identity remains.
    pub fn new_with_shape_environment(
        registry: FrozenScalarRegistry,
        environment: Arc<ShapeEnv>,
    ) -> Result<Self, IndexBuildError> {
        Self::open(registry, Some(ExtentSources::new(environment)))
    }

    fn open(
        registry: FrozenScalarRegistry,
        sources: Option<ExtentSources>,
    ) -> Result<Self, IndexBuildError> {
        let owner = next_builder_id().ok_or(IndexBuildError::BuilderIdentityExhausted)?;
        Ok(Self {
            owner,
            registry,
            sources,
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
        self.push_tensor(role, value_type, SourcedShape::from_shape(shape))
    }

    /// Declares one tensor boundary whose extents may name `ShapeEnv` symbols.
    ///
    /// The counterpart of [`Self::symbolic_dimension`] on the boundary side,
    /// and what makes a *dynamically shaped output* expressible: an output
    /// extent that names the same symbol the iteration domain does states that
    /// the two are one size, which is the fact a write-ownership argument then
    /// discharges through the environment rather than against a literal.
    ///
    /// Every symbolic extent is admitted against this region's one environment
    /// under the same ceiling a domain extent obeys — see
    /// [`EXTENT_PHASE_CEILING`](super::EXTENT_PHASE_CEILING), where the boundary case is the quoted one.
    ///
    /// # Errors
    ///
    /// Returns [`SymbolicExtentError::Source`] when no environment is bound,
    /// when the environment does not declare a named symbol, or when a symbol's
    /// root binding arrives later than a boundary extent may be sourced from;
    /// [`SymbolicExtentError::ShapeVocabulary`] when the shape vocabulary cannot
    /// represent the normalized boundary; and
    /// [`SymbolicExtentError::Structural`] for an unknown type or exceeded
    /// structural limit.
    pub fn sourced_tensor(
        &mut self,
        role: TensorRole,
        value_type: ResolvedValueType,
        extents: Vec<SourcedExtent>,
    ) -> Result<TensorId, SymbolicExtentError> {
        // Admitted before the boundary exists, so a refused source leaves the
        // draft exactly as it was rather than half-applied — and admitted for
        // every extent before any of them is retained, so a boundary is never
        // partly sourced from a refused symbol.
        for extent in &extents {
            let Some(symbol) = extent.symbol() else {
                continue;
            };
            let Some(sources) = self.sources.as_ref() else {
                // No environment can declare the symbol, so it is undeclared
                // here for exactly the reason the variant names.
                return Err(ExtentSourceError::UndeclaredSymbol {
                    symbol: symbol.clone(),
                }
                .into());
            };
            sources.admit(extent)?;
        }
        let shape = SourcedShape::sourced(extents)?;
        Ok(self.push_tensor(role, value_type, shape)?)
    }

    fn push_tensor(
        &mut self,
        role: TensorRole,
        value_type: ResolvedValueType,
        shape: SourcedShape,
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
            .saturating_add(shape.encoded_len())
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
        self.push_dimension(role, SourcedExtent::Static(extent))
    }

    /// Adds a half-open dimension whose extent is a declared `ShapeEnv` symbol.
    ///
    /// The symbol is resolved through this region's environment and nowhere
    /// else; there is no index-local declaration, so a symbol that environment
    /// does not declare has no meaning here and is refused.
    ///
    /// # Errors
    ///
    /// Returns [`SymbolicExtentError::Source`] when no environment is bound,
    /// when the environment does not declare the symbol, or when the symbol's
    /// root binding arrives later than an index extent may be sourced from, and
    /// [`SymbolicExtentError::Structural`] when the dimension-count limit is
    /// exceeded.
    pub fn symbolic_dimension(
        &mut self,
        role: DomainRole,
        symbol: ShapeSymbol,
    ) -> Result<DimensionId, SymbolicExtentError> {
        let Some(sources) = self.sources.as_ref() else {
            // No environment can declare the symbol, so it is undeclared here
            // for exactly the reason the variant names.
            return Err(SymbolicExtentError::Source(
                ExtentSourceError::UndeclaredSymbol { symbol },
            ));
        };
        let extent = SourcedExtent::Symbol(symbol);
        // Admitted before the dimension exists, so a refused source leaves the
        // draft exactly as it was rather than half-applied.
        sources
            .admit(&extent)
            .map_err(SymbolicExtentError::Source)?;
        self.push_dimension(role, extent)
            .map_err(SymbolicExtentError::Structural)
    }

    fn push_dimension(
        &mut self,
        role: DomainRole,
        extent: SourcedExtent,
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
        self.dimensions.push(DimensionData { role, extent });
        Ok(id)
    }

    /// Returns the closed interval every model confines one sourced extent to.
    ///
    /// The single place this builder resolves an extent against its
    /// environment. Every stronger question below — is it determined, is it
    /// nonempty, does it bound a coordinate — is asked through this one, so a
    /// rule cannot be extended for the domain side and forgotten for the
    /// boundary side.
    ///
    /// `None` means the environment says nothing this builder can prove
    /// against. That is not zero and not "very many", and substituting either
    /// would be the silent approximation the contract forbids.
    fn extent_interval(&self, extent: &SourcedExtent) -> Option<ExtentInterval> {
        match (extent.as_static(), self.sources.as_ref()) {
            (Some(value), _) => Some(ExtentInterval {
                lower: value.get(),
                upper: value.get(),
            }),
            // A symbolic extent with no environment is unresolvable rather than
            // unconstrained. No constructor can produce one, so this is a
            // fail-closed floor rather than a reachable path.
            (None, None) => None,
            (None, Some(sources)) => sources.interval(extent),
        }
    }

    /// Returns the one value this region's environment fixes for an extent.
    ///
    /// A one-point interval leaves one admissible value, which is sound because
    /// the interval contains every model.
    fn determined(&self, extent: &SourcedExtent) -> Option<u64> {
        self.extent_interval(extent)
            .filter(|interval| interval.lower == interval.upper)
            .map(|interval| interval.lower)
    }

    /// Returns whether the environment proves two extents are one extent.
    ///
    /// The symbolic form of a literal `==`. A wholly static region has no
    /// environment to ask, and there only two literals are comparable.
    fn extents_proved_equal(&self, left: &SourcedExtent, right: &SourcedExtent) -> bool {
        match self.sources.as_ref() {
            Some(sources) => sources.proves_equal(left, right),
            None => match (left.as_static(), right.as_static()) {
                (Some(left), Some(right)) => left == right,
                _ => false,
            },
        }
    }

    /// Returns the one extent this region's environment fixes for `dimension`.
    ///
    /// `None` means the environment admits more than one value, which is a
    /// different answer from zero and must never be substituted for one: it is
    /// what makes an exhaustive enumeration or a permutation argument
    /// unavailable rather than false.
    fn determined_extent(&self, dimension: u32) -> Option<u64> {
        self.determined(&self.dimensions[dimension as usize].extent)
    }

    /// Returns the largest extent any binding admits for `dimension`.
    ///
    /// Sound to bound a coordinate against: the interval contains every model,
    /// so a coordinate below it is below every admissible extent.
    fn extent_upper_bound(&self, dimension: u32) -> Option<u64> {
        self.extent_interval(&self.dimensions[dimension as usize].extent)
            .map(|interval| interval.upper)
    }

    /// Returns whether every dimension of `domain` is proved to have a point.
    ///
    /// A conclusion that a coordinate is *definitely* out of bounds is only
    /// sound over a domain that is visited at all, and a symbolic extent whose
    /// lower bound is zero may not be.
    fn domain_is_nonempty(&self, domain: &[u32]) -> bool {
        domain.iter().all(|dimension| {
            self.extent_interval(&self.dimensions[*dimension as usize].extent)
                .is_some_and(|interval| interval.lower >= 1)
        })
    }

    /// Returns each boundary axis's exact extent, when all of them are known.
    ///
    /// The boundary counterpart of [`Self::domain_extents`], and `None` for the
    /// same reason: an enumeration walks a boundary axis by axis, and one
    /// undetermined axis means there is no enumeration rather than a shorter
    /// one. Kept apart from an element count that does not fit this host, which
    /// is a different failure with a different remedy.
    fn boundary_extents(&self, shape: &SourcedShape) -> Option<Vec<u64>> {
        shape
            .extents()
            .map(|extent| self.determined(&extent))
            .collect()
    }

    /// Returns the dense element count of a boundary whose extents are known.
    ///
    /// Mirrors [`Shape::element_count`] exactly, including that one zero extent
    /// makes the count zero even when another extent does not fit this host.
    ///
    /// `None` covers two different failures — an undetermined axis and a count
    /// too large for this host — because every caller of *this* answer alone
    /// fails closed on both. A caller that must tell them apart asks
    /// [`Self::boundary_extents`] first, and the verifier does exactly that
    /// before it budgets or enumerates anything.
    fn boundary_element_count(&self, shape: &SourcedShape) -> Option<usize> {
        let extents = self.boundary_extents(shape)?;
        if extents.contains(&0) {
            return Some(0);
        }
        extents.iter().try_fold(1_usize, |count, extent| {
            count.checked_mul(usize::try_from(*extent).ok()?)
        })
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
        let _ = data;
        // `0 <= d < extent`, so the largest extent the environment admits is
        // the largest coordinate plus one. An extent with no admitted upper
        // bound leaves the interval unknown rather than guessed, which is what
        // later makes the access fall to a proof it cannot complete.
        let interval = self
            .extent_upper_bound(dimension.index)
            .filter(|upper| *upper != 0)
            .map(|upper| (BigInt::zero(), BigInt::from(upper - 1)));
        self.intern_index(
            IndexNode::Dimension(dimension.index),
            dimensions,
            IndexExprClass::Affine,
            interval,
            0,
        )
    }
    /// Creates a normalized affine linear combination over exact integers.
    ///
    /// The literal entry point, beside
    /// [`Self::sourced_linear_combination`] exactly as [`Self::tensor`] is
    /// beside [`Self::sourced_tensor`] and [`Self::dimension`] beside
    /// [`Self::symbolic_dimension`]. Both author the *same* node and are
    /// inspected through one total view; only the vocabulary a caller spells
    /// its scalars in differs, and a caller with no symbols never names the
    /// sourced type.
    ///
    /// # Errors
    ///
    /// Returns an error for foreign operands or exceeded expression limits. No
    /// source can be refused here, because none can be named.
    pub fn linear_combination(
        &mut self,
        constant: IndexInteger,
        terms: &[(IndexInteger, IndexExprId)],
    ) -> Result<IndexExprId, IndexBuildError> {
        let terms: Vec<(SourcedIndexInteger, IndexExprId)> = terms
            .iter()
            .map(|(coefficient, id)| (coefficient.clone().into(), *id))
            .collect();
        // Admission is skipped rather than run and discarded: every scalar here
        // is a literal, and admission is the *only* step that can refuse a
        // source. Returning `IndexBuildError` is therefore a fact about this
        // path's inputs rather than a narrowing that could drop a real refusal.
        self.assemble_linear_combination(constant.into(), &terms)
    }

    /// Creates a normalized linear combination whose scalars may be sourced.
    ///
    /// **Draft surface, not yet accepted.** This constructor, its argument
    /// vocabulary, and the widened view it produces are a concrete draft
    /// pending Tom's acceptance; [`SourcedIndexInteger`] carries the full
    /// label.
    ///
    /// A coefficient or constant that names a declared symbol makes the
    /// expression [`IndexExprClass::SemiAffine`], which is what ADR 0046's
    /// "symbolic coefficients" half admits. The symbol is one declared
    /// [`ShapeSymbol`] and deliberately not an expression tree, for the reason
    /// [`SourcedExtent`] gives about divisors: a composed magnitude such as
    /// `S - T` is a relation in the environment's constraint set, where it can
    /// be decided, rather than arithmetic the index layer re-derives. `i + (S -
    /// T)` is therefore written as `i + U` over a symbol the environment
    /// relates to `S` and `T`.
    ///
    /// # What a symbolic scalar costs
    ///
    /// Normalization declines on it — see
    /// [`LinearTermRef::coefficient`](super::LinearTermRef::coefficient) — and
    /// so does interval propagation, so an access whose coordinate carries one
    /// is proved by another argument or retains its bound as an explicit
    /// obligation under
    /// [`IndexDomainUnknownReason::InsufficientFacts`](super::IndexDomainUnknownReason::InsufficientFacts).
    /// That is ADR 0046's permission to "conservatively decline semi-affine
    /// maps they cannot analyze", taken rather than approximated.
    ///
    /// # Errors
    ///
    /// Returns [`SymbolicExtentError::Source`] when a symbolic scalar is not
    /// declared by this region's environment or arrives after
    /// [`EXTENT_PHASE_CEILING`](super::EXTENT_PHASE_CEILING), and
    /// [`SymbolicExtentError::Structural`] for a foreign operand or an exceeded
    /// limit. A refused scalar leaves the draft exactly as it was.
    ///
    /// Note which predicate is *not* applied: unlike a divisor, a coefficient
    /// is never required to be proved positive. `x floordiv 0` is undefined and
    /// so a divisor's positivity is a condition of the expression having a
    /// meaning at all, whereas `0 * x` and every other admissible magnitude
    /// denote a coordinate. Requiring positivity here would refuse programs
    /// ADR 0046 admits.
    pub fn sourced_linear_combination(
        &mut self,
        constant: SourcedIndexInteger,
        terms: &[(SourcedIndexInteger, IndexExprId)],
    ) -> Result<IndexExprId, SymbolicExtentError> {
        // Every source is admitted before anything is resolved, retained, or
        // interned, so a refused coefficient leaves the draft exactly as it was
        // rather than half-applied.
        self.admit_index_scalar(&constant)?;
        for (coefficient, _) in terms {
            self.admit_index_scalar(coefficient)?;
        }
        Ok(self.assemble_linear_combination(constant, terms)?)
    }

    /// Admits one index scalar's source, without deciding its sign.
    ///
    /// Reuses [`ExtentSources::admit`] rather than restating the ceiling or the
    /// declaration check, so a coefficient, a domain extent, a boundary axis,
    /// and a divisor are all refused by one authority for one set of reasons.
    /// [`ExtentSources::proves_positive`] is deliberately not consulted: see
    /// [`Self::sourced_linear_combination`].
    fn admit_index_scalar(&self, value: &SourcedIndexInteger) -> Result<(), SymbolicExtentError> {
        let Some(symbol) = value.symbol() else {
            return Ok(());
        };
        let Some(sources) = self.sources.as_ref() else {
            // No environment can declare the symbol, so it is undeclared here
            // for exactly the reason the variant names.
            return Err(ExtentSourceError::UndeclaredSymbol {
                symbol: symbol.clone(),
            }
            .into());
        };
        sources.admit(&SourcedExtent::Symbol(symbol.clone()))?;
        Ok(())
    }

    /// Normalizes and interns one linear combination whose sources are admitted.
    ///
    /// Split from [`Self::sourced_linear_combination`] so that admission is the
    /// only step able to refuse a source: everything below can fail only under
    /// the index layer's own structural authority, which is what lets
    /// [`Self::linear_combination`] return [`IndexBuildError`] without
    /// discarding a refusal it could have reported.
    fn assemble_linear_combination(
        &mut self,
        constant: SourcedIndexInteger,
        terms: &[(SourcedIndexInteger, IndexExprId)],
    ) -> Result<IndexExprId, IndexBuildError> {
        limit(
            terms.len(),
            MAX_INDEX_EXPRESSION_OPERANDS,
            IndexLimitKind::IndexExpressionOperands,
        )?;
        if let Some(constant) = constant.as_literal() {
            check_integer(&constant.0)?;
        }
        for (coefficient, _) in terms {
            if let Some(coefficient) = coefficient.as_literal() {
                check_integer(&coefficient.0)?;
            }
        }

        // Two accumulators, because the two coefficient kinds admit different
        // rewrites.
        //
        // A literal term folds by exact arithmetic into `coefficients`, keyed
        // by the operand it scales: it multiplies into a constant operand,
        // distributes over a nested sum, merges with another term over the same
        // operand, and disappears at zero.
        //
        // **A symbolic term declines every one of those, deliberately.** None
        // is available without a value the environment need not pin, and
        // performing them *when* it happens to pin one would make
        // canonicalization a function of the environment rather than of the
        // program — collapsing `graph identity` into `specialized identity`,
        // which the sourced boundary exists to keep apart. So `S * x` is
        // retained verbatim even when the environment fixes `S == 0` or `S ==
        // 1`, two symbolic terms over one operand both survive, and a nested
        // sum under a symbolic coefficient is not distributed. The cost is
        // paid in analysis, not in soundness: interval propagation declines on
        // the same terms, and the bound is retained as an explicit obligation.
        let mut normalized_constant = BigInt::zero();
        let mut coefficients: BTreeMap<Arc<Vec<u8>>, (u32, BigInt)> = BTreeMap::new();
        let mut sourced: Vec<LinearTermData> = Vec::new();
        // A symbolic addend is carried as the term `symbol * 1` rather than
        // stored in the constant slot. Keeping that slot exact is what lets a
        // literal constant reached through any operand still fold into it, so
        // `S + 2*3` and `S + 6*1` stay one region; a slot holding either kind
        // would have had nowhere to fold them.
        match constant {
            SourcedIndexInteger::Literal(value) => normalized_constant = value.0,
            SourcedIndexInteger::Symbol(symbol) => {
                let one = self.constant(IndexInteger::from_i128(1))?;
                sourced.push(LinearTermData {
                    coefficient: SourcedIndexInteger::Symbol(symbol),
                    value: one.index,
                });
            }
        }
        for (coefficient, id) in terms {
            self.resolve_expr(*id)?;
            match coefficient {
                SourcedIndexInteger::Literal(value) => accumulate_linear_term(
                    &mut normalized_constant,
                    &mut coefficients,
                    &value.0,
                    id.index,
                    &self.expressions,
                )?,
                SourcedIndexInteger::Symbol(_) => sourced.push(LinearTermData {
                    coefficient: coefficient.clone(),
                    value: id.index,
                }),
            }
        }
        let mut terms: Vec<_> = coefficients
            .into_iter()
            .filter(|(_, (_, coefficient))| !coefficient.is_zero())
            .map(|(_, (value, coefficient))| LinearTermData {
                coefficient: SourcedIndexInteger::Literal(IndexInteger(coefficient)),
                value,
            })
            .chain(sourced)
            .collect();
        limit(
            terms.len(),
            MAX_INDEX_EXPRESSION_OPERANDS,
            IndexLimitKind::IndexExpressionOperands,
        )?;
        if terms.is_empty() {
            return self.constant(IndexInteger(normalized_constant));
        }
        // Ordered by operand *and then* by coefficient. The operand alone was a
        // total key while every term folded, because folding left one term per
        // operand; a declined symbolic term breaks that, so two terms can now
        // scale one operand and the coefficient's own canonical bytes are what
        // keeps the order total and reproducible.
        terms.sort_by_cached_key(|term| {
            let mut key = self.expressions[term.value as usize]
                .structural_key
                .as_ref()
                .clone();
            term.coefficient.encode(&mut key);
            key
        });
        if normalized_constant.is_zero()
            && terms.len() == 1
            && terms[0].coefficient.as_literal().map(|value| &value.0) == Some(&BigInt::from(1_u8))
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
            match term.coefficient.as_literal() {
                Some(coefficient) => check_integer(&coefficient.0)?,
                // A symbol the region names but does not fix participates in
                // the arithmetic, so the form is semi-affine whatever its
                // operands are.
                None => class = class.join(IndexExprClass::SemiAffine),
            }
            let expression = &self.expressions[term.value as usize];
            dimensions.extend(&expression.dimensions);
            class = class.join(expression.class);
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
    /// Creates Euclidean floor division by a proven-positive extent.
    ///
    /// The divisor uses the crate's one constant-or-symbol extent vocabulary, so
    /// a literal divisor is [`SourcedExtent::Static`] and a caller-supplied tile
    /// size is [`SourcedExtent::Symbol`]. A symbolic divisor makes the
    /// expression [`IndexExprClass::SemiAffine`].
    ///
    /// # Errors
    ///
    /// Returns [`SymbolicExtentError::Source`] when a symbolic divisor is not
    /// declared by this region's environment, arrives after
    /// [`EXTENT_PHASE_CEILING`](super::EXTENT_PHASE_CEILING), or is not proved to be at least one, and
    /// [`SymbolicExtentError::Structural`] for a zero literal divisor, a
    /// foreign dividend, or an exceeded limit.
    pub fn floor_div(
        &mut self,
        dividend: IndexExprId,
        divisor: SourcedExtent,
    ) -> Result<IndexExprId, SymbolicExtentError> {
        self.div_mod(dividend, divisor, true)
    }
    /// Creates Euclidean modulo by a proven-positive extent.
    ///
    /// # Errors
    ///
    /// The same refusals as [`Self::floor_div`].
    pub fn modulo(
        &mut self,
        dividend: IndexExprId,
        divisor: SourcedExtent,
    ) -> Result<IndexExprId, SymbolicExtentError> {
        self.div_mod(dividend, divisor, false)
    }
    fn div_mod(
        &mut self,
        dividend: IndexExprId,
        divisor: SourcedExtent,
        div: bool,
    ) -> Result<IndexExprId, SymbolicExtentError> {
        // Admitted before anything is retained, so a refused divisor leaves the
        // draft exactly as it was rather than half-applied.
        let class = self.admit_divisor(&divisor)?;
        let data = self.resolve_expr(dividend)?.clone();
        // Exact arithmetic needs the divisor's *value*, which a symbol has only
        // when the environment pins it. Anything less leaves the range unknown
        // rather than approximated: an interval derived from a divisor nobody
        // fixed would be a bound nothing proved, and a `None` instead makes the
        // access fall through to a proof that either closes another way or is
        // retained as an explicit obligation.
        let interval = match (div, self.determined(&divisor)) {
            (true, Some(value)) => {
                let value = BigInt::from(value);
                data.interval
                    .map(|(low, high)| (low.div_floor(&value), high.div_floor(&value)))
            }
            // `admit_divisor` proved the divisor at least one, so the
            // subtraction cannot wrap.
            (false, Some(value)) => Some((BigInt::zero(), BigInt::from(value - 1))),
            (_, None) => None,
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
        Ok(self.intern_index(node, data.dimensions, class, interval, data.depth + 1)?)
    }

    /// Admits one divisor and returns the class its form implies.
    ///
    /// Positivity is a condition of the expression being *defined* — `x
    /// floordiv 0` has no meaning under any plan — so it is decided here rather
    /// than deferred to a consumer, and for a symbol it comes only from the
    /// environment's semantic input constraints. A variant guard cannot supply
    /// it: a guard's failure selects another plan, and an expression whose
    /// definedness rested on one would be admitted into a region a later plan
    /// choice could render meaningless.
    fn admit_divisor(
        &self,
        divisor: &SourcedExtent,
    ) -> Result<IndexExprClass, SymbolicExtentError> {
        let Some(symbol) = divisor.symbol() else {
            if divisor.as_static() == Some(Extent::new(0)) {
                return Err(IndexBuildError::NonPositiveDivisor.into());
            }
            return Ok(IndexExprClass::QuasiAffine);
        };
        let Some(sources) = self.sources.as_ref() else {
            // No environment can declare the symbol, so it is undeclared here
            // for exactly the reason the variant names.
            return Err(ExtentSourceError::UndeclaredSymbol {
                symbol: symbol.clone(),
            }
            .into());
        };
        sources.admit(divisor)?;
        if !sources.proves_positive(divisor) {
            return Err(ExtentSourceError::DivisorNotProvedPositive {
                symbol: symbol.clone(),
            }
            .into());
        }
        Ok(IndexExprClass::SemiAffine)
    }

    /// Creates or reuses a logical write access.
    ///
    /// `domain` may be **any subset of the region's parallel dimensions**, not
    /// only all of them. Each write root therefore carries its own iteration
    /// space, which is what makes a partition whose members have unequal
    /// extents expressible: two roots over one output declare dimensions of
    /// different extents rather than sharing one, and a zero-extent member is
    /// the degenerate case of that rather than a shape nothing can spell. A
    /// dimension the domain omits is simply not iterated by this root; the
    /// obligation that keeps that coherent is that the stored value may not vary
    /// along it either, which verification refuses under
    /// [`IndexRegionDiagnostic::ValueDimensionOutsideWriteDomain`].
    ///
    /// A reduction dimension is still refused, and that is the whole of what
    /// [`IndexBuildError::InvalidWriteDomain`] now means.
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
        // A write iterates parallel dimensions and only parallel dimensions.
        // Subset rather than equality: two roots partitioning one output into
        // unequally sized pieces need different point counts, and a point count
        // is the product of the extents a root iterates, so roots that share one
        // domain own equal shares by construction. The reduction half of the
        // rule is unchanged and is what the subset still excludes — a write
        // iterating a reduction dimension would store to one element once per
        // reduced point, which is the double-write the ownership proof exists to
        // refuse and which no coordinate could distinguish.
        //
        // Read from each dimension's own role rather than from the parallel set
        // accumulated so far, so a write authored before the last parallel
        // dimension is declared is admitted or refused on what it names rather
        // than on when it was named.
        if mode == AccessMode::Write
            && domain_set
                .iter()
                .any(|dimension| self.dimensions[*dimension as usize].role != DomainRole::Parallel)
        {
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
    /// **Several roots may name one output tensor.** A repeat is not a
    /// construction error because whether two roots over one boundary are a
    /// legal partition is not decidable here: it depends on the coordinates
    /// every root will carry and on extents the environment resolves, neither
    /// of which is settled while the draft is still open. Refusing the second
    /// root at this site would have refused the legal partition along with the
    /// illegal one, and it also refused it for the wrong reason — the caller
    /// would read "this tensor already has a root" where the real question is
    /// whether the roots jointly own the boundary. The obligation is discharged
    /// at verification instead, where every coordinate exists, and an unsound
    /// set is refused under a diagnostic naming what is actually wrong with it:
    /// [`IndexRegionDiagnostic::OutputPartitionUncovered`],
    /// [`OutputPartitionRangesOverlap`](IndexRegionDiagnostic::OutputPartitionRangesOverlap),
    /// or
    /// [`OutputPartitionDoubleWritten`](IndexRegionDiagnostic::OutputPartitionDoubleWritten).
    /// A root repeated verbatim is the degenerate partition whose two members
    /// occupy one rectangle, and it refuses as an overlap.
    ///
    /// # Errors
    ///
    /// Returns an error for foreign handles, a read access, a type mismatch, or
    /// an exceeded root limit.
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
        self.output_tensors.insert(access_data.tensor);
        self.outputs.push(OutputData {
            access: access.index,
            value: value.index,
        });
        Ok(())
    }

    /// Constructs, authors, verifies, and canonicalizes one region in a single
    /// scoped step, delegating to the same transactional builder and consuming
    /// [`IndexRegionBuilder::build`] verifier as manual construction.
    ///
    /// The mutable draft is confined to `assemble`; only the immutable verified
    /// product escapes on success. Because `assemble` receives the draft by
    /// mutable reference it cannot itself reach the consuming verifier, so the
    /// opaque [`VerifiedIndexRegion`] is only ever produced by the checked path.
    ///
    /// The convenience is exactly equivalent to the ordinary transactional call
    /// site — the manual and closure forms below produce the identical verified
    /// region:
    ///
    /// ```ignore
    /// // Ordinary transactional call site.
    /// let mut builder = IndexRegionBuilder::new(registry)?;
    /// let value = builder
    ///     .apply(constant_key, ScalarAttributes::empty(), &[])?
    ///     .get(0)
    ///     .expect("the constant operation yields one result");
    /// let output = builder.tensor(TensorRole::Output, pixel, Shape::from_dims([]))?;
    /// let write = builder.write(output, &[], &[])?;
    /// builder.output(write, value)?;
    /// let region = builder.build()?;
    ///
    /// // Equivalent closure call site producing the identical verified region.
    /// let region = IndexRegionBuilder::build_with(registry, |builder| {
    ///     let value = builder
    ///         .apply(constant_key, ScalarAttributes::empty(), &[])?
    ///         .get(0)
    ///         .expect("the constant operation yields one result");
    ///     let output = builder.tensor(TensorRole::Output, pixel, Shape::from_dims([]))?;
    ///     let write = builder.write(output, &[], &[])?;
    ///     builder.output(write, value)?;
    ///     Ok(())
    /// })?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CheckedBuildError::Admission`] when builder construction or any
    /// closure insertion is rejected, or [`CheckedBuildError::Verification`]
    /// carrying every deterministic diagnostic and the recoverable builder when
    /// whole-region verification rejects the assembled draft.
    pub fn build_with<F>(
        registry: FrozenScalarRegistry,
        assemble: F,
    ) -> Result<VerifiedIndexRegion, CheckedBuildError<IndexBuildError, IndexRegionBuildError>>
    where
        F: FnOnce(&mut Self) -> Result<(), IndexBuildError>,
    {
        let builder = Self::new(registry).map_err(CheckedBuildError::Admission)?;
        build_checked(builder, assemble, Self::build)
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

    const fn unknown_reason(self) -> IndexDomainUnknownReason {
        match self {
            Self::Cells { required, limit } => IndexDomainUnknownReason::ResourceLimit {
                resource: ProofResource::Cells,
                required,
                limit,
            },
            Self::IntegerBytes { required, limit } => IndexDomainUnknownReason::ResourceLimit {
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
                // A symbolic coefficient has no magnitude to bound here; what
                // bounds it is the environment's own declaration limits, and
                // its admission already ran.
                if let Some(coefficient) = term.coefficient.as_literal() {
                    check_integer(&coefficient.0)?;
                }
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
/// Returns the point count an admitted enumeration actually walked.
fn enumerated_points(points: Option<u64>) -> u64 {
    points.expect("a retained exhaustive proof enumerated a determined domain")
}

fn bounded_index(index: usize) -> u32 {
    u32::try_from(index).expect("governed region limits fit u32")
}
mod compact;
mod identity;
mod proof;
mod reduction;

use identity::{
    access_read_key, alpha_expr_key_impl, apply_operation_key, assign_dimension,
    encode_reducer_body, encode_region, encode_u32s, encoded_reducer_body_len,
    encoded_reducer_operation_base_len, encoded_reducer_operation_result_increment,
    encoded_reducer_operation_result_overhead, encoded_reducer_parameter_len, encoded_region_len,
    interval_linear, nested_operation_key, operation_structural_key, remap_node, remap_operation,
    structural_index_key,
};
use reduction::{
    accumulate_linear_term, advance_point, compact_reducer_body, minimum_reducer_body,
    stage_reducer_results,
};

#[cfg(test)]
mod tests;
