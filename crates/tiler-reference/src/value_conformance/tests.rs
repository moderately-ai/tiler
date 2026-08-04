//! Evidence that the reference path binds, composes, and refuses correctly.

use super::*;
use crate::tensor::{ReferenceComponent, ReferenceElement};
use crate::{EvaluationError, InputBinding, ReferenceEvaluator};
use tiler_ir::semantic::{
    OperationAttributes, OutputKey, SemanticProgramBuilder, StrictAffineU4, ValueConformanceCause,
    dequantize_strict_affine_op,
};

/// A program whose only input is a **direct** strict-affine encoded value.
///
/// This is the case the whole ticket is about: no `Assemble` and no `Quantize`
/// occurrence produced this value, so no operation precondition can speak about
/// its bytes and the binding validator is the only thing between a malformed
/// payload and the decode that consumes it.
struct DirectEncodedInput {
    program: tiler_ir::semantic::SemanticProgram,
    key: InputKey,
}

impl DirectEncodedInput {
    fn new() -> Self {
        let key = InputKey::new("encoded").unwrap();
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let encoded = builder
            .input_resolved(
                key.clone(),
                Shape::from_dims([2]),
                StrictAffineU4::resolved_type(),
            )
            .unwrap();
        let decoded = builder
            .apply(
                dequantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[encoded],
            )
            .unwrap()[0];
        builder
            .output_resolved(OutputKey::new("decoded").unwrap(), decoded)
            .unwrap();
        Self {
            program: builder.build().unwrap(),
            key,
        }
    }

    fn evaluate(&self, payload: &Tensor) -> Result<Vec<Tensor>, EvaluationError> {
        ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(&self.program, &[InputBinding::new(&self.key, payload)])
    }
}

fn element(bytes: impl AsRef<[u8]>) -> ReferenceElement {
    ReferenceElement::new(bytes).unwrap()
}

fn codes(values: &[u8]) -> Tensor {
    Tensor::dense(
        U4::resolved_type(),
        Shape::from_dims([u64::try_from(values.len()).unwrap()]),
        values.iter().map(|value| element([*value])).collect(),
    )
    .unwrap()
}

fn scale(bits: u32) -> Tensor {
    Tensor::scalar(F32::resolved_type(), element(bits.to_be_bytes())).unwrap()
}

fn zero(value: u8) -> Tensor {
    Tensor::scalar(U4::resolved_type(), element([value])).unwrap()
}

fn compound(codes: &Tensor, scale: &Tensor, zero: &Tensor) -> Tensor {
    Tensor::compound(
        StrictAffineU4::resolved_type(),
        codes.shape().clone(),
        vec![
            ReferenceComponent::new(STRICT_AFFINE_CODES_ROLE, codes.clone()),
            ReferenceComponent::new(tiler_ir::semantic::STRICT_AFFINE_SCALE_ROLE, scale.clone()),
            ReferenceComponent::new(STRICT_AFFINE_ZERO_POINT_ROLE, zero.clone()),
        ],
    )
    .unwrap()
}

/// A compound tensor presents exactly the components it holds.
#[test]
fn a_compound_tensor_presents_its_own_ordered_components() {
    let value = compound(&codes(&[7, 8]), &scale(0.5_f32.to_bits()), &zero(8));
    let view = TensorLogicalView::new(&value);
    assert_eq!(view.presented_components(), 3);
    let roles: Vec<_> = (0..3)
        .map(|position| view.presented_component(position).unwrap().role)
        .collect();
    assert_eq!(
        roles,
        vec![
            STRICT_AFFINE_CODES_ROLE,
            tiler_ir::semantic::STRICT_AFFINE_SCALE_ROLE,
            STRICT_AFFINE_ZERO_POINT_ROLE,
        ]
    );
    assert_eq!(
        view.read_logical_scalar(0, 1).unwrap(),
        LogicalScalar::UnsignedCode(8),
    );
    assert_eq!(
        view.read_logical_scalar(1, 0).unwrap(),
        LogicalScalar::F32Bits(0.5_f32.to_bits()),
    );
    assert_eq!(
        view.read_logical_scalar(0, 2),
        Err(LogicalViewFault::UnreconstructableIndex),
        "the logical value has two codes and there is no third",
    );
}

/// A dense tensor presents one component under the reserved dense role.
#[test]
fn a_dense_tensor_presents_one_component_under_the_reserved_role() {
    let value = codes(&[3, 4]);
    let view = TensorLogicalView::new(&value);
    assert_eq!(view.presented_components(), 1);
    let presented = view.presented_component(0).unwrap();
    assert_eq!(presented.role, DENSE_VALUE_COMPONENT_ROLE);
    assert_eq!(presented.resolved_type, &U4::resolved_type());
    assert_eq!(view.presented_component(1), None);
}

/// An element whose width disagrees with its declared type is not reinterpreted.
#[test]
fn an_element_of_the_wrong_width_is_unrepresentable_rather_than_reinterpreted() {
    let wide = Tensor::dense(
        U4::resolved_type(),
        Shape::from_dims([1]),
        vec![element([0, 0, 0, 1])],
    )
    .unwrap();
    assert_eq!(
        TensorLogicalView::new(&wide).read_logical_scalar(0, 0),
        Err(LogicalViewFault::UnrepresentableScalar),
    );
    let narrow = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([1]),
        vec![element([1])],
    )
    .unwrap();
    assert_eq!(
        TensorLogicalView::new(&narrow).read_logical_scalar(0, 0),
        Err(LogicalViewFault::UnrepresentableScalar),
    );
}

/// A bound encoded input is refused by the conformance vocabulary, with its
/// interface key and its deterministic diagnostic coordinate.
#[test]
fn a_bound_encoded_input_is_refused_by_name_with_its_logical_index() {
    let fixture = DirectEncodedInput::new();
    for (payload, expect) in [
        (
            compound(&codes(&[7, 16]), &scale(0.5_f32.to_bits()), &zero(8)),
            (1_u64, 0_u32),
        ),
        (
            compound(&codes(&[7, 8]), &scale(0.0_f32.to_bits()), &zero(8)),
            (0, 1),
        ),
        (
            compound(&codes(&[7, 8]), &scale(0.5_f32.to_bits()), &zero(16)),
            (0, 2),
        ),
    ] {
        let error = fixture.evaluate(&payload).unwrap_err();
        let EvaluationError::ValueConformance { key, rejection } = error else {
            panic!("a bound encoded input must be refused by the conformance vocabulary")
        };
        assert_eq!(key.as_ref().map(InputKey::as_str), Some("encoded"));
        assert_eq!(rejection.logical_index(), Some(expect.0));
        assert_eq!(rejection.component_ordinal(), Some(expect.1));
    }
    // The complete payload evaluates, which is what makes each refusal above a
    // property of the perturbation rather than of the fixture.
    let good = compound(&codes(&[7, 8]), &scale(0.5_f32.to_bits()), &zero(8));
    fixture.evaluate(&good).unwrap();
}

/// A swapped component structure is refused before any payload is read.
#[test]
fn a_swapped_component_structure_is_refused_structurally() {
    let fixture = DirectEncodedInput::new();
    let swapped = Tensor::compound(
        StrictAffineU4::resolved_type(),
        Shape::from_dims([2]),
        vec![
            ReferenceComponent::new(STRICT_AFFINE_CODES_ROLE, codes(&[7, 8])),
            ReferenceComponent::new(STRICT_AFFINE_ZERO_POINT_ROLE, zero(8)),
            ReferenceComponent::new(
                tiler_ir::semantic::STRICT_AFFINE_SCALE_ROLE,
                scale(0.5_f32.to_bits()),
            ),
        ],
    )
    .unwrap();
    let error = fixture.evaluate(&swapped).unwrap_err();
    let EvaluationError::ValueConformance { rejection, .. } = error else {
        panic!("a swapped structure must be refused by the conformance vocabulary")
    };
    assert!(matches!(
        rejection.cause(),
        ValueConformanceCause::ComponentRole { .. }
    ));
    assert_eq!(rejection.logical_index(), None);
}

/// An assembled result carries a composed proof and is never rescanned.
#[test]
fn an_assembled_result_holds_a_composed_proof_over_its_operands() {
    let codes_key = InputKey::new("codes").unwrap();
    let scale_key = InputKey::new("scale").unwrap();
    let zero_key = InputKey::new("zero").unwrap();
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let code_value = builder
        .input_resolved(
            codes_key.clone(),
            Shape::from_dims([2]),
            U4::resolved_type(),
        )
        .unwrap();
    let scale_value = builder
        .input_resolved(scale_key.clone(), Shape::new([]), F32::resolved_type())
        .unwrap();
    let zero_value = builder
        .input_resolved(zero_key.clone(), Shape::new([]), U4::resolved_type())
        .unwrap();
    let assembled = builder
        .apply(
            assemble_strict_affine_op(),
            OperationAttributes::empty(),
            &[code_value, scale_value, zero_value],
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
    let program = builder.build().unwrap();

    let code_payload = codes(&[7, 8]);
    let zero_payload = zero(8);
    let evaluate = |scale_bits: u32| {
        let scale_payload = scale(scale_bits);
        ReferenceEvaluator::standard().unwrap().evaluate(
            &program,
            &[
                InputBinding::new(&codes_key, &code_payload),
                InputBinding::new(&scale_key, &scale_payload),
                InputBinding::new(&zero_key, &zero_payload),
            ],
        )
    };
    evaluate(0.5_f32.to_bits()).unwrap();

    // A residual the payload disproves is refused by the operation, which is
    // what the composition depends on: reaching a composed proof means every
    // residual the occurrence declared was enforced against the payload.
    for invalid in [
        0.0_f32.to_bits(),
        f32::MIN_POSITIVE.to_bits() - 1,
        f32::INFINITY.to_bits(),
    ] {
        assert!(
            evaluate(invalid).is_err(),
            "{invalid:#010x} must be refused before a proof composes",
        );
    }
}
