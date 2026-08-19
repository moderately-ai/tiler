//! Semantic-graph lookups the recognizers share.
//!
//! Producer resolution, attribute decoding, shape and element-count reads. Each
//! answers one question about a verified program and refuses by name rather than
//! resolving a symbolic extent, guessing a payload width, or reporting a
//! saturated count — a recognized shape derived from any of those would be a
//! confidently wrong statement about the caller's program.

use super::*;

/// Requires reduction axes to be in range and in strictly ascending order.
pub(super) fn check_canonical_reduction_axes(
    axes: &[Axis],
    rank: usize,
) -> Result<(), RequestError> {
    let mut previous = None;
    for axis in axes {
        let index =
            usize::try_from(axis.get()).map_err(|_| RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "sum-axis-range",
            })?;
        if index >= rank {
            return mismatch("sum-axis-range");
        }
        if previous.is_some_and(|previous| previous >= axis.get()) {
            return mismatch("sum-axes-canonical");
        }
        previous = Some(axis.get());
    }
    Ok(())
}

pub(super) fn producer<'a>(
    program: &'a SemanticProgram,
    value: ValueId,
    expected: &OpKey,
) -> Result<(u32, tiler_ir::semantic::OperationRef<'a>), RequestError> {
    let (ordinal, operation) = producer_for_value(program, value)?;
    if operation.key() != expected {
        return mismatch("operation-family");
    }
    Ok((ordinal, operation))
}

pub(super) fn producer_for_value(
    program: &SemanticProgram,
    value: ValueId,
) -> Result<(u32, tiler_ir::semantic::OperationRef<'_>), RequestError> {
    let (ordinal, operation) = program
        .operations()
        .enumerate()
        .find(|(_, operation)| operation.results().any(|result| result == value))
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-producer",
        })?;
    let ordinal = u32::try_from(ordinal).map_err(|_| RequestError::UnsupportedCapability {
        phase: "strategy",
        rule: "operation-ordinal",
    })?;
    Ok((ordinal, operation))
}

/// Reads one exact constant occurrence's declared payload, in its own width.
///
/// **The payload is returned in a `u32` for both widths, and that is a carrier
/// rather than a widening.** The declared byte run is read at the exact length
/// the family declares — four for binary32, two for `bf16` — and a run of any
/// other length is refused rather than zero-extended, so a `bf16` payload that
/// arrived four bytes wide is a malformed record here instead of a number whose
/// upper half nobody stated. [`Bf16Mint::constant`](super::elementwise::PointwiseMintSink::constant) narrows back before minting.
///
/// The format key is checked against the *family's own* type key rather than
/// against binary32's: a record naming one family and carrying another's format
/// is a disagreement between its two halves, and admitting it would let a
/// `bf16` occurrence carry a binary32 pattern into a region whose identity
/// claims `bf16`.
pub(super) fn constant_bits(
    program: &SemanticProgram,
    value: ValueId,
    arithmetic: ArithmeticType,
) -> Result<(u32, u32), RequestError> {
    let Some(family) = constant_family(arithmetic) else {
        return mismatch("dtype-recognized");
    };
    let (ordinal, operation) = producer(program, value, &family)?;
    if operation.operands().len() != 0 || operation.results().len() != 1 {
        return mismatch("constant-signature");
    }
    let (attribute, name) = match arithmetic {
        ArithmeticType::F32 => (F32_CONSTANT_BITS_ATTRIBUTE, "f32"),
        ArithmeticType::Bf16 => (BF16_CONSTANT_BITS_ATTRIBUTE, "bf16"),
        // Unreachable through the family lookup above, and refused rather than
        // defaulted to either row: a payload field guessed for a width this
        // recognizer states no constant family for would read some other
        // family's bytes.
        ArithmeticType::F16 | ArithmeticType::F64 => return mismatch("dtype-recognized"),
    };
    let Some(CanonicalValueView::FloatBits(bits)) = operation
        .attributes()
        .get(attribute)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return mismatch("constant-bits");
    };
    let governed =
        TypeKey::new("tiler", name, 1).map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "governed-constant-key",
        })?;
    if bits.format() != &governed {
        return mismatch("constant-bits-format");
    }
    let packed = match arithmetic {
        ArithmeticType::F32 => <[u8; 4]>::try_from(bits.bits())
            .map(u32::from_be_bytes)
            .ok(),
        ArithmeticType::Bf16 => <[u8; 2]>::try_from(bits.bits())
            .map(|bytes| u32::from(u16::from_be_bytes(bytes)))
            .ok(),
        ArithmeticType::F16 | ArithmeticType::F64 => None,
    };
    packed
        .map(|packed| (packed, ordinal))
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "constant-bits",
        })
}

pub(super) fn reduction_axes(
    attributes: &tiler_ir::semantic::OperationAttributes,
) -> Result<Vec<Axis>, RequestError> {
    let Some(CanonicalValueView::Sequence(values)) = attributes
        .get(REDUCTION_AXES_ATTRIBUTE)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return mismatch("sum-axes");
    };
    values
        .iter()
        .map(|value| {
            let CanonicalValueView::Unsigned { width, bits } = value.view() else {
                return mismatch("sum-axes");
            };
            if width != CanonicalIntegerWidth::Bits32 {
                return mismatch("sum-axes-width");
            }
            u32::try_from(bits)
                .map(Axis::new)
                .map_err(|_| RequestError::UnsupportedCapability {
                    phase: "strategy",
                    rule: "sum-axes",
                })
        })
        .collect()
}

/// Returns one semantic value's fixed shape, refusing a symbolic one by name.
///
/// Recognition matches a program against a physical strategy. Same-shape
/// elementwise and a sourced broadcast (the labelled-draft parametric carrier)
/// are admitted with extents left symbolic. Every other strategy below is still
/// stated over fixed extents: a domain a launch geometry is derived from, an
/// element count, or a reindex axis decode. A symbolic extent is refused here
/// rather than resolved through the environment, which would make the recognized
/// region name extents nobody wrote.
///
/// The refusal names the extent as written, not the handle lookup that
/// observed it. A bound symbol is still this refusal: specializing it into the
/// logical plan is a physical-planning decision this boundary must not make.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] for a foreign handle, and
/// [`RequestError::UnsupportedSymbolicExtent`] naming the first non-static
/// extent when the value's shape is symbolic.
pub(super) fn static_shape(
    program: &SemanticProgram,
    value: ValueId,
    rule: &'static str,
) -> Result<Shape, RequestError> {
    let sourced = program
        .shape(value)
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule,
        })?;
    if let Some(shape) = sourced.as_static() {
        return Ok(shape.clone());
    }
    let extent = sourced
        .extents()
        .find(|extent| extent.as_static().is_none())
        .expect("a non-static SourcedShape holds at least one symbol");
    Err(RequestError::UnsupportedSymbolicExtent {
        phase: "strategy",
        rule: "symbolic-extent",
        extent,
    })
}

/// Returns one value's fixed shape, or `None` for a foreign or symbolic one.
///
/// The borrowing form, for the comparisons that already treat an unreadable
/// shape as a mismatch. A symbolic shape compares unequal to every [`Shape`],
/// which is the answer those sites want: the strategy is not recognized.
pub(super) fn static_shape_ref(program: &SemanticProgram, value: ValueId) -> Option<&Shape> {
    program.shape(value).ok()?.as_static()
}

/// Returns one semantic value's sourced shape, refusing only a foreign handle.
pub(super) fn sourced_shape(
    program: &SemanticProgram,
    value: ValueId,
    rule: &'static str,
) -> Result<SourcedShape, RequestError> {
    program
        .shape(value)
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule,
        })
        .cloned()
}

/// Returns one value's sourced shape, or `None` for a foreign handle.
pub(super) fn sourced_shape_ref(
    program: &SemanticProgram,
    value: ValueId,
) -> Option<&SourcedShape> {
    program.shape(value).ok()
}

fn first_nonstatic_extent(program: &SemanticProgram, value: ValueId) -> Option<SourcedExtent> {
    program
        .shape(value)
        .ok()?
        .extents()
        .find(|extent| extent.as_static().is_none())
}

pub(super) fn unsupported_symbolic_extent(
    program: &SemanticProgram,
    value: ValueId,
    domain: &SourcedShape,
) -> RequestError {
    first_nonstatic_extent(program, value)
        .or_else(|| domain.extents().find(|extent| extent.as_static().is_none()))
        .map_or(
            RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "operation-set",
            },
            |extent| RequestError::UnsupportedSymbolicExtent {
                phase: "strategy",
                rule: "symbolic-extent",
                extent,
            },
        )
}

pub(super) fn element_count_u64(shape: &Shape, role: &'static str) -> Result<u64, RequestError> {
    if shape.extents().iter().any(|extent| extent.get() == 0) {
        return Ok(0);
    }
    shape.extents().iter().try_fold(1_u64, |count, extent| {
        count
            .checked_mul(extent.get())
            .ok_or(RequestError::ShapeProductOverflow { role })
    })
}
