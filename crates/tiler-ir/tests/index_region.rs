//! Public generic scalar-SSA and verified index-region integration tests.

use std::error::Error as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tiler_ir::CheckedBuildError;
use tiler_ir::index::{
    BoundsProofView, DomainRole, IndexBuildError, IndexDomainPredicate, IndexDomainUnknownReason,
    IndexInteger, IndexIntegerSign, IndexLimitKind, IndexRegionBuilder, IndexRegionDiagnostic,
    JointPartitionProofView, MAX_EXHAUSTIVE_PROOF_BYTES, MAX_EXHAUSTIVE_PROOF_CELLS,
    MAX_INDEX_EXPRESSION_DEPTH, MAX_INDEX_INTEGER_BYTES, MAX_TENSOR_RANK, ProofResource,
    ScalarArity, ScalarAttributeField, ScalarAttributeSchema, ScalarAttributes, ScalarEffect,
    ScalarInferenceError, ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey,
    ScalarOperationContract, ScalarOperationDefinition, ScalarOperationInferencer,
    ScalarRegistryBuilder, ScalarRegistryError, SourcedExtent, SymbolicExtentError, TensorRole,
    WriteOwnershipProofView,
};
use tiler_ir::semantic::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueKind, FrozenSemanticRegistry,
    MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES, NormativeDefinitionRef, ProviderDiagnosticCode,
    ProviderDiagnosticError, ProviderIdentity, RegistryError, ResolvedValueType,
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

struct Types(u32);
impl SemanticRegistryProvider for Types {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("example", "types", self.0).unwrap()
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

struct UnrelatedTypes(u32);
impl SemanticRegistryProvider for UnrelatedTypes {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("example", "unrelated-types", self.0).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("example", "unrelated", 1).unwrap()),
            NormativeDefinitionRef::new("urn:example:unrelated:v1").unwrap(),
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
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        for value_type in &self.0 {
            outputs.try_push(value_type.clone())?;
        }
        Ok(())
    }
}
struct Same;
impl ScalarOperationInferencer for Same {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        let Some(first) = request.operands().first() else {
            return Err(inference_error("arity", "operand required"));
        };
        if request.operands().iter().any(|value| value != first) {
            return Err(inference_error("type", "operands differ"));
        }
        outputs.try_push(first.clone())
    }
}
struct PairState;
impl ScalarOperationInferencer for PairState {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        if request.operands().len() != 4 {
            return Err(inference_error(
                "arity",
                "four state/contributor operands required",
            ));
        }
        outputs.try_push(request.operands()[0].clone())?;
        outputs.try_push(request.operands()[1].clone())
    }
}

fn inference_error(code: &str, message: &str) -> ScalarInferenceError {
    ScalarInferenceError::new(ProviderDiagnosticCode::new(code).unwrap(), message).unwrap()
}

fn scalar_definition(
    name: &str,
    operands: usize,
    results: usize,
    inferencer: Arc<dyn ScalarOperationInferencer>,
) -> ScalarOperationDefinition {
    ScalarOperationDefinition::new(
        ScalarOpKey::new("example", name, 1).unwrap(),
        NormativeDefinitionRef::from_owned(format!("urn:example:{name}:v1")).unwrap(),
        ScalarOperationContract::new(
            ScalarAttributeSchema::empty(),
            ScalarArity::exact(operands).unwrap(),
            ScalarArity::exact(results).unwrap(),
            ScalarEffect::Pure,
            record(),
            record(),
        ),
        inferencer,
    )
}

fn registries() -> (
    FrozenSemanticRegistry,
    tiler_ir::index::FrozenScalarRegistry,
) {
    registries_with_revision(1)
}

fn registries_with_revision(
    revision: u32,
) -> (
    FrozenSemanticRegistry,
    tiler_ir::index::FrozenScalarRegistry,
) {
    registries_with_revisions(1, revision)
}

fn registries_with_revisions(
    type_revision: u32,
    scalar_revision: u32,
) -> (
    FrozenSemanticRegistry,
    tiler_ir::index::FrozenScalarRegistry,
) {
    registries_with_extras(type_revision, scalar_revision, None, None)
}

fn registries_with_extras(
    type_revision: u32,
    scalar_revision: u32,
    unrelated_type_revision: Option<u32>,
    unrelated_scalar_revision: Option<u32>,
) -> (
    FrozenSemanticRegistry,
    tiler_ir::index::FrozenScalarRegistry,
) {
    let mut semantic = SemanticRegistryBuilder::new();
    semantic.register_provider(&Types(type_revision)).unwrap();
    if let Some(revision) = unrelated_type_revision {
        semantic
            .register_provider(&UnrelatedTypes(revision))
            .unwrap();
    }
    let semantic = semantic.freeze().unwrap();
    // Ad-hoc: `scalar_revision` is a parameter and the vocabulary is externally
    // registered (`state_step` over a test type). The standard profile has one
    // fixed revision and cannot express either.
    let mut scalar = ScalarRegistryBuilder::new(semantic.clone());
    let provider = ProviderIdentity::new("example", "scalar", scalar_revision).unwrap();
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
                ScalarOperationContract::new(
                    ScalarAttributeSchema::new([ScalarAttributeField::required(
                        AttributeFieldId::new(7),
                        CanonicalValueKind::Unsigned,
                    )])
                    .unwrap(),
                    ScalarArity::exact(0).unwrap(),
                    ScalarArity::exact(1).unwrap(),
                    ScalarEffect::Pure,
                    record(),
                    record(),
                ),
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
    if let Some(revision) = unrelated_scalar_revision {
        scalar
            .register(
                ProviderIdentity::new("example", "unrelated-scalar", revision).unwrap(),
                scalar_definition("unrelated", 0, 1, Arc::new(Fixed(vec![test_type()]))),
            )
            .unwrap();
    }
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
fn scalar_authority_revalidation_is_region_bound_and_provider_separate() {
    let (_, first_registry) = registries_with_revisions(1, 1);
    let (_, type_changed_registry) = registries_with_revisions(2, 1);
    let (_, scalar_changed_registry) = registries_with_revisions(1, 2);
    let (_, unrelated_type_registry) = registries_with_extras(1, 1, Some(1), None);
    let (_, unrelated_scalar_registry) = registries_with_extras(1, 1, None, Some(1));
    let mut builder = IndexRegionBuilder::new(first_registry.clone()).unwrap();
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
    let region = builder.build().unwrap();

    let first = first_registry.revalidate_region(&region).unwrap();
    let type_changed = type_changed_registry.revalidate_region(&region).unwrap();
    let scalar_changed = scalar_changed_registry.revalidate_region(&region).unwrap();
    assert_eq!(first.region(), region.canonical_identity());
    assert_eq!(first.definitions(), type_changed.definitions());
    assert_eq!(first.admission(), type_changed.admission());
    assert_eq!(first.type_definitions(), type_changed.type_definitions());
    assert_ne!(first.type_admission(), type_changed.type_admission());
    assert_ne!(first.semantic_snapshot(), type_changed.semantic_snapshot());
    assert_eq!(first.scalar_snapshot(), type_changed.scalar_snapshot());

    assert_eq!(first.definitions(), scalar_changed.definitions());
    assert_ne!(first.admission(), scalar_changed.admission());
    assert_eq!(first.type_admission(), scalar_changed.type_admission());
    assert_ne!(first.scalar_snapshot(), scalar_changed.scalar_snapshot());

    let unrelated_type = unrelated_type_registry.revalidate_region(&region).unwrap();
    assert_eq!(first.type_definitions(), unrelated_type.type_definitions());
    assert_eq!(first.type_admission(), unrelated_type.type_admission());
    assert_ne!(
        first.semantic_snapshot(),
        unrelated_type.semantic_snapshot()
    );

    let unrelated_scalar = unrelated_scalar_registry
        .revalidate_region(&region)
        .unwrap();
    assert_eq!(first.definitions(), unrelated_scalar.definitions());
    assert_eq!(first.admission(), unrelated_scalar.admission());
    assert_ne!(first.scalar_snapshot(), unrelated_scalar.scalar_snapshot());
}

#[test]
fn scalar_schema_defaults_normalize_before_structural_identity() {
    fn build(explicit: bool) -> Vec<u8> {
        let (semantic, _) = registries();
        let field = AttributeFieldId::new(11);
        let default = CanonicalValue::unsigned_u32(4);
        // Ad-hoc: registers `example::defaulted-scalar`, whose subject is a defaulted
        // attribute schema the governed profile does not carry.
        let mut scalar = ScalarRegistryBuilder::new(semantic);
        scalar
            .register(
                ProviderIdentity::new("example", "defaulted-scalar", 1).unwrap(),
                ScalarOperationDefinition::new(
                    ScalarOpKey::new("example", "defaulted", 1).unwrap(),
                    NormativeDefinitionRef::new("urn:example:defaulted:v1").unwrap(),
                    ScalarOperationContract::new(
                        ScalarAttributeSchema::new([ScalarAttributeField::defaulted(
                            field,
                            CanonicalValueKind::Unsigned,
                            default.clone(),
                        )
                        .unwrap()])
                        .unwrap(),
                        ScalarArity::exact(0).unwrap(),
                        ScalarArity::exact(1).unwrap(),
                        ScalarEffect::Pure,
                        record(),
                        record(),
                    ),
                    Arc::new(Fixed(vec![test_type()])),
                ),
            )
            .unwrap();
        let mut builder = IndexRegionBuilder::new(scalar.freeze()).unwrap();
        let attributes = if explicit {
            ScalarAttributes::new(
                CanonicalValue::record([CanonicalField::new(field, default)]).unwrap(),
            )
            .unwrap()
        } else {
            ScalarAttributes::empty()
        };
        let value = builder
            .apply(
                ScalarOpKey::new("example", "defaulted", 1).unwrap(),
                attributes,
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

    assert_eq!(build(false), build(true));
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
        CanonicalValue::record([CanonicalField::new(
            AttributeFieldId::new(9),
            CanonicalValue::unsigned_u64(attribute),
        )])
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
    fn build(reverse_declaration: bool, reverse_traversal: bool) -> Vec<u8> {
        let (_, registry) = registries();
        let mut builder = IndexRegionBuilder::new(registry).unwrap();
        let (a, b) = if reverse_declaration {
            let b = builder
                .dimension(DomainRole::Reduction, Extent::new(3))
                .unwrap();
            let a = builder
                .dimension(DomainRole::Reduction, Extent::new(2))
                .unwrap();
            (a, b)
        } else {
            let a = builder
                .dimension(DomainRole::Reduction, Extent::new(2))
                .unwrap();
            let b = builder
                .dimension(DomainRole::Reduction, Extent::new(3))
                .unwrap();
            (a, b)
        };
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
        let dims = if reverse_traversal {
            vec![b, a]
        } else {
            vec![a, b]
        };
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
    assert_eq!(build(false, false), build(true, false));
    assert_ne!(build(false, false), build(false, true));
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
        CanonicalValue::record([CanonicalField::new(
            AttributeFieldId::new(7),
            CanonicalValue::unsigned_u64(1),
        )])
        .unwrap(),
    )
    .unwrap();
    let attrs2 = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(
            AttributeFieldId::new(7),
            CanonicalValue::unsigned_u64(2),
        )])
        .unwrap(),
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
fn failed_reducer_rolls_back_graph_state_but_never_reuses_escaped_owner_nonce() {
    fn build(with_failed_attempt: bool) -> Vec<u8> {
        let (_, registry) = registries();
        let mut builder = IndexRegionBuilder::new(registry).unwrap();
        let dimension = builder
            .dimension(DomainRole::Reduction, Extent::new(2))
            .unwrap();
        let value = constant_value(&mut builder);
        let mut escaped = None;
        if with_failed_attempt {
            let error = builder
                .reduce(&[dimension], &[value], &[value], |body| {
                    escaped = body.state(0);
                    Err(IndexBuildError::MissingReducerYield)
                })
                .unwrap_err();
            assert_eq!(error, IndexBuildError::MissingReducerYield);
        }
        let reduced = builder
            .reduce(&[dimension], &[value], &[value], |body| {
                if let Some(stale) = escaped {
                    assert!(matches!(
                        body.apply(
                            ScalarOpKey::new("example", "binary", 1).unwrap(),
                            ScalarAttributes::empty(),
                            &[stale, body.contributor(0).unwrap()],
                        ),
                        Err(IndexBuildError::ForeignHandle { .. })
                    ));
                }
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
fn reduction_depth_failure_precedes_user_closure_invocation() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let dimension = builder
        .dimension(DomainRole::Reduction, Extent::new(2))
        .unwrap();
    let constant = constant_value(&mut builder);
    let mut value = constant;
    for _ in 0..tiler_ir::index::MAX_SCALAR_EXPRESSION_DEPTH {
        value = builder
            .apply(
                ScalarOpKey::new("example", "binary", 1).unwrap(),
                ScalarAttributes::empty(),
                &[value, constant],
            )
            .unwrap()
            .get(0)
            .unwrap();
    }
    let calls = AtomicUsize::new(0);
    assert!(matches!(
        builder.reduce(&[dimension], &[value], &[constant], |_| {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }),
        Err(IndexBuildError::StructuralLimit {
            resource: IndexLimitKind::ScalarExpressionDepth,
            ..
        })
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

#[test]
fn multi_result_structural_key_storage_is_preflighted_transactionally() {
    let (semantic, _) = registries();
    // Ad-hoc: needs a multi-result scalar to preflight structural-key storage.
    // Every governed scalar has arity 1, so the standard profile cannot reach
    // the transaction under test.
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
                ScalarOperationContract::new(
                    ScalarAttributeSchema::new([ScalarAttributeField::required(
                        AttributeFieldId::new(1),
                        CanonicalValueKind::Bytes,
                    )])
                    .unwrap(),
                    ScalarArity::exact(0).unwrap(),
                    ScalarArity::exact(4_096).unwrap(),
                    ScalarEffect::Pure,
                    record(),
                    record(),
                ),
                Arc::new(Fixed(vec![test_type(); 4_096])),
            ),
        )
        .unwrap();
    let mut builder = IndexRegionBuilder::new(scalar.freeze()).unwrap();
    let attributes = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(
            AttributeFieldId::new(1),
            CanonicalValue::bytes_owned(vec![0; 5_000]).unwrap(),
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
                Shape::try_from_dims(std::iter::repeat_n(1, MAX_TENSOR_RANK)).unwrap(),
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
            Shape::try_from_dims(std::iter::repeat_n(1, MAX_TENSOR_RANK)).unwrap(),
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
        let coordinate = builder
            .modulo(doubled, SourcedExtent::Static(Extent::new(7)))
            .unwrap();
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
    // Ad-hoc: the subject is two *different* providers registering one definition,
    // which is what `DuplicateDefinition` rejects. A frozen profile has one
    // provider per definition by construction and cannot stage the collision.
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
        ScalarOperationContract::new(
            ScalarAttributeSchema::empty(),
            ScalarArity::exact(0).unwrap(),
            ScalarArity::exact(1).unwrap(),
            ScalarEffect::Pure,
            nested_unknown,
            record(),
        ),
        Arc::new(Fixed(vec![test_type()])),
    );
    assert!(matches!(
        scalar.register(first_provider.clone(), invalid),
        Err(ScalarRegistryError::TypeAuthority(_))
    ));
    let zero_result = ScalarOperationDefinition::new(
        ScalarOpKey::new("example", "zero_result", 1).unwrap(),
        NormativeDefinitionRef::new("urn:example:zero-result:v1").unwrap(),
        ScalarOperationContract::new(
            ScalarAttributeSchema::empty(),
            ScalarArity::exact(0).unwrap(),
            ScalarArity::exact(0).unwrap(),
            ScalarEffect::Pure,
            record(),
            record(),
        ),
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
    // Ad-hoc: registers a projection whose reached-definition bytes are sized to sit
    // against the byte limit under test. The governed definitions are far below it.
    let mut scalar = ScalarRegistryBuilder::new(semantic);
    let provider = ProviderIdentity::new("example", "projection", 1).unwrap();
    let mut keys = Vec::new();
    for index in 0..129 {
        let key =
            ScalarOpKey::from_owned(String::from("example"), format!("projection_{index}"), 1)
                .unwrap();
        let definition = ScalarOperationDefinition::new(
            key.clone(),
            NormativeDefinitionRef::from_owned(format!("urn:example:projection:{index}:v1"))
                .unwrap(),
            ScalarOperationContract::new(
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(0).unwrap(),
                ScalarArity::exact(1).unwrap(),
                ScalarEffect::Pure,
                CanonicalValue::bytes_owned(vec![0; 65_536]).unwrap(),
                record(),
            ),
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
    // Ad-hoc: sized to exceed the aggregate registry byte budget. The standard
    // profile is deliberately well inside it, so it cannot reach the preflight.
    let mut scalar = ScalarRegistryBuilder::new(semantic);
    let provider = ProviderIdentity::new("example", "registry-budget", 1).unwrap();
    let mut rejected = None;
    for index in 0..257 {
        let key =
            ScalarOpKey::from_owned(String::from("example"), format!("budget_{index}"), 1).unwrap();
        let definition = ScalarOperationDefinition::new(
            key.clone(),
            NormativeDefinitionRef::from_owned(format!("urn:example:budget:{index}:v1")).unwrap(),
            ScalarOperationContract::new(
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(0).unwrap(),
                ScalarArity::exact(1).unwrap(),
                ScalarEffect::Pure,
                CanonicalValue::bytes_owned(vec![0; 65_536]).unwrap(),
                record(),
            ),
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
    let two = SourcedExtent::Static(Extent::new(2));
    let modulo = builder.modulo(expression, two.clone()).unwrap();
    let quotient = builder.floor_div(expression, two).unwrap();
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
    assert!(region.accesses().any(|access| {
        access.bounds_proof() == Some(BoundsProofView::Exhaustive { points: 5 })
    }));
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

/// A root repeated verbatim is admitted at construction and refused at
/// verification, which is the whole shape of the relaxation: whether two roots
/// over one boundary partition it is not a question `output` can answer.
#[test]
fn a_repeated_output_root_is_admitted_then_refused_as_an_overlap() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([]))
        .unwrap();
    let value = constant_value(&mut builder);
    let write = builder.write(output, &[], &[]).unwrap();
    builder.output(write, value).unwrap();
    builder.output(write, value).unwrap();
    let error = builder.build().unwrap_err();
    assert!(
        error.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::OutputPartitionRangesOverlap { .. }
        )),
        "two roots occupying one rectangle overlap: {:?}",
        error.diagnostics()
    );
}

/// Builds one output written by several roots over a single parallel dimension,
/// so the partition cases below differ only in the coordinates that are the
/// subject.
fn partitioned_output_region(
    extent: u64,
    parallel_extent: u64,
    roots: &[(i128, i128)],
) -> IndexRegionBuilder {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([extent]))
        .unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(parallel_extent))
        .unwrap();
    let expression = builder.dimension_expr(dimension).unwrap();
    let value = constant_value(&mut builder);
    for (coefficient, offset) in roots {
        let coordinate = builder
            .linear_combination((*offset).into(), &[((*coefficient).into(), expression)])
            .unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
    }
    builder
}

fn partition_ownership(
    region: &tiler_ir::index::VerifiedIndexRegion,
) -> Vec<WriteOwnershipProofView> {
    region
        .accesses()
        .filter(|access| access.mode() == tiler_ir::index::AccessMode::Write)
        .map(|access| access.write_ownership_proof().unwrap())
        .collect()
}

/// Unit-coefficient displaced dimensions are contiguous static ranges, so the
/// joint obligation closes without enumerating anything.
#[test]
fn contiguous_partitions_are_admitted_by_interval_reasoning() {
    // `k` and `k + 2` over a domain of two points tile `[0, 4)` as `[0, 2)` and
    // `[2, 4)`.
    let region = partitioned_output_region(4, 2, &[(1, 0), (1, 2)])
        .build()
        .unwrap();
    assert_eq!(
        partition_ownership(&region),
        vec![
            WriteOwnershipProofView::PartitionMember {
                joint: JointPartitionProofView::Interval
            };
            2
        ]
    );
}

/// A coordinate outside the displaced-dimension vocabulary declines interval
/// reasoning rather than refuting the region, and the joint walk decides it.
#[test]
fn strided_partitions_fall_back_to_the_recorded_joint_walk() {
    // `2k` and `2k + 1` interleave rather than tile, so no rectangle describes
    // either root; together they still cover `[0, 4)` exactly once.
    let region = partitioned_output_region(4, 2, &[(2, 0), (2, 1)])
        .build()
        .unwrap();
    assert_eq!(
        partition_ownership(&region),
        vec![
            WriteOwnershipProofView::PartitionMember {
                joint: JointPartitionProofView::Exhaustive { points: 4 }
            };
            2
        ]
    );
}

#[test]
fn an_uncovered_element_refuses_under_interval_reasoning() {
    // `[0, 2)` and `[2, 4)` leave element 4 of a five-element boundary bare.
    let error = partitioned_output_region(5, 2, &[(1, 0), (1, 2)])
        .build()
        .unwrap_err();
    assert!(
        error.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::OutputPartitionUncovered { .. }
        )),
        "{:?}",
        error.diagnostics()
    );
}

#[test]
fn an_uncovered_element_refuses_under_the_joint_walk() {
    // `2k` covers {0, 2} and `3k + 1` covers {1, 4}; element 3 is bare, and
    // neither coefficient admits a rectangle.
    let error = partitioned_output_region(5, 2, &[(2, 0), (3, 1)])
        .build()
        .unwrap_err();
    assert!(
        error.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::OutputPartitionUncovered { .. }
        )),
        "{:?}",
        error.diagnostics()
    );
}

#[test]
fn overlapping_ranges_refuse_before_coverage_is_considered() {
    // `[0, 2)` and `[1, 3)` share element 1. The summed volume also misses the
    // boundary's element count, and the overlap is what must be reported.
    let error = partitioned_output_region(3, 2, &[(1, 0), (1, 1)])
        .build()
        .unwrap_err();
    let diagnostics = error.diagnostics();
    assert!(
        diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::OutputPartitionRangesOverlap { .. }
        )),
        "{diagnostics:?}"
    );
    assert!(
        !diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::OutputPartitionUncovered { .. }
        )),
        "coverage is derived from disjointness, so a refuted pair reports the \
         overlap alone: {diagnostics:?}"
    );
}

#[test]
fn a_double_written_element_refuses_under_the_joint_walk() {
    // `2k` covers {0, 2} and `3k` covers {0, 3}; both claim element 0, and
    // neither coefficient admits a rectangle.
    let error = partitioned_output_region(4, 2, &[(2, 0), (3, 0)])
        .build()
        .unwrap_err();
    assert!(
        error.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::OutputPartitionDoubleWritten { .. }
        )),
        "{:?}",
        error.diagnostics()
    );
}

/// A sole root is unaffected by the partition path: its evidence is still the
/// form the verifier recorded before partitions existed.
#[test]
fn a_sole_root_keeps_its_whole_boundary_evidence() {
    let region = partitioned_output_region(4, 4, &[(1, 0)]).build().unwrap();
    assert_eq!(
        partition_ownership(&region),
        vec![WriteOwnershipProofView::CoordinatePermutation]
    );
}

/// Builds one output whose roots each iterate a parallel dimension of their
/// **own** extent, which is what an unequal partition needs and what one shared
/// domain cannot express: a root's point count is the product of the extents it
/// iterates, so roots sharing one domain own equal shares by construction.
///
/// Each root is `(extent, offset)` and writes `d + offset` over a dimension of
/// that extent, so the roots' rectangles are `[offset, offset + extent)`.
fn unequal_partition_region(boundary: u64, roots: &[(u64, i128)]) -> IndexRegionBuilder {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            test_type(),
            Shape::from_dims([boundary]),
        )
        .unwrap();
    let value = constant_value(&mut builder);
    for (extent, offset) in roots {
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(*extent))
            .unwrap();
        let expression = builder.dimension_expr(dimension).unwrap();
        let coordinate = builder
            .linear_combination((*offset).into(), &[(1_i128.into(), expression)])
            .unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
    }
    builder
}

/// The case the equal-share contract could not spell: three and five into eight.
#[test]
fn unequally_sized_contiguous_partitions_are_admitted_by_interval_reasoning() {
    let region = unequal_partition_region(8, &[(3, 0), (5, 3)])
        .build()
        .unwrap();
    assert_eq!(
        partition_ownership(&region),
        vec![
            WriteOwnershipProofView::PartitionMember {
                joint: JointPartitionProofView::Interval
            };
            2
        ]
    );
    // Canonicalization is part of what must hold, not only verification: an
    // identity that cannot be produced is a region that cannot be cached.
    assert!(!region.canonical_identity().as_bytes().is_empty());
}

/// The pinned concatenate occurrence's shape, with the zero-extent operand
/// first: `[0]` joined with `[4]` is `[4]`. The empty root visits no point, owns
/// nothing, and contributes zero volume.
#[test]
fn a_zero_extent_partition_member_owns_nothing_and_is_admitted() {
    let region = unequal_partition_region(4, &[(0, 0), (4, 0)])
        .build()
        .unwrap();
    assert_eq!(
        partition_ownership(&region),
        vec![
            WriteOwnershipProofView::PartitionMember {
                joint: JointPartitionProofView::Interval
            };
            2
        ]
    );
    // The empty root's own bounds obligation is vacuous rather than unproved:
    // it has no point at which a coordinate could leave the boundary.
    assert!(
        region
            .accesses()
            .any(|access| access.bounds_proof() == Some(BoundsProofView::VacuousEmptyDomain)),
        "the empty root's coordinates are discharged over an unvisited domain"
    );
}

/// An empty rectangle is disjoint from everything, including a sibling whose
/// range strictly contains its offset — the one placement where the per-axis
/// separation test alone is not exact, because `b > c && d > a` decides
/// `max(a, c) < min(b, d)` only for nonempty ranges.
#[test]
fn an_empty_partition_member_inside_a_sibling_range_is_still_disjoint() {
    let region = unequal_partition_region(4, &[(4, 0), (0, 2)])
        .build()
        .unwrap();
    assert_eq!(
        partition_ownership(&region),
        vec![
            WriteOwnershipProofView::PartitionMember {
                joint: JointPartitionProofView::Interval
            };
            2
        ]
    );
}

/// Unequal members whose volumes still sum to the boundary, so only disjointness
/// can refuse them. This is the perturbation that shows coverage is derived from
/// disjointness rather than checked beside it.
#[test]
fn overlapping_unequal_partitions_refuse_despite_an_exact_volume_sum() {
    // `[0, 3)` and `[2, 7)` share element 2 of an eight-element boundary while
    // summing to exactly eight.
    let error = unequal_partition_region(8, &[(3, 0), (5, 2)])
        .build()
        .unwrap_err();
    let diagnostics = error.diagnostics();
    assert!(
        diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::OutputPartitionRangesOverlap { .. }
        )),
        "{diagnostics:?}"
    );
    assert!(
        !diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::OutputPartitionUncovered { .. }
        )),
        "the summed volume is exact, so only the overlap may be reported: \
         {diagnostics:?}"
    );
}

/// The other perturbation: unequal members that are disjoint but leave a gap.
#[test]
fn unequal_partitions_that_leave_a_gap_refuse_as_uncovered() {
    // `[0, 3)` and `[4, 8)` leave element 3 of an eight-element boundary bare.
    let error = unequal_partition_region(8, &[(3, 0), (4, 4)])
        .build()
        .unwrap_err();
    assert!(
        error.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::OutputPartitionUncovered { .. }
        )),
        "{:?}",
        error.diagnostics()
    );
}

/// The subset relaxation stops at the role: a write may omit a parallel
/// dimension, never name a reduction one. This is what `InvalidWriteDomain`
/// still means.
#[test]
fn a_write_may_not_iterate_a_reduction_dimension() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([2]))
        .unwrap();
    let parallel = builder
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let reduction = builder
        .dimension(DomainRole::Reduction, Extent::new(3))
        .unwrap();
    let coordinate = builder.dimension_expr(parallel).unwrap();
    assert_eq!(
        builder
            .write(output, &[parallel, reduction], &[coordinate])
            .unwrap_err(),
        IndexBuildError::InvalidWriteDomain
    );
    // The same write over the parallel dimension alone is admitted, so the
    // refusal above is the reduction dimension and not the shape of the call.
    assert!(builder.write(output, &[parallel], &[coordinate]).is_ok());
}

/// The obligation a subset domain replaces the old rule with: a root may not
/// store a value that varies along a dimension the root does not iterate.
#[test]
fn a_value_may_not_vary_along_a_dimension_its_write_omits() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let input = builder
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([3]))
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([2]))
        .unwrap();
    let written = builder
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let unwritten = builder
        .dimension(DomainRole::Parallel, Extent::new(3))
        .unwrap();
    let written_expr = builder.dimension_expr(written).unwrap();
    let unwritten_expr = builder.dimension_expr(unwritten).unwrap();
    // The read varies along `unwritten`; the write does not iterate it.
    let value = builder
        .read(input, &[unwritten], &[unwritten_expr])
        .unwrap();
    let write = builder.write(output, &[written], &[written_expr]).unwrap();
    builder.output(write, value).unwrap();
    let error = builder.build().unwrap_err();
    assert!(
        error.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::ValueDimensionOutsideWriteDomain { .. }
        )),
        "{:?}",
        error.diagnostics()
    );
}

/// A parallel dimension nothing iterates and nothing varies along is retained by
/// compaction and would sit in the identity of a region whose meaning does not
/// include it. Unreachable until a write could omit a parallel dimension.
#[test]
fn a_parallel_dimension_no_root_iterates_is_refused_as_unused() {
    let mut builder = unequal_partition_region(4, &[(4, 0)]);
    let _dangling = builder
        .dimension(DomainRole::Parallel, Extent::new(7))
        .unwrap();
    let error = builder.build().unwrap_err();
    assert!(
        error.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::UnusedDomainDimension { .. }
        )),
        "{:?}",
        error.diagnostics()
    );
}

#[test]
fn boundary_rank_budget_failure_leaves_builder_usable() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let oversized = Shape::try_from_dims(std::iter::repeat_n(1, MAX_TENSOR_RANK + 1)).unwrap();
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
        Some(BoundsProofView::VacuousEmptyDomain)
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
    let error = builder.build().unwrap_err();
    assert!(error.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        IndexRegionDiagnostic::WriteOwnershipNotProven { .. }
    )));
    assert!(
        error.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            IndexRegionDiagnostic::ProofResourceLimit { .. }
        ))
    );
}

fn aggregate_read_budget_builder(include_unrooted_output: bool) -> IndexRegionBuilder {
    let extent = 199_999;
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let left_tensor = builder
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([100_000]))
        .unwrap();
    let right_tensor = builder
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([100_000]))
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([extent]))
        .unwrap();
    if include_unrooted_output {
        builder
            .tensor(TensorRole::Output, test_type(), Shape::from_dims([]))
            .unwrap();
    }
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(extent))
        .unwrap();
    let expression = builder.dimension_expr(dimension).unwrap();
    let two = SourcedExtent::Static(Extent::new(2));
    let modulo = builder.modulo(expression, two.clone()).unwrap();
    let quotient = builder.floor_div(expression, two).unwrap();
    let conservative = builder
        .linear_combination(
            0_i128.into(),
            &[(1_i128.into(), modulo), (1_i128.into(), quotient)],
        )
        .unwrap();
    let left = builder
        .read(left_tensor, &[dimension], &[conservative])
        .unwrap();
    let right = builder
        .read(right_tensor, &[dimension], &[conservative])
        .unwrap();
    let combined = builder
        .apply(
            ScalarOpKey::new("example", "binary", 1).unwrap(),
            ScalarAttributes::empty(),
            &[left, right],
        )
        .unwrap()
        .get(0)
        .unwrap();
    let write = builder.write(output, &[dimension], &[expression]).unwrap();
    builder.output(write, combined).unwrap();
    builder
}

#[test]
fn individually_admissible_accesses_share_one_aggregate_proof_budget() {
    let region = aggregate_read_budget_builder(false)
        .build()
        .expect("a read-only proof-budget stop becomes a semantic obligation");
    let unknown = region.unknown_index_domain_predicates().collect::<Vec<_>>();
    assert_eq!(unknown.len(), 2);
    assert!(unknown.iter().all(|record| matches!(
        record.predicate(),
        IndexDomainPredicate::LessThanExtent { .. }
    )));
    assert!(unknown.iter().all(|record| record.reason()
        == IndexDomainUnknownReason::ResourceLimit {
            resource: ProofResource::Cells,
            required: 1_599_992,
            limit: MAX_EXHAUSTIVE_PROOF_CELLS,
        }));
    assert!(unknown.iter().all(|record| {
        region
            .index_domain_evidence(record.subject(), record.predicate())
            .unwrap()
            .is_none()
    }));
}

#[test]
fn a_hard_diagnostic_is_not_upgraded_by_a_read_budget_stop() {
    let error = aggregate_read_budget_builder(true).build().unwrap_err();
    assert!(error.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        IndexRegionDiagnostic::MissingOutputTensor { .. }
    )));
    assert!(error.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        IndexRegionDiagnostic::ProofResourceLimit {
            resource: ProofResource::Cells,
            ..
        }
    )));
}

#[test]
fn read_integer_storage_budget_retains_the_exact_resource() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let input = builder
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([1]))
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([65]))
        .unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(65))
        .unwrap();
    let write_coordinate = builder.dimension_expr(dimension).unwrap();
    let mut magnitude = vec![0xff; MAX_INDEX_INTEGER_BYTES];
    magnitude[0] = 0x7f;
    magnitude[MAX_INDEX_INTEGER_BYTES - 1] = 0xfe;
    let huge_even = IndexInteger::from_sign_magnitude(IndexIntegerSign::Positive, &magnitude)
        .expect("the fixture sits exactly at the admitted integer size");
    let constant = builder.constant(huge_even).unwrap();
    let coordinate = builder
        .modulo(constant, SourcedExtent::Static(Extent::new(2)))
        .unwrap();
    let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
    let write = builder
        .write(output, &[dimension], &[write_coordinate])
        .unwrap();
    builder.output(write, value).unwrap();

    let region = builder
        .build()
        .expect("integer-storage exhaustion on a read is retained");
    let unknown = region.unknown_index_domain_predicates().collect::<Vec<_>>();
    assert_eq!(unknown.len(), 1);
    assert_eq!(
        unknown[0].reason(),
        IndexDomainUnknownReason::ResourceLimit {
            resource: ProofResource::IntegerBytes,
            required: 68_157_505,
            limit: MAX_EXHAUSTIVE_PROOF_BYTES,
        }
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

#[test]
fn late_zero_domain_is_vacuous_before_cardinality_overflow() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let shape = Shape::from_dims([u64::MAX, 2, 0]);
    let input = builder
        .tensor(TensorRole::Input, test_type(), shape.clone())
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, test_type(), shape)
        .unwrap();
    let dimensions = [u64::MAX, 2, 0].map(|extent| {
        builder
            .dimension(DomainRole::Parallel, Extent::new(extent))
            .unwrap()
    });
    let coordinates = dimensions.map(|dimension| builder.dimension_expr(dimension).unwrap());
    let value = builder.read(input, &dimensions, &coordinates).unwrap();
    let write = builder.write(output, &dimensions, &coordinates).unwrap();
    builder.output(write, value).unwrap();
    let region = builder.build().unwrap();
    assert!(
        region
            .accesses()
            .all(|access| { access.bounds_proof() == Some(BoundsProofView::VacuousEmptyDomain) })
    );
}

#[test]
fn dimension_declaration_order_is_alpha_canonical() {
    fn build(swapped: bool) -> Vec<u8> {
        let (_, registry) = registries();
        let mut builder = IndexRegionBuilder::new(registry).unwrap();
        let input = builder
            .tensor(TensorRole::Input, test_type(), Shape::from_dims([3, 5]))
            .unwrap();
        let output = builder
            .tensor(TensorRole::Output, test_type(), Shape::from_dims([3, 5]))
            .unwrap();
        let (row, column) = if swapped {
            let column = builder
                .dimension(DomainRole::Parallel, Extent::new(5))
                .unwrap();
            let row = builder
                .dimension(DomainRole::Parallel, Extent::new(3))
                .unwrap();
            (row, column)
        } else {
            let row = builder
                .dimension(DomainRole::Parallel, Extent::new(3))
                .unwrap();
            let column = builder
                .dimension(DomainRole::Parallel, Extent::new(5))
                .unwrap();
            (row, column)
        };
        let row_expr = builder.dimension_expr(row).unwrap();
        let column_expr = builder.dimension_expr(column).unwrap();
        let value = builder
            .read(input, &[row, column], &[row_expr, column_expr])
            .unwrap();
        let write = builder
            .write(output, &[row, column], &[row_expr, column_expr])
            .unwrap();
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

#[test]
fn deep_index_expression_is_rejected_at_its_specific_budget() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(8))
        .unwrap();
    let mut expression = builder.dimension_expr(dimension).unwrap();
    let two = SourcedExtent::Static(Extent::new(2));
    for _ in 0..MAX_INDEX_EXPRESSION_DEPTH {
        expression = builder.floor_div(expression, two.clone()).unwrap();
    }
    // The index layer's own depth budget, reported as a structural refusal and
    // not as a source one: the divisor was admissible and the expression was
    // too deep.
    assert!(matches!(
        builder.floor_div(expression, two),
        Err(SymbolicExtentError::Structural(
            IndexBuildError::StructuralLimit {
                resource: IndexLimitKind::IndexExpressionDepth,
                ..
            }
        ))
    ));
}

#[test]
fn index_integer_magnitude_is_bounded_and_zero_encoding_is_canonical() {
    assert_eq!(
        IndexInteger::from_i128(0).to_sign_magnitude(),
        (IndexIntegerSign::Zero, Vec::new())
    );
    let oversized = IndexInteger::from_sign_magnitude(
        IndexIntegerSign::Positive,
        &vec![1_u8; MAX_INDEX_INTEGER_BYTES + 1],
    )
    .unwrap();
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    assert!(matches!(
        builder.constant(oversized),
        Err(IndexBuildError::StructuralLimit {
            resource: IndexLimitKind::IndexIntegerBytes,
            ..
        })
    ));
    assert!(builder.constant(IndexInteger::from_i128(0)).is_ok());
}

#[test]
fn derived_linear_product_is_rejected_before_oversized_bigint_construction() {
    let magnitude_bytes = MAX_INDEX_INTEGER_BYTES / 2 + 1;
    let mut magnitude = vec![0_u8; magnitude_bytes];
    magnitude[0] = 0x80;
    let coefficient =
        IndexInteger::from_sign_magnitude(IndexIntegerSign::Positive, &magnitude).unwrap();
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(2))
        .unwrap();
    let base = builder.dimension_expr(dimension).unwrap();
    let inner = builder
        .linear_combination(0_i128.into(), &[(coefficient.clone(), base)])
        .unwrap();
    assert!(matches!(
        builder.linear_combination(0_i128.into(), &[(coefficient, inner)]),
        Err(IndexBuildError::StructuralLimit {
            resource: IndexLimitKind::IndexIntegerBytes,
            ..
        })
    ));
}

#[test]
fn attribute_schema_stops_consuming_an_unbounded_source() {
    let consumed = AtomicUsize::new(0);
    let fields = std::iter::repeat_with(|| {
        let index = consumed.fetch_add(1, Ordering::Relaxed);
        ScalarAttributeField::optional(
            AttributeFieldId::new(u32::try_from(index + 1).unwrap()),
            CanonicalValueKind::Unsigned,
        )
    });
    assert!(matches!(
        ScalarAttributeSchema::new(fields),
        Err(ScalarRegistryError::TooManyAttributeFields { .. })
    ));
    assert_eq!(consumed.load(Ordering::Relaxed), 257);
}

#[test]
fn ignored_provider_writer_failure_is_sticky_and_preserves_error_sources() {
    struct IgnoresOverflow;
    impl ScalarOperationInferencer for IgnoresOverflow {
        fn infer(
            &self,
            _: ScalarInferenceRequest<'_>,
            outputs: &mut ScalarInferenceOutputs,
        ) -> Result<(), ScalarInferenceError> {
            outputs.try_push(test_type())?;
            let _ = outputs.try_push(test_type());
            Ok(())
        }
    }

    let (semantic, _) = registries();
    // Ad-hoc: registers an inferencer that overflows on purpose. No governed
    // definition may do that, so the standard profile cannot exercise the path.
    let mut registry = ScalarRegistryBuilder::new(semantic);
    registry
        .register(
            ProviderIdentity::new("example", "overflow", 1).unwrap(),
            scalar_definition("overflow", 0, 1, Arc::new(IgnoresOverflow)),
        )
        .unwrap();
    let mut builder = IndexRegionBuilder::new(registry.freeze()).unwrap();
    let error = builder
        .apply(
            ScalarOpKey::new("example", "overflow", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap_err();
    let registry_error = error.source().expect("index error retains registry source");
    let rejection = registry_error
        .source()
        .expect("registry error retains attributed rejection");
    assert!(rejection.to_string().contains("example::overflow"));
    assert!(rejection.to_string().contains("result-limit"));
    assert!(
        rejection
            .source()
            .expect("attributed rejection retains provider diagnostic")
            .to_string()
            .contains("result-limit")
    );
}

#[test]
fn provider_diagnostics_are_bounded_before_they_enter_registry_errors() {
    assert_eq!(
        ScalarInferenceError::new(ProviderDiagnosticCode::new("empty").unwrap(), ""),
        Err(ProviderDiagnosticError::EmptyMessage)
    );
    let oversized = "x".repeat(MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES + 1);
    assert_eq!(
        ScalarInferenceError::new(ProviderDiagnosticCode::new("oversized").unwrap(), oversized),
        Err(ProviderDiagnosticError::MessageTooLong {
            bytes: MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES + 1,
        })
    );
}

#[test]
fn scalar_authority_closes_type_and_float_bits_dependencies_in_definition_values() {
    let mut semantic_builder = SemanticRegistryBuilder::new();
    semantic_builder.register_provider(&Types(1)).unwrap();
    let semantic = semantic_builder.freeze().unwrap();
    let facts = CanonicalValue::sequence([
        CanonicalValue::value_type(alternate_type()),
        CanonicalValue::float_bits(
            TypeKey::new("example", "alternate", 1).unwrap(),
            [0_u8, 0, 0, 0],
        )
        .unwrap(),
    ])
    .unwrap();
    let default_type = CanonicalValue::value_type(alternate_type());
    let conformance = CanonicalValue::value_type(alternate_type());
    let occurrence_bits = CanonicalValue::float_bits(
        TypeKey::new("example", "alternate", 1).unwrap(),
        [1_u8, 2, 3, 4],
    )
    .unwrap();
    let occurrence_attributes = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(
            AttributeFieldId::new(2),
            occurrence_bits.clone(),
        )])
        .unwrap(),
    )
    .unwrap();
    // Ad-hoc: the subject is the scalar authority closing type and float-bits
    // dependencies in definition *values*, which needs a definition carrying an
    // unclosed dependency -- something a governed definition never does.
    let mut scalar = ScalarRegistryBuilder::new(semantic.clone());
    scalar
        .register(
            ProviderIdentity::new("example", "dependency-scalar", 1).unwrap(),
            ScalarOperationDefinition::new(
                ScalarOpKey::new("example", "dependency_constant", 1).unwrap(),
                NormativeDefinitionRef::new("urn:example:dependency-constant:v1").unwrap(),
                ScalarOperationContract::new(
                    ScalarAttributeSchema::new([
                        ScalarAttributeField::defaulted(
                            AttributeFieldId::new(1),
                            CanonicalValueKind::Type,
                            default_type.clone(),
                        )
                        .unwrap(),
                        ScalarAttributeField::required(
                            AttributeFieldId::new(2),
                            CanonicalValueKind::FloatBits,
                        ),
                    ])
                    .unwrap(),
                    ScalarArity::exact(0).unwrap(),
                    ScalarArity::exact(1).unwrap(),
                    ScalarEffect::Pure,
                    facts.clone(),
                    conformance.clone(),
                ),
                Arc::new(Fixed(vec![test_type()])),
            ),
        )
        .unwrap();
    let registry = scalar.freeze();
    let mut builder = IndexRegionBuilder::new(registry.clone()).unwrap();
    let value = builder
        .apply(
            ScalarOpKey::new("example", "dependency_constant", 1).unwrap(),
            occurrence_attributes.clone(),
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap();
    scalar_output(&mut builder, value);
    let region = builder.build().unwrap();
    let evidence = registry.revalidate_region(&region).unwrap();
    let boundary_type = test_type();
    let expected = semantic
        .project_value_set_authority(
            [&boundary_type],
            [
                &facts,
                &default_type,
                &conformance,
                occurrence_attributes.value(),
            ],
        )
        .unwrap();
    assert_eq!(evidence.type_definitions(), expected.reached_definitions());
    assert_eq!(evidence.type_admission(), expected.admission_provenance());
}

/// Authors the same minimal single-constant output region used to compare the
/// manual and closure call sites. Returning `Result` lets both the ordinary
/// call site and the `build_with` closure share one identical authoring body.
fn author_constant_region(builder: &mut IndexRegionBuilder) -> Result<(), IndexBuildError> {
    let value = builder
        .apply(
            ScalarOpKey::new("example", "constant", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )?
        .get(0)
        .expect("the constant operation yields one result");
    let output = builder.tensor(TensorRole::Output, test_type(), Shape::from_dims([]))?;
    let write = builder.write(output, &[], &[])?;
    builder.output(write, value)?;
    Ok(())
}

#[test]
fn closure_convenience_matches_manual_construction() {
    let (_, registry) = registries();

    let mut manual = IndexRegionBuilder::new(registry.clone()).unwrap();
    author_constant_region(&mut manual).unwrap();
    let manual = manual.build().unwrap();

    let via_closure = IndexRegionBuilder::build_with(registry, author_constant_region).unwrap();

    // Canonical identity is owner-independent, so equal bytes prove the closure
    // path yields the same canonical region as manual construction plus build().
    assert_eq!(
        via_closure.canonical_identity().as_bytes(),
        manual.canonical_identity().as_bytes()
    );
    assert_eq!(via_closure.scalar_values().count(), 1);
    assert_eq!(via_closure.outputs().count(), 1);
}

#[test]
fn closure_admission_failure_surfaces_typed_error_without_verification() {
    let (_, registry) = registries();

    let error = IndexRegionBuilder::build_with(registry, |builder| {
        // Writing to an input boundary is an insertion-time admission failure;
        // whole-region verification is never reached.
        let input = builder.tensor(TensorRole::Input, test_type(), Shape::from_dims([]))?;
        builder.write(input, &[], &[])?;
        Ok(())
    })
    .unwrap_err();

    match error {
        CheckedBuildError::Admission(IndexBuildError::WriteToInput) => {}
        other => panic!("expected a WriteToInput admission failure, got {other:?}"),
    }
}

#[test]
fn closure_verification_failure_preserves_recoverable_diagnostic() {
    let (_, registry) = registries();

    // A region that authors a value but declares no output root fails whole-region
    // verification with the same recoverable diagnostic as the manual path.
    let error = IndexRegionBuilder::build_with(registry, |builder| {
        builder
            .apply(
                ScalarOpKey::new("example", "constant", 1).unwrap(),
                ScalarAttributes::empty(),
                &[],
            )
            .map(|_| ())
    })
    .unwrap_err();

    let CheckedBuildError::Verification(verification) = error else {
        panic!("expected a whole-region verification failure, got {error:?}");
    };
    assert_eq!(
        verification.diagnostics(),
        [IndexRegionDiagnostic::NoOutputs]
    );

    // The recoverable builder is intact and can be amended and rebuilt, proving
    // the verification variant erases neither the diagnostic nor the draft.
    let (mut recovered, diagnostics) = verification.into_parts();
    assert_eq!(diagnostics, [IndexRegionDiagnostic::NoOutputs]);
    let value = recovered
        .apply(
            ScalarOpKey::new("example", "constant", 1).unwrap(),
            ScalarAttributes::empty(),
            &[],
        )
        .unwrap()
        .get(0)
        .unwrap();
    scalar_output(&mut recovered, value);
    assert_eq!(recovered.build().unwrap().outputs().count(), 1);
}

/// The contraction access relation: two operand maps projecting away *different*
/// iteration coordinates.
///
/// `tiler::strict-tensor-contraction-f32@1`'s index structure `td,od->to` has
/// iteration domain `(t, o)`, reduction domain `(d)`, operand 0's map
/// `(t, o, d) -> (t, d)` never mentioning `o`, and operand 1's map
/// `(t, o, d) -> (o, d)` never mentioning `t`. Both are pure projections of the
/// coordinate vector, so the family needs no new access class — but no admitted
/// operation had ever produced two operand maps that drop *different* iteration
/// coordinates, and "needs no new class" is a claim about this vocabulary that
/// only emitting the region can settle. This test emits it.
///
/// Deliberately not a lowering capability. Registering one is a separate,
/// gated obligation; what is exercised here is that the index layer can express
/// the relation at all, and that both projections discharge their bounds without
/// enumeration.
#[test]
fn a_contraction_emits_two_operand_projections_dropping_different_coordinates() {
    let (_, registry) = registries();
    let mut builder = IndexRegionBuilder::new(registry).unwrap();

    // The B1-a prefill gate projection's extents, scaled down: `[T, D]` against
    // `[O, D]` producing `[T, O]`, contracting the LAST axis of both operands.
    let t = builder
        .dimension(DomainRole::Parallel, Extent::new(4))
        .unwrap();
    let o = builder
        .dimension(DomainRole::Parallel, Extent::new(6))
        .unwrap();
    let d = builder
        .dimension(DomainRole::Reduction, Extent::new(8))
        .unwrap();
    let activations = builder
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([4, 8]))
        .unwrap();
    let weights = builder
        .tensor(TensorRole::Input, test_type(), Shape::from_dims([6, 8]))
        .unwrap();
    let projected = builder
        .tensor(TensorRole::Output, test_type(), Shape::from_dims([4, 6]))
        .unwrap();

    let te = builder.dimension_expr(t).unwrap();
    let oe = builder.dimension_expr(o).unwrap();
    let de = builder.dimension_expr(d).unwrap();

    // The two maps. Operand 0 omits `o`; operand 1 omits `t`.
    let activation = builder.read(activations, &[t, d], &[te, de]).unwrap();
    let weight = builder.read(weights, &[o, d], &[oe, de]).unwrap();
    let product = builder
        .apply(
            ScalarOpKey::new("example", "binary", 1).unwrap(),
            ScalarAttributes::empty(),
            &[activation, weight],
        )
        .unwrap()
        .get(0)
        .unwrap();
    let init = constant_value(&mut builder);
    let contracted = builder
        .reduce(&[d], &[init], &[product], |body| {
            let accumulated = body.apply(
                ScalarOpKey::new("example", "binary", 1).unwrap(),
                ScalarAttributes::empty(),
                &[body.state(0).unwrap(), body.contributor(0).unwrap()],
            )?;
            body.yield_values(&[accumulated.get(0).unwrap()])
        })
        .unwrap()
        .get(0)
        .unwrap();
    let write = builder.write(projected, &[t, o], &[te, oe]).unwrap();
    builder.output(write, contracted).unwrap();
    let region = builder.build().unwrap();

    let accesses: Vec<_> = region.accesses().collect();
    assert_eq!(accesses.len(), 3);
    let domain_of = |position: usize| {
        accesses[position]
            .domain()
            .collect::<std::collections::BTreeSet<_>>()
    };
    let activation_domain = domain_of(0);
    let weight_domain = domain_of(1);
    assert_eq!(activation_domain.len(), 2);
    assert_eq!(weight_domain.len(), 2);
    assert_eq!(
        activation_domain.intersection(&weight_domain).count(),
        1,
        "the two operand maps share exactly the reduction coordinate"
    );
    assert_eq!(
        activation_domain.difference(&weight_domain).count(),
        1,
        "operand 0 keeps one iteration coordinate operand 1 drops"
    );
    assert_eq!(
        weight_domain.difference(&activation_domain).count(),
        1,
        "and operand 1 keeps one operand 0 drops -- which is the property no \
         previously admitted family had"
    );

    // Every coordinate is a bare dimension reference against an axis of equal
    // static extent, so both projections and the write discharge cheaply; a
    // contraction does not pay for an exhaustive proof.
    for access in &accesses[..2] {
        assert_eq!(access.bounds_proof(), Some(BoundsProofView::Interval));
    }
    assert_eq!(
        accesses[2].write_ownership_proof(),
        Some(WriteOwnershipProofView::CoordinatePermutation)
    );
}
