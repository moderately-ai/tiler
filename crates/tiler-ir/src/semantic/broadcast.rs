//! The governed `Broadcast` family and its explicit axis mapping.
//!
//! **What a `Broadcast` is.** The one family that states a many-to-one
//! output-to-input coordinate relation: several result coordinates read one
//! operand element. Every result axis is accounted for by exactly one entry of an
//! explicit mapping, and every entry says which of three relations it is, so a
//! reader never has to infer a replication from a shape.
//!
//! **Two many-to-one relations, deliberately distinct.**
//! [`BroadcastAxisSource::Replicate`] is a rank pad — a result axis with no
//! operand axis behind it at all — and [`BroadcastAxisSource::StretchUnit`] is an
//! extent-one operand axis widened in place. They are not the same relation:
//! one consumes an operand axis and one does not, and collapsing them would make
//! `[2, 1] -> [2, 64]` and `[2] -> [2, 64]` indistinguishable in the attribute
//! while they remain different programs. The pinned workload needs both — the
//! normalization weight `[1024]` against `[T, 1024]` is a rank pad, and the
//! rotary sign operand `[2, 1]` against `[…, 2, 64]` is a unit stretch.
//!
//! **Nothing is normalized.** A mapping that does not account for every result
//! axis, that drops or reorders an operand axis, that claims a stretch of an axis
//! whose extent is not one, or that states no many-to-one relation at all is
//! refused under its own named rule. In particular an implicit rank pad — a
//! mapping whose entries count the *operand's* axes rather than the *result's* —
//! is a length disagreement and is refused rather than padded, and an
//! extent-one stretch presented as an ordinary axis correspondence is an extent
//! disagreement and is refused rather than stretched.
//!
//! **What this family is not.** The narrow rank-zero admission on
//! `tiler::add-f32@1` and `tiler::multiply-f32@1` is a shape rule inside those
//! signatures and synthesizes no node here; it covers a rank-zero operand alone.
//! Rank padding, extent-one stretching, and every other many-to-one mapping
//! require an occurrence of this key, in every signature and at every rank. A map
//! that reorders axes is a [`super::reindex`], and one that drops an operand axis
//! is a reduction or a slice; each is refused by name.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::identity::push_slice;
use crate::program::abi::AvailabilityPhase;
use crate::shape::{Axis, Extent, ExtentSources, Shape, ShapeSymbol, SourcedExtent, SourcedShape};

use super::registry::standard_conformance;
use super::{
    AttributeFieldId, CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind,
    CanonicalValueView, F32, NormativeDefinitionRef, OpKey, OperationArity,
    OperationAttributeSchema, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, ProviderDiagnosticCode, RegistryError,
    SemanticRegistryRegistrar, TypeIdentityError, ValueFact,
};

/// Maximum result axes one broadcast axis mapping may account for.
///
/// The same bound a canonical sequence admits, so an oversized mapping is
/// refused with a broadcast-shaped diagnostic rather than an anonymous
/// canonical-bound one.
pub const MAX_BROADCAST_MAPPING_AXES: usize = super::types::MAX_RESOLVED_TYPE_ITEMS;

/// Stable field ID carrying the canonical axis mapping on the broadcast.
pub const BROADCAST_AXIS_MAPPING_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// Mapping-record field carrying the declared result extents.
///
/// The result shape is *declared* rather than derived, because a replicated axis
/// has no operand axis to derive an extent from. Every declared extent is then
/// checked against the relation its entry states, so declaring one does not make
/// it true.
pub const BROADCAST_MAPPING_RESULT_EXTENTS: AttributeFieldId = AttributeFieldId::new(1);
/// Mapping-record field carrying one source entry per result axis.
pub const BROADCAST_MAPPING_SOURCES: AttributeFieldId = AttributeFieldId::new(2);

/// Source-record field naming which of the three relations this entry states.
///
/// The two constants below are fields of the *broadcast source record*, a
/// different record from the mapping record that contains it and from every
/// other record in this corpus; equal integers across records are unrelated.
pub const BROADCAST_SOURCE_RELATION: AttributeFieldId = AttributeFieldId::new(1);
/// Source-record field naming the operand axis a non-replicating entry consumes.
pub const BROADCAST_SOURCE_AXIS: AttributeFieldId = AttributeFieldId::new(2);

/// Fact field naming what a broadcast does to values.
///
/// The four fields below are the family's semantic signature, and none of them is
/// numerical, because a broadcast performs no arithmetic. Every one is
/// unconditional on this definition: absence is a malformed record, never a
/// default.
pub const BROADCAST_FACT_VALUE_BEHAVIOUR: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming the mapping's totality and directionality guarantee.
pub const BROADCAST_FACT_MAPPING_CLASS: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming what this family claims about storage.
pub const BROADCAST_FACT_STORAGE_CLAIM: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field naming the closed set of admitted axis relations.
pub const BROADCAST_FACT_ADMITTED_RELATIONS: AttributeFieldId = AttributeFieldId::new(4);

/// Canonical name of the one-to-one axis correspondence.
///
/// The three names below are canonical identity, not display text: a source
/// record carries the exact string, so respelling one changes every occurrence's
/// attribute bytes. An unrecognized name is refused rather than mapped to a
/// nearest match.
pub const BROADCAST_RELATION_FROM_OPERAND: &str = "from-operand";
/// Canonical name of the in-place widening of an extent-one operand axis.
pub const BROADCAST_RELATION_STRETCH_UNIT: &str = "stretch-unit";
/// Canonical name of the result axis with no operand axis behind it.
pub const BROADCAST_RELATION_REPLICATE: &str = "replicate";

/// Domain separator of a canonical broadcast axis-mapping encoding.
///
/// `v2` rather than `v1`: declared result extents are sourced, so a mapping
/// encodes a tag-and-payload per axis instead of a bare unsigned literal. The
/// domain steps with that grammar rather than letting two encodings share one
/// separator.
const BROADCAST_AXIS_MAPPING_DOMAIN: &[u8] = b"tiler.broadcast-axis-mapping.v2\0";

/// Returns the governed binary32 broadcast operation key.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn broadcast_f32_op() -> OpKey {
    OpKey::new("tiler", "broadcast-f32", 2).expect("the governed broadcast key is valid")
}

/// What one result axis of a broadcast reads.
///
/// Deliberately **not** `#[non_exhaustive]`, on the precedent
/// [`super::OperationEffect`] sets: the governed index-access lowering maps this
/// vocabulary *totally* onto a coordinate or its absence, and no wildcard
/// coordinate is derivable from a relation it has not seen. A fourth relation
/// must be a build error there rather than a silent fall through.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BroadcastAxisSource {
    /// One-to-one with the named operand axis, whose extent it must equal.
    FromOperand(Axis),
    /// A many-to-one widening of the named operand axis, whose extent must be one.
    StretchUnit(Axis),
    /// A many-to-one replication with no operand axis behind it: a rank pad.
    Replicate,
}

impl BroadcastAxisSource {
    /// Returns the operand axis this entry consumes, if it consumes one.
    #[must_use]
    pub const fn operand_axis(self) -> Option<Axis> {
        match self {
            Self::FromOperand(axis) | Self::StretchUnit(axis) => Some(axis),
            Self::Replicate => None,
        }
    }

    /// Returns whether this entry states a many-to-one relation.
    #[must_use]
    pub const fn is_many_to_one(self) -> bool {
        matches!(self, Self::StretchUnit(_) | Self::Replicate)
    }

    /// Returns the canonical name this relation carries in its source record.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::FromOperand(_) => BROADCAST_RELATION_FROM_OPERAND,
            Self::StretchUnit(_) => BROADCAST_RELATION_STRETCH_UNIT,
            Self::Replicate => BROADCAST_RELATION_REPLICATE,
        }
    }
}

impl fmt::Display for BroadcastAxisSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FromOperand(axis) | Self::StretchUnit(axis) => {
                write!(
                    formatter,
                    "{} of operand axis {}",
                    self.canonical_name(),
                    axis.get()
                )
            }
            Self::Replicate => formatter.write_str(BROADCAST_RELATION_REPLICATE),
        }
    }
}

/// Which part of a malformed mapping attribute was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BroadcastAttributeSubject {
    /// The attribute was not the two-field mapping record.
    MappingRecord,
    /// The result-extent field was not a sequence of canonical extents.
    ResultExtents,
    /// The source field was not a sequence.
    SourceSequence,
    /// One source entry was not a well-formed relation record.
    SourceRecord,
    /// One relation name was not canonical UTF-8.
    Relation,
    /// One operand axis was not a canonical unsigned 32-bit value.
    Axis,
}

impl fmt::Display for BroadcastAttributeSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MappingRecord => formatter.write_str("mapping record"),
            Self::ResultExtents => formatter.write_str("result-extent sequence"),
            Self::SourceSequence => formatter.write_str("source sequence"),
            Self::SourceRecord => formatter.write_str("source record"),
            Self::Relation => formatter.write_str("relation name"),
            Self::Axis => formatter.write_str("operand axis"),
        }
    }
}

/// A typed refusal of one broadcast axis mapping.
///
/// Every variant is one named admission rule. [`BroadcastAxisMapping`] has no
/// unchecked constructor and [`BroadcastAxisMapping::result_shape`] is the only
/// path to a result, so holding a result is evidence that the mapping's own rules
/// and its rules against the operand were all decided.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BroadcastMappingError {
    /// The named relation is not one this family admits.
    UnadmittedRelation {
        /// The rejected name, truncated to a bounded prefix.
        name: String,
    },
    /// The mapping does not state one source per declared result axis.
    ///
    /// This is where an implicit rank pad lands: a mapping written against the
    /// operand's rank rather than the result's leaves result axes unaccounted
    /// for, and the family refuses rather than filling them in.
    SourceCountMismatch {
        /// Sources the mapping states.
        sources: usize,
        /// Result axes it declares.
        result_axes: usize,
    },
    /// An operand axis was named out of order, twice, or not at all.
    ///
    /// A broadcast preserves the operand's axis order and consumes every operand
    /// axis exactly once. A mapping that reorders them is a reindex composed with
    /// a broadcast, and one that skips an operand axis drops data; neither is
    /// this family, and folding either in would let one attribute denote two
    /// different programs.
    OperandAxisOutOfOrder {
        /// The axis the mapping named.
        named: Axis,
        /// The axis the mapping was required to name there.
        expected: Axis,
    },
    /// The mapping consumed fewer operand axes than the operand has.
    OperandAxesUnconsumed {
        /// Operand axes the mapping consumed.
        consumed: usize,
        /// Operand axes the occurrence supplies.
        rank: usize,
    },
    /// A one-to-one entry named a result extent its operand axis does not have.
    ///
    /// This is where an extent-one stretch presented without an axis mapping
    /// lands: `[2, 1]` against a `[2, 64]` result stated as two one-to-one
    /// correspondences disagrees at the second axis, and the family refuses
    /// rather than stretching an axis the mapping did not say to stretch.
    ExtentDisagreement {
        /// The result axis.
        result_axis: usize,
        /// The extent it declares.
        declared: u64,
        /// The operand axis behind it.
        operand_axis: Axis,
        /// That axis's extent.
        operand_extent: u64,
    },
    /// A stretch named an operand axis whose extent is not one.
    StretchSourceNotUnit {
        /// The named operand axis.
        operand_axis: Axis,
        /// Its extent.
        extent: u64,
    },
    /// A stretch or replication does not widen, so it states no many-to-one relation.
    ///
    /// A result axis of extent one duplicates nothing: written as a stretch it is
    /// the one-to-one correspondence `from-operand`, and written as a replication
    /// it is a reindex's unit-axis insertion. A result axis of extent zero
    /// duplicates nothing either and is not a widening of a unit axis. Both are
    /// refused so that one relation has one spelling.
    RelationDoesNotWiden {
        /// The result axis.
        result_axis: usize,
        /// The relation it states.
        relation: &'static str,
        /// The extent it declares.
        declared: u64,
    },
    /// The mapping states no many-to-one relation, so it denotes no broadcast.
    ///
    /// A mapping of nothing but one-to-one correspondences returns its operand.
    /// It is refused for the reason the contraction refuses a structure that sums
    /// over nothing: an operation that denotes no member of its family belongs to
    /// a different family, or to none.
    NoManyToOneRelation,
    /// The mapping accounted for more result axes than one canonical sequence admits.
    TooManyAxes {
        /// First rejected axis count.
        axes: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// The attribute was not a well-formed mapping record.
    MalformedAttribute {
        /// The rejected part.
        subject: BroadcastAttributeSubject,
    },
    /// The mapping exceeded a canonical structural bound.
    CanonicalBound(TypeIdentityError),
    /// The result shape exceeded the governed rank profile.
    ResultShape(crate::shape::ShapeError),
    /// A declared extent named a symbol the program environment does not declare.
    UndeclaredSymbol {
        /// The symbol the rejected extent named.
        symbol: ShapeSymbol,
    },
    /// A declared extent named a symbol whose value arrives after it is needed.
    SourceTooLate {
        /// The symbol the rejected extent named.
        symbol: ShapeSymbol,
        /// The phase its root binding declares.
        available: AvailabilityPhase,
        /// The last phase an extent may be sourced from.
        ceiling: AvailabilityPhase,
    },
    /// A symbolic many-to-one extent is not proved positive.
    ///
    /// A symbol whose interval includes zero is not a defined widening: the
    /// family refuses it at application, before the graph mutates. Distinct from
    /// [`Self::RelationDoesNotWiden`], which is the literal canonicality rule.
    ExtentNotProvedPositive {
        /// The result axis.
        result_axis: usize,
        /// The extent it declares.
        declared: SourcedExtent,
    },
    /// The environment does not prove a one-to-one pair is one extent.
    ///
    /// Distinct from [`Self::ExtentDisagreement`]: that variant is two different
    /// literals, and this one is a pair the environment does not prove equal.
    ExtentsNotProvedEqual {
        /// The result axis.
        result_axis: usize,
        /// The extent the mapping declares.
        declared: SourcedExtent,
        /// The operand extent behind it.
        operand: SourcedExtent,
    },
    /// The environment does not prove a stretched operand extent is one.
    ///
    /// Distinct from [`Self::StretchSourceNotUnit`], which fires when the
    /// operand extent is a literal other than one.
    StretchSourceNotProvedUnit {
        /// The named operand axis.
        operand_axis: Axis,
        /// The operand extent the mapping asked about.
        extent: SourcedExtent,
    },
}

impl BroadcastMappingError {
    /// Returns the stable provider diagnostic code naming this refusal.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::UnadmittedRelation { .. } => "broadcast.mapping.unadmitted-relation",
            Self::SourceCountMismatch { .. } => "broadcast.mapping.source-count",
            Self::OperandAxisOutOfOrder { .. } => "broadcast.mapping.operand-axis-out-of-order",
            Self::OperandAxesUnconsumed { .. } => "broadcast.mapping.operand-axes-unconsumed",
            Self::ExtentDisagreement { .. } => "broadcast.mapping.extent-disagreement",
            Self::StretchSourceNotUnit { .. } => "broadcast.mapping.stretch-source-not-unit",
            Self::RelationDoesNotWiden { .. } => "broadcast.mapping.relation-does-not-widen",
            Self::NoManyToOneRelation => "broadcast.mapping.no-many-to-one-relation",
            Self::TooManyAxes { .. } => "broadcast.mapping.too-many-axes",
            Self::MalformedAttribute { .. } => "broadcast.mapping.malformed-attribute",
            Self::CanonicalBound(_) => "broadcast.mapping.canonical-bound",
            Self::ResultShape(_) => "broadcast.mapping.result-shape",
            Self::UndeclaredSymbol { .. } => "broadcast.mapping.undeclared-symbol",
            Self::SourceTooLate { .. } => "broadcast.mapping.source-too-late",
            Self::ExtentNotProvedPositive { .. } => "broadcast.mapping.extent-not-proved-positive",
            Self::ExtentsNotProvedEqual { .. } => "broadcast.mapping.extent-not-proved-equal",
            Self::StretchSourceNotProvedUnit { .. } => {
                "broadcast.mapping.stretch-source-not-proved-unit"
            }
        }
    }
}

impl fmt::Display for BroadcastMappingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnadmittedRelation { name } => write!(
                formatter,
                "{name} is not an admitted broadcast axis relation; the admitted relations are {BROADCAST_RELATION_FROM_OPERAND}, {BROADCAST_RELATION_STRETCH_UNIT}, and {BROADCAST_RELATION_REPLICATE}"
            ),
            Self::SourceCountMismatch {
                sources,
                result_axes,
            } => write!(
                formatter,
                "the mapping states {sources} sources and declares {result_axes} result axes, so it does not account for every result axis"
            ),
            Self::OperandAxisOutOfOrder { named, expected } => write!(
                formatter,
                "the mapping names operand axis {} where it must name operand axis {}, and a broadcast consumes every operand axis exactly once in order",
                named.get(),
                expected.get()
            ),
            Self::OperandAxesUnconsumed { consumed, rank } => write!(
                formatter,
                "the mapping consumes {consumed} of the operand's {rank} axes, and an operand axis with no result axis behind it is a reduction or a slice rather than a broadcast"
            ),
            Self::ExtentDisagreement {
                result_axis,
                declared,
                operand_axis,
                operand_extent,
            } => write!(
                formatter,
                "result axis {result_axis} declares extent {declared} and reads operand axis {} of extent {operand_extent}, and a {BROADCAST_RELATION_FROM_OPERAND} correspondence requires them to be equal; a widening of an extent-one operand axis states {BROADCAST_RELATION_STRETCH_UNIT} instead",
                operand_axis.get()
            ),
            Self::StretchSourceNotUnit {
                operand_axis,
                extent,
            } => write!(
                formatter,
                "operand axis {} has extent {extent}, and only an extent-one axis may be stretched",
                operand_axis.get()
            ),
            Self::RelationDoesNotWiden {
                result_axis,
                relation,
                declared,
            } => write!(
                formatter,
                "result axis {result_axis} states {relation} at extent {declared}, which duplicates nothing; a many-to-one relation widens to an extent of at least two"
            ),
            Self::NoManyToOneRelation => formatter.write_str(
                "the mapping states no many-to-one relation, so it returns its operand and denotes no broadcast",
            ),
            Self::TooManyAxes { axes, limit } => write!(
                formatter,
                "the mapping accounts for {axes} result axes, exceeding {limit}"
            ),
            Self::MalformedAttribute { subject } => {
                write!(formatter, "the {subject} is malformed")
            }
            Self::CanonicalBound(source) => {
                write!(formatter, "the mapping exceeds a canonical bound: {source}")
            }
            Self::ResultShape(source) => {
                write!(formatter, "the result shape is not admitted: {source}")
            }
            Self::UndeclaredSymbol { symbol } => write!(
                formatter,
                "{symbol} is not declared by this program's shape environment"
            ),
            Self::SourceTooLate {
                symbol,
                available,
                ceiling,
            } => write!(
                formatter,
                "{symbol} is available at {available}, after {ceiling}"
            ),
            Self::ExtentNotProvedPositive {
                result_axis,
                declared,
            } => write!(
                formatter,
                "result axis {result_axis} declares {declared}, and a many-to-one relation requires this program's shape environment to prove that extent is at least one"
            ),
            Self::ExtentsNotProvedEqual {
                result_axis,
                declared,
                operand,
            } => write!(
                formatter,
                "result axis {result_axis} declares {declared} and reads operand extent {operand}, and this program's shape environment does not prove they are one extent"
            ),
            Self::StretchSourceNotProvedUnit {
                operand_axis,
                extent,
            } => write!(
                formatter,
                "operand axis {} names {extent}, and this program's shape environment does not prove that extent is one",
                operand_axis.get()
            ),
        }
    }
}

impl Error for BroadcastMappingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalBound(source) => Some(source),
            Self::ResultShape(source) => Some(source),
            _ => None,
        }
    }
}

/// Collision-free canonical encoding of one broadcast axis mapping.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalBroadcastAxisMapping(Vec<u8>);

impl CanonicalBroadcastAxisMapping {
    /// Returns the domain-separated canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A validated broadcast axis mapping.
///
/// Construction decides every rule that is a property of the mapping alone —
/// that it accounts for every declared result axis, that it consumes operand
/// axes in order, that a *literal* widening relation actually widens, and that
/// at least one many-to-one relation is stated. Environment-dependent rules —
/// admission, positivity, `from-operand` equality, and `stretch-unit` unit
/// proof — are decided when the mapping is applied against the program's one
/// environment, never here.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BroadcastAxisMapping {
    result_extents: Vec<SourcedExtent>,
    sources: Vec<BroadcastAxisSource>,
    canonical_value: CanonicalValue,
}

impl BroadcastAxisMapping {
    /// Builds an axis mapping from a declared result shape and one source per axis.
    ///
    /// Context-free: a symbolic many-to-one extent is admitted here and proved
    /// positive only when the mapping is applied. A literal many-to-one extent
    /// below two is still refused, so one relation keeps one spelling.
    ///
    /// # Errors
    ///
    /// Returns [`BroadcastMappingError`] naming the violated rule.
    pub fn new(
        result_extents: impl IntoIterator<Item = impl Into<SourcedExtent>>,
        sources: impl IntoIterator<Item = BroadcastAxisSource>,
    ) -> Result<Self, BroadcastMappingError> {
        let result_extents = collect_bounded(result_extents.into_iter().map(Into::into))?;
        let sources = collect_bounded(sources)?;
        if sources.len() != result_extents.len() {
            return Err(BroadcastMappingError::SourceCountMismatch {
                sources: sources.len(),
                result_axes: result_extents.len(),
            });
        }
        // Operand axes are consumed in strictly ascending order starting at zero.
        // Deciding it here rather than against the operand means a reordered
        // mapping is refused as a reordering, not as an out-of-range axis.
        let mut expected = 0_u32;
        for source in &sources {
            if let Some(axis) = source.operand_axis() {
                if axis.get() != expected {
                    return Err(BroadcastMappingError::OperandAxisOutOfOrder {
                        named: axis,
                        expected: Axis::new(expected),
                    });
                }
                expected = expected.saturating_add(1);
            }
        }
        for (result_axis, (source, extent)) in sources.iter().zip(&result_extents).enumerate() {
            if let (true, Some(literal)) = (source.is_many_to_one(), extent.as_static())
                && literal.get() < 2
            {
                return Err(BroadcastMappingError::RelationDoesNotWiden {
                    result_axis,
                    relation: source.canonical_name(),
                    declared: literal.get(),
                });
            }
        }
        if !sources.iter().any(|source| source.is_many_to_one()) {
            return Err(BroadcastMappingError::NoManyToOneRelation);
        }
        let canonical_value = encode_mapping(&result_extents, &sources)
            .map_err(BroadcastMappingError::CanonicalBound)?;
        Ok(Self {
            result_extents,
            sources,
            canonical_value,
        })
    }

    /// Decodes one mapping attribute exactly as an occurrence carries it.
    ///
    /// # Errors
    ///
    /// Returns [`BroadcastMappingError`] for a malformed record, an unadmitted
    /// relation name, or a violated mapping rule. The mapping's own rules are
    /// re-decided here rather than trusted, because a hand-assembled attribute
    /// never passed the constructor.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, BroadcastMappingError> {
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(malformed(BroadcastAttributeSubject::MappingRecord));
        };
        let [extents_field, sources_field] = fields else {
            return Err(malformed(BroadcastAttributeSubject::MappingRecord));
        };
        if extents_field.id() != BROADCAST_MAPPING_RESULT_EXTENTS
            || sources_field.id() != BROADCAST_MAPPING_SOURCES
        {
            return Err(malformed(BroadcastAttributeSubject::MappingRecord));
        }
        let result_extents = decode_extents(extents_field.value())?;
        let sources = decode_sources(sources_field.value())?;
        Self::new(result_extents, sources)
    }

    /// Returns the declared result extents.
    #[must_use]
    pub fn result_extents(&self) -> &[SourcedExtent] {
        &self.result_extents
    }

    /// Returns one source per result axis, in result-axis order.
    #[must_use]
    pub fn sources(&self) -> &[BroadcastAxisSource] {
        &self.sources
    }

    /// Returns the canonical attribute value an occurrence carries.
    #[must_use]
    pub const fn canonical_value(&self) -> &CanonicalValue {
        &self.canonical_value
    }

    /// Returns the domain-separated canonical encoding of this mapping.
    ///
    /// Derived from [`Self::canonical_value`] rather than from a second walk of
    /// the mapping, so the identity a reader compares and the attribute an
    /// occurrence carries cannot disagree about what a mapping is.
    #[must_use]
    pub fn canonical_encoding(&self) -> CanonicalBroadcastAxisMapping {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, BROADCAST_AXIS_MAPPING_DOMAIN);
        self.canonical_value.encode(&mut bytes);
        CanonicalBroadcastAxisMapping(bytes)
    }

    /// Decides this mapping against one static operand shape and returns the result.
    ///
    /// The static/reference path: every declared extent must be a literal, and
    /// equality and unit proofs are decided by comparing those literals. A
    /// mapping that names a symbol is refused rather than applied against a
    /// second environment this method does not hold.
    ///
    /// # Errors
    ///
    /// Returns [`BroadcastMappingError`] naming the violated rule.
    pub fn result_shape(&self, operand: &Shape) -> Result<Shape, BroadcastMappingError> {
        let sourced = self.apply(&SourcedShape::from(operand.clone()), None)?;
        if let Some(shape) = sourced.as_static() {
            return Ok(shape.clone());
        }
        if let Some(symbol) = self.result_extents.iter().find_map(SourcedExtent::symbol) {
            return Err(BroadcastMappingError::UndeclaredSymbol {
                symbol: symbol.clone(),
            });
        }
        Err(BroadcastMappingError::NoManyToOneRelation)
    }

    /// Decides this mapping against one operand and the program's environment.
    ///
    /// One `O(rank)` walk. Admission, positivity, equality, and unit proofs
    /// read the environment's retained summary; this method does not solve.
    /// `sources == None` treats every symbol as undeclared.
    ///
    /// # Errors
    ///
    /// Returns [`BroadcastMappingError`] naming the violated rule.
    pub(crate) fn apply(
        &self,
        operand: &SourcedShape,
        sources: Option<&ExtentSources>,
    ) -> Result<SourcedShape, BroadcastMappingError> {
        let consumed = self
            .sources
            .iter()
            .filter(|source| source.operand_axis().is_some())
            .count();
        if consumed != operand.rank() {
            return Err(BroadcastMappingError::OperandAxesUnconsumed {
                consumed,
                rank: operand.rank(),
            });
        }
        let operand_extents: Vec<SourcedExtent> = operand.extents().collect();
        for (result_axis, (source, declared)) in
            self.sources.iter().zip(&self.result_extents).enumerate()
        {
            admit_declared(declared, sources)?;
            if source.is_many_to_one()
                && declared.as_static().is_none()
                && !sources.is_some_and(|sources| sources.proves_positive(declared))
            {
                return Err(BroadcastMappingError::ExtentNotProvedPositive {
                    result_axis,
                    declared: declared.clone(),
                });
            }
            // The mapping consumes axes `0..consumed` in order and `consumed`
            // equals the operand's rank, so every named axis indexes `operand_extents`.
            match source {
                BroadcastAxisSource::FromOperand(axis) => {
                    let operand_extent = &operand_extents[usize::try_from(axis.get()).unwrap_or(0)];
                    match (declared.as_static(), operand_extent.as_static()) {
                        (Some(declared_literal), Some(operand_literal)) => {
                            if declared_literal != operand_literal {
                                return Err(BroadcastMappingError::ExtentDisagreement {
                                    result_axis,
                                    declared: declared_literal.get(),
                                    operand_axis: *axis,
                                    operand_extent: operand_literal.get(),
                                });
                            }
                        }
                        _ if sources.is_some_and(|sources| {
                            sources.proves_equal(declared, operand_extent)
                        }) => {}
                        _ => {
                            return Err(BroadcastMappingError::ExtentsNotProvedEqual {
                                result_axis,
                                declared: declared.clone(),
                                operand: operand_extent.clone(),
                            });
                        }
                    }
                }
                BroadcastAxisSource::StretchUnit(axis) => {
                    let operand_extent = &operand_extents[usize::try_from(axis.get()).unwrap_or(0)];
                    match operand_extent.as_static() {
                        Some(literal) if literal.get() != 1 => {
                            return Err(BroadcastMappingError::StretchSourceNotUnit {
                                operand_axis: *axis,
                                extent: literal.get(),
                            });
                        }
                        Some(_) => {}
                        None if sources.is_some_and(|sources| {
                            sources.proves_equal(
                                operand_extent,
                                &SourcedExtent::Static(Extent::new(1)),
                            )
                        }) => {}
                        None => {
                            return Err(BroadcastMappingError::StretchSourceNotProvedUnit {
                                operand_axis: *axis,
                                extent: operand_extent.clone(),
                            });
                        }
                    }
                }
                BroadcastAxisSource::Replicate => {}
            }
        }
        SourcedShape::sourced(self.result_extents.clone())
            .map_err(BroadcastMappingError::ResultShape)
    }
}

fn admit_declared(
    extent: &SourcedExtent,
    sources: Option<&ExtentSources>,
) -> Result<(), BroadcastMappingError> {
    match sources {
        Some(sources) => sources
            .admit(extent)
            .map(|_| ())
            .map_err(|error| match error {
                crate::shape::ExtentSourceError::UndeclaredSymbol { symbol } => {
                    BroadcastMappingError::UndeclaredSymbol { symbol }
                }
                crate::shape::ExtentSourceError::SourceTooLate {
                    symbol,
                    available,
                    ceiling,
                } => BroadcastMappingError::SourceTooLate {
                    symbol,
                    available,
                    ceiling,
                },
                crate::shape::ExtentSourceError::DivisorNotProvedPositive { .. }
                | crate::shape::ExtentSourceError::ExtentsNotProvedEqual(_) => {
                    unreachable!("admit reports only undeclared and too-late")
                }
            }),
        None => match extent.symbol() {
            Some(symbol) => Err(BroadcastMappingError::UndeclaredSymbol {
                symbol: symbol.clone(),
            }),
            None => Ok(()),
        },
    }
}

fn malformed(subject: BroadcastAttributeSubject) -> BroadcastMappingError {
    BroadcastMappingError::MalformedAttribute { subject }
}

/// Truncates a rejected relation name to a bounded prefix.
///
/// A diagnostic message has a governed byte bound and the rejected name comes
/// from an attribute a caller assembled, so truncating here keeps the refusal a
/// refusal instead of a provider-contract failure about the message's length.
fn bounded_name(name: &str) -> String {
    const LIMIT: usize = 64;
    let end = name
        .char_indices()
        .map(|(index, character)| index + character.len_utf8())
        .take_while(|end| *end <= LIMIT)
        .last()
        .unwrap_or(0);
    name[..end].to_owned()
}

fn collect_bounded<T>(items: impl IntoIterator<Item = T>) -> Result<Vec<T>, BroadcastMappingError> {
    let mut collected = Vec::new();
    for item in items
        .into_iter()
        .take(MAX_BROADCAST_MAPPING_AXES.saturating_add(1))
    {
        if collected.len() == MAX_BROADCAST_MAPPING_AXES {
            return Err(BroadcastMappingError::TooManyAxes {
                axes: MAX_BROADCAST_MAPPING_AXES.saturating_add(1),
                limit: MAX_BROADCAST_MAPPING_AXES,
            });
        }
        collected.push(item);
    }
    Ok(collected)
}

fn decode_extents(value: &CanonicalValue) -> Result<Vec<SourcedExtent>, BroadcastMappingError> {
    let CanonicalValueView::Sequence(values) = value.view() else {
        return Err(malformed(BroadcastAttributeSubject::ResultExtents));
    };
    if values.len() > MAX_BROADCAST_MAPPING_AXES {
        return Err(BroadcastMappingError::TooManyAxes {
            axes: values.len(),
            limit: MAX_BROADCAST_MAPPING_AXES,
        });
    }
    values
        .iter()
        .map(|value| {
            let CanonicalValueView::Bytes(bytes) = value.view() else {
                return Err(malformed(BroadcastAttributeSubject::ResultExtents));
            };
            SourcedExtent::decode(bytes).ok_or(malformed(BroadcastAttributeSubject::ResultExtents))
        })
        .collect()
}

fn decode_sources(
    value: &CanonicalValue,
) -> Result<Vec<BroadcastAxisSource>, BroadcastMappingError> {
    let CanonicalValueView::Sequence(values) = value.view() else {
        return Err(malformed(BroadcastAttributeSubject::SourceSequence));
    };
    if values.len() > MAX_BROADCAST_MAPPING_AXES {
        return Err(BroadcastMappingError::TooManyAxes {
            axes: values.len(),
            limit: MAX_BROADCAST_MAPPING_AXES,
        });
    }
    values.iter().map(decode_source).collect()
}

fn decode_source(value: &CanonicalValue) -> Result<BroadcastAxisSource, BroadcastMappingError> {
    let CanonicalValueView::Record(fields) = value.view() else {
        return Err(malformed(BroadcastAttributeSubject::SourceRecord));
    };
    let Some(relation_field) = fields.first() else {
        return Err(malformed(BroadcastAttributeSubject::SourceRecord));
    };
    if relation_field.id() != BROADCAST_SOURCE_RELATION {
        return Err(malformed(BroadcastAttributeSubject::SourceRecord));
    }
    let CanonicalValueView::Utf8(name) = relation_field.value().view() else {
        return Err(malformed(BroadcastAttributeSubject::Relation));
    };
    // Exactly the fields the relation uses. A replication carrying an axis, or a
    // correspondence missing one, is as malformed as a bad relation name:
    // admitting either would let two records denote one entry.
    match (name, fields) {
        (BROADCAST_RELATION_REPLICATE, [_]) => Ok(BroadcastAxisSource::Replicate),
        (BROADCAST_RELATION_FROM_OPERAND, [_, axis_field])
            if axis_field.id() == BROADCAST_SOURCE_AXIS =>
        {
            Ok(BroadcastAxisSource::FromOperand(decode_axis(
                axis_field.value(),
            )?))
        }
        (BROADCAST_RELATION_STRETCH_UNIT, [_, axis_field])
            if axis_field.id() == BROADCAST_SOURCE_AXIS =>
        {
            Ok(BroadcastAxisSource::StretchUnit(decode_axis(
                axis_field.value(),
            )?))
        }
        (
            BROADCAST_RELATION_REPLICATE
            | BROADCAST_RELATION_FROM_OPERAND
            | BROADCAST_RELATION_STRETCH_UNIT,
            _,
        ) => Err(malformed(BroadcastAttributeSubject::SourceRecord)),
        _ => Err(BroadcastMappingError::UnadmittedRelation {
            name: bounded_name(name),
        }),
    }
}

fn decode_axis(value: &CanonicalValue) -> Result<Axis, BroadcastMappingError> {
    let CanonicalValueView::Unsigned {
        width: CanonicalIntegerWidth::Bits32,
        bits,
    } = value.view()
    else {
        return Err(malformed(BroadcastAttributeSubject::Axis));
    };
    u32::try_from(bits)
        .map(Axis::new)
        .map_err(|_| malformed(BroadcastAttributeSubject::Axis))
}

/// Builds the canonical attribute value of one validated mapping.
///
/// The sources are a sequence *of records* rather than two parallel sequences of
/// names and axes, and that framing is load-bearing rather than cosmetic: a
/// replication has no axis, so a parallel encoding would need a sentinel axis
/// value, and a sentinel is a value a caller can also write.
fn encode_mapping(
    result_extents: &[SourcedExtent],
    sources: &[BroadcastAxisSource],
) -> Result<CanonicalValue, TypeIdentityError> {
    let mut encoded_extents = Vec::with_capacity(result_extents.len());
    for extent in result_extents {
        let mut bytes = Vec::with_capacity(extent.encoded_len());
        extent.encode(&mut bytes);
        encoded_extents.push(CanonicalValue::bytes_owned(bytes)?);
    }
    let extents = CanonicalValue::sequence(encoded_extents)?;
    let mut encoded = Vec::with_capacity(sources.len());
    for source in sources {
        let relation = CanonicalField::new(
            BROADCAST_SOURCE_RELATION,
            CanonicalValue::utf8(source.canonical_name())?,
        );
        encoded.push(match source.operand_axis() {
            None => CanonicalValue::record([relation])?,
            Some(axis) => CanonicalValue::record([
                relation,
                CanonicalField::new(
                    BROADCAST_SOURCE_AXIS,
                    CanonicalValue::unsigned_u32(axis.get()),
                ),
            ])?,
        });
    }
    CanonicalValue::record([
        CanonicalField::new(BROADCAST_MAPPING_RESULT_EXTENTS, extents),
        CanonicalField::new(
            BROADCAST_MAPPING_SOURCES,
            CanonicalValue::sequence(encoded)?,
        ),
    ])
}

/// Registers the governed broadcast family.
pub(super) fn register_standard_broadcast(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new_governed_environment_aware(
        broadcast_f32_op(),
        OperationSchema::new(
            OperationArity::exact(1),
            OperationArity::exact(1),
            [OperationAttributeSchema::required(
                BROADCAST_AXIS_MAPPING_ATTRIBUTE,
                CanonicalValueKind::Record,
            )],
        )
        .expect("the governed broadcast schema is valid"),
        NormativeDefinitionRef::new(BROADCAST_F32_NORMATIVE_DEFINITION)?,
        OperationDefinitionFacts::new(broadcast_facts()),
        standard_conformance("broadcast-f32"),
        OperationEffect::Pure,
        Arc::new(BroadcastF32),
    ))
    // No algebraic capability is declared, deliberately. A broadcast performs no
    // arithmetic, so it has no associativity or commutativity to declare, and a
    // missing declaration is unknown rather than the inverse law.
}

/// The complete normative definition of `tiler::broadcast-f32@2`.
const BROADCAST_F32_NORMATIVE_DEFINITION: &str = concat!(
    "tiler::broadcast-f32@2; a total output-to-input binary32 coordinate relation stated by an ",
    "explicit axis mapping with exactly one entry per result axis, so every result axis is ",
    "accounted for and every many-to-one relation is written down. ",
    "Declared result extents are sourced: a literal or a symbol declared by the program's exact ",
    "shape environment. ",
    "Admitted relations, and no others: from-operand, a one-to-one correspondence whose result ",
    "extent the environment proves equal to the named operand axis's extent; stretch-unit, a ",
    "many-to-one widening of a named operand axis the environment proves has extent one; and ",
    "replicate, a many-to-one result axis with no operand axis behind it. ",
    "A literal many-to-one result extent must be at least two. A symbolic many-to-one result ",
    "extent is admitted only when the environment proves it positive; it may bind to one, ",
    "including when the environment determines it is always one. A symbol that may be zero is ",
    "refused. ",
    "The mapping consumes every operand axis exactly once and in ascending order; a reordering is ",
    "a reindex and a dropped operand axis is a reduction or a slice, and each is refused by name. ",
    "A mapping stating only one-to-one correspondences denotes no broadcast and is refused. ",
    "No value is computed, converted, or rounded: every result element is an operand element ",
    "unchanged, and reads may alias. ",
    "This operation makes no claim that storage was replicated or materialized: it states a ",
    "logical coordinate relation, and every physical realization of it remains a planning outcome.",
);

fn broadcast_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            BROADCAST_FACT_VALUE_BEHAVIOUR,
            fact("none-every-result-element-is-an-operand-element-unchanged"),
        ),
        CanonicalField::new(
            BROADCAST_FACT_MAPPING_CLASS,
            fact("total-over-the-result-domain-and-many-to-one-onto-the-operand-domain"),
        ),
        CanonicalField::new(
            BROADCAST_FACT_STORAGE_CLAIM,
            fact("none-no-replication-or-materialization-is-claimed-and-reads-may-alias"),
        ),
        CanonicalField::new(
            BROADCAST_FACT_ADMITTED_RELATIONS,
            fact("from-operand,stretch-unit,replicate"),
        ),
    ])
    .expect("the governed broadcast facts are canonical")
}

fn fact(value: &str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("a governed broadcast fact is bounded")
}

struct BroadcastF32;

impl OperationInferencer for BroadcastF32 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "broadcast.attributes",
                "a broadcast requires exactly the axis-mapping attribute".to_owned(),
            ));
        }
        let Some(value) = attributes.get(BROADCAST_AXIS_MAPPING_ATTRIBUTE) else {
            return Err(op_error(
                "broadcast.attributes",
                "a broadcast requires exactly the axis-mapping attribute".to_owned(),
            ));
        };
        // The mapping's own rules are decided before anything about the
        // occurrence, so a malformed mapping is refused under its own rule
        // rather than under whichever shape check happened to notice first.
        let mapping = BroadcastAxisMapping::from_canonical_value(value)
            .map_err(|error| mapping_rejection(&error))?;
        let [operand] = operands else {
            return Err(op_error(
                "broadcast.operands",
                format!(
                    "a broadcast takes one operand and {} were supplied",
                    operands.len()
                ),
            ));
        };
        if operand.resolved_type() != &F32::resolved_type() {
            return Err(op_error(
                "broadcast.type",
                "a broadcast operand must be f32".to_owned(),
            ));
        }
        let shape = mapping
            .apply(operand.shape(), request.extent_sources())
            .map_err(|error| mapping_rejection(&error))?;
        outputs.try_push(ValueFact::from_sourced(F32::resolved_type(), shape))
    }
}

fn mapping_rejection(error: &BroadcastMappingError) -> OperationInferenceError {
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
