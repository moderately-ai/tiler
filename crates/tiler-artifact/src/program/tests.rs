//! Bounded tests for the target-neutral artifact program model.
//!
//! Fixtures package real verified kernel programs over real verified semantic
//! programs, so every rejection is a rejection of a plan that the shared IR
//! itself already accepted. Nothing here asserts that a kernel computes the
//! operations its stage covers; that remains compiler-owned evidence.

use std::sync::Arc;

use tiler_ir::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::{
    AllocationOwnership, AllocationSpec, KernelProgramBuilder, MaterializedOrigin,
    MaterializedValueSpec, MemorySpace, SemanticOccurrence, StageAccess, StageAccessMode,
    ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
    ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess, NumericalPermission,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    ReductionTopology, RegionId, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy,
    TensorRole,
};
use tiler_ir::semantic::{
    CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind, CanonicalValueView, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, F32Add, F32Constant, F32Multiply, InputKey,
    NormativeDefinitionRef, OpKey, OperationArity, OperationAttributeSchema, OperationConformance,
    OperationDefinition, OperationDefinitionFacts, OperationEffect, OperationInferenceError,
    OperationInferenceOutputs, OperationInferenceRequest, OperationInferencer, OperationSchema,
    OutputKey, ProviderDiagnosticCode, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE, RegistryError,
    SemanticProgram, SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, StrictSerialF32Sum, TypeDefinitionFacts, TypeKey, ValueFact,
    ValueTypeDefinition, ValueTypeDefinitionKey, add_f32_op, constant_f32_op, multiply_f32_op,
    strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Shape};

use super::{
    AbiBinaryOp, AbiEvaluationError, AbiExprId, AbiExprUse, AbiFactBinder, AbiRoot, AbiType,
    AbiUnaryOp, AbiValue, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind,
    ArtifactExecutionPolicy, ArtifactProgramBuilder, AvailabilityPhase, BackendEntryKey,
    BackendEntryRef, BackendKey, BackendPayloadDescriptor, BindingKind, BindingSpec, CapabilityKey,
    CompilationEnvironment, DeferredPredicateSpec, EntrySpec, FeasibilityRuleSetKey,
    FeasibilityRuleSetRef, LaunchSpec, PayloadDigest, PayloadId, RepresentationKey, SchemaVersion,
    SelectedProvider, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
    TargetPropertyKey, VariantSpec, VerifiedArtifactProgram,
};

// The seven items this suite shares with `crate::proof::tests` are `pub(crate)`
// rather than `pub(super)`; the rest of the fixture set stays module-local. The
// proof sidecar associates with a *real* verified artifact, and a second
// hand-built one would be a second thing to keep correct.
pub(crate) const SCALE_BITS: u32 = 0x4000_0000; // 2.0f32
pub(crate) const OTHER_SCALE_BITS: u32 = 0x4040_0000; // 3.0f32
pub(super) const BIAS_BITS: u32 = 0x3f80_0000; // 1.0f32
pub(super) const CANONICAL_NAN: u32 = 0x7fc0_0000;
pub(super) const ELEMENT_BYTES: u64 = 4;

// -------------------------------------------------------------------------
// Shared-IR fixtures
// -------------------------------------------------------------------------

pub(super) fn strict() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        CANONICAL_NAN,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
    )
}

pub(super) fn input_shape() -> Shape {
    Shape::from_dims([2, 3])
}

pub(super) fn output_shape() -> Shape {
    Shape::from_dims([2])
}

pub(super) fn build_graph(draft: SemanticProgramBuilder) -> SemanticProgram {
    build_graph_scaled(draft, 2.0)
}

/// Builds the fixture graph, parameterized by the pointwise scale constant.
///
/// The scale is the cheapest way to obtain a genuinely different semantic graph
/// that keeps the same named interface: an unreached extra input would be
/// compacted away at commit (ADR 0064) and would not change graph identity.
pub(crate) fn build_graph_scaled(
    mut draft: SemanticProgramBuilder,
    scale_value: f32,
) -> SemanticProgram {
    let input = draft
        .input::<F32>(InputKey::new("input").unwrap(), input_shape())
        .unwrap();
    let scale = F32Constant::apply(&mut draft, scale_value.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut draft, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = F32Add::apply(&mut draft, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)]).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    draft.build().unwrap()
}

pub(crate) fn semantic_program() -> SemanticProgram {
    build_graph(SemanticProgramBuilder::try_standard().unwrap())
}

/// Builds the one fused reduction kernel the packaged plans dispatch.
pub(super) fn fused_kernel(scale_bits: u32) -> VerifiedKernel {
    let axes = vec![Axis::new(1)];
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region.iteration_shape(output_shape()).unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Input,
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
        .unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            kind: BoundsProofKind::LinearRange { element_count: 2 },
        })
        .unwrap();
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
        })
        .unwrap();
    region
        .scalar_program(ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits: BIAS_BITS,
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: CANONICAL_NAN,
            empty_identity_bits: 0,
            contraction: false,
        })
        .unwrap();
    region.numerical(strict()).unwrap();
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: 2,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::Serial {
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: 2,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    lower_scheduled_region(&region.build().unwrap()).unwrap()
}

/// Builds the single-stage kernel program the artifact packages.
pub(crate) fn fused_program(semantic: &SemanticProgram, scale_bits: u32) -> VerifiedKernelProgram {
    let kernel = fused_kernel(scale_bits);
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let external = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 24,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .unwrap();
    let owned = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 8,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .unwrap();
    let source = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::ProgramInput {
                    key: InputKey::new("input").unwrap(),
                },
                role: ValueRole::Input,
                shape: input_shape(),
                element_type: KernelType::F32,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            external,
        )
        .unwrap();
    let result = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::Internal,
                role: ValueRole::Output,
                shape: output_shape(),
                element_type: KernelType::F32,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            owned,
        )
        .unwrap();
    let read = plan.push_whole_view(source).unwrap();
    let write = plan.push_whole_view(result).unwrap();
    plan.push_stage(
        &kernel,
        &(0..5).map(SemanticOccurrence::new).collect::<Vec<_>>(),
        &[
            StageAccess {
                view: read,
                mode: StageAccessMode::Read,
            },
            StageAccess {
                view: write,
                mode: StageAccessMode::Write,
            },
        ],
    )
    .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

// -------------------------------------------------------------------------
// Artifact fixtures
// -------------------------------------------------------------------------

pub(crate) fn lowering_provider(revision: u32) -> ProviderIdentity {
    ProviderIdentity::new("tiler-test", "fused-serial-sum", revision).unwrap()
}

pub(super) fn spare_provider(revision: u32) -> ProviderIdentity {
    ProviderIdentity::new("tiler-test", "never-selected", revision).unwrap()
}

pub(super) fn selection(provider: ProviderIdentity) -> SelectedProvider {
    SelectedProvider {
        provider,
        capability: CapabilityKey::new("tiler.capability.fused-serial-sum").unwrap(),
        capability_api_version: 1,
    }
}

pub(super) fn payload(tag: u8) -> BackendPayloadDescriptor {
    BackendPayloadDescriptor {
        backend: BackendKey::new("tiler.metal").unwrap(),
        representation: RepresentationKey::new("metallib").unwrap(),
        payload_schema: SchemaVersion::new(1, 0),
        digest: PayloadDigest::from_bytes([tag, 0xb2, 0xc3]).unwrap(),
        compatibility: profile(),
        execution_policy: ArtifactExecutionPolicy::RequiresDeviceTranslation,
    }
}

pub(super) fn profile() -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new("tiler.test.baseline").unwrap(),
        descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02]).unwrap(),
    }
}

pub(super) fn rules() -> FeasibilityRuleSetRef {
    FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new("tiler.test.feasibility").unwrap(),
        revision: 1,
    }
}

/// The expression handles every fixture variant is assembled from.
pub(super) struct Formulas {
    pub(super) rows: AbiExprId,
    pub(super) input_bytes: AbiExprId,
    pub(super) output_bytes: AbiExprId,
    pub(super) one: AbiExprId,
    pub(super) always: AbiExprId,
}

pub(super) fn formulas(draft: &mut ArtifactProgramBuilder) -> Formulas {
    let key = InputKey::new("input").unwrap();
    let rows = draft
        .push_root(AbiRoot::InputExtent {
            key: key.clone(),
            axis: Axis::new(0),
        })
        .unwrap();
    let columns = draft
        .push_root(AbiRoot::InputExtent {
            key,
            axis: Axis::new(1),
        })
        .unwrap();
    let width = draft
        .push_root(AbiRoot::UnsignedLiteral(ELEMENT_BYTES))
        .unwrap();
    let elements = draft
        .push_binary(AbiBinaryOp::CheckedMultiply, rows, columns)
        .unwrap();
    let input_bytes = draft
        .push_binary(AbiBinaryOp::CheckedMultiply, elements, width)
        .unwrap();
    let output_bytes = draft
        .push_binary(AbiBinaryOp::CheckedMultiply, rows, width)
        .unwrap();
    let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let always = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
    Formulas {
        rows,
        input_bytes,
        output_bytes,
        one,
        always,
    }
}

pub(super) fn entry(formulas: &Formulas, payload: PayloadId, key: &[u8]) -> EntrySpec {
    EntrySpec {
        bindings: vec![
            BindingSpec {
                kind: BindingKind::Buffer,
                accessible_bytes: formulas.input_bytes,
            },
            BindingSpec {
                kind: BindingKind::Buffer,
                accessible_bytes: formulas.output_bytes,
            },
        ],
        launch: LaunchSpec {
            grid_threads: formulas.rows,
            threads_per_workgroup: formulas.one,
            zero_work_skips_dispatch: true,
            preconditions: Vec::new(),
        },
        implementation: BackendEntryRef {
            payload,
            entry_key: BackendEntryKey::from_bytes(key).unwrap(),
        },
    }
}

pub(super) fn variant(formulas: &Formulas, payload: PayloadId, key: &[u8]) -> VariantSpec {
    VariantSpec {
        applicability_guard: formulas.always,
        target_profile: profile(),
        feasibility_rules: rules(),
        deferred_predicates: Vec::new(),
        entries: vec![entry(formulas, payload, key)],
    }
}

/// Assembles the canonical one-variant artifact over one packaged program.
pub(crate) fn build_artifact(
    semantic: &SemanticProgram,
    program: &VerifiedKernelProgram,
    selected: ProviderIdentity,
    available: &[ProviderIdentity],
) -> VerifiedArtifactProgram {
    let environment = CompilationEnvironment::new(available.iter().cloned()).unwrap();
    let mut draft = ArtifactProgramBuilder::new(semantic, environment).unwrap();
    draft.select_provider(selection(selected)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    draft.build().unwrap()
}

pub(crate) fn default_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    build_artifact(&semantic, &program, provider.clone(), &[provider])
}

// -------------------------------------------------------------------------
// Verified-product construction and consumability
// -------------------------------------------------------------------------

#[test]
fn builds_a_verified_single_variant_artifact() {
    let artifact = default_artifact();
    assert_eq!(artifact.variants().len(), 1);
    assert_eq!(artifact.payloads().len(), 1);
    assert_eq!(artifact.selected_providers().len(), 1);
    assert_eq!(artifact.schema(), super::ArtifactSchema::GOVERNED);
    assert_eq!(
        artifact.routing_policy(),
        super::RoutingPolicy::StablePriority
    );
    let input = artifact.inputs().next().expect("one declared input");
    assert_eq!(input.key().as_str(), "input");
    assert_eq!(input.shape(), &input_shape());
    assert_eq!(input.element_type(), KernelType::F32);
    let output = artifact.outputs().next().expect("one declared output");
    assert_eq!(output.key().as_str(), "result");
    assert_eq!(output.shape(), &output_shape());
}

#[test]
fn an_entry_reads_its_plan_through_the_shared_ir_alone() {
    let artifact = default_artifact();
    let variant = artifact.variants().next().expect("one variant");
    assert_eq!(variant.routing_rank(), 0);
    assert_eq!(variant.target_profile(), &profile());
    assert_eq!(variant.deferred_predicates().len(), 0);
    let entry = variant.entries().next().expect("one entry");
    assert_eq!(
        entry.kernel_identity(),
        entry.stage().kernel().canonical_identity(),
    );
    assert_eq!(entry.resources().buffer_bindings, 2);
    assert_eq!(entry.numerical(), strict());
    assert!(entry.zero_work_skips_dispatch());
    assert_eq!(entry.backend_entry_key().as_bytes(), b"fused");
    assert_eq!(entry.payload().representation.as_str(), "metallib");
    let bindings: Vec<_> = entry.bindings().collect();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].slot(), 0);
    assert_eq!(bindings[0].kind(), BindingKind::Buffer);
    assert_eq!(bindings[0].element_type(), KernelType::F32);
    assert_eq!(bindings[0].value_role(), ValueRole::Input);
    assert_eq!(bindings[0].alignment(), 4);
    assert_eq!(bindings[0].window().length, 24);
    assert_eq!(bindings[1].value_role(), ValueRole::Output);
    assert_eq!(bindings[1].window().length, 8);
    // The plan itself is reachable through the shared IR's own views.
    assert_eq!(variant.program().stages().len(), 1);
    assert_eq!(bindings[0].value().required_bytes(), 24);
}

#[test]
fn abi_expressions_evaluate_against_bound_runtime_facts() {
    let artifact = default_artifact();
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_shape(&InputKey::new("input").unwrap(), &input_shape())
        .unwrap();
    let facts = binder.build();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let bindings: Vec<_> = entry.bindings().collect();
    assert_eq!(
        bindings[0].accessible_bytes().evaluate(&facts).unwrap(),
        AbiValue::Unsigned(24),
    );
    assert_eq!(
        bindings[1].accessible_bytes().evaluate(&facts).unwrap(),
        AbiValue::Unsigned(8),
    );
    assert_eq!(
        entry.launch_threads().evaluate(&facts).unwrap(),
        AbiValue::Unsigned(2),
    );
    assert_eq!(entry.launch_threads().value_type(), AbiType::Unsigned);
}

// -------------------------------------------------------------------------
// Identity determinism and order independence
// -------------------------------------------------------------------------

#[test]
fn identity_is_deterministic_for_equal_artifacts() {
    let first = default_artifact();
    let second = default_artifact();
    assert_eq!(first.canonical_identity(), second.canonical_identity());
    assert_eq!(first, second);
}

#[test]
fn identity_ignores_payload_and_provider_declaration_order() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let providers = [lowering_provider(1), lowering_provider(2)];
    let environment = CompilationEnvironment::new(providers.iter().cloned()).unwrap();

    let alternate = fused_program(&semantic, OTHER_SCALE_BITS);

    let assemble = |forward: bool| {
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
        let (first, second) = if forward { (0, 1) } else { (1, 0) };
        draft
            .select_provider(selection(providers[first].clone()))
            .unwrap();
        draft
            .select_provider(selection(providers[second].clone()))
            .unwrap();
        let (primary, spare) = if forward {
            let primary = draft.push_payload(payload(0x01)).unwrap();
            (primary, draft.push_payload(payload(0x02)).unwrap())
        } else {
            let spare = draft.push_payload(payload(0x02)).unwrap();
            (draft.push_payload(payload(0x01)).unwrap(), spare)
        };
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&program, variant(&formulas, primary, b"fused"))
            .unwrap();
        draft
            .push_variant(&alternate, variant(&formulas, spare, b"alternate"))
            .unwrap();
        draft.build().unwrap()
    };

    assert_eq!(
        assemble(true).canonical_identity(),
        assemble(false).canonical_identity(),
    );
}

#[test]
fn identity_ignores_expression_assembly_order() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();

    let assemble = |reversed: bool| {
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
        draft.select_provider(selection(provider.clone())).unwrap();
        let descriptor = draft.push_payload(payload(0xa1)).unwrap();
        let key = InputKey::new("input").unwrap();
        // Assemble the identical formulas through two different node orders.
        let formulas = if reversed {
            let always = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
            let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
            let width = draft
                .push_root(AbiRoot::UnsignedLiteral(ELEMENT_BYTES))
                .unwrap();
            let columns = draft
                .push_root(AbiRoot::InputExtent {
                    key: key.clone(),
                    axis: Axis::new(1),
                })
                .unwrap();
            let rows = draft
                .push_root(AbiRoot::InputExtent {
                    key,
                    axis: Axis::new(0),
                })
                .unwrap();
            let output_bytes = draft
                .push_binary(AbiBinaryOp::CheckedMultiply, rows, width)
                .unwrap();
            let elements = draft
                .push_binary(AbiBinaryOp::CheckedMultiply, rows, columns)
                .unwrap();
            let input_bytes = draft
                .push_binary(AbiBinaryOp::CheckedMultiply, elements, width)
                .unwrap();
            Formulas {
                rows,
                input_bytes,
                output_bytes,
                one,
                always,
            }
        } else {
            formulas(&mut draft)
        };
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"fused"))
            .unwrap();
        draft.build().unwrap()
    };

    assert_eq!(
        assemble(false).canonical_identity(),
        assemble(true).canonical_identity(),
    );
}

#[test]
fn the_expression_arena_is_canonically_deduplicated() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let first = draft.push_root(AbiRoot::UnsignedLiteral(4)).unwrap();
    let second = draft.push_root(AbiRoot::UnsignedLiteral(4)).unwrap();
    assert_eq!(first, second);
    let sum = draft
        .push_binary(AbiBinaryOp::CheckedAdd, first, second)
        .unwrap();
    let again = draft
        .push_binary(AbiBinaryOp::CheckedAdd, second, first)
        .unwrap();
    assert_eq!(sum, again);
}

// -------------------------------------------------------------------------
// Reached versus unused provenance (ADR 0072)
// -------------------------------------------------------------------------

#[test]
fn a_reached_capability_provider_revision_changes_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let available = [lowering_provider(1), lowering_provider(2)];
    let first = build_artifact(&semantic, &program, lowering_provider(1), &available);
    let second = build_artifact(&semantic, &program, lowering_provider(2), &available);
    assert_ne!(first.canonical_identity(), second.canonical_identity());
}

#[test]
fn an_unused_environment_provider_does_not_change_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let selected = lowering_provider(1);
    let lean = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        std::slice::from_ref(&selected),
    );
    let crowded = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        &[selected.clone(), spare_provider(1)],
    );
    let bumped = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        &[selected, spare_provider(7)],
    );
    assert_eq!(lean.canonical_identity(), crowded.canonical_identity());
    assert_eq!(crowded.canonical_identity(), bumped.canonical_identity());
    // The environments genuinely differed; only the reached half was packaged.
    assert_eq!(lean.selected_providers().len(), 1);
    assert_eq!(crowded.selected_providers().len(), 1);
}

#[test]
fn a_reached_semantic_provider_revision_changes_identity() {
    let first = governed_program(1);
    let second = governed_program(2);
    assert_eq!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph(),
    );
    assert_eq!(
        first.semantic_identity().reached_definitions(),
        second.semantic_identity().reached_definitions(),
    );
    assert_ne!(
        first.semantic_identity().admission_provenance(),
        second.semantic_identity().admission_provenance(),
    );
    let provider = lowering_provider(1);
    let first_artifact = build_artifact(
        &first,
        &fused_program(&first, SCALE_BITS),
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let second_artifact = build_artifact(
        &second,
        &fused_program(&second, SCALE_BITS),
        provider.clone(),
        &[provider],
    );
    assert_ne!(
        first_artifact.canonical_identity(),
        second_artifact.canonical_identity(),
    );
}

#[test]
fn an_unused_semantic_provider_revision_does_not_change_identity() {
    let first = program_with_unused_provider(1);
    let second = program_with_unused_provider(2);
    // The fixture is meaningful only if the two programs really differ.
    assert_ne!(
        first.semantic_identity().registry_snapshot(),
        second.semantic_identity().registry_snapshot(),
    );
    assert_eq!(
        first.semantic_identity().admission_provenance(),
        second.semantic_identity().admission_provenance(),
    );
    let provider = lowering_provider(1);
    let first_artifact = build_artifact(
        &first,
        &fused_program(&first, SCALE_BITS),
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let second_artifact = build_artifact(
        &second,
        &fused_program(&second, SCALE_BITS),
        provider.clone(),
        &[provider],
    );
    assert_eq!(
        first_artifact.canonical_identity(),
        second_artifact.canonical_identity(),
    );
}

// -------------------------------------------------------------------------
// Cross-program and forged-input rejection
// -------------------------------------------------------------------------

#[test]
fn rejects_a_variant_realizing_another_semantic_graph() {
    let packaged = semantic_program();
    let other = build_graph_scaled(SemanticProgramBuilder::try_standard().unwrap(), 3.0);
    assert_ne!(
        packaged.semantic_identity().graph(),
        other.semantic_identity().graph(),
    );
    let foreign = fused_program(&other, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&packaged, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    assert_eq!(
        draft.push_variant(&foreign, variant(&formulas, descriptor, b"fused")),
        Err(ArtifactBuildError::SemanticSubjectMismatch),
    );
}

#[test]
fn rejects_an_expression_handle_from_another_builder() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut donor = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
    let donor_formulas = formulas(&mut donor);
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.applicability_guard = donor_formulas.always;
    assert_eq!(
        draft.push_variant(&program, spec),
        Err(ArtifactBuildError::ForeignHandle {
            entity: ArtifactEntityKind::Expression,
        }),
    );
}

#[test]
fn rejects_a_payload_handle_from_another_builder() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut donor = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
    let donor_payload = donor.push_payload(payload(0xa1)).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    assert_eq!(
        draft.push_variant(&program, variant(&formulas, donor_payload, b"fused")),
        Err(ArtifactBuildError::ForeignHandle {
            entity: ArtifactEntityKind::Payload,
        }),
    );
}

// -------------------------------------------------------------------------
// Negative tests, one per insertion-time rule
// -------------------------------------------------------------------------

#[test]
fn rejects_a_provider_the_environment_never_offered() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    assert_eq!(
        draft.select_provider(selection(lowering_provider(9))),
        Err(ArtifactBuildError::ProviderNotAvailable {
            provider: Box::new(lowering_provider(9)),
        }),
    );
}

#[test]
fn rejects_an_accessible_range_that_contradicts_the_program() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries[0].bindings[0].accessible_bytes = formulas.output_bytes;
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::AccessibleBytesDisagreement {
            entry: 0,
            binding: 0,
            expected: 24,
            actual: 8,
        }),
    );
}

#[test]
fn rejects_a_launch_that_contradicts_the_kernel_requirements() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries[0].launch.threads_per_workgroup = formulas.rows;
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::LaunchDisagreement {
            entry: 0,
            expected: 1,
            actual: 2,
        }),
    );
}

#[test]
fn rejects_a_guard_that_is_not_a_predicate() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.applicability_guard = formulas.rows;
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::ExpressionType {
            use_site: AbiExprUse::ApplicabilityGuard,
            expected: AbiType::Boolean,
            actual: AbiType::Unsigned,
        }),
    );
}

#[test]
fn rejects_a_size_expression_naming_a_device_property() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let property = draft
            .push_root(AbiRoot::TargetProperty {
                key: TargetPropertyKey::new("tiler.target.max-threads").unwrap(),
                phase: AvailabilityPhase::LiveDevicePreflight,
            })
            .unwrap();
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries[0].bindings[0].accessible_bytes = property;
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::NonInterfaceRoot {
            use_site: AbiExprUse::AccessibleBytes,
        }),
    );
}

#[test]
fn rejects_a_guard_naming_a_fact_from_a_later_phase() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let property = draft
            .push_root(AbiRoot::TargetProperty {
                key: TargetPropertyKey::new("tiler.target.pipeline-registers").unwrap(),
                phase: AvailabilityPhase::PreparedKernelPreflight,
            })
            .unwrap();
        let bound = draft.push_root(AbiRoot::UnsignedLiteral(64)).unwrap();
        let guard = draft
            .push_binary(AbiBinaryOp::LessOrEqual, property, bound)
            .unwrap();
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.applicability_guard = guard;
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::RootPhaseEscape {
            use_site: AbiExprUse::ApplicabilityGuard,
            available_at: AvailabilityPhase::PreparedKernelPreflight,
            admitted_through: AvailabilityPhase::LiveDevicePreflight,
        }),
    );
}

#[test]
fn rejects_a_deferred_predicate_that_is_already_decided() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![DeferredPredicateSpec {
            predicate: formulas.always,
            phase: AvailabilityPhase::CompileProfile,
            authority: lowering_provider(1),
        }];
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::NonDeferredPredicatePhase {
            phase: AvailabilityPhase::CompileProfile,
        }),
    );
}

#[test]
fn rejects_a_deferred_authority_that_was_never_selected() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![DeferredPredicateSpec {
            predicate: formulas.always,
            phase: AvailabilityPhase::LaunchPreflight,
            authority: spare_provider(1),
        }];
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::UnselectedDeferredAuthority {
            provider: Box::new(spare_provider(1)),
        }),
    );
}

#[test]
fn accepts_a_deferred_predicate_bound_to_a_selected_authority() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider.clone())).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let property = draft
        .push_root(AbiRoot::TargetProperty {
            key: TargetPropertyKey::new("tiler.target.max-threads-per-workgroup").unwrap(),
            phase: AvailabilityPhase::LaunchPreflight,
        })
        .unwrap();
    let predicate = draft
        .push_binary(AbiBinaryOp::LessOrEqual, formulas.one, property)
        .unwrap();
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.deferred_predicates = vec![DeferredPredicateSpec {
        predicate,
        phase: AvailabilityPhase::LaunchPreflight,
        authority: provider.clone(),
    }];
    draft.push_variant(&program, spec).unwrap();
    let artifact = draft.build().unwrap();
    let deferred = artifact
        .variants()
        .next()
        .expect("one variant")
        .deferred_predicates()
        .next()
        .expect("one deferred predicate");
    assert_eq!(deferred.phase(), AvailabilityPhase::LaunchPreflight);
    assert_eq!(deferred.authority(), &provider);
}

#[test]
fn rejects_a_repeated_deferred_predicate() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let predicate = DeferredPredicateSpec {
            predicate: formulas.always,
            phase: AvailabilityPhase::LaunchPreflight,
            authority: lowering_provider(1),
        };
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![predicate.clone(), predicate];
        draft.push_variant(program, spec)
    });
    assert_eq!(outcome, Err(ArtifactBuildError::DuplicateDeferredPredicate));
}

#[test]
fn rejects_a_repeated_launch_precondition() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries[0].launch.preconditions = vec![formulas.always, formulas.always];
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::DuplicateLaunchPrecondition { entry: 0 }),
    );
}

#[test]
fn rejects_an_entry_count_that_disagrees_with_the_program() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries.push(entry(formulas, descriptor, b"extra"));
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::EntryCardinality {
            expected: 1,
            actual: 2,
        }),
    );
}

#[test]
fn rejects_a_binding_count_that_disagrees_with_the_kernel_signature() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries[0].bindings.pop();
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::BindingCardinality {
            entry: 0,
            expected: 2,
            actual: 1,
        }),
    );
}

#[test]
fn rejects_a_duplicate_plan_variant() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        draft.push_variant(&program, variant(&formulas, descriptor, b"other")),
        Err(ArtifactBuildError::DuplicateVariant),
    );
}

#[test]
fn rejects_a_repeated_payload_descriptor() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.push_payload(payload(0xa1)).unwrap();
    assert_eq!(
        draft.push_payload(payload(0xa1)),
        Err(ArtifactBuildError::DuplicatePayload),
    );
}

#[test]
fn rejects_a_mistyped_expression_operand() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let number = draft.push_root(AbiRoot::UnsignedLiteral(4)).unwrap();
    assert_eq!(
        draft.push_unary(AbiUnaryOp::Not, number),
        Err(ArtifactBuildError::OperandType {
            expected: AbiType::Boolean,
            actual: AbiType::Unsigned,
        }),
    );
    let predicate = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
    assert_eq!(
        draft.push_select(predicate, number, predicate),
        Err(ArtifactBuildError::SelectBranchType {
            if_true: AbiType::Unsigned,
            if_false: AbiType::Boolean,
        }),
    );
}

// -------------------------------------------------------------------------
// Negative tests, one per whole-artifact rule
// -------------------------------------------------------------------------

#[test]
fn rejects_an_empty_portfolio() {
    let semantic = semantic_program();
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let diagnostics = draft.build().expect_err("an empty portfolio is rejected");
    assert!(
        diagnostics
            .diagnostics()
            .contains(&ArtifactDiagnostic::EmptyPortfolio)
    );
}

#[test]
fn rejects_an_artifact_that_selected_no_provider() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    let error = draft.build().expect_err("unattributed plans are rejected");
    assert_eq!(
        error.diagnostics(),
        [ArtifactDiagnostic::MissingSelectedProvider],
    );
    // The builder comes back intact and the failure is recoverable.
    let (mut recovered, _) = error.into_parts();
    recovered
        .select_provider(selection(lowering_provider(1)))
        .unwrap();
    assert_eq!(recovered.build().unwrap().selected_providers().len(), 1);
}

#[test]
fn rejects_an_expression_no_use_site_reaches() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft.push_root(AbiRoot::UnsignedLiteral(999)).unwrap();
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        draft
            .build()
            .expect_err("an unreachable node is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::UnusedExpression],
    );
}

#[test]
fn rejects_a_payload_no_entry_realizes() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    draft.push_payload(payload(0xb1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        draft
            .build()
            .expect_err("an unreferenced payload is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::UnusedPayload],
    );
}

#[test]
fn rejects_two_entries_claiming_one_backend_entry() {
    let semantic = semantic_program();
    let first = fused_program(&semantic, SCALE_BITS);
    let second = fused_program(&semantic, OTHER_SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&first, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    draft
        .push_variant(&second, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        draft
            .build()
            .expect_err("a non-injective backend mapping is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::DuplicateBackendEntry],
    );
}

// -------------------------------------------------------------------------
// Expression evaluation, phases, and failure classification
// -------------------------------------------------------------------------

#[test]
fn a_conditional_selection_evaluates_only_the_branch_it_takes() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let zero = draft.push_root(AbiRoot::UnsignedLiteral(0)).unwrap();
    let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let ten = draft.push_root(AbiRoot::UnsignedLiteral(10)).unwrap();
    let unsafe_division = draft
        .push_binary(AbiBinaryOp::FloorDivide, ten, zero)
        .unwrap();
    let nonzero = draft
        .push_binary(AbiBinaryOp::LessOrEqual, one, zero)
        .unwrap();
    let guarded = draft.push_select(nonzero, unsafe_division, ten).unwrap();
    let facts = AbiFactBinder::new(AvailabilityPhase::CompileProfile).build();
    assert_eq!(
        evaluate_through_draft(&draft, guarded, &facts),
        Ok(AbiValue::Unsigned(10)),
    );
    assert_eq!(
        evaluate_through_draft(&draft, unsafe_division, &facts),
        Err(AbiEvaluationError::DivisionByZero {
            op: AbiBinaryOp::FloorDivide,
        }),
    );
}

#[test]
fn the_fact_binder_refuses_a_fact_from_a_later_phase() {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    let error = binder
        .bind_target_property(
            TargetPropertyKey::new("tiler.target.pipeline-registers").unwrap(),
            AvailabilityPhase::PreparedKernelPreflight,
            64,
        )
        .expect_err("a prepared-kernel fact is not observable at live preflight");
    assert_eq!(
        error,
        super::AbiBindingError::PhaseNotReached {
            available_at: AvailabilityPhase::PreparedKernelPreflight,
            reached: AvailabilityPhase::LiveDevicePreflight,
        },
    );
    assert_eq!(
        binder.build().reached_phase(),
        AvailabilityPhase::LiveDevicePreflight,
    );
}

#[test]
fn evaluation_reports_an_unbound_root_rather_than_guessing() {
    let artifact = default_artifact();
    let facts = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight).build();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    assert_eq!(
        entry.launch_threads().evaluate(&facts),
        Err(AbiEvaluationError::UnboundInputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(0),
        }),
    );
}

#[test]
fn checked_narrowing_rejects_a_value_that_does_not_fit() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let wide = draft
        .push_root(AbiRoot::UnsignedLiteral(u64::from(u32::MAX) + 1))
        .unwrap();
    let narrowed = draft.push_unary(AbiUnaryOp::NarrowU32, wide).unwrap();
    let facts = AbiFactBinder::new(AvailabilityPhase::CompileProfile).build();
    assert_eq!(
        evaluate_through_draft(&draft, narrowed, &facts),
        Err(AbiEvaluationError::NarrowingOverflow {
            op: AbiUnaryOp::NarrowU32,
            value: u64::from(u32::MAX) + 1,
        }),
    );
}

// -------------------------------------------------------------------------
// Test-local helpers
// -------------------------------------------------------------------------

/// Runs one rejection case against the canonical draft state.
fn with_default_draft<T>(
    case: impl FnOnce(
        &mut ArtifactProgramBuilder,
        &Formulas,
        PayloadId,
        &VerifiedKernelProgram,
    ) -> Result<T, ArtifactBuildError>,
) -> Result<T, ArtifactBuildError> {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    case(&mut draft, &formulas, descriptor, &program)
}

/// Evaluates one draft expression by packaging the arena the builder holds.
///
/// Evaluation is a property of the verified product, so this helper builds a
/// throwaway artifact whose only use site is the expression under test.
fn evaluate_through_draft(
    draft: &ArtifactProgramBuilder,
    node: AbiExprId,
    facts: &super::AbiFacts,
) -> Result<AbiValue, AbiEvaluationError> {
    draft.evaluate_draft_expression(node, facts)
}

// -------------------------------------------------------------------------
// Semantic-provider fixtures
// -------------------------------------------------------------------------

fn diagnostic_code(value: &str) -> ProviderDiagnosticCode {
    ProviderDiagnosticCode::new(value).unwrap()
}

#[derive(Clone, Copy)]
enum TestOperation {
    Constant,
    Binary,
    Sum,
}

impl OperationInferencer for TestOperation {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        match self {
            Self::Constant => {
                outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
            }
            Self::Binary => {
                let left = operands[0].shape();
                let right = operands[1].shape();
                let shape = if left.rank() == 0 {
                    right.clone()
                } else if right.rank() == 0 || left == right {
                    left.clone()
                } else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.binary.shape"),
                        "operands must have equal shapes or include one scalar",
                    )
                    .unwrap());
                };
                outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
            }
            Self::Sum => {
                let Some(CanonicalValueView::Sequence(values)) = attributes
                    .get(REDUCTION_AXES_ATTRIBUTE)
                    .map(CanonicalValue::view)
                else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.sum.axes"),
                        "sum axes must be a sequence",
                    )
                    .unwrap());
                };
                let axes = values
                    .iter()
                    .map(|value| match value.view() {
                        CanonicalValueView::Unsigned {
                            width: CanonicalIntegerWidth::Bits32,
                            bits,
                        } => u32::try_from(bits).map(Axis::new).map_err(|_| {
                            OperationInferenceError::new(
                                diagnostic_code("test.sum.axis-width"),
                                "sum axis exceeds u32",
                            )
                            .unwrap()
                        }),
                        _ => Err(OperationInferenceError::new(
                            diagnostic_code("test.sum.axis-kind"),
                            "sum axes must be u32 values",
                        )
                        .unwrap()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                outputs.try_push(ValueFact::new(
                    F32::resolved_type(),
                    operands[0].shape().without_axes(&axes),
                ))
            }
        }
    }
}

/// A provider the packaged graph actually reaches, with a settable revision.
struct GovernedTestSemantics {
    revision: u32,
}

impl SemanticRegistryProvider for GovernedTestSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "governed-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_marked_value_type::<F32>(
            ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("tiler", "f32", 1).unwrap()),
                NormativeDefinitionRef::new("test binary32 semantics")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ),
            F32::resolved_type(),
        )?;
        register_test_operation(
            registrar,
            constant_f32_op(),
            0,
            [OperationAttributeSchema::required(
                F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValueKind::FloatBits,
            )],
            TestOperation::Constant,
        )?;
        register_test_operation(registrar, multiply_f32_op(), 2, [], TestOperation::Binary)?;
        register_test_operation(registrar, add_f32_op(), 2, [], TestOperation::Binary)?;
        register_test_operation(
            registrar,
            strict_serial_sum_f32_op(),
            1,
            [OperationAttributeSchema::required(
                REDUCTION_AXES_ATTRIBUTE,
                CanonicalValueKind::Sequence,
            )],
            TestOperation::Sum,
        )
    }
}

fn register_test_operation<const N: usize>(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    key: OpKey,
    operands: u32,
    attributes: [OperationAttributeSchema; N],
    inferencer: TestOperation,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        key,
        OperationSchema::new(
            OperationArity::exact(operands),
            OperationArity::exact(1),
            attributes,
        )
        .unwrap(),
        NormativeDefinitionRef::new("test governed operation semantics")?,
        OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
        OperationConformance::new(CanonicalValue::boolean(true)),
        OperationEffect::Pure,
        Arc::new(inferencer),
    ))
}

/// A provider the packaged graph never reaches.
struct UnusedSemantics {
    revision: u32,
}

impl SemanticRegistryProvider for UnusedSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "unused-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("tiler-test", "unused", 1).unwrap()),
            NormativeDefinitionRef::new("unused test semantics")?,
            TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
        ))
    }
}

fn governed_program(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::new();
    registry
        .register_provider(&GovernedTestSemantics { revision })
        .unwrap();
    build_graph(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}

fn program_with_unused_provider(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::standard().unwrap();
    registry
        .register_provider(&UnusedSemantics { revision })
        .unwrap();
    build_graph(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}
