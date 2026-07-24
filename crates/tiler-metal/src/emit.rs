//! Deterministic translation of verified structured kernels into MSL text.
//!
//! # Mechanical translation, not pattern recognition
//!
//! This module reads exactly one vocabulary: the structured kernel IR's own
//! [`OperationView`](tiler_ir::kernel::OperationView). There is no branch here
//! that asks whether a kernel is a reduction, whether a predicate is a tail
//! guard, whether an index expression realizes a particular access relation, or
//! which semantic operation a multiply-then-add came from. Every such fact is
//! already a signature field or an explicit operation, so emission is a
//! per-operation rewrite and nothing else. If this file ever needs to recognize
//! a shape, either it is reconstructing meaning the IR already states or the IR
//! has a gap; both are defects, not lowering techniques.
//!
//! # Determinism
//!
//! The emitted bytes are a pure function of the verified kernels and the target
//! facts:
//!
//! - Entry points are ordered by canonical kernel identity bytes and
//!   deduplicated by identity, so caller order cannot change the output.
//! - Symbols are content-derived: an entry point is named by a bounded digest of
//!   its canonical identity, and a helper by the exact bit pattern it realizes.
//!   Emission proves entry-point symbols are pairwise distinct before returning
//!   rather than trusting the digest. Local names come from a per-kernel counter
//!   advanced by a fixed pre-order walk of the structured body.
//! - Only ordered containers are used. No hash map, no global counter, no
//!   address, no clock, and no environment value reaches the output.
//!
//! # Numerics
//!
//! `f32` immediates are emitted as exact bit patterns through `as_type`, never
//! as decimal text, so no rounding can be introduced by the emitter or by the
//! Metal compiler's literal parsing. Every arithmetic operation is emitted as
//! its own statement, so no contraction can form across two structured
//! operations. NaN canonicalization is emitted as an explicit call to a helper
//! realizing the kernel's own canonical pattern; it is never left to a target
//! default. The compiler flags this source depends on are reported as
//! [`MetalNumericalRequirement`](crate::record::MetalNumericalRequirement)s
//! rather than assumed.
//!
//! Every public item here is a reviewed *draft* boundary (ADR 0074 §7).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use tiler_ir::kernel::{
    AddressSpace, BarrierOrdering, BarrierSpec, BinaryOp, BlockRef, BufferAccess, Builtin,
    CompareOp, ConvertOp, ExecutionScope, KernelConstant, KernelType, MemoryScope, OperationRef,
    OperationView, SerialLoopRef, VerifiedBufferId, VerifiedKernel, VerifiedValueId,
};
use tiler_ir::schedule::{NumericalPermission, NumericalRealization, SubnormalMode};

use crate::diagnostic::{BarrierRejection, MetalEmitError, MetalOperationFamily};
use crate::record::{
    MetalBufferBinding, MetalEntryPoint, MetalNumericalRequirement, MetalTranslationUnit,
};
use crate::target::MetalTargetFacts;

/// One level of emitted indentation.
const INDENT: &str = "    ";

/// Prefix of every emitted entry-point symbol.
const ENTRY_PREFIX: &str = "tiler_kernel_";

/// Prefix of every emitted NaN-canonicalization helper symbol.
const CANONICALIZE_PREFIX: &str = "tiler_canonicalize_nan_f32_";

/// Appends formatted text to an emitted buffer.
///
/// Formatting into a `String` is infallible, so the discarded result cannot
/// hide a failure; emission signatures stay reserved for translation
/// rejections.
macro_rules! emit {
    ($buffer:expr, $($argument:tt)*) => {{
        let _ = write!($buffer, $($argument)*);
    }};
}

/// Emits one deterministic Metal translation unit for a portfolio of kernels.
///
/// The kernels are deduplicated by canonical identity and emitted in ascending
/// canonical-identity order, so the result is a pure function of the *set* of
/// kernels and the target facts. Helpers required by more than one entry point
/// are emitted once.
///
/// # Errors
///
/// Returns a [`MetalEmitError`] naming the rejected entity when a governed
/// structured construct has no realization in the selected Metal profile, when
/// the signature exceeds the target's binding capacity, when the kernel's
/// canonical NaN pattern is not a NaN encoding, or when two structurally
/// distinct kernels derive the same entry-point symbol. Emission never returns
/// partial or best-effort source.
pub fn emit_translation_unit(
    kernels: &[&VerifiedKernel],
    target: &MetalTargetFacts,
) -> Result<MetalTranslationUnit, MetalEmitError> {
    let ordered = order_kernels(kernels);
    let mut helpers = BTreeSet::new();
    let mut numerical = BTreeSet::new();
    let mut symbols: BTreeMap<String, &[u8]> = BTreeMap::new();
    let mut entry_points = Vec::with_capacity(ordered.len());
    let mut bodies = Vec::with_capacity(ordered.len());

    for kernel in ordered {
        let emitted = emit_entry_point(kernel, target, &mut helpers, &mut numerical)?;
        reserve_symbol(
            &mut symbols,
            emitted.entry.symbol(),
            kernel.canonical_identity().as_bytes(),
        )?;
        entry_points.push(emitted.entry);
        bodies.push(emitted.text);
    }

    let source = assemble(target, &helpers, &bodies);
    Ok(MetalTranslationUnit::new(
        source,
        entry_points,
        numerical.into_iter().collect(),
    ))
}

/// Orders a portfolio by canonical identity and removes exact duplicates.
fn order_kernels<'a>(kernels: &[&'a VerifiedKernel]) -> Vec<&'a VerifiedKernel> {
    let mut ordered: Vec<&VerifiedKernel> = kernels.to_vec();
    ordered.sort_by(|left, right| {
        left.canonical_identity()
            .as_bytes()
            .cmp(right.canonical_identity().as_bytes())
    });
    ordered.dedup_by(|left, right| left.canonical_identity() == right.canonical_identity());
    ordered
}

/// Reserves one entry-point symbol for the exact identity that derived it.
///
/// The symbol is a bounded digest, so distinctness is proven rather than
/// assumed: a second, structurally different identity claiming the same symbol
/// is a hard rejection, not a silently shared entry point.
pub(crate) fn reserve_symbol<'a>(
    symbols: &mut BTreeMap<String, &'a [u8]>,
    symbol: &str,
    identity: &'a [u8],
) -> Result<(), MetalEmitError> {
    match symbols.get(symbol) {
        Some(reserved) if *reserved != identity => Err(MetalEmitError::SymbolCollision {
            symbol: symbol.to_owned(),
        }),
        Some(_) => Ok(()),
        None => {
            symbols.insert(symbol.to_owned(), identity);
            Ok(())
        }
    }
}

/// Assembles the provenance header, the required helpers, and the entry points.
fn assemble(target: &MetalTargetFacts, helpers: &BTreeSet<u32>, bodies: &[String]) -> String {
    let mut source = String::new();
    source.push_str("// Generated by tiler-metal from verified structured kernel IR.\n");
    source.push_str("// Deterministic output: do not edit.\n");
    source.push_str("//\n");
    emit!(
        source,
        "// Metal Shading Language: {}\n",
        target.language.std_token()
    );
    emit!(
        source,
        "// Artifact family: {} (deployment minimum {})\n",
        target.platform,
        target.deployment_minimum
    );
    emit!(
        source,
        "// Launch index: [[{}]] declared as {}\n",
        target.launch_index.attribute(),
        target.launch_index.declared_type()
    );
    emit!(
        source,
        "// Launch precondition: no invocation index may exceed {}.\n",
        target.launch_index.maximum_index()
    );
    source.push_str("//\n");
    source.push_str("// Every f32 immediate is emitted as its exact bit pattern, and every\n");
    source.push_str("// arithmetic operation is emitted as one statement, so neither literal\n");
    source.push_str("// parsing nor contraction across operations can change a result.\n");
    source.push('\n');
    source.push_str("#include <metal_stdlib>\n");
    source.push_str("using namespace metal;\n");

    for bits in helpers {
        source.push('\n');
        source.push_str(&canonicalize_helper(*bits));
    }
    for body in bodies {
        source.push('\n');
        source.push_str(body);
    }
    source
}

/// Returns the NaN-canonicalization helper for one exact canonical pattern.
///
/// The helper is correct only under a math mode that does not assume the
/// absence of NaNs; emission records that as a
/// [`MetalNumericalRequirement::SafeMathMode`] rather than leaving it implicit.
fn canonicalize_helper(bits: u32) -> String {
    let symbol = canonicalize_symbol(bits);
    format!(
        "// Replaces an arithmetic NaN with the canonical pattern {bits:#010x}.\n\
         static inline float {symbol}(float value) {{\n\
         {INDENT}return isnan(value) ? as_type<float>({bits:#010x}u) : value;\n\
         }}\n"
    )
}

/// Returns the deterministic helper symbol for one canonical NaN pattern.
fn canonicalize_symbol(bits: u32) -> String {
    format!("{CANONICALIZE_PREFIX}{bits:08x}")
}

/// Returns whether a bit pattern encodes an IEEE-754 binary32 NaN.
///
/// Quietness is the numerical contract's rule, not this backend's; emission
/// only refuses a "canonical NaN" that is not a NaN at all.
pub(crate) const fn is_f32_nan(bits: u32) -> bool {
    bits & 0x7f80_0000 == 0x7f80_0000 && bits & 0x007f_ffff != 0
}

/// Returns the bounded presentation digest of canonical identity bytes.
///
/// This is FNV-1a over the canonical bytes, matching the digest labels the
/// compiler already uses for explain output. It names an entry point; it never
/// decides equality.
fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// One emitted entry point: its record and its complete source text.
struct EmittedEntryPoint {
    entry: MetalEntryPoint,
    text: String,
}

/// Emits one entry point, collecting the helpers and requirements it needs.
fn emit_entry_point(
    kernel: &VerifiedKernel,
    target: &MetalTargetFacts,
    helpers: &mut BTreeSet<u32>,
    numerical: &mut BTreeSet<MetalNumericalRequirement>,
) -> Result<EmittedEntryPoint, MetalEmitError> {
    let declared = kernel.buffers().len();
    if u64::try_from(declared).unwrap_or(u64::MAX) > u64::from(target.buffer_binding_limit) {
        return Err(MetalEmitError::BufferBindingLimit {
            required: declared,
            limit: target.buffer_binding_limit,
        });
    }
    let symbol = format!(
        "{ENTRY_PREFIX}{:016x}",
        digest(kernel.canonical_identity().as_bytes())
    );
    numerical.extend(realization_requirements(kernel.numerical()));

    let mut emitter = KernelEmitter {
        kernel,
        target,
        helpers,
        numerical,
        names: BTreeMap::new(),
        next: 0,
        buffers: BTreeMap::new(),
        bindings: Vec::new(),
        out: String::new(),
        indent: 1,
    };
    emitter.emit_block(kernel.body())?;
    let KernelEmitter {
        bindings,
        out: body,
        ..
    } = emitter;
    // The binding table is derived from body use, so a declared parameter the
    // body never touches has no argument-table position to occupy. Emitting a
    // signature that silently drops it would change the ABI.
    if bindings.len() != declared {
        return Err(MetalEmitError::MalformedKernel {
            rule: "unreferenced-buffer-parameter",
        });
    }

    let mut text = String::new();
    emit!(text, "// Entry point {symbol}\n");
    emit!(
        text,
        "//   kernel identity digest: {:016x}\n",
        digest(kernel.canonical_identity().as_bytes())
    );
    emit!(
        text,
        "//   scheduled region identity digest: {:016x}\n",
        digest(kernel.scheduled_region_identity().as_bytes())
    );
    emit!(
        text,
        "//   numerical profile: {}\n",
        kernel.numerical().profile_key
    );
    for binding in &bindings {
        let parameter = binding.parameter();
        emit!(
            text,
            "//   buffer({}): {:?} tensor, {:?}, {:?} space, {:?} access, {} element(s)\n",
            binding.index(),
            parameter.tensor,
            parameter.element_type,
            parameter.address_space,
            parameter.access,
            parameter.element_count,
        );
    }

    let mut parameters = Vec::with_capacity(bindings.len().saturating_add(1));
    for binding in &bindings {
        parameters.push(parameter_declaration(binding)?);
    }
    for builtin in kernel.admitted_builtins() {
        parameters.push(builtin_declaration(*builtin, target)?);
    }
    emit!(text, "kernel void {symbol}(");
    if parameters.is_empty() {
        text.push_str(") {\n");
    } else {
        text.push('\n');
        for (position, parameter) in parameters.iter().enumerate() {
            let suffix = if position + 1 == parameters.len() {
                ") {"
            } else {
                ","
            };
            emit!(text, "{INDENT}{INDENT}{parameter}{suffix}\n");
        }
    }
    text.push_str(&body);
    text.push_str("}\n");

    Ok(EmittedEntryPoint {
        entry: MetalEntryPoint::new(symbol, kernel.canonical_identity().clone(), bindings),
        text,
    })
}

/// Returns the MSL declaration of one buffer parameter.
fn parameter_declaration(binding: &MetalBufferBinding) -> Result<String, MetalEmitError> {
    let parameter = binding.parameter();
    let element = msl_type(parameter.element_type)?;
    let space = address_space_declaration(parameter.address_space, parameter.access)?;
    let index = binding.index();
    Ok(format!("{space} {element} *b{index} [[buffer({index})]]"))
}

/// Returns the MSL address-space and mutability qualifiers of one parameter.
///
/// `device` and `constant` are the argument-table parameter spaces. Workgroup
/// storage binds through the separate `[[threadgroup(N)]]` namespace, which a
/// structured buffer parameter cannot name, and invocation-private storage is
/// not a parameter space at all.
pub(crate) fn address_space_declaration(
    space: AddressSpace,
    access: BufferAccess,
) -> Result<&'static str, MetalEmitError> {
    match space {
        AddressSpace::Device => match access {
            BufferAccess::Read => Ok("device const"),
            BufferAccess::Write => Ok("device"),
            _ => Err(MetalEmitError::UnsupportedBufferAccess { space, access }),
        },
        AddressSpace::Constant => match access {
            BufferAccess::Read => Ok("constant"),
            // The constant space is read-only, so a writable parameter is
            // rejected instead of being silently promoted to a device binding.
            _ => Err(MetalEmitError::UnsupportedBufferAccess { space, access }),
        },
        // Workgroup storage is a `[[threadgroup(N)]]` binding a buffer
        // parameter cannot name, invocation-private storage is not a parameter
        // space, and an unrecognized space has no known realization.
        _ => Err(MetalEmitError::UnsupportedAddressSpace { space }),
    }
}

/// Returns the MSL declaration of one admitted launch builtin parameter.
fn builtin_declaration(
    builtin: Builtin,
    target: &MetalTargetFacts,
) -> Result<String, MetalEmitError> {
    let name = builtin_parameter(builtin)?;
    Ok(format!(
        "{} {name} [[{}]]",
        target.launch_index.declared_type(),
        target.launch_index.attribute()
    ))
}

/// Returns the deterministic parameter name of one governed launch builtin.
fn builtin_parameter(builtin: Builtin) -> Result<&'static str, MetalEmitError> {
    match builtin {
        Builtin::GlobalInvocationIndex => Ok("tiler_global_invocation_index"),
        _ => Err(MetalEmitError::UnsupportedOperation {
            family: MetalOperationFamily::Builtin,
        }),
    }
}

/// Returns the MSL spelling of one governed structured type.
pub(crate) fn msl_type(value_type: KernelType) -> Result<&'static str, MetalEmitError> {
    match value_type {
        KernelType::Bool => Ok("bool"),
        KernelType::Index => Ok("ulong"),
        KernelType::F32 => Ok("float"),
        _ => Err(MetalEmitError::UnsupportedType { value_type }),
    }
}

/// Returns the compiler requirements one numerical realization imposes.
///
/// Each vocabulary matched here is *not* `#[non_exhaustive]`, so widening the
/// numerical contract is a compile error at this site rather than a silently
/// dropped requirement.
fn realization_requirements(
    realization: NumericalRealization,
) -> BTreeSet<MetalNumericalRequirement> {
    let mut requirements = BTreeSet::new();
    match realization.contraction {
        NumericalPermission::Forbidden => {
            requirements.insert(MetalNumericalRequirement::NoFloatingPointContraction);
        }
    }
    match realization.reassociation {
        NumericalPermission::Forbidden => {
            requirements.insert(MetalNumericalRequirement::SafeMathMode);
        }
    }
    for mode in [realization.input_subnormals, realization.result_subnormals] {
        match mode {
            SubnormalMode::Preserve => {
                requirements.insert(MetalNumericalRequirement::SafeMathMode);
            }
        }
    }
    requirements
}

/// Per-kernel emission state.
struct KernelEmitter<'a> {
    kernel: &'a VerifiedKernel,
    target: &'a MetalTargetFacts,
    helpers: &'a mut BTreeSet<u32>,
    numerical: &'a mut BTreeSet<MetalNumericalRequirement>,
    names: BTreeMap<VerifiedValueId, String>,
    next: u32,
    buffers: BTreeMap<VerifiedBufferId, u32>,
    bindings: Vec<MetalBufferBinding>,
    out: String,
    indent: usize,
}

impl KernelEmitter<'_> {
    /// Appends one indented line of emitted source.
    fn line(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.out.push_str(INDENT);
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    /// Binds a fresh deterministic local name to one structured value.
    fn bind(&mut self, value: VerifiedValueId) -> Result<String, MetalEmitError> {
        let name = format!("v{}", self.next);
        self.next = self.next.saturating_add(1);
        if self.names.insert(value, name.clone()).is_some() {
            return Err(MetalEmitError::MalformedKernel {
                rule: "duplicate-value-definition",
            });
        }
        Ok(name)
    }

    /// Resolves the emitted name of one already-defined structured value.
    fn name(&self, value: VerifiedValueId) -> Result<&str, MetalEmitError> {
        self.names
            .get(&value)
            .map(String::as_str)
            .ok_or(MetalEmitError::UnresolvedValue)
    }

    /// Returns the MSL spelling of one structured value's resolved type.
    fn value_type(&self, value: VerifiedValueId) -> Result<&'static str, MetalEmitError> {
        msl_type(self.kernel.value_type(value)?)
    }

    /// Returns the argument-table index bound to one verified buffer handle.
    ///
    /// The structured kernel IR does not expose the signature ordinal of a
    /// [`VerifiedBufferId`], so this backend assigns argument-table indices in
    /// first-use order and reports exactly the table it emitted. Every emitted
    /// subscript and the reported binding table therefore agree by
    /// construction, and a declared parameter the body never references is
    /// rejected in [`emit_entry_point`] rather than dropped.
    fn buffer_binding(&mut self, buffer: VerifiedBufferId) -> Result<u32, MetalEmitError> {
        if let Some(index) = self.buffers.get(&buffer) {
            return Ok(*index);
        }
        let parameter = self.kernel.buffer(buffer)?;
        let assigned = self.bindings.len();
        let limit = self.target.buffer_binding_limit;
        let index = u32::try_from(assigned).map_err(|_| MetalEmitError::BufferBindingLimit {
            required: assigned.saturating_add(1),
            limit,
        })?;
        if index >= limit {
            return Err(MetalEmitError::BufferBindingLimit {
                required: assigned.saturating_add(1),
                limit,
            });
        }
        self.buffers.insert(buffer, index);
        self.bindings
            .push(MetalBufferBinding::new(index, parameter));
        Ok(index)
    }

    /// Emits every operation of one structured block in order.
    fn emit_block(&mut self, block: BlockRef<'_>) -> Result<(), MetalEmitError> {
        for operation in block.operations() {
            self.emit_operation(operation)?;
        }
        Ok(())
    }

    /// Emits one structured operation as one or more MSL statements.
    fn emit_operation(&mut self, operation: OperationRef<'_>) -> Result<(), MetalEmitError> {
        let results: Vec<VerifiedValueId> = operation.results().collect();
        match operation.view() {
            OperationView::Builtin { builtin } => self.emit_builtin(builtin, &results),
            OperationView::Constant { value } => self.emit_constant(value, &results),
            OperationView::Binary { op, lhs, rhs } => self.emit_binary(op, lhs, rhs, &results),
            OperationView::Compare { op, lhs, rhs } => self.emit_compare(op, lhs, rhs, &results),
            OperationView::Convert { op, source } => self.emit_convert(op, source, &results),
            OperationView::Load {
                buffer,
                offset,
                bounds,
            } => {
                let [result] = results.as_slice() else {
                    return Err(arity("load-result"));
                };
                let index = self.buffer_binding(buffer)?;
                let offset = self.name(offset)?.to_owned();
                let witness = bounds.get();
                let value_type = self.value_type(*result)?;
                let name = self.bind(*result)?;
                self.line(&format!(
                    "{value_type} {name} = b{index}[{offset}];  // bounds witness {witness}"
                ));
                Ok(())
            }
            OperationView::Store {
                buffer,
                offset,
                value,
                bounds,
                ownership,
            } => {
                if !results.is_empty() {
                    return Err(arity("store-result"));
                }
                let index = self.buffer_binding(buffer)?;
                let offset = self.name(offset)?.to_owned();
                let value = self.name(value)?.to_owned();
                let bounds = bounds.get();
                let ownership = ownership.get();
                self.line(&format!(
                    "b{index}[{offset}] = {value};  \
                     // bounds witness {bounds}, ownership witness {ownership}"
                ));
                Ok(())
            }
            OperationView::Predicated { predicate, body } => {
                if !results.is_empty() {
                    return Err(arity("predicated-result"));
                }
                let predicate = self.name(predicate)?.to_owned();
                self.line(&format!("if ({predicate}) {{"));
                self.indent = self.indent.saturating_add(1);
                self.emit_block(body)?;
                self.indent = self.indent.saturating_sub(1);
                self.line("}");
                Ok(())
            }
            OperationView::SerialLoop(loop_ref) => self.emit_serial_loop(loop_ref, &results),
            OperationView::Barrier { spec } => {
                if !results.is_empty() {
                    return Err(arity("barrier-result"));
                }
                let call = barrier_call(spec)?;
                self.line(&call);
                Ok(())
            }
            _ => Err(MetalEmitError::UnrecognizedOperation),
        }
    }

    /// Emits one admitted launch-builtin read.
    fn emit_builtin(
        &mut self,
        builtin: Builtin,
        results: &[VerifiedValueId],
    ) -> Result<(), MetalEmitError> {
        let [result] = results else {
            return Err(arity("builtin-result"));
        };
        let parameter = builtin_parameter(builtin)?;
        let declared = self.target.launch_index.declared_type();
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        // The launch attribute's declared type is fixed by the language, so a
        // narrower delivery is widened exactly rather than reinterpreted.
        if declared == value_type {
            self.line(&format!("{value_type} {name} = {parameter};"));
        } else {
            self.line(&format!(
                "{value_type} {name} = {value_type}({parameter});  // widened from {declared}"
            ));
        }
        Ok(())
    }

    /// Emits one typed immediate constant.
    fn emit_constant(
        &mut self,
        value: KernelConstant,
        results: &[VerifiedValueId],
    ) -> Result<(), MetalEmitError> {
        let [result] = results else {
            return Err(arity("constant-result"));
        };
        let literal = match value {
            KernelConstant::Bool(true) => "true".to_owned(),
            KernelConstant::Bool(false) => "false".to_owned(),
            KernelConstant::Index(index) => format!("{index}ul"),
            // The exact bit pattern is emitted, never a decimal rendering, so
            // no literal parsing can round the declared value.
            KernelConstant::F32Bits(bits) => format!("as_type<float>({bits:#010x}u)"),
            _ => {
                return Err(MetalEmitError::UnsupportedOperation {
                    family: MetalOperationFamily::Constant,
                });
            }
        };
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        self.line(&format!("{value_type} {name} = {literal};"));
        Ok(())
    }

    /// Emits one pure binary operation as its own statement.
    fn emit_binary(
        &mut self,
        op: BinaryOp,
        lhs: VerifiedValueId,
        rhs: VerifiedValueId,
        results: &[VerifiedValueId],
    ) -> Result<(), MetalEmitError> {
        let [result] = results else {
            return Err(arity("binary-result"));
        };
        let operator = match op {
            BinaryOp::IndexAdd | BinaryOp::F32Add => "+",
            BinaryOp::IndexMultiply | BinaryOp::F32Multiply => "*",
            BinaryOp::IndexDivide => "/",
            BinaryOp::IndexModulo => "%",
            _ => {
                return Err(MetalEmitError::UnsupportedOperation {
                    family: MetalOperationFamily::Binary,
                });
            }
        };
        // Defence in depth on an invariant the kernel builder already proves: a
        // zero divisor would be emitted as undefined behaviour on device.
        if op.requires_constant_divisor() {
            let divisor = self
                .kernel
                .value_constant(rhs)?
                .and_then(KernelConstant::as_index);
            if divisor.is_none_or(|value| value == 0) {
                return Err(MetalEmitError::MalformedKernel {
                    rule: "non-positive-divisor",
                });
            }
        }
        let lhs = self.name(lhs)?.to_owned();
        let rhs = self.name(rhs)?.to_owned();
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        self.line(&format!("{value_type} {name} = {lhs} {operator} {rhs};"));
        Ok(())
    }

    /// Emits one predicate-producing comparison.
    fn emit_compare(
        &mut self,
        op: CompareOp,
        lhs: VerifiedValueId,
        rhs: VerifiedValueId,
        results: &[VerifiedValueId],
    ) -> Result<(), MetalEmitError> {
        let [result] = results else {
            return Err(arity("compare-result"));
        };
        let operator = match op {
            CompareOp::IndexLessThan => "<",
            _ => {
                return Err(MetalEmitError::UnsupportedOperation {
                    family: MetalOperationFamily::Compare,
                });
            }
        };
        let lhs = self.name(lhs)?.to_owned();
        let rhs = self.name(rhs)?.to_owned();
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        self.line(&format!("{value_type} {name} = {lhs} {operator} {rhs};"));
        Ok(())
    }

    /// Emits one named typed conversion.
    fn emit_convert(
        &mut self,
        op: ConvertOp,
        source: VerifiedValueId,
        results: &[VerifiedValueId],
    ) -> Result<(), MetalEmitError> {
        let [result] = results else {
            return Err(arity("convert-result"));
        };
        let call = match op {
            ConvertOp::CanonicalizeF32Nan => {
                // The conversion deliberately does not carry a second copy of
                // the pattern: it is the kernel's own canonical arithmetic NaN.
                let bits = self.kernel.numerical().canonical_arithmetic_nan_bits;
                if !is_f32_nan(bits) {
                    return Err(MetalEmitError::InvalidCanonicalNan { bits });
                }
                self.helpers.insert(bits);
                self.numerical
                    .insert(MetalNumericalRequirement::SafeMathMode);
                canonicalize_symbol(bits)
            }
            _ => {
                return Err(MetalEmitError::UnsupportedOperation {
                    family: MetalOperationFamily::Convert,
                });
            }
        };
        let source = self.name(source)?.to_owned();
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        self.line(&format!("{value_type} {name} = {call}({source});"));
        Ok(())
    }

    /// Emits one bounded structured loop carrying typed accumulator state.
    ///
    /// The induction variable and each accumulator become one mutable local, so
    /// the loop-carried dependence is exactly the one the IR states. Loop
    /// results are copied into their own locals so every structured value keeps
    /// a distinct name.
    fn emit_serial_loop(
        &mut self,
        loop_ref: SerialLoopRef<'_>,
        results: &[VerifiedValueId],
    ) -> Result<(), MetalEmitError> {
        let induction = loop_ref
            .induction()
            .ok_or(MetalEmitError::MalformedKernel {
                rule: "loop-induction-missing",
            })?;
        let accumulators: Vec<VerifiedValueId> = loop_ref.accumulators().collect();
        let initial: Vec<VerifiedValueId> = loop_ref.initial().collect();
        let yields: Vec<VerifiedValueId> = loop_ref.yields().collect();
        if accumulators.len() != initial.len()
            || accumulators.len() != yields.len()
            || accumulators.len() != results.len()
        {
            return Err(MetalEmitError::MalformedKernel {
                rule: "loop-accumulator-arity",
            });
        }

        let (start, end) = (loop_ref.start(), loop_ref.end());
        self.line(&format!("// serial loop over [{start}, {end})"));
        let induction_type = self.value_type(induction)?;
        let induction_name = self.bind(induction)?;
        self.line(&format!("{induction_type} {induction_name} = {start}ul;"));
        let mut accumulator_names = Vec::with_capacity(accumulators.len());
        for (accumulator, seed) in accumulators.iter().zip(&initial) {
            let seed = self.name(*seed)?.to_owned();
            let value_type = self.value_type(*accumulator)?;
            let name = self.bind(*accumulator)?;
            self.line(&format!("{value_type} {name} = {seed};"));
            accumulator_names.push(name);
        }

        self.line(&format!(
            "for (; {induction_name} < {end}ul; {induction_name} = {induction_name} + 1ul) {{"
        ));
        self.indent = self.indent.saturating_add(1);
        self.emit_block(loop_ref.body())?;
        for (name, yielded) in accumulator_names.iter().zip(&yields) {
            let name = name.clone();
            let yielded = self.name(*yielded)?.to_owned();
            self.line(&format!("{name} = {yielded};"));
        }
        self.indent = self.indent.saturating_sub(1);
        self.line("}");

        for (result, accumulator) in results.iter().zip(&accumulator_names) {
            let accumulator = accumulator.clone();
            let value_type = self.value_type(*result)?;
            let name = self.bind(*result)?;
            self.line(&format!("{value_type} {name} = {accumulator};"));
        }
        Ok(())
    }
}

/// Returns the MSL barrier statement realizing one governed specification.
pub(crate) fn barrier_call(spec: &BarrierSpec) -> Result<String, MetalEmitError> {
    let call = match spec.execution_scope {
        ExecutionScope::Workgroup => "threadgroup_barrier",
        ExecutionScope::Subgroup => "simdgroup_barrier",
        _ => {
            return Err(MetalEmitError::UnsupportedBarrier {
                reason: BarrierRejection::ExecutionScope {
                    scope: spec.execution_scope,
                },
            });
        }
    };
    // Metal couples visibility to the barrier builtin: `threadgroup_barrier`
    // establishes workgroup visibility and `simdgroup_barrier` establishes
    // SIMD-group visibility. No in-kernel barrier establishes device-wide
    // visibility, and the governed memory scopes cannot name SIMD-group
    // visibility at all, so a SIMD-group barrier has no admissible scope here.
    match (spec.execution_scope, spec.memory_scope) {
        (ExecutionScope::Workgroup, MemoryScope::Workgroup) => {}
        _ => {
            return Err(MetalEmitError::UnsupportedBarrier {
                reason: BarrierRejection::MemoryVisibility {
                    execution: spec.execution_scope,
                    memory: spec.memory_scope,
                },
            });
        }
    }
    match spec.ordering {
        // A Metal barrier is a full acquire-release fence over the flagged
        // address spaces; no weaker or stronger ordering is expressible.
        BarrierOrdering::AcquireRelease => {}
        _ => {
            return Err(MetalEmitError::UnsupportedBarrier {
                reason: BarrierRejection::Ordering {
                    ordering: spec.ordering,
                },
            });
        }
    }

    // Ordering and deduplicating the fenced spaces makes the emitted flag
    // expression independent of the order the specification listed them in.
    let spaces: BTreeSet<AddressSpace> = spec.fenced_spaces.iter().copied().collect();
    let mut flags = Vec::with_capacity(spaces.len());
    for space in spaces {
        flags.push(fence_flag(space)?);
    }
    let joined = flags.join(" | ");
    let flags = if joined.is_empty() {
        "mem_flags::mem_none"
    } else {
        joined.as_str()
    };
    Ok(format!("{call}({flags});"))
}

/// Returns the Metal memory-fence flag for one governed address space.
fn fence_flag(space: AddressSpace) -> Result<&'static str, MetalEmitError> {
    match space {
        AddressSpace::Device => Ok("mem_flags::mem_device"),
        AddressSpace::Workgroup => Ok("mem_flags::mem_threadgroup"),
        // Constant memory is read-only for the dispatch and invocation-private
        // memory is visible to one invocation, so neither has a fence flag.
        // Dropping them silently would lose the specification.
        _ => Err(MetalEmitError::UnsupportedBarrier {
            reason: BarrierRejection::FencedSpace { space },
        }),
    }
}

/// Returns the malformed-kernel rejection for one violated arity rule.
const fn arity(rule: &'static str) -> MetalEmitError {
    MetalEmitError::MalformedKernel { rule }
}
