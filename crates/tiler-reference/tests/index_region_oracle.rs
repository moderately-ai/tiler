//! Public-API integration proof for the verified index-region reference oracle.
//!
//! These end-to-end cases drive the oracle exclusively through the crate's
//! re-exported public surface, so they live in an admitted integration target
//! rather than inside the module they exercise.

use std::sync::Arc;

use tiler_ir::index::{
    DimensionId, DomainRole, FrozenScalarRegistry, IndexExprId, IndexInteger, IndexRegionBuilder,
    ScalarArity, ScalarAttributeField, ScalarAttributeSchema, ScalarAttributes, ScalarEffect,
    ScalarInferenceError, ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey,
    ScalarOperationContract, ScalarOperationDefinition, ScalarOperationInferencer,
    ScalarRegistryBuilder, ScalarValueId, TensorRole, VerifiedIndexRegion, VerifiedTensorId,
};
use tiler_ir::semantic::{
    AttributeFieldId, CANONICAL_F32_ARITHMETIC_NAN_BITS, CanonicalField, CanonicalValue,
    CanonicalValueKind, CanonicalValueView, F32, FrozenSemanticRegistry, NormativeDefinitionRef,
    ProviderDiagnosticCode, ProviderIdentity, ResolvedValueType, TypeKey,
};
use tiler_ir::shape::{Extent, Shape};
use tiler_reference::{
    FloatBitOrder, FrozenReferenceRegistry, FrozenScalarReferenceRegistry, IndexRegionAuthority,
    IndexRegionEvaluationError, IndexRegionEvaluator, IndexRegionInput,
    ReferenceCapabilityRevision, ReferenceElement, ReferenceOperationError, ReferenceSignature,
    ScalarReferenceOperation, ScalarReferenceOutputs, ScalarReferenceRegistryBuilder,
    ScalarReferenceRequest, Tensor, TensorPayloadView,
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

fn semantic_authority() -> FrozenSemanticRegistry {
    FrozenSemanticRegistry::standard().unwrap()
}

#[test]
fn matvec_region_evaluates_through_registered_scalar_capabilities() {
    let scalars = scalar_registry(1);
    let region = matvec_region(&scalars, 3, 4).unwrap();
    let semantic = semantic_authority();
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
        .evaluate(
            &region,
            IndexRegionAuthority::new(&scalars, &semantic),
            &bindings,
        )
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
    let semantic = semantic_authority();
    let evaluator = evaluator(&scalars);

    let region = matvec_region(&scalars, 3, 0).unwrap();
    let left = f32_tensor(Shape::from_dims([3, 0]), []);
    let right = f32_tensor(Shape::from_dims([0]), []);
    let ids = input_ids(&region);
    let outputs = evaluator
        .evaluate(
            &region,
            IndexRegionAuthority::new(&scalars, &semantic),
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
            IndexRegionAuthority::new(&scalars, &semantic),
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
    let semantic = semantic_authority();
    let ids = input_ids(region);
    f32_values(
        &evaluator(scalars)
            .evaluate(
                region,
                IndexRegionAuthority::new(scalars, &semantic),
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
        let quotient = builder.floor_div(index, 3)?;
        let remainder = builder.modulo(index, 3)?;
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
    let semantic = semantic_authority();
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
            IndexRegionAuthority::new(&scalars, &semantic),
            &bindings
        ),
        Err(IndexRegionEvaluationError::MissingScalarCapability { operation, .. })
            if *operation == key("add")
    ));

    let evaluator = evaluator(&scalars);
    assert!(matches!(
        evaluator.evaluate(
            &region,
            IndexRegionAuthority::new(&scalars, &semantic),
            &bindings[..1]
        ),
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
        evaluator.evaluate(
            &region,
            IndexRegionAuthority::new(&scalars, &semantic),
            &swapped
        ),
        Err(IndexRegionEvaluationError::InputBoundary { input_index: 0 })
    ));
    let wrong = f32_tensor(Shape::from_dims([2, 3]), [0.0; 6]);
    assert!(matches!(
        evaluator.evaluate(
            &region,
            IndexRegionAuthority::new(&scalars, &semantic),
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
            IndexRegionAuthority::new(&foreign, &semantic),
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
    let semantic = semantic_authority();
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
            .evaluate(
                &region,
                IndexRegionAuthority::new(&scalars, &semantic),
                &bindings,
            )
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
