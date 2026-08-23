use super::super::{
    AddressSpace, BinaryOp, BlockRef, BufferAccess, BufferParameter, Builtin, ConvertOp,
    KernelBuildError, KernelBuilder, KernelDiagnostic, KernelType, OperationView, SerialLoopRef,
    SerialLoopSpec, VerifiedKernel, lower_scheduled_region,
};
use super::support::{contraction_region, guard, numerical};
use crate::schedule::{BoundsWitnessId, OwnershipWitnessId, RegionId, TensorRole};

fn contraction_loop(kernel: &VerifiedKernel) -> SerialLoopRef<'_> {
    let guarded = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .expect("a guarded contraction");
    guarded
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::SerialLoop(reduction) => Some(reduction),
            _ => None,
        })
        .expect("a first-product contraction loop")
}

fn count_canonicalizations(kernel: &VerifiedKernel) -> usize {
    fn walk(block: BlockRef<'_>, count: &mut usize) {
        for operation in block.operations() {
            match operation.view() {
                OperationView::Convert {
                    op: ConvertOp::CanonicalizeF32Nan,
                    ..
                } => *count += 1,
                OperationView::Predicated { body, .. } => walk(body, count),
                OperationView::SerialLoop(serial) => walk(serial.body(), count),
                _ => {}
            }
        }
    }
    let mut count = 0;
    walk(kernel.body(), &mut count);
    count
}

/// The canonical lowering is a first-product separately-rounded fold.
///
/// This is the owning KIR classification: there is no fused multiply-add
/// construct and the loop starts at the first product. A simdgroup matrix
/// instruction is never formed here, so it is not a realization of `@1`.
#[test]
fn the_contraction_lowers_to_a_first_product_separately_rounded_fold() {
    let scheduled = contraction_region(RegionId::new(9), 2, 3, 4);
    let kernel = lower_scheduled_region(&scheduled).expect("the direct contraction lowers");
    let reduction = contraction_loop(&kernel);
    assert_eq!(
        (reduction.start(), reduction.end()),
        (1, 4),
        "the accumulator must start at the first product"
    );
    assert_eq!(
        count_canonicalizations(&kernel),
        3,
        "the seed product, the fold product, and the fold sum each canonicalize"
    );
}

/// A `+0.0` seed is `reduction-contract`, not a realization of `@1`.
///
/// The subject is the loop start. Fusion and NaN sites are left unperturbed so
/// this refusal cannot be confused with either of those obligations.
#[test]
fn a_positive_zero_seeded_contraction_loop_is_reduction_contract() {
    let scheduled = contraction_region(RegionId::new(9), 2, 3, 4);
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let left = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 8,
        })
        .unwrap();
    let right = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 12,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            component_role: None,
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
            let left_value = builder.load(left, invocation, BoundsWitnessId::new(0))?;
            let right_value = builder.load(right, invocation, BoundsWitnessId::new(1))?;
            let product = builder.binary(BinaryOp::F32Multiply, left_value, right_value)?;
            let seed = builder.convert(ConvertOp::CanonicalizeF32Nan, product)?;
            let results = builder.serial_loop(
                SerialLoopSpec { start: 0, end: 4 },
                &[seed],
                |builder, parameters| {
                    let accumulator = parameters
                        .accumulator(0)
                        .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                    let product = builder.convert(ConvertOp::CanonicalizeF32Nan, accumulator)?;
                    let sum = builder.binary(BinaryOp::F32Add, accumulator, product)?;
                    let sum = builder.convert(ConvertOp::CanonicalizeF32Nan, sum)?;
                    Ok(vec![sum])
                },
            )?;
            let total = results
                .get(0)
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
            builder.store(
                write,
                invocation,
                total,
                BoundsWitnessId::new(2),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    let error = builder
        .build()
        .expect_err("a +0.0-seeded contraction loop must not verify");
    assert_eq!(
        error.diagnostics(),
        [KernelDiagnostic::ReductionContract],
        "the seed subject must fail as reduction-contract, not as a later catch-all: {error:?}"
    );
    assert_eq!(
        error.diagnostics()[0].rule(),
        "reduction-contract",
        "the quoted seed refusal is the stable rule id"
    );
}
