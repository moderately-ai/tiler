use super::super::{
    BinaryOp, Builtin, ConvertOp, KernelBuildError, KernelBuilder, KernelComponent, KernelConstant,
    KernelDiagnostic, KernelEntityKind, KernelLoweringError, KernelType, KernelValueId,
    SerialLoopSpec, VerifiedKernelHandleError, lower_scheduled_region,
};
use super::support::{
    canonical_pointwise, diagnostics, guard, numerical, pointwise_region, pointwise_signature,
    scale_bias,
};
use crate::schedule::{BoundsWitnessId, OwnershipWitnessId, RegionId};
use crate::shape::Shape;
use std::cell::Cell;

#[test]
fn an_incomplete_kernel_names_its_missing_component() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let builder = KernelBuilder::new(&scheduled).unwrap();
    assert_eq!(
        diagnostics(builder),
        [KernelDiagnostic::IncompleteKernel {
            component: KernelComponent::NumericalRealization,
        }]
    );
}

#[test]
fn a_rejected_kernel_returns_its_builder_intact_for_amend_and_retry() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            builder.load(read, invocation, BoundsWitnessId::new(0))?;
            Ok(())
        })
        .unwrap();
    let (mut recovered, diagnostics) = builder.build().unwrap_err().into_parts();
    assert_eq!(diagnostics, [KernelDiagnostic::OutputCoverage]);
    // The recovered builder still owns its buffers and values, so the caller can
    // append the missing commit instead of restarting construction.
    assert_eq!(recovered.derived_requirements(), scheduled.requirements());
    assert_eq!(
        recovered.store(
            write,
            invocation,
            invocation,
            BoundsWitnessId::new(1),
            OwnershipWitnessId::new(0),
        ),
        Err(KernelBuildError::TypeMismatch {
            expected: KernelType::F32,
            actual: KernelType::Index,
        })
    );
}

#[test]
fn a_handle_from_another_builder_or_kernel_is_rejected() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut first = KernelBuilder::new(&scheduled).unwrap();
    let (first_read, _) = pointwise_signature(&mut first, &scheduled, 6);
    let (first_invocation, _) = guard(&mut first, 6);

    let mut second = KernelBuilder::new(&scheduled).unwrap();
    let (second_read, _) = pointwise_signature(&mut second, &scheduled, 6);
    let (second_invocation, _) = guard(&mut second, 6);

    assert_eq!(
        second.load(first_read, second_invocation, BoundsWitnessId::new(0)),
        Err(KernelBuildError::ForeignHandle {
            entity: KernelEntityKind::Buffer,
        })
    );
    assert_eq!(
        second.load(second_read, first_invocation, BoundsWitnessId::new(0)),
        Err(KernelBuildError::ForeignHandle {
            entity: KernelEntityKind::Value,
        })
    );

    let owner = lower_scheduled_region(&scheduled).unwrap();
    let foreign = lower_scheduled_region(&scheduled).unwrap();
    let value = owner
        .body()
        .operations()
        .next()
        .expect("a first operation")
        .results()
        .next()
        .expect("a defined result");
    assert_eq!(owner.value_type(value), Ok(KernelType::Index));
    assert_eq!(
        foreign.value_type(value),
        Err(VerifiedKernelHandleError::ForeignKernel {
            entity: KernelEntityKind::Value,
        })
    );
}

#[test]
fn a_value_defined_in_a_closed_nested_block_leaves_scope() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    let escaped: Cell<Option<KernelValueId>> = Cell::new(None);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            escaped.set(Some(value));
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    let escaped = escaped.get().expect("a value defined inside the guard");
    assert_eq!(
        builder.convert(ConvertOp::CanonicalizeF32Nan, escaped),
        Err(KernelBuildError::ValueOutOfScope)
    );
}

#[test]
fn locally_decidable_operand_and_signature_rules_reject_at_insertion() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    assert_eq!(
        builder.builtin(Builtin::GlobalInvocationIndex),
        Err(KernelBuildError::UndeclaredBuiltin)
    );
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    assert_eq!(
        builder.admit_builtin(Builtin::GlobalInvocationIndex),
        Err(KernelBuildError::DuplicateAdmittedBuiltin)
    );
    assert_eq!(
        builder.numerical(numerical()),
        Err(KernelBuildError::ComponentAlreadySet {
            component: KernelComponent::NumericalRealization,
        })
    );
    assert_eq!(
        builder.requirements(scheduled.requirements()),
        Err(KernelBuildError::ComponentAlreadySet {
            component: KernelComponent::ResourceRequirements,
        })
    );

    let (invocation, _) = guard(&mut builder, 6);
    assert_eq!(
        builder.binary(BinaryOp::F32Add, invocation, invocation),
        Err(KernelBuildError::TypeMismatch {
            expected: KernelType::F32,
            actual: KernelType::Index,
        })
    );
    assert_eq!(
        builder.load(write, invocation, BoundsWitnessId::new(1)),
        Err(KernelBuildError::BufferAccessViolation)
    );
    let loaded = builder
        .load(read, invocation, BoundsWitnessId::new(0))
        .unwrap();
    assert_eq!(
        builder.store(
            read,
            invocation,
            loaded,
            BoundsWitnessId::new(0),
            OwnershipWitnessId::new(0),
        ),
        Err(KernelBuildError::BufferAccessViolation)
    );
    assert_eq!(
        builder.binary(BinaryOp::IndexDivide, invocation, invocation),
        Err(KernelBuildError::NonConstantDivisor)
    );
    let zero = builder.constant(KernelConstant::Index(0)).unwrap();
    assert_eq!(
        builder.binary(BinaryOp::IndexModulo, invocation, zero),
        Err(KernelBuildError::NonPositiveDivisor)
    );
}

#[test]
fn structured_loop_shape_and_yields_are_checked_at_insertion() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, _write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, _) = guard(&mut builder, 6);
    let seed = builder
        .load(read, invocation, BoundsWitnessId::new(0))
        .unwrap();

    assert_eq!(
        builder
            .serial_loop(SerialLoopSpec { start: 2, end: 2 }, &[seed], |_, _| Ok(
                Vec::new()
            ))
            .unwrap_err(),
        KernelBuildError::InvalidLoopRange { start: 2, end: 2 }
    );
    assert_eq!(
        builder
            .serial_loop(SerialLoopSpec { start: 0, end: 3 }, &[], |_, _| Ok(
                Vec::new()
            ))
            .unwrap_err(),
        KernelBuildError::EmptyLoopAccumulators
    );
    assert_eq!(
        builder
            .serial_loop(SerialLoopSpec { start: 0, end: 3 }, &[seed], |_, _| Ok(
                Vec::new()
            ))
            .unwrap_err(),
        KernelBuildError::LoopYieldArity {
            expected: 1,
            actual: 0,
        }
    );
    assert_eq!(
        builder
            .serial_loop(
                SerialLoopSpec { start: 0, end: 3 },
                &[seed],
                |_, parameters| Ok(vec![parameters.induction()]),
            )
            .unwrap_err(),
        KernelBuildError::LoopYieldTypeMismatch {
            position: 0,
            expected: KernelType::F32,
            actual: KernelType::Index,
        }
    );

    // Every failed nested insertion left the builder exactly as it was, so the
    // canonical body can still be completed and verified afterwards.
    let canonical = canonical_pointwise(&scheduled, 6).build().unwrap();
    assert_eq!(canonical, lower_scheduled_region(&scheduled).unwrap());
}

#[test]
fn diagnostics_and_errors_expose_stable_rule_identifiers() {
    assert_eq!(KernelDiagnostic::BodyRefinement.rule(), "body-refinement");
    assert_eq!(
        KernelDiagnostic::UnexpectedSynchronization.rule(),
        "unexpected-synchronization"
    );
    assert_eq!(
        KernelLoweringError::Verification(KernelDiagnostic::OutputCoverage).rule(),
        "output-coverage"
    );
    assert_eq!(
        KernelLoweringError::Construction(KernelBuildError::ValueOutOfScope).rule(),
        "kernel-construction"
    );
    assert_eq!(
        KernelLoweringError::UnsupportedRegion { rule: "fixture" }.rule(),
        "fixture"
    );
}
