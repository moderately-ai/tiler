//! Transactional builder for the kernel-program IR.
//!
//! Construction follows the ADR 0071 discipline: a public transactional builder
//! with private storage, insertion-time checks for every locally decidable
//! invariant (handle ownership, interface agreement, storage capacity and
//! alignment, view range, stage/kernel signature agreement, coverage range and
//! disjointness, and governed limits), and a consuming
//! [`KernelProgramBuilder::build`] that runs whole-program verification and
//! returns an opaque [`VerifiedKernelProgram`] or the intact builder with typed
//! diagnostics.
//!
//! A builder is opened against a [`SemanticProgram`], so the semantic subject a
//! program claims to realize — its canonical graph identity, its operation
//! count, and its named interface — is taken from a verified program rather
//! than declared by the producer. Complete coverage and named-output coverage
//! are therefore obligations against an unforgeable subject.

use crate::kernel::{BufferAccess, VerifiedKernel};
use crate::schedule::element_count;
use crate::semantic::{InputKey, OutputKey, SemanticGraphIdentity, SemanticProgram};
use crate::shape::{Axis, Shape};

use super::abi::{
    AbiBinaryOp, AbiFacts, AbiRoot, AbiType, AbiUnaryOp, AbiValue, AvailabilityPhase, ExprNode,
    binary_operand_type, evaluate, expr_key, node_is_interface_only, node_phase, node_type,
    unary_operand_type,
};
use super::error::{
    KernelProgramBuildError, KernelProgramVerificationError, ProgramAbiUse, ProgramEntityKind,
    ProgramLimitKind, invalid_handle,
};
use super::handles::{
    AbiExprId, AllocationId, MaterializedValueId, ProgramBuilderId, StageId, ViewId,
    next_program_builder_id,
};
use super::model::{
    AllocationData, AllocationOwnership, AllocationSpec, ByteWindow, DependencyData,
    DependencyReasonData, KernelProgramData, MaterializedOrigin, MaterializedValueData,
    MaterializedValueSpec, ProgramOutputData, ROUTING_COMMIT_TRANSITIONS, RoutingCommitState,
    RoutingCommitTransition, SemanticOccurrence, StageAccess, StageAccessData, StageAccessMode,
    StageData, StageLaunch, StageLaunchData, ValueRole, VerifiedKernelProgram, ViewData,
    element_bytes, encode_identity,
};
use super::{
    MAX_PROGRAM_ABI_EXPRESSIONS, MAX_PROGRAM_ALLOCATIONS, MAX_PROGRAM_DEPENDENCIES,
    MAX_PROGRAM_OUTPUTS, MAX_PROGRAM_STAGES, MAX_PROGRAM_VALUES, MAX_PROGRAM_VIEWS,
    MAX_STAGE_ACCESSES, MAX_STAGE_COVERAGE,
};

/// The unforgeable semantic subject one kernel program must completely realize.
#[derive(Clone, Debug)]
pub(super) struct SemanticSubject {
    pub(super) graph: SemanticGraphIdentity,
    pub(super) operations: u32,
    inputs: Vec<(InputKey, Shape)>,
    pub(super) outputs: Vec<(OutputKey, Shape)>,
}

/// A transactional kernel-program builder with private storage.
#[derive(Clone, Debug)]
pub struct KernelProgramBuilder {
    owner: ProgramBuilderId,
    subject: SemanticSubject,
    stages: Vec<StageData>,
    values: Vec<MaterializedValueData>,
    views: Vec<ViewData>,
    allocations: Vec<AllocationData>,
    dependencies: Vec<DependencyData>,
    outputs: Vec<ProgramOutputData>,
    covered: Vec<SemanticOccurrence>,
    claimed_inputs: Vec<InputKey>,
    expressions: Vec<ExprNode>,
    expression_keys: Vec<Vec<u8>>,
    expression_types: Vec<AbiType>,
    expression_phases: Vec<AvailabilityPhase>,
    expression_interface_only: Vec<bool>,
    applicability_guard: Option<u32>,
    routing_commit: Vec<RoutingCommitTransition>,
}

impl KernelProgramBuilder {
    /// Opens a builder that must completely realize one verified semantic program.
    ///
    /// # Errors
    ///
    /// Returns [`KernelProgramBuildError::BuilderIdentityExhausted`] when no
    /// fresh builder ownership identity remains, or
    /// [`KernelProgramBuildError::StructuralLimit`] when the semantic program
    /// exceeds the governed coverage space.
    pub fn new(semantic: &SemanticProgram) -> Result<Self, KernelProgramBuildError> {
        let owner =
            next_program_builder_id().ok_or(KernelProgramBuildError::BuilderIdentityExhausted)?;
        let operations = u32::try_from(semantic.operation_count()).map_err(|_| {
            KernelProgramBuildError::StructuralLimit {
                resource: ProgramLimitKind::StageCoverage,
                actual: semantic.operation_count(),
                limit: MAX_STAGE_COVERAGE,
            }
        })?;
        let inputs = interface_input_shapes(semantic);
        let outputs = interface_output_shapes(semantic);
        Ok(Self {
            owner,
            subject: SemanticSubject {
                graph: semantic.semantic_identity().graph().clone(),
                operations,
                inputs,
                outputs,
            },
            stages: Vec::new(),
            values: Vec::new(),
            views: Vec::new(),
            allocations: Vec::new(),
            dependencies: Vec::new(),
            outputs: Vec::new(),
            covered: Vec::new(),
            claimed_inputs: Vec::new(),
            expressions: Vec::new(),
            expression_keys: Vec::new(),
            expression_types: Vec::new(),
            expression_phases: Vec::new(),
            expression_interface_only: Vec::new(),
            applicability_guard: None,
            routing_commit: Vec::new(),
        })
    }

    /// Declares one typed root fact of the program's ABI expression arena.
    ///
    /// The arena is canonically deduplicated by content key: an identical
    /// expression returns the handle already minted for it, so the arena is a
    /// function of what the program says rather than of how often a producer
    /// rebuilt the same formula.
    ///
    /// # Errors
    ///
    /// Returns a structural-limit error.
    pub fn push_abi_root(&mut self, root: AbiRoot) -> Result<AbiExprId, KernelProgramBuildError> {
        self.push_abi_node(ExprNode::Root(root))
    }

    /// Declares one checked unary operation over an existing ABI expression.
    ///
    /// # Errors
    ///
    /// Returns a handle error,
    /// [`KernelProgramBuildError::AbiOperandType`] for a mistyped operand, or a
    /// structural-limit error.
    pub fn push_abi_unary(
        &mut self,
        op: AbiUnaryOp,
        operand: AbiExprId,
    ) -> Result<AbiExprId, KernelProgramBuildError> {
        let operand = self.resolve_expression(operand)?;
        self.expect_abi_type(operand, unary_operand_type(op))?;
        self.push_abi_node(ExprNode::Unary { op, operand })
    }

    /// Declares one checked binary operation over two existing ABI expressions.
    ///
    /// # Errors
    ///
    /// Returns a handle error,
    /// [`KernelProgramBuildError::AbiOperandType`] for a mistyped operand, or a
    /// structural-limit error.
    pub fn push_abi_binary(
        &mut self,
        op: AbiBinaryOp,
        left: AbiExprId,
        right: AbiExprId,
    ) -> Result<AbiExprId, KernelProgramBuildError> {
        let left = self.resolve_expression(left)?;
        let right = self.resolve_expression(right)?;
        self.expect_abi_type(left, binary_operand_type(op))?;
        self.expect_abi_type(right, binary_operand_type(op))?;
        self.push_abi_node(ExprNode::Binary { op, left, right })
    }

    /// Declares one conditional selection between two equally typed branches.
    ///
    /// Only the selected branch is evaluated, so a branch that would fail on a
    /// zero-sized bound is legal behind a guarding condition.
    ///
    /// # Errors
    ///
    /// Returns a handle error,
    /// [`KernelProgramBuildError::AbiOperandType`] for a non-predicate
    /// condition, [`KernelProgramBuildError::AbiSelectBranchType`] for
    /// disagreeing branches, or a structural-limit error.
    pub fn push_abi_select(
        &mut self,
        condition: AbiExprId,
        if_true: AbiExprId,
        if_false: AbiExprId,
    ) -> Result<AbiExprId, KernelProgramBuildError> {
        let condition = self.resolve_expression(condition)?;
        let if_true = self.resolve_expression(if_true)?;
        let if_false = self.resolve_expression(if_false)?;
        self.expect_abi_type(condition, AbiType::Boolean)?;
        let (left, right) = (
            self.expression_types[as_position(if_true)],
            self.expression_types[as_position(if_false)],
        );
        if left != right {
            return Err(KernelProgramBuildError::AbiSelectBranchType {
                if_true: left,
                if_false: right,
            });
        }
        self.push_abi_node(ExprNode::Select {
            condition,
            if_true,
            if_false,
        })
    }

    /// Declares the guard deciding whether this program may be routed to.
    ///
    /// The guard is a predicate over facts available no later than live-device
    /// preflight, because routing commit happens after every phase in that
    /// order: a guard that could only be read afterwards could not decide
    /// anything.
    ///
    /// # Errors
    ///
    /// Returns a handle error,
    /// [`KernelProgramBuildError::DuplicateApplicabilityGuard`],
    /// [`KernelProgramBuildError::AbiUseType`], or
    /// [`KernelProgramBuildError::AbiRootPhaseEscape`].
    pub fn applicability_guard(&mut self, guard: AbiExprId) -> Result<(), KernelProgramBuildError> {
        if self.applicability_guard.is_some() {
            return Err(KernelProgramBuildError::DuplicateApplicabilityGuard);
        }
        let node = self.check_abi_use(
            guard,
            ProgramAbiUse::ApplicabilityGuard,
            AbiType::Boolean,
            false,
        )?;
        self.applicability_guard = Some(node);
        Ok(())
    }

    /// Declares the next step of the program's routing-commit lifecycle.
    ///
    /// Steps are declared in lifecycle order from
    /// [`RoutingCommitState::Preflight`]. A step is admitted only when it
    /// leaves the state the previous one reached, and only the step leaving
    /// `Preflight` may permit fallback — after commit the program owns work a
    /// fallback would have to undo.
    ///
    /// # Errors
    ///
    /// Returns [`KernelProgramBuildError::RoutingCommitOutOfOrder`],
    /// [`KernelProgramBuildError::RoutingCommitFallbackAfterCommit`], or a
    /// structural-limit error.
    pub fn push_routing_commit_transition(
        &mut self,
        transition: RoutingCommitTransition,
    ) -> Result<(), KernelProgramBuildError> {
        limit(
            self.routing_commit.len().saturating_add(1),
            ROUTING_COMMIT_TRANSITIONS,
            ProgramLimitKind::RoutingCommitTransitions,
        )?;
        let expected_from = self
            .routing_commit
            .last()
            .map_or(RoutingCommitState::Preflight, |previous| previous.to);
        let expected_to = expected_from.next();
        if transition.from != expected_from || Some(transition.to) != expected_to {
            return Err(KernelProgramBuildError::RoutingCommitOutOfOrder {
                expected: expected_from,
                actual: transition.from,
            });
        }
        if transition.fallback_permitted && transition.from != RoutingCommitState::Preflight {
            return Err(KernelProgramBuildError::RoutingCommitFallbackAfterCommit {
                from: transition.from,
            });
        }
        self.routing_commit.push(transition);
        Ok(())
    }

    /// Declares one program storage allocation.
    ///
    /// # Errors
    ///
    /// Returns [`KernelProgramBuildError::InvalidAlignment`] for a zero or
    /// non-power-of-two alignment, or a structural-limit error.
    pub fn push_allocation(
        &mut self,
        spec: AllocationSpec,
    ) -> Result<AllocationId, KernelProgramBuildError> {
        check_alignment(spec.alignment)?;
        limit(
            self.allocations.len().saturating_add(1),
            MAX_PROGRAM_ALLOCATIONS,
            ProgramLimitKind::Allocations,
        )?;
        let id = AllocationId::from_len(self.owner, self.allocations.len()).ok_or(
            KernelProgramBuildError::StructuralLimit {
                resource: ProgramLimitKind::Allocations,
                actual: self.allocations.len().saturating_add(1),
                limit: MAX_PROGRAM_ALLOCATIONS,
            },
        )?;
        self.allocations.push(AllocationData {
            capacity_bytes: spec.capacity_bytes,
            alignment: spec.alignment,
            memory_space: spec.memory_space,
            ownership: spec.ownership,
        });
        Ok(id)
    }

    /// Declares one materialized program value bound to an allocation.
    ///
    /// The required byte count is derived from the declared shape and element
    /// type; the allocation must provide at least that capacity, a compatible
    /// alignment, the same memory domain, and ownership matching the role.
    ///
    /// # Errors
    ///
    /// Returns a handle error, an interface disagreement, an alignment,
    /// capacity, memory-space, or ownership violation, an overflow, or a
    /// structural-limit error.
    pub fn push_value(
        &mut self,
        spec: MaterializedValueSpec,
        allocation: AllocationId,
    ) -> Result<MaterializedValueId, KernelProgramBuildError> {
        let storage = self.resolve_allocation(allocation)?;
        check_alignment(spec.alignment)?;
        self.check_origin(&spec)?;
        let elements = element_count(&spec.shape)
            .map_err(|_| KernelProgramBuildError::ElementCountOverflow)?;
        let required_bytes = elements
            .checked_mul(element_bytes(spec.element_type))
            .ok_or(KernelProgramBuildError::ElementCountOverflow)?;
        if storage.memory_space != spec.memory_space {
            return Err(KernelProgramBuildError::AllocationMemorySpace {
                required: spec.memory_space,
                provided: storage.memory_space,
            });
        }
        if storage.alignment % spec.alignment != 0 {
            return Err(KernelProgramBuildError::AllocationAlignment {
                required: spec.alignment,
                provided: storage.alignment,
            });
        }
        if storage.capacity_bytes < required_bytes {
            return Err(KernelProgramBuildError::AllocationCapacity {
                required: required_bytes,
                capacity: storage.capacity_bytes,
            });
        }
        let external = storage.ownership == AllocationOwnership::External;
        if external != (spec.role == ValueRole::Input) {
            return Err(KernelProgramBuildError::AllocationOwnershipRole {
                ownership: storage.ownership,
                role: spec.role,
            });
        }
        limit(
            self.values.len().saturating_add(1),
            MAX_PROGRAM_VALUES,
            ProgramLimitKind::Values,
        )?;
        let id = MaterializedValueId::from_len(self.owner, self.values.len()).ok_or(
            KernelProgramBuildError::StructuralLimit {
                resource: ProgramLimitKind::Values,
                actual: self.values.len().saturating_add(1),
                limit: MAX_PROGRAM_VALUES,
            },
        )?;
        if let MaterializedOrigin::ProgramInput { key } = &spec.origin {
            self.claimed_inputs.push(key.clone());
        }
        self.values.push(MaterializedValueData {
            origin: spec.origin,
            role: spec.role,
            shape: spec.shape,
            element_type: spec.element_type,
            required_bytes,
            alignment: spec.alignment,
            memory_space: spec.memory_space,
            allocation: allocation.index,
        });
        Ok(id)
    }

    /// Declares one byte view through which stages address a value.
    ///
    /// Views are canonically deduplicated: two stages addressing the same
    /// window of the same value share one view.
    ///
    /// # Errors
    ///
    /// Returns a handle error, [`KernelProgramBuildError::ViewOutOfRange`],
    /// [`KernelProgramBuildError::DuplicateView`], or a structural-limit error.
    pub fn push_view(
        &mut self,
        value: MaterializedValueId,
        window: ByteWindow,
    ) -> Result<ViewId, KernelProgramBuildError> {
        let base = self.resolve_value(value)?;
        let end = window
            .offset
            .checked_add(window.length)
            .ok_or(KernelProgramBuildError::ElementCountOverflow)?;
        if end > base.required_bytes {
            return Err(KernelProgramBuildError::ViewOutOfRange {
                offset: window.offset,
                length: window.length,
                value_bytes: base.required_bytes,
            });
        }
        if self
            .views
            .iter()
            .any(|view| view.value == value.index && view.window == window)
        {
            return Err(KernelProgramBuildError::DuplicateView);
        }
        limit(
            self.views.len().saturating_add(1),
            MAX_PROGRAM_VIEWS,
            ProgramLimitKind::Views,
        )?;
        let id = ViewId::from_len(self.owner, self.views.len()).ok_or(
            KernelProgramBuildError::StructuralLimit {
                resource: ProgramLimitKind::Views,
                actual: self.views.len().saturating_add(1),
                limit: MAX_PROGRAM_VIEWS,
            },
        )?;
        self.views.push(ViewData {
            value: value.index,
            window,
        });
        Ok(id)
    }

    /// Declares the whole byte range of one value as a view.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`KernelProgramBuilder::push_view`].
    pub fn push_whole_view(
        &mut self,
        value: MaterializedValueId,
    ) -> Result<ViewId, KernelProgramBuildError> {
        let length = self.resolve_value(value)?.required_bytes;
        self.push_view(value, ByteWindow { offset: 0, length })
    }

    /// Declares one stage dispatching an exact verified structured kernel.
    ///
    /// The stage's accesses realize the kernel's buffer parameters in order:
    /// each access must match its buffer's access mode, tensor role, element
    /// type, and addressed element count, and must state an ABI expression
    /// computing exactly the byte count its own view addresses. The launch
    /// geometry must likewise state a workgroup width the bound kernel
    /// requires. The covered occurrences are the exact operations of the bound
    /// semantic program this stage implements.
    ///
    /// # Errors
    ///
    /// Returns a handle error, a coverage range or disjointness violation, a
    /// stage/kernel signature disagreement, an ABI use-site type, phase,
    /// interface-root, evaluation, accessible-range, or workgroup-width
    /// rejection, or a structural-limit error.
    pub fn push_stage(
        &mut self,
        kernel: &VerifiedKernel,
        coverage: &[SemanticOccurrence],
        accesses: &[StageAccess],
        launch: StageLaunch,
    ) -> Result<StageId, KernelProgramBuildError> {
        // Handle ownership and signature agreement are checked before coverage
        // so a forged or foreign handle names itself rather than being masked
        // by whichever occurrence the caller happened to claim.
        let accesses = self.check_stage_accesses(kernel, accesses)?;
        let launch = self.check_stage_launch(kernel, launch)?;
        let coverage = self.check_coverage(coverage)?;
        limit(
            self.stages.len().saturating_add(1),
            MAX_PROGRAM_STAGES,
            ProgramLimitKind::Stages,
        )?;
        let id = StageId::from_len(self.owner, self.stages.len()).ok_or(
            KernelProgramBuildError::StructuralLimit {
                resource: ProgramLimitKind::Stages,
                actual: self.stages.len().saturating_add(1),
                limit: MAX_PROGRAM_STAGES,
            },
        )?;
        self.covered.extend_from_slice(&coverage);
        self.stages.push(StageData {
            kernel: kernel.clone(),
            coverage,
            accesses,
            launch,
        });
        Ok(id)
    }

    /// Declares that `successor` reads a value `predecessor` defines.
    ///
    /// # Errors
    ///
    /// Returns a handle error,
    /// [`KernelProgramBuildError::SelfDependency`],
    /// [`KernelProgramBuildError::DuplicateDependency`], or a structural-limit
    /// error.
    pub fn push_data_dependency(
        &mut self,
        predecessor: StageId,
        successor: StageId,
        value: MaterializedValueId,
    ) -> Result<(), KernelProgramBuildError> {
        self.resolve_value(value)?;
        self.push_dependency(
            predecessor,
            successor,
            DependencyReasonData::Data(value.index),
        )
    }

    /// Declares that `successor` reuses storage `predecessor` released.
    ///
    /// # Errors
    ///
    /// Returns a handle error,
    /// [`KernelProgramBuildError::SelfDependency`],
    /// [`KernelProgramBuildError::DuplicateDependency`], or a structural-limit
    /// error.
    pub fn push_storage_handoff(
        &mut self,
        predecessor: StageId,
        successor: StageId,
        allocation: AllocationId,
    ) -> Result<(), KernelProgramBuildError> {
        self.resolve_allocation(allocation)?;
        self.push_dependency(
            predecessor,
            successor,
            DependencyReasonData::StorageHandoff(allocation.index),
        )
    }

    /// Publishes one materialized value under a named semantic output key.
    ///
    /// # Errors
    ///
    /// Returns a handle error,
    /// [`KernelProgramBuildError::UnknownOutputKey`],
    /// [`KernelProgramBuildError::DuplicateOutput`],
    /// [`KernelProgramBuildError::OutputValueRole`],
    /// [`KernelProgramBuildError::InterfaceShapeMismatch`], or a
    /// structural-limit error.
    pub fn push_output(
        &mut self,
        key: OutputKey,
        value: MaterializedValueId,
    ) -> Result<(), KernelProgramBuildError> {
        let published = self.resolve_value(value)?.clone();
        let Some((_, shape)) = self
            .subject
            .outputs
            .iter()
            .find(|(declared, _)| *declared == key)
        else {
            return Err(KernelProgramBuildError::UnknownOutputKey { key });
        };
        if self.outputs.iter().any(|output| output.key == key) {
            return Err(KernelProgramBuildError::DuplicateOutput { key });
        }
        if published.role != ValueRole::Output {
            return Err(KernelProgramBuildError::OutputValueRole {
                role: published.role,
            });
        }
        if published.shape != *shape {
            return Err(KernelProgramBuildError::InterfaceShapeMismatch {
                entity: ProgramEntityKind::Value,
            });
        }
        limit(
            self.outputs.len().saturating_add(1),
            MAX_PROGRAM_OUTPUTS,
            ProgramLimitKind::Outputs,
        )?;
        self.outputs.push(ProgramOutputData {
            key,
            value: value.index,
        });
        Ok(())
    }

    /// Verifies the whole program and freezes it, or returns the intact builder.
    ///
    /// # Errors
    ///
    /// Returns a [`KernelProgramVerificationError`] carrying every whole-program
    /// diagnostic and the recoverable builder when verification fails.
    pub fn build(self) -> Result<VerifiedKernelProgram, KernelProgramVerificationError> {
        let data = self.assemble();
        match super::verify::verify_program(&data, &self.subject)
            .and_then(|(derived, keys)| Ok((derived, encode_identity(&data, &keys)?)))
        {
            Ok((derived, identity)) => Ok(VerifiedKernelProgram {
                data,
                derived,
                identity,
            }),
            Err(diagnostic) => Err(KernelProgramVerificationError {
                builder: Box::new(self),
                diagnostics: vec![diagnostic],
            }),
        }
    }

    fn assemble(&self) -> KernelProgramData {
        KernelProgramData {
            semantic_graph: self.subject.graph.clone(),
            stages: self.stages.clone(),
            values: self.values.clone(),
            views: self.views.clone(),
            allocations: self.allocations.clone(),
            dependencies: self.dependencies.clone(),
            outputs: self.outputs.clone(),
            abi_expressions: self.expressions.clone(),
            applicability_guard: self.applicability_guard,
            routing_commit: self.routing_commit.clone(),
        }
    }

    /// Interns one ABI arena node, returning the handle of an equal earlier one.
    fn push_abi_node(&mut self, node: ExprNode) -> Result<AbiExprId, KernelProgramBuildError> {
        let key = expr_key(&node, &self.expression_keys);
        if let Some(existing) = self.expression_keys.iter().position(|held| *held == key) {
            return AbiExprId::from_len(self.owner, existing).ok_or(
                KernelProgramBuildError::StructuralLimit {
                    resource: ProgramLimitKind::AbiExpressions,
                    actual: existing,
                    limit: MAX_PROGRAM_ABI_EXPRESSIONS,
                },
            );
        }
        limit(
            self.expressions.len().saturating_add(1),
            MAX_PROGRAM_ABI_EXPRESSIONS,
            ProgramLimitKind::AbiExpressions,
        )?;
        let id = AbiExprId::from_len(self.owner, self.expressions.len()).ok_or(
            KernelProgramBuildError::StructuralLimit {
                resource: ProgramLimitKind::AbiExpressions,
                actual: self.expressions.len().saturating_add(1),
                limit: MAX_PROGRAM_ABI_EXPRESSIONS,
            },
        )?;
        self.expression_types
            .push(node_type(&node, &self.expression_types));
        self.expression_phases
            .push(node_phase(&node, &self.expression_phases));
        self.expression_interface_only.push(node_is_interface_only(
            &node,
            &self.expression_interface_only,
        ));
        self.expression_keys.push(key);
        self.expressions.push(node);
        Ok(id)
    }

    fn expect_abi_type(&self, node: u32, expected: AbiType) -> Result<(), KernelProgramBuildError> {
        let actual = self.expression_types[as_position(node)];
        if actual == expected {
            Ok(())
        } else {
            Err(KernelProgramBuildError::AbiOperandType { expected, actual })
        }
    }

    /// Resolves one expression handle against a declared program use site.
    ///
    /// Every program use site admits roots through
    /// [`AvailabilityPhase::LiveDevicePreflight`], the last phase before
    /// routing commit. `interface_only` additionally forbids target and device
    /// properties, so an accessible range or a launch geometry can be computed
    /// from the bound interface before any device-dependent query.
    fn check_abi_use(
        &self,
        id: AbiExprId,
        use_site: ProgramAbiUse,
        expected: AbiType,
        interface_only: bool,
    ) -> Result<u32, KernelProgramBuildError> {
        let node = self.resolve_expression(id)?;
        let actual = self.expression_types[as_position(node)];
        if actual != expected {
            return Err(KernelProgramBuildError::AbiUseType {
                use_site,
                expected,
                actual,
            });
        }
        let available_at = self.expression_phases[as_position(node)];
        if available_at > AvailabilityPhase::LiveDevicePreflight {
            return Err(KernelProgramBuildError::AbiRootPhaseEscape {
                use_site,
                available_at,
                admitted_through: AvailabilityPhase::LiveDevicePreflight,
            });
        }
        if interface_only && !self.expression_interface_only[as_position(node)] {
            return Err(KernelProgramBuildError::AbiNonInterfaceRoot { use_site });
        }
        Ok(node)
    }

    /// Evaluates one interface-only expression against the program's own shapes.
    ///
    /// This is a compile-time consistency check, not a runtime evaluation. The
    /// facts are the bound semantic program's declared input extents, which the
    /// static-shape profile already knows, so a producer cannot declare an
    /// accessible range or a launch geometry that its own program contradicts.
    /// The phase the environment claims is `LiveDevicePreflight` because that
    /// is the phase at which an `InputExtent` root becomes readable in general;
    /// here the same values happen to be known earlier.
    fn evaluate_static_abi(
        &self,
        node: u32,
        use_site: ProgramAbiUse,
    ) -> Result<u64, KernelProgramBuildError> {
        match evaluate(&self.expressions, node, &self.static_facts()) {
            Ok(AbiValue::Unsigned(value)) => Ok(value),
            // Unreachable through the use sites that call this, each of which
            // has already required `AbiType::Unsigned`; reported rather than
            // asserted so a future unsigned-typed use site cannot reach a panic.
            Ok(AbiValue::Boolean(_)) => Err(KernelProgramBuildError::AbiUseType {
                use_site,
                expected: AbiType::Unsigned,
                actual: AbiType::Boolean,
            }),
            Err(cause) => Err(KernelProgramBuildError::AbiStaticEvaluation { use_site, cause }),
        }
    }

    /// Binds the bound semantic program's declared input extents as ABI facts.
    fn static_facts(&self) -> AbiFacts {
        let mut extents = Vec::new();
        for (key, shape) in &self.subject.inputs {
            for (axis, extent) in shape.extents().iter().enumerate() {
                let axis = u32::try_from(axis).expect("a governed shape rank fits u32");
                extents.push((key.clone(), Axis::new(axis), extent.get()));
            }
        }
        AbiFacts::new(AvailabilityPhase::LiveDevicePreflight, extents, Vec::new())
    }

    fn push_dependency(
        &mut self,
        predecessor: StageId,
        successor: StageId,
        reason: DependencyReasonData,
    ) -> Result<(), KernelProgramBuildError> {
        self.resolve_stage(predecessor)?;
        self.resolve_stage(successor)?;
        if predecessor.index == successor.index {
            return Err(KernelProgramBuildError::SelfDependency);
        }
        let edge = DependencyData {
            predecessor: predecessor.index,
            successor: successor.index,
            reason,
        };
        if self.dependencies.contains(&edge) {
            return Err(KernelProgramBuildError::DuplicateDependency);
        }
        limit(
            self.dependencies.len().saturating_add(1),
            MAX_PROGRAM_DEPENDENCIES,
            ProgramLimitKind::Dependencies,
        )?;
        self.dependencies.push(edge);
        Ok(())
    }

    fn check_origin(&self, spec: &MaterializedValueSpec) -> Result<(), KernelProgramBuildError> {
        match (&spec.origin, spec.role) {
            (MaterializedOrigin::ProgramInput { key }, ValueRole::Input) => {
                let Some((_, shape)) = self
                    .subject
                    .inputs
                    .iter()
                    .find(|(declared, _)| declared == key)
                else {
                    return Err(KernelProgramBuildError::UnknownProgramInput { key: key.clone() });
                };
                if self.claimed_inputs.contains(key) {
                    return Err(KernelProgramBuildError::DuplicateProgramInput {
                        key: key.clone(),
                    });
                }
                if spec.shape != *shape {
                    return Err(KernelProgramBuildError::InterfaceShapeMismatch {
                        entity: ProgramEntityKind::Value,
                    });
                }
                Ok(())
            }
            (MaterializedOrigin::Internal, ValueRole::Temporary | ValueRole::Output) => Ok(()),
            (MaterializedOrigin::ProgramInput { .. } | MaterializedOrigin::Internal, role) => {
                Err(KernelProgramBuildError::ValueRoleOrigin { role })
            }
        }
    }

    fn check_coverage(
        &self,
        coverage: &[SemanticOccurrence],
    ) -> Result<Vec<SemanticOccurrence>, KernelProgramBuildError> {
        if coverage.is_empty() {
            return Err(KernelProgramBuildError::EmptyCoverage);
        }
        limit(
            coverage.len(),
            MAX_STAGE_COVERAGE,
            ProgramLimitKind::StageCoverage,
        )?;
        let mut ordered = coverage.to_vec();
        ordered.sort_unstable();
        for (position, occurrence) in ordered.iter().enumerate() {
            if occurrence.get() >= self.subject.operations {
                return Err(KernelProgramBuildError::CoverageOutOfRange {
                    occurrence: *occurrence,
                    operations: self.subject.operations,
                });
            }
            if ordered.get(position.saturating_add(1)) == Some(occurrence)
                || self.covered.contains(occurrence)
            {
                return Err(KernelProgramBuildError::DuplicateCoverage {
                    occurrence: *occurrence,
                });
            }
        }
        Ok(ordered)
    }

    fn check_stage_accesses(
        &self,
        kernel: &VerifiedKernel,
        accesses: &[StageAccess],
    ) -> Result<Vec<StageAccessData>, KernelProgramBuildError> {
        let buffers: Vec<_> = kernel.buffers().collect();
        if buffers.len() != accesses.len() {
            return Err(KernelProgramBuildError::StageAccessArity {
                expected: buffers.len(),
                actual: accesses.len(),
            });
        }
        limit(
            accesses.len(),
            MAX_STAGE_ACCESSES,
            ProgramLimitKind::StageAccesses,
        )?;
        let mut resolved = Vec::with_capacity(accesses.len());
        for (position, (access, buffer)) in accesses.iter().zip(&buffers).enumerate() {
            let view = self.resolve_view(access.view)?;
            let value = self.view_base(view);
            let expected_mode = match buffer.access {
                BufferAccess::Read => StageAccessMode::Read,
                BufferAccess::Write => StageAccessMode::Write,
            };
            if expected_mode != access.mode {
                return Err(KernelProgramBuildError::StageAccessMode {
                    position,
                    expected: expected_mode,
                    actual: access.mode,
                });
            }
            if buffer.tensor != value.role.tensor_role() {
                return Err(KernelProgramBuildError::StageTensorRole {
                    position,
                    expected: buffer.tensor,
                    actual: value.role.tensor_role(),
                });
            }
            if buffer.element_type != value.element_type {
                return Err(KernelProgramBuildError::StageElementType {
                    position,
                    expected: buffer.element_type,
                    actual: value.element_type,
                });
            }
            let expected_bytes = buffer
                .element_count
                .checked_mul(element_bytes(value.element_type))
                .ok_or(KernelProgramBuildError::ElementCountOverflow)?;
            if expected_bytes != view.window.length {
                return Err(KernelProgramBuildError::StageElementCount {
                    position,
                    expected: buffer.element_count,
                    actual: view.window.length / element_bytes(value.element_type),
                });
            }
            let accessible_bytes = self.check_abi_use(
                access.accessible_bytes,
                ProgramAbiUse::AccessibleBytes,
                AbiType::Unsigned,
                true,
            )?;
            let computed =
                self.evaluate_static_abi(accessible_bytes, ProgramAbiUse::AccessibleBytes)?;
            if computed != view.window.length {
                return Err(KernelProgramBuildError::AccessibleBytesDisagreement {
                    position,
                    expected: view.window.length,
                    actual: computed,
                });
            }
            resolved.push(StageAccessData {
                view: access.view.index,
                mode: access.mode,
                accessible_bytes,
            });
        }
        Ok(resolved)
    }

    /// Proves one stage's declared launch geometry realizes its bound kernel.
    ///
    /// Only the workgroup width has a kernel-side counterpart to check against:
    /// a verified kernel states the width its body requires in its resource
    /// requirements, while the grid extent is a property of the launch and not
    /// of the kernel. So the width is proven equal and the grid extent is
    /// proven well-typed, phase-legal, interface-only, and evaluable — never
    /// approximated against a number the kernel does not carry.
    fn check_stage_launch(
        &self,
        kernel: &VerifiedKernel,
        launch: StageLaunch,
    ) -> Result<StageLaunchData, KernelProgramBuildError> {
        let grid_threads = self.check_abi_use(
            launch.grid_threads,
            ProgramAbiUse::GridThreads,
            AbiType::Unsigned,
            true,
        )?;
        self.evaluate_static_abi(grid_threads, ProgramAbiUse::GridThreads)?;
        let threads_per_workgroup = self.check_abi_use(
            launch.threads_per_workgroup,
            ProgramAbiUse::ThreadsPerWorkgroup,
            AbiType::Unsigned,
            true,
        )?;
        let declared =
            self.evaluate_static_abi(threads_per_workgroup, ProgramAbiUse::ThreadsPerWorkgroup)?;
        let required = u64::from(kernel.requirements().threads_per_workgroup);
        if declared != required {
            return Err(KernelProgramBuildError::ThreadsPerWorkgroupDisagreement {
                expected: required,
                actual: declared,
            });
        }
        Ok(StageLaunchData {
            grid_threads,
            threads_per_workgroup,
        })
    }

    /// Returns the value a declared view addresses.
    ///
    /// A view is only ever created from a resolved value handle, so its base
    /// index is a builder invariant rather than caller input.
    fn view_base(&self, view: ViewData) -> &MaterializedValueData {
        let index = usize::try_from(view.value).expect("u32 fits every supported host usize");
        self.values
            .get(index)
            .expect("a declared view addresses a declared value")
    }

    fn resolve_stage(&self, id: StageId) -> Result<&StageData, KernelProgramBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ProgramEntityKind::Stage, true));
        }
        self.stages
            .get(id.as_usize())
            .ok_or_else(|| invalid_handle(ProgramEntityKind::Stage, false))
    }

    fn resolve_value(
        &self,
        id: MaterializedValueId,
    ) -> Result<&MaterializedValueData, KernelProgramBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ProgramEntityKind::Value, true));
        }
        self.values
            .get(id.as_usize())
            .ok_or_else(|| invalid_handle(ProgramEntityKind::Value, false))
    }

    fn resolve_view(&self, id: ViewId) -> Result<ViewData, KernelProgramBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ProgramEntityKind::View, true));
        }
        self.views
            .get(id.as_usize())
            .copied()
            .ok_or_else(|| invalid_handle(ProgramEntityKind::View, false))
    }

    fn resolve_expression(&self, id: AbiExprId) -> Result<u32, KernelProgramBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ProgramEntityKind::AbiExpression, true));
        }
        if id.as_usize() >= self.expressions.len() {
            return Err(invalid_handle(ProgramEntityKind::AbiExpression, false));
        }
        Ok(id.index)
    }

    fn resolve_allocation(
        &self,
        id: AllocationId,
    ) -> Result<AllocationData, KernelProgramBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(ProgramEntityKind::Allocation, true));
        }
        self.allocations
            .get(id.as_usize())
            .copied()
            .ok_or_else(|| invalid_handle(ProgramEntityKind::Allocation, false))
    }
}

/// Reads the ordered input interface of a verified semantic program.
///
/// A verified program always resolves its own interface values, so the lookup
/// is a program invariant rather than caller input.
fn interface_input_shapes(semantic: &SemanticProgram) -> Vec<(InputKey, Shape)> {
    semantic
        .inputs()
        .map(|input| {
            let shape = semantic
                .shape(input.value())
                .expect("a verified program resolves its own input value")
                .clone();
            (input.key().clone(), shape)
        })
        .collect()
}

/// Reads the ordered output interface of a verified semantic program.
fn interface_output_shapes(semantic: &SemanticProgram) -> Vec<(OutputKey, Shape)> {
    semantic
        .outputs()
        .map(|output| {
            let shape = semantic
                .shape(output.value())
                .expect("a verified program resolves its own output value")
                .clone();
            (output.key().clone(), shape)
        })
        .collect()
}

/// Converts a checked arena ordinal into a host index.
fn as_position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

fn check_alignment(alignment: u32) -> Result<(), KernelProgramBuildError> {
    if alignment == 0 || !alignment.is_power_of_two() {
        return Err(KernelProgramBuildError::InvalidAlignment { alignment });
    }
    Ok(())
}

fn limit(
    actual: usize,
    limit: usize,
    resource: ProgramLimitKind,
) -> Result<(), KernelProgramBuildError> {
    if actual > limit {
        return Err(KernelProgramBuildError::StructuralLimit {
            resource,
            actual,
            limit,
        });
    }
    Ok(())
}
