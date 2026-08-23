use super::support::{Bf16Canonicalization, interpret_bf16, semantic};
use super::*;

/// The `(x * 1.0) + 2.0` program in BF16, as the pipeline's own fixture.
fn bf16_scale_bias_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    // 1.0 and 2.0 in bf16.
    let one = Bf16Constant::apply(&mut builder, 0x3f80).unwrap();
    let two = Bf16Constant::apply(&mut builder, 0x4000).unwrap();
    let scaled = Bf16Multiply::apply(&mut builder, input, one).unwrap();
    let shifted = Bf16Add::apply(&mut builder, scaled, two).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), shifted)
        .unwrap();
    builder.build().unwrap()
}

/// The governed baseline refuses a pure-BF16 program by its own dtype row.
///
/// The gap this guards is the one registration opens: `builder.build()` now
/// succeeds for BF16, and "the program verifies" is the step most easily
/// mistaken for "the dtype works". The governed baseline declares
/// dispatchability for `tiler::f32@1` and says nothing about `tiler::bf16@1`, so
/// the dtype is `Unknown` to it and the program is rejected *per target*: the
/// refusal names the profile that could not take the program rather than a
/// property of the request.
///
/// **The stated contract is BF16's, and that is now load-bearing rather than
/// incidental.** `CompilationRequest::governed` states the strict `f32`
/// contract, and while the recognizer refused every non-`f32` program the
/// pairing never had to be examined. It does now: a contract's arithmetic is
/// part of its identity, so an `f32` contract stated for a BF16 program is
/// refused before any target is consulted and this test would have been
/// asserting *that* refusal instead of the profile's own dtype row.
/// `an_f32_contract_stated_for_a_bf16_program_is_refused_before_any_target` is
/// where the pairing is asserted, and stating a BF16 contract here is what keeps
/// this test about the dtype row it names.
#[test]
fn a_pure_bf16_program_is_statable_and_refused_at_the_request_boundary() {
    let bf16_program = bf16_scale_bias_program();
    assert_eq!(bf16_program.operation_count(), 4);

    let product = compile(CompilationRequest::governed_under(
        &bf16_program,
        crate::session::NumericalContract::STRICT_BF16.resolve(),
    ))
    .expect("a target-local dtype refusal is a batch outcome, not a request error");
    let [target] = product.targets.as_slice() else {
        panic!("the governed request names exactly one target");
    };
    let Some(CompileError::NoFeasiblePlan(NoFeasiblePlanError::Request(
        RequestError::DTypeNotDispatchable {
            target_profile,
            resolved_type,
            disposition,
        },
    ))) = target.failure()
    else {
        panic!("expected the profile's own dtype refusal, got {target:?}");
    };
    assert_eq!(
        resolved_type.as_ref(),
        &tiler_ir::semantic::Bf16::resolved_type(),
        "the refusal names the exact value identity the profile was asked about",
    );
    assert_eq!(
        *disposition,
        crate::request::DTypeDispatchRefusalDisposition::Unknown,
        "silence about bf16 is Unknown, never an inherited f32 verdict",
    );
    assert_eq!(
        target_profile.as_str(),
        target.target_profile().profile_key().as_str(),
    );

    // The neighbour that keeps this about bf16 rather than about a dead
    // request path: the same shape of program in f32 compiles.
    compile(CompilationRequest::governed(&semantic(false)))
        .expect("the governed f32 fixture still compiles");
}

/// An `f32` contract stated for a BF16 program is refused before any target.
///
/// **The refusal the recognizer's `dtype-f32` rule used to absorb.** That rule
/// refused every non-`f32` program, so a caller pairing a program with a
/// contract about another width was caught incidentally and reported as an
/// unrecognized dtype. Recognition now admits the program, and the pairing is
/// refused on its own terms: ADR 0076 item 6 makes a contract's arithmetic part
/// of its identity and a target's honourability rows are keyed by subject, so
/// there is no declaration any profile could make that would answer the
/// question — which is why the refusal precedes every target rather than being
/// one target's.
///
/// The stated list is reported whole, in the caller's order, so a caller that
/// named two inapplicable contracts can see both rather than only the first.
#[test]
fn an_f32_contract_stated_for_a_bf16_program_is_refused_before_any_target() {
    let program = bf16_scale_bias_program();
    let error = compile(CompilationRequest::governed(&program))
        .expect_err("an inapplicable preference is a request error, not a target outcome");
    let CompileError::InvalidRequest(RequestError::NoApplicableNumericalContract {
        program: arithmetic,
        stated,
    }) = &error
    else {
        panic!("expected the contract-applicability refusal, got {error:?}");
    };
    assert_eq!(*arithmetic, tiler_ir::schedule::ArithmeticType::Bf16);
    let [(key, stated_arithmetic)] = stated.as_slice() else {
        panic!("the governed request states exactly one contract");
    };
    assert_eq!(
        *stated_arithmetic,
        tiler_ir::schedule::ArithmeticType::F32,
        "the refusal names the arithmetic the stated contract resolves",
    );
    assert_eq!(
        *key,
        crate::request::StrictF32NumericalContract::governed().key,
        "the refusal names the exact contract the caller stated",
    );

    // The neighbour that keeps this about the *pairing*: the same program under
    // a contract of its own width is admitted past this check and reaches the
    // governed profile's dtype row, which the test above asserts.
    assert!(
        compile(CompilationRequest::governed_under(
            &program,
            crate::session::NumericalContract::STRICT_BF16.resolve(),
        ))
        .is_ok(),
        "an applicable contract passes this check and leaves the answer to the target",
    );
}

/// The `bf16` encodings this vertical's witnesses are stated in.
mod bf16_bits {
    /// `3.0`, the scale: a multiplier that forces a rounding at every ordinary
    /// operand rather than one that would pass under any implementation.
    pub(super) const THREE: u16 = 0x4040;
    /// `-0.0`, the bias: chosen so a signed zero and a subnormal survive the
    /// whole expression instead of being swallowed by a nonzero addend.
    pub(super) const NEGATIVE_ZERO: u16 = 0x8000;
    pub(super) const LEAST_POSITIVE_SUBNORMAL: u16 = 0x0001;
    pub(super) const LEAST_NEGATIVE_SUBNORMAL: u16 = 0x8001;
    /// A quiet NaN whose payload is *not* the family's canonical one.
    pub(super) const NONCANONICAL_NAN: u16 = 0x7fc1;
    pub(super) const CANONICAL_NAN: u16 = 0x7fc0;
    pub(super) const POSITIVE_INFINITY: u16 = 0x7f80;
    pub(super) const NEGATIVE_INFINITY: u16 = 0xff80;
    pub(super) const MAX_FINITE: u16 = 0x7f7f;
    /// `1 + 2^-7`, whose product with `3.0` lands exactly on a rounding tie.
    pub(super) const ONE_PLUS_ULP: u16 = 0x3f81;
}

/// The boundary payload the BF16 vertical is driven over.
///
/// Every class the region's arithmetic can distinguish, in one tensor: a signed
/// zero, both least subnormals, a non-canonical NaN, both infinities, the
/// overflow boundary from both sides, and a tie the rounding rule has to break.
const BF16_WITNESSES: [u16; 10] = [
    bf16_bits::NEGATIVE_ZERO,
    bf16_bits::LEAST_POSITIVE_SUBNORMAL,
    bf16_bits::LEAST_NEGATIVE_SUBNORMAL,
    bf16_bits::NONCANONICAL_NAN,
    bf16_bits::POSITIVE_INFINITY,
    bf16_bits::NEGATIVE_INFINITY,
    bf16_bits::MAX_FINITE,
    0xff7f,
    bf16_bits::ONE_PLUS_ULP,
    0x3f80,
];

/// The semantic `(x * 3.0) + (-0.0)` program in BF16, for the reference oracle.
fn bf16_semantic_program(key: &InputKey, elements: u64) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<Bf16>(key.clone(), Shape::from_dims([elements]))
        .unwrap();
    let scale = Bf16Constant::apply(&mut builder, bf16_bits::THREE).unwrap();
    let product = Bf16Multiply::apply(&mut builder, input, scale).unwrap();
    let bias = Bf16Constant::apply(&mut builder, bf16_bits::NEGATIVE_ZERO).unwrap();
    let root = Bf16Add::apply(&mut builder, product, bias).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// The same computation as a verified BF16 scheduled region.
///
/// Assembled through `tiler-ir`'s public builders rather than through
/// `compile()`, and the reason has changed rather than gone away. It used to be
/// the recognizer's `dtype-f32` rule, which refused every non-`f32` program
/// before a subject was ever normalized, so no BF16 region was reachable from
/// the request boundary at all. That rule is gone: a BF16 program is recognized,
/// planned and selected, which
/// `crates/tiler-compiler/tests/bf16_numerical_contract.rs`'s
/// `a_flush_accepting_bf16_contract_reaches_a_selected_plan` asserts.
///
/// **The fusion wall was the second reason and it is gone too; what keeps this
/// region hand-assembled is the realization it needs *stated* rather than
/// resolved.** This fixture is a `(x * 3.0) + (-0.0)` chain, whose region covers
/// four occurrences, and a multi-occurrence region is put to
/// `derive_fusion_legality` before any cover survives — an authority that now
/// carries governed rows for the three registered BF16 families, so such a
/// region derives its own legality and fuses, which
/// `crates/tiler-compiler/tests/bf16_numerical_contract.rs`'s
/// `a_multi_occurrence_bf16_program_derives_its_own_fusion_legality` asserts.
/// What survives there is narrower and is asserted separately: a BF16 region
/// under a contraction-*permitting* contract still stops, at
/// `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall`.
/// And where a compiled BF16 request still stops on the profile this build
/// ships, it stops a phase *earlier* than fusion legality — the authoritative
/// macOS Apple9 ledger states its reshaping rows for the `f32` subject alone, so
/// a flush-accepting BF16 contract clears the measured subnormal dimensions and
/// meets contraction undeclared, which
/// `the_measured_subnormal_rows_alone_leave_the_remaining_dimensions_unknown`
/// asserts. Neither boundary is this fixture's.
///
/// The two tests below need a region whose numerical realization is the strict
/// BF16 vector — both subnormal dimensions preserving — written down here and
/// handed straight to `lower_scheduled_region`, so the interpreted kernel and
/// the reference oracle are compared over preserved subnormals rather than over
/// whatever a contract and a profile resolved between them. So what is
/// established here stays what it was: that the schedule, kernel, and
/// physical-carrier vocabularies admit and verify this region.
fn bf16_scheduled_region(elements: u64) -> tiler_ir::schedule::VerifiedScheduledRegion {
    use tiler_ir::schedule::{
        Access, AccessMode, AccessOrdinal, ApproximationEnvelope, BoundsProof, BoundsProofKind,
        BoundsWitnessId, ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan,
        LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof,
        OwnershipProofKind, OwnershipWitnessId, PointwiseBf16ExpressionBuilder, ReductionTopology,
        ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
    };

    let mut expression = PointwiseBf16ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(bf16_bits::THREE).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(bf16_bits::NEGATIVE_ZERO).unwrap();
    let root = expression.add(product, bias).unwrap();
    let expression = expression.build(root).unwrap();

    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder
        .iteration_shape(Shape::from_dims([elements]))
        .unwrap();
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
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [(0, TensorRole::Input), (1, TensorRole::Output)] {
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
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseBf16(expression),
            numerical: NumericalRealization::new(
                "tiler.test.strict-bf16",
                u32::from(tiler_ir::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS),
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
            ),
        })
        .unwrap();
    // The accepted `tiler.contract.bf16.v1` strict vector, restated as the
    // region's own realization: preserving subnormals in both dimensions, every
    // numeric-reshaping permission withheld, no exceptional value assumed
    // absent, and the family's canonical arithmetic NaN payload zero-extended
    // into the thirty-two-bit field. `NumericalContract::STRICT_BF16` resolves
    // exactly these dimensions;
    builder
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: elements,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: elements,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    builder.build().unwrap()
}

fn bf16_tensor(elements: &[u16]) -> Tensor {
    Tensor::dense(
        Bf16::resolved_type(),
        Shape::from_dims([u64::try_from(elements.len()).unwrap()]),
        elements
            .iter()
            .map(|encoding| {
                ReferenceElement::from_float_bits(
                    encoding.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("a two-byte payload is a bounded element")
            })
            .collect(),
    )
    .expect("a bounded bf16 tensor")
}

fn bf16_tensor_bits(tensor: &Tensor) -> Vec<u16> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a bf16 result is dense");
    };
    elements
        .iter()
        .map(|element| {
            u16::from_be_bytes(
                <[u8; 2]>::try_from(element.as_bytes()).expect("a bf16 element is two bytes"),
            )
        })
        .collect()
}

/// A BF16 kernel's interpreted result agrees bit for bit with the reference.
///
/// **The two sides are independent.** The reference decodes each operand to its
/// exact rational value from the registered descriptor's own fields and rounds
/// once; the machine holds `bf16` encodings, computes in `f64`, and rounds with
/// its own half-to-even implementation. Neither consults the other, and neither
/// consults the semantic graph while executing the kernel: the machine resolves
/// buffer extents, addressing, predication, the constants, the arithmetic width,
/// and the NaN canonicalization from the structured kernel alone.
///
/// **The comparison runs in `tiler-compiler`** because it is the only crate that
/// sees both `tiler-ir`'s lowering and `tiler-reference`'s oracle: `tiler-ir`
/// declares no dependency on the reference and the reference depends on
/// `tiler-ir`, so an in-crate comparison would be against an oracle that is not
/// independent — the weaker claim, recorded as the stronger one.
#[test]
fn a_bf16_kernel_agrees_with_the_reference_oracle_bit_for_bit() {
    let elements = u64::try_from(BF16_WITNESSES.len()).unwrap();
    let scheduled = bf16_scheduled_region(elements);
    let kernel = tiler_ir::kernel::lower_scheduled_region(&scheduled)
        .expect("the bf16 region lowers to a verified kernel");

    // The physical carrier the compiler derives for this region is two bytes
    // wide, which is the other half of this ticket's stated outcome.
    assert_eq!(
        tiler_ir::program::StorageScalar::Bf16.byte_width(),
        2,
        "the bf16 carrier is two bytes at its single width authority"
    );

    let key = InputKey::new("x").unwrap();
    let program = bf16_semantic_program(&key, elements);
    let tensor = bf16_tensor(&BF16_WITNESSES);
    let outputs = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&program, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    let expected = bf16_tensor_bits(&outputs[0]);

    let interpreted = interpret_bf16(&kernel, &BF16_WITNESSES, Bf16Canonicalization::Applied);
    assert_eq!(
        interpreted, expected,
        "the kernel's interpreted result differs from the reference"
    );

    // The witnesses reached the classes they were chosen for, so a fixture that
    // silently stopped exercising them would fail here rather than pass.
    assert_eq!(interpreted[0], bf16_bits::NEGATIVE_ZERO);
    assert_eq!(interpreted[1], 0x0003, "three least positive quanta");
    assert_eq!(interpreted[2], 0x8003, "three least negative quanta");
    assert_eq!(interpreted[3], bf16_bits::CANONICAL_NAN);
    assert_eq!(interpreted[4], bf16_bits::POSITIVE_INFINITY);
    assert_eq!(interpreted[5], bf16_bits::NEGATIVE_INFINITY);
    assert_eq!(
        interpreted[6],
        bf16_bits::POSITIVE_INFINITY,
        "three times the greatest finite value overflows"
    );
    assert_eq!(interpreted[7], bf16_bits::NEGATIVE_INFINITY);
    // `(1 + 2^-7) * 3` is exactly `193.5` quanta of the grid at that exponent,
    // so the tie rule decides it: half-to-even rounds up to `194`, which is
    // `3.03125`. A machine rounding half-away-from-zero agrees here, and one
    // rounding half-to-odd does not — this is the witness that makes the rule
    // observable rather than assumed.
    assert_eq!(interpreted[8], 0x4042);
    assert_eq!(interpreted[9], bf16_bits::THREE);
}

/// Dropping the BF16 canonicalization fails on exactly the NaN element.
///
/// The perturbation the CPU vertical uses, applied at this width: the same
/// kernel is interpreted by a machine modelling a lowering that emitted the BF16
/// arithmetic without its `CanonicalizeBf16Nan` beside it, and the reference
/// comparison is watched failing. What makes it evidence rather than a red test
/// is *where* it fails — the one element whose arithmetic result is a NaN, and
/// no other — so the conversion's obligation is the reason rather than a
/// coincidence.
#[test]
fn deleting_the_bf16_canonicalization_disagrees_at_exactly_the_nan_element() {
    let elements = u64::try_from(BF16_WITNESSES.len()).unwrap();
    let scheduled = bf16_scheduled_region(elements);
    let kernel = tiler_ir::kernel::lower_scheduled_region(&scheduled).unwrap();

    let canonical = interpret_bf16(&kernel, &BF16_WITNESSES, Bf16Canonicalization::Applied);
    let perturbed = interpret_bf16(&kernel, &BF16_WITNESSES, Bf16Canonicalization::Omitted);

    let disagreements: Vec<usize> = canonical
        .iter()
        .zip(&perturbed)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect();
    assert_eq!(
        disagreements,
        vec![3],
        "only the non-canonical NaN witness depends on the canonicalization"
    );
    assert_eq!(canonical[3], bf16_bits::CANONICAL_NAN);
    assert_eq!(
        perturbed[3],
        bf16_bits::NONCANONICAL_NAN,
        "without the conversion the operand's own payload reaches the boundary"
    );
}
