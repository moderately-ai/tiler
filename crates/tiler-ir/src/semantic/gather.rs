//! The governed `Gather` family: reading one tensor at coordinates another tensor holds.
//!
//! **What a `Gather` is, and why it is not one more structural family.** The
//! other four element-moving families — [`super::reindex`], [`super::broadcast`],
//! [`super::slice`], and [`super::concatenate`] — all state a coordinate relation
//! that is a function of the *iteration coordinate alone*. Every one of them can
//! be decided, bounded, and verified before a single element is read. This family
//! cannot: its source coordinate along one named axis is an element of a second
//! operand, so the relation is a function of tensor *data*.
//!
//! That is a different access class rather than a different mapping form, and the
//! corpus says so in two accepted places. [IR](../../../../docs/ir.md)'s Layer 2
//! bounds the index-expression vocabulary and states that "tensor-data-derived
//! indices are rejected", and
//! [ADR 0046](../../../../docs/decisions/0046-separate-logical-access-from-storage-addressing.md)
//! carries the same rejection normatively while separately reserving that
//! "data-dependent gather, scatter, sparse iteration, and data-dependent
//! cardinality require later explicit IR contracts". This module, and
//! [ADR 0107](../../../../docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md),
//! are that later contract for the gather half.
//!
//! # What is admitted here, and what is deliberately left refused
//!
//! **A semantic identity, and nothing below it.** An occurrence of this key states
//! what the program means. It does not state that any index region can express the
//! access, and none can: `AccessData` carries one tensor ordinal, so an access has
//! nowhere to name a second tensor as a coordinate source, and `IndexNode` has no
//! variant that reads one. ADR 0108 was returned for a complete comparison of a
//! verified nested read/value expression with an append-only tagged access; no
//! proposed form has yet shown how it preserves every direct-access verifier
//! guarantee ADR 0046 requires. The family is therefore registered,
//! reference-evaluated, and **fails closed at the request boundary**: no lowering
//! capability resolves it, no fusion role classifies it,
//! and a program stating one compiles no further than the refusal.
//!
//! This is a *labelled draft* public boundary under ADR 0075 until Tom accepts its
//! exact included and excluded surface. Included: the key, the gathered-axis
//! attribute, [`GatherAxis`], [`GatherError`], and the shape rule
//! [`gather_result_shape`]. Excluded and refused by name: scatter, a signed index
//! operand, a data-dependent result shape, and every index type but
//! `tiler::u32@1`.
//!
//! # The four rules Q-SHAPE-007 names as this class's closure condition
//!
//! [Q-SHAPE-007](../../../../docs/open-questions.md#q-shape-007--indirect-gatherscatter-relations)
//! records that closure "needs bounds, duplicate-write, determinism, and
//! validation rules". A read-only gather owes all four *stated*; it owes three
//! *implemented*, because it performs no write.
//!
//! **Bounds.** Every index element must lie in `0..extent` of the gathered axis.
//! This is the one obligation current construction cannot discharge from shape
//! facts alone: the values are data and no tensor element is present at that
//! boundary. The posture is therefore the one [IR](../../../../docs/ir.md) already
//! fixes for a semantic precondition — proved statically, or validated at a
//! *named enforcement boundary* — and the refusal is
//! total: an out-of-range index is never clamped to the axis, never wrapped modulo
//! the extent, and never read at the offset it names. The named boundary that
//! exists today is the reference evaluator, which holds the elements and refuses
//! under [`GatherError::IndexOutOfBounds`]. A physical plan has no such boundary
//! yet, which is one of the reasons this family reaches no plan at all.
//!
//! Two conventions are attested in primary sources and both are refused rather
//! than inherited, on the precedent [`super::slice`] sets for the same reason: a
//! clamping gather and a wrapping gather return a *different tensor* for one
//! program rather than a different diagnostic, so inheriting either would make a
//! frontend's meaning depend on which specification its author had read. Negative
//! indexing is refused a step earlier still, by refusing a signed index operand —
//! see below.
//!
//! **Duplicate-write.** Stated and not implemented, because this family performs
//! no write. It is stated so that admitting scatter later is *additive* rather
//! than a reinterpretation of what is registered here: a gather's read map may be
//! many-to-one — two index elements may name one source row, and
//! [ADR 0046](../../../../docs/decisions/0046-separate-logical-access-from-storage-addressing.md)
//! already admits that reads may be many-to-one — whereas the corresponding write
//! map of a scatter may not, and would owe either an exclusive-ownership proof or
//! an explicit combining contract. Nothing registered here grants that, and
//! nothing registered here forbids a duplicate *index*.
//!
//! **Determinism.** A result element is a source element cloned. Repeated indices
//! read the same source element and produce equal result elements; there is no
//! accumulation, no reassociation freedom, no ordering choice, and no reduction,
//! so no numerical permission is declared and none is needed. The family computes
//! nothing, so an exceptional payload — a non-canonical NaN, a signalling NaN, a
//! signed zero, a subnormal — crosses a gather exactly as it left the source. That
//! is the same bit-preservation obligation the other four element-moving families
//! carry, and this module discharges it the same way: by cloning rather than
//! decoding.
//!
//! **Validation.** Every rule that is a function of the operands' shapes, types,
//! and the attribute is decided at construction under its own named diagnostic —
//! the gathered axis exists, the source has an axis to gather along, the index
//! operand is `tiler::u32@1`, the source and result are `tiler::f32@1`, and the
//! result rank stays inside the governed shape profile.
//!
//! # Why the index operand is unsigned, and only `tiler::u32@1`
//!
//! A signed index type is refused by name under
//! [`GatherError::SignedIndexUnsupported`] rather than admitted and then bounded
//! below zero, because a signed index raises the negative-indexing convention —
//! `-1` meaning the last row — which is a *second* place the primary authorities
//! diverge and which this family does not answer. Refusing the type refuses the
//! question; admitting the type and rejecting negative values would answer it
//! silently in one direction.
//!
//! `tiler::u32@1` alone is admitted among the unsigned identities, and the
//! widening path is deliberately additive rather than pre-fixed: a wider or
//! narrower index type is one more admitted signature under *this same key*,
//! registered as one more reference capability, exactly as [`super::concatenate`]
//! enumerates one capability per admitted arity. It is not a second key. The
//! pinned workload's vocabulary is 151,936, which needs eighteen bits, so
//! `tiler::u8@1` and `tiler::u16@1` could not carry it and `tiler::u32@1` is the
//! narrowest identity that can.
//!
//! # The shape rule
//!
//! The result composes the index operand's shape into the position the gathered
//! axis occupied. For a source of rank `n` gathered on axis `a` by an index
//! operand of rank `m`, the result has rank `n - 1 + m`:
//!
//! ```text
//! source  [d₀ … d_{a-1}, d_a, d_{a+1} … d_{n-1}]
//! index   [i₀ … i_{m-1}]
//! result  [d₀ … d_{a-1}, i₀ … i_{m-1}, d_{a+1} … d_{n-1}]
//! ```
//!
//! The pinned occurrence is the whole of the workload's motivation and lands
//! exactly here: a `[151936, 1024]` source gathered on axis 0 by a `[T]` index
//! operand yields `[T, 1024]`.
//!
//! **A rank-zero index operand is admitted** and drops the gathered axis, which is
//! how a single-row selection is written. It is not refused by analogy with
//! [`super::slice`]'s `NoRestrictedAxis`, because that refusal exists to stop an
//! occurrence that *returns its operand*, and a rank-zero gather does not: it
//! returns one row chosen by data the graph cannot see.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::shape::{Axis, Extent, Shape};

use super::registry::standard_conformance;
use super::{
    AttributeFieldId, CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind,
    CanonicalValueView, F32, NormativeDefinitionRef, OpKey, OperationArity,
    OperationAttributeSchema, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, ProviderDiagnosticCode, RegistryError, ResolvedValueType,
    SemanticRegistryRegistrar, TypeKey, ValueFact,
};

/// Stable field ID carrying the gathered axis on the occurrence.
pub const GATHER_AXIS_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// Fact field naming what a gather does to values.
///
/// The six fields below are the family's semantic signature. Every one is
/// unconditional on this definition: absence is a malformed record, never a
/// default. Two of them exist because this family is the first whose coordinate
/// relation is not decidable from the graph, and a reader needs that stated in
/// canonical attribute bytes rather than in a doc comment.
pub const GATHER_FACT_VALUE_BEHAVIOUR: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming that the coordinate relation is derived from tensor data.
pub const GATHER_FACT_COORDINATE_SOURCE: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming what an out-of-range index does.
pub const GATHER_FACT_OUT_OF_BOUNDS: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field naming what a repeated index does.
pub const GATHER_FACT_DUPLICATE_INDEX: AttributeFieldId = AttributeFieldId::new(4);
/// Fact field naming the family's determinism obligation.
pub const GATHER_FACT_DETERMINISM: AttributeFieldId = AttributeFieldId::new(5);
/// Fact field naming what this family claims about storage.
pub const GATHER_FACT_STORAGE_CLAIM: AttributeFieldId = AttributeFieldId::new(6);

/// Returns the governed binary32 gather operation key.
///
/// The key names the *gathered* value type, as every other keyed family in this
/// profile names the type it computes over. The index operand's type is part of
/// the admitted signature rather than part of the key, so widening the admitted
/// index identities is an additive registration under this key rather than a
/// second family.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn gather_f32_op() -> OpKey {
    OpKey::new("tiler", "gather-f32", 1).expect("the governed gather key is valid")
}

/// Returns the one admitted index-operand identity, `tiler::u32@1`.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn gather_index_resolved_type() -> ResolvedValueType {
    ResolvedValueType::nominal(
        TypeKey::new("tiler", "u32", 1).expect("the governed gather index key is valid"),
    )
}

/// A typed refusal of one gather occurrence.
///
/// Every variant is one named admission rule. [`GatherError::IndexOutOfBounds`] is
/// the one that is *not* decidable here, and it is carried in this enum anyway so
/// that the named enforcement boundary refuses under the same vocabulary the
/// construction-time rules use rather than inventing a second one.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GatherError {
    /// The occurrence did not supply exactly a source and an index operand.
    OperandCount {
        /// Operands supplied.
        operands: usize,
    },
    /// The gathered-axis attribute was not a canonical unsigned 32-bit value.
    MalformedAxisAttribute,
    /// The source operand has no axis to gather along.
    ///
    /// A rank-zero source has no coordinate for an index to select, so there is
    /// nothing an occurrence over one could mean.
    SourceIsRankZero,
    /// The gathered axis does not exist on the source.
    AxisOutOfRange {
        /// The named axis.
        axis: Axis,
        /// The source's rank.
        rank: usize,
    },
    /// The source or result operand is not `tiler::f32@1`.
    ///
    /// The binary32 gather admits no implicit promotion and converts no operand.
    SourceNotF32,
    /// The index operand is a signed integer identity.
    ///
    /// Refused under its own rule rather than as an unadmitted type, because the
    /// reason is a reserved convention rather than a narrow profile: a signed
    /// index raises negative indexing, which is a second place the primary
    /// authorities diverge and which this family does not answer. Refusing the
    /// type refuses the question.
    SignedIndexUnsupported,
    /// The index operand is not the one admitted index identity.
    UnadmittedIndexType,
    /// An index element lies outside the gathered axis.
    ///
    /// Not decidable at construction: the values are tensor data. It is refused at
    /// the named enforcement boundary — never clamped to the axis and never
    /// wrapped modulo its extent.
    IndexOutOfBounds {
        /// Position of the offending element in the index operand, row-major.
        position: usize,
        /// The value it holds.
        value: u64,
        /// The gathered axis's extent, which it must stay below.
        extent: u64,
    },
    /// The result shape exceeded the governed rank profile.
    ResultShape(crate::shape::ShapeError),
}

impl GatherError {
    /// Returns the stable provider diagnostic code naming this refusal.
    ///
    /// Each rule has its own code, so a caller reads *which* rule refused from the
    /// code rather than by matching on a message.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::OperandCount { .. } => "gather.operands.arity",
            Self::MalformedAxisAttribute => "gather.axis.malformed-attribute",
            Self::SourceIsRankZero => "gather.source.rank-zero",
            Self::AxisOutOfRange { .. } => "gather.axis.out-of-range",
            Self::SourceNotF32 => "gather.source.implicit-promotion",
            Self::SignedIndexUnsupported => "gather.index.signed-unsupported",
            Self::UnadmittedIndexType => "gather.index.unadmitted-type",
            Self::IndexOutOfBounds { .. } => "gather.index.out-of-bounds",
            Self::ResultShape(_) => "gather.result-shape",
        }
    }
}

impl fmt::Display for GatherError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperandCount { operands } => write!(
                formatter,
                "a gather takes a source and an index operand and {operands} were supplied"
            ),
            Self::MalformedAxisAttribute => {
                formatter.write_str("the gathered-axis attribute is malformed")
            }
            Self::SourceIsRankZero => formatter.write_str(
                "the source has rank zero, so it has no axis to gather along and no coordinate an index could select",
            ),
            Self::AxisOutOfRange { axis, rank } => write!(
                formatter,
                "axis {} does not exist on a source of rank {rank}",
                axis.get()
            ),
            Self::SourceNotF32 => formatter.write_str(
                "the source is not tiler::f32@1; the binary32 gather admits no implicit promotion and converts no operand",
            ),
            Self::SignedIndexUnsupported => formatter.write_str(
                "a signed index operand is reserved and not admitted: a signed index raises negative indexing, a convention the primary authorities diverge on and this family does not answer, so the type is refused rather than the values",
            ),
            Self::UnadmittedIndexType => formatter.write_str(
                "the index operand is not tiler::u32@1, the one admitted index identity; a wider or narrower index type is an additive admission under this same key rather than a different one",
            ),
            Self::IndexOutOfBounds {
                position,
                value,
                extent,
            } => write!(
                formatter,
                "index element {position} holds {value} and the gathered axis has extent {extent}, so it names no coordinate; an out-of-range index is refused rather than clamped to the axis or wrapped modulo its extent"
            ),
            Self::ResultShape(source) => {
                write!(formatter, "the result shape is not admitted: {source}")
            }
        }
    }
}

impl Error for GatherError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ResultShape(source) => Some(source),
            _ => None,
        }
    }
}

/// A validated gathered axis, decided against the source it gathers along.
///
/// There is no unchecked constructor and [`gather_result_shape`] is the only path
/// to a result, so holding one is evidence that the axis exists on the source it
/// was decided against.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GatherAxis(Axis);

impl GatherAxis {
    /// Returns the axis this occurrence gathers along.
    #[must_use]
    pub const fn axis(self) -> Axis {
        self.0
    }

    /// Returns the axis as a host position.
    #[must_use]
    pub fn position(self) -> usize {
        usize::try_from(self.0.get()).unwrap_or(usize::MAX)
    }
}

/// Decodes the gathered axis exactly as an occurrence carries it.
///
/// # Errors
///
/// Returns [`GatherError::MalformedAxisAttribute`] when the attribute is not a
/// canonical unsigned 32-bit value. Whether the axis exists is a question about
/// the source and is decided by [`gather_result_shape`].
pub fn gather_axis(value: &CanonicalValue) -> Result<Axis, GatherError> {
    let CanonicalValueView::Unsigned {
        width: CanonicalIntegerWidth::Bits32,
        bits,
    } = value.view()
    else {
        return Err(GatherError::MalformedAxisAttribute);
    };
    u32::try_from(bits)
        .map(Axis::new)
        .map_err(|_| GatherError::MalformedAxisAttribute)
}

/// Decides one gather against its operands' shapes and derives the result.
///
/// The result composes the index operand's shape into the position the gathered
/// axis occupied, so the result rank is `source.rank() - 1 + index.rank()` and
/// nothing about it is declared by a caller. A rank-zero index operand is
/// admitted and drops the gathered axis.
///
/// This decides everything about the occurrence that the *shapes* fix. It
/// deliberately decides nothing about the index operand's element values, which
/// are tensor data and are the named enforcement boundary's to refuse.
///
/// # Errors
///
/// Returns [`GatherError`] naming the violated rule.
pub fn gather_result_shape(
    axis: Axis,
    source: &Shape,
    index: &Shape,
) -> Result<(GatherAxis, Shape), GatherError> {
    if source.rank() == 0 {
        return Err(GatherError::SourceIsRankZero);
    }
    let position = usize::try_from(axis.get()).unwrap_or(usize::MAX);
    if position >= source.rank() {
        return Err(GatherError::AxisOutOfRange {
            axis,
            rank: source.rank(),
        });
    }
    let extents = source.extents();
    // Composed in one pass rather than by splicing two slices, so the result's
    // axis order is read off this function directly: the source's axes before the
    // gathered one, then the whole index shape, then the source's axes after it.
    let mut result: Vec<Extent> = Vec::new();
    result.extend_from_slice(extents.get(..position).unwrap_or_default());
    result.extend_from_slice(index.extents());
    result.extend_from_slice(
        extents
            .get(position.saturating_add(1)..)
            .unwrap_or_default(),
    );
    let shape = Shape::try_new(result).map_err(GatherError::ResultShape)?;
    Ok((GatherAxis(axis), shape))
}

/// Decides one index element against the gathered axis it selects along.
///
/// This is the *bounds* half of the family's obligation, factored out of the
/// reference evaluator so that a second enforcement boundary — a host-side
/// pre-dispatch validation, when one exists — refuses under the same rule and the
/// same diagnostic rather than restating it.
///
/// # Errors
///
/// Returns [`GatherError::IndexOutOfBounds`] naming the position, the value, and
/// the extent. It never clamps and never wraps.
pub fn decide_gather_index(
    position: usize,
    value: u64,
    extent: Extent,
) -> Result<usize, GatherError> {
    if value >= extent.get() {
        return Err(GatherError::IndexOutOfBounds {
            position,
            value,
            extent: extent.get(),
        });
    }
    usize::try_from(value).map_err(|_| GatherError::IndexOutOfBounds {
        position,
        value,
        extent: extent.get(),
    })
}

/// Registers the governed gather family.
pub(super) fn register_standard_gather(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        gather_f32_op(),
        OperationSchema::new(
            OperationArity::exact(2),
            OperationArity::exact(1),
            [OperationAttributeSchema::required(
                GATHER_AXIS_ATTRIBUTE,
                CanonicalValueKind::Unsigned,
            )],
        )
        .expect("the governed gather schema is valid"),
        NormativeDefinitionRef::new(GATHER_F32_NORMATIVE_DEFINITION)?,
        OperationDefinitionFacts::new(gather_facts()),
        standard_conformance("gather-f32"),
        OperationEffect::Pure,
        Arc::new(GatherF32),
    ))
    // No algebraic capability is declared, deliberately, and the reason is
    // stronger here than it is for the four element-moving families. A gather
    // performs no arithmetic, so it has no associativity or commutativity *of
    // rounding* to declare; and unlike a reindex or a broadcast it additionally
    // has no permutation or replication law a pass could exploit, because the
    // relation it states is not knowable until the index operand's elements are.
}

/// The complete normative definition of `tiler::gather-f32@1`.
///
/// Held as a constant rather than written inline because it is where the bounds
/// posture, the duplicate-index rule, and the two reserved refusals are stated,
/// and a reader looking for any of them should find it under a name rather than
/// inside a registration call.
const GATHER_F32_NORMATIVE_DEFINITION: &str = concat!(
    "tiler::gather-f32@1; a total binary32 read of one source operand at coordinates a second operand ",
    "holds. The relation is derived from tensor data and is therefore outside the admitted index-expression ",
    "vocabulary, which rejects tensor-data-derived indices; this key states the semantics of that access ",
    "class and claims nothing about any index region, physical plan, or target being able to realize it. ",
    "Operands are a tiler::f32@1 source and a tiler::u32@1 index operand, in that order, with one named ",
    "gathered axis carried as a canonical unsigned attribute. The result composes the index operand's shape ",
    "into the position the gathered axis occupied: for a source of rank n gathered on axis a by an index ",
    "operand of rank m, the result has rank n - 1 + m, being the source's axes before a, then the index ",
    "operand's axes, then the source's axes after a. The result shape is derived and never declared by a ",
    "caller. A rank-zero index operand is admitted and drops the gathered axis. A rank-zero source is ",
    "refused, having no axis to gather along. ",
    "Every result element is a source element unchanged: no value is computed, converted, rounded, or ",
    "canonicalized, so an exceptional payload — a non-canonical NaN, a signalling NaN, a signed zero, a ",
    "subnormal — arrives at the result exactly as it left the source. ",
    "Bounds: every index element must lie in 0..extent of the gathered axis. The values are tensor data, so ",
    "this obligation is not decidable at construction; it is proved statically or validated at a named ",
    "enforcement boundary, and a semantic validation failure is never a plan miss. An out-of-range index is ",
    "refused naming the element position, the value, and the extent. It is never clamped to the axis and ",
    "never wrapped modulo its extent: both conventions are attested in primary sources and they return a ",
    "different tensor for one program rather than a different diagnostic. ",
    "Duplicate indices are admitted. The read map may be many-to-one, so two index elements may name one ",
    "source coordinate and the result then holds two copies of one element, bit for bit. ",
    "Determinism: the result is a total function of the source, the index operand, and the gathered axis. ",
    "There is no accumulation, no reassociation freedom, no ordering choice, and no reduction, so this family ",
    "declares no numerical permission and needs none. ",
    "The corresponding duplicate-*write* rule is stated and not implemented, because this family performs no ",
    "write. It is stated so that admitting a scatter later is additive rather than a reinterpretation of this ",
    "definition: a scatter's write map may not be many-to-one without either an exclusive-ownership proof or ",
    "an explicit combining contract, and nothing here grants one. Scatter and any data-dependent output shape ",
    "are outside this family. ",
    "A signed index operand is reserved and refused by name, because a signed index raises negative indexing, ",
    "which this family does not answer; refusing the type refuses the question rather than answering it ",
    "silently. tiler::u32@1 is the one admitted index identity, and a wider or narrower one is an additive ",
    "admission under this same key rather than a second family. ",
    "This operation makes no claim that storage was copied, viewed, or left alone: it states a logical ",
    "coordinate relation, and every physical realization of it remains a planning outcome.",
);

fn gather_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            GATHER_FACT_VALUE_BEHAVIOUR,
            fact("none-every-result-element-is-a-source-element-unchanged"),
        ),
        CanonicalField::new(
            GATHER_FACT_COORDINATE_SOURCE,
            fact(
                "tensor-data-derived-from-the-index-operand-outside-the-admitted-index-vocabulary",
            ),
        ),
        CanonicalField::new(
            GATHER_FACT_OUT_OF_BOUNDS,
            fact("refused-at-a-named-enforcement-boundary-never-clamped-and-never-wrapped"),
        ),
        CanonicalField::new(
            GATHER_FACT_DUPLICATE_INDEX,
            fact("admitted-the-read-map-may-be-many-to-one-and-the-write-rule-is-scatters"),
        ),
        CanonicalField::new(
            GATHER_FACT_DETERMINISM,
            fact("total-function-of-source-index-and-axis-with-no-accumulation-or-order-freedom"),
        ),
        CanonicalField::new(
            GATHER_FACT_STORAGE_CLAIM,
            fact("none-no-copy-view-or-materialization-is-claimed"),
        ),
    ])
    .expect("the governed gather facts are canonical")
}

fn fact(value: &str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("a governed gather fact is bounded")
}

struct GatherF32;

impl OperationInferencer for GatherF32 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "gather.attributes",
                "a gather requires exactly the gathered-axis attribute".to_owned(),
            ));
        }
        let Some(value) = attributes.get(GATHER_AXIS_ATTRIBUTE) else {
            return Err(op_error(
                "gather.attributes",
                "a gather requires exactly the gathered-axis attribute".to_owned(),
            ));
        };
        // The attribute's own rule is decided before anything about the operands,
        // so a malformed axis is refused under its own name rather than under
        // whichever shape check happened to notice first.
        let axis = gather_axis(value).map_err(|error| rejection(&error))?;
        let [source, index] = operands else {
            return Err(rejection(&GatherError::OperandCount {
                operands: operands.len(),
            }));
        };
        let f32_type = F32::resolved_type();
        if source.resolved_type() != &f32_type {
            return Err(rejection(&GatherError::SourceNotF32));
        }
        // The signed identities are refused before the general unadmitted-type
        // rule, so a caller reaching for negative indexing is told that the
        // convention is reserved rather than that the type is narrow.
        if is_signed_integer_identity(index.resolved_type()) {
            return Err(rejection(&GatherError::SignedIndexUnsupported));
        }
        if index.resolved_type() != &gather_index_resolved_type() {
            return Err(rejection(&GatherError::UnadmittedIndexType));
        }
        // A gather splices the index boundary into the source boundary at the
        // gathered axis and bounds every index against the gathered extent.
        // Both are facts about the extents themselves rather than about their
        // equality, so a symbolic operand is declined by name rather than
        // spliced on spelling.
        let (_, shape) = gather_result_shape(
            axis,
            request.static_operand_shape(0)?,
            request.static_operand_shape(1)?,
        )
        .map_err(|error| rejection(&error))?;
        outputs.try_push(ValueFact::new(f32_type, shape))
    }
}

/// Returns whether a resolved identity is one of the governed signed integers.
///
/// Matched by key rather than by inspecting the catalog's scalar kind, because
/// this is a *refusal* list whose membership is a decision this family makes:
/// admitting a signed index later means removing a name here deliberately, not
/// discovering that a catalog entry changed shape.
fn is_signed_integer_identity(resolved: &ResolvedValueType) -> bool {
    ["i4", "i8", "i16", "i32", "i64"].iter().any(|name| {
        TypeKey::new("tiler", name, 1).is_ok_and(|key| resolved == &ResolvedValueType::nominal(key))
    })
}

fn rejection(error: &GatherError) -> OperationInferenceError {
    op_error(error.diagnostic_code(), error.to_string())
}

fn op_error(code: &str, message: String) -> OperationInferenceError {
    OperationInferenceError::new(
        ProviderDiagnosticCode::new(code).expect("a governed diagnostic code is canonical"),
        message,
    )
    .expect("a governed diagnostic message is canonical")
}

#[cfg(test)]
mod tests;
