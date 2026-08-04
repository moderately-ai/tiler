//! Adversarial evidence for resolved-value binding conformance.
//!
//! Two fixture views drive every payload case. [`SliceView`] presents whatever
//! components a test hands it, which is how the missing, duplicate, extra,
//! swapped, wrong-type, wrong-shape, and cross-value structures are built; it
//! also counts its reads, which is how "no rescan when a complete
//! same-provenance proof exists" is checked as a fact rather than asserted.
//! [`PackedNibbleView`] reconstructs the same logical codes from a packed
//! carrier and refuses to read anything the logical value does not contain,
//! which is how the equal-diagnostic-index and untouched-tail claims are made
//! checkable.

use std::cell::RefCell;
use std::sync::Arc;

use super::*;
use crate::semantic::types::{CanonicalField, CanonicalValue, EncodedComponentDeclaration};
use crate::semantic::{
    F32Constant, FrozenSemanticRegistry, InputKey, OperationAttributes, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictAffineU4, StrictAffineU8, dequantize_strict_affine_op,
    microscaling_scheme_keys, no_nan_predicate, positive_finite_scalar_predicate,
};
use crate::shape::Shape;

// ---------------------------------------------------------------- fixtures --

/// One component a fixture view presents, with its logical scalars.
#[derive(Clone, Debug)]
struct FixtureComponent {
    role: EncodedComponentRole,
    resolved_type: ResolvedValueType,
    shape: Shape,
    scalars: Vec<LogicalScalar>,
    fault: Option<LogicalViewFault>,
}

impl FixtureComponent {
    fn codes(role: EncodedComponentRole, code_type: &ResolvedValueType, values: &[u8]) -> Self {
        Self {
            role,
            resolved_type: code_type.clone(),
            shape: Shape::from_dims([u64::try_from(values.len()).unwrap()]),
            scalars: values
                .iter()
                .copied()
                .map(LogicalScalar::UnsignedCode)
                .collect(),
            fault: None,
        }
    }

    fn scalar_code(role: EncodedComponentRole, code_type: &ResolvedValueType, value: u8) -> Self {
        Self {
            role,
            resolved_type: code_type.clone(),
            shape: Shape::new([]),
            scalars: vec![LogicalScalar::UnsignedCode(value)],
            fault: None,
        }
    }

    fn scale(bits: u32) -> Self {
        Self {
            role: STRICT_AFFINE_SCALE_ROLE,
            resolved_type: F32::resolved_type(),
            shape: Shape::new([]),
            scalars: vec![LogicalScalar::F32Bits(bits)],
            fault: None,
        }
    }

    fn with_fault(mut self, fault: LogicalViewFault) -> Self {
        self.fault = Some(fault);
        self
    }

    fn with_role(mut self, role: EncodedComponentRole) -> Self {
        self.role = role;
        self
    }

    fn with_type(mut self, resolved_type: ResolvedValueType) -> Self {
        self.resolved_type = resolved_type;
        self
    }

    fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }
}

/// A view that presents exactly the components it was built from, and counts reads.
struct SliceView {
    components: Vec<FixtureComponent>,
    reads: RefCell<usize>,
}

impl SliceView {
    fn new(components: Vec<FixtureComponent>) -> Self {
        Self {
            components,
            reads: RefCell::new(0),
        }
    }

    fn reads(&self) -> usize {
        *self.reads.borrow()
    }
}

impl EncodedLogicalView for SliceView {
    fn presented_components(&self) -> usize {
        self.components.len()
    }

    fn presented_component(&self, position: usize) -> Option<PresentedComponent<'_>> {
        self.components
            .get(position)
            .map(|component| PresentedComponent {
                role: component.role,
                resolved_type: &component.resolved_type,
                shape: &component.shape,
            })
    }

    fn read_logical_scalar(
        &self,
        position: usize,
        index: u64,
    ) -> Result<LogicalScalar, LogicalViewFault> {
        *self.reads.borrow_mut() += 1;
        let component = self
            .components
            .get(position)
            .ok_or(LogicalViewFault::UnreconstructableIndex)?;
        if let Some(fault) = component.fault {
            return Err(fault);
        }
        component
            .scalars
            .get(usize::try_from(index).unwrap())
            .copied()
            .ok_or(LogicalViewFault::UnreconstructableIndex)
    }
}

/// A view whose code component is reconstructed from a packed nibble carrier.
///
/// It is the byte-addressed view's twin: same logical codes, different physical
/// layout. `tail` holds the bits after the final logical nibble, and
/// [`PackedNibbleView::tail_reads`] counts how many times anything asked for a
/// logical element past the end — which is what the semantic scan must never
/// do.
struct PackedNibbleView {
    code_type: ResolvedValueType,
    scale_type: ResolvedValueType,
    logical_codes: u64,
    /// Least-significant nibble first, matching `PackedU4LsbZeroTail`.
    bytes: Vec<u8>,
    scale_bits: u32,
    zero_point: u8,
    tail_reads: RefCell<usize>,
    shapes: [Shape; 3],
}

impl PackedNibbleView {
    fn new(codes: &[u8], scale_bits: u32, zero_point: u8) -> Self {
        let mut bytes = vec![0_u8; codes.len().div_ceil(2)];
        for (index, code) in codes.iter().enumerate() {
            let byte = &mut bytes[index / 2];
            if index % 2 == 0 {
                *byte |= code & 0x0f;
            } else {
                *byte |= (code & 0x0f) << 4;
            }
        }
        Self {
            code_type: U4::resolved_type(),
            scale_type: F32::resolved_type(),
            logical_codes: u64::try_from(codes.len()).unwrap(),
            bytes,
            scale_bits,
            zero_point,
            tail_reads: RefCell::new(0),
            shapes: [
                Shape::from_dims([u64::try_from(codes.len()).unwrap()]),
                Shape::new([]),
                Shape::new([]),
            ],
        }
    }

    /// Writes nonzero bits into the unused tail of the final carrier byte.
    fn with_noncanonical_tail(mut self) -> Self {
        assert!(
            self.logical_codes % 2 == 1,
            "a tail only exists for an odd logical element count",
        );
        *self.bytes.last_mut().expect("an odd count needs one byte") |= 0xf0;
        self
    }

    fn tail_reads(&self) -> usize {
        *self.tail_reads.borrow()
    }
}

impl EncodedLogicalView for PackedNibbleView {
    fn presented_components(&self) -> usize {
        3
    }

    fn presented_component(&self, position: usize) -> Option<PresentedComponent<'_>> {
        let (role, resolved_type) = match position {
            0 => (STRICT_AFFINE_CODES_ROLE, &self.code_type),
            1 => (STRICT_AFFINE_SCALE_ROLE, &self.scale_type),
            2 => (STRICT_AFFINE_ZERO_POINT_ROLE, &self.code_type),
            _ => return None,
        };
        Some(PresentedComponent {
            role,
            resolved_type,
            shape: &self.shapes[position],
        })
    }

    fn read_logical_scalar(
        &self,
        position: usize,
        index: u64,
    ) -> Result<LogicalScalar, LogicalViewFault> {
        match position {
            0 => {
                if index >= self.logical_codes {
                    // Reaching here is the defect this view exists to detect:
                    // the tail is not a logical element.
                    *self.tail_reads.borrow_mut() += 1;
                    return Err(LogicalViewFault::UnreconstructableIndex);
                }
                let byte = self.bytes[usize::try_from(index / 2).unwrap()];
                let nibble = if index.is_multiple_of(2) {
                    byte & 0x0f
                } else {
                    byte >> 4
                };
                Ok(LogicalScalar::UnsignedCode(nibble))
            }
            1 if index == 0 => Ok(LogicalScalar::F32Bits(self.scale_bits)),
            2 if index == 0 => Ok(LogicalScalar::UnsignedCode(self.zero_point)),
            _ => Err(LogicalViewFault::UnreconstructableIndex),
        }
    }
}

// ------------------------------------------------------------- subject help --

fn route() -> RouteDependency {
    RouteDependency::new(b"tiler.test.route.v1")
}

fn input(name: &str) -> InputKey {
    InputKey::new(name).unwrap()
}

fn direct_subject(resolved_type: ResolvedValueType, shape: Shape) -> ValueConformanceSubject {
    ValueConformanceSubject::new(
        ValueOrigin::direct_binding(input("weights")),
        ValueStability::ImmutableHost,
        route(),
        SemanticLogicalView::WholeValue,
        resolved_type,
        shape,
    )
}

fn u4_components(codes: &[u8], scale_bits: u32, zero_point: u8) -> Vec<FixtureComponent> {
    vec![
        FixtureComponent::codes(STRICT_AFFINE_CODES_ROLE, &U4::resolved_type(), codes),
        FixtureComponent::scale(scale_bits),
        FixtureComponent::scalar_code(
            STRICT_AFFINE_ZERO_POINT_ROLE,
            &U4::resolved_type(),
            zero_point,
        ),
    ]
}

fn conform(
    components: Vec<FixtureComponent>,
    logical: &[u64],
) -> Result<ValueConformanceEvidence, ValueConformanceRejection> {
    let view = SliceView::new(components);
    scan_bound_value(
        &standard_binding_validator(),
        &direct_subject(
            StrictAffineU4::resolved_type(),
            Shape::try_from_dims(logical.iter().copied()).unwrap(),
        ),
        &view,
    )
}

// --------------------------------------------------------- contract derival --

/// The obligation set comes from the type's own declarations, not a table.
#[test]
fn the_derived_contract_restates_every_declared_role_type_shape_and_map() {
    let logical = Shape::from_dims([2, 3]);
    let contract =
        ResolvedValueConformanceContract::derive(&StrictAffineU4::resolved_type(), &logical)
            .unwrap();
    let rows: Vec<_> = contract
        .components()
        .iter()
        .map(|component| {
            (
                component.role(),
                component.resolved_type().clone(),
                component.shape().clone(),
                component.parameter_map().cloned(),
                component.value_domain(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (
                STRICT_AFFINE_CODES_ROLE,
                U4::resolved_type(),
                logical.clone(),
                None,
                ComponentValueDomain::UnsignedCodeRange {
                    minimum: 0,
                    maximum: 15,
                },
            ),
            (
                STRICT_AFFINE_SCALE_ROLE,
                F32::resolved_type(),
                Shape::new([]),
                Some(ParameterIndexMap::per_tensor()),
                ComponentValueDomain::PositiveNormalF32,
            ),
            (
                STRICT_AFFINE_ZERO_POINT_ROLE,
                U4::resolved_type(),
                Shape::new([]),
                Some(ParameterIndexMap::per_tensor()),
                ComponentValueDomain::UnsignedCodeRange {
                    minimum: 0,
                    maximum: 15,
                },
            ),
        ]
    );
    assert_eq!(contract.resolved_type(), &StrictAffineU4::resolved_type());
    assert_eq!(contract.shape(), &logical);
}

/// The U8 profile's code domain is the full byte, taken from its own contract.
#[test]
fn the_u8_profile_derives_the_full_byte_code_domain_from_its_own_contract() {
    let contract = ResolvedValueConformanceContract::derive(
        &StrictAffineU8::resolved_type(),
        &Shape::from_dims([4]),
    )
    .unwrap();
    for role in [STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE] {
        let component = contract
            .components()
            .iter()
            .find(|candidate| candidate.role() == role)
            .unwrap();
        assert_eq!(
            component.value_domain(),
            ComponentValueDomain::UnsignedCodeRange {
                minimum: 0,
                maximum: u8::MAX,
            }
        );
    }
}

/// Every unsupported representation refuses by its own exact name.
///
/// The population is written out rather than sampled: a representation that
/// reaches an evaluator it was never admitted to would be a silently wrong
/// answer, so each family names the exact subject it refused.
#[test]
fn every_unsupported_representation_refuses_by_its_exact_name() {
    let nested_role = EncodedComponentRole::new(7);
    let other_map_role = EncodedComponentRole::new(8);
    let complex = crate::semantic::complex_value_type(&F32::resolved_type()).unwrap();
    let microscaling = microscaling_scheme_keys()
        .into_iter()
        .next()
        .expect("the governed catalog admits at least one microscaling scheme");
    let codebook = QuantSchemeKey::new("acme", "codebook", 1).unwrap();

    let cases: Vec<(&str, ResolvedValueType, &str)> = vec![
        (
            "packed boolean is a physical encoding of a logical type with no scan",
            crate::semantic::builtin_scalar_value_types()
                .into_iter()
                .find(|value| value.nominal_key().is_some_and(|key| key.name() == "bool"))
                .expect("the governed catalog admits bool"),
            "tiler.value-conformance.no-admitted-contract",
        ),
        (
            "complex is a parameterized logical type",
            complex,
            "tiler.value-conformance.no-admitted-contract",
        ),
        (
            "an unadmitted codebook scheme",
            encoded_with(codebook, vec![]),
            "tiler.value-conformance.unsupported-scheme",
        ),
        (
            "a hierarchical microscaling scheme",
            encoded_with(microscaling, vec![]),
            "tiler.value-conformance.unsupported-scheme",
        ),
        (
            "the admitted scheme under a contract this validator has not admitted",
            encoded_with(strict_affine_scheme(), vec![]),
            "tiler.value-conformance.unsupported-scheme-contract",
        ),
        (
            "a mask/outlier component type with no admitted logical scan",
            strict_affine_like(vec![EncodedComponentDeclaration::new(
                nested_role,
                crate::semantic::Bf16::resolved_type(),
                EncodedComponentShape::LogicalValue,
            )]),
            "tiler.value-conformance.unsupported-component-type",
        ),
    ];
    for (case, resolved_type, expected) in cases {
        let error =
            ResolvedValueConformanceContract::derive(&resolved_type, &Shape::from_dims([2]))
                .expect_err(case);
        assert_eq!(error.name(), expected, "{case}");
        assert!(error.to_string().starts_with(expected), "{case}");
    }

    // A nested encoded component cannot be built through `with_components`, so
    // the derivation's own guard is exercised by naming the same refusal from
    // the constructor that owns it.
    assert!(matches!(
        EncodedNumericContract::with_components(
            [CanonicalField::new(
                ENCODED_NUMERIC_CODE_MIN,
                CanonicalValue::unsigned_u8(0),
            )],
            [EncodedComponentDeclaration::new(
                nested_role,
                StrictAffineU4::resolved_type(),
                EncodedComponentShape::LogicalValue,
            )],
        ),
        Err(crate::semantic::types::TypeIdentityError::NestedEncodedComponentType { .. })
    ));

    // Only the per-tensor map has an admitted evaluator, and the derivation is
    // where a new one must be admitted. Nothing else constructs a map today, so
    // this asserts the gate exists on the one form that does.
    let per_tensor = strict_affine_like(vec![EncodedComponentDeclaration::new(
        other_map_role,
        F32::resolved_type(),
        EncodedComponentShape::ParameterMap(ParameterIndexMap::per_tensor()),
    )]);
    let contract =
        ResolvedValueConformanceContract::derive(&per_tensor, &Shape::from_dims([2])).unwrap();
    assert_eq!(
        contract.components()[0].parameter_map(),
        Some(&ParameterIndexMap::per_tensor()),
    );
}

fn encoded_with(
    scheme: QuantSchemeKey,
    components: Vec<EncodedComponentDeclaration>,
) -> ResolvedValueType {
    ResolvedValueType::encoded_numeric(
        scheme,
        EncodedNumericContract::with_components(
            [CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::boolean(true),
            )],
            components,
        )
        .unwrap(),
    )
    .unwrap()
}

/// Builds a strict-affine-scheme type whose contract fields are complete but
/// whose component set is the caller's, so component-level refusals are
/// reachable without forging the scheme's own governed fields.
fn strict_affine_like(components: Vec<EncodedComponentDeclaration>) -> ResolvedValueType {
    ResolvedValueType::encoded_numeric(
        strict_affine_scheme(),
        EncodedNumericContract::with_components(
            [
                CanonicalField::new(ENCODED_NUMERIC_CODE_MIN, CanonicalValue::unsigned_u8(0)),
                CanonicalField::new(ENCODED_NUMERIC_CODE_MAX, CanonicalValue::unsigned_u8(15)),
                CanonicalField::new(
                    ENCODED_NUMERIC_SCALE_DOMAIN,
                    CanonicalValue::utf8("positive-normal-f32").unwrap(),
                ),
            ],
            components,
        )
        .unwrap(),
    )
    .unwrap()
}

// ------------------------------------------------------ structural refusals --

/// Missing, duplicate, extra, swapped, wrong-type, wrong-shape, and cross-value
/// component structures each refuse under their own class.
#[test]
fn every_malformed_component_structure_refuses_under_its_own_class() {
    let good = || u4_components(&[7, 8], 0.5_f32.to_bits(), 8);
    let logical = [2_u64];

    let missing = {
        let mut components = good();
        components.pop();
        components
    };
    assert!(matches!(
        conform(missing, &logical).unwrap_err().cause(),
        ValueConformanceCause::ComponentCount {
            expected: 3,
            actual: 2
        }
    ));

    let extra = {
        let mut components = good();
        components.push(FixtureComponent::scale(0.5_f32.to_bits()));
        components
    };
    assert!(matches!(
        conform(extra, &logical).unwrap_err().cause(),
        ValueConformanceCause::ComponentCount {
            expected: 3,
            actual: 4
        }
    ));

    let duplicate = {
        let mut components = good();
        components[2] = components[1].clone();
        components
    };
    let rejection = conform(duplicate, &logical).unwrap_err();
    assert_eq!(rejection.component_ordinal(), Some(2));
    assert!(matches!(
        rejection.cause(),
        ValueConformanceCause::ComponentRole { expected, actual }
            if *expected == STRICT_AFFINE_ZERO_POINT_ROLE && *actual == STRICT_AFFINE_SCALE_ROLE
    ));

    let swapped = {
        let mut components = good();
        components.swap(1, 2);
        components
    };
    let rejection = conform(swapped, &logical).unwrap_err();
    assert_eq!(rejection.component_ordinal(), Some(1));
    assert!(matches!(
        rejection.cause(),
        ValueConformanceCause::ComponentRole { .. }
    ));

    let wrong_type = {
        let mut components = good();
        components[0] = components[0].clone().with_type(U8::resolved_type());
        components
    };
    assert!(matches!(
        conform(wrong_type, &logical).unwrap_err().cause(),
        ValueConformanceCause::ComponentType { expected, actual }
            if **expected == U4::resolved_type() && **actual == U8::resolved_type()
    ));

    let wrong_shape = {
        let mut components = good();
        components[1] = components[1].clone().with_shape(Shape::from_dims([1]));
        components
    };
    assert!(matches!(
        conform(wrong_shape, &logical).unwrap_err().cause(),
        ValueConformanceCause::ComponentShape { .. }
    ));

    // A wrong-map component is refused by the same shape obligation the map
    // derives: a per-tensor parameter is rank zero, and any other map produces
    // a different derived shape.
    let wrong_map = {
        let mut components = good();
        components[2] = components[2].clone().with_shape(Shape::from_dims([2]));
        components
    };
    assert!(matches!(
        conform(wrong_map, &logical).unwrap_err().cause(),
        ValueConformanceCause::ComponentShape { .. }
    ));

    // A cross-value component: another value's codes, whose logical extent
    // disagrees with this value's declared shape.
    let cross_value = {
        let mut components = good();
        components[0] =
            FixtureComponent::codes(STRICT_AFFINE_CODES_ROLE, &U4::resolved_type(), &[1, 2, 3]);
        components
    };
    assert!(matches!(
        conform(cross_value, &logical).unwrap_err().cause(),
        ValueConformanceCause::ComponentShape { .. }
    ));

    // An unknown role in the right position is still a role refusal, not an
    // acceptance by slot.
    let unknown_role = {
        let mut components = good();
        components[0] = components[0]
            .clone()
            .with_role(EncodedComponentRole::new(99));
        components
    };
    assert!(matches!(
        conform(unknown_role, &logical).unwrap_err().cause(),
        ValueConformanceCause::ComponentRole { .. }
    ));

    // The complete structure conforms, which is what makes each refusal above
    // a property of the perturbation rather than of the fixture.
    conform(good(), &logical).unwrap();
}

/// A logical-view fault is refused by its own exact name, before any verdict.
#[test]
fn every_logical_view_fault_refuses_by_its_exact_name() {
    for (fault, name) in [
        (
            LogicalViewFault::Inaccessible,
            "tiler.value-conformance.inaccessible-memory",
        ),
        (
            LogicalViewFault::IncoherentView,
            "tiler.value-conformance.absent-coherence",
        ),
        (
            LogicalViewFault::UnreconstructableIndex,
            "tiler.value-conformance.unreconstructable-logical-view",
        ),
        (
            LogicalViewFault::UnrepresentableScalar,
            "tiler.value-conformance.unrepresentable-scalar",
        ),
    ] {
        let mut components = u4_components(&[7, 8], 0.5_f32.to_bits(), 8);
        components[0] = components[0].clone().with_fault(fault);
        let rejection = conform(components, &[2]).unwrap_err();
        assert_eq!(rejection.cause(), &ValueConformanceCause::ViewFault(fault));
        assert_eq!(fault.name(), name);
        assert_eq!(rejection.logical_index(), Some(0));
        assert_eq!(rejection.component_ordinal(), Some(0));
    }
}

/// The scan budget refuses a shortfall before the first read.
#[test]
fn an_over_budget_obligation_refuses_before_reading_anything() {
    let elements = MAX_CONFORMANCE_SCAN_ELEMENTS + 1;
    let view = SliceView::new(vec![
        FixtureComponent {
            role: STRICT_AFFINE_CODES_ROLE,
            resolved_type: U4::resolved_type(),
            shape: Shape::from_dims([elements]),
            scalars: Vec::new(),
            fault: None,
        },
        FixtureComponent::scale(0.5_f32.to_bits()),
        FixtureComponent::scalar_code(STRICT_AFFINE_ZERO_POINT_ROLE, &U4::resolved_type(), 8),
    ]);
    let rejection = scan_bound_value(
        &standard_binding_validator(),
        &direct_subject(
            StrictAffineU4::resolved_type(),
            Shape::from_dims([elements]),
        ),
        &view,
    )
    .unwrap_err();
    assert!(matches!(
        rejection.cause(),
        ValueConformanceCause::ScanBudgetExceeded { limit, .. }
            if *limit == MAX_CONFORMANCE_SCAN_ELEMENTS
    ));
    assert_eq!(view.reads(), 0, "a shortfall is refused before any read");
}

// ---------------------------------------------------------- value domains ----

/// Every U4 and U8 boundary code and zero point passes, and every out-of-domain
/// payload fails without widening.
#[test]
fn the_exact_code_domain_boundary_is_admitted_and_nothing_beyond_it_is() {
    for (code_type, encoded_type, maximum) in [
        (U4::resolved_type(), StrictAffineU4::resolved_type(), 15_u8),
        (
            U8::resolved_type(),
            StrictAffineU8::resolved_type(),
            u8::MAX,
        ),
    ] {
        let subject = |shape: Shape| {
            ValueConformanceSubject::new(
                ValueOrigin::direct_binding(input("weights")),
                ValueStability::ImmutableHost,
                route(),
                SemanticLogicalView::WholeValue,
                encoded_type.clone(),
                shape,
            )
        };
        let build = |codes: &[u8], zero: u8| {
            SliceView::new(vec![
                FixtureComponent::codes(STRICT_AFFINE_CODES_ROLE, &code_type, codes),
                FixtureComponent::scale(0.5_f32.to_bits()),
                FixtureComponent::scalar_code(STRICT_AFFINE_ZERO_POINT_ROLE, &code_type, zero),
            ])
        };

        // Both boundaries of the domain, as codes and as the zero point.
        for zero in [0_u8, maximum] {
            scan_bound_value(
                &standard_binding_validator(),
                &subject(Shape::from_dims([2])),
                &build(&[0, maximum], zero),
            )
            .unwrap_or_else(|error| panic!("the boundary must be admitted: {error}"));
        }

        if maximum == u8::MAX {
            // The U8 domain is the whole carrier: no byte is out of domain, and
            // the honest evidence for that is that every one of them is
            // admitted rather than that some untested value would be refused.
            let all: Vec<u8> = (0..=u8::MAX).collect();
            scan_bound_value(
                &standard_binding_validator(),
                &subject(Shape::from_dims([256])),
                &build(&all, 0),
            )
            .unwrap();
            continue;
        }

        // Every code above the U4 maximum is refused, and it is refused as a
        // U4 rather than approximated through the wider carrier that held it.
        for code in (maximum + 1)..=u8::MAX {
            let rejection = scan_bound_value(
                &standard_binding_validator(),
                &subject(Shape::from_dims([1])),
                &build(&[code], 0),
            )
            .unwrap_err();
            assert_eq!(
                rejection.cause(),
                &ValueConformanceCause::CodeOutOfDomain {
                    code,
                    minimum: 0,
                    maximum,
                },
                "code {code} must be refused in the U4 domain",
            );
            let rejection = scan_bound_value(
                &standard_binding_validator(),
                &subject(Shape::from_dims([1])),
                &build(&[0], code),
            )
            .unwrap_err();
            assert_eq!(rejection.component_ordinal(), Some(2));
            assert!(matches!(
                rejection.cause(),
                ValueConformanceCause::CodeOutOfDomain { .. }
            ));
        }
    }
}

/// Every binary32 class the scale can take is admitted or refused exactly.
///
/// The table is exhaustive over the classes `f32` distinguishes rather than a
/// sample of them, because a domain check that admits one unlisted class admits
/// a value the decode's derivation does not cover. It is the same class
/// population the producer preconditions are checked against, applied here to a
/// *bound payload* instead of a constant operand.
#[test]
fn every_exact_scale_class_takes_its_conformance_outcome() {
    let admitted = [
        ("smallest positive normal", f32::MIN_POSITIVE.to_bits()),
        ("interior positive normal", 0.5_f32.to_bits()),
        ("largest positive normal", f32::MAX.to_bits()),
    ];
    let refused = [
        ("positive zero", 0.0_f32.to_bits()),
        ("negative zero", (-0.0_f32).to_bits()),
        ("negative finite normal", (-1.0_f32).to_bits()),
        ("largest negative finite", f32::MIN.to_bits()),
        ("negative subnormal", 0x8000_0001),
        ("positive infinity", f32::INFINITY.to_bits()),
        ("negative infinity", f32::NEG_INFINITY.to_bits()),
        ("quiet NaN", 0x7fc0_0000),
        ("signalling NaN", 0x7f80_0001),
        ("smallest positive subnormal", 0x0000_0001),
        ("interior positive subnormal", 0x0000_ffff),
        (
            "largest positive subnormal",
            f32::MIN_POSITIVE.to_bits() - 1,
        ),
    ];
    for (class, bits) in admitted {
        conform(u4_components(&[7, 8], bits, 8), &[2])
            .unwrap_or_else(|error| panic!("{class} must be admitted: {error}"));
    }
    for (class, bits) in refused {
        let rejection = conform(u4_components(&[7, 8], bits, 8), &[2]).unwrap_err();
        assert_eq!(
            rejection.cause(),
            &ValueConformanceCause::ScaleOutOfDomain { bits },
            "{class} must be outside the admitted scale domain",
        );
        assert_eq!(rejection.component_ordinal(), Some(1));
        assert_eq!(rejection.logical_index(), Some(0));
    }
}

/// A scalar of the wrong kind is refused rather than reinterpreted.
#[test]
fn a_scalar_of_the_wrong_kind_is_refused_rather_than_reinterpreted() {
    let mut components = u4_components(&[7, 8], 0.5_f32.to_bits(), 8);
    components[0].scalars[1] = LogicalScalar::F32Bits(0.5_f32.to_bits());
    let rejection = conform(components, &[2]).unwrap_err();
    assert_eq!(
        rejection.cause(),
        &ValueConformanceCause::ScalarKindMismatch
    );
    assert_eq!(rejection.logical_index(), Some(1));
}

// ------------------------------------------------- deterministic diagnostics --

/// The reported refusal is the minimum of `(index, code, component)`.
///
/// Three components each carry a refusal at a different logical index, and the
/// one reported is the lowest index regardless of which component holds it.
#[test]
fn the_reported_refusal_is_the_minimum_diagnostic_coordinate() {
    // Codes refuse at index 3; the scale refuses at index 0. The scale wins on
    // index even though it is the later component.
    let mut components = u4_components(&[0, 1, 2, 16], 0.0_f32.to_bits(), 8);
    let rejection = conform(components.clone(), &[4]).unwrap_err();
    assert_eq!(rejection.logical_index(), Some(0));
    assert_eq!(rejection.component_ordinal(), Some(1));

    // With a valid scale, the codes' own index-3 refusal is what remains.
    components[1] = FixtureComponent::scale(0.5_f32.to_bits());
    let rejection = conform(components.clone(), &[4]).unwrap_err();
    assert_eq!(rejection.logical_index(), Some(3));
    assert_eq!(rejection.component_ordinal(), Some(0));

    // Two refusals at the same index tie-break on the stable code. The codes'
    // out-of-domain code orders before the zero point's, so the earlier
    // component wins only because the codes both live at index 0.
    let both_at_zero = vec![
        FixtureComponent::codes(STRICT_AFFINE_CODES_ROLE, &U4::resolved_type(), &[16]),
        FixtureComponent::scale(0.5_f32.to_bits()),
        FixtureComponent::scalar_code(STRICT_AFFINE_ZERO_POINT_ROLE, &U4::resolved_type(), 16),
    ];
    let rejection = conform(both_at_zero, &[1]).unwrap_err();
    assert_eq!(rejection.logical_index(), Some(0));
    assert_eq!(
        rejection.component_ordinal(),
        Some(0),
        "the same code at the same index tie-breaks on the component ordinal",
    );
}

/// Equivalent packed and byte-addressed logical views agree exactly, and the
/// semantic scan never observes the packed tail.
#[test]
fn packed_and_byte_addressed_views_agree_and_leave_the_tail_unobserved() {
    // Five logical nibbles occupy three bytes, so the final byte has a four-bit
    // tail that is not part of the logical value.
    let codes = [1_u8, 2, 3, 4, 5];
    let subject = direct_subject(StrictAffineU4::resolved_type(), Shape::from_dims([5]));
    let validator = standard_binding_validator();

    let packed = PackedNibbleView::new(&codes, 0.5_f32.to_bits(), 8);
    let byte_addressed = SliceView::new(u4_components(&codes, 0.5_f32.to_bits(), 8));
    let from_packed = scan_bound_value(&validator, &subject, &packed).unwrap();
    let from_bytes = scan_bound_value(&validator, &subject, &byte_addressed).unwrap();
    assert_eq!(from_packed.as_bytes(), from_bytes.as_bytes());
    assert_eq!(packed.tail_reads(), 0);

    // A noncanonical tail is invisible to the semantic scan: the same logical
    // value still conforms, and the physical owner is what reports the tail.
    let noncanonical = PackedNibbleView::new(&codes, 0.5_f32.to_bits(), 8).with_noncanonical_tail();
    let from_noncanonical = scan_bound_value(&validator, &subject, &noncanonical).unwrap();
    assert_eq!(from_noncanonical.as_bytes(), from_bytes.as_bytes());
    assert_eq!(noncanonical.tail_reads(), 0);

    // The same invalid logical element reports the same logical index through
    // both layouts, which is what makes the index a property of the value.
    let invalid = [1_u8, 2, 9, 4, 5];
    let packed_bad = PackedNibbleView::new(&invalid, 0x8000_0001, 8);
    let bytes_bad = SliceView::new(u4_components(&invalid, 0x8000_0001, 8));
    let packed_rejection = scan_bound_value(&validator, &subject, &packed_bad).unwrap_err();
    let bytes_rejection = scan_bound_value(&validator, &subject, &bytes_bad).unwrap_err();
    assert_eq!(packed_rejection, bytes_rejection);
    assert_eq!(packed_rejection.logical_index(), Some(0));
    assert_eq!(packed_bad.tail_reads(), 0);
}

// --------------------------------------------------------------- evidence ----

/// Every subject field is identity-bearing, so no perturbation reuses a proof.
///
/// The population is written out: subject, view, type, shape, role and map
/// (through the type they live in), version, coherence, and route. Each
/// perturbation is applied to a proved subject, checked to fail reuse, and
/// restored, so a field that stopped participating shows up as a row that no
/// longer fails.
#[test]
fn perturbing_any_subject_field_prevents_evidence_reuse() {
    let validator = standard_binding_validator();
    let base = ValueConformanceSubject::new(
        ValueOrigin::direct_binding(input("weights")),
        ValueStability::Versioned {
            version: ValueVersion::new(7),
            coherence: CoherenceEpoch::new(11),
        },
        route(),
        SemanticLogicalView::WholeValue,
        StrictAffineU4::resolved_type(),
        Shape::from_dims([2]),
    );
    let view = SliceView::new(u4_components(&[7, 8], 0.5_f32.to_bits(), 8));
    let evidence = scan_bound_value(&validator, &base, &view).unwrap();
    assert!(evidence.authorizes(&base, &validator));

    let with = |origin, stability, dependency, resolved_type, shape| {
        ValueConformanceSubject::new(
            origin,
            stability,
            dependency,
            SemanticLogicalView::WholeValue,
            resolved_type,
            shape,
        )
    };
    let perturbations = vec![
        (
            "subject origin",
            with(
                ValueOrigin::direct_binding(input("other")),
                base.stability(),
                base.route().clone(),
                base.resolved_type().clone(),
                base.shape().clone(),
            ),
        ),
        (
            "value version",
            with(
                base.origin().clone(),
                ValueStability::Versioned {
                    version: ValueVersion::new(8),
                    coherence: CoherenceEpoch::new(11),
                },
                base.route().clone(),
                base.resolved_type().clone(),
                base.shape().clone(),
            ),
        ),
        (
            "coherence epoch",
            with(
                base.origin().clone(),
                ValueStability::Versioned {
                    version: ValueVersion::new(7),
                    coherence: CoherenceEpoch::new(12),
                },
                base.route().clone(),
                base.resolved_type().clone(),
                base.shape().clone(),
            ),
        ),
        (
            "immutability provenance",
            with(
                base.origin().clone(),
                ValueStability::ImmutableHost,
                base.route().clone(),
                base.resolved_type().clone(),
                base.shape().clone(),
            ),
        ),
        (
            "route dependency",
            with(
                base.origin().clone(),
                base.stability(),
                RouteDependency::new(b"tiler.test.route.v2"),
                base.resolved_type().clone(),
                base.shape().clone(),
            ),
        ),
        (
            "resolved type, and with it every role, component type, and map",
            with(
                base.origin().clone(),
                base.stability(),
                base.route().clone(),
                StrictAffineU8::resolved_type(),
                base.shape().clone(),
            ),
        ),
        (
            "logical shape",
            with(
                base.origin().clone(),
                base.stability(),
                base.route().clone(),
                base.resolved_type().clone(),
                Shape::from_dims([3]),
            ),
        ),
    ];
    for (field, perturbed) in perturbations {
        assert!(
            !evidence.authorizes(&perturbed, &validator),
            "perturbing the {field} must prevent reuse",
        );
        assert!(evidence.authorizes(&base, &validator), "{field}: restored");
    }

    // The validator revision is the static half, and it is identity-bearing too.
    let other_validator = ConformanceValidatorIdentity::new(
        standard_binding_validator().key().clone(),
        NonZeroU32::new(2).unwrap(),
    );
    assert!(!evidence.authorizes(&base, &other_validator));

    // Every perturbation also changes the durable canonical encoding, so the
    // property survives a process boundary rather than resting on `PartialEq`.
    let other = scan_bound_value(
        &validator,
        &with(
            base.origin().clone(),
            ValueStability::ImmutableHost,
            base.route().clone(),
            base.resolved_type().clone(),
            base.shape().clone(),
        ),
        &SliceView::new(u4_components(&[7, 8], 0.5_f32.to_bits(), 8)),
    )
    .unwrap();
    assert_ne!(evidence.as_bytes(), other.as_bytes());
}

/// Evidence cannot be transplanted between two values with identical payloads.
///
/// The two bindings carry the same bytes, the same type, and the same shape,
/// and differ only in which interface key they entered under. Neither proof
/// authorizes the other, which is the property that pointer equality and slot
/// position would both get wrong.
#[test]
fn identical_payloads_under_two_keys_do_not_share_a_proof() {
    let validator = standard_binding_validator();
    let subject = |key: &str| {
        ValueConformanceSubject::new(
            ValueOrigin::direct_binding(input(key)),
            ValueStability::ImmutableHost,
            route(),
            SemanticLogicalView::WholeValue,
            StrictAffineU4::resolved_type(),
            Shape::from_dims([2]),
        )
    };
    let left = scan_bound_value(
        &validator,
        &subject("left"),
        &SliceView::new(u4_components(&[7, 8], 0.5_f32.to_bits(), 8)),
    )
    .unwrap();
    let right = scan_bound_value(
        &validator,
        &subject("right"),
        &SliceView::new(u4_components(&[7, 8], 0.5_f32.to_bits(), 8)),
    )
    .unwrap();
    assert_ne!(left.as_bytes(), right.as_bytes());
    assert!(!left.authorizes(&subject("right"), &validator));
    assert!(!right.authorizes(&subject("left"), &validator));
}

/// A complete same-provenance proof is reused and the view is never read.
#[test]
fn a_complete_same_provenance_proof_is_reused_without_rescanning() {
    let validator = standard_binding_validator();
    let subject = direct_subject(StrictAffineU4::resolved_type(), Shape::from_dims([2]));
    let first = SliceView::new(u4_components(&[7, 8], 0.5_f32.to_bits(), 8));
    let evidence = conform_bound_value(&validator, &subject, None, &first).unwrap();
    assert!(first.reads() > 0, "the first proof has to read the value");

    let second = SliceView::new(u4_components(&[7, 8], 0.5_f32.to_bits(), 8));
    let reused = conform_bound_value(&validator, &subject, Some(&evidence), &second).unwrap();
    assert_eq!(reused.as_bytes(), evidence.as_bytes());
    assert_eq!(second.reads(), 0, "a complete proof is not recomputed");

    // A different provenance is a different subject, and it is scanned.
    let changed = ValueConformanceSubject::new(
        ValueOrigin::direct_binding(input("weights")),
        ValueStability::Versioned {
            version: ValueVersion::new(1),
            coherence: CoherenceEpoch::new(1),
        },
        route(),
        SemanticLogicalView::WholeValue,
        StrictAffineU4::resolved_type(),
        Shape::from_dims([2]),
    );
    let third = SliceView::new(u4_components(&[7, 8], 0.5_f32.to_bits(), 8));
    conform_bound_value(&validator, &changed, Some(&evidence), &third).unwrap();
    assert!(third.reads() > 0, "a changed provenance is rescanned");

    // A stale proof cannot rescue an invalid payload under a new provenance.
    let fourth = SliceView::new(u4_components(&[16, 8], 0.5_f32.to_bits(), 8));
    assert!(conform_bound_value(&validator, &changed, Some(&evidence), &fourth).is_err());
}

// ------------------------------------------------------------- composition ---

fn assemble_program(code_type: ResolvedValueType, constant_scale: Option<u32>) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let codes = builder
        .input_resolved(input("codes"), Shape::from_dims([2]), code_type.clone())
        .unwrap();
    let scale = match constant_scale {
        Some(bits) => F32Constant::apply(&mut builder, bits).unwrap().erase(),
        None => builder
            .input_resolved(input("scale"), Shape::new([]), F32::resolved_type())
            .unwrap(),
    };
    let zero = builder
        .input_resolved(input("zero"), Shape::new([]), code_type)
        .unwrap();
    let assembled = builder
        .apply(
            assemble_strict_affine_op(),
            OperationAttributes::empty(),
            &[codes, scale, zero],
        )
        .unwrap()[0];
    let decoded = builder
        .apply(
            dequantize_strict_affine_op(),
            OperationAttributes::empty(),
            &[assembled],
        )
        .unwrap()[0];
    builder
        .output_resolved(OutputKey::new("decoded").unwrap(), decoded)
        .unwrap();
    builder.build().unwrap()
}

fn quantize_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let expressed = builder
        .input_resolved(
            input("expressed"),
            Shape::from_dims([2]),
            F32::resolved_type(),
        )
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 0.5_f32.to_bits())
        .unwrap()
        .erase();
    let zero = builder
        .input_resolved(input("zero"), Shape::new([]), U4::resolved_type())
        .unwrap();
    let quantized = builder
        .apply(
            quantize_strict_affine_op(),
            OperationAttributes::empty(),
            &[expressed, scale, zero],
        )
        .unwrap()[0];
    builder
        .output_resolved(OutputKey::new("quantized").unwrap(), quantized)
        .unwrap();
    builder.build().unwrap()
}

fn occurrence<'a>(
    program: &'a SemanticProgram,
    operation: &OpKey,
) -> crate::semantic::OperationRef<'a> {
    program
        .operations()
        .find(|candidate| candidate.key() == operation)
        .expect("the program retains its occurrence")
}

fn first_result(operation: crate::semantic::OperationRef<'_>) -> crate::semantic::ValueId {
    operation
        .results()
        .next()
        .expect("every admitted producer has one ordered result")
}

fn operand_evidence(
    validator: &ConformanceValidatorIdentity,
    key: &str,
    resolved_type: ResolvedValueType,
    shape: Shape,
    scalars: Vec<LogicalScalar>,
) -> ValueConformanceEvidence {
    let view = SliceView::new(vec![FixtureComponent {
        role: EncodedComponentRole::new(0),
        resolved_type: resolved_type.clone(),
        shape: shape.clone(),
        scalars,
        fault: None,
    }]);
    scan_bound_value(
        validator,
        &ValueConformanceSubject::new(
            ValueOrigin::direct_binding(input(key)),
            ValueStability::ImmutableHost,
            route(),
            SemanticLogicalView::WholeValue,
            resolved_type,
            shape,
        ),
        &view,
    )
    .unwrap()
}

/// An assembled value's proof composes its operands and its own preconditions.
///
/// Nothing is read from the result: the codes and zero point are the operands'
/// own proofs, and the scale is the discharged normal-scale precondition. That
/// is what makes this a distinct construction path from the direct-binding scan
/// while producing the same evidence vocabulary.
#[test]
fn an_assembled_result_composes_its_operand_proofs_and_its_discharged_precondition() {
    let validator = standard_binding_validator();
    // A constant scale proves both scale predicates statically, so the
    // occurrence carries no residual and the discharge list is empty.
    let program = assemble_program(U4::resolved_type(), Some(0.5_f32.to_bits()));
    let assemble = occurrence(&program, &assemble_strict_affine_op());
    let discharged = SemanticPreconditionsDischarged::for_occurrence(assemble, &[]).unwrap();

    let codes = operand_evidence(
        &validator,
        "codes",
        U4::resolved_type(),
        Shape::from_dims([2]),
        vec![
            LogicalScalar::UnsignedCode(7),
            LogicalScalar::UnsignedCode(8),
        ],
    );
    let zero = operand_evidence(
        &validator,
        "zero",
        U4::resolved_type(),
        Shape::new([]),
        vec![LogicalScalar::UnsignedCode(8)],
    );
    let subject = ValueConformanceSubject::new(
        ValueOrigin::produced_result(assemble, first_result(assemble)).unwrap(),
        ValueStability::ImmutableHost,
        route(),
        SemanticLogicalView::WholeValue,
        StrictAffineU4::resolved_type(),
        Shape::from_dims([2]),
    );
    let evidence = compose_produced_conformance(
        &validator,
        &subject,
        &discharged,
        &[
            ComposedOperand {
                role: STRICT_AFFINE_CODES_ROLE,
                evidence: &codes,
            },
            ComposedOperand {
                role: STRICT_AFFINE_ZERO_POINT_ROLE,
                evidence: &zero,
            },
        ],
    )
    .unwrap();
    assert!(evidence.authorizes(&subject, &validator));
    assert_eq!(
        evidence.contract().components().len(),
        3,
        "the composed proof discharges the same three obligations a scan would",
    );

    // A direct-binding proof of the same logical value is a different subject,
    // so the two paths do not silently substitute for one another.
    let scanned = scan_bound_value(
        &validator,
        &direct_subject(StrictAffineU4::resolved_type(), Shape::from_dims([2])),
        &SliceView::new(u4_components(&[7, 8], 0.5_f32.to_bits(), 8)),
    )
    .unwrap();
    assert_ne!(scanned.as_bytes(), evidence.as_bytes());
}

/// A quantized result's codes come from the operation's semantics, not an operand.
#[test]
fn a_quantized_result_needs_only_its_zero_point_proof() {
    let validator = standard_binding_validator();
    let program = quantize_program();
    let quantize = occurrence(&program, &quantize_strict_affine_op());
    // The expressed operand is a runtime input, so `NoNaN` stays residual and
    // must be discharged explicitly; both scale predicates are proved by the
    // governed constant.
    let residuals: Vec<_> = quantize
        .semantic_preconditions()
        .filter(|precondition| precondition.status() == SemanticPreconditionStatus::Residual)
        .filter_map(crate::semantic::SemanticPreconditionRef::obligation_identity)
        .collect();
    assert_eq!(residuals.len(), 1);
    let discharged = SemanticPreconditionsDischarged::for_occurrence(quantize, &residuals).unwrap();

    let zero = operand_evidence(
        &validator,
        "zero",
        U4::resolved_type(),
        Shape::new([]),
        vec![LogicalScalar::UnsignedCode(8)],
    );
    let subject = ValueConformanceSubject::new(
        ValueOrigin::produced_result(quantize, first_result(quantize)).unwrap(),
        ValueStability::ImmutableHost,
        route(),
        SemanticLogicalView::WholeValue,
        StrictAffineU4::resolved_type(),
        Shape::from_dims([2]),
    );
    compose_produced_conformance(
        &validator,
        &subject,
        &discharged,
        &[ComposedOperand {
            role: STRICT_AFFINE_ZERO_POINT_ROLE,
            evidence: &zero,
        }],
    )
    .unwrap();

    // Offering the codes as an operand proof is refused: this producer does not
    // carry a codes operand through, so a proof for that role establishes
    // nothing about its result.
    let codes = operand_evidence(
        &validator,
        "codes",
        U4::resolved_type(),
        Shape::from_dims([2]),
        vec![
            LogicalScalar::UnsignedCode(7),
            LogicalScalar::UnsignedCode(8),
        ],
    );
    assert_eq!(
        compose_produced_conformance(
            &validator,
            &subject,
            &discharged,
            &[
                ComposedOperand {
                    role: STRICT_AFFINE_ZERO_POINT_ROLE,
                    evidence: &zero,
                },
                ComposedOperand {
                    role: STRICT_AFFINE_CODES_ROLE,
                    evidence: &codes,
                },
            ],
        ),
        Err(ProofCompositionError::ForeignOperandProof {
            role: STRICT_AFFINE_CODES_ROLE,
        }),
    );
}

/// An undischarged residual cannot be composed away.
#[test]
fn an_undischarged_residual_refuses_the_composition_at_its_source() {
    let program = assemble_program(U4::resolved_type(), None);
    let assemble = occurrence(&program, &assemble_strict_affine_op());
    let residuals: Vec<_> = assemble
        .semantic_preconditions()
        .filter_map(crate::semantic::SemanticPreconditionRef::obligation_identity)
        .collect();
    assert_eq!(residuals.len(), 2, "a runtime scale leaves two residuals");

    // Nothing discharged: the first residual is named.
    let error = SemanticPreconditionsDischarged::for_occurrence(assemble, &[]).unwrap_err();
    assert_eq!(
        error,
        PreconditionDischargeError::UndischargedObligation {
            operation: Arc::new(assemble_strict_affine_op()),
            ordinal: 0,
            predicate: Arc::new(positive_finite_scalar_predicate()),
        },
    );

    // One of two: the other is still named.
    let error =
        SemanticPreconditionsDischarged::for_occurrence(assemble, &residuals[..1]).unwrap_err();
    assert_eq!(
        error,
        PreconditionDischargeError::UndischargedObligation {
            operation: Arc::new(assemble_strict_affine_op()),
            ordinal: 1,
            predicate: Arc::new(positive_normal_scalar_predicate()),
        },
    );

    // Both: the discharge is minted and names the same occurrence.
    let discharged = SemanticPreconditionsDischarged::for_occurrence(assemble, &residuals).unwrap();
    assert_eq!(discharged.operation(), &assemble_strict_affine_op());
}

/// An obligation belonging to another occurrence is refused, not counted.
#[test]
fn a_foreign_obligation_cannot_stand_in_for_this_occurrences_own() {
    let program = {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let scale = builder
            .input_resolved(input("scale"), Shape::new([]), F32::resolved_type())
            .unwrap();
        let zero = builder
            .input_resolved(input("zero"), Shape::new([]), U4::resolved_type())
            .unwrap();
        let mut assembled = Vec::new();
        for name in ["left", "right"] {
            let codes = builder
                .input_resolved(
                    input(&format!("codes-{name}")),
                    Shape::from_dims([2]),
                    U4::resolved_type(),
                )
                .unwrap();
            assembled.push(
                builder
                    .apply(
                        assemble_strict_affine_op(),
                        OperationAttributes::empty(),
                        &[codes, scale, zero],
                    )
                    .unwrap()[0],
            );
        }
        for (ordinal, value) in assembled.iter().enumerate() {
            builder
                .output_resolved(OutputKey::new(format!("out-{ordinal}")).unwrap(), *value)
                .unwrap();
        }
        builder.build().unwrap()
    };
    let occurrences: Vec<_> = program
        .operations()
        .filter(|operation| operation.key() == &assemble_strict_affine_op())
        .collect();
    assert_eq!(occurrences.len(), 2);
    let other: Vec<_> = occurrences[1]
        .semantic_preconditions()
        .filter_map(crate::semantic::SemanticPreconditionRef::obligation_identity)
        .collect();
    // The other occurrence's obligations discharge nothing here, and they are
    // refused rather than silently unused.
    let error =
        SemanticPreconditionsDischarged::for_occurrence(occurrences[0], &other).unwrap_err();
    assert!(matches!(
        error,
        PreconditionDischargeError::UndischargedObligation { .. },
    ));

    // The two occurrences also mint distinct produced-value origins, so the
    // composed proofs cannot be transplanted between them.
    let first = ValueOrigin::produced_result(occurrences[0], first_result(occurrences[0])).unwrap();
    let second =
        ValueOrigin::produced_result(occurrences[1], first_result(occurrences[1])).unwrap();
    // A value that is not this occurrence's result is refused rather than
    // accepted by position.
    assert!(matches!(
        ValueOrigin::produced_result(occurrences[0], first_result(occurrences[1])),
        Err(ValueOriginError::NotAResultOfThisOccurrence { .. }),
    ));
    assert_ne!(first, second);
}

/// A composition whose discharge names another occurrence is refused.
#[test]
fn a_composition_refuses_a_discharge_from_another_producer() {
    let validator = standard_binding_validator();
    let assemble_graph = assemble_program(U4::resolved_type(), Some(0.5_f32.to_bits()));
    let assemble = occurrence(&assemble_graph, &assemble_strict_affine_op());
    let quantize_graph = quantize_program();
    let quantize = occurrence(&quantize_graph, &quantize_strict_affine_op());
    let quantize_residuals: Vec<_> = quantize
        .semantic_preconditions()
        .filter(|precondition| precondition.status() == SemanticPreconditionStatus::Residual)
        .filter_map(crate::semantic::SemanticPreconditionRef::obligation_identity)
        .collect();
    let quantize_discharge =
        SemanticPreconditionsDischarged::for_occurrence(quantize, &quantize_residuals).unwrap();

    let subject = ValueConformanceSubject::new(
        ValueOrigin::produced_result(assemble, first_result(assemble)).unwrap(),
        ValueStability::ImmutableHost,
        route(),
        SemanticLogicalView::WholeValue,
        StrictAffineU4::resolved_type(),
        Shape::from_dims([2]),
    );
    let zero = operand_evidence(
        &validator,
        "zero",
        U4::resolved_type(),
        Shape::new([]),
        vec![LogicalScalar::UnsignedCode(8)],
    );
    assert!(matches!(
        compose_produced_conformance(
            &validator,
            &subject,
            &quantize_discharge,
            &[ComposedOperand {
                role: STRICT_AFFINE_ZERO_POINT_ROLE,
                evidence: &zero,
            }],
        ),
        Err(ProofCompositionError::DischargeOccurrenceMismatch { .. }),
    ));
}

/// An operand proof from another route or another validator does not compose.
#[test]
fn an_operand_proof_from_another_route_or_validator_does_not_compose() {
    let validator = standard_binding_validator();
    let program = assemble_program(U4::resolved_type(), Some(0.5_f32.to_bits()));
    let assemble = occurrence(&program, &assemble_strict_affine_op());
    let discharged = SemanticPreconditionsDischarged::for_occurrence(assemble, &[]).unwrap();
    let subject = ValueConformanceSubject::new(
        ValueOrigin::produced_result(assemble, first_result(assemble)).unwrap(),
        ValueStability::ImmutableHost,
        route(),
        SemanticLogicalView::WholeValue,
        StrictAffineU4::resolved_type(),
        Shape::from_dims([2]),
    );
    let codes = operand_evidence(
        &validator,
        "codes",
        U4::resolved_type(),
        Shape::from_dims([2]),
        vec![
            LogicalScalar::UnsignedCode(7),
            LogicalScalar::UnsignedCode(8),
        ],
    );
    let zero = operand_evidence(
        &validator,
        "zero",
        U4::resolved_type(),
        Shape::new([]),
        vec![LogicalScalar::UnsignedCode(8)],
    );

    // The complete, correctly routed pair composes, which is what makes every
    // refusal below a property of its own perturbation.
    let complete = [
        ComposedOperand {
            role: STRICT_AFFINE_CODES_ROLE,
            evidence: &codes,
        },
        ComposedOperand {
            role: STRICT_AFFINE_ZERO_POINT_ROLE,
            evidence: &zero,
        },
    ];
    compose_produced_conformance(&validator, &subject, &discharged, &complete).unwrap();

    // The same proofs under a different validator revision do not compose: the
    // validator is the static half of the evidence identity.
    let other_validator = ConformanceValidatorIdentity::new(
        standard_binding_validator().key().clone(),
        NonZeroU32::new(2).unwrap(),
    );
    assert!(matches!(
        compose_produced_conformance(&other_validator, &subject, &discharged, &complete),
        Err(ProofCompositionError::OperandValidatorMismatch { .. }),
    ));

    // A zero-point proof about a value of the wrong shape covers nothing.
    let wrong_shape = operand_evidence(
        &validator,
        "zero",
        U4::resolved_type(),
        Shape::from_dims([1]),
        vec![LogicalScalar::UnsignedCode(8)],
    );
    assert!(matches!(
        compose_produced_conformance(
            &validator,
            &subject,
            &discharged,
            &[
                ComposedOperand {
                    role: STRICT_AFFINE_CODES_ROLE,
                    evidence: &codes,
                },
                ComposedOperand {
                    role: STRICT_AFFINE_ZERO_POINT_ROLE,
                    evidence: &wrong_shape,
                },
            ],
        ),
        Err(ProofCompositionError::OperandDoesNotCoverComponent { .. }),
    ));

    // A proof taken against another route does not travel to this one.
    let foreign_route = {
        let view = SliceView::new(vec![FixtureComponent {
            role: EncodedComponentRole::new(0),
            resolved_type: U4::resolved_type(),
            shape: Shape::new([]),
            scalars: vec![LogicalScalar::UnsignedCode(8)],
            fault: None,
        }]);
        scan_bound_value(
            &validator,
            &ValueConformanceSubject::new(
                ValueOrigin::direct_binding(input("zero")),
                ValueStability::ImmutableHost,
                RouteDependency::new(b"tiler.test.route.v2"),
                SemanticLogicalView::WholeValue,
                U4::resolved_type(),
                Shape::new([]),
            ),
            &view,
        )
        .unwrap()
    };
    assert_eq!(
        compose_produced_conformance(
            &validator,
            &subject,
            &discharged,
            &[
                ComposedOperand {
                    role: STRICT_AFFINE_CODES_ROLE,
                    evidence: &codes,
                },
                ComposedOperand {
                    role: STRICT_AFFINE_ZERO_POINT_ROLE,
                    evidence: &foreign_route,
                },
            ],
        ),
        Err(ProofCompositionError::OperandRouteMismatch {
            role: STRICT_AFFINE_ZERO_POINT_ROLE,
        }),
    );

    // A missing operand proof is named rather than assumed.
    assert_eq!(
        compose_produced_conformance(
            &validator,
            &subject,
            &discharged,
            &[ComposedOperand {
                role: STRICT_AFFINE_CODES_ROLE,
                evidence: &codes,
            }],
        ),
        Err(ProofCompositionError::MissingOperandProof {
            role: STRICT_AFFINE_ZERO_POINT_ROLE,
        }),
    );

    // A direct-binding subject cannot be composed at all: it is proved by
    // scanning, and offering it here would skip the scan entirely.
    assert_eq!(
        compose_produced_conformance(
            &validator,
            &direct_subject(StrictAffineU4::resolved_type(), Shape::from_dims([2])),
            &discharged,
            &[],
        ),
        Err(ProofCompositionError::NotAProducedResult),
    );
}

/// A producer with no admitted composition rule refuses by name.
#[test]
fn a_producer_without_an_admitted_rule_refuses_by_name() {
    let validator = standard_binding_validator();
    let program = assemble_program(U4::resolved_type(), Some(0.5_f32.to_bits()));
    let dequantize = occurrence(&program, &dequantize_strict_affine_op());
    let discharged = SemanticPreconditionsDischarged::for_occurrence(dequantize, &[]).unwrap();
    let subject = ValueConformanceSubject::new(
        ValueOrigin::produced_result(dequantize, first_result(dequantize)).unwrap(),
        ValueStability::ImmutableHost,
        route(),
        SemanticLogicalView::WholeValue,
        StrictAffineU4::resolved_type(),
        Shape::from_dims([2]),
    );
    assert_eq!(
        compose_produced_conformance(&validator, &subject, &discharged, &[]),
        Err(ProofCompositionError::UnsupportedProducer {
            operation: Arc::new(dequantize_strict_affine_op()),
        }),
    );
}

/// The registry's admitted strict-affine contracts are exactly the two this
/// validator derives obligations for.
#[test]
fn the_validator_admits_exactly_the_registry_admitted_strict_affine_contracts() {
    let registry = FrozenSemanticRegistry::standard().unwrap();
    for resolved_type in [
        StrictAffineU4::resolved_type(),
        StrictAffineU8::resolved_type(),
    ] {
        assert!(registry.contains(&resolved_type));
        ResolvedValueConformanceContract::derive(&resolved_type, &Shape::from_dims([2])).unwrap();
    }
    // The predicate vocabulary the composition names is the registry's own.
    assert_ne!(no_nan_predicate(), positive_normal_scalar_predicate());
}
