//! The facts an expansion emits, and the checks that read them at runtime.
//!
//! Everything here is reachable only through [`crate::__private`]: these are the
//! items generated tokens name, not a surface a consumer writes. They are `pub`
//! because generated code lands in the consumer's crate and has to resolve them,
//! and they carry no compatibility claim.
//!
//! # Why the facts are data and not a rebuilt environment
//!
//! An expansion builds a real `tiler_ir::shape::ShapeEnv` on the host, in the
//! proc-macro process, and decides everything decidable there: which axis
//! sources each symbol, which axes owe an equality, and what the region's result
//! shape is in terms of those symbols. None of that machinery has to exist at
//! runtime, and shipping it would put the compiler's IR inside the consumer's
//! hot path to re-derive a conclusion already reached.
//!
//! What survives into tokens is therefore the *residue* of that decision — a
//! flat table of operands, symbol sources, equality obligations, and result
//! axes. The environment remains the authority; this is its lowered form, and
//! `tiler_macros::binding` is the one place that lowers it.
//!
//! # Why a symbol source is an index here and a key in the environment
//!
//! The environment binds `n` to `BindingSource::InputDimension { input, axis }`
//! — an interface *key*, which is what makes graph identity independent of the
//! order the `in` list was written in. These facts index the operand table
//! instead, because a runtime check wants an array offset rather than a string
//! comparison. Both spellings name one fact, and the key travels along on
//! [`OperandFacts::key`] so every diagnostic can name the operand a consumer
//! wrote rather than a position it did not.

use crate::value::{
    AdapterCapability, BindError, OperandAxis, ResultRequest, StorageScalar, Tensor, TensorAdapter,
    ValueMetadata,
};

/// One declared operand of a region.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OperandFacts {
    /// Stable interface key the region declared this operand under.
    pub key: &'static str,
    /// Scalar the supplied value's storage must hold.
    pub storage_scalar: StorageScalar,
    /// Rank the supplied value must have.
    pub rank: usize,
}

/// One axis of one operand, by position in [`RegionFacts::operands`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AxisRef {
    /// Index into [`RegionFacts::operands`].
    pub operand: usize,
    /// Zero-based axis position within that operand's shape.
    pub axis: usize,
}

/// One symbolic extent, its canonical source, and every equality it obliges.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SymbolFacts {
    /// The name the region declared with `sym`.
    pub name: &'static str,
    /// The axis the value is read from.
    ///
    /// Chosen by the expansion from the canonical order of interface keys and
    /// axes, so it does not depend on which occurrence appeared first in the
    /// region text.
    pub source: AxisRef,
    /// Every other axis naming this symbol, which must report the same extent.
    pub obligations: &'static [AxisRef],
}

/// Where one axis of the region's result gets its extent.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResultAxis {
    /// An extent the region fixed literally.
    Literal(u64),
    /// An extent equal to a bound symbol, by index into [`RegionFacts::symbols`].
    Symbol(usize),
}

/// The region's single declared result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResultFacts {
    /// Stable interface key the region declared this result under.
    pub key: &'static str,
    /// Scalar the constructed value's storage holds.
    pub storage_scalar: StorageScalar,
    /// The result's axes, outermost first.
    pub axes: &'static [ResultAxis],
}

/// Everything one expanded region needs in order to check and bind its values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RegionFacts {
    /// Declared operands, in the order the region's interface names them.
    pub operands: &'static [OperandFacts],
    /// Declared symbols, in the environment's canonical order.
    pub symbols: &'static [SymbolFacts],
    /// Capabilities this region requires of the adapter.
    pub capabilities: &'static [AdapterCapability],
    /// The region's single result.
    pub result: ResultFacts,
}

/// The extent every declared symbol resolved to, in [`RegionFacts::symbols`] order.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BoundExtents {
    values: Vec<u64>,
}

impl BoundExtents {
    /// Returns the extent bound for one symbol, by index into
    /// [`RegionFacts::symbols`].
    #[must_use]
    pub fn get(&self, symbol: usize) -> Option<u64> {
        self.values.get(symbol).copied()
    }

    /// Returns every bound extent, in [`RegionFacts::symbols`] order.
    #[must_use]
    pub fn values(&self) -> &[u64] {
        &self.values
    }
}

/// Checks one region's operands and binds every declared symbol.
///
/// The order is what makes the diagnostics usable rather than incidental:
/// capabilities first, because a region an adapter cannot serve at all should
/// not report a shape mismatch; then per-operand shape and scalar checks, so a
/// symbol is never bound from a value whose rank was already wrong; then
/// unification, whose only remaining failure is two axes disagreeing.
///
/// # Errors
///
/// Returns [`BindError::UnsupportedCapability`], [`BindError::OperandCountMismatch`],
/// [`BindError::RankMismatch`], [`BindError::StorageScalarMismatch`],
/// [`BindError::InconsistentExtent`], [`BindError::MalformedRegionFacts`], or
/// [`BindError::Adapter`] carrying the adapter's own failure.
pub fn bind_region<A: TensorAdapter>(
    facts: &RegionFacts,
    operands: &[&Tensor<A>],
) -> Result<BoundExtents, BindError<A::Error>> {
    for capability in facts.capabilities {
        if !A::supports(*capability) {
            return Err(BindError::UnsupportedCapability {
                capability: *capability,
            });
        }
    }

    if facts.operands.len() != operands.len() {
        return Err(BindError::OperandCountMismatch {
            declared: facts.operands.len(),
            supplied: operands.len(),
        });
    }

    let mut metadata: Vec<ValueMetadata> = Vec::with_capacity(operands.len());
    for (declared, supplied) in facts.operands.iter().zip(operands) {
        let reported = A::metadata(supplied.value()).map_err(BindError::Adapter)?;
        if reported.rank() != declared.rank {
            return Err(BindError::RankMismatch {
                input: declared.key,
                declared: declared.rank,
                actual: reported.rank(),
            });
        }
        if reported.storage_scalar() != declared.storage_scalar {
            return Err(BindError::StorageScalarMismatch {
                input: declared.key,
                declared: declared.storage_scalar,
                actual: reported.storage_scalar(),
            });
        }
        metadata.push(reported);
    }

    let mut values = Vec::with_capacity(facts.symbols.len());
    for symbol in facts.symbols {
        let source = extent_at(&metadata, symbol.source)?;
        for obligation in symbol.obligations {
            let observed = extent_at(&metadata, *obligation)?;
            if observed != source {
                return Err(BindError::InconsistentExtent {
                    symbol: symbol.name,
                    source: named(facts, symbol.source)?,
                    source_extent: source,
                    conflicting: named(facts, *obligation)?,
                    conflicting_extent: observed,
                });
            }
        }
        values.push(source);
    }

    Ok(BoundExtents { values })
}

/// Constructs the region's declared result through the adapter.
///
/// The context is the caller's rather than one picked from the operands: a
/// region whose operands live on different contexts is outside this bounded
/// profile, and choosing one silently is exactly the kind of implicit placement
/// the architecture keeps explicit. Generated code passes the context of the
/// first declared operand.
///
/// # Errors
///
/// Returns [`BindError::UnsupportedCapability`] when the adapter cannot
/// construct values, [`BindError::MalformedRegionFacts`] when a result axis
/// names a symbol the region did not declare, and [`BindError::Adapter`]
/// carrying the adapter's own failure.
pub fn build_result<A: TensorAdapter>(
    facts: &RegionFacts,
    bound: &BoundExtents,
    context: &A::Context,
) -> Result<A::Value, BindError<A::Error>> {
    if !A::supports(AdapterCapability::ResultConstruction) {
        return Err(BindError::UnsupportedCapability {
            capability: AdapterCapability::ResultConstruction,
        });
    }

    let mut extents = Vec::with_capacity(facts.result.axes.len());
    for axis in facts.result.axes {
        extents.push(match axis {
            ResultAxis::Literal(extent) => *extent,
            ResultAxis::Symbol(index) => {
                bound.get(*index).ok_or(BindError::MalformedRegionFacts {
                    detail: "a result axis names a symbol index the region does not declare",
                })?
            }
        });
    }

    A::build(
        context,
        &ResultRequest::new(facts.result.storage_scalar, &extents),
    )
    .map_err(BindError::Adapter)
}

/// Checks one region's operands and constructs its declared result.
///
/// The composition of [`bind_region`] and [`build_result`], in that order, and
/// the one item a `tiler::tensor!` expansion actually calls.
///
/// # Why generated code cannot call the two directly
///
/// [`build_result`]'s adapter parameter appears only in `A::Context` and in its
/// return type, and neither position determines `A`: an associated type is not
/// injective, so `&A::Context` infers nothing, and the result type is whatever
/// the caller's `let` says. A generated call would therefore need to spell the
/// adapter's name — which an expansion does not know, because the adapter is the
/// consumer's own type and the region text never names it. Here `A` is inferred
/// from `operands`, where it appears as `&Tensor<A>`.
///
/// The decomposed pair stays public and unchanged. It is what a caller with a
/// concrete adapter in hand uses, and separating the two checks is what lets a
/// test observe each independently.
///
/// The result is constructed through the context of the first declared operand,
/// which is [`build_result`]'s documented contract: a region whose operands live
/// on different contexts is outside this bounded profile.
///
/// # Errors
///
/// Returns whatever [`bind_region`] or [`build_result`] returns, and
/// [`BindError::MalformedRegionFacts`] for a region that declares no operand at
/// all — which has no context to construct a result from.
pub fn bind_and_build<A: TensorAdapter>(
    facts: &RegionFacts,
    operands: &[&Tensor<A>],
) -> Result<A::Value, BindError<A::Error>> {
    let bound = bind_region(facts, operands)?;
    let first = operands.first().ok_or(BindError::MalformedRegionFacts {
        detail: "a region declares no operand, so no context exists to construct its result from",
    })?;
    // The turbofish is the same inference gap this function exists to close,
    // observed from inside: `A` is known here only because `operands` named it.
    build_result::<A>(facts, &bound, first.context())
}

/// Reads the extent one [`AxisRef`] points at.
///
/// The rank check in [`bind_region`] already proved every reported rank equals
/// the declared one, so an out-of-range index here means the emitted facts
/// disagree with the ranks they themselves declared — a defect in the
/// expansion, refused rather than indexed past.
fn extent_at<E>(metadata: &[ValueMetadata], reference: AxisRef) -> Result<u64, BindError<E>> {
    metadata
        .get(reference.operand)
        .and_then(|reported| reported.extents().get(reference.axis))
        .copied()
        .ok_or(BindError::MalformedRegionFacts {
            detail: "a symbol source or obligation names an operand axis the region does not declare",
        })
}

/// Names one [`AxisRef`] the way a diagnostic must name it.
fn named<E>(facts: &RegionFacts, reference: AxisRef) -> Result<OperandAxis, BindError<E>> {
    facts
        .operands
        .get(reference.operand)
        .map(|operand| OperandAxis {
            input: operand.key,
            axis: reference.axis,
        })
        .ok_or(BindError::MalformedRegionFacts {
            detail: "a symbol source or obligation names an operand the region does not declare",
        })
}
