//! Compiler-owned program layers over the shared target-neutral kernel program.
//!
//! The stage DAG, the exact selected scheduled/KIR refinements, the checked
//! materialized values, views, allocations, lifetimes and handoffs, the typed
//! dependencies, the named outputs, complete semantic coverage, the host
//! preflight expression arena, the entry ABI, the applicability guard, and the
//! routing-commit contract all live in [`tiler_ir::program`] (ADR 0070), where
//! they are constructed through the ADR 0071 checked builder and carry a
//! canonical identity folding the ADR 0072 layers.
//!
//! `complete-program-identity-with-abi-guards-and-routing` moved the last four
//! of those down. This module previously held a second copy of each, verified
//! against the shared core; the copies are gone rather than re-checked, because
//! two representations of one ABI that nothing keeps in agreement is the drift
//! ADR 0068 exists to prevent.
//!
//! What remains here is what only a *compilation* can decide: the target
//! binding, the request budgets, the compile-time truth of the applicability
//! guard, the agreement between each stage's declared launch and the scheduled
//! region it was planned from, and the artifact construction plan that binds
//! all of it to one compilation request.

use std::error::Error;
use std::fmt;

use tiler_ir::kernel::KernelType;
use tiler_ir::program::{
    AbiExprId, AllocationOwnership, AllocationSpec, KernelProgramBuildError, KernelProgramBuilder,
    KernelProgramDiagnostic, MaterializedOrigin, MaterializedValueSpec, MemorySpace,
    RoutingCommitState, RoutingCommitTransition, SemanticOccurrence, StageAccess, StageAccessMode,
    StageLaunch, StageRef, StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram,
    ViewId,
};
use tiler_ir::semantic::{F32, SemanticIdentity, SemanticProgram};
use tiler_ir::shape::Shape;

use tiler_ir::program::abi::{
    AbiBinaryOp, AbiEvaluationError, AbiFacts, AbiRoot, AbiValue, AvailabilityPhase, ExprNode,
    evaluate as abi_evaluate,
};

use crate::physical::{
    NumericalRealization, RegionId, VerifiedKernel, VerifiedScheduledRegion,
    lower_structured_kernel,
};
use crate::region::SemanticMemberId;
use crate::request::{LoweringProviderIdentity, TargetProfile, VerifiedTargetRequest};
use crate::target::feasibility::DeferredPredicate;
use crate::target::feasibility::{FeasibilityRuleSetIdentity, GOVERNED_FEASIBILITY_RULE_SET};

/// Element byte width of the bounded profile's single tensor element type.
const ELEMENT_BYTES: u64 = 4;
/// Byte alignment every bounded-profile value and allocation requires.
const ELEMENT_ALIGNMENT: u32 = 4;

/// Arena position of one node of the program's ABI expression arena.
///
/// This is a reference into the [`ExprNode`] arena
/// [`VerifiedKernelProgram::abi_expressions`] retains, not a second expression
/// vocabulary. `relocate-abi-expressions-into-tiler-ir` replaced the compiler's
/// own `HostExpr` — a nine-node table of `U64`/`Bool`/`CheckedMultiply` — with
/// the shared `AbiExpr` domain, because the two covered the same three facts
/// (guards, accessible byte counts, launch geometry) with two vocabularies that
/// nothing kept in agreement. That is the drift hazard ADR 0068 exists to
/// prevent, and it is why the width widened from `u8` to the arena's own `u32`.
///
/// It survives `complete-program-identity-with-abi-guards-and-routing` — which
/// moved the arena itself into the program — only as the spelling this crate's
/// typed errors and explain subjects use to name a rejected node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct HostExprId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct StageId(u8);

impl HostExprId {
    pub(crate) const fn index(self) -> u32 {
        self.0
    }
}

impl StageId {
    pub(crate) const fn index(self) -> u8 {
        self.0
    }
}

/// One target-bound executable program: a verified shared kernel program and
/// the target profile whose feasibility it was assessed under.
///
/// The ABI arena, the applicability guard, the entry ABI, and the
/// routing-commit contract are all inside `core` and inside its canonical
/// identity. This wrapper adds the one fact the target-neutral program
/// deliberately does not carry: which target profile it was planned for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelProgram {
    target_profile: TargetProfile,
    core: VerifiedKernelProgram,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactConstructionPlan {
    semantic_identity: SemanticIdentity,
    numerical_contract_key: &'static str,
    numerical_realizations: Vec<NumericalRealization>,
    /// The immutable profile the plan was assessed against. Its key and exact
    /// descriptor remain inseparable throughout artifact construction.
    target_profile: TargetProfile,
    /// Feasibility rules the plan's candidates were assessed under.
    ///
    /// A second, independent identity beside the profile rather than a field of
    /// it: one profile can be re-assessed under new rules and one rule set
    /// applies across profiles, so neither determines the other. The artifact
    /// layer records them as two references for exactly that reason.
    feasibility_rule_set: FeasibilityRuleSetIdentity,
    entry_regions: Vec<RegionId>,
    entry_deferred_predicates: Vec<EntryDeferredPredicate>,
    /// Arena position of the guard deciding whether this plan may be routed to.
    ///
    /// Named for what it decides rather than for "routing": the portfolio-level
    /// sense of that word orders variants against each other, and this guard
    /// orders nothing.
    applicability_guard: u32,
    lowering_providers: Vec<LoweringProviderIdentity>,
    request_subject: crate::request::VerifiedRequestSubject,
    verified_program: KernelProgram,
    verified_schedules: Vec<VerifiedScheduledRegion>,
    verified_kernels: Vec<VerifiedKernel>,
}

/// One compiler-minted deferred predicate bound to its exact program entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EntryDeferredPredicate {
    entry: u32,
    predicate: DeferredPredicate,
}

impl EntryDeferredPredicate {
    /// The zero-based program-entry ordinal whose prepared subject is queried.
    pub(crate) const fn entry(&self) -> u32 {
        self.entry
    }

    /// The typed predicate and executable query contract.
    pub(crate) const fn predicate(&self) -> &DeferredPredicate {
        &self.predicate
    }
}

impl KernelProgram {
    /// Returns the verified target-neutral program this target binding wraps.
    ///
    /// No longer `#[cfg(test)]`: `crate::session` is the reviewed public facade
    /// this accessor's previous comment was waiting for, and an artifact
    /// assembler outside this crate cannot package a variant without the
    /// program `push_variant` binds against.
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

    /// Returns compiler-minted deferred predicates in entry then predicate order.
    pub(crate) fn entry_deferred_predicates(&self) -> &[EntryDeferredPredicate] {
        &self.entry_deferred_predicates
    }

    /// Returns the target-bound program whose ABI contract this plan packages.
    pub(crate) const fn verified_program(&self) -> &KernelProgram {
        &self.verified_program
    }

    /// Returns the canonical descriptor bytes of the assessed target profile.
    pub(crate) fn target_profile_descriptor(&self) -> &[u8] {
        self.target_profile.canonical_descriptor()
    }

    /// Returns the feasibility rules this plan's candidates were assessed under.
    ///
    /// Minted by the feasibility authority and handed over whole, like a
    /// capability key: the pair enters artifact identity, and a consumer
    /// composing a key and a revision of its own would be a second derivation of
    /// one identity.
    pub(crate) const fn feasibility_rule_set(&self) -> FeasibilityRuleSetIdentity {
        self.feasibility_rule_set
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
        target_profile: request.target_profile().clone(),
        core,
    };
    verify_kernel_program_layers(&program, request, scheduled)?;
    Ok(program)
}

/// Builds a single-stage whole-program kernel for one request.
///
/// The scheduled region may be either a fused serial sum or a standalone
/// governed pointwise program.
pub(crate) fn build_fused_kernel_program(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &VerifiedScheduledRegion,
) -> Result<KernelProgram, ProgramError> {
    let core = build_fused_core(semantic, request, scheduled)?;
    let program = KernelProgram {
        target_profile: request.target_profile().clone(),
        core,
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
    let abi = declare_host_abi(
        &mut builder,
        subject.input_elements,
        subject.output_elements,
    )?;
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
        &[
            read(input_view, abi.input_bytes),
            write(temporary_view, abi.input_bytes),
        ],
        abi.launch(abi.input_elements),
    )?;
    let reduce_stage = builder.push_stage(
        &lower(reduction)?,
        &covered(subject.members.reduction()),
        &[
            read(temporary_view, abi.input_bytes),
            write(output_view, abi.output_bytes),
        ],
        abi.launch(abi.output_elements),
    )?;
    builder.push_data_dependency(map_stage, reduce_stage, temporary)?;
    builder.push_output(subject.output_key.clone(), output)?;
    declare_routing_commit(&mut builder)?;
    finish_core(builder)
}

/// Assembles the shared verified program of the fused strategy.
fn build_fused_core(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
    scheduled: &VerifiedScheduledRegion,
) -> Result<VerifiedKernelProgram, ProgramError> {
    let (
        input_key,
        output_key,
        input_shape,
        output_shape,
        input_elements,
        output_elements,
        members,
    ) = if let Some(pointwise) = request.pointwise() {
        (
            pointwise.input_key.clone(),
            pointwise.output_key.clone(),
            pointwise.shape.clone(),
            pointwise.shape.clone(),
            pointwise.elements,
            pointwise.elements,
            pointwise.members.clone(),
        )
    } else {
        let subject = request.serial_sum();
        (
            subject.input_key.clone(),
            subject.output_key.clone(),
            subject.input_shape.clone(),
            subject.output_shape.clone(),
            subject.input_elements,
            subject.output_elements,
            subject.members.all(),
        )
    };
    let input_bytes = byte_count(input_elements)?;
    let output_bytes = byte_count(output_elements)?;
    let mut builder = open_core_builder(semantic, request)?;
    let abi = declare_host_abi(&mut builder, input_elements, output_elements)?;
    let external = builder.push_allocation(storage(input_bytes, AllocationOwnership::External))?;
    let output_storage =
        builder.push_allocation(storage(output_bytes, AllocationOwnership::Program))?;
    let input = builder.push_value(program_input(input_key, input_shape), external)?;
    let output = builder.push_value(internal(ValueRole::Output, output_shape), output_storage)?;
    let input_view = builder.push_whole_view(input)?;
    let output_view = builder.push_whole_view(output)?;
    builder.push_stage(
        &lower(scheduled)?,
        &covered(&members),
        &[
            read(input_view, abi.input_bytes),
            write(output_view, abi.output_bytes),
        ],
        abi.launch(abi.output_elements),
    )?;
    builder.push_output(output_key, output)?;
    declare_routing_commit(&mut builder)?;
    finish_core(builder)
}

/// The ABI quantities named by programs in the bounded governed profile.
///
/// Every extent is an `UnsignedLiteral` because the bounded profile's shapes
/// are static, so each is already known at `CompileProfile`. The domain also
/// admits an `InputExtent` root that resolves at `LiveDevicePreflight`, which is
/// what a dynamic-shape subject would name instead; promoting these literals is
/// a capability question tied to dynamic shapes, not a property of the
/// vocabulary, and nothing in this contract has to change shape for it.
#[derive(Clone, Copy, Debug)]
struct HostAbi {
    input_bytes: AbiExprId,
    output_bytes: AbiExprId,
    input_elements: AbiExprId,
    output_elements: AbiExprId,
    threads_per_workgroup: AbiExprId,
}

impl HostAbi {
    /// Returns the launch of a stage whose work items are `grid_threads`.
    ///
    /// Every current region uses the profile's fixed workgroup width; the width
    /// the *kernel* requires is what the program builder proves the declared
    /// expression against.
    const fn launch(self, grid_threads: AbiExprId) -> StageLaunch {
        StageLaunch {
            grid_threads,
            threads_per_workgroup: self.threads_per_workgroup,
        }
    }
}

/// Declares the ABI arena and applicability guard of one bounded-profile program.
///
/// The arena is deduplicated by content inside the builder, so declaring the
/// same formula at several use sites yields one node. Operands always precede
/// their use, which is the arena's acyclicity invariant.
fn declare_host_abi(
    builder: &mut KernelProgramBuilder,
    input_elements: u64,
    output_elements: u64,
) -> Result<HostAbi, ProgramError> {
    // The element byte width every accessible range scales by.
    let element_bytes = builder.push_abi_root(AbiRoot::UnsignedLiteral(ELEMENT_BYTES))?;
    let input_elements = builder.push_abi_root(AbiRoot::UnsignedLiteral(input_elements))?;
    let output_elements = builder.push_abi_root(AbiRoot::UnsignedLiteral(output_elements))?;
    let abi = HostAbi {
        input_bytes: builder.push_abi_binary(
            AbiBinaryOp::CheckedMultiply,
            element_bytes,
            input_elements,
        )?,
        output_bytes: builder.push_abi_binary(
            AbiBinaryOp::CheckedMultiply,
            element_bytes,
            output_elements,
        )?,
        input_elements,
        output_elements,
        threads_per_workgroup: builder.push_abi_root(AbiRoot::UnsignedLiteral(1))?,
    };
    // The bounded profile admits every governed target unconditionally, so the
    // guard is a constant. It is still declared rather than assumed, because a
    // program identity blind to its guard is the hazard ADR 0072 names.
    let guard = builder.push_abi_root(AbiRoot::BooleanLiteral(true))?;
    builder.applicability_guard(guard)?;
    Ok(abi)
}

/// Declares the routing-commit lifecycle every compiled program shares.
///
/// Fallback to another plan is admitted only while nothing is committed. The
/// shared builder proves that rule rather than trusting it, so this states an
/// intent instead of re-deriving a policy.
fn declare_routing_commit(builder: &mut KernelProgramBuilder) -> Result<(), ProgramError> {
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
        builder.push_routing_commit_transition(RoutingCommitTransition {
            from,
            to,
            fallback_permitted,
        })?;
    }
    Ok(())
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
        storage_scalar: StorageScalar::F32,
        encoding: StorageEncoding::Unpacked,
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
        storage_scalar: StorageScalar::F32,
        encoding: StorageEncoding::Unpacked,
        element_type: KernelType::F32,
        alignment: ELEMENT_ALIGNMENT,
        memory_space: MemorySpace::Device,
    }
}

const fn read(view: ViewId, accessible_bytes: AbiExprId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Read,
        accessible_bytes,
    }
}

const fn write(view: ViewId, accessible_bytes: AbiExprId) -> StageAccess {
    StageAccess {
        view,
        mode: StageAccessMode::Write,
        accessible_bytes,
    }
}

fn byte_count(elements: u64) -> Result<u64, ProgramError> {
    elements
        .checked_mul(ELEMENT_BYTES)
        .ok_or(ProgramError::Storage {
            rule: "required-byte-overflow",
        })
}

/// Verifies the compiler-owned layers of one program against its shared core.
///
/// The shared core is already verified by construction — including its ABI
/// arena, guard, entry ABI and routing-commit contract — so this proves only
/// what a target-neutral program cannot: the request and target binding, the
/// request's budgets, the compile-time truth of the guard, and the agreement
/// between each stage's declared launch and the scheduled region it was planned
/// from.
pub(crate) fn verify_kernel_program_layers(
    program: &KernelProgram,
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<(), ProgramError> {
    if scheduled.is_empty() || program.core.stages().len() != scheduled.len() {
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
    if &program.target_profile != request.target_profile()
        || scheduled
            .iter()
            .any(|region| region.target_profile() != &program.target_profile)
    {
        return Err(ProgramError::Structure {
            rule: "target-profile",
        });
    }
    let values = verify_host_contract(program, request)?;
    for (position, (stage, region)) in program.core.stages().zip(scheduled).enumerate() {
        verify_entry(stage, region, position, &values)?;
    }
    // Fallback before commit is what makes preflight rejection recoverable, so
    // a governed compilation states it. The complementary rule — that no later
    // step permits fallback — is proven by the shared builder, which rejects
    // such a step as `RoutingCommitFallbackAfterCommit`.
    if !program
        .core
        .routing_commit_contract()
        .first()
        .is_some_and(|first| first.fallback_permitted)
    {
        return Err(ProgramError::Routing {
            rule: "pre-commit-fallback",
        });
    }
    Ok(())
}

fn verify_host_contract(
    program: &KernelProgram,
    request: &VerifiedTargetRequest,
) -> Result<Vec<AbiValue>, ProgramError> {
    let expressions = program.core.abi_expressions();
    if expressions.len()
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
    let values = evaluate_expressions(expressions)?;
    if values.get(position(program.core.applicability_guard())) != Some(&AbiValue::Boolean(true)) {
        return Err(ProgramError::Structure {
            rule: "applicability-guard",
        });
    }
    Ok(values)
}

/// Proves one stage's entry ABI realizes the region it was planned from.
///
/// The shared program already proves the structural half — that each access
/// binds the view its kernel buffer names, that its accessible range equals
/// that view's window, and that the declared workgroup width is the bound
/// kernel's. What only a compilation can add is the *planning* half: the region
/// this stage was scheduled from, whose launch extent and numerical realization
/// the entry must not contradict.
fn verify_entry(
    stage: StageRef<'_>,
    scheduled: &VerifiedScheduledRegion,
    position_of_stage: usize,
    values: &[AbiValue],
) -> Result<(), ProgramError> {
    let index = u8::try_from(position_of_stage).map_err(|_| ProgramError::Structure {
        rule: "stage-id-overflow",
    })?;
    let stage_id = StageId(index);
    if stage.kernel().requirements() != scheduled.requirements()
        || stage.kernel().numerical() != scheduled.region().index.numerical
    {
        return Err(ProgramError::Abi {
            rule: "entry-contract",
            stage: stage_id,
        });
    }
    let launch = stage.launch();
    if values.get(position(launch.grid_threads))
        != Some(&AbiValue::Unsigned(scheduled.region().schedule.work_items))
        || values.get(position(launch.threads_per_workgroup))
            != Some(&AbiValue::Unsigned(u64::from(
                scheduled.region().schedule.threads_per_workgroup,
            )))
    {
        return Err(ProgramError::Abi {
            rule: "launch-expression",
            stage: stage_id,
        });
    }
    // The shared layer permits a partial view; a bounded-profile entry binds a
    // whole materialized value, so its accessible range is that value's bytes.
    for access in stage.accesses() {
        let expected = AbiValue::Unsigned(access.view().value().required_bytes());
        if values.get(position(access.accessible_bytes())) != Some(&expected) {
            return Err(ProgramError::Abi {
                rule: "binding",
                stage: stage_id,
            });
        }
    }
    Ok(())
}

/// Converts a checked ABI arena ordinal into a host index.
fn position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
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
            .core
            .stages()
            .map(|stage| stage.kernel().numerical())
            .collect(),
        target_profile: program.target_profile.clone(),
        // Read from the authority that decides feasibility rather than composed
        // here. It is a constant and not a function of the request because the
        // rules do not vary by target: `CheckedTargetProfile::assess` applies
        // exactly these rules to every profile, so a per-target derivation would
        // imply a variation that cannot occur.
        feasibility_rule_set: GOVERNED_FEASIBILITY_RULE_SET,
        entry_regions: program
            .core
            .stages()
            .map(|stage| stage.kernel().scheduled_region())
            .collect(),
        entry_deferred_predicates: scheduled
            .iter()
            .enumerate()
            .flat_map(|(entry, region)| {
                region
                    .admission()
                    .deferred()
                    .into_iter()
                    .flat_map(move |deferred| {
                        deferred.predicates().iter().cloned().map(move |predicate| {
                            EntryDeferredPredicate {
                                entry: u32::try_from(entry)
                                    .expect("program stage counts are bounded below u32"),
                                predicate,
                            }
                        })
                    })
            })
            .collect(),
        applicability_guard: program.core.applicability_guard(),
        lowering_providers: providers,
        request_subject: request.subject().clone(),
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
    if &program.target_profile != request.target_profile()
        || program
            .core
            .stages()
            .any(|stage| stage.kernel().numerical() != request.numerical_contract().realization())
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

/// Evaluates every node of the program's ABI arena in arena order.
///
/// The shared evaluator is the authority for what each node means, so this
/// function owns only the mapping from its typed failures onto this crate's
/// rules. Every node is evaluated rather than only the roots the entries name:
/// the program layer proves every node is *reachable* from a use site, and this
/// proves every node is *evaluable* at compile time, which the bounded profile
/// requires but a program in general does not.
///
/// The bound environment is empty and reaches only `CompileProfile`: the
/// bounded profile's graph is entirely literals, so binding a device fact here
/// would be claiming an availability this stage does not have.
fn evaluate_expressions(expressions: &[ExprNode]) -> Result<Vec<AbiValue>, ProgramError> {
    let facts = AbiFacts::new(AvailabilityPhase::CompileProfile, Vec::new(), Vec::new());
    let mut values = Vec::with_capacity(expressions.len());
    for position in 0..expressions.len() {
        let root = u32::try_from(position).map_err(|_| ProgramError::Structure {
            rule: "host-expression-budget",
        })?;
        values.push(
            abi_evaluate(expressions, root, &facts)
                .map_err(|error| host_expression_error(&error, HostExprId(root)))?,
        );
    }
    Ok(values)
}

/// Maps one shared evaluation failure onto this crate's stable rule vocabulary.
///
/// The match is exhaustive over the wildcard-free arms it can name; the shared
/// error is `#[non_exhaustive]`, so a variant added upstream reaches the final
/// arm and reports `evaluation` rather than being silently reclassified as one
/// of the specific rules.
fn host_expression_error(error: &AbiEvaluationError, at: HostExprId) -> ProgramError {
    let rule = match error {
        AbiEvaluationError::Overflow { .. } => "overflow",
        AbiEvaluationError::UnboundInputExtent { .. }
        | AbiEvaluationError::UnboundTargetProperty { .. } => "operand",
        _ => "evaluation",
    };
    ProgramError::HostExpression {
        rule,
        expression: at,
    }
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
