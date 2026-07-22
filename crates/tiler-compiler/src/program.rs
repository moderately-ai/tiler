use std::error::Error;
use std::fmt;

use tiler_ir::semantic::{F32, SemanticIdentity, SemanticProgram};
use tiler_ir::shape::Shape;

use crate::physical::{
    NumericalRealization, RegionId, ResourceRequirements, TensorRole, VerifiedScheduledRegion,
    VerifiedStructuredKernel, lower_structured_kernel,
};
use crate::request::{LoweringProviderIdentity, VerifiedTargetRequest};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct HostExprId(u8);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct StageId(u8);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct MaterializedValueId(pub(crate) u8);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct AllocationId(u8);

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
pub(crate) enum MemorySpace {
    Device,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ValueRole {
    Input,
    Temporary,
    Output,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AllocationOwnership {
    External,
    Program,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Allocation {
    pub(crate) id: AllocationId,
    pub(crate) capacity_bytes: HostExprId,
    pub(crate) alignment: u32,
    pub(crate) memory_space: MemorySpace,
    pub(crate) ownership: AllocationOwnership,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MaterializedValue {
    pub(crate) id: MaterializedValueId,
    pub(crate) tensor: TensorRole,
    pub(crate) role: ValueRole,
    pub(crate) shape: Shape,
    pub(crate) required_bytes: HostExprId,
    pub(crate) alignment: u32,
    pub(crate) memory_space: MemorySpace,
    pub(crate) definition: Option<StageId>,
    pub(crate) allocation: AllocationId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StageAccess {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StageValueAccess {
    pub(crate) value: MaterializedValueId,
    pub(crate) access: StageAccess,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramStage {
    pub(crate) id: StageId,
    pub(crate) scheduled_region: RegionId,
    pub(crate) values: Vec<StageValueAccess>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DependencyReason {
    Data(MaterializedValueId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Dependency {
    pub(crate) predecessor: StageId,
    pub(crate) successor: StageId,
    pub(crate) reason: DependencyReason,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramOutput {
    pub(crate) key: String,
    pub(crate) value: MaterializedValueId,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BufferPlan {
    pub(crate) values: Vec<MaterializedValue>,
    pub(crate) allocations: Vec<Allocation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelProgram {
    target_profile_key: &'static str,
    host_expressions: Vec<HostExpr>,
    applicability_guard: HostExprId,
    stages: Vec<ProgramStage>,
    dependencies: Vec<Dependency>,
    buffer_plan: BufferPlan,
    entries: Vec<EntryContract>,
    outputs: Vec<ProgramOutput>,
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
    verified_kernels: Vec<VerifiedStructuredKernel>,
}

impl KernelProgram {
    #[cfg(test)]
    pub(crate) fn stages(&self) -> &[ProgramStage] {
        &self.stages
    }

    #[cfg(test)]
    pub(crate) const fn buffer_plan(&self) -> &BufferPlan {
        &self.buffer_plan
    }

    #[cfg(test)]
    pub(crate) fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
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
    Dependency {
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
            Self::Dependency { rule } => write!(formatter, "program.dependency.{rule}: rejected"),
            Self::Storage { rule } => write!(formatter, "program.storage.{rule}: rejected"),
            Self::Abi { rule, stage } => {
                write!(formatter, "program.abi.{rule}: stage {} rejected", stage.0)
            }
            Self::Routing { rule } => write!(formatter, "program.routing.{rule}: rejected"),
        }
    }
}

impl Error for ProgramError {}

pub(crate) fn build_kernel_program(
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<KernelProgram, ProgramError> {
    if scheduled.len() != 2 {
        return Err(ProgramError::Structure {
            rule: "strategy-cardinality",
        });
    }
    let expressions = host_expressions(request)?;
    let input_bytes = HostExprId(2);
    let output_bytes = HostExprId(4);
    let program = KernelProgram {
        target_profile_key: request.target_profile().key,
        host_expressions: expressions,
        applicability_guard: HostExprId(7),
        stages: program_stages(scheduled),
        dependencies: vec![Dependency {
            predecessor: StageId(0),
            successor: StageId(1),
            reason: DependencyReason::Data(MaterializedValueId(1)),
        }],
        buffer_plan: BufferPlan {
            values: materialized_values(request, input_bytes, output_bytes),
            allocations: program_allocations(input_bytes, output_bytes),
        },
        entries: entry_contracts(scheduled, input_bytes, output_bytes),
        outputs: vec![ProgramOutput {
            key: request.serial_sum().output_key.as_str().to_owned(),
            value: MaterializedValueId(2),
        }],
        routing: routing_policy(),
    };
    verify_materialized_serial_sum_program(program, request, scheduled)
}

pub(crate) fn build_fused_kernel_program(
    request: &VerifiedTargetRequest,
    scheduled: &VerifiedScheduledRegion,
) -> Result<KernelProgram, ProgramError> {
    let expressions = host_expressions(request)?;
    let input_bytes = HostExprId(2);
    let output_bytes = HostExprId(4);
    let scheduled = std::slice::from_ref(scheduled);
    let program = KernelProgram {
        target_profile_key: request.target_profile().key,
        host_expressions: expressions,
        applicability_guard: HostExprId(7),
        stages: vec![ProgramStage {
            id: StageId(0),
            scheduled_region: scheduled[0].region().index.id,
            values: vec![
                StageValueAccess {
                    value: MaterializedValueId(0),
                    access: StageAccess::Read,
                },
                StageValueAccess {
                    value: MaterializedValueId(1),
                    access: StageAccess::Write,
                },
            ],
        }],
        dependencies: Vec::new(),
        buffer_plan: BufferPlan {
            values: vec![
                materialized(
                    0,
                    TensorRole::Input,
                    ValueRole::Input,
                    request.serial_sum().input_shape.clone(),
                    input_bytes,
                    None,
                    0,
                ),
                materialized(
                    1,
                    TensorRole::Output,
                    ValueRole::Output,
                    request.serial_sum().output_shape.clone(),
                    output_bytes,
                    Some(StageId(0)),
                    1,
                ),
            ],
            allocations: vec![
                allocation(0, input_bytes, AllocationOwnership::External),
                allocation(1, output_bytes, AllocationOwnership::Program),
            ],
        },
        entries: vec![entry(
            0,
            vec![
                binding(0, 0, ComponentRole::Input, AbiAccess::Read, input_bytes),
                binding(1, 1, ComponentRole::Output, AbiAccess::Write, output_bytes),
            ],
            HostExprId(6),
            scheduled[0].requirements(),
            scheduled[0].region().index.numerical,
        )],
        outputs: vec![ProgramOutput {
            key: request.serial_sum().output_key.as_str().to_owned(),
            value: MaterializedValueId(1),
        }],
        routing: routing_policy(),
    };
    verify_fused_serial_sum_program(program, request, scheduled)
}

fn program_stages(scheduled: &[VerifiedScheduledRegion]) -> Vec<ProgramStage> {
    vec![
        ProgramStage {
            id: StageId(0),
            scheduled_region: scheduled[0].region().index.id,
            values: vec![
                StageValueAccess {
                    value: MaterializedValueId(0),
                    access: StageAccess::Read,
                },
                StageValueAccess {
                    value: MaterializedValueId(1),
                    access: StageAccess::Write,
                },
            ],
        },
        ProgramStage {
            id: StageId(1),
            scheduled_region: scheduled[1].region().index.id,
            values: vec![
                StageValueAccess {
                    value: MaterializedValueId(1),
                    access: StageAccess::Read,
                },
                StageValueAccess {
                    value: MaterializedValueId(2),
                    access: StageAccess::Write,
                },
            ],
        },
    ]
}

fn materialized_values(
    request: &VerifiedTargetRequest,
    input_bytes: HostExprId,
    output_bytes: HostExprId,
) -> Vec<MaterializedValue> {
    vec![
        materialized(
            0,
            TensorRole::Input,
            ValueRole::Input,
            request.serial_sum().input_shape.clone(),
            input_bytes,
            None,
            0,
        ),
        materialized(
            1,
            TensorRole::Intermediate,
            ValueRole::Temporary,
            request.serial_sum().input_shape.clone(),
            input_bytes,
            Some(StageId(0)),
            1,
        ),
        materialized(
            2,
            TensorRole::Output,
            ValueRole::Output,
            request.serial_sum().output_shape.clone(),
            output_bytes,
            Some(StageId(1)),
            2,
        ),
    ]
}

fn program_allocations(input_bytes: HostExprId, output_bytes: HostExprId) -> Vec<Allocation> {
    vec![
        allocation(0, input_bytes, AllocationOwnership::External),
        allocation(1, input_bytes, AllocationOwnership::Program),
        allocation(2, output_bytes, AllocationOwnership::Program),
    ]
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

pub(crate) fn verify_materialized_serial_sum_program(
    program: KernelProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<KernelProgram, ProgramError> {
    if scheduled.len() != 2
        || program.stages.len() != 2
        || program.dependencies.len() != 1
        || program.entries.len() != 2
        || program.outputs.len() != 1
        || program.buffer_plan.values.len() != 3
        || program.buffer_plan.allocations.len() != 3
    {
        return Err(ProgramError::Structure {
            rule: "strategy-cardinality",
        });
    }
    let values = verify_program_structure(&program, request, scheduled)?;
    if program.stages != program_stages(scheduled) {
        return Err(ProgramError::Structure { rule: "stages" });
    }
    if program.dependencies
        != [Dependency {
            predecessor: StageId(0),
            successor: StageId(1),
            reason: DependencyReason::Data(MaterializedValueId(1)),
        }]
        || program.stages[0].values[1]
            != (StageValueAccess {
                value: MaterializedValueId(1),
                access: StageAccess::Write,
            })
        || program.stages[1].values[0]
            != (StageValueAccess {
                value: MaterializedValueId(1),
                access: StageAccess::Read,
            })
    {
        return Err(ProgramError::Dependency {
            rule: "initialized-cross-stage-value",
        });
    }
    verify_storage(&program, &values, scheduled)?;
    verify_entry(
        &program,
        &program.entries[0],
        StageId(0),
        &scheduled[0],
        &values,
    )?;
    verify_entry(
        &program,
        &program.entries[1],
        StageId(1),
        &scheduled[1],
        &values,
    )?;
    if program.outputs.len() != 1
        || program.outputs[0].value != MaterializedValueId(2)
        || program.outputs[0].key != request.serial_sum().output_key.as_str()
    {
        return Err(ProgramError::Structure {
            rule: "semantic-output-coverage",
        });
    }
    Ok(program)
}

#[cfg(test)]
fn verify_kernel_program(
    program: KernelProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<KernelProgram, ProgramError> {
    verify_materialized_serial_sum_program(program, request, scheduled)
}

pub(crate) fn verify_fused_serial_sum_program(
    program: KernelProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<KernelProgram, ProgramError> {
    if scheduled.len() != 1
        || program.stages.len() != 1
        || !program.dependencies.is_empty()
        || program.entries.len() != 1
        || program.outputs.len() != 1
        || program.buffer_plan.values.len() != 2
        || program.buffer_plan.allocations.len() != 2
    {
        return Err(ProgramError::Structure {
            rule: "fused-strategy-cardinality",
        });
    }
    let values = verify_program_structure(&program, request, scheduled)?;
    let expected_stage = ProgramStage {
        id: StageId(0),
        scheduled_region: scheduled[0].region().index.id,
        values: vec![
            StageValueAccess {
                value: MaterializedValueId(0),
                access: StageAccess::Read,
            },
            StageValueAccess {
                value: MaterializedValueId(1),
                access: StageAccess::Write,
            },
        ],
    };
    if program.stages != [expected_stage] {
        return Err(ProgramError::Structure {
            rule: "fused-stage",
        });
    }
    verify_fused_storage(&program, &values, scheduled)?;
    verify_entry(
        &program,
        &program.entries[0],
        StageId(0),
        &scheduled[0],
        &values,
    )?;
    if program.outputs[0].value != MaterializedValueId(1)
        || program.outputs[0].key != request.serial_sum().output_key.as_str()
    {
        return Err(ProgramError::Structure {
            rule: "semantic-output-coverage",
        });
    }
    Ok(program)
}

fn verify_program_structure(
    program: &KernelProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<Vec<HostValue>, ProgramError> {
    if program.stages.is_empty()
        || program.stages.len() != scheduled.len()
        || program.entries.len() != program.stages.len()
        || program.outputs.is_empty()
        || program.buffer_plan.values.is_empty()
        || program.buffer_plan.allocations.is_empty()
    {
        return Err(ProgramError::Structure {
            rule: "cardinality",
        });
    }
    let Some(_first_schedule) = scheduled.first() else {
        return Err(ProgramError::Structure {
            rule: "cardinality",
        });
    };
    if scheduled
        .iter()
        .any(|region| !region.matches_request(request))
    {
        return Err(ProgramError::Structure {
            rule: "request-subject",
        });
    }
    for region in scheduled {
        crate::physical::lower_structured_kernel(region).map_err(|_| ProgramError::Structure {
            rule: "schedule-verification",
        })?;
    }
    let values = verify_host_contract(program, request, scheduled)?;
    if scheduled
        .iter()
        .any(|region| region.target_profile_key() != program.target_profile_key)
    {
        return Err(ProgramError::Structure {
            rule: "target-profile",
        });
    }
    for (index, stage) in program.stages.iter().enumerate() {
        if stage.id
            != StageId(u8::try_from(index).map_err(|_| ProgramError::Structure {
                rule: "stage-id-overflow",
            })?)
            || stage.scheduled_region != scheduled[index].region().index.id
            || stage.values.is_empty()
        {
            return Err(ProgramError::Structure { rule: "stage-id" });
        }
    }
    for (index, value) in program.buffer_plan.values.iter().enumerate() {
        if value.id
            != MaterializedValueId(u8::try_from(index).map_err(|_| ProgramError::Storage {
                rule: "value-id-overflow",
            })?)
            || usize::from(value.allocation.0) >= program.buffer_plan.allocations.len()
        {
            return Err(ProgramError::Storage { rule: "value-id" });
        }
    }
    for (index, allocation) in program.buffer_plan.allocations.iter().enumerate() {
        if allocation.id
            != AllocationId(u8::try_from(index).map_err(|_| ProgramError::Storage {
                rule: "allocation-id-overflow",
            })?)
        {
            return Err(ProgramError::Storage {
                rule: "allocation-id",
            });
        }
    }
    if program.routing != routing_policy() {
        return Err(ProgramError::Routing {
            rule: "fallback-after-commit",
        });
    }
    Ok(values)
}

fn verify_host_contract(
    program: &KernelProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<Vec<HostValue>, ProgramError> {
    let (input_elements, output_elements) = scheduled_element_counts(scheduled)?;
    let expected_expressions = canonical_host_expressions(input_elements, output_elements);
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
    if program.buffer_plan.values.len()
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

pub(crate) fn build_artifact_plan(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
    kernels: &[VerifiedStructuredKernel],
    program: &KernelProgram,
    providers: Vec<LoweringProviderIdentity>,
) -> Result<ArtifactConstructionPlan, ProgramError> {
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
        let expected = crate::physical::lower_structured_kernel(region).map_err(|_| {
            ProgramError::Structure {
                rule: "artifact-schedule-refinement",
            }
        })?;
        if kernel != &expected {
            return Err(ProgramError::Structure {
                rule: "artifact-kernel-refinement",
            });
        }
    }
    let expected_program = match scheduled {
        [single] => build_fused_kernel_program(request, single)?,
        [_, _] => build_kernel_program(request, scheduled)?,
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
    if semantic.output_count() != 1
        || program.outputs.len() != 1
        || program.outputs[0].key != semantic_output.key().as_str()
    {
        return Err(ProgramError::Structure {
            rule: "semantic-output-coverage",
        });
    }
    if program.target_profile_key != request.target_profile().key
        || program
            .entries
            .iter()
            .any(|entry| entry.numerical != request.numerical_contract().into())
    {
        return Err(ProgramError::Structure {
            rule: "artifact-numerical-realization",
        });
    }
    let expected_providers = match program.stages.len() {
        1 => request
            .capabilities()
            .fused_serial_sum
            .into_iter()
            .collect::<Vec<_>>(),
        2 => vec![request.capabilities().materialized_serial_sum],
        _ => Vec::new(),
    };
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
            .stages
            .iter()
            .map(|stage| stage.scheduled_region)
            .collect(),
        routing_guard: program.applicability_guard,
        lowering_providers: providers,
        request_subject: request.subject(),
        verified_program: program.clone(),
        verified_schedules: scheduled.to_vec(),
        verified_kernels: kernels.to_vec(),
    })
}

pub(crate) fn verify_artifact_plan(
    plan: &ArtifactConstructionPlan,
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
    kernels: &[VerifiedStructuredKernel],
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
        expression(0, HostValueType::U64, HostExprNode::U64(4)),
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

fn scheduled_element_counts(
    scheduled: &[VerifiedScheduledRegion],
) -> Result<(u64, u64), ProgramError> {
    let first = scheduled.first().ok_or(ProgramError::Structure {
        rule: "cardinality",
    })?;
    let last = scheduled.last().ok_or(ProgramError::Structure {
        rule: "cardinality",
    })?;
    let input_elements = match first
        .region()
        .index
        .accesses
        .first()
        .map(|access| &access.map)
    {
        Some(crate::physical::LogicalAccess::ReductionContributor { input_shape, .. }) => {
            shape_elements(input_shape)
        }
        Some(crate::physical::LogicalAccess::LinearIdentity) => {
            shape_elements(&first.region().index.iteration_shape)
        }
        None => None,
    }
    .ok_or(ProgramError::Structure {
        rule: "input-element-count",
    })?;
    let output_elements =
        shape_elements(&last.region().index.iteration_shape).ok_or(ProgramError::Structure {
            rule: "output-element-count",
        })?;
    Ok((input_elements, output_elements))
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

fn verify_storage(
    program: &KernelProgram,
    values: &[HostValue],
    scheduled: &[VerifiedScheduledRegion],
) -> Result<(), ProgramError> {
    if scheduled.len() != 2
        || program.buffer_plan.values.len() != 3
        || program.buffer_plan.allocations.len() != 3
    {
        return Err(ProgramError::Storage {
            rule: "strategy-cardinality",
        });
    }
    let expected_shapes = [
        &scheduled[0].region().index.iteration_shape,
        &scheduled[0].region().index.iteration_shape,
        &scheduled[1].region().index.iteration_shape,
    ];
    let expected_required_bytes = [HostExprId(2), HostExprId(2), HostExprId(4)];
    for (position, value) in program.buffer_plan.values.iter().enumerate() {
        if usize::from(value.id.0) != position
            || value.tensor
                != [
                    TensorRole::Input,
                    TensorRole::Intermediate,
                    TensorRole::Output,
                ][position]
            || value.role != [ValueRole::Input, ValueRole::Temporary, ValueRole::Output][position]
            || value.memory_space != MemorySpace::Device
            || value.alignment != 4
            || value.required_bytes != expected_required_bytes[position]
            || value.allocation != AllocationId(u8::try_from(position).expect("three values"))
            || value.definition != [None, Some(StageId(0)), Some(StageId(1))][position]
            || &value.shape != expected_shapes[position]
        {
            return Err(ProgramError::Storage {
                rule: "materialized-values",
            });
        }
        let allocation = &program.buffer_plan.allocations[position];
        if allocation.id != value.allocation
            || allocation.capacity_bytes != value.required_bytes
            || allocation.alignment != value.alignment
            || allocation.memory_space != value.memory_space
            || allocation.ownership
                != if position == 0 {
                    AllocationOwnership::External
                } else {
                    AllocationOwnership::Program
                }
            || !matches!(
                values.get(usize::from(value.required_bytes.0)),
                Some(HostValue::U64(_))
            )
        {
            return Err(ProgramError::Storage {
                rule: "allocation-binding",
            });
        }
        let expected_bytes = shape_elements(&value.shape)
            .and_then(|elements| elements.checked_mul(4))
            .ok_or(ProgramError::Storage {
                rule: "required-byte-overflow",
            })?;
        if values.get(usize::from(value.required_bytes.0)) != Some(&HostValue::U64(expected_bytes))
        {
            return Err(ProgramError::Storage {
                rule: "required-byte-count",
            });
        }
    }
    if program.buffer_plan.values[0].shape != program.buffer_plan.values[1].shape
        || program.buffer_plan.values[2].shape == program.buffer_plan.values[1].shape
        || program.buffer_plan.allocations[0].id == program.buffer_plan.allocations[1].id
        || program.buffer_plan.allocations[0].id == program.buffer_plan.allocations[2].id
        || program.buffer_plan.allocations[1].id == program.buffer_plan.allocations[2].id
    {
        return Err(ProgramError::Storage {
            rule: "forbidden-alias-or-shape",
        });
    }
    Ok(())
}

fn verify_fused_storage(
    program: &KernelProgram,
    values: &[HostValue],
    scheduled: &[VerifiedScheduledRegion],
) -> Result<(), ProgramError> {
    let [input, output] = program.buffer_plan.values.as_slice() else {
        return Err(ProgramError::Storage {
            rule: "fused-cardinality",
        });
    };
    let [input_allocation, output_allocation] = program.buffer_plan.allocations.as_slice() else {
        return Err(ProgramError::Storage {
            rule: "fused-cardinality",
        });
    };
    if scheduled.len() != 1
        || input.tensor != TensorRole::Input
        || input.role != ValueRole::Input
        || input.definition.is_some()
        || output.tensor != TensorRole::Output
        || output.role != ValueRole::Output
        || output.definition != Some(StageId(0))
        || input.allocation != AllocationId(0)
        || output.allocation != AllocationId(1)
        || input.required_bytes != HostExprId(2)
        || output.required_bytes != HostExprId(4)
        || input_allocation.id != input.allocation
        || output_allocation.id != output.allocation
        || input.shape
            != match scheduled[0]
                .region()
                .index
                .accesses
                .first()
                .map(|access| &access.map)
            {
                Some(crate::physical::LogicalAccess::ReductionContributor {
                    input_shape, ..
                }) => input_shape.clone(),
                Some(crate::physical::LogicalAccess::LinearIdentity) | None => {
                    return Err(ProgramError::Storage {
                        rule: "fused-input-map",
                    });
                }
            }
        || output.shape != scheduled[0].region().index.iteration_shape
        || input_allocation.ownership != AllocationOwnership::External
        || output_allocation.ownership != AllocationOwnership::Program
        || input.allocation == output.allocation
    {
        return Err(ProgramError::Storage {
            rule: "fused-values",
        });
    }
    for value in [input, output] {
        let allocation = &program.buffer_plan.allocations[usize::from(value.allocation.0)];
        let expected_bytes = shape_elements(&value.shape)
            .and_then(|elements| elements.checked_mul(4))
            .ok_or(ProgramError::Storage {
                rule: "required-byte-overflow",
            })?;
        if value.memory_space != MemorySpace::Device
            || value.alignment != 4
            || allocation.capacity_bytes != value.required_bytes
            || allocation.memory_space != value.memory_space
            || allocation.alignment != value.alignment
            || values.get(usize::from(value.required_bytes.0))
                != Some(&HostValue::U64(expected_bytes))
        {
            return Err(ProgramError::Storage {
                rule: "fused-allocation-binding",
            });
        }
    }
    Ok(())
}

fn verify_entry(
    program: &KernelProgram,
    entry: &EntryContract,
    expected_stage: StageId,
    scheduled: &VerifiedScheduledRegion,
    values: &[HostValue],
) -> Result<(), ProgramError> {
    let stage = usize::from(entry.stage.0);
    if entry.stage != expected_stage
        || stage >= program.stages.len()
        || entry.requirements != scheduled.requirements()
        || entry.numerical != scheduled.region().index.numerical
        || entry.threads_per_workgroup != HostExprId(8)
        || entry.launch_threads
            != if expected_stage == StageId(0) && program.stages.len() == 2 {
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
    let stage_values = &program.stages[stage].values;
    if entry.bindings.len() != stage_values.len() {
        return Err(ProgramError::Abi {
            rule: "binding-cardinality",
            stage: entry.stage,
        });
    }
    for (position, binding) in entry.bindings.iter().enumerate() {
        let expected_access = match stage_values[position].access {
            StageAccess::Read => AbiAccess::Read,
            StageAccess::Write => AbiAccess::Write,
        };
        let Some(materialized_value) = program.buffer_plan.values.get(usize::from(binding.value.0))
        else {
            return Err(ProgramError::Abi {
                rule: "binding-value",
                stage: entry.stage,
            });
        };
        if binding.id != EntryBindingId(u8::try_from(position).expect("bounded binding count"))
            || binding.value != stage_values[position].value
            || binding.access != expected_access
            || binding.alignment != 4
            || binding.accessible_bytes != materialized_value.required_bytes
            || binding.role != component_role(materialized_value)
        {
            return Err(ProgramError::Abi {
                rule: "binding",
                stage: entry.stage,
            });
        }
    }
    Ok(())
}

const fn component_role(value: &MaterializedValue) -> ComponentRole {
    match value.role {
        ValueRole::Input => ComponentRole::Input,
        ValueRole::Temporary => ComponentRole::Intermediate,
        ValueRole::Output => ComponentRole::Output,
    }
}

fn shape_elements(shape: &Shape) -> Option<u64> {
    if shape.extents().iter().any(|extent| extent.get() == 0) {
        return Some(0);
    }
    shape
        .extents()
        .iter()
        .try_fold(1_u64, |count, extent| count.checked_mul(extent.get()))
}

fn materialized(
    id: u8,
    tensor: TensorRole,
    role: ValueRole,
    shape: Shape,
    required_bytes: HostExprId,
    definition: Option<StageId>,
    allocation: u8,
) -> MaterializedValue {
    MaterializedValue {
        id: MaterializedValueId(id),
        tensor,
        role,
        shape,
        required_bytes,
        alignment: 4,
        memory_space: MemorySpace::Device,
        definition,
        allocation: AllocationId(allocation),
    }
}

fn allocation(id: u8, capacity_bytes: HostExprId, ownership: AllocationOwnership) -> Allocation {
    Allocation {
        id: AllocationId(id),
        capacity_bytes,
        alignment: 4,
        memory_space: MemorySpace::Device,
        ownership,
    }
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
        alignment: 4,
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

pub(crate) fn assert_kernels_match_program(
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
    program: &KernelProgram,
    kernels: &[VerifiedStructuredKernel],
) -> Result<(), ProgramError> {
    if kernels.len() != scheduled.len()
        || kernels.len() != program.stages.len()
        || kernels.len() != program.entries.len()
        || scheduled
            .iter()
            .any(|region| !region.matches_request(request))
    {
        return Err(ProgramError::Structure {
            rule: "kernel-entry-cardinality",
        });
    }
    for (index, (scheduled, kernel)) in scheduled.iter().zip(kernels).enumerate() {
        if lower_structured_kernel(scheduled).map_err(|_| ProgramError::Structure {
            rule: "kernel-schedule-refinement",
        })? != *kernel
        {
            return Err(ProgramError::Structure {
                rule: "kernel-schedule-refinement",
            });
        }
        if kernel.kernel().buffers.len() != 2 {
            return Err(ProgramError::Structure {
                rule: "kernel-buffer-cardinality",
            });
        }
        let stage_values = &program.stages[index].values;
        if stage_values.len() != 2 {
            return Err(ProgramError::Structure {
                rule: "kernel-stage-value-cardinality",
            });
        }
        let read = program
            .buffer_plan
            .values
            .get(usize::from(stage_values[0].value.0))
            .ok_or(ProgramError::Structure {
                rule: "kernel-stage-value",
            })?;
        let write = program
            .buffer_plan
            .values
            .get(usize::from(stage_values[1].value.0))
            .ok_or(ProgramError::Structure {
                rule: "kernel-stage-value",
            })?;
        if kernel.kernel().scheduled_region != program.stages[index].scheduled_region
            || kernel.kernel().requirements != program.entries[index].requirements
            || kernel.kernel().numerical != program.entries[index].numerical
            || kernel.kernel().buffers[0].tensor != read.tensor
            || kernel.kernel().buffers[1].tensor != write.tensor
        {
            return Err(ProgramError::Structure {
                rule: "kernel-entry-refinement",
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
mod tests {
    use super::*;
    use crate::physical::{
        build_fused_scheduled_region, build_scheduled_regions, lower_structured_kernel,
    };
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::semantic::{
        F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::Axis;

    fn fixture() -> (
        SemanticProgram,
        VerifiedTargetRequest,
        Vec<VerifiedScheduledRegion>,
    ) {
        fixture_with_scale(2.0_f32.to_bits())
    }

    fn fixture_with_scale(
        scale_bits: u32,
    ) -> (
        SemanticProgram,
        VerifiedTargetRequest,
        Vec<VerifiedScheduledRegion>,
    ) {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, scale_bits).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let semantic = builder.build().unwrap();
        let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
        let request = request.for_target(request.target_profiles()[0]).unwrap();
        let scheduled = build_scheduled_regions(&request).unwrap();
        (semantic, request, scheduled)
    }

    #[test]
    fn artifact_construction_rejects_a_cross_program_semantic_request_mix() {
        let (_, request, scheduled) = fixture_with_scale(2.0_f32.to_bits());
        let (different_semantic, _, _) = fixture_with_scale(3.0_f32.to_bits());
        let program = build_kernel_program(&request, &scheduled).unwrap();

        assert_eq!(
            build_artifact_plan(
                &different_semantic,
                &request,
                &scheduled,
                &scheduled
                    .iter()
                    .map(lower_structured_kernel)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
                &program,
                vec![request.capabilities().materialized_serial_sum],
            ),
            Err(ProgramError::Structure {
                rule: "semantic-request-binding",
            })
        );
    }

    #[test]
    fn two_stage_program_has_explicit_temporary_abi_and_routing_commit() {
        let (semantic, request, scheduled) = fixture();
        let program = build_kernel_program(&request, &scheduled).unwrap();
        let kernels = [
            lower_structured_kernel(&scheduled[0]).unwrap(),
            lower_structured_kernel(&scheduled[1]).unwrap(),
        ];
        assert_kernels_match_program(&request, &scheduled, &program, &kernels).unwrap();
        verify_semantic_output_type(&semantic).unwrap();
        let artifact = build_artifact_plan(
            &semantic,
            &request,
            &scheduled,
            &kernels,
            &program,
            vec![request.capabilities().materialized_serial_sum],
        )
        .unwrap();

        assert_eq!(program.buffer_plan.values[1].role, ValueRole::Temporary);
        assert_eq!(
            program.dependencies[0].reason,
            DependencyReason::Data(MaterializedValueId(1))
        );
        assert_eq!(
            program.buffer_plan.allocations[1].ownership,
            AllocationOwnership::Program
        );
        assert_ne!(
            program.buffer_plan.allocations[1].id,
            program.buffer_plan.allocations[2].id
        );
        assert_eq!(
            program.entries[0].bindings[1].role,
            ComponentRole::Intermediate
        );
        assert_eq!(
            program.entries[1].bindings[0].role,
            ComponentRole::Intermediate
        );
        assert!(!program.routing[1].fallback_permitted);
        assert!(!program.routing[2].fallback_permitted);
        assert_eq!(artifact.entry_regions, [RegionId(0), RegionId(1)]);
        assert_eq!(
            artifact.numerical_realizations,
            [
                scheduled[0].region().index.numerical,
                scheduled[1].region().index.numerical,
            ]
        );
        assert!(!artifact.semantic_identity.graph().as_bytes().is_empty());
        assert!(
            !artifact
                .semantic_identity
                .reached_definitions()
                .as_bytes()
                .is_empty()
        );
        assert!(
            !artifact
                .semantic_identity
                .admission_provenance()
                .as_bytes()
                .is_empty()
        );
        assert!(
            !artifact
                .semantic_identity
                .registry_snapshot()
                .as_bytes()
                .is_empty()
        );
    }

    #[test]
    fn whole_program_verifier_rejects_dependency_alias_abi_and_routing_failures() {
        let (_, request, scheduled) = fixture();
        let valid = build_kernel_program(&request, &scheduled).unwrap();

        let mut missing_dependency = valid.clone();
        missing_dependency.dependencies[0].predecessor = StageId(1);
        assert_eq!(
            verify_kernel_program(missing_dependency, &request, &scheduled),
            Err(ProgramError::Dependency {
                rule: "initialized-cross-stage-value"
            })
        );

        let mut aliased = valid.clone();
        aliased.buffer_plan.values[2].allocation = AllocationId(1);
        assert_eq!(
            verify_kernel_program(aliased, &request, &scheduled),
            Err(ProgramError::Storage {
                rule: "materialized-values"
            })
        );

        let mut invalid_abi = valid.clone();
        invalid_abi.entries[1].bindings[0].access = AbiAccess::Write;
        assert_eq!(
            verify_kernel_program(invalid_abi, &request, &scheduled),
            Err(ProgramError::Abi {
                rule: "binding",
                stage: StageId(1),
            })
        );

        let mut invalid_routing = valid;
        invalid_routing.routing[1].fallback_permitted = true;
        assert_eq!(
            verify_kernel_program(invalid_routing, &request, &scheduled),
            Err(ProgramError::Routing {
                rule: "fallback-after-commit"
            })
        );
    }

    #[test]
    fn whole_program_verifier_rechecks_target_shape_bytes_launch_and_outputs() {
        let (_, request, scheduled) = fixture();
        let valid = build_kernel_program(&request, &scheduled).unwrap();

        let mut wrong_target = valid.clone();
        wrong_target.target_profile_key = "wrong-target";
        assert_eq!(
            verify_kernel_program(wrong_target, &request, &scheduled),
            Err(ProgramError::Structure {
                rule: "target-profile"
            })
        );

        let mut wrong_shape = valid.clone();
        wrong_shape.buffer_plan.values[2].shape = Shape::from_dims([1]);
        assert_eq!(
            verify_kernel_program(wrong_shape, &request, &scheduled),
            Err(ProgramError::Storage {
                rule: "materialized-values"
            })
        );

        let mut wrong_bytes = valid.clone();
        wrong_bytes.host_expressions[2].node = HostExprNode::U64(4);
        assert_eq!(
            verify_kernel_program(wrong_bytes, &request, &scheduled),
            Err(ProgramError::HostExpression {
                rule: "canonical-graph",
                expression: HostExprId(0),
            })
        );

        let mut wrong_launch = valid.clone();
        wrong_launch.host_expressions[5].node = HostExprNode::U64(5);
        assert_eq!(
            verify_kernel_program(wrong_launch, &request, &scheduled),
            Err(ProgramError::HostExpression {
                rule: "canonical-graph",
                expression: HostExprId(0),
            })
        );

        let mut missing_output = valid;
        missing_output.outputs[0].key.clear();
        assert_eq!(
            verify_kernel_program(missing_output, &request, &scheduled),
            Err(ProgramError::Structure {
                rule: "semantic-output-coverage"
            })
        );
    }

    #[test]
    fn variable_length_program_collections_fail_closed_on_wrong_cardinality() {
        let (_, request, scheduled) = fixture();
        let valid = build_kernel_program(&request, &scheduled).unwrap();

        let mut missing_stage = valid.clone();
        missing_stage.stages.pop();
        assert_eq!(
            verify_kernel_program(missing_stage, &request, &scheduled),
            Err(ProgramError::Structure {
                rule: "strategy-cardinality"
            })
        );

        let mut extra_binding = valid.clone();
        let duplicate_binding = extra_binding.entries[0].bindings[0];
        extra_binding.entries[0].bindings.push(duplicate_binding);
        assert_eq!(
            verify_kernel_program(extra_binding, &request, &scheduled),
            Err(ProgramError::Abi {
                rule: "binding-cardinality",
                stage: StageId(0),
            })
        );

        let kernels = [lower_structured_kernel(&scheduled[0]).unwrap()];
        assert_eq!(
            assert_kernels_match_program(&request, &scheduled, &valid, &kernels),
            Err(ProgramError::Structure {
                rule: "kernel-entry-cardinality"
            })
        );
    }

    #[test]
    fn fused_program_verifier_is_cardinality_independent_and_fails_closed() {
        let (_, request, _) = fixture();
        let scheduled = build_fused_scheduled_region(&request).unwrap();
        let valid = build_fused_kernel_program(&request, &scheduled).unwrap();
        let kernel = lower_structured_kernel(&scheduled).unwrap();
        assert_kernels_match_program(
            &request,
            std::slice::from_ref(&scheduled),
            &valid,
            std::slice::from_ref(&kernel),
        )
        .unwrap();

        let mut malformed = valid.clone();
        malformed.buffer_plan.values[1].definition = None;
        assert_eq!(
            verify_fused_serial_sum_program(malformed, &request, std::slice::from_ref(&scheduled),),
            Err(ProgramError::Storage {
                rule: "fused-values"
            })
        );

        let mut malformed = valid.clone();
        malformed.stages[0].values[1].value = MaterializedValueId(7);
        assert_eq!(
            verify_fused_serial_sum_program(malformed, &request, std::slice::from_ref(&scheduled),),
            Err(ProgramError::Structure {
                rule: "fused-stage"
            })
        );

        let mut malformed = valid;
        malformed.dependencies.push(Dependency {
            predecessor: StageId(0),
            successor: StageId(0),
            reason: DependencyReason::Data(MaterializedValueId(1)),
        });
        assert_eq!(
            verify_fused_serial_sum_program(malformed, &request, std::slice::from_ref(&scheduled),),
            Err(ProgramError::Structure {
                rule: "fused-strategy-cardinality"
            })
        );
    }

    #[test]
    fn host_expression_overflow_is_a_hard_failure() {
        let (_, request, scheduled) = fixture();
        let mut program = build_kernel_program(&request, &scheduled).unwrap();
        program.host_expressions[0].node = HostExprNode::U64(u64::MAX);
        assert_eq!(
            evaluate_expressions(&program.host_expressions),
            Err(ProgramError::HostExpression {
                rule: "overflow",
                expression: HostExprId(2),
            })
        );

        let mut malformed = build_kernel_program(&request, &scheduled).unwrap();
        malformed.host_expressions[2].node =
            HostExprNode::CheckedMultiply(HostExprId(99), HostExprId(1));
        assert_eq!(
            verify_kernel_program(malformed, &request, &scheduled),
            Err(ProgramError::HostExpression {
                rule: "canonical-graph",
                expression: HostExprId(0),
            })
        );
    }

    #[test]
    fn builders_and_verifiers_are_total_over_short_and_forged_slices() {
        let (_, request, scheduled) = fixture();
        assert_eq!(
            build_kernel_program(&request, &[]),
            Err(ProgramError::Structure {
                rule: "strategy-cardinality",
            })
        );
        assert_eq!(
            build_kernel_program(&request, &scheduled[..1]),
            Err(ProgramError::Structure {
                rule: "strategy-cardinality",
            })
        );

        let fused = build_fused_scheduled_region(&request).unwrap();
        let program = build_fused_kernel_program(&request, &fused).unwrap();
        assert!(
            verify_fused_serial_sum_program(program, &request, std::slice::from_ref(&fused),)
                .is_ok()
        );
    }

    #[test]
    fn artifact_receipt_rejects_provider_program_and_receipt_mutations() {
        let (semantic, request, scheduled) = fixture();
        let program = build_kernel_program(&request, &scheduled).unwrap();
        let kernels = scheduled
            .iter()
            .map(lower_structured_kernel)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let provider = request.capabilities().materialized_serial_sum;

        for providers in [Vec::new(), vec![provider, provider]] {
            assert_eq!(
                build_artifact_plan(
                    &semantic, &request, &scheduled, &kernels, &program, providers,
                ),
                Err(ProgramError::Structure {
                    rule: "artifact-provider-coverage",
                })
            );
        }

        let plan = build_artifact_plan(
            &semantic,
            &request,
            &scheduled,
            &kernels,
            &program,
            vec![provider],
        )
        .unwrap();
        let mut forged = plan.clone();
        forged.routing_guard = HostExprId(6);
        assert_eq!(
            verify_artifact_plan(
                &forged,
                &semantic,
                &request,
                &scheduled,
                &kernels,
                &program,
                vec![provider],
            ),
            Err(ProgramError::Structure {
                rule: "artifact-receipt",
            })
        );

        let mut swapped =
            build_fused_kernel_program(&request, &build_fused_scheduled_region(&request).unwrap())
                .unwrap();
        swapped.buffer_plan.values[0].allocation = AllocationId(1);
        swapped.buffer_plan.values[1].allocation = AllocationId(0);
        let fused = build_fused_scheduled_region(&request).unwrap();
        assert!(matches!(
            verify_fused_serial_sum_program(swapped, &request, std::slice::from_ref(&fused)),
            Err(ProgramError::Storage { .. })
        ));
    }
}
