//! Bounded tests for the target-neutral kernel-program IR.
//!
//! Fixtures bind real verified structured kernels to real verified semantic
//! programs. Coverage assignments are structural partitions: this layer proves
//! that every operation of the bound graph is covered exactly once, never that
//! a given kernel computes the operations its stage claims.

use crate::index::{
    FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexDomainProofBudget,
    IndexRealizationAuthority, IndexRefinementSubject, IndexRefinementVerificationOutcome,
    MAX_FINITE_DOMAIN_PROOF_CELLS, NumericalContractIdentity, ResolvedIndexRealization,
};
use crate::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use crate::schedule::{
    Access, AccessMode, ApproximationEnvelope, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContributorOrder, ExceptionalValueAssumption, ExecutionBinding, F32NumericalContractKey,
    FlushedZeroSign, InputOrdinal, KernelSchedule, LaunchPlan, LogicalAccess,
    MaterializationRounding, NumericalPermission, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseF32ExpressionBuilder, ReductionTopology,
    RegionId, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
    VerifiedScheduledRegion,
};
use crate::semantic::{
    EncodedComponentRole, F32, F32Add, F32Constant, F32Multiply, InputKey, OperationAttributes,
    OutputKey, STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
    SemanticProgram, SemanticProgramBuilder, StrictAffineU4, StrictSerialF32Sum,
    dequantize_strict_affine_op,
};
use crate::shape::{Axis, Shape};

use super::abi::{AbiBinaryOp, AbiRoot, AbiType, AbiUnaryOp, AvailabilityPhase, TargetPropertyKey};
use super::{
    AbiExprId, AllocationId, AllocationOwnership, AllocationSpec, ByteWindow, CoveredOccurrence,
    KernelProgramBuildError, KernelProgramBuilder, KernelProgramDiagnostic,
    MaterializedComponentSpec, MaterializedOrigin, MaterializedValueId, MaterializedValueSpec,
    MemorySpace, PartialReduction, ProgramAbiUse, ProgramEntityKind, RoutingCommitState,
    RoutingCommitTransition, SemanticOccurrence, StageAccess, StageAccessMode, StageId,
    StageLaunch, StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram, ViewId,
};

const SCALE_BITS: u32 = 0x4000_0000; // 2.0f32
const OTHER_SCALE_BITS: u32 = 0x4040_0000; // 3.0f32
const BIAS_BITS: u32 = 0x3f80_0000; // 1.0f32
const CANONICAL_NAN: u32 = 0x7fc0_0000;

fn strict() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        CANONICAL_NAN,
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

fn elements(shape: &Shape) -> u64 {
    crate::schedule::element_count(shape).expect("test shapes do not overflow")
}

fn input_shape() -> Shape {
    Shape::from_dims([2, 3])
}

fn output_shape() -> Shape {
    Shape::from_dims([2])
}

fn scale_bias_expression(scale_bits: u32) -> crate::schedule::PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(InputOrdinal::FIRST).expect("input");
    let scale = expression.constant(scale_bits).expect("scale");
    let product = expression.multiply(input, scale).expect("product");
    let bias = expression.constant(BIAS_BITS).expect("bias");
    let root = expression.add(product, bias).expect("sum");
    expression.build(root).expect("pointwise expression")
}

/// Builds the canonical pointwise region: one program input to one temporary.
fn pointwise_region(region: u32, scale_bits: u32) -> VerifiedScheduledRegion {
    let shape = input_shape();
    let count = elements(&shape);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(region));
    builder.iteration_shape(shape).expect("iteration shape");
    builder
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("read access");
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("write access");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: count,
            },
        })
        .expect("read proof");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: count,
            },
        })
        .expect("write proof");
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: count,
            },
        })
        .expect("ownership proof");
    builder
        .scalar_program(ScalarProgram::PointwiseF32(scale_bias_expression(
            scale_bits,
        )))
        .expect("scalar program");
    builder.numerical(strict()).expect("numerical realization");
    builder
        .schedule(linear_schedule(count, OwnershipWitnessId::new(0)))
        .expect("schedule");
    builder.build().expect("verified pointwise region")
}

/// Builds the canonical reduction region: one temporary to one program output.
fn reduction_region(region: u32) -> VerifiedScheduledRegion {
    let axes = vec![Axis::new(1)];
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(region));
    builder.iteration_shape(output_shape()).expect("shape");
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("read access");
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("write access");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .expect("read proof");
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: elements(&output_shape()),
            },
        })
        .expect("write proof");
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements(&output_shape()),
            },
        })
        .expect("ownership proof");
    builder
        .scalar_program(ScalarProgram::StrictSerialSum {
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: CANONICAL_NAN,
            empty_identity_bits: 0,
        })
        .expect("scalar program");
    builder.numerical(strict()).expect("numerical realization");
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(elements(&output_shape()), OwnershipWitnessId::new(0))
        })
        .expect("schedule");
    builder.build().expect("verified reduction region")
}

fn pointwise_kernel(region: u32, scale_bits: u32) -> VerifiedKernel {
    lower_scheduled_region(&pointwise_region(region, scale_bits)).expect("pointwise kernel")
}

fn reduction_kernel(region: u32) -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(region)).expect("reduction kernel")
}

/// A five-operation graph: `result = strict_serial_sum(input * scale + 1.0, 1)`.
fn serial_sum_program(scale_bits: u32) -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let input = draft
        .input::<F32>(InputKey::new("input").expect("key"), input_shape())
        .expect("input");
    let scale = F32Constant::apply(&mut draft, scale_bits).expect("scale");
    let bias = F32Constant::apply(&mut draft, BIAS_BITS).expect("bias");
    let product = F32Multiply::apply(&mut draft, input, scale).expect("product");
    let mapped = F32Add::apply(&mut draft, product, bias).expect("mapped");
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)]).expect("sum");
    draft
        .output(OutputKey::new("result").expect("key"), sum)
        .expect("output");
    let program = draft.build().expect("verified semantic program");
    assert_eq!(program.operation_count(), 5);
    program
}

/// An eight-operation graph with two independent chains and two named outputs.
fn two_chain_program() -> SemanticProgram {
    two_chain_program_keyed(["sum_a", "sum_b"])
}

/// The two-chain graph publishing its two reductions under the given keys.
///
/// The keys are a parameter because the interface order and the order the
/// superseded sorted encoding produced coincide for `sum_a`/`sum_b` and differ
/// for a reverse-lexicographic pair, and an ordering rule can only be told from
/// a content sort by a program where the two disagree.
fn two_chain_program_keyed(keys: [&str; 2]) -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let first = draft
        .input::<F32>(InputKey::new("a").expect("key"), input_shape())
        .expect("first input");
    let second = draft
        .input::<F32>(InputKey::new("b").expect("key"), input_shape())
        .expect("second input");
    let scale = F32Constant::apply(&mut draft, SCALE_BITS).expect("scale");
    let bias = F32Constant::apply(&mut draft, BIAS_BITS).expect("bias");
    let first_product = F32Multiply::apply(&mut draft, first, scale).expect("first product");
    let first_mapped = F32Add::apply(&mut draft, first_product, bias).expect("first mapped");
    let second_product = F32Multiply::apply(&mut draft, second, scale).expect("second product");
    let second_mapped = F32Add::apply(&mut draft, second_product, bias).expect("second mapped");
    let first_sum =
        StrictSerialF32Sum::apply(&mut draft, first_mapped, [Axis::new(1)]).expect("first sum");
    let second_sum =
        StrictSerialF32Sum::apply(&mut draft, second_mapped, [Axis::new(1)]).expect("second sum");
    draft
        .output(OutputKey::new(keys[0]).expect("key"), first_sum)
        .expect("first output");
    draft
        .output(OutputKey::new(keys[1]).expect("key"), second_sum)
        .expect("second output");
    let program = draft.build().expect("verified semantic program");
    assert_eq!(program.operation_count(), 8);
    program
}

fn strict_affine_u4_passthrough_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let input = draft
        .input::<StrictAffineU4>(InputKey::new("input").expect("key"), Shape::from_dims([5]))
        .expect("encoded input");
    draft
        .output(OutputKey::new("result").expect("key"), input)
        .expect("encoded output");
    draft.build().expect("verified semantic program")
}

fn strict_affine_u4_dequantize_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let input = draft
        .input_resolved(
            InputKey::new("input").expect("key"),
            Shape::from_dims([5]),
            StrictAffineU4::resolved_type(),
        )
        .expect("encoded input");
    let output = draft
        .apply(
            dequantize_strict_affine_op(),
            OperationAttributes::empty(),
            &[input],
        )
        .expect("strict affine dequantization")[0];
    draft
        .output_resolved(OutputKey::new("result").expect("key"), output)
        .expect("dense output");
    draft.build().expect("verified semantic program")
}

fn strict_affine_u4_dequantize_kernel() -> VerifiedKernel {
    let logical_elements = 5;
    let owner = OwnershipWitnessId::new(0);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(17));
    builder
        .iteration_shape(Shape::from_dims([logical_elements]))
        .expect("iteration shape");
    for access in [
        Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: Some(STRICT_AFFINE_CODES_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::PackedU4LsbZeroTail { logical_elements },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: Some(STRICT_AFFINE_SCALE_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::ScalarBroadcast,
            bounds: BoundsWitnessId::new(1),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: Some(STRICT_AFFINE_ZERO_POINT_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::ScalarBroadcast,
            bounds: BoundsWitnessId::new(2),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(3),
            ownership: Some(owner),
        },
    ] {
        builder.push_access(access).expect("access");
    }
    for (id, tensor, component_role, element_count) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            Some(STRICT_AFFINE_CODES_ROLE),
            logical_elements.div_ceil(2),
        ),
        (
            1,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            Some(STRICT_AFFINE_SCALE_ROLE),
            1,
        ),
        (
            2,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            Some(STRICT_AFFINE_ZERO_POINT_ROLE),
            1,
        ),
        (3, TensorRole::Output, None, logical_elements),
    ] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(id),
                tensor,
                component_role,
                kind: BoundsProofKind::LinearRange { element_count },
            })
            .expect("bounds proof");
    }
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: logical_elements,
            },
        })
        .expect("ownership proof");
    builder
        .scalar_program(ScalarProgram::StrictAffineU4Dequantize {
            codes_role: STRICT_AFFINE_CODES_ROLE,
            scale_role: STRICT_AFFINE_SCALE_ROLE,
            zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
        })
        .expect("scalar program");
    builder.numerical(strict()).expect("numerical contract");
    builder
        .schedule(linear_schedule(logical_elements, owner))
        .expect("schedule");
    lower_scheduled_region(&builder.build().expect("verified schedule"))
        .expect("verified structured kernel")
}

/// The governed strict F32 contract every fixture kernel realizes.
fn strict_contract() -> NumericalContractIdentity {
    f32_contract(SubnormalMode::Preserve)
}

/// The same contract flushing subnormals, used only to perturb *evidence*.
///
/// A numerical contract is folded into a refinement receipt's executable
/// coverage and is not part of semantic graph meaning, so two coverages minted
/// under these two contracts name the same occurrences of the same graph and
/// carry different proofs. That is the exact perturbation the identity tests
/// need, and there is no honest way to fabricate it.
fn flush_contract() -> NumericalContractIdentity {
    f32_contract(SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    })
}

fn f32_contract(subnormals: SubnormalMode) -> NumericalContractIdentity {
    F32NumericalContractKey::new(
        subnormals,
        subnormals,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
        MaterializationRounding::NearestTiesToEven,
    )
    .expect("the fixture contract vector is coherent")
    .into()
}

/// Mints one real refinement receipt per operation of `semantic`.
///
/// Nothing here is a fixture shortcut: each record comes from the same sealed
/// path a lowering consumer walks — derive the subject, resolve the registered
/// realization law, realize that law's exact canonical region, admit an
/// authority whose scalar-emission ceiling is the region's own reached set, and
/// submit the pair to the verifier. Only the verifier mints the receipt, so a
/// coverage record this suite hands to `push_stage` is evidence rather than an
/// assertion, and the identity properties tested below are properties of real
/// evidence bytes.
///
/// The result is indexed by canonical occurrence ordinal, which is what the
/// coverage partitions below select ranges of.
fn checked_coverage(
    semantic: &SemanticProgram,
    contract: &NumericalContractIdentity,
) -> Vec<CoveredOccurrence> {
    let registry = semantic.semantic_registry().clone();
    let scalars = FrozenScalarRegistry::standard().expect("the standard scalar authority freezes");
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(registry.clone(), scalars.clone())
        .expect("the standard scalar authority coheres with its semantic authority");
    let mut coverage: Vec<CoveredOccurrence> = semantic
        .operations()
        .map(|operation| {
            let subject =
                IndexRefinementSubject::derive(semantic, operation.id(), contract.clone())
                    .expect("every fixture operation derives a refinement subject");
            let law = registry
                .index_realization_law(subject.operation())
                .expect("every fixture operation has a registered realization law")
                .law
                .clone();
            let region = law
                .realize(&subject, &scalars)
                .expect("the registered law realizes its own subject");
            let reached = scalars
                .revalidate_region(&region)
                .expect("the law's own region revalidates against the scalar authority");
            let authority = IndexRealizationAuthority::admit(
                &registry,
                &scalars,
                subject.operation().clone(),
                subject.signature().clone(),
                reached.reached_operations(),
            )
            .expect("the region's reached scalar operations are an admissible ceiling");
            let resolution = laws
                .resolve(&subject)
                .expect("the registered law resolves for its own subject");
            let receipt = match resolution
                .verify(&authority, &region)
                .expect("the law's own region satisfies the law")
            {
                IndexRefinementVerificationOutcome::Verified(receipt) => *receipt,
                IndexRefinementVerificationOutcome::Pending(pending) => {
                    ResolvedIndexRealization::complete(&pending, proof_budget())
                        .expect("the fixture's residual index-domain obligations are provable")
                        .0
                }
            };
            CoveredOccurrence::from_receipt(&receipt)
        })
        .collect();
    coverage.sort_unstable_by_key(CoveredOccurrence::occurrence);
    coverage
}

fn proof_budget() -> IndexDomainProofBudget {
    IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 1 << 20)
        .expect("the fixture proof budget is within IR's hard bounds")
}

/// Selects the coverage records for one canonical occurrence range.
fn occurrences(semantic: &SemanticProgram, range: std::ops::Range<u32>) -> Vec<CoveredOccurrence> {
    coverage_range(&checked_coverage(semantic, &strict_contract()), range)
}

fn coverage_range(
    coverage: &[CoveredOccurrence],
    range: std::ops::Range<u32>,
) -> Vec<CoveredOccurrence> {
    coverage
        .iter()
        .filter(|covered| range.contains(&covered.occurrence().get()))
        .cloned()
        .collect()
}

fn device(capacity_bytes: u64, ownership: AllocationOwnership) -> AllocationSpec {
    AllocationSpec {
        capacity_bytes,
        alignment: 4,
        memory_space: MemorySpace::Device,
        ownership,
    }
}

fn value(origin: MaterializedOrigin, role: ValueRole, shape: Shape) -> MaterializedValueSpec {
    MaterializedValueSpec {
        origin,
        role,
        shape,
        storage_scalar: StorageScalar::F32,
        encoding: StorageEncoding::Unpacked,
        element_type: KernelType::F32,
        alignment: 4,
        memory_space: MemorySpace::Device,
    }
}

fn program_input(key: &str) -> MaterializedOrigin {
    MaterializedOrigin::ProgramInput {
        key: InputKey::new(key).expect("input key"),
    }
}

fn read(view: ViewId, accessible_bytes: AbiExprId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Read,
        accessible_bytes,
    }
}

fn write(view: ViewId, accessible_bytes: AbiExprId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Write,
        accessible_bytes,
    }
}

/// The bounded profile's shapes are static, so every ABI quantity a fixture
/// needs is a literal. A dynamic subject would name an input extent instead.
fn literal(builder: &mut KernelProgramBuilder, value: u64) -> AbiExprId {
    builder
        .push_abi_root(AbiRoot::UnsignedLiteral(value))
        .expect("abi literal")
}

/// The ABI quantities every fixture in this file shares.
///
/// The arena deduplicates by content, so minting these once per builder and
/// once per fixture produce the same arena.
#[derive(Clone, Copy, Debug)]
struct FixtureAbi {
    /// Byte count of a whole `[2, 3]` `f32` value.
    input_bytes: AbiExprId,
    /// Byte count of a whole `[2]` `f32` value.
    output_bytes: AbiExprId,
    /// Launch extent of a stage iterating the `[2, 3]` shape.
    pointwise_threads: AbiExprId,
    /// Launch extent of a stage iterating the `[2]` shape.
    reduction_threads: AbiExprId,
    /// Workgroup width every fixture kernel requires.
    threads_per_workgroup: AbiExprId,
}

/// Mints the shared ABI quantities every fixture stage names.
fn fixture_abi(builder: &mut KernelProgramBuilder) -> FixtureAbi {
    FixtureAbi {
        input_bytes: literal(builder, 24),
        output_bytes: literal(builder, 8),
        pointwise_threads: literal(builder, 6),
        reduction_threads: literal(builder, 2),
        threads_per_workgroup: literal(builder, 1),
    }
}

/// Mints the same ABI quantities, writing each byte count as a product.
///
/// Every value equals [`fixture_abi`]'s, so a program built with this differs
/// from the canonical one in the *form* of its ABI and in nothing else. That is
/// what makes it a usable probe of whether identity folds the expression rather
/// than only the number it happens to evaluate to.
fn computed_fixture_abi(builder: &mut KernelProgramBuilder) -> FixtureAbi {
    let element_bytes = literal(builder, 4);
    let pointwise_threads = literal(builder, 6);
    let reduction_threads = literal(builder, 2);
    let product = |builder: &mut KernelProgramBuilder, elements: AbiExprId| {
        builder
            .push_abi_binary(AbiBinaryOp::CheckedMultiply, element_bytes, elements)
            .expect("checked byte product")
    };
    FixtureAbi {
        input_bytes: product(builder, pointwise_threads),
        output_bytes: product(builder, reduction_threads),
        pointwise_threads,
        reduction_threads,
        threads_per_workgroup: literal(builder, 1),
    }
}

/// Declares the always-true applicability guard.
fn declare_guard(builder: &mut KernelProgramBuilder) {
    let guard = builder
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("guard predicate");
    builder
        .applicability_guard(guard)
        .expect("applicability guard");
}

/// Declares the whole program contract a verified fixture must state.
fn declare_program_contract(builder: &mut KernelProgramBuilder) {
    declare_guard(builder);
    declare_routing_commit(builder);
}

impl FixtureAbi {
    fn pointwise_launch(self) -> StageLaunch {
        StageLaunch {
            grid_threads: self.pointwise_threads,
            threads_per_workgroup: self.threads_per_workgroup,
        }
    }

    fn reduction_launch(self) -> StageLaunch {
        StageLaunch {
            grid_threads: self.reduction_threads,
            threads_per_workgroup: self.threads_per_workgroup,
        }
    }
}

/// The one lifecycle every verified program must span, with fallback admitted
/// exactly while nothing is committed.
const ROUTING_COMMIT_LIFECYCLE: [(RoutingCommitState, RoutingCommitState, bool); 3] = [
    (
        RoutingCommitState::Preflight,
        RoutingCommitState::Committed,
        true,
    ),
    (
        RoutingCommitState::Committed,
        RoutingCommitState::Executing,
        false,
    ),
    (
        RoutingCommitState::Executing,
        RoutingCommitState::Published,
        false,
    ),
];

fn declare_routing_commit(builder: &mut KernelProgramBuilder) {
    declare_routing_commit_with_fallback(builder, true);
}

/// Declares the whole lifecycle, choosing whether pre-commit fallback is
/// permitted. Every later step forbids it, which is the rule the builder proves.
fn declare_routing_commit_with_fallback(
    builder: &mut KernelProgramBuilder,
    fallback_before_commit: bool,
) {
    for (from, to, fallback_permitted) in ROUTING_COMMIT_LIFECYCLE {
        builder
            .push_routing_commit_transition(RoutingCommitTransition {
                from,
                to,
                fallback_permitted: fallback_permitted && fallback_before_commit,
            })
            .expect("routing-commit transition");
    }
}

fn diagnostic(builder: KernelProgramBuilder) -> KernelProgramDiagnostic {
    let error = builder.build().expect_err("verification must fail");
    *error.diagnostics().first().expect("one diagnostic")
}

/// The wired materialized two-stage serial-sum program.
struct TwoStage {
    builder: KernelProgramBuilder,
    pointwise: StageId,
    reduction: StageId,
    source: MaterializedValueId,
    temporary: MaterializedValueId,
    output: MaterializedValueId,
    temporary_allocation: AllocationId,
    output_allocation: AllocationId,
    source_view: ViewId,
    temporary_view: ViewId,
    abi: FixtureAbi,
}

/// How one two-stage fixture deviates from the canonical program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TwoStageShape {
    /// The canonical complete program.
    Canonical,
    /// The temporary and the output share one program-owned allocation.
    SharedOutputStorage,
    /// The coverage partition assigns a different split to the two stages.
    ShiftedCoverage,
    /// The two stages leave one occurrence for a third stage to cover.
    ReservedCoverage,
    /// Stages are declared in reverse order and arenas are declared last-first.
    ReversedDeclaration,
    /// The identical accessible byte counts, written as products rather than
    /// literals.
    ComputedAccessibleBytes,
    /// The canonical structure over coverage proved under another contract.
    ///
    /// Every stage, kernel, value, view, allocation, and covered occurrence is
    /// the canonical shape's; only the refinement evidence differs, because the
    /// receipts were minted under a numerical contract the semantic graph does
    /// not carry.
    AlternateRefinementEvidence,
    /// The first stage claims every occurrence and the second claims none.
    ///
    /// This is the coverage shape of a split reduction: the pass that computes
    /// the reduction claims it, and the pass that only combines its partials
    /// claims nothing, because claiming it again would double-cover the graph.
    UncoveringSecondStage,
}

/// The allocations, values, and views of the two-stage fixture.
struct TwoStageStorage {
    temporary_allocation: AllocationId,
    output_allocation: AllocationId,
    source: MaterializedValueId,
    temporary: MaterializedValueId,
    output: MaterializedValueId,
    source_view: ViewId,
    temporary_view: ViewId,
    output_view: ViewId,
}

/// Declares the externally bound input, the temporary, and the program output.
fn wire_two_stage_storage(
    builder: &mut KernelProgramBuilder,
    shape: TwoStageShape,
) -> TwoStageStorage {
    // Slot 0 is the externally bound input, slot 1 the temporary, slot 2 the
    // output. The shared-storage fixture declares no separate output storage.
    let mut requested = vec![
        (0_usize, device(24, AllocationOwnership::External)),
        (1, device(24, AllocationOwnership::Program)),
    ];
    if shape != TwoStageShape::SharedOutputStorage {
        requested.push((2, device(8, AllocationOwnership::Program)));
    }
    if shape == TwoStageShape::ReversedDeclaration {
        requested.reverse();
    }
    let mut slots: [Option<AllocationId>; 3] = [None; 3];
    for (slot, spec) in requested {
        slots[slot] = Some(builder.push_allocation(spec).expect("allocation"));
    }
    let external = slots[0].expect("external allocation");
    let temporary_allocation = slots[1].expect("temporary allocation");
    let output_allocation = slots[2].unwrap_or(temporary_allocation);

    let source = builder
        .push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            external,
        )
        .expect("input value");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                input_shape(),
            ),
            temporary_allocation,
        )
        .expect("temporary value");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            output_allocation,
        )
        .expect("output value");
    TwoStageStorage {
        temporary_allocation,
        output_allocation,
        source,
        temporary,
        output,
        source_view: builder.push_whole_view(source).expect("input view"),
        temporary_view: builder.push_whole_view(temporary).expect("temporary view"),
        output_view: builder.push_whole_view(output).expect("output view"),
    }
}

/// Wires the two-stage program without declaring dependencies or named outputs.
fn wire_two_stage(
    semantic: &SemanticProgram,
    pointwise_kernel: &VerifiedKernel,
    reduction_kernel: &VerifiedKernel,
    shape: TwoStageShape,
) -> TwoStage {
    let contract = if shape == TwoStageShape::AlternateRefinementEvidence {
        flush_contract()
    } else {
        strict_contract()
    };
    let coverage = checked_coverage(semantic, &contract);
    let split = |range: std::ops::Range<u32>| coverage_range(&coverage, range);
    let (pointwise_coverage, reduction_coverage) = match shape {
        TwoStageShape::ShiftedCoverage => (split(0..3), split(3..5)),
        TwoStageShape::ReservedCoverage => (split(0..2), split(2..4)),
        TwoStageShape::UncoveringSecondStage => (split(0..5), Vec::new()),
        TwoStageShape::Canonical
        | TwoStageShape::SharedOutputStorage
        | TwoStageShape::ReversedDeclaration
        | TwoStageShape::AlternateRefinementEvidence
        | TwoStageShape::ComputedAccessibleBytes => (split(0..4), split(4..5)),
    };
    let mut builder = KernelProgramBuilder::new(semantic).expect("builder");
    let abi = if shape == TwoStageShape::ComputedAccessibleBytes {
        computed_fixture_abi(&mut builder)
    } else {
        fixture_abi(&mut builder)
    };
    let reversed = shape == TwoStageShape::ReversedDeclaration;
    let TwoStageStorage {
        temporary_allocation,
        output_allocation,
        source,
        temporary,
        output,
        source_view,
        temporary_view,
        output_view,
    } = wire_two_stage_storage(&mut builder, shape);

    let push_pointwise = |builder: &mut KernelProgramBuilder| {
        builder
            .push_stage(
                pointwise_kernel,
                &pointwise_coverage,
                &[
                    read(source_view, abi.input_bytes),
                    write(temporary_view, abi.input_bytes),
                ],
                abi.pointwise_launch(),
            )
            .expect("pointwise stage")
    };
    let push_reduction = |builder: &mut KernelProgramBuilder| {
        builder
            .push_stage(
                reduction_kernel,
                &reduction_coverage,
                &[
                    read(temporary_view, abi.input_bytes),
                    write(output_view, abi.output_bytes),
                ],
                abi.reduction_launch(),
            )
            .expect("reduction stage")
    };
    let (pointwise, reduction) = if reversed {
        let reduction = push_reduction(&mut builder);
        (push_pointwise(&mut builder), reduction)
    } else {
        let pointwise = push_pointwise(&mut builder);
        (pointwise, push_reduction(&mut builder))
    };

    TwoStage {
        builder,
        pointwise,
        reduction,
        source,
        temporary,
        output,
        temporary_allocation,
        output_allocation,
        source_view,
        temporary_view,
        abi,
    }
}

/// Completes the two-stage program's structure, leaving its program contract
/// undeclared so a test can state exactly the contract it is probing.
fn wire_two_stage_structure(mut wired: TwoStage) -> KernelProgramBuilder {
    wired
        .builder
        .push_data_dependency(wired.pointwise, wired.reduction, wired.temporary)
        .expect("data dependency");
    wired
        .builder
        .push_output(OutputKey::new("result").expect("key"), wired.output)
        .expect("named output");
    wired.builder
}

/// Completes the two-stage program with its data dependency, named output,
/// applicability guard, and routing-commit contract.
fn complete_two_stage(wired: TwoStage) -> KernelProgramBuilder {
    let mut builder = wire_two_stage_structure(wired);
    declare_program_contract(&mut builder);
    builder
}

fn two_stage(semantic: &SemanticProgram, shape: TwoStageShape) -> TwoStage {
    wire_two_stage(
        semantic,
        &pointwise_kernel(0, SCALE_BITS),
        &reduction_kernel(1),
        shape,
    )
}

fn canonical_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    complete_two_stage(two_stage(semantic, TwoStageShape::Canonical))
        .build()
        .expect("verified kernel program")
}

/// The separator is what distinguishes the reinterpreting steps, and only it.
///
/// `v7` and `v8` reinterpret retained bytes rather than adding any: `v7` reads
/// the same raw four-byte coverage ordinal as a canonical semantic occurrence,
/// and `v8` reads the same output records in interface order rather than sorted
/// by content. This fixture publishes exactly one output — asserted, because
/// that is the argument — so sorting its record list is the identity
/// permutation and its payload is byte-identical under all of those tags. A
/// `v6`, `v7`, or `v8` reader handed these bytes would recover records under a
/// meaning this layer no longer holds, which is why each tag stepped.
///
/// `v9` is a different kind of step and is included here for the same reason:
/// it *adds* framed refinement evidence inside the stage section, so the
/// historical spellings below are not merely reinterpretations of the current
/// payload — they are shorter encodings this test cannot reconstruct. What the
/// loop still proves is the property that matters at every step: no historical
/// separator over these bytes is the current identity.
#[test]
fn the_program_domain_separator_is_what_distinguishes_the_reinterpreting_steps() {
    const V6: &[u8] = b"tiler.kernel-program.v6\0";
    const V7: &[u8] = b"tiler.kernel-program.v7\0";
    const V8: &[u8] = b"tiler.kernel-program.v8\0";
    const V9: &[u8] = b"tiler.kernel-program.v9\0";
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);
    // One record: the v8 encoding and the v7 sort agree on this payload.
    assert_eq!(program.outputs().len(), 1);
    let current = program.canonical_identity().as_bytes();
    assert!(current.starts_with(V9));

    for historical in [V6, V7, V8] {
        let mut spelling = historical.to_vec();
        spelling.extend_from_slice(&current[V9.len()..]);
        assert_eq!(spelling.len(), current.len());
        assert_ne!(current, spelling.as_slice());
    }
}

/// Byte offsets at which `needle` occurs in `haystack`.
fn byte_offsets_of(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    haystack
        .windows(needle.len())
        .enumerate()
        .filter(|(_, window)| *window == needle)
        .map(|(offset, _)| offset)
        .collect()
}

/// Identity folds the published outputs in the interface's order.
///
/// The fixture's interface order is the reverse of the order the `v7` sort
/// produced — `z_sum` is declared first and sorts second — so the two rules
/// disagree on this program and agree on every one-output program. Every place
/// a key appears must now agree on the order; under the sorted rule the output
/// section was transposed against the rest.
///
/// The population is counted rather than pinned. A key appears once inside the
/// folded semantic graph identity, which has encoded outputs in declaration
/// order all along; once inside each coverage record's refinement evidence,
/// which nests that same graph identity; and once in the program's own output
/// section. Deriving the expected count from the program's own coverage is what
/// keeps this check able to say no after a coverage change, instead of failing
/// on a stale literal that says nothing about ordering.
#[test]
fn published_output_interface_order_reaches_program_identity() {
    let semantic = two_chain_program_keyed(["z_sum", "a_sum"]);
    let program = publish_two_chain_keyed(two_chain(&semantic, true), ["z_sum", "a_sum"], false)
        .build()
        .expect("interface-ordered publication verifies");
    assert_eq!(program.outputs().len(), 2);
    assert_eq!(
        program
            .outputs()
            .map(|output| output.key().as_str().to_owned())
            .collect::<Vec<_>>(),
        vec!["z_sum".to_owned(), "a_sum".to_owned()],
    );

    let identity = program.canonical_identity().as_bytes();
    let declared_first = byte_offsets_of(identity, b"z_sum");
    let declared_second = byte_offsets_of(identity, b"a_sum");
    let coverage_records: usize = program.stages().map(|stage| stage.coverage().len()).sum();
    let expected = coverage_records + 2;
    assert_eq!(
        declared_first.len(),
        expected,
        "semantic fold, one per coverage record, then output section"
    );
    assert_eq!(declared_second.len(), expected);
    for (first, second) in declared_first.iter().zip(&declared_second) {
        assert!(
            first < second,
            "the output section holds the sorted order, not the interface order",
        );
    }

    // The check can say no about the order rather than about rebuilding:
    // re-declaring the same interface reproduces the bytes exactly.
    let rebuilt = publish_two_chain_keyed(two_chain(&semantic, true), ["z_sum", "a_sum"], false)
        .build()
        .expect("verified kernel program");
    assert_eq!(identity, rebuilt.canonical_identity().as_bytes());
}

/// Publishing the interface in any other order fails closed.
///
/// This is the neighbour that makes the identity claim above meaningful: the
/// permuted program is not a second identity to distinguish, it is not a
/// program. Rejecting it is what makes
/// [`VerifiedKernelProgram::outputs`]'s ordering claim true rather than a
/// convention every consumer would have to trust its producer to have kept.
#[test]
fn publishing_the_outputs_out_of_interface_order_is_rejected() {
    let semantic = two_chain_program_keyed(["z_sum", "a_sum"]);
    assert_eq!(
        diagnostic(publish_two_chain_keyed(
            two_chain(&semantic, true),
            ["z_sum", "a_sum"],
            true,
        )),
        KernelProgramDiagnostic::MisorderedNamedOutput { position: 0 },
    );
    // The lexicographic fixture reaches the same refusal, so the rule is the
    // interface order and not an incidental agreement with the sorted one.
    let sorted_interface = two_chain_program();
    assert_eq!(
        diagnostic(publish_two_chain_keyed(
            two_chain(&sorted_interface, true),
            ["sum_a", "sum_b"],
            true,
        )),
        KernelProgramDiagnostic::MisorderedNamedOutput { position: 0 },
    );
    assert_eq!(
        KernelProgramDiagnostic::MisorderedNamedOutput { position: 0 }.rule(),
        "misordered-named-output",
    );
}

#[test]
fn a_verified_program_binds_its_refinements_coverage_and_named_outputs() {
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);

    assert_eq!(program.stages().len(), 2);
    assert_eq!(program.values().len(), 3);
    assert_eq!(program.allocations().len(), 3);
    assert_eq!(program.views().len(), 3);
    assert_eq!(program.dependencies().len(), 1);
    assert_eq!(program.outputs().len(), 1);
    assert_eq!(
        program.semantic_graph_identity(),
        semantic.semantic_identity().graph()
    );

    // The stage DAG is ordered by its typed dependency, not by insertion.
    let order: Vec<_> = program
        .execution_order()
        .map(|stage| stage.coverage().to_vec())
        .collect();
    assert_eq!(
        order,
        vec![occurrences(&semantic, 0..4), occurrences(&semantic, 4..5)]
    );

    // Each stage retains the exact structured kernel it dispatches, which in
    // turn retains the exact scheduled region that kernel refines.
    let pointwise = program.stages().next().expect("pointwise stage");
    assert_eq!(
        pointwise.kernel().canonical_identity(),
        pointwise_kernel(0, SCALE_BITS).canonical_identity()
    );
    assert_eq!(
        pointwise.kernel().scheduled_region_identity(),
        pointwise_region(0, SCALE_BITS).canonical_identity()
    );
    assert_eq!(pointwise.accesses().len(), 2);

    // The temporary is defined by the pointwise stage and lives in its own
    // program-owned allocation.
    let temporary = program
        .values()
        .find(|value| value.role() == ValueRole::Temporary)
        .expect("one temporary");
    assert_eq!(temporary.required_bytes(), 24);
    assert_eq!(temporary.shape(), &input_shape());
    assert_eq!(temporary.definition(), Some(pointwise));
    assert_eq!(
        temporary.allocation().ownership(),
        AllocationOwnership::Program
    );
    assert_eq!(temporary.allocation().values().count(), 1);

    // The input is externally bound and has no defining stage.
    let source = program
        .values()
        .find(|value| value.role() == ValueRole::Input)
        .expect("one input");
    assert_eq!(source.definition(), None);
    assert_eq!(
        source.origin(),
        &MaterializedOrigin::ProgramInput {
            key: InputKey::new("input").expect("key"),
        }
    );

    let output = program.outputs().next().expect("one output");
    assert_eq!(output.key().as_str(), "result");
    assert_eq!(output.value().role(), ValueRole::Output);
    assert_eq!(output.value().required_bytes(), 8);
}

#[test]
fn identity_is_deterministic_and_independent_of_declaration_order() {
    let semantic = serial_sum_program(SCALE_BITS);
    let first = canonical_program(&semantic);
    let second = canonical_program(&semantic);
    assert_eq!(
        first.canonical_identity().as_bytes(),
        second.canonical_identity().as_bytes()
    );

    let reordered = complete_two_stage(two_stage(&semantic, TwoStageShape::ReversedDeclaration))
        .build()
        .expect("verified kernel program");
    assert_eq!(
        first.canonical_identity().as_bytes(),
        reordered.canonical_identity().as_bytes()
    );
    assert_eq!(first, reordered);
}

#[test]
fn identity_excludes_the_transient_planning_region_ordinal() {
    let semantic = serial_sum_program(SCALE_BITS);
    // The same schedules planned under different `RegionId` ordinals.
    let renumbered_pointwise = pointwise_kernel(41, SCALE_BITS);
    let renumbered_reduction = reduction_kernel(97);
    assert_ne!(
        renumbered_pointwise.scheduled_region(),
        pointwise_kernel(0, SCALE_BITS).scheduled_region()
    );
    assert_eq!(
        renumbered_pointwise.canonical_identity(),
        pointwise_kernel(0, SCALE_BITS).canonical_identity()
    );

    let renumbered = complete_two_stage(wire_two_stage(
        &semantic,
        &renumbered_pointwise,
        &renumbered_reduction,
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("verified kernel program");
    assert_eq!(
        canonical_program(&semantic).canonical_identity().as_bytes(),
        renumbered.canonical_identity().as_bytes()
    );
}

#[test]
fn identity_changes_when_the_semantic_graph_layer_changes() {
    // Identical bound implementations, coverage, and structure over two graphs
    // that differ only in one constant: only the ADR 0072 semantic-graph layer
    // moves, and program identity must move with it.
    let first = serial_sum_program(SCALE_BITS);
    let second = serial_sum_program(OTHER_SCALE_BITS);
    assert_ne!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph()
    );

    let over_first = canonical_program(&first);
    let over_second = canonical_program(&second);
    assert_ne!(
        over_first.canonical_identity().as_bytes(),
        over_second.canonical_identity().as_bytes()
    );
    assert_ne!(over_first, over_second);
}

#[test]
fn identity_changes_when_a_bound_refinement_changes() {
    // One semantic graph, one coverage split, one structure: only the selected
    // pointwise refinement differs.
    let semantic = serial_sum_program(SCALE_BITS);
    let selected = pointwise_kernel(0, SCALE_BITS);
    let alternative = pointwise_kernel(0, OTHER_SCALE_BITS);
    assert_ne!(
        selected.canonical_identity(),
        alternative.canonical_identity()
    );

    let first = complete_two_stage(wire_two_stage(
        &semantic,
        &selected,
        &reduction_kernel(1),
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("verified kernel program");
    let second = complete_two_stage(wire_two_stage(
        &semantic,
        &alternative,
        &reduction_kernel(1),
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("verified kernel program");
    assert_ne!(
        first.canonical_identity().as_bytes(),
        second.canonical_identity().as_bytes()
    );
}

/// Evidence is identity, not decoration.
///
/// The two programs agree on the semantic graph, the bound kernels, the
/// coverage partition, and every covered occurrence — asserted, because those
/// agreements are what make the remaining difference the refinement evidence
/// and nothing else. The receipts were minted under two governed numerical
/// contracts, which is a real difference in what was proved and not a fixture
/// trick: a contract is folded into executable coverage and is deliberately
/// absent from semantic graph meaning.
#[test]
fn identity_changes_when_only_the_refinement_evidence_changes() {
    let semantic = serial_sum_program(SCALE_BITS);
    let strict = canonical_program(&semantic);
    let alternative = complete_two_stage(two_stage(
        &semantic,
        TwoStageShape::AlternateRefinementEvidence,
    ))
    .build()
    .expect("verified kernel program over alternate refinement evidence");

    assert_eq!(
        strict.semantic_graph_identity(),
        alternative.semantic_graph_identity()
    );
    let paired = || strict.stages().zip(alternative.stages());
    assert!(paired().all(|(left, right)| left.kernel() == right.kernel()));
    assert!(paired().all(|(left, right)| {
        left.coverage()
            .iter()
            .map(CoveredOccurrence::occurrence)
            .eq(right.coverage().iter().map(CoveredOccurrence::occurrence))
    }));
    assert!(paired().any(|(left, right)| {
        left.coverage()
            .iter()
            .zip(right.coverage())
            .any(|(left, right)| left.refinement() != right.refinement())
    }));

    assert_ne!(
        strict.canonical_identity().as_bytes(),
        alternative.canonical_identity().as_bytes(),
    );
}

/// A receipt from another graph is refused before it can stand in for one here.
///
/// The foreign graph has the same five operations at the same canonical
/// ordinals, so nothing about the occurrence itself would catch the
/// substitution — only the retained graph does.
#[test]
fn coverage_proved_against_another_graph_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let foreign = serial_sum_program(OTHER_SCALE_BITS);
    assert_ne!(
        semantic.semantic_identity().graph(),
        foreign.semantic_identity().graph()
    );
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let storage = wire_two_stage_storage(&mut builder, TwoStageShape::Canonical);
    assert_eq!(
        builder
            .push_stage(
                &pointwise_kernel(0, SCALE_BITS),
                &occurrences(&foreign, 0..4),
                &[
                    read(storage.source_view, abi.input_bytes),
                    write(storage.temporary_view, abi.input_bytes),
                ],
                abi.pointwise_launch(),
            )
            .expect_err("a receipt minted against another graph is not evidence here"),
        KernelProgramBuildError::ForeignCoverageGraph {
            occurrence: SemanticOccurrence::new(0),
        }
    );
}

#[test]
fn identity_changes_when_complete_coverage_is_partitioned_differently() {
    // One semantic graph and one pair of bound implementations; two different
    // complete and disjoint coverage partitions.
    let semantic = serial_sum_program(SCALE_BITS);
    let canonical = canonical_program(&semantic);
    let shifted = complete_two_stage(two_stage(&semantic, TwoStageShape::ShiftedCoverage))
        .build()
        .expect("verified kernel program");
    assert_ne!(
        canonical.canonical_identity().as_bytes(),
        shifted.canonical_identity().as_bytes()
    );
}

#[test]
fn incomplete_coverage_of_the_bound_graph_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let external = builder
        .push_allocation(device(24, AllocationOwnership::External))
        .expect("external allocation");
    let owned = builder
        .push_allocation(device(24, AllocationOwnership::Program))
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let source = builder
        .push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            external,
        )
        .expect("input value");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                input_shape(),
            ),
            owned,
        )
        .expect("temporary value");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            output_allocation,
        )
        .expect("output value");
    let source_view = builder.push_whole_view(source).expect("input view");
    let temporary_view = builder.push_whole_view(temporary).expect("temporary view");
    let output_view = builder.push_whole_view(output).expect("output view");
    let pointwise = builder
        .push_stage(
            &pointwise_kernel(0, SCALE_BITS),
            // One graph operation is left uncovered.
            &occurrences(&semantic, 0..3),
            &[
                read(source_view, abi.input_bytes),
                write(temporary_view, abi.input_bytes),
            ],
            abi.pointwise_launch(),
        )
        .expect("pointwise stage");
    let reduction = builder
        .push_stage(
            &reduction_kernel(1),
            &occurrences(&semantic, 3..4),
            &[
                read(temporary_view, abi.input_bytes),
                write(output_view, abi.output_bytes),
            ],
            abi.reduction_launch(),
        )
        .expect("reduction stage");
    builder
        .push_data_dependency(pointwise, reduction, temporary)
        .expect("data dependency");
    builder
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("named output");
    declare_program_contract(&mut builder);

    assert_eq!(
        diagnostic(builder),
        KernelProgramDiagnostic::IncompleteCoverage {
            covered: 4,
            required: 5,
        }
    );
}

#[test]
fn covering_one_occurrence_twice_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    // Coverage the two wired stages already claim, but proved under another
    // numerical contract. The refusal is therefore about the occurrence being
    // claimed twice and not about a record repeating byte for byte — the case
    // that matters, because two *different* proofs of one occurrence are the
    // ambiguity this binding exists to make impossible.
    let conflicting = coverage_range(&checked_coverage(&semantic, &flush_contract()), 3..5);
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(2, OTHER_SCALE_BITS),
                &conflicting,
                &[
                    read(wired.source_view, wired.abi.input_bytes),
                    write(wired.temporary_view, wired.abi.input_bytes),
                ],
                wired.abi.pointwise_launch(),
            )
            .expect_err("repeated coverage is rejected"),
        KernelProgramBuildError::DuplicateCoverage {
            occurrence: SemanticOccurrence::new(3),
        }
    );
}

#[test]
fn a_read_without_its_declared_data_dependency_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_output(OutputKey::new("result").expect("key"), wired.output)
        .expect("named output");
    assert_eq!(
        diagnostic(wired.builder),
        KernelProgramDiagnostic::MissingDataDependency
    );
}

#[test]
fn a_dependency_that_states_an_unrealized_obligation_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    // A handoff on an allocation holding a single value can never release
    // storage from one value to another: the edge names an obligation its two
    // stages do not realize.
    wired
        .builder
        .push_storage_handoff(wired.reduction, wired.pointwise, wired.temporary_allocation)
        .expect("the edge is locally well formed");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::MisattributedDependency
    );
}

#[test]
fn an_output_may_not_share_storage_with_another_value() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::SharedOutputStorage);
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::ForbiddenAlias
    );
}

#[test]
fn an_unused_view_or_allocation_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);

    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_view(
            wired.temporary,
            ByteWindow {
                offset: 0,
                length: 4,
            },
        )
        .expect("the view is locally well formed");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::UnusedView
    );

    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_allocation(device(64, AllocationOwnership::Program))
        .expect("the allocation is locally well formed");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::UnusedAllocation
    );
}

#[test]
fn two_indistinguishable_entities_make_identity_ambiguous_and_are_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    // Two allocations with identical content and identical (empty) bindings
    // cannot be told apart by any canonical key, so identity would be
    // ambiguous rather than merely redundant.
    for _ in 0..2 {
        wired
            .builder
            .push_allocation(device(64, AllocationOwnership::Program))
            .expect("the allocation is locally well formed");
    }
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::AmbiguousCanonicalKey {
            entity: ProgramEntityKind::Allocation,
        }
    );
}

#[test]
fn a_value_with_no_writer_or_two_writers_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);

    // No writer: the reduction stage alone reads a temporary nobody defines.
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let owned = builder
        .push_allocation(device(24, AllocationOwnership::Program))
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                input_shape(),
            ),
            owned,
        )
        .expect("temporary value");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            output_allocation,
        )
        .expect("output value");
    let temporary_view = builder.push_whole_view(temporary).expect("temporary view");
    let output_view = builder.push_whole_view(output).expect("output view");
    builder
        .push_stage(
            &reduction_kernel(1),
            &occurrences(&semantic, 0..5),
            &[
                read(temporary_view, abi.input_bytes),
                write(output_view, abi.output_bytes),
            ],
            abi.reduction_launch(),
        )
        .expect("reduction stage");
    builder
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("named output");
    declare_program_contract(&mut builder);
    assert_eq!(diagnostic(builder), KernelProgramDiagnostic::MissingWriter);

    // Two writers: a third stage redefines the temporary the pointwise stage
    // already fully initializes.
    let mut wired = two_stage(&semantic, TwoStageShape::ReservedCoverage);
    wired
        .builder
        .push_stage(
            &pointwise_kernel(2, OTHER_SCALE_BITS),
            &occurrences(&semantic, 4..5),
            &[
                read(wired.source_view, wired.abi.input_bytes),
                write(wired.temporary_view, wired.abi.input_bytes),
            ],
            wired.abi.pointwise_launch(),
        )
        .expect("second writing stage");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::MultipleWriters
    );
}

#[test]
fn a_handle_minted_by_another_program_builder_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let foreign = two_stage(&semantic, TwoStageShape::Canonical);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);

    assert_eq!(
        wired
            .builder
            .push_data_dependency(wired.pointwise, wired.reduction, foreign.temporary)
            .expect_err("a foreign value handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Value,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_data_dependency(foreign.pointwise, wired.reduction, wired.temporary)
            .expect_err("a foreign stage handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Stage,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_whole_view(foreign.source)
            .expect_err("a foreign value handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Value,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_storage_handoff(wired.pointwise, wired.reduction, foreign.output_allocation)
            .expect_err("a foreign allocation handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Allocation,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(3, SCALE_BITS),
                &occurrences(&semantic, 4..5),
                &[
                    read(foreign.source_view, wired.abi.input_bytes),
                    write(foreign.temporary_view, wired.abi.input_bytes),
                ],
                wired.abi.pointwise_launch(),
            )
            .expect_err("a foreign view handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::View,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(3, SCALE_BITS),
                &occurrences(&semantic, 4..5),
                &[
                    read(wired.source_view, foreign.abi.input_bytes),
                    write(wired.temporary_view, wired.abi.input_bytes),
                ],
                wired.abi.pointwise_launch(),
            )
            .expect_err("a foreign ABI expression handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::AbiExpression,
        }
    );
    assert_eq!(
        wired
            .builder
            .applicability_guard(foreign.abi.input_bytes)
            .expect_err("a foreign ABI expression handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::AbiExpression,
        }
    );
}

#[test]
fn a_stage_access_must_realize_its_bound_kernel_signature() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::ShiftedCoverage);
    let kernel = pointwise_kernel(2, OTHER_SCALE_BITS);

    let bytes = wired.abi.input_bytes;
    let launch = wired.abi.pointwise_launch();
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(&semantic, 3..4),
                &[read(wired.source_view, bytes)],
                launch,
            )
            .expect_err("access arity is checked"),
        KernelProgramBuildError::StageAccessArity {
            expected: 2,
            actual: 1,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(&semantic, 3..4),
                &[
                    read(wired.temporary_view, bytes),
                    write(wired.temporary_view, bytes),
                ],
                launch,
            )
            .expect_err("tensor roles are checked"),
        KernelProgramBuildError::StageTensorRole {
            position: 0,
            expected: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            actual: ValueRole::Temporary,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(&semantic, 3..4),
                &[
                    write(wired.source_view, bytes),
                    write(wired.temporary_view, bytes),
                ],
                launch,
            )
            .expect_err("access modes are checked"),
        KernelProgramBuildError::StageAccessMode {
            position: 0,
            expected: StageAccessMode::Read,
            actual: StageAccessMode::Write,
        }
    );

    let partial = wired
        .builder
        .push_view(
            wired.source,
            ByteWindow {
                offset: 0,
                length: 8,
            },
        )
        .expect("partial view");
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(&semantic, 3..4),
                &[read(partial, bytes), write(wired.temporary_view, bytes)],
                launch,
            )
            .expect_err("addressed extents are checked"),
        KernelProgramBuildError::StageElementCount {
            position: 0,
            expected: 6,
            actual: 2,
        }
    );
    assert!(matches!(
        wired.builder.push_view(
            wired.source,
            ByteWindow {
                offset: 16,
                length: 16,
            }
        ),
        Err(KernelProgramBuildError::ViewOutOfRange { .. })
    ));
}

/// The wired four-stage two-chain program with a shared temporary allocation.
struct TwoChain {
    builder: KernelProgramBuilder,
    first_map: StageId,
    second_reduce: StageId,
    first_output: MaterializedValueId,
    second_output: MaterializedValueId,
    shared: AllocationId,
}

/// Wires two independent chains whose temporaries share one allocation.
///
/// The forward handoff orders the first chain's final reader before the second
/// chain's writer, which is what makes reusing the shared allocation legal.
fn two_chain(semantic: &SemanticProgram, handoff: bool) -> TwoChain {
    let pointwise = pointwise_kernel(0, SCALE_BITS);
    let reduction = reduction_kernel(1);
    let mut builder = KernelProgramBuilder::new(semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let storage = wire_chain_storage(&mut builder);
    // Verified once for the whole eight-operation graph, then partitioned; the
    // four stages claim disjoint ranges of the same evidence.
    let coverage = checked_coverage(semantic, &strict_contract());

    let first_map = builder
        .push_stage(
            &pointwise,
            &coverage_range(&coverage, 0..4),
            &[
                read(storage.first_source_view, abi.input_bytes),
                write(storage.first_temporary_view, abi.input_bytes),
            ],
            abi.pointwise_launch(),
        )
        .expect("first map stage");
    let first_reduce = builder
        .push_stage(
            &reduction,
            &coverage_range(&coverage, 4..5),
            &[
                read(storage.first_temporary_view, abi.input_bytes),
                write(storage.first_output_view, abi.output_bytes),
            ],
            abi.reduction_launch(),
        )
        .expect("first reduce stage");
    let second_map = builder
        .push_stage(
            &pointwise,
            &coverage_range(&coverage, 5..7),
            &[
                read(storage.second_source_view, abi.input_bytes),
                write(storage.second_temporary_view, abi.input_bytes),
            ],
            abi.pointwise_launch(),
        )
        .expect("second map stage");
    let second_reduce = builder
        .push_stage(
            &reduction,
            &coverage_range(&coverage, 7..8),
            &[
                read(storage.second_temporary_view, abi.input_bytes),
                write(storage.second_output_view, abi.output_bytes),
            ],
            abi.reduction_launch(),
        )
        .expect("second reduce stage");

    builder
        .push_data_dependency(first_map, first_reduce, storage.first_temporary)
        .expect("first data dependency");
    builder
        .push_data_dependency(second_map, second_reduce, storage.second_temporary)
        .expect("second data dependency");
    if handoff {
        builder
            .push_storage_handoff(first_reduce, second_map, storage.shared)
            .expect("storage handoff");
    }
    TwoChain {
        builder,
        first_map,
        second_reduce,
        first_output: storage.first_output,
        second_output: storage.second_output,
        shared: storage.shared,
    }
}

/// The allocations, values, and views of the two-chain fixture.
struct ChainStorage {
    shared: AllocationId,
    first_temporary: MaterializedValueId,
    second_temporary: MaterializedValueId,
    first_output: MaterializedValueId,
    second_output: MaterializedValueId,
    first_source_view: ViewId,
    second_source_view: ViewId,
    first_temporary_view: ViewId,
    second_temporary_view: ViewId,
    first_output_view: ViewId,
    second_output_view: ViewId,
}

/// Declares two externally bound inputs, two temporaries sharing one
/// program-owned allocation, and two separately allocated program outputs.
fn wire_chain_storage(builder: &mut KernelProgramBuilder) -> ChainStorage {
    let first_external = builder
        .push_allocation(device(24, AllocationOwnership::External))
        .expect("first external allocation");
    let second_external = builder
        .push_allocation(device(24, AllocationOwnership::External))
        .expect("second external allocation");
    let shared = builder
        .push_allocation(device(24, AllocationOwnership::Program))
        .expect("shared temporary allocation");
    let first_output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("first output allocation");
    let second_output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("second output allocation");

    let internal_temporary = || {
        value(
            MaterializedOrigin::Internal,
            ValueRole::Temporary,
            input_shape(),
        )
    };
    let internal_output = || {
        value(
            MaterializedOrigin::Internal,
            ValueRole::Output,
            output_shape(),
        )
    };
    let first_source = builder
        .push_value(
            value(program_input("a"), ValueRole::Input, input_shape()),
            first_external,
        )
        .expect("first input value");
    let second_source = builder
        .push_value(
            value(program_input("b"), ValueRole::Input, input_shape()),
            second_external,
        )
        .expect("second input value");
    let first_temporary = builder
        .push_value(internal_temporary(), shared)
        .expect("first temporary");
    let second_temporary = builder
        .push_value(internal_temporary(), shared)
        .expect("second temporary");
    let first_output = builder
        .push_value(internal_output(), first_output_allocation)
        .expect("first output value");
    let second_output = builder
        .push_value(internal_output(), second_output_allocation)
        .expect("second output value");

    ChainStorage {
        shared,
        first_temporary,
        second_temporary,
        first_output,
        second_output,
        first_source_view: builder.push_whole_view(first_source).expect("view"),
        second_source_view: builder.push_whole_view(second_source).expect("view"),
        first_temporary_view: builder.push_whole_view(first_temporary).expect("view"),
        second_temporary_view: builder.push_whole_view(second_temporary).expect("view"),
        first_output_view: builder.push_whole_view(first_output).expect("view"),
        second_output_view: builder.push_whole_view(second_output).expect("view"),
    }
}

fn publish_two_chain(chains: TwoChain) -> KernelProgramBuilder {
    publish_two_chain_keyed(chains, ["sum_a", "sum_b"], false)
}

/// Publishes the two chain outputs, optionally against the interface order.
///
/// Insertion admits either order — it checks key membership and rejects a
/// repeated key and role, nothing more — so `reversed` produces a builder that
/// only whole-program verification can refuse.
fn publish_two_chain_keyed(
    mut chains: TwoChain,
    keys: [&str; 2],
    reversed: bool,
) -> KernelProgramBuilder {
    let mut published = [
        (keys[0], chains.first_output),
        (keys[1], chains.second_output),
    ];
    if reversed {
        published.reverse();
    }
    for (key, value) in published {
        chains
            .builder
            .push_output(OutputKey::new(key).expect("key"), value)
            .expect("named output");
    }
    declare_program_contract(&mut chains.builder);
    chains.builder
}

#[test]
fn storage_reuse_is_admitted_only_with_an_explicit_handoff() {
    let semantic = two_chain_program();
    let program = publish_two_chain(two_chain(&semantic, true))
        .build()
        .expect("reuse with an explicit handoff verifies");
    assert_eq!(program.stages().len(), 4);
    assert_eq!(program.allocations().len(), 5);
    assert_eq!(program.outputs().len(), 2);

    // The shared allocation carries exactly the two internal temporaries.
    let shared = program
        .allocations()
        .find(|allocation| allocation.values().count() == 2)
        .expect("one shared allocation");
    assert!(
        shared
            .values()
            .all(|value| value.role() == ValueRole::Temporary)
    );

    // Without the handoff the reuse is unproven and the program fails closed.
    let rejected = diagnostic(publish_two_chain(two_chain(&semantic, false)));
    assert!(
        matches!(
            rejected,
            KernelProgramDiagnostic::ReuseMissingHandoff
                | KernelProgramDiagnostic::ReuseLifetimeOverlap
        ),
        "unexpected diagnostic: {rejected:?}"
    );
}

#[test]
fn a_dependency_cycle_is_rejected() {
    let semantic = two_chain_program();
    let mut chains = two_chain(&semantic, true);
    // The opposite handoff is locally well formed and realized — the second
    // chain's reader precedes the first chain's writer — but together with the
    // forward handoff it closes a cycle.
    chains
        .builder
        .push_storage_handoff(chains.second_reduce, chains.first_map, chains.shared)
        .expect("the edge is locally well formed");
    assert_eq!(
        diagnostic(publish_two_chain(chains)),
        KernelProgramDiagnostic::DependencyCycle
    );
}

#[test]
fn a_missing_named_output_is_rejected() {
    let semantic = two_chain_program();
    let mut chains = two_chain(&semantic, true);
    chains
        .builder
        .push_output(OutputKey::new("sum_a").expect("key"), chains.first_output)
        .expect("first named output");
    // The second declared semantic output is never published.
    assert_eq!(
        diagnostic(chains.builder),
        KernelProgramDiagnostic::MissingNamedOutput
    );
}

#[test]
fn an_output_key_outside_the_bound_interface_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    assert!(matches!(
        wired
            .builder
            .push_output(OutputKey::new("other").expect("key"), wired.output),
        Err(KernelProgramBuildError::UnknownOutputKey { .. })
    ));
    assert!(matches!(
        wired.builder.push_value(
            value(program_input("other"), ValueRole::Input, input_shape()),
            wired.output_allocation,
        ),
        Err(KernelProgramBuildError::UnknownProgramInput { .. })
    ));
    // The one declared input is already claimed by another materialized value.
    assert!(matches!(
        wired.builder.push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            wired.output_allocation,
        ),
        Err(KernelProgramBuildError::DuplicateProgramInput { .. })
    ));
    // A temporary claiming a program input is a role/origin contradiction.
    assert_eq!(
        wired
            .builder
            .push_value(
                value(program_input("input"), ValueRole::Temporary, input_shape()),
                wired.temporary_allocation,
            )
            .expect_err("role and origin must agree"),
        KernelProgramBuildError::ValueRoleOrigin {
            role: ValueRole::Temporary,
        }
    );
}

#[test]
fn an_internal_component_without_a_logical_group_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    let role = EncodedComponentRole::new(77);
    let error = wired
        .builder
        .push_component_value(
            MaterializedComponentSpec {
                origin: MaterializedOrigin::Internal,
                role: ValueRole::Temporary,
                component_role: role,
                shape: input_shape(),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            wired.temporary_allocation,
        )
        .expect_err("ungrouped internal components must fail closed");
    assert_eq!(
        error,
        KernelProgramBuildError::UngroupedInternalComponent { role }
    );
}

#[test]
fn physical_storage_scalar_and_kernel_access_type_are_checked_separately() {
    let semantic = strict_affine_u4_passthrough_program();
    let mut builder = KernelProgramBuilder::new(&semantic).expect("program builder");
    let allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 20,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("allocation");
    let spec = |storage_scalar, element_type| MaterializedComponentSpec {
        origin: program_input("input"),
        role: ValueRole::Input,
        component_role: STRICT_AFFINE_CODES_ROLE,
        shape: Shape::from_dims([5]),
        storage_scalar,
        element_type,
        encoding: StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
        alignment: 1,
        memory_space: MemorySpace::Device,
    };

    assert_eq!(
        builder
            .push_component_value(spec(StorageScalar::F32, KernelType::U8), allocation)
            .expect_err("a float scalar cannot carry packed codes"),
        KernelProgramBuildError::StorageEncodingScalar {
            scalar: StorageScalar::F32,
            encoding: StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
        }
    );
    assert_eq!(
        builder
            .push_component_value(spec(StorageScalar::U8, KernelType::Bool), allocation)
            .expect_err("a boolean access must not stand in for an unsigned byte"),
        KernelProgramBuildError::StorageAccessType {
            scalar: StorageScalar::U8,
            encoding: StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
            expected: KernelType::U8,
            actual: KernelType::Bool,
        }
    );
}

#[test]
fn packed_program_views_are_bounded_to_the_complete_component() {
    let semantic = strict_affine_u4_passthrough_program();
    let mut builder = KernelProgramBuilder::new(&semantic).expect("program builder");
    let allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 3,
            alignment: 1,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("allocation");
    let value = builder
        .push_component_value(
            MaterializedComponentSpec {
                origin: program_input("input"),
                role: ValueRole::Input,
                component_role: STRICT_AFFINE_CODES_ROLE,
                shape: Shape::from_dims([5]),
                storage_scalar: StorageScalar::U8,
                element_type: KernelType::U8,
                encoding: StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
                alignment: 1,
                memory_space: MemorySpace::Device,
            },
            allocation,
        )
        .expect("packed codes");
    assert_eq!(
        builder
            .push_view(
                value,
                ByteWindow {
                    offset: 0,
                    length: 2,
                },
            )
            .expect_err("a partial packed byte view has no logical ownership proof"),
        KernelProgramBuildError::PartialPackedView {
            offset: 0,
            length: 2,
            value_bytes: 3,
        }
    );
    builder
        .push_whole_view(value)
        .expect("the whole packed component is stage-visible");
}

#[test]
fn strict_affine_stage_bindings_are_addressed_by_component_role() {
    let semantic = strict_affine_u4_dequantize_program();
    let kernel = strict_affine_u4_dequantize_kernel();
    let mut builder = KernelProgramBuilder::new(&semantic).expect("program builder");

    let mut component = |role, shape, storage_scalar, element_type, encoding, bytes, alignment| {
        let allocation = builder
            .push_allocation(AllocationSpec {
                capacity_bytes: bytes,
                alignment,
                memory_space: MemorySpace::Device,
                ownership: AllocationOwnership::External,
            })
            .expect("component allocation");
        let value = builder
            .push_component_value(
                MaterializedComponentSpec {
                    origin: program_input("input"),
                    role: ValueRole::Input,
                    component_role: role,
                    shape,
                    storage_scalar,
                    element_type,
                    encoding,
                    alignment,
                    memory_space: MemorySpace::Device,
                },
                allocation,
            )
            .expect("materialized component");
        builder.push_whole_view(value).expect("component view")
    };
    let codes = component(
        STRICT_AFFINE_CODES_ROLE,
        Shape::from_dims([5]),
        StorageScalar::U8,
        KernelType::U8,
        StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
        3,
        1,
    );
    let scale = component(
        STRICT_AFFINE_SCALE_ROLE,
        Shape::new([]),
        StorageScalar::F32,
        KernelType::F32,
        StorageEncoding::Unpacked,
        4,
        4,
    );
    let zero_point = component(
        STRICT_AFFINE_ZERO_POINT_ROLE,
        Shape::new([]),
        StorageScalar::U8,
        KernelType::U8,
        StorageEncoding::Unpacked,
        1,
        1,
    );
    let output_allocation = builder
        .push_allocation(device(20, AllocationOwnership::Program))
        .expect("output allocation");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                Shape::from_dims([5]),
            ),
            output_allocation,
        )
        .expect("dense output");
    let output = builder.push_whole_view(output).expect("output view");

    let codes_bytes = literal(&mut builder, 3);
    let scale_bytes = literal(&mut builder, 4);
    let zero_point_bytes = literal(&mut builder, 1);
    let output_bytes = literal(&mut builder, 20);
    let grid_threads = literal(&mut builder, 5);
    let threads_per_workgroup = literal(&mut builder, 1);
    let error = builder
        .push_stage(
            &kernel,
            &occurrences(&semantic, 0..1),
            &[
                read(zero_point, zero_point_bytes),
                read(scale, scale_bytes),
                read(codes, codes_bytes),
                write(output, output_bytes),
            ],
            StageLaunch {
                grid_threads,
                threads_per_workgroup,
            },
        )
        .expect_err("same-width input components must not bind by position");
    assert_eq!(
        error,
        KernelProgramBuildError::StageComponentRole {
            position: 0,
            expected: Some(STRICT_AFFINE_CODES_ROLE),
            actual: Some(STRICT_AFFINE_ZERO_POINT_ROLE),
        }
    );
}

#[test]
fn identity_changes_when_the_applicability_guard_changes() {
    // One semantic graph, one pair of bound implementations, one structure and
    // one routing contract: only the predicate deciding whether this program
    // may be routed to differs. Under `tiler.kernel-program.v1` these two were
    // the same bytes, which is the cache hazard the domain bump closes.
    let semantic = serial_sum_program(SCALE_BITS);
    let canonical = canonical_program(&semantic);

    let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
    let two = literal(&mut builder, 2);
    let guard = builder
        .push_abi_binary(AbiBinaryOp::Equal, two, two)
        .expect("a differently spelled predicate");
    builder.applicability_guard(guard).expect("guard");
    declare_routing_commit(&mut builder);
    let guarded = builder.build().expect("verified kernel program");

    assert_ne!(
        canonical.canonical_identity().as_bytes(),
        guarded.canonical_identity().as_bytes()
    );
    assert_ne!(canonical, guarded);
}

#[test]
fn identity_changes_when_the_entry_abi_changes() {
    // The two programs agree on every byte count and every launch extent; they
    // disagree only on how those quantities are *computed*. A dynamic subject
    // computes them from bound input extents, so an identity blind to the
    // expression would collapse two programs whose ABI differs at run time.
    let semantic = serial_sum_program(SCALE_BITS);
    let canonical = canonical_program(&semantic);
    let computed = complete_two_stage(two_stage(&semantic, TwoStageShape::ComputedAccessibleBytes))
        .build()
        .expect("verified kernel program");

    let accesses = |program: &VerifiedKernelProgram| {
        program
            .stages()
            .map(|stage| stage.accesses().len())
            .sum::<usize>()
    };
    assert_eq!(accesses(&canonical), accesses(&computed));
    assert_ne!(
        canonical.canonical_identity().as_bytes(),
        computed.canonical_identity().as_bytes()
    );
}

#[test]
fn identity_changes_when_pre_commit_fallback_permission_changes() {
    // A program that may still be abandoned before commit and one that may not
    // are different execution contracts over identical work.
    let semantic = serial_sum_program(SCALE_BITS);
    let permitted = canonical_program(&semantic);

    let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
    declare_guard(&mut builder);
    declare_routing_commit_with_fallback(&mut builder, false);
    let forbidden = builder.build().expect("verified kernel program");

    assert!(permitted.routing_commit_contract()[0].fallback_permitted);
    assert!(!forbidden.routing_commit_contract()[0].fallback_permitted);
    assert_ne!(
        permitted.canonical_identity().as_bytes(),
        forbidden.canonical_identity().as_bytes()
    );
}

#[test]
fn a_program_without_an_applicability_guard_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
    declare_routing_commit(&mut builder);
    assert_eq!(
        diagnostic(builder),
        KernelProgramDiagnostic::MissingApplicabilityGuard
    );
}

#[test]
fn a_routing_commit_contract_that_stops_short_of_publication_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
    declare_guard(&mut builder);
    builder
        .push_routing_commit_transition(RoutingCommitTransition {
            from: RoutingCommitState::Preflight,
            to: RoutingCommitState::Committed,
            fallback_permitted: true,
        })
        .expect("the first transition is well formed");
    assert_eq!(
        diagnostic(builder),
        KernelProgramDiagnostic::IncompleteRoutingCommitContract {
            declared: 1,
            required: 3,
        }
    );
}

#[test]
fn a_routing_commit_step_that_breaks_the_lifecycle_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_routing_commit_transition(RoutingCommitTransition {
            from: RoutingCommitState::Preflight,
            to: RoutingCommitState::Committed,
            fallback_permitted: true,
        })
        .expect("the first transition is well formed");
    assert_eq!(
        wired
            .builder
            .push_routing_commit_transition(RoutingCommitTransition {
                from: RoutingCommitState::Committed,
                to: RoutingCommitState::Executing,
                fallback_permitted: true,
            })
            .expect_err("fallback after commit is rejected"),
        KernelProgramBuildError::RoutingCommitFallbackAfterCommit {
            from: RoutingCommitState::Committed,
        }
    );
    // A step that skips the state the previous one reached is rejected too.
    assert_eq!(
        wired
            .builder
            .push_routing_commit_transition(RoutingCommitTransition {
                from: RoutingCommitState::Executing,
                to: RoutingCommitState::Published,
                fallback_permitted: false,
            })
            .expect_err("the lifecycle order is checked"),
        KernelProgramBuildError::RoutingCommitOutOfOrder {
            expected: RoutingCommitState::Committed,
            actual: RoutingCommitState::Executing,
        }
    );
}

#[test]
fn an_abi_expression_no_use_site_reaches_is_rejected() {
    // Identity folds each use site by content key and nothing else, so an arena
    // node no use site reaches would be retained bytes identity does not cover.
    let semantic = serial_sum_program(SCALE_BITS);
    let mut builder = complete_two_stage(two_stage(&semantic, TwoStageShape::Canonical));
    literal(&mut builder, 4_096);
    assert_eq!(
        diagnostic(builder),
        KernelProgramDiagnostic::UnreferencedAbiExpression
    );
}

#[test]
fn an_accessible_range_the_declared_view_contradicts_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::ShiftedCoverage);
    let wrong = literal(&mut wired.builder, 25);
    let abi = wired.abi;
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(2, OTHER_SCALE_BITS),
                &occurrences(&semantic, 3..4),
                &[
                    read(wired.source_view, wrong),
                    write(wired.temporary_view, abi.input_bytes),
                ],
                abi.pointwise_launch(),
            )
            .expect_err("an accessible range must equal the view it addresses"),
        KernelProgramBuildError::AccessibleBytesDisagreement {
            position: 0,
            expected: 24,
            actual: 25,
        }
    );
}

#[test]
fn a_workgroup_width_the_bound_kernel_contradicts_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::ShiftedCoverage);
    let wrong_width = literal(&mut wired.builder, 32);
    let abi = wired.abi;
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(2, OTHER_SCALE_BITS),
                &occurrences(&semantic, 3..4),
                &[
                    read(wired.source_view, abi.input_bytes),
                    write(wired.temporary_view, abi.input_bytes),
                ],
                StageLaunch {
                    grid_threads: abi.pointwise_threads,
                    threads_per_workgroup: wrong_width,
                },
            )
            .expect_err("a declared workgroup width must be the kernel's"),
        KernelProgramBuildError::ThreadsPerWorkgroupDisagreement {
            expected: 1,
            actual: 32,
        }
    );
}

#[test]
fn an_abi_use_site_rejects_a_mistyped_or_target_dependent_expression() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);

    // A size is not a guard.
    assert_eq!(
        wired
            .builder
            .applicability_guard(wired.abi.input_bytes)
            .expect_err("a guard must be a predicate"),
        KernelProgramBuildError::AbiUseType {
            use_site: ProgramAbiUse::ApplicabilityGuard,
            expected: AbiType::Boolean,
            actual: AbiType::Unsigned,
        }
    );
    // A guard is not a size.
    let predicate = wired
        .builder
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("predicate");
    assert_eq!(
        wired
            .builder
            .push_abi_unary(AbiUnaryOp::NarrowU32, predicate)
            .expect_err("a narrowing operand must be unsigned"),
        KernelProgramBuildError::AbiOperandType {
            expected: AbiType::Unsigned,
            actual: AbiType::Boolean,
        }
    );

    // A launch extent must be computable before any device-dependent query, so
    // a governed target property is refused at that use site.
    let property = wired
        .builder
        .push_abi_root(AbiRoot::TargetProperty {
            key: TargetPropertyKey::new("tiler.test.max-threads").expect("property key"),
            phase: AvailabilityPhase::LiveDevicePreflight,
        })
        .expect("target property root");
    let abi = wired.abi;
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(2, OTHER_SCALE_BITS),
                &occurrences(&semantic, 0..1),
                &[
                    read(wired.source_view, abi.input_bytes),
                    write(wired.temporary_view, abi.input_bytes),
                ],
                StageLaunch {
                    grid_threads: property,
                    threads_per_workgroup: abi.threads_per_workgroup,
                },
            )
            .expect_err("a launch extent must read only interface facts"),
        KernelProgramBuildError::AbiNonInterfaceRoot {
            use_site: ProgramAbiUse::GridThreads,
        }
    );
}

#[test]
fn the_abi_arena_is_deduplicated_by_content() {
    // The canonical fixture names the same input byte count at three accesses
    // and the same workgroup width at both stages; the arena keeps one node per
    // distinct formula, so it stays a function of what the program says.
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);
    // 24, 8, 6, 2, 1, and the guard predicate.
    assert_eq!(program.abi_expressions().len(), 6);

    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    assert_eq!(literal(&mut wired.builder, 24), wired.abi.input_bytes);
}

/// How an arena-growth fixture wires each level to the one below it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AbiGrowth {
    /// Each level names the level below it once, and a shared `false` leaf.
    ///
    /// A key that embeds its operands' keys restates the whole level below at
    /// every level, so the array of all keys is quadratic in arena size.
    Chain,
    /// Each level names the level below it *twice*.
    ///
    /// The same nesting then restates it twice per level, so one key alone
    /// doubles per level — the case where the identity bound, rather than the
    /// program's size, was the only thing containing the encoding.
    SharedDag,
}

/// Builds an always-true guard whose subtree spans `levels` composed nodes.
///
/// Every level evaluates to `true`, so the fixture grows the arena and changes
/// nothing else about the program.
fn grown_guard(builder: &mut KernelProgramBuilder, growth: AbiGrowth, levels: usize) -> AbiExprId {
    let mut node = builder
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("guard root");
    for _ in 0..levels {
        node = match growth {
            AbiGrowth::Chain => {
                // Minted inside the loop so a zero-level fixture retains no
                // node that no use site reaches, which verification rejects.
                let filler = builder
                    .push_abi_root(AbiRoot::BooleanLiteral(false))
                    .expect("filler root");
                builder.push_abi_binary(AbiBinaryOp::Or, node, filler)
            }
            AbiGrowth::SharedDag => builder.push_abi_binary(AbiBinaryOp::And, node, node),
        }
        .expect("guard level");
    }
    node
}

/// The canonical two-stage program, with its guard grown to `levels`.
fn program_with_grown_abi(
    semantic: &SemanticProgram,
    growth: AbiGrowth,
    levels: usize,
) -> VerifiedKernelProgram {
    let mut builder = wire_two_stage_structure(two_stage(semantic, TwoStageShape::Canonical));
    let guard = grown_guard(&mut builder, growth, levels);
    builder
        .applicability_guard(guard)
        .expect("applicability guard");
    declare_routing_commit(&mut builder);
    builder.build().expect("verified kernel program")
}

/// The structural comparator is a strict total order over an arena.
///
/// This is what the two sorted expression sets in artifact identity rest on, so
/// a comparator that were merely *consistent* would not do: an intransitive one
/// makes `sort_by` produce an order that depends on the input permutation,
/// which is precisely the canonicity the sort exists to provide.
///
/// Checked exhaustively over every ordered pair and triple of a small arena
/// that carries all four constructors, sharing included.
#[test]
fn the_structural_comparator_is_a_total_order() {
    use crate::program::abi::{AbiBinaryOp, AbiRoot, AbiUnaryOp, ExprNode, compare_expr_nodes};
    use core::cmp::Ordering;

    // 0,1 are distinct leaves; 2 shares leaf 0; 3 and 4 differ only in operand
    // order, which is what a comparator that ignored operand position would miss.
    let nodes = vec![
        ExprNode::Root(AbiRoot::UnsignedLiteral(7)),
        ExprNode::Root(AbiRoot::UnsignedLiteral(9)),
        ExprNode::Unary {
            op: AbiUnaryOp::Not,
            operand: 0,
        },
        ExprNode::Binary {
            op: AbiBinaryOp::CheckedAdd,
            left: 0,
            right: 1,
        },
        ExprNode::Binary {
            op: AbiBinaryOp::CheckedAdd,
            left: 1,
            right: 0,
        },
        ExprNode::Select {
            condition: 2,
            if_true: 3,
            if_false: 4,
        },
    ];
    let all: Vec<u32> = (0..u32::try_from(nodes.len()).unwrap()).collect();
    let cmp = |a: u32, b: u32| compare_expr_nodes(&nodes, a, b);

    for &a in &all {
        assert_eq!(cmp(a, a), Ordering::Equal, "not reflexive at {a}");
        for &b in &all {
            assert_eq!(
                cmp(a, b),
                cmp(b, a).reverse(),
                "not antisymmetric at ({a}, {b})"
            );
            // Distinct arena positions holding structurally distinct nodes must
            // not compare equal, or two different expressions would tie and the
            // sorted order would depend on input order.
            if a != b {
                assert_ne!(cmp(a, b), Ordering::Equal, "{a} and {b} tied");
            }
            for &c in &all {
                if cmp(a, b) == Ordering::Less && cmp(b, c) == Ordering::Less {
                    assert_eq!(
                        cmp(a, c),
                        Ordering::Less,
                        "not transitive at ({a}, {b}, {c})"
                    );
                }
            }
        }
    }

    // Operand order is part of the structure: nodes 3 and 4 are `a + b` and
    // `b + a` over the same leaves and must not tie.
    assert_ne!(cmp(3, 4), Ordering::Equal, "operand order was ignored");
}

/// Reports identity size against arena size, and proves the curve is a line.
///
/// **The growth rate is the finding, not the absolute number.** Identity size
/// is deterministic — the same program yields the same byte count on every host
/// — so this needs neither repetition nor statistics, unlike a timing
/// measurement.
///
/// A constant increment per level is exactly the property the `v3` encoding
/// buys, and asserting it is what makes this a guard rather than a print: under
/// `v2`, which named each use site by a key that embedded its operands' keys,
/// the increment grew with the level under `Chain` and doubled under
/// `SharedDag`. See the ticket outcome for the two measured curves.
///
/// Reproduce with:
///
/// ```text
/// cargo nextest run -p tiler-ir -E 'test(abi_identity_size)' --no-capture
/// ```
#[test]
fn abi_identity_size_grows_linearly_with_the_arena() {
    /// Enough levels that a quadratic or an exponential curve is unmistakable,
    /// and few enough that a `SharedDag` fixture still fits in memory for
    /// anyone re-running this against the `v2` encoding.
    const LEVELS: std::ops::Range<usize> = 0..17;

    let semantic = serial_sum_program(SCALE_BITS);
    for growth in [AbiGrowth::Chain, AbiGrowth::SharedDag] {
        let mut sizes = Vec::new();
        for levels in LEVELS {
            let program = program_with_grown_abi(&semantic, growth, levels);
            let nodes = program.abi_expressions().len();
            let bytes = program.canonical_identity().as_bytes().len();
            println!("MEASURE {growth:?} {levels:>2} levels: {nodes:>2} nodes, {bytes} bytes");
            sizes.push((nodes, bytes));
        }

        // The first level is the one that mints the shared `false` leaf under
        // `Chain`, so the constant-increment claim starts after it.
        let increments: Vec<usize> = sizes
            .windows(2)
            .skip(1)
            .map(|pair| pair[1].1 - pair[0].1)
            .collect();
        assert!(
            increments.windows(2).all(|pair| pair[0] == pair[1]),
            "{growth:?} identity size must grow by a constant per level, measured {increments:?}"
        );
        let added_nodes: Vec<usize> = sizes
            .windows(2)
            .skip(1)
            .map(|pair| pair[1].0 - pair[0].0)
            .collect();
        assert!(
            added_nodes.iter().all(|added| *added == 1),
            "each level must add exactly one arena node, measured {added_nodes:?}"
        );
    }
}

/// Two guards over the same node kinds, wired differently, must differ.
///
/// Encoding the arena once and naming nodes by canonical position moves the
/// whole burden of distinguishing two expressions onto those position
/// references. This is the case that a reference encoding losing operand order,
/// or losing which node an operand names, would pass anyway: both programs hold
/// one `true`, one `false`, and two `Or`s, and differ only in what those `Or`s
/// name.
#[test]
fn identity_distinguishes_two_arenas_that_differ_only_in_their_wiring() {
    let semantic = serial_sum_program(SCALE_BITS);
    let build = |nest_left: bool| {
        let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
        let yes = builder
            .push_abi_root(AbiRoot::BooleanLiteral(true))
            .expect("true root");
        let no = builder
            .push_abi_root(AbiRoot::BooleanLiteral(false))
            .expect("false root");
        let inner = builder
            .push_abi_binary(AbiBinaryOp::Or, yes, no)
            .expect("inner disjunction");
        let guard = if nest_left {
            builder.push_abi_binary(AbiBinaryOp::Or, inner, no)
        } else {
            builder.push_abi_binary(AbiBinaryOp::Or, yes, inner)
        }
        .expect("outer disjunction");
        builder
            .applicability_guard(guard)
            .expect("applicability guard");
        declare_routing_commit(&mut builder);
        builder.build().expect("verified kernel program")
    };

    let left = build(true);
    let right = build(false);
    assert_eq!(left.abi_expressions().len(), right.abi_expressions().len());
    assert_ne!(
        left.canonical_identity().as_bytes(),
        right.canonical_identity().as_bytes()
    );
}

/// Declares the canonical split contract over the two-stage fixture.
///
/// The fixture's temporary is `[2, 3]` and its output `[2]`, so a split of
/// three partitions each combining one contributor is the structurally exact
/// contract over it. That the pointwise stage is not *semantically* a partial
/// reducer is deliberate and not a gap: this layer proves the structure of a
/// split — who writes the partials, who reads them, and that the coverage
/// arithmetic closes — while whether each pass really is the reduction pass it
/// claims is proven by the region verifier in `crate::schedule`.
fn split_over(wired: &TwoStage) -> PartialReduction {
    PartialReduction {
        producer: wired.pointwise,
        combiner: wired.reduction,
        partial: wired.temporary,
        result: wired.output,
        partitions: 3,
        contributors_per_partition: 1,
    }
}

fn program_with_split(
    semantic: &SemanticProgram,
    amend: impl FnOnce(&TwoStage, PartialReduction) -> PartialReduction,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let wired = two_stage(semantic, TwoStageShape::Canonical);
    let split = amend(&wired, split_over(&wired));
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_partial_reduction(split)
        .expect("a well-formed split declaration");
    declare_program_contract(&mut builder);
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// A declared split verifies and is readable back off the verified program.
#[test]
fn a_declared_split_reduction_is_verified_and_retained() {
    let semantic = serial_sum_program(SCALE_BITS);
    let program = program_with_split(&semantic, |_, split| split).expect("verified program");
    let split = program
        .partial_reductions()
        .next()
        .expect("one declared split");
    assert_eq!(split.partitions(), 3);
    assert_eq!(split.contributors_per_partition(), 1);
    assert_eq!(split.total_contributors(), Some(3));
    assert_eq!(split.producer(), program.stages().next().expect("a stage"));
    // The partials the producer stages are exactly the ones the combiner reads,
    // and the dispatch dependency between the two is the ordinary data edge.
    assert_eq!(split.partial().definition(), Some(split.producer()));
    assert!(program.dependencies().any(|edge| {
        edge.predecessor() == split.producer() && edge.successor() == split.combiner()
    }));
}

/// The split contract changes program identity, so two splits never collide.
#[test]
fn the_declared_split_separates_kernel_program_identity() {
    let semantic = serial_sum_program(SCALE_BITS);
    let undeclared = canonical_program(&semantic);
    let declared = program_with_split(&semantic, |_, split| split).expect("verified program");
    assert_ne!(
        undeclared.canonical_identity(),
        declared.canonical_identity(),
        "a program that proves a split must not share identity with one that does not"
    );
    // `contributors_per_partition` is the field program scope cannot derive, so
    // it is exactly the one identity has to carry.
    let restated = program_with_split(&semantic, |_, split| PartialReduction {
        contributors_per_partition: 7,
        ..split
    })
    .expect("verified program");
    assert_ne!(
        declared.canonical_identity(),
        restated.canonical_identity(),
        "two splits claiming different contributor coverage must differ"
    );
}

/// A split whose partial is written by some other stage is rejected.
#[test]
fn a_partial_not_initialized_by_its_producer_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    assert_eq!(
        program_with_split(&semantic, |wired, split| PartialReduction {
            producer: wired.reduction,
            combiner: wired.pointwise,
            ..split
        }),
        Err(KernelProgramDiagnostic::PartialNotInitializedByProducer)
    );
}

/// A split whose combiner does not produce the result is rejected.
#[test]
fn a_result_not_produced_by_its_combiner_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    assert_eq!(
        program_with_split(&semantic, |wired, split| PartialReduction {
            result: wired.temporary,
            partial: wired.output,
            ..split
        }),
        // The output is written by the combiner, not the producer, so the
        // partial obligation is the first one that fails.
        Err(KernelProgramDiagnostic::PartialNotInitializedByProducer)
    );
}

/// A split staging its partials in a published output is rejected.
#[test]
fn a_partial_that_is_not_an_internal_temporary_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::Canonical);
    let split = PartialReduction {
        producer: wired.reduction,
        combiner: wired.pointwise,
        partial: wired.output,
        result: wired.temporary,
        partitions: 3,
        contributors_per_partition: 1,
    };
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_partial_reduction(split)
        .expect("a well-formed split declaration");
    declare_program_contract(&mut builder);
    // The output *is* written by the named producer and read by nobody, so this
    // reaches the consumption rule rather than the materialization one; either
    // way the published output cannot serve as a split's staging tensor.
    assert_eq!(
        builder
            .build()
            .map_err(|error| *error.diagnostics().first().expect("one diagnostic")),
        Err(KernelProgramDiagnostic::PartialNotConsumedByCombiner)
    );
}

/// A split whose partial extent is not one value per partition is rejected.
#[test]
fn a_partial_extent_that_is_not_one_value_per_partition_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    for partitions in [2, 4, 6] {
        assert_eq!(
            program_with_split(&semantic, |_, split| PartialReduction {
                partitions,
                ..split
            }),
            Err(KernelProgramDiagnostic::PartialExtentMismatch),
            "a `[2]` result and a `[2, 3]` partial admit only three partitions"
        );
    }
}

/// A split covering nothing, or an unrepresentable amount, is rejected.
#[test]
fn an_unrepresentable_split_coverage_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    for (partitions, contributors_per_partition) in [(0, 4), (3, u64::MAX)] {
        assert_eq!(
            program_with_split(&semantic, |_, split| PartialReduction {
                partitions,
                contributors_per_partition,
                ..split
            }),
            Err(KernelProgramDiagnostic::PartialCoverageUnrepresentable),
            "{partitions} x {contributors_per_partition} states no checkable coverage"
        );
    }
}

/// One stage cannot be both passes, and one partial cannot be split twice.
#[test]
fn a_malformed_split_declaration_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::Canonical);
    let split = split_over(&wired);
    let mut builder = wire_two_stage_structure(wired);
    assert_eq!(
        builder.push_partial_reduction(PartialReduction {
            combiner: split.producer,
            ..split
        }),
        Err(KernelProgramBuildError::SelfDependency)
    );
    builder
        .push_partial_reduction(split)
        .expect("the first declaration is well formed");
    assert_eq!(
        builder.push_partial_reduction(PartialReduction {
            partitions: 1,
            contributors_per_partition: 3,
            ..split
        }),
        Err(KernelProgramBuildError::DuplicatePartialReduction)
    );
}

/// A stage computing nothing is admitted only as a declared split's combiner.
///
/// Both directions are driven from the same coverage shape, so the difference
/// between them is exactly the declaration: without it the program has a
/// dispatch it cannot account for, and with it the dispatch is the final pass
/// of a split whose partial pass already claims the reduction.
#[test]
fn an_uncovering_stage_is_admitted_only_as_a_declared_splits_combiner() {
    let semantic = serial_sum_program(SCALE_BITS);

    let undeclared = complete_two_stage(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    assert_eq!(
        undeclared
            .build()
            .map_err(|error| *error.diagnostics().first().expect("one diagnostic")),
        Err(KernelProgramDiagnostic::UncoveringStage)
    );

    let wired = two_stage(&semantic, TwoStageShape::UncoveringSecondStage);
    let split = split_over(&wired);
    let mut declared = wire_two_stage_structure(wired);
    declared
        .push_partial_reduction(split)
        .expect("a well-formed split declaration");
    declare_program_contract(&mut declared);
    let program = declared.build().expect("verified program");
    assert!(
        program.stages().any(|stage| stage.coverage().is_empty()),
        "the combiner is retained as the uncovering stage the split accounts for"
    );
}
