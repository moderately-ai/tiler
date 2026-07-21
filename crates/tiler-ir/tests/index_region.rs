//! Public generic scalar-SSA and verified index-region integration tests.

use std::sync::Arc;

use tiler_ir::index::{
    BoundsProofView, DomainRole, IndexBuildError, IndexInteger, IndexRegionBuilder,
    IndexRegionDiagnostic, MAX_TENSOR_RANK, ScalarArity, ScalarAttributeField,
    ScalarAttributeSchema, ScalarAttributes, ScalarEffect, ScalarInferenceError,
    ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract, ScalarOperationDefinition,
    ScalarOperationInferencer, ScalarRegistryBuilder, ScalarRegistryError, TensorRole,
    WriteOwnershipProofView,
};
use tiler_ir::semantic::{
    CanonicalField, CanonicalValue, CanonicalValueKind, FrozenSemanticRegistry,
    NormativeDefinitionRef, ProviderIdentity, RegistryError, ResolvedValueType,
    SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
    TypeDefinitionFacts, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
};
use tiler_ir::shape::{Extent, Shape};

fn record() -> CanonicalValue {
    CanonicalValue::record([]).unwrap()
}
fn test_type() -> ResolvedValueType {
    ResolvedValueType::nominal(TypeKey::new("example", "pixel", 1).unwrap())
}
fn alternate_type() -> ResolvedValueType {
    ResolvedValueType::nominal(TypeKey::new("example", "alternate", 1).unwrap())
}

struct Types;
impl SemanticRegistryProvider for Types {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("example", "types", 1).unwrap()
    }
    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("example", "pixel", 1).unwrap()),
            NormativeDefinitionRef::new("urn:example:pixel:v1").unwrap(),
            TypeDefinitionFacts::new(record()),
        ))?;
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("example", "alternate", 1).unwrap()),
            NormativeDefinitionRef::new("urn:example:alternate:v1").unwrap(),
            TypeDefinitionFacts::new(record()),
        ))
    }
}

#[derive(Clone)]
struct Fixed(Vec<ResolvedValueType>);
impl ScalarOperationInferencer for Fixed {
    fn infer(
        &self,
        _: ScalarInferenceRequest<'_>,
    ) -> Result<Vec<ResolvedValueType>, ScalarInferenceError> {
        Ok(self.0.clone())
    }
}
struct Same;
impl ScalarOperationInferencer for Same {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
    ) -> Result<Vec<ResolvedValueType>, ScalarInferenceError> {
        let Some(first) = request.operands.first() else {
            return Err(ScalarInferenceError::new("arity", "operand required"));
        };
        if request.operands.iter().any(|value| value != first) {
            return Err(ScalarInferenceError::new("type", "operands differ"));
        }
        Ok(vec![first.clone()])
    }
}
struct PairState;
impl ScalarOperationInferencer for PairState {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
    ) -> Result<Vec<ResolvedValueType>, ScalarInferenceError> {
        if request.operands.len() != 4 {
            return Err(ScalarInferenceError::new(
                "arity",
                "four state/contributor operands required",
            ));
        }
        Ok(vec![
            request.operands[0].clone(),
            request.operands[1].clone(),
        ])
    }
}

fn scalar_definition(
    name: &str,
    operands: usize,
    results: usize,
    inferencer: Arc<dyn ScalarOperationInferencer>,
) -> ScalarOperationDefinition {
    ScalarOperationDefinition::new(
        ScalarOpKey::new("example", name, 1).unwrap(),
        NormativeDefinitionRef::new(format!("urn:example:{name}:v1")).unwrap(),
        ScalarOperationContract {
            attributes: ScalarAttributeSchema::empty(),
            operands: ScalarArity::exact(operands).unwrap(),
            results: ScalarArity::exact(results).unwrap(),
            effect: ScalarEffect::Pure,
            facts: record(),
            conformance: record(),
        },
        inferencer,
    )
}

fn registries() -> (
    FrozenSemanticRegistry,
    tiler_ir::index::FrozenScalarRegistry,
) {
    let mut semantic = SemanticRegistryBuilder::new();
    semantic.register_provider(&Types).unwrap();
    let semantic = semantic.freeze().unwrap();
    let mut scalar = ScalarRegistryBuilder::new(semantic.clone());
    let provider = ProviderIdentity::new("example", "scalar", 1).unwrap();
    scalar
        .register(
            provider.clone(),
            scalar_definition("constant", 0, 1, Arc::new(Fixed(vec![test_type()]))),
        )
        .unwrap();
    scalar
        .register(
            provider.clone(),
            scalar_definition("state_step", 4, 2, Arc::new(PairState)),
        )
        .unwrap();
    scalar
        .register(
            provider.clone(),
            scalar_definition("constant_alt_key", 0, 1, Arc::new(Fixed(vec![test_type()]))),
        )
        .unwrap();
    scalar
        .register(
            provider.clone(),
            scalar_definition(
                "constant_alt_type",
                0,
                1,
                Arc::new(Fixed(vec![alternate_type()])),
            ),
        )
        .unwrap();
    scalar
        .register(
            provider.clone(),
            ScalarOperationDefinition::new(
                ScalarOpKey::new("example", "attributed", 1).unwrap(),
                NormativeDefinitionRef::new("urn:example:attributed:v1").unwrap(),
                ScalarOperationContract {
                    attributes: ScalarAttributeSchema::new([ScalarAttributeField::new(
                        7,
                        CanonicalValueKind::Unsigned,
                        true,
                    )])
                    .unwrap(),
                    operands: ScalarArity::exact(0).unwrap(),
                    results: ScalarArity::exact(1).unwrap(),
                    effect: ScalarEffect::Pure,
                    facts: record(),
                    conformance: record(),
                },
                Arc::new(Fixed(vec![test_type()])),
            ),
        )
        .unwrap();
    scalar
        .register(
            provider.clone(),
            scalar_definition("binary", 2, 1, Arc::new(Same)),
        )
        .unwrap();
    scalar
        .register(
            provider,
            scalar_definition(
                "split",
                1,
                2,
                Arc::new(Fixed(vec![test_type(), test_type()])),
            ),
        )
        .unwrap();
    (semantic, scalar.freeze())
}

fn scalar_output(builder: &mut IndexRegionBuilder, value: tiler_ir::index::ScalarValueId) {
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([]))
        .unwrap();
    let write = builder.write(output, &[], &[]).unwrap();
    builder.output(write, value).unwrap();
}

#[test]
fn external_non_f32_constant_and_multi_result_apply_form_generic_ssa() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let constant = builder
        .apply(
            ScalarOpKey::new("example", "constant", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap();
    let split = builder
        .apply(
            ScalarOpKey::new("example", "split", 1).unwrap(),
            ScalarAttributes::empty(),
            &[constant.get(0).unwrap()],
        )
        .unwrap();
    assert_eq!(split.len(), 2);
    scalar_output(&mut builder, split.get(1).unwrap());
    let region = builder.build().unwrap();
    assert_eq!(region.scalar_operations().count(), 2);
    assert_eq!(region.scalar_values().count(), 3);
}

#[test]
fn binary_pointwise_and_generic_n_state_reduction_are_typed() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let k = builder
        .dimension(DomainRole::Reduction, Extent::new(4))
        .unwrap();
    let constant_key = ScalarOpKey::new("example", "constant", 1).unwrap();
    let init = builder
        .apply(constant_key.clone(), ScalarAttributes::empty(), &[])
        .unwrap()
        .get(0)
        .unwrap();
    let contributor = builder
        .apply(constant_key, ScalarAttributes::empty(), &[])
        .unwrap()
        .get(0)
        .unwrap();
    let binary = ScalarOpKey::new("example", "binary", 1).unwrap();
    let reduced = builder
        .reduce(&[k], &[init], &[contributor], |body| {
            let result = body.apply(
                binary,
                ScalarAttributes::empty(),
                &[body.state(0).unwrap(), body.contributor(0).unwrap()],
            )?;
            body.yield_values(&[result.get(0).unwrap()])
        })
        .unwrap();
    scalar_output(&mut builder, reduced.get(0).unwrap());
    assert!(builder.build().is_ok());
}

#[test]
fn signature_and_foreign_handles_fail_closed() {
    let (_, registry) = registries();
    let mut first = IndexRegionBuilder::new(registry.clone()).unwrap();
    let mut second = IndexRegionBuilder::new(registry).unwrap();
    let foreign = second
        .dimension(DomainRole::Reduction, Extent::new(2))
        .unwrap();
    assert!(matches!(
        first.dimension_expr(foreign),
        Err(IndexBuildError::ForeignHandle { .. })
    ));
    let error = first
        .apply(
            ScalarOpKey::new("example", "binary", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap_err();
    assert!(matches!(error, IndexBuildError::ScalarAuthority(_)));
}

fn constant_region(extra_dead_first: bool, attribute: u64) -> Vec<u8> {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    if extra_dead_first {
        let _ = builder.constant(999_i128.into()).unwrap();
    }
    let attributes = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(9, CanonicalValue::unsigned(attribute))])
            .unwrap(),
    );
    // The governed constant has an empty schema, so use the attribute only to exercise deterministic rejection separately.
    assert!(attributes.is_ok());
    let value = builder
        .apply(
            ScalarOpKey::new("example", "constant", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap();
    scalar_output(&mut builder, value);
    builder
        .build()
        .unwrap()
        .canonical_identity()
        .as_bytes()
        .to_vec()
}

#[test]
fn unreachable_insertion_order_does_not_change_canonical_identity() {
    assert_eq!(constant_region(false, 1), constant_region(true, 1));
}

#[test]
fn ordered_reduction_dimensions_change_identity() {
    fn build(reverse: bool) -> Vec<u8> {
        let (_, registry) = registries();
        let mut builder = IndexRegionBuilder::new(registry).unwrap();
        let a = builder
            .dimension(DomainRole::Reduction, Extent::new(2))
            .unwrap();
        let b = builder
            .dimension(DomainRole::Reduction, Extent::new(3))
            .unwrap();
        let key = ScalarOpKey::new("example", "constant", 1).unwrap();
        let init = builder
            .apply(key.clone(), ScalarAttributes::empty(), &[])
            .unwrap()
            .get(0)
            .unwrap();
        let contributor = builder
            .apply(key, ScalarAttributes::empty(), &[])
            .unwrap()
            .get(0)
            .unwrap();
        let binary = ScalarOpKey::new("example", "binary", 1).unwrap();
        let dims = if reverse { vec![b, a] } else { vec![a, b] };
        let value = builder
            .reduce(&dims, &[init], &[contributor], |body| {
                let r = body.apply(
                    binary,
                    ScalarAttributes::empty(),
                    &[body.state(0).unwrap(), body.contributor(0).unwrap()],
                )?;
                body.yield_values(&[r.get(0).unwrap()])
            })
            .unwrap()
            .get(0)
            .unwrap();
        scalar_output(&mut builder, value);
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }
    assert_ne!(build(false), build(true));
}

fn operation_identity(
    name: &str,
    attributes: ScalarAttributes,
    output_type: ResolvedValueType,
) -> Vec<u8> {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let value = builder
        .apply(
            ScalarOpKey::new("example", name, 1).unwrap(),
            attributes,
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, output_type, Shape::from_dims([]))
        .unwrap();
    let write = builder.write(output, &[], &[]).unwrap();
    builder.output(write, value).unwrap();
    builder
        .build()
        .unwrap()
        .canonical_identity()
        .as_bytes()
        .to_vec()
}

#[test]
fn identity_distinguishes_operation_key_attributes_and_resolved_types() {
    let empty = ScalarAttributes::empty();
    assert_ne!(
        operation_identity("constant", empty.clone(), test_type()),
        operation_identity("constant_alt_key", empty.clone(), test_type())
    );
    let attrs1 = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(7, CanonicalValue::unsigned(1))]).unwrap(),
    )
    .unwrap();
    let attrs2 = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(7, CanonicalValue::unsigned(2))]).unwrap(),
    )
    .unwrap();
    assert_ne!(
        operation_identity("attributed", attrs1, test_type()),
        operation_identity("attributed", attrs2, test_type())
    );
    assert_ne!(
        operation_identity("constant", empty.clone(), test_type()),
        operation_identity("constant_alt_type", empty, alternate_type())
    );
}

#[test]
fn two_state_reduction_shares_one_occurrence_and_exposes_typed_body() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let dimension = builder
        .dimension(DomainRole::Reduction, Extent::new(8))
        .unwrap();
    let value_init = builder
        .apply(
            ScalarOpKey::new("example", "constant", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap();
    let index_init = builder
        .apply(
            ScalarOpKey::new("example", "constant_alt_type", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap();
    let results = builder
        .reduce(
            &[dimension],
            &[value_init, index_init],
            &[value_init, index_init],
            |body| {
                let step = body.apply(
                    ScalarOpKey::new("example", "state_step", 1).unwrap(),
                    ScalarAttributes::empty(),
                    &[
                        body.state(0).unwrap(),
                        body.state(1).unwrap(),
                        body.contributor(0).unwrap(),
                        body.contributor(1).unwrap(),
                    ],
                )?;
                body.yield_values(&[step.get(0).unwrap(), step.get(1).unwrap()])
            },
        )
        .unwrap();
    let value_output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([]))
        .unwrap();
    let value_write = builder.write(value_output, &[], &[]).unwrap();
    builder
        .output(value_write, results.get(0).unwrap())
        .unwrap();
    let index_output = builder
        .tensor(TensorRole::Output, alternate_type(), Shape::from_dims([]))
        .unwrap();
    let index_write = builder.write(index_output, &[], &[]).unwrap();
    builder
        .output(index_write, results.get(1).unwrap())
        .unwrap();
    let region = builder.build().unwrap();
    let mut outputs = region.outputs();
    let first = region
        .scalar_value(outputs.next().unwrap().value())
        .unwrap();
    let second = region
        .scalar_value(outputs.next().unwrap().value())
        .unwrap();
    let (
        tiler_ir::index::ScalarValueDefinitionView::OperationResult {
            operation: first_operation,
            ..
        },
        tiler_ir::index::ScalarValueDefinitionView::OperationResult {
            operation: second_operation,
            ..
        },
    ) = (first.definition(), second.definition())
    else {
        panic!("reduction outputs must be operation results")
    };
    assert_eq!(first_operation, second_operation);
    let operation = region.scalar_operation(first_operation).unwrap();
    let tiler_ir::index::ScalarOperationKindRef::Reduce(reduction) = operation.kind() else {
        panic!("expected reduction")
    };
    assert_eq!(reduction.body().values().count(), 6);
    assert_eq!(reduction.body().operations().count(), 1);
    assert_eq!(reduction.body().yields().count(), 2);
}

#[test]
fn reducer_body_handles_cannot_cross_reduction_closures() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let dimension = builder
        .dimension(DomainRole::Reduction, Extent::new(2))
        .unwrap();
    let value = builder
        .apply(
            ScalarOpKey::new("example", "constant", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap();
    let mut captured = None;
    let first = builder
        .reduce(&[dimension], &[value], &[value], |body| {
            captured = body.state(0);
            body.yield_values(&[body.state(0).unwrap()])
        })
        .unwrap();
    let error = builder
        .reduce(&[dimension], &[value], &[value], |body| {
            body.apply(
                ScalarOpKey::new("example", "binary", 1).unwrap(),
                ScalarAttributes::empty(),
                &[captured.unwrap(), body.contributor(0).unwrap()],
            )
            .map(|_| ())
        })
        .unwrap_err();
    assert!(matches!(error, IndexBuildError::ForeignHandle { .. }));
    scalar_output(&mut builder, first.get(0).unwrap());
    assert!(builder.build().is_ok());
}

#[test]
fn reducer_yield_failures_are_specific_and_transactional() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let dimension = builder
        .dimension(DomainRole::Reduction, Extent::new(2))
        .unwrap();
    let value = constant_value(&mut builder);
    let alternate = builder
        .apply(
            ScalarOpKey::new("example", "constant_alt_type", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap();

    assert_eq!(
        builder
            .reduce(&[dimension], &[value], &[value], |_| Ok(()))
            .unwrap_err(),
        IndexBuildError::MissingReducerYield
    );
    assert_eq!(
        builder
            .reduce(&[dimension], &[value], &[value], |body| body
                .yield_values(&[]))
            .unwrap_err(),
        IndexBuildError::ReducerYieldArity {
            expected: 1,
            actual: 0,
        }
    );
    assert!(matches!(
        builder
            .reduce(&[dimension], &[value], &[alternate], |body| {
                body.yield_values(&[body.contributor(0).unwrap()])
            })
            .unwrap_err(),
        IndexBuildError::ReducerYieldTypeMismatch { position: 0, .. }
    ));
    assert_eq!(
        builder
            .reduce(&[dimension], &[value], &[value], |body| {
                body.yield_values(&[body.state(0).unwrap()])?;
                body.yield_values(&[body.state(0).unwrap()])
            })
            .unwrap_err(),
        IndexBuildError::ReducerYieldAlreadySet
    );

    let reduced = builder
        .reduce(&[dimension], &[value], &[value], |body| {
            body.yield_values(&[body.state(0).unwrap()])
        })
        .unwrap();
    scalar_output(&mut builder, reduced.get(0).unwrap());
    assert!(builder.build().is_ok());
}

#[test]
fn multi_result_structural_key_storage_is_preflighted_transactionally() {
    let (semantic, _) = registries();
    let mut scalar = ScalarRegistryBuilder::new(semantic);
    let provider = ProviderIdentity::new("example", "wide", 1).unwrap();
    scalar
        .register(
            provider.clone(),
            scalar_definition("constant", 0, 1, Arc::new(Fixed(vec![test_type()]))),
        )
        .unwrap();
    scalar
        .register(
            provider,
            ScalarOperationDefinition::new(
                ScalarOpKey::new("example", "wide", 1).unwrap(),
                NormativeDefinitionRef::new("urn:example:wide:v1").unwrap(),
                ScalarOperationContract {
                    attributes: ScalarAttributeSchema::new([ScalarAttributeField::new(
                        1,
                        CanonicalValueKind::Bytes,
                        true,
                    )])
                    .unwrap(),
                    operands: ScalarArity::exact(0).unwrap(),
                    results: ScalarArity::exact(4_096).unwrap(),
                    effect: ScalarEffect::Pure,
                    facts: record(),
                    conformance: record(),
                },
                Arc::new(Fixed(vec![test_type(); 4_096])),
            ),
        )
        .unwrap();
    let mut builder = IndexRegionBuilder::new(scalar.freeze()).unwrap();
    let attributes = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(
            1,
            CanonicalValue::bytes(vec![0; 5_000]).unwrap(),
        )])
        .unwrap(),
    )
    .unwrap();
    assert!(matches!(
        builder.apply(
            ScalarOpKey::new("example", "wide", 1).unwrap(),
            attributes,
            &[],
        ),
        Err(IndexBuildError::StructuralLimit { .. })
    ));
    let value = constant_value(&mut builder);
    scalar_output(&mut builder, value);
    assert!(builder.build().is_ok());
}

#[test]
fn read_structural_keys_are_governed_before_access_commit() {
    fn clean_identity() -> Vec<u8> {
        let (_, registry) = registries();
        let mut builder = IndexRegionBuilder::new(registry).unwrap();
        let input = builder
            .tensor(
                TensorRole::Input,
                test_type(),
                Shape::from_dims(std::iter::repeat_n(1, MAX_TENSOR_RANK)),
            )
            .unwrap();
        let zero = builder.constant(0_i128.into()).unwrap();
        let value = builder
            .read(input, &[], &vec![zero; MAX_TENSOR_RANK])
            .unwrap();
        scalar_output(&mut builder, value);
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }

    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            test_type(),
            Shape::from_dims(std::iter::repeat_n(1, MAX_TENSOR_RANK)),
        )
        .unwrap();
    let zero = builder.constant(0_i128.into()).unwrap();
    let first = builder
        .read(input, &[], &vec![zero; MAX_TENSOR_RANK])
        .unwrap();
    let mut rejected = false;
    for value in 1_i128..2_000 {
        let distinct = builder.constant(value.into()).unwrap();
        let mut coordinates = vec![zero; MAX_TENSOR_RANK];
        coordinates[0] = distinct;
        match builder.read(input, &[], &coordinates) {
            Ok(_) => {}
            Err(IndexBuildError::StructuralLimit {
                resource: tiler_ir::index::IndexLimitKind::ScalarCanonicalBytes,
                ..
            }) => {
                rejected = true;
                break;
            }
            Err(error) => panic!("unexpected read rejection: {error}"),
        }
    }
    assert!(
        rejected,
        "read keys must reach the governed scalar-byte limit"
    );
    scalar_output(&mut builder, first);
    assert_eq!(
        builder.build().unwrap().canonical_identity().as_bytes(),
        clean_identity()
    );
}

#[test]
fn scalar_apply_and_nested_apply_are_commoned_only_under_pure_contract() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let key = ScalarOpKey::new("example", "constant", 1).unwrap();
    let first = builder
        .apply(key.clone(), ScalarAttributes::empty(), &[])
        .unwrap();
    let second = builder.apply(key, ScalarAttributes::empty(), &[]).unwrap();
    assert_eq!(first, second);
    scalar_output(&mut builder, first.get(0).unwrap());
    let region = builder.build().unwrap();
    assert_eq!(region.scalar_operations().count(), 1);
}

fn constant_value(builder: &mut IndexRegionBuilder) -> tiler_ir::index::ScalarValueId {
    builder
        .apply(
            ScalarOpKey::new("example", "constant", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap()
}

fn copy_region(
    extent: u64,
) -> (
    IndexRegionBuilder,
    tiler_ir::index::TensorId,
    tiler_ir::index::TensorId,
    tiler_ir::index::DimensionId,
    tiler_ir::index::IndexExprId,
) {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let input = builder
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([extent]))
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([extent]))
        .unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(extent))
        .unwrap();
    let expression = builder.dimension_expr(dimension).unwrap();
    (builder, input, output, dimension, expression)
}

#[test]
fn linear_normalization_is_independent_of_nested_construction_form() {
    fn build(nested: bool) -> Vec<u8> {
        let (mut builder, input, output, dimension, expression) = copy_region(7);
        let doubled = if nested {
            let doubled = builder
                .linear_combination(0_i128.into(), &[(2_i128.into(), expression)])
                .unwrap();
            builder
                .linear_combination(0_i128.into(), &[(1_i128.into(), doubled)])
                .unwrap()
        } else {
            builder
                .linear_combination(
                    0_i128.into(),
                    &[(1_i128.into(), expression), (1_i128.into(), expression)],
                )
                .unwrap()
        };
        let coordinate = builder.modulo(doubled, 7).unwrap();
        let read = builder.read(input, &[dimension], &[coordinate]).unwrap();
        let write = builder.write(output, &[dimension], &[expression]).unwrap();
        builder.output(write, read).unwrap();
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }
    assert_eq!(build(false), build(true));
}

#[test]
fn scalar_registration_failures_are_atomic_and_validate_nested_types() {
    let (semantic, _) = registries();
    let first_provider = ProviderIdentity::new("example", "first", 1).unwrap();
    let second_provider = ProviderIdentity::new("example", "second", 1).unwrap();
    let mut scalar = ScalarRegistryBuilder::new(semantic);
    let definition = scalar_definition("constant", 0, 1, Arc::new(Fixed(vec![test_type()])));
    scalar
        .register(first_provider.clone(), definition.clone())
        .unwrap();
    assert!(matches!(
        scalar.register(second_provider, definition),
        Err(ScalarRegistryError::DuplicateDefinition { .. })
    ));

    let unknown = ResolvedValueType::nominal(TypeKey::new("unknown", "type", 1).unwrap());
    let nested_unknown = CanonicalValue::sequence([CanonicalValue::sequence([
        CanonicalValue::value_type(unknown),
    ])
    .unwrap()])
    .unwrap();
    let invalid = ScalarOperationDefinition::new(
        ScalarOpKey::new("example", "invalid_nested_type", 1).unwrap(),
        NormativeDefinitionRef::new("urn:example:invalid-nested-type:v1").unwrap(),
        ScalarOperationContract {
            attributes: ScalarAttributeSchema::empty(),
            operands: ScalarArity::exact(0).unwrap(),
            results: ScalarArity::exact(1).unwrap(),
            effect: ScalarEffect::Pure,
            facts: nested_unknown,
            conformance: record(),
        },
        Arc::new(Fixed(vec![test_type()])),
    );
    assert!(matches!(
        scalar.register(first_provider.clone(), invalid),
        Err(ScalarRegistryError::TypeAuthority(_))
    ));
    let zero_result = ScalarOperationDefinition::new(
        ScalarOpKey::new("example", "zero_result", 1).unwrap(),
        NormativeDefinitionRef::new("urn:example:zero-result:v1").unwrap(),
        ScalarOperationContract {
            attributes: ScalarAttributeSchema::empty(),
            operands: ScalarArity::exact(0).unwrap(),
            results: ScalarArity::exact(0).unwrap(),
            effect: ScalarEffect::Pure,
            facts: record(),
            conformance: record(),
        },
        Arc::new(Fixed(Vec::new())),
    );
    assert_eq!(
        scalar.register(first_provider.clone(), zero_result),
        Err(ScalarRegistryError::ZeroResultDefinition)
    );

    let frozen = scalar.freeze();
    let key = ScalarOpKey::new("example", "constant", 1).unwrap();
    assert_eq!(frozen.provider(&key), Some(&first_provider));
    assert_eq!(
        frozen
            .definition(&key)
            .unwrap()
            .normative_definition()
            .as_str(),
        "urn:example:constant:v1"
    );
}

#[test]
fn reached_definition_projection_has_a_checked_byte_limit() {
    let (semantic, _) = registries();
    let mut scalar = ScalarRegistryBuilder::new(semantic);
    let provider = ProviderIdentity::new("example", "projection", 1).unwrap();
    let mut keys = Vec::new();
    for index in 0..129 {
        let key = ScalarOpKey::new("example", format!("projection_{index}"), 1).unwrap();
        let definition = ScalarOperationDefinition::new(
            key.clone(),
            NormativeDefinitionRef::new(format!("urn:example:projection:{index}:v1")).unwrap(),
            ScalarOperationContract {
                attributes: ScalarAttributeSchema::empty(),
                operands: ScalarArity::exact(0).unwrap(),
                results: ScalarArity::exact(1).unwrap(),
                effect: ScalarEffect::Pure,
                facts: CanonicalValue::bytes(vec![0; 65_536]).unwrap(),
                conformance: record(),
            },
            Arc::new(Fixed(vec![test_type()])),
        );
        scalar.register(provider.clone(), definition).unwrap();
        keys.push(key);
    }
    let frozen = scalar.freeze();
    assert!(matches!(
        frozen.project_reached(keys.iter()),
        Err(ScalarRegistryError::ProjectionByteLimit { .. })
    ));
}

#[test]
fn aggregate_registry_bytes_are_preflighted_transactionally() {
    let (semantic, _) = registries();
    let mut scalar = ScalarRegistryBuilder::new(semantic);
    let provider = ProviderIdentity::new("example", "registry-budget", 1).unwrap();
    let mut rejected = None;
    for index in 0..257 {
        let key = ScalarOpKey::new("example", format!("budget_{index}"), 1).unwrap();
        let definition = ScalarOperationDefinition::new(
            key.clone(),
            NormativeDefinitionRef::new(format!("urn:example:budget:{index}:v1")).unwrap(),
            ScalarOperationContract {
                attributes: ScalarAttributeSchema::empty(),
                operands: ScalarArity::exact(0).unwrap(),
                results: ScalarArity::exact(1).unwrap(),
                effect: ScalarEffect::Pure,
                facts: CanonicalValue::bytes(vec![0; 65_536]).unwrap(),
                conformance: record(),
            },
            Arc::new(Fixed(vec![test_type()])),
        );
        match scalar.register(provider.clone(), definition) {
            Ok(()) => {}
            Err(ScalarRegistryError::RegistryByteLimit { .. }) => {
                rejected = Some(key);
                break;
            }
            Err(error) => panic!("unexpected registry rejection: {error}"),
        }
    }
    let rejected = rejected.expect("fixture must exceed the registry byte limit");
    let frozen = scalar.freeze();
    let first = ScalarOpKey::new("example", "budget_0", 1).unwrap();
    assert!(frozen.definition(&first).is_some());
    assert!(frozen.definition(&rejected).is_none());
}

#[test]
fn ordered_tensor_bindings_participate_in_identity() {
    fn build(reverse_operands: bool) -> Vec<u8> {
        let (_, registry) = registries();
        let mut builder = IndexRegionBuilder::new(registry).unwrap();
        let first = builder
            .tensor(TensorRole::Input, test_type(), Shape::from_dims([]))
            .unwrap();
        let second = builder
            .tensor(TensorRole::Input, test_type(), Shape::from_dims([]))
            .unwrap();
        let left = builder.read(first, &[], &[]).unwrap();
        let right = builder.read(second, &[], &[]).unwrap();
        let operands = if reverse_operands {
            [right, left]
        } else {
            [left, right]
        };
        let value = builder
            .apply(
                ScalarOpKey::new("example", "binary", 1).unwrap(),
                ScalarAttributes::empty(),
                &operands,
            )
            .unwrap()
            .get(0)
            .unwrap();
        scalar_output(&mut builder, value);
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }
    assert_ne!(build(false), build(true));
}

#[test]
fn reachable_read_insertion_order_does_not_change_identity() {
    fn build(reverse: bool) -> Vec<u8> {
        let (_, registry) = registries();
        let mut builder = IndexRegionBuilder::new(registry).unwrap();
        let left_tensor = builder
            .tensor(TensorRole::Input, test_type(), Shape::from_dims([]))
            .unwrap();
        let right_tensor = builder
            .tensor(TensorRole::Input, test_type(), Shape::from_dims([]))
            .unwrap();
        let (left, right) = if reverse {
            let right = builder.read(right_tensor, &[], &[]).unwrap();
            let left = builder.read(left_tensor, &[], &[]).unwrap();
            (left, right)
        } else {
            let left = builder.read(left_tensor, &[], &[]).unwrap();
            let right = builder.read(right_tensor, &[], &[]).unwrap();
            (left, right)
        };
        let value = builder
            .apply(
                ScalarOpKey::new("example", "binary", 1).unwrap(),
                ScalarAttributes::empty(),
                &[left, right],
            )
            .unwrap()
            .get(0)
            .unwrap();
        scalar_output(&mut builder, value);
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }
    assert_eq!(build(false), build(true));
}

#[test]
fn access_domain_rejects_unbound_coordinates() {
    let (mut builder, input, _, _, expression) = copy_region(2);
    assert_eq!(
        builder.read(input, &[], &[expression]).unwrap_err(),
        IndexBuildError::CoordinateOutsideAccessDomain
    );
}

#[test]
fn conservative_interval_overlap_uses_finite_proof() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let input = builder
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([3]))
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([5]))
        .unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(5))
        .unwrap();
    let expression = builder.dimension_expr(dimension).unwrap();
    let modulo = builder.modulo(expression, 2).unwrap();
    let quotient = builder.floor_div(expression, 2).unwrap();
    let coordinate = builder
        .linear_combination(
            0_i128.into(),
            &[(1_i128.into(), modulo), (1_i128.into(), quotient)],
        )
        .unwrap();
    let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
    let write = builder.write(output, &[dimension], &[expression]).unwrap();
    builder.output(write, value).unwrap();
    let region = builder.build().unwrap();
    assert!(
        region
            .accesses()
            .any(|access| { access.bounds_proof() == BoundsProofView::Exhaustive { points: 5 } })
    );
}

#[test]
fn every_output_tensor_requires_exactly_one_root() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let written = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([]))
        .unwrap();
    let _missing = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([]))
        .unwrap();
    let value = constant_value(&mut builder);
    let write = builder.write(written, &[], &[]).unwrap();
    builder.output(write, value).unwrap();
    assert!(
        builder
            .build()
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::MissingOutputTensor { .. }
            ))
    );
}

#[test]
fn duplicate_output_roots_are_rejected_transactionally() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([]))
        .unwrap();
    let value = constant_value(&mut builder);
    let write = builder.write(output, &[], &[]).unwrap();
    builder.output(write, value).unwrap();
    assert_eq!(
        builder.output(write, value).unwrap_err(),
        IndexBuildError::DuplicateOutputTensor
    );
    assert!(builder.build().is_ok());
}

#[test]
fn boundary_rank_budget_failure_leaves_builder_usable() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let oversized = Shape::from_dims(std::iter::repeat_n(1, MAX_TENSOR_RANK + 1));
    assert!(matches!(
        builder.tensor(TensorRole::Input, test_type(), oversized),
        Err(IndexBuildError::StructuralLimit { .. })
    ));
    let value = constant_value(&mut builder);
    scalar_output(&mut builder, value);
    assert!(builder.build().is_ok());
}

#[test]
fn empty_reduction_read_is_vacuous_and_parallel_write_is_proved() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let input = builder
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([2, 0]))
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([2]))
        .unwrap();
    let parallel = builder
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let reduction = builder
        .dimension(DomainRole::Reduction, Extent::new(0))
        .unwrap();
    let parallel_expr = builder.dimension_expr(parallel).unwrap();
    let reduction_expr = builder.dimension_expr(reduction).unwrap();
    let contributor = builder
        .read(
            input,
            &[parallel, reduction],
            &[parallel_expr, reduction_expr],
        )
        .unwrap();
    let init = constant_value(&mut builder);
    let reduced = builder
        .reduce(&[reduction], &[init], &[contributor], |body| {
            let result = body.apply(
                ScalarOpKey::new("example", "binary", 1).unwrap(),
                ScalarAttributes::empty(),
                &[body.state(0).unwrap(), body.contributor(0).unwrap()],
            )?;
            body.yield_values(&[result.get(0).unwrap()])
        })
        .unwrap()
        .get(0)
        .unwrap();
    let write = builder
        .write(output, &[parallel], &[parallel_expr])
        .unwrap();
    builder.output(write, reduced).unwrap();
    let region = builder.build().unwrap();
    let mut accesses = region.accesses();
    assert_eq!(
        accesses.next().unwrap().bounds_proof(),
        BoundsProofView::VacuousEmptyDomain
    );
    assert_eq!(
        accesses.next().unwrap().write_ownership_proof(),
        Some(WriteOwnershipProofView::CoordinatePermutation)
    );
}

#[test]
fn explicit_evaluation_scope_supports_constant_reduction_contributors() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let reduction = builder
        .dimension(DomainRole::Reduction, Extent::new(3))
        .unwrap();
    let init = constant_value(&mut builder);
    let contributor = builder
        .apply_in(
            &[reduction],
            ScalarOpKey::new("example", "constant", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap();
    let reduced = builder
        .reduce(&[reduction], &[init], &[contributor], |body| {
            let result = body.apply(
                ScalarOpKey::new("example", "binary", 1).unwrap(),
                ScalarAttributes::empty(),
                &[body.state(0).unwrap(), body.contributor(0).unwrap()],
            )?;
            body.yield_values(&[result.get(0).unwrap()])
        })
        .unwrap()
        .get(0)
        .unwrap();
    scalar_output(&mut builder, reduced);
    assert!(builder.build().is_ok());
}

#[test]
fn unused_and_free_reduction_dimensions_fail_closed() {
    let (_, registry) = registries();
    let mut unused = IndexRegionBuilder::new(registry.clone()).unwrap();
    let reduction = unused
        .dimension(DomainRole::Reduction, Extent::new(2))
        .unwrap();
    let value = constant_value(&mut unused);
    scalar_output(&mut unused, value);
    assert!(unused.build().unwrap_err().diagnostics().iter().any(
        |diagnostic| matches!(diagnostic, IndexRegionDiagnostic::UnusedDomainDimension { dimension } if *dimension == reduction)
    ));

    let mut free = IndexRegionBuilder::new(registry).unwrap();
    let input = free
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([2]))
        .unwrap();
    let reduction = free
        .dimension(DomainRole::Reduction, Extent::new(2))
        .unwrap();
    let expression = free.dimension_expr(reduction).unwrap();
    let value = free.read(input, &[reduction], &[expression]).unwrap();
    scalar_output(&mut free, value);
    assert!(
        free.build()
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::FreeReductionDimension { .. }
            ))
    );
}

#[test]
fn non_permutation_write_retains_bounded_exhaustive_evidence() {
    let (mut builder, input, output, dimension, expression) = copy_region(4);
    let reversed = builder
        .linear_combination(3_i128.into(), &[((-1_i128).into(), expression)])
        .unwrap();
    let value = builder.read(input, &[dimension], &[expression]).unwrap();
    let write = builder.write(output, &[dimension], &[reversed]).unwrap();
    builder.output(write, value).unwrap();
    let region = builder.build().unwrap();
    let write = region
        .accesses()
        .find(|access| access.mode() == tiler_ir::index::AccessMode::Write)
        .unwrap();
    assert_eq!(
        write.write_ownership_proof(),
        Some(WriteOwnershipProofView::Exhaustive { points: 4 })
    );
}

#[test]
fn exhaustive_ownership_obeys_proof_cell_budget() {
    let extent = 1_048_577;
    let (mut builder, input, output, dimension, expression) = copy_region(extent);
    let reversed = builder
        .linear_combination(
            IndexInteger::from_u64(extent - 1),
            &[((-1_i128).into(), expression)],
        )
        .unwrap();
    let value = builder.read(input, &[dimension], &[expression]).unwrap();
    let write = builder.write(output, &[dimension], &[reversed]).unwrap();
    builder.output(write, value).unwrap();
    assert!(
        builder
            .build()
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(
                diagnostic,
                IndexRegionDiagnostic::ProofResourceLimit { .. }
            ))
    );
}

#[test]
fn failed_foreign_insertion_leaves_builder_usable() {
    let (_, registry) = registries();
    let mut first = IndexRegionBuilder::new(registry.clone()).unwrap();
    let foreign = first
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    assert!(matches!(
        builder.dimension_expr(foreign),
        Err(IndexBuildError::ForeignHandle { .. })
    ));
    let value = constant_value(&mut builder);
    scalar_output(&mut builder, value);
    assert!(builder.build().is_ok());
}

#[test]
fn interleaved_boundary_declarations_have_canonical_role_local_identity() {
    fn build(interleave_output: bool) -> Vec<u8> {
        let (_, registry) = registries();
        let mut builder = IndexRegionBuilder::new(registry).unwrap();
        let first = builder
            .tensor(TensorRole::Input, test_type(), Shape::from_dims([]))
            .unwrap();
        let output = interleave_output.then(|| {
            builder
                .tensor(TensorRole::Output, test_type(), Shape::from_dims([]))
                .unwrap()
        });
        let second = builder
            .tensor(TensorRole::Input, test_type(), Shape::from_dims([]))
            .unwrap();
        let output = output.unwrap_or_else(|| {
            builder
                .tensor(TensorRole::Output, test_type(), Shape::from_dims([]))
                .unwrap()
        });
        let left = builder.read(first, &[], &[]).unwrap();
        let right = builder.read(second, &[], &[]).unwrap();
        let value = builder
            .apply(
                ScalarOpKey::new("example", "binary", 1).unwrap(),
                ScalarAttributes::empty(),
                &[left, right],
            )
            .unwrap()
            .get(0)
            .unwrap();
        let write = builder.write(output, &[], &[]).unwrap();
        builder.output(write, value).unwrap();
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }
    assert_eq!(build(false), build(true));
}
