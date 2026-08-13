//! Bounded tests for the target-neutral artifact program model.
//!
//! Fixtures package real verified kernel programs over real verified semantic
//! programs, so every rejection is a rejection of a plan that the shared IR
//! itself already accepted. Nothing here asserts that a kernel computes the
//! operations its stage covers; that remains compiler-owned evidence.

use std::collections::HashMap;
use std::sync::Arc;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{
    DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexInteger,
    IndexRealizationAuthority, IndexRealizationLaw, IndexRefinementSubject,
    IndexRefinementVerificationOutcome, IndexRegionBuilder, NumericalContractIdentity, ScalarArity,
    ScalarAttributeSchema, ScalarAttributes, ScalarEffect, ScalarInferenceError,
    ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract,
    ScalarOperationDefinition, ScalarOperationInferencer, ScalarRegistryBuilder,
    TensorRole as IndexTensorRole, VerifiedIndexRegion, add_bf16_scalar_op, add_f32_scalar_op,
    constant_bf16_scalar_op, constant_f32_scalar_op, multiply_bf16_scalar_op,
    multiply_f32_scalar_op, strict_affine_u4_dequantize_scalar_op,
};
use tiler_ir::kernel::{
    KernelType, MAX_KERNEL_IDENTITY_BYTES, VerifiedKernel, lower_scheduled_region,
};
use tiler_ir::program::abi::{
    ExprNode, PreparedEntryTargetRequirement, TargetPropertyProviderIdentity, TargetPropertyQuery,
    TargetPropertyRequirementRelation,
};
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec,
    BitPackedEncoding, ByteWindow, CoveredOccurrence, KernelProgramBuilder,
    MaterializedComponentSpec, MaterializedOrigin, MaterializedValueSpec, MemorySpace,
    PackedBitOrder, PackedTailRule, RoutingCommitState, RoutingCommitTransition, StageAccess,
    StageAccessMode, StageLaunch, StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram,
    ViewId,
};
use tiler_ir::schedule::{
    Access, AccessMode, ApproximationEnvelope, ArithmeticType, Bf16NumericalContractKey,
    BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder, ExceptionalValueAssumption,
    ExecutionBinding, F32NumericalContractKey, FencedSpaces, FlushedZeroSign, InputOrdinal,
    KernelSchedule, LaunchPlan, LogicalAccess, MaterializationRounding, MemoryOrdering,
    NumericalPermission, NumericalRealization, OwnershipProof, OwnershipProofKind,
    OwnershipWitnessId, PointwiseBf16ExpressionBuilder, PointwiseF32ExpressionBuilder,
    ReductionTopology, RegionId, ScalarProgram, ScheduledRegionBuilder, SubnormalMode,
    SynchronizationKind, SynchronizationScope, SynchronizationSubject, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    AttributeFieldId, BF16_CONSTANT_BITS_ATTRIBUTE, Bf16, Bf16Add, Bf16Constant, Bf16Multiply,
    CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind, CanonicalValueView,
    F32, F32_CONSTANT_BITS_ATTRIBUTE, F32Add, F32Constant, F32Multiply, FrozenSemanticRegistry,
    InputKey, NormativeDefinitionRef, OpKey, OperationArity, OperationAttributeSchema,
    OperationAttributes, OperationConformance, OperationDefinition, OperationDefinitionFacts,
    OperationEffect, OperationId, OperationInferenceError, OperationInferenceOutputs,
    OperationInferenceRequest, OperationInferencer, OperationSchema, OutputKey,
    ProviderDiagnosticCode, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE, RegistryError,
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
    SemanticProgram, SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, StrictAffineU4, StrictSerialF32Sum, TypeDefinitionFacts, TypeKey,
    ValueFact, ValueTypeDefinition, ValueTypeDefinitionKey, add_bf16_op, add_f32_op,
    constant_bf16_op, constant_f32_op, dequantize_strict_affine_op, multiply_bf16_op,
    multiply_f32_op, strict_serial_sum_f32_op,
};
use tiler_ir::shape::{
    Axis, BindingSource, Extent, FactProvenance, RootBinding, Shape, ShapeEnvBuilder, ShapeSymbol,
    SourcedExtent, SymbolScope,
};

use tiler_ir::numerics::{CANONICAL_DIMENSIONS, DIMENSION_COUNT};

use super::BackendFeatureRequirement;
use super::model::{
    ARTIFACT_DOMAIN_LABEL, LENGTH_BYTES, STAGE_KEY_DOMAIN, framed, push_storage_encoding,
    push_synchronization, stage_key,
};
use super::{
    AbiBinaryOp, AbiEvaluationError, AbiExprId, AbiFactBinder, AbiFacts, AbiRoot, AbiType,
    AbiUnaryOp, AbiValue, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind,
    ArtifactExecutionPolicy, ArtifactKeyKind, ArtifactProgramBuilder, AvailabilityPhase,
    BackendEntryKey, BackendEntryRef, BackendKey, BackendPayloadDescriptor, BindingKind,
    BindingSpec, BindingTarget, CapabilityKey, CompilationEnvironment, DeferredPredicateSpec,
    EntrySpec, FeasibilityRuleSetKey, FeasibilityRuleSetRef, LaunchSpec,
    MAX_ARTIFACT_IDENTITY_BYTES, MAX_ROUTE_FEATURE_PAYLOAD_BYTES, PayloadDigest, PayloadId,
    RecordedArtifactIdentityError, RecordedArtifactProgramIdentity, RepresentationKey,
    RouteFeatureKey, RouteRequirement, RouteRequirementError, RouteRequirementSubject,
    RouteResourceDimension, RouteResourceRequirement, SchemaVersion, SelectedProvider,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, TargetPropertyKey,
    VariantSpec, VerifiedArtifactProgram,
};
use super::{
    DeliveredRealizationBuilder, DeliveredRealizationRecord, DimensionBehaviour, EntryRealization,
    FactSourceProvenance, HonouringMeans, NumericalDimension, NumericalObligationKey, PolicyLocus,
    ProvenanceIdentity, ScalarArithmeticSubject, SemanticOccurrence, TargetEvidenceDeclaration,
    overlapping_behaviour,
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

/// Declares the ABI, applicability guard, and routing-commit contract that both
/// single-stage kernel-program fixtures in this file share.
///
/// A verified kernel program states its own entry ABI since
/// `complete-program-identity-with-abi-guards-and-routing`, and folds it into
/// its canonical identity. The quantities are the fused kernel's: a whole
/// `[2, 3]` `f32` read, a whole `[2]` `f32` write, and a launch of two threads
/// at one thread per workgroup.
///
/// This is deliberately *not* the artifact-side ABI a `VariantSpec` declares.
/// That one lives on the artifact's own arena, under its own separately
/// versioned schema, and is asserted against the same program facts.
fn declare_program_contract(
    plan: &mut KernelProgramBuilder,
    read: ViewId,
    write: ViewId,
) -> ([StageAccess; 2], StageLaunch) {
    let mut literal = |value: u64| {
        plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
            .expect("abi literal")
    };
    let read_bytes = literal(ELEMENT_BYTES * 6);
    let write_bytes = literal(ELEMENT_BYTES * 2);
    let grid_threads = literal(2);
    let threads_per_workgroup = literal(1);
    let guard = plan
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("guard predicate");
    plan.applicability_guard(guard)
        .expect("applicability guard");
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
        .expect("routing-commit transition");
    }
    (
        [
            StageAccess {
                view: read,
                mode: StageAccessMode::Read,
                accessible_bytes: read_bytes,
            },
            StageAccess {
                view: write,
                mode: StageAccessMode::Write,
                accessible_bytes: write_bytes,
            },
        ],
        StageLaunch {
            grid_threads,
            threads_per_workgroup,
        },
    )
}

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
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
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

/// Obtains proof-derived coverage over the governed standard scalar authority.
///
/// This crate cannot mint a refinement receipt, and the point of the coverage
/// binding is that it cannot: a `CoveredOccurrence` exists only where a
/// completed receipt does. So the fixtures walk the same sealed IR path a
/// lowering consumer walks — derive each occurrence's subject, admit an
/// authority, build a *candidate* index region here, and submit the pair to the
/// verifier, which mints a receipt only when the candidate's canonical identity
/// equals the registered law's.
///
/// Building the candidate here rather than asking the law for its own answer is
/// forced and is also the point: a caller that could obtain the expected region
/// and hand it straight back would turn the verifier into a rubber stamp.
fn checked_coverage(semantic: &SemanticProgram) -> Vec<CoveredOccurrence> {
    checked_coverage_under(semantic, &strict_contract())
}

fn checked_coverage_under(
    semantic: &SemanticProgram,
    contract: &NumericalContractIdentity,
) -> Vec<CoveredOccurrence> {
    let scalars = FrozenScalarRegistry::standard().expect("the standard scalar authority freezes");
    checked_coverage_over(semantic, &scalars, contract)
}

/// The same walk over a scalar authority composed for this exact graph.
///
/// The provider-provenance fixtures build semantic registries the standard
/// scalar profile is not composed with — that profile is pinned to
/// [`tiler_ir::semantic::FrozenSemanticRegistry::standard`], and a refinement
/// verifier refuses a scalar authority frozen over another semantic authority.
/// Those fixtures therefore pair their registry with [`scalars_over`].
fn checked_coverage_over(
    semantic: &SemanticProgram,
    scalars: &FrozenScalarRegistry,
    contract: &NumericalContractIdentity,
) -> Vec<CoveredOccurrence> {
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(
        semantic.semantic_registry().clone(),
        scalars.clone(),
    )
    .expect("the fixture's scalar and semantic authorities cohere");
    let mut coverage: Vec<CoveredOccurrence> = semantic
        .operations()
        .map(|operation| checked_occurrence(semantic, scalars, &laws, operation.id(), contract))
        .collect();
    coverage.sort_unstable_by_key(CoveredOccurrence::occurrence);
    coverage
}

fn checked_occurrence(
    semantic: &SemanticProgram,
    scalars: &FrozenScalarRegistry,
    laws: &FrozenIndexRealizationLawRegistry,
    operation: OperationId,
    contract: &NumericalContractIdentity,
) -> CoveredOccurrence {
    let subject = IndexRefinementSubject::derive(semantic, operation, contract.clone())
        .expect("every fixture operation derives a refinement subject");
    let (emitted, region) = if subject.operation() == &constant_f32_op() {
        (
            vec![constant_f32_scalar_op()],
            constant_region(
                &subject,
                scalars,
                F32_CONSTANT_BITS_ATTRIBUTE,
                constant_f32_scalar_op(),
            ),
        )
    } else if subject.operation() == &multiply_f32_op() {
        (
            vec![multiply_f32_scalar_op()],
            pointwise_region(&subject, scalars, multiply_f32_scalar_op()),
        )
    } else if subject.operation() == &add_f32_op() {
        (
            vec![add_f32_scalar_op()],
            pointwise_region(&subject, scalars, add_f32_scalar_op()),
        )
    } else if subject.operation() == &strict_serial_sum_f32_op() {
        (
            vec![add_f32_scalar_op()],
            serial_sum_region(&subject, scalars),
        )
    } else if subject.operation() == &constant_bf16_op() {
        (
            vec![constant_bf16_scalar_op()],
            constant_region(
                &subject,
                scalars,
                BF16_CONSTANT_BITS_ATTRIBUTE,
                constant_bf16_scalar_op(),
            ),
        )
    } else if subject.operation() == &multiply_bf16_op() {
        (
            vec![multiply_bf16_scalar_op()],
            pointwise_region(&subject, scalars, multiply_bf16_scalar_op()),
        )
    } else if subject.operation() == &add_bf16_op() {
        (
            vec![add_bf16_scalar_op()],
            pointwise_region(&subject, scalars, add_bf16_scalar_op()),
        )
    } else {
        panic!(
            "the fixture has no candidate region for {}",
            subject.operation()
        )
    };
    let authority = IndexRealizationAuthority::admit(
        semantic.semantic_registry(),
        scalars,
        subject.operation().clone(),
        subject.signature().clone(),
        &emitted,
    )
    .expect("the fixture's emission ceiling is admissible");
    let resolution = laws
        .resolve(&subject)
        .expect("the registered law resolves for this subject");
    match resolution
        .verify(&authority, &region)
        .expect("the fixture's candidate region realizes its operation")
    {
        IndexRefinementVerificationOutcome::Verified(receipt) => {
            CoveredOccurrence::from_receipt(&receipt)
        }
        IndexRefinementVerificationOutcome::Pending(_) => {
            panic!("the fixture's static regions retain no residual index-domain obligation")
        }
    }
}

/// The governed strict F32 contract the fixture kernels realize.
fn strict_contract() -> NumericalContractIdentity {
    F32NumericalContractKey::new(
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
    .expect("the fixture contract vector is coherent")
    .into()
}

/// The same contract flushing subnormals, used only to perturb *evidence*.
///
/// A numerical contract reaches a receipt's executable coverage and is
/// deliberately absent from semantic graph meaning, so two coverages minted
/// under these two contracts name the same occurrences of the same graph and
/// carry different proofs.
fn flush_contract() -> NumericalContractIdentity {
    F32NumericalContractKey::new(
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
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

/// Builds a rank-zero constant region from one width's bits attribute and scalar.
///
/// Both are parameters rather than the `f32` pair spelled inline, because the two
/// registered constant families carry *different* attribute identities and
/// different scalar operations while sharing one law template. A helper that
/// hardcoded either would build a region the `bf16` law's own realization does
/// not equal, and the verifier would refuse it — which is the check working, but
/// at the cost of a fixture that cannot express the second width at all.
fn constant_region(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    attribute: AttributeFieldId,
    scalar: ScalarOpKey,
) -> VerifiedIndexRegion {
    let [result] = subject.results() else {
        panic!("a constant has one result")
    };
    let bits = subject
        .attributes()
        .get(attribute)
        .expect("a constant carries its bits attribute")
        .clone();
    let attributes = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(attribute, bits)])
            .expect("the scalar attribute record composes"),
    )
    .expect("scalar attributes are a record");
    let mut region = IndexRegionBuilder::new(scalars.clone()).expect("an index region builder");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the constant's output tensor");
    let value = region
        .apply(scalar, attributes, &[])
        .expect("the constant scalar applies")
        .get(0)
        .expect("one constant result");
    let write = region.write(output, &[], &[]).expect("the constant write");
    region.output(write, value).expect("the output root");
    region.build().expect("a verified constant region")
}

fn pointwise_region(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    operation: ScalarOpKey,
) -> VerifiedIndexRegion {
    let [result] = subject.results() else {
        panic!("a binary pointwise operation has one result")
    };
    let mut region = IndexRegionBuilder::new(scalars.clone()).expect("an index region builder");
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
    let tensors = subject
        .inputs()
        .iter()
        .map(|input| {
            region
                .tensor(
                    IndexTensorRole::Input,
                    input.value_type().clone(),
                    input.shape().clone(),
                )
                .expect("a pointwise input tensor")
        })
        .collect::<Vec<_>>();
    let operands = subject
        .operands()
        .iter()
        .map(|position| {
            let input = &subject.inputs()[*position];
            if input.shape() == result.shape() {
                region
                    .read(tensors[*position], &dimensions, &coordinates)
                    .expect("an elementwise read")
            } else {
                region
                    .read(tensors[*position], &[], &[])
                    .expect("a rank-zero broadcast read")
            }
        })
        .collect::<Vec<_>>();
    let value = region
        .apply(operation, ScalarAttributes::empty(), &operands)
        .expect("the pointwise scalar applies")
        .get(0)
        .expect("one pointwise result");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the pointwise output tensor");
    let write = region
        .write(output, &dimensions, &coordinates)
        .expect("the pointwise write");
    region.output(write, value).expect("the output root");
    region.build().expect("a verified pointwise region")
}

fn serial_sum_region(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
) -> VerifiedIndexRegion {
    let ([input], [result]) = (subject.inputs(), subject.results()) else {
        panic!("a serial sum has one input and one result")
    };
    let [rows, columns] = input.shape().extents() else {
        panic!("the fixture reduces a rank-two input")
    };
    let (rows, columns) = (*rows, columns.get());
    let mut region = IndexRegionBuilder::new(scalars.clone()).expect("an index region builder");
    let row = region
        .dimension(DomainRole::Parallel, rows)
        .expect("the row dimension");
    let row_coordinate = region.dimension_expr(row).expect("the row coordinate");
    let zero = region
        .constant(IndexInteger::from_u64(0))
        .expect("the seed column");
    let input_tensor = region
        .tensor(
            IndexTensorRole::Input,
            input.value_type().clone(),
            input.shape().clone(),
        )
        .expect("the reduction input tensor");
    let seed = region
        .read(input_tensor, &[row], &[row_coordinate, zero])
        .expect("the first contributor");
    let tail = region
        .dimension(DomainRole::Reduction, Extent::new(columns - 1))
        .expect("the tail dimension");
    let tail_coordinate = region.dimension_expr(tail).expect("the tail coordinate");
    let one = IndexInteger::from_u64(1);
    let contributor_column = region
        .linear_combination(one.clone(), &[(one, tail_coordinate)])
        .expect("the tail contributor coordinate");
    let contributor = region
        .read(
            input_tensor,
            &[row, tail],
            &[row_coordinate, contributor_column],
        )
        .expect("a tail contributor");
    let total = region
        .reduce(&[tail], &[seed], &[contributor], |body| {
            let state = body.state(0).expect("one reduction state");
            let value = body.contributor(0).expect("one contributor");
            let accumulated = body
                .apply(
                    add_f32_scalar_op(),
                    ScalarAttributes::empty(),
                    &[state, value],
                )?
                .get(0)
                .expect("one accumulated result");
            body.yield_values(&[accumulated])
        })
        .expect("the serial reduction")
        .get(0)
        .expect("one reduction result");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the reduction output tensor");
    let write = region
        .write(output, &[row], &[row_coordinate])
        .expect("the reduction write");
    region.output(write, total).expect("the output root");
    region.build().expect("a verified serial-sum region")
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

/// Builds the one fused reduction kernel the packaged plans dispatch.
pub(super) fn fused_kernel(scale_bits: u32) -> VerifiedKernel {
    let axes = vec![Axis::new(1)];
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region.iteration_shape(output_shape()).unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
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
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
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
            component_role: None,
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
    fused_program_with_coverage(semantic, scale_bits, &checked_coverage(semantic))
}

/// The fused program over a graph whose registry is not the standard one.
///
/// See [`scalars_over`] for why these fixtures cannot use the governed standard
/// scalar profile.
fn fused_program_over_fixture_scalars(
    semantic: &SemanticProgram,
    scale_bits: u32,
) -> VerifiedKernelProgram {
    let scalars = scalars_over(semantic.semantic_registry());
    fused_program_with_coverage(
        semantic,
        scale_bits,
        &checked_coverage_over(semantic, &scalars, &strict_contract()),
    )
}

/// The same program over supplied coverage, so a test can vary only the proof.
fn fused_program_with_coverage(
    semantic: &SemanticProgram,
    scale_bits: u32,
    coverage: &[CoveredOccurrence],
) -> VerifiedKernelProgram {
    let kernel = fused_kernel(scale_bits);
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
    plan.push_stage(&kernel, coverage, &accesses, launch)
        .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

// -------------------------------------------------------------------------
// The two-stage intermediate-role fixture
// -------------------------------------------------------------------------
//
// Everything below exists because a *partial* binding window is not reachable
// through the single-stage fixtures above, and the reason is exact rather than
// an omission. `check_origin` pins a program input value's shape to the declared
// interface shape and `push_output` pins a published output value's, while
// `push_stage` requires each access to address exactly its buffer's element
// count — so an input or output value is always addressed whole. Only a
// `ValueRole::Temporary` value can be larger than what one stage addresses, and
// a stage binding one needs a kernel declaring a `TensorRole::Intermediate`
// buffer. A verified kernel refines the canonical lowering of a scheduled
// region, and of the three admitted region refinements the only two that name
// an intermediate role are the pointwise write and the reduction read, which
// live in different regions. So the smallest plan that can address part of a
// value is two stages, and these are those two stages.

/// The scratch shape a partial binding window addresses part of.
///
/// Twice the `[2, 3]` working set the stages exchange, so the plan can place
/// that working set in the upper half of one program-owned buffer. Nothing about
/// a temporary requires a stage to address the whole of it, and `push_view`
/// admits any window inside the value, so this is a plan the shared IR accepts
/// rather than one contrived to defeat a check.
fn scratch_shape() -> Shape {
    Shape::from_dims([4, 3])
}

/// First byte of the scratch buffer the two stages exchange their values through.
pub(super) const SCRATCH_OFFSET: u64 = ELEMENT_BYTES * 6;

/// Builds the pointwise region's kernel: one program input to one temporary.
fn pointwise_kernel() -> VerifiedKernel {
    let elements = 6;
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region.iteration_shape(input_shape()).unwrap();
    region
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
        .unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
        ),
        (1, TensorRole::Intermediate),
    ] {
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
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(InputOrdinal::FIRST).unwrap();
    let scale = expression.constant(SCALE_BITS).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(BIAS_BITS).unwrap();
    let root = expression.add(product, bias).unwrap();
    region
        .scalar_program(ScalarProgram::PointwiseF32(expression.build(root).unwrap()))
        .unwrap();
    region.numerical(strict()).unwrap();
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

/// Builds the reduction region's kernel: one temporary to one program output.
fn reduction_kernel() -> VerifiedKernel {
    let axes = vec![Axis::new(1)];
    let mut region = ScheduledRegionBuilder::new(RegionId::new(1));
    region.iteration_shape(output_shape()).unwrap();
    region
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
    region
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
        .unwrap();
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
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
        .scalar_program(ScalarProgram::StrictSerialSum {
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: CANONICAL_NAN,
            empty_identity_bits: 0,
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

/// The program-level ABI quantities the two-stage fixture's stages name.
///
/// These are handles on the *kernel program*'s own arena, which the artifact's
/// same-named handle type is deliberately not interchangeable with; the artifact
/// declares its own expressions for the same quantities and each is proven
/// against the program separately.
struct TwoStageAbi {
    /// Byte count of the `[2, 3]` working set both stages address.
    working_bytes: tiler_ir::program::AbiExprId,
    /// Byte count of the whole `[2]` program output.
    output_bytes: tiler_ir::program::AbiExprId,
    /// Launch extent of the stage iterating the `[2, 3]` shape.
    pointwise_threads: tiler_ir::program::AbiExprId,
    /// Launch extent of the stage iterating the `[2]` shape.
    reduction_threads: tiler_ir::program::AbiExprId,
    /// Workgroup width both fixture kernels require.
    one: tiler_ir::program::AbiExprId,
}

/// Declares the ABI, applicability guard, and routing-commit contract of the
/// two-stage fixture.
fn declare_two_stage_contract(plan: &mut KernelProgramBuilder) -> TwoStageAbi {
    let mut literal = |value: u64| {
        plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
            .expect("abi literal")
    };
    let abi = TwoStageAbi {
        working_bytes: literal(ELEMENT_BYTES * 6),
        output_bytes: literal(ELEMENT_BYTES * 2),
        pointwise_threads: literal(6),
        reduction_threads: literal(2),
        one: literal(1),
    };
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
    abi
}

/// The storage the two-stage fixture's stages exchange values through.
struct TwoStageStorage {
    /// The scratch value both stages address part of.
    temporary: tiler_ir::program::MaterializedValueId,
    /// The published program output.
    result: tiler_ir::program::MaterializedValueId,
    /// Whole view of the externally bound program input.
    read: ViewId,
    /// The partial view: the upper half of a scratch buffer sized for two.
    scratch_view: ViewId,
    /// Whole view of the published program output.
    write: ViewId,
}

/// Declares the input, the oversized scratch temporary, and the program output.
fn wire_two_stage_storage(plan: &mut KernelProgramBuilder) -> TwoStageStorage {
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let external = plan
        .push_allocation(device(24, AllocationOwnership::External))
        .unwrap();
    // Twice the working set: the scratch value is what makes a partial window
    // expressible, so its allocation is sized for the value and not the window.
    let scratch = plan
        .push_allocation(device(48, AllocationOwnership::Program))
        .unwrap();
    let owned = plan
        .push_allocation(device(8, AllocationOwnership::Program))
        .unwrap();
    let value = |origin, role, shape| MaterializedValueSpec {
        origin,
        role,
        shape,
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
                input_shape(),
            ),
            external,
        )
        .unwrap();
    let temporary = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                scratch_shape(),
            ),
            scratch,
        )
        .unwrap();
    let result = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            owned,
        )
        .unwrap();
    TwoStageStorage {
        temporary,
        result,
        read: plan.push_whole_view(source).unwrap(),
        scratch_view: plan
            .push_view(
                temporary,
                ByteWindow {
                    offset: SCRATCH_OFFSET,
                    length: ELEMENT_BYTES * 6,
                },
            )
            .unwrap(),
        write: plan.push_whole_view(result).unwrap(),
    }
}

/// Builds the two-stage plan whose temporary is addressed at a nonzero offset.
///
/// The scratch buffer holds twice the working set and both stages address its
/// upper half through one shared view, so every binding of that value carries an
/// offset of [`SCRATCH_OFFSET`] rather than zero.
pub(super) fn partial_window_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let pointwise = pointwise_kernel();
    let reduction = reduction_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let TwoStageAbi {
        working_bytes,
        output_bytes,
        pointwise_threads,
        reduction_threads,
        one,
    } = declare_two_stage_contract(&mut plan);
    let TwoStageStorage {
        temporary,
        result,
        read,
        scratch_view,
        write,
    } = wire_two_stage_storage(&mut plan);
    let coverage = checked_coverage(semantic);

    let first = plan
        .push_stage(
            &pointwise,
            &coverage_range(&coverage, 0..4),
            &[
                StageAccess {
                    view: read,
                    mode: StageAccessMode::Read,
                    accessible_bytes: working_bytes,
                },
                StageAccess {
                    view: scratch_view,
                    mode: StageAccessMode::Write,
                    accessible_bytes: working_bytes,
                },
            ],
            StageLaunch {
                grid_threads: pointwise_threads,
                threads_per_workgroup: one,
            },
        )
        .unwrap();
    let second = plan
        .push_stage(
            &reduction,
            &coverage_range(&coverage, 4..5),
            &[
                StageAccess {
                    view: scratch_view,
                    mode: StageAccessMode::Read,
                    accessible_bytes: working_bytes,
                },
                StageAccess {
                    view: write,
                    mode: StageAccessMode::Write,
                    accessible_bytes: output_bytes,
                },
            ],
            StageLaunch {
                grid_threads: reduction_threads,
                threads_per_workgroup: one,
            },
        )
        .unwrap();
    plan.push_data_dependency(first, second, temporary).unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

// -------------------------------------------------------------------------
fn strict_affine_u4_dequantize_semantic() -> SemanticProgram {
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
fn strict_affine_checked_coverage(semantic: &SemanticProgram) -> Vec<CoveredOccurrence> {
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

fn strict_affine_u4_dequantize_kernel() -> VerifiedKernel {
    let logical_elements = 5;
    let owner = OwnershipWitnessId::new(0);
    let mut region = ScheduledRegionBuilder::new(RegionId::new(17));
    region
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
        region.push_access(access).expect("component access");
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
        .scalar_program(ScalarProgram::StrictAffineU4Dequantize {
            codes_role: STRICT_AFFINE_CODES_ROLE,
            scale_role: STRICT_AFFINE_SCALE_ROLE,
            zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
        })
        .expect("strict-affine scalar program");
    region.numerical(strict()).expect("numerical contract");
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

fn strict_affine_u4_dequantize_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
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
        capability_revision: 1,
    }
}

pub(super) fn payload(tag: u8) -> BackendPayloadDescriptor {
    BackendPayloadDescriptor {
        backend: BackendKey::new("tiler.metal").unwrap(),
        representation: RepresentationKey::new("metallib").unwrap(),
        payload_schema: SchemaVersion::new(1, 0),
        digest: PayloadDigest::from_bytes([tag, 0xb2, 0xc3]).unwrap(),
        compatibility: profile(),
        execution_policy: ArtifactExecutionPolicy::NativeImage,
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

/// Builds the delivered-realization record every artifact fixture must carry.
///
/// Derived from the packaged entries' own realization rather than written out,
/// which is what lets one helper serve every fixture in this file whatever
/// numerical contract its program schedules: the eight overlapping dimensions
/// come from [`overlapping_behaviour`], and the three the scheduled realization
/// does not carry take the strict values that contract implies. A fixture whose
/// realization changed would otherwise need its record edited beside it, and the
/// two would drift.
///
/// `entries` is the flat **declared** packaged-entry count — every variant's
/// entries, summed — because [`ArtifactProgramBuilder::declare_realization`]
/// takes the space a producer can see and `build` remaps it.
///
/// One obligation, at the computation locus of occurrence 0, so every fixture
/// exercises a `Required` disposition and a canonical obligation range rather
/// than only the all-`NotRequired` shape.
///
/// `subject` is the arithmetic the packaged program actually computes in, and it
/// is a parameter because nothing downstream can catch it being wrong:
/// [`validate_against_artifact`] compares the record's behaviours to each bound
/// entry's realization and never reads the subject's arithmetic type, so a
/// `bf16` artifact carrying the `f32` subject would build, encode, decode, and
/// state something false about which arithmetic its delivered numerics govern.
///
/// [`validate_against_artifact`]: super::validate_against_artifact
pub(crate) fn realization_record(
    profile: &TargetProfileRef,
    subject: &ScalarArithmeticSubject,
    numerical: NumericalRealization,
    entries: u32,
) -> DeliveredRealizationRecord {
    let entry = EntryRealization::of(numerical);
    let mut resolutions =
        [DimensionBehaviour::Transform(NumericalPermission::Forbidden); DIMENSION_COUNT];
    for dimension in CANONICAL_DIMENSIONS {
        resolutions[dimension.index()] =
            overlapping_behaviour(dimension, entry).unwrap_or(match dimension {
                NumericalDimension::ApproximateIntrinsics => {
                    DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden)
                }
                NumericalDimension::MaterializationRounding => {
                    DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven)
                }
                // Reciprocal transform, the third dimension no scheduled
                // realization carries. Written as the remaining arm rather than
                // a wildcard over all eleven, so a dimension leaving the
                // overlapping set stops the build here.
                _ => DimensionBehaviour::Transform(NumericalPermission::Forbidden),
            });
    }
    let subject = subject.identity();
    let mut record = DeliveredRealizationBuilder::new(profile.clone());
    record
        .declare_scalar_arithmetic(subject.clone(), resolutions)
        .expect("the fixture contract");
    record
        .require(
            &subject,
            NumericalDimension::Contraction,
            NumericalObligationKey::new(SemanticOccurrence::new(0), PolicyLocus::Computation),
            resolutions[NumericalDimension::Contraction.index()],
            TargetEvidenceDeclaration {
                declared: resolutions[NumericalDimension::Contraction.index()],
                means: HonouringMeans::SupportedExactly,
                profile: profile.clone(),
                source: FactSourceProvenance::governed(
                    ProvenanceIdentity::new("tiler.test.baseline", 1),
                    ProvenanceIdentity::new("tiler.test.guarantee", 1),
                ),
            },
        )
        .expect("the fixture obligation");
    for entry in 0..entries {
        record
            .bind_entry(entry, &subject)
            .expect("a packaged entry");
    }
    record.build().expect("the fixture record")
}

/// Declares the fixture record for a draft that packages one program once.
///
/// The overwhelmingly common shape in this file, spelled once so a fixture that
/// is *not* that shape is visible by not using it.
pub(super) fn declare_realization(
    draft: &mut ArtifactProgramBuilder,
    program: &VerifiedKernelProgram,
) {
    declare_realization_over(draft, program, 1);
}

/// Declares the fixture record for a draft packaging one program `variants` times.
pub(super) fn declare_realization_over(
    draft: &mut ArtifactProgramBuilder,
    program: &VerifiedKernelProgram,
    variants: u32,
) {
    declare_realization_at(draft, program, &ScalarArithmeticSubject::f32(), variants);
}

/// The same declaration for a program computing in `subject`'s arithmetic.
fn declare_realization_at(
    draft: &mut ArtifactProgramBuilder,
    program: &VerifiedKernelProgram,
    subject: &ScalarArithmeticSubject,
    variants: u32,
) {
    let numerical = program
        .stages()
        .next()
        .expect("a packaged program has a stage")
        .kernel()
        .numerical();
    let stages = u32::try_from(program.stages().len()).expect("a bounded stage table fits u32");
    draft
        .declare_realization(realization_record(
            &profile(),
            subject,
            numerical,
            stages * variants,
        ))
        .expect("the fixture record");
}

pub(super) fn prepared_requirement(
    required: u64,
    relation: TargetPropertyRequirementRelation,
) -> PreparedEntryTargetRequirement {
    let query = TargetPropertyQuery::new(
        TargetPropertyKey::new("tiler.target.prepared-entry.max-threads-per-workgroup").unwrap(),
        AvailabilityPhase::PreparedKernelPreflight,
        TargetPropertyProviderIdentity::new("tiler", "prepared-entry-properties", 1).unwrap(),
    )
    .unwrap();
    PreparedEntryTargetRequirement::new(query, required, relation).unwrap()
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
            capability: CapabilityKey::new("tiler.capability.strict-affine-u4-dequantize")
                .expect("capability"),
            capability_revision: 1,
        })
        .expect("selected provider");
    let payload = draft
        .push_payload(BackendPayloadDescriptor {
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

/// The expression handles every fixture variant is assembled from.
pub(super) struct Formulas {
    /// The literal `1`, used by launch-precondition fixtures.
    pub(super) one: AbiExprId,
    /// The literal `true`, used by deferred-predicate fixtures.
    pub(super) always: AbiExprId,
}

pub(super) fn formulas(draft: &mut ArtifactProgramBuilder) -> Formulas {
    // Only what a caller still *supplies*. The applicability guard, launch
    // geometry, and accessible ranges are derived from the bound program now, so
    // minting the extent and byte-count formulas would leave them unreachable
    // from any use site -- the `UnusedExpression` the artifact refuses, and what
    // made two earlier attempts at this change look like an obligation conflict.
    let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let always = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
    Formulas { one, always }
}

pub(super) fn entry(_formulas: &Formulas, payload: PayloadId, key: &[u8]) -> EntrySpec {
    EntrySpec {
        bindings: vec![
            BindingSpec {
                kind: BindingKind::Buffer,
            },
            BindingSpec {
                kind: BindingKind::Buffer,
            },
        ],
        launch: LaunchSpec {
            zero_work_skips_dispatch: true,
            preconditions: Vec::new(),
        },
        implementation: BackendEntryRef {
            payloads: vec![payload],
            entry_key: BackendEntryKey::from_bytes(key).unwrap(),
        },
    }
}

pub(super) fn variant(formulas: &Formulas, payload: PayloadId, key: &[u8]) -> VariantSpec {
    VariantSpec {
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
    declare_realization(&mut draft, program);
    draft.build().unwrap()
}

/// The two-stage variant whose scratch bindings start at a nonzero offset.
///
/// Nothing here states that offset. The guard, launch geometry, and accessible
/// ranges — the offset included — are derived from the bound program, so the
/// spec only pairs each stage with its backend entry; a producer has no field
/// through which it could restate the placement, honestly or otherwise.
fn partial_window_variant(payload: PayloadId) -> VariantSpec {
    let entry = |key: &[u8]| EntrySpec {
        bindings: vec![
            BindingSpec {
                kind: BindingKind::Buffer,
            },
            BindingSpec {
                kind: BindingKind::Buffer,
            },
        ],
        launch: LaunchSpec {
            zero_work_skips_dispatch: true,
            preconditions: Vec::new(),
        },
        implementation: BackendEntryRef {
            payloads: vec![payload],
            entry_key: BackendEntryKey::from_bytes(key).unwrap(),
        },
    };
    VariantSpec {
        target_profile: profile(),
        feasibility_rules: rules(),
        deferred_predicates: Vec::new(),
        entries: vec![entry(b"pointwise"), entry(b"reduction")],
    }
}

/// Assembles the two-stage artifact whose temporary is bound at a nonzero offset.
pub(super) fn partial_window_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    draft
        .push_variant(&program, partial_window_variant(descriptor))
        .unwrap();
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
}

pub(crate) fn default_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    build_artifact(&semantic, &program, provider.clone(), &[provider])
}

/// A kernel whose body consumes one live input-axis extent.
///
/// Iteration is the static outer product; only axis 1 of the declared input is
/// live. The write is the program output so a single-stage artifact can bind it.
fn live_extent_kernel() -> VerifiedKernel {
    let rows = 2;
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region.iteration_shape(Shape::from_dims([rows])).unwrap();
    let inner = Axis::new(1);
    region
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LiveRowMajor { inner_axis: inner },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LiveRowMajor { inner_axis: inner },
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
        ),
        (1, TensorRole::Output),
    ] {
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
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: rows },
        })
        .unwrap();
    region
        .scalar_program(ScalarProgram::PointwiseF32(scale_bias_expression()))
        .unwrap();
    region.numerical(strict()).unwrap();
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: rows,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: rows,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    lower_scheduled_region(&region.build().unwrap()).unwrap()
}

fn scale_bias_expression() -> tiler_ir::schedule::PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(InputOrdinal::FIRST).unwrap();
    let scale = expression.constant(SCALE_BITS).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(BIAS_BITS).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

pub(crate) fn live_extent_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    live_extent_program_with_guard(semantic, LiveExtentGuard::AlwaysHolds)
}

pub(crate) fn live_extent_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = live_extent_program(&semantic);
    let provider = lowering_provider(1);
    build_artifact(&semantic, &program, provider.clone(), &[provider])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LiveExtentGuard {
    AlwaysHolds,
    MultipleOfSixteen,
}

fn live_extent_guard(
    plan: &mut KernelProgramBuilder,
    live_n: tiler_ir::program::AbiExprId,
    kind: LiveExtentGuard,
) -> tiler_ir::program::AbiExprId {
    match kind {
        LiveExtentGuard::AlwaysHolds => plan.push_abi_root(AbiRoot::BooleanLiteral(true)).unwrap(),
        LiveExtentGuard::MultipleOfSixteen => {
            let sixteen = plan.push_abi_root(AbiRoot::UnsignedLiteral(16)).unwrap();
            plan.push_abi_binary(AbiBinaryOp::IsMultipleOf, live_n, sixteen)
                .unwrap()
        }
    }
}

fn live_extent_program_with_guard(
    semantic: &SemanticProgram,
    kind: LiveExtentGuard,
) -> VerifiedKernelProgram {
    let kernel = live_extent_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let external = plan
        .push_allocation(device(24, AllocationOwnership::External))
        .unwrap();
    let owned = plan
        .push_allocation(device(24, AllocationOwnership::Program))
        .unwrap();
    let value = |origin, role, shape| MaterializedValueSpec {
        origin,
        role,
        shape,
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
                input_shape(),
            ),
            external,
        )
        .unwrap();
    let result = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            owned,
        )
        .unwrap();
    let read = plan
        .push_view(
            source,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .unwrap();
    let write = plan
        .push_view(
            result,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .unwrap();
    let zero = plan.push_abi_root(AbiRoot::UnsignedLiteral(0)).unwrap();
    let two = plan.push_abi_root(AbiRoot::UnsignedLiteral(2)).unwrap();
    let one = plan.push_abi_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let live_n = plan
        .push_abi_root(AbiRoot::InputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(1),
        })
        .unwrap();
    let accessible = plan
        .push_abi_binary(AbiBinaryOp::CheckedMultiply, zero, live_n)
        .unwrap();
    let guard = live_extent_guard(&mut plan, live_n, kind);
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
                view: read,
                mode: StageAccessMode::Read,
                accessible_bytes: accessible,
            },
            StageAccess {
                view: write,
                mode: StageAccessMode::Write,
                accessible_bytes: accessible,
            },
        ],
        StageLaunch {
            grid_threads: two,
            threads_per_workgroup: one,
        },
    )
    .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

fn live_extent_c1_portfolio() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let aligned = live_extent_program_with_guard(&semantic, LiveExtentGuard::MultipleOfSixteen);
    let direct = live_extent_program_with_guard(&semantic, LiveExtentGuard::AlwaysHolds);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let aligned_payload = draft.push_payload(payload(0xa1)).unwrap();
    let direct_payload = draft.push_payload(payload(0xa2)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&aligned, variant(&formulas, aligned_payload, b"aligned"))
        .unwrap();
    draft
        .push_variant(&direct, variant(&formulas, direct_payload, b"direct"))
        .unwrap();
    declare_realization_over(&mut draft, &aligned, 2);
    draft.build().unwrap()
}

// -------------------------------------------------------------------------
// The pointwise producer path at two widths
// -------------------------------------------------------------------------
//
// `admit-a-bf16-index-realization-law-and-refinement-contract` made a pure-BF16
// occurrence able to obtain executable coverage, so a BF16 kernel program now
// verifies. This crate owns what that unblocked and `tiler-ir` cannot reach: the
// packaging half, since the dependency direction is `tiler-artifact → tiler-ir`.
//
// The candidate index regions are still hand-built with `IndexRegionBuilder`
// through `checked_coverage_under`, exactly as every other fixture here does —
// `IndexRealizationLaw::realize` is `pub(crate)` to `tiler-ir` — which is the
// stronger arrangement anyway: a caller that could ask the law for its own
// answer and hand it straight back would turn the verifier into a rubber stamp.

/// `2.0` in the ratified BF16 operand format.
const BF16_SCALE_BITS: u16 = 0x4000;
/// `1.0` in the same format.
const BF16_BIAS_BITS: u16 = 0x3f80;

/// The two arithmetic widths the pointwise fixture is built at.
///
/// One parameterized construction rather than two hand-written fixture families,
/// because the identity comparison below is only worth making if the width is
/// the *sole* difference between the two artifacts. Two twins written out
/// separately drift, and a drifted pair still yields two distinct identities —
/// for a reason no later reader could attribute to the carrier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PointwiseWidth {
    F32,
    Bf16,
}

impl PointwiseWidth {
    /// Storage width of one element, in bytes.
    const fn element_bytes(self) -> u64 {
        match self {
            Self::F32 => 4,
            Self::Bf16 => 2,
        }
    }

    const fn storage_scalar(self) -> StorageScalar {
        match self {
            Self::F32 => StorageScalar::F32,
            Self::Bf16 => StorageScalar::Bf16,
        }
    }

    const fn access_type(self) -> KernelType {
        match self {
            Self::F32 => KernelType::F32,
            Self::Bf16 => KernelType::Bf16,
        }
    }

    /// The scalar-arithmetic subject the delivered-realization record names.
    fn subject(self) -> ScalarArithmeticSubject {
        match self {
            Self::F32 => ScalarArithmeticSubject::f32(),
            Self::Bf16 => ScalarArithmeticSubject::new(ArithmeticType::Bf16, Bf16::resolved_type())
                .expect("the governed bf16 arithmetic subject is registered"),
        }
    }

    /// The governed strict contract for this width.
    ///
    /// The two keys are separate types rather than one key carrying a width,
    /// which is what makes a contract stated for the other width a *named*
    /// refusal at refinement rather than a silent mismatch.
    fn contract(self) -> NumericalContractIdentity {
        match self {
            Self::F32 => strict_contract(),
            Self::Bf16 => Bf16NumericalContractKey::new(
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
            .into(),
        }
    }

    /// The scheduled region's numerical realization for this width.
    ///
    /// The canonical NaN payload is this width's own: `verify_pointwise_bf16`
    /// refuses a `bf16` region declaring any other, so the two arms cannot be
    /// collapsed onto one constant.
    fn numerical(self) -> NumericalRealization {
        match self {
            Self::F32 => strict(),
            Self::Bf16 => NumericalRealization::new(
                "tiler.test.strict-bf16",
                u32::from(tiler_ir::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS),
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
            ),
        }
    }

    /// The four-operation graph `result = input * 2.0 + 1.0` at this width.
    ///
    /// Constant, multiply, and add are the complete registered `bf16` semantic
    /// vocabulary, so the `Bf16` arm is the widest pure-`bf16` program the
    /// semantic layer can state rather than a subset chosen to be easy; the
    /// `F32` arm states the same four operations so the two are twins.
    fn semantic(self) -> SemanticProgram {
        let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
        let key = InputKey::new("input").expect("input key");
        let result = OutputKey::new("result").expect("output key");
        match self {
            Self::F32 => {
                let input = draft.input::<F32>(key, input_shape()).expect("input");
                let scale = F32Constant::apply(&mut draft, SCALE_BITS).expect("scale");
                let bias = F32Constant::apply(&mut draft, BIAS_BITS).expect("bias");
                let product = F32Multiply::apply(&mut draft, input, scale).expect("product");
                let mapped = F32Add::apply(&mut draft, product, bias).expect("mapped");
                draft.output(result, mapped).expect("output");
            }
            Self::Bf16 => {
                let input = draft.input::<Bf16>(key, input_shape()).expect("input");
                let scale = Bf16Constant::apply(&mut draft, BF16_SCALE_BITS).expect("scale");
                let bias = Bf16Constant::apply(&mut draft, BF16_BIAS_BITS).expect("bias");
                let product = Bf16Multiply::apply(&mut draft, input, scale).expect("product");
                let mapped = Bf16Add::apply(&mut draft, product, bias).expect("mapped");
                draft.output(result, mapped).expect("output");
            }
        }
        let program = draft
            .build()
            .expect("a verified pointwise semantic program");
        assert_eq!(program.operation_count(), 4);
        program
    }

    /// The fused pointwise expression the scheduled region computes.
    fn scalar_program(self) -> ScalarProgram {
        match self {
            Self::F32 => {
                let mut expression = PointwiseF32ExpressionBuilder::new();
                let leaf = expression.input(InputOrdinal::FIRST).expect("input");
                let scale = expression.constant(SCALE_BITS).expect("scale");
                let product = expression.multiply(leaf, scale).expect("product");
                let bias = expression.constant(BIAS_BITS).expect("bias");
                let root = expression.add(product, bias).expect("sum");
                ScalarProgram::PointwiseF32(
                    expression.build(root).expect("an f32 pointwise expression"),
                )
            }
            Self::Bf16 => {
                let mut expression = PointwiseBf16ExpressionBuilder::new();
                let leaf = expression.input(InputOrdinal::FIRST).expect("input");
                let scale = expression.constant(BF16_SCALE_BITS).expect("scale");
                let product = expression.multiply(leaf, scale).expect("product");
                let bias = expression.constant(BF16_BIAS_BITS).expect("bias");
                let root = expression.add(product, bias).expect("sum");
                ScalarProgram::PointwiseBf16(
                    expression.build(root).expect("a bf16 pointwise expression"),
                )
            }
        }
    }

    /// The one-region kernel: a whole `[2, 3]` read to a whole `[2, 3]` write.
    fn kernel(self) -> VerifiedKernel {
        let count = elements();
        let owner = OwnershipWitnessId::new(0);
        let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
        region
            .iteration_shape(input_shape())
            .expect("iteration shape");
        for (tensor, mode, bounds, ownership) in [
            (
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
                AccessMode::Read,
                BoundsWitnessId::new(0),
                None,
            ),
            (
                TensorRole::Output,
                AccessMode::Write,
                BoundsWitnessId::new(1),
                Some(owner),
            ),
        ] {
            region
                .push_access(Access {
                    tensor,
                    component_role: None,
                    mode,
                    map: LogicalAccess::LinearIdentity,
                    bounds,
                    ownership,
                })
                .expect("a whole-tensor access");
            region
                .push_bounds_proof(BoundsProof {
                    id: bounds,
                    tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: count,
                    },
                })
                .expect("linear bounds");
        }
        region
            .ownership_proof(OwnershipProof {
                id: owner,
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: count,
                },
            })
            .expect("output ownership");
        region
            .scalar_program(self.scalar_program())
            .expect("scalar program");
        region.numerical(self.numerical()).expect("numerical");
        region
            .schedule(KernelSchedule {
                binding: ExecutionBinding::GlobalLinearInvocation,
                work_items: count,
                threads_per_workgroup: 1,
                tail: TailPolicy::Exact,
                output_owner: owner,
                reduction: ReductionTopology::None,
                launch: LaunchPlan {
                    grid_threads: count,
                    threads_per_workgroup: 1,
                    zero_work_skips_dispatch: true,
                },
            })
            .expect("schedule");
        lower_scheduled_region(&region.build().expect("a verified pointwise region"))
            .expect("a verified pointwise kernel")
    }

    fn value(self, origin: MaterializedOrigin, role: ValueRole) -> MaterializedValueSpec {
        MaterializedValueSpec {
            origin,
            role,
            shape: input_shape(),
            storage_scalar: self.storage_scalar(),
            element_type: self.access_type(),
            encoding: StorageEncoding::Unpacked,
            alignment: AlignmentRequirement::natural_for(self.storage_scalar()),
            memory_space: MemorySpace::Device,
        }
    }

    /// The single-stage kernel program the artifact packages.
    fn program(self, semantic: &SemanticProgram) -> VerifiedKernelProgram {
        self.program_with_kernel(semantic, &self.kernel())
    }

    fn program_with_kernel(
        self,
        semantic: &SemanticProgram,
        kernel: &VerifiedKernel,
    ) -> VerifiedKernelProgram {
        let coverage = checked_coverage_under(semantic, &self.contract());
        let bytes = self.element_bytes() * elements();
        let mut plan = KernelProgramBuilder::new(semantic).expect("program builder");
        let external = plan
            .push_allocation(AllocationSpec {
                capacity_bytes: bytes,
                alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
                memory_space: MemorySpace::Device,
                ownership: AllocationOwnership::External,
            })
            .expect("input allocation");
        let owned = plan
            .push_allocation(AllocationSpec {
                capacity_bytes: bytes,
                alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
                memory_space: MemorySpace::Device,
                ownership: AllocationOwnership::Program,
            })
            .expect("output allocation");
        let source = plan
            .push_value(
                self.value(
                    MaterializedOrigin::ProgramInput {
                        key: InputKey::new("input").expect("input key"),
                    },
                    ValueRole::Input,
                ),
                external,
            )
            .expect("input value");
        let result = plan
            .push_value(
                self.value(MaterializedOrigin::Internal, ValueRole::Output),
                owned,
            )
            .expect("output value");
        let read = plan.push_whole_view(source).expect("input view");
        let write = plan.push_whole_view(result).expect("output view");
        // Only the quantities this one stage names. Minting the byte counts the
        // reduction fixtures share would leave an ABI expression no stage
        // references, which the program verifier refuses by name.
        let mut literal = |value| {
            plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
                .expect("abi literal")
        };
        let value_bytes = literal(bytes);
        let grid_threads = literal(elements());
        let threads_per_workgroup = literal(1);
        let guard = plan
            .push_abi_root(AbiRoot::BooleanLiteral(true))
            .expect("guard predicate");
        plan.applicability_guard(guard)
            .expect("applicability guard");
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
            .expect("routing-commit transition");
        }
        plan.push_stage(
            kernel,
            &coverage,
            &[
                StageAccess {
                    view: read,
                    mode: StageAccessMode::Read,
                    accessible_bytes: value_bytes,
                },
                StageAccess {
                    view: write,
                    mode: StageAccessMode::Write,
                    accessible_bytes: value_bytes,
                },
            ],
            StageLaunch {
                grid_threads,
                threads_per_workgroup,
            },
        )
        .expect("the pointwise stage covers every occurrence of its bound graph");
        plan.push_output(OutputKey::new("result").expect("output key"), result)
            .expect("named output");
        plan.build().expect("a verified pointwise kernel program")
    }

    /// The one-variant artifact packaging that program.
    fn artifact(self) -> VerifiedArtifactProgram {
        let semantic = self.semantic();
        let program = self.program(&semantic);
        let provider =
            ProviderIdentity::new("tiler-test", "pointwise-scale-bias", 1).expect("provider");
        let environment = CompilationEnvironment::new([provider.clone()]).expect("environment");
        let mut draft =
            ArtifactProgramBuilder::new(&semantic, environment).expect("artifact builder");
        draft
            .select_provider(SelectedProvider {
                provider,
                capability: CapabilityKey::new("tiler.capability.pointwise-scale-bias")
                    .expect("capability"),
                capability_revision: 1,
            })
            .expect("selected provider");
        let descriptor = draft.push_payload(payload(0xb1)).expect("payload");
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"pointwise"))
            .expect("pointwise variant");
        declare_realization_at(&mut draft, &program, &self.subject(), 1);
        draft.build().expect("a verified pointwise artifact")
    }
}

/// Element count of the `[2, 3]` tensor both widths' fixtures address whole.
fn elements() -> u64 {
    input_shape()
        .extents()
        .iter()
        .map(|extent| extent.get())
        .product()
}

/// The pure-BF16 artifact reached through the ordinary producer path.
pub(super) fn bf16_pointwise_artifact() -> VerifiedArtifactProgram {
    PointwiseWidth::Bf16.artifact()
}

/// Its F32 twin: the same four operations, the same shape, the other width.
pub(super) fn f32_pointwise_artifact() -> VerifiedArtifactProgram {
    PointwiseWidth::F32.artifact()
}

/// A pure-BF16 program travels semantics to packaged artifact through the
/// ordinary producer path.
///
/// The composition `carry-bf16-through-the-artifact-encoding-and-identity`
/// recorded as unreachable, now walked end to end: every one of the four
/// coverage records is minted by the refinement verifier from a candidate region
/// this crate built, the program verifier accepts a stage claiming them, and the
/// artifact builder packages the result. Nothing here forges an envelope.
///
/// The carrier assertions are on what a *consumer* reads — the declared
/// interface component and the entry's binding windows — because those are what
/// a runtime uses to size a buffer, and twelve versus twenty-four bytes over the
/// same six-element tensor is the whole reason the width has to survive.
#[test]
fn a_pure_bf16_program_reaches_a_verified_artifact_through_the_builder() {
    let semantic = PointwiseWidth::Bf16.semantic();
    let coverage = checked_coverage_under(&semantic, &PointwiseWidth::Bf16.contract());
    assert_eq!(
        coverage
            .iter()
            .map(|covered| covered.occurrence().get())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3],
        "the coverage partition is the graph's complete canonical occurrence run",
    );
    let program = PointwiseWidth::Bf16.program(&semantic);
    assert_eq!(program.stages().count(), 1);

    let artifact = PointwiseWidth::Bf16.artifact();
    let component = artifact
        .inputs()
        .next()
        .expect("one declared input")
        .components()
        .next()
        .expect("one dense component");
    assert_eq!(component.storage_scalar(), StorageScalar::Bf16);
    assert_eq!(component.access_type(), KernelType::Bf16);
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    assert_eq!(
        entry
            .bindings()
            .map(|binding| (
                binding.storage_scalar(),
                binding.access_type(),
                binding.window().length,
            ))
            .collect::<Vec<_>>(),
        [(StorageScalar::Bf16, KernelType::Bf16, 12); 2],
        "six bf16 elements are twelve bytes on both the read and the write side",
    );

    // The delivered-realization record names the arithmetic the program
    // computes in. Asserted because nothing downstream can: the artifact-level
    // cross-check compares behaviours against each entry's realization and never
    // reads the subject, so a record naming `f32` for this program would build,
    // encode, decode, and state something false to every consumer of it.
    let record = artifact.delivered_realization();
    assert!(
        record
            .scalar_arithmetic(&PointwiseWidth::Bf16.subject().identity())
            .is_some(),
        "the record must carry the bf16 scalar-arithmetic subject",
    );
    assert!(
        record
            .scalar_arithmetic(&PointwiseWidth::F32.subject().identity())
            .is_none(),
        "and must not carry the f32 one, which no other check would catch",
    );
}

/// The same program at the other width is a different artifact.
///
/// The producer-path counterpart of the encoding rung's carrier-only comparison,
/// and it answers a strictly larger question. There the two envelopes were one
/// artifact with two tag bytes rewritten, so the four differing identity bytes
/// were the carrier and nothing else. Here the two are separately *derived* from
/// separately verified semantic graphs, so the difference spans the semantic
/// operation keys, the refinement evidence minted under each width's own
/// contract, the scheduled expression, the canonical NaN payload, and the buffer
/// sizes — every place the width is load-bearing rather than only the two the
/// forgery could reach.
#[test]
fn the_bf16_artifact_and_its_f32_twin_are_two_artifacts() {
    let bf16 = PointwiseWidth::Bf16.artifact();
    let twin = PointwiseWidth::F32.artifact();
    assert_ne!(
        bf16.canonical_identity(),
        twin.canonical_identity(),
        "two artifacts differing in the arithmetic they compute in must not share an identity",
    );
    assert_eq!(
        bf16.canonical_identity(),
        PointwiseWidth::Bf16.artifact().canonical_identity(),
        "nothing else in the fixture varies between two builds at one width",
    );
    let window = |artifact: &VerifiedArtifactProgram| {
        artifact
            .variants()
            .next()
            .expect("one variant")
            .entries()
            .next()
            .expect("one entry")
            .bindings()
            .next()
            .expect("one read binding")
            .window()
            .length
    };
    assert_eq!(
        (window(&bf16), window(&twin)),
        (12, 24),
        "the twin addresses the same six elements at twice the width",
    );
}

/// Reconstructs the historical stage-key payload: the bound kernel identity and
/// the bare coverage ordinals, with no refinement evidence beside them.
///
/// `v1` and `v2` share this payload byte for byte and differ only in their
/// separator, because the canonical-coverage step *reinterpreted* those raw
/// ordinals rather than changing them. `v3` is the first step that changes the
/// payload, which is why this reconstruction is written here rather than
/// obtained by handing another separator to the production encoder.
fn coverage_only_stage_key(stage: tiler_ir::program::StageRef<'_>, domain: &[u8]) -> Vec<u8> {
    let mut bytes = domain.to_vec();
    push_slice(&mut bytes, stage.kernel().canonical_identity().as_bytes());
    push_len(&mut bytes, stage.coverage().len());
    for covered in stage.coverage() {
        bytes.extend_from_slice(&covered.occurrence().get().to_be_bytes());
    }
    bytes
}

#[test]
fn each_artifact_stage_key_generation_is_separated_from_the_last() {
    const V1: &[u8] = b"tiler.artifact-program.stage.v1\0";
    const V2: &[u8] = b"tiler.artifact-program.stage.v2\0";
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let stage = program.stages().next().expect("one fused stage");
    let current = stage_key(stage);
    let v1 = coverage_only_stage_key(stage, V1);
    let v2 = coverage_only_stage_key(stage, V2);

    assert!(current.starts_with(STAGE_KEY_DOMAIN));
    assert!(!current.starts_with(V1));
    assert!(!current.starts_with(V2));
    // v1 → v2 moved the separator over an unchanged payload, because the step
    // reinterpreted the ordinals rather than rewriting them.
    assert_eq!(
        v1[V1.len()..],
        v2[V2.len()..],
        "v1 and v2 spell the same coverage payload"
    );
    assert_ne!(v1, v2);
    // v2 → v3 rewrote the payload: each record gained its framed evidence, so
    // the current key is strictly longer and shares no reading with either.
    assert!(current.len() > v2.len());
    assert_ne!(current, v2);
    assert_ne!(current, v1);
}

/// The two independent stage encoders agree on what a coverage record is.
///
/// `tiler-ir`'s two encoder sites destructure `CoveredOccurrence`, so a field
/// added to that record stops their build until it is folded. This encoder
/// cannot do that: the fields are private and this is another crate, which is
/// the sealed-construction design working as intended — the same privacy that
/// stops a caller assembling a record stops this crate reading it apart. So the
/// artifact side is held here instead, and this is the check that fails.
///
/// Both encoders write the same per-record run — the occurrence's four bytes
/// then its length-framed evidence — and differ only in what precedes it: this
/// key opens with its own separator, the program encoding does not. The run
/// this key writes is therefore a contiguous subsequence of the program
/// identity, and stays one exactly while the two agree. If `tiler-ir` folds a
/// new field and this encoder does not, its run stops matching and this test
/// says so; the reverse fails the same way.
///
/// What this does *not* hold is the framing around the run, which the two
/// deliberately spell differently. That is why the assertion is containment
/// rather than equality.
#[test]
fn the_artifact_stage_key_encodes_the_same_coverage_record_as_the_kernel_program() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let identity = program.canonical_identity().as_bytes();

    let mut stages = 0_usize;
    for stage in program.stages() {
        stages += 1;
        assert!(
            !stage.coverage().is_empty(),
            "a stage covering nothing would make the run below empty, and an \
             empty needle occurs everywhere",
        );
        let key = stage_key(stage);
        // Everything this key writes after its count: the coverage run itself.
        let prefix = STAGE_KEY_DOMAIN.len()
            + framed(stage.kernel().canonical_identity().as_bytes().len())
            + LENGTH_BYTES;
        let run = &key[prefix..];
        assert_eq!(
            identity
                .windows(run.len())
                .filter(|window| *window == run)
                .count(),
            1,
            "the coverage run this stage key writes must appear exactly once in \
             the kernel-program identity; the two encoders have drifted",
        );
    }
    assert_eq!(stages, 1, "the fused fixture packages one stage");
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
    assert_eq!(output.shape(), &output_shape());
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
        super::BindingTarget::ProgramInput(&InputKey::new("input").unwrap()),
    );
    assert_eq!(
        bindings[1].target(),
        super::BindingTarget::ProgramOutput(std::slice::from_ref(&result)),
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
    let super::BindingTarget::ProgramOutput(keys) = bindings[1].target() else {
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
        assert_eq!(scratch.target(), super::BindingTarget::Internal);
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

// -------------------------------------------------------------------------
// Recorded identity assertions
// -------------------------------------------------------------------------

/// The cold-consumer round trip: a producer records the identity it derived, a
/// consumer reads those bytes back and states them. The stated assertion carries
/// the same bytes, and is a different type from the derivation it came from.
#[test]
fn recorded_bytes_from_a_derived_identity_state_that_identity() {
    let artifact = default_artifact();
    let derived = artifact.canonical_identity();

    let recorded = RecordedArtifactProgramIdentity::from_bytes(derived.as_bytes()).unwrap();

    assert_eq!(recorded.as_bytes(), derived.as_bytes());
}

#[test]
fn an_empty_recording_is_refused() {
    assert_eq!(
        RecordedArtifactProgramIdentity::from_bytes([]),
        Err(RecordedArtifactIdentityError::Empty),
    );
}

/// Checked before the domain frame is read, so an over-bound recording is
/// refused whatever it leads with.
#[test]
fn a_recording_beyond_the_identity_bound_is_refused() {
    let oversized = vec![0_u8; MAX_ARTIFACT_IDENTITY_BYTES + 1];

    assert_eq!(
        RecordedArtifactProgramIdentity::from_bytes(&oversized),
        Err(RecordedArtifactIdentityError::TooLong {
            bytes: MAX_ARTIFACT_IDENTITY_BYTES + 1,
            limit: MAX_ARTIFACT_IDENTITY_BYTES,
        }),
    );
}

/// The three shapes a wrong recording actually takes: bytes of some other
/// subject, an identity from a superseded artifact domain, and a recording
/// truncated inside the separator itself.
#[test]
fn a_recording_under_a_foreign_domain_is_refused() {
    let foreign: [&[u8]; 3] = [
        b"tiler.kernel.v3\0some other subject",
        b"tiler.artifact-program.v10\0",
        // The label is the separator without its terminator, so this is a
        // recording truncated one byte inside the frame being matched.
        ARTIFACT_DOMAIN_LABEL.as_bytes(),
    ];

    for bytes in foreign {
        assert_eq!(
            RecordedArtifactProgramIdentity::from_bytes(bytes),
            Err(RecordedArtifactIdentityError::ForeignDomain { bytes: bytes.len() }),
            "expected a foreign-domain refusal for {bytes:?}",
        );
    }
}

/// The domain a rejection names is the one the encoder writes, not a second
/// copy of the string that a version bump could leave behind.
#[test]
fn a_foreign_domain_rejection_names_the_current_domain() {
    let rejection = RecordedArtifactProgramIdentity::from_bytes(b"not an artifact identity")
        .expect_err("bytes under no artifact domain are refused");

    let rendered = rejection.to_string();
    assert!(
        rendered.contains(ARTIFACT_DOMAIN_LABEL),
        "{rendered} does not name the {ARTIFACT_DOMAIN_LABEL} domain",
    );
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
        declare_realization_over(&mut draft, &program, 2);
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
        // Assemble the identical formulas through two different node orders.
        let formulas = if reversed {
            // The same two expressions in the opposite declaration order; the
            // variant's ABI is the program's now, so what remains under test is
            // that a caller-supplied expression's declaration order does not
            // reach identity.
            let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
            let always = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
            Formulas { one, always }
        } else {
            formulas(&mut draft)
        };
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"fused"))
            .unwrap();
        declare_realization(&mut draft, &program);
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

/// The refinement evidence a stage names reaches artifact identity.
///
/// This is the artifact half of the coverage binding, and it is the half that
/// needs its own test: the artifact writes the stage subject through its own
/// encoder, so an artifact blind to a difference the kernel program folds would
/// be a real divergence rather than a duplicated assertion. The perturbation is
/// the governed numerical contract the receipts were minted under — a genuine
/// difference in what was proved, and one the semantic graph does not carry.
#[test]
fn refinement_evidence_moves_program_and_artifact_identity() {
    let semantic = semantic_program();
    let strict = checked_coverage_under(&semantic, &strict_contract());
    let flushed = checked_coverage_under(&semantic, &flush_contract());
    assert_eq!(
        strict
            .iter()
            .map(CoveredOccurrence::occurrence)
            .collect::<Vec<_>>(),
        flushed
            .iter()
            .map(CoveredOccurrence::occurrence)
            .collect::<Vec<_>>(),
        "the perturbation changes evidence, not which occurrences are covered",
    );
    assert!(
        strict
            .iter()
            .zip(&flushed)
            .any(|(left, right)| left.refinement() != right.refinement()),
        "two governed contracts must mint distinct executable-coverage evidence",
    );

    let strict_program = fused_program_with_coverage(&semantic, SCALE_BITS, &strict);
    let flushed_program = fused_program_with_coverage(&semantic, SCALE_BITS, &flushed);
    assert_ne!(
        strict_program.canonical_identity(),
        flushed_program.canonical_identity(),
    );

    let provider = lowering_provider(1);
    let strict_artifact = build_artifact(
        &semantic,
        &strict_program,
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let flushed_artifact = build_artifact(
        &semantic,
        &flushed_program,
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    assert_ne!(
        strict_artifact.canonical_identity(),
        flushed_artifact.canonical_identity(),
    );
}

#[test]
fn a_reached_capability_provider_revision_changes_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let available = [lowering_provider(1), lowering_provider(2)];
    let first = build_artifact(&semantic, &program, lowering_provider(1), &available);
    let second = build_artifact(&semantic, &program, lowering_provider(2), &available);
    assert_ne!(first.canonical_identity(), second.canonical_identity());
}

/// The capability's own revision reaches identity, independently of the provider's.
///
/// `docs/operation-extensions.md` makes the two revisions independent — one
/// provider registers several capabilities that move at different rates — so
/// folding only the provider's left a provider free to change what its lowering
/// emits and produce a byte-identical artifact identity, which is exactly the
/// drift the capability revision exists to catch. Both directions are asserted:
/// the revision moving changes identity, and everything else held equal it is
/// the only thing that did.
#[test]
fn a_reached_capability_revision_changes_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let build = |capability_revision: u32| {
        let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
        draft
            .select_provider(SelectedProvider {
                provider: provider.clone(),
                capability: CapabilityKey::new("tiler.capability.fused-serial-sum").unwrap(),
                capability_revision,
            })
            .unwrap();
        let descriptor = draft.push_payload(payload(0xa1)).unwrap();
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"fused"))
            .unwrap();
        declare_realization(&mut draft, &program);
        draft.build().unwrap()
    };

    let first = build(1);
    let second = build(2);
    assert_ne!(first.canonical_identity(), second.canonical_identity());
    assert_eq!(
        first.canonical_identity(),
        build(1).canonical_identity(),
        "nothing else in the fixture varies with the revision",
    );
    assert_eq!(
        first.selected_providers()[0].provider,
        second.selected_providers()[0].provider,
        "the provider's own revision is unchanged; only the capability's moved",
    );
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
        &fused_program_over_fixture_scalars(&first, SCALE_BITS),
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let second_artifact = build_artifact(
        &second,
        &fused_program_over_fixture_scalars(&second, SCALE_BITS),
        provider.clone(),
        &[provider],
    );
    assert_ne!(
        first_artifact.canonical_identity(),
        second_artifact.canonical_identity(),
    );
}

/// A symbolic semantic program never opens an artifact builder.
///
/// **This is the assertion `project_semantic`'s three carried subjects rest
/// on.** The envelope travels the semantic graph identity, the reached
/// definitions, and the admission provenance; it deliberately leaves the
/// registry snapshot behind under ADR 0072, and it leaves the shape-environment
/// subject behind for a weaker reason — no two packaged artifacts can differ by
/// it. That is only true while no program whose interface names a declared
/// symbol reaches this builder. If this test fails, two programs over
/// differently bound environments encode to one envelope digest, and
/// `docs/artifact-abi.md`'s "only the three reached subjects travel" becomes an
/// unkeyed symbolic program rather than a deliberate exclusion.
#[test]
fn a_symbolic_semantic_program_never_reaches_the_artifact_builder() {
    let scope = SymbolScope::new("artifact/0").unwrap();
    let rows = ShapeSymbol::new(scope, "rows").unwrap();
    let mut draft = ShapeEnvBuilder::new();
    draft.declare(rows.clone()).unwrap();
    draft
        .bind(
            &rows,
            RootBinding::new(
                BindingSource::InputDimension {
                    input: InputKey::new("input").unwrap(),
                    axis: Axis::new(0),
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    let environment = Arc::new(draft.build().unwrap());

    let mut builder =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let value = builder
        .input_sourced::<F32>(
            InputKey::new("input").unwrap(),
            vec![SourcedExtent::Symbol(rows)],
        )
        .expect("the symbolic input is admitted at the semantic layer");
    builder
        .output(OutputKey::new("result").unwrap(), value)
        .unwrap();
    let symbolic = builder.build().unwrap();
    assert_ne!(
        symbolic.semantic_identity().shape_environment(),
        semantic_program().semantic_identity().shape_environment(),
        "the fixture really does carry a non-empty environment subject",
    );

    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider]).expect("environment");
    assert!(
        matches!(
            ArtifactProgramBuilder::new(&symbolic, environment.clone()),
            Err(ArtifactBuildError::SymbolicSemanticInterface { interface })
                if interface == "input",
        ),
        "a symbolic interface extent is refused before any subject is projected",
    );
    assert!(
        ArtifactProgramBuilder::new(&semantic_program(), environment).is_ok(),
        "the neighbour differing only in the extent's source kind opens",
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
    let first_program = fused_program_over_fixture_scalars(&first, SCALE_BITS);
    let second_program = fused_program_over_fixture_scalars(&second, SCALE_BITS);
    // The kernel-program leg is asserted separately from the artifact leg: the
    // artifact folds the program identity, so equal artifacts would otherwise
    // leave a program-level divergence indistinguishable from an artifact-level
    // one that happened to cancel.
    assert_eq!(
        first_program.canonical_identity(),
        second_program.canonical_identity(),
    );
    let first_artifact = build_artifact(
        &first,
        &first_program,
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let second_artifact = build_artifact(&second, &second_program, provider.clone(), &[provider]);
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
    // Injected through a launch precondition, which is still caller-supplied.
    // The guard and launch geometry are derived from the program now, so they
    // are no longer a way to hand the builder a foreign handle at all.
    spec.entries[0]
        .launch
        .preconditions
        .push(donor_formulas.always);
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
fn rejects_a_deferred_requirement_naming_no_entry() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![DeferredPredicateSpec {
            requirement: prepared_requirement(
                1,
                TargetPropertyRequirementRelation::ObservedAtLeastRequired,
            ),
            entry: 1,
        }];
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::DeferredQueryEntryOutOfRange {
            entry: 1,
            entries: 1,
        }),
    );
}

#[test]
fn a_target_query_provider_is_distinct_from_a_selected_lowering_provider() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![DeferredPredicateSpec {
            requirement: prepared_requirement(
                1,
                TargetPropertyRequirementRelation::ObservedAtLeastRequired,
            ),
            entry: 0,
        }];
        draft.push_variant(program, spec)
    });
    assert!(outcome.is_ok());
}

#[test]
fn accepts_a_complete_prepared_entry_requirement() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider.clone())).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.deferred_predicates = vec![DeferredPredicateSpec {
        requirement: prepared_requirement(
            1,
            TargetPropertyRequirementRelation::ObservedAtLeastRequired,
        ),
        entry: 0,
    }];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().unwrap();
    let deferred = artifact
        .variants()
        .next()
        .expect("one variant")
        .deferred_predicates()
        .next()
        .expect("one deferred predicate");
    assert_eq!(deferred.entry().backend_entry_key().as_bytes(), b"fused");
    assert_eq!(deferred.requirement().required(), 1);
    assert_eq!(
        deferred.requirement().relation(),
        TargetPropertyRequirementRelation::ObservedAtLeastRequired,
    );
    assert_eq!(
        deferred.requirement().query().provider().name(),
        "prepared-entry-properties",
    );
}

#[test]
fn a_reversed_directional_predicate_does_not_match_its_requirement() {
    let requirement = prepared_requirement(
        8,
        TargetPropertyRequirementRelation::ObservedAtLeastRequired,
    );
    let roots = [
        ExprNode::Root(AbiRoot::TargetProperty {
            key: requirement.query().key().clone(),
            phase: requirement.query().available_at(),
        }),
        ExprNode::Root(AbiRoot::UnsignedLiteral(requirement.required())),
    ];
    let correct = [
        roots[0].clone(),
        roots[1].clone(),
        ExprNode::Binary {
            op: AbiBinaryOp::LessOrEqual,
            left: 1,
            right: 0,
        },
    ];
    assert!(
        super::model::deferred_predicate_matches_requirement(&correct, 2, &requirement),
        "required <= observed is the admitted direction",
    );

    let reversed = [
        roots[0].clone(),
        roots[1].clone(),
        ExprNode::Binary {
            op: AbiBinaryOp::LessOrEqual,
            left: 0,
            right: 1,
        },
    ];
    assert!(
        !super::model::deferred_predicate_matches_requirement(&reversed, 2, &requirement),
        "observed <= required must not masquerade as an at-least requirement",
    );
}

#[test]
fn rejects_a_repeated_deferred_predicate() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let predicate = DeferredPredicateSpec {
            requirement: prepared_requirement(
                1,
                TargetPropertyRequirementRelation::ObservedAtLeastRequired,
            ),
            entry: 0,
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

/// Every variant of one artifact declares one target profile.
///
/// The refusal that makes "share one compiled object across variants declaring
/// different profiles" a shape no artifact can express — the sentence
/// `docs/artifact-abi.md` withdrew for exactly that reason. It went unpinned
/// through the delivery-position step, so nothing would have caught the check
/// weakening and leaving the contract describing a build that no longer
/// refused.
///
/// The accepting half is asserted first and is load-bearing: the two variants
/// differ only in their declared profile between the halves, so a refusal alone
/// could not distinguish this rule from the duplicate-variant and delivery
/// rules beside it. Both fields of the profile are exercised, because a
/// descriptor that moved under an unchanged key is a different target with the
/// same name.
#[test]
fn refuses_a_second_variant_declaring_a_different_target_profile() {
    let semantic = semantic_program();
    let first = fused_program(&semantic, SCALE_BITS);
    let second = fused_program(&semantic, OTHER_SCALE_BITS);
    let provider = lowering_provider(1);

    let assemble = |declared: TargetProfileRef| {
        let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
        draft.select_provider(selection(provider.clone())).unwrap();
        let primary = draft.push_payload(payload(0xa1)).unwrap();
        let spare = draft.push_payload(payload(0xb1)).unwrap();
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&first, variant(&formulas, primary, b"fused"))
            .unwrap();
        let mut spec = variant(&formulas, spare, b"alternate");
        spec.target_profile = declared;
        draft.push_variant(&second, spec).map(|_| ())
    };

    assert_eq!(
        assemble(profile()),
        Ok(()),
        "agreeing siblings are accepted"
    );
    assert_eq!(
        assemble(TargetProfileRef {
            key: TargetProfileKey::new("tiler.test.other").unwrap(),
            descriptor: profile().descriptor,
        }),
        Err(ArtifactBuildError::TargetProfileMismatch),
    );
    assert_eq!(
        assemble(TargetProfileRef {
            key: profile().key,
            descriptor: TargetProfileDescriptorDigest::from_bytes([0x09, 0x09]).unwrap(),
        }),
        Err(ArtifactBuildError::TargetProfileMismatch),
        "the descriptor is half the profile, not decoration",
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
    // A record with no entry binding, because there is no packaged entry to
    // bind: what is under test is the empty portfolio, and a record naming an
    // entry this draft does not have would be refused for that instead.
    draft
        .declare_realization(realization_record(
            &profile(),
            &ScalarArithmeticSubject::f32(),
            strict(),
            0,
        ))
        .expect("a record over no packaged entry");
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
    declare_realization(&mut draft, &program);
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
    declare_realization(&mut draft, &program);
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
    declare_realization(&mut draft, &program);
    assert_eq!(
        draft
            .build()
            .expect_err("an unreferenced payload is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::UnusedPayload],
    );
}

/// Two delivery positions carry two objects realizing the same entries.
///
/// The positive case the whole delivery-position record exists for: a selection
/// built for two consumer targets is one plan, one kernel program, and two
/// compiled objects, so the entry names one backend entry key at two payloads
/// and both are referenced. Identity folds the run *as stated*, which the
/// sibling case below turns into a measurement.
#[test]
fn packages_one_payload_per_delivery_position() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let first = draft.push_payload(payload(0xa1)).unwrap();
    let second = draft.push_payload(payload(0xb1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, first, b"fused");
    spec.entries[0].implementation.payloads = vec![first, second];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().expect("both positions are realized");

    assert_eq!(artifact.delivery_positions(), 2);
    assert_eq!(artifact.payloads().len(), 2);
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    assert_eq!(entry.payloads().len(), 2);
    assert_ne!(
        entry.payload(0).expect("position 0").digest,
        entry.payload(1).expect("position 1").digest,
        "one payload per built family means two objects, not one under two names",
    );
    assert!(entry.payload(2).is_none());
}

/// A one-position artifact and a two-position one are never one artifact.
///
/// The identity consequence the `tiler.artifact-program.v13` step exists for.
/// The one-position artifact below carries the *first* of the two-position
/// artifact's payloads, so the two differ only in whether a second family was
/// built — and an identity that folded only the sorted payload table would
/// still distinguish them. What this pins is stronger and is the property a
/// cache needs: the *order* is folded too, so the same two payloads delivered
/// the other way round is a third artifact.
#[test]
fn delivery_order_and_count_are_both_artifact_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);

    let build = |positions: &[u8]| {
        let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
        draft.select_provider(selection(provider.clone())).unwrap();
        let declared: Vec<_> = positions
            .iter()
            .map(|tag| draft.push_payload(payload(*tag)).unwrap())
            .collect();
        let formulas = formulas(&mut draft);
        let mut spec = variant(&formulas, declared[0], b"fused");
        spec.entries[0].implementation.payloads = declared;
        draft.push_variant(&program, spec).unwrap();
        declare_realization(&mut draft, &program);
        draft
            .build()
            .expect("every declared payload is realized")
            .canonical_identity()
            .as_bytes()
            .to_vec()
    };

    let one = build(&[0xa1]);
    let two = build(&[0xa1, 0xb1]);
    let reversed = build(&[0xb1, 0xa1]);
    assert_ne!(one, two, "a second family is a second artifact");
    assert_ne!(two, reversed, "delivery order is meaning");
}

/// An entry naming no payload has no consumer target that could dispatch it.
#[test]
fn rejects_an_entry_realized_at_no_delivery_position() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].implementation.payloads = Vec::new();
    assert_eq!(
        draft.push_variant(&program, spec),
        Err(ArtifactBuildError::EmptyDelivery { entry: 0 }),
    );
}

/// Every entry of an artifact is realized at the same delivery positions.
///
/// A consumer resolves one position for the whole artifact, so an entry short of
/// it would leave that consumer with no object for a stage its route must
/// dispatch. The two-stage program makes the disagreement expressible: entry 0
/// establishes two positions and entry 1 declares one.
#[test]
fn rejects_an_entry_disagreeing_about_delivery_positions() {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let first = draft.push_payload(payload(0xa1)).unwrap();
    let second = draft.push_payload(payload(0xb1)).unwrap();
    let mut spec = partial_window_variant(first);
    spec.entries[0].implementation.payloads = vec![first, second];
    assert_eq!(
        draft.push_variant(&program, spec),
        Err(ArtifactBuildError::DeliveryCardinality {
            entry: 1,
            expected: 2,
            actual: 1,
        }),
    );
}

/// One payload may not stand in for two consumer build targets.
///
/// Two entries, two objects, and each object reached from a different delivery
/// position by one entry and the other position by the other. Every payload is
/// referenced and no `(payload, entry key)` pair repeats, so neither existing
/// obligation notices; what is wrong is that the artifact declares two consumer
/// targets and carries one object for each *position*, mixed. The neutral layer
/// cannot decide which target a shared object was built for, so it refuses the
/// shape.
#[test]
fn rejects_a_payload_reached_from_two_delivery_positions() {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let first = draft.push_payload(payload(0xa1)).unwrap();
    let second = draft.push_payload(payload(0xb1)).unwrap();
    let mut spec = partial_window_variant(first);
    spec.entries[0].implementation.payloads = vec![first, second];
    spec.entries[1].implementation.payloads = vec![second, first];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    let diagnostics = draft
        .build()
        .expect_err("one object cannot serve two delivery positions")
        .diagnostics()
        .to_vec();
    assert_eq!(
        diagnostics,
        [ArtifactDiagnostic::AmbiguousPayloadDelivery { payload: 1 }],
    );
}

/// One entry naming one payload at two positions is refused twice over.
///
/// Kept separate from the case above because it is decided by *both*
/// obligations, and observing that is what says the older one still does work
/// here: repeating a payload within one entry repeats a `(payload, entry key)`
/// pair, which the backend-entry injectivity rule that predates delivery
/// positions catches on its own, and it also puts one object at two positions.
/// Whole-artifact verification reports every diagnostic, so both are asserted
/// rather than only whichever happens to be first.
#[test]
fn rejects_one_payload_at_two_positions_of_one_entry() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].implementation.payloads = vec![descriptor, descriptor];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    assert_eq!(
        draft
            .build()
            .expect_err("one object cannot realize one entry twice")
            .diagnostics(),
        [
            ArtifactDiagnostic::DuplicateBackendEntry,
            ArtifactDiagnostic::AmbiguousPayloadDelivery { payload: 0 },
        ],
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
    declare_realization_over(&mut draft, &first, 2);
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
    // Exercised through a launch precondition rather than the launch geometry.
    // The geometry is derived from the program now and that program's is a
    // constant, so it evaluates without consulting any fact -- which would make
    // this test pass for the wrong reason. A precondition is still
    // caller-supplied and can name a fact that is deliberately left unbound.
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let rows = draft
        .push_root(AbiRoot::InputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(0),
        })
        .unwrap();
    let predicate = draft
        .push_binary(AbiBinaryOp::LessOrEqual, formulas.one, rows)
        .unwrap();
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].launch.preconditions = vec![predicate];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().unwrap();

    let facts = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight).build();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let precondition = entry
        .launch_preconditions()
        .next()
        .expect("one launch precondition");
    assert_eq!(
        precondition.evaluate(&facts),
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
// A governed key is spelled in one alphabet
// -------------------------------------------------------------------------

/// Every governed key refuses a byte outside the governed-key alphabet.
///
/// One case per key type rather than one case, because the six share a single
/// validator through the `governed_key!` macro and the per-type `kind` is the
/// only thing their refusals differ by. A key wired to the wrong subject would
/// report a rejection about something the producer did not write, and the
/// shared validator is exactly what stops that from being caught elsewhere.
///
/// The refused bytes are the classes the alphabet exists to exclude: case,
/// which leaves two keys a reader sees as one comparing unequal; a space, a
/// NUL, and a non-ASCII byte, which cannot be reproduced from the rejection
/// that prints them; and a separator from another naming scheme, which would
/// let one subject be spelled two ways.
#[test]
fn a_governed_key_refuses_a_byte_outside_the_governed_alphabet() {
    assert_eq!(
        BackendKey::new("tiler.Metal"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::Backend,
            index: 6,
            value: b'M',
        }),
    );
    assert_eq!(
        RepresentationKey::new("metal lib"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::Representation,
            index: 5,
            value: b' ',
        }),
    );
    assert_eq!(
        TargetProfileKey::new("tiler.target\0"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::TargetProfile,
            index: 12,
            value: 0,
        }),
    );
    assert_eq!(
        FeasibilityRuleSetKey::new("tiler/feasibility"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::FeasibilityRuleSet,
            index: 5,
            value: b'/',
        }),
    );
    assert_eq!(
        CapabilityKey::new("tiler.capability.fusé"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::Capability,
            index: 20,
            value: 0xc3,
        }),
    );
    assert_eq!(
        RouteFeatureKey::new("tiler.test.strict-f32!"),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::RouteFeature,
            index: 21,
            value: b'!',
        }),
    );
}

/// The decoding constructor enforces the same grammar as the building one.
///
/// `from_owned` is what the decoder calls on every governed key it reads out of
/// foreign bytes, and the macro gives it its own body rather than routing it
/// through `new`. A grammar only `new` enforced would be a producer courtesy
/// instead of the boundary check this layer exists to perform.
#[test]
fn the_decoding_constructor_enforces_the_governed_alphabet() {
    assert_eq!(
        TargetProfileKey::from_owned("tiler.Target.v1".to_owned()),
        Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind: ArtifactKeyKind::TargetProfile,
            index: 6,
            value: b'T',
        }),
    );
    BackendKey::from_owned("tiler.metal".to_owned())
        .expect("a canonically spelled key is admitted");
}

/// The empty and bound refusals fire too, and the bound stays this layer's own.
///
/// The maximum-length case is the deliberate half of the reconciliation: this
/// bound is what the artifact layer will *hold*, not what any one producer will
/// *mint*, so its own maximum is admitted rather than narrowed to the smaller
/// minting bound `tiler_compiler::target::MAX_TARGET_PROFILE_KEY_BYTES` sets.
#[test]
fn a_governed_key_refuses_an_empty_and_an_oversized_spelling() {
    assert_eq!(
        BackendKey::new(""),
        Err(ArtifactBuildError::EmptyKey {
            kind: ArtifactKeyKind::Backend,
        }),
    );
    assert_eq!(
        TargetProfileKey::new("a".repeat(super::MAX_GOVERNED_KEY_BYTES + 1)),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::TargetProfile,
            bytes: super::MAX_GOVERNED_KEY_BYTES + 1,
            limit: super::MAX_GOVERNED_KEY_BYTES,
        }),
    );
    TargetProfileKey::new("a".repeat(super::MAX_GOVERNED_KEY_BYTES))
        .expect("the admission bound admits its own maximum");
}

// -------------------------------------------------------------------------
// Received opaque identities are bounded by whoever mints them
// -------------------------------------------------------------------------

/// Each opaque identity is bounded by the authority that derives its subject.
///
/// **The over-bound vector is fabricated, and its length is derived rather than
/// measured.** It is one byte past [`super::MAX_OPAQUE_IDENTITY_BYTES`] — the
/// smallest length the shared bound refuses — so it states only that a
/// `BackendEntryKey` is admitted past that bound, which is this case's whole
/// subject. No kernel is involved and none can be: this crate carries no
/// `tiler-compiler` edge, for the reason stated above `[dependencies]` in its
/// manifest, so it can never compile a real reduction to measure one.
///
/// **The measured claim lives in `tiler-conformance`**, whose
/// `serial_sum::tests::the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor`
/// compiles a serial `f32` sum at one and at two contributors and asserts the
/// crossing from both sides. A length written here would restate a figure from a
/// tree that has since moved, which is what the previous `vec![0x5a; 1_121]`
/// named "measured" did: 1,121 was the two-contributor identity on 2026-07-25
/// and it measured 1,309 on 2026-08-08, while this case stayed green throughout.
///
/// The fixed-width payload digest keeps the smaller bound, while the structured
/// target-profile descriptor takes the compiler's larger minting bound.
#[test]
fn an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it() {
    BackendEntryKey::from_bytes(vec![0x5a; super::MAX_OPAQUE_IDENTITY_BYTES + 1])
        .expect("a backend entry key is admitted past the shared opaque-identity bound");

    assert_eq!(
        BackendEntryKey::from_bytes(vec![0x5a; MAX_KERNEL_IDENTITY_BYTES + 1]),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::BackendEntry,
            bytes: MAX_KERNEL_IDENTITY_BYTES + 1,
            limit: MAX_KERNEL_IDENTITY_BYTES,
        }),
        "beyond what the shared IR can mint, the refusal is still loud",
    );

    assert_eq!(
        PayloadDigest::from_bytes(vec![0x5a; super::MAX_OPAQUE_IDENTITY_BYTES + 1]),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::PayloadDigest,
            bytes: super::MAX_OPAQUE_IDENTITY_BYTES + 1,
            limit: super::MAX_OPAQUE_IDENTITY_BYTES,
        }),
    );
    assert_eq!(
        TargetProfileDescriptorDigest::from_bytes(vec![
            0x5a;
            super::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES
                + 1
        ]),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::TargetProfileDescriptor,
            bytes: super::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES + 1,
            limit: super::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
        }),
    );
}

/// The bound admits every entry key the packaged program itself carries.
///
/// An artifact carries one entry's kernel identity twice — as the entry key,
/// and inside the stage subject `stage_key` derives — so the two bounds have to
/// admit the same values or an artifact could be built and not encoded. This
/// asserts the first half against the second at a length the old bound refused.
///
/// That length is derived from [`super::MAX_OPAQUE_IDENTITY_BYTES`] rather than
/// written out, for the reason
/// [`an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it`] states:
/// the smallest refused length is the one this case wants, and a literal here
/// would be a figure about a tree rather than about the bound.
#[test]
fn an_artifact_encodes_an_entry_key_longer_than_the_digest_bound() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let long_key = vec![0x5a; super::MAX_OPAQUE_IDENTITY_BYTES + 1];
    draft
        .push_variant(&program, variant(&formulas, descriptor, &long_key))
        .unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().unwrap();

    let bytes = artifact.encode().expect("the envelope encodes");
    let decoded = super::decode_artifact(&bytes).expect("the envelope decodes");
    assert_eq!(
        decoded
            .variants()
            .next()
            .expect("one variant")
            .entries()
            .next()
            .expect("one entry")
            .backend_entry_key()
            .as_bytes(),
        long_key.as_slice(),
    );
}

// -------------------------------------------------------------------------
// Live input-extent operand envelope row
// -------------------------------------------------------------------------

#[test]
fn a_live_extent_operand_round_trips_through_the_envelope() {
    let artifact = live_extent_artifact();
    let extents: Vec<_> = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry")
        .extent_operands()
        .collect();
    assert_eq!(extents.len(), 1);
    assert_eq!(extents[0].key().as_str(), "input");
    assert_eq!(extents[0].axis(), Axis::new(1));
    assert_eq!(extents[0].value_type(), super::AbiType::Unsigned);

    let bytes = artifact.encode().expect("a live-extent artifact encodes");
    let decoded = super::decode_artifact(&bytes).expect("the envelope decodes");
    let decoded_extents: Vec<_> = decoded
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry")
        .extent_operands()
        .collect();
    assert_eq!(decoded_extents.len(), 1);
    assert_eq!(decoded_extents[0].key().as_str(), "input");
    assert_eq!(decoded_extents[0].axis(), Axis::new(1));
    assert_eq!(decoded_extents[0].value_type(), super::AbiType::Unsigned);

    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_extent(InputKey::new("input").unwrap(), Axis::new(1), 15)
        .unwrap();
    let facts = binder.build();
    assert_eq!(
        facts.input_extent(&InputKey::new("input").unwrap(), Axis::new(1)),
        Some(15),
        "the same AbiFacts used for range and launch answer the live extent",
    );
    assert_eq!(
        decoded
            .variants()
            .next()
            .expect("one variant")
            .entries()
            .next()
            .expect("one entry")
            .launch_threads()
            .evaluate(&facts)
            .expect("launch evaluates from the same facts"),
        super::AbiValue::Unsigned(2),
    );
}

#[test]
fn empty_extent_lists_do_not_move_previously_encodable_artifact_bytes() {
    let without = default_artifact();
    let again = default_artifact();
    assert_eq!(
        without.canonical_identity().as_bytes(),
        again.canonical_identity().as_bytes(),
        "two no-extent artifacts must keep one identity",
    );
    assert!(
        super::model::ARTIFACT_DOMAIN.ends_with(b"v16\0"),
        "empty extent lists write nothing, so the artifact identity domain must not step",
    );
    let with = live_extent_artifact();
    assert_ne!(
        without.canonical_identity().as_bytes(),
        with.canonical_identity().as_bytes(),
        "declaring a live extent is a new subject, not a reinterpretation",
    );
}

/// Dense F32 `[2, N]`: semantic `(row = 1, column = 0)` is element `N`, so bytes `4N`.
const fn dense_f32_row_major_bytes(row: u64, column: u64, inner_extent: u64) -> u64 {
    4 * (row * inner_extent + column)
}

/// One live-extent artifact, two bound N values, and a baked neighbour.
///
/// The bound value is not an artifact subject. Baking either neighbour is.
#[test]
fn one_live_extent_artifact_indexes_dense_f32_at_two_n_without_baking() {
    let artifact = live_extent_artifact();
    let again = live_extent_artifact();
    assert_eq!(
        artifact.canonical_identity().as_bytes(),
        again.canonical_identity().as_bytes(),
        "two constructions of the live subject must keep one identity",
    );

    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let extents: Vec<_> = entry.extent_operands().collect();
    assert_eq!(extents.len(), 1);
    assert_eq!(extents[0].key().as_str(), "input");
    assert_eq!(extents[0].axis(), Axis::new(1));

    let mut addresses = Vec::new();
    for n in [14_u64, 15] {
        let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
        binder
            .bind_input_extent(InputKey::new("input").unwrap(), Axis::new(1), n)
            .unwrap();
        let facts = binder.build();
        assert_eq!(
            facts.input_extent(&InputKey::new("input").unwrap(), Axis::new(1)),
            Some(n),
            "range, launch, and the payload operand share this bound fact",
        );
        let bindings: Vec<_> = entry.bindings().collect();
        assert_eq!(
            bindings[0]
                .accessible_bytes()
                .evaluate(&facts)
                .expect("range evaluates from the bound live extent"),
            super::AbiValue::Unsigned(0),
            "the LiveRowMajor window is zero-length; the expression still names N={n}",
        );
        assert_eq!(
            entry
                .launch_threads()
                .evaluate(&facts)
                .expect("launch evaluates from the same facts"),
            super::AbiValue::Unsigned(2),
        );
        let address = dense_f32_row_major_bytes(1, 0, n);
        addresses.push(address);
    }
    assert_eq!(
        addresses,
        [56, 60],
        "semantic (row = 1, column = 0) at N=14 and N=15"
    );

    let baked_14 = baked_dense_artifact(14);
    let baked_15 = baked_dense_artifact(15);
    assert_ne!(
        artifact.canonical_identity().as_bytes(),
        baked_14.canonical_identity().as_bytes(),
        "baking N = 14 must change artifact identity",
    );
    assert_ne!(
        baked_14.canonical_identity().as_bytes(),
        baked_15.canonical_identity().as_bytes(),
        "baking neighbouring extents must change artifact identity",
    );
}

/// C1 sequence extents: prefill at 10 and eight decode steps through 18.
const C1_SEQUENCE_EXTENTS: [u64; 9] = [10, 11, 12, 13, 14, 15, 16, 17, 18];

const RETAINED_HEADS: u64 = 8;
const RETAINED_WIDTH: u64 = 128;
const RETAINED_CAPACITY: u64 = 18;
const RETAINED_ELEMENT_BYTES: u64 = 4;

const fn exact_live_span(sequence: u64) -> u64 {
    RETAINED_HEADS * sequence * RETAINED_WIDTH * RETAINED_ELEMENT_BYTES
}

const fn exact_live_head1(sequence: u64) -> u64 {
    sequence * RETAINED_WIDTH * RETAINED_ELEMENT_BYTES
}

const fn capacity_strided_head1(capacity: u64) -> u64 {
    capacity * RETAINED_WIDTH * RETAINED_ELEMENT_BYTES
}

const fn retained_pool_bytes() -> u64 {
    RETAINED_CAPACITY * RETAINED_HEADS * RETAINED_WIDTH * RETAINED_ELEMENT_BYTES
}

/// One artifact identity across C1's nine extents, with the ≡ 0 (mod 16) guard
/// selecting only at 16.
#[test]
fn one_artifact_identity_routes_c1_extents_and_selects_the_aligned_guard_at_sixteen() {
    let artifact = live_extent_c1_portfolio();
    let again = live_extent_c1_portfolio();
    assert_eq!(
        artifact.canonical_identity().as_bytes(),
        again.canonical_identity().as_bytes(),
        "reassembling the C1 portfolio must keep one identity",
    );

    let mut variants: Vec<_> = artifact.variants().collect();
    assert_eq!(
        variants.len(),
        2,
        "two complete variants, not one per extent"
    );
    let aligned = variants.remove(0);
    let direct = variants.remove(0);
    assert_eq!(aligned.routing_rank(), 0);
    assert_eq!(direct.routing_rank(), 1);

    let mut selected = Vec::new();
    for sequence in C1_SEQUENCE_EXTENTS {
        let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
        binder
            .bind_input_extent(InputKey::new("input").unwrap(), Axis::new(1), sequence)
            .unwrap();
        let facts = binder.build();
        let aligned_holds = aligned
            .applicability_guard()
            .evaluate(&facts)
            .expect("the aligned guard evaluates from the bound extent");
        let direct_holds = direct
            .applicability_guard()
            .evaluate(&facts)
            .expect("the direct guard evaluates from the bound extent");
        assert_eq!(
            direct_holds,
            super::AbiValue::Boolean(true),
            "the fallback variant must remain applicable at S={sequence}",
        );
        let rank = if aligned_holds == super::AbiValue::Boolean(true) {
            0
        } else {
            assert_eq!(
                aligned_holds,
                super::AbiValue::Boolean(false),
                "the aligned guard must be boolean at S={sequence}",
            );
            1
        };
        selected.push(rank);
    }
    assert_eq!(
        selected,
        [1, 1, 1, 1, 1, 1, 0, 1, 1],
        "StablePriority must select the ≡ 0 (mod 16) variant only at S=16",
    );
}

/// Exact-live addressing inside a capacity-sized pool: span and head-1 come
/// from the bound extent, never from the allocation length.
#[test]
fn a_longer_pool_addresses_the_exact_live_span_and_capacity_stride_fails_the_oracle() {
    assert_eq!(retained_pool_bytes(), 73_728);
    assert_eq!(exact_live_span(14), 57_344);
    assert_eq!(exact_live_span(15), 61_440);
    assert_eq!(exact_live_head1(14), 7_168);
    assert_eq!(exact_live_head1(15), 7_680);
    assert_eq!(capacity_strided_head1(RETAINED_CAPACITY), 9_216);

    let artifact = live_extent_artifact();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let mut addresses = Vec::new();
    let mut spans = Vec::new();
    for sequence in [14_u64, 15] {
        let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
        binder
            .bind_input_extent(InputKey::new("input").unwrap(), Axis::new(1), sequence)
            .unwrap();
        let facts = binder.build();
        let bound = facts
            .input_extent(&InputKey::new("input").unwrap(), Axis::new(1))
            .expect("the live sequence is the bound fact");
        assert_eq!(bound, sequence, "the bound fact is S, not the pool length");
        assert_eq!(
            entry
                .extent_operands()
                .next()
                .expect("the payload reads the live axis")
                .axis(),
            Axis::new(1),
            "the routed operand is the bound sequence axis, not the allocation",
        );
        let allocation_as_sequence =
            retained_pool_bytes() / (RETAINED_HEADS * RETAINED_WIDTH * RETAINED_ELEMENT_BYTES);
        assert_eq!(allocation_as_sequence, RETAINED_CAPACITY);
        assert_ne!(
            bound, allocation_as_sequence,
            "deriving S from the allocation would silently pick the capacity stride",
        );
        addresses.push(exact_live_head1(bound));
        spans.push(exact_live_span(bound));
    }
    assert_eq!(addresses, [7_168, 7_680]);
    assert_eq!(spans, [57_344, 61_440]);
    assert_ne!(
        exact_live_head1(14),
        capacity_strided_head1(RETAINED_CAPACITY),
        "the capacity-strided head-1 address 9,216 must fail the retained oracle",
    );
}

/// A kernel that baked a bound extent is refused at assembly, not packaged.
#[test]
fn packaging_a_kernel_specialized_on_a_bound_extent_is_refused() {
    let semantic = baked_semantic_program(14);
    let program = baked_dense_program_with_live_range(&semantic, 14);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let error = draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .expect_err("a baked kernel must not assemble over a bound extent");
    assert_eq!(
        error,
        ArtifactBuildError::BoundExtentSpecialization {
            entry: 0,
            key: "input".to_owned(),
            axis: 1,
            element_count: 28,
        },
        "the refusal must name the bound-extent specialization, not a later check",
    );
}

/// A launch that names a different axis than the payload operand refuses before
/// the bound N can be used as two meanings.
#[test]
fn a_host_side_payload_disagreement_refuses_before_program_work() {
    let artifact = live_extent_two_n_artifact();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let precondition = entry
        .launch_preconditions()
        .next()
        .expect("the two-N artifact names the live axis in a launch precondition");

    let mut only_rows = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    only_rows
        .bind_input_extent(InputKey::new("input").unwrap(), Axis::new(0), 2)
        .unwrap();
    let rows_only = only_rows.build();
    assert_eq!(
        precondition.evaluate(&rows_only),
        Err(AbiEvaluationError::UnboundInputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(1),
        }),
        "binding the static row axis is not an answer for the live inner extent",
    );

    let mut both = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    both.bind_input_extent(InputKey::new("input").unwrap(), Axis::new(0), 2)
        .unwrap();
    both.bind_input_extent(InputKey::new("input").unwrap(), Axis::new(1), 14)
        .unwrap();
    assert_eq!(
        precondition
            .evaluate(&both.build())
            .expect("the live axis answers the launch precondition"),
        super::AbiValue::Boolean(true),
    );
}

/// `LinearIdentity` over a baked `[2, N]`, packaged the same way as the live subject.
fn baked_dense_kernel(columns: u64) -> VerifiedKernel {
    let rows = 2_u64;
    let elements = rows
        .checked_mul(columns)
        .expect("the two-N fixture stays inside u64");
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region
        .iteration_shape(Shape::from_dims([rows, columns]))
        .unwrap();
    region
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
    for (witness, tensor) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
        ),
        (1, TensorRole::Output),
    ] {
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
    region
        .scalar_program(ScalarProgram::PointwiseF32(scale_bias_expression()))
        .unwrap();
    region.numerical(strict()).unwrap();
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

fn baked_dense_program(semantic: &SemanticProgram, columns: u64) -> VerifiedKernelProgram {
    let kernel = baked_dense_kernel(columns);
    let rows = 2_u64;
    let bytes = 4 * rows * columns;
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let external = plan
        .push_allocation(device(bytes, AllocationOwnership::External))
        .unwrap();
    let owned = plan
        .push_allocation(device(bytes, AllocationOwnership::Program))
        .unwrap();
    let value = |origin, role, shape| MaterializedValueSpec {
        origin,
        role,
        shape,
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
                Shape::from_dims([rows, columns]),
            ),
            external,
        )
        .unwrap();
    let result = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                Shape::from_dims([rows, columns]),
            ),
            owned,
        )
        .unwrap();
    let read = plan
        .push_view(
            source,
            ByteWindow {
                offset: 0,
                length: bytes,
            },
        )
        .unwrap();
    let write = plan
        .push_view(
            result,
            ByteWindow {
                offset: 0,
                length: bytes,
            },
        )
        .unwrap();
    let accessible = plan.push_abi_root(AbiRoot::UnsignedLiteral(bytes)).unwrap();
    let grid = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(rows * columns))
        .unwrap();
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
                view: read,
                mode: StageAccessMode::Read,
                accessible_bytes: accessible,
            },
            StageAccess {
                view: write,
                mode: StageAccessMode::Write,
                accessible_bytes: accessible,
            },
        ],
        StageLaunch {
            grid_threads: grid,
            threads_per_workgroup: one,
        },
    )
    .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

fn baked_semantic_program(columns: u64) -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().unwrap();
    let input = draft
        .input::<F32>(
            InputKey::new("input").unwrap(),
            Shape::from_dims([2, columns]),
        )
        .unwrap();
    let scale = F32Constant::apply(&mut draft, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut draft, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = F32Add::apply(&mut draft, product, bias).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), mapped)
        .unwrap();
    draft.build().unwrap()
}

fn baked_dense_artifact(columns: u64) -> VerifiedArtifactProgram {
    let semantic = baked_semantic_program(columns);
    let program = baked_dense_program(&semantic, columns);
    let provider = lowering_provider(1);
    build_artifact(&semantic, &program, provider.clone(), &[provider])
}

/// The baked `[2, N]` program whose accessible range still names `InputExtent`.
///
/// That pairing is the specialization the assembly check refuses: the kernel
/// folded `N` into `element_count` while the ABI treats the same axis as a
/// per-invocation binding.
fn baked_dense_program_with_live_range(
    semantic: &SemanticProgram,
    columns: u64,
) -> VerifiedKernelProgram {
    let kernel = baked_dense_kernel(columns);
    let rows = 2_u64;
    let bytes = 4 * rows * columns;
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let external = plan
        .push_allocation(device(bytes, AllocationOwnership::External))
        .unwrap();
    let owned = plan
        .push_allocation(device(bytes, AllocationOwnership::Program))
        .unwrap();
    let value = |origin, role, shape| MaterializedValueSpec {
        origin,
        role,
        shape,
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
                Shape::from_dims([rows, columns]),
            ),
            external,
        )
        .unwrap();
    let result = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                Shape::from_dims([rows, columns]),
            ),
            owned,
        )
        .unwrap();
    let read = plan
        .push_view(
            source,
            ByteWindow {
                offset: 0,
                length: bytes,
            },
        )
        .unwrap();
    let write = plan
        .push_view(
            result,
            ByteWindow {
                offset: 0,
                length: bytes,
            },
        )
        .unwrap();
    let four = plan.push_abi_root(AbiRoot::UnsignedLiteral(4)).unwrap();
    let row_count = plan.push_abi_root(AbiRoot::UnsignedLiteral(rows)).unwrap();
    let live_n = plan
        .push_abi_root(AbiRoot::InputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(1),
        })
        .unwrap();
    let row_bytes = plan
        .push_abi_binary(AbiBinaryOp::CheckedMultiply, four, row_count)
        .unwrap();
    let accessible = plan
        .push_abi_binary(AbiBinaryOp::CheckedMultiply, row_bytes, live_n)
        .unwrap();
    let grid = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(rows * columns))
        .unwrap();
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
                view: read,
                mode: StageAccessMode::Read,
                accessible_bytes: accessible,
            },
            StageAccess {
                view: write,
                mode: StageAccessMode::Write,
                accessible_bytes: accessible,
            },
        ],
        StageLaunch {
            grid_threads: grid,
            threads_per_workgroup: one,
        },
    )
    .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

/// The live artifact plus a launch precondition that names the same live axis.
fn live_extent_two_n_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = live_extent_program(&semantic);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let n = draft
        .push_root(AbiRoot::InputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(1),
        })
        .unwrap();
    let predicate = draft
        .push_binary(AbiBinaryOp::LessOrEqual, formulas.one, n)
        .unwrap();
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].launch.preconditions = vec![predicate];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
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
        let attributes = request.attributes();
        match self {
            Self::Constant => {
                outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
            }
            Self::Binary => {
                let left = request.static_operand_shape(0)?;
                let right = request.static_operand_shape(1)?;
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
                    request.static_operand_shape(0)?.without_axes(&axes),
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
            IndexRealizationLaw::constant_f32(),
        )?;
        register_test_operation(
            registrar,
            multiply_f32_op(),
            2,
            [],
            TestOperation::Binary,
            IndexRealizationLaw::multiply_f32(),
        )?;
        register_test_operation(
            registrar,
            add_f32_op(),
            2,
            [],
            TestOperation::Binary,
            IndexRealizationLaw::add_f32(),
        )?;
        register_test_operation(
            registrar,
            strict_serial_sum_f32_op(),
            1,
            [OperationAttributeSchema::required(
                REDUCTION_AXES_ATTRIBUTE,
                CanonicalValueKind::Sequence,
            )],
            TestOperation::Sum,
            IndexRealizationLaw::strict_serial_sum_f32(),
        )
    }
}

/// Registers one governed test operation together with its realization law.
///
/// The law travels with the operation because an operation without one cannot
/// be refined, and a stage covering it therefore cannot name the proof its
/// coverage record requires. A "governed" test provider that registered
/// operations and no laws would describe a registry no program could execute.
fn register_test_operation<const N: usize>(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    key: OpKey,
    operands: u32,
    attributes: [OperationAttributeSchema; N],
    inferencer: TestOperation,
    law: IndexRealizationLaw,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        key.clone(),
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
    ))?;
    registrar.register_index_realization_law(key, 1, law)
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

/// A scalar authority composed with an exact non-standard semantic registry.
///
/// The governed standard scalar profile is frozen over
/// [`FrozenSemanticRegistry::standard`], and a refinement verifier refuses a
/// scalar authority frozen over a different semantic authority — deliberately,
/// because the two together name what a region's arithmetic *means*. The
/// provider-provenance fixtures build their own semantic registries, so they
/// compose their own scalar profile over exactly those registries. The
/// definitions are the fixture's rather than the standard ones, which is
/// visible in the resulting evidence bytes and irrelevant to what these tests
/// compare: every artifact they compare was built through this same authority.
fn scalars_over(semantic: &FrozenSemanticRegistry) -> FrozenScalarRegistry {
    let provider = ProviderIdentity::new("tiler-test", "fixture-scalars", 1).unwrap();
    let mut builder = ScalarRegistryBuilder::new(semantic.clone());
    builder
        .register(
            provider.clone(),
            fixture_scalar_definition(
                constant_f32_scalar_op(),
                ScalarAttributeSchema::new([tiler_ir::index::ScalarAttributeField::required(
                    F32_CONSTANT_BITS_ATTRIBUTE,
                    CanonicalValueKind::FloatBits,
                )])
                .unwrap(),
                0,
            ),
        )
        .unwrap();
    for key in [multiply_f32_scalar_op(), add_f32_scalar_op()] {
        builder
            .register(
                provider.clone(),
                fixture_scalar_definition(key, ScalarAttributeSchema::empty(), 2),
            )
            .unwrap();
    }
    builder.freeze()
}

fn fixture_scalar_definition(
    key: ScalarOpKey,
    attributes: ScalarAttributeSchema,
    operands: usize,
) -> ScalarOperationDefinition {
    ScalarOperationDefinition::new(
        key,
        NormativeDefinitionRef::new("fixture scalar semantics").unwrap(),
        ScalarOperationContract::new(
            attributes,
            ScalarArity::exact(operands).unwrap(),
            ScalarArity::exact(1).unwrap(),
            ScalarEffect::Pure,
            CanonicalValue::record([]).unwrap(),
            CanonicalValue::record([]).unwrap(),
        ),
        Arc::new(FixtureF32Scalar),
    )
}

/// Every fixture scalar operation produces one `f32`.
///
/// A constant takes no operand, so the result type cannot be read off the
/// operands; the fixture's graph is homogeneous `f32` throughout, and this
/// states that rather than inferring it.
struct FixtureF32Scalar;

impl ScalarOperationInferencer for FixtureF32Scalar {
    fn infer(
        &self,
        _request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        outputs.try_push(F32::resolved_type())
    }
}

/// Artifact identity grows linearly with the ABI arena, on a chain and on a
/// shared DAG.
///
/// This is the instrument the flattening exists for, mirroring `tiler-ir`'s
/// `abi_identity_size_grows_linearly_with_the_arena`. Under the `v4` encoding a
/// node's key embedded its whole subtree, so the chain was quadratic and the
/// shared DAG **doubled per level** — a 16-level DAG reached megabytes. A
/// constant increment per level is the property that says the arena is written
/// once and referenced by position.
#[test]
fn artifact_identity_size_grows_linearly_with_the_abi_arena() {
    /// Enough levels that a quadratic or exponential curve is unmistakable, and
    /// few enough that a `v4` re-run would still finish.
    const LEVELS: std::ops::Range<usize> = 0..17;

    for shared in [false, true] {
        let mut sizes = Vec::new();
        for levels in LEVELS {
            let semantic = semantic_program();
            let program = fused_program(&semantic, SCALE_BITS);
            let provider = lowering_provider(1);
            let environment =
                CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
            let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
            draft.select_provider(selection(provider)).unwrap();
            let descriptor = draft.push_payload(payload(0xa1)).unwrap();
            let formulas = formulas(&mut draft);

            // Grow the guard, which is a use site, so every added node is
            // reached and verification admits the artifact.
            // Grown through a **launch precondition**, not the applicability
            // guard: the guard is derived from the program now, so it is no
            // longer a caller-supplied place to add arena depth. A precondition
            // is still artifact-owned and still reaches identity, so this
            // measures what it always measured -- identity size against arena
            // size -- through the seam that survives the binding.
            let mut grown = formulas.always;
            for _ in 0..levels {
                grown = if shared {
                    draft.push_binary(AbiBinaryOp::And, grown, grown).unwrap()
                } else {
                    let filler = draft.push_root(AbiRoot::BooleanLiteral(false)).unwrap();
                    draft.push_binary(AbiBinaryOp::Or, grown, filler).unwrap()
                };
            }
            let mut spec = variant(&formulas, descriptor, b"fused");
            spec.entries[0].launch.preconditions = vec![grown];
            draft.push_variant(&program, spec).unwrap();
            declare_realization(&mut draft, &program);
            let artifact = draft.build().unwrap();

            let nodes = artifact.expressions().len();
            let bytes = artifact.canonical_identity().as_bytes().len();
            let shape = if shared { "SharedDag" } else { "Chain" };
            println!("MEASURE {shape} {levels:>2} levels: {nodes:>3} nodes, {bytes} bytes");
            sizes.push((nodes, bytes));
        }

        let increments: Vec<usize> = sizes
            .windows(2)
            .skip(1)
            .map(|pair| pair[1].1 - pair[0].1)
            .collect();
        assert!(
            increments.windows(2).all(|pair| pair[0] == pair[1]),
            "identity size must grow by a constant per level, measured {increments:?}"
        );
    }
}

/// `adopt_abi` replays a program's arena and resolves every reached position.
///
/// This is the mechanism that makes "a variant's ABI is its program's ABI"
/// checkable instead of a producer convention. The dedup assertion is the part
/// worth having: the builder keys by content, so replaying an arena that names
/// one expression from two positions must yield one handle, or a variant would
/// carry two spellings of one formula and the identity would distinguish them.
#[test]
fn adopting_a_program_abi_replays_every_reached_position() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();

    let arena = program.abi_expressions();
    let roots: Vec<u32> = (0..u32::try_from(arena.len()).unwrap()).collect();
    let minted = draft.adopt_abi(arena, &roots).expect("the arena replays");

    assert_eq!(minted.len(), arena.len());
    assert!(
        minted.iter().all(Option::is_some),
        "every position was named as a root, so every one must be replayed"
    );

    // Replaying the same arena again must mint no new handles: the builder
    // deduplicates by content, so the second pass resolves to the first's.
    let again = draft
        .adopt_abi(arena, &roots)
        .expect("the arena replays twice");
    assert_eq!(
        minted, again,
        "replay is not idempotent, so content dedup failed"
    );
}

/// A root outside the arena is a typed rejection, not a panic.
#[test]
fn adopting_an_abi_with_an_out_of_range_root_is_rejected() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();

    let arena = program.abi_expressions();
    let beyond = u32::try_from(arena.len()).unwrap();
    assert_eq!(
        draft.adopt_abi(arena, &[beyond]),
        Err(ArtifactBuildError::ExpressionOutOfRange { position: beyond }),
    );
}

/// Does the artifact layer accept a *program-owned* ABI expression?
///
/// This is the question `reconcile-the-artifact-and-program-abi-expression-obligations`
/// exists to answer, isolated from the build path so a wiring fault in a
/// larger change cannot be mistaken for a layer disagreement. It adopts the
/// program's arena and then asks the artifact builder to accept the program's
/// own launch expression at the use site that expression is for.
#[test]
fn probe_whether_a_program_expression_satisfies_the_artifact_obligations() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();

    let stage = program.stages().next().expect("one stage");
    let launch = stage.launch();
    let roots = vec![launch.grid_threads, launch.threads_per_workgroup];
    let adopted = draft
        .adopt_abi(program.abi_expressions(), &roots)
        .expect("the program arena replays onto the artifact builder");

    let grid = adopted[usize::try_from(launch.grid_threads).unwrap()]
        .expect("the grid expression was replayed");
    let workgroup = adopted[usize::try_from(launch.threads_per_workgroup).unwrap()]
        .expect("the workgroup expression was replayed");

    println!("PROBE grid handle {grid:?} workgroup handle {workgroup:?}");
    println!(
        "PROBE program arena {} nodes",
        program.abi_expressions().len()
    );
    println!("PROBE artifact arena after replay");

    // The handles are the artifact builder's own, minted by `adopt_abi`, so if
    // anything below fails it is an obligation and not a foreign handle.
    assert_ne!(grid, workgroup, "two distinct launch expressions collapsed");
}

// -------------------------------------------------------------------------
// Live-device route requirements
// -------------------------------------------------------------------------

/// Builds one well-formed backend feature row in the Metal namespace.
pub(crate) fn route_feature(key: &str, version: u32, payload: &[u8]) -> RouteRequirement {
    RouteRequirement::BackendFeature(
        BackendFeatureRequirement::new(
            BackendKey::new("tiler.metal").expect("a governed backend key"),
            RouteFeatureKey::new(key).expect("a governed route feature key"),
            version,
            payload,
        )
        .expect("a well-formed backend feature requirement"),
    )
}

/// Builds one well-formed quantitative row.
pub(crate) fn route_resource(required: u64) -> RouteRequirement {
    RouteRequirement::Resource(
        RouteResourceRequirement::new(RouteResourceDimension::SubgroupThreads, required)
            .expect("a nonzero required quantity"),
    )
}

/// Assembles the canonical artifact with route requirements attached.
///
/// Declaration order is the caller's, which is what makes the canonical-order
/// cases meaningful: the builder retains it and the envelope projection is where
/// it stops mattering.
pub(crate) fn requiring_artifact(requirements: &[RouteRequirement]) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let variant = draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    for requirement in requirements {
        draft
            .require_route(variant, requirement.clone())
            .expect("each requirement names a distinct subject");
    }
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
}

/// Every neutral dimension round-trips through its governed tag, distinctly.
///
/// Population-counted against `ALL` rather than against a list written here, so
/// a dimension added to the vocabulary lands in this check instead of leaving it
/// silently covering a subset.
#[test]
fn every_route_resource_dimension_round_trips_through_its_governed_tag() {
    let mut tags = Vec::new();
    for dimension in RouteResourceDimension::ALL {
        let tag = dimension.tag();
        assert_eq!(RouteResourceDimension::from_tag(tag), Some(dimension));
        assert!(!tags.contains(&tag), "tag {tag:#04x} is not distinct");
        tags.push(tag);
    }
    assert_eq!(
        tags.len(),
        RouteResourceDimension::ALL.len(),
        "every dimension the vocabulary names was checked",
    );
    assert_eq!(RouteResourceDimension::from_tag(0x00), None);
    assert_eq!(RouteResourceDimension::from_tag(0xff), None);
}

/// A subgroup width is satisfied by an exactly equal observation and nothing else.
///
/// The **wider** case is the load-bearing one and it is the case the superseded
/// floor accepted: a device executing more threads per subgroup than the route
/// was verified at runs lane arithmetic nothing checked, so admitting it is the
/// silent wrongness this relation exists to refuse. The narrower case is driven
/// beside it so a relation that refused everything could not pass as a fix.
///
/// Populations are named rather than sampled: every dimension the vocabulary
/// carries is exercised, and the count is asserted, so a dimension added without
/// a relation of its own lands here instead of leaving a subset checked.
#[test]
fn a_route_resource_row_is_satisfied_only_by_an_exactly_equal_observation() {
    const REQUIRED: u64 = 32;

    let mut checked = 0;
    for dimension in RouteResourceDimension::ALL {
        let row = RouteResourceRequirement::new(dimension, REQUIRED).expect("a nonzero quantity");
        assert_eq!(row.required(), REQUIRED);
        assert!(
            row.is_satisfied_by(REQUIRED),
            "{dimension} must accept the width the route was verified at",
        );
        assert!(
            !row.is_satisfied_by(REQUIRED - 1),
            "{dimension} must refuse a narrower device",
        );
        assert!(
            !row.is_satisfied_by(REQUIRED * 2),
            "{dimension} must refuse a wider device, which a floor accepted",
        );
        checked += 1;
    }
    assert_eq!(
        checked,
        RouteResourceDimension::ALL.len(),
        "every dimension the vocabulary names was checked",
    );
}

/// Every synchronization vocabulary round-trips through this crate's own tags.
///
/// Three separate tables, each a forward and inverse pair kept in one place, and
/// each counted against a population written *here* rather than derived from the
/// encoder — so a widened vocabulary in `tiler-ir` fails this count instead of
/// silently leaving the new variant untested. That the tables are this crate's
/// own copy is the design: the schedule identity and the artifact identity are
/// different subjects, and a shared table would let one domain's step move the
/// other's bytes.
#[test]
fn every_synchronization_vocabulary_round_trips_through_its_governed_tag() {
    use tiler_ir::schedule::{MemoryOrdering, SynchronizationKind, SynchronizationScope};

    let kinds = [
        SynchronizationKind::ControlBarrier,
        SynchronizationKind::AsynchronousCopy,
        SynchronizationKind::SplitPhaseBarrier,
        SynchronizationKind::Collective,
        SynchronizationKind::Atomic,
        SynchronizationKind::InterDispatchDependency,
    ];
    let mut tags = Vec::new();
    for kind in kinds {
        let tag = super::model::synchronization_kind_tag(kind);
        assert_eq!(super::model::synchronization_kind_from_tag(tag), Some(kind));
        assert!(!tags.contains(&tag), "kind tag {tag:#04x} is not distinct");
        tags.push(tag);
    }
    assert_eq!(tags.len(), 6, "every admitted-or-refused kind was checked");
    assert_eq!(super::model::synchronization_kind_from_tag(0x00), None);
    assert_eq!(super::model::synchronization_kind_from_tag(0xff), None);

    let scopes = [
        SynchronizationScope::Subgroup,
        SynchronizationScope::Workgroup,
        SynchronizationScope::Device,
    ];
    let mut tags = Vec::new();
    for scope in scopes {
        let tag = super::model::synchronization_scope_tag(scope);
        assert_eq!(
            super::model::synchronization_scope_from_tag(tag),
            Some(scope)
        );
        assert!(!tags.contains(&tag), "scope tag {tag:#04x} is not distinct");
        tags.push(tag);
    }
    assert_eq!(tags.len(), 3);
    assert_eq!(super::model::synchronization_scope_from_tag(0x00), None);
    assert_eq!(super::model::synchronization_scope_from_tag(0xff), None);

    let orderings = [
        MemoryOrdering::Relaxed,
        MemoryOrdering::AcquireRelease,
        MemoryOrdering::SequentiallyConsistent,
    ];
    let mut tags = Vec::new();
    for ordering in orderings {
        let tag = super::model::memory_ordering_tag(ordering);
        assert_eq!(super::model::memory_ordering_from_tag(tag), Some(ordering));
        assert!(
            !tags.contains(&tag),
            "ordering tag {tag:#04x} is not distinct"
        );
        tags.push(tag);
    }
    assert_eq!(tags.len(), 3);
    assert_eq!(super::model::memory_ordering_from_tag(0x00), None);
    assert_eq!(super::model::memory_ordering_from_tag(0xff), None);
}

/// The recorded absence of a synchronization requirement changes the bytes.
///
/// The load-bearing half of the `v14` step: an entry that requires no
/// realization writes a byte saying so, so its identity is not the identity an
/// entry that had never been able to state one would have had. Asserting the
/// *presence* of the recorded absence is what stops a later change quietly
/// reverting to omission, which would make a synchronized entry and an
/// unsynchronized one share bytes again.
#[test]
fn an_entry_records_the_absence_of_a_synchronization_requirement() {
    let artifact = default_artifact();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    assert_eq!(entry.resources().synchronization, None);

    // The presence byte is written, not omitted: encoding the same resource
    // record with and without it differ by exactly one byte.
    let mut with_absence = Vec::new();
    super::model::push_resources(&mut with_absence, entry.resources());
    let mut without = Vec::new();
    super::model::push_synchronization(&mut without, None);
    assert_eq!(without, vec![0x00], "absence is one recorded byte");
    assert!(
        with_absence.windows(1).any(|byte| byte == [0x00]),
        "the resource record carries the recorded absence"
    );

    // And a `Some` occupies seven, so no synchronized entry can encode into the
    // byte count an unsynchronized one occupies.
    let mut present = Vec::new();
    super::model::push_synchronization(
        &mut present,
        Some(tiler_ir::schedule::SynchronizationSubject {
            kind: tiler_ir::schedule::SynchronizationKind::ControlBarrier,
            execution_scope: tiler_ir::schedule::SynchronizationScope::Workgroup,
            visibility_scope: tiler_ir::schedule::SynchronizationScope::Workgroup,
            fenced_spaces: tiler_ir::schedule::FencedSpaces {
                workgroup: true,
                device: false,
            },
            ordering: tiler_ir::schedule::MemoryOrdering::AcquireRelease,
        }),
    );
    assert_eq!(present.len(), 7);
    assert_ne!(present[0], without[0]);
}

/// Each way a route requirement can be malformed is refused by its own cause.
///
/// Every case is paired with the well-formed neighbour it perturbs, so a
/// rejection is attributable to the one field that changed rather than to a
/// constructor that refuses everything.
#[test]
fn a_malformed_route_requirement_is_refused_by_its_own_cause() {
    assert!(RouteResourceRequirement::new(RouteResourceDimension::SubgroupThreads, 32).is_ok());
    assert_eq!(
        RouteResourceRequirement::new(RouteResourceDimension::SubgroupThreads, 0),
        Err(RouteRequirementError::ZeroResourceQuantity {
            dimension: RouteResourceDimension::SubgroupThreads,
        }),
    );

    let owner = || BackendKey::new("tiler.metal").unwrap();
    let key = || RouteFeatureKey::new("tiler.metal.route-requirement.minimum-gpu-family").unwrap();
    assert!(BackendFeatureRequirement::new(owner(), key(), 1, b"apple9").is_ok());
    assert_eq!(
        BackendFeatureRequirement::new(owner(), key(), 0, b"apple9"),
        Err(RouteRequirementError::ZeroFeatureVersion),
    );
    assert_eq!(
        BackendFeatureRequirement::new(owner(), key(), 1, b""),
        Err(RouteRequirementError::EmptyFeaturePayload),
    );
    let oversized = vec![0_u8; MAX_ROUTE_FEATURE_PAYLOAD_BYTES + 1];
    assert_eq!(
        BackendFeatureRequirement::new(owner(), key(), 1, &oversized),
        Err(RouteRequirementError::FeaturePayloadTooLong {
            bytes: MAX_ROUTE_FEATURE_PAYLOAD_BYTES + 1,
            limit: MAX_ROUTE_FEATURE_PAYLOAD_BYTES,
        }),
    );
    assert!(
        BackendFeatureRequirement::new(owner(), key(), 1, &oversized[1..]).is_ok(),
        "the bound admits exactly its own length",
    );
}

/// Two rows constraining one subject are refused at construction.
///
/// The differing quantity is what makes them contradictory: the builder holds
/// two answers to one question and nothing can say which the producer meant.
/// A *different* subject is accepted in the same breath, so the rejection is
/// about the subject rather than about a second row at all.
#[test]
fn two_route_requirements_naming_one_subject_are_refused() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let id = draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();

    draft.require_route(id, route_resource(32)).unwrap();
    assert_eq!(
        draft.require_route(id, route_resource(64)),
        Err(ArtifactBuildError::DuplicateRouteRequirementSubject {
            subject: Box::new(RouteRequirementSubject::Resource {
                dimension: RouteResourceDimension::SubgroupThreads,
            }),
        }),
    );
    // A distinct subject is admitted, so what was refused is the repetition.
    draft
        .require_route(
            id,
            route_feature("tiler.metal.route-requirement.a", 1, b"x"),
        )
        .unwrap();
    // The same key at another version is another subject, deliberately: one key
    // at two versions can mean two things, so they are not the same question.
    draft
        .require_route(
            id,
            route_feature("tiler.metal.route-requirement.a", 2, b"x"),
        )
        .unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().unwrap();
    assert_eq!(
        artifact
            .variants()
            .next()
            .expect("one variant")
            .route_requirements()
            .len(),
        3,
    );
}

/// A variant handle another builder minted cannot attach a requirement.
#[test]
fn a_route_requirement_needs_a_variant_this_builder_minted() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let mut first = {
        let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
        ArtifactProgramBuilder::new(&semantic, environment).unwrap()
    };
    let mut second = {
        let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
        ArtifactProgramBuilder::new(&semantic, environment).unwrap()
    };
    for draft in [&mut first, &mut second] {
        draft.select_provider(selection(provider.clone())).unwrap();
    }
    let descriptor = first.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut first);
    let foreign = first
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        second.require_route(foreign, route_resource(32)),
        Err(ArtifactBuildError::ForeignHandle {
            entity: ArtifactEntityKind::Variant,
        }),
    );
    // The handle is good against the builder that minted it, which is what makes
    // the refusal above about ownership rather than about the handle's shape.
    assert!(first.require_route(foreign, route_resource(32)).is_ok());
}

// -------------------------------------------------------------------------
// Exhaustive injectivity of the finite-domain identity encoders
// -------------------------------------------------------------------------
//
// An encoder is injective when no two distinct inputs produce the same bytes,
// and a collision in artifact identity is two different packaged programs
// answering to one name — a cache hit on the wrong artifact, not a miss. Most
// encoders here carry a `u32` ordinal, a slice, or a string, so their domains
// cannot be walked and their injectivity rests on the framing argument in
// `tiler_ir::identity`. The two below can be walked, and walking them turns the
// claim into exhaustive finite evidence: every pair is compared because every
// value is.
//
// The enumerations are sized by `variant_count`, so a vocabulary widened in
// `tiler-ir` is a build error here. That guard is why enumerating these
// vocabularies a second time on this side of the crate boundary is safe: the
// two lists cannot silently disagree about how large the domain is, because
// neither can silently stop covering it.

/// Every construct class an artifact's synchronization requirement can name.
const SYNCHRONIZATION_KINDS: [SynchronizationKind;
    std::mem::variant_count::<SynchronizationKind>()] = [
    SynchronizationKind::ControlBarrier,
    SynchronizationKind::AsynchronousCopy,
    SynchronizationKind::SplitPhaseBarrier,
    SynchronizationKind::Collective,
    SynchronizationKind::Atomic,
    SynchronizationKind::InterDispatchDependency,
];

/// Every invocation set an arrival or a publication can range over.
const SYNCHRONIZATION_SCOPES: [SynchronizationScope;
    std::mem::variant_count::<SynchronizationScope>()] = [
    SynchronizationScope::Subgroup,
    SynchronizationScope::Workgroup,
    SynchronizationScope::Device,
];

/// Every ordering a synchronization requirement can establish.
const MEMORY_ORDERINGS: [MemoryOrdering; std::mem::variant_count::<MemoryOrdering>()] = [
    MemoryOrdering::Relaxed,
    MemoryOrdering::AcquireRelease,
    MemoryOrdering::SequentiallyConsistent,
];

/// Returns the number of bools in one exhaustive field census.
const fn bool_field_count<const N: usize>(_: [bool; N]) -> usize {
    N
}

/// The independent boolean fields carried by [`FencedSpaces`].
const FENCED_SPACE_FIELD_COUNT: usize = {
    let FencedSpaces { workgroup, device } = FencedSpaces::NONE;
    bool_field_count([workgroup, device])
};

/// Every fence a synchronization requirement can name.
///
/// `FencedSpaces` is a struct, so `variant_count` does not apply. Each field in
/// the exhaustive census above is boolean, making the inhabitant count two to
/// the power of the field count.
const FENCED_SPACES: [FencedSpaces; 1 << FENCED_SPACE_FIELD_COUNT] = [
    FencedSpaces {
        workgroup: false,
        device: false,
    },
    FencedSpaces {
        workgroup: false,
        device: true,
    },
    FencedSpaces {
        workgroup: true,
        device: false,
    },
    FencedSpaces {
        workgroup: true,
        device: true,
    },
];

/// The artifact synchronization encoder is injective over all 649 inhabitants.
///
/// **Exhaustive finite evidence.** The domain is `Option<SynchronizationSubject>`:
/// the product of five closed vocabularies — 6 construct kinds, 3 arrival
/// scopes, 3 publication scopes, 4 fences, 3 orderings — plus the stated
/// absence. The subject's fields are independent and carry no constructor
/// invariant, so `6 * 3 * 3 * 4 * 3 + 1 = 649` is the inhabitant count and not
/// an estimate of it.
///
/// The three component tag tables are separately round-tripped elsewhere in this
/// crate, and that is a strictly weaker claim than this one: three injective
/// component maps can still compose into a non-injective record if a field is
/// dropped or written twice. Only the product distinguishes those, which is why
/// it is enumerated rather than inferred.
#[test]
fn the_artifact_synchronization_encoding_is_injective_over_its_whole_domain() {
    const POPULATION: usize = 1 + SYNCHRONIZATION_KINDS.len()
        * SYNCHRONIZATION_SCOPES.len()
        * SYNCHRONIZATION_SCOPES.len()
        * FENCED_SPACES.len()
        * MEMORY_ORDERINGS.len();

    let mut subjects: Vec<Option<SynchronizationSubject>> = vec![None];
    for kind in SYNCHRONIZATION_KINDS {
        for execution_scope in SYNCHRONIZATION_SCOPES {
            for visibility_scope in SYNCHRONIZATION_SCOPES {
                for fenced_spaces in FENCED_SPACES {
                    for ordering in MEMORY_ORDERINGS {
                        subjects.push(Some(SynchronizationSubject {
                            kind,
                            execution_scope,
                            visibility_scope,
                            fenced_spaces,
                            ordering,
                        }));
                    }
                }
            }
        }
    }

    assert_eq!(subjects.len(), POPULATION);
    assert_eq!(
        POPULATION, 649,
        "the subject domain changed size; the exhaustive claim is about whatever it is now, \
         so restate it deliberately"
    );

    let mut seen: HashMap<Vec<u8>, Option<SynchronizationSubject>> =
        HashMap::with_capacity(POPULATION);
    for subject in subjects {
        let mut bytes = Vec::new();
        push_synchronization(&mut bytes, subject);
        // One presence tag, and six subject bytes when present. The width is
        // variable, so what keeps the record unambiguous is the presence tag —
        // and the collision check below is what confirms it is doing that work.
        let expected = if subject.is_some() { 7 } else { 1 };
        assert_eq!(bytes.len(), expected, "{subject:?} changed width");
        if let Some(previous) = seen.insert(bytes, subject) {
            panic!("{subject:?} and {previous:?} share one encoding");
        }
    }
    assert_eq!(seen.len(), POPULATION);
}

/// The artifact storage-encoding encoder is injective over every constructible value.
///
/// **Exhaustive finite evidence over the constructible domain.**
/// `BitPackedEncoding` has private fields and one constructor, which admits only
/// element widths below eight that divide eight. The sweep offers all 512
/// `(u8, PackedBitOrder, PackedTailRule)` candidates to that constructor and
/// enumerates the survivors, so the population is *derived* from the admission
/// rule instead of asserted alongside it — a widened rule grows this domain
/// rather than leaving new values untested.
///
/// A second, independent copy of `tiler-ir`'s program encoder, so it earns a
/// second proof: this crate's copy inlines its own `PackedBitOrder` and
/// `PackedTailRule` tables with no shared tag function and no decode inverse, so
/// nothing about the other copy's bytes constrains these.
#[test]
fn the_artifact_storage_encoding_is_injective_over_its_constructible_domain() {
    const BIT_ORDERS: [PackedBitOrder; std::mem::variant_count::<PackedBitOrder>()] = [
        PackedBitOrder::LeastSignificantElementFirst,
        PackedBitOrder::MostSignificantElementFirst,
    ];
    const TAIL_RULES: [PackedTailRule; std::mem::variant_count::<PackedTailRule>()] =
        [PackedTailRule::Zero];

    let mut candidates = 0_usize;
    let mut encodings = vec![StorageEncoding::Unpacked];
    for element_bits in 0..=u8::MAX {
        for bit_order in BIT_ORDERS {
            for tail in TAIL_RULES {
                candidates += 1;
                if let Some(packed) = BitPackedEncoding::new(element_bits, bit_order, tail) {
                    encodings.push(StorageEncoding::BitPacked(packed));
                }
            }
        }
    }

    assert_eq!(
        candidates,
        256 * BIT_ORDERS.len() * TAIL_RULES.len(),
        "the candidate sweep did not cover the whole field product"
    );
    assert_eq!(
        encodings.len(),
        1 + 3 * BIT_ORDERS.len() * TAIL_RULES.len(),
        "the constructible domain changed size; restate the claim deliberately"
    );
    assert_eq!(encodings.len(), 7);

    let mut seen: HashMap<Vec<u8>, StorageEncoding> = HashMap::with_capacity(encodings.len());
    for encoding in encodings {
        let mut bytes = Vec::new();
        push_storage_encoding(&mut bytes, encoding);
        let expected = match encoding {
            StorageEncoding::Unpacked => 1,
            StorageEncoding::BitPacked(_) => 4,
        };
        assert_eq!(bytes.len(), expected, "{encoding:?} changed width");
        if let Some(previous) = seen.insert(bytes, encoding) {
            panic!("{encoding:?} and {previous:?} share one encoding");
        }
    }
    assert_eq!(seen.len(), 7);
}
