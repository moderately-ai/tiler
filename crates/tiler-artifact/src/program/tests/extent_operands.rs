//! Live input-extent operand association with the semantic interface.

use super::super::{
    AbiRoot, ArtifactBuildError, ArtifactProgramBuilder, AvailabilityPhase, BackendEntryKey,
    BackendEntryRef, BindingKind, BindingSpec, CompilationEnvironment, EntrySpec, LaunchSpec,
    TargetPropertyKey, VariantSpec,
};
use super::support::graphs::checked_coverage;
use super::{
    CANONICAL_NAN, SCALE_BITS, build_artifact, formulas, fused_program, live_extent_program,
    lowering_provider, payload, profile, rules, selection, semantic_program, strict, variant,
};
use std::sync::Arc;
use tiler_ir::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec, ByteWindow,
    KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec, MemorySpace,
    RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode, StageLaunch,
    StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContractionAxisSource, ContributorOrder, ExecutionBinding, KernelSchedule, LaunchPlan,
    LogicalAccess, OwnershipProof, OwnershipProofKind, OwnershipWitnessId, ReductionTopology,
    RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    F32, F32Add, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::{
    Axis, BindingSource, FactProvenance, RootBinding, Shape, ShapeEnvBuilder, ShapeSymbol,
    SourcedExtent, SymbolScope,
};

fn live_contraction_semantic() -> SemanticProgram {
    let shape = Shape::from_dims([2, 3]);
    let mut draft = SemanticProgramBuilder::try_standard().unwrap();
    let left = draft
        .input::<F32>(InputKey::new("left").unwrap(), shape.clone())
        .unwrap();
    let right = draft
        .input::<F32>(InputKey::new("right").unwrap(), shape)
        .unwrap();
    let result = F32Add::apply(&mut draft, left, right).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    draft.build().unwrap()
}

/// A strict unseeded fold whose contributor count is input 0, axis 1.
fn live_contraction_kernel() -> VerifiedKernel {
    let left = Shape::from_dims([2]);
    let right = Shape::from_dims([3]);
    let output = Shape::from_dims([2, 3]);
    let contracted = Shape::from_dims([]);
    let owner = OwnershipWitnessId::new(0);
    let mut region = ScheduledRegionBuilder::new(RegionId::new(41));
    region.iteration_shape(output.clone()).unwrap();
    for (ordinal, (operand, free)) in [(&left, 0_u32), (&right, 1)].into_iter().enumerate() {
        let witness = u32::try_from(ordinal).unwrap();
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
            .unwrap();
        region
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 0 },
            })
            .unwrap();
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
        .unwrap();
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 6 },
        })
        .unwrap();
    region
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 6 },
        })
        .unwrap();
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
            },
            numerical: strict(),
        })
        .unwrap();
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: 6,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: owner,
            reduction: ReductionTopology::LiveContraction {
                live_access: AccessOrdinal::FIRST,
                live_axis: Axis::new(1),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: 6,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    lower_scheduled_region(&region.build().unwrap()).unwrap()
}

fn live_contraction_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let kernel = live_contraction_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let allocation = |ownership| AllocationSpec {
        capacity_bytes: 24,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let left_allocation = plan
        .push_allocation(allocation(AllocationOwnership::External))
        .unwrap();
    let right_allocation = plan
        .push_allocation(allocation(AllocationOwnership::External))
        .unwrap();
    let output_allocation = plan
        .push_allocation(allocation(AllocationOwnership::Program))
        .unwrap();
    let value = |origin, role| MaterializedValueSpec {
        origin,
        role,
        shape: Shape::from_dims([2, 3]),
        storage_scalar: StorageScalar::F32,
        element_type: KernelType::F32,
        encoding: StorageEncoding::Unpacked,
        alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
    };
    let left = plan
        .push_value(
            value(
                MaterializedOrigin::ProgramInput {
                    key: InputKey::new("left").unwrap(),
                },
                ValueRole::Input,
            ),
            left_allocation,
        )
        .unwrap();
    let right = plan
        .push_value(
            value(
                MaterializedOrigin::ProgramInput {
                    key: InputKey::new("right").unwrap(),
                },
                ValueRole::Input,
            ),
            right_allocation,
        )
        .unwrap();
    let output = plan
        .push_value(
            value(MaterializedOrigin::Internal, ValueRole::Output),
            output_allocation,
        )
        .unwrap();
    let left_view = plan
        .push_view(
            left,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .unwrap();
    let right_view = plan
        .push_view(
            right,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .unwrap();
    let output_view = plan.push_whole_view(output).unwrap();
    let zero = plan.push_abi_root(AbiRoot::UnsignedLiteral(0)).unwrap();
    let twenty_four = plan.push_abi_root(AbiRoot::UnsignedLiteral(24)).unwrap();
    let six = plan.push_abi_root(AbiRoot::UnsignedLiteral(6)).unwrap();
    let one = plan.push_abi_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let guard = plan.push_abi_root(AbiRoot::BooleanLiteral(true)).unwrap();
    plan.applicability_guard(guard).unwrap();
    for (from, to, fallback_permitted) in [
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
    ] {
        plan.push_routing_commit_transition(RoutingCommitTransition {
            from,
            to,
            fallback_permitted,
        })
        .unwrap();
    }
    plan.push_stage(
        &kernel,
        &checked_coverage(semantic),
        &[
            StageAccess {
                view: left_view,
                mode: StageAccessMode::Read,
                accessible_bytes: zero,
            },
            StageAccess {
                view: right_view,
                mode: StageAccessMode::Read,
                accessible_bytes: zero,
            },
            StageAccess {
                view: output_view,
                mode: StageAccessMode::Write,
                accessible_bytes: twenty_four,
            },
        ],
        StageLaunch {
            grid_threads: six,
            threads_per_workgroup: one,
        },
    )
    .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), output)
        .unwrap();
    plan.build().unwrap()
}

// -------------------------------------------------------------------------
// Live input-extent operand association with the semantic interface
// -------------------------------------------------------------------------
//
// A live operand makes an interface axis's extent a per-invocation binding, so
// the axis must be one the semantic subject actually leaves open. The former
// worked examples here executed a fixed `[2, 3]` subject at bound extents 14
// and 15 — meanings outside the fixed program's own semantic graph — and their
// envelope, identity, and addressing evidence is withdrawn until a true
// symbolic `[2, N]` artifact can be packaged
// (`package-the-admitted-live-schedule-into-a-symbolic-kernel-program`);
// `prove-one-live-extent-artifact-payload-and-pipeline-at-two-n` owns the
// rerun.

/// The exact former wrong-positive, now refused at artifact construction.
///
/// Subject perturbed, assertion unchanged: the fused plan over the same
/// `[2, 3]` semantic program still packages, and adding the live operand on
/// axis 1 is the one perturbation that flips construction into the refusal.
#[test]
fn a_live_operand_over_a_fixed_semantic_axis_refuses_at_artifact_construction() {
    let semantic = semantic_program();
    let static_sibling = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    build_artifact(
        &semantic,
        &static_sibling,
        provider.clone(),
        std::slice::from_ref(&provider),
    );

    let live = live_extent_program(&semantic);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let error = draft
        .push_variant(&live, variant(&formulas, descriptor, b"live-row-major"))
        .expect_err("a fixed semantic axis must not acquire a caller-selected extent");
    assert_eq!(
        error,
        ArtifactBuildError::ExtentOperandStaticAxis {
            entry: 0,
            key: "input".to_owned(),
            axis: 1,
            extent: 3,
        },
        "the refusal must name the fixed semantic axis, not a later check",
    );
}

/// The contraction spelling of the same defect refuses identically.
#[test]
fn a_live_contraction_operand_over_a_fixed_semantic_axis_refuses() {
    let semantic = live_contraction_semantic();
    let program = live_contraction_program(&semantic);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let error = draft
        .push_variant(
            &program,
            VariantSpec {
                target_profile: profile(),
                feasibility_rules: rules(),
                deferred_predicates: Vec::new(),
                entries: vec![EntrySpec {
                    bindings: vec![
                        BindingSpec {
                            kind: BindingKind::Buffer,
                        };
                        3
                    ],
                    launch: LaunchSpec {
                        zero_work_skips_dispatch: true,
                        preconditions: Vec::new(),
                    },
                    implementation: BackendEntryRef {
                        payloads: vec![descriptor],
                        entry_key: BackendEntryKey::from_bytes(b"live-contraction").unwrap(),
                    },
                }],
            },
        )
        .expect_err("a live contributor extent over a fixed semantic axis must refuse");
    assert_eq!(
        error,
        ArtifactBuildError::ExtentOperandStaticAxis {
            entry: 0,
            key: "left".to_owned(),
            axis: 1,
            extent: 3,
        },
    );
}

/// Builds one rank-one two-input symbolic graph `a + b` over `[n]`.
///
/// The real construction path for the association's symbolic arms: a verified
/// semantic program whose interface extents name a declared symbol, exactly as
/// the compiler's admitted live population authors them. Since
/// `tiler.artifact-program.v21` the published interface carries such a boundary,
/// so the association below reads the *production* projection
/// `ArtifactProgramBuilder::new` builds rather than a test-local one.
fn symbolic_two_input_semantic(environment: Arc<tiler_ir::shape::ShapeEnv>) -> SemanticProgram {
    let mut draft =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let n = SourcedExtent::Symbol(association_symbol("n"));
    let a = draft
        .input_sourced::<F32>(InputKey::new("a").unwrap(), vec![n.clone()])
        .unwrap();
    let b = draft
        .input_sourced::<F32>(InputKey::new("b").unwrap(), vec![n])
        .unwrap();
    let root = F32Add::apply(&mut draft, a, b).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    draft.build().unwrap()
}

fn association_symbol(name: &str) -> ShapeSymbol {
    ShapeSymbol::new(SymbolScope::new("program/0").unwrap(), name).unwrap()
}

/// One environment declaring `n` at a caller-chosen root.
fn association_environment(root: BindingSource) -> Arc<tiler_ir::shape::ShapeEnv> {
    let mut draft = ShapeEnvBuilder::new();
    let n = association_symbol("n");
    draft.declare(n.clone()).unwrap();
    draft
        .bind(
            &n,
            RootBinding::new(
                root,
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    Arc::new(draft.build().unwrap())
}

fn input_dimension_root(input: &str, axis: u32) -> BindingSource {
    BindingSource::InputDimension {
        input: InputKey::new(input).unwrap(),
        axis: Axis::new(axis),
    }
}

/// The association's inputs, derived from one real verified program.
#[allow(
    clippy::type_complexity,
    reason = "the pair is the association's two exact authorities and is destructured at every call site"
)]
fn association_authorities(
    semantic: &SemanticProgram,
) -> (
    Vec<(InputKey, Vec<SourcedExtent>)>,
    Vec<(ShapeSymbol, RootBinding)>,
) {
    let sources = super::super::builder::read_semantic_interface(semantic)
        .expect("the fixture's boundary is publishable")
        .input_extent_sources();
    let retained = super::super::retained::RetainedShapeEnvironment::project(semantic)
        .expect("the fixture environment projects");
    (sources, retained.bindings().to_vec())
}

fn associate(
    sources: &[(InputKey, Vec<SourcedExtent>)],
    bindings: &[(ShapeSymbol, RootBinding)],
    key: &str,
    axis: u32,
) -> Result<(), ArtifactBuildError> {
    let key = InputKey::new(key).unwrap();
    let (_, extents) = sources
        .iter()
        .find(|(declared, _)| *declared == key)
        .expect("the fixture names a declared input");
    super::super::builder::check_extent_operand_association(
        0,
        &key,
        Axis::new(axis),
        extents,
        bindings,
    )
}

/// The accepting arm: the operand names the environment's exact root axis.
#[test]
fn a_live_operand_on_the_source_bearing_symbolic_axis_associates() {
    let semantic =
        symbolic_two_input_semantic(association_environment(input_dimension_root("a", 0)));
    let (sources, bindings) = association_authorities(&semantic);
    associate(&sources, &bindings, "a", 0)
        .expect("the source-bearing axis is the association's one accepting arm");
}

/// An equal-shaped sibling axis is an inferred occurrence, not the source.
#[test]
fn a_live_operand_on_an_inferred_equal_axis_refuses() {
    let semantic =
        symbolic_two_input_semantic(association_environment(input_dimension_root("a", 0)));
    let (sources, bindings) = association_authorities(&semantic);
    assert_eq!(
        associate(&sources, &bindings, "b", 0),
        Err(ArtifactBuildError::ExtentOperandSourceMismatch {
            entry: 0,
            key: "b".to_owned(),
            axis: 0,
            root_key: "a".to_owned(),
            root_axis: 0,
        }),
        "an axis that merely names the symbol must not stand in for its root",
    );
}

/// A swapped environment leaves the axis's symbol unbound and refuses.
#[test]
fn a_live_operand_under_a_swapped_environment_refuses() {
    let semantic =
        symbolic_two_input_semantic(association_environment(input_dimension_root("a", 0)));
    let (sources, _) = association_authorities(&semantic);
    // A second real environment whose one declared symbol is another scope's:
    // the foreign-environment perturbation, built exactly as the first.
    let foreign = {
        let mut draft = ShapeEnvBuilder::new();
        let m = ShapeSymbol::new(SymbolScope::new("program/1").unwrap(), "n").unwrap();
        draft.declare(m.clone()).unwrap();
        draft
            .bind(
                &m,
                RootBinding::new(
                    input_dimension_root("a", 0),
                    AvailabilityPhase::LiveDevicePreflight,
                    FactProvenance::RuntimeValidated,
                )
                .unwrap(),
            )
            .unwrap();
        draft.build().unwrap()
    };
    let foreign_bindings: Vec<_> = foreign
        .bindings()
        .map(|(symbol, binding)| (symbol.clone(), binding.clone()))
        .collect();
    assert_eq!(
        associate(&sources, &foreign_bindings, "a", 0),
        Err(ArtifactBuildError::ExtentOperandForeignSymbol {
            entry: 0,
            key: "a".to_owned(),
            axis: 0,
            symbol: association_symbol("n").to_string(),
        }),
        "a symbol the artifact's one environment does not bind has no authority",
    );
}

/// A symbol rooted outside the input dimensions has no operand to answer.
#[test]
fn a_live_operand_on_a_symbol_without_an_input_source_refuses() {
    let semantic =
        symbolic_two_input_semantic(association_environment(BindingSource::TargetProperty {
            key: TargetPropertyKey::new("tiler.target.test.n@1").unwrap(),
        }));
    let (sources, bindings) = association_authorities(&semantic);
    assert_eq!(
        associate(&sources, &bindings, "a", 0),
        Err(ArtifactBuildError::ExtentOperandUnsourcedSymbol {
            entry: 0,
            key: "a".to_owned(),
            axis: 0,
            symbol: association_symbol("n").to_string(),
            source: "target-property `tiler.target.test.n@1`".to_owned(),
        }),
        "a target-property root is answered by its own authority, never an operand",
    );
}

/// An axis outside the semantic rank is absent from the interface.
#[test]
fn a_live_operand_outside_the_semantic_rank_refuses() {
    let semantic =
        symbolic_two_input_semantic(association_environment(input_dimension_root("a", 0)));
    let (sources, bindings) = association_authorities(&semantic);
    assert_eq!(
        associate(&sources, &bindings, "a", 4),
        Err(ArtifactBuildError::ExtentOperandAxis {
            entry: 0,
            key: "a".to_owned(),
            axis: 4,
            rank: 1,
        }),
    );
}

/// The association's static arm refuses either fixed axis, not only axis 1.
#[test]
fn the_association_refuses_every_fixed_axis_by_its_own_extent() {
    let semantic = semantic_program();
    let (sources, bindings) = association_authorities(&semantic);
    for (axis, extent) in [(0_u32, 2_u64), (1, 3)] {
        assert_eq!(
            associate(&sources, &bindings, "input", axis),
            Err(ArtifactBuildError::ExtentOperandStaticAxis {
                entry: 0,
                key: "input".to_owned(),
                axis,
                extent,
            }),
            "axis {axis} is fixed at {extent} and must refuse by that value",
        );
    }
}
