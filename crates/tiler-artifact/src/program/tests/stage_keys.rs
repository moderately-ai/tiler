//! Artifact stage-key generation and the kernel-program subject it encodes.

use super::super::AbiRoot;
use super::super::model::{STAGE_KEY_DOMAIN, push_component_role, stage_key};
use super::support::graphs::checked_coverage;
use super::support::kernels::pointwise_kernel;
use super::{ELEMENT_BYTES, SCALE_BITS, fused_program, input_shape, semantic_program, strict};
use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec,
    KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec, MemorySpace, PublishingCopy,
    RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode, StageLaunch,
    StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseF32ExpressionBuilder, ReductionTopology,
    RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder,
};

/// The pointwise-only graph the administrative publication control copies.
fn publication_semantic_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().unwrap();
    let input = draft
        .input::<F32>(InputKey::new("input").unwrap(), input_shape())
        .unwrap();
    let scale = F32Constant::apply(&mut draft, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut draft, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = F32Add::apply(&mut draft, product, bias).unwrap();
    draft
        .output(OutputKey::new("published").unwrap(), mapped)
        .unwrap();
    draft.build().unwrap()
}

/// Builds the administrative byte-preserving publication dispatch: one
/// intermediate to one named program output over the same `[2, 3]` extent.
fn publication_copy_kernel() -> VerifiedKernel {
    let elements = 6;
    let mut region = ScheduledRegionBuilder::new(RegionId::new(2));
    region.iteration_shape(input_shape()).unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [(0, TensorRole::Intermediate), (1, TensorRole::Output)] {
        region
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
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let root = expression.input(AccessOrdinal::FIRST).unwrap();
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(expression.build(root).unwrap()),
            numerical: strict(),
        })
        .unwrap();
    region
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
    lower_scheduled_region(&region.build().unwrap()).unwrap()
}

/// Builds an administrative publication stage over a closed, nonempty first
/// stage. The second stage owns no semantic occurrence: its exact ownership is
/// the named output it publishes, which makes all publication-subject fields
/// reachable by the independent stage-key agreement check.
fn publication_owned_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let pointwise = pointwise_kernel();
    let copy = publication_copy_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let external = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ELEMENT_BYTES * 6,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .unwrap();
    let owned = || AllocationSpec {
        capacity_bytes: ELEMENT_BYTES * 6,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership: AllocationOwnership::Program,
    };
    let temporary_allocation = plan.push_allocation(owned()).unwrap();
    let output_allocation = plan.push_allocation(owned()).unwrap();
    let value = |origin, role| MaterializedValueSpec {
        origin,
        role,
        shape: input_shape(),
        storage_scalar: StorageScalar::F32,
        element_type: KernelType::F32,
        encoding: StorageEncoding::Unpacked,
        alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
    };
    let source = plan
        .push_value(
            value(
                MaterializedOrigin::ProgramInput {
                    key: InputKey::new("input").unwrap(),
                },
                ValueRole::Input,
            ),
            external,
        )
        .unwrap();
    let temporary = plan
        .push_value(
            value(MaterializedOrigin::Internal, ValueRole::Temporary),
            temporary_allocation,
        )
        .unwrap();
    let result = plan
        .push_value(
            value(MaterializedOrigin::Internal, ValueRole::Output),
            output_allocation,
        )
        .unwrap();
    let read = plan.push_whole_view(source).unwrap();
    let temporary_view = plan.push_whole_view(temporary).unwrap();
    let write = plan.push_whole_view(result).unwrap();
    let mut literal = |value| {
        plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
            .expect("abi literal")
    };
    let bytes = literal(ELEMENT_BYTES * 6);
    let threads = literal(6);
    let one = literal(1);
    let guard = plan
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("guard predicate");
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
    let coverage = checked_coverage(semantic);

    let first = plan
        .push_stage(
            &pointwise,
            &coverage,
            &[
                StageAccess {
                    view: read,
                    mode: StageAccessMode::Read,
                    accessible_bytes: bytes,
                },
                StageAccess {
                    view: temporary_view,
                    mode: StageAccessMode::Write,
                    accessible_bytes: bytes,
                },
            ],
            StageLaunch {
                grid_threads: threads,
                threads_per_workgroup: one,
            },
        )
        .unwrap();
    let publisher = plan
        .push_stage(
            &copy,
            &[],
            &[
                StageAccess {
                    view: temporary_view,
                    mode: StageAccessMode::Read,
                    accessible_bytes: bytes,
                },
                StageAccess {
                    view: write,
                    mode: StageAccessMode::Write,
                    accessible_bytes: bytes,
                },
            ],
            StageLaunch {
                grid_threads: threads,
                threads_per_workgroup: one,
            },
        )
        .unwrap();
    plan.push_data_dependency(first, publisher, temporary)
        .unwrap();
    plan.push_publishing_copy(PublishingCopy {
        source_stage: first,
        publisher,
        source: temporary,
        published: result,
    })
    .unwrap();
    plan.push_output(OutputKey::new("published").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

/// Reconstructs the historical stage-key payload: the bound kernel identity and
/// the bare coverage ordinals, with no refinement evidence beside them.
///
/// `v1` and `v2` share this payload byte for byte and differ only in their
/// separator, because the canonical-coverage step *reinterpreted* those raw
/// ordinals rather than changing them. `v3` adds proof evidence and `v4`
/// replaces this coverage-only grammar with tagged complete ownership, so each
/// historical reconstruction stays local rather than reusing production code.
fn coverage_only_stage_key(stage: tiler_ir::program::StageRef<'_>, domain: &[u8]) -> Vec<u8> {
    let mut bytes = domain.to_vec();
    push_slice(&mut bytes, stage.kernel().canonical_identity().as_bytes());
    push_len(&mut bytes, stage.coverage().len());
    for covered in stage.coverage() {
        bytes.extend_from_slice(&covered.occurrence().get().to_be_bytes());
    }
    bytes
}

/// Reconstructs the `v3` proof-bound-coverage grammar before `v4` introduced
/// the owner tag, claim count, continuation ordinal, and publication arm.
fn proof_bound_coverage_stage_key(
    stage: tiler_ir::program::StageRef<'_>,
    domain: &[u8],
) -> Vec<u8> {
    let mut bytes = domain.to_vec();
    push_slice(&mut bytes, stage.kernel().canonical_identity().as_bytes());
    push_len(&mut bytes, stage.coverage().len());
    for covered in stage.coverage() {
        bytes.extend_from_slice(&covered.occurrence().get().to_be_bytes());
        push_slice(&mut bytes, covered.refinement().as_bytes());
    }
    bytes
}

#[test]
fn each_artifact_stage_key_generation_is_separated_from_the_last() {
    const V1: &[u8] = b"tiler.artifact-program.stage.v1\0";
    const V2: &[u8] = b"tiler.artifact-program.stage.v2\0";
    const V3: &[u8] = b"tiler.artifact-program.stage.v3\0";
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let stage = program.stages().next().expect("one fused stage");
    let current = stage_key(&program, stage);
    let v1 = coverage_only_stage_key(stage, V1);
    let v2 = coverage_only_stage_key(stage, V2);
    let v3 = proof_bound_coverage_stage_key(stage, V3);

    assert!(current.starts_with(STAGE_KEY_DOMAIN));
    assert!(!current.starts_with(V1));
    assert!(!current.starts_with(V2));
    assert!(!current.starts_with(V3));
    // v1 → v2 moved the separator over an unchanged payload, because the step
    // reinterpreted the ordinals rather than rewriting them.
    assert_eq!(
        v1[V1.len()..],
        v2[V2.len()..],
        "v1 and v2 spell the same coverage payload"
    );
    assert_ne!(v1, v2);
    // v2 → v3 adds each proof record, then v3 → v4 replaces the coverage count
    // with one tagged owner arm and its count plus realization ordinal.
    assert!(v3.len() > v2.len());
    assert!(current.len() > v3.len());
    assert_ne!(current, v3);
    assert_ne!(current, v2);
    assert_ne!(current, v1);
}

/// Independently reconstructs the complete subject shared by the kernel-program
/// stage section and an artifact stage key, excluding only their own domains.
///
/// This is deliberately test-local rather than a call to `stage_key`: the
/// production writers must remain independent, and this third spelling makes a
/// changed owner tag, count framing, ordinal, proof record, output key, or
/// component role fail visibly. The verified program guarantees one closed
/// owner, so this reconstruction may fail loudly instead of supplying a
/// fallback owner of its own.
fn independently_encoded_complete_stage_subject(
    program: &VerifiedKernelProgram,
    stage: tiler_ir::program::StageRef<'_>,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, stage.kernel().canonical_identity().as_bytes());

    let mut realization: Vec<_> = stage
        .coverage()
        .iter()
        .cloned()
        .map(|covered| (0_u32, covered))
        .collect();
    let mut occurrences: Vec<_> = program
        .partial_reductions()
        .filter(|split| split.combiner() == stage)
        .map(tiler_ir::program::PartialReductionRef::occurrence)
        .chain(
            program
                .staged_realizations()
                .filter(|row| row.consumer() == stage)
                .map(tiler_ir::program::StagedRealizationRef::occurrence),
        )
        .collect();
    occurrences.sort_unstable();
    occurrences.dedup();
    for occurrence in occurrences {
        let (root, proof) = program
            .stages()
            .find_map(|candidate| {
                candidate
                    .coverage()
                    .iter()
                    .find(|covered| covered.occurrence() == occurrence)
                    .cloned()
                    .map(|covered| (candidate, covered))
            })
            .expect("verified continuation has one proof-bound root");
        let mut current = root;
        let mut ordinal = 0_u32;
        while let Some(next) = program
            .partial_reductions()
            .filter(|split| split.occurrence() == occurrence)
            .map(|split| (split.producer(), split.combiner()))
            .chain(
                program
                    .staged_realizations()
                    .filter(|row| row.occurrence() == occurrence)
                    .map(|row| (row.producer(), row.consumer())),
            )
            .find(|(producer, _)| *producer == current)
            .map(|(_, consumer)| consumer)
        {
            ordinal = ordinal.saturating_add(1);
            if next == stage {
                realization.push((ordinal, proof.clone()));
                break;
            }
            current = next;
        }
    }
    realization.sort_by(|left, right| {
        left.1
            .occurrence()
            .get()
            .cmp(&right.1.occurrence().get())
            .then_with(|| left.0.cmp(&right.0))
    });

    let mut publication: Vec<_> = program
        .publishing_copies()
        .filter(|copy| copy.publisher() == stage)
        .map(|copy| {
            let published = copy.published();
            let output = program
                .outputs()
                .find(|output| output.value() == published)
                .expect("verified publishing owner names one output component");
            (output.key().clone(), published.component_role())
        })
        .collect();
    publication.sort_by(|left, right| {
        left.0
            .as_str()
            .cmp(right.0.as_str())
            .then_with(|| left.1.cmp(&right.1))
    });

    match (realization.is_empty(), publication.is_empty()) {
        (false, true) => {
            bytes.push(0x01);
            push_len(&mut bytes, realization.len());
            for (ordinal, covered) in realization {
                bytes.extend_from_slice(&ordinal.to_be_bytes());
                bytes.extend_from_slice(&covered.occurrence().get().to_be_bytes());
                push_slice(&mut bytes, covered.refinement().as_bytes());
            }
        }
        (true, false) => {
            bytes.push(0x02);
            push_len(&mut bytes, publication.len());
            for (key, role) in publication {
                push_slice(&mut bytes, key.as_str().as_bytes());
                push_component_role(&mut bytes, role);
            }
        }
        _ => panic!("verified program carries exactly one complete stage owner"),
    }
    bytes
}

/// The independently serialized artifact stage key and kernel-program stage
/// section agree on the complete owner subject, including every realization
/// and publication field.
#[test]
fn the_artifact_stage_key_encodes_the_complete_kernel_program_stage_subject() {
    let realization_semantic = semantic_program();
    let publication_semantic = publication_semantic_program();
    let programs = [
        (
            "realization",
            fused_program(&realization_semantic, SCALE_BITS),
        ),
        (
            "publication",
            publication_owned_program(&publication_semantic),
        ),
    ];

    let mut realization_stages = 0_usize;
    let mut publication_stages = 0_usize;
    for (kind, program) in &programs {
        let identity = program.canonical_identity().as_bytes();
        for stage in program.stages() {
            let key = stage_key(program, stage);
            let subject = independently_encoded_complete_stage_subject(program, stage);
            assert_eq!(&key[STAGE_KEY_DOMAIN.len()..], subject);
            assert_eq!(
                identity
                    .windows(subject.len())
                    .filter(|window| *window == subject)
                    .count(),
                1,
                "the complete {kind} stage subject, including owner tag and count framing, \
                 must appear exactly once in kernel-program identity",
            );
            if stage.coverage().is_empty() {
                publication_stages += 1;
                assert_eq!(*kind, "publication");
                assert_eq!(program.publishing_copies().count(), 1);
                assert_eq!(
                    program.outputs().next().expect("one output").key().as_str(),
                    "published"
                );
            } else {
                realization_stages += 1;
            }
        }
    }
    assert_eq!(
        realization_stages, 2,
        "the two controls retain their computing owners"
    );
    assert_eq!(
        publication_stages, 1,
        "the administrative control reaches the publication arm"
    );
}
