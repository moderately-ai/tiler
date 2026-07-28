//! Structured-kernel construction, verification, and identity tests.
//!
//! Positive tests prove that the canonical lowering and an independently
//! hand-built producer kernel reach the same verified product and identity.
//! Each verification rule then has a negative test that builds a deliberately
//! wrong kernel through the public builder and asserts the exact typed
//! diagnostic, so a rejected kernel names the obligation it violated.

use std::cell::Cell;

use super::*;
use crate::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
    ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess,
    NumericalPermission, NumericalRealization, OwnershipProof, OwnershipProofKind,
    OwnershipWitnessId, PointwiseF32Expression, PointwiseF32ExpressionBuilder, ReductionTopology,
    RegionId, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
    VerifiedScheduledRegion,
};
use crate::shape::{Axis, Shape};

const NAN_BITS: u32 = 0x7fc0_0000;
const SCALE_BITS: u32 = 0x4000_0000;
const BIAS_BITS: u32 = 0x3f80_0000;

fn numerical() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        NAN_BITS,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

fn linear_schedule(work_items: u64, owner: OwnershipWitnessId) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: owner,
        reduction: ReductionTopology::None,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

fn scale_bias_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input().unwrap();
    let scale = expression.constant(scale_bits).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(bias_bits).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

/// A pointwise scale-then-bias region over `shape`.
fn pointwise_region(id: RegionId, shape: &Shape) -> VerifiedScheduledRegion {
    pointwise_expression_region(id, shape, scale_bias_expression(SCALE_BITS, BIAS_BITS))
}

fn pointwise_expression_region(
    id: RegionId,
    shape: &Shape,
    expression: PointwiseF32Expression,
) -> VerifiedScheduledRegion {
    let elements = crate::schedule::element_count(shape).expect("bounded fixture shape");
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(shape.clone()).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [(0, TensorRole::Input), (1, TensorRole::Intermediate)] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::PointwiseF32(expression))
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder
        .schedule(linear_schedule(elements, OwnershipWitnessId::new(0)))
        .unwrap();
    builder.build().unwrap()
}

#[test]
fn pointwise_lowering_preserves_left_and_right_operands_and_canonicalizes_each_operation() {
    let expressions = [
        {
            let mut expression = PointwiseF32ExpressionBuilder::new();
            let input = expression.input().unwrap();
            let two = expression.constant(2.0_f32.to_bits()).unwrap();
            let sum = expression.add(input, two).unwrap();
            let three = expression.constant(3.0_f32.to_bits()).unwrap();
            let root = expression.multiply(sum, three).unwrap();
            expression.build(root).unwrap()
        },
        {
            let mut expression = PointwiseF32ExpressionBuilder::new();
            let input = expression.input().unwrap();
            let two = expression.constant(2.0_f32.to_bits()).unwrap();
            let sum = expression.add(two, input).unwrap();
            let three = expression.constant(3.0_f32.to_bits()).unwrap();
            let root = expression.multiply(three, sum).unwrap();
            expression.build(root).unwrap()
        },
    ];

    for (position, expression) in expressions.into_iter().enumerate() {
        let scheduled = pointwise_expression_region(
            RegionId::new(u32::try_from(position).unwrap()),
            &Shape::from_dims([4]),
            expression,
        );
        let kernel = lower_scheduled_region(&scheduled).unwrap();
        let guarded = kernel
            .body()
            .operations()
            .find_map(|operation| match operation.view() {
                OperationView::Predicated { body, .. } => Some(body),
                _ => None,
            })
            .unwrap();
        let operations: Vec<_> = guarded.operations().map(OperationRef::view).collect();
        let binaries: Vec<_> = operations
            .iter()
            .filter_map(|operation| match operation {
                OperationView::Binary { op, lhs, rhs } => Some((*op, *lhs, *rhs)),
                _ => None,
            })
            .collect();
        assert_eq!(
            binaries.iter().map(|(op, _, _)| *op).collect::<Vec<_>>(),
            [BinaryOp::F32Add, BinaryOp::F32Multiply]
        );
        assert_eq!(
            operations
                .iter()
                .filter(|operation| matches!(
                    operation,
                    OperationView::Convert {
                        op: ConvertOp::CanonicalizeF32Nan,
                        ..
                    }
                ))
                .count(),
            2
        );
        let (_, add_lhs, add_rhs) = binaries[0];
        let (_, multiply_lhs, multiply_rhs) = binaries[1];
        if position == 0 {
            assert_eq!(kernel.value_constant(add_lhs).unwrap(), None);
            assert_eq!(
                kernel.value_constant(add_rhs).unwrap(),
                Some(KernelConstant::F32Bits(2.0_f32.to_bits()))
            );
            assert_eq!(kernel.value_constant(multiply_lhs).unwrap(), None);
            assert_eq!(
                kernel.value_constant(multiply_rhs).unwrap(),
                Some(KernelConstant::F32Bits(3.0_f32.to_bits()))
            );
        } else {
            assert_eq!(
                kernel.value_constant(add_lhs).unwrap(),
                Some(KernelConstant::F32Bits(2.0_f32.to_bits()))
            );
            assert_eq!(kernel.value_constant(add_rhs).unwrap(), None);
            assert_eq!(
                kernel.value_constant(multiply_lhs).unwrap(),
                Some(KernelConstant::F32Bits(3.0_f32.to_bits()))
            );
            assert_eq!(kernel.value_constant(multiply_rhs).unwrap(), None);
        }
    }
}

/// A strict serial sum over `axes` of `input`.
fn reduction_region(id: RegionId, input: &Shape, axes: &[Axis]) -> VerifiedScheduledRegion {
    let output = input.without_axes(axes);
    let output_elements = crate::schedule::element_count(&output).expect("bounded fixture shape");
    let contributor_map = LogicalAccess::ReductionContributor {
        input_shape: input.clone(),
        output_shape: output.clone(),
        axes: axes.to_vec(),
        order: ContributorOrder::OriginalAxisLexicographic,
    };
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(output).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            mode: AccessMode::Read,
            map: contributor_map,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input.clone(),
                output_shape: input.without_axes(axes),
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictSerialSum {
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
        })
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder.build().unwrap()
}

/// Declares the canonical pointwise signature.
fn pointwise_signature(
    builder: &mut KernelBuilder,
    scheduled: &VerifiedScheduledRegion,
    elements: u64,
) -> (KernelBufferId, KernelBufferId) {
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: elements,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: elements,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    (read, write)
}

/// Emits the canonical invocation index and its bounds predicate.
fn guard(builder: &mut KernelBuilder, work_items: u64) -> (KernelValueId, KernelValueId) {
    let invocation = builder.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let extent = builder.constant(KernelConstant::Index(work_items)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, invocation, extent)
        .unwrap();
    (invocation, active)
}

/// Emits the canonical scale-then-bias arithmetic with its NaN normalizations.
fn scale_bias(builder: &mut KernelBuilder, loaded: KernelValueId) -> KernelValueId {
    let scale = builder
        .constant(KernelConstant::F32Bits(SCALE_BITS))
        .unwrap();
    let product = builder
        .binary(BinaryOp::F32Multiply, loaded, scale)
        .unwrap();
    let product = builder
        .convert(ConvertOp::CanonicalizeF32Nan, product)
        .unwrap();
    let bias = builder
        .constant(KernelConstant::F32Bits(BIAS_BITS))
        .unwrap();
    let sum = builder.binary(BinaryOp::F32Add, product, bias).unwrap();
    builder.convert(ConvertOp::CanonicalizeF32Nan, sum).unwrap()
}

/// Builds the canonical pointwise kernel entirely through the public builder.
fn canonical_pointwise(scheduled: &VerifiedScheduledRegion, elements: u64) -> KernelBuilder {
    let mut builder = KernelBuilder::new(scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, scheduled, elements);
    let (invocation, active) = guard(&mut builder, elements);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    builder
}

fn diagnostics(builder: KernelBuilder) -> Vec<KernelDiagnostic> {
    builder.build().unwrap_err().into_parts().1
}

#[test]
fn canonical_lowering_produces_a_verified_backend_consumable_kernel() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let kernel = lower_scheduled_region(&scheduled).unwrap();

    assert_eq!(kernel.scheduled_region(), RegionId::new(0));
    assert_eq!(
        kernel.scheduled_region_identity(),
        scheduled.canonical_identity()
    );
    assert_eq!(kernel.numerical(), numerical());
    assert_eq!(kernel.requirements(), scheduled.requirements());
    assert_eq!(kernel.admitted_builtins(), [Builtin::GlobalInvocationIndex]);

    let buffers: Vec<_> = kernel.buffers().collect();
    assert_eq!(
        buffers,
        [
            BufferParameter {
                tensor: TensorRole::Input,
                element_type: KernelType::F32,
                address_space: AddressSpace::Device,
                access: BufferAccess::Read,
                element_count: 6,
            },
            BufferParameter {
                tensor: TensorRole::Intermediate,
                element_type: KernelType::F32,
                address_space: AddressSpace::Device,
                access: BufferAccess::Write,
                element_count: 6,
            },
        ]
    );

    // Every effect is lexically inside one explicit bounds-predicated region.
    let guarded: Vec<_> = kernel
        .body()
        .operations()
        .filter_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .collect();
    assert_eq!(guarded.len(), 1);
    let effects = guarded[0]
        .operations()
        .filter(|operation| {
            matches!(
                operation.view(),
                OperationView::Load { .. } | OperationView::Store { .. }
            )
        })
        .count();
    assert_eq!(effects, 2);
}

/// Compile-time tripwire for `revisit-kernel-body-single-spelling-gate`.
///
/// The refinement gate re-derives the canonical body with
/// `lower::derive_canonical` — a deterministic function of the scheduled region
/// — and requires structural equality, so the profile admits **exactly one
/// spelling** of a legal kernel. That is correct only while the surface is
/// narrow enough that no two genuinely different bodies are both legal for one
/// region. Past that point derive-and-compare starts rejecting *valid* kernels.
///
/// The ticket names that widening as its trigger for reconsideration, and a
/// trigger nobody is told about is a trigger nobody notices. These matches are
/// exhaustive with no wildcard arm, so adding a variant to any of the closed
/// vocabularies that decide a body's shape is a **compile error here**. Whoever
/// hits it should read that ticket before adding an arm: the fix may be to widen
/// the gate rather than to widen this match.
///
/// Deliberately a spelling check, not a semantic one — it cannot tell that a
/// widened vocabulary admits two bodies, only that the vocabulary widened, which
/// is the point at which a human has to look.
fn body_shaping_vocabulary_is_closed(
    binding: ExecutionBinding,
    tail: TailPolicy,
    access: &LogicalAccess,
    topology: &ReductionTopology,
    program: &ScalarProgram,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    (
        match binding {
            ExecutionBinding::GlobalLinearInvocation => "global-linear-invocation",
        },
        match tail {
            TailPolicy::Exact => "exact",
        },
        match access {
            LogicalAccess::LinearIdentity => "linear-identity",
            LogicalAccess::ReductionContributor { .. } => "reduction-contributor",
        },
        match topology {
            ReductionTopology::None => "none",
            ReductionTopology::Serial { .. } => "serial",
        },
        match program {
            ScalarProgram::PointwiseF32(_) => "pointwise-f32",
            ScalarProgram::StrictSerialSum { .. } => "strict-serial-sum",
            ScalarProgram::FusedMultiplyAddSerialSum { .. } => "fused-multiply-add-serial-sum",
        },
    )
}

/// The single-spelling gate's precondition still holds.
///
/// One execution binding and one tail policy is the substance of it: with a
/// single way to bind invocations to coordinates and no tail to handle, a
/// scheduled region's body has no legal degree of freedom for a producer to
/// spell differently. See [`body_shaping_vocabulary_is_closed`].
#[test]
fn the_single_spelling_profile_is_still_narrow_enough_for_derive_and_compare() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let region = scheduled.region();
    let names = body_shaping_vocabulary_is_closed(
        region.schedule.binding,
        region.schedule.tail,
        &region.index.accesses[0].map,
        &region.schedule.reduction,
        &region.index.scalar_program,
    );
    assert_eq!(
        names,
        (
            "global-linear-invocation",
            "exact",
            "linear-identity",
            "none",
            "pointwise-f32",
        )
    );
}

/// Collects every buffer handle a block's effects reference, descending into
/// predicated bodies.
fn referenced_buffers(block: BlockRef<'_>) -> Vec<VerifiedBufferId> {
    let mut found = Vec::new();
    for operation in block.operations() {
        match operation.view() {
            OperationView::Load { buffer, .. } | OperationView::Store { buffer, .. } => {
                found.push(buffer);
            }
            OperationView::Predicated { body, .. } => found.extend(referenced_buffers(body)),
            _ => {}
        }
    }
    found
}

/// The buffer handles a body references recover the signature, in handle order.
///
/// This is evidence for `pair-verified-buffer-handles-with-signature-ordinals`,
/// not a public guarantee. A backend that must emit an argument-table index per
/// load and store can today only recover the pairing by *sorting handles*,
/// which works solely because a verified handle is `(owner, index)` and every
/// handle of one kernel shares an owner — a private representation detail the
/// derived `Ord` exposes and no contract promises.
///
/// The test pins that the fact is already true, so publishing it is exposing an
/// invariant rather than computing a new one. It deliberately does not assert a
/// *position*, because no public accessor yields one; that is precisely the gap
/// the ticket asks the IR to close.
#[test]
fn referenced_buffer_handles_recover_the_signature_in_handle_order() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let kernel = lower_scheduled_region(&scheduled).unwrap();

    let mut referenced = referenced_buffers(kernel.body());
    referenced.sort_unstable();
    referenced.dedup();

    // Every signature parameter is referenced exactly once by this lowering, so
    // the recovered sequence is the whole signature rather than a prefix of it.
    assert_eq!(referenced.len(), kernel.buffers().len());
    let recovered: Vec<_> = referenced
        .iter()
        .map(|id| kernel.buffer(*id).unwrap())
        .collect();
    assert_eq!(recovered, kernel.buffers().collect::<Vec<_>>());

    // A handle from another kernel is rejected rather than silently resolving
    // to the same ordinal, which is what makes the pairing kernel-scoped.
    let other = lower_scheduled_region(&pointwise_region(
        RegionId::new(1),
        &Shape::from_dims([2, 3]),
    ))
    .unwrap();
    assert!(matches!(
        other.buffer(referenced[0]),
        Err(VerifiedKernelHandleError::ForeignKernel { .. })
    ));
}

#[test]
fn a_producer_built_canonical_kernel_verifies_and_equals_the_lowering() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let produced = canonical_pointwise(&scheduled, 6).build().unwrap();
    let lowered = lower_scheduled_region(&scheduled).unwrap();
    assert_eq!(produced, lowered);
    assert_eq!(
        produced.canonical_identity().as_bytes(),
        lowered.canonical_identity().as_bytes()
    );
}

#[test]
fn identity_is_independent_of_planning_ordinals_and_separates_content() {
    let first = lower_scheduled_region(&pointwise_region(
        RegionId::new(0),
        &Shape::from_dims([2, 3]),
    ))
    .unwrap();
    let renumbered = lower_scheduled_region(&pointwise_region(
        RegionId::new(7),
        &Shape::from_dims([2, 3]),
    ))
    .unwrap();
    assert_ne!(first.scheduled_region(), renumbered.scheduled_region());
    assert_eq!(
        first.canonical_identity().as_bytes(),
        renumbered.canonical_identity().as_bytes()
    );

    let wider = lower_scheduled_region(&pointwise_region(
        RegionId::new(0),
        &Shape::from_dims([2, 4]),
    ))
    .unwrap();
    assert_ne!(
        first.canonical_identity().as_bytes(),
        wider.canonical_identity().as_bytes()
    );

    // A kernel identity separates two regions that differ only in schedule.
    let reduction = lower_scheduled_region(&reduction_region(
        RegionId::new(0),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
    ))
    .unwrap();
    assert_ne!(
        first.canonical_identity().as_bytes(),
        reduction.canonical_identity().as_bytes()
    );
}

#[test]
fn a_reduction_lowers_to_a_bounded_loop_carrying_one_accumulator() {
    let scheduled = reduction_region(RegionId::new(1), &Shape::from_dims([2, 3]), &[Axis::new(1)]);
    let kernel = lower_scheduled_region(&scheduled).unwrap();
    let guarded = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .expect("a guarded region");
    let reduction = guarded
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::SerialLoop(reduction) => Some(reduction),
            _ => None,
        })
        .expect("a bounded reduction loop");
    assert_eq!((reduction.start(), reduction.end()), (1, 3));
    assert_eq!(reduction.initial().len(), 1);
    assert_eq!(reduction.accumulators().len(), 1);
    assert_eq!(reduction.yields().len(), 1);
    assert_eq!(
        kernel
            .value_type(reduction.accumulators().next().unwrap())
            .unwrap(),
        KernelType::F32
    );
    assert_eq!(
        kernel.value_type(reduction.induction().unwrap()).unwrap(),
        KernelType::Index
    );
}

#[test]
fn buffer_contract_rejects_a_signature_that_misstates_the_scheduled_access() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 7,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(builder), [KernelDiagnostic::BufferContract]);
}

#[test]
fn address_space_contract_rejects_a_space_the_schedule_does_not_provide() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            element_type: KernelType::F32,
            address_space: AddressSpace::Workgroup,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(
        diagnostics(builder),
        [KernelDiagnostic::AddressSpaceContract]
    );
}

#[test]
fn builtin_contract_rejects_a_kernel_that_never_admits_the_execution_binding() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let position = builder.constant(KernelConstant::Index(0)).unwrap();
    let extent = builder.constant(KernelConstant::Index(6)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, position, extent)
        .unwrap();
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, position, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                position,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(builder), [KernelDiagnostic::BuiltinContract]);
}

#[test]
fn numerical_and_resource_declarations_must_equal_the_schedule() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    let mut drifted = KernelBuilder::new(&scheduled).unwrap();
    let read = drifted
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = drifted
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    drifted
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    let mut wrong = numerical();
    wrong.canonical_arithmetic_nan_bits ^= 1;
    drifted.numerical(wrong).unwrap();
    drifted.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut drifted, 6);
    drifted
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(
        diagnostics(drifted),
        [KernelDiagnostic::NumericalRealization]
    );

    let mut inflated = KernelBuilder::new(&scheduled).unwrap();
    let read = inflated
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = inflated
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    inflated
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    inflated.numerical(numerical()).unwrap();
    let mut requirements = scheduled.requirements();
    requirements.local_memory_bytes += 1;
    inflated.requirements(requirements).unwrap();
    let (invocation, active) = guard(&mut inflated, 6);
    inflated
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(
        diagnostics(inflated),
        [KernelDiagnostic::ResourceRequirements]
    );
}

#[test]
fn predicate_dominance_rejects_unguarded_and_ungoverned_effects() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    // No predicate at all: the effects are not dominated by bounds evidence.
    let mut unguarded = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut unguarded, &scheduled, 6);
    let invocation = unguarded.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let loaded = unguarded
        .load(read, invocation, BoundsWitnessId::new(0))
        .unwrap();
    let value = scale_bias(&mut unguarded, loaded);
    unguarded
        .store(
            write,
            invocation,
            value,
            BoundsWitnessId::new(1),
            OwnershipWitnessId::new(0),
        )
        .unwrap();
    assert_eq!(
        diagnostics(unguarded),
        [KernelDiagnostic::PredicateDominance]
    );

    // A predicate that is not the scheduled bounds predicate is also rejected.
    let mut ungoverned = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut ungoverned, &scheduled, 6);
    let invocation = ungoverned.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let wrong_extent = ungoverned.constant(KernelConstant::Index(9)).unwrap();
    let active = ungoverned
        .compare(CompareOp::IndexLessThan, invocation, wrong_extent)
        .unwrap();
    ungoverned
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(
        diagnostics(ungoverned),
        [KernelDiagnostic::PredicateDominance]
    );
}

#[test]
fn bounds_and_ownership_evidence_must_be_the_scheduled_witnesses() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    let mut swapped = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut swapped, &scheduled, 6);
    let (invocation, active) = guard(&mut swapped, 6);
    swapped
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(0),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(swapped), [KernelDiagnostic::BoundsEvidence]);

    let mut disowned = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut disowned, &scheduled, 6);
    let (invocation, active) = guard(&mut disowned, 6);
    disowned
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(9),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(disowned), [KernelDiagnostic::OwnershipEvidence]);
}

#[test]
fn output_coverage_requires_exactly_one_owning_commit() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    let mut silent = KernelBuilder::new(&scheduled).unwrap();
    let (read, _write) = pointwise_signature(&mut silent, &scheduled, 6);
    let (invocation, active) = guard(&mut silent, 6);
    silent
        .predicated(active, |builder| {
            builder.load(read, invocation, BoundsWitnessId::new(0))?;
            Ok(())
        })
        .unwrap();
    assert_eq!(diagnostics(silent), [KernelDiagnostic::OutputCoverage]);

    let mut doubled = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut doubled, &scheduled, 6);
    let (invocation, active) = guard(&mut doubled, 6);
    doubled
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )?;
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(doubled), [KernelDiagnostic::OutputCoverage]);
}

#[test]
fn effect_ordering_requires_the_owning_commit_to_be_last() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )?;
            builder.load(read, invocation, BoundsWitnessId::new(0))?;
            Ok(())
        })
        .unwrap();
    assert_eq!(diagnostics(builder), [KernelDiagnostic::EffectOrdering]);
}

#[test]
fn a_barrier_the_schedule_does_not_require_is_rejected_explicitly() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    assert_eq!(scheduled.requirements().barriers, 0);
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            builder.barrier(BarrierSpec {
                execution_scope: ExecutionScope::Workgroup,
                memory_scope: MemoryScope::Device,
                fenced_spaces: vec![AddressSpace::Device],
                ordering: BarrierOrdering::AcquireRelease,
            })?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(
        diagnostics(builder),
        [KernelDiagnostic::BarrierCount {
            emitted: 1,
            required: 0,
        }]
    );
}

#[test]
fn reduction_contract_requires_the_scheduled_contributor_loop() {
    let scheduled = reduction_region(RegionId::new(1), &Shape::from_dims([2, 3]), &[Axis::new(1)]);
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 2,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut builder, 2);
    // Commit only the first contributor: structurally well formed, but it does
    // not realize the scheduled three-contributor serial reduction.
    builder
        .predicated(active, |builder| {
            let stride = builder.constant(KernelConstant::Index(3)).unwrap();
            let base = builder.binary(BinaryOp::IndexMultiply, invocation, stride)?;
            let loaded = builder.load(read, base, BoundsWitnessId::new(0))?;
            builder.store(
                write,
                invocation,
                loaded,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(builder), [KernelDiagnostic::ReductionContract]);
}

#[test]
fn body_refinement_rejects_a_structurally_legal_but_non_canonical_body() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    // Every structural obligation holds, but the numerical contract's NaN
    // normalization is missing after each arithmetic step.
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let scale = builder.constant(KernelConstant::F32Bits(SCALE_BITS))?;
            let product = builder.binary(BinaryOp::F32Multiply, loaded, scale)?;
            let bias = builder.constant(KernelConstant::F32Bits(BIAS_BITS))?;
            let value = builder.binary(BinaryOp::F32Add, product, bias)?;
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(builder), [KernelDiagnostic::BodyRefinement]);
}

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
        KernelDiagnostic::BarrierCount {
            emitted: 1,
            required: 0,
        }
        .rule(),
        "barrier-count"
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
