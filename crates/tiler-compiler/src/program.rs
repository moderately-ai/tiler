use std::error::Error;
use std::fmt;

use tiler_ir::semantic::{F32, SemanticProgram};
use tiler_ir::shape::Shape;

use crate::physical::{
    NumericalRealization, RegionId, ResourceRequirements, TensorRole, VerifiedScheduledRegion,
    VerifiedStructuredKernel,
};
use crate::request::VerifiedRequest;

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
    pub(crate) values: [StageValueAccess; 2],
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
    pub(crate) bindings: [EntryBinding; 2],
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
    pub(crate) values: [MaterializedValue; 3],
    pub(crate) allocations: [Allocation; 3],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelProgram {
    pub(crate) target_profile_key: &'static str,
    pub(crate) host_expressions: Vec<HostExpr>,
    pub(crate) applicability_guard: HostExprId,
    pub(crate) stages: [ProgramStage; 2],
    pub(crate) dependencies: [Dependency; 1],
    pub(crate) buffer_plan: BufferPlan,
    pub(crate) entries: [EntryContract; 2],
    pub(crate) outputs: [ProgramOutput; 1],
    pub(crate) routing: [RoutingTransition; 3],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactConstructionPlan {
    pub(crate) semantic_identity: Vec<u8>,
    pub(crate) reached_semantic_authority: Vec<u8>,
    pub(crate) numerical_profile_key: &'static str,
    pub(crate) numerical_realizations: [NumericalRealization; 2],
    pub(crate) target_profile_key: &'static str,
    pub(crate) entry_regions: [RegionId; 2],
    pub(crate) routing_guard: HostExprId,
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
    request: &VerifiedRequest,
    scheduled: &[VerifiedScheduledRegion; 2],
) -> Result<KernelProgram, ProgramError> {
    let expressions = host_expressions(request)?;
    let input_bytes = HostExprId(2);
    let output_bytes = HostExprId(4);
    let program = KernelProgram {
        target_profile_key: request.target_profile.key,
        host_expressions: expressions,
        applicability_guard: HostExprId(7),
        stages: program_stages(scheduled),
        dependencies: [Dependency {
            predecessor: StageId(0),
            successor: StageId(1),
            reason: DependencyReason::Data(MaterializedValueId(1)),
        }],
        buffer_plan: BufferPlan {
            values: materialized_values(request, input_bytes, output_bytes),
            allocations: program_allocations(input_bytes, output_bytes),
        },
        entries: entry_contracts(scheduled, input_bytes, output_bytes),
        outputs: [ProgramOutput {
            key: request.normalized.output_key.as_str().to_owned(),
            value: MaterializedValueId(2),
        }],
        routing: routing_policy(),
    };
    verify_kernel_program(program, scheduled)
}

fn program_stages(scheduled: &[VerifiedScheduledRegion; 2]) -> [ProgramStage; 2] {
    [
        ProgramStage {
            id: StageId(0),
            scheduled_region: scheduled[0].region.index.id,
            values: [
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
            scheduled_region: scheduled[1].region.index.id,
            values: [
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
    request: &VerifiedRequest,
    input_bytes: HostExprId,
    output_bytes: HostExprId,
) -> [MaterializedValue; 3] {
    [
        materialized(
            0,
            TensorRole::Input,
            ValueRole::Input,
            request.normalized.input_shape.clone(),
            input_bytes,
            None,
            0,
        ),
        materialized(
            1,
            TensorRole::Intermediate,
            ValueRole::Temporary,
            request.normalized.input_shape.clone(),
            input_bytes,
            Some(StageId(0)),
            1,
        ),
        materialized(
            2,
            TensorRole::Output,
            ValueRole::Output,
            request.normalized.output_shape.clone(),
            output_bytes,
            Some(StageId(1)),
            2,
        ),
    ]
}

fn program_allocations(input_bytes: HostExprId, output_bytes: HostExprId) -> [Allocation; 3] {
    [
        allocation(0, input_bytes, AllocationOwnership::External),
        allocation(1, input_bytes, AllocationOwnership::Program),
        allocation(2, output_bytes, AllocationOwnership::Program),
    ]
}

fn entry_contracts(
    scheduled: &[VerifiedScheduledRegion; 2],
    input_bytes: HostExprId,
    output_bytes: HostExprId,
) -> [EntryContract; 2] {
    [
        entry(
            0,
            [
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
            scheduled[0].requirements,
            scheduled[0].region.index.numerical,
        ),
        entry(
            1,
            [
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
            scheduled[1].requirements,
            scheduled[1].region.index.numerical,
        ),
    ]
}

fn routing_policy() -> [RoutingTransition; 3] {
    [
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

pub(crate) fn verify_kernel_program(
    program: KernelProgram,
    scheduled: &[VerifiedScheduledRegion; 2],
) -> Result<KernelProgram, ProgramError> {
    let values = evaluate_expressions(&program.host_expressions)?;
    if values.get(usize::from(program.applicability_guard.0)) != Some(&HostValue::Bool(true)) {
        return Err(ProgramError::Structure {
            rule: "applicability-guard",
        });
    }
    if scheduled
        .iter()
        .any(|region| region.target_profile_key != program.target_profile_key)
    {
        return Err(ProgramError::Structure {
            rule: "target-profile",
        });
    }
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
        || program.outputs[0].key.is_empty()
    {
        return Err(ProgramError::Structure {
            rule: "semantic-output-coverage",
        });
    }
    if program.routing
        != [
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
    {
        return Err(ProgramError::Routing {
            rule: "fallback-after-commit",
        });
    }
    Ok(program)
}

pub(crate) fn build_artifact_plan(
    semantic: &SemanticProgram,
    request: &VerifiedRequest,
    program: &KernelProgram,
) -> Result<ArtifactConstructionPlan, ProgramError> {
    let semantic_output = semantic.outputs().next().ok_or(ProgramError::Structure {
        rule: "semantic-output-coverage",
    })?;
    if semantic.output_count() != 1 || program.outputs[0].key != semantic_output.key().as_str() {
        return Err(ProgramError::Structure {
            rule: "semantic-output-coverage",
        });
    }
    if program
        .entries
        .iter()
        .any(|entry| entry.numerical.profile_key != request.numerical_profile.key)
    {
        return Err(ProgramError::Structure {
            rule: "artifact-numerical-realization",
        });
    }
    Ok(ArtifactConstructionPlan {
        semantic_identity: semantic.canonical_identity().as_bytes().to_vec(),
        reached_semantic_authority: request.semantic_authority.as_bytes().to_vec(),
        numerical_profile_key: request.numerical_profile.key,
        numerical_realizations: [program.entries[0].numerical, program.entries[1].numerical],
        target_profile_key: program.target_profile_key,
        entry_regions: [
            program.stages[0].scheduled_region,
            program.stages[1].scheduled_region,
        ],
        routing_guard: program.applicability_guard,
    })
}

fn host_expressions(request: &VerifiedRequest) -> Result<Vec<HostExpr>, ProgramError> {
    let expressions = vec![
        expression(0, HostValueType::U64, HostExprNode::U64(4)),
        expression(
            1,
            HostValueType::U64,
            HostExprNode::U64(request.normalized.input_elements),
        ),
        expression(
            2,
            HostValueType::U64,
            HostExprNode::CheckedMultiply(HostExprId(0), HostExprId(1)),
        ),
        expression(
            3,
            HostValueType::U64,
            HostExprNode::U64(request.normalized.output_elements),
        ),
        expression(
            4,
            HostValueType::U64,
            HostExprNode::CheckedMultiply(HostExprId(0), HostExprId(3)),
        ),
        expression(
            5,
            HostValueType::U64,
            HostExprNode::U64(request.normalized.input_elements),
        ),
        expression(
            6,
            HostValueType::U64,
            HostExprNode::U64(request.normalized.output_elements),
        ),
        expression(7, HostValueType::Bool, HostExprNode::Bool(true)),
        expression(8, HostValueType::U64, HostExprNode::U64(1)),
    ];
    let actual = expressions.len();
    if actual
        > usize::try_from(request.budgets.host_expression_nodes)
            .expect("u32 fits every supported host")
    {
        return Err(ProgramError::Structure {
            rule: "host-expression-budget",
        });
    }
    Ok(expressions)
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
    scheduled: &[VerifiedScheduledRegion; 2],
) -> Result<(), ProgramError> {
    let expected_shapes = [
        &scheduled[0].region.index.iteration_shape,
        &scheduled[0].region.index.iteration_shape,
        &scheduled[1].region.index.iteration_shape,
    ];
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
        || entry.requirements != scheduled.requirements
        || entry.numerical != scheduled.region.index.numerical
        || entry.threads_per_workgroup != HostExprId(8)
        || entry.launch_threads != [HostExprId(5), HostExprId(6)][stage]
    {
        return Err(ProgramError::Abi {
            rule: "entry-contract",
            stage: entry.stage,
        });
    }
    if values.get(usize::from(entry.launch_threads.0))
        != Some(&HostValue::U64(scheduled.region.schedule.work_items))
        || values.get(usize::from(entry.threads_per_workgroup.0))
            != Some(&HostValue::U64(u64::from(
                scheduled.region.schedule.threads_per_workgroup,
            )))
    {
        return Err(ProgramError::Abi {
            rule: "launch-expression",
            stage: entry.stage,
        });
    }
    let stage_values = program.stages[stage].values;
    for (position, binding) in entry.bindings.iter().enumerate() {
        let expected_access = match stage_values[position].access {
            StageAccess::Read => AbiAccess::Read,
            StageAccess::Write => AbiAccess::Write,
        };
        if binding.id != EntryBindingId(u8::try_from(position).expect("two bindings"))
            || binding.value != stage_values[position].value
            || binding.access != expected_access
            || binding.alignment != 4
            || binding.accessible_bytes
                != program.buffer_plan.values[usize::from(binding.value.0)].required_bytes
            || binding.role
                != match binding.value.0 {
                    0 => ComponentRole::Input,
                    1 => ComponentRole::Intermediate,
                    2 => ComponentRole::Output,
                    _ => {
                        return Err(ProgramError::Abi {
                            rule: "binding-value",
                            stage: entry.stage,
                        });
                    }
                }
        {
            return Err(ProgramError::Abi {
                rule: "binding",
                stage: entry.stage,
            });
        }
    }
    Ok(())
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
    bindings: [EntryBinding; 2],
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
    program: &KernelProgram,
    kernels: &[VerifiedStructuredKernel; 2],
) -> Result<(), ProgramError> {
    for (index, kernel) in kernels.iter().enumerate() {
        if kernel.kernel.scheduled_region != program.stages[index].scheduled_region
            || kernel.kernel.requirements != program.entries[index].requirements
            || kernel.kernel.numerical != program.entries[index].numerical
            || kernel.kernel.buffers[0].tensor
                != [TensorRole::Input, TensorRole::Intermediate][index]
            || kernel.kernel.buffers[1].tensor
                != [TensorRole::Intermediate, TensorRole::Output][index]
        {
            return Err(ProgramError::Structure {
                rule: "kernel-entry-refinement",
            });
        }
    }
    Ok(())
}

pub(crate) fn verify_semantic_output_type(program: &SemanticProgram) -> Result<(), ProgramError> {
    if program.outputs().any(|output| {
        program
            .value(output.value())
            .map_or(true, |value| value.resolved_type() != &F32::resolved_type())
    }) {
        return Err(ProgramError::Structure {
            rule: "semantic-output-type",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical::{build_scheduled_regions, lower_structured_kernel};
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::semantic::{
        F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::Axis;

    fn fixture() -> (
        SemanticProgram,
        VerifiedRequest,
        [VerifiedScheduledRegion; 2],
    ) {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let semantic = builder.build().unwrap();
        let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
        let scheduled = build_scheduled_regions(&request).unwrap();
        (semantic, request, scheduled)
    }

    #[test]
    fn two_stage_program_has_explicit_temporary_abi_and_routing_commit() {
        let (semantic, request, scheduled) = fixture();
        let program = build_kernel_program(&request, &scheduled).unwrap();
        let kernels = [
            lower_structured_kernel(&scheduled[0]).unwrap(),
            lower_structured_kernel(&scheduled[1]).unwrap(),
        ];
        assert_kernels_match_program(&program, &kernels).unwrap();
        verify_semantic_output_type(&semantic).unwrap();
        let artifact = build_artifact_plan(&semantic, &request, &program).unwrap();

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
                scheduled[0].region.index.numerical,
                scheduled[1].region.index.numerical,
            ]
        );
        assert!(!artifact.semantic_identity.is_empty());
        assert!(!artifact.reached_semantic_authority.is_empty());
    }

    #[test]
    fn whole_program_verifier_rejects_dependency_alias_abi_and_routing_failures() {
        let (_, request, scheduled) = fixture();
        let valid = build_kernel_program(&request, &scheduled).unwrap();

        let mut missing_dependency = valid.clone();
        missing_dependency.dependencies[0].predecessor = StageId(1);
        assert_eq!(
            verify_kernel_program(missing_dependency, &scheduled),
            Err(ProgramError::Dependency {
                rule: "initialized-cross-stage-value"
            })
        );

        let mut aliased = valid.clone();
        aliased.buffer_plan.values[2].allocation = AllocationId(1);
        assert_eq!(
            verify_kernel_program(aliased, &scheduled),
            Err(ProgramError::Storage {
                rule: "materialized-values"
            })
        );

        let mut invalid_abi = valid.clone();
        invalid_abi.entries[1].bindings[0].access = AbiAccess::Write;
        assert_eq!(
            verify_kernel_program(invalid_abi, &scheduled),
            Err(ProgramError::Abi {
                rule: "binding",
                stage: StageId(1),
            })
        );

        let mut invalid_routing = valid;
        invalid_routing.routing[1].fallback_permitted = true;
        assert_eq!(
            verify_kernel_program(invalid_routing, &scheduled),
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
            verify_kernel_program(wrong_target, &scheduled),
            Err(ProgramError::Structure {
                rule: "target-profile"
            })
        );

        let mut wrong_shape = valid.clone();
        wrong_shape.buffer_plan.values[2].shape = Shape::from_dims([1]);
        assert_eq!(
            verify_kernel_program(wrong_shape, &scheduled),
            Err(ProgramError::Storage {
                rule: "materialized-values"
            })
        );

        let mut wrong_bytes = valid.clone();
        wrong_bytes.host_expressions[2].node = HostExprNode::U64(4);
        assert_eq!(
            verify_kernel_program(wrong_bytes, &scheduled),
            Err(ProgramError::Storage {
                rule: "required-byte-count"
            })
        );

        let mut wrong_launch = valid.clone();
        wrong_launch.host_expressions[5].node = HostExprNode::U64(5);
        assert_eq!(
            verify_kernel_program(wrong_launch, &scheduled),
            Err(ProgramError::Abi {
                rule: "launch-expression",
                stage: StageId(0),
            })
        );

        let mut missing_output = valid;
        missing_output.outputs[0].key.clear();
        assert_eq!(
            verify_kernel_program(missing_output, &scheduled),
            Err(ProgramError::Structure {
                rule: "semantic-output-coverage"
            })
        );
    }

    #[test]
    fn host_expression_overflow_is_a_hard_failure() {
        let (_, request, scheduled) = fixture();
        let mut program = build_kernel_program(&request, &scheduled).unwrap();
        program.host_expressions[0].node = HostExprNode::U64(u64::MAX);
        assert_eq!(
            verify_kernel_program(program, &scheduled),
            Err(ProgramError::HostExpression {
                rule: "overflow",
                expression: HostExprId(2),
            })
        );

        let mut malformed = build_kernel_program(&request, &scheduled).unwrap();
        malformed.host_expressions[2].node =
            HostExprNode::CheckedMultiply(HostExprId(99), HostExprId(1));
        assert_eq!(
            verify_kernel_program(malformed, &scheduled),
            Err(ProgramError::HostExpression {
                rule: "operand",
                expression: HostExprId(2),
            })
        );
    }
}
