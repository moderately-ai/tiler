//! Compiler-owned program layers over the shared target-neutral kernel program.
//!
//! The stage DAG, the exact selected scheduled/KIR refinements, the checked
//! materialized values, views, allocations, lifetimes and handoffs, the typed
//! dependencies, the named outputs, and complete semantic coverage all live in
//! [`tiler_ir::program`] (ADR 0070), where they are constructed through the
//! ADR 0071 checked builder and carry a canonical identity folding the ADR 0072
//! layers. This module owns only the compiler-specific layers the shared IR
//! deliberately excludes for now: the host preflight expression graph, the
//! bounded entry ABI, the routing-commit contract, and the artifact
//! construction plan that binds them to a compilation request.

use std::error::Error;
use std::fmt;

use tiler_ir::kernel::KernelType;
use tiler_ir::program::{
    AllocationOwnership, AllocationSpec, KernelProgramBuildError, KernelProgramBuilder,
    KernelProgramDiagnostic, MaterializedOrigin, MaterializedValueRef, MaterializedValueSpec,
    MemorySpace, SemanticOccurrence, StageAccess, StageAccessMode, ValueRole,
    VerifiedKernelProgram,
};
use tiler_ir::semantic::{F32, SemanticIdentity, SemanticProgram};
use tiler_ir::shape::Shape;

use crate::physical::{
    NumericalRealization, RegionId, ResourceRequirements, VerifiedKernel, VerifiedScheduledRegion,
    lower_structured_kernel,
};
use crate::region::SemanticMemberId;
use crate::request::{LoweringProviderIdentity, VerifiedTargetRequest};

/// Element byte width of the bounded profile's single tensor element type.
const ELEMENT_BYTES: u64 = 4;
/// Byte alignment every bounded-profile value and allocation requires.
const ELEMENT_ALIGNMENT: u32 = 4;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct HostExprId(u8);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct StageId(u8);

impl HostExprId {
    pub(crate) const fn index(self) -> u8 {
        self.0
    }
}

impl StageId {
    pub(crate) const fn index(self) -> u8 {
        self.0
    }
}

/// Declaration-order position of one materialized value of the shared program.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct MaterializedValueId(pub(crate) u8);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct EntryBindingId(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostValueType {
    U64,
    Bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HostExprNode {
    U64(u64),
    Bool(bool),
    CheckedMultiply(HostExprId, HostExprId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HostExpr {
    pub(crate) id: HostExprId,
    pub(crate) value_type: HostValueType,
    pub(crate) node: HostExprNode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AbiAccess {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComponentRole {
    Input,
    Intermediate,
    Output,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EntryBinding {
    pub(crate) id: EntryBindingId,
    pub(crate) value: MaterializedValueId,
    pub(crate) role: ComponentRole,
    pub(crate) access: AbiAccess,
    pub(crate) alignment: u32,
    pub(crate) accessible_bytes: HostExprId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EntryContract {
    pub(crate) stage: StageId,
    pub(crate) bindings: Vec<EntryBinding>,
    pub(crate) launch_threads: HostExprId,
    pub(crate) threads_per_workgroup: HostExprId,
    pub(crate) requirements: ResourceRequirements,
    pub(crate) numerical: NumericalRealization,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RoutingState {
    Preflight,
    Committed,
    Executing,
    Published,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RoutingTransition {
    pub(crate) from: RoutingState,
    pub(crate) to: RoutingState,
    pub(crate) fallback_permitted: bool,
}

/// One target-bound executable program: a verified shared kernel program plus
/// the compiler-owned host, ABI, and routing layers that dispatch it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelProgram {
    target_profile_key: &'static str,
    host_expressions: Vec<HostExpr>,
    applicability_guard: HostExprId,
    core: VerifiedKernelProgram,
    entries: Vec<EntryContract>,
    routing: Vec<RoutingTransition>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactConstructionPlan {
    semantic_identity: SemanticIdentity,
    numerical_contract_key: &'static str,
    numerical_realizations: Vec<NumericalRealization>,
    target_profile_key: &'static str,
    entry_regions: Vec<RegionId>,
    routing_guard: HostExprId,
    lowering_providers: Vec<LoweringProviderIdentity>,
    request_subject: crate::request::VerifiedRequestSubject,
    verified_program: KernelProgram,
    verified_schedules: Vec<VerifiedScheduledRegion>,
    verified_kernels: Vec<VerifiedKernel>,
}

impl KernelProgram {
    /// Returns the verified target-neutral program this target binding wraps.
    ///
    /// The compiler reaches the shared program through its own private field;
    /// this accessor exists for the crate's tests until a reviewed public
    /// compiler facade exposes the compilation product.
    #[cfg(test)]
    pub(crate) const fn core(&self) -> &VerifiedKernelProgram {
        &self.core
    }

    pub(crate) fn stage_count(&self) -> usize {
        self.core.stages().len()
    }

    #[allow(
        dead_code,
        reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
    )]
    pub(crate) fn dependency_count(&self) -> usize {
        self.core.dependencies().len()
    }
}

impl ArtifactConstructionPlan {
    pub(crate) fn lowering_providers(&self) -> &[LoweringProviderIdentity] {
        &self.lowering_providers
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProgramError {
    HostExpression {
        rule: &'static str,
        expression: HostExprId,
    },
    Structure {
        rule: &'static str,
    },
    Storage {
        rule: &'static str,
    },
    Abi {
        rule: &'static str,
        stage: StageId,
    },
    Routing {
        rule: &'static str,
    },
    /// The shared kernel-program builder rejected a locally malformed insertion.
    CoreConstruction(KernelProgramBuildError),
    /// The shared whole-program verifier rejected the assembled program.
    CoreVerification(KernelProgramDiagnostic),
}

impl ProgramError {
    /// Returns the stable rule identifier a rejected program reports.
    pub(crate) fn rule(&self) -> &str {
        match self {
            Self::HostExpression { rule, .. }
            | Self::Structure { rule }
            | Self::Storage { rule }
            | Self::Abi { rule, .. }
            | Self::Routing { rule } => rule,
            Self::CoreConstruction(_) => "core-construction",
            Self::CoreVerification(diagnostic) => diagnostic.rule(),
        }
    }
}

impl fmt::Display for ProgramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HostExpression { rule, expression } => write!(
                formatter,
                "program.host-expression.{rule}: expression {} rejected",
                expression.0
            ),
            Self::Structure { rule } => write!(formatter, "program.structure.{rule}: rejected"),
            Self::Storage { rule } => write!(formatter, "program.storage.{rule}: rejected"),
            Self::Abi { rule, stage } => {
                write!(formatter, "program.abi.{rule}: stage {} rejected", stage.0)
            }
            Self::Routing { rule } => write!(formatter, "program.routing.{rule}: rejected"),
            Self::CoreConstruction(_) => {
                write!(formatter, "program.core.core-construction: rejected")
            }
            Self::CoreVerification(diagnostic) => {
                write!(formatter, "program.core.{}: rejected", diagnostic.rule())
            }
        }
    }
}

impl Error for ProgramError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HostExpression { .. }
            | Self::Structure { .. }
            | Self::Storage { .. }
            | Self::Abi { .. }
            | Self::Routing { .. } => None,
            Self::CoreConstruction(source) => Some(source),
            Self::CoreVerification(source) => Some(source),
        }
    }
}

impl From<KernelProgramBuildError> for ProgramError {
    fn from(value: KernelProgramBuildError) -> Self {
        Self::CoreConstruction(value)
    }
}

/// Builds the two-stage materialized serial-sum program for one request.
pub(crate) fn build_kernel_program(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<KernelProgram, ProgramError> {
    let [pointwise, reduction] = scheduled else {
        return Err(ProgramError::Structure {
            rule: "strategy-cardinality",
        });
    };
    let core = build_materialized_core(semantic, request, pointwise, reduction)?;
    let program = KernelProgram {
        target_profile_key: request.target_profile().key,
        host_expressions: host_expressions(request)?,
        applicability_guard: HostExprId(7),
        core,
        entries: entry_contracts(scheduled, HostExprId(2), HostExprId(4)),
        routing: routing_policy(),
    };
    verify_kernel_program_layers(&program, request, scheduled)?;
    Ok(program)
}

/// Builds the single-stage fused serial-sum program for one request.
pub(crate) fn build_fused_kernel_program(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &VerifiedScheduledRegion,
) -> Result<KernelProgram, ProgramError> {
    let core = build_fused_core(semantic, request, scheduled)?;
    let program = KernelProgram {
        target_profile_key: request.target_profile().key,
        host_expressions: host_expressions(request)?,
        applicability_guard: HostExprId(7),
        core,
        entries: vec![entry(
            0,
            vec![
                binding(0, 0, ComponentRole::Input, AbiAccess::Read, HostExprId(2)),
                binding(1, 1, ComponentRole::Output, AbiAccess::Write, HostExprId(4)),
            ],
            HostExprId(6),
            scheduled.requirements(),
            scheduled.region().index.numerical,
        )],
        routing: routing_policy(),
    };
    verify_kernel_program_layers(&program, request, std::slice::from_ref(scheduled))?;
    Ok(program)
}

/// Assembles the shared verified program of the materialized strategy.
///
/// Every structural obligation — complete disjoint coverage of the semantic
/// graph, a unique writer per materialized value, the data dependency behind
/// the cross-stage read, the aliasing contract, and named-output coverage — is
/// proven by [`KernelProgramBuilder::build`], not re-implemented here.
fn build_materialized_core(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    pointwise: &VerifiedScheduledRegion,
    reduction: &VerifiedScheduledRegion,
) -> Result<VerifiedKernelProgram, ProgramError> {
    let subject = request.serial_sum();
    let input_bytes = byte_count(subject.input_elements)?;
    let output_bytes = byte_count(subject.output_elements)?;
    let mut builder = open_core_builder(semantic, request)?;
    let external = builder.push_allocation(storage(input_bytes, AllocationOwnership::External))?;
    let temporary_storage =
        builder.push_allocation(storage(input_bytes, AllocationOwnership::Program))?;
    let output_storage =
        builder.push_allocation(storage(output_bytes, AllocationOwnership::Program))?;
    let input = builder.push_value(
        program_input(subject.input_key.clone(), subject.input_shape.clone()),
        external,
    )?;
    let temporary = builder.push_value(
        internal(ValueRole::Temporary, subject.input_shape.clone()),
        temporary_storage,
    )?;
    let output = builder.push_value(
        internal(ValueRole::Output, subject.output_shape.clone()),
        output_storage,
    )?;
    let input_view = builder.push_whole_view(input)?;
    let temporary_view = builder.push_whole_view(temporary)?;
    let output_view = builder.push_whole_view(output)?;
    let map_stage = builder.push_stage(
        &lower(pointwise)?,
        &covered(subject.members.pointwise()),
        &[read(input_view), write(temporary_view)],
    )?;
    let reduce_stage = builder.push_stage(
        &lower(reduction)?,
        &covered(subject.members.reduction()),
        &[read(temporary_view), write(output_view)],
    )?;
    builder.push_data_dependency(map_stage, reduce_stage, temporary)?;
    builder.push_output(subject.output_key.clone(), output)?;
    finish_core(builder)
}

/// Assembles the shared verified program of the fused strategy.
fn build_fused_core(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &VerifiedScheduledRegion,
) -> Result<VerifiedKernelProgram, ProgramError> {
    let subject = request.serial_sum();
    let input_bytes = byte_count(subject.input_elements)?;
    let output_bytes = byte_count(subject.output_elements)?;
    let mut builder = open_core_builder(semantic, request)?;
    let external = builder.push_allocation(storage(input_bytes, AllocationOwnership::External))?;
    let output_storage =
        builder.push_allocation(storage(output_bytes, AllocationOwnership::Program))?;
    let input = builder.push_value(
        program_input(subject.input_key.clone(), subject.input_shape.clone()),
        external,
    )?;
    let output = builder.push_value(
        internal(ValueRole::Output, subject.output_shape.clone()),
        output_storage,
    )?;
    let input_view = builder.push_whole_view(input)?;
    let output_view = builder.push_whole_view(output)?;
    builder.push_stage(
        &lower(scheduled)?,
        &covered(&subject.members.all()),
        &[read(input_view), write(output_view)],
    )?;
    builder.push_output(subject.output_key.clone(), output)?;
    finish_core(builder)
}

/// Opens a shared program builder bound to the request's exact semantic program.
fn open_core_builder(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
) -> Result<KernelProgramBuilder, ProgramError> {
    if semantic.semantic_identity() != request.semantic_identity() {
        return Err(ProgramError::Structure {
            rule: "semantic-request-binding",
        });
    }
    Ok(KernelProgramBuilder::new(semantic)?)
}

fn finish_core(builder: KernelProgramBuilder) -> Result<VerifiedKernelProgram, ProgramError> {
    builder.build().map_err(|error| {
        error.diagnostics().first().copied().map_or(
            ProgramError::Structure {
                rule: "core-verification",
            },
            ProgramError::CoreVerification,
        )
    })
}

/// Lowers one verified scheduled region to the kernel its stage dispatches.
fn lower(scheduled: &VerifiedScheduledRegion) -> Result<VerifiedKernel, ProgramError> {
    lower_structured_kernel(scheduled).map_err(|_| ProgramError::Structure {
        rule: "schedule-verification",
    })
}

fn covered(members: &[SemanticMemberId]) -> Vec<SemanticOccurrence> {
    members
        .iter()
        .map(|member| SemanticOccurrence::new(member.0))
        .collect()
}

fn storage(capacity_bytes: u64, ownership: AllocationOwnership) -> AllocationSpec {
    AllocationSpec {
        capacity_bytes,
        alignment: ELEMENT_ALIGNMENT,
        memory_space: MemorySpace::Device,
        ownership,
    }
}

fn program_input(key: tiler_ir::semantic::InputKey, shape: Shape) -> MaterializedValueSpec {
    MaterializedValueSpec {
        origin: MaterializedOrigin::ProgramInput { key },
        role: ValueRole::Input,
        shape,
        element_type: KernelType::F32,
        alignment: ELEMENT_ALIGNMENT,
        memory_space: MemorySpace::Device,
    }
}

fn internal(role: ValueRole, shape: Shape) -> MaterializedValueSpec {
    MaterializedValueSpec {
        origin: MaterializedOrigin::Internal,
        role,
        shape,
        element_type: KernelType::F32,
        alignment: ELEMENT_ALIGNMENT,
        memory_space: MemorySpace::Device,
    }
}

const fn read(view: tiler_ir::program::ViewId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Read,
    }
}

const fn write(view: tiler_ir::program::ViewId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Write,
    }
}

fn byte_count(elements: u64) -> Result<u64, ProgramError> {
    elements
        .checked_mul(ELEMENT_BYTES)
        .ok_or(ProgramError::Storage {
            rule: "required-byte-overflow",
        })
}

fn entry_contracts(
    scheduled: &[VerifiedScheduledRegion],
    input_bytes: HostExprId,
    output_bytes: HostExprId,
) -> Vec<EntryContract> {
    vec![
        entry(
            0,
            vec![
                binding(0, 0, ComponentRole::Input, AbiAccess::Read, input_bytes),
                binding(
                    1,
                    1,
                    ComponentRole::Intermediate,
                    AbiAccess::Write,
                    input_bytes,
                ),
            ],
            HostExprId(5),
            scheduled[0].requirements(),
            scheduled[0].region().index.numerical,
        ),
        entry(
            1,
            vec![
                binding(
                    0,
                    1,
                    ComponentRole::Intermediate,
                    AbiAccess::Read,
                    input_bytes,
                ),
                binding(1, 2, ComponentRole::Output, AbiAccess::Write, output_bytes),
            ],
            HostExprId(6),
            scheduled[1].requirements(),
            scheduled[1].region().index.numerical,
        ),
    ]
}

fn routing_policy() -> Vec<RoutingTransition> {
    vec![
        RoutingTransition {
            from: RoutingState::Preflight,
            to: RoutingState::Committed,
            fallback_permitted: true,
        },
        RoutingTransition {
            from: RoutingState::Committed,
            to: RoutingState::Executing,
            fallback_permitted: false,
        },
        RoutingTransition {
            from: RoutingState::Executing,
            to: RoutingState::Published,
            fallback_permitted: false,
        },
    ]
}

/// Verifies the compiler-owned layers of one program against its shared core.
///
/// The shared core is already verified by construction, so this proves only
/// what the core deliberately does not model: the canonical host preflight
/// graph and its budgets, the request/target binding, the entry ABI's agreement
/// with the stages and values the core retains, and the routing contract.
pub(crate) fn verify_kernel_program_layers(
    program: &KernelProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<(), ProgramError> {
    if scheduled.is_empty()
        || program.core.stages().len() != scheduled.len()
        || program.entries.len() != scheduled.len()
    {
        return Err(ProgramError::Structure {
            rule: "cardinality",
        });
    }
    if scheduled
        .iter()
        .any(|region| !region.matches_request(request))
    {
        return Err(ProgramError::Structure {
            rule: "request-subject",
        });
    }
    if program.target_profile_key != request.target_profile().key
        || scheduled
            .iter()
            .any(|region| region.target_profile_key() != program.target_profile_key)
    {
        return Err(ProgramError::Structure {
            rule: "target-profile",
        });
    }
    let values = verify_host_contract(program, request)?;
    for (index, entry) in program.entries.iter().enumerate() {
        verify_entry(program, entry, index, &scheduled[index], &values)?;
    }
    if program.routing != routing_policy() {
        return Err(ProgramError::Routing {
            rule: "fallback-after-commit",
        });
    }
    Ok(())
}

fn verify_host_contract(
    program: &KernelProgram,
    request: &VerifiedTargetRequest,
) -> Result<Vec<HostValue>, ProgramError> {
    let subject = request.serial_sum();
    let expected_expressions =
        canonical_host_expressions(subject.input_elements, subject.output_elements);
    if program.host_expressions != expected_expressions {
        return Err(ProgramError::HostExpression {
            rule: "canonical-graph",
            expression: HostExprId(0),
        });
    }
    if program.host_expressions.len()
        > usize::try_from(request.budgets().host_expression_nodes).map_err(|_| {
            ProgramError::Structure {
                rule: "host-expression-budget",
            }
        })?
    {
        return Err(ProgramError::Structure {
            rule: "host-expression-budget",
        });
    }
    if program.core.values().len()
        > usize::try_from(request.budgets().buffers).map_err(|_| ProgramError::Storage {
            rule: "buffer-budget",
        })?
    {
        return Err(ProgramError::Storage {
            rule: "buffer-budget",
        });
    }
    let values = evaluate_expressions(&program.host_expressions)?;
    if values.get(usize::from(program.applicability_guard.0)) != Some(&HostValue::Bool(true)) {
        return Err(ProgramError::Structure {
            rule: "applicability-guard",
        });
    }
    Ok(values)
}

/// Proves one entry contract realizes the exact stage the core retains.
fn verify_entry(
    program: &KernelProgram,
    entry: &EntryContract,
    position: usize,
    scheduled: &VerifiedScheduledRegion,
    values: &[HostValue],
) -> Result<(), ProgramError> {
    let index = u8::try_from(position).map_err(|_| ProgramError::Structure {
        rule: "stage-id-overflow",
    })?;
    let stage = program
        .core
        .stages()
        .nth(position)
        .ok_or(ProgramError::Abi {
            rule: "entry-stage",
            stage: entry.stage,
        })?;
    if entry.stage != StageId(index)
        || entry.requirements != scheduled.requirements()
        || entry.numerical != scheduled.region().index.numerical
        || entry.threads_per_workgroup != HostExprId(8)
        || entry.launch_threads
            != if position == 0 && program.core.stages().len() == 2 {
                HostExprId(5)
            } else {
                HostExprId(6)
            }
    {
        return Err(ProgramError::Abi {
            rule: "entry-contract",
            stage: entry.stage,
        });
    }
    if values.get(usize::from(entry.launch_threads.0))
        != Some(&HostValue::U64(scheduled.region().schedule.work_items))
        || values.get(usize::from(entry.threads_per_workgroup.0))
            != Some(&HostValue::U64(u64::from(
                scheduled.region().schedule.threads_per_workgroup,
            )))
    {
        return Err(ProgramError::Abi {
            rule: "launch-expression",
            stage: entry.stage,
        });
    }
    if entry.bindings.len() != stage.accesses().len() {
        return Err(ProgramError::Abi {
            rule: "binding-cardinality",
            stage: entry.stage,
        });
    }
    for (position, (binding, access)) in entry.bindings.iter().zip(stage.accesses()).enumerate() {
        let bound = program
            .core
            .values()
            .nth(usize::from(binding.value.0))
            .ok_or(ProgramError::Abi {
                rule: "binding-value",
                stage: entry.stage,
            })?;
        let expected_access = match access.mode() {
            StageAccessMode::Read => AbiAccess::Read,
            StageAccessMode::Write => AbiAccess::Write,
        };
        let expected_bytes = HostValue::U64(bound.required_bytes());
        if binding.id != EntryBindingId(u8::try_from(position).expect("bounded binding count"))
            || bound != access.view().value()
            || binding.access != expected_access
            || binding.alignment != bound.alignment()
            || binding.role != component_role(bound)
            || values.get(usize::from(binding.accessible_bytes.0)) != Some(&expected_bytes)
        {
            return Err(ProgramError::Abi {
                rule: "binding",
                stage: entry.stage,
            });
        }
    }
    Ok(())
}

fn component_role(value: MaterializedValueRef<'_>) -> ComponentRole {
    match value.role() {
        ValueRole::Input => ComponentRole::Input,
        ValueRole::Temporary => ComponentRole::Intermediate,
        ValueRole::Output => ComponentRole::Output,
    }
}

pub(crate) fn build_artifact_plan(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
    kernels: &[VerifiedKernel],
    program: &KernelProgram,
    providers: Vec<LoweringProviderIdentity>,
) -> Result<ArtifactConstructionPlan, ProgramError> {
    verify_artifact_refinements(semantic, request, scheduled, kernels, program)?;
    // Lowering provenance is re-derived from the request's own installed
    // registry rather than trusted from the caller, so a plan cannot claim a
    // provider the registry never resolved for this program.
    let expected_providers =
        crate::lowering::resolve_capabilities(semantic, request).map_err(|_| {
            ProgramError::Structure {
                rule: "artifact-provider-resolution",
            }
        })?;
    if providers.is_empty() || providers != expected_providers {
        return Err(ProgramError::Structure {
            rule: "artifact-provider-coverage",
        });
    }
    Ok(ArtifactConstructionPlan {
        semantic_identity: request.semantic_identity().clone(),
        numerical_contract_key: request.numerical_contract().key,
        numerical_realizations: program
            .entries
            .iter()
            .map(|entry| entry.numerical)
            .collect(),
        target_profile_key: program.target_profile_key,
        entry_regions: program
            .core
            .stages()
            .map(|stage| stage.kernel().scheduled_region())
            .collect(),
        routing_guard: program.applicability_guard,
        lowering_providers: providers,
        request_subject: request.subject(),
        verified_program: program.clone(),
        verified_schedules: scheduled.to_vec(),
        verified_kernels: kernels.to_vec(),
    })
}

/// Proves the artifact's inputs are the exact refinements of one request.
fn verify_artifact_refinements(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
    kernels: &[VerifiedKernel],
    program: &KernelProgram,
) -> Result<(), ProgramError> {
    if semantic.semantic_identity() != request.semantic_identity() {
        return Err(ProgramError::Structure {
            rule: "semantic-request-binding",
        });
    }
    if scheduled.is_empty()
        || scheduled
            .iter()
            .any(|region| !region.matches_request(request))
        || kernels.len() != scheduled.len()
    {
        return Err(ProgramError::Structure {
            rule: "artifact-refinement-cardinality",
        });
    }
    for (region, kernel) in scheduled.iter().zip(kernels) {
        let expected = lower(region).map_err(|_| ProgramError::Structure {
            rule: "artifact-schedule-refinement",
        })?;
        if kernel != &expected {
            return Err(ProgramError::Structure {
                rule: "artifact-kernel-refinement",
            });
        }
    }
    let expected_program = match scheduled {
        [single] => build_fused_kernel_program(semantic, request, single)?,
        [_, _] => build_kernel_program(semantic, request, scheduled)?,
        _ => {
            return Err(ProgramError::Structure {
                rule: "artifact-strategy-cardinality",
            });
        }
    };
    if program != &expected_program {
        return Err(ProgramError::Structure {
            rule: "artifact-program-refinement",
        });
    }
    assert_kernels_match_program(request, scheduled, program, kernels)?;
    let semantic_output = semantic.outputs().next().ok_or(ProgramError::Structure {
        rule: "semantic-output-coverage",
    })?;
    let named = program
        .core
        .outputs()
        .next()
        .ok_or(ProgramError::Structure {
            rule: "semantic-output-coverage",
        })?;
    if semantic.output_count() != 1
        || program.core.outputs().len() != 1
        || named.key() != semantic_output.key()
    {
        return Err(ProgramError::Structure {
            rule: "semantic-output-coverage",
        });
    }
    if program.target_profile_key != request.target_profile().key
        || program
            .entries
            .iter()
            .any(|entry| entry.numerical != request.numerical_contract().realization())
    {
        return Err(ProgramError::Structure {
            rule: "artifact-numerical-realization",
        });
    }
    Ok(())
}

pub(crate) fn verify_artifact_plan(
    plan: &ArtifactConstructionPlan,
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
    kernels: &[VerifiedKernel],
    program: &KernelProgram,
    providers: Vec<LoweringProviderIdentity>,
) -> Result<(), ProgramError> {
    let expected = build_artifact_plan(semantic, request, scheduled, kernels, program, providers)?;
    if plan != &expected {
        return Err(ProgramError::Structure {
            rule: "artifact-receipt",
        });
    }
    Ok(())
}

fn host_expressions(request: &VerifiedTargetRequest) -> Result<Vec<HostExpr>, ProgramError> {
    let expressions = canonical_host_expressions(
        request.serial_sum().input_elements,
        request.serial_sum().output_elements,
    );
    let actual = expressions.len();
    if actual
        > usize::try_from(request.budgets().host_expression_nodes).map_err(|_| {
            ProgramError::Structure {
                rule: "host-expression-budget",
            }
        })?
    {
        return Err(ProgramError::Structure {
            rule: "host-expression-budget",
        });
    }
    Ok(expressions)
}

fn canonical_host_expressions(input_elements: u64, output_elements: u64) -> Vec<HostExpr> {
    vec![
        expression(0, HostValueType::U64, HostExprNode::U64(ELEMENT_BYTES)),
        expression(1, HostValueType::U64, HostExprNode::U64(input_elements)),
        expression(
            2,
            HostValueType::U64,
            HostExprNode::CheckedMultiply(HostExprId(0), HostExprId(1)),
        ),
        expression(3, HostValueType::U64, HostExprNode::U64(output_elements)),
        expression(
            4,
            HostValueType::U64,
            HostExprNode::CheckedMultiply(HostExprId(0), HostExprId(3)),
        ),
        expression(5, HostValueType::U64, HostExprNode::U64(input_elements)),
        expression(6, HostValueType::U64, HostExprNode::U64(output_elements)),
        expression(7, HostValueType::Bool, HostExprNode::Bool(true)),
        expression(8, HostValueType::U64, HostExprNode::U64(1)),
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HostValue {
    U64(u64),
    Bool(bool),
}

fn evaluate_expressions(expressions: &[HostExpr]) -> Result<Vec<HostValue>, ProgramError> {
    let mut values = Vec::with_capacity(expressions.len());
    for (position, expression) in expressions.iter().enumerate() {
        if usize::from(expression.id.0) != position {
            return Err(ProgramError::HostExpression {
                rule: "canonical-id",
                expression: expression.id,
            });
        }
        let value = match expression.node {
            HostExprNode::U64(value) if expression.value_type == HostValueType::U64 => {
                HostValue::U64(value)
            }
            HostExprNode::Bool(value) if expression.value_type == HostValueType::Bool => {
                HostValue::Bool(value)
            }
            HostExprNode::CheckedMultiply(left, right)
                if expression.value_type == HostValueType::U64 =>
            {
                let Some(HostValue::U64(left)) = values.get(usize::from(left.0)) else {
                    return host_error("operand", expression.id);
                };
                let Some(HostValue::U64(right)) = values.get(usize::from(right.0)) else {
                    return host_error("operand", expression.id);
                };
                HostValue::U64(
                    left.checked_mul(*right)
                        .ok_or(ProgramError::HostExpression {
                            rule: "overflow",
                            expression: expression.id,
                        })?,
                )
            }
            _ => return host_error("type", expression.id),
        };
        values.push(value);
    }
    Ok(values)
}

fn binding(
    id: u8,
    value: u8,
    role: ComponentRole,
    access: AbiAccess,
    bytes: HostExprId,
) -> EntryBinding {
    EntryBinding {
        id: EntryBindingId(id),
        value: MaterializedValueId(value),
        role,
        access,
        alignment: ELEMENT_ALIGNMENT,
        accessible_bytes: bytes,
    }
}

fn entry(
    stage: u8,
    bindings: Vec<EntryBinding>,
    launch_threads: HostExprId,
    requirements: ResourceRequirements,
    numerical: NumericalRealization,
) -> EntryContract {
    EntryContract {
        stage: StageId(stage),
        bindings,
        launch_threads,
        threads_per_workgroup: HostExprId(8),
        requirements,
        numerical,
    }
}

fn expression(id: u8, value_type: HostValueType, node: HostExprNode) -> HostExpr {
    HostExpr {
        id: HostExprId(id),
        value_type,
        node,
    }
}

fn host_error<T>(rule: &'static str, expression: HostExprId) -> Result<T, ProgramError> {
    Err(ProgramError::HostExpression { rule, expression })
}

/// Proves the separately retained kernels are exactly the ones the program binds.
///
/// The shared program already holds each stage's verified kernel, so this
/// checks the compilation product's parallel kernel list against that binding
/// and against the schedules it claims to refine.
pub(crate) fn assert_kernels_match_program(
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
    program: &KernelProgram,
    kernels: &[VerifiedKernel],
) -> Result<(), ProgramError> {
    if kernels.len() != scheduled.len()
        || kernels.len() != program.core.stages().len()
        || kernels.len() != program.entries.len()
        || scheduled
            .iter()
            .any(|region| !region.matches_request(request))
    {
        return Err(ProgramError::Structure {
            rule: "kernel-entry-cardinality",
        });
    }
    for ((region, kernel), stage) in scheduled.iter().zip(kernels).zip(program.core.stages()) {
        if lower(region)? != *kernel || stage.kernel() != kernel {
            return Err(ProgramError::Structure {
                rule: "kernel-schedule-refinement",
            });
        }
    }
    Ok(())
}

pub(crate) fn verify_semantic_output_type(program: &SemanticProgram) -> Result<(), ProgramError> {
    if program.output_count() == 0
        || program.outputs().any(|output| {
            program
                .value(output.value())
                .map_or(true, |value| value.resolved_type() != &F32::resolved_type())
        })
    {
        return Err(ProgramError::Structure {
            rule: "semantic-output-type",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
