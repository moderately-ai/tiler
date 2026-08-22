//! Bounded scalar CPU backend: translate KIR, assemble, validate, execute.
//!
//! This is not a production CPU backend. It claims the CPU family rows of the
//! ADR 0090 responsibility matrix: a representation, a payload validator, and a
//! runtime adapter. It installs no physical provider and needs none.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ArithmeticType, ArtifactExecutionPolicy, AvailabilityPhase,
    BackendEntryKey, BackendKey, BindingKind, BindingTarget, DigestAlgorithm, PayloadContent,
    PayloadEntryMapping, PayloadMetadata, PayloadPlatform, PayloadProvenance,
    RecordedArtifactProgramIdentity, RepresentationKey, SchemaVersion, TargetProfileRef,
    ToolComponent, VerifiedArtifactProgram,
};
use tiler_build::{BackendEntryDeclaration, PlanDeterminismDeclaration, assemble_plan_artifact};
use tiler_compiler::session::PlanAlternative;
use tiler_ir::kernel::{
    AddressSpace, BinaryOp, BlockRef, BufferAccess, BufferParameter, CompareOp, ConvertOp,
    KernelConstant, KernelType, OperationView, VerifiedBufferId, VerifiedKernel, VerifiedValueId,
};
use tiler_ir::program::StageRef;
use tiler_ir::semantic::SemanticProgram;
use tiler_runtime::adapter::{LiveExecutionContext, RuntimeAdapter, route_with_adapter};
use tiler_runtime::load::{
    DTypeDispatch, DecodedProgram, ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest,
    LoadRejection, Preflight, PreparedEntryObservation, RoutedDispatch, RoutedEntry,
    TargetEnvironmentObservation, TargetEnvironmentSupport, TargetPropertyRequest,
};

/// Governed backend family of this spike's CPU path.
pub const BACKEND_KEY: &str = "tiler.cpu.scalar";
/// Governed executable representation of the carried image.
pub const REPRESENTATION_KEY: &str = "tiler.cpu.scalar-image-v1";
/// Governed representation of the retained kernel-identity source.
pub const SOURCE_REPRESENTATION_KEY: &str = "tiler.cpu.kernel-identity-list-v1";
/// In-process translator identity recorded as payload provenance.
pub const TOOLCHAIN_KEY: &str = "tiler.cpu.scalar-translator";
/// Domain separator opening one serialized container.
const CONTAINER_MAGIC: &[u8] = b"tiler.cpu.scalar-image-v1\0";
/// Domain separator under which entry-point symbols are derived.
const SYMBOL_DOMAIN: &[u8] = b"tiler.cpu.scalar.symbol.v1\0";
/// Governed schema version of this backend's payload.
const PAYLOAD_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0);
/// Byte width of one `f32`.
const F32_BYTES: u64 = 4;
/// Exact prepared-entry property this adapter owns.
const MAX_THREADS_PER_WORKGROUP_PROPERTY: &str =
    "tiler.target.prepared-entry.max-threads-per-workgroup.v1";
/// Exact provider namespace for the prepared-entry property this adapter owns.
const PREPARED_ENTRY_PROVIDER_NAMESPACE: &str = "tiler";
/// Exact provider name for the prepared-entry property this adapter owns.
const PREPARED_ENTRY_PROVIDER_NAME: &str = "prepared-entry-properties";
/// Exact provider revision for the prepared-entry property this adapter owns.
const PREPARED_ENTRY_PROVIDER_REVISION: u32 = 1;
/// Bounded workgroup capacity this scalar adapter enforces for each prepared entry.
const MAX_THREADS_PER_WORKGROUP: u64 = 1_024;
/// The one delivery position this spike's artifacts declare.
pub const SOLE_DELIVERY: usize = 0;

/// Why the CPU path refused a plan, payload, or dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CpuError {
    /// A kernel binds a number of buffers this backend cannot place.
    UnsupportedBufferCount {
        /// Ordinal of the refused kernel.
        entry: usize,
        /// Buffers the kernel binds.
        buffers: usize,
    },
    /// Translation refused a construct by name.
    Translate(&'static str),
    /// The artifact layer or assembly seam refused a declaration.
    Assemble(String),
    /// The carried bytes are not a scalar image this backend executes.
    Payload(String),
    /// A mapped symbol names no entry of the carried image.
    UnmappedSymbol(String),
    /// The loader refused the route.
    Load(String),
    /// The adapter refused before the commit.
    Adapter(String),
    /// The committed dispatch failed.
    Execute(String),
    /// Output bits disagreed with the independent reference.
    Mismatch {
        /// Index of the first disagreeing element.
        index: usize,
        /// Bits the CPU backend produced.
        actual: u32,
        /// Bits the reference requires.
        expected: u32,
    },
}

impl fmt::Display for CpuError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedBufferCount { entry, buffers } => write!(
                formatter,
                "cpu: kernel {entry} binds {buffers} buffers and this backend places two",
            ),
            Self::Translate(construct) => {
                write!(
                    formatter,
                    "cpu.translate: this backend does not implement {construct}"
                )
            }
            Self::Assemble(message) => write!(formatter, "cpu.assemble: {message}"),
            Self::Payload(message) => write!(formatter, "cpu.payload: {message}"),
            Self::UnmappedSymbol(symbol) => {
                write!(
                    formatter,
                    "cpu.payload: symbol `{symbol}` names no image entry"
                )
            }
            Self::Load(message) => write!(formatter, "cpu.load: {message}"),
            Self::Adapter(message) => write!(formatter, "cpu.adapter: {message}"),
            Self::Execute(message) => write!(formatter, "cpu.execute: {message}"),
            Self::Mismatch {
                index,
                actual,
                expected,
            } => write!(
                formatter,
                "cpu.compare: element {index} is 0x{actual:08x} and the reference requires 0x{expected:08x}",
            ),
        }
    }
}

impl std::error::Error for CpuError {}

/// One buffer parameter of a scalar entry, in signature order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ImageBuffer {
    write: bool,
    element_count: u64,
}

/// One instruction of a scalar entry.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Instruction {
    GlobalInvocationIndex {
        result: u32,
    },
    ConstF32 {
        result: u32,
        bits: u32,
    },
    ConstIndex {
        result: u32,
        value: u64,
    },
    F32Multiply {
        result: u32,
        lhs: u32,
        rhs: u32,
    },
    F32Add {
        result: u32,
        lhs: u32,
        rhs: u32,
    },
    IndexAdd {
        result: u32,
        lhs: u32,
        rhs: u32,
    },
    IndexMultiply {
        result: u32,
        lhs: u32,
        rhs: u32,
    },
    CanonicalizeF32Nan {
        result: u32,
        source: u32,
    },
    Load {
        result: u32,
        buffer: u32,
        offset: u32,
    },
    Store {
        buffer: u32,
        offset: u32,
        value: u32,
    },
    IndexLessThan {
        result: u32,
        lhs: u32,
        rhs: u32,
    },
    Predicated {
        predicate: u32,
        body: Vec<Instruction>,
    },
}

/// One translated scalar entry.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ScalarEntry {
    symbol: String,
    numerics_nan: u32,
    buffers: Vec<ImageBuffer>,
    slot_count: u32,
    body: Vec<Instruction>,
}

/// A container of translated scalar entries, keyed by backend symbol.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ScalarImage {
    entries: Vec<ScalarEntry>,
}

impl ScalarImage {
    fn entry_for(&self, symbol: &str) -> Option<&ScalarEntry> {
        self.entries.iter().find(|entry| entry.symbol == symbol)
    }
}

/// Derives one entry-point symbol from a kernel's canonical identity.
fn symbol_for(kernel: &VerifiedKernel) -> String {
    let digest = DigestAlgorithm::GOVERNED
        .digest(SYMBOL_DOMAIN, kernel.canonical_identity().as_bytes())
        .label();
    format!("scalar_cpu_{}", &digest[..16])
}

/// Translates verified structured kernels into this backend's image.
fn emit(kernels: &[&VerifiedKernel]) -> Result<ScalarImage, CpuError> {
    let mut entries = Vec::with_capacity(kernels.len());
    for (ordinal, kernel) in kernels.iter().enumerate() {
        let buffers: Vec<_> = kernel.buffers().collect();
        if buffers.len() != 2 {
            return Err(CpuError::UnsupportedBufferCount {
                entry: ordinal,
                buffers: buffers.len(),
            });
        }
        entries.push(translate_kernel(kernel)?);
    }
    Ok(ScalarImage { entries })
}

fn translate_buffer(parameter: BufferParameter) -> Result<ImageBuffer, CpuError> {
    if parameter.element_type != KernelType::F32 {
        return Err(CpuError::Translate("a non-f32 buffer"));
    }
    if parameter.address_space != AddressSpace::Device {
        return Err(CpuError::Translate("a non-device address space"));
    }
    Ok(ImageBuffer {
        write: parameter.access == BufferAccess::Write,
        element_count: parameter.element_count,
    })
}

struct Translator<'a> {
    kernel: &'a VerifiedKernel,
    buffer_slots: HashMap<VerifiedBufferId, u32>,
    slots: HashMap<VerifiedValueId, u32>,
    next_slot: u32,
}

impl Translator<'_> {
    fn slot(&mut self, value: VerifiedValueId) -> Result<u32, CpuError> {
        if let Some(slot) = self.slots.get(&value) {
            return Ok(*slot);
        }
        let kind = self
            .kernel
            .value_type(value)
            .map_err(|_| CpuError::Translate("a value handle the kernel does not retain"))?;
        match kind {
            KernelType::Bool | KernelType::Index | KernelType::F32 => {}
            KernelType::U8 | KernelType::I32 | KernelType::Bf16 | KernelType::U32 => {
                return Err(CpuError::Translate("a value type this image refuses"));
            }
        }
        let slot = self.next_slot;
        self.next_slot = self
            .next_slot
            .checked_add(1)
            .ok_or(CpuError::Translate("too many slots"))?;
        self.slots.insert(value, slot);
        Ok(slot)
    }

    fn buffer(&self, id: VerifiedBufferId) -> Result<u32, CpuError> {
        self.buffer_slots
            .get(&id)
            .copied()
            .ok_or(CpuError::Translate(
                "a buffer the signature does not declare",
            ))
    }

    fn sole_result(&mut self, results: &[VerifiedValueId]) -> Result<u32, CpuError> {
        let [result] = results else {
            return Err(CpuError::Translate(
                "an operation defining other than exactly one result",
            ));
        };
        self.slot(*result)
    }

    fn block(&mut self, block: BlockRef<'_>) -> Result<Vec<Instruction>, CpuError> {
        let mut instructions = Vec::new();
        for operation in block.operations() {
            let results: Vec<_> = operation.results().collect();
            instructions.push(match operation.view() {
                OperationView::Builtin { builtin } => match builtin {
                    tiler_ir::kernel::Builtin::GlobalInvocationIndex => {
                        Instruction::GlobalInvocationIndex {
                            result: self.sole_result(&results)?,
                        }
                    }
                    tiler_ir::kernel::Builtin::LocalInvocationIndex => {
                        return Err(CpuError::Translate("LocalInvocationIndex"));
                    }
                    _ => return Err(CpuError::Translate("an unadmitted launch builtin")),
                },
                OperationView::Constant { value } => {
                    let result = self.sole_result(&results)?;
                    match value {
                        KernelConstant::F32Bits(bits) => Instruction::ConstF32 { result, bits },
                        KernelConstant::Index(index) => Instruction::ConstIndex {
                            result,
                            value: index,
                        },
                        KernelConstant::Bool(_) => {
                            return Err(CpuError::Translate("a bool constant"));
                        }
                        _ => {
                            return Err(CpuError::Translate(
                                "a constant outside the image vocabulary",
                            ));
                        }
                    }
                }
                OperationView::Binary { op, lhs, rhs } => {
                    let result = self.sole_result(&results)?;
                    let lhs = self.slot(lhs)?;
                    let rhs = self.slot(rhs)?;
                    match op {
                        BinaryOp::F32Multiply => Instruction::F32Multiply { result, lhs, rhs },
                        BinaryOp::F32Add => Instruction::F32Add { result, lhs, rhs },
                        BinaryOp::IndexAdd => Instruction::IndexAdd { result, lhs, rhs },
                        BinaryOp::IndexMultiply => Instruction::IndexMultiply { result, lhs, rhs },
                        BinaryOp::IndexDivide
                        | BinaryOp::IndexModulo
                        | BinaryOp::IndexSubtract
                        | BinaryOp::I32Subtract
                        | BinaryOp::F32Divide
                        | BinaryOp::F32Maximum
                        | _ => {
                            return Err(CpuError::Translate("an unadmitted binary operation"));
                        }
                    }
                }
                OperationView::Compare { op, lhs, rhs } => match op {
                    CompareOp::IndexLessThan => Instruction::IndexLessThan {
                        result: self.sole_result(&results)?,
                        lhs: self.slot(lhs)?,
                        rhs: self.slot(rhs)?,
                    },
                    _ => {
                        return Err(CpuError::Translate(
                            "a comparison outside unsigned index ordering",
                        ));
                    }
                },
                OperationView::Convert { op, source } => match op {
                    ConvertOp::CanonicalizeF32Nan => Instruction::CanonicalizeF32Nan {
                        result: self.sole_result(&results)?,
                        source: self.slot(source)?,
                    },
                    ConvertOp::U8ToI32 | ConvertOp::I32ToF32 | ConvertOp::CanonicalizeBf16Nan => {
                        return Err(CpuError::Translate("a conversion this image refuses"));
                    }
                    _ => return Err(CpuError::Translate("an unadmitted conversion")),
                },
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
                OperationView::PackedExtract { .. } => {
                    return Err(CpuError::Translate("packed-nibble extraction"));
                }
                OperationView::Unary { .. } => {
                    return Err(CpuError::Translate("a unary elementary function"));
                }
                OperationView::SerialLoop(_) => {
                    return Err(CpuError::Translate("a serial loop"));
                }
                OperationView::Barrier { .. } => {
                    return Err(CpuError::Translate("a barrier"));
                }
                _ => {
                    return Err(CpuError::Translate(
                        "an operation kind this image version does not name",
                    ));
                }
            });
        }
        Ok(instructions)
    }
}

fn translate_kernel(kernel: &VerifiedKernel) -> Result<ScalarEntry, CpuError> {
    let mut buffers = Vec::new();
    let mut buffer_slots = HashMap::new();
    for (position, (id, parameter)) in kernel.declared_buffers().enumerate() {
        buffers.push(translate_buffer(parameter)?);
        buffer_slots.insert(
            id,
            u32::try_from(position).map_err(|_| CpuError::Translate("too many buffers"))?,
        );
    }
    let mut translator = Translator {
        kernel,
        buffer_slots,
        slots: HashMap::new(),
        next_slot: 0,
    };
    let body = translator.block(kernel.body())?;
    Ok(ScalarEntry {
        symbol: symbol_for(kernel),
        numerics_nan: kernel.numerical().canonical_arithmetic_nan_bits,
        buffers,
        slot_count: translator.next_slot,
        body,
    })
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn encode_instructions(bytes: &mut Vec<u8>, instructions: &[Instruction]) {
    push_u32(
        bytes,
        u32::try_from(instructions.len()).expect("a bounded instruction list fits u32"),
    );
    for instruction in instructions {
        match instruction {
            Instruction::GlobalInvocationIndex { result } => {
                bytes.push(0x01);
                push_u32(bytes, *result);
            }
            Instruction::ConstF32 { result, bits } => {
                bytes.push(0x02);
                push_u32(bytes, *result);
                push_u32(bytes, *bits);
            }
            Instruction::ConstIndex { result, value } => {
                bytes.push(0x03);
                push_u32(bytes, *result);
                push_u64(bytes, *value);
            }
            Instruction::F32Multiply { result, lhs, rhs } => {
                bytes.push(0x04);
                push_u32(bytes, *result);
                push_u32(bytes, *lhs);
                push_u32(bytes, *rhs);
            }
            Instruction::F32Add { result, lhs, rhs } => {
                bytes.push(0x05);
                push_u32(bytes, *result);
                push_u32(bytes, *lhs);
                push_u32(bytes, *rhs);
            }
            Instruction::IndexAdd { result, lhs, rhs } => {
                bytes.push(0x06);
                push_u32(bytes, *result);
                push_u32(bytes, *lhs);
                push_u32(bytes, *rhs);
            }
            Instruction::IndexMultiply { result, lhs, rhs } => {
                bytes.push(0x07);
                push_u32(bytes, *result);
                push_u32(bytes, *lhs);
                push_u32(bytes, *rhs);
            }
            Instruction::CanonicalizeF32Nan { result, source } => {
                bytes.push(0x08);
                push_u32(bytes, *result);
                push_u32(bytes, *source);
            }
            Instruction::Load {
                result,
                buffer,
                offset,
            } => {
                bytes.push(0x09);
                push_u32(bytes, *result);
                push_u32(bytes, *buffer);
                push_u32(bytes, *offset);
            }
            Instruction::Store {
                buffer,
                offset,
                value,
            } => {
                bytes.push(0x0a);
                push_u32(bytes, *buffer);
                push_u32(bytes, *offset);
                push_u32(bytes, *value);
            }
            Instruction::IndexLessThan { result, lhs, rhs } => {
                bytes.push(0x0b);
                push_u32(bytes, *result);
                push_u32(bytes, *lhs);
                push_u32(bytes, *rhs);
            }
            Instruction::Predicated { predicate, body } => {
                bytes.push(0x0c);
                push_u32(bytes, *predicate);
                encode_instructions(bytes, body);
            }
        }
    }
}

fn encode(image: &ScalarImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(CONTAINER_MAGIC);
    push_u32(
        &mut bytes,
        u32::try_from(image.entries.len()).expect("a bounded entry count fits u32"),
    );
    for entry in &image.entries {
        let symbol = entry.symbol.as_bytes();
        push_u32(
            &mut bytes,
            u32::try_from(symbol.len()).expect("a bounded symbol length fits u32"),
        );
        bytes.extend_from_slice(symbol);
        push_u32(&mut bytes, entry.numerics_nan);
        push_u32(&mut bytes, entry.slot_count);
        push_u32(
            &mut bytes,
            u32::try_from(entry.buffers.len()).expect("a bounded buffer count fits u32"),
        );
        for buffer in &entry.buffers {
            bytes.push(u8::from(buffer.write));
            push_u64(&mut bytes, buffer.element_count);
        }
        encode_instructions(&mut bytes, &entry.body);
    }
    bytes
}

fn take_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, CpuError> {
    let start = *cursor;
    let end = start
        .checked_add(4)
        .ok_or_else(|| CpuError::Payload("truncated u32".into()))?;
    let slice = bytes
        .get(start..end)
        .ok_or_else(|| CpuError::Payload("truncated u32".into()))?;
    *cursor = end;
    Ok(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn take_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, CpuError> {
    let start = *cursor;
    let end = start
        .checked_add(8)
        .ok_or_else(|| CpuError::Payload("truncated u64".into()))?;
    let slice = bytes
        .get(start..end)
        .ok_or_else(|| CpuError::Payload("truncated u64".into()))?;
    *cursor = end;
    Ok(u64::from_be_bytes([
        slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
    ]))
}

fn take_u8(bytes: &[u8], cursor: &mut usize) -> Result<u8, CpuError> {
    let value = *bytes
        .get(*cursor)
        .ok_or_else(|| CpuError::Payload("truncated byte".into()))?;
    *cursor += 1;
    Ok(value)
}

fn decode_instructions(bytes: &[u8], cursor: &mut usize) -> Result<Vec<Instruction>, CpuError> {
    let count = take_u32(bytes, cursor)?;
    let mut instructions = Vec::with_capacity(count as usize);
    for _ in 0..count {
        instructions.push(match take_u8(bytes, cursor)? {
            0x01 => Instruction::GlobalInvocationIndex {
                result: take_u32(bytes, cursor)?,
            },
            0x02 => Instruction::ConstF32 {
                result: take_u32(bytes, cursor)?,
                bits: take_u32(bytes, cursor)?,
            },
            0x03 => Instruction::ConstIndex {
                result: take_u32(bytes, cursor)?,
                value: take_u64(bytes, cursor)?,
            },
            0x04 => Instruction::F32Multiply {
                result: take_u32(bytes, cursor)?,
                lhs: take_u32(bytes, cursor)?,
                rhs: take_u32(bytes, cursor)?,
            },
            0x05 => Instruction::F32Add {
                result: take_u32(bytes, cursor)?,
                lhs: take_u32(bytes, cursor)?,
                rhs: take_u32(bytes, cursor)?,
            },
            0x06 => Instruction::IndexAdd {
                result: take_u32(bytes, cursor)?,
                lhs: take_u32(bytes, cursor)?,
                rhs: take_u32(bytes, cursor)?,
            },
            0x07 => Instruction::IndexMultiply {
                result: take_u32(bytes, cursor)?,
                lhs: take_u32(bytes, cursor)?,
                rhs: take_u32(bytes, cursor)?,
            },
            0x08 => Instruction::CanonicalizeF32Nan {
                result: take_u32(bytes, cursor)?,
                source: take_u32(bytes, cursor)?,
            },
            0x09 => Instruction::Load {
                result: take_u32(bytes, cursor)?,
                buffer: take_u32(bytes, cursor)?,
                offset: take_u32(bytes, cursor)?,
            },
            0x0a => Instruction::Store {
                buffer: take_u32(bytes, cursor)?,
                offset: take_u32(bytes, cursor)?,
                value: take_u32(bytes, cursor)?,
            },
            0x0b => Instruction::IndexLessThan {
                result: take_u32(bytes, cursor)?,
                lhs: take_u32(bytes, cursor)?,
                rhs: take_u32(bytes, cursor)?,
            },
            0x0c => Instruction::Predicated {
                predicate: take_u32(bytes, cursor)?,
                body: decode_instructions(bytes, cursor)?,
            },
            tag => {
                return Err(CpuError::Payload(format!(
                    "unknown instruction tag 0x{tag:02x}"
                )));
            }
        });
    }
    Ok(instructions)
}

fn decode(bytes: &[u8]) -> Result<ScalarImage, CpuError> {
    if !bytes.starts_with(CONTAINER_MAGIC) {
        return Err(CpuError::Payload("foreign domain separator".into()));
    }
    let mut cursor = CONTAINER_MAGIC.len();
    let count = take_u32(bytes, &mut cursor)?;
    let mut entries = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let symbol_len = take_u32(bytes, &mut cursor)? as usize;
        let symbol_bytes = bytes
            .get(cursor..cursor + symbol_len)
            .ok_or_else(|| CpuError::Payload("truncated symbol".into()))?;
        let symbol = std::str::from_utf8(symbol_bytes)
            .map_err(|_| CpuError::Payload("non-utf8 symbol".into()))?
            .to_owned();
        cursor += symbol_len;
        let numerics_nan = take_u32(bytes, &mut cursor)?;
        let slot_count = take_u32(bytes, &mut cursor)?;
        let buffer_count = take_u32(bytes, &mut cursor)?;
        let mut buffers = Vec::with_capacity(buffer_count as usize);
        for _ in 0..buffer_count {
            let write = take_u8(bytes, &mut cursor)? != 0;
            let element_count = take_u64(bytes, &mut cursor)?;
            buffers.push(ImageBuffer {
                write,
                element_count,
            });
        }
        let body = decode_instructions(bytes, &mut cursor)?;
        entries.push(ScalarEntry {
            symbol,
            numerics_nan,
            buffers,
            slot_count,
            body,
        });
    }
    if cursor != bytes.len() {
        return Err(CpuError::Payload("trailing bytes".into()));
    }
    Ok(ScalarImage { entries })
}

fn canonicalize(bits: u32, nan: u32) -> u32 {
    if f32::from_bits(bits).is_nan() {
        nan
    } else {
        bits
    }
}

fn mul_f32(lhs: u32, rhs: u32, nan: u32) -> u32 {
    canonicalize((f32::from_bits(lhs) * f32::from_bits(rhs)).to_bits(), nan)
}

fn add_f32(lhs: u32, rhs: u32, nan: u32) -> u32 {
    canonicalize((f32::from_bits(lhs) + f32::from_bits(rhs)).to_bits(), nan)
}

enum Cell {
    Unset,
    Bool(bool),
    Index(u64),
    F32(u32),
}

fn execute_entry(
    entry: &ScalarEntry,
    invocation: u64,
    allocations: &mut [Vec<u8>],
    placements: &[(usize, u64, u64)],
) -> Result<(), CpuError> {
    let mut slots = (0..entry.slot_count)
        .map(|_| Cell::Unset)
        .collect::<Vec<_>>();
    execute_block(
        &entry.body,
        entry,
        invocation,
        allocations,
        placements,
        &mut slots,
    )
}

fn read_f32(slots: &[Cell], slot: u32) -> Result<u32, CpuError> {
    match slots.get(slot as usize) {
        Some(Cell::F32(bits)) => Ok(*bits),
        _ => Err(CpuError::Execute(format!("slot {slot} is not an f32"))),
    }
}

fn read_index(slots: &[Cell], slot: u32) -> Result<u64, CpuError> {
    match slots.get(slot as usize) {
        Some(Cell::Index(value)) => Ok(*value),
        _ => Err(CpuError::Execute(format!("slot {slot} is not an index"))),
    }
}

fn read_bool(slots: &[Cell], slot: u32) -> Result<bool, CpuError> {
    match slots.get(slot as usize) {
        Some(Cell::Bool(value)) => Ok(*value),
        _ => Err(CpuError::Execute(format!("slot {slot} is not a bool"))),
    }
}

fn write_slot(slots: &mut [Cell], slot: u32, value: Cell) -> Result<(), CpuError> {
    let cell = slots
        .get_mut(slot as usize)
        .ok_or_else(|| CpuError::Execute(format!("slot {slot} is out of range")))?;
    *cell = value;
    Ok(())
}

fn execute_block(
    body: &[Instruction],
    entry: &ScalarEntry,
    invocation: u64,
    allocations: &mut [Vec<u8>],
    placements: &[(usize, u64, u64)],
    slots: &mut [Cell],
) -> Result<(), CpuError> {
    for instruction in body {
        match instruction {
            Instruction::GlobalInvocationIndex { result } => {
                write_slot(slots, *result, Cell::Index(invocation))?;
            }
            Instruction::ConstF32 { result, bits } => {
                write_slot(slots, *result, Cell::F32(*bits))?;
            }
            Instruction::ConstIndex { result, value } => {
                write_slot(slots, *result, Cell::Index(*value))?;
            }
            Instruction::F32Multiply { result, lhs, rhs } => {
                let value = mul_f32(
                    read_f32(slots, *lhs)?,
                    read_f32(slots, *rhs)?,
                    entry.numerics_nan,
                );
                write_slot(slots, *result, Cell::F32(value))?;
            }
            Instruction::F32Add { result, lhs, rhs } => {
                let value = add_f32(
                    read_f32(slots, *lhs)?,
                    read_f32(slots, *rhs)?,
                    entry.numerics_nan,
                );
                write_slot(slots, *result, Cell::F32(value))?;
            }
            Instruction::IndexAdd { result, lhs, rhs } => {
                let value = read_index(slots, *lhs)?
                    .checked_add(read_index(slots, *rhs)?)
                    .ok_or_else(|| CpuError::Execute("index add overflow".into()))?;
                write_slot(slots, *result, Cell::Index(value))?;
            }
            Instruction::IndexMultiply { result, lhs, rhs } => {
                let value = read_index(slots, *lhs)?
                    .checked_mul(read_index(slots, *rhs)?)
                    .ok_or_else(|| CpuError::Execute("index multiply overflow".into()))?;
                write_slot(slots, *result, Cell::Index(value))?;
            }
            Instruction::CanonicalizeF32Nan { result, source } => {
                let value = canonicalize(read_f32(slots, *source)?, entry.numerics_nan);
                write_slot(slots, *result, Cell::F32(value))?;
            }
            Instruction::IndexLessThan { result, lhs, rhs } => {
                write_slot(
                    slots,
                    *result,
                    Cell::Bool(read_index(slots, *lhs)? < read_index(slots, *rhs)?),
                )?;
            }
            Instruction::Load {
                result,
                buffer,
                offset,
            } => {
                let bits = access_element(
                    entry,
                    allocations,
                    placements,
                    *buffer,
                    read_index(slots, *offset)?,
                    None,
                )?;
                write_slot(slots, *result, Cell::F32(bits))?;
            }
            Instruction::Store {
                buffer,
                offset,
                value,
            } => {
                let bits = read_f32(slots, *value)?;
                access_element(
                    entry,
                    allocations,
                    placements,
                    *buffer,
                    read_index(slots, *offset)?,
                    Some(bits),
                )?;
            }
            Instruction::Predicated { predicate, body } => {
                if read_bool(slots, *predicate)? {
                    execute_block(body, entry, invocation, allocations, placements, slots)?;
                }
            }
        }
    }
    Ok(())
}

fn access_element(
    entry: &ScalarEntry,
    allocations: &mut [Vec<u8>],
    placements: &[(usize, u64, u64)],
    buffer: u32,
    element: u64,
    store: Option<u32>,
) -> Result<u32, CpuError> {
    let parameter = entry
        .buffers
        .get(buffer as usize)
        .ok_or_else(|| CpuError::Execute(format!("buffer {buffer} is undeclared")))?;
    if element >= parameter.element_count {
        return Err(CpuError::Execute(format!(
            "element {element} is outside buffer {buffer}'s declared {count}",
            count = parameter.element_count,
        )));
    }
    let (allocation, offset, bytes) = placements
        .get(buffer as usize)
        .copied()
        .ok_or_else(|| CpuError::Execute(format!("buffer {buffer} has no placement")))?;
    let at = element
        .checked_mul(F32_BYTES)
        .ok_or_else(|| CpuError::Execute("element byte offset overflow".into()))?;
    if at
        .checked_add(F32_BYTES)
        .ok_or_else(|| CpuError::Execute("element span overflow".into()))?
        > bytes
    {
        return Err(CpuError::Execute(format!(
            "element {element} is outside the routed range of {bytes} byte(s)"
        )));
    }
    let storage = allocations
        .get_mut(allocation)
        .ok_or_else(|| CpuError::Execute(format!("allocation {allocation} is unbound")))?;
    let start = usize::try_from(
        offset
            .checked_add(at)
            .ok_or_else(|| CpuError::Execute("placement offset overflow".into()))?,
    )
    .map_err(|_| CpuError::Execute("placement offset exceeds usize".into()))?;
    let end = start + 4;
    if end > storage.len() {
        return Err(CpuError::Execute(format!(
            "allocation {allocation} holds {} bytes and the access needs {end}",
            storage.len()
        )));
    }
    if let Some(bits) = store {
        storage[start..end].copy_from_slice(&bits.to_le_bytes());
        Ok(bits)
    } else {
        Ok(u32::from_le_bytes([
            storage[start],
            storage[start + 1],
            storage[start + 2],
            storage[start + 3],
        ]))
    }
}

/// One produced CPU payload and the artifact `assemble_plan_artifact` built for it.
pub struct ProducedCpu {
    /// The verified artifact the CPU family assembled on its own.
    pub artifact: VerifiedArtifactProgram,
    /// Encoded envelope bytes of that single-family artifact.
    pub bytes: Vec<u8>,
    /// Carried payload content reused when the portfolio is assembled.
    pub content: PayloadContent,
}

/// Translates one plan and assembles it through the neutral build seam.
pub fn assemble(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
) -> Result<ProducedCpu, CpuError> {
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    let image = emit(&kernels)?;
    let mut source = Vec::new();
    let mut mappings = Vec::with_capacity(kernels.len());
    for (kernel, entry) in kernels.iter().zip(&image.entries) {
        source.extend_from_slice(kernel.canonical_identity().as_bytes());
        mappings.push(PayloadEntryMapping {
            entry_key: BackendEntryKey::from_bytes(kernel.canonical_identity().as_bytes())
                .map_err(|error| CpuError::Assemble(error.to_string()))?,
            symbol: entry.symbol.clone(),
            transports: (0..u32::try_from(entry.buffers.len()).expect("two bindings")).collect(),
        });
    }
    mappings.sort_by(|left, right| left.entry_key.as_bytes().cmp(right.entry_key.as_bytes()));
    let metadata = PayloadMetadata {
        source_representation: RepresentationKey::new(SOURCE_REPRESENTATION_KEY)
            .map_err(|error| CpuError::Assemble(error.to_string()))?,
        source,
        provenance: PayloadProvenance {
            toolchain: TOOLCHAIN_KEY.to_owned(),
            target: "aarch64-apple-darwin".to_owned(),
            family: BACKEND_KEY.to_owned(),
            language: "tiler.cpu.scalar-image".to_owned(),
            platform: PayloadPlatform::Unversioned,
            components: vec![ToolComponent {
                role: "translator".to_owned(),
                version: "1".to_owned(),
            }],
            compile_flags: Vec::new(),
            link_flags: Vec::new(),
        },
        entries: mappings,
        obligations: Vec::new(),
    };
    let content = PayloadContent {
        metadata,
        code: encode(&image),
    };
    let artifact = assemble_plan_artifact(
        semantic,
        plan,
        // `Unclaimed`, which is every current build: a `Claimed` declaration
        // needs a compiler witness joined to per-payload receipts, and no
        // accepted provider can mint a receipt today. The artifact lands
        // admitting nothing and stays routable.
        PlanDeterminismDeclaration::Unclaimed,
        |builder, profile| {
            builder
                .push_carried_payload(
                    BackendKey::new(BACKEND_KEY).expect("a governed backend key"),
                    RepresentationKey::new(REPRESENTATION_KEY)
                        .expect("a governed representation key"),
                    PAYLOAD_SCHEMA,
                    profile,
                    ArtifactExecutionPolicy::NativeImage,
                    // No target-environment declaration; this spike registers
                    // no provider descriptor schema to name one under.
                    None,
                    content.clone(),
                )
                .map(|payload| vec![payload])
        },
        |_, stage: StageRef<'_>| {
            Ok(BackendEntryDeclaration {
                bindings: stage.accesses().map(|_| BindingKind::Buffer).collect(),
                zero_work_skips_dispatch: true,
                preconditions: Vec::new(),
            })
        },
    )
    .map_err(|error| CpuError::Assemble(error.to_string()))?;
    let bytes = artifact
        .encode()
        .map_err(|error| CpuError::Assemble(error.to_string()))?;
    Ok(ProducedCpu {
        artifact,
        bytes,
        content,
    })
}

/// Returns this backend's governed family key.
#[must_use]
pub fn backend() -> BackendKey {
    BackendKey::new(BACKEND_KEY).expect("a governed backend key")
}

/// Returns this backend's governed representation key.
#[must_use]
pub fn representation() -> RepresentationKey {
    RepresentationKey::new(REPRESENTATION_KEY).expect("a governed representation key")
}

/// Returns this backend's payload schema.
#[must_use]
pub const fn payload_schema() -> SchemaVersion {
    PAYLOAD_SCHEMA
}

/// A consumer-selected CPU adapter. Nothing registers it.
pub struct CpuAdapter {
    profile: TargetProfileRef,
    dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
    input: Vec<u8>,
    prepared: Vec<PreparedScalarEntry>,
    allocations: Vec<Vec<u8>>,
    placements: Vec<Vec<(usize, u64, u64)>>,
    readback: Option<(usize, u64, usize)>,
    /// Bits read back from the committed dispatch.
    pub result_bits: Vec<u32>,
}

/// One decoded scalar entry with the exact prepared-entry capacity this adapter enforces.
struct PreparedScalarEntry {
    entry: ScalarEntry,
    max_threads_per_workgroup: u64,
}

impl CpuAdapter {
    /// Builds an adapter over the caller's input element bits.
    #[must_use]
    pub fn new(
        profile: TargetProfileRef,
        dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
        input_bits: &[u32],
    ) -> Self {
        let mut input = Vec::with_capacity(input_bits.len() * 4);
        for bits in input_bits {
            input.extend_from_slice(&bits.to_le_bytes());
        }
        Self {
            profile,
            dtype_dispatch,
            input,
            prepared: Vec::new(),
            allocations: Vec::new(),
            placements: Vec::new(),
            readback: None,
            result_bits: Vec::new(),
        }
    }
}

impl RuntimeAdapter for CpuAdapter {
    type Refusal = CpuError;
    type Failure = CpuError;
    type Completion = Vec<u32>;

    /// Registers no target-environment descriptor schema.
    ///
    /// This spike's scalar CPU image is its own representation, and no accepted
    /// provider schema describes the host arithmetic conditions such a
    /// declaration would have to fix. There is no permissive default here on
    /// purpose: `Unsupported` filters every claimed `Plan` cell while leaving
    /// `Unclaimed` routes routable, which is the fail-closed answer for an
    /// adapter that cannot stand behind an exact provider schema.
    fn target_environment_support(&self) -> TargetEnvironmentSupport<'_> {
        TargetEnvironmentSupport::Unsupported
    }

    /// Never reached while no schema is registered; unavailable regardless.
    ///
    /// An observation is an assertion rather than an attestation, and this
    /// adapter has nothing to assert.
    fn observe_target_environment(
        &mut self,
        _context: &LiveExecutionContext,
    ) -> TargetEnvironmentObservation {
        TargetEnvironmentObservation::Unavailable {
            reason:
                "this spike's scalar CPU adapter registers no target-environment descriptor schema"
                    .to_owned(),
        }
    }

    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Self::Refusal> {
        Ok(ExecutionEnvironment {
            target_profile: self.profile.clone(),
            backend: backend(),
            representation: representation(),
            dtype_dispatch: self.dtype_dispatch.clone(),
        })
    }

    fn validate_payload(
        &mut self,
        _context: &LiveExecutionContext,
        entry: &RoutedEntry<'_>,
    ) -> Result<(), Self::Refusal> {
        let image = decode(entry.object())?;
        let prepared = image
            .entry_for(entry.entry_symbol())
            .cloned()
            .ok_or_else(|| CpuError::UnmappedSymbol(entry.entry_symbol().to_owned()))?;
        self.prepared.push(PreparedScalarEntry {
            entry: prepared,
            max_threads_per_workgroup: MAX_THREADS_PER_WORKGROUP,
        });
        Ok(())
    }

    fn observe_live_device(
        &mut self,
        _context: &LiveExecutionContext,
        _request: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        LiveDeviceObservation::Unrecognized
    }

    fn prepare_entries(
        &mut self,
        _context: &LiveExecutionContext,
        entries: &[RoutedEntry<'_>],
    ) -> Result<(), Self::Refusal> {
        if self.prepared.len() != entries.len() {
            return Err(CpuError::Adapter(
                "prepared entries disagree with the route".into(),
            ));
        }
        Ok(())
    }

    fn observe_prepared_entry(
        &mut self,
        _context: &LiveExecutionContext,
        request: TargetPropertyRequest<'_>,
    ) -> PreparedEntryObservation {
        let query = request.requirement().query();
        let provider = query.provider();
        if query.key().as_str() != MAX_THREADS_PER_WORKGROUP_PROPERTY
            || provider.namespace() != PREPARED_ENTRY_PROVIDER_NAMESPACE
            || provider.name() != PREPARED_ENTRY_PROVIDER_NAME
            || provider.revision() != PREPARED_ENTRY_PROVIDER_REVISION
        {
            return PreparedEntryObservation::Unrecognized;
        }
        self.prepared
            .get(request.entry())
            .map_or(PreparedEntryObservation::Unrecognized, |prepared| {
                PreparedEntryObservation::Quantity(prepared.max_threads_per_workgroup)
            })
    }

    fn plan_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        preflight: &Preflight<'_>,
    ) -> Result<(), Self::Refusal> {
        for (position, entry) in preflight.entries().iter().enumerate() {
            let prepared = self.prepared.get(position).ok_or_else(|| {
                CpuError::Adapter(format!("entry {position} has no prepared scalar image"))
            })?;
            if entry.launch().threads_per_workgroup() > prepared.max_threads_per_workgroup {
                return Err(CpuError::Adapter(format!(
                    "entry {position} launches {} threads per workgroup and this prepared scalar entry admits {}",
                    entry.launch().threads_per_workgroup(),
                    prepared.max_threads_per_workgroup,
                )));
            }
            for binding in entry.bindings() {
                if matches!(binding.binding().target(), BindingTarget::ProgramInput(_))
                    && u64::try_from(self.input.len()).expect("input length fits u64")
                        < binding.accessible_offset() + binding.accessible_bytes()
                {
                    return Err(CpuError::Adapter(format!(
                        "entry {position} slot {} needs more input bytes than were supplied",
                        binding.slot()
                    )));
                }
            }
        }
        Ok(())
    }

    fn allocate_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<(), Self::Failure> {
        self.allocations.clear();
        self.placements.clear();
        self.readback = None;
        for entry in routed.entries() {
            let mut placements = Vec::new();
            for binding in entry.bindings() {
                let needed = binding.accessible_offset() + binding.accessible_bytes();
                let allocation = match binding.binding().target() {
                    BindingTarget::ProgramInput(_) => {
                        let index = self.allocations.len();
                        self.allocations.push(self.input.clone());
                        index
                    }
                    BindingTarget::ProgramOutput(_) => {
                        let index = self.allocations.len();
                        self.allocations.push(vec![
                            0_u8;
                            usize::try_from(needed)
                                .expect("range fits usize")
                        ]);
                        self.readback = Some((
                            index,
                            binding.accessible_offset(),
                            usize::try_from(binding.accessible_bytes() / F32_BYTES)
                                .expect("element count fits usize"),
                        ));
                        index
                    }
                    BindingTarget::Internal => {
                        let index = self.allocations.len();
                        self.allocations.push(vec![
                            0_u8;
                            usize::try_from(needed)
                                .expect("range fits usize")
                        ]);
                        index
                    }
                };
                placements.push((
                    allocation,
                    binding.accessible_offset(),
                    binding.accessible_bytes(),
                ));
            }
            self.placements.push(placements);
        }
        Ok(())
    }

    fn dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<Self::Completion, Self::Failure> {
        for (position, entry) in routed.entries().iter().enumerate() {
            let prepared = &self.prepared[position].entry;
            let launch = entry.launch();
            if launch.grid_threads() == 0 && launch.zero_work_skips_dispatch() {
                continue;
            }
            let placements = self.placements[position].clone();
            for invocation in 0..launch.grid_threads() {
                execute_entry(prepared, invocation, &mut self.allocations, &placements)?;
            }
        }
        let Some((allocation, offset, count)) = self.readback else {
            return Err(CpuError::Execute(
                "the route published no program output".into(),
            ));
        };
        let storage = &self.allocations[allocation];
        let start = usize::try_from(offset).expect("offset fits usize");
        let mut bits = Vec::with_capacity(count);
        for index in 0..count {
            let at = start + index * 4;
            bits.push(u32::from_le_bytes([
                storage[at],
                storage[at + 1],
                storage[at + 2],
                storage[at + 3],
            ]));
        }
        self.result_bits.clone_from(&bits);
        Ok(bits)
    }
}

/// Binds the ABI facts a route evaluates its formulas against.
///
/// The *literal* axes only. An interface axis may name a `ShapeEnv` symbol,
/// whose value is the caller's bound buffer rather than anything the artifact
/// declares, so binding one here would state a fact this artifact does not
/// know. A symbolic axis is left unbound instead, which makes every expression
/// over it fail closed as an unbound input extent rather than silently
/// evaluating against an invented extent. Every fixture this spike packages
/// declares a wholly literal boundary, so the bound set is unchanged for them.
#[must_use]
pub fn bind_facts(program: &DecodedProgram) -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    for input in program.inputs() {
        binder
            .bind_declared_extents(input.key(), input.extents())
            .expect("the declared interface binds");
    }
    binder.build()
}

/// Routes one decoded artifact through the CPU adapter and compares the result.
pub fn route_and_compare(
    bytes: &[u8],
    expected_identity: &RecordedArtifactProgramIdentity,
    environment_profile: TargetProfileRef,
    dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
    reference: &[u32],
) -> Result<Vec<u32>, CpuError> {
    let mut program = DecodedProgram::decode(bytes, SOLE_DELIVERY)
        .map_err(|rejection| CpuError::Load(rejection.to_string()))?;
    let facts = bind_facts(&program);
    let mut adapter = CpuAdapter::new(
        environment_profile,
        dtype_dispatch,
        &crate::semantic::OPERANDS,
    );
    let bits = route_with_adapter(&mut program, &mut adapter, expected_identity, &facts).map_err(
        |failure| match failure {
            tiler_runtime::adapter::AdapterRouteFailure::Load(rejection) => {
                CpuError::Load(rejection.to_string())
            }
            other => CpuError::Adapter(format!("{other:?}")),
        },
    )?;
    if bits.len() != reference.len() {
        return Err(CpuError::Execute(format!(
            "result length {} disagrees with the reference {}",
            bits.len(),
            reference.len()
        )));
    }
    for (index, (actual, expected)) in bits.iter().zip(reference).enumerate() {
        if actual != expected {
            return Err(CpuError::Mismatch {
                index,
                actual: *actual,
                expected: *expected,
            });
        }
    }
    Ok(bits)
}

/// Routes one deliberately perturbed artifact and returns the loader's refusal.
pub fn route_refusal(
    bytes: &[u8],
    expected_identity: &RecordedArtifactProgramIdentity,
    environment_profile: TargetProfileRef,
    dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
) -> Result<LoadRejection, CpuError> {
    let mut program = DecodedProgram::decode(bytes, SOLE_DELIVERY)
        .map_err(|rejection| CpuError::Load(rejection.to_string()))?;
    let facts = bind_facts(&program);
    let mut adapter = CpuAdapter::new(
        environment_profile,
        dtype_dispatch,
        &crate::semantic::OPERANDS,
    );
    match route_with_adapter(&mut program, &mut adapter, expected_identity, &facts) {
        Err(tiler_runtime::adapter::AdapterRouteFailure::Load(rejection)) => Ok(rejection),
        Err(other) => Err(CpuError::Adapter(format!(
            "the perturbed request failed outside the loader: {other:?}"
        ))),
        Ok(_) => Err(CpuError::Load(
            "the perturbed prepared-entry request routed successfully".into(),
        )),
    }
}

/// Preflights one artifact under a stated environment and reports the refusal.
pub fn preflight_refusal(
    bytes: &[u8],
    expected: &RecordedArtifactProgramIdentity,
    environment: &ExecutionEnvironment,
) -> Result<LoadRejection, CpuError> {
    let mut program = DecodedProgram::decode(bytes, SOLE_DELIVERY)
        .map_err(|rejection| CpuError::Load(rejection.to_string()))?;
    let facts = bind_facts(&program);
    match program.preflight(environment, expected, &facts) {
        Err(rejection) => Ok(rejection),
        Ok(_) => Err(CpuError::Load(
            "preflight succeeded where a refusal was required".into(),
        )),
    }
}
