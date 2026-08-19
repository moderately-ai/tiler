//! The strict-affine U4 encoded-input fixture, graph through packaged artifact.

use super::super::super::{
    AbiRoot, ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey, BackendEntryRef,
    BackendKey, BackendPayloadDescriptor, BindingKind, BindingSpec, CompilationEnvironment,
    EntrySpec, LaunchSpec, PayloadDigest, RepresentationKey, SchemaVersion, SelectedProvider,
    VariantSpec, VerifiedArtifactProgram,
};
use super::artifacts::{declare_realization, lowering_subject, profile, rules};
use super::graphs::{strict, strict_contract};
use tiler_ir::index::{
    DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexRealizationAuthority,
    IndexRefinementSubject, IndexRefinementVerificationOutcome, IndexRegionBuilder,
    ScalarAttributes, TensorRole as IndexTensorRole, strict_affine_u4_dequantize_scalar_op,
};
use tiler_ir::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec,
    CoveredOccurrence, KernelProgramBuilder, MaterializedComponentSpec, MaterializedOrigin,
    MaterializedValueSpec, MemorySpace, RoutingCommitState, RoutingCommitTransition, StageAccess,
    StageAccessMode, StageLaunch, StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ExecutionBinding,
    KernelSchedule, LaunchPlan, LogicalAccess, OwnershipProof, OwnershipProofKind,
    OwnershipWitnessId, ReductionTopology, RegionId, RegionProgram, ScalarProgram,
    ScheduledRegionBuilder, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    InputKey, OperationAttributes, OutputKey, ProviderIdentity, STRICT_AFFINE_CODES_ROLE,
    STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE, SemanticProgram,
    SemanticProgramBuilder, StrictAffineU4, dequantize_strict_affine_op,
};
use tiler_ir::shape::Shape;

// -------------------------------------------------------------------------
pub(crate) fn strict_affine_u4_dequantize_semantic() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let input = draft
        .input_resolved(
            InputKey::new("input").expect("input key"),
            Shape::from_dims([5]),
            StrictAffineU4::resolved_type(),
        )
        .expect("strict-affine input");
    let output = draft
        .apply(
            dequantize_strict_affine_op(),
            OperationAttributes::empty(),
            &[input],
        )
        .expect("strict-affine dequantization")[0];
    draft
        .output_resolved(OutputKey::new("result").expect("output key"), output)
        .expect("dense output");
    draft.build().expect("verified semantic program")
}

/// Mints the strict-affine fixture's real IR-owned refinement receipt.
///
/// Governed compilation is not available here and the reason is not a fixture
/// gap: the current target profile supports dense F32 only, so a graph with an
/// encoded input is refused before physical planning. That must not tempt this
/// fixture to forge coverage. Instead it builds the candidate region through
/// the public index builder and submits it to the same sealed authority the
/// compiler would use. The verifier compares canonical identities and is the
/// only code that can mint the receipt retained below, so this coverage is as
/// real as the compiled fixtures' — it simply reaches the verifier by the other
/// public door.
pub(crate) fn strict_affine_checked_coverage(semantic: &SemanticProgram) -> Vec<CoveredOccurrence> {
    let scalars = FrozenScalarRegistry::standard().expect("the standard scalar authority freezes");
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(
        semantic.semantic_registry().clone(),
        scalars.clone(),
    )
    .expect("the standard scalar and semantic authorities cohere");
    let operation = semantic
        .operations()
        .next()
        .expect("the fixture graph has one operation")
        .id();
    let subject = IndexRefinementSubject::derive(semantic, operation, strict_contract())
        .expect("the strict-affine subject derives");
    let authority = IndexRealizationAuthority::admit(
        semantic.semantic_registry(),
        &scalars,
        subject.operation().clone(),
        subject.signature().clone(),
        &[strict_affine_u4_dequantize_scalar_op()],
    )
    .expect("the strict-affine emission ceiling is admissible");
    let resolution = laws
        .resolve(&subject)
        .expect("the strict-affine law resolves");

    let [input] = subject.inputs() else {
        panic!("the strict-affine subject has one input")
    };
    let [result] = subject.results() else {
        panic!("the strict-affine subject has one result")
    };
    let (_, encoded) = input
        .value_type()
        .encoded_numeric_parts()
        .expect("the input is an encoded numeric value");
    let mut region = IndexRegionBuilder::new(scalars).expect("an index region builder");
    let dimensions = result
        .shape()
        .extents()
        .iter()
        .copied()
        .map(|extent| {
            region
                .dimension(DomainRole::Parallel, extent)
                .expect("a parallel dimension")
        })
        .collect::<Vec<_>>();
    let coordinates = dimensions
        .iter()
        .copied()
        .map(|dimension| {
            region
                .dimension_expr(dimension)
                .expect("a dimension coordinate")
        })
        .collect::<Vec<_>>();
    let tensors = encoded
        .components()
        .iter()
        .map(|component| {
            region
                .tensor(
                    IndexTensorRole::Input,
                    component.resolved_type().clone(),
                    component.shape_relation().component_shape(input.shape()),
                )
                .expect("an encoded component tensor")
        })
        .collect::<Vec<_>>();
    let codes = region
        .read(tensors[0], &dimensions, &coordinates)
        .expect("the codes read");
    let scale = region.read(tensors[1], &[], &[]).expect("the scale read");
    let zero_point = region
        .read(tensors[2], &[], &[])
        .expect("the zero-point read");
    let decoded = region
        .apply(
            strict_affine_u4_dequantize_scalar_op(),
            ScalarAttributes::empty(),
            &[codes, scale, zero_point],
        )
        .expect("the strict-affine decode applies")
        .get(0)
        .expect("one decoded result");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the dense output tensor");
    let write = region
        .write(output, &dimensions, &coordinates)
        .expect("the dense output write");
    region.output(write, decoded).expect("the output root");
    let region = region.build().expect("a verified canonical index region");
    match resolution
        .verify(&authority, &region)
        .expect("the candidate region realizes the governed strict-affine law")
    {
        IndexRefinementVerificationOutcome::Verified(receipt) => {
            vec![CoveredOccurrence::from_receipt(&receipt)]
        }
        IndexRefinementVerificationOutcome::Pending(_) => {
            panic!("the static strict-affine region retains no residual proof obligation")
        }
    }
}

pub(crate) fn strict_affine_u4_dequantize_kernel() -> VerifiedKernel {
    let logical_elements = 5;
    let owner = OwnershipWitnessId::new(0);
    let mut region = ScheduledRegionBuilder::new(RegionId::new(17));
    region
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
        region.push_access(access).expect("component access");
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
        region
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(id),
                tensor,
                component_role,
                kind: BoundsProofKind::LinearRange { element_count },
            })
            .expect("component bounds");
    }
    region
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: logical_elements,
            },
        })
        .expect("output ownership");
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictAffineU4Dequantize {
                codes_role: STRICT_AFFINE_CODES_ROLE,
                scale_role: STRICT_AFFINE_SCALE_ROLE,
                zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            },
            numerical: strict(),
        })
        .expect("strict-affine scalar program");
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: logical_elements,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: owner,
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: logical_elements,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("strict-affine schedule");
    lower_scheduled_region(&region.build().expect("verified schedule"))
        .expect("verified strict-affine kernel")
}

pub(crate) fn strict_affine_u4_dequantize_program(
    semantic: &SemanticProgram,
) -> VerifiedKernelProgram {
    let kernel = strict_affine_u4_dequantize_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).expect("program builder");
    let mut component = |role, shape, storage_scalar, element_type, encoding, bytes| {
        let allocation = plan
            .push_allocation(AllocationSpec {
                capacity_bytes: bytes,
                alignment: AlignmentGuarantee::natural_for(storage_scalar),
                memory_space: MemorySpace::Device,
                ownership: AllocationOwnership::External,
            })
            .expect("component allocation");
        let value = plan
            .push_component_value(
                MaterializedComponentSpec {
                    origin: MaterializedOrigin::ProgramInput {
                        key: InputKey::new("input").expect("input key"),
                    },
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
        plan.push_whole_view(value).expect("component view")
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
    let output_allocation = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 20,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .expect("output allocation");
    let output_value = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::Internal,
                role: ValueRole::Output,
                shape: Shape::from_dims([5]),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
                memory_space: MemorySpace::Device,
            },
            output_allocation,
        )
        .expect("dense output");
    let output = plan.push_whole_view(output_value).expect("output view");
    let mut literal = |value| {
        plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
            .expect("ABI literal")
    };
    let codes_bytes = literal(3);
    let scale_bytes = literal(4);
    let zero_point_bytes = literal(1);
    let output_bytes = literal(20);
    let grid_threads = literal(5);
    let threads_per_workgroup = literal(1);
    let guard = plan
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("applicability guard");
    plan.applicability_guard(guard)
        .expect("applicability guard");
    plan.push_stage(
        &kernel,
        &strict_affine_checked_coverage(semantic),
        &[
            StageAccess {
                view: codes,
                mode: StageAccessMode::Read,
                accessible_bytes: codes_bytes,
            },
            StageAccess {
                view: scale,
                mode: StageAccessMode::Read,
                accessible_bytes: scale_bytes,
            },
            StageAccess {
                view: zero_point,
                mode: StageAccessMode::Read,
                accessible_bytes: zero_point_bytes,
            },
            StageAccess {
                view: output,
                mode: StageAccessMode::Write,
                accessible_bytes: output_bytes,
            },
        ],
        StageLaunch {
            grid_threads,
            threads_per_workgroup,
        },
    )
    .expect("strict-affine stage");
    plan.push_output(OutputKey::new("result").expect("output key"), output_value)
        .expect("published output");
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
        .expect("routing transition");
    }
    plan.build().expect("verified strict-affine program")
}

pub(crate) fn strict_affine_u4_dequantize_artifact() -> VerifiedArtifactProgram {
    let semantic = strict_affine_u4_dequantize_semantic();
    let program = strict_affine_u4_dequantize_program(&semantic);
    let provider =
        ProviderIdentity::new("tiler-test", "strict-affine-u4-dequantize", 1).expect("provider");
    let environment = CompilationEnvironment::new([provider.clone()]).expect("environment");
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).expect("artifact builder");
    draft
        .select_provider(SelectedProvider {
            provider,
            capability: lowering_subject("tiler", "strict-affine-u4-dequantize", 1),
            capability_revision: 1,
        })
        .expect("selected provider");
    let payload = draft
        .push_payload(BackendPayloadDescriptor {
            environment: None,
            backend: BackendKey::new("tiler.test.target-neutral").expect("backend"),
            representation: RepresentationKey::new("structural-kir-record")
                .expect("representation"),
            payload_schema: SchemaVersion::new(1, 0),
            digest: PayloadDigest::from_bytes([0xd4, 0x04]).expect("payload digest"),
            compatibility: profile(),
            execution_policy: ArtifactExecutionPolicy::NativeImage,
        })
        .expect("payload");
    draft
        .push_variant(
            &program,
            VariantSpec {
                target_profile: profile(),
                feasibility_rules: rules(),
                deferred_predicates: Vec::new(),
                entries: vec![EntrySpec {
                    bindings: (0..4)
                        .map(|_| BindingSpec {
                            kind: BindingKind::Buffer,
                        })
                        .collect(),
                    launch: LaunchSpec {
                        zero_work_skips_dispatch: true,
                        preconditions: Vec::new(),
                    },
                    implementation: BackendEntryRef {
                        payloads: vec![payload],
                        entry_key: BackendEntryKey::from_bytes(b"strict-affine-u4-dequantize")
                            .expect("entry key"),
                    },
                }],
            },
        )
        .expect("strict-affine variant");
    declare_realization(&mut draft, &program);
    draft.build().expect("verified strict-affine artifact")
}
