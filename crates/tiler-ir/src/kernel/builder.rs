//! Transactional builder for the structured kernel IR.
//!
//! Construction follows the ADR 0071 discipline: a public transactional builder
//! with private storage, insertion-time checks for every locally decidable
//! invariant (ownership, scope, type, buffer access mode, admitted builtins,
//! loop shape, and governed limits), and a consuming
//! [`KernelBuilder::build`] that runs whole-kernel verification and returns an
//! opaque [`VerifiedKernel`] or the intact builder with typed diagnostics.
//!
//! A builder is opened against a [`VerifiedScheduledRegion`], so a kernel can
//! only ever exist as a refinement of an already intrinsically verified
//! schedule. There is no constructor that accepts an unverified region.

use crate::schedule::{BoundsWitnessId, OwnershipWitnessId, PhaseId};
use crate::schedule::{
    CanonicalScheduledRegionIdentity, LogicalAccess, NumericalRealization, ReductionTopology,
    RegionId, ResourceRequirements, ScheduledRegion, TensorRole, VerifiedScheduledRegion,
    subnormal_freedom_of,
};

use super::error::{
    KernelBuildError, KernelComponent, KernelDiagnostic, KernelEntityKind, KernelLimitKind,
    KernelVerificationError, invalid_handle,
};
use super::handles::{
    KernelBufferId, KernelBuilderId, KernelInputExtentId, KernelStagingId, KernelValueId,
    next_kernel_builder_id,
};
use super::model::{
    BarrierSpec, BinaryOp, BlockData, BufferAccess, BufferParameter, Builtin, CompareOp, ConvertOp,
    InputExtentParameter, KernelConstant, KernelData, KernelType, OperationData, OperationKind,
    PackedExtractOp, SerialLoopSpec, StagingParameter, UnaryOp, ValueData, VerifiedKernel,
    encode_identity,
};
use super::{
    MAX_KERNEL_ADMITTED_BUILTINS, MAX_KERNEL_BLOCK_DEPTH, MAX_KERNEL_BLOCKS, MAX_KERNEL_BUFFERS,
    MAX_KERNEL_INPUT_EXTENTS, MAX_KERNEL_LOOP_ACCUMULATORS, MAX_KERNEL_OPERATIONS,
    MAX_KERNEL_STAGING, MAX_KERNEL_VALUES,
};

/// The induction variable and accumulator parameters of one structured loop.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerialLoopParameters {
    induction: KernelValueId,
    accumulators: Vec<KernelValueId>,
}

impl SerialLoopParameters {
    /// Returns the loop induction variable.
    #[must_use]
    pub const fn induction(&self) -> KernelValueId {
        self.induction
    }

    /// Returns one ordered accumulator parameter.
    #[must_use]
    pub fn accumulator(&self, index: usize) -> Option<KernelValueId> {
        self.accumulators.get(index).copied()
    }

    /// Returns the number of carried accumulators.
    #[must_use]
    pub fn len(&self) -> usize {
        self.accumulators.len()
    }

    /// Returns whether the loop carries no accumulator.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.accumulators.is_empty()
    }
}

/// The ordered results one structured loop defines in its enclosing block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerialLoopResults(Vec<KernelValueId>);

impl SerialLoopResults {
    /// Returns one ordered loop result.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<KernelValueId> {
        self.0.get(index).copied()
    }

    /// Returns the number of loop results.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the loop defines no result.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A transactional structured-kernel builder with private storage.
#[derive(Clone, Debug)]
pub struct KernelBuilder {
    owner: KernelBuilderId,
    region: RegionId,
    schedule: ScheduledRegion,
    schedule_identity: CanonicalScheduledRegionIdentity,
    derived_requirements: ResourceRequirements,
    buffers: Vec<BufferParameter>,
    staging: Vec<StagingParameter>,
    input_extents: Vec<InputExtentParameter>,
    admitted_builtins: Vec<Builtin>,
    numerical: Option<NumericalRealization>,
    requirements: Option<ResourceRequirements>,
    values: Vec<ValueData>,
    blocks: Vec<BlockData>,
    open: Vec<u32>,
    operations: usize,
}

/// Lengths restored when a nested-block insertion fails.
#[derive(Clone, Copy, Debug)]
struct Checkpoint {
    buffers: usize,
    staging: usize,
    input_extents: usize,
    admitted_builtins: usize,
    values: usize,
    blocks: usize,
    open: usize,
    operations: usize,
}

impl KernelBuilder {
    /// Opens a builder refining one verified scheduled region.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::BuilderIdentityExhausted`] when no fresh
    /// builder ownership identity remains.
    pub fn new(scheduled: &VerifiedScheduledRegion) -> Result<Self, KernelBuildError> {
        Self::from_parts(
            scheduled.region().clone(),
            scheduled.canonical_identity().clone(),
            scheduled.requirements(),
        )
    }

    pub(super) fn from_parts(
        schedule: ScheduledRegion,
        schedule_identity: CanonicalScheduledRegionIdentity,
        derived_requirements: ResourceRequirements,
    ) -> Result<Self, KernelBuildError> {
        let owner = next_kernel_builder_id().ok_or(KernelBuildError::BuilderIdentityExhausted)?;
        Ok(Self {
            owner,
            region: schedule.index.id,
            schedule,
            schedule_identity,
            derived_requirements,
            buffers: Vec::new(),
            staging: Vec::new(),
            input_extents: Vec::new(),
            admitted_builtins: Vec::new(),
            numerical: None,
            requirements: None,
            values: Vec::new(),
            blocks: vec![BlockData {
                parameters: Vec::new(),
                operations: Vec::new(),
            }],
            open: vec![0],
            operations: 0,
        })
    }

    /// Returns the requirements the scheduled region derived.
    ///
    /// A producer declares these through [`KernelBuilder::requirements`]; the
    /// verifier proves the declaration equals this derived value rather than
    /// letting the kernel become a second editable authority.
    #[must_use]
    pub const fn derived_requirements(&self) -> ResourceRequirements {
        self.derived_requirements
    }

    /// Declares one buffer parameter of the kernel signature.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::StructuralLimit`] when the buffer limit is
    /// exceeded.
    pub fn declare_buffer(
        &mut self,
        parameter: BufferParameter,
    ) -> Result<KernelBufferId, KernelBuildError> {
        limit(
            self.buffers.len().saturating_add(1),
            MAX_KERNEL_BUFFERS,
            KernelLimitKind::Buffers,
        )?;
        let id = KernelBufferId::from_len(self.owner, self.buffers.len()).ok_or(
            KernelBuildError::StructuralLimit {
                resource: KernelLimitKind::Buffers,
                actual: self.buffers.len().saturating_add(1),
                limit: MAX_KERNEL_BUFFERS,
            },
        )?;
        self.buffers.push(parameter);
        Ok(id)
    }

    /// Declares one workgroup staging allocation.
    ///
    /// Returns a handle in its own space, not a [`KernelBufferId`]: the two
    /// index separate declaration lists, and a buffer parameter's position is
    /// its argument-table ordinal while an allocation's is not an argument at
    /// all. One handle type would let a staged load reach a buffer parameter by
    /// ordinal coincidence.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::StructuralLimit`] when the staging limit is
    /// exceeded.
    pub fn declare_staging(
        &mut self,
        parameter: StagingParameter,
    ) -> Result<KernelStagingId, KernelBuildError> {
        limit(
            self.staging.len().saturating_add(1),
            MAX_KERNEL_STAGING,
            KernelLimitKind::Staging,
        )?;
        let id = KernelStagingId::from_len(self.owner, self.staging.len()).ok_or(
            KernelBuildError::StructuralLimit {
                resource: KernelLimitKind::Staging,
                actual: self.staging.len().saturating_add(1),
                limit: MAX_KERNEL_STAGING,
            },
        )?;
        self.staging.push(parameter);
        Ok(id)
    }

    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Declares one live input-extent operand of the kernel signature.
    ///
    /// Declaration order is canonical `(input ordinal, axis)` order: the
    /// verifier refuses any other. The live value is not recorded; only the
    /// root is.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::InputExtentNotInput`] when the tensor is not
    /// a scheduled input, [`KernelBuildError::InputExtentWrongAxis`] when the
    /// axis is outside that input's scheduled rank, [`KernelBuildError::DuplicateInputExtent`]
    /// when the same axis is declared twice, or [`KernelBuildError::StructuralLimit`]
    /// when the operand limit is exceeded.
    pub fn declare_input_extent(
        &mut self,
        parameter: InputExtentParameter,
    ) -> Result<KernelInputExtentId, KernelBuildError> {
        let crate::schedule::TensorRole::Input { .. } = parameter.tensor else {
            return Err(KernelBuildError::InputExtentNotInput);
        };
        if u64::from(parameter.axis.get()) >= scheduled_input_rank(&self.schedule, parameter.tensor)
        {
            return Err(KernelBuildError::InputExtentWrongAxis);
        }
        if self
            .input_extents
            .iter()
            .any(|declared| declared.tensor == parameter.tensor && declared.axis == parameter.axis)
        {
            return Err(KernelBuildError::DuplicateInputExtent);
        }
        limit(
            self.input_extents.len().saturating_add(1),
            MAX_KERNEL_INPUT_EXTENTS,
            KernelLimitKind::InputExtents,
        )?;
        let id = KernelInputExtentId::from_len(self.owner, self.input_extents.len()).ok_or(
            KernelBuildError::StructuralLimit {
                resource: KernelLimitKind::InputExtents,
                actual: self.input_extents.len().saturating_add(1),
                limit: MAX_KERNEL_INPUT_EXTENTS,
            },
        )?;
        self.input_extents.push(parameter);
        Ok(id)
    }

    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Reads one declared live input-extent operand as an index-typed value.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::UndeclaredInputExtent`] when the handle does
    /// not name a declared operand of this builder, or a foreign/invalid handle
    /// error.
    pub fn input_extent(
        &mut self,
        id: KernelInputExtentId,
    ) -> Result<KernelValueId, KernelBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(KernelEntityKind::InputExtent, true));
        }
        if self.input_extents.get(id.as_usize()).is_none() {
            return Err(KernelBuildError::UndeclaredInputExtent);
        }
        self.emit_single(
            OperationKind::InputExtent {
                parameter: id.index,
            },
            KernelType::Index,
            None,
        )
    }

    /// Admits one governed launch builtin into the kernel signature.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::DuplicateAdmittedBuiltin`] when the builtin
    /// is already admitted, or [`KernelBuildError::StructuralLimit`] when the
    /// admitted-builtin limit is exceeded.
    pub fn admit_builtin(&mut self, builtin: Builtin) -> Result<(), KernelBuildError> {
        if self.admitted_builtins.contains(&builtin) {
            return Err(KernelBuildError::DuplicateAdmittedBuiltin);
        }
        limit(
            self.admitted_builtins.len().saturating_add(1),
            MAX_KERNEL_ADMITTED_BUILTINS,
            KernelLimitKind::AdmittedBuiltins,
        )?;
        self.admitted_builtins.push(builtin);
        Ok(())
    }

    /// Declares the numerical realization the kernel preserves.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::ComponentAlreadySet`] if already declared.
    pub fn numerical(&mut self, numerical: NumericalRealization) -> Result<(), KernelBuildError> {
        set_once(
            &mut self.numerical,
            numerical,
            KernelComponent::NumericalRealization,
        )
    }

    /// Declares the resource requirements the kernel claims.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::ComponentAlreadySet`] if already declared.
    pub fn requirements(
        &mut self,
        requirements: ResourceRequirements,
    ) -> Result<(), KernelBuildError> {
        set_once(
            &mut self.requirements,
            requirements,
            KernelComponent::ResourceRequirements,
        )
    }

    /// Reads one admitted launch builtin.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::UndeclaredBuiltin`] when the signature does
    /// not admit the builtin, or a structural-limit error.
    pub fn builtin(&mut self, builtin: Builtin) -> Result<KernelValueId, KernelBuildError> {
        if !self.admitted_builtins.contains(&builtin) {
            return Err(KernelBuildError::UndeclaredBuiltin);
        }
        self.emit_single(
            OperationKind::Builtin { builtin },
            builtin.result_type(),
            None,
        )
    }

    /// Defines one typed immediate constant.
    ///
    /// # Errors
    ///
    /// Returns a structural-limit error when a governed bound is exceeded.
    pub fn constant(&mut self, value: KernelConstant) -> Result<KernelValueId, KernelBuildError> {
        self.emit_single(
            OperationKind::Constant { value },
            value.value_type(),
            Some(value),
        )
    }

    /// Applies one pure binary operation.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, divisor, or structural-limit error.
    pub fn binary(
        &mut self,
        op: BinaryOp,
        lhs: KernelValueId,
        rhs: KernelValueId,
    ) -> Result<KernelValueId, KernelBuildError> {
        let left = self.resolve(lhs)?;
        expect_type(op.operand_type(), left.value_type)?;
        let right = self.resolve(rhs)?;
        expect_type(op.operand_type(), right.value_type)?;
        if op.requires_constant_divisor() {
            let divisor = right
                .constant
                .and_then(KernelConstant::as_index)
                .ok_or(KernelBuildError::NonConstantDivisor)?;
            if divisor == 0 {
                return Err(KernelBuildError::NonPositiveDivisor);
            }
        }
        self.emit_single(
            OperationKind::Binary {
                op,
                lhs: lhs.index,
                rhs: rhs.index,
            },
            op.result_type(),
            None,
        )
    }

    /// Applies one predicate-producing comparison.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, or structural-limit error.
    pub fn compare(
        &mut self,
        op: CompareOp,
        lhs: KernelValueId,
        rhs: KernelValueId,
    ) -> Result<KernelValueId, KernelBuildError> {
        expect_type(op.operand_type(), self.resolve(lhs)?.value_type)?;
        expect_type(op.operand_type(), self.resolve(rhs)?.value_type)?;
        self.emit_single(
            OperationKind::Compare {
                op,
                lhs: lhs.index,
                rhs: rhs.index,
            },
            op.result_type(),
            None,
        )
    }

    /// Applies one named typed conversion.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, or structural-limit error.
    pub fn convert(
        &mut self,
        op: ConvertOp,
        source: KernelValueId,
    ) -> Result<KernelValueId, KernelBuildError> {
        expect_type(op.source_type(), self.resolve(source)?.value_type)?;
        self.emit_single(
            OperationKind::Convert {
                op,
                source: source.index,
            },
            op.result_type(),
            None,
        )
    }

    /// Applies one pure unary elementary function.
    ///
    /// The construct names the *precise* function. What it may deliver is the
    /// registered accuracy contract of the semantic operation being realized, and
    /// this builder states no accuracy of its own — a second spelling here would
    /// be a second authority over the same obligation.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, or structural-limit error.
    pub fn unary(
        &mut self,
        op: UnaryOp,
        source: KernelValueId,
    ) -> Result<KernelValueId, KernelBuildError> {
        expect_type(op.operand_type(), self.resolve(source)?.value_type)?;
        self.emit_single(
            OperationKind::Unary {
                op,
                source: source.index,
            },
            op.result_type(),
            None,
        )
    }

    /// Extracts one logical packed value from a loaded carrier byte.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, or structural-limit error.
    pub fn packed_extract(
        &mut self,
        op: PackedExtractOp,
        carrier: KernelValueId,
        logical_index: KernelValueId,
    ) -> Result<KernelValueId, KernelBuildError> {
        expect_type(op.carrier_type(), self.resolve(carrier)?.value_type)?;
        expect_type(op.index_type(), self.resolve(logical_index)?.value_type)?;
        self.emit_single(
            OperationKind::PackedExtract {
                op,
                carrier: carrier.index,
                logical_index: logical_index.index,
            },
            op.result_type(),
            None,
        )
    }

    /// Loads one element from a readable buffer under bounds evidence.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, access-mode, or structural-limit error.
    pub fn load(
        &mut self,
        buffer: KernelBufferId,
        offset: KernelValueId,
        bounds: BoundsWitnessId,
    ) -> Result<KernelValueId, KernelBuildError> {
        let parameter = self.resolve_buffer(buffer)?;
        if parameter.access != BufferAccess::Read {
            return Err(KernelBuildError::BufferAccessViolation);
        }
        expect_type(KernelType::Index, self.resolve(offset)?.value_type)?;
        self.emit_single(
            OperationKind::Load {
                buffer: buffer.index,
                offset: offset.index,
                bounds,
            },
            parameter.element_type,
            None,
        )
    }

    /// Loads one element when `predicate` is true; otherwise returns `inactive`
    /// without a memory access.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, access-mode, or structural-limit error.
    /// The predicate must be Boolean and `inactive` must have the buffer
    /// element type.
    pub fn guarded_load(
        &mut self,
        predicate: KernelValueId,
        buffer: KernelBufferId,
        offset: KernelValueId,
        bounds: BoundsWitnessId,
        inactive: KernelValueId,
    ) -> Result<KernelValueId, KernelBuildError> {
        let parameter = self.resolve_buffer(buffer)?;
        if parameter.access != BufferAccess::Read {
            return Err(KernelBuildError::BufferAccessViolation);
        }
        expect_type(KernelType::Bool, self.resolve(predicate)?.value_type)?;
        expect_type(KernelType::Index, self.resolve(offset)?.value_type)?;
        expect_type(parameter.element_type, self.resolve(inactive)?.value_type)?;
        self.emit_single(
            OperationKind::GuardedLoad {
                predicate: predicate.index,
                buffer: buffer.index,
                offset: offset.index,
                bounds,
                inactive: inactive.index,
            },
            parameter.element_type,
            None,
        )
    }

    /// Stores one element to a writable buffer under bounds and ownership evidence.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, access-mode, or structural-limit error.
    pub fn store(
        &mut self,
        buffer: KernelBufferId,
        offset: KernelValueId,
        value: KernelValueId,
        bounds: BoundsWitnessId,
        ownership: OwnershipWitnessId,
    ) -> Result<(), KernelBuildError> {
        let parameter = self.resolve_buffer(buffer)?;
        if parameter.access != BufferAccess::Write {
            return Err(KernelBuildError::BufferAccessViolation);
        }
        expect_type(KernelType::Index, self.resolve(offset)?.value_type)?;
        expect_type(parameter.element_type, self.resolve(value)?.value_type)?;
        self.emit(
            OperationKind::Store {
                buffer: buffer.index,
                offset: offset.index,
                value: value.index,
                bounds,
                ownership,
            },
            &[],
            None,
        )
        .map(|_| ())
    }

    /// Stores one element into a workgroup staging allocation.
    ///
    /// The phase names the tile phase whose declared staged write authorizes
    /// this effect; whole-kernel verification resolves it against the region's
    /// cooperative tile, so a producer cannot stage in a phase that declares no
    /// write. There is no access-mode check here because a staging allocation is
    /// both read and written by its workgroup, which is why it is not a
    /// [`BufferParameter`] and why [`BufferAccess`] stayed a two-value
    /// vocabulary.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, or structural-limit error.
    pub fn staged_store(
        &mut self,
        staging: KernelStagingId,
        offset: KernelValueId,
        value: KernelValueId,
        phase: PhaseId,
    ) -> Result<(), KernelBuildError> {
        let parameter = self.resolve_staging(staging)?;
        expect_type(KernelType::Index, self.resolve(offset)?.value_type)?;
        expect_type(parameter.element_type, self.resolve(value)?.value_type)?;
        self.emit(
            OperationKind::StagedStore {
                staging: staging.index,
                offset: offset.index,
                value: value.index,
                phase,
            },
            &[],
            None,
        )
        .map(|_| ())
    }

    /// Loads one element from a workgroup staging allocation.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, type, or structural-limit error.
    pub fn staged_load(
        &mut self,
        staging: KernelStagingId,
        offset: KernelValueId,
        phase: PhaseId,
    ) -> Result<KernelValueId, KernelBuildError> {
        let parameter = self.resolve_staging(staging)?;
        expect_type(KernelType::Index, self.resolve(offset)?.value_type)?;
        self.emit_single(
            OperationKind::StagedLoad {
                staging: staging.index,
                offset: offset.index,
                phase,
            },
            parameter.element_type,
            None,
        )
    }

    /// Emits one synchronization point with explicitly named scopes and fences.
    ///
    /// The spec names the schedule synchronization point this barrier realizes.
    /// Whole-kernel verification resolves that reference against the region's
    /// cooperative tile and proves the declared spelling projects onto the
    /// point's subject, so a barrier can never be a second authority over what
    /// the schedule requires.
    ///
    /// # Errors
    ///
    /// Returns a structural-limit error when a governed bound is exceeded.
    pub fn barrier(&mut self, spec: BarrierSpec) -> Result<(), KernelBuildError> {
        self.emit(OperationKind::Barrier { spec }, &[], None)
            .map(|_| ())
    }

    /// Emits a nested block executed only when `predicate` holds.
    ///
    /// # Errors
    ///
    /// Returns a handle, scope, or type error for the predicate, any error the
    /// body raises, or a structural-limit error. A failed body leaves the
    /// builder exactly as it was before the call.
    pub fn predicated<F>(
        &mut self,
        predicate: KernelValueId,
        body: F,
    ) -> Result<(), KernelBuildError>
    where
        F: FnOnce(&mut Self) -> Result<(), KernelBuildError>,
    {
        expect_type(KernelType::Bool, self.resolve(predicate)?.value_type)?;
        let checkpoint = self.checkpoint();
        let block = match self.open_block(&[]).and_then(|block| {
            body(self)?;
            Ok(block)
        }) {
            Ok(block) => block,
            Err(error) => {
                self.restore(checkpoint);
                return Err(error);
            }
        };
        self.open.pop();
        self.emit(
            OperationKind::Predicated {
                predicate: predicate.index,
                body: block,
            },
            &[],
            None,
        )
        .map(|_| ())
    }

    /// Emits a bounded loop carrying typed accumulator state.
    ///
    /// The body receives the induction variable and the accumulator parameters
    /// and returns the values yielded at the end of one iteration. Yield arity
    /// and types must match the carried accumulators exactly.
    ///
    /// # Errors
    ///
    /// Returns [`KernelBuildError::InvalidLoopRange`] for an empty or
    /// descending range, [`KernelBuildError::EmptyLoopAccumulators`] when no
    /// state is carried, a yield arity or type error, any error the body
    /// raises, or a structural-limit error. A failed body leaves the builder
    /// exactly as it was before the call.
    pub fn serial_loop<F>(
        &mut self,
        spec: SerialLoopSpec,
        initial: &[KernelValueId],
        body: F,
    ) -> Result<SerialLoopResults, KernelBuildError>
    where
        F: FnOnce(&mut Self, &SerialLoopParameters) -> Result<Vec<KernelValueId>, KernelBuildError>,
    {
        if spec.start >= spec.end {
            return Err(KernelBuildError::InvalidLoopRange {
                start: spec.start,
                end: spec.end,
            });
        }
        if initial.is_empty() {
            return Err(KernelBuildError::EmptyLoopAccumulators);
        }
        limit(
            initial.len(),
            MAX_KERNEL_LOOP_ACCUMULATORS,
            KernelLimitKind::LoopAccumulators,
        )?;
        let mut carried = Vec::with_capacity(initial.len());
        for value in initial {
            carried.push(self.resolve(*value)?.value_type);
        }
        let mut parameter_types = Vec::with_capacity(carried.len().saturating_add(1));
        parameter_types.push(KernelType::Index);
        parameter_types.extend(carried.iter().copied());

        let checkpoint = self.checkpoint();
        let outcome = self.run_loop_body(&parameter_types, &carried, body);
        match outcome {
            Ok((block, yields)) => {
                self.open.pop();
                let results = self.emit(
                    OperationKind::SerialLoop {
                        start: spec.start,
                        end: spec.end,
                        initial: initial.iter().map(|value| value.index).collect(),
                        body: block,
                        yields,
                    },
                    &carried,
                    None,
                )?;
                Ok(SerialLoopResults(results))
            }
            Err(error) => {
                self.restore(checkpoint);
                Err(error)
            }
        }
    }

    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Executes a bounded loop whose start and end are index-typed SSA values.
    ///
    /// This is the form a live input extent uses as a trip count. The static
    /// [`Self::serial_loop`] path remains for compile-time ranges; a kernel
    /// cannot bake a live extent into that path and also read it here.
    ///
    /// # Errors
    ///
    /// Returns a type mismatch when a bound is not index-typed, or the same
    /// accumulator and structural errors as [`Self::serial_loop`].
    pub fn serial_loop_range<F>(
        &mut self,
        start: KernelValueId,
        end: KernelValueId,
        initial: &[KernelValueId],
        body: F,
    ) -> Result<SerialLoopResults, KernelBuildError>
    where
        F: FnOnce(&mut Self, &SerialLoopParameters) -> Result<Vec<KernelValueId>, KernelBuildError>,
    {
        let start_type = self.resolve(start)?.value_type;
        if start_type != KernelType::Index {
            return Err(KernelBuildError::TypeMismatch {
                expected: KernelType::Index,
                actual: start_type,
            });
        }
        let end_type = self.resolve(end)?.value_type;
        if end_type != KernelType::Index {
            return Err(KernelBuildError::TypeMismatch {
                expected: KernelType::Index,
                actual: end_type,
            });
        }
        if initial.is_empty() {
            return Err(KernelBuildError::EmptyLoopAccumulators);
        }
        limit(
            initial.len(),
            MAX_KERNEL_LOOP_ACCUMULATORS,
            KernelLimitKind::LoopAccumulators,
        )?;
        let mut carried = Vec::with_capacity(initial.len());
        for value in initial {
            carried.push(self.resolve(*value)?.value_type);
        }
        let mut parameter_types = Vec::with_capacity(carried.len().saturating_add(1));
        parameter_types.push(KernelType::Index);
        parameter_types.extend(carried.iter().copied());

        let checkpoint = self.checkpoint();
        let outcome = self.run_loop_body(&parameter_types, &carried, body);
        match outcome {
            Ok((block, yields)) => {
                self.open.pop();
                let results = self.emit(
                    OperationKind::SerialLoopRange {
                        start: start.index,
                        end: end.index,
                        initial: initial.iter().map(|value| value.index).collect(),
                        body: block,
                        yields,
                    },
                    &carried,
                    None,
                )?;
                Ok(SerialLoopResults(results))
            }
            Err(error) => {
                self.restore(checkpoint);
                Err(error)
            }
        }
    }

    /// Verifies the whole kernel and freezes it, or returns the intact builder.
    ///
    /// # Errors
    ///
    /// Returns a [`KernelVerificationError`] carrying every whole-kernel
    /// diagnostic and the recoverable builder when verification fails.
    pub fn build(mut self) -> Result<VerifiedKernel, KernelVerificationError> {
        // Completeness is checked before anything moves, so an incomplete
        // builder is returned in exactly the state it arrived in.
        let data = match self.take_data() {
            Ok(data) => data,
            Err(diagnostic) => {
                return Err(KernelVerificationError {
                    builder: Box::new(self),
                    diagnostics: vec![diagnostic],
                });
            }
        };
        let verified = super::verify::verify_kernel(
            &data,
            &self.schedule,
            &self.schedule_identity,
            self.derived_requirements,
        )
        .and_then(|()| encode_identity(&self.schedule_identity, &data));
        match verified {
            // The freedom is read from the region this builder was opened
            // against, never declared: `KernelBuilder::new` is the only path
            // that reaches `build`, and it takes a `VerifiedScheduledRegion`,
            // so the scalar program classified here is one the intrinsic
            // schedule verifier already accepted.
            Ok(identity) => Ok(VerifiedKernel {
                owner: self.owner.verified_owner(),
                region: self.region,
                schedule_identity: self.schedule_identity,
                subnormal_freedom: subnormal_freedom_of(&self.schedule.index.scalar_program),
                required_nonzero_input_extents: required_nonzero_input_extents(&self.schedule),
                data,
                identity,
            }),
            Err(diagnostic) => {
                self.restore_data(data);
                Err(KernelVerificationError {
                    builder: Box::new(self),
                    diagnostics: vec![diagnostic],
                })
            }
        }
    }

    /// Moves the assembled arena out of a builder that is about to be dropped.
    pub(super) fn into_data(mut self) -> Result<KernelData, KernelDiagnostic> {
        self.take_data()
    }

    /// Moves the arena into a [`KernelData`], emptying the builder's storage.
    ///
    /// **The builder is left recoverable, not consistent.** Its arena is gone
    /// until [`Self::restore_data`] puts it back, so the only admissible use is
    /// the one in [`Self::build`]: take, verify, and either publish the data or
    /// restore it before the builder becomes reachable again. Verification reads
    /// the `KernelData` it was handed and never the builder's storage, so the
    /// window is not observable.
    ///
    /// This replaced a deep clone of all four arenas. The clone was paid on
    /// every kernel build *and* on every refinement check — `verify_kernel`
    /// re-derives the canonical body through this same builder — while the
    /// verifier only ever reads the assembled copy, so the original was
    /// duplicated to be dropped.
    fn take_data(&mut self) -> Result<KernelData, KernelDiagnostic> {
        let numerical = self.numerical.ok_or(KernelDiagnostic::IncompleteKernel {
            component: KernelComponent::NumericalRealization,
        })?;
        let requirements = self
            .requirements
            .ok_or(KernelDiagnostic::IncompleteKernel {
                component: KernelComponent::ResourceRequirements,
            })?;
        Ok(KernelData {
            buffers: std::mem::take(&mut self.buffers),
            staging: std::mem::take(&mut self.staging),
            input_extents: std::mem::take(&mut self.input_extents),
            admitted_builtins: std::mem::take(&mut self.admitted_builtins),
            numerical,
            requirements,
            values: std::mem::take(&mut self.values),
            blocks: std::mem::take(&mut self.blocks),
        })
    }

    /// Returns a taken arena to the builder, restoring the recoverable state
    /// [`KernelVerificationError`] documents.
    fn restore_data(&mut self, data: KernelData) {
        self.buffers = data.buffers;
        self.staging = data.staging;
        self.input_extents = data.input_extents;
        self.admitted_builtins = data.admitted_builtins;
        self.values = data.values;
        self.blocks = data.blocks;
    }

    fn run_loop_body<F>(
        &mut self,
        parameter_types: &[KernelType],
        carried: &[KernelType],
        body: F,
    ) -> Result<(u32, Vec<u32>), KernelBuildError>
    where
        F: FnOnce(&mut Self, &SerialLoopParameters) -> Result<Vec<KernelValueId>, KernelBuildError>,
    {
        let block = self.open_block(parameter_types)?;
        let parameters = {
            let ids: Vec<KernelValueId> = self.blocks[block as usize]
                .parameters
                .iter()
                .map(|index| KernelValueId {
                    owner: self.owner,
                    index: *index,
                })
                .collect();
            let (induction, accumulators) = ids
                .split_first()
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
            SerialLoopParameters {
                induction: *induction,
                accumulators: accumulators.to_vec(),
            }
        };
        let yields = body(self, &parameters)?;
        if yields.len() != carried.len() {
            return Err(KernelBuildError::LoopYieldArity {
                expected: carried.len(),
                actual: yields.len(),
            });
        }
        let mut indices = Vec::with_capacity(yields.len());
        for (position, (value, expected)) in yields.iter().zip(carried).enumerate() {
            let actual = self.resolve(*value)?.value_type;
            if actual != *expected {
                return Err(KernelBuildError::LoopYieldTypeMismatch {
                    position,
                    expected: *expected,
                    actual,
                });
            }
            indices.push(value.index);
        }
        Ok((block, indices))
    }

    fn open_block(&mut self, parameter_types: &[KernelType]) -> Result<u32, KernelBuildError> {
        limit(
            self.blocks.len().saturating_add(1),
            MAX_KERNEL_BLOCKS,
            KernelLimitKind::Blocks,
        )?;
        limit(
            self.open.len().saturating_add(1),
            MAX_KERNEL_BLOCK_DEPTH,
            KernelLimitKind::BlockDepth,
        )?;
        let block =
            u32::try_from(self.blocks.len()).map_err(|_| KernelBuildError::StructuralLimit {
                resource: KernelLimitKind::Blocks,
                actual: self.blocks.len().saturating_add(1),
                limit: MAX_KERNEL_BLOCKS,
            })?;
        self.blocks.push(BlockData {
            parameters: Vec::new(),
            operations: Vec::new(),
        });
        self.open.push(block);
        let mut parameters = Vec::with_capacity(parameter_types.len());
        for value_type in parameter_types {
            parameters.push(self.push_value(*value_type, None)?.index);
        }
        self.blocks[block as usize].parameters = parameters;
        Ok(block)
    }

    fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            buffers: self.buffers.len(),
            staging: self.staging.len(),
            input_extents: self.input_extents.len(),
            admitted_builtins: self.admitted_builtins.len(),
            values: self.values.len(),
            blocks: self.blocks.len(),
            open: self.open.len(),
            operations: self.operations,
        }
    }

    fn restore(&mut self, checkpoint: Checkpoint) {
        self.buffers.truncate(checkpoint.buffers);
        self.staging.truncate(checkpoint.staging);
        self.input_extents.truncate(checkpoint.input_extents);
        self.admitted_builtins
            .truncate(checkpoint.admitted_builtins);
        self.values.truncate(checkpoint.values);
        self.blocks.truncate(checkpoint.blocks);
        self.open.truncate(checkpoint.open);
        self.operations = checkpoint.operations;
    }

    fn current_block(&self) -> u32 {
        *self
            .open
            .last()
            .expect("the top-level block is never closed")
    }

    fn push_value(
        &mut self,
        value_type: KernelType,
        constant: Option<KernelConstant>,
    ) -> Result<KernelValueId, KernelBuildError> {
        limit(
            self.values.len().saturating_add(1),
            MAX_KERNEL_VALUES,
            KernelLimitKind::Values,
        )?;
        let id = KernelValueId::from_len(self.owner, self.values.len()).ok_or(
            KernelBuildError::StructuralLimit {
                resource: KernelLimitKind::Values,
                actual: self.values.len().saturating_add(1),
                limit: MAX_KERNEL_VALUES,
            },
        )?;
        let block = self.current_block();
        self.values.push(ValueData {
            value_type,
            block,
            constant,
        });
        Ok(id)
    }

    fn emit_single(
        &mut self,
        kind: OperationKind,
        result_type: KernelType,
        constant: Option<KernelConstant>,
    ) -> Result<KernelValueId, KernelBuildError> {
        let results = self.emit(kind, &[result_type], constant)?;
        results
            .first()
            .copied()
            .ok_or(invalid_handle(KernelEntityKind::Value, false))
    }

    fn emit(
        &mut self,
        kind: OperationKind,
        result_types: &[KernelType],
        constant: Option<KernelConstant>,
    ) -> Result<Vec<KernelValueId>, KernelBuildError> {
        limit(
            self.operations.saturating_add(1),
            MAX_KERNEL_OPERATIONS,
            KernelLimitKind::Operations,
        )?;
        let mut results = Vec::with_capacity(result_types.len());
        for value_type in result_types {
            results.push(self.push_value(*value_type, constant)?);
        }
        let block = self.current_block();
        self.blocks[block as usize].operations.push(OperationData {
            kind,
            results: results.iter().map(|value| value.index).collect(),
        });
        self.operations = self.operations.saturating_add(1);
        Ok(results)
    }

    fn resolve(&self, id: KernelValueId) -> Result<ValueData, KernelBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(KernelEntityKind::Value, true));
        }
        let value = *self
            .values
            .get(id.as_usize())
            .ok_or_else(|| invalid_handle(KernelEntityKind::Value, false))?;
        if !self.open.contains(&value.block) {
            return Err(KernelBuildError::ValueOutOfScope);
        }
        Ok(value)
    }

    fn resolve_buffer(&self, id: KernelBufferId) -> Result<BufferParameter, KernelBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(KernelEntityKind::Buffer, true));
        }
        self.buffers
            .get(id.as_usize())
            .copied()
            .ok_or_else(|| invalid_handle(KernelEntityKind::Buffer, false))
    }

    fn resolve_staging(&self, id: KernelStagingId) -> Result<StagingParameter, KernelBuildError> {
        if id.owner != self.owner {
            return Err(invalid_handle(KernelEntityKind::Staging, true));
        }
        self.staging
            .get(id.as_usize())
            .copied()
            .ok_or_else(|| invalid_handle(KernelEntityKind::Staging, false))
    }
}

/// Derives every input extent whose scheduled operation has no empty result.
///
/// A live contraction seeds its strict fold from contributor zero before its
/// remaining range begins, so its selected contraction extent must be nonzero.
/// `LiveRowMajor` deliberately contributes nothing here: its whole element
/// access is inside a range bounded by the live extent and therefore performs
/// no work when that extent is zero.
fn required_nonzero_input_extents(schedule: &ScheduledRegion) -> Vec<InputExtentParameter> {
    match schedule.schedule.reduction {
        ReductionTopology::LiveContraction {
            live_input,
            live_axis,
            ..
        } => vec![InputExtentParameter {
            tensor: TensorRole::Input {
                ordinal: live_input,
            },
            axis: live_axis,
        }],
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::MultiPass { .. }
        | ReductionTopology::Contraction { .. }
        | ReductionTopology::CooperativeWorkgroup { .. }
        | ReductionTopology::CooperativeContraction { .. } => Vec::new(),
    }
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    component: KernelComponent,
) -> Result<(), KernelBuildError> {
    if slot.is_some() {
        return Err(KernelBuildError::ComponentAlreadySet { component });
    }
    *slot = Some(value);
    Ok(())
}

fn expect_type(expected: KernelType, actual: KernelType) -> Result<(), KernelBuildError> {
    if expected == actual {
        return Ok(());
    }
    Err(KernelBuildError::TypeMismatch { expected, actual })
}

fn scheduled_input_rank(schedule: &ScheduledRegion, tensor: TensorRole) -> u64 {
    let Some(access) = schedule
        .index
        .accesses
        .iter()
        .find(|access| access.tensor == tensor)
    else {
        return 0;
    };
    let static_rank = match &access.map {
        LogicalAccess::LinearIdentity
        | LogicalAccess::ScalarBroadcast
        | LogicalAccess::PackedU4LsbZeroTail { .. } => 1,
        LogicalAccess::ReductionContributor { input_shape, .. } => input_shape.rank() as u64,
        LogicalAccess::ContractionOperand { operand_shape, .. }
        | LogicalAccess::ReindexBijection { operand_shape, .. }
        | LogicalAccess::BroadcastReplication { operand_shape, .. } => operand_shape.rank() as u64,
        LogicalAccess::ParametricBroadcast { operand_shape, .. } => operand_shape.rank() as u64,
        LogicalAccess::LiveRowMajor { inner_axis } => u64::from(inner_axis.get()).saturating_add(1),
    };
    if matches!(
        schedule.schedule.reduction,
        ReductionTopology::LiveContraction { .. }
    ) && matches!(access.map, LogicalAccess::ContractionOperand { .. })
    {
        static_rank.saturating_add(1)
    } else {
        static_rank
    }
}

fn limit(actual: usize, limit: usize, resource: KernelLimitKind) -> Result<(), KernelBuildError> {
    if actual > limit {
        return Err(KernelBuildError::StructuralLimit {
            resource,
            actual,
            limit,
        });
    }
    Ok(())
}
