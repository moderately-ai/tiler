//! Bounded tests for the target-neutral kernel-program IR.
//!
//! Fixtures bind real verified structured kernels to real verified semantic
//! programs. Coverage assignments are structural partitions: this layer proves
//! that every operation of the bound graph is covered exactly once, never that
//! a given kernel computes the operations its stage claims.

use crate::index::{
    FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexDomainProofBudget,
    IndexRealizationAuthority, IndexRefinementSubject, IndexRefinementVerificationError,
    IndexRefinementVerificationOutcome, MAX_FINITE_DOMAIN_PROOF_CELLS, NumericalContractIdentity,
    ResolvedIndexRealization,
};
use crate::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, ApproximationEnvelope, Bf16NumericalContractKey,
    BoundsProof, BoundsProofKind, BoundsWitnessId, ContractionAxisSource, ContributorOrder,
    ExceptionalValueAssumption, ExecutionBinding, F32NumericalContractKey, FlushedZeroSign,
    KernelSchedule, LaunchPlan, LogicalAccess, MaterializationRounding, NumericalPermission,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    PointwiseBf16ExpressionBuilder, PointwiseF32ExpressionBuilder, ReductionTopology, RegionId,
    RegionProgram, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
    VerifiedScheduledRegion,
};
use crate::semantic::{
    Bf16, Bf16Add, Bf16Constant, Bf16Multiply, EncodedComponentRole, F32, F32Add, F32Constant,
    F32Multiply, InputKey, OperationAttributes, OutputKey, STRICT_AFFINE_CODES_ROLE,
    STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE, SemanticProgram,
    SemanticProgramBuilder, StrictAffineU4, StrictSerialF32Sum, dequantize_strict_affine_op,
};
use crate::shape::{Axis, Shape};

use super::abi::{
    AbiBinaryOp, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue, AvailabilityPhase, ExprNode,
    TargetPropertyKey, evaluate,
};
use super::{
    AbiExprId, AlignmentGuarantee, AlignmentRequirement, AllocationId, AllocationOwnership,
    AllocationSpec, ByteWindow, CoveredOccurrence, KernelProgramBuildError, KernelProgramBuilder,
    KernelProgramDiagnostic, MAX_PROGRAM_ABI_EXPRESSIONS, MaterializedComponentSpec,
    MaterializedOrigin, MaterializedValueId, MaterializedValueSpec, MemorySpace, PartialReduction,
    ProgramAbiUse, ProgramEntityKind, PublishingCopy, RoutingCommitState, RoutingCommitTransition,
    SemanticOccurrence, StageAccess, StageAccessMode, StageId, StageLaunch, StagedRealization,
    StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram, ViewId,
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
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
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
    let input = expression.input(AccessOrdinal::FIRST).expect("input");
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
            tensor: TensorRole::Input,
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
            tensor: TensorRole::Input,
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(scale_bias_expression(scale_bits)),
            numerical: strict(),
        })
        .expect("scalar program");
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
                empty_identity_bits: 0,
            },
            numerical: strict(),
        })
        .expect("scalar program");
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
            tensor: TensorRole::Input,
            component_role: Some(STRICT_AFFINE_CODES_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::PackedU4LsbZeroTail { logical_elements },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Input,
            component_role: Some(STRICT_AFFINE_SCALE_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::ScalarBroadcast,
            bounds: BoundsWitnessId::new(1),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Input,
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
            TensorRole::Input,
            Some(STRICT_AFFINE_CODES_ROLE),
            logical_elements.div_ceil(2),
        ),
        (1, TensorRole::Input, Some(STRICT_AFFINE_SCALE_ROLE), 1),
        (2, TensorRole::Input, Some(STRICT_AFFINE_ZERO_POINT_ROLE), 1),
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictAffineU4Dequantize {
                codes_role: STRICT_AFFINE_CODES_ROLE,
                scale_role: STRICT_AFFINE_SCALE_ROLE,
                zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            },
            numerical: strict(),
        })
        .expect("scalar program");
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
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
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
        alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
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
/// `v9`, `v10`, and `v11` are a different kind of step and are included here for
/// the same reason: `v9` *adds* framed refinement evidence inside the stage
/// section, `v10` *adds* a publishing-copy declaration section, and `v11` *adds*
/// a staged-realization declaration section, so the historical spellings below
/// are not merely reinterpretations of the current payload — they are shorter
/// encodings this test cannot reconstruct. What the loop still proves is the
/// property that matters at every step: no historical separator over these bytes
/// is the current identity.
///
/// The separators are not all the same length, so the spliced spelling is
/// compared by inequality alone where the lengths differ. Padding it back to a
/// common length would compare a byte string no encoder produces.
#[test]
fn the_program_domain_separator_is_what_distinguishes_the_reinterpreting_steps() {
    const V6: &[u8] = b"tiler.kernel-program.v6\0";
    const V7: &[u8] = b"tiler.kernel-program.v7\0";
    const V8: &[u8] = b"tiler.kernel-program.v8\0";
    const V9: &[u8] = b"tiler.kernel-program.v9\0";
    const V10: &[u8] = b"tiler.kernel-program.v10\0";
    const V11: &[u8] = b"tiler.kernel-program.v11\0";
    const V12: &[u8] = b"tiler.kernel-program.v12\0";
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);
    // One record: the v8 encoding and the v7 sort agree on this payload.
    assert_eq!(program.outputs().len(), 1);
    let current = program.canonical_identity().as_bytes();
    assert!(current.starts_with(V12));

    for historical in [V6, V7, V8, V9, V10, V11] {
        let mut spelling = historical.to_vec();
        spelling.extend_from_slice(&current[V12.len()..]);
        assert_ne!(current, spelling.as_slice());
    }
    // The check can say no about the separator rather than about the length: the
    // current separator over the current payload *is* the current identity.
    let mut rebuilt = V12.to_vec();
    rebuilt.extend_from_slice(&current[V11.len()..]);
    assert_eq!(current, rebuilt.as_slice());
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
/// **The population is two, and it says which change moved it there.** A key
/// appears once inside the folded semantic graph identity, which has encoded
/// outputs in declaration order all along, and once in the program's own output
/// section. It used to appear a third time per coverage record, because every
/// record restated the whole bound graph identity; ADR 0104 replaced that
/// restatement with a fixed-width digest, and a digest of a key is not the key.
/// The coverage population is asserted non-empty beside the count for the reason
/// the count used to be derived from it — a program that covered nothing would
/// otherwise satisfy a literal `2` while proving nothing at all.
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
    assert!(
        coverage_records > 0,
        "a program covering no occurrence proves nothing about what coverage restates",
    );
    let expected = 2;
    assert_eq!(
        declared_first.len(),
        expected,
        "the semantic fold and the output section, and no per-record graph restatement"
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
            expected: TensorRole::Input,
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
    first_reduce: StageId,
    first_temporary: MaterializedValueId,
    second_map: StageId,
    second_reduce: StageId,
    first_output: MaterializedValueId,
    second_output: MaterializedValueId,
    second_temporary: MaterializedValueId,
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
        first_reduce,
        first_temporary: storage.first_temporary,
        second_map,
        second_reduce,
        first_output: storage.first_output,
        second_output: storage.second_output,
        second_temporary: storage.second_temporary,
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
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
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
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
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
        alignment: AlignmentRequirement::natural_for(StorageScalar::U8),
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
            alignment: AlignmentGuarantee::natural_for(StorageScalar::U8),
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
                alignment: AlignmentRequirement::natural_for(StorageScalar::U8),
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

    let mut component = |role, shape, storage_scalar, element_type, encoding, bytes| {
        let allocation = builder
            .push_allocation(AllocationSpec {
                capacity_bytes: bytes,
                alignment: AlignmentGuarantee::natural_for(storage_scalar),
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
                    alignment: AlignmentRequirement::natural_for(storage_scalar),
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
    );
    let scale = component(
        STRICT_AFFINE_SCALE_ROLE,
        Shape::new([]),
        StorageScalar::F32,
        KernelType::F32,
        StorageEncoding::Unpacked,
        4,
    );
    let zero_point = component(
        STRICT_AFFINE_ZERO_POINT_ROLE,
        Shape::new([]),
        StorageScalar::U8,
        KernelType::U8,
        StorageEncoding::Unpacked,
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
    // Identity writes the reached arena once and names each use by canonical
    // position, so a node no use reaches would be retained program state omitted
    // by that traversal.
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
        occurrence: SemanticOccurrence::new(1),
        partitions: 3,
        contributors_per_partition: 1,
    }
}

fn program_with_split(
    semantic: &SemanticProgram,
    amend: impl FnOnce(&TwoStage, PartialReduction) -> PartialReduction,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let wired = two_stage(semantic, TwoStageShape::UncoveringSecondStage);
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

/// Retains the assembled owner subject so these tests can perturb the graph the
/// owner derivation reads, rather than weakening an assertion around a verified
/// program that no longer contains the malformed shape.
fn owner_data_with_split(
    semantic: &SemanticProgram,
    amend: impl FnOnce(&TwoStage, PartialReduction) -> PartialReduction,
) -> super::model::KernelProgramData {
    let wired = two_stage(semantic, TwoStageShape::UncoveringSecondStage);
    let split = amend(&wired, split_over(&wired));
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_partial_reduction(split)
        .expect("the base split declaration is locally well formed");
    declare_program_contract(&mut builder);
    builder.into_data_for_owner_test()
}

fn owner_refusal(data: &super::model::KernelProgramData) -> KernelProgramDiagnostic {
    match super::verify::derive_stage_owners(data) {
        Ok(_) => panic!("the perturbed owner graph unexpectedly derived an owner"),
        Err(diagnostic) => diagnostic,
    }
}

#[test]
fn complete_stage_owner_refusals_reach_their_exact_graph_branches() {
    use super::model::{PublishingCopyData, StagedRealizationData};

    let semantic = serial_sum_program(SCALE_BITS);

    let mut missing_builder =
        wire_two_stage_structure(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    declare_program_contract(&mut missing_builder);
    assert_eq!(
        owner_refusal(&missing_builder.into_data_for_owner_test()),
        KernelProgramDiagnostic::MissingStageOwner,
        "the uncovered combiner has no owner when its continuation declaration is absent",
    );

    let mut foreign = owner_data_with_split(&semantic, |_, split| PartialReduction {
        occurrence: SemanticOccurrence::new(4),
        ..split
    });
    foreign.stages[0]
        .coverage
        .retain(|covered| covered.occurrence().get() != 4);
    assert_eq!(
        owner_refusal(&foreign),
        KernelProgramDiagnostic::ForeignStageOwnerProof,
        "changing the split subject to an occurrence no stage covers must not invent a root",
    );

    let mut fork = owner_data_with_split(&semantic, |_, split| split);
    fork.partial_reductions.push(fork.partial_reductions[0]);
    assert_eq!(
        owner_refusal(&fork),
        KernelProgramDiagnostic::DuplicateStageOwnerOrdinal,
        "two continuation edges from one root are a fork, not two owners for ordinal one",
    );

    let mut looped = owner_data_with_split(&semantic, |_, split| split);
    let split = looped.partial_reductions[0];
    looped.staged_realizations.push(StagedRealizationData {
        producer: split.combiner,
        consumer: split.producer,
        handed: split.partial,
        occurrence: split.occurrence,
    });
    assert_eq!(
        owner_refusal(&looped),
        KernelProgramDiagnostic::DuplicateStageOwnerOrdinal,
        "a continuation that revisits a reached stage is a loop, not a new ordinal",
    );

    let mut merged = owner_data_with_split(&semantic, |_, split| split);
    merged.stages.push(merged.stages[1].clone());
    let split = merged.partial_reductions[0];
    merged.staged_realizations.push(StagedRealizationData {
        producer: 2,
        consumer: split.combiner,
        handed: split.partial,
        occurrence: split.occurrence,
    });
    assert_eq!(
        owner_refusal(&merged),
        KernelProgramDiagnostic::SkippedStageOwnerOrdinal,
        "a second incoming continuation is a merge only through an edge detached from the root path",
    );

    let mut disconnected_builder =
        wire_two_stage_structure(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    declare_program_contract(&mut disconnected_builder);
    let mut disconnected = disconnected_builder.into_data_for_owner_test();
    disconnected
        .staged_realizations
        .push(StagedRealizationData {
            producer: 1,
            consumer: 0,
            handed: 1,
            occurrence: 1,
        });
    assert_eq!(
        owner_refusal(&disconnected),
        KernelProgramDiagnostic::SkippedStageOwnerOrdinal,
        "an edge not reachable from its proof-bound root is disconnected",
    );

    let mut publication = complete_two_stage(two_stage(&semantic, TwoStageShape::Canonical))
        .into_data_for_owner_test();
    publication.publishing_copies.push(PublishingCopyData {
        source_stage: 0,
        publisher: 1,
        source: 1,
        published: 1,
    });
    assert_eq!(
        owner_refusal(&publication),
        KernelProgramDiagnostic::MissingPublicationOwner,
        "a copy whose published value has no named output cannot claim publication ownership",
    );

    let mut mixed = complete_two_stage(two_stage(&semantic, TwoStageShape::Canonical))
        .into_data_for_owner_test();
    mixed.publishing_copies.push(PublishingCopyData {
        source_stage: 0,
        publisher: 1,
        source: 1,
        published: 2,
    });
    assert_eq!(
        owner_refusal(&mixed),
        KernelProgramDiagnostic::AmbiguousStageOwner,
        "a computing stage cannot also be the administrative publisher",
    );
}

#[test]
fn complete_stage_owner_identity_changes_only_for_admitted_owner_claims() {
    use super::model::{
        PublicationStageClaim, RealizationStageClaim, StageOwner, encoded_stage_owner_for_test,
    };

    let semantic = serial_sum_program(SCALE_BITS);
    let strict = checked_coverage(&semantic, &strict_contract());
    let flushed = checked_coverage(&semantic, &flush_contract());
    let bytes = |covered: CoveredOccurrence, ordinal| {
        encoded_stage_owner_for_test(&StageOwner::Realization(vec![RealizationStageClaim {
            covered,
            ordinal,
        }]))
    };
    let baseline = bytes(strict[0].clone(), 1);
    assert_ne!(
        baseline,
        bytes(strict[1].clone(), 1),
        "changing the proof-bound occurrence changes the complete owner subject",
    );
    assert_ne!(
        baseline,
        bytes(flushed[0].clone(), 1),
        "changing the reached refinement proof changes the complete owner subject",
    );
    assert_ne!(
        baseline,
        bytes(strict[0].clone(), 2),
        "changing only the continuation ordinal changes the complete owner subject",
    );
    assert_eq!(
        baseline,
        bytes(strict[0].clone(), 1),
        "the owner encoder has no downstream value, allocation, dependency, or builder-order input to distinguish",
    );

    // This is deliberately the crate-private owner projection rather than a
    // purported verified publisher. Existing fixtures can construct only a
    // plain-output copy, so the public artifact control proves `None` framing;
    // this encoder-level subject probe proves a nonempty component role is not
    // silently omitted when a future verified producer makes it reachable.
    let publication = |key, component_role| {
        encoded_stage_owner_for_test(&StageOwner::Publication(vec![PublicationStageClaim {
            key: OutputKey::new(key).expect("output key"),
            component_role,
        }]))
    };
    let publication_baseline = publication("published", None);
    assert_ne!(
        publication_baseline,
        publication("renamed-publication", None),
        "changing the exact publication key changes the owner subject",
    );
    assert_ne!(
        publication_baseline,
        publication("published", Some(EncodedComponentRole::new(99))),
        "changing the publication component role from None to a concrete role changes the owner subject",
    );
    assert_eq!(
        publication_baseline,
        publication("published", None),
        "the publication owner encoder has no producer, downstream value, allocation, dependency, or builder-order input",
    );
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
    // Contributor coverage is an independently declared split fact, so changing
    // it must move identity alongside the exact occurrence and owner claims.
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
        occurrence: SemanticOccurrence::new(1),
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

/// Declares one publishing copy over the two-stage fixture and builds.
fn program_with_copy(
    semantic: &SemanticProgram,
    amend: impl FnOnce(&TwoStage, PublishingCopy) -> PublishingCopy,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let wired = two_stage(semantic, TwoStageShape::UncoveringSecondStage);
    let copy = amend(
        &wired,
        PublishingCopy {
            source_stage: wired.pointwise,
            publisher: wired.reduction,
            source: wired.temporary,
            published: wired.output,
        },
    );
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_publishing_copy(copy)
        .expect("a well-formed copy declaration");
    declare_program_contract(&mut builder);
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// An uncovering stage is admitted by a declared copy, and refused without one.
///
/// **The two directions differ by exactly the declaration.** The undeclared
/// program has a dispatch it cannot account for; the declared one has the
/// publisher of a copy whose source stage already claims every occurrence. That
/// is the same shape a split's final pass has, one fold up, and it is why the
/// arm is a second *account* rather than a relaxation of the rule.
///
/// **Measurement boundary, and it bounds two claims rather than one.** This
/// drives the coverage arm alone: no fixture in this module can state a copy
/// whose obligations *all* hold, because a copy publishes what it read and every
/// fixture here writes its output at a reduced extent — the two-stage
/// temporary is `[2, 3]` against a `[2]` output, and both chains of the
/// two-chain fixture are the same shape. So the declared program below is
/// structurally a copy in every respect but its extents, and it is refused by
/// the extent obligation rather than admitted.
///
/// The complete admitting path is exercised end to end by `tiler-compiler`'s
/// `pipeline::conformance::a_published_and_consumed_intermediate_compiles_and_agrees`,
/// which asserts the declared copy, the single uncovering stage, and bit
/// agreement for both published outputs. The identity claim is bounded the same
/// way: that the declaration section is folded is evidenced by the domain step
/// this change carries and by
/// [`the_program_domain_separator_is_what_distinguishes_the_reinterpreting_steps`],
/// while *injectivity against an otherwise identical program* rests on the
/// section being length-framed and written unconditionally, and has no fixture
/// here that could state the pair. Building one would need a third kernel
/// writing an output at the temporary's extent, which would re-state the
/// compiler's evidence rather than add any.
#[test]
fn an_undeclared_uncovering_stage_still_refuses_by_name() {
    let semantic = serial_sum_program(SCALE_BITS);
    let undeclared = complete_two_stage(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    assert_eq!(
        undeclared
            .build()
            .map_err(|error| *error.diagnostics().first().expect("one diagnostic")),
        Err(KernelProgramDiagnostic::UncoveringStage)
    );
    // With the declaration, the coverage arm no longer fires: the program now
    // fails on the copy's own extent obligation, which is a later phase and a
    // different rule.
    assert_eq!(
        program_with_copy(&semantic, |_, copy| copy),
        Err(KernelProgramDiagnostic::PublishedCopyExtentMismatch)
    );
}

/// Declares one publishing copy over the two-chain fixture and builds.
fn two_chain_copy(
    semantic: &SemanticProgram,
    state: impl FnOnce(&TwoChain) -> PublishingCopy,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let chains = two_chain(semantic, true);
    let copy = state(&chains);
    let mut builder = publish_two_chain(chains);
    builder
        .push_publishing_copy(copy)
        .expect("a well-formed copy declaration");
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// Each publishing-copy obligation is driven against a case that must fail.
///
/// **Two fixtures, and which rows each carries is forced by their shapes rather
/// than chosen.** The two-stage fixture has an uncovering second stage, so its
/// publisher must be that stage for the coverage arm not to fire first — which
/// fixes what its rows can perturb. The two-chain fixture has four stages and
/// two independently published outputs, which is what a row naming *another*
/// stage's value needs. Every row differs from a well-formed declaration by
/// exactly one named entity.
#[test]
fn the_publishing_copy_obligations_can_each_say_no() {
    let semantic = serial_sum_program(SCALE_BITS);

    // The named source is written by the publisher rather than by the named
    // source stage, so the publisher would copy values that stage never
    // produced.
    assert_eq!(
        program_with_copy(&semantic, |wired, copy| PublishingCopy {
            source: wired.output,
            ..copy
        }),
        Err(KernelProgramDiagnostic::CopiedSourceNotInitializedBySourceStage)
    );

    // The published value is an internal temporary. A declaration naming one has
    // nothing to publish whichever stage wrote it, which is why the role is
    // checked before the writer.
    assert_eq!(
        program_with_copy(&semantic, |wired, copy| PublishingCopy {
            published: wired.temporary,
            ..copy
        }),
        Err(KernelProgramDiagnostic::PublishedCopyNotOutput)
    );

    let chained = two_chain_program();

    // The publisher never reads the value it claims to copy: the first chain's
    // temporary is defined by the first chain's map stage and read only by the
    // first chain's reduction.
    assert_eq!(
        two_chain_copy(&chained, |chains| PublishingCopy {
            source_stage: chains.first_map,
            publisher: chains.second_reduce,
            source: chains.first_temporary,
            published: chains.second_output,
        }),
        Err(KernelProgramDiagnostic::CopiedSourceNotReadByPublisher)
    );

    // The published value is a genuine output written by a *different* stage.
    assert_eq!(
        two_chain_copy(&chained, |chains| PublishingCopy {
            source_stage: chains.second_map,
            publisher: chains.second_reduce,
            source: chains.second_temporary,
            published: chains.first_output,
        }),
        Err(KernelProgramDiagnostic::PublishedCopyNotWrittenByPublisher)
    );
}

/// One stage cannot be both halves, and one value cannot be published twice.
#[test]
fn a_malformed_publishing_copy_declaration_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::UncoveringSecondStage);
    let copy = PublishingCopy {
        source_stage: wired.pointwise,
        publisher: wired.reduction,
        source: wired.temporary,
        published: wired.output,
    };
    let mut builder = wire_two_stage_structure(wired);
    assert_eq!(
        builder.push_publishing_copy(PublishingCopy {
            publisher: copy.source_stage,
            ..copy
        }),
        Err(KernelProgramBuildError::SelfDependency)
    );
    builder
        .push_publishing_copy(copy)
        .expect("the first declaration is well formed");
    assert_eq!(
        builder.push_publishing_copy(PublishingCopy {
            source: copy.published,
            ..copy
        }),
        Err(KernelProgramBuildError::DuplicatePublishingCopy)
    );
}

/// Declares one staged realization over the two-stage fixture and builds.
///
/// The fixture's second stage covers nothing, which is the coverage shape a
/// staged realization's consumer has: the stage that *began* the realization
/// claims the occurrence, and claiming it again would double-cover the graph.
fn program_with_staged_realization(
    semantic: &SemanticProgram,
    amend: impl FnOnce(&TwoStage, StagedRealization) -> StagedRealization,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let wired = two_stage(semantic, TwoStageShape::UncoveringSecondStage);
    let realization = amend(
        &wired,
        StagedRealization {
            producer: wired.pointwise,
            consumer: wired.reduction,
            handed: wired.temporary,
            occurrence: SemanticOccurrence::new(4),
        },
    );
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_staged_realization(realization)
        .expect("a well-formed staged declaration");
    declare_program_contract(&mut builder);
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// Declares one staged realization over the two-chain fixture and builds.
fn two_chain_staged(
    semantic: &SemanticProgram,
    state: impl FnOnce(&TwoChain) -> StagedRealization,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let chains = two_chain(semantic, true);
    let realization = state(&chains);
    let mut builder = publish_two_chain(chains);
    builder
        .push_staged_realization(realization)
        .expect("a well-formed staged declaration");
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// An uncovering stage is admitted by a declared staged realization.
///
/// **The two directions differ by exactly the declaration.** The undeclared
/// program has a dispatch it cannot account for; the declared one has the
/// consumer of a realization whose producer already claims the occurrence they
/// jointly compute. That is the third account beside a split's final pass and a
/// copy's publisher, and it is a second *account* rather than a relaxation:
/// nothing here weakens the rule that a dispatch computing no operation must be
/// explained.
///
/// **It is also the one of the three whose admitting path completes on this
/// fixture.** A publishing copy's does not — every fixture in this module writes
/// its output at a reduced extent, and a copy publishes what it read — while a
/// staged realization deliberately carries no extent obligation, because a
/// realization's later stage iterates its own domain. That asymmetry is the
/// declaration's whole reason for existing rather than a gap in the fixture.
///
/// The check that can say no is the declaration itself: dropping the
/// `push_staged_realization` call in [`program_with_staged_realization`] returns
/// the first assertion's `UncoveringStage`.
#[test]
fn an_uncovering_stage_is_admitted_as_a_declared_staged_realizations_consumer() {
    let semantic = serial_sum_program(SCALE_BITS);

    let undeclared = complete_two_stage(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    assert_eq!(
        undeclared
            .build()
            .map_err(|error| *error.diagnostics().first().expect("one diagnostic")),
        Err(KernelProgramDiagnostic::UncoveringStage)
    );

    let program = program_with_staged_realization(&semantic, |_, realization| realization)
        .expect("the declared staged realization is admitted");
    let declared: Vec<_> = program.staged_realizations().collect();
    assert_eq!(declared.len(), 1);
    assert_eq!(declared[0].occurrence(), SemanticOccurrence::new(4));
    assert!(declared[0].consumer().coverage().is_empty());
    assert!(
        declared[0]
            .producer()
            .coverage()
            .iter()
            .any(|covered| covered.occurrence() == SemanticOccurrence::new(4)),
        "the chain is rooted at the stage that covers the occurrence it continues"
    );
}

/// The declaration is folded, and an otherwise identical program differs by it.
///
/// **Two programs alike in every stage, value, view, allocation, and edge.** The
/// only difference is whether the second stage is *declared* to continue the
/// first's realization, and identity says so — which is the property the domain
/// step exists for, and the one no comparison of the entities already folded
/// could have recovered.
///
/// The pair is stated over the canonical fixture, whose second stage covers an
/// occurrence of its own, rather than over the uncovering one: the declaration
/// is optional there and both sides verify, which is what makes the declaration
/// the only variable. Over the uncovering fixture the undeclared side cannot be
/// built at all, because `UncoveringStage` refuses it.
#[test]
fn a_declared_staged_realization_changes_program_identity() {
    let semantic = serial_sum_program(SCALE_BITS);
    let bare = canonical_program(&semantic);

    let wired = two_stage(&semantic, TwoStageShape::Canonical);
    let realization = StagedRealization {
        producer: wired.pointwise,
        consumer: wired.reduction,
        handed: wired.temporary,
        occurrence: SemanticOccurrence::new(0),
    };
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_staged_realization(realization)
        .expect("a well-formed staged declaration");
    declare_program_contract(&mut builder);
    let declared = builder.build().expect("the declared program verifies");

    assert_eq!(bare.stages().len(), declared.stages().len());
    assert_eq!(bare.values().len(), declared.values().len());
    assert_eq!(bare.dependencies().len(), declared.dependencies().len());
    assert_eq!(bare.staged_realizations().len(), 0);
    assert_eq!(declared.staged_realizations().len(), 1);
    assert_ne!(bare.canonical_identity(), declared.canonical_identity());
}

/// Each staged-realization row obligation is driven against a case that fails.
///
/// **Two fixtures, and which rows each carries is forced by their shapes.** The
/// two-stage fixture has an uncovering second stage, so its consumer must be
/// that stage for the coverage arm not to fire first. The two-chain fixture has
/// four stages, each covering a disjoint range, which is what a row naming
/// another chain's stage needs. Every row differs from a well-formed declaration
/// by exactly one named entity.
///
/// **One obligation is deliberately not driven here, and it is unreachable
/// rather than untested.** `HandedValueNotMaterialized` needs a handed value
/// that is neither a temporary nor an externally bound input, and no program can
/// present one: an input is refused a writer by `ExternalValueWritten` two
/// phases earlier, and `ValueRole::Output` fills only `TensorRole::Output`,
/// which is a write — so no stage can *read* an output-role value and the read
/// obligation above it always fires first. It is stated for the reason
/// `PartialNotMaterialized` is: the declaration owes the obligation whether or
/// not today's role vocabulary can spell a violation of it.
#[test]
fn the_staged_realization_row_obligations_can_each_say_no() {
    let semantic = serial_sum_program(SCALE_BITS);

    // The handed value is written by the consumer rather than by the named
    // producer, so the consumer would continue from values that stage never
    // produced.
    assert_eq!(
        program_with_staged_realization(&semantic, |wired, realization| StagedRealization {
            handed: wired.output,
            ..realization
        }),
        Err(KernelProgramDiagnostic::HandedValueNotInitializedByProducer)
    );

    let chained = two_chain_program();

    // The consumer never reads the value it claims to continue from: the first
    // chain's temporary is defined by the first chain's map stage and read only
    // by the first chain's reduction.
    assert_eq!(
        two_chain_staged(&chained, |chains| StagedRealization {
            producer: chains.first_map,
            consumer: chains.second_reduce,
            handed: chains.first_temporary,
            occurrence: SemanticOccurrence::new(0),
        }),
        Err(KernelProgramDiagnostic::HandedValueNotReadByConsumer)
    );
}

/// The chain must run from the stage that covers the occurrence it continues.
///
/// **The obligation no single declaration can see, and the one the row checks
/// above cannot reach.** Every named entity is right in both rows below — the
/// handed value's definer is the producer, the consumer reads it, and it is a
/// temporary — and both programs are still refused, because the occurrence each
/// claims to continue was begun by a stage its chain never runs from. A
/// realization's stages run in order and each runs once; a chain rooted
/// elsewhere describes later dispatches computing a stage nobody began.
///
/// The positive control is stated first and is what makes the two refusals
/// about the *root* rather than about the fixture: the identical declaration
/// naming an occurrence its producer covers verifies.
#[test]
fn a_staged_realization_chain_must_start_where_its_occurrence_is_covered() {
    let chained = two_chain_program();

    // The control: occurrence 0 is covered by `first_map`, which is this
    // declaration's producer, so the walk reaches its one row.
    two_chain_staged(&chained, |chains| StagedRealization {
        producer: chains.first_map,
        consumer: chains.first_reduce,
        handed: chains.first_temporary,
        occurrence: SemanticOccurrence::new(0),
    })
    .expect("a chain rooted at the covering stage verifies");

    // Occurrence 4 is the first chain's reduction, covered by this declaration's
    // *consumer*. The walk starts there, finds no continuation, and the one
    // declared row lies on no path.
    assert_eq!(
        two_chain_staged(&chained, |chains| StagedRealization {
            producer: chains.first_map,
            consumer: chains.first_reduce,
            handed: chains.first_temporary,
            occurrence: SemanticOccurrence::new(4),
        }),
        Err(KernelProgramDiagnostic::StagedRealizationChainBroken)
    );

    // Occurrence 7 is the second chain's reduction, covered by a stage in the
    // other chain entirely — so the walk starts somewhere this declaration's
    // two stages never appear.
    assert_eq!(
        two_chain_staged(&chained, |chains| StagedRealization {
            producer: chains.first_map,
            consumer: chains.first_reduce,
            handed: chains.first_temporary,
            occurrence: SemanticOccurrence::new(7),
        }),
        Err(KernelProgramDiagnostic::StagedRealizationChainBroken)
    );
}

/// One stage cannot be both halves, and one consumer cannot continue one
/// occurrence twice.
#[test]
fn a_malformed_staged_realization_declaration_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::UncoveringSecondStage);
    let realization = StagedRealization {
        producer: wired.pointwise,
        consumer: wired.reduction,
        handed: wired.temporary,
        occurrence: SemanticOccurrence::new(4),
    };
    let other_value = wired.output;
    let mut builder = wire_two_stage_structure(wired);
    assert_eq!(
        builder.push_staged_realization(StagedRealization {
            consumer: realization.producer,
            ..realization
        }),
        Err(KernelProgramBuildError::SelfDependency)
    );
    assert_eq!(
        builder.push_staged_realization(StagedRealization {
            occurrence: SemanticOccurrence::new(5),
            ..realization
        }),
        Err(KernelProgramBuildError::CoverageOutOfRange {
            occurrence: SemanticOccurrence::new(5),
            operations: 5,
        }),
        "the fixture graph has five operations, so ordinal five names none of them"
    );
    builder
        .push_staged_realization(realization)
        .expect("the first declaration is well formed");
    // A second declaration by the same consumer naming a *different* occurrence
    // is admitted: one fused dispatch may continue several realizations, and the
    // key is the pair rather than the stage.
    builder
        .push_staged_realization(StagedRealization {
            occurrence: SemanticOccurrence::new(3),
            ..realization
        })
        .expect("one consumer may continue two occurrences");
    // The same pair a second time has no reading: two handed values for one
    // stage boundary leave which one carries the realization undecided.
    assert_eq!(
        builder.push_staged_realization(StagedRealization {
            handed: other_value,
            ..realization
        }),
        Err(KernelProgramBuildError::DuplicateStagedRealization)
    );
}

// ---------------------------------------------------------------------------
// The pure-BF16 producer path.
//
// Every layer below the program was already implemented and tested for `bf16`,
// and the composition was unreachable: a `bf16` occurrence could not obtain
// executable coverage, so no `bf16` kernel program verified. These fixtures walk
// the same sealed path the `f32` ones do — no shortcut mints a receipt — so what
// they demonstrate is that the refinement layer now admits the width, not that a
// test can assert it does.
// ---------------------------------------------------------------------------

const BF16_SCALE_BITS: u16 = 0x4000; // 2.0bf16
const BF16_BIAS_BITS: u16 = 0x3f80; // 1.0bf16

/// The strict `bf16` contract, the direct sibling of [`strict_contract`].
fn strict_bf16_contract() -> NumericalContractIdentity {
    Bf16NumericalContractKey::new(
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
        MaterializationRounding::NearestTiesToEven,
    )
    .expect("the fixture bf16 contract vector is coherent")
    .into()
}

/// A four-operation pure-BF16 graph: `result = input * 2.0 + 1.0`.
///
/// Constant, multiply, and add are the complete registered `bf16` vocabulary, so
/// this is the widest pure-`bf16` program the semantic layer can state — not a
/// subset chosen to be easy.
fn bf16_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let input = draft
        .input::<Bf16>(InputKey::new("input").expect("key"), input_shape())
        .expect("input");
    let scale = Bf16Constant::apply(&mut draft, BF16_SCALE_BITS).expect("scale");
    let bias = Bf16Constant::apply(&mut draft, BF16_BIAS_BITS).expect("bias");
    let product = Bf16Multiply::apply(&mut draft, input, scale).expect("product");
    let mapped = Bf16Add::apply(&mut draft, product, bias).expect("mapped");
    draft
        .output(OutputKey::new("result").expect("key"), mapped)
        .expect("output");
    let program = draft.build().expect("verified bf16 semantic program");
    assert_eq!(program.operation_count(), 4);
    program
}

fn bf16_numerical() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-bf16",
        u32::from(crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS),
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
    )
}

/// `(x * 2.0) + 1.0` in `bf16`, writing the program output directly.
fn bf16_output_region() -> VerifiedScheduledRegion {
    let shape = input_shape();
    let count = elements(&shape);
    let mut expression = PointwiseBf16ExpressionBuilder::new();
    let leaf = expression.input(AccessOrdinal::FIRST).expect("input");
    let scale = expression.constant(BF16_SCALE_BITS).expect("scale");
    let product = expression.multiply(leaf, scale).expect("product");
    let bias = expression.constant(BF16_BIAS_BITS).expect("bias");
    let root = expression.add(product, bias).expect("sum");
    let expression = expression.build(root).expect("bf16 pointwise expression");

    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder.iteration_shape(shape).expect("iteration shape");
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
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
    for (witness, tensor) in [(0, TensorRole::Input), (1, TensorRole::Output)] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: count,
                },
            })
            .expect("bounds proof");
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: count,
            },
        })
        .expect("ownership proof");
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseBf16(expression),
            numerical: bf16_numerical(),
        })
        .expect("scalar program");
    builder
        .schedule(linear_schedule(count, OwnershipWitnessId::new(0)))
        .expect("schedule");
    builder.build().expect("verified bf16 region")
}

fn bf16_value(origin: MaterializedOrigin, role: ValueRole) -> MaterializedValueSpec {
    MaterializedValueSpec {
        origin,
        role,
        shape: input_shape(),
        storage_scalar: StorageScalar::Bf16,
        encoding: StorageEncoding::Unpacked,
        element_type: KernelType::Bf16,
        alignment: AlignmentRequirement::natural_for(StorageScalar::Bf16),
        memory_space: MemorySpace::Device,
    }
}

/// A pure-BF16 program obtains verified coverage for every occurrence and
/// reaches a verified kernel program over a `PointwiseBf16` region.
///
/// This is the composition the refinement layer previously made unreachable.
/// Every one of the four coverage records is minted by the verifier through
/// [`checked_coverage`], the same helper the `f32` fixtures use, so a record here
/// is refinement evidence rather than a fixture assertion — and the program
/// builds, which is what proves no stage covers nothing.
#[test]
fn a_pure_bf16_program_covers_every_occurrence_and_builds_a_verified_kernel_program() {
    let semantic = bf16_program();
    let coverage = checked_coverage(&semantic, &strict_bf16_contract());
    assert_eq!(
        coverage.len(),
        4,
        "every bf16 occurrence obtains executable coverage"
    );
    assert_eq!(
        coverage
            .iter()
            .map(|covered| covered.occurrence().get())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3],
        "the coverage partition is the graph's complete canonical occurrence run"
    );

    let kernel = lower_scheduled_region(&bf16_output_region()).expect("bf16 kernel");
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    // Only the quantities this one stage names: six `bf16` elements are twelve
    // bytes on both sides. Minting the reduction extents the `f32` fixtures
    // share would leave an ABI expression no stage references, which the
    // program verifier refuses by name.
    let value_bytes = literal(&mut builder, 12);
    let grid_threads = literal(&mut builder, 6);
    let threads_per_workgroup = literal(&mut builder, 1);
    let external = builder
        .push_allocation(device(12, AllocationOwnership::External))
        .expect("external allocation");
    let produced = builder
        .push_allocation(device(12, AllocationOwnership::Program))
        .expect("output allocation");
    let source = builder
        .push_value(
            bf16_value(program_input("input"), ValueRole::Input),
            external,
        )
        .expect("input value");
    let output = builder
        .push_value(
            bf16_value(MaterializedOrigin::Internal, ValueRole::Output),
            produced,
        )
        .expect("output value");
    let source_view = builder.push_whole_view(source).expect("input view");
    let output_view = builder.push_whole_view(output).expect("output view");
    builder
        .push_stage(
            &kernel,
            &coverage,
            &[
                read(source_view, value_bytes),
                write(output_view, value_bytes),
            ],
            StageLaunch {
                grid_threads,
                threads_per_workgroup,
            },
        )
        .expect("the bf16 stage covers every occurrence of its bound graph");
    builder
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("named output");
    declare_program_contract(&mut builder);
    let program = builder
        .build()
        .map_err(|error| error.diagnostics().to_vec())
        .expect("verified bf16 kernel program");
    assert_eq!(program.stages().count(), 1);
}

/// A candidate region that does not realize its occurrence is refused.
///
/// The rubber-stamp perturbation. The candidate handed to the verifier is a
/// *real* verified region minted by a *different* occurrence's law — the add's
/// region offered as the multiply's realization — so it passes every structural
/// check and fails only the one that matters: the law derives its own expected
/// region and compares canonical identities. A verifier that consulted the
/// caller's region instead of deriving would admit this.
#[test]
fn a_bf16_candidate_that_does_not_realize_its_occurrence_is_refused() {
    let semantic = bf16_program();
    let contract = strict_bf16_contract();
    let registry = semantic.semantic_registry().clone();
    let scalars = FrozenScalarRegistry::standard().expect("scalar authority");
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(registry.clone(), scalars.clone())
        .expect("the standard authorities cohere");

    let subject_for = |ordinal: usize| {
        let operation = semantic
            .operations()
            .nth(ordinal)
            .expect("the fixture has four operations");
        IndexRefinementSubject::derive(&semantic, operation.id(), contract.clone())
            .expect("every bf16 occurrence derives a subject")
    };
    let region_for = |subject: &IndexRefinementSubject| {
        registry
            .index_realization_law(subject.operation())
            .expect("every bf16 occurrence has a registered law")
            .law
            .clone()
            .realize(subject, &scalars)
            .expect("the registered law realizes its own subject")
    };

    let multiply = subject_for(2);
    let add = subject_for(3);
    assert_ne!(
        multiply.operation(),
        add.operation(),
        "two distinct families"
    );
    let honest = region_for(&multiply);
    let foreign = region_for(&add);
    assert_ne!(
        honest.canonical_identity(),
        foreign.canonical_identity(),
        "the perturbation is a genuinely different region"
    );

    let verify_against = |candidate: &crate::index::VerifiedIndexRegion| {
        let reached = scalars
            .revalidate_region(candidate)
            .expect("both candidates are themselves well-formed");
        let authority = IndexRealizationAuthority::admit(
            &registry,
            &scalars,
            multiply.operation().clone(),
            multiply.signature().clone(),
            reached.reached_operations(),
        )
        .expect("the authority admits the candidate's reached ceiling");
        laws.resolve(&multiply)
            .expect("the multiply resolves its own law")
            .verify(&authority, candidate)
    };

    // The positive control. Without it a refusal below would be consistent with
    // the fixture being broken in some way that has nothing to do with the
    // perturbation, and the test would assert nothing about the verifier.
    assert!(
        matches!(
            verify_against(&honest).expect("the multiply's own region verifies"),
            IndexRefinementVerificationOutcome::Verified(_)
        ),
        "the honest candidate must verify, or the refusal below proves nothing"
    );

    let error = verify_against(&foreign)
        .expect_err("a region realizing another occurrence must be refused");
    assert!(
        matches!(
            error,
            IndexRefinementVerificationError::SemanticRealizationMismatch { .. }
        ),
        "expected SemanticRealizationMismatch, got {error:?}"
    );
}

/// Neither width's program verifies under the other width's contract.
///
/// Both directions, because they fail for the same reason and a check that only
/// ran one way would not establish it: the law derives the arithmetic its result
/// is produced in from the verified subject, and a contract stated for another
/// width governs another format's subnormals, rounding, and canonical NaN. The
/// refusal is named rather than a generic mismatch, so a reader is told which of
/// the verifier's obligations the pair failed.
#[test]
fn a_program_under_the_other_widths_contract_is_refused_by_name() {
    let cases = [
        (
            "a bf16 program under an f32 contract",
            bf16_program(),
            strict_contract(),
            strict_bf16_contract(),
        ),
        (
            "an f32 program under a bf16 contract",
            serial_sum_program(SCALE_BITS),
            strict_bf16_contract(),
            strict_contract(),
        ),
    ];
    for (case, semantic, foreign_contract, native_contract) in cases {
        let registry = semantic.semantic_registry().clone();
        let scalars = FrozenScalarRegistry::standard().expect("scalar authority");
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(registry.clone(), scalars.clone())
                .expect("the standard authorities cohere");
        let operation = semantic.operations().next().expect("a first operation");
        let outcome_under = |contract: NumericalContractIdentity| {
            let subject = IndexRefinementSubject::derive(&semantic, operation.id(), contract)
                .expect("a subject derives under any validated contract identity");
            let region = registry
                .index_realization_law(subject.operation())
                .expect("the fixture's first operation has a registered law")
                .law
                .clone()
                .realize(&subject, &scalars)
                .expect("the law realizes a region from types, not from the contract");
            let reached = scalars
                .revalidate_region(&region)
                .expect("the law's own region revalidates");
            let authority = IndexRealizationAuthority::admit(
                &registry,
                &scalars,
                subject.operation().clone(),
                subject.signature().clone(),
                reached.reached_operations(),
            )
            .expect("the authority admits the region's reached ceiling");
            laws.resolve(&subject)
                .expect("resolution does not consult the contract")
                .verify(&authority, &region)
        };

        // The positive control: the identical setup under the program's own
        // width verifies. Without it, a refusal would be consistent with the
        // fixture never having been verifiable at all, and the contract would
        // not be shown to be the thing that decided it.
        assert!(
            matches!(
                outcome_under(native_contract).expect("the native contract governs"),
                IndexRefinementVerificationOutcome::Verified(_)
            ),
            "{case}: the native contract must verify, or the refusal proves nothing"
        );

        let error =
            outcome_under(foreign_contract).expect_err("the cross-width contract must be refused");
        assert!(
            matches!(
                error,
                IndexRefinementVerificationError::NumericalContractNotGoverned
            ),
            "{case}: expected NumericalContractNotGoverned, got {error:?}"
        );
    }
}

/// The `bf16` rows did not disturb the `f32` refinement evidence beside them.
///
/// The load-bearing property of this whole step. A refinement receipt's
/// *executable coverage* is what reaches kernel-program and artifact identity,
/// and it restates only reached-only projections — never the whole scalar or
/// law-registry snapshots that the three new rows moved. Pinning the f32
/// coverage bytes against a value computed from the graph itself would move with
/// whatever moved them, so this compares the two widths' coverage for the one
/// property that must hold: the f32 records are unchanged in content while the
/// registries beneath them are not.
#[test]
fn the_bf16_rows_leave_f32_executable_coverage_untouched() {
    let semantic = serial_sum_program(SCALE_BITS);
    let coverage = checked_coverage(&semantic, &strict_contract());
    assert_eq!(coverage.len(), 5);

    // The scalar authority beneath them did move: it now defines the three bf16
    // per-point operations. If executable coverage folded that snapshot, the
    // records above could not be stable across it.
    let scalars = FrozenScalarRegistry::standard().expect("scalar authority");
    for key in [
        crate::index::constant_bf16_scalar_op(),
        crate::index::multiply_bf16_scalar_op(),
        crate::index::add_bf16_scalar_op(),
    ] {
        assert!(
            scalars.definition(&key).is_some(),
            "{key:?} is registered in the same snapshot the f32 coverage was minted under"
        );
    }

    // And the f32 records reached none of them.
    let reached_bf16 = coverage.iter().any(|covered| {
        let bytes = covered.refinement().as_bytes();
        bytes
            .windows(b"constant-bf16".len())
            .any(|window| window == b"constant-bf16")
    });
    assert!(
        !reached_bf16,
        "an f32 occurrence's reached-only coverage names no bf16 scalar"
    );
}

/// A two-stage serial-sum whose temporary is larger than the working set so a
/// partial window can start at a chosen byte offset.
fn push_partial_temporary_stage(
    offset: u64,
) -> Result<(KernelProgramBuilder, ViewId), KernelProgramBuildError> {
    let semantic = serial_sum_program(SCALE_BITS);
    let coverage = checked_coverage(&semantic, &strict_contract());
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let source_allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 24,
            alignment: AlignmentGuarantee::new(16).expect("16 is a power of two"),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("input allocation");
    let temporary_allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 32,
            alignment: AlignmentGuarantee::new(16).expect("16 is a power of two"),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let source = builder
        .push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            source_allocation,
        )
        .expect("input value");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                Shape::from_dims([8]),
            ),
            temporary_allocation,
        )
        .expect("oversized temporary");
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
    let temporary_view = builder.push_view(temporary, ByteWindow { offset, length: 24 })?;
    let _output_view = builder.push_whole_view(output).expect("output view");
    builder.push_stage(
        &pointwise_kernel(0, SCALE_BITS),
        &coverage_range(&coverage, 0..4),
        &[
            read(source_view, abi.input_bytes),
            write(temporary_view, abi.input_bytes),
        ],
        abi.pointwise_launch(),
    )?;
    Ok((builder, temporary_view))
}

fn complete_partial_temporary_program(offset: u64) -> VerifiedKernelProgram {
    let semantic = serial_sum_program(SCALE_BITS);
    let coverage = checked_coverage(&semantic, &strict_contract());
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let source_allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 24,
            alignment: AlignmentGuarantee::new(16).expect("16 is a power of two"),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("input allocation");
    let temporary_allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 32,
            alignment: AlignmentGuarantee::new(16).expect("16 is a power of two"),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let source = builder
        .push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            source_allocation,
        )
        .expect("input value");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                Shape::from_dims([8]),
            ),
            temporary_allocation,
        )
        .expect("oversized temporary");
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
    let temporary_view = builder
        .push_view(temporary, ByteWindow { offset, length: 24 })
        .expect("partial temporary view");
    let output_view = builder.push_whole_view(output).expect("output view");
    let pointwise = builder
        .push_stage(
            &pointwise_kernel(0, SCALE_BITS),
            &coverage_range(&coverage, 0..4),
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
            &coverage_range(&coverage, 4..5),
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
    builder.build().expect("verified partial-view program")
}

#[test]
fn a_naturally_aligned_partial_f32_view_builds() {
    let program = complete_partial_temporary_program(4);
    let temporary = program
        .views()
        .find(|view| view.window().offset == 4)
        .expect("the partial temporary view");
    assert_eq!(temporary.alignment().bytes(), 4);
    assert!(
        temporary
            .alignment()
            .satisfies(AlignmentRequirement::natural_for(StorageScalar::F32))
    );
}

#[test]
fn a_one_byte_shifted_f32_view_fails_before_the_stage_is_verified() {
    let error = push_partial_temporary_stage(1)
        .expect_err("a one-byte-shifted F32 view must not reach artifact construction");
    assert_eq!(
        error,
        KernelProgramBuildError::StageAccessAlignment {
            position: 1,
            required: AlignmentRequirement::natural_for(StorageScalar::F32),
            guaranteed: AlignmentGuarantee::new(1).expect("1 is a power of two"),
        }
    );
}

/// The schedule whose strict fold seeds contributor zero before looping over
/// contributors `1..S`.
fn live_contraction_kernel_for_program() -> VerifiedKernel {
    let left = Shape::from_dims([2]);
    let right = Shape::from_dims([3]);
    let output = Shape::from_dims([2, 3]);
    let contracted = Shape::from_dims([]);
    let output_elements = elements(&output);
    let owner = OwnershipWitnessId::new(0);
    let mut region = ScheduledRegionBuilder::new(RegionId::new(40));
    region.iteration_shape(output.clone()).expect("shape");
    for (ordinal, (operand, free)) in [(&left, 0_u32), (&right, 1)].into_iter().enumerate() {
        let witness = u32::try_from(ordinal).expect("two operands");
        let tensor = TensorRole::Input;
        region
            .push_access(Access {
                tensor,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ContractionOperand {
                    operand_shape: operand.clone(),
                    output_shape: output.clone(),
                    contracted_shape: contracted.clone(),
                    sources: vec![ContractionAxisSource::Output { position: free }],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .expect("operand access");
        region
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 0 },
            })
            .expect("live bounds");
    }
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(owner),
        })
        .expect("output access");
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .expect("output bounds");
    region
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .expect("ownership");
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
            },
            numerical: strict(),
        })
        .expect("strict contraction");
    region
        .schedule(KernelSchedule {
            reduction: ReductionTopology::LiveContraction {
                live_access: AccessOrdinal::FIRST,
                live_axis: Axis::new(1),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements, owner)
        })
        .expect("schedule");
    lower_scheduled_region(&region.build().expect("verified schedule"))
        .expect("lowered contraction")
}

/// A two-input graph whose one occurrence supplies real refinement evidence.
///
/// Program coverage is structural rather than a second semantic equivalence
/// proof, so this deliberately small graph keeps the test focused on the stage
/// binding and ABI guard added by the live-contraction kernel.
fn live_contraction_semantic(shape: Shape) -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("registry");
    let left = draft
        .input::<F32>(InputKey::new("left").expect("key"), shape.clone())
        .expect("left");
    let right = draft
        .input::<F32>(InputKey::new("right").expect("key"), shape)
        .expect("right");
    let result = F32Add::apply(&mut draft, left, right).expect("add");
    draft
        .output(OutputKey::new("result").expect("key"), result)
        .expect("output");
    draft.build().expect("semantic")
}

/// One live-contraction stage before its guard, coverage, and named output are
/// committed to the program builder.
///
/// Keeping these pieces separately lets the limit tests exercise the
/// transactional `push_stage` boundary without forging a kernel or reaching
/// through the builder's private state.
struct LiveContractionProgramDraft {
    builder: KernelProgramBuilder,
    kernel: VerifiedKernel,
    coverage: Vec<CoveredOccurrence>,
    accesses: [StageAccess; 3],
    launch: StageLaunch,
    output: MaterializedValueId,
}

fn live_contraction_program_draft(
    semantic: &SemanticProgram,
) -> Result<LiveContractionProgramDraft, KernelProgramBuildError> {
    let kernel = live_contraction_kernel_for_program();
    let mut builder = KernelProgramBuilder::new(semantic)?;
    let shape = semantic
        .shape(semantic.inputs().next().expect("left input").value())
        .expect("input shape")
        .as_static()
        .expect("static fixture")
        .clone();
    let bytes = elements(&shape) * 4;
    let left_allocation = builder.push_allocation(device(bytes, AllocationOwnership::External))?;
    let right_allocation = builder.push_allocation(device(bytes, AllocationOwnership::External))?;
    let output_allocation = builder.push_allocation(device(bytes, AllocationOwnership::Program))?;
    let left = builder.push_value(
        value(program_input("left"), ValueRole::Input, shape.clone()),
        left_allocation,
    )?;
    let right = builder.push_value(
        value(program_input("right"), ValueRole::Input, shape.clone()),
        right_allocation,
    )?;
    let output = builder.push_value(
        value(MaterializedOrigin::Internal, ValueRole::Output, shape),
        output_allocation,
    )?;
    let left_view = builder.push_view(
        left,
        ByteWindow {
            offset: 0,
            length: 0,
        },
    )?;
    let right_view = builder.push_view(
        right,
        ByteWindow {
            offset: 0,
            length: 0,
        },
    )?;
    let output_view = builder.push_whole_view(output)?;
    let zero = literal(&mut builder, 0);
    let six = literal(&mut builder, 6);
    let one = literal(&mut builder, 1);
    let output_bytes = literal(&mut builder, bytes);
    Ok(LiveContractionProgramDraft {
        builder,
        kernel,
        coverage: checked_coverage(semantic, &strict_contract()),
        accesses: [
            read(left_view, zero),
            read(right_view, zero),
            write(output_view, output_bytes),
        ],
        launch: StageLaunch {
            grid_threads: six,
            threads_per_workgroup: one,
        },
        output,
    })
}

fn push_live_contraction_stage(
    draft: &mut LiveContractionProgramDraft,
) -> Result<StageId, KernelProgramBuildError> {
    draft.builder.push_stage(
        &draft.kernel,
        &draft.coverage,
        &draft.accesses,
        draft.launch,
    )
}

fn publish_live_contraction_output(
    draft: &mut LiveContractionProgramDraft,
) -> Result<(), KernelProgramBuildError> {
    draft
        .builder
        .push_output(OutputKey::new("result").expect("key"), draft.output)
}

fn live_contraction_program_builder(
    semantic: &SemanticProgram,
) -> Result<KernelProgramBuilder, KernelProgramBuildError> {
    let mut draft = live_contraction_program_draft(semantic)?;
    declare_program_contract(&mut draft.builder);
    push_live_contraction_stage(&mut draft)?;
    publish_live_contraction_output(&mut draft)?;
    Ok(draft.builder)
}

#[test]
fn a_live_contraction_derives_the_same_input_extent_guard_and_refuses_zero() {
    let semantic = live_contraction_semantic(Shape::from_dims([2, 3]));
    let program = live_contraction_program_builder(&semantic)
        .expect("stage binding")
        .build()
        .expect("verified program");
    let key = InputKey::new("left").expect("key");
    let evaluate_guard = |extent| {
        evaluate(
            program.abi_expressions(),
            program.applicability_guard(),
            &AbiFacts::new(
                AvailabilityPhase::LiveDevicePreflight,
                vec![(key.clone(), Axis::new(1), extent)],
                Vec::new(),
            ),
        )
        .expect("guard evaluates")
    };
    assert_eq!(evaluate_guard(0), AbiValue::Boolean(false));
    assert_eq!(evaluate_guard(1), AbiValue::Boolean(true));
    assert_eq!(evaluate_guard(14), AbiValue::Boolean(true));
    assert_eq!(evaluate_guard(15), AbiValue::Boolean(true));
}

#[test]
fn a_live_contraction_conjoins_its_requirement_with_a_nontrivial_authored_guard() {
    let semantic = live_contraction_semantic(Shape::from_dims([2, 3]));
    let mut draft = live_contraction_program_draft(&semantic).expect("draft");
    let two = literal(&mut draft.builder, 2);
    let right_axis_zero = draft
        .builder
        .push_abi_root(AbiRoot::InputExtent {
            key: InputKey::new("right").expect("key"),
            axis: Axis::new(0),
        })
        .expect("right extent root");
    let authored = draft
        .builder
        .push_abi_binary(AbiBinaryOp::LessOrEqual, two, right_axis_zero)
        .expect("2 <= right axis 0");
    draft
        .builder
        .applicability_guard(authored)
        .expect("authored guard");
    declare_routing_commit(&mut draft.builder);
    push_live_contraction_stage(&mut draft).expect("stage");
    publish_live_contraction_output(&mut draft).expect("output");
    let program = draft.builder.build().expect("verified program");

    let left = InputKey::new("left").expect("key");
    let right = InputKey::new("right").expect("key");
    let evaluate_guard = |left_axis_one, right_axis_zero| {
        evaluate(
            program.abi_expressions(),
            program.applicability_guard(),
            &AbiFacts::new(
                AvailabilityPhase::LiveDevicePreflight,
                vec![
                    (left.clone(), Axis::new(1), left_axis_one),
                    (right.clone(), Axis::new(0), right_axis_zero),
                ],
                Vec::new(),
            ),
        )
        .expect("guard evaluates")
    };
    for (left_axis_one, right_axis_zero, expected) in
        [(0, 1, false), (0, 2, false), (1, 1, false), (1, 2, true)]
    {
        assert_eq!(
            evaluate_guard(left_axis_one, right_axis_zero),
            AbiValue::Boolean(expected),
            "left axis 1 = {left_axis_one}, right axis 0 = {right_axis_zero}"
        );
    }
}

/// Grows the authored guard so the live requirement reaches the exact ABI-node
/// limit after `push_stage` reserves and `build` materializes it.
fn saturated_live_contraction_draft(
    semantic: &SemanticProgram,
    growth_levels: usize,
) -> LiveContractionProgramDraft {
    let mut draft = live_contraction_program_draft(semantic).expect("draft");
    let authored = grown_guard(&mut draft.builder, AbiGrowth::SharedDag, growth_levels);
    draft
        .builder
        .applicability_guard(authored)
        .expect("authored guard");
    declare_routing_commit(&mut draft.builder);
    draft
}

#[test]
fn an_exact_limit_failed_build_recovers_and_retries_to_the_same_identity_as_a_twin() {
    // Four authored size/launch roots, one authored boolean root, 4,088 shared
    // DAG levels, and three derived nodes (extent, predicate, final `And`) are
    // exactly 4,096. The derived guard reuses the authored literal one.
    const GROWTH_LEVELS: usize = MAX_PROGRAM_ABI_EXPRESSIONS - 8;

    let semantic = live_contraction_semantic(Shape::from_dims([2, 3]));
    let mut interrupted = saturated_live_contraction_draft(&semantic, GROWTH_LEVELS);
    push_live_contraction_stage(&mut interrupted).expect("exact-limit stage");
    let output = interrupted.output;
    let error = interrupted
        .builder
        .build()
        .expect_err("the deliberately omitted named output fails verification");
    assert_eq!(
        error.diagnostics(),
        &[KernelProgramDiagnostic::EmptyProgram]
    );
    let (mut recovered, diagnostics) = error.into_parts();
    assert_eq!(diagnostics, vec![KernelProgramDiagnostic::EmptyProgram]);
    recovered
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("repair the recovered builder");
    let retried = recovered.build().expect("recovered exact-limit program");

    let mut twin = saturated_live_contraction_draft(&semantic, GROWTH_LEVELS);
    push_live_contraction_stage(&mut twin).expect("twin stage");
    publish_live_contraction_output(&mut twin).expect("twin output");
    let twin = twin.builder.build().expect("fresh exact-limit twin");

    assert_eq!(retried.abi_expressions().len(), MAX_PROGRAM_ABI_EXPRESSIONS);
    assert_eq!(twin.abi_expressions().len(), MAX_PROGRAM_ABI_EXPRESSIONS);
    assert_eq!(retried.abi_expressions(), twin.abi_expressions());
    assert_eq!(retried.applicability_guard(), twin.applicability_guard());
    assert_eq!(retried.canonical_identity(), twin.canonical_identity());
    assert_eq!(retried, twin);
}

#[test]
fn a_stage_whose_derived_guard_would_be_node_4097_refuses_transactionally() {
    const GROWTH_LEVELS: usize = MAX_PROGRAM_ABI_EXPRESSIONS - 7;

    let semantic = live_contraction_semantic(Shape::from_dims([2, 3]));
    let mut draft = saturated_live_contraction_draft(&semantic, GROWTH_LEVELS);
    let expected = KernelProgramBuildError::StructuralLimit {
        resource: super::ProgramLimitKind::AbiExpressions,
        actual: MAX_PROGRAM_ABI_EXPRESSIONS + 1,
        limit: MAX_PROGRAM_ABI_EXPRESSIONS,
    };
    assert_eq!(
        push_live_contraction_stage(&mut draft).expect_err("node 4,097 must refuse"),
        expected
    );
    assert_eq!(
        push_live_contraction_stage(&mut draft)
            .expect_err("the failed insertion must not retain coverage or a requirement"),
        expected
    );
    draft
        .builder
        .push_abi_root(AbiRoot::BooleanLiteral(false))
        .expect("the failed stage left the authored 4,094-node arena intact");
}

/// Two independent operations over the same pair of program inputs.
///
/// Structural coverage is the program layer's authority, so the two stages may
/// bind the same verified kernel while carrying distinct refinement receipts
/// and distinct output storage. That makes this a direct probe of requirement
/// deduplication and ordering rather than a second semantic-equivalence test.
fn two_live_contraction_semantic() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("registry");
    let shape = Shape::from_dims([2, 3]);
    let left = draft
        .input::<F32>(InputKey::new("left").expect("key"), shape.clone())
        .expect("left");
    let right = draft
        .input::<F32>(InputKey::new("right").expect("key"), shape)
        .expect("right");
    let first = F32Add::apply(&mut draft, left, right).expect("first operation");
    let second = F32Multiply::apply(&mut draft, left, right).expect("second operation");
    draft
        .output(OutputKey::new("first").expect("key"), first)
        .expect("first output");
    draft
        .output(OutputKey::new("second").expect("key"), second)
        .expect("second output");
    draft.build().expect("semantic")
}

/// Builds two independent live-contraction stages.
///
/// The first always binds kernel input zero to `left`. When
/// `distinct_requirements` is true, the second swaps its two input bindings so
/// its live requirement resolves to `right`. `reverse_insertion` changes only
/// builder insertion order; canonical stage keys still order identity.
fn two_live_contraction_stage_program(
    semantic: &SemanticProgram,
    distinct_requirements: bool,
    reverse_insertion: bool,
) -> VerifiedKernelProgram {
    let kernel = live_contraction_kernel_for_program();
    let coverage = checked_coverage(semantic, &strict_contract());
    let mut builder = KernelProgramBuilder::new(semantic).expect("builder");
    let bytes = 24;
    let left_allocation = builder
        .push_allocation(device(bytes, AllocationOwnership::External))
        .expect("left allocation");
    let right_allocation = builder
        .push_allocation(device(bytes, AllocationOwnership::External))
        .expect("right allocation");
    let first_allocation = builder
        .push_allocation(device(bytes, AllocationOwnership::Program))
        .expect("first output allocation");
    let second_allocation = builder
        .push_allocation(device(bytes, AllocationOwnership::Program))
        .expect("second output allocation");
    let shape = Shape::from_dims([2, 3]);
    let left = builder
        .push_value(
            value(program_input("left"), ValueRole::Input, shape.clone()),
            left_allocation,
        )
        .expect("left value");
    let right = builder
        .push_value(
            value(program_input("right"), ValueRole::Input, shape.clone()),
            right_allocation,
        )
        .expect("right value");
    let first = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                shape.clone(),
            ),
            first_allocation,
        )
        .expect("first output value");
    let second = builder
        .push_value(
            value(MaterializedOrigin::Internal, ValueRole::Output, shape),
            second_allocation,
        )
        .expect("second output value");
    let left_view = builder
        .push_view(
            left,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .expect("left live view");
    let right_view = builder
        .push_view(
            right,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .expect("right live view");
    let first_view = builder.push_whole_view(first).expect("first output view");
    let second_view = builder.push_whole_view(second).expect("second output view");
    let zero = literal(&mut builder, 0);
    let six = literal(&mut builder, 6);
    let one = literal(&mut builder, 1);
    let output_bytes = literal(&mut builder, bytes);
    declare_program_contract(&mut builder);
    let first_accesses = [
        read(left_view, zero),
        read(right_view, zero),
        write(first_view, output_bytes),
    ];
    let second_accesses = if distinct_requirements {
        [
            read(right_view, zero),
            read(left_view, zero),
            write(second_view, output_bytes),
        ]
    } else {
        [
            read(left_view, zero),
            read(right_view, zero),
            write(second_view, output_bytes),
        ]
    };
    let launch = StageLaunch {
        grid_threads: six,
        threads_per_workgroup: one,
    };
    let push = |builder: &mut KernelProgramBuilder,
                covered: &[CoveredOccurrence],
                accesses: &[StageAccess]| {
        builder
            .push_stage(&kernel, covered, accesses, launch)
            .expect("independent live-contraction stage");
    };
    if reverse_insertion {
        push(
            &mut builder,
            &coverage_range(&coverage, 1..2),
            &second_accesses,
        );
        push(
            &mut builder,
            &coverage_range(&coverage, 0..1),
            &first_accesses,
        );
    } else {
        push(
            &mut builder,
            &coverage_range(&coverage, 0..1),
            &first_accesses,
        );
        push(
            &mut builder,
            &coverage_range(&coverage, 1..2),
            &second_accesses,
        );
    }
    builder
        .push_output(OutputKey::new("first").expect("key"), first)
        .expect("first output");
    builder
        .push_output(OutputKey::new("second").expect("key"), second)
        .expect("second output");
    builder.build().expect("verified two-stage program")
}

#[test]
fn two_live_stages_requiring_the_same_input_extent_share_one_guard_fact() {
    let semantic = two_live_contraction_semantic();
    let program = two_live_contraction_stage_program(&semantic, false, false);
    let left = InputKey::new("left").expect("key");
    let roots: Vec<_> = program
        .abi_expressions()
        .iter()
        .filter_map(|node| match node {
            ExprNode::Root(AbiRoot::InputExtent { key, axis }) => Some((key, *axis)),
            _ => None,
        })
        .collect();
    assert_eq!(roots, vec![(&left, Axis::new(1))]);
    assert_eq!(program.abi_expressions().len(), 8);
    for (extent, expected) in [(0, false), (1, true)] {
        assert_eq!(
            evaluate(
                program.abi_expressions(),
                program.applicability_guard(),
                &AbiFacts::new(
                    AvailabilityPhase::LiveDevicePreflight,
                    vec![(left.clone(), Axis::new(1), extent)],
                    Vec::new(),
                ),
            ),
            Ok(AbiValue::Boolean(expected))
        );
    }
}

#[test]
fn two_distinct_required_extents_have_one_guard_and_identity_in_either_stage_order() {
    let semantic = two_live_contraction_semantic();
    let forward = two_live_contraction_stage_program(&semantic, true, false);
    let reversed = two_live_contraction_stage_program(&semantic, true, true);

    assert_eq!(forward.abi_expressions(), reversed.abi_expressions());
    assert_eq!(
        forward.applicability_guard(),
        reversed.applicability_guard()
    );
    assert_eq!(forward.canonical_identity(), reversed.canonical_identity());
    assert_eq!(forward, reversed);

    let left = InputKey::new("left").expect("key");
    let right = InputKey::new("right").expect("key");
    for (left_extent, right_extent, expected) in
        [(0, 0, false), (0, 1, false), (1, 0, false), (1, 1, true)]
    {
        let facts = AbiFacts::new(
            AvailabilityPhase::LiveDevicePreflight,
            vec![
                (left.clone(), Axis::new(1), left_extent),
                (right.clone(), Axis::new(1), right_extent),
            ],
            Vec::new(),
        );
        assert_eq!(
            evaluate(
                forward.abi_expressions(),
                forward.applicability_guard(),
                &facts,
            ),
            Ok(AbiValue::Boolean(expected))
        );
        assert_eq!(
            evaluate(
                reversed.abi_expressions(),
                reversed.applicability_guard(),
                &facts,
            ),
            Ok(AbiValue::Boolean(expected))
        );
    }
}

#[test]
fn a_live_contraction_requirement_without_one_logical_owner_fails_typed() {
    let semantic = live_contraction_semantic(Shape::from_dims([6]));
    let error = live_contraction_program_builder(&semantic)
        .expect_err("axis 1 cannot resolve against rank-one program inputs");
    assert_eq!(
        error,
        KernelProgramBuildError::RequiredInputExtentBinding {
            tensor: TensorRole::Input,
            axis: Axis::new(1),
            matches: 0,
        }
    );
}

// -------------------------------------------------------------------------
// The ADR 0013 plan-determinism witness
// -------------------------------------------------------------------------

/// The witness covers the whole canonical program and projects its identities.
///
/// The positive control every refusal case below leans on, with its population
/// counted from the program: two stages, two scheduled-region identities, one
/// kernel-program identity — each read back through the witness's own
/// accessors and compared against the program's, so the witness cannot claim a
/// program other than the one it borrowed.
#[test]
fn the_plan_determinism_witness_covers_the_canonical_program() {
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);
    let witness = crate::kernel::verify_plan_determinism(&program)
        .expect("the strict canonical program is plan deterministic");
    assert_eq!(
        witness.kernel_program_identity().as_bytes(),
        program.canonical_identity().as_bytes(),
        "the witness projects exactly the program it proves",
    );
    let regions: Vec<_> = witness
        .scheduled_region_identities()
        .map(|identity| identity.as_bytes().to_vec())
        .collect();
    assert_eq!(
        regions.len(),
        program.stages().len(),
        "one scheduled-region identity per stage",
    );
    assert_eq!(regions.len(), 2, "the canonical program has two stages");
    for (stage, identity) in program.stages().zip(&regions) {
        assert_eq!(
            stage.kernel().scheduled_region_identity().as_bytes(),
            identity.as_slice(),
            "the topology binding is each stage's own scheduled-region identity",
        );
    }
}

/// Builds the pointwise kernel under a permutation-permitted realization.
///
/// One numerical field moves — the contributor-permutation permission — and
/// nothing else: the region's accesses, proofs, expression, and schedule are
/// the canonical pointwise fixture's own bytes.
fn permutation_permitted_pointwise_kernel(region: u32) -> VerifiedKernel {
    let mut raw = pointwise_region(region, SCALE_BITS).region().clone();
    match &mut raw.index.program {
        RegionProgram::Numerical { numerical, .. } => {
            *numerical = NumericalRealization::new(
                "tiler.test.permutation-permitted-f32",
                CANONICAL_NAN,
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Permitted,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
            );
        }
        RegionProgram::PartitionedCopy { .. } => {
            panic!("the pointwise fixture declares a numerical program")
        }
    }
    let region = ScheduledRegionBuilder::from_region(raw)
        .build()
        .expect("the permutation-permitted region verifies");
    lower_scheduled_region(&region).expect("the permissive kernel lowers")
}

/// Granting permutation must not yield a plan-deterministic witness.
///
/// The accepted arrival perturbation, on its reachable spelling: the current
/// builders refuse `NondeterministicArrival`, `AtomicAccumulation`, and
/// `SynchronizationKind::Atomic` by name before a verified schedule exists, so
/// the freedom those spellings consume — the contributor-permutation
/// permission — is the arrival subject a verified program can still carry.
/// The witness refuses it by name, at the exact stage, because nothing in the
/// program proves the granted freedom went unused; accepting it would let a
/// later admitted unfixed-arrival construct arrive already holding a witness.
#[test]
fn a_permutation_permitted_stage_is_refused_as_unfixed_arrival_by_name() {
    let semantic = serial_sum_program(SCALE_BITS);
    // The same program shape as the positive control above; only the pointwise
    // stage's permutation permission moves.
    let program = complete_two_stage(wire_two_stage(
        &semantic,
        &permutation_permitted_pointwise_kernel(0),
        &reduction_kernel(1),
        TwoStageShape::Canonical,
    ))
    .build()
    .expect("the permissive program verifies");
    let refusal = crate::kernel::verify_plan_determinism(&program)
        .expect_err("a granted arrival freedom must not inherit plan determinism");
    assert_eq!(
        refusal,
        crate::kernel::PlanDeterminismRefusal::UnfixedContributorArrival { stage: 0 },
        "the refusal names the exact stage and class",
    );
    assert_eq!(
        refusal.to_string(),
        "plan-determinism.unfixed-contributor-arrival: stage 0's declared realization permits \
         contributor permutation, so its arrival order is not fixed by canonical program bytes",
    );
}
