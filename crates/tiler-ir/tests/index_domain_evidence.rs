//! Public index-domain evidence custody tests.

use tiler_ir::index::{
    DomainRole, FrozenScalarRegistry, IndexDomainEvidence, IndexDomainPredicate,
    IndexDomainSoundProof, IndexDomainUnknownReason, IndexExtentRef, IndexRegionBuilder,
    ScalarRegistryBuilder, SourcedExtent, TensorRole, VerifiedIndexHandleError,
};
use tiler_ir::semantic::{
    AttributeFieldId, CanonicalField, CanonicalValue, EncodedNumericContract, F32,
    NormativeDefinitionRef, ProviderIdentity, QuantSchemeKey, RegistryError, ResolvedValueType,
    SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar, TypeArguments,
    TypeDefinitionFacts, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
};
use tiler_ir::shape::{Extent, Shape};

struct FutureValueTypes;

impl SemanticRegistryProvider for FutureValueTypes {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "future-value-types", 1).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        for name in ["bool", "i4", "u4"] {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("test", name, 1).unwrap()),
                NormativeDefinitionRef::from_owned(format!("urn:test:{name}:v1"))?,
                TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
            ))?;
        }
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Parameterized(TypeKey::new("test", "complex", 1).unwrap()),
            NormativeDefinitionRef::new("urn:test:complex:v1")?,
            TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
        ))?;
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::EncodedNumeric(
                QuantSchemeKey::new("test", "block-quantized", 1).unwrap(),
            ),
            NormativeDefinitionRef::new("urn:test:block-quantized:v1")?,
            TypeDefinitionFacts::new(CanonicalValue::record([]).unwrap()),
        ))
    }
}

fn future_value_types() -> (FrozenScalarRegistry, Vec<ResolvedValueType>) {
    let mut semantic = SemanticRegistryBuilder::standard().unwrap();
    semantic.register_provider(&FutureValueTypes).unwrap();
    let semantic = semantic.freeze().unwrap();
    let i4 = ResolvedValueType::nominal(TypeKey::new("test", "i4", 1).unwrap());
    let types = vec![
        ResolvedValueType::nominal(TypeKey::new("test", "bool", 1).unwrap()),
        i4.clone(),
        ResolvedValueType::nominal(TypeKey::new("test", "u4", 1).unwrap()),
        ResolvedValueType::parameterized(
            TypeKey::new("test", "complex", 1).unwrap(),
            TypeArguments::new([CanonicalValue::value_type(F32::resolved_type())]).unwrap(),
        )
        .unwrap(),
        ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("test", "block-quantized", 1).unwrap(),
            EncodedNumericContract::new([
                CanonicalField::new(AttributeFieldId::new(1), CanonicalValue::value_type(i4)),
                CanonicalField::new(
                    AttributeFieldId::new(2),
                    CanonicalValue::value_type(F32::resolved_type()),
                ),
            ])
            .unwrap(),
        )
        .unwrap(),
    ];
    (ScalarRegistryBuilder::new(semantic).freeze(), types)
}

fn verified_copy() -> tiler_ir::index::VerifiedIndexRegion {
    let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap()).unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(5))
        .unwrap();
    let coordinate = builder.dimension_expr(dimension).unwrap();
    let input = builder
        .tensor(
            TensorRole::Input,
            F32::resolved_type(),
            Shape::new([Extent::new(5)]),
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            F32::resolved_type(),
            Shape::new([Extent::new(5)]),
        )
        .unwrap();
    let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
    let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
    builder.output(write, value).unwrap();
    builder.build().unwrap()
}

#[test]
fn downstream_can_inspect_each_exact_discharged_predicate() {
    let region = verified_copy();
    let records = region
        .discharged_index_domain_predicates()
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 4);

    for access in region.accesses() {
        let expression = access.coordinates().next().unwrap();
        for predicate in [
            IndexDomainPredicate::NonNegative { expression },
            IndexDomainPredicate::LessThanExtent {
                expression,
                extent: IndexExtentRef::TensorAxis {
                    tensor: access.tensor(),
                    axis: 0,
                },
            },
        ] {
            let record = region
                .index_domain_evidence(access.id(), predicate)
                .unwrap()
                .expect("the verified copy discharges both coordinate bounds");
            assert_eq!(record.subject(), access.id());
            assert_eq!(record.predicate(), predicate);
            assert_eq!(
                record.evidence(),
                IndexDomainEvidence::SoundProof(IndexDomainSoundProof::Interval)
            );
        }
    }
}

#[test]
fn lookup_refuses_foreign_subjects_and_predicates() {
    let region = verified_copy();
    let foreign = verified_copy();
    let local_access = region.accesses().next().unwrap();
    let subject = local_access.id();
    let local_expression = local_access.coordinates().next().unwrap();
    let foreign_access = foreign.accesses().next().unwrap();
    let foreign_expression = foreign_access.coordinates().next().unwrap();

    assert!(matches!(
        region.index_domain_evidence(
            foreign_access.id(),
            IndexDomainPredicate::NonNegative {
                expression: local_expression,
            },
        ),
        Err(VerifiedIndexHandleError::ForeignRegion { .. })
    ));
    assert!(matches!(
        region.index_domain_evidence(
            subject,
            IndexDomainPredicate::NonNegative {
                expression: foreign_expression,
            },
        ),
        Err(VerifiedIndexHandleError::ForeignRegion { .. })
    ));
    assert!(matches!(
        region.index_domain_evidence(
            subject,
            IndexDomainPredicate::LessThanExtent {
                expression: local_expression,
                extent: IndexExtentRef::TensorAxis {
                    tensor: foreign_access.tensor(),
                    axis: 0,
                },
            },
        ),
        Err(VerifiedIndexHandleError::ForeignRegion { .. })
    ));
}

fn verified_budget_limited_copy(
    registry: FrozenScalarRegistry,
    value_type: ResolvedValueType,
) -> tiler_ir::index::VerifiedIndexRegion {
    let domain_extent = 199_999;
    let input_extent = 100_000;
    let mut builder = IndexRegionBuilder::new(registry).unwrap();
    let left_input = builder
        .tensor(
            TensorRole::Input,
            value_type.clone(),
            Shape::from_dims([input_extent]),
        )
        .unwrap();
    let right_input = builder
        .tensor(
            TensorRole::Input,
            value_type.clone(),
            Shape::from_dims([input_extent]),
        )
        .unwrap();
    let left_output = builder
        .tensor(
            TensorRole::Output,
            value_type.clone(),
            Shape::from_dims([domain_extent]),
        )
        .unwrap();
    let right_output = builder
        .tensor(
            TensorRole::Output,
            value_type,
            Shape::from_dims([domain_extent]),
        )
        .unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(domain_extent))
        .unwrap();
    let coordinate = builder.dimension_expr(dimension).unwrap();
    let two = SourcedExtent::Static(Extent::new(2));
    let modulo = builder.modulo(coordinate, two.clone()).unwrap();
    let quotient = builder.floor_div(coordinate, two).unwrap();
    let conservative = builder
        .linear_combination(
            0_i128.into(),
            &[(1_i128.into(), modulo), (1_i128.into(), quotient)],
        )
        .unwrap();
    for (input, output) in [(left_input, left_output), (right_input, right_output)] {
        let value = builder.read(input, &[dimension], &[conservative]).unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
    }
    builder
        .build()
        .expect("a proof-budget stop retains logical obligations for every value type")
}

#[test]
fn logical_coordinate_obligations_are_independent_of_value_type_family() {
    let (registry, value_types) = future_value_types();
    let mut identities = Vec::new();
    for value_type in value_types {
        let region = verified_budget_limited_copy(registry.clone(), value_type);
        let unknown = region.unknown_index_domain_predicates().collect::<Vec<_>>();
        assert_eq!(unknown.len(), 2);
        assert!(unknown.iter().all(|record| matches!(
            record.predicate(),
            IndexDomainPredicate::LessThanExtent { .. }
        )));
        assert!(unknown.iter().all(|record| matches!(
            record.reason(),
            IndexDomainUnknownReason::ResourceLimit { .. }
        )));
        assert_eq!(region.discharged_index_domain_predicates().count(), 6);
        identities.push(region.canonical_identity().as_bytes().to_vec());
    }
    identities.sort();
    identities.dedup();
    assert_eq!(
        identities.len(),
        5,
        "value type changes identity without changing coordinate obligations"
    );
}

const fn predicate_is_physical_guard(predicate: IndexDomainPredicate) -> bool {
    match predicate {
        IndexDomainPredicate::NonNegative { .. } | IndexDomainPredicate::LessThanExtent { .. } => {
            false
        }
    }
}

#[test]
fn unknown_lookup_is_exact_region_owned_and_contains_no_physical_guard() {
    let (registry, value_types) = future_value_types();
    let region = verified_budget_limited_copy(registry.clone(), value_types[0].clone());
    let foreign = verified_budget_limited_copy(registry, value_types[0].clone());
    let record = region
        .unknown_index_domain_predicates()
        .next()
        .expect("the fixture retains a read upper bound");
    let subject = record.subject();
    let unrelated_tensor = region
        .accesses()
        .find(|access| access.id() != subject && access.mode() == tiler_ir::index::AccessMode::Read)
        .expect("the fixture has a second read")
        .tensor();
    let IndexDomainPredicate::LessThanExtent { expression, .. } = record.predicate() else {
        panic!("the retained read predicate is its upper bound");
    };

    assert_eq!(
        region
            .index_domain_unknown(subject, record.predicate())
            .unwrap(),
        Some(record)
    );
    assert!(!predicate_is_physical_guard(record.predicate()));
    assert!(matches!(
        region.index_domain_unknown(foreign.accesses().next().unwrap().id(), record.predicate(),),
        Err(VerifiedIndexHandleError::ForeignRegion { .. })
    ));
    assert!(matches!(
        region.index_domain_unknown(
            subject,
            IndexDomainPredicate::LessThanExtent {
                expression,
                extent: IndexExtentRef::TensorAxis {
                    tensor: unrelated_tensor,
                    axis: 0,
                },
            },
        ),
        Err(VerifiedIndexHandleError::InvalidHandle { .. })
    ));
}
