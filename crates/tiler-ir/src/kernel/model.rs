//! Structured-kernel data model, read-only views, verified wrapper, identity.
//!
//! The vocabulary is deliberately explicit: a backend reads typed values,
//! governed address spaces, admitted launch builtins, computed element offsets,
//! typed loads and stores carrying their schedule evidence, structured
//! predicates and bounded loops, named conversions, and ordered effects. Nothing
//! here requires a backend to reconstruct semantic-graph structure, an access
//! relation, a reduction order, or a numerical rule; every such fact is either a
//! field of this IR or an explicit operation in the body.
//!
//! Only [`super::KernelBuilder::build`] can bind a draft into an opaque
//! [`VerifiedKernel`]. The verified wrapper exposes read-only meaning and never
//! mutation, thawing, or unchecked construction.

use crate::schedule::{BoundsWitnessId, OwnershipWitnessId};
use crate::schedule::{
    CanonicalScheduledRegionIdentity, NumericalPermission, NumericalRealization, RegionId,
    ResourceRequirements, SubnormalMode, TensorRole,
};

use super::MAX_KERNEL_IDENTITY_BYTES;
use super::error::{KernelDiagnostic, KernelEntityKind, VerifiedKernelHandleError};
use super::handles::{VerifiedBufferId, VerifiedKernelOwner, VerifiedValueId};

/// The resolved type of one structured-kernel SSA value.
///
/// The bounded profile resolves exactly the roles the scheduled-region IR can
/// require: a control predicate, an unsigned 64-bit index role used for element
/// offsets and induction variables, and the `f32` element type. Widening this
/// vocabulary is a versioned extension, not an open type universe.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum KernelType {
    /// A one-bit control predicate.
    Bool,
    /// An unsigned 64-bit index-role integer.
    Index,
    /// An IEEE-754 binary32 value.
    F32,
}

impl KernelType {
    const fn tag(self) -> u8 {
        match self {
            Self::Bool => 0x01,
            Self::Index => 0x02,
            Self::F32 => 0x03,
        }
    }
}

/// A governed memory visibility and lifetime domain.
///
/// These describe visibility and lifetime in target-neutral terms. A target
/// profile maps a supported governed space onto its own realization or rejects
/// it; caches and registers are lowering facts, not address spaces.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum AddressSpace {
    /// Device-visible memory that outlives one dispatch.
    Device,
    /// Workgroup-scoped memory shared by one workgroup's invocations.
    Workgroup,
    /// Memory private to one invocation.
    InvocationPrivate,
    /// Read-only memory constant for the whole dispatch.
    Constant,
}

impl AddressSpace {
    const fn tag(self) -> u8 {
        match self {
            Self::Device => 0x01,
            Self::Workgroup => 0x02,
            Self::InvocationPrivate => 0x03,
            Self::Constant => 0x04,
        }
    }
}

/// The access mode one kernel buffer parameter admits.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum BufferAccess {
    /// The kernel may only load from the buffer.
    Read,
    /// The kernel may only store to the buffer.
    Write,
}

impl BufferAccess {
    const fn tag(self) -> u8 {
        match self {
            Self::Read => 0x01,
            Self::Write => 0x02,
        }
    }
}

/// One typed buffer parameter of a kernel signature.
///
/// The parameter names the scheduled boundary tensor it binds, its element
/// type, its governed address space, its access mode, and the exact number of
/// elements the kernel may address. A backend needs no other fact to emit the
/// binding.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BufferParameter {
    /// Scheduled boundary tensor this parameter binds.
    pub tensor: TensorRole,
    /// Element type stored in the buffer.
    pub element_type: KernelType,
    /// Governed address space of the buffer.
    pub address_space: AddressSpace,
    /// Access mode admitted for this parameter.
    pub access: BufferAccess,
    /// Number of addressable elements.
    pub element_count: u64,
}

/// A governed launch builtin a kernel signature may admit.
///
/// Builtins use governed execution keys, never a target spelling such as
/// `thread_position_in_grid`. Each admitted builtin realizes one scheduled
/// execution binding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Builtin {
    /// The linear index of one global invocation in the launch grid.
    GlobalInvocationIndex,
}

impl Builtin {
    const fn tag(self) -> u8 {
        match self {
            Self::GlobalInvocationIndex => 0x01,
        }
    }

    /// Returns the resolved type this builtin produces.
    #[must_use]
    pub const fn result_type(self) -> KernelType {
        match self {
            Self::GlobalInvocationIndex => KernelType::Index,
        }
    }
}

/// A typed immediate constant.
///
/// Floating-point constants carry their exact bit pattern so a backend cannot
/// reinterpret, round, or canonicalize them while emitting source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum KernelConstant {
    /// A control-predicate constant.
    Bool(bool),
    /// An index-role constant.
    Index(u64),
    /// An `f32` constant given by its exact bit pattern.
    F32Bits(u32),
}

impl KernelConstant {
    /// Returns the resolved type of this constant.
    #[must_use]
    pub const fn value_type(self) -> KernelType {
        match self {
            Self::Bool(_) => KernelType::Bool,
            Self::Index(_) => KernelType::Index,
            Self::F32Bits(_) => KernelType::F32,
        }
    }

    /// Returns the index-role value when this constant is index-typed.
    #[must_use]
    pub const fn as_index(self) -> Option<u64> {
        match self {
            Self::Index(value) => Some(value),
            Self::Bool(_) | Self::F32Bits(_) => None,
        }
    }
}

/// A pure binary operation over two same-typed operands.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum BinaryOp {
    /// Wrapping-free index addition.
    IndexAdd,
    /// Wrapping-free index multiplication.
    IndexMultiply,
    /// Truncating index division by a positive constant.
    IndexDivide,
    /// Index remainder by a positive constant.
    IndexModulo,
    /// IEEE-754 binary32 addition.
    F32Add,
    /// IEEE-754 binary32 multiplication.
    F32Multiply,
}

impl BinaryOp {
    const fn tag(self) -> u8 {
        match self {
            Self::IndexAdd => 0x01,
            Self::IndexMultiply => 0x02,
            Self::IndexDivide => 0x03,
            Self::IndexModulo => 0x04,
            Self::F32Add => 0x05,
            Self::F32Multiply => 0x06,
        }
    }

    /// Returns the required operand type.
    #[must_use]
    pub const fn operand_type(self) -> KernelType {
        match self {
            Self::IndexAdd | Self::IndexMultiply | Self::IndexDivide | Self::IndexModulo => {
                KernelType::Index
            }
            Self::F32Add | Self::F32Multiply => KernelType::F32,
        }
    }

    /// Returns the produced result type.
    #[must_use]
    pub const fn result_type(self) -> KernelType {
        self.operand_type()
    }

    /// Returns whether the right operand must be a positive constant.
    #[must_use]
    pub const fn requires_constant_divisor(self) -> bool {
        matches!(self, Self::IndexDivide | Self::IndexModulo)
    }
}

/// A predicate-producing comparison.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum CompareOp {
    /// Unsigned index ordering.
    IndexLessThan,
}

impl CompareOp {
    const fn tag(self) -> u8 {
        match self {
            Self::IndexLessThan => 0x01,
        }
    }

    /// Returns the required operand type.
    #[must_use]
    pub const fn operand_type(self) -> KernelType {
        match self {
            Self::IndexLessThan => KernelType::Index,
        }
    }

    /// Returns the produced result type.
    #[must_use]
    pub const fn result_type(self) -> KernelType {
        match self {
            Self::IndexLessThan => KernelType::Bool,
        }
    }
}

/// A typed conversion that names the exact contract it realizes.
///
/// A conversion is never implicit. The bounded profile governs the one
/// conversion the accepted numerical contract requires; a representation,
/// narrowing, or rounding conversion is a versioned extension that must name
/// its own rounding, overflow, and exceptional-value behaviour.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ConvertOp {
    /// Replaces a NaN result with the numerical realization's canonical NaN.
    ///
    /// The canonical bit pattern is the kernel's
    /// [`NumericalRealization::canonical_arithmetic_nan_bits`]; the operation
    /// deliberately does not carry a second copy of it.
    CanonicalizeF32Nan,
}

impl ConvertOp {
    const fn tag(self) -> u8 {
        match self {
            Self::CanonicalizeF32Nan => 0x01,
        }
    }

    /// Returns the required source type.
    #[must_use]
    pub const fn source_type(self) -> KernelType {
        match self {
            Self::CanonicalizeF32Nan => KernelType::F32,
        }
    }

    /// Returns the produced result type.
    #[must_use]
    pub const fn result_type(self) -> KernelType {
        match self {
            Self::CanonicalizeF32Nan => KernelType::F32,
        }
    }
}

/// The execution scope whose invocations must reach one barrier instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ExecutionScope {
    /// All invocations of one subgroup.
    Subgroup,
    /// All invocations of one workgroup.
    Workgroup,
}

impl ExecutionScope {
    const fn tag(self) -> u8 {
        match self {
            Self::Subgroup => 0x01,
            Self::Workgroup => 0x02,
        }
    }
}

/// The memory scope across which a barrier makes effects visible.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MemoryScope {
    /// Visibility within one workgroup.
    Workgroup,
    /// Visibility across the device.
    Device,
}

impl MemoryScope {
    const fn tag(self) -> u8 {
        match self {
            Self::Workgroup => 0x01,
            Self::Device => 0x02,
        }
    }
}

/// The ordering a barrier establishes over the effects it fences.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum BarrierOrdering {
    /// Prior effects are released and subsequent effects acquire them.
    AcquireRelease,
}

impl BarrierOrdering {
    const fn tag(self) -> u8 {
        match self {
            Self::AcquireRelease => 0x01,
        }
    }
}

/// A synchronization point with separately named scopes, fences, and ordering.
///
/// Execution scope, memory scope, fenced address spaces, and ordering stay
/// distinct even when one target builtin combines them (ADR 0048); collapsing
/// them into a single flag would lose the portable contract.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BarrierSpec {
    /// Invocations that must reach the same dynamic barrier instance.
    pub execution_scope: ExecutionScope,
    /// Scope across which fenced effects become visible.
    pub memory_scope: MemoryScope,
    /// Address spaces the barrier fences, in ascending governed order.
    pub fenced_spaces: Vec<AddressSpace>,
    /// Ordering established over the fenced effects.
    pub ordering: BarrierOrdering,
}

/// The bounded iteration range of one structured loop.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SerialLoopSpec {
    /// Inclusive first induction value.
    pub start: u64,
    /// Exclusive last induction value.
    pub end: u64,
}

/// Storage for one structured block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BlockData {
    /// Ordered block parameters, as value indices.
    pub(super) parameters: Vec<u32>,
    /// Ordered operations.
    pub(super) operations: Vec<OperationData>,
}

/// Storage for one structured operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OperationData {
    pub(super) kind: OperationKind,
    /// Ordered result value indices.
    pub(super) results: Vec<u32>,
}

/// Storage for one structured operation's kind and operands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum OperationKind {
    Builtin {
        builtin: Builtin,
    },
    Constant {
        value: KernelConstant,
    },
    Binary {
        op: BinaryOp,
        lhs: u32,
        rhs: u32,
    },
    Compare {
        op: CompareOp,
        lhs: u32,
        rhs: u32,
    },
    Convert {
        op: ConvertOp,
        source: u32,
    },
    Load {
        buffer: u32,
        offset: u32,
        bounds: BoundsWitnessId,
    },
    Store {
        buffer: u32,
        offset: u32,
        value: u32,
        bounds: BoundsWitnessId,
        ownership: OwnershipWitnessId,
    },
    Predicated {
        predicate: u32,
        body: u32,
    },
    SerialLoop {
        start: u64,
        end: u64,
        initial: Vec<u32>,
        body: u32,
        yields: Vec<u32>,
    },
    Barrier {
        spec: BarrierSpec,
    },
}

/// Storage for one structured SSA value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ValueData {
    pub(super) value_type: KernelType,
    pub(super) block: u32,
    pub(super) constant: Option<KernelConstant>,
}

/// The assembled, not-yet-verified structured kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct KernelData {
    pub(super) buffers: Vec<BufferParameter>,
    pub(super) admitted_builtins: Vec<Builtin>,
    pub(super) numerical: NumericalRealization,
    pub(super) requirements: ResourceRequirements,
    pub(super) values: Vec<ValueData>,
    pub(super) blocks: Vec<BlockData>,
}

/// Opaque canonical bytes identifying one verified structured kernel.
///
/// The identity folds the exact canonical identity of the scheduled region the
/// kernel refines (ADR 0071) together with the kernel signature, numerical
/// realization, derived requirements, and the whole structured body. It
/// excludes the transient [`RegionId`] planning ordinal, so equivalent kernels
/// produced by different planning histories share identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalKernelIdentity(Vec<u8>);

impl CanonicalKernelIdentity {
    /// Returns the canonical identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An immutable, verified structured kernel refining one scheduled region.
///
/// Only [`super::KernelBuilder::build`] produces one. Equality compares the
/// planning correlation and the canonical identity, so two independently
/// lowered kernels for the same scheduled region compare equal even though
/// their handle ownership tags differ.
#[derive(Clone, Debug)]
pub struct VerifiedKernel {
    pub(super) owner: VerifiedKernelOwner,
    pub(super) region: RegionId,
    pub(super) schedule_identity: CanonicalScheduledRegionIdentity,
    pub(super) data: KernelData,
    pub(super) identity: CanonicalKernelIdentity,
}

impl PartialEq for VerifiedKernel {
    fn eq(&self, other: &Self) -> bool {
        self.region == other.region && self.identity == other.identity
    }
}

impl Eq for VerifiedKernel {}

impl VerifiedKernel {
    /// Returns the planning ordinal of the scheduled region this kernel refines.
    #[must_use]
    pub const fn scheduled_region(&self) -> RegionId {
        self.region
    }

    /// Returns the canonical identity of the scheduled region this refines.
    #[must_use]
    pub const fn scheduled_region_identity(&self) -> &CanonicalScheduledRegionIdentity {
        &self.schedule_identity
    }

    /// Returns the canonical structural identity of this kernel.
    #[must_use]
    pub const fn canonical_identity(&self) -> &CanonicalKernelIdentity {
        &self.identity
    }

    /// Returns the ordered buffer parameters of the kernel signature.
    #[must_use]
    pub fn buffers(&self) -> impl ExactSizeIterator<Item = BufferParameter> + '_ {
        self.data.buffers.iter().copied()
    }

    /// Returns the launch builtins this kernel signature admits.
    #[must_use]
    pub fn admitted_builtins(&self) -> &[Builtin] {
        &self.data.admitted_builtins
    }

    /// Returns the preserved numerical realization.
    #[must_use]
    pub const fn numerical(&self) -> NumericalRealization {
        self.data.numerical
    }

    /// Returns the derived resource requirements.
    #[must_use]
    pub const fn requirements(&self) -> ResourceRequirements {
        self.data.requirements
    }

    /// Returns the kernel's top-level structured block.
    #[must_use]
    pub fn body(&self) -> BlockRef<'_> {
        BlockRef {
            kernel: self,
            block: 0,
        }
    }

    /// Returns the resolved type of one verified value.
    ///
    /// # Errors
    ///
    /// Returns [`VerifiedKernelHandleError`] when the handle belongs to another
    /// kernel or does not identify a retained value.
    pub fn value_type(&self, id: VerifiedValueId) -> Result<KernelType, VerifiedKernelHandleError> {
        self.value(id).map(|value| value.value_type)
    }

    /// Returns the constant a verified value denotes, when it is a constant.
    ///
    /// # Errors
    ///
    /// Returns [`VerifiedKernelHandleError`] when the handle belongs to another
    /// kernel or does not identify a retained value.
    pub fn value_constant(
        &self,
        id: VerifiedValueId,
    ) -> Result<Option<KernelConstant>, VerifiedKernelHandleError> {
        self.value(id).map(|value| value.constant)
    }

    /// Returns one verified buffer parameter.
    ///
    /// # Errors
    ///
    /// Returns [`VerifiedKernelHandleError`] when the handle belongs to another
    /// kernel or does not identify a retained parameter.
    pub fn buffer(
        &self,
        id: VerifiedBufferId,
    ) -> Result<BufferParameter, VerifiedKernelHandleError> {
        if id.owner != self.owner {
            return Err(VerifiedKernelHandleError::ForeignKernel {
                entity: KernelEntityKind::Buffer,
            });
        }
        self.data.buffers.get(id.as_usize()).copied().ok_or(
            VerifiedKernelHandleError::InvalidHandle {
                entity: KernelEntityKind::Buffer,
            },
        )
    }

    fn value(&self, id: VerifiedValueId) -> Result<&ValueData, VerifiedKernelHandleError> {
        if id.owner != self.owner {
            return Err(VerifiedKernelHandleError::ForeignKernel {
                entity: KernelEntityKind::Value,
            });
        }
        self.data
            .values
            .get(id.as_usize())
            .ok_or(VerifiedKernelHandleError::InvalidHandle {
                entity: KernelEntityKind::Value,
            })
    }

    fn value_id(&self, index: u32) -> VerifiedValueId {
        VerifiedValueId::from_verified(self.owner, index)
    }

    fn buffer_id(&self, index: u32) -> VerifiedBufferId {
        VerifiedBufferId::from_verified(self.owner, index)
    }
}

/// A read-only view of one structured block.
#[derive(Clone, Copy, Debug)]
pub struct BlockRef<'a> {
    kernel: &'a VerifiedKernel,
    block: u32,
}

impl<'a> BlockRef<'a> {
    /// Returns the ordered block parameters.
    #[must_use]
    pub fn parameters(self) -> impl ExactSizeIterator<Item = VerifiedValueId> + 'a {
        let kernel = self.kernel;
        self.data()
            .parameters
            .iter()
            .map(move |index| kernel.value_id(*index))
    }

    /// Returns the ordered operations of this block.
    #[must_use]
    pub fn operations(self) -> impl ExactSizeIterator<Item = OperationRef<'a>> + 'a {
        let kernel = self.kernel;
        let block = self.block;
        (0..self.data().operations.len()).map(move |position| OperationRef {
            kernel,
            block,
            position,
        })
    }

    fn data(self) -> &'a BlockData {
        &self.kernel.data.blocks[self.block as usize]
    }
}

/// A read-only view of one structured operation.
#[derive(Clone, Copy, Debug)]
pub struct OperationRef<'a> {
    kernel: &'a VerifiedKernel,
    block: u32,
    position: usize,
}

impl<'a> OperationRef<'a> {
    /// Returns the ordered result values this operation defines.
    #[must_use]
    pub fn results(self) -> impl ExactSizeIterator<Item = VerifiedValueId> + 'a {
        let kernel = self.kernel;
        self.data()
            .results
            .iter()
            .map(move |index| kernel.value_id(*index))
    }

    /// Returns the typed view of this operation's kind and operands.
    #[must_use]
    pub fn view(self) -> OperationView<'a> {
        let kernel = self.kernel;
        match &self.data().kind {
            OperationKind::Builtin { builtin } => OperationView::Builtin { builtin: *builtin },
            OperationKind::Constant { value } => OperationView::Constant { value: *value },
            OperationKind::Binary { op, lhs, rhs } => OperationView::Binary {
                op: *op,
                lhs: kernel.value_id(*lhs),
                rhs: kernel.value_id(*rhs),
            },
            OperationKind::Compare { op, lhs, rhs } => OperationView::Compare {
                op: *op,
                lhs: kernel.value_id(*lhs),
                rhs: kernel.value_id(*rhs),
            },
            OperationKind::Convert { op, source } => OperationView::Convert {
                op: *op,
                source: kernel.value_id(*source),
            },
            OperationKind::Load {
                buffer,
                offset,
                bounds,
            } => OperationView::Load {
                buffer: kernel.buffer_id(*buffer),
                offset: kernel.value_id(*offset),
                bounds: *bounds,
            },
            OperationKind::Store {
                buffer,
                offset,
                value,
                bounds,
                ownership,
            } => OperationView::Store {
                buffer: kernel.buffer_id(*buffer),
                offset: kernel.value_id(*offset),
                value: kernel.value_id(*value),
                bounds: *bounds,
                ownership: *ownership,
            },
            OperationKind::Predicated { predicate, body } => OperationView::Predicated {
                predicate: kernel.value_id(*predicate),
                body: BlockRef {
                    kernel,
                    block: *body,
                },
            },
            OperationKind::SerialLoop {
                start,
                end,
                initial,
                body,
                yields,
            } => OperationView::SerialLoop(SerialLoopRef {
                kernel,
                start: *start,
                end: *end,
                initial,
                yields,
                block: *body,
            }),
            OperationKind::Barrier { spec } => OperationView::Barrier { spec },
        }
    }

    fn data(self) -> &'a OperationData {
        &self.kernel.data.blocks[self.block as usize].operations[self.position]
    }
}

/// The typed view of one structured operation.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum OperationView<'a> {
    /// Reads an admitted launch builtin.
    Builtin {
        /// The governed builtin read.
        builtin: Builtin,
    },
    /// Defines a typed immediate constant.
    Constant {
        /// The immediate value.
        value: KernelConstant,
    },
    /// Applies a pure binary operation.
    Binary {
        /// The applied operation.
        op: BinaryOp,
        /// Left operand.
        lhs: VerifiedValueId,
        /// Right operand.
        rhs: VerifiedValueId,
    },
    /// Applies a predicate-producing comparison.
    Compare {
        /// The applied comparison.
        op: CompareOp,
        /// Left operand.
        lhs: VerifiedValueId,
        /// Right operand.
        rhs: VerifiedValueId,
    },
    /// Applies a named typed conversion.
    Convert {
        /// The applied conversion contract.
        op: ConvertOp,
        /// Converted source value.
        source: VerifiedValueId,
    },
    /// Loads one element under schedule-derived bounds evidence.
    Load {
        /// Buffer read.
        buffer: VerifiedBufferId,
        /// Element offset within the buffer.
        offset: VerifiedValueId,
        /// Schedule bounds witness authorizing the access.
        bounds: BoundsWitnessId,
    },
    /// Stores one element under bounds and write-ownership evidence.
    Store {
        /// Buffer written.
        buffer: VerifiedBufferId,
        /// Element offset within the buffer.
        offset: VerifiedValueId,
        /// Stored value.
        value: VerifiedValueId,
        /// Schedule bounds witness authorizing the access.
        bounds: BoundsWitnessId,
        /// Schedule write-ownership witness authorizing the commit.
        ownership: OwnershipWitnessId,
    },
    /// Executes a nested block when a predicate holds.
    Predicated {
        /// Controlling predicate.
        predicate: VerifiedValueId,
        /// Guarded block.
        body: BlockRef<'a>,
    },
    /// Executes a bounded loop carrying typed accumulator state.
    SerialLoop(SerialLoopRef<'a>),
    /// Synchronizes an execution scope and fences named address spaces.
    Barrier {
        /// The barrier's separately named scopes, fences, and ordering.
        spec: &'a BarrierSpec,
    },
}

/// A read-only view of one bounded structured loop.
///
/// The loop body binds an induction variable followed by one parameter per
/// carried accumulator; the yielded values become the accumulators of the next
/// iteration and, after the final iteration, the loop's results.
#[derive(Clone, Copy, Debug)]
pub struct SerialLoopRef<'a> {
    kernel: &'a VerifiedKernel,
    start: u64,
    end: u64,
    initial: &'a [u32],
    yields: &'a [u32],
    block: u32,
}

impl<'a> SerialLoopRef<'a> {
    /// Returns the inclusive first induction value.
    #[must_use]
    pub const fn start(self) -> u64 {
        self.start
    }

    /// Returns the exclusive last induction value.
    #[must_use]
    pub const fn end(self) -> u64 {
        self.end
    }

    /// Returns the induction variable bound in the loop body, when present.
    #[must_use]
    pub fn induction(self) -> Option<VerifiedValueId> {
        self.body_data()
            .parameters
            .first()
            .map(|index| self.kernel.value_id(*index))
    }

    /// Returns the ordered accumulator parameters bound in the loop body.
    #[must_use]
    pub fn accumulators(self) -> impl ExactSizeIterator<Item = VerifiedValueId> + 'a {
        let kernel = self.kernel;
        self.body_data()
            .parameters
            .get(1..)
            .unwrap_or_default()
            .iter()
            .map(move |index| kernel.value_id(*index))
    }

    /// Returns the ordered initial accumulator values.
    #[must_use]
    pub fn initial(self) -> impl ExactSizeIterator<Item = VerifiedValueId> + 'a {
        let kernel = self.kernel;
        self.initial
            .iter()
            .map(move |index| kernel.value_id(*index))
    }

    /// Returns the ordered values yielded at the end of one iteration.
    #[must_use]
    pub fn yields(self) -> impl ExactSizeIterator<Item = VerifiedValueId> + 'a {
        let kernel = self.kernel;
        self.yields.iter().map(move |index| kernel.value_id(*index))
    }

    /// Returns the loop body block.
    #[must_use]
    pub const fn body(self) -> BlockRef<'a> {
        BlockRef {
            kernel: self.kernel,
            block: self.block,
        }
    }

    fn body_data(self) -> &'a BlockData {
        &self.kernel.data.blocks[self.block as usize]
    }
}

fn push_len(bytes: &mut Vec<u8>, len: usize) {
    bytes.extend_from_slice(&(len as u64).to_be_bytes());
}

fn push_slice(bytes: &mut Vec<u8>, value: &[u8]) {
    push_len(bytes, value.len());
    bytes.extend_from_slice(value);
}

fn push_tensor_role(bytes: &mut Vec<u8>, role: TensorRole) {
    bytes.push(match role {
        TensorRole::Input => 0x01,
        TensorRole::Intermediate => 0x02,
        TensorRole::Output => 0x03,
    });
}

fn push_subnormal(bytes: &mut Vec<u8>, mode: SubnormalMode) {
    bytes.push(match mode {
        SubnormalMode::Preserve => 0x01,
    });
}

fn push_permission(bytes: &mut Vec<u8>, permission: NumericalPermission) {
    bytes.push(match permission {
        NumericalPermission::Forbidden => 0x01,
    });
}

fn push_numerical(bytes: &mut Vec<u8>, numerical: &NumericalRealization) {
    push_slice(bytes, numerical.profile_key.as_bytes());
    bytes.extend_from_slice(&numerical.canonical_arithmetic_nan_bits.to_be_bytes());
    push_subnormal(bytes, numerical.input_subnormals);
    push_subnormal(bytes, numerical.result_subnormals);
    push_permission(bytes, numerical.contraction);
    push_permission(bytes, numerical.reassociation);
}

fn push_requirements(bytes: &mut Vec<u8>, requirements: &ResourceRequirements) {
    bytes.extend_from_slice(&requirements.buffer_bindings.to_be_bytes());
    bytes.extend_from_slice(&requirements.threads_per_workgroup.to_be_bytes());
    bytes.extend_from_slice(&requirements.local_memory_bytes.to_be_bytes());
    bytes.extend_from_slice(&requirements.barriers.to_be_bytes());
    bytes.push(u8::from(requirements.requires_device_memory));
    bytes.push(u8::from(requirements.requires_strict_f32));
}

fn push_buffer(bytes: &mut Vec<u8>, buffer: &BufferParameter) {
    push_tensor_role(bytes, buffer.tensor);
    bytes.push(buffer.element_type.tag());
    bytes.push(buffer.address_space.tag());
    bytes.push(buffer.access.tag());
    bytes.extend_from_slice(&buffer.element_count.to_be_bytes());
}

fn push_constant(bytes: &mut Vec<u8>, value: KernelConstant) {
    match value {
        KernelConstant::Bool(flag) => {
            bytes.push(0x01);
            bytes.push(u8::from(flag));
        }
        KernelConstant::Index(index) => {
            bytes.push(0x02);
            bytes.extend_from_slice(&index.to_be_bytes());
        }
        KernelConstant::F32Bits(pattern) => {
            bytes.push(0x03);
            bytes.extend_from_slice(&pattern.to_be_bytes());
        }
    }
}

fn push_barrier(bytes: &mut Vec<u8>, spec: &BarrierSpec) {
    bytes.push(spec.execution_scope.tag());
    bytes.push(spec.memory_scope.tag());
    push_len(bytes, spec.fenced_spaces.len());
    for space in &spec.fenced_spaces {
        bytes.push(space.tag());
    }
    bytes.push(spec.ordering.tag());
}

fn push_indices(bytes: &mut Vec<u8>, indices: &[u32]) {
    push_len(bytes, indices.len());
    for index in indices {
        bytes.extend_from_slice(&index.to_be_bytes());
    }
}

fn push_operation(bytes: &mut Vec<u8>, data: &KernelData, operation: &OperationData) {
    match &operation.kind {
        OperationKind::Builtin { builtin } => {
            bytes.push(0x11);
            bytes.push(builtin.tag());
        }
        OperationKind::Constant { value } => {
            bytes.push(0x12);
            push_constant(bytes, *value);
        }
        OperationKind::Binary { op, lhs, rhs } => {
            bytes.push(0x13);
            bytes.push(op.tag());
            bytes.extend_from_slice(&lhs.to_be_bytes());
            bytes.extend_from_slice(&rhs.to_be_bytes());
        }
        OperationKind::Compare { op, lhs, rhs } => {
            bytes.push(0x14);
            bytes.push(op.tag());
            bytes.extend_from_slice(&lhs.to_be_bytes());
            bytes.extend_from_slice(&rhs.to_be_bytes());
        }
        OperationKind::Convert { op, source } => {
            bytes.push(0x15);
            bytes.push(op.tag());
            bytes.extend_from_slice(&source.to_be_bytes());
        }
        OperationKind::Load {
            buffer,
            offset,
            bounds,
        } => {
            bytes.push(0x16);
            bytes.extend_from_slice(&buffer.to_be_bytes());
            bytes.extend_from_slice(&offset.to_be_bytes());
            bytes.extend_from_slice(&bounds.get().to_be_bytes());
        }
        OperationKind::Store {
            buffer,
            offset,
            value,
            bounds,
            ownership,
        } => {
            bytes.push(0x17);
            bytes.extend_from_slice(&buffer.to_be_bytes());
            bytes.extend_from_slice(&offset.to_be_bytes());
            bytes.extend_from_slice(&value.to_be_bytes());
            bytes.extend_from_slice(&bounds.get().to_be_bytes());
            bytes.extend_from_slice(&ownership.get().to_be_bytes());
        }
        OperationKind::Predicated { predicate, body } => {
            bytes.push(0x18);
            bytes.extend_from_slice(&predicate.to_be_bytes());
            push_block(bytes, data, *body);
        }
        OperationKind::SerialLoop {
            start,
            end,
            initial,
            body,
            yields,
        } => {
            bytes.push(0x19);
            bytes.extend_from_slice(&start.to_be_bytes());
            bytes.extend_from_slice(&end.to_be_bytes());
            push_indices(bytes, initial);
            push_indices(bytes, yields);
            push_block(bytes, data, *body);
        }
        OperationKind::Barrier { spec } => {
            bytes.push(0x1a);
            push_barrier(bytes, spec);
        }
    }
    push_indices(bytes, &operation.results);
}

fn push_block(bytes: &mut Vec<u8>, data: &KernelData, block: u32) {
    let block = &data.blocks[block as usize];
    push_indices(bytes, &block.parameters);
    push_len(bytes, block.operations.len());
    for operation in &block.operations {
        push_operation(bytes, data, operation);
    }
}

/// Encodes the canonical identity of one verified structured kernel.
///
/// # Errors
///
/// Returns [`KernelDiagnostic::IdentityLimit`] when the encoding exceeds the
/// governed byte bound.
pub(super) fn encode_identity(
    schedule_identity: &CanonicalScheduledRegionIdentity,
    data: &KernelData,
) -> Result<CanonicalKernelIdentity, KernelDiagnostic> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.kernel.v1\0");
    push_slice(&mut bytes, schedule_identity.as_bytes());
    push_len(&mut bytes, data.buffers.len());
    for buffer in &data.buffers {
        push_buffer(&mut bytes, buffer);
    }
    push_len(&mut bytes, data.admitted_builtins.len());
    for builtin in &data.admitted_builtins {
        bytes.push(builtin.tag());
    }
    push_numerical(&mut bytes, &data.numerical);
    push_requirements(&mut bytes, &data.requirements);
    push_len(&mut bytes, data.values.len());
    for value in &data.values {
        bytes.push(value.value_type.tag());
    }
    push_block(&mut bytes, data, 0);
    if bytes.len() > MAX_KERNEL_IDENTITY_BYTES {
        return Err(KernelDiagnostic::IdentityLimit {
            bytes: bytes.len(),
            limit: MAX_KERNEL_IDENTITY_BYTES,
        });
    }
    Ok(CanonicalKernelIdentity(bytes))
}
