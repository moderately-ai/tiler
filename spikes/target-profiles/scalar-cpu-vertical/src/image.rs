//! The scalar CPU executable representation: `tiler.cpu.scalar-image-v1`.
//!
//! # What this representation is, and why it is not a relabelled evaluator
//!
//! A backend payload has to be *bytes a host can execute without holding the
//! compiler that produced them*. Metal's answer is a `metallib`: an object image
//! an offline toolchain emitted, which the device loads and the host never
//! interprets. A scalar CPU host in this repository has no equivalent — nothing
//! here emits machine code — so this spike's answer is an **image of the
//! structured kernel body itself**, translated once from verified KIR, serialized
//! into a self-describing byte encoding, carried in the artifact, and decoded and
//! executed by [`super::interpret`] with no access to the `VerifiedKernel` it came
//! from.
//!
//! That distinction is the whole point of the exercise. `tiler-reference`
//! evaluates a `SemanticProgram` — the *what* — through the semantic capability
//! registry. This executes a scheduled, lowered kernel body — the *how*: a launch
//! grid, a bounds predicate, element offsets into bound buffers, typed loads and
//! stores, and the exact NaN canonicalization the numerical realization names.
//! Two implementations of one declared contract, arriving at bits independently;
//! the comparison in [`super::vertical`] is worth something exactly because
//! neither shares code with the other.
//!
//! # What it deliberately cannot express
//!
//! The bounded image covers the vocabulary a scalar CPU realization of this
//! profile's dispatchable dtype needs, and refuses the rest **by name** rather
//! than approximating it: packed-nibble extraction, barriers, workgroup and
//! constant address spaces, non-`F32` buffer element types, and the conversions
//! belonging to the quantized path. Each refusal is a [`TranslationError`]
//! variant, and each is a case the spike exercises. A wider profile is a wider
//! image, and widening it is a versioned change to the magic below rather than a
//! silent reinterpretation of the same bytes.

use std::collections::HashMap;

use tiler_ir::kernel::{
    AddressSpace, BinaryOp, BlockRef, BufferAccess, BufferParameter, CompareOp, ConvertOp,
    KernelConstant, KernelType, OperationView, VerifiedBufferId, VerifiedKernel, VerifiedValueId,
};
use tiler_ir::schedule::{FlushedZeroSign, SubnormalMode};

/// Domain separator opening one serialized container of scalar entries.
const CONTAINER_MAGIC: &[u8] = b"tiler.cpu.scalar-image-v1\0";

/// Domain separator opening one serialized scalar entry.
const ENTRY_MAGIC: &[u8] = b"tiler.cpu.scalar-entry-v1\0";

/// Byte width of the one buffer element type this image admits.
pub const F32_BYTES: u64 = 4;

/// A slot in one entry's dense SSA value space.
type Slot = u32;

/// The value types the bounded scalar image carries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageType {
    /// A control predicate.
    Bool,
    /// An unsigned 64-bit index-role integer.
    Index,
    /// An IEEE-754 binary32 value.
    F32,
}

impl ImageType {
    const fn tag(self) -> u8 {
        match self {
            Self::Bool => 0x01,
            Self::Index => 0x02,
            Self::F32 => 0x03,
        }
    }

    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Bool),
            0x02 => Some(Self::Index),
            0x03 => Some(Self::F32),
            _ => None,
        }
    }

    /// Maps one KIR value type onto this image's vocabulary.
    ///
    /// `U8` and `I32` are refused rather than carried: they exist in KIR for the
    /// quantized dequantization path, whose dtypes this profile declares nothing
    /// about, and carrying a type whose operations the image cannot execute
    /// would move the refusal from translation into execution.
    ///
    /// `Bf16` is refused for the same reason and on its own authority: this
    /// profile declares `f32` dispatchable and says nothing about `bf16`, and
    /// [`KernelType::Bf16`] is admitted in KIR as a type rather than a lowerable
    /// one, asking a backend that cannot lower it to refuse it *by name*. The
    /// refusal path does exactly that — [`TranslationError::UnsupportedValueType`]
    /// carries the `KernelType` — so the widened vocabulary cannot make an
    /// unimplemented path look reachable here.
    ///
    /// `U32` is refused on the same reasoning. It is an exact-width storage and
    /// SSA type carrying no arithmetic, conversion, or backend producer, and
    /// this image has no unsigned 32-bit value class to map it onto; mapping it
    /// to [`Self::Index`] would silently widen it and mapping it to
    /// [`Self::Bool`] would reinterpret it.
    const fn from_kernel(value: KernelType) -> Option<Self> {
        match value {
            KernelType::Bool => Some(Self::Bool),
            KernelType::Index => Some(Self::Index),
            KernelType::F32 => Some(Self::F32),
            KernelType::U8 | KernelType::I32 | KernelType::Bf16 | KernelType::U32 => None,
        }
    }
}

/// How a buffer parameter is addressed by this entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageAccess {
    /// The entry may only load from the buffer.
    Read,
    /// The entry may only store to the buffer.
    Write,
}

impl ImageAccess {
    const fn tag(self) -> u8 {
        match self {
            Self::Read => 0x01,
            Self::Write => 0x02,
        }
    }

    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Read),
            0x02 => Some(Self::Write),
            _ => None,
        }
    }
}

/// One buffer parameter of a scalar entry, in signature order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageBuffer {
    /// Access mode this entry admits for the parameter.
    pub access: ImageAccess,
    /// Number of `f32` elements the entry may address.
    pub element_count: u64,
}

/// The exact numerical realization the entry's arithmetic must be executed under.
///
/// Carried in the image rather than re-derived, because it is what a host has to
/// *check itself against*: an image declaring preserved subnormals executed on a
/// host that flushes them is a wrong answer, not a slow one, and the check is
/// only possible if the declaration travels with the bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageNumerics {
    /// Bit pattern every arithmetic NaN is canonicalized to.
    pub canonical_nan_bits: u32,
    /// Declared behaviour for subnormal operands.
    pub input_subnormals: ImageSubnormals,
    /// Declared behaviour for subnormal results.
    pub result_subnormals: ImageSubnormals,
}

/// The subnormal realizations the image can state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageSubnormals {
    /// Subnormal values are preserved exactly.
    Preserve,
    /// Subnormals are flushed to a sign-preserving zero.
    FlushSignedZero,
    /// Subnormals are flushed to positive zero.
    FlushPositiveZero,
}

impl ImageSubnormals {
    const fn tag(self) -> u8 {
        match self {
            Self::Preserve => 0x01,
            Self::FlushSignedZero => 0x02,
            Self::FlushPositiveZero => 0x03,
        }
    }

    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Preserve),
            0x02 => Some(Self::FlushSignedZero),
            0x03 => Some(Self::FlushPositiveZero),
            _ => None,
        }
    }

    const fn from_schedule(mode: SubnormalMode) -> Self {
        match mode {
            SubnormalMode::Preserve => Self::Preserve,
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            } => Self::FlushSignedZero,
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            } => Self::FlushPositiveZero,
        }
    }

    /// A stable name for reporting.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preserve => "preserve",
            Self::FlushSignedZero => "flush-to-signed-zero",
            Self::FlushPositiveZero => "flush-to-positive-zero",
        }
    }
}

/// One instruction of a scalar entry.
///
/// Every operand is a slot in the entry's dense value space and every result is
/// the slot the instruction defines, so an executing host needs no handle
/// authority and no side table. The nesting of [`Self::Predicated`] and
/// [`Self::SerialLoop`] is preserved rather than flattened into branches: the
/// structure is what the KIR verifier proved, and flattening it would make the
/// image a different program that a reader would have to re-prove.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    /// Reads the linear index of this invocation in the launch grid.
    GlobalInvocationIndex { result: Slot },
    /// Defines a control-predicate constant.
    ConstBool { result: Slot, value: bool },
    /// Defines an index-role constant.
    ConstIndex { result: Slot, value: u64 },
    /// Defines an `f32` constant by its exact bit pattern.
    ConstF32 { result: Slot, bits: u32 },
    /// Applies a pure binary operation over two same-typed operands.
    Binary {
        result: Slot,
        op: ImageBinaryOp,
        lhs: Slot,
        rhs: Slot,
    },
    /// Compares two index-role operands.
    Compare {
        result: Slot,
        op: ImageCompareOp,
        lhs: Slot,
        rhs: Slot,
    },
    /// Canonicalizes an arithmetic NaN to the realization's declared pattern.
    CanonicalizeF32Nan { result: Slot, source: Slot },
    /// Loads one `f32` element from a bound buffer.
    Load {
        result: Slot,
        buffer: u32,
        offset: Slot,
    },
    /// Stores one `f32` element into a bound buffer.
    Store {
        buffer: u32,
        offset: Slot,
        value: Slot,
    },
    /// Executes a nested block when a predicate holds.
    Predicated { predicate: Slot, body: Block },
    /// Executes a bounded loop carrying typed accumulator state.
    SerialLoop {
        /// Inclusive first induction value.
        start: u64,
        /// Exclusive last induction value.
        end: u64,
        /// Slot the induction variable is bound to inside the body, if any.
        induction: Option<Slot>,
        /// Slots the carried accumulators are bound to inside the body.
        accumulators: Vec<Slot>,
        /// Slots holding the initial accumulator values.
        initial: Vec<Slot>,
        /// Slots yielded at the end of one iteration.
        yields: Vec<Slot>,
        /// Slots the loop's final accumulator values are published to.
        results: Vec<Slot>,
        /// The loop body.
        body: Block,
    },
}

/// The binary operations the bounded image admits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageBinaryOp {
    IndexAdd,
    IndexMultiply,
    IndexDivide,
    IndexModulo,
    F32Add,
    F32Multiply,
}

impl ImageBinaryOp {
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

    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::IndexAdd),
            0x02 => Some(Self::IndexMultiply),
            0x03 => Some(Self::IndexDivide),
            0x04 => Some(Self::IndexModulo),
            0x05 => Some(Self::F32Add),
            0x06 => Some(Self::F32Multiply),
            _ => None,
        }
    }

    /// The type both operands and the result carry.
    #[must_use]
    pub const fn value_type(self) -> ImageType {
        match self {
            Self::IndexAdd | Self::IndexMultiply | Self::IndexDivide | Self::IndexModulo => {
                ImageType::Index
            }
            Self::F32Add | Self::F32Multiply => ImageType::F32,
        }
    }
}

/// The comparisons the bounded image admits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageCompareOp {
    /// Unsigned index ordering.
    IndexLessThan,
}

impl ImageCompareOp {
    const fn tag(self) -> u8 {
        match self {
            Self::IndexLessThan => 0x01,
        }
    }

    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::IndexLessThan),
            _ => None,
        }
    }
}

/// One structured block of instructions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Block {
    /// Instructions in execution order.
    pub instructions: Vec<Instruction>,
}

/// One executable scalar entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarEntry {
    /// Numerical realization the entry's arithmetic is executed under.
    pub numerics: ImageNumerics,
    /// Buffer parameters in signature order; position is the ABI slot.
    pub buffers: Vec<ImageBuffer>,
    /// Declared type of every SSA slot, indexed by slot.
    pub slot_types: Vec<ImageType>,
    /// The entry's top-level block.
    pub body: Block,
}

/// One carried payload: every scalar entry, keyed by its backend symbol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarImage {
    /// Entries in the order the container carries them.
    pub entries: Vec<(String, ScalarEntry)>,
}

impl ScalarImage {
    /// Returns the entry one backend symbol names.
    #[must_use]
    pub fn entry(&self, symbol: &str) -> Option<&ScalarEntry> {
        self.entries
            .iter()
            .find(|(name, _)| name == symbol)
            .map(|(_, entry)| entry)
    }
}

/// Why one verified kernel has no scalar CPU realization.
///
/// Each variant names a construct this bounded backend does **not** implement.
/// None of them is a defect in the kernel: they are the exact boundary of the
/// declared profile, and a widened profile closes them one at a time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TranslationError {
    /// A buffer parameter is not an `f32` buffer in the device address space.
    UnsupportedBuffer {
        /// Signature position of the parameter.
        position: usize,
        /// What made it unsupported.
        detail: &'static str,
    },
    /// A value carries a type outside the bounded image's vocabulary.
    UnsupportedValueType {
        /// The KIR type that has no image spelling.
        found: KernelType,
    },
    /// The body contains an operation this backend does not implement.
    UnsupportedOperation {
        /// A stable name for the construct.
        construct: &'static str,
    },
    /// A launch builtin outside the admitted set was read.
    UnsupportedBuiltin,
    /// The kernel names more slots than one entry may carry.
    TooManySlots {
        /// Observed slot count.
        actual: usize,
        /// Maximum admitted slot count.
        limit: usize,
    },
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedBuffer { position, detail } => write!(
                formatter,
                "cpu.translate.buffer: signature position {position} is {detail}, and this scalar \
                 backend binds f32 device buffers only",
            ),
            Self::UnsupportedValueType { found } => write!(
                formatter,
                "cpu.translate.value-type: the bounded scalar image has no spelling for {found:?}",
            ),
            Self::UnsupportedOperation { construct } => write!(
                formatter,
                "cpu.translate.operation: this scalar backend does not implement {construct}",
            ),
            Self::UnsupportedBuiltin => formatter
                .write_str("cpu.translate.builtin: only the global invocation index is admitted"),
            Self::TooManySlots { actual, limit } => write!(
                formatter,
                "cpu.translate.slots: the kernel names {actual} value(s) and one entry carries at \
                 most {limit}",
            ),
        }
    }
}

impl std::error::Error for TranslationError {}

/// Maximum SSA slots one scalar entry may carry.
///
/// A resource ceiling on decode, not a claim about kernels: the decoder
/// allocates one cell per slot before reading a body, so an unbounded count read
/// from untrusted bytes is an allocation a malformed payload chooses.
pub const MAX_SLOTS: usize = 1 << 16;

/// Translates one verified structured kernel into a scalar entry.
///
/// # Errors
///
/// Returns the first [`TranslationError`] naming the construct this bounded
/// backend does not realize. Nothing partial is returned.
pub fn translate(kernel: &VerifiedKernel) -> Result<ScalarEntry, TranslationError> {
    let mut buffers = Vec::new();
    let mut buffer_slots: HashMap<VerifiedBufferId, u32> = HashMap::new();
    for (position, (id, parameter)) in kernel.declared_buffers().enumerate() {
        buffers.push(translate_buffer(position, parameter)?);
        let ordinal = u32::try_from(position).map_err(|_| TranslationError::TooManySlots {
            actual: position,
            limit: MAX_SLOTS,
        })?;
        buffer_slots.insert(id, ordinal);
    }

    let mut translator = Translator {
        kernel,
        buffer_slots,
        slots: HashMap::new(),
        slot_types: Vec::new(),
    };
    let body = translator.block(kernel.body())?;

    let numerical = kernel.numerical();
    Ok(ScalarEntry {
        numerics: ImageNumerics {
            canonical_nan_bits: numerical.canonical_arithmetic_nan_bits,
            input_subnormals: ImageSubnormals::from_schedule(numerical.input_subnormals),
            result_subnormals: ImageSubnormals::from_schedule(numerical.result_subnormals),
        },
        buffers,
        slot_types: translator.slot_types,
        body,
    })
}

/// Decides whether one buffer parameter has a scalar CPU realization.
///
/// `pub(crate)` rather than private so the vertical can put a synthetic
/// parameter through the exact decision the translator makes, and watch it
/// refuse. A refusal nobody has observed is a claim, not a check.
pub(crate) fn translate_buffer(
    position: usize,
    parameter: BufferParameter,
) -> Result<ImageBuffer, TranslationError> {
    if parameter.element_type != KernelType::F32 {
        return Err(TranslationError::UnsupportedBuffer {
            position,
            detail: "not an f32 buffer",
        });
    }
    if parameter.component_role.is_some() {
        return Err(TranslationError::UnsupportedBuffer {
            position,
            detail: "a component of a compound value",
        });
    }
    match parameter.address_space {
        AddressSpace::Device => {}
        AddressSpace::Workgroup => {
            return Err(TranslationError::UnsupportedBuffer {
                position,
                detail: "in the workgroup address space, which a scalar host has no realization of",
            });
        }
        AddressSpace::InvocationPrivate => {
            return Err(TranslationError::UnsupportedBuffer {
                position,
                detail: "in the invocation-private address space",
            });
        }
        AddressSpace::Constant => {
            return Err(TranslationError::UnsupportedBuffer {
                position,
                detail: "in the constant address space",
            });
        }
    }
    Ok(ImageBuffer {
        access: match parameter.access {
            BufferAccess::Read => ImageAccess::Read,
            BufferAccess::Write => ImageAccess::Write,
        },
        element_count: parameter.element_count,
    })
}

/// Carries the handle-to-slot assignment across one kernel's nested blocks.
struct Translator<'a> {
    kernel: &'a VerifiedKernel,
    buffer_slots: HashMap<VerifiedBufferId, u32>,
    slots: HashMap<VerifiedValueId, Slot>,
    slot_types: Vec<ImageType>,
}

impl Translator<'_> {
    /// Assigns a dense slot to one verified value, or returns the one it has.
    ///
    /// A dense space is minted here rather than read off the handle because a
    /// `VerifiedValueId` publishes no ordinal outside `tiler-ir` — it is `Eq` and
    /// `Hash` and nothing else. That is the correct boundary and it is also the
    /// reason the image cannot simply reuse the kernel's numbering.
    fn slot(&mut self, value: VerifiedValueId) -> Result<Slot, TranslationError> {
        if let Some(slot) = self.slots.get(&value) {
            return Ok(*slot);
        }
        let kind =
            self.kernel
                .value_type(value)
                .map_err(|_| TranslationError::UnsupportedOperation {
                    construct: "a value handle the kernel does not retain",
                })?;
        let kind = ImageType::from_kernel(kind)
            .ok_or(TranslationError::UnsupportedValueType { found: kind })?;
        let next = self.slot_types.len();
        if next >= MAX_SLOTS {
            return Err(TranslationError::TooManySlots {
                actual: next.saturating_add(1),
                limit: MAX_SLOTS,
            });
        }
        let slot = Slot::try_from(next).map_err(|_| TranslationError::TooManySlots {
            actual: next,
            limit: MAX_SLOTS,
        })?;
        self.slot_types.push(kind);
        self.slots.insert(value, slot);
        Ok(slot)
    }

    fn buffer(&self, id: VerifiedBufferId) -> Result<u32, TranslationError> {
        self.buffer_slots
            .get(&id)
            .copied()
            .ok_or(TranslationError::UnsupportedOperation {
                construct: "a buffer handle the kernel signature does not declare",
            })
    }

    fn sole_result(&mut self, operation: &[VerifiedValueId]) -> Result<Slot, TranslationError> {
        let [result] = operation else {
            return Err(TranslationError::UnsupportedOperation {
                construct: "an operation defining other than exactly one result",
            });
        };
        self.slot(*result)
    }

    #[allow(
        clippy::too_many_lines,
        reason = "one arm per structured operation kind, and the exhaustive match is what makes a widened KIR vocabulary a build error here"
    )]
    fn block(&mut self, block: BlockRef<'_>) -> Result<Block, TranslationError> {
        let mut instructions = Vec::new();
        for operation in block.operations() {
            let results: Vec<_> = operation.results().collect();
            let instruction = match operation.view() {
                OperationView::Builtin { builtin } => {
                    // Exhaustive over the admitted set rather than a wildcard: a
                    // builtin added to KIR must be given a scalar meaning here or
                    // refused, and a catch-all would silently give it the grid
                    // index.
                    match builtin {
                        tiler_ir::kernel::Builtin::GlobalInvocationIndex => {
                            Instruction::GlobalInvocationIndex {
                                result: self.sole_result(&results)?,
                            }
                        }
                        _ => return Err(TranslationError::UnsupportedBuiltin),
                    }
                }
                OperationView::Constant { value } => {
                    let result = self.sole_result(&results)?;
                    match value {
                        KernelConstant::Bool(flag) => Instruction::ConstBool {
                            result,
                            value: flag,
                        },
                        KernelConstant::Index(index) => Instruction::ConstIndex {
                            result,
                            value: index,
                        },
                        KernelConstant::F32Bits(bits) => Instruction::ConstF32 { result, bits },
                        _ => {
                            return Err(TranslationError::UnsupportedOperation {
                                construct: "a constant outside the bool/index/f32 vocabulary",
                            });
                        }
                    }
                }
                OperationView::Binary { op, lhs, rhs } => {
                    let op = match op {
                        BinaryOp::IndexAdd => ImageBinaryOp::IndexAdd,
                        BinaryOp::IndexMultiply => ImageBinaryOp::IndexMultiply,
                        BinaryOp::IndexDivide => ImageBinaryOp::IndexDivide,
                        BinaryOp::IndexModulo => ImageBinaryOp::IndexModulo,
                        BinaryOp::F32Add => ImageBinaryOp::F32Add,
                        BinaryOp::F32Multiply => ImageBinaryOp::F32Multiply,
                        BinaryOp::I32Subtract | _ => {
                            return Err(TranslationError::UnsupportedOperation {
                                construct: "signed 32-bit arithmetic",
                            });
                        }
                    };
                    Instruction::Binary {
                        result: self.sole_result(&results)?,
                        op,
                        lhs: self.slot(lhs)?,
                        rhs: self.slot(rhs)?,
                    }
                }
                OperationView::Compare { op, lhs, rhs } => {
                    let op = match op {
                        CompareOp::IndexLessThan => ImageCompareOp::IndexLessThan,
                        _ => {
                            return Err(TranslationError::UnsupportedOperation {
                                construct: "a comparison outside unsigned index ordering",
                            });
                        }
                    };
                    Instruction::Compare {
                        result: self.sole_result(&results)?,
                        op,
                        lhs: self.slot(lhs)?,
                        rhs: self.slot(rhs)?,
                    }
                }
                OperationView::Convert { op, source } => match op {
                    ConvertOp::CanonicalizeF32Nan => Instruction::CanonicalizeF32Nan {
                        result: self.sole_result(&results)?,
                        source: self.slot(source)?,
                    },
                    ConvertOp::U8ToI32 | ConvertOp::I32ToF32 | _ => {
                        return Err(TranslationError::UnsupportedOperation {
                            construct: "a dequantization conversion",
                        });
                    }
                },
                OperationView::PackedExtract { .. } => {
                    return Err(TranslationError::UnsupportedOperation {
                        construct: "packed-nibble extraction",
                    });
                }
                OperationView::Load { buffer, offset, .. } => Instruction::Load {
                    result: self.sole_result(&results)?,
                    buffer: self.buffer(buffer)?,
                    offset: self.slot(offset)?,
                },
                OperationView::Store {
                    buffer,
                    offset,
                    value,
                    ..
                } => Instruction::Store {
                    buffer: self.buffer(buffer)?,
                    offset: self.slot(offset)?,
                    value: self.slot(value)?,
                },
                OperationView::Predicated { predicate, body } => Instruction::Predicated {
                    predicate: self.slot(predicate)?,
                    body: self.block(body)?,
                },
                OperationView::SerialLoop(serial) => {
                    // The induction variable and the accumulator parameters are
                    // bound *inside* the body, so their slots are minted before
                    // the body is walked; the yields and the loop's own results
                    // are ordinary values of the enclosing spaces.
                    let induction = serial
                        .induction()
                        .map(|value| self.slot(value))
                        .transpose()?;
                    let accumulators = serial
                        .accumulators()
                        .map(|value| self.slot(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    let initial = serial
                        .initial()
                        .map(|value| self.slot(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    let body = self.block(serial.body())?;
                    let yields = serial
                        .yields()
                        .map(|value| self.slot(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    let loop_results = results
                        .iter()
                        .map(|value| self.slot(*value))
                        .collect::<Result<Vec<_>, _>>()?;
                    if accumulators.len() != initial.len()
                        || accumulators.len() != yields.len()
                        || accumulators.len() != loop_results.len()
                    {
                        return Err(TranslationError::UnsupportedOperation {
                            construct: "a serial loop whose accumulator arity is not uniform",
                        });
                    }
                    Instruction::SerialLoop {
                        start: serial.start(),
                        end: serial.end(),
                        induction,
                        accumulators,
                        initial,
                        yields,
                        results: loop_results,
                        body,
                    }
                }
                OperationView::Barrier { .. } => {
                    return Err(TranslationError::UnsupportedOperation {
                        construct: "a barrier, which has no participants in a scalar execution model",
                    });
                }
                _ => {
                    return Err(TranslationError::UnsupportedOperation {
                        construct: "an operation kind added to KIR after this image version",
                    });
                }
            };
            instructions.push(instruction);
        }
        Ok(Block { instructions })
    }
}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_slots(bytes: &mut Vec<u8>, slots: &[Slot]) {
    push_u32(
        bytes,
        u32::try_from(slots.len()).expect("a bounded slot list fits a u32"),
    );
    for slot in slots {
        push_u32(bytes, *slot);
    }
}

/// Serializes one container of scalar entries into carried payload bytes.
///
/// Big-endian throughout and length-framed everywhere, so the bytes are
/// independent of the host that wrote them: a payload is an artifact's content,
/// and a representation that decoded differently on another host would make the
/// artifact's own content digest a claim about the producer's machine.
#[must_use]
pub fn encode(image: &ScalarImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(CONTAINER_MAGIC);
    push_u32(
        &mut bytes,
        u32::try_from(image.entries.len()).expect("a bounded entry count fits a u32"),
    );
    for (symbol, entry) in &image.entries {
        push_u32(
            &mut bytes,
            u32::try_from(symbol.len()).expect("a bounded symbol length fits a u32"),
        );
        bytes.extend_from_slice(symbol.as_bytes());
        let encoded = encode_entry(entry);
        push_u32(
            &mut bytes,
            u32::try_from(encoded.len()).expect("a bounded entry length fits a u32"),
        );
        bytes.extend_from_slice(&encoded);
    }
    bytes
}

fn encode_entry(entry: &ScalarEntry) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(ENTRY_MAGIC);
    push_u32(&mut bytes, entry.numerics.canonical_nan_bits);
    bytes.push(entry.numerics.input_subnormals.tag());
    bytes.push(entry.numerics.result_subnormals.tag());
    push_u32(
        &mut bytes,
        u32::try_from(entry.buffers.len()).expect("a bounded buffer count fits a u32"),
    );
    for buffer in &entry.buffers {
        bytes.push(buffer.access.tag());
        push_u64(&mut bytes, buffer.element_count);
    }
    push_u32(
        &mut bytes,
        u32::try_from(entry.slot_types.len()).expect("a bounded slot count fits a u32"),
    );
    for kind in &entry.slot_types {
        bytes.push(kind.tag());
    }
    encode_block(&mut bytes, &entry.body);
    bytes
}

#[allow(
    clippy::too_many_lines,
    reason = "one arm per instruction, mirroring the decoder arm for arm so the pair is read together"
)]
fn encode_block(bytes: &mut Vec<u8>, block: &Block) {
    push_u32(
        bytes,
        u32::try_from(block.instructions.len()).expect("a bounded instruction count fits a u32"),
    );
    for instruction in &block.instructions {
        match instruction {
            Instruction::GlobalInvocationIndex { result } => {
                bytes.push(0x01);
                push_u32(bytes, *result);
            }
            Instruction::ConstBool { result, value } => {
                bytes.push(0x02);
                push_u32(bytes, *result);
                bytes.push(u8::from(*value));
            }
            Instruction::ConstIndex { result, value } => {
                bytes.push(0x03);
                push_u32(bytes, *result);
                push_u64(bytes, *value);
            }
            Instruction::ConstF32 { result, bits } => {
                bytes.push(0x04);
                push_u32(bytes, *result);
                push_u32(bytes, *bits);
            }
            Instruction::Binary {
                result,
                op,
                lhs,
                rhs,
            } => {
                bytes.push(0x05);
                push_u32(bytes, *result);
                bytes.push(op.tag());
                push_u32(bytes, *lhs);
                push_u32(bytes, *rhs);
            }
            Instruction::Compare {
                result,
                op,
                lhs,
                rhs,
            } => {
                bytes.push(0x06);
                push_u32(bytes, *result);
                bytes.push(op.tag());
                push_u32(bytes, *lhs);
                push_u32(bytes, *rhs);
            }
            Instruction::CanonicalizeF32Nan { result, source } => {
                bytes.push(0x07);
                push_u32(bytes, *result);
                push_u32(bytes, *source);
            }
            Instruction::Load {
                result,
                buffer,
                offset,
            } => {
                bytes.push(0x08);
                push_u32(bytes, *result);
                push_u32(bytes, *buffer);
                push_u32(bytes, *offset);
            }
            Instruction::Store {
                buffer,
                offset,
                value,
            } => {
                bytes.push(0x09);
                push_u32(bytes, *buffer);
                push_u32(bytes, *offset);
                push_u32(bytes, *value);
            }
            Instruction::Predicated { predicate, body } => {
                bytes.push(0x0a);
                push_u32(bytes, *predicate);
                encode_block(bytes, body);
            }
            Instruction::SerialLoop {
                start,
                end,
                induction,
                accumulators,
                initial,
                yields,
                results,
                body,
            } => {
                bytes.push(0x0b);
                push_u64(bytes, *start);
                push_u64(bytes, *end);
                match induction {
                    Some(slot) => {
                        bytes.push(0x01);
                        push_u32(bytes, *slot);
                    }
                    None => bytes.push(0x00),
                }
                push_slots(bytes, accumulators);
                push_slots(bytes, initial);
                push_slots(bytes, yields);
                push_slots(bytes, results);
                encode_block(bytes, body);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

/// Why a sequence of bytes is not a scalar CPU image this host may execute.
///
/// The classes are separate because they mean different things to do next, in
/// exactly the way `tiler_runtime::load::LoadRejection`'s are: bytes that are
/// not this representation at all send a reader to look for another payload,
/// bytes that are truncated or mis-framed send them to re-fetch, and bytes that
/// frame correctly and name a slot that does not exist are a producer defect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImageDecodeError {
    /// The bytes do not open with this representation's domain separator.
    NotThisRepresentation,
    /// A framed field extends past the end of the input.
    Truncated {
        /// Byte offset the read began at.
        at: usize,
        /// How many bytes it needed.
        needed: usize,
        /// How many remained.
        available: usize,
    },
    /// Bytes remained after the last framed field.
    TrailingBytes {
        /// How many bytes were not consumed.
        remaining: usize,
    },
    /// A discriminant tag is not one this version defines.
    UnknownTag {
        /// Which vocabulary the tag was read for.
        vocabulary: &'static str,
        /// The unrecognized tag.
        tag: u8,
    },
    /// An operand or result names a slot the entry does not declare.
    SlotOutOfRange {
        /// The named slot.
        slot: Slot,
        /// How many slots the entry declares.
        declared: usize,
    },
    /// An instruction names a buffer the signature does not declare.
    BufferOutOfRange {
        /// The named buffer.
        buffer: u32,
        /// How many buffers the signature declares.
        declared: usize,
    },
    /// An instruction accesses a buffer in a mode its parameter does not admit.
    BufferAccessViolation {
        /// The named buffer.
        buffer: u32,
        /// The mode the signature declares.
        declared: ImageAccess,
        /// The mode the instruction attempts.
        attempted: ImageAccess,
    },
    /// An operand's declared type is not the one the instruction requires.
    TypeMismatch {
        /// Which instruction the operand belongs to.
        instruction: &'static str,
        /// The type the instruction requires.
        expected: ImageType,
        /// The type the slot declares.
        found: ImageType,
    },
    /// A slot is defined more than once, so the entry is not in SSA form.
    SlotRedefined {
        /// The slot defined twice.
        slot: Slot,
    },
    /// An operand is read by an instruction that precedes its definition.
    SlotReadBeforeDefinition {
        /// The slot read too early.
        slot: Slot,
    },
    /// A symbol's bytes are not UTF-8.
    SymbolNotUtf8,
    /// A declared count exceeds this decoder's resource ceiling.
    ResourceExceeded {
        /// Which resource.
        resource: &'static str,
        /// Declared count.
        actual: usize,
        /// Admitted maximum.
        limit: usize,
    },
    /// A serial loop's accumulator arity is not uniform across its four lists.
    NonUniformLoopArity,
    /// A serial loop's range runs backwards.
    InvalidLoopRange {
        /// Inclusive first induction value.
        start: u64,
        /// Exclusive last induction value.
        end: u64,
    },
}

impl std::fmt::Display for ImageDecodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotThisRepresentation => formatter.write_str(
                "cpu.image.foreign: the payload does not open with this representation's domain",
            ),
            Self::Truncated {
                at,
                needed,
                available,
            } => write!(
                formatter,
                "cpu.image.truncated: a field at byte {at} needs {needed} byte(s) and {available} \
                 remain",
            ),
            Self::TrailingBytes { remaining } => write!(
                formatter,
                "cpu.image.trailing: {remaining} byte(s) followed the last framed field",
            ),
            Self::UnknownTag { vocabulary, tag } => write!(
                formatter,
                "cpu.image.unknown-tag: {tag:#04x} is not a {vocabulary} this version defines",
            ),
            Self::SlotOutOfRange { slot, declared } => write!(
                formatter,
                "cpu.image.slot-range: slot {slot} is named and the entry declares {declared}",
            ),
            Self::BufferOutOfRange { buffer, declared } => write!(
                formatter,
                "cpu.image.buffer-range: buffer {buffer} is named and the signature declares \
                 {declared}",
            ),
            Self::BufferAccessViolation {
                buffer,
                declared,
                attempted,
            } => write!(
                formatter,
                "cpu.image.buffer-access: buffer {buffer} is declared {declared:?} and an \
                 instruction attempts {attempted:?}",
            ),
            Self::TypeMismatch {
                instruction,
                expected,
                found,
            } => write!(
                formatter,
                "cpu.image.type: {instruction} requires {expected:?} and the slot declares \
                 {found:?}",
            ),
            Self::SlotRedefined { slot } => write!(
                formatter,
                "cpu.image.ssa: slot {slot} is defined more than once",
            ),
            Self::SlotReadBeforeDefinition { slot } => write!(
                formatter,
                "cpu.image.ssa: slot {slot} is read before the instruction defining it",
            ),
            Self::SymbolNotUtf8 => {
                formatter.write_str("cpu.image.symbol: an entry symbol is not UTF-8")
            }
            Self::ResourceExceeded {
                resource,
                actual,
                limit,
            } => write!(
                formatter,
                "cpu.image.resource: {actual} {resource}(s) declared and {limit} admitted",
            ),
            Self::NonUniformLoopArity => formatter.write_str(
                "cpu.image.loop-arity: a serial loop's accumulator lists have different lengths",
            ),
            Self::InvalidLoopRange { start, end } => write!(
                formatter,
                "cpu.image.loop-range: a serial loop runs from {start} to {end}",
            ),
        }
    }
}

impl std::error::Error for ImageDecodeError {}

/// Maximum entries one container may carry.
const MAX_ENTRIES: usize = 64;
/// Maximum buffers one entry's signature may declare.
const MAX_BUFFERS: usize = 64;
/// Maximum instructions one block may carry.
const MAX_INSTRUCTIONS: usize = 1 << 16;
/// Maximum bytes one entry symbol may occupy.
const MAX_SYMBOL_BYTES: usize = 256;

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn take(&mut self, needed: usize) -> Result<&'a [u8], ImageDecodeError> {
        let available = self.bytes.len().saturating_sub(self.at);
        if available < needed {
            return Err(ImageDecodeError::Truncated {
                at: self.at,
                needed,
                available,
            });
        }
        let slice = &self.bytes[self.at..self.at + needed];
        self.at += needed;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, ImageDecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, ImageDecodeError> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes(
            <[u8; 4]>::try_from(bytes).expect("four bytes were taken"),
        ))
    }

    fn u64(&mut self) -> Result<u64, ImageDecodeError> {
        let bytes = self.take(8)?;
        Ok(u64::from_be_bytes(
            <[u8; 8]>::try_from(bytes).expect("eight bytes were taken"),
        ))
    }

    fn magic(&mut self, expected: &[u8]) -> Result<(), ImageDecodeError> {
        let found = self
            .take(expected.len())
            .map_err(|_| ImageDecodeError::NotThisRepresentation)?;
        if found == expected {
            Ok(())
        } else {
            Err(ImageDecodeError::NotThisRepresentation)
        }
    }

    fn count(&mut self, resource: &'static str, limit: usize) -> Result<usize, ImageDecodeError> {
        let declared = usize::try_from(self.u32()?).expect("u32 fits every supported host usize");
        if declared > limit {
            return Err(ImageDecodeError::ResourceExceeded {
                resource,
                actual: declared,
                limit,
            });
        }
        Ok(declared)
    }

    const fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.at)
    }
}

/// Decodes one carried payload into an executable scalar image.
///
/// # Errors
///
/// Returns the first [`ImageDecodeError`] naming the boundary that refused.
/// Nothing partially decoded is returned, so holding a [`ScalarImage`] is the
/// evidence that every slot, buffer, tag, type, and arity check passed.
pub fn decode(bytes: &[u8]) -> Result<ScalarImage, ImageDecodeError> {
    let mut reader = Reader::new(bytes);
    reader.magic(CONTAINER_MAGIC)?;
    let count = reader.count("entry", MAX_ENTRIES)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let length = reader.count("symbol byte", MAX_SYMBOL_BYTES)?;
        let symbol = std::str::from_utf8(reader.take(length)?)
            .map_err(|_| ImageDecodeError::SymbolNotUtf8)?
            .to_owned();
        // Framed so an entry cannot read past its own extent into the next
        // one's: a decoder that shared one cursor would let a malformed entry
        // consume its neighbour and still report a well-formed container.
        let extent = usize::try_from(reader.u32()?).expect("u32 fits every supported host usize");
        let body = reader.take(extent)?;
        entries.push((symbol, decode_entry(body)?));
    }
    if reader.remaining() != 0 {
        return Err(ImageDecodeError::TrailingBytes {
            remaining: reader.remaining(),
        });
    }
    Ok(ScalarImage { entries })
}

fn decode_entry(bytes: &[u8]) -> Result<ScalarEntry, ImageDecodeError> {
    let mut reader = Reader::new(bytes);
    reader.magic(ENTRY_MAGIC)?;
    let canonical_nan_bits = reader.u32()?;
    let input_tag = reader.u8()?;
    let input_subnormals =
        ImageSubnormals::from_tag(input_tag).ok_or(ImageDecodeError::UnknownTag {
            vocabulary: "subnormal realization",
            tag: input_tag,
        })?;
    let result_tag = reader.u8()?;
    let result_subnormals =
        ImageSubnormals::from_tag(result_tag).ok_or(ImageDecodeError::UnknownTag {
            vocabulary: "subnormal realization",
            tag: result_tag,
        })?;

    let buffer_count = reader.count("buffer", MAX_BUFFERS)?;
    let mut buffers = Vec::with_capacity(buffer_count);
    for _ in 0..buffer_count {
        let access_tag = reader.u8()?;
        let access = ImageAccess::from_tag(access_tag).ok_or(ImageDecodeError::UnknownTag {
            vocabulary: "buffer access mode",
            tag: access_tag,
        })?;
        buffers.push(ImageBuffer {
            access,
            element_count: reader.u64()?,
        });
    }

    let slot_count = reader.count("slot", MAX_SLOTS)?;
    let mut slot_types = Vec::with_capacity(slot_count);
    for _ in 0..slot_count {
        let tag = reader.u8()?;
        slot_types.push(
            ImageType::from_tag(tag).ok_or(ImageDecodeError::UnknownTag {
                vocabulary: "value type",
                tag,
            })?,
        );
    }

    let mut validator = Validator {
        slot_types: &slot_types,
        buffers: &buffers,
        defined: vec![false; slot_count],
    };
    let body = decode_block(&mut reader, &mut validator)?;
    if reader.remaining() != 0 {
        return Err(ImageDecodeError::TrailingBytes {
            remaining: reader.remaining(),
        });
    }
    Ok(ScalarEntry {
        numerics: ImageNumerics {
            canonical_nan_bits,
            input_subnormals,
            result_subnormals,
        },
        buffers,
        slot_types,
        body,
    })
}

/// Checks every slot, buffer, type, and SSA obligation while a body decodes.
///
/// Checked here rather than at execution because a payload that names a slot it
/// does not declare must be refused by the loader, not discovered by an
/// interpreter mid-dispatch: after the routing commit there is nothing left to
/// fall back to, and this decode runs while the preflight is still held.
struct Validator<'a> {
    slot_types: &'a [ImageType],
    buffers: &'a [ImageBuffer],
    defined: Vec<bool>,
}

impl Validator<'_> {
    /// Proves an operand's slot exists, carries the required type, and has
    /// already been defined by an earlier instruction.
    ///
    /// Definition-before-use is checked because the decoder walks the body in
    /// execution order, so "already defined here" is exactly "already defined
    /// when this runs". Leaving it to the interpreter would move a decidable
    /// refusal past the routing commit.
    fn read(
        &self,
        slot: Slot,
        instruction: &'static str,
        expected: ImageType,
    ) -> Result<(), ImageDecodeError> {
        self.typed(slot, instruction, expected)?;
        let at = usize::try_from(slot).expect("u32 fits every supported host usize");
        if self.defined[at] {
            Ok(())
        } else {
            Err(ImageDecodeError::SlotReadBeforeDefinition { slot })
        }
    }

    fn typed(
        &self,
        slot: Slot,
        instruction: &'static str,
        expected: ImageType,
    ) -> Result<(), ImageDecodeError> {
        let found = self.kind(slot)?;
        if found == expected {
            Ok(())
        } else {
            Err(ImageDecodeError::TypeMismatch {
                instruction,
                expected,
                found,
            })
        }
    }

    fn kind(&self, slot: Slot) -> Result<ImageType, ImageDecodeError> {
        let at = usize::try_from(slot).expect("u32 fits every supported host usize");
        self.slot_types
            .get(at)
            .copied()
            .ok_or(ImageDecodeError::SlotOutOfRange {
                slot,
                declared: self.slot_types.len(),
            })
    }

    fn define(
        &mut self,
        slot: Slot,
        instruction: &'static str,
        expected: ImageType,
    ) -> Result<(), ImageDecodeError> {
        self.typed(slot, instruction, expected)?;
        let at = usize::try_from(slot).expect("u32 fits every supported host usize");
        if std::mem::replace(&mut self.defined[at], true) {
            return Err(ImageDecodeError::SlotRedefined { slot });
        }
        Ok(())
    }

    /// Proves a named buffer exists and admits the access being decoded.
    ///
    /// The access mode is checked here rather than at execution for the same
    /// reason every other obligation is: a store into a read-only parameter is
    /// decidable from the bytes, and discovering it mid-dispatch would be a
    /// refusal after the routing commit.
    fn buffer(&self, buffer: u32, access: ImageAccess) -> Result<(), ImageDecodeError> {
        let at = usize::try_from(buffer).expect("u32 fits every supported host usize");
        let declared = self
            .buffers
            .get(at)
            .ok_or(ImageDecodeError::BufferOutOfRange {
                buffer,
                declared: self.buffers.len(),
            })?;
        if declared.access == access {
            Ok(())
        } else {
            Err(ImageDecodeError::BufferAccessViolation {
                buffer,
                declared: declared.access,
                attempted: access,
            })
        }
    }
}

fn decode_slots(
    reader: &mut Reader<'_>,
    validator: &Validator<'_>,
) -> Result<Vec<Slot>, ImageDecodeError> {
    let count = reader.count("slot reference", MAX_SLOTS)?;
    let mut slots = Vec::with_capacity(count);
    for _ in 0..count {
        let slot = reader.u32()?;
        validator.kind(slot)?;
        slots.push(slot);
    }
    Ok(slots)
}

#[allow(
    clippy::too_many_lines,
    reason = "one arm per instruction tag, mirroring the encoder arm for arm so the pair is read together"
)]
fn decode_block(
    reader: &mut Reader<'_>,
    validator: &mut Validator<'_>,
) -> Result<Block, ImageDecodeError> {
    let count = reader.count("instruction", MAX_INSTRUCTIONS)?;
    let mut instructions = Vec::with_capacity(count);
    for _ in 0..count {
        let tag = reader.u8()?;
        let instruction = match tag {
            0x01 => {
                let result = reader.u32()?;
                validator.define(result, "a global invocation index", ImageType::Index)?;
                Instruction::GlobalInvocationIndex { result }
            }
            0x02 => {
                let result = reader.u32()?;
                validator.define(result, "a bool constant", ImageType::Bool)?;
                Instruction::ConstBool {
                    result,
                    value: reader.u8()? != 0,
                }
            }
            0x03 => {
                let result = reader.u32()?;
                validator.define(result, "an index constant", ImageType::Index)?;
                Instruction::ConstIndex {
                    result,
                    value: reader.u64()?,
                }
            }
            0x04 => {
                let result = reader.u32()?;
                validator.define(result, "an f32 constant", ImageType::F32)?;
                Instruction::ConstF32 {
                    result,
                    bits: reader.u32()?,
                }
            }
            0x05 => {
                let result = reader.u32()?;
                let op_tag = reader.u8()?;
                let op = ImageBinaryOp::from_tag(op_tag).ok_or(ImageDecodeError::UnknownTag {
                    vocabulary: "binary operation",
                    tag: op_tag,
                })?;
                let lhs = reader.u32()?;
                let rhs = reader.u32()?;
                validator.read(lhs, "a binary operation", op.value_type())?;
                validator.read(rhs, "a binary operation", op.value_type())?;
                validator.define(result, "a binary operation", op.value_type())?;
                Instruction::Binary {
                    result,
                    op,
                    lhs,
                    rhs,
                }
            }
            0x06 => {
                let result = reader.u32()?;
                let op_tag = reader.u8()?;
                let op = ImageCompareOp::from_tag(op_tag).ok_or(ImageDecodeError::UnknownTag {
                    vocabulary: "comparison",
                    tag: op_tag,
                })?;
                let lhs = reader.u32()?;
                let rhs = reader.u32()?;
                validator.read(lhs, "a comparison", ImageType::Index)?;
                validator.read(rhs, "a comparison", ImageType::Index)?;
                validator.define(result, "a comparison", ImageType::Bool)?;
                Instruction::Compare {
                    result,
                    op,
                    lhs,
                    rhs,
                }
            }
            0x07 => {
                let result = reader.u32()?;
                let source = reader.u32()?;
                validator.read(source, "a NaN canonicalization", ImageType::F32)?;
                validator.define(result, "a NaN canonicalization", ImageType::F32)?;
                Instruction::CanonicalizeF32Nan { result, source }
            }
            0x08 => {
                let result = reader.u32()?;
                let buffer = reader.u32()?;
                let offset = reader.u32()?;
                validator.buffer(buffer, ImageAccess::Read)?;
                validator.read(offset, "a load", ImageType::Index)?;
                validator.define(result, "a load", ImageType::F32)?;
                Instruction::Load {
                    result,
                    buffer,
                    offset,
                }
            }
            0x09 => {
                let buffer = reader.u32()?;
                let offset = reader.u32()?;
                let value = reader.u32()?;
                validator.buffer(buffer, ImageAccess::Write)?;
                validator.read(offset, "a store", ImageType::Index)?;
                validator.read(value, "a store", ImageType::F32)?;
                Instruction::Store {
                    buffer,
                    offset,
                    value,
                }
            }
            0x0a => {
                let predicate = reader.u32()?;
                validator.read(predicate, "a predicated block", ImageType::Bool)?;
                Instruction::Predicated {
                    predicate,
                    body: decode_block(reader, validator)?,
                }
            }
            0x0b => {
                let start = reader.u64()?;
                let end = reader.u64()?;
                if end < start {
                    return Err(ImageDecodeError::InvalidLoopRange { start, end });
                }
                let induction = match reader.u8()? {
                    0x00 => None,
                    0x01 => {
                        let slot = reader.u32()?;
                        validator.define(slot, "a loop induction variable", ImageType::Index)?;
                        Some(slot)
                    }
                    tag => {
                        return Err(ImageDecodeError::UnknownTag {
                            vocabulary: "loop induction presence",
                            tag,
                        });
                    }
                };
                let accumulators = decode_slots(reader, validator)?;
                for slot in &accumulators {
                    let kind = validator.kind(*slot)?;
                    validator.define(*slot, "a loop accumulator", kind)?;
                }
                let initial = decode_slots(reader, validator)?;
                let yields = decode_slots(reader, validator)?;
                let results = decode_slots(reader, validator)?;
                if accumulators.len() != initial.len()
                    || accumulators.len() != yields.len()
                    || accumulators.len() != results.len()
                {
                    return Err(ImageDecodeError::NonUniformLoopArity);
                }
                // Every carried value keeps one type through the whole cycle:
                // the initial value, the parameter it binds, the value yielded
                // back into it, and the result published out of it.
                for position in 0..accumulators.len() {
                    let kind = validator.kind(accumulators[position])?;
                    validator.read(initial[position], "a loop initial value", kind)?;
                    validator.typed(yields[position], "a loop yield", kind)?;
                    validator.define(results[position], "a loop result", kind)?;
                }
                let body = decode_block(reader, validator)?;
                // Deferred until the body has decoded, because a yielded value
                // is defined *inside* the loop: checking it above would refuse
                // every well-formed loop.
                for position in 0..accumulators.len() {
                    let kind = validator.kind(accumulators[position])?;
                    validator.read(yields[position], "a loop yield", kind)?;
                }
                Instruction::SerialLoop {
                    start,
                    end,
                    induction,
                    accumulators,
                    initial,
                    yields,
                    results,
                    body,
                }
            }
            tag => {
                return Err(ImageDecodeError::UnknownTag {
                    vocabulary: "instruction",
                    tag,
                });
            }
        };
        instructions.push(instruction);
    }
    Ok(Block { instructions })
}
