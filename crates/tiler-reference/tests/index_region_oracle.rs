//! Public-API integration proof for the verified index-region reference oracle.
//!
//! These end-to-end cases drive the oracle exclusively through the crate's
//! re-exported public surface, so they live in an admitted integration target
//! rather than inside the module they exercise.

use std::sync::Arc;

use tiler_ir::index::{
    DimensionId, DomainRole, FrozenScalarRegistry, IndexExprId, IndexInteger, IndexRegionBuilder,
    IndexRegionDiagnostic, ScalarArity, ScalarAttributeField, ScalarAttributeSchema,
    ScalarAttributes, ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs,
    ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract, ScalarOperationDefinition,
    ScalarOperationInferencer, ScalarRegistryBuilder, ScalarValueId, SourcedExtent, TensorRole,
    VerifiedIndexRegion, VerifiedTensorAccessId, VerifiedTensorId,
};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::semantic::{
    AttributeFieldId, CANONICAL_F32_ARITHMETIC_NAN_BITS, CanonicalField, CanonicalValue,
    CanonicalValueKind, CanonicalValueView, F32, FrozenSemanticRegistry, NormativeDefinitionRef,
    ProviderDiagnosticCode, ProviderIdentity, ResolvedValueType, TypeKey,
};
use tiler_ir::shape::{
    BindingSource, Extent, ExtentRelation, ExtentTerm, FactProvenance, InterfaceParameterKey,
    RootBinding, SemanticInputConstraint, Shape, ShapeEnvBuilder, ShapeSymbol, SymbolScope,
};
use tiler_reference::{
    FloatBitOrder, FrozenReferenceRegistry, FrozenScalarReferenceRegistry, IndexRegionAuthority,
    IndexRegionEvaluationError, IndexRegionEvaluator, IndexRegionInput,
    ReferenceCapabilityRevision, ReferenceElement, ReferenceOperationError, ReferenceSignature,
    ScalarReferenceOperation, ScalarReferenceOutputs, ScalarReferenceRegistryBuilder,
    ScalarReferenceRequest, Tensor, TensorPayloadView, UnsupportedRegionFeature,
};

const CONSTANT_BITS: AttributeFieldId = AttributeFieldId::new(1);

fn f32_type() -> ResolvedValueType {
    F32::resolved_type()
}

fn f32_format() -> TypeKey {
    TypeKey::new("tiler", "f32", 1).unwrap()
}

fn key(name: &str) -> ScalarOpKey {
    ScalarOpKey::new("example", name, 1).unwrap()
}

fn record() -> CanonicalValue {
    CanonicalValue::record([]).unwrap()
}

struct FixedF32;
impl ScalarOperationInferencer for FixedF32 {
    fn infer(
        &self,
        _: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        outputs.try_push(f32_type())
    }
}

struct SameType;
impl ScalarOperationInferencer for SameType {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        let Some(first) = request.operands().first() else {
            return Err(ScalarInferenceError::new(
                ProviderDiagnosticCode::new("example.arity").unwrap(),
                "at least one operand is required",
            )
            .unwrap());
        };
        if request.operands().iter().any(|operand| operand != first) {
            return Err(ScalarInferenceError::new(
                ProviderDiagnosticCode::new("example.type").unwrap(),
                "operand types differ",
            )
            .unwrap());
        }
        outputs.try_push(first.clone())
    }
}

fn definition(
    name: &str,
    operands: usize,
    attributes: ScalarAttributeSchema,
    inferencer: Arc<dyn ScalarOperationInferencer>,
) -> ScalarOperationDefinition {
    ScalarOperationDefinition::new(
        key(name),
        NormativeDefinitionRef::from_owned(format!("urn:example:{name}:v1")).unwrap(),
        ScalarOperationContract::new(
            attributes,
            ScalarArity::exact(operands).unwrap(),
            ScalarArity::exact(1).unwrap(),
            ScalarEffect::Pure,
            record(),
            record(),
        ),
        inferencer,
    )
}

fn scalar_registry(provider_revision: u32) -> FrozenScalarRegistry {
    // Ad-hoc: `provider_revision` is a parameter, because the subject is that a
    // provider revision change moves the registry identity. The standard profile
    // is a single frozen revision and cannot vary.
    let mut builder = ScalarRegistryBuilder::new(FrozenSemanticRegistry::standard().unwrap());
    let provider = ProviderIdentity::new("example", "f32-scalars", provider_revision).unwrap();
    let constant_schema = ScalarAttributeSchema::new([ScalarAttributeField::required(
        CONSTANT_BITS,
        CanonicalValueKind::FloatBits,
    )])
    .unwrap();
    builder
        .register(
            provider.clone(),
            definition("constant", 0, constant_schema, Arc::new(FixedF32)),
        )
        .unwrap();
    for name in ["multiply", "add"] {
        builder
            .register(
                provider.clone(),
                definition(name, 2, ScalarAttributeSchema::empty(), Arc::new(SameType)),
            )
            .unwrap();
    }
    builder.freeze()
}

fn constant_attributes(value: f32) -> ScalarAttributes {
    ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(
            CONSTANT_BITS,
            CanonicalValue::float_bits(f32_format(), value.to_bits().to_be_bytes()).unwrap(),
        )])
        .unwrap(),
    )
    .unwrap()
}

fn element(value: f32) -> ReferenceElement {
    ReferenceElement::from_float_bits(
        value.to_bits().to_be_bytes(),
        FloatBitOrder::MostSignificantByteFirst,
    )
    .unwrap()
}

fn decode(tensor: &Tensor) -> Result<f32, ReferenceOperationError> {
    let TensorPayloadView::Dense([value]) = tensor.payload() else {
        return Err(ReferenceOperationError::InvalidApplication);
    };
    let bits = <[u8; 4]>::try_from(value.as_bytes())
        .map_err(|_| ReferenceOperationError::InvalidApplication)?;
    Ok(f32::from_bits(u32::from_be_bytes(bits)))
}

fn scalar(value: f32) -> Result<Tensor, ReferenceOperationError> {
    let canonical = if value.is_nan() {
        f32::from_bits(CANONICAL_F32_ARITHMETIC_NAN_BITS)
    } else {
        value
    };
    Tensor::scalar(f32_type(), element(canonical))
        .map_err(|_| ReferenceOperationError::InvalidApplication)
}

fn f32_tensor(shape: Shape, values: impl IntoIterator<Item = f32>) -> Tensor {
    Tensor::dense(f32_type(), shape, values.into_iter().map(element).collect()).unwrap()
}

fn f32_values(tensor: &Tensor) -> Vec<f32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("expected a dense f32 tensor")
    };
    elements
        .iter()
        .map(|value| {
            f32::from_bits(u32::from_be_bytes(
                <[u8; 4]>::try_from(value.as_bytes()).unwrap(),
            ))
        })
        .collect()
}

struct ConstantReference;
impl ScalarReferenceOperation for ConstantReference {
    fn evaluate(
        &self,
        request: ScalarReferenceRequest<'_>,
        outputs: &mut ScalarReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        if !request.operands().is_empty() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let CanonicalValueView::Record(fields) = request.attributes().value().view() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let Some(CanonicalValueView::FloatBits(bits)) = fields
            .iter()
            .find(|field| field.id() == CONSTANT_BITS)
            .map(|field| field.value().view())
        else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        if bits.format() != &f32_format() {
            return Err(ReferenceOperationError::InvalidApplication);
        }
        let value =
            ReferenceElement::from_float_bits(bits.bits(), FloatBitOrder::MostSignificantByteFirst)
                .map_err(|_| ReferenceOperationError::InvalidApplication)?;
        outputs.push(
            Tensor::scalar(f32_type(), value)
                .map_err(|_| ReferenceOperationError::InvalidApplication)?,
        )
    }
}

struct BinaryReference(fn(f32, f32) -> f32);
impl ScalarReferenceOperation for BinaryReference {
    fn evaluate(
        &self,
        request: ScalarReferenceRequest<'_>,
        outputs: &mut ScalarReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        let [left, right] = request.operands() else {
            return Err(ReferenceOperationError::InvalidApplication);
        };
        let value = (self.0)(decode(left)?, decode(right)?);
        outputs.push(scalar(value)?)
    }
}

#[derive(Clone, Copy)]
enum Malformed {
    Failure,
    NoResult,
    WrongType,
}

struct MalformedReference(Malformed);
impl ScalarReferenceOperation for MalformedReference {
    fn evaluate(
        &self,
        _: ScalarReferenceRequest<'_>,
        outputs: &mut ScalarReferenceOutputs,
    ) -> Result<(), ReferenceOperationError> {
        match self.0 {
            Malformed::Failure => Err(ReferenceOperationError::InvalidApplication),
            Malformed::NoResult => Ok(()),
            Malformed::WrongType => outputs.push(
                Tensor::scalar(
                    ResolvedValueType::nominal(TypeKey::new("example", "other", 1).unwrap()),
                    ReferenceElement::new([1]).unwrap(),
                )
                .map_err(|_| ReferenceOperationError::InvalidApplication)?,
            ),
        }
    }
}

fn binary_signature() -> ReferenceSignature {
    ReferenceSignature::new([f32_type(), f32_type()], [f32_type()]).unwrap()
}

fn capabilities(
    scalars: &FrozenScalarRegistry,
    multiply: Arc<dyn ScalarReferenceOperation>,
    include_add: bool,
) -> FrozenScalarReferenceRegistry {
    let provider = ProviderIdentity::new("example", "f32-scalar-reference", 1).unwrap();
    let revision = ReferenceCapabilityRevision::new(1).unwrap();
    let mut builder = ScalarReferenceRegistryBuilder::new(scalars.clone());
    builder
        .register(
            provider.clone(),
            key("constant"),
            ReferenceSignature::new([], [f32_type()]).unwrap(),
            revision,
            Arc::new(ConstantReference),
        )
        .unwrap();
    builder
        .register(
            provider.clone(),
            key("multiply"),
            binary_signature(),
            revision,
            multiply,
        )
        .unwrap();
    if include_add {
        builder
            .register(
                provider,
                key("add"),
                binary_signature(),
                revision,
                Arc::new(BinaryReference(|left, right| left + right)),
            )
            .unwrap();
    }
    builder.freeze().unwrap()
}

fn standard_capabilities(scalars: &FrozenScalarRegistry) -> FrozenScalarReferenceRegistry {
    capabilities(
        scalars,
        Arc::new(BinaryReference(|left, right| left * right)),
        true,
    )
}

fn evaluator(scalars: &FrozenScalarRegistry) -> IndexRegionEvaluator {
    IndexRegionEvaluator::new(
        FrozenReferenceRegistry::standard().unwrap(),
        standard_capabilities(scalars),
    )
}

/// Builds `out[i] = fold(k, 0.0, |acc, k| acc + left[i, k] * right[k])`.
fn matvec_region(
    scalars: &FrozenScalarRegistry,
    rows: u64,
    columns: u64,
) -> Result<VerifiedIndexRegion, Box<dyn std::error::Error>> {
    let mut builder = IndexRegionBuilder::new(scalars.clone())?;
    let i = builder.dimension(DomainRole::Parallel, Extent::new(rows))?;
    let k = builder.dimension(DomainRole::Reduction, Extent::new(columns))?;
    let left = builder.tensor(
        TensorRole::Input,
        f32_type(),
        Shape::from_dims([rows, columns]),
    )?;
    let right = builder.tensor(TensorRole::Input, f32_type(), Shape::from_dims([columns]))?;
    let out = builder.tensor(TensorRole::Output, f32_type(), Shape::from_dims([rows]))?;
    let row = builder.dimension_expr(i)?;
    let column = builder.dimension_expr(k)?;
    let left_value = builder.read(left, &[i, k], &[row, column])?;
    let right_value = builder.read(right, &[k], &[column])?;
    let product = builder
        .apply(
            key("multiply"),
            ScalarAttributes::empty(),
            &[left_value, right_value],
        )?
        .get(0)
        .ok_or("multiply produces one result")?;
    let zero = builder
        .apply(key("constant"), constant_attributes(0.0), &[])?
        .get(0)
        .ok_or("constant produces one result")?;
    let reduced = builder
        .reduce(&[k], &[zero], &[product], |body| {
            let accumulated = body.apply(
                key("add"),
                ScalarAttributes::empty(),
                &[
                    body.state(0).expect("one state parameter"),
                    body.contributor(0).expect("one contributor parameter"),
                ],
            )?;
            body.yield_values(&[accumulated.get(0).expect("add produces one result")])
        })?
        .get(0)
        .ok_or("the reduction produces one result")?;
    let write = builder.write(out, &[i], &[row])?;
    builder.output(write, reduced)?;
    Ok(builder.build()?)
}

fn input_ids(region: &VerifiedIndexRegion) -> Vec<VerifiedTensorId> {
    region
        .tensors()
        .filter(|tensor| tensor.role() == TensorRole::Input)
        .map(tiler_ir::index::TensorRef::id)
        .collect()
}

#[test]
fn matvec_region_evaluates_through_registered_scalar_capabilities() {
    let scalars = scalar_registry(1);
    let region = matvec_region(&scalars, 3, 4).unwrap();
    let left = f32_tensor(
        Shape::from_dims([3, 4]),
        [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ],
    );
    let right = f32_tensor(Shape::from_dims([4]), [1.0, 10.0, 100.0, 1000.0]);
    let ids = input_ids(&region);
    let bindings = [
        IndexRegionInput::new(ids[0], &left),
        IndexRegionInput::new(ids[1], &right),
    ];

    let evaluation = evaluator(&scalars)
        .evaluate(&region, IndexRegionAuthority::new(&scalars), &bindings)
        .unwrap();

    assert_eq!(evaluation.outputs().len(), 1);
    assert_eq!(evaluation.outputs()[0].shape(), &Shape::from_dims([3]));
    assert_eq!(
        f32_values(&evaluation.outputs()[0]),
        [4321.0, 8765.0, 13209.0]
    );
    assert_eq!(
        evaluation.authority().region(),
        region.canonical_identity(),
        "the receipt binds to this exact structural region"
    );
}

#[test]
fn empty_reduction_and_parallel_domains_keep_their_documented_results() {
    let scalars = scalar_registry(1);
    let evaluator = evaluator(&scalars);

    let region = matvec_region(&scalars, 3, 0).unwrap();
    let left = f32_tensor(Shape::from_dims([3, 0]), []);
    let right = f32_tensor(Shape::from_dims([0]), []);
    let ids = input_ids(&region);
    let outputs = evaluator
        .evaluate(
            &region,
            IndexRegionAuthority::new(&scalars),
            &[
                IndexRegionInput::new(ids[0], &left),
                IndexRegionInput::new(ids[1], &right),
            ],
        )
        .unwrap()
        .into_outputs();
    assert_eq!(
        f32_values(&outputs[0])
            .into_iter()
            .map(f32::to_bits)
            .collect::<Vec<_>>(),
        [0.0_f32.to_bits(); 3],
        "an empty reduction domain yields the initial state"
    );

    let region = matvec_region(&scalars, 0, 4).unwrap();
    let left = f32_tensor(Shape::from_dims([0, 4]), []);
    let right = f32_tensor(Shape::from_dims([4]), [1.0, 2.0, 3.0, 4.0]);
    let ids = input_ids(&region);
    let outputs = evaluator
        .evaluate(
            &region,
            IndexRegionAuthority::new(&scalars),
            &[
                IndexRegionInput::new(ids[0], &left),
                IndexRegionInput::new(ids[1], &right),
            ],
        )
        .unwrap()
        .into_outputs();
    assert_eq!(outputs[0].shape(), &Shape::from_dims([0]));
    assert!(f32_values(&outputs[0]).is_empty());
}

/// Builds the concatenate shape: `roots` inputs of `points` elements each,
/// written into one `boundary`-element output at disjoint contiguous offsets.
///
/// Root `r` writes `out[r * points + i] = source_r[i]` over the region's one
/// parallel dimension, so with `boundary == roots * points` the roots tile the
/// output exactly and none of them produces it alone. Returns the builder rather
/// than the region so a case can watch verification refuse.
fn concatenated_output_region(
    scalars: &FrozenScalarRegistry,
    points: u64,
    roots: u64,
    boundary: u64,
) -> Result<IndexRegionBuilder, Box<dyn std::error::Error>> {
    let mut builder = IndexRegionBuilder::new(scalars.clone())?;
    let i = builder.dimension(DomainRole::Parallel, Extent::new(points))?;
    let out = builder.tensor(TensorRole::Output, f32_type(), Shape::from_dims([boundary]))?;
    let index = builder.dimension_expr(i)?;
    for root in 0..roots {
        let source = builder.tensor(TensorRole::Input, f32_type(), Shape::from_dims([points]))?;
        let value = builder.read(source, &[i], &[index])?;
        let offset = i128::from(
            root.checked_mul(points)
                .ok_or("the offset is representable")?,
        );
        let coordinate = builder.linear_combination(
            IndexInteger::from_i128(offset),
            &[(IndexInteger::from_i128(1), index)],
        )?;
        let write = builder.write(out, &[i], &[coordinate])?;
        builder.output(write, value)?;
    }
    Ok(builder)
}

/// An output two roots partition evaluates to the one tensor they fill jointly.
///
/// One buffer per root would leave each root's copy filled only where that root
/// wrote — no part of the program produces such a tensor, and the evaluation
/// would report two of them where the region declares one boundary.
#[test]
fn partitioned_output_roots_fill_one_joined_tensor() {
    let scalars = scalar_registry(1);
    let region = concatenated_output_region(&scalars, 3, 2, 6)
        .unwrap()
        .build()
        .unwrap();
    let lower = f32_tensor(Shape::from_dims([3]), [1.0, 2.0, 3.0]);
    let upper = f32_tensor(Shape::from_dims([3]), [10.0, 20.0, 30.0]);
    let ids = input_ids(&region);

    let outputs = evaluator(&scalars)
        .evaluate(
            &region,
            IndexRegionAuthority::new(&scalars),
            &[
                IndexRegionInput::new(ids[0], &lower),
                IndexRegionInput::new(ids[1], &upper),
            ],
        )
        .unwrap()
        .into_outputs();

    assert_eq!(
        outputs.len(),
        1,
        "two roots partition one boundary, and one boundary is one tensor"
    );
    assert_eq!(outputs[0].shape(), &Shape::from_dims([6]));
    // Root 0 writes `out[i] = lower[i]` and root 1 writes
    // `out[i + 3] = upper[i]`, so the joined tensor is `lower` then `upper`.
    assert_eq!(
        f32_values(&outputs[0]),
        [1.0, 2.0, 3.0, 10.0, 20.0, 30.0],
        "each root's contribution lands in its own half of the one output"
    );
}

/// An empty partitioned boundary is one output tensor, not one per root.
///
/// The one partition shape whose per-root planning produced a *result* rather
/// than a refusal: with no elements to fill, each root's own buffer was already
/// complete, so the evaluation succeeded and reported two tensors where the
/// region declares one boundary. Retained because it is the case where the
/// wrong answer was silent.
#[test]
fn an_empty_partitioned_boundary_is_one_output_tensor() {
    let scalars = scalar_registry(1);
    let region = concatenated_output_region(&scalars, 0, 2, 0)
        .unwrap()
        .build()
        .unwrap();
    let empty = f32_tensor(Shape::from_dims([0]), []);
    let ids = input_ids(&region);

    let outputs = evaluator(&scalars)
        .evaluate(
            &region,
            IndexRegionAuthority::new(&scalars),
            &[
                IndexRegionInput::new(ids[0], &empty),
                IndexRegionInput::new(ids[1], &empty),
            ],
        )
        .unwrap()
        .into_outputs();

    assert_eq!(outputs.len(), 1, "two roots, one boundary, one tensor");
    assert_eq!(outputs[0].shape(), &Shape::from_dims([0]));
}

/// Dropping one root's contribution is refused rather than quietly evaluated.
///
/// The same shape with a single root over the same six-element boundary covers
/// half of it. The refusal is the verifier's, which is what lets the oracle fill
/// a partitioned buffer from several roots without re-deriving coverage: a
/// region that would leave a gap does not reach it. The oracle's own
/// `IncompleteWrite` check remains as the independent floor beneath that.
#[test]
fn a_partition_missing_a_root_is_refused_before_evaluation() {
    let scalars = scalar_registry(1);
    let diagnostics = concatenated_output_region(&scalars, 3, 1, 6)
        .unwrap()
        .build()
        .unwrap_err();
    assert!(
        diagnostics.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::WriteOwnershipNotProven { .. }
        )),
        "one root covering three of six elements owns no boundary: {:?}",
        diagnostics.diagnostics()
    );
}

/// Builds one `boundary`-element output whose roots each iterate a parallel
/// dimension of their **own** extent, copying an input of that same extent.
///
/// This is the shape the write-domain relaxation admitted and one shared domain
/// cannot express: a root's point count is the product of the extents it
/// iterates, so roots sharing one domain own equal shares by construction. Each
/// member is `(extent, offset)` and copies its own `[extent]`-shaped input to
/// `out[d + offset]`, so its rectangle is `[offset, offset + extent)` and the
/// joined output is the members' inputs concatenated in declaration order.
///
/// The stored value *varies along the root's own dimension* — it is a read at
/// `source[d]` rather than a constant — which is what makes the walk seed each
/// root's frame from that root's domain rather than reach a value it could have
/// produced from any environment at all.
///
/// With more than one member every domain is a strict subset of the region's
/// parallel dimensions; with exactly one member of the full extent the sole
/// domain *is* that set, which is the control the cases below are measured
/// against.
fn unequal_partition_region(
    scalars: &FrozenScalarRegistry,
    boundary: u64,
    members: &[(u64, i128)],
) -> Result<VerifiedIndexRegion, Box<dyn std::error::Error>> {
    let mut builder = IndexRegionBuilder::new(scalars.clone())?;
    let out = builder.tensor(TensorRole::Output, f32_type(), Shape::from_dims([boundary]))?;
    for (extent, offset) in members {
        let dimension = builder.dimension(DomainRole::Parallel, Extent::new(*extent))?;
        let index = builder.dimension_expr(dimension)?;
        let source = builder.tensor(TensorRole::Input, f32_type(), Shape::from_dims([*extent]))?;
        let value = builder.read(source, &[dimension], &[index])?;
        let coordinate = builder.linear_combination(
            IndexInteger::from_i128(*offset),
            &[(IndexInteger::from_i128(1), index)],
        )?;
        let write = builder.write(out, &[dimension], &[coordinate])?;
        builder.output(write, value)?;
    }
    Ok(builder.build()?)
}

/// Returns the write access of each output root in region order.
fn output_accesses(region: &VerifiedIndexRegion) -> Vec<VerifiedTensorAccessId> {
    region
        .outputs()
        .map(tiler_ir::index::OutputRef::access)
        .collect()
}

/// Roots of unequal extents each walk their own domain into one joined tensor.
///
/// Three and five into eight. The roots iterate dimensions of extent three and
/// five, so the region's parallel dimension set has a fifteen-point product —
/// and *no root walks it*: root 0 has three points and root 1 has five. Each is
/// sent over its own domain, so the boundary comes back as the two inputs
/// concatenated at the offsets the roots declare.
///
/// `root_point_count` is the assertion that separates this from the shared walk
/// that preceded it. Eight is the number of elements the boundary retains, one
/// per (root, point-of-that-root's-domain) pair; fifteen was the number the one
/// shared parallel space reported, and dividing spans into it would have been
/// dividing into a number nothing counts.
#[test]
fn unequally_partitioned_roots_each_walk_their_own_domain() {
    let scalars = scalar_registry(1);
    let evaluator = evaluator(&scalars);
    let region = unequal_partition_region(&scalars, 8, &[(3, 0), (5, 3)]).unwrap();
    let lower = f32_tensor(Shape::from_dims([3]), [1.0, 2.0, 3.0]);
    let upper = f32_tensor(Shape::from_dims([5]), [10.0, 20.0, 30.0, 40.0, 50.0]);
    let ids = input_ids(&region);
    let inputs = [
        IndexRegionInput::new(ids[0], &lower),
        IndexRegionInput::new(ids[1], &upper),
    ];

    let staged = evaluator
        .stage(&region, IndexRegionAuthority::new(&scalars), &inputs)
        .expect("a partition into unequal members stages");
    assert_eq!(
        staged.root_point_count(),
        Some(8),
        "the walk is one point per retained element, not the fifteen-point \
         product of two dimensions no root iterates together",
    );

    let outputs = evaluator
        .evaluate(&region, IndexRegionAuthority::new(&scalars), &inputs)
        .unwrap()
        .into_outputs();
    assert_eq!(outputs.len(), 1, "two roots, one boundary, one tensor");
    assert_eq!(outputs[0].shape(), &Shape::from_dims([8]));
    assert_eq!(
        f32_values(&outputs[0]),
        [1.0, 2.0, 3.0, 10.0, 20.0, 30.0, 40.0, 50.0],
        "root 0 fills [0, 3) from its own three-element input and root 1 fills \
         [3, 8) from its own five-element one",
    );

    // The control holds the shape fixed and varies only the property under
    // test: one root over the whole boundary, whose sole domain *is* the
    // region's parallel dimension set.
    let whole = unequal_partition_region(&scalars, 8, &[(8, 0)]).unwrap();
    let source = f32_tensor(
        Shape::from_dims([8]),
        [1.0, 2.0, 3.0, 10.0, 20.0, 30.0, 40.0, 50.0],
    );
    let ids = input_ids(&whole);
    let outputs = evaluator
        .evaluate(
            &whole,
            IndexRegionAuthority::new(&scalars),
            &[IndexRegionInput::new(ids[0], &source)],
        )
        .unwrap()
        .into_outputs();
    assert_eq!(
        f32_values(&outputs[0]),
        [1.0, 2.0, 3.0, 10.0, 20.0, 30.0, 40.0, 50.0],
        "a sole root whose domain is the parallel dimension set is unchanged",
    );
}

/// A zero-extent root contributes nothing without emptying a sibling's walk.
///
/// The degenerate partition member, and the one the concatenate lowering emits
/// at its pinned occurrence: `out` is `[2, 1]`, root 0 iterates both parallel
/// dimensions and writes `out[d0, d1]` — owning no element, because `d1` has
/// extent zero — and root 1 iterates `d0` alone and writes `out[d0, 0]`, owning
/// the whole boundary. The verifier admits this; the roots are disjoint and
/// cover exactly.
///
/// Under one shared parallel space the zero extent zeroed the *whole* product,
/// so no point was walked, nothing was written, and the boundary came back as an
/// `IncompleteWrite` blaming root 0 — the sibling that owns nothing. Per-root
/// domains leave root 0 empty and root 1's two points to walk, and the stored
/// constants are what assert which of them landed: root 0 stores `7.0` and root
/// 1 stores `1.0`, so a result of `[1.0, 1.0]` is reachable only by evaluating
/// root 1 over its own domain.
#[test]
fn a_zero_extent_write_root_contributes_nothing_and_empties_no_sibling() {
    let scalars = scalar_registry(1);
    let evaluator = evaluator(&scalars);
    let mut builder = IndexRegionBuilder::new(scalars.clone()).unwrap();
    let out = builder
        .tensor(TensorRole::Output, f32_type(), Shape::from_dims([2, 1]))
        .unwrap();
    let full = builder
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let empty = builder
        .dimension(DomainRole::Parallel, Extent::new(0))
        .unwrap();
    let row = builder.dimension_expr(full).unwrap();
    let column = builder.dimension_expr(empty).unwrap();
    let zero = builder.constant(IndexInteger::from_i128(0)).unwrap();
    let unreachable = builder
        .apply(key("constant"), constant_attributes(7.0), &[])
        .unwrap()
        .get(0)
        .expect("constant produces one result");
    let stored = builder
        .apply(key("constant"), constant_attributes(1.0), &[])
        .unwrap()
        .get(0)
        .expect("constant produces one result");
    let whole_domain = builder.write(out, &[full, empty], &[row, column]).unwrap();
    builder.output(whole_domain, unreachable).unwrap();
    let short_domain = builder.write(out, &[full], &[row, zero]).unwrap();
    builder.output(short_domain, stored).unwrap();
    let region = builder.build().unwrap();

    // What makes the value assertion below a claim about *which* root ran: were
    // the two writes interned into one access, or the two constants into one
    // value, it would hold whichever root the walk had reached.
    let accesses = output_accesses(&region);
    assert_ne!(
        accesses[0], accesses[1],
        "the whole-domain root and the short root are distinct accesses",
    );
    assert_ne!(
        unreachable, stored,
        "the two roots store distinct values, so the result names one of them",
    );

    let staged = evaluator
        .stage(&region, IndexRegionAuthority::new(&scalars), &[])
        .expect("a zero-extent member stages");
    assert_eq!(
        staged.root_point_count(),
        Some(2),
        "the empty root contributes no point and the full one contributes both",
    );

    let outputs = evaluator
        .evaluate(&region, IndexRegionAuthority::new(&scalars), &[])
        .unwrap()
        .into_outputs();
    assert_eq!(outputs.len(), 1, "two roots, one boundary, one tensor");
    assert_eq!(outputs[0].shape(), &Shape::from_dims([2, 1]));
    assert_eq!(
        f32_values(&outputs[0]),
        [1.0, 1.0],
        "the short root filled the boundary; the empty root stored nothing, and \
         its `7.0` appears nowhere",
    );
}

/// Spans of root points compose to the same tensor, across a root boundary.
///
/// The executable half of `StagedIndexRegionEvaluation`'s span argument under
/// roots that do not share a domain. Five widths over the three-and-five
/// partition — one, the width that ends exactly on the root boundary, two that
/// straddle it, and one wider than the whole walk — commit identical values, and
/// the whole-region path commits them too.
///
/// The straddle is what the argument is about, so it is watched rather than left
/// to the arithmetic of a width: a span of five walks root 0's three points and
/// two of root 1's, and the next span finishes the remaining three. Crossing
/// from one root to the next mid-span is sound for the same reason crossing
/// between two points of one root is — no root point can observe another's
/// write — and nothing else in this file would notice if it were not.
#[test]
fn spans_of_root_points_compose_across_a_root_boundary() {
    let scalars = scalar_registry(1);
    let evaluator = evaluator(&scalars);
    let region = unequal_partition_region(&scalars, 8, &[(3, 0), (5, 3)]).unwrap();
    let lower = f32_tensor(Shape::from_dims([3]), [1.0, 2.0, 3.0]);
    let upper = f32_tensor(Shape::from_dims([5]), [10.0, 20.0, 30.0, 40.0, 50.0]);
    let ids = input_ids(&region);
    let inputs = [
        IndexRegionInput::new(ids[0], &lower),
        IndexRegionInput::new(ids[1], &upper),
    ];

    let baseline = f32_values(
        &evaluator
            .evaluate(&region, IndexRegionAuthority::new(&scalars), &inputs)
            .unwrap()
            .into_outputs()[0],
    );
    // Compared as bits, which is the comparison this oracle exists to make: a
    // margin would admit exactly the drift the equalities below catch.
    assert!(
        baseline
            .windows(2)
            .any(|pair| pair[0].to_bits() != pair[1].to_bits()),
        "a degenerate constant result would satisfy every equality below"
    );

    for span in [1_u64, 2, 3, 5, 64] {
        let mut staged = evaluator
            .stage(&region, IndexRegionAuthority::new(&scalars), &inputs)
            .unwrap();
        while staged
            .evaluate_root_points(span)
            .expect("every span of this width is under the step budget")
            > 0
        {}
        assert!(staged.is_exhausted());
        assert_eq!(
            staged.evaluated_root_points(),
            8,
            "the spans must cover the root points exactly once"
        );
        assert_eq!(
            f32_values(&staged.finish().unwrap().into_outputs()[0]),
            baseline,
            "a span of {span} root points changed a committed value"
        );
    }

    let mut staged = evaluator
        .stage(&region, IndexRegionAuthority::new(&scalars), &inputs)
        .unwrap();
    assert_eq!(
        staged.evaluate_root_points(5),
        Ok(5),
        "a span of five ends two points into the second root"
    );
    assert!(!staged.is_exhausted());
    assert_eq!(staged.evaluate_root_points(5), Ok(3));
    assert!(staged.is_exhausted());
    assert_eq!(staged.evaluate_root_points(5), Ok(0));
}

/// Builds `out[i] = source[coordinate(i)]` over one input of `extent` elements.
fn gather_region(
    scalars: &FrozenScalarRegistry,
    points: u64,
    extent: u64,
    coordinate: impl FnOnce(
        &mut IndexRegionBuilder,
        DimensionId,
        IndexExprId,
    ) -> Result<IndexExprId, Box<dyn std::error::Error>>,
) -> Result<VerifiedIndexRegion, Box<dyn std::error::Error>> {
    let mut builder = IndexRegionBuilder::new(scalars.clone())?;
    let i = builder.dimension(DomainRole::Parallel, Extent::new(points))?;
    let source = builder.tensor(TensorRole::Input, f32_type(), Shape::from_dims([extent]))?;
    let out = builder.tensor(TensorRole::Output, f32_type(), Shape::from_dims([points]))?;
    let index = builder.dimension_expr(i)?;
    let selected = coordinate(&mut builder, i, index)?;
    let value: ScalarValueId = builder.read(source, &[i], &[selected])?;
    let write = builder.write(out, &[i], &[index])?;
    builder.output(write, value)?;
    Ok(builder.build()?)
}

fn gather(
    region: &VerifiedIndexRegion,
    scalars: &FrozenScalarRegistry,
    source: &Tensor,
) -> Vec<f32> {
    let ids = input_ids(region);
    f32_values(
        &evaluator(scalars)
            .evaluate(
                region,
                IndexRegionAuthority::new(scalars),
                &[IndexRegionInput::new(ids[0], source)],
            )
            .unwrap()
            .into_outputs()[0],
    )
}

#[test]
fn scaled_and_quasi_affine_coordinates_resolve_through_exact_index_arithmetic() {
    let scalars = scalar_registry(1);
    let source = f32_tensor(Shape::from_dims([7]), [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);

    let scaled = gather_region(&scalars, 3, 7, |builder, _, index| {
        Ok(builder.linear_combination(
            IndexInteger::from_i128(1),
            &[(IndexInteger::from_i128(2), index)],
        )?)
    })
    .unwrap();
    assert_eq!(gather(&scaled, &scalars, &source), [1.0, 3.0, 5.0]);

    let folded = gather_region(&scalars, 6, 7, |builder, _, index| {
        let three = SourcedExtent::Static(Extent::new(3));
        let quotient = builder.floor_div(index, three.clone())?;
        let remainder = builder.modulo(index, three)?;
        Ok(builder.linear_combination(
            IndexInteger::from_i128(0),
            &[
                (IndexInteger::from_i128(2), remainder),
                (IndexInteger::from_i128(1), quotient),
            ],
        )?)
    })
    .unwrap();
    assert_eq!(
        gather(&folded, &scalars, &source),
        [0.0, 2.0, 4.0, 1.0, 3.0, 5.0],
        "floor division and modulo transpose a 2x3 view of the flat source"
    );
}

/// This oracle declines a semi-affine coordinate instead of resolving it.
///
/// **The refusal is deliberate even though the value is derivable.** The
/// environment below pins `d == 2`, so the oracle could ask the region's shape
/// environment what the divisor resolves to and evaluate the quotient exactly.
/// It must not: this evaluator is the correctness oracle other results are
/// compared against, and a value it derived from a second authority would be a
/// value the comparison could no longer distinguish from the subject's own
/// derivation of it. ADR 0046 admits exactly this conservative decline.
///
/// The neighbour is the same region with the divisor written as a literal,
/// which evaluates — so the refusal is about the divisor's *form* and not about
/// floor division or about the region failing to verify.
#[test]
fn a_semi_affine_divisor_is_declined_rather_than_resolved() {
    let scalars = scalar_registry(1);
    let source = f32_tensor(Shape::from_dims([6]), [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);

    let divisor = ShapeSymbol::new(SymbolScope::new("region/0").unwrap(), "d").unwrap();
    let mut draft = ShapeEnvBuilder::new();
    draft.declare(divisor.clone()).unwrap();
    draft
        .bind(
            &divisor,
            RootBinding::new(
                BindingSource::InterfaceParameter {
                    key: InterfaceParameterKey::new("d").unwrap(),
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    draft
        .require(SemanticInputConstraint::new(
            ExtentRelation::interval(ExtentTerm::Symbol(divisor.clone()), 2, 2).unwrap(),
            FactProvenance::FrontendRequired,
        ))
        .unwrap();
    let environment = Arc::new(draft.build().unwrap());

    let build = |divisor: SourcedExtent| -> VerifiedIndexRegion {
        let mut builder = IndexRegionBuilder::new_with_shape_environment(
            scalars.clone(),
            Arc::clone(&environment),
        )
        .unwrap();
        let i = builder
            .dimension(DomainRole::Parallel, Extent::new(6))
            .unwrap();
        let input = builder
            .tensor(TensorRole::Input, f32_type(), Shape::from_dims([6]))
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, f32_type(), Shape::from_dims([6]))
            .unwrap();
        let index = builder.dimension_expr(i).unwrap();
        let quotient = builder.floor_div(index, divisor).unwrap();
        let value = builder.read(input, &[i], &[quotient]).unwrap();
        let write = builder.write(output, &[i], &[index]).unwrap();
        builder.output(write, value).unwrap();
        builder.build().unwrap()
    };

    let symbolic = build(SourcedExtent::Symbol(divisor));
    let ids = input_ids(&symbolic);
    assert_eq!(
        evaluator(&scalars)
            .evaluate(
                &symbolic,
                IndexRegionAuthority::new(&scalars),
                &[IndexRegionInput::new(ids[0], &source)],
            )
            .unwrap_err(),
        IndexRegionEvaluationError::Unsupported {
            feature: UnsupportedRegionFeature::SymbolicIndexDivisor,
        },
    );

    let literal = build(SourcedExtent::Static(Extent::new(2)));
    assert_eq!(
        gather(&literal, &scalars, &source),
        [0.0, 0.0, 1.0, 1.0, 2.0, 2.0],
        "the same arithmetic with a literal divisor evaluates exactly",
    );
}

#[test]
fn a_structurally_valid_but_wrong_coordinate_relation_is_distinguished() {
    let scalars = scalar_registry(1);
    let source = f32_tensor(Shape::from_dims([3]), [10.0, 20.0, 30.0]);
    let identity = gather_region(&scalars, 3, 3, |_, _, index| Ok(index)).unwrap();
    let broadcast = gather_region(&scalars, 3, 3, |builder, _, _| {
        Ok(builder.constant(IndexInteger::from_i128(0))?)
    })
    .unwrap();

    assert_ne!(
        identity.canonical_identity(),
        broadcast.canonical_identity()
    );
    assert_eq!(gather(&identity, &scalars, &source), [10.0, 20.0, 30.0]);
    assert_eq!(gather(&broadcast, &scalars, &source), [10.0, 10.0, 10.0]);
}

#[test]
fn reads_and_writes_preserve_exact_element_bits() {
    let scalars = scalar_registry(1);
    let source = f32_tensor(
        Shape::from_dims([3]),
        [
            f32::from_bits(0x7fc0_1234),
            -0.0,
            f32::from_bits(0x0000_0001),
        ],
    );
    let region = gather_region(&scalars, 3, 3, |_, _, index| Ok(index)).unwrap();

    assert_eq!(
        gather(&region, &scalars, &source)
            .into_iter()
            .map(f32::to_bits)
            .collect::<Vec<_>>(),
        [0x7fc0_1234, 0x8000_0000, 0x0000_0001],
        "a read and its write copy exact canonical element bytes"
    );
}

#[test]
fn missing_authority_and_missing_capabilities_fail_closed() {
    let scalars = scalar_registry(1);
    let region = matvec_region(&scalars, 2, 2).unwrap();
    let left = f32_tensor(Shape::from_dims([2, 2]), [1.0, 2.0, 3.0, 4.0]);
    let right = f32_tensor(Shape::from_dims([2]), [1.0, 1.0]);
    let ids = input_ids(&region);
    let bindings = [
        IndexRegionInput::new(ids[0], &left),
        IndexRegionInput::new(ids[1], &right),
    ];

    let incomplete = IndexRegionEvaluator::new(
        FrozenReferenceRegistry::standard().unwrap(),
        capabilities(
            &scalars,
            Arc::new(BinaryReference(|left, right| left * right)),
            false,
        ),
    );
    assert!(matches!(
        incomplete.evaluate(
            &region,
            IndexRegionAuthority::new(&scalars),
            &bindings
        ),
        Err(IndexRegionEvaluationError::MissingScalarCapability { operation, .. })
            if *operation == key("add")
    ));

    let evaluator = evaluator(&scalars);
    assert!(matches!(
        evaluator.evaluate(&region, IndexRegionAuthority::new(&scalars), &bindings[..1]),
        Err(IndexRegionEvaluationError::InputCount {
            expected: 2,
            actual: 1
        })
    ));
    let swapped = [
        IndexRegionInput::new(ids[1], &right),
        IndexRegionInput::new(ids[0], &left),
    ];
    assert!(matches!(
        evaluator.evaluate(&region, IndexRegionAuthority::new(&scalars), &swapped),
        Err(IndexRegionEvaluationError::InputBoundary { input_index: 0 })
    ));
    let wrong = f32_tensor(Shape::from_dims([2, 3]), [0.0; 6]);
    assert!(matches!(
        evaluator.evaluate(
            &region,
            IndexRegionAuthority::new(&scalars),
            &[
                IndexRegionInput::new(ids[0], &wrong),
                IndexRegionInput::new(ids[1], &right),
            ]
        ),
        Err(IndexRegionEvaluationError::InputShape { input_index: 0, .. })
    ));

    let foreign = scalar_registry(2);
    assert!(matches!(
        evaluator.evaluate(
            &region,
            IndexRegionAuthority::new(&foreign),
            &bindings
        ),
        Err(IndexRegionEvaluationError::ScalarCapabilityAuthorityMismatch { capability })
            if capability.provider().name() == "f32-scalar-reference"
    ));
}

#[test]
fn callback_failures_retain_exact_capability_attribution() {
    let scalars = scalar_registry(1);
    let region = matvec_region(&scalars, 1, 1).unwrap();
    let left = f32_tensor(Shape::from_dims([1, 1]), [2.0]);
    let right = f32_tensor(Shape::from_dims([1]), [3.0]);
    let ids = input_ids(&region);
    let bindings = [
        IndexRegionInput::new(ids[0], &left),
        IndexRegionInput::new(ids[1], &right),
    ];

    for malformed in [
        Malformed::Failure,
        Malformed::NoResult,
        Malformed::WrongType,
    ] {
        let evaluator = IndexRegionEvaluator::new(
            FrozenReferenceRegistry::standard().unwrap(),
            capabilities(&scalars, Arc::new(MalformedReference(malformed)), true),
        );
        let error = evaluator
            .evaluate(&region, IndexRegionAuthority::new(&scalars), &bindings)
            .unwrap_err();
        match malformed {
            Malformed::Failure => assert!(matches!(
                error,
                IndexRegionEvaluationError::ScalarOperation {
                    capability,
                    source: ReferenceOperationError::InvalidApplication,
                } if *capability.operation() == key("multiply")
                    && capability.revision().get() == 1
            )),
            Malformed::NoResult => assert!(matches!(
                error,
                IndexRegionEvaluationError::ScalarOperation {
                    source: ReferenceOperationError::ResultCount { .. },
                    ..
                }
            )),
            Malformed::WrongType => assert!(matches!(
                error,
                IndexRegionEvaluationError::ScalarResult {
                    result_index: 0,
                    capability,
                } if *capability.operation() == key("multiply")
            )),
        }
    }
}

#[test]
fn scalar_reference_identity_is_deterministic_and_authority_complete() {
    let scalars = scalar_registry(1);
    assert_eq!(
        standard_capabilities(&scalars).canonical_identity(),
        standard_capabilities(&scalars).canonical_identity()
    );
    assert_eq!(
        standard_capabilities(&scalars)
            .scalar_registry()
            .snapshot_identity(),
        scalars.snapshot_identity()
    );

    let readmitted = scalar_registry(2);
    assert_ne!(
        standard_capabilities(&scalars).canonical_identity(),
        standard_capabilities(&readmitted).canonical_identity(),
        "a different admitting scalar provider changes capability provenance"
    );
    assert_ne!(
        standard_capabilities(&scalars).canonical_identity(),
        capabilities(
            &scalars,
            Arc::new(BinaryReference(|left, right| left * right)),
            false,
        )
        .canonical_identity()
    );
}

/// Conformance proof for the governed standard scalar reference profile.
///
/// The cases above drive the oracle with a deliberately *external* scalar
/// vocabulary, which is what proves the registration boundary. These drive it
/// with Tiler's own governed one, which is what proves the numerical contract:
/// `FrozenScalarReferenceRegistry::standard()` makes the regions the governed
/// `f32` index-access lowerings emit executable, so a lowering that is proved to
/// *realize* an occurrence structurally can finally be checked to *compute* it.
///
/// Each region here is a hand-written mirror of what `tiler_compiler::governed`
/// emits for that family. The mirror is necessary rather than preferred:
/// `tiler-reference` is a dependency of `tiler-compiler` and cannot import it,
/// and inverting that edge would put the reference oracle downstream of the
/// compiler. So the constructions follow `governed.rs` step for step and say
/// where. `execute-governed-refined-regions-against-the-oracle` closes the
/// residue by running the emitted regions themselves from the compiler crate.
///
/// Vectors are explicit bit patterns rather than round-number floats. A vector
/// built from `f32::NAN` cannot distinguish canonicalizing from propagating,
/// because `f32::NAN` already *is* the canonical arithmetic payload.
mod governed {
    use std::error::Error;
    use std::sync::Arc;

    use tiler_ir::index::{
        DimensionId, DomainRole, FrozenScalarRegistry, IndexInteger, IndexRegionBuilder,
        ScalarAttributes, ScalarValueId, TensorRole, VerifiedIndexRegion, add_f32_scalar_op,
        canonicalize_nan_f32_scalar_op, constant_f32_scalar_op, multiply_f32_scalar_op,
    };
    use tiler_ir::semantic::{
        CANONICAL_F32_ARITHMETIC_NAN_BITS, CanonicalField, CanonicalValue, F32,
        F32_CONSTANT_BITS_ATTRIBUTE, F32Add, F32Constant, F32Multiply, InputKey, OutputKey,
        ProviderIdentity, SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum, TypeKey,
        Value,
    };
    use tiler_ir::shape::{Axis, Extent, Shape};
    use tiler_reference::{
        FloatBitOrder, FrozenReferenceRegistry, FrozenScalarReferenceRegistry,
        IndexRegionAuthority, IndexRegionEvaluator, IndexRegionInput, InputBinding,
        ReferenceCapabilityRevision, ReferenceElement, ReferenceEvaluator, ReferenceOperationError,
        ReferenceSignature, ScalarReferenceOperation, ScalarReferenceOutputs,
        ScalarReferenceRegistryBuilder, ScalarReferenceRequest, Tensor, TensorPayloadView,
    };

    use super::{f32_type, input_ids};

    /// A quiet NaN whose payload is *not* the governed canonical arithmetic one.
    const NONCANONICAL_QUIET_NAN: u32 = 0x7fc0_1234;
    /// A negative quiet NaN, differing from the canonical payload in its sign.
    const NEGATIVE_QUIET_NAN: u32 = 0xffc0_0000;
    /// A signalling NaN, which no arithmetic result may reproduce verbatim.
    const SIGNALLING_NAN: u32 = 0x7f80_0001;
    /// The smallest positive subnormal binary32.
    const LEAST_SUBNORMAL: u32 = 0x0000_0001;

    fn governed_scalars() -> FrozenScalarRegistry {
        FrozenScalarRegistry::standard().expect("the governed scalar authority composes")
    }

    fn governed_evaluator() -> IndexRegionEvaluator {
        IndexRegionEvaluator::new(
            FrozenReferenceRegistry::standard().expect("the governed value profile composes"),
            FrozenScalarReferenceRegistry::standard()
                .expect("the governed scalar reference profile composes"),
        )
    }

    /// Builds one element from exact bits, never routing them through an `f32`.
    fn bit_element(bits: u32) -> ReferenceElement {
        ReferenceElement::from_float_bits(
            bits.to_be_bytes(),
            FloatBitOrder::MostSignificantByteFirst,
        )
        .expect("binary32 payloads are bounded")
    }

    fn bit_tensor(shape: Shape, bits: &[u32]) -> Tensor {
        Tensor::dense(
            f32_type(),
            shape,
            bits.iter().copied().map(bit_element).collect(),
        )
        .expect("the fixture tensor is well formed")
    }

    fn tensor_bits(tensor: &Tensor) -> Vec<u32> {
        let TensorPayloadView::Dense(elements) = tensor.payload() else {
            panic!("expected a dense f32 tensor")
        };
        elements
            .iter()
            .map(|value| {
                u32::from_be_bytes(
                    <[u8; 4]>::try_from(value.as_bytes())
                        .expect("binary32 elements are four bytes"),
                )
            })
            .collect()
    }

    /// Executes one region through the governed standard scalar oracle.
    fn evaluate_region(region: &VerifiedIndexRegion, inputs: &[&Tensor]) -> Vec<u32> {
        let ids = input_ids(region);
        assert_eq!(ids.len(), inputs.len(), "one binding per input boundary");
        let bindings: Vec<_> = ids
            .iter()
            .zip(inputs)
            .map(|(id, tensor)| IndexRegionInput::new(*id, tensor))
            .collect();
        let scalars = governed_scalars();
        let evaluation = governed_evaluator()
            .evaluate(region, IndexRegionAuthority::new(&scalars), &bindings)
            .expect("the governed region evaluates");
        assert_eq!(evaluation.outputs().len(), 1, "one output root");
        tensor_bits(&evaluation.outputs()[0])
    }

    /// Executes one semantic program through the tensor-level oracle.
    fn evaluate_program(
        program: &SemanticProgram,
        input: Option<(&InputKey, &Tensor)>,
    ) -> Vec<u32> {
        let bindings: Vec<_> = input
            .into_iter()
            .map(|(key, tensor)| InputBinding::new(key, tensor))
            .collect();
        let outputs = ReferenceEvaluator::standard()
            .expect("the governed reference profile composes")
            .evaluate(program, &bindings)
            .expect("the governed program evaluates");
        assert_eq!(outputs.len(), 1, "one program output");
        tensor_bits(&outputs[0])
    }

    // Region mirrors of the governed index-access lowerings.

    fn governed_constant_attributes(bits: u32) -> ScalarAttributes {
        ScalarAttributes::new(
            CanonicalValue::record([CanonicalField::new(
                F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValue::float_bits(
                    TypeKey::new("tiler", "f32", 1).expect("the governed f32 format key is valid"),
                    bits.to_be_bytes(),
                )
                .expect("binary32 payloads are bounded"),
            )])
            .expect("the attribute record is canonical"),
        )
        .expect("the scalar attributes match the governed schema")
    }

    /// Mirrors `governed::GovernedConstantF32`: a rank-zero apply and write.
    fn constant_region(bits: u32) -> Result<VerifiedIndexRegion, Box<dyn Error>> {
        let mut builder = IndexRegionBuilder::new(governed_scalars())?;
        let output = builder.tensor(TensorRole::Output, f32_type(), Shape::new([]))?;
        let value = builder
            .apply(
                constant_f32_scalar_op(),
                governed_constant_attributes(bits),
                &[],
            )?
            .get(0)
            .ok_or("the governed constant produces one result")?;
        let write = builder.write(output, &[], &[])?;
        builder.output(write, value)?;
        Ok(builder.build()?)
    }

    /// Mirrors `governed::GovernedPointwiseF32` with a rank-zero right operand.
    ///
    /// The occurrence is `out[i] = left[i] <op> right`, which is the one
    /// broadcast form the governed provider supports.
    fn pointwise_broadcast_region(
        multiply: bool,
        length: u64,
    ) -> Result<VerifiedIndexRegion, Box<dyn Error>> {
        let mut builder = IndexRegionBuilder::new(governed_scalars())?;
        let shape = Shape::from_dims([length]);
        let i = builder.dimension(DomainRole::Parallel, Extent::new(length))?;
        let coordinate = builder.dimension_expr(i)?;
        let left = builder.tensor(TensorRole::Input, f32_type(), shape.clone())?;
        let right = builder.tensor(TensorRole::Input, f32_type(), Shape::new([]))?;
        let output = builder.tensor(TensorRole::Output, f32_type(), shape)?;
        let left_value = builder.read(left, &[i], &[coordinate])?;
        let right_value = builder.read(right, &[], &[])?;
        let scalar = if multiply {
            multiply_f32_scalar_op()
        } else {
            add_f32_scalar_op()
        };
        let applied = builder
            .apply(
                scalar,
                ScalarAttributes::empty(),
                &[left_value, right_value],
            )?
            .get(0)
            .ok_or("the governed pointwise scalar produces one result")?;
        let write = builder.write(output, &[i], &[coordinate])?;
        builder.output(write, applied)?;
        Ok(builder.build()?)
    }

    /// Folds the bound contributors onto `seed` with the governed reducer body.
    fn fold_contributors(
        builder: &mut IndexRegionBuilder,
        bound: DimensionId,
        contributor: ScalarValueId,
        seed: ScalarValueId,
    ) -> Result<ScalarValueId, Box<dyn Error>> {
        Ok(builder
            .reduce(&[bound], &[seed], &[contributor], |body| {
                let accumulated = body.apply(
                    add_f32_scalar_op(),
                    ScalarAttributes::empty(),
                    &[
                        body.state(0).expect("one state parameter"),
                        body.contributor(0).expect("one contributor parameter"),
                    ],
                )?;
                body.yield_values(&[accumulated.get(0).expect("the governed add has one result")])
            })?
            .get(0)
            .ok_or("the reduction produces one result")?)
    }

    /// Mirrors `governed::GovernedStrictSerialSumF32` for a rank-one input.
    ///
    /// The kept domain is empty, the reduced sub-shape is the whole input, and
    /// the linearized reduced offset is the contributor coordinate directly. So
    /// the emission reduces to exactly what `SumPlan` produces for `[length]`
    /// with axis zero reduced: a seed read at offset zero, then a fold over
    /// `length - 1` contributors read at `tail + 1`.
    fn serial_sum_region(length: u64) -> Result<VerifiedIndexRegion, Box<dyn Error>> {
        let mut builder = IndexRegionBuilder::new(governed_scalars())?;
        let input = builder.tensor(TensorRole::Input, f32_type(), Shape::from_dims([length]))?;
        let output = builder.tensor(TensorRole::Output, f32_type(), Shape::new([]))?;

        let total = if length == 0 {
            // `SumPlan::fold_empty`: the operand is still read over the vacuous
            // reduced domain, and the fold yields the `+0.0` identity.
            let reduced = builder.dimension(DomainRole::Reduction, Extent::new(0))?;
            let coordinate = builder.dimension_expr(reduced)?;
            let contributor = builder.read(input, &[reduced], &[coordinate])?;
            let identity = builder
                .apply(
                    constant_f32_scalar_op(),
                    governed_constant_attributes(0.0_f32.to_bits()),
                    &[],
                )?
                .get(0)
                .ok_or("the governed constant produces one result")?;
            fold_contributors(&mut builder, reduced, contributor, identity)?
        } else {
            let zero = builder.constant(IndexInteger::from_u64(0))?;
            let seed = builder.read(input, &[], &[zero])?;
            if length == 1 {
                // `GovernedStrictSerialSumF32` canonicalizes the lone
                // contributor at the reduction's result boundary. No combine has
                // run, so this is the only path on which the boundary rule has
                // work to do, and it is a conversion rather than an addition
                // because adding the `+0.0` identity would turn an observable
                // `-0.0` into `+0.0`.
                builder
                    .apply(
                        canonicalize_nan_f32_scalar_op(),
                        ScalarAttributes::empty(),
                        &[seed],
                    )?
                    .get(0)
                    .ok_or("the governed canonicalization produces one result")?
            } else {
                let tail = builder.dimension(DomainRole::Reduction, Extent::new(length - 1))?;
                let induction = builder.dimension_expr(tail)?;
                let one = IndexInteger::from_u64(1);
                let offset = builder.linear_combination(one.clone(), &[(one, induction)])?;
                let contributor = builder.read(input, &[tail], &[offset])?;
                fold_contributors(&mut builder, tail, contributor, seed)?
            }
        };
        let write = builder.write(output, &[], &[])?;
        builder.output(write, total)?;
        Ok(builder.build()?)
    }

    // Semantic programs for the same occurrences.

    fn constant_program(bits: u32) -> SemanticProgram {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the governed profile composes");
        let value = F32Constant::apply(&mut builder, bits).expect("the constant applies");
        builder
            .output(
                OutputKey::new("result").expect("the output key is valid"),
                value,
            )
            .expect("the output binds");
        builder.build().expect("the program verifies")
    }

    fn pointwise_program(
        multiply: bool,
        length: u64,
        right_bits: u32,
    ) -> (SemanticProgram, InputKey) {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the governed profile composes");
        let key = InputKey::new("input").expect("the input key is valid");
        let left = builder
            .input::<F32>(key.clone(), Shape::from_dims([length]))
            .expect("the input binds");
        let right = F32Constant::apply(&mut builder, right_bits).expect("the constant applies");
        let applied = if multiply {
            F32Multiply::apply(&mut builder, left, right)
        } else {
            F32Add::apply(&mut builder, left, right)
        }
        .expect("the pointwise operation applies");
        builder
            .output(
                OutputKey::new("result").expect("the output key is valid"),
                applied,
            )
            .expect("the output binds");
        (builder.build().expect("the program verifies"), key)
    }

    fn serial_sum_program(length: u64) -> (SemanticProgram, InputKey) {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the governed profile composes");
        let key = InputKey::new("input").expect("the input key is valid");
        let input: Value<F32> = builder
            .input::<F32>(key.clone(), Shape::from_dims([length]))
            .expect("the input binds");
        let total = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(0)])
            .expect("the sum applies");
        builder
            .output(
                OutputKey::new("result").expect("the output key is valid"),
                total,
            )
            .expect("the output binds");
        (builder.build().expect("the program verifies"), key)
    }

    // Question 1 — does the governed scalar arithmetic canonicalize a NaN result?

    /// The governed scalar `add`/`multiply` canonicalize every arithmetic NaN.
    ///
    /// The tensor-level `tiler::add-f32@1` and `tiler::multiply-f32@1` oracles
    /// do, because both operations carry `CANONICAL_F32_ARITHMETIC_NAN_BITS` as
    /// a declared operation fact. A scalar oracle that propagated the host
    /// payload instead would make the refined region and the semantic evaluator
    /// disagree on the very programs refinement exists to check.
    #[test]
    fn governed_scalar_arithmetic_canonicalizes_every_nan_result() {
        for (left, right) in [
            (NONCANONICAL_QUIET_NAN, 1.0_f32.to_bits()),
            (1.0_f32.to_bits(), NEGATIVE_QUIET_NAN),
            (SIGNALLING_NAN, SIGNALLING_NAN),
            (NEGATIVE_QUIET_NAN, NONCANONICAL_QUIET_NAN),
            // NaN produced by the operation rather than propagated from an operand.
            (f32::INFINITY.to_bits(), f32::NEG_INFINITY.to_bits()),
            (f32::INFINITY.to_bits(), 0.0_f32.to_bits()),
        ] {
            for multiply in [false, true] {
                let region = pointwise_broadcast_region(multiply, 1).expect("the region verifies");
                let left_tensor = bit_tensor(Shape::from_dims([1]), &[left]);
                let right_tensor = bit_tensor(Shape::new([]), &[right]);
                let actual = evaluate_region(&region, &[&left_tensor, &right_tensor]);
                // `INFINITY + NEG_INFINITY` is NaN but `INFINITY * NEG_INFINITY`
                // is not, so assert canonicality only where the result is NaN.
                if f32::from_bits(actual[0]).is_nan() {
                    assert_eq!(
                        actual,
                        [CANONICAL_F32_ARITHMETIC_NAN_BITS],
                        "multiply={multiply} left={left:#010x} right={right:#010x}"
                    );
                }

                let (program, key) = pointwise_program(multiply, 1, right);
                assert_eq!(
                    actual,
                    evaluate_program(&program, Some((&key, &left_tensor))),
                    "the scalar and tensor oracles must agree: \
                     multiply={multiply} left={left:#010x} right={right:#010x}"
                );
            }
        }
    }

    /// Non-NaN results keep their exact payload, including the sign of a zero.
    ///
    /// Canonicalization applies to an arithmetic NaN and to nothing else. A
    /// scalar oracle that normalized more than that would silently erase
    /// `-0.0`, which the governed serial sum's seeding rule exists to preserve.
    #[test]
    fn governed_scalar_arithmetic_preserves_every_non_nan_payload() {
        for (multiply, left, right, expected) in [
            (
                false,
                (-0.0_f32).to_bits(),
                0.0_f32.to_bits(),
                0.0_f32.to_bits(),
            ),
            (
                false,
                (-0.0_f32).to_bits(),
                (-0.0_f32).to_bits(),
                (-0.0_f32).to_bits(),
            ),
            (
                true,
                0.0_f32.to_bits(),
                (-1.0_f32).to_bits(),
                (-0.0_f32).to_bits(),
            ),
            (true, LEAST_SUBNORMAL, 1.0_f32.to_bits(), LEAST_SUBNORMAL),
            (false, LEAST_SUBNORMAL, LEAST_SUBNORMAL, 0x0000_0002),
            (
                true,
                f32::INFINITY.to_bits(),
                f32::NEG_INFINITY.to_bits(),
                f32::NEG_INFINITY.to_bits(),
            ),
        ] {
            let region = pointwise_broadcast_region(multiply, 1).expect("the region verifies");
            let left_tensor = bit_tensor(Shape::from_dims([1]), &[left]);
            let right_tensor = bit_tensor(Shape::new([]), &[right]);
            assert_eq!(
                evaluate_region(&region, &[&left_tensor, &right_tensor]),
                [expected],
                "multiply={multiply} left={left:#010x} right={right:#010x}"
            );

            let (program, key) = pointwise_program(multiply, 1, right);
            assert_eq!(
                evaluate_program(&program, Some((&key, &left_tensor))),
                [expected],
                "the tensor oracle must agree: multiply={multiply}"
            );
        }
    }

    /// The governed scalar constant reproduces its payload verbatim.
    ///
    /// A constant performs no arithmetic, so the canonical-NaN fact does not
    /// reach it. Canonicalizing here would make it impossible for a region to
    /// materialize an exact binary32 pattern, which is precisely what
    /// `tiler::constant-f32@1` promises ("exact IEEE-754 payload").
    #[test]
    fn the_governed_scalar_constant_reproduces_an_exact_payload() {
        for bits in [
            NONCANONICAL_QUIET_NAN,
            NEGATIVE_QUIET_NAN,
            SIGNALLING_NAN,
            (-0.0_f32).to_bits(),
            LEAST_SUBNORMAL,
            CANONICAL_F32_ARITHMETIC_NAN_BITS,
        ] {
            let region = constant_region(bits).expect("the region verifies");
            assert_eq!(
                evaluate_region(&region, &[]),
                [bits],
                "the scalar constant must not canonicalize {bits:#010x}"
            );
            assert_eq!(
                evaluate_program(&constant_program(bits), None),
                [bits],
                "the tensor constant must not canonicalize {bits:#010x}"
            );
        }
    }

    // Question 2 — does the seed-with-first-contributor fold match `strict_sum`?

    /// Vectors that separate a first-contributor seed from a `+0.0` one.
    fn seeding_vectors() -> Vec<Vec<u32>> {
        vec![
            // The vector `structured_fused_body_interpreter_matches_reference_evaluator`
            // already uses, which mixes a subnormal with both signed zeros.
            vec![
                1.0_f32.to_bits(),
                (-2.0_f32).to_bits(),
                3.5_f32.to_bits(),
                f32::MIN_POSITIVE.to_bits(),
                (-0.0_f32).to_bits(),
                0.0_f32.to_bits(),
            ],
            // Sign of zero: only `-0.0 + -0.0` stays negative, so a `+0.0` seed
            // would turn the first of these into `+0.0` and leave the rest alone.
            vec![(-0.0_f32).to_bits(); 2],
            vec![(-0.0_f32).to_bits(), 0.0_f32.to_bits()],
            vec![0.0_f32.to_bits(), (-0.0_f32).to_bits()],
            vec![(-0.0_f32).to_bits(); 5],
            // NaN ordering: the payload that survives must not depend on which
            // operand the host propagates, nor on where the NaN sits in the fold.
            vec![NONCANONICAL_QUIET_NAN, 1.0_f32.to_bits()],
            vec![1.0_f32.to_bits(), NEGATIVE_QUIET_NAN],
            vec![1.0_f32.to_bits(), SIGNALLING_NAN, 2.0_f32.to_bits()],
            vec![NEGATIVE_QUIET_NAN, NONCANONICAL_QUIET_NAN],
            vec![f32::INFINITY.to_bits(), f32::NEG_INFINITY.to_bits()],
            // Order sensitivity: these sums differ under any reassociation, so
            // an agreeing result is evidence the contributor order agrees too.
            vec![1.0_f32.to_bits(), 1e-30_f32.to_bits(), (-1.0_f32).to_bits()],
            vec![1e-30_f32.to_bits(), 1.0_f32.to_bits(), (-1.0_f32).to_bits()],
            vec![
                1e30_f32.to_bits(),
                1e30_f32.to_bits(),
                (-1e30_f32).to_bits(),
                (-1e30_f32).to_bits(),
            ],
            vec![LEAST_SUBNORMAL; 4],
        ]
    }

    /// The governed fold reproduces `strict_sum` exactly for every fold with work.
    ///
    /// This is the load-bearing claim: the governed lowering seeds with the
    /// first contributor rather than with a `+0.0` identity, and the normative
    /// oracle seeds the same way, so the two agree on sign-of-zero and NaN
    /// vectors that a `+0.0`-seeded fold would get wrong.
    #[test]
    fn the_governed_serial_fold_matches_the_semantic_oracle_bit_for_bit() {
        for values in seeding_vectors() {
            let length = u64::try_from(values.len()).expect("the fixture is small");
            assert!(
                length >= 2,
                "this case covers folds that perform arithmetic"
            );
            let region = serial_sum_region(length).expect("the region verifies");
            let input = bit_tensor(Shape::from_dims([length]), &values);
            let (program, key) = serial_sum_program(length);
            assert_eq!(
                evaluate_region(&region, &[&input]),
                evaluate_program(&program, Some((&key, &input))),
                "the governed fold and the semantic oracle disagree on {values:#010x?}"
            );
        }
    }

    /// A `+0.0`-seeded fold really would be observably wrong on these vectors.
    ///
    /// Without this the agreement above proves only that two implementations
    /// match, not that the seeding rule matters. `-0.0 + -0.0` is `-0.0` while
    /// `0.0 + (-0.0)` is `+0.0`, so the identity seed loses the sign.
    #[test]
    fn a_positive_zero_seed_would_change_the_observable_result() {
        let values = vec![(-0.0_f32).to_bits(); 3];
        let length = u64::try_from(values.len()).expect("the fixture is small");
        let input = bit_tensor(Shape::from_dims([length]), &values);
        let (program, key) = serial_sum_program(length);
        assert_eq!(
            evaluate_program(&program, Some((&key, &input))),
            [(-0.0_f32).to_bits()],
            "the strict serial sum of negative zeros is negative zero"
        );

        let seeded: f32 = values
            .iter()
            .fold(0.0_f32, |total, bits| total + f32::from_bits(*bits));
        assert_eq!(
            seeded.to_bits(),
            0.0_f32.to_bits(),
            "a +0.0 seed loses the sign, which is the error this rule prevents"
        );
    }

    /// An empty reduced domain yields the `+0.0` identity in both oracles.
    #[test]
    fn an_empty_reduced_domain_is_positive_zero_in_both_oracles() {
        let region = serial_sum_region(0).expect("the region verifies");
        let input = bit_tensor(Shape::from_dims([0]), &[]);
        let (program, key) = serial_sum_program(0);
        assert_eq!(evaluate_region(&region, &[&input]), [0.0_f32.to_bits()]);
        assert_eq!(
            evaluate_region(&region, &[&input]),
            evaluate_program(&program, Some((&key, &input)))
        );
    }

    /// A single contributor agrees on every payload but a non-canonical NaN.
    #[test]
    fn a_single_contributor_agrees_on_every_payload_but_a_non_canonical_nan() {
        for bits in [
            (-0.0_f32).to_bits(),
            0.0_f32.to_bits(),
            f32::INFINITY.to_bits(),
            f32::NEG_INFINITY.to_bits(),
            LEAST_SUBNORMAL,
            1.0_f32.to_bits(),
            CANONICAL_F32_ARITHMETIC_NAN_BITS,
        ] {
            let region = serial_sum_region(1).expect("the region verifies");
            let input = bit_tensor(Shape::from_dims([1]), &[bits]);
            let (program, key) = serial_sum_program(1);
            assert_eq!(
                evaluate_region(&region, &[&input]),
                evaluate_program(&program, Some((&key, &input))),
                "the oracles disagree on a lone {bits:#010x}"
            );
            assert_eq!(evaluate_region(&region, &[&input]), [bits]);
        }
    }

    /// A lone non-canonical NaN contributor is canonicalized by both oracles.
    ///
    /// This case was a pinned *divergence*: `strict_sum` canonicalized its
    /// accumulator before writing it, so a fold performing zero additions still
    /// reported the canonical payload, while the governed lowering and
    /// `tiler_ir::kernel::lower` both wrote the seed unchanged. Two of the three
    /// implementations agreed with each other and disagreed with the normative
    /// oracle.
    ///
    /// The contract decides which side was wrong, and it is not the oracle.
    /// `docs/numerical-semantics.md` requires strict `f32` Sum to apply the
    /// canonicalization "at its result boundary even when the contributor
    /// sequence is a singleton", and states the reason: "The redundant
    /// result-boundary rule prevents an uncombined input payload from leaking
    /// through an arithmetic reduction." ADR 0055 says the same, "including
    /// singleton results". A lone contributor is precisely an uncombined input
    /// payload, so the two lowerings were realizing a rule the contract does not
    /// state, and the oracle was already correct.
    ///
    /// Both now emit `tiler.scalar::canonicalize-nan-f32@1` on that path — a
    /// conversion rather than an addition, because adding the `+0.0` identity
    /// would turn an observable `-0.0` into `+0.0`, which
    /// `a_single_contributor_agrees_on_every_payload_but_a_non_canonical_nan`
    /// pins in the other direction.
    #[test]
    fn a_lone_non_canonical_nan_contributor_canonicalizes_in_both_oracles() {
        for bits in [NONCANONICAL_QUIET_NAN, NEGATIVE_QUIET_NAN, SIGNALLING_NAN] {
            let region = serial_sum_region(1).expect("the region verifies");
            let input = bit_tensor(Shape::from_dims([1]), &[bits]);
            let (program, key) = serial_sum_program(1);

            assert_eq!(
                evaluate_region(&region, &[&input]),
                [CANONICAL_F32_ARITHMETIC_NAN_BITS],
                "the governed region canonicalizes a lone {bits:#010x} at its result boundary"
            );
            assert_eq!(
                evaluate_program(&program, Some((&key, &input))),
                [CANONICAL_F32_ARITHMETIC_NAN_BITS],
                "the semantic oracle canonicalizes a zero-step fold"
            );
            assert_eq!(
                evaluate_region(&region, &[&input]),
                evaluate_program(&program, Some((&key, &input))),
                "the two oracles agree on a lone {bits:#010x}"
            );
        }
    }

    // Registry identity and authority binding.

    /// The governed scalar oracle is bound to the governed scalar authority.
    #[test]
    fn the_standard_scalar_oracle_binds_the_governed_scalar_authority() {
        let registry =
            FrozenScalarReferenceRegistry::standard().expect("the governed scalar oracle composes");
        assert_eq!(
            registry.scalar_registry().snapshot_identity(),
            governed_scalars().snapshot_identity(),
            "the oracle and the lowerings must share one scalar snapshot"
        );
        assert_eq!(
            registry.canonical_identity(),
            FrozenScalarReferenceRegistry::standard()
                .expect("the governed scalar oracle composes")
                .canonical_identity(),
            "the shared snapshot is deterministic"
        );
    }

    struct RejectingScalar;

    impl ScalarReferenceOperation for RejectingScalar {
        fn evaluate(
            &self,
            _: ScalarReferenceRequest<'_>,
            _: &mut ScalarReferenceOutputs,
        ) -> Result<(), ReferenceOperationError> {
            Err(ReferenceOperationError::InvalidApplication)
        }
    }

    /// The governed profile owns its three keys and refuses to be shadowed.
    ///
    /// An extension composes on the returned builder by *adding* capabilities;
    /// it cannot quietly substitute a different oracle for a governed scalar,
    /// which is what would let a region be checked against arithmetic the
    /// governed contract never admitted.
    #[test]
    fn a_governed_scalar_capability_cannot_be_shadowed_by_an_extension() {
        let mut builder = ScalarReferenceRegistryBuilder::standard()
            .expect("the governed scalar reference profile composes");
        let shadow = builder.register(
            ProviderIdentity::new("example", "shadow", 1).expect("the provider identity is valid"),
            add_f32_scalar_op(),
            ReferenceSignature::new([f32_type(), f32_type()], [f32_type()])
                .expect("the signature is bounded"),
            ReferenceCapabilityRevision::new(1).expect("the revision is nonzero"),
            Arc::new(RejectingScalar),
        );
        assert!(
            matches!(
                shadow,
                Err(tiler_reference::ScalarReferenceRegistryError::DuplicateCapability { .. })
            ),
            "the governed profile already owns tiler.scalar::add-f32@1"
        );
    }
}
