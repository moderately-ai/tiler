//! Where the elementwise-epilogue wall actually is, measured rather than asserted.
//!
//! `admit-elementwise-epilogues-over-a-materialized-intermediate` was filed on a
//! premise this file refutes. Its "Why this exists" section stated the wall as
//! *the physical layer's, not the schedule IR's* — "`TensorRole::Intermediate`
//! is a per-region role, so nothing in `tiler-ir` forbids a chain that stages a
//! second temporary" — and `request.rs` carried the same claim in
//! [`select_supported_strategy`]'s own documentation. The claim is about the
//! role, and the role is indeed per-region; the conclusion drawn from it is
//! wrong, because what forbids the chain is not the role but the *access
//! contract* each scalar-program family declares around it.
//!
//! # The three walls, and which of them is open
//!
//! A chain `producer -> materialized intermediate -> elementwise epilogue` needs
//! a producer region that writes `TensorRole::Intermediate` and a pointwise
//! region that reads one. Three families are involved and they do not agree:
//!
//! | Region | Needs | Admitted by `tiler-ir` |
//! | --- | --- | --- |
//! | elementwise epilogue | read `TensorRole::Intermediate` | **no** — [`a_pointwise_region_cannot_read_a_materialized_intermediate`] |
//! | serial-sum producer | write `TensorRole::Intermediate` | **no** — [`a_strict_serial_sum_region_cannot_write_a_materialized_intermediate`] |
//! | contraction producer | write `TensorRole::Intermediate` | yes — [`a_contraction_region_can_already_write_a_materialized_intermediate`] |
//!
//! The refusing halves live in `crates/tiler-ir/src/schedule/builder.rs`:
//! `verify_pointwise_region` requires read access `i` to be
//! `TensorRole::Input { ordinal: i }` for every `i`, and
//! `verify_access_and_semantics` admits a `ScalarProgram::StrictSerialSum` under
//! a `ReductionTopology::Serial` only when `write.tensor == TensorRole::Output`.
//! Neither is reachable from `tiler-compiler`, so the epilogue admission is a
//! `tiler-ir` widening with a compiler-side dependent — exactly the shape
//! `admit-multi-input-elementwise-programs-at-the-compiler-boundary` hit, and
//! `admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`
//! now owns it.
//!
//! **The compiler cannot route around it by binding differently, either.** A
//! region could not declare `TensorRole::Input { ordinal }` for the read and let
//! program assembly bind a temporary there: `tiler_ir::program::ValueRole::fills`
//! refuses a `Temporary` value for an `Input` buffer, and
//! `KernelProgramBuilder::push_stage` is where that bites. That mechanism is
//! already pinned by `multi_output_boundary.rs`'s
//! `a_published_output_value_cannot_fill_an_intermediate_buffer`, so it is cited
//! here rather than re-asserted.
//!
//! # What the caller sees today
//!
//! Both epilogue shapes the ticket names refuse at the request boundary under
//! `operation-set`, which is the elementwise walk reporting that the operand it
//! reached is produced by a family its expression vocabulary has no node for.
//! Each travels with the bare producer as a control, so a refusal here is
//! evidence about the epilogue rather than about the profile.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContractionAxisSource,
    ContributorOrder, ExceptionalValueAssumption, ExecutionBinding, IndexRegion, InputOrdinal,
    KernelSchedule, LaunchPlan, LogicalAccess, NumericalPermission, NumericalRealization,
    OwnershipProof, OwnershipProofKind, OwnershipWitnessId, PointwiseF32Expression,
    PointwiseF32ExpressionBuilder, ReductionTopology, RegionId, ScalarProgram, ScheduledRegion,
    ScheduledRegionBuilder, ScheduledRegionDiagnostic, SubnormalMode, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    CANONICAL_F32_ARITHMETIC_NAN_BITS, ContractionIndex, ContractionIndexStructure, F32,
    F32Constant, F32Multiply, F32TensorContraction, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled, for the reason the sibling boundary
/// files state it: recognition is structural, so a contract that changed the
/// outcome would mean the boundary moved for a reason this file does not model.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

// ---------------------------------------------------------------------------
// What the caller sees: both epilogue shapes refuse at the request boundary
// ---------------------------------------------------------------------------

/// The contraction structure `mk,nk->mn`, the one `contraction_direct_path` uses.
fn projection_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .unwrap()
}

/// `projected = contract(a, b)`, the accepted control for the contraction pair.
///
/// The extents keep the output at four elements, which is the governed baseline
/// profile's grid-axis bound — the same bound every fixture in
/// `contraction_direct_path.rs` respects.
fn bare_contraction() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), a, b).unwrap();
    builder
        .output(OutputKey::new("projected").unwrap(), projected)
        .unwrap();
    builder.build().unwrap()
}

/// `scaled = contract(a, b) * 2.0` — the ticket's first named shape.
///
/// Differs from [`bare_contraction`] by exactly the epilogue: same inputs, same
/// structure, same contraction occurrence, one further multiply on the result.
fn contraction_with_epilogue() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), a, b).unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, projected, two).unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    builder.build().unwrap()
}

/// The contributor domain of the reduction pair: one row of four.
///
/// Four contributors is what the sibling `composed_family_recognition.rs` uses
/// and for its reason — the prologue's one-invocation-per-contributor launch
/// must stay inside the governed profile's four-thread grid axis.
fn reduction_domain() -> Shape {
    Shape::from_dims([1, 4])
}

/// `reduced = sum(x * x, axis 1)`, the accepted control for the reduction pair.
fn bare_reduction() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let x = builder
        .input::<F32>(InputKey::new("x").unwrap(), reduction_domain())
        .unwrap();
    let squared = F32Multiply::apply(&mut builder, x, x).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, squared, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("reduced").unwrap(), reduced)
        .unwrap();
    builder.build().unwrap()
}

/// `scaled = sum(x * x, axis 1) * 2.0` — the ticket's second named shape.
///
/// Differs from [`bare_reduction`] by exactly the epilogue. The fold reduces
/// axis 1 of a rank-two domain rather than every axis, so the epilogue's own
/// domain is rank one and the refusal below is the walk's rather than
/// `recognize_pointwise`'s `elementwise-rank` guard.
fn reduction_with_epilogue() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let x = builder
        .input::<F32>(InputKey::new("x").unwrap(), reduction_domain())
        .unwrap();
    let squared = F32Multiply::apply(&mut builder, x, x).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, squared, [Axis::new(1)]).unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, reduced, two).unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    builder.build().unwrap()
}

/// Compiles one program under one contract against the governed profile.
fn compile_under(
    program: &SemanticProgram,
    contract: NumericalContract,
) -> Result<(), CompileFailureClass> {
    let targets = TargetRequest::new([TargetProfile::governed()]).unwrap();
    match compile(CompileRequest::new(program, contract, targets)) {
        Ok(batch) => {
            let outcome = batch.targets().next().expect("one requested profile");
            outcome
                .outcome()
                .map(|_| ())
                .map_err(TargetCompileFailure::class)
        }
        Err(failure) => Err(failure.class()),
    }
}

/// An elementwise epilogue over a contraction refuses, and its producer compiles.
///
/// The refusal is the elementwise walk's `operation-set`: the walk reaches the
/// contraction occurrence as an operand, and no `PointwiseF32Node` spells a sum
/// over indices shared by two operands, so there is no leaf to mint for it. The
/// bare contraction travels with it under the identical request, so a green run
/// is evidence about the epilogue rather than about the profile or the target.
#[test]
fn an_elementwise_epilogue_over_a_contraction_refuses_at_the_request_boundary() {
    let control = bare_contraction();
    let epilogue = contraction_with_epilogue();
    // One further occurrence than the control, and nothing else moved.
    assert_eq!(
        epilogue.operation_count(),
        control.operation_count() + 2,
        "the epilogue adds exactly the constant and the multiply",
    );

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&control, contract),
            Ok(()),
            "{contract:?} refused the bare contraction, so nothing this test \
             asserts about the epilogue would be evidence about the epilogue",
        );
        assert_eq!(
            compile_under(&epilogue, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "operation-set"
            }),
            "{contract:?} did not refuse the contraction epilogue under the \
             elementwise walk's own rule",
        );
    }
}

/// An elementwise epilogue over a reduction refuses, and its producer compiles.
#[test]
fn an_elementwise_epilogue_over_a_reduction_refuses_at_the_request_boundary() {
    let control = bare_reduction();
    let epilogue = reduction_with_epilogue();
    assert_eq!(
        epilogue.operation_count(),
        control.operation_count() + 2,
        "the epilogue adds exactly the constant and the multiply",
    );

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&control, contract),
            Ok(()),
            "{contract:?} refused the bare reduction, so nothing this test \
             asserts about the epilogue would be evidence about the epilogue",
        );
        assert_eq!(
            compile_under(&epilogue, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "operation-set"
            }),
            "{contract:?} did not refuse the reduction epilogue under the \
             elementwise walk's own rule",
        );
    }
}

// ---------------------------------------------------------------------------
// Where the wall is: the schedule vocabulary's own access contracts
// ---------------------------------------------------------------------------

/// The strict `f32` realization every hand-built region below declares.
///
/// The profile key is a literal rather than a minted
/// [`tiler_ir::schedule::F32NumericalContractKey`] because no obligation under
/// test reads it: the intrinsic verifier compares the *permissions* against each
/// region's declared topology and never parses the key. Every region below —
/// refused and admitted alike — carries this same value, so no outcome here can
/// be attributed to the numerical declaration.
fn strict_f32_realization() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        CANONICAL_F32_ARITHMETIC_NAN_BITS,
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

/// The one-invocation-per-element schedule every region below is launched with.
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

/// `input(0) * 2.0`: the smallest expression an epilogue region could carry.
fn scale_expression() -> PointwiseF32Expression {
    let mut builder = PointwiseF32ExpressionBuilder::new();
    let leaf = builder.input(InputOrdinal::new(0)).unwrap();
    let two = builder.constant(2.0_f32.to_bits()).unwrap();
    let root = builder.multiply(leaf, two).unwrap();
    builder.build(root).unwrap()
}

/// A one-read elementwise region over `shape`, reading whichever tensor is named.
///
/// The *only* thing that varies between the refused region and its control is
/// `read`, and it varies in the access, the bounds proof, and nowhere else — so
/// the verifier's verdict is attributable to the read's boundary role alone.
fn elementwise_region(read: TensorRole, elements: u64) -> ScheduledRegion {
    ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: Shape::from_dims([elements]),
            accesses: vec![
                Access {
                    tensor: read,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(1),
                    ownership: Some(OwnershipWitnessId::new(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(0),
                    tensor: read,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: elements,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: TensorRole::Output,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: elements,
                },
            },
            scalar_program: ScalarProgram::PointwiseF32(scale_expression()),
            numerical: strict_f32_realization(),
        },
        schedule: linear_schedule(elements, OwnershipWitnessId::new(0)),
    }
}

/// Verifies one hand-built region through the shared intrinsic verifier.
fn verify(region: ScheduledRegion) -> Result<(), Vec<ScheduledRegionDiagnostic>> {
    ScheduledRegionBuilder::from_region(region)
        .build()
        .map(|_| ())
        .map_err(|error| error.diagnostics().to_vec())
}

/// **The falsification.** A pointwise region may not read a materialized
/// intermediate, so the epilogue this ticket owns has no region to be built as.
///
/// `verify_pointwise_region` requires read access `i` to be
/// `TensorRole::Input { ordinal: i }` at every position — both halves of that
/// correspondence are load-bearing there, and the role half is what refuses
/// here. The control is the identical region with the read at ordinal zero, so
/// the verdict cannot be attributed to the expression, the domain, the proofs,
/// the ownership, the schedule, or the numerical declaration.
///
/// This is what makes the epilogue admission a `crates/tiler-ir/**` widening
/// with a `crates/tiler-compiler/**` dependent rather than a compiler-side gap,
/// and it is why
/// `admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`
/// exists. If that widening lands, this test fails and says so.
#[test]
fn a_pointwise_region_cannot_read_a_materialized_intermediate() {
    let control = elementwise_region(
        TensorRole::Input {
            ordinal: InputOrdinal::new(0),
        },
        4,
    );
    assert_eq!(
        verify(control),
        Ok(()),
        "the input-reading control must verify, or the refusal below is not \
         about the read's boundary role",
    );

    let epilogue = elementwise_region(TensorRole::Intermediate, 4);
    let diagnostics = verify(epilogue).expect_err(
        "a pointwise region reading a materialized intermediate must be refused \
         by the intrinsic schedule verifier",
    );
    assert!(
        diagnostics.contains(&ScheduledRegionDiagnostic::NumericalOrAccessRefinement),
        "expected the access-refinement diagnostic, got {diagnostics:?}",
    );
}

/// A strict serial fold may not write a materialized intermediate.
///
/// The second wall, and independent of the first: even if a pointwise region
/// could read an intermediate, `sum(x * x) * scale` would still have no chain,
/// because `verify_access_and_semantics` admits a `ScalarProgram::StrictSerialSum`
/// under a `ReductionTopology::Serial` only when the owning write targets
/// `TensorRole::Output`. The multi-pass partial arm is the one place a fold
/// writes an intermediate, and it is a different topology declaring a split.
#[test]
fn a_strict_serial_sum_region_cannot_write_a_materialized_intermediate() {
    let input_shape = Shape::from_dims([1, 4]);
    let axes = vec![Axis::new(1)];
    let output_shape = input_shape.without_axes(&axes);
    let outputs = 1_u64;

    let fold = |write: TensorRole| ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(1),
            iteration_shape: output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: input_shape.clone(),
                        output_shape: output_shape.clone(),
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: write,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(1),
                    ownership: Some(OwnershipWitnessId::new(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(0),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: input_shape.clone(),
                        output_shape: output_shape.clone(),
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: write,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: outputs,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: write,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: outputs,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_F32_ARITHMETIC_NAN_BITS,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: strict_f32_realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(outputs, OwnershipWitnessId::new(0))
        },
    };

    assert_eq!(
        verify(fold(TensorRole::Output)),
        Ok(()),
        "the output-writing control must verify, or the refusal below is not \
         about the write's boundary role",
    );
    let diagnostics = verify(fold(TensorRole::Intermediate)).expect_err(
        "a strict serial sum writing a materialized intermediate must be \
         refused by the intrinsic schedule verifier",
    );
    assert!(
        diagnostics.contains(&ScheduledRegionDiagnostic::NumericalOrAccessRefinement),
        "expected the access-refinement diagnostic, got {diagnostics:?}",
    );
}

/// A contraction may already write a materialized intermediate, and that bounds
/// the widening.
///
/// The producer half of `contract(a, b) * 2.0` needs no `tiler-ir` change at
/// all: `verify_contraction` admits `TensorRole::Intermediate | TensorRole::Output`
/// for the owning write. `crate::physical::contraction_region` hard-codes
/// `TensorRole::Output`, which is a compiler-side choice this ticket can move
/// once the consumer half exists.
///
/// Recorded so the blocking ticket is scoped to the two walls that are real
/// rather than to three, and so a reader does not re-derive which of the
/// producer families already compose.
#[test]
fn a_contraction_region_can_already_write_a_materialized_intermediate() {
    let operand = Shape::from_dims([2, 2]);
    let output_shape = Shape::from_dims([2, 2]);
    let contracted_shape = Shape::from_dims([2]);
    let operand_elements = 4_u64;
    let outputs = 4_u64;

    let contraction = |write: TensorRole| ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Input {
                        ordinal: InputOrdinal::new(0),
                    },
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ContractionOperand {
                        operand_shape: operand.clone(),
                        output_shape: output_shape.clone(),
                        contracted_shape: contracted_shape.clone(),
                        sources: vec![
                            ContractionAxisSource::Output { position: 0 },
                            ContractionAxisSource::Contracted { position: 0 },
                        ],
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Input {
                        ordinal: InputOrdinal::new(1),
                    },
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ContractionOperand {
                        operand_shape: operand.clone(),
                        output_shape: output_shape.clone(),
                        contracted_shape: contracted_shape.clone(),
                        sources: vec![
                            ContractionAxisSource::Output { position: 1 },
                            ContractionAxisSource::Contracted { position: 0 },
                        ],
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(1),
                    ownership: None,
                },
                Access {
                    tensor: write,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(2),
                    ownership: Some(OwnershipWitnessId::new(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(0),
                    tensor: TensorRole::Input {
                        ordinal: InputOrdinal::new(0),
                    },
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: operand_elements,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: TensorRole::Input {
                        ordinal: InputOrdinal::new(1),
                    },
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: operand_elements,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(2),
                    tensor: write,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: outputs,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: write,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: outputs,
                },
            },
            scalar_program: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_F32_ARITHMETIC_NAN_BITS,
            },
            numerical: strict_f32_realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Contraction {
                contracted_shape: contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(outputs, OwnershipWitnessId::new(0))
        },
    };

    // Both roles verify, which is the whole point: the producer half of the
    // chain is already expressible and only the consumer half is not.
    assert_eq!(verify(contraction(TensorRole::Output)), Ok(()));
    assert_eq!(verify(contraction(TensorRole::Intermediate)), Ok(()));
}
