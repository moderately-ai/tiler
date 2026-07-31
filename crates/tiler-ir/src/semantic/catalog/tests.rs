use std::collections::BTreeSet;

use super::*;
use crate::semantic::{
    AttributeFieldId, BuildError, EncodedComponentDeclaration, EncodedComponentRole,
    EncodedComponentShape, EncodedNumericContract, FrozenSemanticRegistry, InputKey,
    OperationAttributes, OutputKey, ParameterIndexMap, ProviderIdentity, SemanticProgramBuilder,
    SemanticRegistryBuilder, SemanticRegistryProvider, StrictAffineU4, U4, ValueFact, add_f32_op,
};
use crate::shape::Shape;

fn standard() -> FrozenSemanticRegistry {
    FrozenSemanticRegistry::standard().expect("the standard registry freezes")
}

fn nominal(name: &str) -> ResolvedValueType {
    governed_nominal_type(name)
}

fn definition_of(registry: &FrozenSemanticRegistry, name: &str) -> ValueTypeDefinition {
    registry
        .value_type_definition(&ValueTypeDefinitionKey::Nominal(governed_type_key(name)))
        .unwrap_or_else(|| panic!("{name} is a registered built-in identity"))
        .clone()
}

/// Every family key the catalog claims, in the order the tables state them.
fn catalog_definition_keys() -> Vec<ValueTypeDefinitionKey> {
    BUILT_IN_SCALARS
        .iter()
        .map(|scalar| ValueTypeDefinitionKey::Nominal(scalar.type_key()))
        .chain([ValueTypeDefinitionKey::Parameterized(
            complex_type_constructor(),
        )])
        .chain(
            MICROSCALING_SCHEMES
                .iter()
                .map(|scheme| ValueTypeDefinitionKey::EncodedNumeric(scheme.scheme_key())),
        )
        .collect()
}

fn field(definition: &ValueTypeDefinition, id: AttributeFieldId) -> Option<&CanonicalValue> {
    let CanonicalValueView::Record(fields) = definition.canonical_facts().value().view() else {
        panic!("a governed catalog descriptor is a record")
    };
    fields
        .iter()
        .find(|candidate| candidate.id() == id)
        .map(super::super::CanonicalField::value)
}

fn utf8_field(definition: &ValueTypeDefinition, id: AttributeFieldId) -> Option<String> {
    field(definition, id).map(|value| match value.view() {
        CanonicalValueView::Utf8(text) => text.to_owned(),
        other => panic!("field {id} is not UTF-8: {other:?}"),
    })
}

fn unsigned_field(definition: &ValueTypeDefinition, id: AttributeFieldId) -> Option<u64> {
    field(definition, id).map(|value| match value.view() {
        CanonicalValueView::Unsigned { bits, .. } => bits,
        other => panic!("field {id} is not unsigned: {other:?}"),
    })
}

fn signed_field(definition: &ValueTypeDefinition, id: AttributeFieldId) -> Option<i32> {
    field(definition, id).map(|value| match value.view() {
        CanonicalValueView::Signed { bits, .. } => {
            i32::try_from(bits).expect("a governed bias fits its declared width")
        }
        other => panic!("field {id} is not signed: {other:?}"),
    })
}

fn bool_field(definition: &ValueTypeDefinition, id: AttributeFieldId) -> Option<bool> {
    field(definition, id).map(|value| match value.view() {
        CanonicalValueView::Bool(flag) => flag,
        other => panic!("field {id} is not a Boolean: {other:?}"),
    })
}

fn class(definition: &ValueTypeDefinition) -> String {
    utf8_field(definition, SCALAR_TYPE_FACT_CLASS)
        .expect("every governed catalog descriptor states its class")
}

/// The accepted catalog is registered exactly, by canonical key and provenance.
///
/// This walks the tables rather than a hand-written expectation, so a row that
/// stops being registered fails here instead of quietly leaving the catalog.
#[test]
fn every_accepted_identity_is_registered_with_its_normative_reference() {
    let registry = standard();
    assert_eq!(
        BUILT_IN_SCALARS.len(),
        27,
        "the accepted scalar catalog is bool, twelve integers, four IEEE binary \
         floats, bf16, six OCP formats, and three IEEE decimals"
    );
    assert_eq!(MICROSCALING_SCHEMES.len(), 6);

    for scalar in BUILT_IN_SCALARS {
        let key = ValueTypeDefinitionKey::Nominal(scalar.type_key());
        let definition = registry
            .value_type_definition(&key)
            .unwrap_or_else(|| panic!("{} is an accepted built-in identity", scalar.name));
        assert_eq!(
            definition.normative_definition().as_str(),
            scalar.normative_definition
        );
        assert!(registry.contains(&nominal(scalar.name)));
    }

    let complex = registry
        .value_type_definition(&ValueTypeDefinitionKey::Parameterized(
            complex_type_constructor(),
        ))
        .expect("the complex family is an accepted built-in identity");
    assert_eq!(
        complex.normative_definition().as_str(),
        COMPLEX_NORMATIVE_DEFINITION
    );

    for scheme in MICROSCALING_SCHEMES {
        let definition = registry
            .value_type_definition(&ValueTypeDefinitionKey::EncodedNumeric(scheme.scheme_key()))
            .unwrap_or_else(|| panic!("{} is an accepted MX scheme identity", scheme.name));
        assert_eq!(
            definition.normative_definition().as_str(),
            scheme.normative_definition
        );
    }
}

/// The registered catalog is exactly the accepted names, stated independently.
///
/// Deliberately a second list. Every other test here walks `BUILT_IN_SCALARS`,
/// so a mistyped, dropped, or invented row would be registered and then
/// confirmed by its own table. This expectation is transcribed from the accepted
/// decisions instead — ADR 0028 for the predicate and integers, ADR 0036 for the
/// binary and OCP formats, ADR 0035 for the decimals, ADR 0037 for the complex
/// family, and ADR 0038 for the schemes — so a table typo fails here.
#[test]
fn the_registered_catalog_is_exactly_the_accepted_names() {
    let registry = standard();
    let mut accepted_scalars = vec![
        "bool",
        "i2",
        "i4",
        "i8",
        "i16",
        "i32",
        "i64",
        "u2",
        "u4",
        "u8",
        "u16",
        "u32",
        "u64",
        "f16",
        "f32",
        "f64",
        "f128",
        "bf16",
        "f8e4m3fn",
        "f8e5m2",
        "f6e2m3fn",
        "f6e3m2fn",
        "f4e2m1fn",
        "f8e8m0fnu",
        "decimal32",
        "decimal64",
        "decimal128",
    ];
    accepted_scalars.sort_unstable();
    let mut accepted_schemes = [
        "mxfp8_e4m3",
        "mxfp8_e5m2",
        "mxfp6_e2m3",
        "mxfp6_e3m2",
        "mxfp4_e2m1",
        "mxint8",
    ];
    accepted_schemes.sort_unstable();

    let mut nominal_names = Vec::new();
    let mut parameterized_names = Vec::new();
    let mut scheme_names = Vec::new();
    for definition in registry.value_type_definitions() {
        match definition.key() {
            ValueTypeDefinitionKey::Nominal(key) => nominal_names.push(key.to_string()),
            ValueTypeDefinitionKey::Parameterized(key) => parameterized_names.push(key.to_string()),
            ValueTypeDefinitionKey::EncodedNumeric(key) => scheme_names.push(key.to_string()),
        }
    }

    let expected_scalars: Vec<_> = accepted_scalars
        .iter()
        .map(|name| format!("tiler::{name}@1"))
        .collect();
    assert_eq!(nominal_names, expected_scalars);
    assert_eq!(parameterized_names, vec!["tiler::complex@1".to_owned()]);

    // The strict-affine proof scheme is a governed scheme this catalog does not
    // own, so it joins the accepted MX schemes here rather than being filtered
    // out — an unexpected extra scheme must still fail.
    let mut expected_schemes: Vec<_> = accepted_schemes
        .iter()
        .map(|name| format!("tiler::{name}@1"))
        .collect();
    expected_schemes.push("tiler::strict-affine@1".to_owned());
    expected_schemes.sort();
    assert_eq!(scheme_names, expected_schemes);
}

/// Every normative reference names the exact key it defines.
///
/// A copy-pasted row whose reference still cites its neighbour would otherwise
/// pass every table-walking check, because the table is what those compare
/// against. This derives the expectation from the key instead.
#[test]
fn every_normative_reference_names_its_own_key_and_a_pinned_authority() {
    let registry = standard();
    for definition in registry.value_type_definitions() {
        let (key, authority) = match definition.key() {
            ValueTypeDefinitionKey::Nominal(key) | ValueTypeDefinitionKey::Parameterized(key) => {
                (key.to_string(), key.namespace().to_owned())
            }
            ValueTypeDefinitionKey::EncodedNumeric(key) => {
                (key.to_string(), key.namespace().to_owned())
            }
        };
        assert_eq!(authority, "tiler", "the standard profile is Tiler-governed");
        let reference = definition.normative_definition().as_str();
        assert!(
            reference.contains(&key),
            "the normative reference for {key} does not name it: {reference}"
        );
        // Every reference resolves either to a preserved-source id from
        // `docs/research/numerics/sources/expected-sources.tsv` or to the
        // accepted decision that governs a Tiler-defined contract.
        assert!(
            reference.contains("source ")
                || reference.contains("ADR ")
                || reference.contains("ADRs "),
            "the normative reference for {key} pins no authority: {reference}"
        );
    }
}

/// Each descriptor carries its class, its alias policy, and exactly the
/// conditional fields its class declares.
#[test]
fn every_descriptor_carries_its_class_alias_policy_and_conditional_fields() {
    let registry = standard();
    let float_fields = [
        SCALAR_TYPE_FACT_SIGN_BITS,
        SCALAR_TYPE_FACT_EXPONENT_BITS,
        SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS,
        SCALAR_TYPE_FACT_HAS_INFINITIES,
        SCALAR_TYPE_FACT_HAS_NAN,
        SCALAR_TYPE_FACT_HAS_ZERO,
        SCALAR_TYPE_FACT_HAS_SIGNED_ZERO,
        SCALAR_TYPE_FACT_HAS_SUBNORMALS,
    ];

    for key in catalog_definition_keys() {
        let definition = registry
            .value_type_definition(&key)
            .expect("every catalog row is registered");
        assert_eq!(
            utf8_field(definition, SCALAR_TYPE_FACT_ALIAS_POLICY).as_deref(),
            Some(ALIAS_AND_EQUIVALENCE_POLICY),
            "{key:?} must carry the governed alias and equivalence policy"
        );
        let class = class(definition);
        let width = unsigned_field(definition, SCALAR_TYPE_FACT_WIDTH_BITS);
        match class.as_str() {
            "logical-predicate" => {
                assert!(width.is_none(), "a predicate has no logical width");
                assert_eq!(
                    unsigned_field(definition, SCALAR_TYPE_FACT_VALUE_CARDINALITY),
                    Some(2)
                );
            }
            "signed-integer" | "unsigned-integer" | "ieee-decimal" => {
                assert!(width.is_some(), "{class} states a logical width");
                for float_field in float_fields {
                    assert!(field(definition, float_field).is_none());
                }
            }
            "ieee-binary" | "bfloat" | "ocp-binary-element" | "ocp-exponent-scale" => {
                assert!(width.is_some());
                for float_field in float_fields {
                    assert!(
                        field(definition, float_field).is_some(),
                        "{class} states every binary floating-point parameter"
                    );
                }
            }
            "complex" | "ocp-microscaling-block-scheme" => {
                assert!(
                    width.is_none(),
                    "a compound identity's width follows its constituents"
                );
                assert!(field(definition, SCALAR_TYPE_FACT_COMPONENT_TYPES).is_some());
                assert!(field(definition, SCALAR_TYPE_FACT_COMPONENT_ORDER).is_some());
            }
            other => panic!("unclassified catalog descriptor {other}"),
        }
    }

    for (name, width, digits) in [
        ("decimal32", 32, 7),
        ("decimal64", 64, 16),
        ("decimal128", 128, 34),
    ] {
        let definition = definition_of(&registry, name);
        assert_eq!(
            unsigned_field(&definition, SCALAR_TYPE_FACT_WIDTH_BITS),
            Some(width),
            "{name} interchange width"
        );
        assert_eq!(
            unsigned_field(&definition, SCALAR_TYPE_FACT_COEFFICIENT_DIGITS),
            Some(digits),
            "{name} coefficient digits"
        );
    }
    for (name, width) in [
        ("bool", None),
        ("i2", Some(2)),
        ("i4", Some(4)),
        ("i8", Some(8)),
        ("i16", Some(16)),
        ("i32", Some(32)),
        ("i64", Some(64)),
        ("u2", Some(2)),
        ("u4", Some(4)),
        ("u8", Some(8)),
        ("u16", Some(16)),
        ("u32", Some(32)),
        ("u64", Some(64)),
    ] {
        assert_eq!(
            unsigned_field(&definition_of(&registry, name), SCALAR_TYPE_FACT_WIDTH_BITS),
            width,
            "{name} logical width"
        );
    }
    for (name, width, exponent, significand) in [
        ("f16", 16, 5, 10),
        ("f32", 32, 8, 23),
        ("f64", 64, 11, 52),
        ("f128", 128, 15, 112),
        ("bf16", 16, 8, 7),
        ("f8e4m3fn", 8, 4, 3),
        ("f8e5m2", 8, 5, 2),
        ("f6e2m3fn", 6, 2, 3),
        ("f6e3m2fn", 6, 3, 2),
        ("f4e2m1fn", 4, 2, 1),
        ("f8e8m0fnu", 8, 8, 0),
    ] {
        let definition = definition_of(&registry, name);
        assert_eq!(
            unsigned_field(&definition, SCALAR_TYPE_FACT_WIDTH_BITS),
            Some(width),
            "{name} width"
        );
        assert_eq!(
            unsigned_field(&definition, SCALAR_TYPE_FACT_EXPONENT_BITS),
            Some(exponent),
            "{name} exponent bits"
        );
        assert_eq!(
            unsigned_field(&definition, SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS),
            Some(significand),
            "{name} trailing significand bits"
        );
    }

    // The one conditional field whose absence is an evidence boundary rather
    // than a property of the format: the OCP rows state a bias, and the IEEE
    // and BF16 rows leave it to their pinned normative reference.
    for (name, bias) in [
        ("f8e4m3fn", Some(7_i32)),
        ("f8e5m2", Some(15)),
        ("f6e2m3fn", Some(1)),
        ("f6e3m2fn", Some(3)),
        ("f4e2m1fn", Some(1)),
        ("f8e8m0fnu", Some(127)),
        ("f16", None),
        ("f32", None),
        ("f64", None),
        ("f128", None),
        ("bf16", None),
    ] {
        assert_eq!(
            signed_field(
                &definition_of(&registry, name),
                SCALAR_TYPE_FACT_EXPONENT_BIAS
            ),
            bias,
            "{name} exponent bias"
        );
    }
}

/// The exponent-only scale format is not an ordinary arithmetic element.
#[test]
fn the_ocp_scale_format_states_its_restricted_value_set() {
    let registry = standard();
    let definition = definition_of(&registry, "f8e8m0fnu");
    assert_eq!(class(&definition), "ocp-exponent-scale");
    assert_eq!(
        unsigned_field(&definition, SCALAR_TYPE_FACT_SIGN_BITS),
        Some(0),
        "scale data is unsigned"
    );
    for (id, expected) in [
        (SCALAR_TYPE_FACT_HAS_ZERO, false),
        (SCALAR_TYPE_FACT_HAS_SIGNED_ZERO, false),
        (SCALAR_TYPE_FACT_HAS_INFINITIES, false),
        (SCALAR_TYPE_FACT_HAS_SUBNORMALS, false),
        (SCALAR_TYPE_FACT_HAS_NAN, true),
    ] {
        assert_eq!(bool_field(&definition, id), Some(expected), "field {id}");
    }

    // The OCP element formats it scales are ordinary signed elements, so the
    // scale's class must not be shared with them.
    for element in ["f8e4m3fn", "f8e5m2", "f6e2m3fn", "f6e3m2fn", "f4e2m1fn"] {
        let element = definition_of(&registry, element);
        assert_eq!(class(&element), "ocp-binary-element");
        assert_eq!(
            unsigned_field(&element, SCALAR_TYPE_FACT_SIGN_BITS),
            Some(1)
        );
    }
}

/// Descriptors are pairwise distinct and reproducible from the definition.
#[test]
fn every_descriptor_has_a_distinct_reproducible_fingerprint() {
    let registry = standard();
    let mut fingerprints = BTreeSet::new();
    for definition in registry.value_type_definitions() {
        let descriptor = definition.canonical_descriptor();
        assert_eq!(descriptor, definition.canonical_descriptor());
        assert!(
            descriptor
                .as_bytes()
                .starts_with(b"tiler.value-type-descriptor.v1\0"),
            "a descriptor carries its own versioned domain separator"
        );
        assert!(
            fingerprints.insert(descriptor.as_bytes().to_vec()),
            "two registered definitions share one descriptor"
        );
    }
    assert_eq!(fingerprints.len(), registry.value_type_definitions().len());
    assert!(
        fingerprints.len() > 1,
        "a single-entry registry would make distinctness vacuous"
    );

    // A descriptor is the key, the normative reference, and the facts together:
    // changing any one of the three must move it.
    let source = definition_of(&registry, "f32");
    let base = source.canonical_descriptor();
    let renamed = ValueTypeDefinition::structurally_valid(
        ValueTypeDefinitionKey::Nominal(governed_type_key("f64")),
        source.normative_definition().clone(),
        source.canonical_facts().clone(),
    );
    let reworded = ValueTypeDefinition::structurally_valid(
        source.key().clone(),
        NormativeDefinitionRef::new("a different normative reference").unwrap(),
        source.canonical_facts().clone(),
    );
    let refactored = ValueTypeDefinition::structurally_valid(
        source.key().clone(),
        source.normative_definition().clone(),
        TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
    );
    for changed in [renamed, reworded, refactored] {
        assert_ne!(base, changed.canonical_descriptor());
    }
}

/// The frozen registry iterates definitions in canonical key order.
#[test]
fn registered_definitions_are_iterated_in_canonical_key_order() {
    let registry = standard();
    let observed: Vec<_> = registry
        .value_type_definitions()
        .map(|definition| definition.key().clone())
        .collect();
    let mut sorted = observed.clone();
    sorted.sort();
    assert_eq!(observed, sorted);

    // Every catalog key appears exactly once, and nominal keys precede the
    // parameterized family, which precedes the encoded schemes.
    for key in catalog_definition_keys() {
        assert_eq!(
            observed
                .iter()
                .filter(|candidate| *candidate == &key)
                .count(),
            1,
            "{key:?} is registered exactly once"
        );
    }
    let complex_position = observed
        .iter()
        .position(|key| matches!(key, ValueTypeDefinitionKey::Parameterized(_)))
        .expect("the complex family is registered");
    let first_scheme = observed
        .iter()
        .position(|key| matches!(key, ValueTypeDefinitionKey::EncodedNumeric(_)))
        .expect("encoded schemes are registered");
    assert!(complex_position < first_scheme);
    assert!(
        observed[..complex_position]
            .iter()
            .all(|key| matches!(key, ValueTypeDefinitionKey::Nominal(_)))
    );

    // The public enumerations are documented as canonical key order, and a
    // caller cannot read a catalog the registry does not hold. Both halves are
    // asserted: membership alone would let the stated order drift unchecked.
    let enumerated = builtin_scalar_value_types();
    for value_type in &enumerated {
        assert!(registry.contains(value_type));
    }
    let mut sorted_enumeration = enumerated.clone();
    sorted_enumeration.sort();
    assert_eq!(
        enumerated, sorted_enumeration,
        "the public scalar enumeration is documented as canonical key order"
    );
    let registered_scalars: Vec<_> = observed
        .iter()
        .filter_map(|key| match key {
            ValueTypeDefinitionKey::Nominal(key) => Some(ResolvedValueType::nominal(key.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(enumerated, registered_scalars);

    let schemes = microscaling_scheme_keys();
    let mut sorted_schemes = schemes.clone();
    sorted_schemes.sort();
    assert_eq!(
        schemes, sorted_schemes,
        "the public scheme enumeration is documented as canonical key order"
    );
    assert_eq!(schemes.len(), MICROSCALING_SCHEMES.len());
}

/// A second registration of the same catalog is a duplicate-authority failure.
#[test]
fn registering_the_catalog_twice_is_a_duplicate_authority_failure() {
    struct DoubleCatalog;

    impl SemanticRegistryProvider for DoubleCatalog {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("test", "double-catalog", 1).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            register_builtin_dtype_catalog(registrar)?;
            register_builtin_dtype_catalog(registrar)
        }
    }

    let mut builder = SemanticRegistryBuilder::new();
    let error = builder.register_provider(&DoubleCatalog).unwrap_err();
    let RegistryError::DuplicateTypeAuthority { key } = error else {
        panic!("a repeated catalog row must collide by its exact family key")
    };
    assert_eq!(
        *key,
        ValueTypeDefinitionKey::Nominal(governed_type_key(BUILT_IN_SCALARS[0].name))
    );
}

/// The complex family admits exactly its accepted components.
#[test]
fn complex_admits_exactly_the_accepted_components() {
    let registry = standard();
    assert_eq!(
        admitted_complex_component_types(),
        vec![nominal("f16"), nominal("f32"), nominal("f64")]
    );

    let mut encodings = BTreeSet::new();
    for component in admitted_complex_component_types() {
        let value =
            complex_value_type(&component).expect("an admitted complex instance is bounded");
        registry
            .validate_type(&value)
            .expect("an admitted complex instance has authority");
        let (constructor, arguments) = value
            .parameterized_parts()
            .expect("complex is a parameterized identity");
        assert_eq!(constructor, &complex_type_constructor());
        assert_eq!(arguments.values().len(), 1);
        assert!(encodings.insert(value.canonical_encoding().as_bytes().to_vec()));
    }
    assert_eq!(encodings.len(), 3, "component identity separates instances");
}

/// Every unadmitted component and malformed argument list is refused by reason.
#[test]
fn complex_refuses_unadmitted_components_and_malformed_arguments() {
    let registry = standard();
    let reject = |value: &ResolvedValueType, expected: &str| {
        let Err(RegistryError::RejectedTypeInstance(rejection)) = registry.validate_type(value)
        else {
            panic!("{value:?} must be refused by the complex family")
        };
        assert_eq!(rejection.source_error().code().as_str(), expected);
    };

    // Recognized real formats that ADR 0037 has not admitted as components.
    for component in ["bf16", "f128", "f8e4m3fn", "decimal64", "i8", "bool"] {
        reject(
            &complex_value_type(&nominal(component)).unwrap(),
            "complex.unsupported-component",
        );
    }
    // Nested complex is structurally possible and semantically unadmitted.
    reject(
        &complex_value_type(&complex_value_type(&nominal("f32")).unwrap()).unwrap(),
        "complex.unsupported-component",
    );
    // A second argument, and an argument that is not a type at all.
    reject(
        &ResolvedValueType::parameterized(
            complex_type_constructor(),
            TypeArguments::new([
                CanonicalValue::value_type(nominal("f32")),
                CanonicalValue::value_type(nominal("f32")),
            ])
            .unwrap(),
        )
        .unwrap(),
        "complex.arity",
    );
    reject(
        &ResolvedValueType::parameterized(
            complex_type_constructor(),
            TypeArguments::new([CanonicalValue::unsigned_u32(32)]).unwrap(),
        )
        .unwrap(),
        "complex.argument-kind",
    );
}

fn plausible_microscaling_contract(scheme: &MicroscalingScheme) -> ResolvedValueType {
    ResolvedValueType::encoded_numeric(
        scheme.scheme_key(),
        EncodedNumericContract::with_components(
            [
                CanonicalField::new(
                    AttributeFieldId::new(1),
                    CanonicalValue::value_type(nominal(scheme.element)),
                ),
                CanonicalField::new(
                    AttributeFieldId::new(2),
                    CanonicalValue::value_type(nominal(MICROSCALING_SCALE_TYPE)),
                ),
            ],
            [
                EncodedComponentDeclaration::new(
                    EncodedComponentRole::new(1),
                    nominal(scheme.element),
                    EncodedComponentShape::LogicalValue,
                ),
                EncodedComponentDeclaration::new(
                    EncodedComponentRole::new(2),
                    nominal(MICROSCALING_SCALE_TYPE),
                    EncodedComponentShape::ParameterMap(ParameterIndexMap::per_tensor()),
                ),
            ],
        )
        .unwrap(),
    )
    .unwrap()
}

/// MX identities are recognized and refuse every contract, by reason.
///
/// The distinction this pins is ADR 0026's: an unknown scheme key fails as
/// missing authority, while an admitted one fails as an unsupported contract.
/// Collapsing the two would report a standardized identity as unknown.
#[test]
fn microscaling_schemes_are_recognized_and_refuse_every_static_contract() {
    let registry = standard();
    for scheme in MICROSCALING_SCHEMES {
        let key = ValueTypeDefinitionKey::EncodedNumeric(scheme.scheme_key());
        assert!(registry.value_type_definition(&key).is_some());

        let value = plausible_microscaling_contract(scheme);
        let Err(RegistryError::RejectedTypeInstance(rejection)) = registry.validate_type(&value)
        else {
            panic!("{} must refuse a per-tensor scale association", scheme.name)
        };
        assert_eq!(
            rejection.source_error().code().as_str(),
            "microscaling.unsupported-contract"
        );
        assert_eq!(rejection.key(), &key);
    }

    // An unregistered scheme spelling fails as missing authority instead.
    let unknown = ResolvedValueType::encoded_numeric(
        QuantSchemeKey::new("tiler", "mxfp4_e3m0", 1).unwrap(),
        EncodedNumericContract::new([CanonicalField::new(
            AttributeFieldId::new(1),
            CanonicalValue::boolean(true),
        )])
        .unwrap(),
    )
    .unwrap();
    assert!(matches!(
        registry.validate_type(&unknown),
        Err(RegistryError::UnregisteredTypeAuthority { .. })
    ));
}

/// Every recognized-but-unsupported identity is refused by every operation.
///
/// The reachable boundary here is [`FrozenSemanticRegistry::infer_operation`],
/// the only path by which a value type acquires an operation signature. The
/// operation set is read from the frozen registry rather than listed, so a
/// newly registered operation is covered without editing this test.
#[test]
fn no_registered_operation_admits_a_recognized_but_unsupported_identity() {
    let registry = standard();
    let operations: Vec<_> = registry
        .operation_definitions()
        .map(|definition| definition.key().clone())
        .collect();
    assert!(
        operations.len() >= 7,
        "the standard operation set is loaded"
    );

    let subjects = [
        nominal("bool"),
        nominal("i8"),
        nominal("u16"),
        nominal("f16"),
        nominal("bf16"),
        nominal("f64"),
        nominal("f8e4m3fn"),
        nominal("f8e8m0fnu"),
        nominal("decimal64"),
        complex_value_type(&nominal("f32")).unwrap(),
    ];
    for subject in &subjects {
        assert!(
            registry.contains(subject),
            "the subject must be recognized for this to be a support check"
        );
        for operation in &operations {
            for arity in 0..=3_usize {
                let operands: Vec<_> = std::iter::repeat_n(
                    ValueFact::new(subject.clone(), Shape::from_dims([2])),
                    arity,
                )
                .collect();
                let result =
                    registry.infer_operation(operation, &operands, &OperationAttributes::empty());
                assert!(
                    result.is_err(),
                    "{operation} admitted {subject:?} at arity {arity}"
                );
            }
        }
    }
}

/// A recognized identity may be a program interface and still no operand.
///
/// The reachable boundary here is program construction: `input_resolved` and
/// `output_resolved` carry the identity, while `apply` refuses it. This is
/// exactly what ADR 0026 means by representable without operation support.
#[test]
fn a_recognized_identity_is_representable_and_still_rejected_by_apply() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let value = builder
        .input_resolved(
            InputKey::new("half").unwrap(),
            Shape::from_dims([4]),
            nominal("f16"),
        )
        .expect("a recognized identity is representable at the interface");
    let error = builder
        .apply(add_f32_op(), OperationAttributes::empty(), &[value, value])
        .expect_err("no operation admits f16");
    let BuildError::SemanticRegistry(RegistryError::RejectedOperationApplication(rejection)) =
        &error
    else {
        panic!(
            "the refusal must be a typed operation rejection, not a panic or silent cast: {error:?}"
        )
    };
    assert_eq!(rejection.key(), &add_f32_op());
    assert_eq!(rejection.source_error().code().as_str(), "binary.type");
    builder
        .output_resolved(OutputKey::new("half").unwrap(), value)
        .unwrap();
    let program = builder.build().expect("an f16 interface program is valid");
    assert_eq!(program.operation_count(), 0);
}

/// Alias spellings and lookalike identities are not registered identities.
///
/// ADR 0034 keeps a published key immutable and forbids a similar spelling or
/// width from establishing equivalence, so each name below must fail as missing
/// authority rather than resolve to a neighbouring catalog row.
#[test]
fn alias_spellings_and_lookalikes_have_no_authority() {
    let registry = standard();
    for name in [
        // Frontend width-based and shorthand spellings (ADR 0037, taxonomy).
        "complex32",
        "complex64",
        "complex128",
        "chalf",
        "cfloat",
        "cdouble",
        "half",
        "float",
        "double",
        "byte",
        // Deliberately outside logical identity (ADR 0036, taxonomy).
        "tf32",
        "x86_fp80",
        "ppc_fp128",
        // External owner-namespaced candidates (ADR 0034).
        "f8e4m3fnuz",
        "f8e5m2fnuz",
        "f8e4m3b11fnuz",
        "f8e3m4",
        "f8e4m3",
        // Extension-only families (dtype identity admission policy).
        "i128",
        "u128",
        "posit8",
        "decimal256",
        // Integer i1 is a different concept from the logical predicate.
        "i1",
    ] {
        let key = ValueTypeDefinitionKey::Nominal(governed_type_key(name));
        assert!(
            registry.value_type_definition(&key).is_none(),
            "{name} must not be a registered built-in identity"
        );
        assert!(matches!(
            registry.validate_type(&nominal(name)),
            Err(RegistryError::UnregisteredTypeAuthority { .. })
        ));
    }

    // A version the catalog never published is a different key, not this one.
    assert!(matches!(
        registry.validate_type(&ResolvedValueType::nominal(
            TypeKey::new("tiler", "f16", 2).unwrap()
        )),
        Err(RegistryError::UnregisteredTypeAuthority { .. })
    ));
    // An owner-namespaced identity with the same name is a different identity.
    assert!(matches!(
        registry.validate_type(&ResolvedValueType::nominal(
            TypeKey::new("acme", "f16", 1).unwrap()
        )),
        Err(RegistryError::UnregisteredTypeAuthority { .. })
    ));
}

/// The catalog keeps the distinctions its accepted decisions require.
#[test]
fn the_catalog_keeps_its_required_distinctions_separate() {
    let registry = standard();

    // A logical predicate is not an integer, and states no logical width.
    let boolean = definition_of(&registry, "bool");
    assert_eq!(class(&boolean), "logical-predicate");
    assert!(field(&boolean, SCALAR_TYPE_FACT_WIDTH_BITS).is_none());
    for integer in ["i2", "u2"] {
        assert_eq!(
            unsigned_field(
                &definition_of(&registry, integer),
                SCALAR_TYPE_FACT_WIDTH_BITS
            ),
            Some(2)
        );
    }

    // A plain integer is not a quantized code: the nominal u4 and the
    // strict-affine encoded value over u4 are different resolved identities
    // governed by different families.
    assert_ne!(U4::resolved_type(), StrictAffineU4::resolved_type());
    assert_ne!(
        registry
            .definition(&U4::resolved_type())
            .expect("u4 is registered")
            .key(),
        registry
            .definition(&StrictAffineU4::resolved_type())
            .expect("the strict-affine scheme is registered")
            .key()
    );

    // Logical width separates the sub-byte integers from the byte-wide one, and
    // no descriptor names a packing.
    let widths: BTreeSet<_> = ["u2", "u4", "u8"]
        .into_iter()
        .map(|name| {
            definition_of(&registry, name)
                .canonical_descriptor()
                .as_bytes()
                .to_vec()
        })
        .collect();
    assert_eq!(widths.len(), 3);

    // An MX element format is not its MX scheme.
    assert!(registry.contains(&nominal("f4e2m1fn")));
    assert_ne!(
        ValueTypeDefinitionKey::Nominal(governed_type_key("f4e2m1fn")),
        ValueTypeDefinitionKey::EncodedNumeric(
            QuantSchemeKey::new("tiler", "mxfp4_e2m1", 1).unwrap()
        )
    );
}
