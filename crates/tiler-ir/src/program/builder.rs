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
use crate::shape::Shape;

use super::error::{
    KernelProgramBuildError, KernelProgramVerificationError, ProgramEntityKind, ProgramLimitKind,
    invalid_handle,
};
use super::handles::{
    AllocationId, MaterializedValueId, ProgramBuilderId, StageId, ViewId, next_program_builder_id,
};
use super::model::{
    AllocationData, AllocationOwnership, AllocationSpec, ByteWindow, DependencyData,
    DependencyReasonData, KernelProgramData, MaterializedOrigin, MaterializedValueData,
    MaterializedValueSpec, ProgramOutputData, SemanticOccurrence, StageAccess, StageAccessData,
    StageAccessMode, StageData, ValueRole, VerifiedKernelProgram, ViewData, element_bytes,
    encode_identity,
};
use super::{
    MAX_PROGRAM_ALLOCATIONS, MAX_PROGRAM_DEPENDENCIES, MAX_PROGRAM_OUTPUTS, MAX_PROGRAM_STAGES,
    MAX_PROGRAM_VALUES, MAX_PROGRAM_VIEWS, MAX_STAGE_ACCESSES, MAX_STAGE_COVERAGE,
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
        })
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
    /// type, and addressed element count. The covered occurrences are the exact
    /// operations of the bound semantic program this stage implements.
    ///
    /// # Errors
    ///
    /// Returns a handle error, a coverage range or disjointness violation, a
    /// stage/kernel signature disagreement, or a structural-limit error.
    pub fn push_stage(
        &mut self,
        kernel: &VerifiedKernel,
        coverage: &[SemanticOccurrence],
        accesses: &[StageAccess],
    ) -> Result<StageId, KernelProgramBuildError> {
        // Handle ownership and signature agreement are checked before coverage
        // so a forged or foreign handle names itself rather than being masked
        // by whichever occurrence the caller happened to claim.
        let accesses = self.check_stage_accesses(kernel, accesses)?;
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
        }
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
            resolved.push(StageAccessData {
                view: access.view.index,
                mode: access.mode,
            });
        }
        Ok(resolved)
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
