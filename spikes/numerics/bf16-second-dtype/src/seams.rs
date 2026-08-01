//! The seam probes: which accepted boundaries admit a second scalar float dtype.
//!
//! Each probe is a positive control paired with the BF16 case, so a refusal is
//! evidence about BF16 rather than about a harness that refuses everything.

use tiler_compiler::target::{
    DTypeDispatchability, ScalarArithmetic, TargetFactProducerIdentity, TargetFactSource,
    TargetNormativeReferenceIdentity, TargetProfileBuilder, TargetProfileKey,
};
use tiler_ir::semantic::{
    CanonicalValueView, F32, ResolvedValueType, SCALAR_TYPE_FACT_EXPONENT_BITS,
    SCALAR_TYPE_FACT_SIGN_BITS, SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS,
    SCALAR_TYPE_FACT_WIDTH_BITS, TypeKey, ValueTypeMarker, accuracy::UlpFormat,
    builtin_scalar_value_type_facts,
};
use tiler_reference::{FloatBitOrder, ReferenceElement};

use crate::bf16;

/// Governed Rust marker for RISC-V BF16 values, minted by this spike.
///
/// `ValueTypeMarker` is documented as an *open local marker*: implementing it
/// grants no semantic authority, and a frozen registry must separately bind the
/// marker to an admitted complete resolved type. This is therefore an extension
/// point used as designed, not a production edit — the identity it names is the
/// one `crates/tiler-ir`'s catalog already registers.
pub enum Bf16 {}

impl ValueTypeMarker for Bf16 {}

impl Bf16 {
    /// Returns the governed complete BF16 semantic identity, `tiler::bf16@1`.
    #[must_use]
    pub fn resolved_type() -> ResolvedValueType {
        ResolvedValueType::nominal(
            TypeKey::new("tiler", "bf16", 1).expect("the governed BF16 key is valid"),
        )
    }
}

/// One probe's verdict, rendered into the run narrative.
pub struct Verdict {
    /// What was probed.
    pub subject: &'static str,
    /// Whether the seam admitted BF16.
    pub admitted: bool,
    /// The exact observation.
    pub detail: String,
}

fn source() -> TargetFactSource {
    let producer = TargetFactProducerIdentity::new("tiler.spike.bf16-second-dtype".to_owned(), 1)
        .expect("the spike producer identity is valid");
    let reference =
        TargetNormativeReferenceIdentity::new("riscv.unprivileged.isa.20260120.bf16".to_owned(), 1)
            .expect("the BF16 normative reference identity is valid");
    TargetFactSource::external_guarantee(producer, reference)
}

/// Probes whether the registered catalog descriptor is reachable for BF16.
#[must_use]
pub fn descriptor_seam() -> Verdict {
    let f32_facts = builtin_scalar_value_type_facts(&F32::resolved_type());
    let bf16_facts = builtin_scalar_value_type_facts(&Bf16::resolved_type());
    Verdict {
        subject: "catalog descriptor (builtin_scalar_value_type_facts)",
        admitted: bf16_facts.is_some(),
        detail: format!(
            "f32 descriptor present={}, bf16 descriptor present={}",
            f32_facts.is_some(),
            bf16_facts.is_some()
        ),
    }
}

/// Checks this spike's BF16 parameters against the registered catalog descriptor.
///
/// The oracle's format constants are a second copy of what
/// `crates/tiler-ir/src/semantic/catalog.rs` states, and a second copy is a
/// second place to be wrong. This reads the registered descriptor's own fields
/// and refuses any disagreement, so a catalog change breaks this spike loudly
/// instead of leaving it quietly measuring a format Tiler no longer recognizes.
#[must_use]
pub fn descriptor_agreement_seam() -> Verdict {
    let facts = builtin_scalar_value_type_facts(&Bf16::resolved_type())
        .expect("the governed BF16 descriptor is registered");
    let CanonicalValueView::Record(fields) = facts.view() else {
        return Verdict {
            subject: "spike constants against the registered BF16 descriptor",
            admitted: false,
            detail: "the descriptor is not a canonical record".to_owned(),
        };
    };
    let unsigned = |id| {
        fields
            .iter()
            .find(|field| field.id() == id)
            .and_then(|field| match field.value().view() {
                CanonicalValueView::Unsigned { bits, .. } => u32::try_from(bits).ok(),
                _ => None,
            })
    };
    let expected = [
        ("width", SCALAR_TYPE_FACT_WIDTH_BITS, bf16::WIDTH_BITS),
        ("sign", SCALAR_TYPE_FACT_SIGN_BITS, bf16::SIGN_BITS),
        (
            "exponent",
            SCALAR_TYPE_FACT_EXPONENT_BITS,
            bf16::EXPONENT_BITS,
        ),
        (
            "trailing significand",
            SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS,
            bf16::TRAILING_SIGNIFICAND_BITS,
        ),
    ];
    let mismatches: Vec<_> = expected
        .iter()
        .filter_map(|(name, id, ours)| {
            let theirs = unsigned(*id);
            (theirs != Some(*ours)).then(|| format!("{name}: catalog {theirs:?} vs spike {ours}"))
        })
        .collect();
    Verdict {
        subject: "spike constants against the registered BF16 descriptor",
        admitted: mismatches.is_empty(),
        detail: if mismatches.is_empty() {
            format!(
                "all four structural parameters agree (width {}, sign {}, exponent {}, trailing {})",
                bf16::WIDTH_BITS,
                bf16::SIGN_BITS,
                bf16::EXPONENT_BITS,
                bf16::TRAILING_SIGNIFICAND_BITS
            )
        } else {
            mismatches.join("; ")
        },
    }
}

/// Probes whether `tiler::ulp-reference-gap@1` accepts BF16's descriptor.
///
/// This is the strongest example of a **generic seam that already admits BF16**.
/// The metric derives a format's finite value set from the registered
/// descriptor's own fields and refuses any dtype whose adjacent-value behaviour
/// is not derivable. BF16 passes not by being special-cased but because the
/// catalog registered a `bfloat` class rule whose basis is the RISC-V operand
/// format. Nothing in this spike had to change for it.
#[must_use]
pub fn ulp_metric_seam() -> Verdict {
    let facts = builtin_scalar_value_type_facts(&Bf16::resolved_type())
        .expect("the governed BF16 descriptor is registered");
    let derived = UlpFormat::from_value_type_facts(&facts);
    // The paired refusal: a class the metric states it cannot interpret. `u8` is
    // an integer, so it must be rejected rather than approximated.
    let integer_facts = builtin_scalar_value_type_facts(&ResolvedValueType::nominal(
        TypeKey::new("tiler", "u8", 1).expect("the governed U8 key is valid"),
    ))
    .expect("the governed U8 descriptor is registered");
    let integer = UlpFormat::from_value_type_facts(&integer_facts);
    Verdict {
        subject: "tiler::ulp-reference-gap@1 dtype compatibility",
        admitted: derived.is_ok(),
        detail: match (&derived, &integer) {
            (Ok(format), Err(error)) => format!(
                "bf16 derives a value set (precision {}, exponents {}..={}, subnormals {}); \
                 u8 is refused as {}",
                format.precision(),
                format.min_exponent(),
                format.max_exponent(),
                format.has_subnormals(),
                error.diagnostic_code()
            ),
            (Ok(_), Ok(_)) => {
                "bf16 derives a value set, but so does an integer -- the check cannot refuse"
                    .to_owned()
            }
            (Err(error), _) => format!("refused: {error}"),
        },
    }
}

/// Probes whether the reference tensor vocabulary carries a 16-bit element.
///
/// `ReferenceElement` is a byte vector whose interpretation belongs to the
/// enclosing tensor's resolved type, so a two-byte BF16 element needs no new
/// carrier. The paired control is a four-byte F32 element built the same way.
#[must_use]
pub fn reference_element_seam() -> Verdict {
    let bf16_element = ReferenceElement::from_float_bits(
        0x3f80_u16.to_be_bytes(),
        FloatBitOrder::MostSignificantByteFirst,
    );
    let f32_element = ReferenceElement::from_float_bits(
        0x3f80_0000_u32.to_be_bytes(),
        FloatBitOrder::MostSignificantByteFirst,
    );
    // The paired refusal: an empty payload is rejected, so a width of zero is
    // not silently admitted as "some width".
    let empty = ReferenceElement::from_float_bits([], FloatBitOrder::MostSignificantByteFirst);
    Verdict {
        subject: "reference element carrier width",
        admitted: bf16_element.is_ok(),
        detail: format!(
            "bf16 element {} bytes, f32 element {} bytes, empty payload refused={}",
            bf16_element.map_or(0, |element| element.as_bytes().len()),
            f32_element.map_or(0, |element| element.as_bytes().len()),
            empty.is_err()
        ),
    }
}

/// Probes whether a caller can declare BF16 dispatchability on a target profile.
#[must_use]
pub fn dispatchability_seam(verdict: DTypeDispatchability) -> Verdict {
    let key = TargetProfileKey::new("tiler.target.spike-bf16-probe".to_owned())
        .expect("the probe profile key is valid");
    let mut builder = TargetProfileBuilder::new(key);
    let outcome = builder.declare_dtype_dispatchability(Bf16::resolved_type(), verdict, source());
    Verdict {
        subject: "target profile dtype dispatchability",
        admitted: outcome.is_ok(),
        detail: match outcome {
            Ok(()) => format!("declare_dtype_dispatchability(bf16, {verdict:?}) accepted"),
            Err(error) => format!("refused: {error:?}"),
        },
    }
}

/// Probes whether a caller can declare a BF16 numerical honourability row.
///
/// The positive control is `ScalarArithmetic::f32()`, which is the only public
/// constructor this boundary offers.
#[must_use]
pub fn scalar_arithmetic_seam() -> Verdict {
    let f32_subject = ScalarArithmetic::f32();
    Verdict {
        subject: "target profile scalar arithmetic subject",
        // There is no public constructor that could even name BF16 here, so the
        // refusal is a compile-time absence rather than a runtime error.
        admitted: false,
        detail: format!(
            "ScalarArithmetic::f32() constructs ({:?}); no public constructor accepts any other \
             resolved type, so a BF16 numerical row is unstatable",
            f32_subject.resolved_type()
        ),
    }
}
