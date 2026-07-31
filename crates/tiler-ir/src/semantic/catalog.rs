//! The governed built-in dtype identity catalog.
//!
//! This module is the single source of the built-in identities accepted by
//! ADRs 0027, 0028, and 0034 through 0038. Every governed nominal scalar, the
//! parameterized complex family, and the OCP microscaling scheme identities are
//! constructed from the tables below, so a row is the only place a canonical
//! key, its descriptor, and its normative reference can be stated. A second
//! construction site would let two spellings of one identity drift apart, which
//! is the failure ADR 0034's immutable-descriptor rule exists to prevent.
//!
//! **Recognition, and nothing further.** A row here establishes that Tiler
//! knows an exact logical value identity and can carry it through canonical IR.
//! It creates no operation signature, reference evaluator, storage carrier,
//! kernel type, target dispatch fact, or backend lowering; ADR 0026 keeps every
//! one of those a separate capability that still fails closed. Registering an
//! identity therefore *widens what can be named and rejected explainably*, not
//! what can be executed.
//!
//! **What a descriptor states, and what it defers.** Each definition carries
//! the static parameters Tiler's accepted contracts and preserved primary
//! sources fix for that format, plus a mandatory normative-definition reference
//! naming the authority that owns the complete value set. Where no available
//! evidence fixes a parameter, the field is absent rather than guessed, and the
//! field's own documentation states that condition — an absent conditional
//! field means "Tiler's evidence does not fix this", never "this format has
//! none". The exponent bias is the one such field today.

use std::sync::Arc;

use super::operation::{
    SCALAR_TYPE_FACT_ALIAS_POLICY, SCALAR_TYPE_FACT_BLOCK_SIZE, SCALAR_TYPE_FACT_CLASS,
    SCALAR_TYPE_FACT_COEFFICIENT_DIGITS, SCALAR_TYPE_FACT_COMPONENT_ORDER,
    SCALAR_TYPE_FACT_COMPONENT_TYPES, SCALAR_TYPE_FACT_EXPONENT_BIAS,
    SCALAR_TYPE_FACT_EXPONENT_BITS, SCALAR_TYPE_FACT_HAS_INFINITIES, SCALAR_TYPE_FACT_HAS_NAN,
    SCALAR_TYPE_FACT_HAS_SIGNED_ZERO, SCALAR_TYPE_FACT_HAS_SUBNORMALS, SCALAR_TYPE_FACT_HAS_ZERO,
    SCALAR_TYPE_FACT_SCALE_SELECTION, SCALAR_TYPE_FACT_SIGN_BITS,
    SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS, SCALAR_TYPE_FACT_VALUE_CARDINALITY,
    SCALAR_TYPE_FACT_WIDTH_BITS,
};
use super::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueView, NormativeDefinitionRef,
    ProviderDiagnosticCode, QuantSchemeKey, RegistryError, ResolvedValueType,
    SemanticRegistryRegistrar, TypeArguments, TypeDefinitionFacts, TypeIdentityError,
    TypeInstanceError, TypeKey, ValueTypeDefinition, ValueTypeDefinitionKey,
    ValueTypeInstanceValidator,
};

/// The uniform alias and equivalence policy every governed built-in carries.
///
/// One string rather than a per-row policy because ADRs 0027 and 0034 state one
/// rule for the whole catalog: a frontend spelling resolves to exactly one
/// canonical key before semantic admission and never becomes an identity, and
/// an external identity becomes equivalent only on exact versioned bit/value and
/// conversion conformance evidence. A per-row restatement would be the same
/// sentence copied thirty times, which is how one copy silently becomes wrong.
const ALIAS_AND_EQUIVALENCE_POLICY: &str = "aliases-resolve-to-this-key-before-admission; external-equivalence-requires-versioned-conformance-evidence";

/// Semantic version shared by every identity in the initial built-in catalog.
const BUILT_IN_SEMANTIC_VERSION: u32 = 1;

/// Number of element codes sharing one scale in every OCP MX v1.0 block.
const MICROSCALING_BLOCK_SIZE: u32 = 32;

/// The OCP MX scale-data element type shared by every admitted MX scheme.
const MICROSCALING_SCALE_TYPE: &str = "f8e8m0fnu";

/// How an MX block selects the scale that applies to its element codes.
const MICROSCALING_SCALE_SELECTION: &str =
    "one-shared-scale-per-contiguous-block-of-32-element-codes";

/// The ordered component roles an MX compound value associates.
const MICROSCALING_COMPONENT_ORDER: &str = "ordered-element-codes-then-block-scale";

/// The ordered component roles a complex value associates.
const COMPLEX_COMPONENT_ORDER: &str = "ordered-real-then-imaginary";

/// Which governed family one built-in scalar row belongs to.
enum ScalarKind {
    /// A two-valued logical predicate, deliberately distinct from integer `i1`.
    ///
    /// ADR 0028 admits `bool` as a logical value with two members and leaves
    /// bit-, byte-, and other ABI-sized representations to physical storage, so
    /// this row states a cardinality and no logical width.
    Predicate,
    /// An exact-width two's-complement or unsigned logical integer.
    ///
    /// The code domain is deliberately not a descriptor field: it is exactly
    /// determined by the class and the width, and a stored copy of a derived
    /// value is a second place for it to be wrong.
    Integer {
        /// `true` for two's-complement signed, `false` for unsigned.
        signed: bool,
        /// Logical value width in bits.
        width_bits: u32,
    },
    /// A binary floating-point value set.
    BinaryFloat(BinaryFloat),
    /// An IEEE decimal interchange format.
    ///
    /// DPD and BID encode the same logical format and are storage-encoding
    /// identities under ADR 0035, so neither appears here.
    DecimalFloat {
        /// Interchange width in bits, shared by both IEEE encodings.
        width_bits: u32,
        /// Precision in decimal digits of the coefficient.
        coefficient_digits: u32,
    },
}

/// The complete static parameters of one binary floating-point value set.
struct BinaryFloat {
    /// Descriptor class naming the governing convention.
    class: &'static str,
    /// Total encoded width in bits.
    width_bits: u32,
    /// Sign-field width; zero for unsigned exponent-only scale data.
    sign_bits: u32,
    /// Exponent-field width.
    exponent_bits: u32,
    /// Stored fraction width, excluding any implicit leading significand bit.
    trailing_significand_bits: u32,
    /// Exponent bias, where Tiler's evidence fixes one.
    ///
    /// `None` is an evidence boundary, not a claim that the format has no bias:
    /// the IEEE 754-2019 and RISC-V BF16 rows resolve their bias through the
    /// pinned normative reference, whose bytes this repository either does not
    /// vendor (`ieee-754-2019` is metadata-only) or does not re-derive here.
    /// The OCP rows carry a bias because the vendored
    /// `mlir-builtin-types-llvmorg-22.1.8` states each one exactly, and the
    /// mature dtype taxonomy tabulates the same values.
    exponent_bias: Option<i32>,
    /// Exactly the special members this format's value set contains.
    special_values: &'static [SpecialValue],
}

/// One special member a binary floating-point value set may contain.
///
/// A row lists the members its format has, rather than carrying one Boolean per
/// member: a list cannot be transposed by editing the wrong line, and a member
/// added to [`SPECIAL_VALUE_FIELDS`] is then an explicit `false` on every row
/// that does not name it instead of a field silently defaulting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpecialValue {
    /// Signed infinities.
    Infinities,
    /// Not-a-number encodings.
    Nan,
    /// A zero value.
    Zero,
    /// A zero that carries a sign.
    SignedZero,
    /// Subnormal finite values below the smallest normal magnitude.
    Subnormals,
}

/// Each special member and the descriptor field that reports its presence.
const SPECIAL_VALUE_FIELDS: [(SpecialValue, AttributeFieldId); 5] = [
    (SpecialValue::Infinities, SCALAR_TYPE_FACT_HAS_INFINITIES),
    (SpecialValue::Nan, SCALAR_TYPE_FACT_HAS_NAN),
    (SpecialValue::Zero, SCALAR_TYPE_FACT_HAS_ZERO),
    (SpecialValue::SignedZero, SCALAR_TYPE_FACT_HAS_SIGNED_ZERO),
    (SpecialValue::Subnormals, SCALAR_TYPE_FACT_HAS_SUBNORMALS),
];

/// The complete IEEE 754 binary interchange special-value set.
const IEEE_BINARY_SPECIAL_VALUES: &[SpecialValue] = &[
    SpecialValue::Infinities,
    SpecialValue::Nan,
    SpecialValue::Zero,
    SpecialValue::SignedZero,
    SpecialValue::Subnormals,
];

/// A finite-only signed set that still encodes NaN, as OCP E4M3FN does.
const FINITE_WITH_NAN_SPECIAL_VALUES: &[SpecialValue] = &[
    SpecialValue::Nan,
    SpecialValue::Zero,
    SpecialValue::SignedZero,
    SpecialValue::Subnormals,
];

/// A finite-only signed set with neither infinity nor NaN, as OCP FP6/FP4 have.
const FINITE_ONLY_SPECIAL_VALUES: &[SpecialValue] = &[
    SpecialValue::Zero,
    SpecialValue::SignedZero,
    SpecialValue::Subnormals,
];

/// One governed built-in scalar identity in the `tiler` namespace.
struct BuiltInScalar {
    /// Canonical name within the `tiler` namespace.
    name: &'static str,
    /// The family and its complete static parameters.
    kind: ScalarKind,
    /// Authority, document edition, exact format, preserved source id, and key.
    normative_definition: &'static str,
}

const fn predicate(name: &'static str, normative_definition: &'static str) -> BuiltInScalar {
    BuiltInScalar {
        name,
        kind: ScalarKind::Predicate,
        normative_definition,
    }
}

const fn integer(
    name: &'static str,
    signed: bool,
    width_bits: u32,
    normative_definition: &'static str,
) -> BuiltInScalar {
    BuiltInScalar {
        name,
        kind: ScalarKind::Integer { signed, width_bits },
        normative_definition,
    }
}

/// Builds an IEEE 754 binary interchange row.
///
/// The special-value flags are the ones IEEE 754 defines for every binary
/// interchange format, which is why they are stated once here rather than
/// repeated per row; the bias is deliberately absent for the reason
/// [`BinaryFloat::exponent_bias`] records.
const fn ieee_binary(
    name: &'static str,
    width_bits: u32,
    exponent_bits: u32,
    trailing_significand_bits: u32,
    normative_definition: &'static str,
) -> BuiltInScalar {
    BuiltInScalar {
        name,
        kind: ScalarKind::BinaryFloat(BinaryFloat {
            class: "ieee-binary",
            width_bits,
            sign_bits: 1,
            exponent_bits,
            trailing_significand_bits,
            exponent_bias: None,
            special_values: IEEE_BINARY_SPECIAL_VALUES,
        }),
        normative_definition,
    }
}

const fn decimal(
    name: &'static str,
    width_bits: u32,
    coefficient_digits: u32,
    normative_definition: &'static str,
) -> BuiltInScalar {
    BuiltInScalar {
        name,
        kind: ScalarKind::DecimalFloat {
            width_bits,
            coefficient_digits,
        },
        normative_definition,
    }
}

/// Every governed built-in nominal scalar identity, grouped by accepted decision.
///
/// The order here is the reading order of the decisions that admit these rows;
/// the registry stores and iterates them in canonical key order, which a test
/// pins separately so neither order can be mistaken for the other.
const BUILT_IN_SCALARS: &[BuiltInScalar] = &[
    // ADR 0028: predicate and exact-width integers.
    predicate(
        "bool",
        "Tiler governed two-valued logical predicate, distinct from integer i1; ADR 0028; tiler::bool@1",
    ),
    integer(
        "i2",
        true,
        2,
        "Tiler governed signed two's-complement 2-bit logical integer; ADR 0028; tiler::i2@1",
    ),
    integer(
        "i4",
        true,
        4,
        "Tiler governed signed two's-complement 4-bit logical integer; ADR 0028; tiler::i4@1",
    ),
    integer(
        "i8",
        true,
        8,
        "Tiler governed signed two's-complement 8-bit logical integer; ADR 0028; tiler::i8@1",
    ),
    integer(
        "i16",
        true,
        16,
        "Tiler governed signed two's-complement 16-bit logical integer; ADR 0028; tiler::i16@1",
    ),
    integer(
        "i32",
        true,
        32,
        "Tiler governed signed two's-complement 32-bit logical integer; ADR 0028; tiler::i32@1",
    ),
    integer(
        "i64",
        true,
        64,
        "Tiler governed signed two's-complement 64-bit logical integer; ADR 0028; tiler::i64@1",
    ),
    integer(
        "u2",
        false,
        2,
        "Tiler governed unsigned 2-bit logical integer; ADR 0028; tiler::u2@1",
    ),
    integer(
        "u4",
        false,
        4,
        "Tiler governed unsigned 4-bit logical integer; ADR 0028; tiler::u4@1",
    ),
    integer(
        "u8",
        false,
        8,
        "Tiler governed unsigned 8-bit logical integer; ADR 0028; tiler::u8@1",
    ),
    integer(
        "u16",
        false,
        16,
        "Tiler governed unsigned 16-bit logical integer; ADR 0028; tiler::u16@1",
    ),
    integer(
        "u32",
        false,
        32,
        "Tiler governed unsigned 32-bit logical integer; ADR 0028; tiler::u32@1",
    ),
    integer(
        "u64",
        false,
        64,
        "Tiler governed unsigned 64-bit logical integer; ADR 0028; tiler::u64@1",
    ),
    // ADR 0036: IEEE binary interchange formats.
    ieee_binary(
        "f16",
        16,
        5,
        10,
        "IEEE 754-2019 binary16; source ieee-754-2019; tiler::f16@1",
    ),
    ieee_binary(
        "f32",
        32,
        8,
        23,
        "IEEE 754-2019 binary32; source ieee-754-2019; tiler::f32@1",
    ),
    ieee_binary(
        "f64",
        64,
        11,
        52,
        "IEEE 754-2019 binary64; source ieee-754-2019; tiler::f64@1",
    ),
    ieee_binary(
        "f128",
        128,
        15,
        112,
        "IEEE 754-2019 binary128; source ieee-754-2019; tiler::f128@1",
    ),
    // ADR 0036: bfloat16, pinned to the ratified RISC-V BF16 operand format.
    BuiltInScalar {
        name: "bf16",
        kind: ScalarKind::BinaryFloat(BinaryFloat {
            class: "bfloat",
            width_bits: 16,
            sign_bits: 1,
            exponent_bits: 8,
            trailing_significand_bits: 7,
            exponent_bias: None,
            special_values: IEEE_BINARY_SPECIAL_VALUES,
        }),
        normative_definition: "RISC-V Unprivileged ISA version 20260120, chapter 25, BF16 extensions version 1.0, operand format; source riscv-unprivileged-isa-20260120; tiler::bf16@1",
    },
    // ADR 0036: OCP OFP8 revision 1.0 element formats.
    BuiltInScalar {
        name: "f8e4m3fn",
        kind: ScalarKind::BinaryFloat(BinaryFloat {
            class: "ocp-binary-element",
            width_bits: 8,
            sign_bits: 1,
            exponent_bits: 4,
            trailing_significand_bits: 3,
            exponent_bias: Some(7),
            special_values: FINITE_WITH_NAN_SPECIAL_VALUES,
        }),
        normative_definition: "OCP 8-bit Floating Point Specification (OFP8) revision 1.0, E4M3; source ocp-ofp8-v1.0; tiler::f8e4m3fn@1",
    },
    BuiltInScalar {
        name: "f8e5m2",
        kind: ScalarKind::BinaryFloat(BinaryFloat {
            class: "ocp-binary-element",
            width_bits: 8,
            sign_bits: 1,
            exponent_bits: 5,
            trailing_significand_bits: 2,
            exponent_bias: Some(15),
            special_values: IEEE_BINARY_SPECIAL_VALUES,
        }),
        normative_definition: "OCP 8-bit Floating Point Specification (OFP8) revision 1.0, E5M2; source ocp-ofp8-v1.0; tiler::f8e5m2@1",
    },
    // ADR 0036: OCP MX version 1.0 element and scale formats.
    BuiltInScalar {
        name: "f6e2m3fn",
        kind: ScalarKind::BinaryFloat(BinaryFloat {
            class: "ocp-binary-element",
            width_bits: 6,
            sign_bits: 1,
            exponent_bits: 2,
            trailing_significand_bits: 3,
            exponent_bias: Some(1),
            special_values: FINITE_ONLY_SPECIAL_VALUES,
        }),
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, E2M3; source ocp-mx-v1.0; tiler::f6e2m3fn@1",
    },
    BuiltInScalar {
        name: "f6e3m2fn",
        kind: ScalarKind::BinaryFloat(BinaryFloat {
            class: "ocp-binary-element",
            width_bits: 6,
            sign_bits: 1,
            exponent_bits: 3,
            trailing_significand_bits: 2,
            exponent_bias: Some(3),
            special_values: FINITE_ONLY_SPECIAL_VALUES,
        }),
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, E3M2; source ocp-mx-v1.0; tiler::f6e3m2fn@1",
    },
    BuiltInScalar {
        name: "f4e2m1fn",
        kind: ScalarKind::BinaryFloat(BinaryFloat {
            class: "ocp-binary-element",
            width_bits: 4,
            sign_bits: 1,
            exponent_bits: 2,
            trailing_significand_bits: 1,
            exponent_bias: Some(1),
            special_values: FINITE_ONLY_SPECIAL_VALUES,
        }),
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, E2M1; source ocp-mx-v1.0; tiler::f4e2m1fn@1",
    },
    // Exponent-only scale data, not an ordinary signed arithmetic element.
    // ADR 0036 states the value set exactly: positive powers of two plus NaN,
    // with no zero, sign, or infinity.
    BuiltInScalar {
        name: "f8e8m0fnu",
        kind: ScalarKind::BinaryFloat(BinaryFloat {
            class: "ocp-exponent-scale",
            width_bits: 8,
            sign_bits: 0,
            exponent_bits: 8,
            trailing_significand_bits: 0,
            exponent_bias: Some(127),
            special_values: &[SpecialValue::Nan],
        }),
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, E8M0 scale data; source ocp-mx-v1.0; tiler::f8e8m0fnu@1",
    },
    // ADR 0035: IEEE decimal interchange formats.
    decimal(
        "decimal32",
        32,
        7,
        "IEEE 754-2019 decimal32; source ieee-754-2019; tiler::decimal32@1",
    ),
    decimal(
        "decimal64",
        64,
        16,
        "IEEE 754-2019 decimal64; source ieee-754-2019; tiler::decimal64@1",
    ),
    decimal(
        "decimal128",
        128,
        34,
        "IEEE 754-2019 decimal128; source ieee-754-2019; tiler::decimal128@1",
    ),
];

/// Real component formats ADR 0037 admits into the complex family initially.
const COMPLEX_COMPONENTS: &[&str] = &["f16", "f32", "f64"];

const COMPLEX_NORMATIVE_DEFINITION: &str = "Tiler governed complex family: one logical scalar as an ordered real and imaginary pair over one admitted real component format; ADR 0037; tiler::complex@1";

/// One governed OCP microscaling compound scheme identity.
struct MicroscalingScheme {
    /// Canonical scheme name within the `tiler` namespace.
    name: &'static str,
    /// Canonical name of the element-code type the scheme composes.
    element: &'static str,
    /// Authority, document edition, exact scheme, preserved source id, and key.
    normative_definition: &'static str,
}

/// The six OCP MX version 1.0 schemes ADR 0038 admits.
const MICROSCALING_SCHEMES: &[MicroscalingScheme] = &[
    MicroscalingScheme {
        name: "mxfp8_e4m3",
        element: "f8e4m3fn",
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, MXFP8 with E4M3 elements; source ocp-mx-v1.0; tiler::mxfp8_e4m3@1",
    },
    MicroscalingScheme {
        name: "mxfp8_e5m2",
        element: "f8e5m2",
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, MXFP8 with E5M2 elements; source ocp-mx-v1.0; tiler::mxfp8_e5m2@1",
    },
    MicroscalingScheme {
        name: "mxfp6_e2m3",
        element: "f6e2m3fn",
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, MXFP6 with E2M3 elements; source ocp-mx-v1.0; tiler::mxfp6_e2m3@1",
    },
    MicroscalingScheme {
        name: "mxfp6_e3m2",
        element: "f6e3m2fn",
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, MXFP6 with E3M2 elements; source ocp-mx-v1.0; tiler::mxfp6_e3m2@1",
    },
    MicroscalingScheme {
        name: "mxfp4_e2m1",
        element: "f4e2m1fn",
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, MXFP4 with E2M1 elements; source ocp-mx-v1.0; tiler::mxfp4_e2m1@1",
    },
    MicroscalingScheme {
        name: "mxint8",
        element: "i8",
        normative_definition: "OCP Microscaling Formats (MX) version 1.0, MXINT8 with signed 8-bit integer elements; source ocp-mx-v1.0; tiler::mxint8@1",
    },
];

fn governed_type_key(name: &str) -> TypeKey {
    TypeKey::new("tiler", name, BUILT_IN_SEMANTIC_VERSION)
        .expect("a governed built-in dtype name is canonical")
}

fn governed_nominal_type(name: &str) -> ResolvedValueType {
    ResolvedValueType::nominal(governed_type_key(name))
}

fn utf8_fact(value: &str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("a governed catalog fact is bounded")
}

impl BuiltInScalar {
    fn type_key(&self) -> TypeKey {
        governed_type_key(self.name)
    }

    /// Builds the immutable canonical descriptor facts for this row.
    fn canonical_facts(&self) -> CanonicalValue {
        let mut fields = vec![CanonicalField::new(
            SCALAR_TYPE_FACT_ALIAS_POLICY,
            utf8_fact(ALIAS_AND_EQUIVALENCE_POLICY),
        )];
        match &self.kind {
            ScalarKind::Predicate => {
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_CLASS,
                    utf8_fact("logical-predicate"),
                ));
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_VALUE_CARDINALITY,
                    CanonicalValue::unsigned_u32(2),
                ));
            }
            ScalarKind::Integer { signed, width_bits } => {
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_CLASS,
                    utf8_fact(if *signed {
                        "signed-integer"
                    } else {
                        "unsigned-integer"
                    }),
                ));
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_WIDTH_BITS,
                    CanonicalValue::unsigned_u32(*width_bits),
                ));
            }
            ScalarKind::BinaryFloat(format) => {
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_CLASS,
                    utf8_fact(format.class),
                ));
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_WIDTH_BITS,
                    CanonicalValue::unsigned_u32(format.width_bits),
                ));
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_SIGN_BITS,
                    CanonicalValue::unsigned_u32(format.sign_bits),
                ));
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_EXPONENT_BITS,
                    CanonicalValue::unsigned_u32(format.exponent_bits),
                ));
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS,
                    CanonicalValue::unsigned_u32(format.trailing_significand_bits),
                ));
                if let Some(bias) = format.exponent_bias {
                    fields.push(CanonicalField::new(
                        SCALAR_TYPE_FACT_EXPONENT_BIAS,
                        CanonicalValue::signed_i32(bias),
                    ));
                }
                // Every member is reported, present or absent: a reader must be
                // able to distinguish "this format has no NaN" from "this
                // descriptor does not say", and only an unconditional field can.
                for (special, id) in SPECIAL_VALUE_FIELDS {
                    fields.push(CanonicalField::new(
                        id,
                        CanonicalValue::boolean(format.special_values.contains(&special)),
                    ));
                }
            }
            ScalarKind::DecimalFloat {
                width_bits,
                coefficient_digits,
            } => {
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_CLASS,
                    utf8_fact("ieee-decimal"),
                ));
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_WIDTH_BITS,
                    CanonicalValue::unsigned_u32(*width_bits),
                ));
                fields.push(CanonicalField::new(
                    SCALAR_TYPE_FACT_COEFFICIENT_DIGITS,
                    CanonicalValue::unsigned_u32(*coefficient_digits),
                ));
            }
        }
        CanonicalValue::record(fields).expect("governed catalog facts are canonical")
    }
}

impl MicroscalingScheme {
    fn scheme_key(&self) -> QuantSchemeKey {
        QuantSchemeKey::new("tiler", self.name, BUILT_IN_SEMANTIC_VERSION)
            .expect("a governed microscaling scheme name is canonical")
    }

    /// Builds the immutable canonical descriptor facts for this scheme.
    ///
    /// The constituent element and scale identities are carried as complete
    /// resolved types rather than names, so the frozen registry's authority
    /// closure reaches them: a scheme naming an unregistered constituent cannot
    /// be frozen. The conversion, rounding, saturation, and block-wide
    /// special-value rules stay with the pinned normative reference and are not
    /// restated here — `ocp-mx-v1.0` is a metadata-only record whose bytes this
    /// repository does not hold, so a restatement would be an unverifiable copy
    /// occupying the place where an authority belongs.
    fn canonical_facts(&self) -> CanonicalValue {
        CanonicalValue::record([
            CanonicalField::new(
                SCALAR_TYPE_FACT_CLASS,
                utf8_fact("ocp-microscaling-block-scheme"),
            ),
            CanonicalField::new(
                SCALAR_TYPE_FACT_ALIAS_POLICY,
                utf8_fact(ALIAS_AND_EQUIVALENCE_POLICY),
            ),
            CanonicalField::new(
                SCALAR_TYPE_FACT_COMPONENT_TYPES,
                CanonicalValue::sequence([
                    CanonicalValue::value_type(governed_nominal_type(self.element)),
                    CanonicalValue::value_type(governed_nominal_type(MICROSCALING_SCALE_TYPE)),
                ])
                .expect("two constituent types are within the canonical bounds"),
            ),
            CanonicalField::new(
                SCALAR_TYPE_FACT_COMPONENT_ORDER,
                utf8_fact(MICROSCALING_COMPONENT_ORDER),
            ),
            CanonicalField::new(
                SCALAR_TYPE_FACT_BLOCK_SIZE,
                CanonicalValue::unsigned_u32(MICROSCALING_BLOCK_SIZE),
            ),
            CanonicalField::new(
                SCALAR_TYPE_FACT_SCALE_SELECTION,
                utf8_fact(MICROSCALING_SCALE_SELECTION),
            ),
        ])
        .expect("governed microscaling facts are canonical")
    }
}

fn complex_family_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(SCALAR_TYPE_FACT_CLASS, utf8_fact("complex")),
        CanonicalField::new(
            SCALAR_TYPE_FACT_ALIAS_POLICY,
            utf8_fact(ALIAS_AND_EQUIVALENCE_POLICY),
        ),
        CanonicalField::new(
            SCALAR_TYPE_FACT_COMPONENT_TYPES,
            CanonicalValue::sequence(
                COMPLEX_COMPONENTS
                    .iter()
                    .map(|name| CanonicalValue::value_type(governed_nominal_type(name))),
            )
            .expect("the admitted complex components are within the canonical bounds"),
        ),
        CanonicalField::new(
            SCALAR_TYPE_FACT_COMPONENT_ORDER,
            utf8_fact(COMPLEX_COMPONENT_ORDER),
        ),
        CanonicalField::new(
            SCALAR_TYPE_FACT_VALUE_CARDINALITY,
            CanonicalValue::unsigned_u32(2),
        ),
    ])
    .expect("governed complex family facts are canonical")
}

/// Returns the governed complex type constructor.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn complex_type_constructor() -> TypeKey {
    governed_type_key("complex")
}

/// Returns the real component formats the complex family admits today.
///
/// Extending this set is a catalog decision under ADR 0037, not an inference
/// from a component being a recognized real format.
#[must_use]
pub fn admitted_complex_component_types() -> Vec<ResolvedValueType> {
    COMPLEX_COMPONENTS
        .iter()
        .map(|name| governed_nominal_type(name))
        .collect()
}

/// Builds the complete complex identity over one real component format.
///
/// The argument list is part of canonical identity, so this constructor exists
/// to keep its exact shape in one place. It applies the family constructor and
/// nothing else: whether the component is admitted is decided by the registered
/// family validator when the value reaches semantic authority, which is what
/// keeps recognition and admission separate subjects.
///
/// # Errors
///
/// Returns [`TypeIdentityError`] when the resulting type exceeds a canonical
/// structural bound.
pub fn complex_value_type(
    component: &ResolvedValueType,
) -> Result<ResolvedValueType, TypeIdentityError> {
    ResolvedValueType::parameterized(
        complex_type_constructor(),
        TypeArguments::new([CanonicalValue::value_type(component.clone())])?,
    )
}

/// Returns every governed built-in nominal scalar identity in canonical key order.
#[must_use]
pub fn builtin_scalar_value_types() -> Vec<ResolvedValueType> {
    let mut types: Vec<_> = BUILT_IN_SCALARS
        .iter()
        .map(|scalar| ResolvedValueType::nominal(scalar.type_key()))
        .collect();
    types.sort_unstable();
    types
}

/// Returns every governed OCP microscaling scheme identity in canonical key order.
#[must_use]
pub fn microscaling_scheme_keys() -> Vec<QuantSchemeKey> {
    let mut schemes: Vec<_> = MICROSCALING_SCHEMES
        .iter()
        .map(MicroscalingScheme::scheme_key)
        .collect();
    schemes.sort_unstable();
    schemes
}

/// Registers the complete accepted built-in dtype catalog.
///
/// The standard provider calls this once; every governed identity in the
/// catalog exists because a row above states it, so a duplicate row is a
/// registration failure rather than a silently shadowed definition.
pub(super) fn register_builtin_dtype_catalog(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    for scalar in BUILT_IN_SCALARS {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(scalar.type_key()),
            NormativeDefinitionRef::new(scalar.normative_definition)?,
            TypeDefinitionFacts::new(scalar.canonical_facts()),
        ))?;
    }
    registrar.register_value_type(ValueTypeDefinition::new(
        ValueTypeDefinitionKey::Parameterized(complex_type_constructor()),
        NormativeDefinitionRef::new(COMPLEX_NORMATIVE_DEFINITION)?,
        TypeDefinitionFacts::new(complex_family_facts()),
        Arc::new(ComplexComponentValidator),
    ))?;
    for scheme in MICROSCALING_SCHEMES {
        registrar.register_value_type(ValueTypeDefinition::new(
            ValueTypeDefinitionKey::EncodedNumeric(scheme.scheme_key()),
            NormativeDefinitionRef::new(scheme.normative_definition)?,
            TypeDefinitionFacts::new(scheme.canonical_facts()),
            Arc::new(MicroscalingContractValidator),
        ))?;
    }
    Ok(())
}

/// Admits exactly the complex instances ADR 0037 lists.
struct ComplexComponentValidator;

impl ValueTypeInstanceValidator for ComplexComponentValidator {
    fn validate(&self, value: &ResolvedValueType) -> Result<(), TypeInstanceError> {
        let Some((_, arguments)) = value.parameterized_parts() else {
            return Err(type_error(
                "complex.not-parameterized",
                "complex@1 governs parameterized instances only",
            ));
        };
        let [argument] = arguments.values() else {
            return Err(type_error(
                "complex.arity",
                "complex@1 takes exactly one real component type argument",
            ));
        };
        let CanonicalValueView::Type(component) = argument.view() else {
            return Err(type_error(
                "complex.argument-kind",
                "the complex component argument must be a complete resolved value type",
            ));
        };
        if admitted_complex_component_types()
            .iter()
            .any(|admitted| admitted == component)
        {
            return Ok(());
        }
        Err(type_error(
            "complex.unsupported-component",
            "complex@1 admits only the f16, f32, and f64 real components accepted by ADR 0037",
        ))
    }
}

/// Refuses every microscaling instance while the block map does not exist.
///
/// The scheme identities are registered so an MX value is *recognized and
/// refused with a reason* rather than reported as an unknown identity — ADR 0026
/// keeps unknown and unsupported distinct, and only the second is true here.
/// Admitting a contract would require declaring the shared scale's shape
/// relation, and the only parameter-index map that exists is per-tensor, which
/// is a different and wrong association for a 32-element block. Stating it would
/// put a false numerical contract into durable identity, so the family fails
/// closed until a per-block map arrives with the producer that can validate it.
struct MicroscalingContractValidator;

impl ValueTypeInstanceValidator for MicroscalingContractValidator {
    fn validate(&self, _: &ResolvedValueType) -> Result<(), TypeInstanceError> {
        Err(type_error(
            "microscaling.unsupported-contract",
            "an OCP MX scheme identity is recognized but admits no static contract: no per-block parameter-index map exists to associate one scale with a 32-element block",
        ))
    }
}

fn type_error(code: &'static str, message: &'static str) -> TypeInstanceError {
    TypeInstanceError::new(
        ProviderDiagnosticCode::new(code).expect("a governed diagnostic code is canonical"),
        message,
    )
    .expect("a governed diagnostic message is canonical")
}

#[cfg(test)]
mod tests;
