//! The governed pure-BF16 constant, multiply, and add family.
//!
//! **Why these are new keys rather than widened ones.** Operand type is part of
//! an operation's identity under ADR 0026, so `tiler::multiply-bf16@1` sits
//! *beside* `tiler::multiply-f32@1` and neither admits the other's operand. The
//! `bf16` in a key here is the subject, not a parameter: widening
//! `tiler::multiply-f32@1` to accept a BF16 operand would have made one key mean
//! two arithmetics whose roundings differ.
//!
//! **What "pure BF16" fixes.** Every one of the four type facts each definition
//! carries — computation, accumulator, intermediate materialization, and result
//! — resolves to `tiler::bf16@1`, and they are four separate fields rather than
//! one. A later F32 accumulator is then an explicit edit to
//! [`BF16_FACT_ACCUMULATOR_TYPE`]'s value, which moves the registry snapshot and
//! every identity derived from it, instead of the silent removal of an
//! assumption nothing wrote down.
//!
//! **What this family deliberately does not admit.** No fused multiply-add, no
//! ADR 0015 arithmetic contraction, no reassociation, no mixed precision, and no
//! implicit promotion. The first four are stated as `false` facts and the last
//! two are also *refusals* this module's inferencer raises by name, because an
//! operand type is checkable at application time while a rewrite permission is
//! not. `design-the-bf16-computation-and-accumulator-contract` owns whether any
//! of them is ever admitted; until it does, a missing algebraic capability here
//! is unknown rather than evidence of the inverse law.
//!
//! **The FMA absence is a target fact, not only a scope choice.** The BF16
//! second-dtype spike records `metal` rejecting `fma(bfloat, bfloat, bfloat)`
//! outright, so a pure-BF16 fused operation has no primitive to lower to and
//! there is nothing to contract. That is why it is absent rather than registered
//! and disabled.
//!
//! **Subnormal preservation here is semantics, never a target claim.** These
//! definitions state that the BF16 value set's subnormals participate; whether a
//! given target flushes them is a numerical-honourability fact of that target's
//! profile row, and putting it here would make a target fact travel with
//! consumer-neutral semantics.
//!
//! **No conversion is admitted, deliberately.** Nothing here converts between
//! BF16 and F32 in either direction. A program needing one blocks on the
//! conversion family rather than acquiring an implicit promotion from this
//! module.

use std::sync::Arc;

use crate::shape::Shape;

use super::registry::standard_conformance;
use super::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueKind, CanonicalValueView,
    NormativeDefinitionRef, OpKey, OperationArity, OperationAttributeSchema, OperationDefinition,
    OperationDefinitionFacts, OperationEffect, OperationInferenceError, OperationInferenceOutputs,
    OperationInferenceRequest, OperationInferencer, OperationSchema, ProviderDiagnosticCode,
    RegistryError, ResolvedValueType, SCALAR_TYPE_FACT_WIDTH_BITS, SemanticRegistryRegistrar,
    TypeKey, ValueFact, ValueTypeMarker, builtin_scalar_value_type_facts,
};

/// The bounded profile's canonical quiet NaN produced by BF16 arithmetic.
///
/// BF16 shares binary32's sign and exponent fields, so this is the leading
/// sixteen bits of [`CANONICAL_F32_ARITHMETIC_NAN_BITS`] — the quiet bit set and
/// no payload. It is written as its own constant rather than derived from that
/// one because the two are facts about two different formats that happen to
/// agree, and a future format whose quiet bit sits elsewhere must not silently
/// inherit this value.
///
/// [`CANONICAL_F32_ARITHMETIC_NAN_BITS`]: super::CANONICAL_F32_ARITHMETIC_NAN_BITS
pub const CANONICAL_BF16_ARITHMETIC_NAN_BITS: u16 = 0x7fc0;

/// Stable field ID carrying exact BF16 bits on the governed BF16 constant.
///
/// Record-local, like every attribute field ID: sharing the integer with
/// [`F32_CONSTANT_BITS_ATTRIBUTE`] relates the two records in no way, and a
/// payload declaring the binary32 format is refused here rather than reinterpreted.
///
/// [`F32_CONSTANT_BITS_ATTRIBUTE`]: super::F32_CONSTANT_BITS_ATTRIBUTE
pub const BF16_CONSTANT_BITS_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// # The BF16 family's fact-field vocabulary
///
/// One numbering shared by all three BF16 operation records, so a consumer reads
/// the accumulator type of a constant and of a multiply through the same field
/// ID. Field IDs remain record-local in general — these agree because they are
/// deliberately one vocabulary, not because equal integers are ever normalized.
///
/// Fields 1 through 8 and 10 are unconditional on all three definitions. Fields
/// 9 and 11 through 15 are conditional on the two *arithmetic* definitions: a
/// zero-operand constant installs no arithmetic NaN and has no operand pair to
/// promote, no adjacent rounding to contract, and no contributors to regroup, so
/// a `false` there would claim a permission exists and is withheld. Field 16 is
/// conditional on the constant. Absence of an unconditional field is a malformed
/// record rather than a default.
/// Fact field naming the type the operation's arithmetic is performed at.
pub const BF16_FACT_COMPUTATION_TYPE: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming the type each accumulation step is performed at.
pub const BF16_FACT_ACCUMULATOR_TYPE: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming the type an observable intermediate is materialized at.
pub const BF16_FACT_INTERMEDIATE_MATERIALIZATION_TYPE: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field naming the result value type.
pub const BF16_FACT_RESULT_TYPE: AttributeFieldId = AttributeFieldId::new(4);
/// Fact field naming the rounding rule applied at every observable materialization.
pub const BF16_FACT_ROUNDING: AttributeFieldId = AttributeFieldId::new(5);
/// Fact field naming the operation's behaviour on BF16 subnormals.
pub const BF16_FACT_SUBNORMALS: AttributeFieldId = AttributeFieldId::new(6);
/// Fact field naming the operation's behaviour on signed zero.
pub const BF16_FACT_SIGNED_ZERO: AttributeFieldId = AttributeFieldId::new(7);
/// Fact field naming the operation's NaN behaviour.
pub const BF16_FACT_NAN_BEHAVIOUR: AttributeFieldId = AttributeFieldId::new(8);
/// Fact field carrying the canonical arithmetic-NaN payload the arithmetic installs.
///
/// Conditional on an arithmetic definition. The constant installs no NaN: it
/// preserves the payload it was given, which [`BF16_FACT_NAN_BEHAVIOUR`] states.
pub const BF16_FACT_CANONICAL_NAN_BITS: AttributeFieldId = AttributeFieldId::new(9);
/// Fact field naming the operation's infinity and overflow behaviour.
pub const BF16_FACT_INFINITY_AND_OVERFLOW: AttributeFieldId = AttributeFieldId::new(10);
/// Fact field stating whether operands of differing precision may be combined.
///
/// Conditional on an arithmetic definition.
pub const BF16_FACT_MIXED_PRECISION_PERMITTED: AttributeFieldId = AttributeFieldId::new(11);
/// Fact field stating whether an operand may be implicitly promoted to another type.
///
/// Conditional on an arithmetic definition.
pub const BF16_FACT_IMPLICIT_PROMOTION_PERMITTED: AttributeFieldId = AttributeFieldId::new(12);
/// Fact field stating whether ADR 0015's multiply-add fusion is permitted.
///
/// Conditional on an arithmetic definition.
pub const BF16_FACT_ARITHMETIC_CONTRACTION_PERMITTED: AttributeFieldId = AttributeFieldId::new(13);
/// Fact field stating whether a fused multiply-add may realize this operation.
///
/// Conditional on an arithmetic definition. Distinct from
/// [`BF16_FACT_ARITHMETIC_CONTRACTION_PERMITTED`]: that one is the permission to
/// fuse *this* operation with an adjacent one, while this is the permission to
/// realize it through a fused primitive at all. Both are `false`, and the second
/// has no BF16 primitive behind it to permit.
pub const BF16_FACT_FUSED_MULTIPLY_ADD_PERMITTED: AttributeFieldId = AttributeFieldId::new(14);
/// Fact field stating whether contributors may be regrouped.
///
/// Conditional on an arithmetic definition.
pub const BF16_FACT_REASSOCIATION_PERMITTED: AttributeFieldId = AttributeFieldId::new(15);
/// Fact field naming how the BF16 constant treats its declared payload.
///
/// Conditional on the constant definition.
pub const BF16_CONSTANT_FACT_PAYLOAD_RULE: AttributeFieldId = AttributeFieldId::new(16);

/// Governed Rust marker for BF16 values.
pub enum Bf16 {}

impl ValueTypeMarker for Bf16 {}

impl Bf16 {
    /// Returns the governed complete BF16 semantic identity.
    ///
    /// # Panics
    ///
    /// Panics only if Tiler's compile-time governed key violates its own
    /// canonical identity grammar.
    #[must_use]
    pub fn resolved_type() -> ResolvedValueType {
        ResolvedValueType::nominal(bf16_type_key())
    }
}

/// Returns the governed scalar-BF16 constant operation key.
#[must_use]
pub fn constant_bf16_op() -> OpKey {
    governed_op("constant-bf16")
}

/// Returns the governed elementwise-BF16 multiplication operation key.
#[must_use]
pub fn multiply_bf16_op() -> OpKey {
    governed_op("multiply-bf16")
}

/// Returns the governed elementwise-BF16 addition operation key.
#[must_use]
pub fn add_bf16_op() -> OpKey {
    governed_op("add-bf16")
}

/// Returns the exact fact record the governed BF16 constant carries.
///
/// Built by the same constructor the registration uses rather than restated, so
/// a consumer parameterizing itself on the declared record and the registered
/// definition cannot disagree about what was declared.
///
/// # Panics
///
/// Panics only if this crate's own compile-time fact record violates the
/// canonical value grammar, which registration would reject as well.
#[must_use]
pub fn constant_bf16_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(BF16_FACT_COMPUTATION_TYPE, bf16_value_type()),
        CanonicalField::new(BF16_FACT_ACCUMULATOR_TYPE, bf16_value_type()),
        CanonicalField::new(
            BF16_FACT_INTERMEDIATE_MATERIALIZATION_TYPE,
            bf16_value_type(),
        ),
        CanonicalField::new(BF16_FACT_RESULT_TYPE, bf16_value_type()),
        CanonicalField::new(
            BF16_FACT_ROUNDING,
            fact("none-the-declared-payload-is-already-the-exact-bf16-encoding"),
        ),
        CanonicalField::new(
            BF16_FACT_SUBNORMALS,
            fact("preserved-every-subnormal-encoding-denotes-a-distinct-constant"),
        ),
        CanonicalField::new(
            BF16_FACT_SIGNED_ZERO,
            fact("preserved-negative-zero-and-positive-zero-are-distinct-constants"),
        ),
        CanonicalField::new(
            BF16_FACT_NAN_BEHAVIOUR,
            fact("preserved-exactly-the-declared-payload-is-not-canonicalized"),
        ),
        CanonicalField::new(
            BF16_FACT_INFINITY_AND_OVERFLOW,
            fact("preserved-both-infinity-encodings-denote-constants-and-no-overflow-arises"),
        ),
        CanonicalField::new(BF16_CONSTANT_FACT_PAYLOAD_RULE, fact("exact-bf16-bits")),
    ])
    .expect("the governed bf16 constant facts are canonical")
}

/// Returns the exact fact record the governed BF16 multiply and add both carry.
///
/// One record for both keys: the two operations differ in which arithmetic they
/// name, and in nothing this record states. Built by the same constructor the
/// registration uses rather than restated.
///
/// # Panics
///
/// Panics only if this crate's own compile-time fact record violates the
/// canonical value grammar, which registration would reject as well.
#[must_use]
pub fn arithmetic_bf16_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(BF16_FACT_COMPUTATION_TYPE, bf16_value_type()),
        CanonicalField::new(BF16_FACT_ACCUMULATOR_TYPE, bf16_value_type()),
        CanonicalField::new(
            BF16_FACT_INTERMEDIATE_MATERIALIZATION_TYPE,
            bf16_value_type(),
        ),
        CanonicalField::new(BF16_FACT_RESULT_TYPE, bf16_value_type()),
        CanonicalField::new(
            BF16_FACT_ROUNDING,
            fact("bf16-round-to-nearest-ties-to-even-at-every-observable-materialization"),
        ),
        CanonicalField::new(
            BF16_FACT_SUBNORMALS,
            fact("preserved-operands-and-results-in-the-bf16-subnormal-range-are-not-flushed"),
        ),
        CanonicalField::new(
            BF16_FACT_SIGNED_ZERO,
            fact("ieee-754-signed-zero-rules-over-the-bf16-value-set"),
        ),
        CanonicalField::new(
            BF16_FACT_NAN_BEHAVIOUR,
            fact("quiet-nan-propagates-and-every-arithmetic-nan-result-is-canonicalized"),
        ),
        CanonicalField::new(
            BF16_FACT_CANONICAL_NAN_BITS,
            canonical_bf16_bits(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
        ),
        CanonicalField::new(
            BF16_FACT_INFINITY_AND_OVERFLOW,
            fact("ieee-754-infinity-rules-and-overflow-rounds-to-infinity-under-ties-to-even"),
        ),
        CanonicalField::new(
            BF16_FACT_MIXED_PRECISION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            BF16_FACT_IMPLICIT_PROMOTION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            BF16_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            BF16_FACT_FUSED_MULTIPLY_ADD_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            BF16_FACT_REASSOCIATION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
    ])
    .expect("the governed bf16 arithmetic facts are canonical")
}

/// Encodes one exact BF16 payload as a canonical value.
///
/// Big-endian, like every governed float payload, and tagged with the
/// `tiler::bf16@1` format key so a binary32 payload of the same byte count could
/// not be mistaken for one of these.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed BF16 key violates its own
/// canonical identity grammar. The two-byte payload is always in bounds.
#[must_use]
pub fn canonical_bf16_bits(bits: u16) -> CanonicalValue {
    CanonicalValue::float_bits(bf16_type_key(), bits.to_be_bytes())
        .expect("bf16 has a nonempty bounded payload")
}

pub(super) fn register_standard_bf16(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    // `tiler::bf16@1` is a governed catalog row registered by
    // `catalog::register_builtin_dtype_catalog`; this module binds the Rust
    // marker a typed BF16 authoring handle resolves through, and registers the
    // three operations that give that identity its first executable meaning.
    registrar.bind_marker::<Bf16>(Bf16::resolved_type())?;
    // The payload width comes from the registered descriptor rather than from a
    // literal here, so the constant's validation and the catalog row cannot
    // drift apart: a descriptor that no longer says sixteen bits fails
    // registration instead of silently admitting a payload of the old width.
    let payload_bytes = registered_bf16_payload_bytes();
    registrar.register_operation(OperationDefinition::new(
        constant_bf16_op(),
        exact_schema(
            0,
            1,
            [OperationAttributeSchema::required(
                BF16_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValueKind::FloatBits,
            )],
        ),
        NormativeDefinitionRef::new(
            "tiler::constant-bf16@1; exact payload in the ratified RISC-V BF16 operand format; source riscv-unprivileged-isa-20260120; tiler::bf16@1",
        )?,
        OperationDefinitionFacts::new(constant_bf16_facts()),
        standard_conformance("constant-bf16"),
        OperationEffect::Pure,
        Arc::new(ConstantBf16 { payload_bytes }),
    ))?;
    registrar.register_operation(OperationDefinition::new(
        multiply_bf16_op(),
        exact_schema(2, 1, []),
        NormativeDefinitionRef::new(
            "tiler::multiply-bf16@1; separate multiplication over the ratified RISC-V BF16 operand format; source riscv-unprivileged-isa-20260120; tiler::bf16@1",
        )?,
        OperationDefinitionFacts::new(arithmetic_bf16_facts()),
        standard_conformance("multiply-bf16"),
        OperationEffect::Pure,
        Arc::new(BinaryBf16),
    ))?;
    registrar.register_operation(OperationDefinition::new(
        add_bf16_op(),
        exact_schema(2, 1, []),
        NormativeDefinitionRef::new(
            "tiler::add-bf16@1; separate addition over the ratified RISC-V BF16 operand format; source riscv-unprivileged-isa-20260120; tiler::bf16@1",
        )?,
        OperationDefinitionFacts::new(arithmetic_bf16_facts()),
        standard_conformance("add-bf16"),
        OperationEffect::Pure,
        Arc::new(BinaryBf16),
    ))
    // No algebraic capability is declared on either arithmetic, deliberately.
    // `tiler::add-f32@1` declares ordered associativity; this family withholds
    // it because `BF16_FACT_REASSOCIATION_PERMITTED` is `false` and declaring
    // the law would hand a rewrite the fact forbids. A missing declaration reads
    // as unknown rather than as the inverse law, which is what leaves the
    // question open for the computation-and-accumulator contract to settle.
}

fn exact_schema<const N: usize>(
    operands: u32,
    results: u32,
    attributes: [OperationAttributeSchema; N],
) -> OperationSchema {
    OperationSchema::new(
        OperationArity::exact(operands),
        OperationArity::exact(results),
        attributes,
    )
    .expect("the governed bf16 operation schema is valid")
}

fn bf16_type_key() -> TypeKey {
    TypeKey::new("tiler", "bf16", 1).expect("the governed BF16 key is valid")
}

fn bf16_value_type() -> CanonicalValue {
    CanonicalValue::value_type(Bf16::resolved_type())
}

fn fact(value: &'static str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("the governed bf16 fact is bounded")
}

fn governed_op(name: &str) -> OpKey {
    OpKey::new("tiler", name, 1).expect("the governed BF16 operation key is valid")
}

/// Returns the BF16 payload width in bytes, read from the registered descriptor.
///
/// The **one** derivation of this width. The index layer's `bf16` scalar
/// constant validates its payload against this same function rather than a
/// literal of its own, so the semantic operation and the scalar it lowers to
/// cannot come to disagree about how wide a `bf16` payload is.
///
/// # Panics
///
/// Panics when the governed catalog does not describe `tiler::bf16@1` with a
/// whole-byte width. Both are defects in this crate's own catalog rather than
/// consumer input, and a registration that proceeded past either would validate
/// constant payloads against a width no registered identity claims.
pub(crate) fn registered_bf16_payload_bytes() -> usize {
    let facts = builtin_scalar_value_type_facts(&Bf16::resolved_type())
        .expect("the governed catalog describes tiler::bf16@1");
    let CanonicalValueView::Record(fields) = facts.view() else {
        unreachable!("a governed scalar descriptor is a record")
    };
    let width_bits = fields
        .iter()
        .find(|field| field.id() == SCALAR_TYPE_FACT_WIDTH_BITS)
        .map(CanonicalField::value)
        .and_then(|value| match value.view() {
            CanonicalValueView::Unsigned { bits, .. } => Some(bits),
            _ => None,
        })
        .expect("the governed bf16 descriptor states an unsigned width in bits");
    assert!(
        width_bits > 0 && width_bits.is_multiple_of(8),
        "the governed bf16 descriptor states a whole-byte width, found {width_bits} bits"
    );
    usize::try_from(width_bits / 8).expect("a governed scalar width fits the host")
}

struct ConstantBf16 {
    payload_bytes: usize,
}

impl OperationInferencer for ConstantBf16 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if !operands.is_empty() {
            return Err(op_error(
                "bf16.constant.arity",
                "the bf16 constant requires no operands",
            ));
        }
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "bf16.constant.attributes",
                "the bf16 constant requires exactly the bits attribute",
            ));
        }
        let Some(CanonicalValueView::FloatBits(bits)) = attributes
            .get(BF16_CONSTANT_BITS_ATTRIBUTE)
            .map(CanonicalValue::view)
        else {
            return Err(op_error(
                "bf16.constant.bits.kind",
                "bf16 constant bits must be exact FloatBits",
            ));
        };
        // Format first, then width. A binary32 payload fails on the format, and
        // a payload wearing the bf16 format at another width fails on the
        // width — two distinct refusals, so neither check hides the other.
        if bits.format() != &bf16_type_key() {
            return Err(op_error(
                "bf16.constant.bits.format",
                "bf16 constant bits must declare the tiler::bf16@1 format; no other float format is promoted into it",
            ));
        }
        if bits.bits().len() != self.payload_bytes {
            return Err(op_error(
                "bf16.constant.bits.width",
                "bf16 constant bits must have the width the registered bf16 descriptor states",
            ));
        }
        outputs.try_push(ValueFact::new(Bf16::resolved_type(), Shape::new([])))
    }
}

struct BinaryBf16;

impl OperationInferencer for BinaryBf16 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if operands.len() != 2 {
            return Err(op_error(
                "bf16.binary.arity",
                "a bf16 binary operation requires two operands",
            ));
        }
        if !attributes.fields().is_empty() {
            return Err(op_error(
                "bf16.binary.attributes",
                "a bf16 binary operation has no attributes",
            ));
        }
        let expected = Bf16::resolved_type();
        // The two rejections are separate names because they are separate
        // defects: one operand of another type is a mixed-precision application,
        // and two operands agreeing on another type is a request to promote the
        // whole application into bf16. Neither is admitted, and a caller has to
        // be able to tell which it wrote.
        if operands[0].resolved_type() != operands[1].resolved_type() {
            return Err(op_error(
                "bf16.binary.mixed-precision",
                "a bf16 binary operation admits no mixed-precision operand pair; both operands must be tiler::bf16@1",
            ));
        }
        if operands[0].resolved_type() != &expected {
            return Err(op_error(
                "bf16.binary.implicit-promotion",
                "a bf16 binary operation admits no implicit promotion; an operand of another type is not converted to tiler::bf16@1",
            ));
        }
        let left = operands[0].shape();
        let right = operands[1].shape();
        let shape = if left.rank() == 0 {
            right.clone()
        } else if right.rank() == 0 || left == right {
            left.clone()
        } else {
            return Err(op_error(
                "bf16.binary.shape",
                "operand shapes must match or one operand must be scalar",
            ));
        };
        outputs.try_push(ValueFact::new(expected, shape))
    }
}

fn op_error(code: &str, message: &str) -> OperationInferenceError {
    OperationInferenceError::new(
        ProviderDiagnosticCode::new(code).expect("the governed bf16 diagnostic code is canonical"),
        message,
    )
    .expect("the governed bf16 diagnostic message is canonical")
}

#[cfg(test)]
mod tests;
