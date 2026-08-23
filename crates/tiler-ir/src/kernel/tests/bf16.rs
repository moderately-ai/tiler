use super::super::{
    AddressSpace, BinaryOp, BufferAccess, BufferParameter, Builtin, ConvertOp, KernelBufferId,
    KernelBuildError, KernelBuilder, KernelConstant, KernelDiagnostic, KernelType, OperationView,
    lower_scheduled_region,
};
use super::support::{
    BF16_NAN_BITS, NAN_BITS, SCALE_BITS, bf16_numerical, binary_op_counts, diagnostics, guard,
    linear_schedule, pointwise_region, region_numerical_mut, scale_bias,
};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    LogicalAccess, OwnershipProof, OwnershipProofKind, OwnershipWitnessId, RegionId, RegionProgram,
    ScalarProgram, ScheduledRegionBuilder, TensorRole, VerifiedScheduledRegion,
};
use crate::shape::Shape;

// ---------------------------------------------------------------------------
// BF16
// ---------------------------------------------------------------------------

/// `bf16` 2.0, 1.0, and the `f32` canonical arithmetic NaN payload's width error.
const BF16_SCALE_BITS: u16 = 0x4000;
const BF16_BIAS_BITS: u16 = 0x3f80;

/// `(x * 2.0) + 1.0` in `bf16`, the direct sibling of [`scale_bias_expression`].
fn bf16_scale_bias_expression() -> crate::schedule::PointwiseBf16Expression {
    let mut expression = crate::schedule::PointwiseBf16ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(BF16_SCALE_BITS).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(BF16_BIAS_BITS).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

/// A `bf16` pointwise region over `shape`, built through the public builder.
fn bf16_pointwise_region(id: RegionId, shape: &Shape) -> VerifiedScheduledRegion {
    bf16_pointwise_builder(id, shape).build().unwrap()
}

fn bf16_pointwise_builder(id: RegionId, shape: &Shape) -> ScheduledRegionBuilder {
    let elements = crate::schedule::element_count(shape).expect("bounded fixture shape");
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(shape.clone()).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
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
                component_role: None,
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseBf16(bf16_scale_bias_expression()),
            numerical: bf16_numerical(),
        })
        .unwrap();
    builder
        .schedule(linear_schedule(elements, OwnershipWitnessId::new(0)))
        .unwrap();
    builder
}

/// A BF16 region lowers to a kernel that is BF16 in every position.
///
/// The check is on every position rather than on the arithmetic alone: a
/// vocabulary that admitted the type but declared `f32` buffers, or that reused
/// the `f32` canonicalization, would compute something the region does not mean
/// while still passing an "is there a BF16 kernel" question.
#[test]
fn a_bf16_pointwise_region_lowers_to_a_kernel_that_is_bf16_at_every_position() {
    let scheduled = bf16_pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let kernel = lower_scheduled_region(&scheduled).unwrap();

    assert_eq!(
        kernel.buffers().collect::<Vec<_>>(),
        [
            BufferParameter {
                tensor: TensorRole::Input,
                component_role: None,
                element_type: KernelType::Bf16,
                address_space: AddressSpace::Device,
                access: BufferAccess::Read,
                element_count: 6,
            },
            BufferParameter {
                tensor: TensorRole::Intermediate,
                component_role: None,
                element_type: KernelType::Bf16,
                address_space: AddressSpace::Device,
                access: BufferAccess::Write,
                element_count: 6,
            },
        ]
    );
    assert_eq!(binary_op_counts(&kernel, BinaryOp::Bf16Multiply), 1);
    assert_eq!(binary_op_counts(&kernel, BinaryOp::Bf16Add), 1);
    // The `f32` neighbours are absent, so the arithmetic is a function of the
    // region's width rather than of whatever the emitter reached for first.
    assert_eq!(binary_op_counts(&kernel, BinaryOp::F32Multiply), 0);
    assert_eq!(binary_op_counts(&kernel, BinaryOp::F32Add), 0);

    let guarded = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .expect("the canonical body guards its effects");
    let mut constants = Vec::new();
    let mut conversions = Vec::new();
    for operation in guarded.operations() {
        match operation.view() {
            OperationView::Constant { value } => constants.push(value),
            OperationView::Convert { op, .. } => conversions.push(op),
            _ => {}
        }
    }
    assert_eq!(
        constants,
        [
            KernelConstant::Bf16Bits(BF16_SCALE_BITS),
            KernelConstant::Bf16Bits(BF16_BIAS_BITS),
        ],
        "each constant carries its own sixteen-bit payload unchanged"
    );
    assert_eq!(
        conversions,
        [
            ConvertOp::CanonicalizeBf16Nan,
            ConvertOp::CanonicalizeBf16Nan
        ],
        "every arithmetic result is canonicalized at the bf16 width"
    );

    // The produced kernel is a refinement a hand-built producer reaches too,
    // which is what makes the lowering checked rather than definitional.
    let produced = canonical_bf16_pointwise(&scheduled, 6).build().unwrap();
    assert_eq!(produced, kernel);
    assert_eq!(
        produced.canonical_identity().as_bytes(),
        kernel.canonical_identity().as_bytes()
    );
}

/// Builds the canonical BF16 pointwise kernel by hand.
fn canonical_bf16_pointwise(scheduled: &VerifiedScheduledRegion, elements: u64) -> KernelBuilder {
    let mut builder = KernelBuilder::new(scheduled).unwrap();
    let (read, write) =
        bf16_pointwise_signature(&mut builder, scheduled, elements, KernelType::Bf16);
    let (invocation, active) = guard(&mut builder, elements);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let scale = builder.constant(KernelConstant::Bf16Bits(BF16_SCALE_BITS))?;
            let product = builder.binary(BinaryOp::Bf16Multiply, loaded, scale)?;
            let product = builder.convert(ConvertOp::CanonicalizeBf16Nan, product)?;
            let bias = builder.constant(KernelConstant::Bf16Bits(BF16_BIAS_BITS))?;
            let biased = builder.binary(BinaryOp::Bf16Add, product, bias)?;
            let biased = builder.convert(ConvertOp::CanonicalizeBf16Nan, biased)?;
            builder.store(
                write,
                invocation,
                biased,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    builder
}

/// Declares the BF16 pointwise signature at a caller-chosen element type.
///
/// The type is a parameter so the refusal test below can vary exactly one thing.
fn bf16_pointwise_signature(
    builder: &mut KernelBuilder,
    scheduled: &VerifiedScheduledRegion,
    elements: u64,
    element_type: KernelType,
) -> (KernelBufferId, KernelBufferId) {
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: elements,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: elements,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(bf16_numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    (read, write)
}

/// A kernel mixing BF16 and F32 is refused by name, at both places it can mix.
///
/// **Two mixes, two authorities.** A *signature* that binds `f32` buffers to a
/// BF16 region is refused by whole-kernel verification as `BufferContract`,
/// because the expected element type is derived from the region's scalar program.
/// A *body* that feeds an `f32` value into BF16 arithmetic, or applies the `f32`
/// canonicalization to a BF16 value, never reaches verification at all: the
/// builder's insertion-time type check names both the expected and the actual
/// type.
///
/// Each refusal is stated beside a passing neighbour that differs in exactly one
/// argument, so it is visibly a function of the width rather than of the fixture
/// having stopped working.
#[test]
fn a_kernel_mixing_bf16_and_f32_is_refused_by_name() {
    let scheduled = bf16_pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    // The signature mix: `f32` buffers under a BF16 region.
    let mut mixed = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = bf16_pointwise_signature(&mut mixed, &scheduled, 6, KernelType::F32);
    let (invocation, active) = guard(&mut mixed, 6);
    mixed
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
    assert_eq!(diagnostics(mixed), [KernelDiagnostic::BufferContract]);
    // The same body over BF16 buffers verifies, so the refusal is the element
    // type and not the shape of the kernel.
    assert!(canonical_bf16_pointwise(&scheduled, 6).build().is_ok());

    // The body mix: an `f32` value as a BF16 operand, and the `f32`
    // canonicalization over a BF16 value.
    let mut body = KernelBuilder::new(&scheduled).unwrap();
    let (read, _) = bf16_pointwise_signature(&mut body, &scheduled, 6, KernelType::Bf16);
    let (invocation, _) = guard(&mut body, 6);
    let loaded = body
        .load(read, invocation, BoundsWitnessId::new(0))
        .unwrap();
    let f32_constant = body.constant(KernelConstant::F32Bits(SCALE_BITS)).unwrap();
    assert_eq!(
        body.binary(BinaryOp::Bf16Multiply, loaded, f32_constant),
        Err(KernelBuildError::TypeMismatch {
            expected: KernelType::Bf16,
            actual: KernelType::F32,
        })
    );
    assert_eq!(
        body.convert(ConvertOp::CanonicalizeF32Nan, loaded),
        Err(KernelBuildError::TypeMismatch {
            expected: KernelType::F32,
            actual: KernelType::Bf16,
        })
    );
    // The BF16 constant in the same position is admitted, so the refusals are
    // about the operand width rather than about the builder having stopped.
    let bf16_constant = body
        .constant(KernelConstant::Bf16Bits(BF16_SCALE_BITS))
        .unwrap();
    assert!(
        body.binary(BinaryOp::Bf16Multiply, loaded, bf16_constant)
            .is_ok()
    );
}

/// A BF16 region declaring the `f32` canonical NaN payload is refused.
///
/// The invariant `NumericalRealization::canonical_arithmetic_nan_bits` documents
/// — the region's own arithmetic type's pattern, zero-extended — is checked here
/// rather than assumed, so the reading cannot silently differ between the
/// verifier and the lowering that installs the payload.
#[test]
fn a_bf16_region_declaring_the_f32_canonical_nan_payload_is_refused() {
    let mut wrong_width = bf16_pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]))
        .region()
        .clone();
    region_numerical_mut(&mut wrong_width).canonical_arithmetic_nan_bits = NAN_BITS;
    assert_eq!(
        ScheduledRegionBuilder::from_region(wrong_width.clone())
            .build()
            .unwrap_err()
            .diagnostics(),
        [crate::schedule::ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );

    // A payload that is the right pattern in the wrong place is refused too: the
    // sixteen bits go in the low half, not the high one a truncating reader
    // would produce.
    region_numerical_mut(&mut wrong_width).canonical_arithmetic_nan_bits = BF16_NAN_BITS << 16;
    assert_eq!(
        ScheduledRegionBuilder::from_region(wrong_width.clone())
            .build()
            .unwrap_err()
            .diagnostics(),
        [crate::schedule::ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );

    // Restoring the declared payload readmits the region, so the refusal is the
    // payload and not the perturbation having broken the fixture.
    region_numerical_mut(&mut wrong_width).canonical_arithmetic_nan_bits = BF16_NAN_BITS;
    assert!(
        ScheduledRegionBuilder::from_region(wrong_width)
            .build()
            .is_ok()
    );
}

/// The BF16 vocabulary appended, so no `f32` subject's identity bytes moved.
///
/// Stated as an inequality between two whole identities rather than as a claim
/// about tags: the two regions differ only in the width of their arithmetic, so
/// identities that agreed would mean the scalar-program tag or a node payload
/// was not encoded at all. The `f32` side's own byte pin lives with the schedule
/// builder (`STRICT_F32_REGION_IDENTITY_HEX`), which this widening leaves
/// unchanged.
#[test]
fn a_bf16_kernel_and_its_f32_sibling_do_not_share_identity() {
    let shape = Shape::from_dims([2, 3]);
    let bf16 = lower_scheduled_region(&bf16_pointwise_region(RegionId::new(0), &shape)).unwrap();
    let f32 = lower_scheduled_region(&pointwise_region(RegionId::new(0), &shape)).unwrap();
    assert_ne!(
        bf16.canonical_identity().as_bytes(),
        f32.canonical_identity().as_bytes()
    );
    // Both identities open with the same domain separator: the `bf16` widening
    // was an append, not a version step. The domain has since moved to `v7` for
    // an unrelated reason -- the derived index-arithmetic requirement landing
    // inside the fixed resource-requirement record -- which is why this names
    // the current domain while the comment above still describes the widening
    // this test is about.
    let domain = b"tiler.kernel.v9\0";
    assert!(bf16.canonical_identity().as_bytes().starts_with(domain));
    assert!(f32.canonical_identity().as_bytes().starts_with(domain));
}

/// The BF16 region's derived subnormal freedom is unproven, at both layers.
#[test]
fn a_bf16_region_proves_no_subnormal_freedom() {
    let scheduled = bf16_pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    assert_eq!(
        scheduled.subnormal_freedom(),
        crate::schedule::SubnormalFreedom::Unproven
    );
    let kernel = lower_scheduled_region(&scheduled).unwrap();
    assert_eq!(
        kernel.subnormal_freedom(),
        crate::schedule::SubnormalFreedom::Unproven
    );
    // The one freedom this vocabulary states does not reach bf16 even where it
    // holds, which is why the region above could not have inherited one.
    assert!(
        !crate::schedule::SubnormalFreedom::StrictAffineNormalScaleDecode
            .discharges(crate::schedule::ArithmeticType::Bf16)
    );
}
