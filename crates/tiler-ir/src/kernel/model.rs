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

use crate::identity::{push_len, push_slice};
use crate::schedule::{BoundsWitnessId, OwnershipWitnessId};
use crate::schedule::{
    CanonicalScheduledRegionIdentity, ExceptionalValueAssumption, FlushedZeroSign,
    NumericalPermission, NumericalRealization, RegionId, ResourceRequirements, SubnormalMode,
    TensorRole, ValueDomainProvenance,
};
use crate::semantic::EncodedComponentRole;

use super::MAX_KERNEL_IDENTITY_BYTES;

/// The versioned domain separator opening a canonical kernel identity.
///
/// Named so [`encode_identity`] and [`identity_encoded_len`] measure the same
/// bytes rather than agreeing on a literal by inspection.
///
/// `v5` gives a buffer parameter's input role its ordinal, so a kernel can bind
/// several distinct input tensors. The ordinal lands inside the repeated
/// buffer-parameter record, so a v4 reader would consume the component-role tag
/// at the old offset and lose framing for every buffer after it — and every
/// kernel ever encoded maps to different bytes now, which is what a cache or
/// artifact holding a v4 identity must miss on rather than match.
///
/// `v4` removes the invalid numeric barrier-capacity field from the fixed
/// resource-requirement record. A v3 reader would otherwise consume the
/// following fields at the old offset.
///
/// `v2` appended the unsigned-byte access/SSA type. `v3` additionally binds
/// semantic component roles to buffer parameters and admits the signed-I32
/// arithmetic, explicit conversions, and packed-U4 extraction required by the
/// strict-affine dequantization proof. Existing type and operation tags retain
/// their earlier values and new variants receive appended tags. These changes
/// advance the domain rather than letting an earlier reader interpret the
/// kernel incompletely.
const KERNEL_DOMAIN: &[u8] = b"tiler.kernel.v5\0";

/// The width [`push_len`] frames a length in, as ADR 0074 fixes it.
const LENGTH_BYTES: usize = size_of::<u64>();
use super::error::{KernelDiagnostic, KernelEntityKind, VerifiedKernelHandleError};
use super::handles::{VerifiedBufferId, VerifiedKernelOwner, VerifiedValueId};

/// The resolved type of one structured-kernel SSA value.
///
/// The bounded profile resolves exactly the roles the scheduled-region IR can
/// require: a control predicate, an unsigned byte carrier, an unsigned 64-bit
/// index role used for element offsets and induction variables, and the `f32`
/// element type. Widening this
/// vocabulary is a versioned extension, not an open type universe.
///
/// **Deliberately not `#[non_exhaustive]`, and this one is mandatory rather
/// than a judgement.** `tiler-artifact` encodes this vocabulary into
/// `CanonicalArtifactProgramIdentity` — a cross-crate *total map*, where every
/// variant must yield its own distinct encoding. ADR 0074 convention 5b makes
/// that exhaustive, because the two failure modes differ in kind: an
/// incomplete recognizer silently fails to reach a backend, while an
/// incomplete total map silently gives two structurally different subjects the
/// same identity bytes. The attribute would make the second unreachable at
/// compile time, so widening this enum must be a build error at every encoder
/// that has to decide what the new variant means.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KernelType {
    /// A one-bit control predicate.
    Bool,
    /// An unsigned eight-bit storage carrier.
    ///
    /// This is a kernel access and SSA type, not a semantic `u8` tensor type.
    U8,
    /// An unsigned 64-bit index-role integer.
    Index,
    /// An IEEE-754 binary32 value.
    F32,
    /// A signed 32-bit integer computation value.
    I32,
}

impl KernelType {
    const fn tag(self) -> u8 {
        match self {
            Self::Bool => 0x01,
            Self::Index => 0x02,
            Self::F32 => 0x03,
            Self::U8 => 0x04,
            Self::I32 => 0x05,
        }
    }
}

/// A governed memory visibility and lifetime domain.
///
/// These describe visibility and lifetime in target-neutral terms. A target
/// profile maps a supported governed space onto its own realization or rejects
/// it; caches and registers are lowering facts, not address spaces.
///
/// Not `#[non_exhaustive]`, for the reason [`KernelType`] states: this
/// vocabulary is a cross-crate total map into artifact identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
///
/// Not `#[non_exhaustive]`, for the reason [`KernelType`] states: this
/// vocabulary is a cross-crate total map into artifact identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    /// Semantic component role, or `None` for a dense value.
    pub component_role: Option<EncodedComponentRole>,
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
    /// Exact signed 32-bit subtraction.
    I32Subtract,
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
            Self::I32Subtract => 0x07,
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
            Self::I32Subtract => KernelType::I32,
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
    /// Exactly widens an unsigned byte into signed I32.
    U8ToI32,
    /// Exactly converts an I32 in `-255..=255` into F32.
    I32ToF32,
}

impl ConvertOp {
    const fn tag(self) -> u8 {
        match self {
            Self::CanonicalizeF32Nan => 0x01,
            Self::U8ToI32 => 0x02,
            Self::I32ToF32 => 0x03,
        }
    }

    /// Returns the required source type.
    #[must_use]
    pub const fn source_type(self) -> KernelType {
        match self {
            Self::CanonicalizeF32Nan => KernelType::F32,
            Self::U8ToI32 => KernelType::U8,
            Self::I32ToF32 => KernelType::I32,
        }
    }

    /// Returns the produced result type.
    #[must_use]
    pub const fn result_type(self) -> KernelType {
        match self {
            Self::U8ToI32 => KernelType::I32,
            Self::CanonicalizeF32Nan | Self::I32ToF32 => KernelType::F32,
        }
    }
}

/// Exact extraction from one governed packed byte encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PackedExtractOp {
    /// Selects the low nibble for an even logical index and the high nibble for
    /// an odd index under LSB-first U4 packing with a canonical zero tail.
    U4LsbZeroTail,
}

impl PackedExtractOp {
    const fn tag(self) -> u8 {
        match self {
            Self::U4LsbZeroTail => 0x01,
        }
    }

    /// Returns the carrier input type.
    #[must_use]
    pub const fn carrier_type(self) -> KernelType {
        match self {
            Self::U4LsbZeroTail => KernelType::U8,
        }
    }

    /// Returns the logical-index input type.
    #[must_use]
    pub const fn index_type(self) -> KernelType {
        match self {
            Self::U4LsbZeroTail => KernelType::Index,
        }
    }

    /// Returns the unpacked code type.
    #[must_use]
    pub const fn result_type(self) -> KernelType {
        match self {
            Self::U4LsbZeroTail => KernelType::U8,
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
    PackedExtract {
        op: PackedExtractOp,
        carrier: u32,
        logical_index: u32,
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

    /// Returns each declared buffer parameter with the handle naming it.
    ///
    /// Declaration order, which *is* the signature order: a parameter's position
    /// here is its argument-table ordinal.
    ///
    /// [`Self::buffers`] yields the parameters alone and cannot say which handle
    /// each belongs to, so a backend building a signature from it has no way to
    /// relate a parameter to the loads and stores that reference it. The only
    /// alternative is to assign ordinals in first-use order, which disagrees
    /// with this order for any kernel whose body does not happen to touch its
    /// buffers in declaration sequence, and which cannot place a parameter the
    /// body never touches at all.
    #[must_use]
    pub fn declared_buffers(
        &self,
    ) -> impl ExactSizeIterator<Item = (VerifiedBufferId, BufferParameter)> + '_ {
        // Counted in `u32`, the width a handle's ordinal is, so no fallible
        // conversion appears in the mapping and this cannot panic. `zip` stops
        // at the shorter side, so the saturating count below cannot invent a
        // parameter even in the unrepresentable case it guards against: a
        // kernel with more than `u32::MAX` buffers has no handle for the last
        // of them and could not have been verified.
        let count = u32::try_from(self.data.buffers.len()).unwrap_or(u32::MAX);
        (0..count)
            .zip(self.data.buffers.iter().copied())
            .map(|(index, parameter)| (self.buffer_id(index), parameter))
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
            OperationKind::PackedExtract {
                op,
                carrier,
                logical_index,
            } => OperationView::PackedExtract {
                op: *op,
                carrier: kernel.value_id(*carrier),
                logical_index: kernel.value_id(*logical_index),
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
    /// Extracts one logical packed value from its loaded carrier byte.
    PackedExtract {
        /// Exact governed packing rule.
        op: PackedExtractOp,
        /// Loaded carrier byte.
        carrier: VerifiedValueId,
        /// Logical element index selecting the nibble.
        logical_index: VerifiedValueId,
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

/// Encodes one buffer parameter's boundary tensor role.
///
/// A second, independent copy of the scheduled-region role table by design: the
/// two identities are different subjects, and a shared encoder would let one
/// domain's step silently move the other's bytes. Adding a role is a build error
/// at both.
fn push_tensor_role(bytes: &mut Vec<u8>, role: TensorRole) {
    match role {
        TensorRole::Input { ordinal } => {
            bytes.push(0x01);
            bytes.extend_from_slice(&ordinal.get().to_be_bytes());
        }
        TensorRole::Intermediate => bytes.push(0x02),
        TensorRole::Output => bytes.push(0x03),
    }
}

/// Mirrors [`push_tensor_role`]: one tag byte, plus an ordinal for an input.
const fn tensor_role_encoded_len(role: TensorRole) -> usize {
    match role {
        TensorRole::Input { ordinal } => 1 + size_of_val(&ordinal.get()),
        TensorRole::Intermediate | TensorRole::Output => 1,
    }
}

fn push_subnormal(bytes: &mut Vec<u8>, mode: SubnormalMode) {
    bytes.push(match mode {
        SubnormalMode::Preserve => 0x01,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        } => 0x02,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        } => 0x03,
    });
}

fn push_permission(bytes: &mut Vec<u8>, permission: NumericalPermission) {
    bytes.push(match permission {
        NumericalPermission::Forbidden => 0x01,
        NumericalPermission::Permitted => 0x02,
    });
}

fn push_exceptional_assumption(bytes: &mut Vec<u8>, assumption: ExceptionalValueAssumption) {
    match assumption {
        ExceptionalValueAssumption::MakeNoAssumption => bytes.push(0x01),
        ExceptionalValueAssumption::AssumeAbsent { provenance } => {
            bytes.push(0x02);
            bytes.push(match provenance {
                ValueDomainProvenance::CompilerProven => 0x01,
                ValueDomainProvenance::RuntimeValidated => 0x02,
                ValueDomainProvenance::CallerDeclaredUnvalidated => 0x03,
            });
        }
    }
}

fn push_numerical(bytes: &mut Vec<u8>, numerical: &NumericalRealization) {
    push_slice(bytes, numerical.profile_key.as_bytes());
    bytes.extend_from_slice(&numerical.canonical_arithmetic_nan_bits.to_be_bytes());
    push_subnormal(bytes, numerical.input_subnormals);
    push_subnormal(bytes, numerical.result_subnormals);
    push_permission(bytes, numerical.contraction);
    push_permission(bytes, numerical.reassociation);
    push_permission(bytes, numerical.permutation);
    push_permission(bytes, numerical.signed_zero);
    push_exceptional_assumption(bytes, numerical.nan_assumptions);
    push_exceptional_assumption(bytes, numerical.infinity_assumptions);
}

fn push_requirements(bytes: &mut Vec<u8>, requirements: &ResourceRequirements) {
    bytes.extend_from_slice(&requirements.buffer_bindings.to_be_bytes());
    bytes.extend_from_slice(&requirements.threads_per_workgroup.to_be_bytes());
    bytes.extend_from_slice(&requirements.local_memory_bytes.to_be_bytes());
    bytes.push(u8::from(requirements.requires_device_memory));
    push_subnormal(bytes, requirements.input_subnormals);
    push_subnormal(bytes, requirements.result_subnormals);
    push_permission(bytes, requirements.contraction);
    push_permission(bytes, requirements.reassociation);
    push_permission(bytes, requirements.permutation);
    push_permission(bytes, requirements.signed_zero);
    push_exceptional_assumption(bytes, requirements.nan_assumptions);
    push_exceptional_assumption(bytes, requirements.infinity_assumptions);
}

fn push_buffer(bytes: &mut Vec<u8>, buffer: &BufferParameter) {
    push_tensor_role(bytes, buffer.tensor);
    push_component_role(bytes, buffer.component_role);
    bytes.push(buffer.element_type.tag());
    bytes.push(buffer.address_space.tag());
    bytes.push(buffer.access.tag());
    bytes.extend_from_slice(&buffer.element_count.to_be_bytes());
}

fn push_component_role(bytes: &mut Vec<u8>, role: Option<EncodedComponentRole>) {
    match role {
        None => bytes.push(0x00),
        Some(role) => {
            bytes.push(0x01);
            bytes.extend_from_slice(&role.get().to_be_bytes());
        }
    }
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
        OperationKind::PackedExtract {
            op,
            carrier,
            logical_index,
        } => {
            bytes.push(0x1b);
            bytes.push(op.tag());
            bytes.extend_from_slice(&carrier.to_be_bytes());
            bytes.extend_from_slice(&logical_index.to_be_bytes());
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
    let encoded_len = identity_encoded_len(schedule_identity, data);
    let mut bytes = Vec::with_capacity(encoded_len);
    bytes.extend_from_slice(KERNEL_DOMAIN);
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
    debug_assert_eq!(
        bytes.len(),
        encoded_len,
        "the reserved kernel-identity length must equal what the encoder wrote"
    );
    if bytes.len() > MAX_KERNEL_IDENTITY_BYTES {
        return Err(KernelDiagnostic::IdentityLimit {
            bytes: bytes.len(),
            limit: MAX_KERNEL_IDENTITY_BYTES,
        });
    }
    Ok(CanonicalKernelIdentity(bytes))
}

/// The exact byte length [`encode_identity`] will write.
///
/// **Every arm mirrors one arm of the encoder above it**, so the two are read
/// as a pair and a new [`OperationKind`] variant is a compile error in both.
/// Where the encoder writes an integer field, this measures *that field* with
/// [`size_of_val`] rather than restating a width, so widening a field cannot
/// silently desynchronize the reservation from the encoding. The
/// `debug_assert_eq!` in [`encode_identity`] is the backstop for the rest.
///
/// This exists to size the identity buffer once. Growing it by doubling
/// reallocated and copied the encoding several times per kernel, and a kernel
/// identity is encoded on every build and every refinement check.
fn identity_encoded_len(
    schedule_identity: &CanonicalScheduledRegionIdentity,
    data: &KernelData,
) -> usize {
    KERNEL_DOMAIN
        .len()
        .saturating_add(LENGTH_BYTES)
        .saturating_add(schedule_identity.as_bytes().len())
        .saturating_add(LENGTH_BYTES)
        .saturating_add(
            data.buffers
                .iter()
                .map(buffer_encoded_len)
                .fold(0_usize, usize::saturating_add),
        )
        .saturating_add(LENGTH_BYTES)
        // One tag byte per admitted builtin.
        .saturating_add(data.admitted_builtins.len())
        .saturating_add(numerical_encoded_len(&data.numerical))
        .saturating_add(requirements_encoded_len(&data.requirements))
        .saturating_add(LENGTH_BYTES)
        // One tag byte per value type.
        .saturating_add(data.values.len())
        .saturating_add(block_encoded_len(data, 0))
}

/// Mirrors [`push_buffer`]: tensor, optional role, three tags, and element count.
fn buffer_encoded_len(buffer: &BufferParameter) -> usize {
    let component = if buffer.component_role.is_some() {
        5
    } else {
        1
    };
    tensor_role_encoded_len(buffer.tensor)
        // The element type, address space, and access tags.
        .saturating_add(3)
        .saturating_add(component)
        .saturating_add(size_of_val(&buffer.element_count))
}

/// Mirrors [`push_numerical`].
fn numerical_encoded_len(numerical: &NumericalRealization) -> usize {
    LENGTH_BYTES
        .saturating_add(numerical.profile_key.len())
        .saturating_add(size_of_val(&numerical.canonical_arithmetic_nan_bits))
        // Two subnormal modes and four permissions, one tag byte each.
        .saturating_add(6)
        .saturating_add(exceptional_assumption_encoded_len(
            numerical.nan_assumptions,
        ))
        .saturating_add(exceptional_assumption_encoded_len(
            numerical.infinity_assumptions,
        ))
}

/// Mirrors [`push_exceptional_assumption`].
const fn exceptional_assumption_encoded_len(assumption: ExceptionalValueAssumption) -> usize {
    match assumption {
        ExceptionalValueAssumption::MakeNoAssumption => 1,
        ExceptionalValueAssumption::AssumeAbsent { .. } => 2,
    }
}

/// Mirrors [`push_requirements`].
fn requirements_encoded_len(requirements: &ResourceRequirements) -> usize {
    size_of_val(&requirements.buffer_bindings)
        .saturating_add(size_of_val(&requirements.threads_per_workgroup))
        .saturating_add(size_of_val(&requirements.local_memory_bytes))
        // The device-memory flag, two subnormal modes, and four permissions.
        .saturating_add(7)
        .saturating_add(exceptional_assumption_encoded_len(
            requirements.nan_assumptions,
        ))
        .saturating_add(exceptional_assumption_encoded_len(
            requirements.infinity_assumptions,
        ))
}

/// Mirrors [`push_constant`]: a discriminant tag and the payload.
fn constant_encoded_len(value: KernelConstant) -> usize {
    1_usize.saturating_add(match value {
        KernelConstant::Bool(_) => 1,
        KernelConstant::Index(index) => size_of_val(&index),
        KernelConstant::F32Bits(pattern) => size_of_val(&pattern),
    })
}

/// Mirrors [`push_barrier`].
fn barrier_encoded_len(spec: &BarrierSpec) -> usize {
    // Execution scope, memory scope, then the framed fenced spaces at one tag
    // byte each, then the ordering.
    2_usize
        .saturating_add(LENGTH_BYTES)
        .saturating_add(spec.fenced_spaces.len())
        .saturating_add(1)
}

/// Mirrors [`push_indices`].
fn indices_encoded_len(indices: &[u32]) -> usize {
    LENGTH_BYTES.saturating_add(indices.len().saturating_mul(size_of::<u32>()))
}

/// Mirrors [`push_block`], recursing through nested blocks exactly as it does.
fn block_encoded_len(data: &KernelData, block: u32) -> usize {
    let block = &data.blocks[block as usize];
    indices_encoded_len(&block.parameters)
        .saturating_add(LENGTH_BYTES)
        .saturating_add(
            block
                .operations
                .iter()
                .map(|operation| operation_encoded_len(data, operation))
                .fold(0_usize, usize::saturating_add),
        )
}

/// Mirrors [`push_operation`], one arm per encoded operation kind.
fn operation_encoded_len(data: &KernelData, operation: &OperationData) -> usize {
    let kind = match &operation.kind {
        OperationKind::Builtin { .. } => 1,
        OperationKind::Constant { value } => constant_encoded_len(*value),
        OperationKind::Binary { lhs, rhs, .. } | OperationKind::Compare { lhs, rhs, .. } => 1_usize
            .saturating_add(size_of_val(lhs))
            .saturating_add(size_of_val(rhs)),
        OperationKind::Convert { source, .. } => 1_usize.saturating_add(size_of_val(source)),
        OperationKind::PackedExtract {
            carrier,
            logical_index,
            ..
        } => 1_usize
            .saturating_add(size_of_val(carrier))
            .saturating_add(size_of_val(logical_index)),
        OperationKind::Load {
            buffer,
            offset,
            bounds,
        } => size_of_val(buffer)
            .saturating_add(size_of_val(offset))
            .saturating_add(size_of_val(&bounds.get())),
        OperationKind::Store {
            buffer,
            offset,
            value,
            bounds,
            ownership,
        } => size_of_val(buffer)
            .saturating_add(size_of_val(offset))
            .saturating_add(size_of_val(value))
            .saturating_add(size_of_val(&bounds.get()))
            .saturating_add(size_of_val(&ownership.get())),
        OperationKind::Predicated { predicate, body } => {
            size_of_val(predicate).saturating_add(block_encoded_len(data, *body))
        }
        OperationKind::SerialLoop {
            start,
            end,
            initial,
            body,
            yields,
        } => size_of_val(start)
            .saturating_add(size_of_val(end))
            .saturating_add(indices_encoded_len(initial))
            .saturating_add(indices_encoded_len(yields))
            .saturating_add(block_encoded_len(data, *body)),
        OperationKind::Barrier { spec } => barrier_encoded_len(spec),
    };
    // Every arm is preceded by its one-byte kind tag and followed by results.
    1_usize
        .saturating_add(kind)
        .saturating_add(indices_encoded_len(&operation.results))
}
