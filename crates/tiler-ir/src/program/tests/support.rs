//! Fixtures shared by more than one sibling test module.
//!
//! Reused across subjects: the two-stage and two-chain program rigs, the shared
//! ABI/routing-commit contract helpers, and the coverage/refinement plumbing every
//! subject's fixtures are built from. A fixture used by only one subject module lives
//! there instead — see `tests/mod.rs` for the mapping rule.

use super::super::abi::{AbiBinaryOp, AbiRoot};
use super::super::{
    AbiExprId, AlignmentGuarantee, AlignmentRequirement, AllocationId, AllocationOwnership,
    AllocationSpec, CoveredOccurrence, KernelProgramBuilder, KernelProgramDiagnostic,
    MaterializedOrigin, MaterializedValueId, MaterializedValueSpec, MemorySpace,
    RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode, StageId,
    StageLaunch, StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram, ViewId,
};
use crate::index::{
    FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexDomainProofBudget,
    IndexRealizationAuthority, IndexRefinementSubject, IndexRefinementVerificationOutcome,
    MAX_FINITE_DOMAIN_PROOF_CELLS, NumericalContractIdentity, ResolvedIndexRealization,
};
use crate::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, ApproximationEnvelope, BoundsProof, BoundsProofKind,
    BoundsWitnessId, ContributorOrder, ExceptionalValueAssumption, ExecutionBinding,
    F32NumericalContractKey, FlushedZeroSign, KernelSchedule, LaunchPlan, LogicalAccess,
    MaterializationRounding, NumericalPermission, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseF32ExpressionBuilder, ReductionTopology,
    RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy,
    TensorRole, VerifiedScheduledRegion,
};
use crate::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use crate::shape::{Axis, Shape};

pub(super) const SCALE_BITS: u32 = 0x4000_0000; // 2.0f32

pub(super) const OTHER_SCALE_BITS: u32 = 0x4040_0000; // 3.0f32

pub(super) const BIAS_BITS: u32 = 0x3f80_0000; // 1.0f32

pub(super) const CANONICAL_NAN: u32 = 0x7fc0_0000;

pub(super) fn strict() -> NumericalRealization {
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

pub(super) fn linear_schedule(work_items: u64, owner: OwnershipWitnessId) -> KernelSchedule {
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

pub(super) fn elements(shape: &Shape) -> u64 {
    crate::schedule::element_count(shape).expect("test shapes do not overflow")
}

pub(super) fn input_shape() -> Shape {
    Shape::from_dims([2, 3])
}

pub(super) fn output_shape() -> Shape {
    Shape::from_dims([2])
}

pub(super) fn scale_bias_expression(scale_bits: u32) -> crate::schedule::PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).expect("input");
    let scale = expression.constant(scale_bits).expect("scale");
    let product = expression.multiply(input, scale).expect("product");
    let bias = expression.constant(BIAS_BITS).expect("bias");
    let root = expression.add(product, bias).expect("sum");
    expression.build(root).expect("pointwise expression")
}

/// Builds the canonical pointwise region: one program input to one temporary.
pub(super) fn pointwise_region(region: u32, scale_bits: u32) -> VerifiedScheduledRegion {
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
pub(super) fn reduction_region(region: u32) -> VerifiedScheduledRegion {
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

pub(super) fn pointwise_kernel(region: u32, scale_bits: u32) -> VerifiedKernel {
    lower_scheduled_region(&pointwise_region(region, scale_bits)).expect("pointwise kernel")
}

pub(super) fn reduction_kernel(region: u32) -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(region)).expect("reduction kernel")
}

/// A five-operation graph: `result = strict_serial_sum(input * scale + 1.0, 1)`.
pub(super) fn serial_sum_program(scale_bits: u32) -> SemanticProgram {
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
pub(super) fn two_chain_program() -> SemanticProgram {
    two_chain_program_keyed(["sum_a", "sum_b"])
}

/// The two-chain graph publishing its two reductions under the given keys.
///
/// The keys are a parameter because the interface order and the order the
/// superseded sorted encoding produced coincide for `sum_a`/`sum_b` and differ
/// for a reverse-lexicographic pair, and an ordering rule can only be told from
/// a content sort by a program where the two disagree.
pub(super) fn two_chain_program_keyed(keys: [&str; 2]) -> SemanticProgram {
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

/// The governed strict F32 contract every fixture kernel realizes.
pub(super) fn strict_contract() -> NumericalContractIdentity {
    f32_contract(SubnormalMode::Preserve)
}

/// The same contract flushing subnormals, used only to perturb *evidence*.
///
/// A numerical contract is folded into a refinement receipt's executable
/// coverage and is not part of semantic graph meaning, so two coverages minted
/// under these two contracts name the same occurrences of the same graph and
/// carry different proofs. That is the exact perturbation the identity tests
/// need, and there is no honest way to fabricate it.
pub(super) fn flush_contract() -> NumericalContractIdentity {
    f32_contract(SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    })
}

pub(super) fn f32_contract(subnormals: SubnormalMode) -> NumericalContractIdentity {
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
pub(super) fn checked_coverage(
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

pub(super) fn proof_budget() -> IndexDomainProofBudget {
    IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 1 << 20)
        .expect("the fixture proof budget is within IR's hard bounds")
}

/// Selects the coverage records for one canonical occurrence range.
pub(super) fn occurrences(
    semantic: &SemanticProgram,
    range: std::ops::Range<u32>,
) -> Vec<CoveredOccurrence> {
    coverage_range(&checked_coverage(semantic, &strict_contract()), range)
}

pub(super) fn coverage_range(
    coverage: &[CoveredOccurrence],
    range: std::ops::Range<u32>,
) -> Vec<CoveredOccurrence> {
    coverage
        .iter()
        .filter(|covered| range.contains(&covered.occurrence().get()))
        .cloned()
        .collect()
}

pub(super) fn device(capacity_bytes: u64, ownership: AllocationOwnership) -> AllocationSpec {
    AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    }
}

pub(super) fn value(
    origin: MaterializedOrigin,
    role: ValueRole,
    shape: Shape,
) -> MaterializedValueSpec {
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

pub(super) fn program_input(key: &str) -> MaterializedOrigin {
    MaterializedOrigin::ProgramInput {
        key: InputKey::new(key).expect("input key"),
    }
}

pub(super) fn read(view: ViewId, accessible_bytes: AbiExprId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Read,
        accessible_bytes,
    }
}

pub(super) fn write_access(view: ViewId, accessible_bytes: AbiExprId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Write,
        accessible_bytes,
    }
}

/// The bounded profile's shapes are static, so every ABI quantity a fixture
/// needs is a literal. A dynamic subject would name an input extent instead.
pub(super) fn literal(builder: &mut KernelProgramBuilder, value: u64) -> AbiExprId {
    builder
        .push_abi_root(AbiRoot::UnsignedLiteral(value))
        .expect("abi literal")
}

/// The ABI quantities every fixture in this file shares.
///
/// The arena deduplicates by content, so minting these once per builder and
/// once per fixture produce the same arena.
#[derive(Clone, Copy, Debug)]
pub(super) struct FixtureAbi {
    /// Byte count of a whole `[2, 3]` `f32` value.
    pub(super) input_bytes: AbiExprId,
    /// Byte count of a whole `[2]` `f32` value.
    pub(super) output_bytes: AbiExprId,
    /// Launch extent of a stage iterating the `[2, 3]` shape.
    pub(super) pointwise_threads: AbiExprId,
    /// Launch extent of a stage iterating the `[2]` shape.
    pub(super) reduction_threads: AbiExprId,
    /// Workgroup width every fixture kernel requires.
    pub(super) threads_per_workgroup: AbiExprId,
}

/// Mints the shared ABI quantities every fixture stage names.
pub(super) fn fixture_abi(builder: &mut KernelProgramBuilder) -> FixtureAbi {
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
pub(super) fn computed_fixture_abi(builder: &mut KernelProgramBuilder) -> FixtureAbi {
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
pub(super) fn declare_guard(builder: &mut KernelProgramBuilder) {
    let guard = builder
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("guard predicate");
    builder
        .applicability_guard(guard)
        .expect("applicability guard");
}

/// Declares the whole program contract a verified fixture must state.
pub(super) fn declare_program_contract(builder: &mut KernelProgramBuilder) {
    declare_guard(builder);
    declare_routing_commit(builder);
}

impl FixtureAbi {
    pub(super) fn pointwise_launch(self) -> StageLaunch {
        StageLaunch {
            grid_threads: self.pointwise_threads,
            threads_per_workgroup: self.threads_per_workgroup,
        }
    }

    pub(super) fn reduction_launch(self) -> StageLaunch {
        StageLaunch {
            grid_threads: self.reduction_threads,
            threads_per_workgroup: self.threads_per_workgroup,
        }
    }
}

/// The one lifecycle every verified program must span, with fallback admitted
/// exactly while nothing is committed.
pub(super) const ROUTING_COMMIT_LIFECYCLE: [(RoutingCommitState, RoutingCommitState, bool); 3] = [
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

pub(super) fn declare_routing_commit(builder: &mut KernelProgramBuilder) {
    declare_routing_commit_with_fallback(builder, true);
}

/// Declares the whole lifecycle, choosing whether pre-commit fallback is
/// permitted. Every later step forbids it, which is the rule the builder proves.
pub(super) fn declare_routing_commit_with_fallback(
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

pub(super) fn diagnostic(builder: KernelProgramBuilder) -> KernelProgramDiagnostic {
    let error = builder.build().expect_err("verification must fail");
    *error.diagnostics().first().expect("one diagnostic")
}

/// The wired materialized two-stage serial-sum program.
pub(super) struct TwoStage {
    pub(super) builder: KernelProgramBuilder,
    pub(super) pointwise: StageId,
    pub(super) reduction: StageId,
    pub(super) source: MaterializedValueId,
    pub(super) temporary: MaterializedValueId,
    pub(super) output: MaterializedValueId,
    pub(super) temporary_allocation: AllocationId,
    pub(super) output_allocation: AllocationId,
    pub(super) source_view: ViewId,
    pub(super) temporary_view: ViewId,
    pub(super) abi: FixtureAbi,
}

/// How one two-stage fixture deviates from the canonical program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TwoStageShape {
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
pub(super) struct TwoStageStorage {
    pub(super) temporary_allocation: AllocationId,
    pub(super) output_allocation: AllocationId,
    pub(super) source: MaterializedValueId,
    pub(super) temporary: MaterializedValueId,
    pub(super) output: MaterializedValueId,
    pub(super) source_view: ViewId,
    pub(super) temporary_view: ViewId,
    pub(super) output_view: ViewId,
}

/// Declares the externally bound input, the temporary, and the program output.
pub(super) fn wire_two_stage_storage(
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
pub(super) fn wire_two_stage(
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
                    write_access(temporary_view, abi.input_bytes),
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
                    write_access(output_view, abi.output_bytes),
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
pub(super) fn wire_two_stage_structure(mut wired: TwoStage) -> KernelProgramBuilder {
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
pub(super) fn complete_two_stage(wired: TwoStage) -> KernelProgramBuilder {
    let mut builder = wire_two_stage_structure(wired);
    declare_program_contract(&mut builder);
    builder
}

pub(super) fn two_stage(semantic: &SemanticProgram, shape: TwoStageShape) -> TwoStage {
    wire_two_stage(
        semantic,
        &pointwise_kernel(0, SCALE_BITS),
        &reduction_kernel(1),
        shape,
    )
}

pub(super) fn canonical_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    complete_two_stage(two_stage(semantic, TwoStageShape::Canonical))
        .build()
        .expect("verified kernel program")
}

/// The wired four-stage two-chain program with a shared temporary allocation.
pub(super) struct TwoChain {
    pub(super) builder: KernelProgramBuilder,
    pub(super) first_map: StageId,
    pub(super) first_reduce: StageId,
    pub(super) first_temporary: MaterializedValueId,
    pub(super) second_map: StageId,
    pub(super) second_reduce: StageId,
    pub(super) first_output: MaterializedValueId,
    pub(super) second_output: MaterializedValueId,
    pub(super) second_temporary: MaterializedValueId,
    pub(super) shared: AllocationId,
}

/// Wires two independent chains whose temporaries share one allocation.
///
/// The forward handoff orders the first chain's final reader before the second
/// chain's writer, which is what makes reusing the shared allocation legal.
pub(super) fn two_chain(semantic: &SemanticProgram, handoff: bool) -> TwoChain {
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
                write_access(storage.first_temporary_view, abi.input_bytes),
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
                write_access(storage.first_output_view, abi.output_bytes),
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
                write_access(storage.second_temporary_view, abi.input_bytes),
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
                write_access(storage.second_output_view, abi.output_bytes),
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
pub(super) struct ChainStorage {
    pub(super) shared: AllocationId,
    pub(super) first_temporary: MaterializedValueId,
    pub(super) second_temporary: MaterializedValueId,
    pub(super) first_output: MaterializedValueId,
    pub(super) second_output: MaterializedValueId,
    pub(super) first_source_view: ViewId,
    pub(super) second_source_view: ViewId,
    pub(super) first_temporary_view: ViewId,
    pub(super) second_temporary_view: ViewId,
    pub(super) first_output_view: ViewId,
    pub(super) second_output_view: ViewId,
}

/// Declares two externally bound inputs, two temporaries sharing one
/// program-owned allocation, and two separately allocated program outputs.
pub(super) fn wire_chain_storage(builder: &mut KernelProgramBuilder) -> ChainStorage {
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

pub(super) fn publish_two_chain(chains: TwoChain) -> KernelProgramBuilder {
    publish_two_chain_keyed(chains, ["sum_a", "sum_b"], false)
}

/// Publishes the two chain outputs, optionally against the interface order.
///
/// Insertion admits either order — it checks key membership and rejects a
/// repeated key and role, nothing more — so `reversed` produces a builder that
/// only whole-program verification can refuse.
pub(super) fn publish_two_chain_keyed(
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

/// How an arena-growth fixture wires each level to the one below it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AbiGrowth {
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
pub(super) fn grown_guard(
    builder: &mut KernelProgramBuilder,
    growth: AbiGrowth,
    levels: usize,
) -> AbiExprId {
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
