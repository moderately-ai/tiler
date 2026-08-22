//! What a verified artifact is, and what a consumer reads off it.

use super::super::{
    AbiFactBinder, AbiFacts, AbiType, AbiValue, AvailabilityPhase, BindingKind, BindingTarget,
};
use super::support::graphs::checked_coverage;
use super::support::kernels::declare_program_contract;
use super::{
    BIAS_BITS, ELEMENT_BYTES, SCALE_BITS, SCRATCH_OFFSET, build_artifact, default_artifact,
    fused_kernel, input_shape, lowering_provider, output_shape, partial_window_artifact, profile,
    strict, strict_affine_u4_dequantize_artifact,
};
use tiler_ir::kernel::KernelType;
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec,
    KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec, MemorySpace, StorageEncoding,
    StorageScalar, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, STRICT_AFFINE_CODES_ROLE,
    STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::Axis;

/// Builds the fixture graph publishing its one reduction under two names.
///
/// `SemanticProgramBuilder::output_resolved` rejects a repeated *key* and not a
/// repeated *value*, so two named outputs may name one value all the way down to
/// one materialized program value and one buffer. That is the case a binding
/// target carrying a single output key would encode wrongly, so the fixture
/// exists to make it reachable rather than argued about.
fn dual_output_semantic_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().unwrap();
    let input = draft
        .input::<F32>(InputKey::new("input").unwrap(), input_shape())
        .unwrap();
    let scale = F32Constant::apply(&mut draft, SCALE_BITS).unwrap();
    let bias = F32Constant::apply(&mut draft, BIAS_BITS).unwrap();
    let product = F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = F32Add::apply(&mut draft, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)]).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    draft.output(OutputKey::new("copy").unwrap(), sum).unwrap();
    draft.build().unwrap()
}

/// Builds the single-stage plan that publishes one value under both names.
fn dual_output_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let kernel = fused_kernel(SCALE_BITS);
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let external = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 24,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .unwrap();
    let owned = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 8,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
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
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
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
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
                memory_space: MemorySpace::Device,
            },
            owned,
        )
        .unwrap();
    let read = plan.push_whole_view(source).unwrap();
    let write = plan.push_whole_view(result).unwrap();
    let (accesses, launch) = declare_program_contract(&mut plan, read, write);
    plan.push_stage(&kernel, &checked_coverage(semantic), &accesses, launch)
        .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.push_output(OutputKey::new("copy").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

// -------------------------------------------------------------------------
// Verified-product construction and consumability
// -------------------------------------------------------------------------

#[test]
fn builds_a_verified_single_variant_artifact() {
    let artifact = default_artifact();
    assert_eq!(artifact.variants().len(), 1);
    assert_eq!(artifact.payloads().len(), 1);
    assert_eq!(artifact.selected_lowering_providers().len(), 1);
    assert_eq!(artifact.schema(), super::super::ArtifactSchema::GOVERNED);
    assert_eq!(
        artifact.routing_policy(),
        super::super::RoutingPolicy::StablePriority
    );
    let input = artifact.inputs().next().expect("one declared input");
    assert_eq!(input.key().as_str(), "input");
    assert_eq!(
        input.static_shape(),
        Some(input_shape()),
        "a wholly literal boundary still reads back as one fixed shape",
    );
    assert_eq!(
        input
            .components()
            .next()
            .expect("one dense component")
            .access_type(),
        KernelType::F32
    );
    assert_eq!(
        input
            .components()
            .next()
            .expect("one dense component")
            .storage_scalar(),
        StorageScalar::F32
    );
    let output = artifact.outputs().next().expect("one declared output");
    assert_eq!(output.key().as_str(), "result");
    assert_eq!(output.static_shape(), Some(output_shape()));
}

#[test]
fn strict_affine_components_survive_the_builder_derived_artifact_boundary() {
    let artifact = strict_affine_u4_dequantize_artifact();
    assert_ne!(
        artifact.canonical_identity(),
        default_artifact().canonical_identity()
    );
    let input = artifact.inputs().next().expect("strict-affine input");
    assert!(!input.resolved_type_encoding().is_empty());
    let components: Vec<_> = input.components().collect();
    assert_eq!(
        components
            .iter()
            .map(|component| component.role())
            .collect::<Vec<_>>(),
        [
            Some(STRICT_AFFINE_CODES_ROLE),
            Some(STRICT_AFFINE_SCALE_ROLE),
            Some(STRICT_AFFINE_ZERO_POINT_ROLE),
        ]
    );
    assert_eq!(
        components
            .iter()
            .map(|component| (
                component.storage_scalar(),
                component.storage_encoding(),
                component.access_type(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                StorageScalar::U8,
                StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
                KernelType::U8,
            ),
            (
                StorageScalar::F32,
                StorageEncoding::Unpacked,
                KernelType::F32,
            ),
            (StorageScalar::U8, StorageEncoding::Unpacked, KernelType::U8,),
        ]
    );
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let bindings: Vec<_> = entry.bindings().collect();
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.component_role())
            .collect::<Vec<_>>(),
        [
            Some(STRICT_AFFINE_CODES_ROLE),
            Some(STRICT_AFFINE_SCALE_ROLE),
            Some(STRICT_AFFINE_ZERO_POINT_ROLE),
            None,
        ]
    );
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.window().length)
            .collect::<Vec<_>>(),
        [3, 4, 1, 20]
    );
    let input_key = InputKey::new("input").expect("input key");
    let output_key = OutputKey::new("result").expect("output key");
    for binding in &bindings[..3] {
        assert_eq!(binding.target(), BindingTarget::ProgramInput(&input_key));
    }
    assert_eq!(
        bindings[3].target(),
        BindingTarget::ProgramOutput(std::slice::from_ref(&output_key))
    );
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
    assert_eq!(
        entry
            .payload(0)
            .expect("the sole delivery position")
            .representation
            .as_str(),
        "metallib",
    );
    assert_eq!(entry.payloads().len(), 1);
    let bindings: Vec<_> = entry.bindings().collect();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].slot(), 0);
    assert_eq!(bindings[0].kind(), BindingKind::Buffer);
    assert_eq!(bindings[0].access_type(), KernelType::F32);
    assert_eq!(bindings[0].storage_scalar(), StorageScalar::F32);
    assert_eq!(bindings[0].value_role(), ValueRole::Input);
    assert_eq!(
        bindings[0].alignment(),
        AlignmentRequirement::natural_for(StorageScalar::F32)
    );
    assert_eq!(bindings[0].window().length, 24);
    assert_eq!(bindings[1].value_role(), ValueRole::Output);
    assert_eq!(bindings[1].window().length, 8);
    // The same correspondence the shared-IR walk above reads, spelled as the
    // interface reference the artifact carries — this is the one a consumer
    // holding only bytes can follow.
    let result = OutputKey::new("result").unwrap();
    assert_eq!(
        bindings[0].target(),
        super::super::BindingTarget::ProgramInput(&InputKey::new("input").unwrap()),
    );
    assert_eq!(
        bindings[1].target(),
        super::super::BindingTarget::ProgramOutput(std::slice::from_ref(&result)),
    );
    // The plan itself is reachable through the shared IR's own views.
    assert_eq!(variant.program().stages().len(), 1);
    assert_eq!(bindings[0].value().required_bytes(), 24);
}

/// One buffer published under two names carries both, rather than one of them.
///
/// The failure this excludes is not a missing accessor. A target carrying a
/// single output key would name whichever the producer's declaration order put
/// first, and a loader would bind a second buffer for the other name — two
/// buffers for one value, with the unbound one never written. Carrying the
/// complete set is what makes "one buffer, two names" expressible at all.
#[test]
fn a_value_published_under_two_names_carries_both_in_its_binding_target() {
    let semantic = dual_output_semantic_program();
    let program = dual_output_program(&semantic);
    let provider = lowering_provider(1);
    let artifact = build_artifact(&semantic, &program, provider.clone(), &[provider]);
    let bindings: Vec<_> = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry")
        .bindings()
        .collect();
    let super::super::BindingTarget::ProgramOutput(keys) = bindings[1].target() else {
        panic!("the written binding addresses published output storage");
    };
    // Canonically ordered rather than in declaration order, so the artifact's
    // identity does not fold the order a producer happened to publish in.
    assert_eq!(
        keys.iter().map(OutputKey::as_str).collect::<Vec<_>>(),
        ["copy", "result"],
    );
    assert_eq!(bindings[1].value_role(), ValueRole::Output);
}

/// A slot may address part of the value it names, and it says where.
///
/// The plan sizes one program-owned scratch buffer for two working sets and puts
/// the one its two stages exchange in the upper half. Both stages therefore bind
/// the *same* internal value at byte 24 of 48. What this excludes is the failure
/// the refusal it replaces existed to prevent: a record carrying an extent and no
/// placement leaves a loader binding the right buffer at byte zero, which is a
/// silently wrong dispatch rather than a rejection.
#[test]
fn a_binding_may_address_part_of_the_value_it_names() {
    let artifact = partial_window_artifact();
    let facts = bound_facts();
    let entries: Vec<_> = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .collect();
    assert_eq!(entries.len(), 2);
    let pointwise: Vec<_> = entries[0].bindings().collect();
    let reduction: Vec<_> = entries[1].bindings().collect();

    // The scratch slot of each entry: written by the first stage, read by the
    // second, and the same materialized value in both.
    for scratch in [pointwise[1], reduction[0]] {
        assert_eq!(scratch.target(), super::super::BindingTarget::Internal);
        assert_eq!(scratch.value_role(), ValueRole::Temporary);
        // Partial in the exact sense that matters: the window is shorter than
        // the value, and starts inside it.
        assert_eq!(scratch.value().required_bytes(), ELEMENT_BYTES * 12);
        assert_eq!(scratch.window().offset, SCRATCH_OFFSET);
        assert_eq!(scratch.window().length, ELEMENT_BYTES * 6);
        assert_eq!(
            scratch.accessible_offset().evaluate(&facts).unwrap(),
            AbiValue::Unsigned(SCRATCH_OFFSET),
        );
        assert_eq!(
            scratch.accessible_bytes().evaluate(&facts).unwrap(),
            AbiValue::Unsigned(ELEMENT_BYTES * 6),
        );
    }

    // The interface slots address their values whole, and say that too.
    for whole in [pointwise[0], reduction[1]] {
        assert_eq!(whole.window().offset, 0);
        assert_eq!(
            whole.accessible_offset().evaluate(&facts).unwrap(),
            AbiValue::Unsigned(0),
        );
    }
}

/// Binds the fixture interface's declared shapes as an evaluation environment.
fn bound_facts() -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_shape(&InputKey::new("input").unwrap(), &input_shape())
        .unwrap();
    binder.build()
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
