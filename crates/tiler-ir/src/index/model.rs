use std::collections::BTreeSet;
use std::collections::btree_set;
use std::iter::Zip;
use std::ops::RangeFrom;
use std::slice::Iter;
use std::sync::Arc;

use num_bigint::BigInt;

use crate::semantic::{ResolvedValueType, SemanticProgram};
use crate::shape::Shape;

use super::{
    BoundsWitnessId, IndexInteger, VerifiedDimensionId, VerifiedIndexExprId, VerifiedScalarExprId,
    VerifiedTensorAccessId, VerifiedTensorId, WriteOwnershipWitnessId,
};

/// Canonical identity of the semantic region refined by an index relation.
///
/// The first profile authenticates this correlation from a completed semantic
/// program. It is not evidence that the relation implements a selected region;
/// a later compiler legality layer owns that proof.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticRegionIdentity(Vec<u8>);

impl SemanticRegionIdentity {
    /// Correlates an index relation with an authentic completed semantic program.
    #[must_use]
    pub fn for_program(program: &SemanticProgram) -> Self {
        Self(program.canonical_identity().as_bytes().to_vec())
    }

    /// Returns the exact canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Whether one region boundary tensor is consumed or produced.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum TensorRole {
    /// A value supplied at the region boundary.
    Input,
    /// A value produced at the region boundary.
    Output,
}

/// Whether a domain dimension names independent outputs or reduction contributors.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DomainRole {
    /// Independently owned logical output points.
    Parallel,
    /// Lexically bound contributors to a scalar reduction.
    Reduction,
}

/// A logical tensor access mode supported by the first index profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum AccessMode {
    /// A potentially many-to-one logical read.
    Read,
    /// One complete, uniquely owned logical output write.
    Write,
}

/// Exact contributor order for the first serial reduction profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ContributorOrder {
    /// Axes are visited in declared order; each axis increases from zero.
    AxisLexicographic,
}

/// Classification of one canonical logical index expression.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum IndexExprClass {
    /// Constant coefficients and no division or modulo.
    Affine,
    /// A positive constant divisor introduces floor division or modulo.
    QuasiAffine,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct LinearTermData {
    pub(super) coefficient: IndexInteger,
    pub(super) value: u32,
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
    pub(super) node: IndexNode,
    pub(super) canonical: Vec<u8>,
    pub(super) dimensions: BTreeSet<u32>,
    pub(super) class: IndexExprClass,
    pub(super) interval: Option<(BigInt, BigInt)>,
}

#[derive(Clone, Debug)]
pub(super) struct DimensionData {
    pub(super) role: DomainRole,
    pub(super) extent: u64,
}

#[derive(Clone, Debug)]
pub(super) struct TensorData {
    pub(super) role: TensorRole,
    pub(super) value_type: ResolvedValueType,
    pub(super) shape: Shape,
}

#[derive(Clone, Debug)]
pub(super) struct AccessData {
    pub(super) tensor: u32,
    pub(super) mode: AccessMode,
    pub(super) domain: Vec<u32>,
    pub(super) coordinates: Vec<u32>,
    pub(super) canonical: Vec<u8>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum ScalarNode {
    Load {
        access: u32,
    },
    F32Constant {
        bits: u32,
    },
    F32Multiply {
        left: u32,
        right: u32,
    },
    F32Add {
        left: u32,
        right: u32,
    },
    StrictSerialF32Sum {
        value: u32,
        dimensions: Vec<u32>,
        order: ContributorOrder,
        empty_identity_bits: u32,
    },
}

#[derive(Clone, Debug)]
pub(super) struct ScalarExprData {
    pub(super) node: ScalarNode,
    pub(super) free_dimensions: BTreeSet<u32>,
    pub(super) depth: usize,
    pub(super) canonical: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(super) struct OutputData {
    pub(super) access: u32,
    pub(super) value: u32,
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

/// Opaque canonical bytes for one verified index region.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalIndexRegionIdentity(pub(super) Vec<u8>);

impl CanonicalIndexRegionIdentity {
    /// Returns the collision-free canonical encoding.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Immutable, structurally and relationally verified index region.
#[derive(Clone, Debug)]
pub struct VerifiedIndexRegion {
    pub(super) data: Arc<VerifiedIndexRegionData>,
}

#[derive(Clone, Debug)]
pub(super) struct VerifiedIndexRegionData {
    pub(super) semantic_region: SemanticRegionIdentity,
    pub(super) dimensions: Vec<DimensionData>,
    pub(super) tensors: Vec<TensorData>,
    pub(super) expressions: Vec<IndexExprData>,
    pub(super) accesses: Vec<VerifiedAccessData>,
    pub(super) scalars: Vec<ScalarExprData>,
    pub(super) outputs: Vec<OutputData>,
    pub(super) identity: CanonicalIndexRegionIdentity,
}

#[derive(Clone, Debug)]
pub(super) struct VerifiedAccessData {
    pub(super) tensor: u32,
    pub(super) mode: AccessMode,
    pub(super) domain: Vec<u32>,
    pub(super) coordinates: Vec<u32>,
    pub(super) bounds: BoundsWitnessId,
    pub(super) bounds_proof: BoundsProof,
    pub(super) ownership: Option<(WriteOwnershipWitnessId, WriteOwnershipProof)>,
}

macro_rules! arena_iterator {
    ($name:ident, $item:ident, $data:ty) => {
        #[doc = concat!("Iterator over verified ", stringify!($item), " values.")]
        pub struct $name<'a> {
            pub(super) inner: Zip<Iter<'a, $data>, RangeFrom<u32>>,
        }

        impl<'a> Iterator for $name<'a> {
            type Item = $item<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                self.inner.next().map(|(data, index)| $item { index, data })
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }

        impl ExactSizeIterator for $name<'_> {}
    };
}

/// Borrowed inspection of one domain dimension.
#[derive(Clone, Copy, Debug)]
pub struct DomainDimensionRef<'a> {
    pub(super) index: u32,
    pub(super) data: &'a DimensionData,
}

arena_iterator!(DomainDimensions, DomainDimensionRef, DimensionData);

impl DomainDimensionRef<'_> {
    /// Returns its verified-region-local identity.
    #[must_use]
    pub const fn id(self) -> VerifiedDimensionId {
        VerifiedDimensionId(self.index)
    }

    /// Returns whether the dimension is parallel or reduction.
    #[must_use]
    pub const fn role(self) -> DomainRole {
        self.data.role
    }

    /// Returns the static half-open extent.
    #[must_use]
    pub const fn extent(self) -> u64 {
        self.data.extent
    }
}

/// Borrowed inspection of one ordered tensor boundary.
#[derive(Clone, Copy, Debug)]
pub struct TensorRef<'a> {
    pub(super) index: u32,
    pub(super) data: &'a TensorData,
}

arena_iterator!(Tensors, TensorRef, TensorData);

impl<'a> TensorRef<'a> {
    /// Returns its verified-region-local identity.
    #[must_use]
    pub const fn id(self) -> VerifiedTensorId {
        VerifiedTensorId(self.index)
    }

    /// Returns whether the tensor is consumed or produced.
    #[must_use]
    pub const fn role(self) -> TensorRole {
        self.data.role
    }

    /// Returns its canonical logical value type.
    #[must_use]
    pub const fn resolved_type(self) -> &'a ResolvedValueType {
        &self.data.value_type
    }

    /// Returns its logical shape.
    #[must_use]
    pub const fn shape(self) -> &'a Shape {
        &self.data.shape
    }
}

/// One term in a canonical linear combination.
#[derive(Clone, Copy, Debug)]
pub struct LinearTermRef<'a> {
    data: &'a LinearTermData,
}

impl<'a> LinearTermRef<'a> {
    /// Returns the exact coefficient.
    #[must_use]
    pub const fn coefficient(self) -> &'a IndexInteger {
        &self.data.coefficient
    }

    /// Returns the scaled expression.
    #[must_use]
    pub const fn value(self) -> VerifiedIndexExprId {
        VerifiedIndexExprId(self.data.value)
    }
}

/// Iterator over canonical linear-combination terms.
#[derive(Clone, Debug)]
pub struct LinearTerms<'a> {
    inner: Iter<'a, LinearTermData>,
}

impl<'a> Iterator for LinearTerms<'a> {
    type Item = LinearTermRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|data| LinearTermRef { data })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for LinearTerms<'_> {}

/// Borrowed structural inspection of a canonical index expression.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum IndexExprView<'a> {
    /// An exact integer constant.
    Constant(&'a IndexInteger),
    /// One domain dimension.
    Dimension(VerifiedDimensionId),
    /// A normalized exact linear combination.
    LinearCombination {
        /// Exact constant term.
        constant: &'a IndexInteger,
        /// Nonzero, structurally ordered coefficients and bases.
        terms: LinearTerms<'a>,
    },
    /// Euclidean floor division by a positive constant.
    FloorDiv {
        /// Dividend expression.
        dividend: VerifiedIndexExprId,
        /// Positive constant divisor.
        divisor: u64,
    },
    /// Euclidean modulo by a positive constant.
    Modulo {
        /// Dividend expression.
        dividend: VerifiedIndexExprId,
        /// Positive constant divisor.
        divisor: u64,
    },
}

/// Borrowed inspection of one canonical index expression.
#[derive(Clone, Copy, Debug)]
pub struct IndexExprRef<'a> {
    pub(super) index: u32,
    pub(super) data: &'a IndexExprData,
}

arena_iterator!(IndexExpressions, IndexExprRef, IndexExprData);

impl<'a> IndexExprRef<'a> {
    /// Returns its verified-region-local identity.
    #[must_use]
    pub const fn id(self) -> VerifiedIndexExprId {
        VerifiedIndexExprId(self.index)
    }

    /// Returns its strongest verified expression class.
    #[must_use]
    pub const fn class(self) -> IndexExprClass {
        self.data.class
    }

    /// Returns a borrowed structural view.
    #[must_use]
    pub fn view(self) -> IndexExprView<'a> {
        match &self.data.node {
            IndexNode::Constant(value) => IndexExprView::Constant(value),
            IndexNode::Dimension(dimension) => {
                IndexExprView::Dimension(VerifiedDimensionId(*dimension))
            }
            IndexNode::LinearCombination { constant, terms } => IndexExprView::LinearCombination {
                constant,
                terms: LinearTerms {
                    inner: terms.iter(),
                },
            },
            IndexNode::FloorDiv { dividend, divisor } => IndexExprView::FloorDiv {
                dividend: VerifiedIndexExprId(*dividend),
                divisor: *divisor,
            },
            IndexNode::Modulo { dividend, divisor } => IndexExprView::Modulo {
                dividend: VerifiedIndexExprId(*dividend),
                divisor: *divisor,
            },
        }
    }
}

/// Iterator over verified dimension identities stored by another entity.
#[derive(Clone, Debug)]
pub struct VerifiedDimensionIds<'a> {
    inner: Iter<'a, u32>,
}

impl Iterator for VerifiedDimensionIds<'_> {
    type Item = VerifiedDimensionId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied().map(VerifiedDimensionId)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for VerifiedDimensionIds<'_> {}

/// Iterator over verified index-expression identities stored by an access.
#[derive(Clone, Debug)]
pub struct VerifiedIndexExprIds<'a> {
    inner: Iter<'a, u32>,
}

impl Iterator for VerifiedIndexExprIds<'_> {
    type Item = VerifiedIndexExprId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied().map(VerifiedIndexExprId)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for VerifiedIndexExprIds<'_> {}

/// Iterator over verified dimensions in a scalar evaluation scope.
#[derive(Clone, Debug)]
pub struct ScalarEvaluationDimensions<'a> {
    inner: btree_set::Iter<'a, u32>,
}

impl Iterator for ScalarEvaluationDimensions<'_> {
    type Item = VerifiedDimensionId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied().map(VerifiedDimensionId)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for ScalarEvaluationDimensions<'_> {}

/// How an access-bounds witness was established.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BoundsProofView {
    /// The access domain is empty.
    VacuousEmptyDomain,
    /// Static interval propagation proved every coordinate.
    Interval,
    /// Bounded exhaustive evaluation proved every finite point.
    Exhaustive {
        /// Evaluated point count.
        points: u64,
    },
}

/// How ordinary write completeness and uniqueness were established.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WriteOwnershipProofView {
    /// Coordinates are a permutation of the complete parallel domain.
    CoordinatePermutation,
    /// Bounded exhaustive evaluation proved a bijection.
    Exhaustive {
        /// Evaluated writer-point count.
        points: u64,
    },
}

/// Borrowed inspection of one verified logical tensor access.
#[derive(Clone, Copy, Debug)]
pub struct TensorAccessRef<'a> {
    pub(super) index: u32,
    pub(super) data: &'a VerifiedAccessData,
}

arena_iterator!(TensorAccesses, TensorAccessRef, VerifiedAccessData);

impl<'a> TensorAccessRef<'a> {
    /// Returns its verified-region-local identity.
    #[must_use]
    pub const fn id(self) -> VerifiedTensorAccessId {
        VerifiedTensorAccessId(self.index)
    }

    /// Returns the accessed tensor boundary.
    #[must_use]
    pub const fn tensor(self) -> VerifiedTensorId {
        VerifiedTensorId(self.data.tensor)
    }

    /// Returns the logical access mode.
    #[must_use]
    pub const fn mode(self) -> AccessMode {
        self.data.mode
    }

    /// Returns the explicit evaluation domain in canonical declaration order.
    #[must_use]
    pub fn domain(self) -> VerifiedDimensionIds<'a> {
        VerifiedDimensionIds {
            inner: self.data.domain.iter(),
        }
    }

    /// Returns coordinates in logical tensor-axis order.
    #[must_use]
    pub fn coordinates(self) -> VerifiedIndexExprIds<'a> {
        VerifiedIndexExprIds {
            inner: self.data.coordinates.iter(),
        }
    }

    /// Returns the region-owned bounds witness.
    #[must_use]
    pub const fn bounds_witness(self) -> BoundsWitnessId {
        self.data.bounds
    }

    /// Returns how bounds were proved.
    #[must_use]
    pub const fn bounds_proof(self) -> BoundsProofView {
        match self.data.bounds_proof {
            BoundsProof::VacuousEmptyDomain => BoundsProofView::VacuousEmptyDomain,
            BoundsProof::Interval => BoundsProofView::Interval,
            BoundsProof::Exhaustive { points } => BoundsProofView::Exhaustive { points },
        }
    }

    /// Returns complete unique-write evidence for an ordinary write.
    #[must_use]
    pub const fn write_ownership_witness(self) -> Option<WriteOwnershipWitnessId> {
        match self.data.ownership {
            Some((witness, _)) => Some(witness),
            None => None,
        }
    }

    /// Returns how ordinary write ownership was proved.
    #[must_use]
    pub const fn write_ownership_proof(self) -> Option<WriteOwnershipProofView> {
        match self.data.ownership {
            Some((_, WriteOwnershipProof::CoordinatePermutation)) => {
                Some(WriteOwnershipProofView::CoordinatePermutation)
            }
            Some((_, WriteOwnershipProof::Exhaustive { points })) => {
                Some(WriteOwnershipProofView::Exhaustive { points })
            }
            None => None,
        }
    }
}

/// Borrowed structural inspection of one scalar expression.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ScalarExprView<'a> {
    /// Load through one verified read.
    Load {
        /// Verified read access.
        access: VerifiedTensorAccessId,
    },
    /// Exact IEEE binary32 payload.
    F32Constant {
        /// Exact IEEE binary32 payload.
        bits: u32,
    },
    /// Ordered binary32 multiplication.
    F32Multiply {
        /// Ordered left operand.
        left: VerifiedScalarExprId,
        /// Ordered right operand.
        right: VerifiedScalarExprId,
    },
    /// Ordered binary32 addition.
    F32Add {
        /// Ordered left operand.
        left: VerifiedScalarExprId,
        /// Ordered right operand.
        right: VerifiedScalarExprId,
    },
    /// Exact serial binary32 reduction.
    StrictSerialF32Sum {
        /// Contributor expression.
        value: VerifiedScalarExprId,
        /// Lexically bound reduction dimensions.
        dimensions: VerifiedDimensionIds<'a>,
        /// Exact contributor traversal order.
        order: ContributorOrder,
        /// Exact empty-domain result payload.
        empty_identity_bits: u32,
    },
}

/// Borrowed inspection of one scalar expression.
#[derive(Clone, Copy, Debug)]
pub struct ScalarExprRef<'a> {
    pub(super) index: u32,
    pub(super) data: &'a ScalarExprData,
}

arena_iterator!(ScalarExpressions, ScalarExprRef, ScalarExprData);

impl<'a> ScalarExprRef<'a> {
    /// Returns its verified-region-local identity.
    #[must_use]
    pub const fn id(self) -> VerifiedScalarExprId {
        VerifiedScalarExprId(self.index)
    }

    /// Returns dimensions in the expression's remaining evaluation scope.
    #[must_use]
    pub fn evaluation_domain(self) -> ScalarEvaluationDimensions<'a> {
        ScalarEvaluationDimensions {
            inner: self.data.free_dimensions.iter(),
        }
    }

    /// Returns a borrowed structural view.
    #[must_use]
    pub fn view(self) -> ScalarExprView<'a> {
        match &self.data.node {
            ScalarNode::Load { access } => ScalarExprView::Load {
                access: VerifiedTensorAccessId(*access),
            },
            ScalarNode::F32Constant { bits } => ScalarExprView::F32Constant { bits: *bits },
            ScalarNode::F32Multiply { left, right } => ScalarExprView::F32Multiply {
                left: VerifiedScalarExprId(*left),
                right: VerifiedScalarExprId(*right),
            },
            ScalarNode::F32Add { left, right } => ScalarExprView::F32Add {
                left: VerifiedScalarExprId(*left),
                right: VerifiedScalarExprId(*right),
            },
            ScalarNode::StrictSerialF32Sum {
                value,
                dimensions,
                order,
                empty_identity_bits,
            } => ScalarExprView::StrictSerialF32Sum {
                value: VerifiedScalarExprId(*value),
                dimensions: VerifiedDimensionIds {
                    inner: dimensions.iter(),
                },
                order: *order,
                empty_identity_bits: *empty_identity_bits,
            },
        }
    }
}

/// Borrowed inspection of one ordered output root.
#[derive(Clone, Copy, Debug)]
pub struct OutputRef<'a> {
    pub(super) index: u32,
    pub(super) data: &'a OutputData,
}

arena_iterator!(Outputs, OutputRef, OutputData);

impl OutputRef<'_> {
    /// Returns the stable ordered output position.
    #[must_use]
    pub const fn position(self) -> usize {
        self.index as usize
    }

    /// Returns the output write access.
    #[must_use]
    pub const fn access(self) -> VerifiedTensorAccessId {
        VerifiedTensorAccessId(self.data.access)
    }

    /// Returns the scalar value written at each output point.
    #[must_use]
    pub const fn value(self) -> VerifiedScalarExprId {
        VerifiedScalarExprId(self.data.value)
    }
}

impl VerifiedIndexRegion {
    /// Returns the semantic-region identity this relation claims to refine.
    #[must_use]
    pub fn semantic_region_identity(&self) -> &SemanticRegionIdentity {
        &self.data.semantic_region
    }

    /// Returns domain dimensions in declared semantic iteration order.
    #[must_use]
    pub fn dimensions(&self) -> DomainDimensions<'_> {
        DomainDimensions {
            inner: self.data.dimensions.iter().zip(0_u32..),
        }
    }

    /// Returns ordered input and output tensor boundaries.
    #[must_use]
    pub fn tensors(&self) -> Tensors<'_> {
        Tensors {
            inner: self.data.tensors.iter().zip(0_u32..),
        }
    }

    /// Returns canonical index expressions in verified topological order.
    #[must_use]
    pub fn index_expressions(&self) -> IndexExpressions<'_> {
        IndexExpressions {
            inner: self.data.expressions.iter().zip(0_u32..),
        }
    }

    /// Returns verified logical accesses.
    #[must_use]
    pub fn accesses(&self) -> TensorAccesses<'_> {
        TensorAccesses {
            inner: self.data.accesses.iter().zip(0_u32..),
        }
    }

    /// Returns scalar expressions in verified topological order.
    #[must_use]
    pub fn scalar_expressions(&self) -> ScalarExpressions<'_> {
        ScalarExpressions {
            inner: self.data.scalars.iter().zip(0_u32..),
        }
    }

    /// Returns ordered output roots.
    #[must_use]
    pub fn outputs(&self) -> Outputs<'_> {
        Outputs {
            inner: self.data.outputs.iter().zip(0_u32..),
        }
    }

    /// Returns the canonical content identity.
    #[must_use]
    pub fn canonical_identity(&self) -> &CanonicalIndexRegionIdentity {
        &self.data.identity
    }
}
