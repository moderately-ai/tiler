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
    CanonicalScheduledRegionIdentity, ExceptionalValueAssumption, FlushedZeroSign, IndexArithmetic,
    NumericalPermission, NumericalRealization, REGION_INDEX_ARITHMETIC, RegionId,
    ResourceRequirements, SubgroupRealizationSubject, SubnormalFreedom, SubnormalMode,
    SynchronizationSubject, TensorRole, ValueDomainProvenance,
};
use crate::semantic::EncodedComponentRole;
use crate::shape::Axis;

use super::MAX_KERNEL_IDENTITY_BYTES;

/// The versioned domain separator opening a canonical kernel identity.
///
/// Named so [`encode_identity`] and [`identity_encoded_len`] measure the same
/// bytes rather than agreeing on a literal by inspection.
///
/// `v6` gives the fixed resource-requirement record its synchronization
/// requirement — the complete
/// [`SynchronizationSubject`](crate::schedule::SynchronizationSubject) a
/// region's schedule obliges a target to realize, or its canonical absence.
/// The field lands at the end of a record that is *followed* by the value-type
/// table and the whole body, so a `v5` reader would consume the presence tag as
/// the first value-count byte and lose framing for everything after it. Every
/// kernel ever encoded maps to different bytes now, and that is the point: a
/// cache or artifact holding a `v5` identity must miss rather than match a
/// kernel whose synchronization obligation the earlier record could not state —
/// including, and especially, the zero-synchronization kernels whose absence is
/// now an encoded fact rather than an unstated one.
///
/// The staged-access and barrier operation encodings moved with it and cost
/// nothing extra: no `v5` kernel could contain a `Barrier`, because the verifier
/// refused every one intrinsically, and staged loads and stores did not exist.
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
///
/// `v7` gives every kernel's fixed resource-requirement record its derived
/// index-arithmetic requirement, written between the device-memory flag and the
/// synchronization record. The tag lands *inside* the fixed record rather than
/// after it, so a v6 reader handed v7 bytes would consume the index-arithmetic
/// tag where the synchronization presence byte belongs and lose framing for
/// every field after it — and every kernel ever encoded maps to different bytes
/// now, which is what a cache or artifact holding a v6 identity must miss on
/// rather than match. Appending would not have avoided the step either: the
/// requirement is a fact about the kernel a v6 identity could not state, so two
/// kernels differing only in it were one subject.
const KERNEL_DOMAIN: &[u8] = b"tiler.kernel.v7\0";

/// The width [`push_len`] frames a length in, as ADR 0074 fixes it.
const LENGTH_BYTES: usize = size_of::<u64>();
use super::error::{KernelDiagnostic, KernelEntityKind, VerifiedKernelHandleError};
use super::handles::{
    VerifiedBufferId, VerifiedInputExtentId, VerifiedKernelOwner, VerifiedStagingId,
    VerifiedValueId,
};

/// The resolved type of one structured-kernel SSA value.
///
/// The bounded profile resolves exactly the roles the scheduled-region IR can
/// require: a control predicate, an unsigned byte carrier, an unsigned 64-bit
/// index role used for element offsets and induction variables, a signed 32-bit
/// computation type, and the `f32` and `bf16` element types. Widening this
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
    /// A bfloat16 value: binary32's sign and exponent over a seven-bit
    /// significand, in two bytes.
    ///
    /// A `Bf16`-typed value is now produced: a region whose scalar program is
    /// [`ScalarProgram::PointwiseBf16`](crate::schedule::ScalarProgram::PointwiseBf16)
    /// lowers to a kernel whose buffers, constants, arithmetic, and NaN
    /// canonicalization are all `Bf16`. `crates/tiler-metal` emits this type as
    /// `bfloat`: its exact sixteen-bit constants are reinterpreted through
    /// `ushort`, its multiply and add use `bfloat` arithmetic, and its NaNs use
    /// a separate `bfloat` canonicalization helper.
    ///
    /// The evidence is deliberately bounded. Those are the three supported
    /// operations, and backend execution has been checked only on the declared
    /// macOS Apple9 profile row. This type does not imply support on another
    /// target family, for conversion or contraction, or through a complete
    /// `compile()`/artifact/routing path.
    Bf16,
}

impl KernelType {
    const fn tag(self) -> u8 {
        match self {
            Self::Bool => 0x01,
            Self::Index => 0x02,
            Self::F32 => 0x03,
            Self::U8 => 0x04,
            Self::I32 => 0x05,
            // Appended, like `Builtin::LocalInvocationIndex`: every earlier tag
            // keeps its value and every field keeps its position, so no
            // previously encodable kernel's bytes move and the kernel identity
            // domain does not step. No kernel the earlier vocabulary could
            // express contains `0x06`.
            Self::Bf16 => 0x06,
        }
    }
}

impl IndexArithmetic {
    /// Derives the index arithmetic one governed KIR value type requires.
    ///
    /// Exhaustive and wildcard-free, so a widened [`KernelType`] is an `E0004`
    /// here — in the module that declares it — rather than a type silently
    /// inheriting whichever answer it resembles.
    ///
    /// Only the index role yields a requirement, and a type answering `None` is
    /// making the narrow claim that it needs no *index* arithmetic, not that it
    /// needs no target capability at all. [`KernelType::Bf16`] is where the
    /// distinction bites: whether a target can compute in bfloat16 is a separate
    /// profile fact resolved where that arithmetic is proposed, and answering it
    /// from this classifier would read a capability out of a vocabulary that
    /// does not carry one.
    #[must_use]
    pub const fn of(value_type: KernelType) -> Option<Self> {
        match value_type {
            KernelType::Index => Some(Self::CompleteU64),
            KernelType::Bool
            | KernelType::U8
            | KernelType::F32
            | KernelType::I32
            | KernelType::Bf16 => None,
        }
    }
}

/// Compiles only while the region-level constant and the KIR index role agree.
///
/// The scheduled-region layer states its coordinate arithmetic as a constant of
/// its own `u64` coordinate space, and this layer derives the same requirement
/// from the type that arithmetic is actually performed at. Two derivations of
/// one fact are two authorities unless something compares them, and this is the
/// comparison: it lives here because this is the only module that can see both,
/// and it is a `const` assertion rather than a test so a lowering that changed
/// the index role without changing the region constant cannot reach a test run.
const _: () = {
    assert!(
        matches!(
            IndexArithmetic::of(KernelType::Index),
            Some(REGION_INDEX_ARITHMETIC)
        ),
        "the KIR index role and the scheduled region's coordinate arithmetic must agree",
    );
};

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

/// The element type of the boundary values one scheduled region reads and writes.
///
/// The **one** derivation both the canonical lowering and `verify_signature`
/// read, so a kernel's declared buffer types cannot drift from the types the
/// verifier expects. Both used to name [`KernelType::F32`] at each site, which
/// was correct only while every region was `f32`.
///
/// It answers for the region's *dense boundary* values. The strict-affine decode
/// is the one program whose reads are not all one type — its signature is fixed
/// at `[U8, F32, U8]` by `verify_signature`'s own arm and its lowering declares
/// each buffer explicitly — so what this states for that program is the type of
/// its written value, which is the `f32` its scale multiply produces.
///
/// Exhaustive, so a new scalar program states its element type here rather than
/// inheriting whichever one it resembles.
pub(super) const fn region_element_type(program: &crate::schedule::ScalarProgram) -> KernelType {
    match program {
        crate::schedule::ScalarProgram::PointwiseBf16(_) => KernelType::Bf16,
        crate::schedule::ScalarProgram::PointwiseF32(_)
        | crate::schedule::ScalarProgram::StrictAffineU4Dequantize { .. }
        | crate::schedule::ScalarProgram::StrictSerialSum { .. }
        | crate::schedule::ScalarProgram::SquaredSerialSum { .. }
        | crate::schedule::ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | crate::schedule::ScalarProgram::FusedMultiplyAddSerialSum { .. }
        | crate::schedule::ScalarProgram::StrictTensorContraction { .. }
        | crate::schedule::ScalarProgram::StrictSerialMaximum { .. } => KernelType::F32,
    }
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

/// One workgroup-scoped staging allocation a kernel declares.
///
/// **Deliberately not a [`BufferParameter`], and this is a correctness
/// requirement rather than a preference.** A buffer parameter's position *is*
/// its argument-table ordinal, which [`VerifiedKernel::declared_buffers`]
/// documents as positional; a workgroup allocation is not an argument at all,
/// and placing one in that list would re-base every later ordinal and change
/// what an existing signature position means. Keeping the two lists apart also
/// keeps [`BufferAccess`] a two-value vocabulary: staging is read *and* written
/// by the workgroup, which no parameter access mode expresses.
///
/// The allocation names the scheduled [`crate::schedule::StagingId`] it
/// realizes, so a verifier can compare it against the region's cooperative tile
/// rather than trusting a producer's element count.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StagingParameter {
    /// Scheduled staging allocation this realizes.
    pub staging: crate::schedule::StagingId,
    /// Element type every slot holds.
    pub element_type: KernelType,
    /// Governed address space the allocation lives in.
    pub address_space: AddressSpace,
    /// Number of addressable slots.
    pub element_count: u64,
}

/// **Accepted public surface.** Tom accepted this exact spelling on
/// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
/// Dependents may treat this type as accepted vocabulary.
///
/// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
///
/// One live input-axis extent a kernel signature admits as a read-only operand.
///
/// Names the scheduled input and axis whose runtime-bound extent the body may
/// read. The live *value* is not part of kernel identity; the declaration is.
/// The operand is the structured-kernel spelling of the existing
/// [`crate::program::abi::AbiRoot::InputExtent`] root: the kernel names the
/// region-local input and axis, and the artifact maps that ordinal onto the
/// program-interface key. Callers do not supply a second scalar list.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InputExtentParameter {
    /// Scheduled input tensor whose axis extent is read.
    ///
    /// Must be [`TensorRole::Input`]. Any other role is refused at declaration.
    pub tensor: TensorRole,
    /// Axis of that input whose live extent the body may read.
    pub axis: Axis,
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
    /// The linear index of one invocation within its own workgroup.
    ///
    /// The coordinate a cooperative tile's participants are named by. It is a
    /// separate key rather than a derivation from the global index, because the
    /// workgroup width is a launch fact and a kernel that recomputed the local
    /// coordinate from it would carry a second, unchecked copy of that fact.
    LocalInvocationIndex,
}

impl Builtin {
    const fn tag(self) -> u8 {
        match self {
            Self::GlobalInvocationIndex => 0x01,
            // Appended rather than inserted, like `UnaryOp::F32Rsqrt`: the
            // global index keeps `0x01` and every field keeps its position, so
            // no previously encodable kernel's bytes move and the kernel
            // identity domain does not step. A reader that reaches `0x02` is
            // reading a kernel the earlier vocabulary could not express.
            Self::LocalInvocationIndex => 0x02,
        }
    }

    /// Returns the resolved type this builtin produces.
    #[must_use]
    pub const fn result_type(self) -> KernelType {
        match self {
            Self::GlobalInvocationIndex | Self::LocalInvocationIndex => KernelType::Index,
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
    /// A `bf16` constant given by its exact bit pattern.
    ///
    /// **Sixteen bits, not a `u32` carrying a narrow value.** A `bf16` constant
    /// declaring a 32-bit payload is a value the format has no encoding for, and
    /// making the payload the format's own width refuses it at the type rather
    /// than at a check some producer could route around:
    ///
    /// ```compile_fail,E0308
    /// use tiler_ir::kernel::KernelConstant;
    /// // An `f32` bit pattern is not a `bf16` payload.
    /// let _ = KernelConstant::Bf16Bits(1.0_f32.to_bits());
    /// ```
    ///
    /// The narrower payload is also what keeps the identity encoding honest: the
    /// constant is encoded at its own width, so two `bf16` constants that differ
    /// nowhere in sixteen bits cannot be given distinct identities by padding
    /// nobody can observe.
    Bf16Bits(u16),
}

impl KernelConstant {
    /// Returns the resolved type of this constant.
    #[must_use]
    pub const fn value_type(self) -> KernelType {
        match self {
            Self::Bool(_) => KernelType::Bool,
            Self::Index(_) => KernelType::Index,
            Self::F32Bits(_) => KernelType::F32,
            Self::Bf16Bits(_) => KernelType::Bf16,
        }
    }

    /// Returns the index-role value when this constant is index-typed.
    #[must_use]
    pub const fn as_index(self) -> Option<u64> {
        match self {
            Self::Index(value) => Some(value),
            Self::Bool(_) | Self::F32Bits(_) | Self::Bf16Bits(_) => None,
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
    /// Index subtraction whose result is proven non-negative.
    ///
    /// Admitted for the one coordinate map that needs it: a reindex mirroring an
    /// axis reads `extent − 1 − c`, where `c` is a `% extent` result and the
    /// left operand is the constant `extent − 1`, so the difference is in
    /// `0..extent` by construction. There is deliberately no wrapping or
    /// saturating spelling beside it — an index that could go negative is a
    /// defect in the map that produced it, not a case to define behaviour for.
    IndexSubtract,
    /// IEEE-754 binary32 addition.
    F32Add,
    /// IEEE-754 binary32 multiplication.
    F32Multiply,
    /// Exact signed 32-bit subtraction.
    I32Subtract,
    /// IEEE-754 binary32 division.
    ///
    /// A division, deliberately not a reciprocal followed by a multiply: the two
    /// round a different number of times and are different binary32 functions.
    /// `tiler::silu-f32@1` pins the division form and withholds the reciprocal
    /// permission, so a lowering that substituted one for the other would compute
    /// something the operation does not mean.
    F32Divide,
    /// The IEEE 754-2019 `maximum` of two binary32 values.
    ///
    /// **The NaN-propagating extrema family, with `-0.0` ordered below `+0.0`.**
    /// ADR 0023 makes that one of *two* families and forbids treating a backend
    /// spelling as the semantic authority; this construct means the propagating
    /// one, which is what `tiler::softmax-f32@1`'s row maximum pins, and there is
    /// deliberately no number-preferring sibling beside it. One construct standing
    /// for both would make operand swapping and reduction-tree selection
    /// observable, which is the failure ADR 0023 exists to prevent.
    ///
    /// **It does not lower to `air.fmax.f32`, and the emitter is where that is
    /// enforced.** Metal's `fmax` prefers numbers *and* leaves its signed-zero
    /// result dependent on operand order, so it implements neither Tiler family;
    /// `crates/tiler-metal/src/emit.rs` emits an exact fixup built from ordered
    /// comparisons rather than selecting the intrinsic.
    ///
    /// **It performs no arithmetic**, which is why it carries no rounding
    /// obligation and why the reduction it combines declares no accumulator
    /// width: a maximum selects one of its operands' bit patterns rather than
    /// computing a new value. It is still subject to the *input* subnormal
    /// obligation, because a target that flushed a subnormal operand before
    /// comparing would select a different one.
    F32Maximum,
    /// Ordered `bf16` addition.
    ///
    /// A construct of its own rather than [`Self::F32Add`] over `bf16` operands,
    /// because the operand type is part of an operation's identity: this one
    /// realizes `tiler::add-bf16@1`, which rounds once to eight significand bits
    /// and declares contraction, fused multiply-add, and reassociation all
    /// forbidden. An emitter handed one construct for both widths would have to
    /// reconstruct the width from its operands, which is exactly the inference
    /// this vocabulary exists to remove.
    Bf16Add,
    /// Ordered `bf16` multiplication.
    ///
    /// `tiler::multiply-bf16@1`, under the same separation [`Self::Bf16Add`]
    /// states. There is deliberately no `bf16` division, maximum, or fused form
    /// beside the pair: no registered `bf16` operation means one, so a construct
    /// here would name an obligation nothing states.
    Bf16Multiply,
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
            // Appended rather than inserted, and the kernel domain deliberately
            // did not step with it: every earlier variant keeps its tag and every
            // field keeps its position, so no previously encodable kernel's bytes
            // move. A reader that reaches `0x08` is reading a kernel the earlier
            // vocabulary could not express, never an earlier kernel under a new
            // interpretation.
            Self::F32Divide => 0x08,
            // Appended for the same reason and with the same consequence: `0x01`
            // through `0x08` keep their meanings and every field keeps its
            // position, so no previously encodable kernel's bytes move and the
            // kernel identity domain does not step. A reader that reaches `0x09`
            // is reading a kernel the earlier vocabulary could not express.
            Self::F32Maximum => 0x09,
            // Appended for the same reason and with the same consequence: `0x01`
            // through `0x09` keep their meanings and every field keeps its
            // position, so no previously encodable kernel's bytes move and the
            // kernel identity domain does not step. A reader that reaches `0x0a`
            // or `0x0b` is reading a kernel the earlier vocabulary could not
            // express — no earlier kernel could hold a `bf16` operand at all.
            Self::Bf16Add => 0x0a,
            Self::Bf16Multiply => 0x0b,
            // Appended for the same reason and with the same consequence: `0x01`
            // through `0x0b` keep their meanings and every field keeps its
            // position, so no previously encodable kernel's bytes move and the
            // kernel identity domain does not step. A reader that reaches `0x0c`
            // is reading a kernel the earlier vocabulary could not express — no
            // earlier kernel could mirror a coordinate.
            Self::IndexSubtract => 0x0c,
        }
    }

    /// Returns the required operand type.
    #[must_use]
    pub const fn operand_type(self) -> KernelType {
        match self {
            Self::IndexAdd
            | Self::IndexMultiply
            | Self::IndexDivide
            | Self::IndexModulo
            | Self::IndexSubtract => KernelType::Index,
            Self::F32Add | Self::F32Multiply | Self::F32Divide | Self::F32Maximum => {
                KernelType::F32
            }
            Self::Bf16Add | Self::Bf16Multiply => KernelType::Bf16,
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

/// A pure unary elementary function.
///
/// The first transcendental construct in this vocabulary, and it is one variant
/// rather than a family: a construct here is only admissible once some registered
/// operation's *resolved accuracy contract* says what it must deliver, and exactly
/// one does. Widening this enum is a versioned extension that must bring its own
/// contract, not a matter of adding the next function a backend happens to expose.
///
/// **This names the precise function, never an approximate one.** A backend
/// selecting a fast-math intrinsic to realize this construct is the substitution
/// ADR 0076 forbids, and the emission probe records that it is one default
/// compiler flag away. The construct carries no accuracy attribute of its own
/// because the accuracy lives in the semantic operation's contract; a second
/// spelling here would be a second authority over the same obligation.
///
/// Deliberately not `#[non_exhaustive]`, for the same reason [`KernelType`] is
/// not: `tiler-artifact` encodes this vocabulary into a cross-crate total map,
/// where widening must be a build error at every encoder rather than a silent
/// identity collision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UnaryOp {
    /// The natural exponential over IEEE-754 binary32.
    ///
    /// Realizes the subordinate exponential of `tiler::silu-f32@1` and nothing
    /// else. Its admitted result set is that key's registered accuracy contract.
    F32Exp,
    /// The reciprocal square root over IEEE-754 binary32.
    ///
    /// Realizes the subordinate reciprocal square root of
    /// `tiler::rms-norm-f32@1` and nothing else. Its admitted result set is that
    /// key's registered accuracy contract, which is `Faithful` — a *different*
    /// contract form from the exponential's ULP bound, because the two rest on
    /// different halves of Metal's accuracy table.
    ///
    /// **Deliberately not a reciprocal followed by a square root.** `1 / sqrt(t)`
    /// rounds twice and is a different binary32 function; the pinned reference
    /// measures a one-step disagreement at the `eps` argument the workload's zero
    /// and subnormal rows both reach. This vocabulary has no `sqrt` construct at
    /// all, so the substitution is unstatable here rather than merely forbidden.
    F32Rsqrt,
}

impl UnaryOp {
    const fn tag(self) -> u8 {
        match self {
            Self::F32Exp => 0x01,
            // Appended rather than inserted, like `BinaryOp::F32Divide`: the
            // exponential keeps `0x01` and every field keeps its position, so no
            // previously encodable kernel's bytes move and the kernel identity
            // domain does not step. A reader that reaches `0x02` is reading a
            // kernel the earlier vocabulary could not express.
            Self::F32Rsqrt => 0x02,
        }
    }

    /// Returns the required operand type.
    #[must_use]
    pub const fn operand_type(self) -> KernelType {
        match self {
            Self::F32Exp | Self::F32Rsqrt => KernelType::F32,
        }
    }

    /// Returns the produced result type.
    #[must_use]
    pub const fn result_type(self) -> KernelType {
        self.operand_type()
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
    /// Replaces a `bf16` NaN result with the realization's canonical `bf16` NaN.
    ///
    /// **A conversion of its own, not [`Self::CanonicalizeF32Nan`] at another
    /// width.** The two write different bit patterns into different-width
    /// values, and the payload each installs comes from the same field: the
    /// kernel's
    /// [`NumericalRealization::canonical_arithmetic_nan_bits`], read as this
    /// region's own arithmetic type's pattern zero-extended into thirty-two
    /// bits. A `bf16` region declaring an `f32` payload is refused by the
    /// intrinsic schedule verifier, so the low sixteen bits this construct
    /// installs are the whole declared pattern rather than a truncation.
    ///
    /// It is *not* a narrowing conversion between the two formats: no value
    /// crosses a width here. [ADR
    /// 0091](../../../../docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md)
    /// owns the directional BF16/binary32 conversion families, and neither is
    /// registered or spellable in this vocabulary.
    CanonicalizeBf16Nan,
}

impl ConvertOp {
    const fn tag(self) -> u8 {
        match self {
            Self::CanonicalizeF32Nan => 0x01,
            Self::U8ToI32 => 0x02,
            Self::I32ToF32 => 0x03,
            // Appended, like `BinaryOp::Bf16Add`: `0x01` through `0x03` keep
            // their meanings and every field keeps its position, so no
            // previously encodable kernel's bytes move and the kernel identity
            // domain does not step.
            Self::CanonicalizeBf16Nan => 0x04,
        }
    }

    /// Returns the required source type.
    #[must_use]
    pub const fn source_type(self) -> KernelType {
        match self {
            Self::CanonicalizeF32Nan => KernelType::F32,
            Self::U8ToI32 => KernelType::U8,
            Self::I32ToF32 => KernelType::I32,
            Self::CanonicalizeBf16Nan => KernelType::Bf16,
        }
    }

    /// Returns the produced result type.
    #[must_use]
    pub const fn result_type(self) -> KernelType {
        match self {
            Self::U8ToI32 => KernelType::I32,
            Self::CanonicalizeF32Nan | Self::I32ToF32 => KernelType::F32,
            Self::CanonicalizeBf16Nan => KernelType::Bf16,
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
///
/// # Why the spec both names a schedule point and restates its subject
///
/// `point` is the *reference*: it names the exact
/// [`SynchronizationPoint`](crate::schedule::SynchronizationPoint) of the
/// region's cooperative tile that this barrier realizes, so "which handoff does
/// this order" is a resolvable question rather than a positional coincidence.
/// The four spelling fields are the *self-contained emission fact*: this IR
/// exists so a backend needs no other document, and a backend cannot reach the
/// scheduled region from a kernel operation.
///
/// They are deliberately **not** one authority. The schedule point is the
/// obligation and this is a declaration of it; whole-kernel verification
/// projects the four fields onto the point's
/// [`SynchronizationSubject`](crate::schedule::SynchronizationSubject) through
/// one total mapping and requires equality, exactly as a kernel's declared
/// [`ResourceRequirements`] is proven equal to the derived record rather than
/// being trusted beside it. Equal field shapes did not make these one concept
/// before that projection existed, and the projection is what makes the
/// agreement checked rather than assumed.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BarrierSpec {
    /// Schedule synchronization point this barrier realizes.
    pub point: crate::schedule::SyncPointId,
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
///
/// Literal start and end remain the form every static loop uses. A live bound
/// is a separate builder path that takes SSA values, so a kernel cannot choose
/// between baking an extent and reading it as a parameter for the same range.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SerialLoopSpec {
    /// Inclusive first induction value.
    pub start: u64,
    /// Exclusive last induction value.
    pub end: u64,
}

/// **Accepted public surface.** Tom accepted this exact spelling on
/// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
/// Dependents may treat `LoopBound::Value` as accepted vocabulary.
///
/// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
///
/// One bound of a structured loop: a compile-time literal or an SSA value.
///
/// A value bound is how a live input extent becomes a trip count. The value
/// must be index-typed; the live extent operand is the only non-literal source
/// this ticket admits.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LoopBound {
    /// A compile-time inclusive-start or exclusive-end.
    Literal(u64),
    /// An index-typed SSA value in the enclosing block.
    Value(super::handles::VerifiedValueId),
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
    Unary {
        op: UnaryOp,
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
    GuardedLoad {
        predicate: u32,
        buffer: u32,
        offset: u32,
        bounds: BoundsWitnessId,
        inactive: u32,
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
    SerialLoopRange {
        start: u32,
        end: u32,
        initial: Vec<u32>,
        body: u32,
        yields: Vec<u32>,
    },
    InputExtent {
        parameter: u32,
    },
    Barrier {
        spec: BarrierSpec,
    },
    StagedStore {
        staging: u32,
        offset: u32,
        value: u32,
        phase: crate::schedule::PhaseId,
    },
    StagedLoad {
        staging: u32,
        offset: u32,
        phase: crate::schedule::PhaseId,
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
    pub(super) staging: Vec<StagingParameter>,
    pub(super) input_extents: Vec<InputExtentParameter>,
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
    /// Derived from the refined region's scalar program; see
    /// [`VerifiedKernel::subnormal_freedom`] for why it is not in `data`.
    pub(super) subnormal_freedom: SubnormalFreedom,
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

    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Returns the live input-extent operands this kernel signature admits.
    ///
    /// Canonical declaration order, which is also the argument-table order
    /// after the buffer parameters: a parameter's position here is its scalar
    /// transport ordinal. Empty for every kernel whose body specializes no
    /// live extent.
    #[must_use]
    pub fn input_extents(&self) -> impl ExactSizeIterator<Item = InputExtentParameter> + '_ {
        self.data.input_extents.iter().copied()
    }

    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Returns each declared live input-extent operand with the handle naming it.
    #[must_use]
    pub fn declared_input_extents(
        &self,
    ) -> impl ExactSizeIterator<Item = (VerifiedInputExtentId, InputExtentParameter)> + '_ {
        let count = u32::try_from(self.data.input_extents.len()).unwrap_or(u32::MAX);
        (0..count)
            .zip(self.data.input_extents.iter().copied())
            .map(|(index, parameter)| (self.input_extent_id(index), parameter))
    }

    /// Returns the workgroup staging allocations this kernel declares.
    ///
    /// Declaration order, which is ascending [`crate::schedule::StagingId`]
    /// order — the verifier proves that correspondence against the region's
    /// cooperative tile, so a consumer may index by staging ordinal. Empty for
    /// every kernel whose region stages nothing.
    #[must_use]
    pub fn staging(&self) -> impl ExactSizeIterator<Item = StagingParameter> + '_ {
        self.data.staging.iter().copied()
    }

    /// Returns the preserved numerical realization.
    #[must_use]
    pub const fn numerical(&self) -> NumericalRealization {
        self.data.numerical
    }

    /// **Draft surface, not yet accepted.**
    ///
    /// Returns whether this kernel's arithmetic is bounded away from subnormals.
    ///
    /// A backend deciding whether it can honour [`Self::numerical`]'s declared
    /// subnormal behaviour must consult this first: where the freedom covers the
    /// arithmetic type in question, both resolutions of that type's subnormal
    /// dimensions return identical bits, so a target whose behaviour differs is
    /// not a gap.
    ///
    /// **Copied from the region, not encoded into
    /// [`Self::canonical_identity`].** It is a total function of the scheduled
    /// program, whose canonical identity this kernel's identity already folds
    /// in, so encoding it would add a byte no two distinguishable kernels
    /// differ in. Keeping it out of the assembled kernel arena also keeps it
    /// out of the refinement gate's structural comparison, where it would have
    /// compared a derived value against itself.
    #[must_use]
    pub const fn subnormal_freedom(&self) -> SubnormalFreedom {
        self.subnormal_freedom
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

    fn staging_id(&self, index: u32) -> VerifiedStagingId {
        VerifiedStagingId::from_verified(self.owner, index)
    }

    fn input_extent_id(&self, index: u32) -> VerifiedInputExtentId {
        VerifiedInputExtentId::from_verified(self.owner, index)
    }

    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Returns one verified live input-extent operand.
    ///
    /// # Errors
    ///
    /// Returns [`VerifiedKernelHandleError`] when the handle belongs to another
    /// kernel or does not identify a retained operand.
    pub fn input_extent(
        &self,
        id: VerifiedInputExtentId,
    ) -> Result<InputExtentParameter, VerifiedKernelHandleError> {
        if id.owner != self.owner {
            return Err(VerifiedKernelHandleError::ForeignKernel {
                entity: KernelEntityKind::InputExtent,
            });
        }
        self.data.input_extents.get(id.as_usize()).copied().ok_or(
            VerifiedKernelHandleError::InvalidHandle {
                entity: KernelEntityKind::InputExtent,
            },
        )
    }

    /// Returns one verified workgroup staging allocation.
    ///
    /// # Errors
    ///
    /// Returns [`VerifiedKernelHandleError`] when the handle belongs to another
    /// kernel or does not identify a retained allocation.
    pub fn staging_parameter(
        &self,
        id: VerifiedStagingId,
    ) -> Result<StagingParameter, VerifiedKernelHandleError> {
        if id.owner != self.owner {
            return Err(VerifiedKernelHandleError::ForeignKernel {
                entity: KernelEntityKind::Staging,
            });
        }
        self.data.staging.get(id.as_usize()).copied().ok_or(
            VerifiedKernelHandleError::InvalidHandle {
                entity: KernelEntityKind::Staging,
            },
        )
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
            OperationKind::Unary { op, source } => OperationView::Unary {
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
            OperationKind::GuardedLoad {
                predicate,
                buffer,
                offset,
                bounds,
                inactive,
            } => OperationView::GuardedLoad {
                predicate: kernel.value_id(*predicate),
                buffer: kernel.buffer_id(*buffer),
                offset: kernel.value_id(*offset),
                bounds: *bounds,
                inactive: kernel.value_id(*inactive),
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
                start: LoopBoundData::Literal(*start),
                end: LoopBoundData::Literal(*end),
                initial,
                yields,
                block: *body,
            }),
            OperationKind::SerialLoopRange {
                start,
                end,
                initial,
                body,
                yields,
            } => OperationView::SerialLoop(SerialLoopRef {
                kernel,
                start: LoopBoundData::Value(*start),
                end: LoopBoundData::Value(*end),
                initial,
                yields,
                block: *body,
            }),
            OperationKind::InputExtent { parameter } => OperationView::InputExtent {
                parameter: kernel.input_extent_id(*parameter),
            },
            OperationKind::Barrier { spec } => OperationView::Barrier { spec },
            OperationKind::StagedStore {
                staging,
                offset,
                value,
                phase,
            } => OperationView::StagedStore {
                staging: kernel.staging_id(*staging),
                offset: kernel.value_id(*offset),
                value: kernel.value_id(*value),
                phase: *phase,
            },
            OperationKind::StagedLoad {
                staging,
                offset,
                phase,
            } => OperationView::StagedLoad {
                staging: kernel.staging_id(*staging),
                offset: kernel.value_id(*offset),
                phase: *phase,
            },
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
    /// Applies a pure unary elementary function.
    Unary {
        /// The applied elementary function.
        op: UnaryOp,
        /// The function's argument.
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
    /// Loads one element when `predicate` is true; otherwise returns `inactive`
    /// without a memory access.
    ///
    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-12 under
    /// [`admit-guarded-output-tails-for-cooperative-contraction`]. It is a
    /// scalar structured control over one invocation, not a vector masked load.
    ///
    /// [`admit-guarded-output-tails-for-cooperative-contraction`]: ../../../../../tickets/admit-guarded-output-tails-for-cooperative-contraction.md
    GuardedLoad {
        /// Schedule-derived Boolean that authorizes the memory access.
        predicate: VerifiedValueId,
        /// Buffer read when the predicate is true.
        buffer: VerifiedBufferId,
        /// Element offset within the buffer.
        offset: VerifiedValueId,
        /// Schedule bounds witness authorizing the true-path access.
        bounds: BoundsWitnessId,
        /// Value returned when the predicate is false. No memory access occurs.
        inactive: VerifiedValueId,
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
    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Reads one declared live input-extent operand as an index-typed value.
    InputExtent {
        /// Declared operand whose live extent is read.
        parameter: VerifiedInputExtentId,
    },
    /// Synchronizes an execution scope and fences named address spaces.
    Barrier {
        /// The barrier's schedule point, scopes, fences, and ordering.
        spec: &'a BarrierSpec,
    },
    /// Stores one element into workgroup staging during a named tile phase.
    ///
    /// The phase is the evidence: it names the
    /// [`CooperativePhase`](crate::schedule::CooperativePhase) whose declared
    /// [`StagedWrite`](crate::schedule::StagedWrite) authorizes this effect,
    /// which is what a bounds witness is for a boundary load. It also fixes
    /// *when* the effect happens relative to the tile's visibility edges, so a
    /// verifier can decide whether the barrier discharging an edge actually
    /// separates this write from the read that consumes it.
    StagedStore {
        /// Workgroup allocation written.
        staging: VerifiedStagingId,
        /// Slot index within the allocation.
        offset: VerifiedValueId,
        /// Stored value.
        value: VerifiedValueId,
        /// Tile phase whose staged write authorizes this effect.
        phase: crate::schedule::PhaseId,
    },
    /// Loads one element from workgroup staging during a named tile phase.
    StagedLoad {
        /// Workgroup allocation read.
        staging: VerifiedStagingId,
        /// Slot index within the allocation.
        offset: VerifiedValueId,
        /// Tile phase whose staged read authorizes this effect.
        phase: crate::schedule::PhaseId,
    },
}

/// A read-only view of one bounded structured loop.
///
/// The loop body binds an induction variable followed by one parameter per
/// carried accumulator; the yielded values become the accumulators of the next
/// iteration and, after the final iteration, the loop's results.
#[derive(Clone, Copy, Debug)]
enum LoopBoundData {
    Literal(u64),
    Value(u32),
}

/// A read-only view of one bounded structured loop.
///
/// The loop body binds an induction variable followed by one parameter per
/// carried accumulator; the yielded values become the accumulators of the next
/// iteration and, after the final iteration, the loop's results.
#[derive(Clone, Copy, Debug)]
pub struct SerialLoopRef<'a> {
    kernel: &'a VerifiedKernel,
    start: LoopBoundData,
    end: LoopBoundData,
    initial: &'a [u32],
    yields: &'a [u32],
    block: u32,
}

impl<'a> SerialLoopRef<'a> {
    /// Returns the inclusive first induction value of a literal-bound loop.
    ///
    /// Live-bound loops use [`Self::start_bound`]. A value bound reports `0`
    /// here so existing literal-only readers keep compiling; they must consult
    /// [`Self::start_bound`] before treating the number as the trip start.
    #[must_use]
    pub const fn start(self) -> u64 {
        match self.start {
            LoopBoundData::Literal(value) => value,
            LoopBoundData::Value(_) => 0,
        }
    }

    /// Returns the exclusive last induction value of a literal-bound loop.
    ///
    /// Live-bound loops use [`Self::end_bound`].
    #[must_use]
    pub const fn end(self) -> u64 {
        match self.end {
            LoopBoundData::Literal(value) => value,
            LoopBoundData::Value(_) => 0,
        }
    }

    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Returns the inclusive first induction bound.
    #[must_use]
    pub fn start_bound(self) -> LoopBound {
        self.bound(self.start)
    }

    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Returns the exclusive last induction bound.
    #[must_use]
    pub fn end_bound(self) -> LoopBound {
        self.bound(self.end)
    }

    fn bound(self, bound: LoopBoundData) -> LoopBound {
        match bound {
            LoopBoundData::Literal(value) => LoopBound::Literal(value),
            LoopBoundData::Value(index) => LoopBound::Value(self.kernel.value_id(index)),
        }
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

/// Writes the governed tag of one index-arithmetic requirement.
///
/// Written as an exhaustive match rather than read from the discriminant, so a
/// variant added to [`IndexArithmetic`] is a build error here instead of a
/// silent renumbering of every kernel identity already derived. `0x01` is the
/// first value of a new table rather than a reused one: no earlier tag table
/// covers this vocabulary.
fn push_index_arithmetic(bytes: &mut Vec<u8>, index_arithmetic: IndexArithmetic) {
    bytes.push(match index_arithmetic {
        IndexArithmetic::CompleteU64 => 0x01,
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

/// Encodes the synchronization realization a region requires, or its absence.
///
/// A one-byte presence tag, and the complete subject when present. Absence
/// writes `0x00` rather than nothing: this record is followed by more fields, so
/// the omitted-when-empty idiom that let the staging list append has no analogue
/// here — which is exactly why this field steps the kernel identity domain. What
/// the tag buys is that "this kernel synchronizes nothing" becomes an *encoded*
/// claim rather than an unstated one, so a kernel that later gains a barrier
/// cannot share identity with the one that did not.
fn push_synchronization(bytes: &mut Vec<u8>, subject: Option<SynchronizationSubject>) {
    match subject {
        None => bytes.push(0x00),
        Some(subject) => {
            bytes.push(0x01);
            bytes.push(subject.kind.tag());
            bytes.push(subject.execution_scope.tag());
            bytes.push(subject.visibility_scope.tag());
            bytes.push(u8::from(subject.fenced_spaces.workgroup));
            bytes.push(u8::from(subject.fenced_spaces.device));
            bytes.push(subject.ordering.tag());
        }
    }
}

fn push_requirements(bytes: &mut Vec<u8>, requirements: &ResourceRequirements) {
    bytes.extend_from_slice(&requirements.buffer_bindings.to_be_bytes());
    bytes.extend_from_slice(&requirements.threads_per_workgroup.to_be_bytes());
    bytes.extend_from_slice(&requirements.local_memory_bytes.to_be_bytes());
    bytes.push(u8::from(requirements.requires_device_memory));
    push_index_arithmetic(bytes, requirements.index_arithmetic);
    push_synchronization(bytes, requirements.synchronization);
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

/// Encodes the workgroup staging allocations a kernel declares.
///
/// **Written last, and written as nothing at all when the list is empty.** That
/// is what makes this an append rather than a domain step: a kernel that stages
/// nothing encodes exactly the bytes it encoded before this list existed, so no
/// cached or artifact-held identity moves. Injectivity survives because the
/// block encoding preceding it is fully self-framing — after it the decoder is
/// at a determined offset, so "bytes remain" *is* the presence tag, and a
/// nonempty list carries its own length. A `0` length written unconditionally
/// would have added eight bytes to every kernel ever encoded and stepped
/// `tiler.kernel.v5`.
fn push_staging(bytes: &mut Vec<u8>, staging: &[StagingParameter]) {
    if staging.is_empty() {
        return;
    }
    push_len(bytes, staging.len());
    for parameter in staging {
        bytes.extend_from_slice(&parameter.staging.get().to_be_bytes());
        bytes.push(parameter.element_type.tag());
        bytes.push(parameter.address_space.tag());
        bytes.extend_from_slice(&parameter.element_count.to_be_bytes());
    }
}

/// Presence tag of a nonempty live input-extent declaration list.
///
/// Staging already occupies the "bytes remain after the body" slot, and a
/// small staging count opens with a zero high byte. `0xfe` cannot be that
/// prefix, so a kernel that declares extents and no staging cannot be
/// re-read as a staging list. Empty writes nothing, so kernels that declare
/// no live extent keep the bytes they encoded before this list existed.
const INPUT_EXTENT_BLOCK_TAG: u8 = 0xfe;

/// Encodes the live input-extent operands a kernel declares.
///
/// **Written after staging and before the subgroup requirement, and written
/// as nothing at all when the list is empty.** Kernels that declare no live
/// extent keep the bytes they encoded after the subgroup field landed. A
/// nonempty list is a subject the earlier vocabulary could not express, so
/// those bytes are new rather than a reinterpretation.
fn push_input_extents(bytes: &mut Vec<u8>, extents: &[InputExtentParameter]) {
    if extents.is_empty() {
        return;
    }
    bytes.push(INPUT_EXTENT_BLOCK_TAG);
    push_len(bytes, extents.len());
    for parameter in extents {
        push_tensor_role(bytes, parameter.tensor);
        bytes.extend_from_slice(&parameter.axis.get().to_be_bytes());
    }
}

/// Mirrors [`push_input_extents`], including its empty case writing nothing.
fn input_extents_encoded_len(extents: &[InputExtentParameter]) -> usize {
    if extents.is_empty() {
        return 0;
    }
    1_usize
        .saturating_add(LENGTH_BYTES)
        .saturating_add(extents.iter().fold(0_usize, |total, parameter| {
            total
                .saturating_add(tensor_role_encoded_len(parameter.tensor))
                .saturating_add(size_of_val(&parameter.axis.get()))
        }))
}

/// Encodes the subgroup realization a kernel requires, or writes nothing.
///
/// **Written last, and written as nothing at all when the requirement is
/// absent.** A kernel that requires no subgroup combine and declares no live
/// extent encodes exactly the bytes it encoded before either field existed.
/// Injectivity survives because the staging block and the extent list
/// preceding it are fully self-framing when present, and when all three are
/// absent the encoding ends where it always did. A `0x00` presence tag
/// written unconditionally would have added one byte to every kernel ever
/// encoded and stepped [`KERNEL_DOMAIN`].
fn push_subgroup_requirement(bytes: &mut Vec<u8>, subject: Option<SubgroupRealizationSubject>) {
    let Some(subject) = subject else {
        return;
    };
    bytes.push(0x01);
    subject.encode(bytes);
}

/// Mirrors [`push_staging`], including its empty case writing nothing.
fn staging_encoded_len(staging: &[StagingParameter]) -> usize {
    if staging.is_empty() {
        return 0;
    }
    LENGTH_BYTES.saturating_add(staging.iter().fold(0_usize, |total, parameter| {
        total
            .saturating_add(size_of_val(&parameter.staging.get()))
            // The element type and address space tags.
            .saturating_add(2)
            .saturating_add(size_of_val(&parameter.element_count))
    }))
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
        // Appended: `0x01` through `0x03` keep their meanings and their payload
        // widths, so no previously encodable kernel's bytes move. Injectivity
        // survives the two float payloads having different widths because the
        // tag precedes the payload and fixes it — a decoder at `0x04` reads two
        // bytes and at `0x03` reads four, so no `bf16` constant's encoding is a
        // prefix or a re-reading of an `f32` one.
        KernelConstant::Bf16Bits(pattern) => {
            bytes.push(0x04);
            bytes.extend_from_slice(&pattern.to_be_bytes());
        }
    }
}

fn push_barrier(bytes: &mut Vec<u8>, spec: &BarrierSpec) {
    bytes.extend_from_slice(&spec.point.get().to_be_bytes());
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
        OperationKind::Unary { op, source } => {
            // Appended tag, like `BinaryOp::F32Divide` above: no earlier kernel's
            // bytes move, so the kernel identity domain does not step.
            bytes.push(0x1c);
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
        OperationKind::GuardedLoad {
            predicate,
            buffer,
            offset,
            bounds,
            inactive,
        } => {
            // Next unused append-only tag after `InputExtent` at `0x20`.
            // `0x1f` is `SerialLoopRange`. Exact kernels keep their bytes.
            bytes.push(0x21);
            bytes.extend_from_slice(&predicate.to_be_bytes());
            bytes.extend_from_slice(&buffer.to_be_bytes());
            bytes.extend_from_slice(&offset.to_be_bytes());
            bytes.extend_from_slice(&bounds.get().to_be_bytes());
            bytes.extend_from_slice(&inactive.to_be_bytes());
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
        OperationKind::SerialLoopRange {
            start,
            end,
            initial,
            body,
            yields,
        } => {
            // Appended tag: no earlier kernel could contain a live loop bound,
            // so previously encodable bytes do not move.
            bytes.push(0x1f);
            bytes.extend_from_slice(&start.to_be_bytes());
            bytes.extend_from_slice(&end.to_be_bytes());
            push_indices(bytes, initial);
            push_indices(bytes, yields);
            push_block(bytes, data, *body);
        }
        OperationKind::InputExtent { parameter } => {
            bytes.push(0x20);
            bytes.extend_from_slice(&parameter.to_be_bytes());
        }
        OperationKind::Barrier { spec } => {
            bytes.push(0x1a);
            push_barrier(bytes, spec);
        }
        // Appended tags. No `v5` kernel could contain either construct — staged
        // accesses did not exist and every barrier was refused intrinsically —
        // so these move no earlier subject's bytes on their own account. The
        // domain still steps, for the resource-requirement field above.
        OperationKind::StagedStore {
            staging,
            offset,
            value,
            phase,
        } => {
            bytes.push(0x1d);
            bytes.extend_from_slice(&staging.to_be_bytes());
            bytes.extend_from_slice(&offset.to_be_bytes());
            bytes.extend_from_slice(&value.to_be_bytes());
            bytes.extend_from_slice(&phase.get().to_be_bytes());
        }
        OperationKind::StagedLoad {
            staging,
            offset,
            phase,
        } => {
            bytes.push(0x1e);
            bytes.extend_from_slice(&staging.to_be_bytes());
            bytes.extend_from_slice(&offset.to_be_bytes());
            bytes.extend_from_slice(&phase.get().to_be_bytes());
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
    push_staging(&mut bytes, &data.staging);
    push_input_extents(&mut bytes, &data.input_extents);
    push_subgroup_requirement(&mut bytes, data.requirements.subgroup);
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
        .saturating_add(staging_encoded_len(&data.staging))
        .saturating_add(input_extents_encoded_len(&data.input_extents))
        .saturating_add(subgroup_requirement_encoded_len(data.requirements.subgroup))
}

/// Mirrors [`push_subgroup_requirement`]: nothing when absent, else a
/// presence tag plus the subject's encoded width, arithmetic, and transfer.
const fn subgroup_requirement_encoded_len(subject: Option<SubgroupRealizationSubject>) -> usize {
    match subject {
        None => 0,
        Some(_) => 1 + size_of::<u32>() + 2,
    }
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

/// Mirrors [`push_synchronization`]: a presence tag, plus six bytes when present.
const fn synchronization_encoded_len(subject: Option<SynchronizationSubject>) -> usize {
    match subject {
        None => 1,
        // The kind, execution scope, and visibility scope tags, the two fence
        // flags, and the ordering tag.
        Some(_) => 7,
    }
}

/// Mirrors [`push_requirements`].
fn requirements_encoded_len(requirements: &ResourceRequirements) -> usize {
    size_of_val(&requirements.buffer_bindings)
        .saturating_add(size_of_val(&requirements.threads_per_workgroup))
        .saturating_add(size_of_val(&requirements.local_memory_bytes))
        // The device-memory flag, the index-arithmetic tag, two subnormal
        // modes, and four permissions.
        .saturating_add(8)
        .saturating_add(synchronization_encoded_len(requirements.synchronization))
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
        KernelConstant::Bf16Bits(pattern) => size_of_val(&pattern),
    })
}

/// Mirrors [`push_barrier`].
fn barrier_encoded_len(spec: &BarrierSpec) -> usize {
    // The schedule point, then execution scope, memory scope, then the framed
    // fenced spaces at one tag byte each, then the ordering.
    size_of_val(&spec.point.get())
        .saturating_add(2)
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
        OperationKind::Convert { source, .. } | OperationKind::Unary { source, .. } => {
            1_usize.saturating_add(size_of_val(source))
        }
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
        OperationKind::GuardedLoad {
            predicate,
            buffer,
            offset,
            bounds,
            inactive,
        } => size_of_val(predicate)
            .saturating_add(size_of_val(buffer))
            .saturating_add(size_of_val(offset))
            .saturating_add(size_of_val(&bounds.get()))
            .saturating_add(size_of_val(inactive)),
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
        OperationKind::SerialLoopRange {
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
        OperationKind::InputExtent { parameter } => size_of_val(parameter),
        OperationKind::Barrier { spec } => barrier_encoded_len(spec),
        OperationKind::StagedStore {
            staging,
            offset,
            value,
            phase,
        } => size_of_val(staging)
            .saturating_add(size_of_val(offset))
            .saturating_add(size_of_val(value))
            .saturating_add(size_of_val(&phase.get())),
        OperationKind::StagedLoad {
            staging,
            offset,
            phase,
        } => size_of_val(staging)
            .saturating_add(size_of_val(offset))
            .saturating_add(size_of_val(&phase.get())),
    };
    // Every arm is preceded by its one-byte kind tag and followed by results.
    1_usize
        .saturating_add(kind)
        .saturating_add(indices_encoded_len(&operation.results))
}

#[cfg(test)]
mod injectivity_tests {
    use std::mem::variant_count;

    use crate::exhaustive_injectivity::{
        EXCEPTIONAL_ASSUMPTIONS, PERMISSIONS, SUBJECT_POPULATION, SUBNORMAL_MODES,
        assert_injective, assert_injective_fixed_width, assert_tag_table,
        every_synchronization_subject,
    };
    use crate::schedule::{ExceptionalValueAssumption, IndexArithmetic};

    use super::{
        AddressSpace, BarrierOrdering, BinaryOp, BufferAccess, Builtin, CompareOp, ConvertOp,
        ExecutionScope, KernelType, MemoryScope, PackedExtractOp, UnaryOp,
        push_exceptional_assumption, push_index_arithmetic, push_permission, push_subnormal,
        push_synchronization,
    };

    #[test]
    fn every_kernel_tag_table_is_injective_over_its_variant_set() {
        const TYPES: [KernelType; variant_count::<KernelType>()] = [
            KernelType::Bool,
            KernelType::U8,
            KernelType::Index,
            KernelType::F32,
            KernelType::I32,
            KernelType::Bf16,
        ];
        const SPACES: [AddressSpace; variant_count::<AddressSpace>()] = [
            AddressSpace::Device,
            AddressSpace::Workgroup,
            AddressSpace::InvocationPrivate,
            AddressSpace::Constant,
        ];
        const ACCESSES: [BufferAccess; variant_count::<BufferAccess>()] =
            [BufferAccess::Read, BufferAccess::Write];
        const BUILTINS: [Builtin; variant_count::<Builtin>()] = [
            Builtin::GlobalInvocationIndex,
            Builtin::LocalInvocationIndex,
        ];
        const BINARY: [BinaryOp; variant_count::<BinaryOp>()] = [
            BinaryOp::IndexAdd,
            BinaryOp::IndexMultiply,
            BinaryOp::IndexDivide,
            BinaryOp::IndexModulo,
            BinaryOp::IndexSubtract,
            BinaryOp::F32Add,
            BinaryOp::F32Multiply,
            BinaryOp::I32Subtract,
            BinaryOp::F32Divide,
            BinaryOp::F32Maximum,
            BinaryOp::Bf16Add,
            BinaryOp::Bf16Multiply,
        ];
        const COMPARES: [CompareOp; variant_count::<CompareOp>()] = [CompareOp::IndexLessThan];
        const UNARY: [UnaryOp; variant_count::<UnaryOp>()] = [UnaryOp::F32Exp, UnaryOp::F32Rsqrt];
        const CONVERTS: [ConvertOp; variant_count::<ConvertOp>()] = [
            ConvertOp::CanonicalizeF32Nan,
            ConvertOp::U8ToI32,
            ConvertOp::I32ToF32,
            ConvertOp::CanonicalizeBf16Nan,
        ];
        const EXTRACTS: [PackedExtractOp; variant_count::<PackedExtractOp>()] =
            [PackedExtractOp::U4LsbZeroTail];
        const EXECUTION_SCOPES: [ExecutionScope; variant_count::<ExecutionScope>()] =
            [ExecutionScope::Subgroup, ExecutionScope::Workgroup];
        const MEMORY_SCOPES: [MemoryScope; variant_count::<MemoryScope>()] =
            [MemoryScope::Workgroup, MemoryScope::Device];
        const ORDERINGS: [BarrierOrdering; variant_count::<BarrierOrdering>()] =
            [BarrierOrdering::AcquireRelease];
        const INDEX_ARITHMETIC: [IndexArithmetic; variant_count::<IndexArithmetic>()] =
            [IndexArithmetic::CompleteU64];

        assert_tag_table("KernelType::tag", &TYPES, KernelType::tag);
        assert_tag_table("AddressSpace::tag", &SPACES, AddressSpace::tag);
        assert_tag_table("BufferAccess::tag", &ACCESSES, BufferAccess::tag);
        assert_tag_table("Builtin::tag", &BUILTINS, Builtin::tag);
        assert_tag_table("BinaryOp::tag", &BINARY, BinaryOp::tag);
        assert_tag_table("CompareOp::tag", &COMPARES, CompareOp::tag);
        assert_tag_table("UnaryOp::tag", &UNARY, UnaryOp::tag);
        assert_tag_table("ConvertOp::tag", &CONVERTS, ConvertOp::tag);
        assert_tag_table("PackedExtractOp::tag", &EXTRACTS, PackedExtractOp::tag);
        assert_tag_table(
            "ExecutionScope::tag",
            &EXECUTION_SCOPES,
            ExecutionScope::tag,
        );
        assert_tag_table("MemoryScope::tag", &MEMORY_SCOPES, MemoryScope::tag);
        assert_tag_table("BarrierOrdering::tag", &ORDERINGS, BarrierOrdering::tag);
        assert_eq!(
            INDEX_ARITHMETIC.len(),
            1,
            "push_index_arithmetic walked its whole type-derived population"
        );
        assert_injective_fixed_width(&INDEX_ARITHMETIC, 1, push_index_arithmetic);
    }

    /// The kernel synchronization encoder is injective over all 649 inhabitants.
    ///
    /// **Exhaustive finite evidence.** The domain is `Option<SynchronizationSubject>`:
    /// the 648 subjects plus the stated absence, and the absence is in the domain
    /// rather than encoded as nothing because this record is followed by more
    /// fields. That is what makes "this kernel synchronizes nothing" an encoded
    /// claim, and it is exactly the pair this enumeration has to separate — a
    /// kernel that later gains a barrier must not share identity with the one
    /// that did not.
    ///
    /// A second, independent copy of the schedule's subject encoder by design,
    /// so it gets a second, independent proof: the two identity domains step
    /// separately, and a test that covered one would say nothing about the
    /// other's bytes.
    #[test]
    fn the_kernel_synchronization_encoding_is_injective_over_its_whole_domain() {
        let mut subjects: Vec<Option<_>> = vec![None];
        subjects.extend(every_synchronization_subject().into_iter().map(Some));

        assert_eq!(
            subjects.len(),
            SUBJECT_POPULATION + 1,
            "the domain is every subject plus its stated absence"
        );
        assert_eq!(subjects.len(), 649);
        for subject in &subjects {
            let mut bytes = Vec::new();
            push_synchronization(&mut bytes, *subject);
            // One presence tag, and six subject bytes when present. Variable
            // width, made unambiguous by the presence tag the collision check
            // below confirms is doing that work.
            let expected = if subject.is_some() { 7 } else { 1 };
            assert_eq!(bytes.len(), expected, "{subject:?} changed width");
        }
        assert_injective(&subjects, push_synchronization);
    }

    /// The kernel subnormal encoder is injective over all three inhabitants.
    ///
    /// Exhaustive finite evidence, proved separately from the identically
    /// spelled schedule encoder for the reason the two exist separately: one
    /// domain's widening must not move the other's bytes, so neither test
    /// stands in for the other.
    #[test]
    fn the_kernel_subnormal_encoding_is_injective_over_its_whole_domain() {
        assert_eq!(SUBNORMAL_MODES.len(), 3);
        assert_injective_fixed_width(&SUBNORMAL_MODES, 1, push_subnormal);
    }

    /// The kernel permission encoder is injective over both inhabitants.
    ///
    /// Exhaustive finite evidence.
    #[test]
    fn the_kernel_permission_encoding_is_injective_over_its_whole_domain() {
        assert_eq!(PERMISSIONS.len(), 2);
        assert_injective_fixed_width(&PERMISSIONS, 1, push_permission);
    }

    /// The kernel exceptional-assumption encoder is injective over four inhabitants.
    ///
    /// Exhaustive finite evidence over `1 + 3` values, variable width for the
    /// reason the schedule copy is.
    #[test]
    fn the_kernel_exceptional_assumption_encoding_is_injective_over_its_whole_domain() {
        assert_eq!(EXCEPTIONAL_ASSUMPTIONS.len(), 4);
        for assumption in EXCEPTIONAL_ASSUMPTIONS {
            let mut bytes = Vec::new();
            push_exceptional_assumption(&mut bytes, assumption);
            let expected = match assumption {
                ExceptionalValueAssumption::MakeNoAssumption => 1,
                ExceptionalValueAssumption::AssumeAbsent { .. } => 2,
            };
            assert_eq!(bytes.len(), expected, "{assumption:?} changed width");
        }
        assert_injective(&EXCEPTIONAL_ASSUMPTIONS, push_exceptional_assumption);
    }
}
