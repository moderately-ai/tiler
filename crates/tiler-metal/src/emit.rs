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
//! The emitted bytes are a pure function of the verified kernels, the target facts, and the selected emission realization:
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
//! The dividing question for every numerical obligation is whether the emitted
//! *operations* carry it or whether it depends on a compiler selection. The two
//! are kept apart deliberately, because a flag can be flipped by a caller and
//! an operation cannot.
//!
//! Carried by the emitted operations, under every math mode:
//!
//! - Floating-point immediates are emitted as exact bit patterns through
//!   `as_type`, never as decimal text, so no rounding can be introduced by the
//!   emitter or by the Metal compiler's literal parsing. `f32` reinterprets a
//!   `uint` and `bf16` a `ushort`, because `as_type` requires its source and
//!   result to have the same size and an unsuffixed MSL integer literal is
//!   `uint`.
//! - Every arithmetic operation is emitted as its own statement, so no
//!   contraction can form across two structured operations under
//!   `-ffp-contract=on`.
//! - NaN canonicalization is emitted as a helper whose predicate is an integer
//!   test over the reinterpreted bit pattern, so no floating-point relaxation
//!   licence reaches it.
//! - Reduction order is the IR's own bounded loop with a loop-carried
//!   dependence, never a backend topology choice.
//!
//! Not carried by any emitted operation, and therefore reported rather than
//! assumed: signed-zero results, NaN-valued arithmetic, reassociation, and
//! contraction across statements each depend on a compiler selection, reported
//! as [`MetalNumericalRequirement`](crate::record::MetalNumericalRequirement)s.
//! Subnormal preservation is not reachable by *any* selection on a flushing
//! target and is reported as a
//! [`MetalNumericalGap`](crate::record::MetalNumericalGap).
//!
//! Every public item here is a reviewed *draft* boundary (ADR 0074 §7).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use tiler_ir::kernel::{
    AddressSpace, BarrierOrdering, BarrierSpec, BinaryOp, BlockRef, BufferAccess, Builtin,
    CompareOp, ConvertOp, ExecutionScope, KernelConstant, KernelType, LoopBound, MemoryScope,
    OperationRef, OperationView, PackedExtractOp, SerialLoopRef, StagingParameter, UnaryOp,
    VerifiedBufferId, VerifiedInputExtentId, VerifiedKernel, VerifiedStagingId, VerifiedValueId,
};
use tiler_ir::schedule::{
    ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
    NumericalRealization, StagingId, SubnormalMode, TensorRole, ValueDomainProvenance,
};

use crate::diagnostic::{BarrierRejection, MetalEmitError, MetalOperationFamily};
use crate::record::{
    MetalBufferBinding, MetalEntryPoint, MetalNumericalGap, MetalNumericalRequirement,
    MetalTranslationUnit,
};
use crate::target::{MetalEmissionRealization, MetalFloatArithmeticType, MetalTargetFacts};

/// One level of emitted indentation.
const INDENT: &str = "    ";

/// Prefix of every emitted entry-point symbol.
const ENTRY_PREFIX: &str = "tiler_kernel_";

/// Prefix of every emitted binary32 NaN-canonicalization helper symbol.
const CANONICALIZE_F32_PREFIX: &str = "tiler_canonicalize_nan_f32_";

/// Prefix of every emitted `bfloat16` NaN-canonicalization helper symbol.
///
/// The width is part of the symbol rather than left to overload resolution.
/// One translation unit can carry both helpers — a portfolio may hold an `f32`
/// kernel and a `bf16` kernel — so the two must be separately nameable, and the
/// Apple numerical probe harness spells its own helper the same way, which is
/// what lets a module this backend emits be read by the recognizer that read
/// the harness's. That recognizer matches the *mangled* spelling
/// `_ZL32tiler_canonicalize_nan_bf16_7fc0DF16b`, which encodes the identifier's
/// length and its `bfloat` parameter, so the unmangled name has to agree
/// character for character rather than merely contain the dtype.
const CANONICALIZE_BF16_PREFIX: &str = "tiler_canonicalize_nan_bf16_";

/// Prefix of every emitted workgroup staging allocation.
const STAGING_PREFIX: &str = "tg";

/// The MSL attribute delivering one invocation's index within its workgroup.
///
/// The governed [`Builtin::LocalInvocationIndex`] is the *linear* coordinate a
/// cooperative tile names its participants by, so the linear
/// `[[thread_index_in_threadgroup]]` is the delivery that matches it.
/// `[[thread_position_in_threadgroup]]` is the three-component position and
/// would need a layout rule to linearize — a second, unchecked copy of the
/// workgroup geometry, which is the duplication the governed key exists to
/// avoid.
const LOCAL_INDEX_ATTRIBUTE: &str = "thread_index_in_threadgroup";

/// The MSL type the local-index parameter is declared with.
///
/// One admitted spelling rather than a caller-visible selection, which is the
/// difference from the launch index: MSL admits `ushort` and `uint` here, and
/// this backend fixes `uint` because the structured value is widened to the
/// governed 64-bit index role either way, so the narrower form buys nothing a
/// caller could act on. A second admitted spelling would become a
/// [`MetalEmissionRealization`] field at that point, exactly as the launch
/// index's did — the record exists to carry choices a caller makes, and there is
/// no choice here yet.
const LOCAL_INDEX_TYPE: &str = "uint";

/// The IEEE-754 binary32 biased-exponent field.
///
/// This constant and [`F32_SIGNIFICAND_MASK`] are used both by the Rust
/// predicate [`is_f32_nan`] and by the emitted MSL predicate in
/// [`canonicalize_helper`], so the host-side check and the device-side check
/// cannot drift apart.
const F32_EXPONENT_MASK: u32 = 0x7f80_0000;

/// The IEEE-754 binary32 significand field.
const F32_SIGNIFICAND_MASK: u32 = 0x007f_ffff;

/// The `bfloat16` biased-exponent field.
///
/// Stated as its own constant rather than derived from [`F32_EXPONENT_MASK`] by
/// a shift. `bfloat16` is binary32 truncated to its high sixteen bits, so the
/// derivation happens to hold, but the emitted predicate would then rest on
/// that format relationship silently — and the two formats are separate
/// arithmetic types everywhere else in this backend precisely because they do
/// not behave alike.
const BF16_EXPONENT_MASK: u16 = 0x7f80;

/// The `bfloat16` significand field.
const BF16_SIGNIFICAND_MASK: u16 = 0x007f;

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
/// kernels, the target facts, and the selected emission realization. Helpers required by more than one entry point are emitted once.
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
    emission: MetalEmissionRealization,
) -> Result<MetalTranslationUnit, MetalEmitError> {
    let ordered = order_kernels(kernels);
    let mut helpers = BTreeSet::new();
    let mut numerical = BTreeSet::new();
    let mut gaps = BTreeSet::new();
    let mut unstated = BTreeSet::new();
    let mut symbols: BTreeMap<String, &[u8]> = BTreeMap::new();
    let mut entry_points = Vec::with_capacity(ordered.len());
    let mut bodies = Vec::with_capacity(ordered.len());

    for kernel in ordered {
        let emitted = emit_entry_point(
            kernel,
            target,
            emission,
            &mut helpers,
            &mut numerical,
            &mut gaps,
            &mut unstated,
        )?;
        reserve_symbol(
            &mut symbols,
            emitted.entry.symbol(),
            kernel.canonical_identity().as_bytes(),
        )?;
        entry_points.push(emitted.entry);
        bodies.push(emitted.text);
    }

    let source = assemble(target, emission, &helpers, &gaps, &unstated, &bodies)?;
    Ok(MetalTranslationUnit::new(
        *target,
        emission,
        source,
        entry_points,
        numerical.into_iter().collect(),
        gaps.into_iter().collect(),
        unstated.into_iter().collect(),
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
///
/// Fallible only because the header states the structured index arithmetic type
/// by asking [`msl_type`] for it rather than by repeating a spelling this
/// backend would then hold in two places.
fn assemble(
    target: &MetalTargetFacts,
    emission: MetalEmissionRealization,
    helpers: &BTreeSet<MetalHelper>,
    gaps: &BTreeSet<MetalNumericalGap>,
    unstated: &BTreeSet<MetalFloatArithmeticType>,
    bodies: &[String],
) -> Result<String, MetalEmitError> {
    let mut source = String::new();
    source.push_str("// Generated by tiler-metal from verified structured kernel IR.\n");
    source.push_str("// Deterministic output: do not edit.\n");
    source.push_str("//\n");
    emit!(
        source,
        "// Metal Shading Language: {}\n",
        target.language.semantic_name()
    );
    emit!(
        source,
        "// Artifact family: {} (deployment minimum {})\n",
        target.platform,
        target.deployment_minimum
    );
    emit!(
        source,
        "// Launch delivery realization: [[{}]] declared as {}\n",
        emission.launch_index.attribute(),
        emission.launch_index.declared_type()
    );
    emit!(
        source,
        "// Structured index arithmetic: {}, widened explicitly from {} delivery.\n",
        msl_type(KernelType::Index)?,
        emission.launch_index.declared_type(),
    );
    // One line per arithmetic type rather than one for the target: the measured
    // Apple row flushes in `f32` and preserves in `f16`, so a single line would
    // be false for one of them whichever way it read. Every type the vocabulary
    // names is listed, including the ones this target states nothing about, so
    // a reader of the text alone can tell an unmeasured type from an absent
    // one.
    source.push_str("// Arithmetic subnormals, per floating-point type:\n");
    for arithmetic_type in MetalFloatArithmeticType::ALL {
        match target.subnormal_arithmetic.behaviour(arithmetic_type) {
            Ok(behaviour) => emit!(source, "//   {arithmetic_type}: {behaviour}\n"),
            Err(_) => emit!(source, "//   {arithmetic_type}: not stated\n"),
        }
    }
    source.push_str("//\n");
    // Stated once for every emitted width rather than once per width, unlike
    // the subnormal block above: all three properties are width-independent
    // consequences of how this emitter writes operations, so a per-width line
    // would repeat one claim rather than distinguish two facts.
    source.push_str("// Carried by these operations under every math mode: every floating-point\n");
    source.push_str("// immediate is its exact bit pattern, every arithmetic operation is one\n");
    source
        .push_str("// statement, and every NaN test is an integer test over reinterpreted bits.\n");
    source.push_str("//\n");
    if gaps.is_empty() {
        source.push_str("// Declared numerical obligations this profile cannot realize: none.\n");
    } else {
        source.push_str("// Declared numerical obligations this profile cannot realize:\n");
        for gap in gaps {
            emit!(source, "//   {gap}\n");
        }
    }
    // Stated separately from the gaps above because it is a different claim: a
    // gap says the target cannot honour the obligation, this says nothing is
    // known either way, and it also means the gap list above is incomplete.
    if unstated.is_empty() {
        source.push_str("// Arithmetic types used with no stated subnormal fact: none.\n");
    } else {
        source.push_str("// Arithmetic types used with no stated subnormal fact:\n");
        for arithmetic_type in unstated {
            emit!(source, "//   {arithmetic_type}\n");
        }
        source.push_str("// The obligations above are therefore incomplete.\n");
    }
    source.push('\n');
    source.push_str("#include <metal_stdlib>\n");
    source.push_str("using namespace metal;\n");

    for helper in helpers {
        source.push('\n');
        source.push_str(&helper.definition());
    }
    for body in bodies {
        source.push('\n');
        source.push_str(body);
    }
    Ok(source)
}

/// One static helper an emitted translation unit may need.
///
/// A typed vocabulary rather than the bare canonical-NaN pattern the set once
/// held, because there are now two helpers and they are not two constants of one
/// shape: the canonicalization is parameterized by a bit pattern and the extrema
/// fixup is not. The exhaustive match in [`Self::definition`] is what forces a
/// third helper to state its own text rather than borrow whichever it resembles.
///
/// Ordering is by variant and then by payload, so the emitted preamble is
/// deterministic whatever order the bodies requested their helpers in.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum MetalHelper {
    /// Replaces an arithmetic binary32 NaN with one exact canonical pattern.
    CanonicalizeF32Nan {
        /// The canonical arithmetic-NaN payload this kernel declares.
        bits: u32,
    },
    /// Replaces an arithmetic `bfloat16` NaN with one exact canonical pattern.
    ///
    /// A separate variant rather than a width field on the one above, because
    /// the two helpers differ in more than a payload: parameter type, carrier
    /// type, mask widths, and the literal spelling all change together, and a
    /// unit may need both at once.
    CanonicalizeBf16Nan {
        /// The canonical arithmetic-NaN payload this kernel declares, narrowed
        /// to the width that can hold it.
        bits: u16,
    },
    /// The IEEE 754-2019 `maximum` of two binary32 values.
    MaximumF32,
}

impl MetalHelper {
    /// Returns the helper's complete MSL definition.
    fn definition(self) -> String {
        match self {
            Self::CanonicalizeF32Nan { bits } => canonicalize_helper(bits),
            Self::CanonicalizeBf16Nan { bits } => canonicalize_bf16_helper(bits),
            Self::MaximumF32 => maximum_helper(),
        }
    }
}

/// Returns the symbol of the binary32 `maximum` helper.
const MAXIMUM_F32_SYMBOL: &str = "tiler_maximum_f32";

/// Returns the exact realization of IEEE 754-2019 `maximum` at binary32.
///
/// # Why `fmax` does not appear in it
///
/// ADR 0023 admits two extrema families and requires a backend to lower one
/// "only when the complete NaN and zero-tie behavior matches", emitting an exact
/// fixup otherwise. Metal's `fmax` matches *neither*: [Numerical
/// semantics](../../../docs/numerical-semantics.md) records that it is
/// number-preferring — so it implements the `MaximumNumber` NaN rule rather than
/// the propagating one — and that its signed-zero result can depend on operand
/// order, so it does not deterministically implement `MaximumNumber` either.
/// This is that fixup, and it is written without `fmax` at all rather than around
/// it.
///
/// # Why it is comparisons rather than an intrinsic, and what that buys
///
/// The four arms below are exhaustive over the IEEE-754 trichotomy plus the
/// unordered case, and each is exact:
///
/// - `a < b` and `b < a` give the greater operand;
/// - `a == b` is true for identical values *and* for opposite zeros, and the
///   bitwise `and` of the two payloads returns the common bits in the first case
///   and clears the sign bit in the second — which is `-0.0 < +0.0` written as
///   one operation rather than as a branch on a sign test;
/// - all three comparisons false means the operands are *unordered*, which for
///   binary32 means at least one is a NaN. That is where the propagating family
///   differs from `fmax`, and the canonical pattern is returned directly rather
///   than by producing some NaN and relying on a later canonicalization.
///
/// The consequence worth stating is what this *avoids* claiming. The retained
/// emission probe measured which intrinsic `fmax(a, b)` selects under the
/// governed flags and under the compiler default — `air.fmax.f32` and
/// `air.fast_fmax.f32` — and measured nothing about what either *returns*. A
/// lowering through the intrinsic would have to rest on the unmeasured half; this
/// one rests on the language's comparison semantics, so the intrinsic-selection
/// question does not arise for this construct at all.
///
/// **The safe math mode is nevertheless load-bearing, and the emitter records
/// it.** Under `-fmetal-math-mode=fast` LLVM's `nnan` licence permits folding an
/// unordered comparison to a constant, which would delete the NaN arm. The
/// requirement is about the comparison's *operands*, exactly as the
/// canonicalization helper's is.
fn maximum_helper() -> String {
    format!(
        "// The IEEE 754-2019 maximum of two binary32 values: NaN-propagating, with\n\
         // -0.0 ordered below +0.0.\n\
         //\n\
         // Deliberately not fmax, which prefers numbers and whose signed-zero result\n\
         // depends on operand order, so it implements neither admitted extrema family.\n\
         static inline float {MAXIMUM_F32_SYMBOL}(float left, float right) {{\n\
         {INDENT}if (left < right) {{ return right; }}\n\
         {INDENT}if (right < left) {{ return left; }}\n\
         {INDENT}// Equal values, including the opposite-zero pair: the bitwise and returns\n\
         {INDENT}// the common payload, and clears the sign bit exactly when one operand is\n\
         {INDENT}// a positive zero.\n\
         {INDENT}if (left == right) {{\n\
         {INDENT}{INDENT}return as_type<float>(as_type<uint>(left) & as_type<uint>(right));\n\
         {INDENT}}}\n\
         {INDENT}// Unordered: at least one operand is a NaN, and this family propagates it.\n\
         {INDENT}return as_type<float>({CANONICAL_F32_NAN:#010x}u);\n\
         }}\n"
    )
}

/// The canonical arithmetic-NaN payload the extrema fixup returns.
///
/// Stated as a constant rather than read from the kernel's declaration, because
/// the fixup's NaN arm is *this helper's* answer to an unordered comparison
/// rather than an arithmetic result the kernel canonicalizes. The reduction that
/// consumes it applies `ConvertOp::CanonicalizeF32Nan` afterwards regardless, so a
/// kernel declaring another pattern still commits its own.
const CANONICAL_F32_NAN: u32 = 0x7fc0_0000;

/// Returns the NaN-canonicalization helper for one exact canonical pattern.
///
/// # Why the predicate is an integer test and not `isnan`
///
/// The obligation this ticket carries is that canonicalization is realized by
/// the emitted *operations*, so that flipping a math-mode flag cannot silently
/// change a conforming result. `isnan` is a floating-point predicate: under
/// `-fmetal-math-mode=fast` the Metal Shading Language does not define its
/// behaviour, and LLVM's `nnan` licence permits folding an `fcmp uno` to
/// `false`. Whether a given front end exercises that licence is a property of
/// that front end, not of this source.
///
/// Reinterpreting the value and testing the IEEE-754 fields is not a
/// floating-point operation, so no floating-point relaxation licence reaches
/// it under any math mode. The guarantee becomes a language-level property of
/// the emitted text rather than a measured property of one toolchain build.
///
/// **Measurement.** On Metal 32023.883 the two forms are observationally
/// identical: the front end lowers `isnan` to `bitcast`, `and 0x7fffffff`, and
/// `icmp ugt 0x7f800000` at `-O0` and under `safe`, `relaxed`, and `fast`
/// alike, and the shipped AIR contains no floating-point NaN predicate. Both
/// forms canonicalize `0x7fabcdef`, `0xffc00001`, and `0x7f800001` to
/// `0x7fc00000` on an Apple M4 Max across all fifteen combinations of `-O0`,
/// `-O1`, `-O2`, `-O3`, `-Os` with `safe`, `relaxed`, `fast`. The integer form
/// is emitted because it is guaranteed rather than because the other one was
/// observed to fail.
///
/// This does not make the helper sufficient on its own. Under `nnan` the
/// arithmetic *producing* a NaN has no defined result, so there is nothing left
/// for a canonicalization to map, which is why emission still records
/// [`MetalNumericalRequirement::SafeMathMode`].
fn canonicalize_helper(bits: u32) -> String {
    let symbol = canonicalize_symbol(bits);
    format!(
        "// Replaces an arithmetic NaN with the canonical pattern {bits:#010x}.\n\
         //\n\
         // The predicate is an integer test over the reinterpreted bit pattern rather\n\
         // than a floating-point one, so no math-mode relaxation licence reaches it.\n\
         static inline float {symbol}(float value) {{\n\
         {INDENT}uint pattern = as_type<uint>(value);\n\
         {INDENT}bool nan = (pattern & {F32_EXPONENT_MASK:#010x}u) == {F32_EXPONENT_MASK:#010x}u\n\
         {INDENT}{INDENT}&& (pattern & {F32_SIGNIFICAND_MASK:#010x}u) != 0x00000000u;\n\
         {INDENT}return nan ? as_type<float>({bits:#010x}u) : value;\n\
         }}\n"
    )
}

/// Returns the deterministic helper symbol for one canonical NaN pattern.
fn canonicalize_symbol(bits: u32) -> String {
    format!("{CANONICALIZE_F32_PREFIX}{bits:08x}")
}

/// Returns whether a bit pattern encodes an IEEE-754 binary32 NaN.
///
/// Quietness is the numerical contract's rule, not this backend's; emission
/// only refuses a "canonical NaN" that is not a NaN at all. This is the same
/// predicate [`canonicalize_helper`] emits, over the same two constants.
pub(crate) const fn is_f32_nan(bits: u32) -> bool {
    bits & F32_EXPONENT_MASK == F32_EXPONENT_MASK && bits & F32_SIGNIFICAND_MASK != 0
}

/// Returns the NaN-canonicalization helper for one exact `bfloat16` pattern.
///
/// The reasoning is [`canonicalize_helper`]'s and is not repeated: the
/// predicate is an integer test over the reinterpreted bit pattern, so no
/// floating-point relaxation licence reaches it under any math mode, and the
/// helper is still insufficient on its own because `nnan` leaves the arithmetic
/// that produced a NaN with no defined result to canonicalize.
///
/// What differs is the carrier. `bfloat16` is sixteen bits wide, so the
/// reinterpretation goes through `ushort` rather than `uint`, and the returned
/// literal needs [`bf16_literal`]'s narrowing conversion. That carrier is the
/// one the Apple numerical probe harness measured `bfloat` kernels through, so
/// the emitted text is the shape those measurements were made over rather than
/// a second spelling of the same idea.
fn canonicalize_bf16_helper(bits: u16) -> String {
    let symbol = canonicalize_bf16_symbol(bits);
    let canonical = bf16_literal(bits);
    format!(
        "// Replaces an arithmetic NaN with the canonical pattern {bits:#06x}.\n\
         //\n\
         // The predicate is an integer test over the reinterpreted bit pattern rather\n\
         // than a floating-point one, so no math-mode relaxation licence reaches it.\n\
         static inline bfloat {symbol}(bfloat value) {{\n\
         {INDENT}ushort pattern = as_type<ushort>(value);\n\
         {INDENT}bool nan = (pattern & {BF16_EXPONENT_MASK:#06x}u) == {BF16_EXPONENT_MASK:#06x}u\n\
         {INDENT}{INDENT}&& (pattern & {BF16_SIGNIFICAND_MASK:#06x}u) != 0x0000u;\n\
         {INDENT}return nan ? {canonical} : value;\n\
         }}\n"
    )
}

/// Returns the deterministic helper symbol for one canonical `bfloat16` NaN.
fn canonicalize_bf16_symbol(bits: u16) -> String {
    format!("{CANONICALIZE_BF16_PREFIX}{bits:04x}")
}

/// Returns the MSL expression reinterpreting one exact `bfloat16` bit pattern.
///
/// The `ushort` conversion is load-bearing rather than decoration. An
/// unsuffixed MSL integer literal is `uint`, and `as_type` requires its source
/// and its result to have the same size, so a sixteen-bit pattern has to be
/// narrowed before it can be reinterpreted — while the `f32` form must *not*
/// carry a conversion. That is why the two spellings are written separately
/// instead of being parameterized over a width.
fn bf16_literal(bits: u16) -> String {
    format!("as_type<bfloat>(ushort({bits:#06x}u))")
}

/// Returns whether a bit pattern encodes a `bfloat16` NaN.
///
/// The counterpart of [`is_f32_nan`] at the narrower width, over the same two
/// fields and with the same rule: quietness belongs to the numerical contract,
/// and emission only refuses a "canonical NaN" that is not a NaN at all. This
/// is the predicate [`canonicalize_bf16_helper`] emits.
pub(crate) const fn is_bf16_nan(bits: u16) -> bool {
    bits & BF16_EXPONENT_MASK == BF16_EXPONENT_MASK && bits & BF16_SIGNIFICAND_MASK != 0
}

/// Narrows a kernel's declared canonical arithmetic NaN to `bfloat16`.
///
/// `NumericalRealization::canonical_arithmetic_nan_bits` is a 32-bit field
/// while `bfloat16`'s canonical arithmetic NaN is sixteen bits, so a `bf16`
/// region declares its pattern zero-extended. Both halves are checked: a
/// payload with any high bit set is not a zero-extended `bfloat16` pattern at
/// all — reading its low half would silently discard what the producer
/// declared — and a low half that is not a NaN encoding would make the
/// "canonicalization" produce a finite or infinite value.
///
/// The scheduled-region verifier already requires a `bf16` pointwise region to
/// declare exactly `CANONICAL_BF16_ARITHMETIC_NAN_BITS`, so neither refusal is
/// reachable through a verified kernel today. That is why this is written as a
/// refusal rather than an `expect`, for the same reason `staging_declaration`
/// refuses an address space the verifier already constrains: a widened producer
/// contract must stop at this backend, which is the one that would otherwise
/// emit a helper computing the wrong thing.
pub(crate) fn bf16_canonical_nan(bits: u32) -> Result<u16, MetalEmitError> {
    let narrowed = u16::try_from(bits).map_err(|_| MetalEmitError::InvalidCanonicalNan { bits })?;
    if is_bf16_nan(narrowed) {
        Ok(narrowed)
    } else {
        Err(MetalEmitError::InvalidCanonicalNan { bits })
    }
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
    emission: MetalEmissionRealization,
    helpers: &mut BTreeSet<MetalHelper>,
    numerical: &mut BTreeSet<MetalNumericalRequirement>,
    gaps: &mut BTreeSet<MetalNumericalGap>,
    unstated: &mut BTreeSet<MetalFloatArithmeticType>,
) -> Result<EmittedEntryPoint, MetalEmitError> {
    let declared = kernel
        .buffers()
        .len()
        .saturating_add(kernel.input_extents().len());
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

    // The signature is the kernel's declaration, in declaration order, and the
    // argument-table index is the declaration ordinal. Two things follow, and
    // both were previously wrong.
    //
    // A parameter the body never references still occupies its position. That
    // is what a reduction over an empty domain needs: it writes its identity
    // element for every output element and reads its input never, so under a
    // table derived from *use* it had no position at all and emission refused
    // it. Declaring it and not reading it is legal MSL and leaves the ABI
    // exactly what the artifact's own binding table says it is.
    //
    // And the ordinals no longer depend on the order the body happens to touch
    // its buffers in. Under first-use order a body that stored before it loaded
    // produced a table whose positions disagreed with the declaration the
    // artifact records, and nothing compared the two.
    let mut buffers = BTreeMap::new();
    let mut bindings = Vec::with_capacity(declared);
    for (ordinal, (handle, parameter)) in kernel.declared_buffers().enumerate() {
        let index = u32::try_from(ordinal).map_err(|_| MetalEmitError::BufferBindingLimit {
            required: declared,
            limit: target.buffer_binding_limit,
        })?;
        buffers.insert(handle, index);
        bindings.push(MetalBufferBinding::new(index, parameter));
    }
    let mut extents = BTreeMap::new();
    for (ordinal, (handle, _)) in kernel.declared_input_extents().enumerate() {
        let index = u32::try_from(ordinal).map_err(|_| MetalEmitError::BufferBindingLimit {
            required: declared.saturating_add(kernel.input_extents().len()),
            limit: target.buffer_binding_limit,
        })?;
        extents.insert(handle, index);
    }

    let mut emitter = KernelEmitter {
        kernel,
        target,
        emission,
        helpers,
        numerical,
        gaps,
        unstated,
        names: BTreeMap::new(),
        next: 0,
        buffers,
        extents,
        bindings,
        out: String::new(),
        indent: 1,
    };
    emitter.emit_block(kernel.body())?;
    let KernelEmitter {
        bindings,
        out: body,
        ..
    } = emitter;

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
            "//   buffer({}): {} tensor, {:?}, {:?} space, {:?} access, {} element(s)\n",
            binding.index(),
            tensor_role_comment(parameter.tensor),
            parameter.element_type,
            parameter.address_space,
            parameter.access,
            parameter.element_count,
        );
    }
    for parameter in kernel.staging() {
        emit!(
            text,
            "//   {}: {:?}, {:?} space, {} slot(s)\n",
            staging_name(parameter.staging),
            parameter.element_type,
            parameter.address_space,
            parameter.element_count,
        );
    }
    for (ordinal, (_, parameter)) in kernel.declared_input_extents().enumerate() {
        emit!(
            text,
            "//   extent({ordinal}): {:?} tensor, axis {}\n",
            parameter.tensor,
            parameter.axis.get(),
        );
    }

    let mut parameters = Vec::with_capacity(
        bindings
            .len()
            .saturating_add(kernel.input_extents().len())
            .saturating_add(1),
    );
    for binding in &bindings {
        parameters.push(parameter_declaration(binding)?);
    }
    let extent_base =
        u32::try_from(bindings.len()).map_err(|_| MetalEmitError::BufferBindingLimit {
            required: declared.saturating_add(kernel.input_extents().len()),
            limit: target.buffer_binding_limit,
        })?;
    for (ordinal, _) in kernel.declared_input_extents().enumerate() {
        let index = extent_base.saturating_add(u32::try_from(ordinal).unwrap_or(u32::MAX));
        parameters.push(format!("constant ulong& e{ordinal} [[buffer({index})]]"));
    }
    for builtin in kernel.admitted_builtins() {
        parameters.push(builtin_declaration(*builtin, emission)?);
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
    // Workgroup staging is declared inside the entry point, never as a
    // parameter. A parameter's position is its argument-table ordinal, and
    // threadgroup storage of a statically known extent has no argument-table
    // position at all — declaring one would re-base every later `[[buffer(N)]]`
    // ordinal and change what an existing signature position means. The extents
    // come from the kernel's own declarations, which the structured-kernel
    // verifier already proved equal to the region's cooperative tile, so nothing
    // here re-derives a slot count from a producer's word.
    for parameter in kernel.staging() {
        text.push_str(&staging_declaration(parameter)?);
    }
    text.push_str(&body);
    text.push_str("}\n");

    Ok(EmittedEntryPoint {
        entry: MetalEntryPoint::new(
            symbol,
            kernel.canonical_identity().clone(),
            bindings,
            u32::try_from(kernel.input_extents().len()).unwrap_or(u32::MAX),
        ),
        text,
    })
}

/// Returns the MSL declaration of one buffer parameter.
/// Names one boundary tensor role for the emitted signature comment.
///
/// Rendered rather than `Debug`-formatted: a role carrying an ordinal prints as
/// a Rust struct literal under `{:?}`, and this text lands in generated Metal
/// source that a reader reads beside the buffer index it explains.
fn tensor_role_comment(role: TensorRole) -> String {
    match role {
        TensorRole::Input { ordinal } => format!("Input {}", ordinal.get()),
        TensorRole::Intermediate => "Intermediate".to_owned(),
        TensorRole::Output => "Output".to_owned(),
    }
}

fn parameter_declaration(binding: &MetalBufferBinding) -> Result<String, MetalEmitError> {
    let parameter = binding.parameter();
    let element = msl_type(parameter.element_type)?;
    let space = address_space_declaration(parameter.address_space, parameter.access)?;
    let index = binding.index();
    Ok(format!("{space} {element} *b{index} [[buffer({index})]]"))
}

/// Returns the deterministic emitted name of one workgroup staging allocation.
///
/// Keyed by the scheduled [`StagingId`] rather than by declaration position, so
/// the name a staged access emits and the name the entry point declares come
/// from one fact. The structured-kernel verifier proves the declared list equals
/// the tile's, in order, so the two can never disagree — but deriving both from
/// the same identifier means a future divergence would be a build error rather
/// than a body referencing an allocation that was declared under another name.
fn staging_name(staging: StagingId) -> String {
    format!("{STAGING_PREFIX}{}", staging.get())
}

/// Returns the MSL declaration of one workgroup staging allocation.
///
/// The address space is matched by name and every space but `Workgroup` is
/// refused, rather than assumed from the fact that the IR calls this "staging".
/// The structured-kernel verifier does require `AddressSpace::Workgroup` here,
/// so this refusal is unreachable through a verified kernel — which is exactly
/// why it is written as a refusal and not an `expect`: a widened `AddressSpace`
/// must stop at this backend, the one that has to decide the new space's MSL
/// storage qualifier.
fn staging_declaration(parameter: StagingParameter) -> Result<String, MetalEmitError> {
    let qualifier = match parameter.address_space {
        AddressSpace::Workgroup => "threadgroup",
        space @ (AddressSpace::Device
        | AddressSpace::Constant
        | AddressSpace::InvocationPrivate) => {
            return Err(MetalEmitError::UnsupportedAddressSpace { space });
        }
    };
    let element = msl_type(parameter.element_type)?;
    let name = staging_name(parameter.staging);
    let slots = parameter.element_count;
    Ok(format!("{INDENT}{qualifier} {element} {name}[{slots}];\n"))
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
        },
        AddressSpace::Constant => match access {
            BufferAccess::Read => Ok("constant"),
            // The constant space is read-only, so a writable parameter is
            // rejected instead of being silently promoted to a device binding.
            BufferAccess::Write => Err(MetalEmitError::UnsupportedBufferAccess { space, access }),
        },
        // Workgroup storage is a `[[threadgroup(N)]]` binding a buffer
        // parameter cannot name, and invocation-private storage is not a
        // parameter space. Matched by name rather than by wildcard so a widened
        // `AddressSpace` stops the build at this backend, which is the one that
        // has to decide whether the new space has a realization.
        AddressSpace::Workgroup | AddressSpace::InvocationPrivate => {
            Err(MetalEmitError::UnsupportedAddressSpace { space })
        }
    }
}

/// Returns the MSL declaration of one admitted launch builtin parameter.
fn builtin_declaration(
    builtin: Builtin,
    emission: MetalEmissionRealization,
) -> Result<String, MetalEmitError> {
    let name = builtin_parameter(builtin)?;
    let declared_type = builtin_declared_type(builtin, emission)?;
    let attribute = match builtin {
        Builtin::GlobalInvocationIndex => emission.launch_index.attribute(),
        Builtin::LocalInvocationIndex => LOCAL_INDEX_ATTRIBUTE,
        _ => {
            return Err(MetalEmitError::UnsupportedOperation {
                family: MetalOperationFamily::Builtin,
            });
        }
    };
    Ok(format!("{declared_type} {name} [[{attribute}]]"))
}

/// Returns the MSL type one admitted builtin's attributed parameter carries.
///
/// The global index reads its declared type from the selected
/// [`LaunchIndexRealization`](crate::target::LaunchIndexRealization), because
/// that spelling is a backend *choice* among several MSL admits and a caller
/// records which one it made. The local index has one admitted spelling in this
/// backend and therefore no selection to read.
///
/// **The asymmetry is stated rather than smoothed over.** Reading the launch
/// index's realization for both — which is what this function replaced — would
/// have declared `[[thread_index_in_threadgroup]]` under the launch attribute's
/// type and, worse, under the launch attribute itself. That was unreachable only
/// because [`builtin_parameter`] refused the local index outright, so admitting
/// the local index is exactly what makes the shared read wrong.
fn builtin_declared_type(
    builtin: Builtin,
    emission: MetalEmissionRealization,
) -> Result<&'static str, MetalEmitError> {
    match builtin {
        Builtin::GlobalInvocationIndex => Ok(emission.launch_index.declared_type()),
        Builtin::LocalInvocationIndex => Ok(LOCAL_INDEX_TYPE),
        _ => Err(MetalEmitError::UnsupportedOperation {
            family: MetalOperationFamily::Builtin,
        }),
    }
}

/// Returns the deterministic parameter name of one governed launch builtin.
fn builtin_parameter(builtin: Builtin) -> Result<&'static str, MetalEmitError> {
    match builtin {
        Builtin::GlobalInvocationIndex => Ok("tiler_global_invocation_index"),
        Builtin::LocalInvocationIndex => Ok("tiler_local_invocation_index"),
        _ => Err(MetalEmitError::UnsupportedOperation {
            family: MetalOperationFamily::Builtin,
        }),
    }
}

/// Returns the MSL spelling of one governed structured type, or refuses it.
///
/// Exhaustive by name. `KernelType` is not `#[non_exhaustive]`, so widening it
/// stops the build here — at the backend that has to decide the new type's MSL
/// spelling — and the decision available at that point is a spelling *or* a
/// refusal, which is why this is fallible rather than total.
///
/// `Bf16` spells `bfloat`, and it did not before. The arm was a refusal while
/// this backend had no BF16 constant reinterpretation and no BF16 NaN
/// canonicalization helper, because a spelling on its own would have let a BF16
/// kernel emit source that compiles while the numerical machinery the emitted
/// code depends on was absent. Both now exist — [`bf16_literal`] and
/// [`canonicalize_bf16_helper`] — and the target vocabulary carries a measured
/// `bf16` subnormal row of its own
/// ([`MetalSubnormalArithmeticFacts`](crate::target::MetalSubnormalArithmeticFacts)),
/// so a target that states nothing about `bf16` still fails the conformance
/// claim rather than borrowing a neighbour's fact. The spelling is admissible
/// because that machinery landed with it, not because the refusal was
/// inconvenient.
///
/// This is a translation fact and not a dispatch claim. That the emitted
/// `bfloat` module *runs* is a per-target-family measurement the profile owns:
/// the retained Apple record dispatches `bfloat` on the measured macOS row and
/// records the iOS Simulator compiling and linking the same module and then
/// refusing to create a pipeline for it. Emission is the same on both, which is
/// exactly why the refusal cannot live here.
///
/// **Every governed type now has a spelling, so the `Err` arm is currently
/// vacant, and the signature stays fallible anyway.** This is the seam a
/// widened `KernelType` lands on: the match is exhaustive over a vocabulary
/// that is deliberately not `#[non_exhaustive]`, so adding `F16` or `F64` stops
/// the build here, and the decision available at that point must include
/// "refuse", which a total signature would have removed while the caller chain
/// — `parameter_declaration`, `staging_declaration`,
/// [`KernelEmitter::value_type`], [`KernelEmitter::emit_convert`], and the
/// translation-unit header — was rewritten to drop the propagation.
/// [`MetalEmitError::UnsupportedValueType`] is kept for the same reason, and
/// `the_unspelled_value_type_refusal_keeps_its_rule_and_rendering` exercises its
/// identifier and rendering directly, so the widening that reaches for it finds
/// a surface something still checks.
#[expect(
    clippy::unnecessary_wraps,
    reason = "the Err arm is the seam a widened KernelType must land on; see above"
)]
pub(crate) const fn msl_type(value_type: KernelType) -> Result<&'static str, MetalEmitError> {
    match value_type {
        KernelType::Bool => Ok("bool"),
        KernelType::U8 => Ok("uchar"),
        KernelType::I32 => Ok("int"),
        KernelType::Index => Ok("uint64_t"),
        KernelType::F32 => Ok("float"),
        KernelType::Bf16 => Ok("bfloat"),
    }
}

/// Returns the compiler requirements one numerical realization imposes.
///
/// Each vocabulary matched here is *not* `#[non_exhaustive]`, so widening the
/// numerical contract is a compile error at this site rather than a silently
/// dropped requirement.
///
/// A permission the realization *grants* names no flag. This set says what the
/// emitted source cannot tolerate, and a granted freedom is tolerated under
/// every selection, so the caller stays free to compile more strictly than the
/// contract demands.
///
/// **Neither subnormal behaviour names a compiler selection, and the two
/// reasons differ.** Preservation names none because no `-fmetal-math-mode`,
/// `-ffp-contract`, `-fmetal-math-fp32-functions`, or `-O` selection preserves
/// subnormals through `f32` arithmetic on the measured flushing row — the front
/// end emits `air.compile.denorms_disable` under all of them — so naming a flag
/// would assert a guarantee it does not deliver. Flushing names none because
/// that same measurement makes the flush unconditional: no selection has to be
/// made to obtain it. That `air.compile.denorms_disable` is emitted identically
/// for a dtype whose subnormals are *not* disabled is a further reason no flag
/// belongs here — the module declaration is a compile-side record of what was
/// requested, not of what the hardware does. Whether the target actually
/// honours the declared behaviour is a target fact, not a flag choice, and is
/// routed to [`KernelEmitter::record_subnormal_obligation`] as a
/// [`MetalNumericalGap`] or, where no fact is stated for the arithmetic type,
/// as an unstated type.
pub(crate) fn realization_requirements(
    realization: NumericalRealization,
) -> BTreeSet<MetalNumericalRequirement> {
    let mut requirements = BTreeSet::new();
    match realization.contraction {
        NumericalPermission::Forbidden => {
            requirements.insert(MetalNumericalRequirement::NoFloatingPointContraction);
        }
        NumericalPermission::Permitted => {}
    }
    if requires_safe_math(realization) {
        requirements.insert(MetalNumericalRequirement::SafeMathMode);
    }
    for mode in [realization.input_subnormals, realization.result_subnormals] {
        match mode {
            SubnormalMode::Preserve | SubnormalMode::FlushToZero { zero_sign: _ } => {}
        }
    }
    requirements
}

/// Returns whether a realization forbids a licence of relaxed Metal math.
///
/// `safe` is required for every dimension the emitted operations cannot carry:
/// reassociation and permutation, signed-zero elimination, and exceptional
/// values unless their absence rests on compiler proof or a runtime validation.
/// A caller declaration is intentionally treated like no assumption because it
/// is ineligible to justify a correctness-sensitive relaxation.
const fn requires_safe_math(realization: NumericalRealization) -> bool {
    matches!(realization.reassociation, NumericalPermission::Forbidden)
        || matches!(realization.permutation, NumericalPermission::Forbidden)
        || matches!(realization.signed_zero, NumericalPermission::Forbidden)
        || exceptional_values_require_safe_math(realization.nan_assumptions)
        || exceptional_values_require_safe_math(realization.infinity_assumptions)
}

/// Returns whether one exceptional-value contract can justify relaxed math.
const fn exceptional_values_require_safe_math(assumption: ExceptionalValueAssumption) -> bool {
    match assumption {
        ExceptionalValueAssumption::MakeNoAssumption
        | ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
        } => true,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance:
                ValueDomainProvenance::CompilerProven | ValueDomainProvenance::RuntimeValidated,
        } => false,
    }
}

/// Returns the gap one declared subnormal behaviour has against a target fact.
///
/// `target` is the behaviour stated for the arithmetic type the operation is
/// performed in, resolved by the caller. Nothing here can reach a fact stated
/// for another type, which is the point: the measured Apple row flushes in
/// `f32` and preserves in `f16`, so a comparison that took the target's
/// behaviour without a type would answer for whichever one happened to be
/// stated.
///
/// The target owner projects its fact totally into [`SubnormalMode`] before
/// this comparison. The comparison is therefore exhaustive in one shared
/// vocabulary: widening either vocabulary stops the owner-side projection or
/// this match rather than falling through a wildcard.
///
/// A declared flush is honoured when the owner-projected target behaviour names
/// the *same* zero the program named. The target-side zero vocabulary remains a
/// distinct type at the declaration boundary;
/// [`MetalSubnormalArithmetic::subnormal_mode`](crate::target::MetalSubnormalArithmetic::subnormal_mode)
/// performs its total conversion before this shared-vocabulary comparison. A
/// program asking for `AlwaysPositive` on the measured sign-preserving Apple
/// flush is a gap, because running it would return `0x80000000` where the
/// program asked for `0x00000000`.
const fn subnormal_gap(
    declared: SubnormalMode,
    target: SubnormalMode,
) -> Option<MetalNumericalGap> {
    match (declared, target) {
        (SubnormalMode::Preserve, SubnormalMode::FlushToZero { .. }) => {
            Some(MetalNumericalGap::SubnormalFlushInArithmetic)
        }
        (SubnormalMode::FlushToZero { zero_sign: _ }, SubnormalMode::Preserve) => {
            Some(MetalNumericalGap::SubnormalPreservationInArithmetic)
        }
        (SubnormalMode::Preserve, SubnormalMode::Preserve)
        | (
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
        )
        | (
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
        ) => None,
        (
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
        )
        | (
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
        ) => Some(MetalNumericalGap::FlushedZeroSignMismatch),
    }
}

/// Returns the target-neutral arithmetic type one MSL arithmetic type denotes.
///
/// A translation between two vocabularies for one thing, written as an
/// exhaustive match so widening either set is a build error here rather than a
/// silent mapping onto whichever neighbour happens to be listed. It is needed
/// because subnormal *freedom* is a property of the value domain, stated in the
/// IR's dtype vocabulary, while the subnormal *facts* it is weighed against are
/// measurements indexed by the target's own MSL spellings.
const fn ir_arithmetic_type(arithmetic_type: MetalFloatArithmeticType) -> ArithmeticType {
    match arithmetic_type {
        MetalFloatArithmeticType::F32 => ArithmeticType::F32,
        MetalFloatArithmeticType::F16 => ArithmeticType::F16,
        MetalFloatArithmeticType::Bf16 => ArithmeticType::Bf16,
    }
}

/// How one governed binary construct is realized in emitted MSL.
///
/// Separated from [`KernelEmitter::emit_binary`] so the mapping is a pure
/// function of the operation rather than a step inside a statement emission,
/// which is what lets `every_binary_construct_has_a_metal_realization` call it
/// over a list whose declared length is `variant_count`. That is the only
/// mechanism that turns a construct appended to [`BinaryOp`] into a build error
/// in this crate: the vocabulary is `#[non_exhaustive]`, so the wildcard in
/// [`binary_realization`] cannot be removed and `rustc` will never close that
/// match on this backend's behalf. A tag appended in `tiler-ir` reaching a
/// silent run-time refusal is exactly how `IndexSubtract` arrived unemittable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BinaryRealization {
    /// An MSL infix operator, with the arithmetic type whose subnormal fact it
    /// selects.
    ///
    /// The type is carried rather than reduced to a boolean because the
    /// subnormal fact it selects differs by type: reading `f32`'s fact for a
    /// `f16` operation would report a flush against a value the measured Apple
    /// hardware carries exactly. `None` is an operand type with no
    /// floating-point contract to weigh, not an unknown one.
    Operator {
        /// The emitted MSL operator token.
        operator: &'static str,
        /// The arithmetic type whose subnormal fact this operation weighs.
        arithmetic_type: Option<MetalFloatArithmeticType>,
    },
    /// The emitted IEEE 754-2019 `maximum` helper call.
    ///
    /// A variant rather than an operator spelling because MSL has no operator
    /// for it, the intrinsic that looks like one implements a different
    /// contract, and the call carries two obligations an operator does not.
    MaximumF32,
}

/// Returns the MSL realization of one governed binary construct, or refuses it.
///
/// # The unsigned subtraction, and where its non-negativity actually comes from
///
/// [`BinaryOp::IndexSubtract`] is the one construct here whose contract a
/// realization can violate *silently*. Its result is declared proven
/// non-negative; the index role's MSL spelling is `uint64_t` ([`msl_type`]), and
/// MSL subtraction on an unsigned integer is modular, so a violated proof does
/// not trap or produce a negative value — it produces an index near `2^64` that
/// the next emitted statement scales and adds into a buffer subscript. That is
/// the failure this comment exists to keep visible.
///
/// **The emitted form is the plain difference, and the reasoning is not
/// re-derived here.** Three candidates were weighed and two are defects rather
/// than alternatives: a clamped `minuend - min(minuend, subtrahend)` keeps the
/// index in range by returning a *different element*, which is the silently
/// wrong result the fail-closed rule forbids; and a widened signed form moves
/// the same violation from defined wrapping into C++ signed overflow, which is
/// undefined behaviour and still converts back to a huge unsigned index. The
/// difference is emitted exactly, and what carries the proof is the operation
/// vocabulary, not the statement.
///
/// **The proof, stated once so a reader of the emitted text can check it.** The
/// producing lowering is a reindex mirror, which emits `extent - 1 - c` for the
/// coordinate `c` of the axis whose extent is `extent`. The bound `c < extent`
/// reaches the emitted body by one of two routes, and both are visible in the
/// text this module emits:
///
/// - The decode's own wrap. `c` is the result of an emitted
///   [`BinaryOp::IndexModulo`] whose divisor is that same `extent`, so
///   `c <= extent - 1` follows from the statement two lines above the
///   subtraction.
/// - The domain guard, where the lowering elides that wrap as redundant. It is
///   elided exactly when the decode names the most significant window of the
///   linear coordinate, so `c` is `index / divisor` with
///   `divisor * extent` equal to the region's element count — and the emitted
///   [`OperationView::Predicated`] guard above it establishes
///   `index < element_count`, giving `c < extent` again.
///
/// **This module proves neither, deliberately.** Both derivations are facts
/// about an access relation, and reconstructing an access relation here is the
/// thing this file's own contract forbids — the IR states the map, the schedule
/// verifier discharged its bijectivity, and a second opinion computed from
/// emitted statements would be a shape recognizer with none of that evidence.
/// What [`KernelEmitter::emit_binary`] checks instead is the half a backend
/// *can* check without re-deriving anything: that the minuend is the constant
/// the contract describes. That refuses a widened producer — and a swapped
/// operand pair, which is the one perturbation that turns this construct into a
/// wrapped index while leaving source the Metal compiler happily accepts.
pub(crate) fn binary_realization(op: BinaryOp) -> Result<BinaryRealization, MetalEmitError> {
    let (operator, arithmetic_type) = match op {
        BinaryOp::IndexAdd => ("+", None),
        BinaryOp::IndexMultiply => ("*", None),
        BinaryOp::IndexDivide => ("/", None),
        BinaryOp::IndexModulo => ("%", None),
        // One arm because MSL spells both differences `-` and neither weighs a
        // floating-point fact, **not** because they are one construct. The
        // signed subtraction is exact over its whole operand range; the index
        // one is unsigned and carries the non-negativity proof this function's
        // documentation derives. That difference is enforced where it is
        // observable — [`KernelEmitter::emit_binary`] annotates the index
        // difference and checks its minuend and does neither for the signed
        // one — so sharing the realization here loses nothing the emitted text
        // keeps.
        BinaryOp::IndexSubtract | BinaryOp::I32Subtract => ("-", None),
        BinaryOp::F32Add => ("+", Some(MetalFloatArithmeticType::F32)),
        BinaryOp::F32Multiply => ("*", Some(MetalFloatArithmeticType::F32)),
        // The `/` operator, deliberately, and not `metal::divide(x, y)`.
        // MSL's Table 8.1 states its accuracy against the *operator*
        // spelling; Table 6.4 defines `divide()` as "Compute x / y" and the
        // table gives it no row of its own, so lowering through the function
        // would rest the accuracy claim on a reading rather than on a
        // quotation.
        BinaryOp::F32Divide => ("/", Some(MetalFloatArithmeticType::F32)),
        // The `bfloat` operators, carrying `Bf16` rather than `F32` as the
        // arithmetic type. The two happen to share a measured subnormal
        // behaviour on the macOS row, which is exactly why the distinction
        // has to be made here rather than inferred later: a record that
        // answered `bf16` from the `f32` entry would look right on that row
        // and be a guess, and the third measured type disagrees with both.
        //
        // There is deliberately no fused form beside them. MSL provides no
        // `bfloat` overload of `fma` — the call promotes to `float` and the
        // compiler rejects the narrowing initialization — so a BF16
        // contraction has nothing to lower to at the source level, and
        // `design-the-bf16-computation-and-accumulator-contract` owns the
        // question rather than this backend inventing a promotion.
        BinaryOp::Bf16Add => ("+", Some(MetalFloatArithmeticType::Bf16)),
        BinaryOp::Bf16Multiply => ("*", Some(MetalFloatArithmeticType::Bf16)),
        BinaryOp::F32Maximum => return Ok(BinaryRealization::MaximumF32),
        _ => {
            return Err(MetalEmitError::UnsupportedOperation {
                family: MetalOperationFamily::Binary,
            });
        }
    };
    Ok(BinaryRealization::Operator {
        operator,
        arithmetic_type,
    })
}

/// Returns the constant minuend an index subtraction's contract requires.
///
/// The half of [`BinaryOp::IndexSubtract`]'s non-negativity proof a backend can
/// check without re-deriving an access relation. The contract states the minuend
/// is the constant `extent - 1`; a computed minuend is a widened producer
/// contract, and it must stop at this backend rather than emit a `uint64_t`
/// difference whose bound moved — for the same reason
/// [`staging_declaration`] refuses an address space the verifier already
/// constrains.
///
/// **This is written as a refusal and not an `expect` even though the structured
/// kernel verifier already makes it unreachable.** A body whose mirror subtracts
/// in the other order fails `tiler-ir`'s own body-refinement check, which
/// re-derives the offset expression from the region's access relation and
/// answers `KernelDiagnostic::BodyRefinement` — so the exchange never reaches
/// emission through `lower_scheduled_region`, and
/// `the_verifier_refuses_a_reordered_mirror_before_emission_sees_it` pins that.
/// What stays reachable is a producer that builds a kernel some other way, which
/// is the population this crate's refusals exist for.
///
/// It deliberately says nothing about the *subtrahend*. Bounding that is the
/// access relation's business, the schedule verifier discharged it, and the
/// emitted comment beside the difference attributes it there rather than
/// implying a test happened here.
pub(crate) fn constant_minuend(minuend: Option<KernelConstant>) -> Result<u64, MetalEmitError> {
    minuend
        .and_then(KernelConstant::as_index)
        .ok_or(MetalEmitError::MalformedKernel {
            rule: "non-constant-minuend",
        })
}

/// Per-kernel emission state.
struct KernelEmitter<'a> {
    kernel: &'a VerifiedKernel,
    target: &'a MetalTargetFacts,
    emission: MetalEmissionRealization,
    helpers: &'a mut BTreeSet<MetalHelper>,
    numerical: &'a mut BTreeSet<MetalNumericalRequirement>,
    gaps: &'a mut BTreeSet<MetalNumericalGap>,
    unstated: &'a mut BTreeSet<MetalFloatArithmeticType>,
    names: BTreeMap<VerifiedValueId, String>,
    next: u32,
    buffers: BTreeMap<VerifiedBufferId, u32>,
    extents: BTreeMap<VerifiedInputExtentId, u32>,
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

    /// Returns the argument-table index of one verified buffer handle.
    ///
    /// A lookup rather than an assignment: [`emit_entry_point`] built the table
    /// from [`VerifiedKernel::declared_buffers`] before the body was walked, so
    /// every declared handle already has its ordinal and emitting a subscript
    /// cannot invent one. Every emitted subscript and the reported binding table
    /// agree because they are the same table.
    ///
    /// `self.kernel.buffer` still runs, so a handle belonging to another kernel
    /// or naming no retained parameter is refused by the IR's own check rather
    /// than silently missing from the map.
    fn buffer_binding(&mut self, buffer: VerifiedBufferId) -> Result<u32, MetalEmitError> {
        self.kernel.buffer(buffer)?;
        self.buffers
            .get(&buffer)
            .copied()
            .ok_or(MetalEmitError::MalformedKernel {
                rule: "unresolvable-buffer-parameter",
            })
    }

    /// Returns the emitted name of one verified staging allocation.
    ///
    /// Resolved through the kernel's own handle check, so an allocation handle
    /// belonging to another kernel or naming no retained allocation is refused
    /// by the IR rather than silently formatted into a name.
    fn staging_name(&self, staging: VerifiedStagingId) -> Result<String, MetalEmitError> {
        Ok(staging_name(
            self.kernel.staging_parameter(staging)?.staging,
        ))
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
            OperationView::Unary { op, source } => self.emit_unary(op, source, &results),
            OperationView::PackedExtract {
                op,
                carrier,
                logical_index,
            } => self.emit_packed_extract(op, carrier, logical_index, &results),
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
            OperationView::InputExtent { parameter } => self.emit_input_extent(parameter, &results),
            // The phase is emitted as a comment and never as a guard. It is the
            // schedule-side *evidence* that authorizes the effect — the verifier
            // resolved it against the tile's declared staged access before this
            // kernel existed — so turning it into emitted control flow would add
            // a run-time test for a fact already proven, and one whose failure
            // would silently skip a staged write the consuming phase depends on.
            OperationView::StagedStore {
                staging,
                offset,
                value,
                phase,
            } => {
                if !results.is_empty() {
                    return Err(arity("staged-store-result"));
                }
                let allocation = self.staging_name(staging)?;
                let offset = self.name(offset)?.to_owned();
                let value = self.name(value)?.to_owned();
                let phase = phase.get();
                self.line(&format!(
                    "{allocation}[{offset}] = {value};  // tile phase {phase}"
                ));
                Ok(())
            }
            OperationView::StagedLoad {
                staging,
                offset,
                phase,
            } => {
                let [result] = results.as_slice() else {
                    return Err(arity("staged-load-result"));
                };
                let allocation = self.staging_name(staging)?;
                let offset = self.name(offset)?.to_owned();
                let phase = phase.get();
                let value_type = self.value_type(*result)?;
                let name = self.bind(*result)?;
                self.line(&format!(
                    "{value_type} {name} = {allocation}[{offset}];  // tile phase {phase}"
                ));
                Ok(())
            }
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

    /// Emits one live input-extent operand read.
    fn emit_input_extent(
        &mut self,
        parameter: VerifiedInputExtentId,
        results: &[VerifiedValueId],
    ) -> Result<(), MetalEmitError> {
        let [result] = results else {
            return Err(arity("input-extent-result"));
        };
        self.kernel.input_extent(parameter)?;
        let ordinal =
            self.extents
                .get(&parameter)
                .copied()
                .ok_or(MetalEmitError::MalformedKernel {
                    rule: "unresolvable-input-extent",
                })?;
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        self.line(&format!("{value_type} {name} = e{ordinal};"));
        Ok(())
    }

    fn loop_bound(&self, bound: LoopBound) -> Result<String, MetalEmitError> {
        match bound {
            LoopBound::Literal(value) => Ok(format!("{value}ul")),
            LoopBound::Value(id) => Ok(self.name(id)?.to_owned()),
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
        let declared = builtin_declared_type(builtin, self.emission)?;
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        // This translation unit selected the launch attribute's declared type
        // from the forms MSL 4.0 Table 5.8 admits. A narrower delivery is
        // widened exactly rather than reinterpreted.
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
            // The same rule at the narrower width, through the carrier the
            // width forces: an unsuffixed MSL integer literal is `uint` and
            // `as_type` requires equal sizes, so the pattern is narrowed to
            // `ushort` first. The declared payload is sixteen bits and is
            // carried unchanged; nothing here widens it to `f32` and rounds
            // back, which would be a different value at every pattern the two
            // roundings disagree on.
            KernelConstant::Bf16Bits(bits) => bf16_literal(bits),
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
        let (operator, arithmetic_type) = match binary_realization(op)? {
            // The one binary construct emitted as a call rather than an
            // operator, because MSL has no operator for it and the intrinsic
            // that looks like one implements a different contract. The helper
            // carries the whole derivation; the two obligations it needs are
            // recorded here.
            BinaryRealization::MaximumF32 => {
                self.helpers.insert(MetalHelper::MaximumF32);
                // A maximum performs no arithmetic and produces no new
                // subnormal, but a target that flushed a subnormal *operand*
                // before comparing would select the other one — so the input
                // half of the obligation is real and the call records it.
                self.record_subnormal_obligation(MetalFloatArithmeticType::F32);
                // `nnan` would license folding the unordered comparison the
                // NaN arm rests on. No precise-function requirement is recorded
                // beside it, deliberately: the helper calls no F32 math
                // function, so `-fmetal-math-fp32-functions` governs nothing in
                // it.
                self.numerical
                    .insert(MetalNumericalRequirement::SafeMathMode);
                let lhs = self.name(lhs)?.to_owned();
                let rhs = self.name(rhs)?.to_owned();
                let value_type = self.value_type(*result)?;
                let name = self.bind(*result)?;
                self.line(&format!(
                    "{value_type} {name} = {MAXIMUM_F32_SYMBOL}({lhs}, {rhs});"
                ));
                return Ok(());
            }
            BinaryRealization::Operator {
                operator,
                arithmetic_type,
            } => (operator, arithmetic_type),
        };
        if let Some(arithmetic_type) = arithmetic_type {
            self.record_subnormal_obligation(arithmetic_type);
        }
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
        // The same defence at the one construct whose violation is *silent*
        // rather than undefined; see [`constant_minuend`].
        let proven_non_negative = matches!(op, BinaryOp::IndexSubtract);
        if proven_non_negative {
            constant_minuend(self.kernel.value_constant(lhs)?)?;
        }
        let lhs = self.name(lhs)?.to_owned();
        let rhs = self.name(rhs)?.to_owned();
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        let statement = format!("{value_type} {name} = {lhs} {operator} {rhs};");
        if proven_non_negative {
            self.line(&format!(
                "{statement}  // unsigned; {rhs} <= {lhs} by the IR's proof, not by a test"
            ));
        } else {
            self.line(&statement);
        }
        Ok(())
    }

    /// Records any subnormal obligation this target cannot realize, or the
    /// fact that it states none for this arithmetic type.
    ///
    /// Called once per emitted floating-point arithmetic statement, with that
    /// statement's own arithmetic type. A kernel that only materializes values
    /// never reaches here, which matches the measurement: a load/store round
    /// trip returns every subnormal bit pattern unchanged, while the flush is a
    /// property of arithmetic. The target states which behaviour it has *per
    /// arithmetic type*, so the fact is looked up rather than assumed, and a
    /// type it says nothing about is recorded as unstated instead of falling
    /// back to another type's fact — the two measured types disagree, so there
    /// is no fallback that is merely less precise.
    ///
    /// Each dimension is compared independently. A target that couples input
    /// and result flushing in one execution mode does not couple the contract's
    /// semantic dimensions (ADR 0019), so a divergence on either is recorded on
    /// its own.
    fn record_subnormal_obligation(&mut self, arithmetic_type: MetalFloatArithmeticType) {
        // The declared behaviour is compared only where it is observable. A
        // kernel whose value domain keeps every operand and result of this
        // arithmetic type out of the subnormal range returns the same bits
        // under either resolution, so a target that resolves the dimension
        // differently has nothing to differ about, and recording a gap would
        // refuse a program on a difference that cannot occur.
        //
        // This consults a *discharged* obligation, not a weakened contract. The
        // kernel still declares what it means; `tiler-ir` derives the freedom
        // from the verified program it refines and refuses to let a producer
        // assert one, and the freedom is typed by arithmetic type because the
        // derivation behind it is. Nothing here can widen either.
        if self
            .kernel
            .subnormal_freedom()
            .discharges(ir_arithmetic_type(arithmetic_type))
        {
            return;
        }
        let Ok(target) = self.target.subnormal_arithmetic.behaviour(arithmetic_type) else {
            self.unstated.insert(arithmetic_type);
            return;
        };
        let realization = self.kernel.numerical();
        for mode in [realization.input_subnormals, realization.result_subnormals] {
            if let Some(gap) = subnormal_gap(mode, target.subnormal_mode()) {
                self.gaps.insert(gap);
            }
        }
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

    /// Emits one pure unary elementary function.
    ///
    /// **The namespace is written explicitly, and that is the whole point.**
    /// `exp(x)` unqualified selects `air.exp.f32` under the governed flag set and
    /// `air.fast_exp.f32` under the compiler's own default, which is fast math;
    /// `precise::exp(x)` selects `air.exp.f32` under both. Selecting the fast
    /// intrinsic to satisfy a contract stated against the precise family is the
    /// substitution ADR 0076 forbids, and the emission probe measured it to be one
    /// omitted flag away. Writing the namespace makes the flag a second line of
    /// defence rather than the only one.
    ///
    /// The match is exhaustive over a vocabulary that is deliberately not
    /// `#[non_exhaustive]`, so a second elementary function is a build error here
    /// until someone decides which intrinsic it selects.
    fn emit_unary(
        &mut self,
        op: UnaryOp,
        source: VerifiedValueId,
        results: &[VerifiedValueId],
    ) -> Result<(), MetalEmitError> {
        let [result] = results else {
            return Err(arity("unary-result"));
        };
        let (call, arithmetic_type) = match op {
            UnaryOp::F32Exp => ("precise::exp", MetalFloatArithmeticType::F32),
            // The same namespace discipline, and the hazard is the same shape:
            // `rsqrt(x)` unqualified selects `air.rsqrt.f32` under the governed
            // flag set and `air.fast_rsqrt.f32` under the compiler's own default,
            // which is fast math. The two are different *contracts* rather than
            // two speeds of one — MSL Table 8.1 states `rsqrt` correctly rounded
            // and Table 8.2 states it `<= 2 ulp` — so selecting the fast
            // intrinsic to satisfy `tiler::rms-norm-f32@1`'s faithful contract is
            // the substitution ADR 0076 forbids. There is deliberately no `sqrt`
            // emission beside it: `1 / sqrt(x)` rounds twice and is a different
            // binary32 function.
            UnaryOp::F32Rsqrt => ("precise::rsqrt", MetalFloatArithmeticType::F32),
        };
        self.record_subnormal_obligation(arithmetic_type);
        // Both requirements, and they are separate obligations. The precise
        // selection is what makes Table 8.1 the applicable accuracy table; the
        // safe math mode is what keeps INF and NaN defined, which the `-88.73`
        // band's exact negative zero rests on — under fast math §8.1 makes the
        // handling of INF undefined and that value would have no basis at all.
        self.numerical
            .insert(MetalNumericalRequirement::PreciseFp32Functions);
        self.numerical
            .insert(MetalNumericalRequirement::SafeMathMode);
        let source = self.name(source)?.to_owned();
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        self.line(&format!("{value_type} {name} = {call}({source});"));
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
                self.helpers
                    .insert(MetalHelper::CanonicalizeF32Nan { bits });
                // The emitted predicate itself is integer-only and survives any
                // math mode, but `nnan` leaves the arithmetic that produced a
                // NaN with no defined result to canonicalize. The requirement is
                // about the operand, not about the test.
                self.numerical
                    .insert(MetalNumericalRequirement::SafeMathMode);
                canonicalize_symbol(bits)
            }
            ConvertOp::CanonicalizeBf16Nan => {
                // The `bf16` sibling, and a separate arm rather than a width
                // parameter on the one above. Reaching the `f32` helper from a
                // `bfloat` value would canonicalize to a pattern no `bfloat`
                // can hold, which is the mistake the region verifier's
                // zero-extension rule exists to make checkable — so the
                // narrowing is performed here and refuses rather than reading
                // the low half of whatever the producer declared.
                let bits =
                    bf16_canonical_nan(self.kernel.numerical().canonical_arithmetic_nan_bits)?;
                self.helpers
                    .insert(MetalHelper::CanonicalizeBf16Nan { bits });
                // The same operand-side requirement as the `f32` arm: the
                // emitted predicate is integer-only and survives any math mode,
                // but `nnan` leaves the arithmetic that produced a NaN with no
                // defined result to canonicalize.
                self.numerical
                    .insert(MetalNumericalRequirement::SafeMathMode);
                canonicalize_bf16_symbol(bits)
            }
            ConvertOp::U8ToI32 | ConvertOp::I32ToF32 => msl_type(op.result_type())?.to_owned(),
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

    /// Extracts one logical unsigned four-bit code from its byte carrier.
    fn emit_packed_extract(
        &mut self,
        op: PackedExtractOp,
        carrier: VerifiedValueId,
        logical_index: VerifiedValueId,
        results: &[VerifiedValueId],
    ) -> Result<(), MetalEmitError> {
        let [result] = results else {
            return Err(arity("packed-extract-result"));
        };
        let expression = match op {
            PackedExtractOp::U4LsbZeroTail => {
                let carrier = self.name(carrier)?.to_owned();
                let logical_index = self.name(logical_index)?.to_owned();
                format!("uchar(({carrier} >> (({logical_index} & 1ul) * 4ul)) & 0x0fu)")
            }
        };
        let value_type = self.value_type(*result)?;
        let name = self.bind(*result)?;
        self.line(&format!("{value_type} {name} = {expression};"));
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

        let start = self.loop_bound(loop_ref.start_bound())?;
        let end = self.loop_bound(loop_ref.end_bound())?;
        let comment_bound = |bound: LoopBound, rendered: &str| match bound {
            LoopBound::Literal(value) => value.to_string(),
            LoopBound::Value(_) => rendered.to_owned(),
        };
        self.line(&format!(
            "// serial loop over [{}, {})",
            comment_bound(loop_ref.start_bound(), &start),
            comment_bound(loop_ref.end_bound(), &end)
        ));
        let induction_type = self.value_type(induction)?;
        let induction_name = self.bind(induction)?;
        self.line(&format!("{induction_type} {induction_name} = {start};"));
        let mut accumulator_names = Vec::with_capacity(accumulators.len());
        for (accumulator, seed) in accumulators.iter().zip(&initial) {
            let seed = self.name(*seed)?.to_owned();
            let value_type = self.value_type(*accumulator)?;
            let name = self.bind(*accumulator)?;
            self.line(&format!("{value_type} {name} = {seed};"));
            accumulator_names.push(name);
        }

        self.line(&format!(
            "for (; {induction_name} < {end}; {induction_name} = {induction_name} + 1ul) {{"
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

/// One governed barrier specification Metal realizes, and how it spells it.
///
/// The builtin and its ordered fence flags, held apart from the statement text
/// so that [`barrier_realization`]'s decision is usable by a caller that emits
/// nothing.
pub(crate) struct RealizedBarrier {
    /// The MSL barrier builtin establishing this specification's visibility.
    builtin: &'static str,
    /// The memory-fence flags, in `AddressSpace` order and deduplicated.
    flags: Vec<&'static str>,
}

/// Decides whether Metal realizes one governed barrier specification.
///
/// **The single authority on what this backend's barrier vocabulary delivers**,
/// and that is why it is split out of [`barrier_call`] rather than left inside
/// it. Emission is no longer the only caller that has to know: a delivered
/// artifact states the synchronization realization each of its entries requires,
/// and [`crate::synchronization_requirement`] refuses one this backend cannot
/// deliver *before the routing commit* — on a host that emitted none of these
/// bytes and has no MSL to produce. Deciding that from a second table would let
/// the two answers disagree, and the disagreement is silent in the direction
/// that matters: a delivery-time check admitting more than emission does would
/// pass a route whose kernel could never have been written.
///
/// It returns a [`BarrierRejection`] rather than a [`MetalEmitError`] because the
/// refusal is a property of the specification, not of an emission that failed.
/// The emitting caller wraps it; the preflight caller carries it whole.
pub(crate) fn barrier_realization(spec: &BarrierSpec) -> Result<RealizedBarrier, BarrierRejection> {
    let builtin = match spec.execution_scope {
        ExecutionScope::Workgroup => "threadgroup_barrier",
        ExecutionScope::Subgroup => "simdgroup_barrier",
        _ => {
            return Err(BarrierRejection::ExecutionScope {
                scope: spec.execution_scope,
            });
        }
    };
    // Metal couples visibility to the barrier builtin: `threadgroup_barrier`
    // establishes workgroup visibility and `simdgroup_barrier` establishes
    // SIMD-group visibility. No in-kernel barrier establishes device-wide
    // visibility, and the governed memory scopes cannot name SIMD-group
    // visibility at all, so a SIMD-group barrier has no admissible scope here.
    //
    // Both scope vocabularies are `#[non_exhaustive]`, so the wildcard is
    // required out of crate and a widened scope would reach it silently, at run
    // time. The build error that announces such a widening is therefore in
    // `tiler-ir` rather than here — `barrier_scope_vocabulary_is_closed` — and
    // `add-subgroup-memory-scope-when-collectives-land` owns the arm that would
    // then be added below.
    match (spec.execution_scope, spec.memory_scope) {
        (ExecutionScope::Workgroup, MemoryScope::Workgroup) => {}
        _ => {
            return Err(BarrierRejection::MemoryVisibility {
                execution: spec.execution_scope,
                memory: spec.memory_scope,
            });
        }
    }
    match spec.ordering {
        // A Metal barrier is a full acquire-release fence over the flagged
        // address spaces; no weaker or stronger ordering is expressible.
        BarrierOrdering::AcquireRelease => {}
        _ => {
            return Err(BarrierRejection::Ordering {
                ordering: spec.ordering,
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
    Ok(RealizedBarrier { builtin, flags })
}

/// Returns the MSL barrier statement realizing one governed specification.
pub(crate) fn barrier_call(spec: &BarrierSpec) -> Result<String, MetalEmitError> {
    let RealizedBarrier { builtin, flags } = barrier_realization(spec)
        .map_err(|reason| MetalEmitError::UnsupportedBarrier { reason })?;
    let joined = flags.join(" | ");
    let flags = if joined.is_empty() {
        "mem_flags::mem_none"
    } else {
        joined.as_str()
    };
    Ok(format!("{builtin}({flags});"))
}

/// Returns the Metal memory-fence flag for one governed address space.
fn fence_flag(space: AddressSpace) -> Result<&'static str, BarrierRejection> {
    match space {
        AddressSpace::Device => Ok("mem_flags::mem_device"),
        AddressSpace::Workgroup => Ok("mem_flags::mem_threadgroup"),
        // Constant memory is read-only for the dispatch and invocation-private
        // memory is visible to one invocation, so neither has a fence flag.
        // Dropping them silently would lose the specification.
        _ => Err(BarrierRejection::FencedSpace { space }),
    }
}

/// Returns the malformed-kernel rejection for one violated arity rule.
const fn arity(rule: &'static str) -> MetalEmitError {
    MetalEmitError::MalformedKernel { rule }
}
