//! The governed `Slice` family: reading a rectangular sub-region of a tensor.
//!
//! **What a `Slice` is.** The one family that states an injective, *non*-surjective
//! output-to-input coordinate relation: every result coordinate reads one operand
//! element, no operand element is read twice, and at least one operand element is
//! not read at all. Every result element is an operand element unchanged, so the
//! family computes nothing: it is structural in exactly the sense
//! [`super::reindex`] and [`super::broadcast`] are, and it shares their
//! bit-preservation obligation.
//!
//! It is the family [`super::reindex`] names when it refuses. A split whose
//! factors fall short of its axis is refused under
//! [`ReindexFormError::SplitNotSurjective`](super::ReindexFormError::SplitNotSurjective)
//! as "a slice rather than a reindex", and the reindex normative definition says a
//! non-surjective mapping "is a slice, a different family". This module is that
//! family, and the refusals stay: a selection is written as an occurrence of this
//! key, never as a narrow reindex.
//!
//! **It makes no storage claim.** Registering an occurrence says that the
//! *logical* coordinates were restricted. It does not claim that bytes were
//! copied, that a view was taken, or that anything was left alone. Whether a
//! selection costs a dispatch, becomes an offset in a consumer's access map, or
//! disappears into a base-address adjustment is a physical-planning outcome, and
//! this definition deliberately fixes none of it.
//!
//! # The form, decided before the key was registered
//!
//! The choice was between one keyed family carrying a canonical selection
//! structure and a key per selection class — the same choice
//! [ADR 0087](../../../../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md)
//! settled for the contraction. It is settled here the same way and for the same
//! reasons, which transfer intact: a frontend must never choose among keys,
//! because that moves the selection analysis into every frontend as ungoverned
//! key choice; a key set that grows per class grows without bound, each key owing
//! a full vertical of registry, reference, conformance, ABI, and identity
//! obligations; and generalizing a fixed key later migrates every artifact
//! identity, cache subject, and golden that named it. The costs the contraction
//! weighed against those — a large attribute schema and an inference routine that
//! must reject every malformed structure — are the same costs, and they are paid
//! below rather than avoided.
//!
//! The structure is a **total per-axis selection**: exactly one entry per operand
//! axis, each entry stating one of the admitted relations. Two spellings of one
//! selection are impossible, which is what a canonical identity needs. The
//! alternative — a sparse list of `(axis, offset, extent)` triples naming only the
//! selected axes — was eliminated on that exact point: a sparse list has one
//! spelling per *ordering* of its entries, so either two orderings of one
//! selection encode differently (one program, two identities) or the family sorts
//! a caller's list (normalizing where this corpus refuses). The sparse form also
//! introduces a duplicated axis and an out-of-range axis as representable states
//! needing their own refusals; under the total form both are unrepresentable, and
//! the one rule that replaces them —
//! [`SliceSelectionError::SelectionCountMismatch`] — is decided against the
//! operand's own rank rather than against a caller's assertion.
//!
//! Rank is preserved: a selection restricts coordinates and drops no axis. A
//! selection of one coordinate leaves an extent-one axis behind, and removing it
//! is a `remove-unit-axis` [`super::reindex`] occurrence written after this one.
//! That is this corpus's standing rule that a composition is a chain of
//! occurrences rather than two maps folded into one attribute.
//!
//! # What is deliberately not admitted, and what admitting it would need
//!
//! **A strided window.** The [operation taxonomy](../../../../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s
//! F-24 places the strided and simple forms in one family with a stride
//! attribute, and its `RQ-OP-05` — whether a negative stride is admitted — is
//! stated to close on the ABI question `Q-SHAPE-008` owns, not separately. So the
//! stride's *schema* is not decidable yet, and admitting an unsigned stride now
//! would fix half a schema whose other half is reserved. The relation name
//! [`SLICE_RELATION_STRIDED_WINDOW`] is therefore reserved and refused under its
//! own rule rather than left to read as an unrecognized name.
//!
//! **A symbolic offset.** This is the boundary the family's ticket requires
//! stated rather than assumed, and it is the *index vocabulary* that closes it,
//! not this module. `IndexNode` at `crates/tiler-ir/src/index/model.rs` has five
//! variants — `Constant`, `Dimension`, a `LinearCombination` whose constant and
//! per-term coefficients are `IndexInteger`, and `FloorDiv` and `Modulo` whose
//! `divisor` is a `SourcedExtent`. `SourcedExtent` is the only carrier of a
//! possibly-symbolic extent and it appears in no other position, so the read
//! `t + k` is expressible for a literal `k` and is not expressible for a bound
//! symbol `C`. A second gap sits above it: a semantic occurrence's
//! [`ValueFact`] carries a static [`Shape`], so a symbolic offset has nothing to
//! name at this layer either. The literal-offset form is therefore what this
//! family delivers, [`SLICE_RELATION_SYMBOLIC_WINDOW`] is reserved and refused by
//! name, and the reconsideration trigger is exact: an `IndexNode` variant that
//! carries a `SourcedExtent` in a coordinate position. The refusal reserves a
//! name, not a design — whether a symbolic offset arrives as an attribute symbol
//! or as an index operand is left open, because F-24 contemplates the operand
//! spelling for its own dynamic form.
//!
//! # Where the family's claim is made, and the one degenerate case
//!
//! Every rule here is decided per axis, over coordinate *ranges*: an admitted
//! selection restricts at least one axis, and a restricting window leaves at
//! least one coordinate of that axis unread. So the selected box is always a
//! proper sub-box of the operand's, which is what makes the family's
//! injective-and-not-surjective claim a proved property rather than an
//! assertion.
//!
//! The one case where that does not become "strictly fewer elements" is an
//! operand with a zero extent on an axis the selection leaves whole: both the
//! operand and the result then have no elements, and the strict inclusion is
//! vacuous. That occurrence is admitted rather than refused, on the precedent
//! [`super::concatenate`]'s zero-extent rule sets — the family decides the
//! selection a caller wrote, and an operand that is already empty is a shape the
//! program had before the selection was stated. It is written down here because
//! a reader checking the mapping-class fact against an element count would
//! otherwise find one occurrence where the two do not line up.
//!
//! # The out-of-bounds posture, and why it is a refusal
//!
//! A selection that leaves its axis is refused under
//! [`SliceSelectionError::WindowOutOfBounds`] and is never clamped or wrapped.
//! The taxonomy records that this is a real divergence among the primary
//! authorities rather than a settled convention — ONNX `Slice` and Python-style
//! slicing clamp, `StableHLO` constrains at verification — and that two conventions
//! produce a *different tensor* for one program rather than a different
//! diagnostic. Inheriting either silently would make a frontend's meaning depend
//! on which specification its author had read, so the bound is a validated
//! obligation here. Every extent a semantic occurrence can carry is static, so the
//! obligation is discharged at construction; a symbolic extent would make it the
//! typed host-side pre-dispatch requirement the shape-environment contract's
//! three-outcome path already accepts, and that arrives with the symbolic offset
//! rather than before it.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::identity::push_slice;
use crate::shape::{Extent, Shape};

use super::registry::standard_conformance;
use super::{
    AttributeFieldId, CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind,
    CanonicalValueView, F32, NormativeDefinitionRef, OpKey, OperationArity,
    OperationAttributeSchema, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, ProviderDiagnosticCode, RegistryError,
    SemanticRegistryRegistrar, TypeIdentityError, ValueFact,
};

/// Maximum operand axes one selection may account for.
///
/// The same bound a canonical sequence admits, so an oversized selection is
/// refused with a slice-shaped diagnostic rather than an anonymous canonical-bound
/// one. It is not the only bound a wide selection meets: a window entry costs
/// four canonical nodes where a whole axis costs two, so a selection of windows
/// exhausts the shared node budget below this count and is refused under
/// [`SliceSelectionError::CanonicalBound`]. Both are refusals rather than
/// truncations, and both sit well under the governed shape rank limit, so a legal
/// but very wide shape has no selection rather than a silently narrowed one.
pub const MAX_SLICE_SELECTION_AXES: usize = super::types::MAX_RESOLVED_TYPE_ITEMS;

/// Stable field ID carrying the canonical selection on the slice.
pub const SLICE_SELECTION_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// Selection-record field carrying one entry per operand axis.
///
/// The selection is a one-field record rather than a bare sequence so that a
/// later field — F-24's dynamic-offset operand binding is the named candidate —
/// can be added beside it without changing the bytes of an occurrence that does
/// not carry one.
pub const SLICE_SELECTION_AXES: AttributeFieldId = AttributeFieldId::new(1);

/// Axis-record field naming which of the admitted relations this entry states.
///
/// The three constants below are fields of the *slice axis record*, a different
/// record from the selection record that contains it and from every other record
/// in this corpus; equal integers across records are unrelated.
pub const SLICE_AXIS_RELATION: AttributeFieldId = AttributeFieldId::new(1);
/// Axis-record field carrying a window's first selected coordinate.
pub const SLICE_AXIS_OFFSET: AttributeFieldId = AttributeFieldId::new(2);
/// Axis-record field carrying the number of coordinates a window selects.
pub const SLICE_AXIS_EXTENT: AttributeFieldId = AttributeFieldId::new(3);

/// Fact field naming what a slice does to values.
///
/// The five fields below are the family's semantic signature. Every one is
/// unconditional on this definition: absence is a malformed record, never a
/// default. None of them is numerical, because a selection performs no arithmetic
/// — which is itself the fact a reader needs.
pub const SLICE_FACT_VALUE_BEHAVIOUR: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming the relation's totality, injectivity, and non-surjectivity.
pub const SLICE_FACT_MAPPING_CLASS: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming what this family claims about storage.
pub const SLICE_FACT_STORAGE_CLAIM: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field naming the closed set of admitted axis relations.
pub const SLICE_FACT_ADMITTED_RELATIONS: AttributeFieldId = AttributeFieldId::new(4);
/// Fact field naming what an out-of-bounds selection does.
pub const SLICE_FACT_OUT_OF_BOUNDS: AttributeFieldId = AttributeFieldId::new(5);

/// Canonical name of the relation that selects an axis entirely.
///
/// The four names below are canonical identity, not display text: an axis record
/// carries the exact string, so respelling one changes every occurrence's
/// attribute bytes. The last two are reserved and always refused; an unrecognized
/// name is refused rather than mapped to a nearest match.
pub const SLICE_RELATION_WHOLE_AXIS: &str = "whole-axis";
/// Canonical name of the contiguous sub-range relation.
pub const SLICE_RELATION_WINDOW: &str = "window";
/// Reserved name of the strided sub-range relation, refused until `RQ-OP-05` closes.
pub const SLICE_RELATION_STRIDED_WINDOW: &str = "strided-window";
/// Reserved name of the symbolic-offset relation, refused until the index vocabulary carries one.
pub const SLICE_RELATION_SYMBOLIC_WINDOW: &str = "symbolic-window";

/// Domain separator of a canonical slice selection encoding.
const SLICE_SELECTION_DOMAIN: &[u8] = b"tiler.slice-selection.v1\0";

/// Returns the governed binary32 slice operation key.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn slice_f32_op() -> OpKey {
    OpKey::new("tiler", "slice-f32", 1).expect("the governed slice key is valid")
}

/// What one operand axis of a slice contributes.
///
/// Deliberately **not** `#[non_exhaustive]`, on the precedent
/// [`super::BroadcastAxisSource`] sets and for the same reason: a lowering maps
/// this vocabulary *totally* onto a coordinate expression, and no coordinate is
/// derivable from a relation it has not seen. A third admitted relation must be a
/// build error at every such site rather than a silent fall through.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SliceAxisSelection {
    /// Every coordinate of the axis is read, in order.
    WholeAxis,
    /// A contiguous run of coordinates is read, in order, starting at `offset`.
    Window {
        /// First operand coordinate this axis reads.
        offset: u64,
        /// Number of coordinates read, which becomes the result's extent here.
        extent: Extent,
    },
}

impl SliceAxisSelection {
    /// Returns the canonical name this relation carries in its axis record.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::WholeAxis => SLICE_RELATION_WHOLE_AXIS,
            Self::Window { .. } => SLICE_RELATION_WINDOW,
        }
    }

    /// Returns whether this entry restricts its axis.
    #[must_use]
    pub const fn is_restricting(self) -> bool {
        matches!(self, Self::Window { .. })
    }

    /// Returns the first operand coordinate this entry reads.
    #[must_use]
    pub const fn offset(self) -> u64 {
        match self {
            Self::WholeAxis => 0,
            Self::Window { offset, .. } => offset,
        }
    }
}

impl fmt::Display for SliceAxisSelection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WholeAxis => formatter.write_str(SLICE_RELATION_WHOLE_AXIS),
            Self::Window { offset, extent } => write!(
                formatter,
                "{SLICE_RELATION_WINDOW} of {} coordinates from {offset}",
                extent.get()
            ),
        }
    }
}

/// Which part of a malformed selection attribute was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SliceAttributeSubject {
    /// The attribute was not the one-field selection record.
    SelectionRecord,
    /// The axis field was not a sequence.
    AxisSequence,
    /// One entry was not a well-formed relation record.
    AxisRecord,
    /// One relation name was not canonical UTF-8.
    Relation,
    /// One window offset was not a canonical unsigned 64-bit value.
    Offset,
    /// One window extent was not a canonical unsigned 64-bit value.
    WindowExtent,
}

impl fmt::Display for SliceAttributeSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelectionRecord => formatter.write_str("selection record"),
            Self::AxisSequence => formatter.write_str("axis sequence"),
            Self::AxisRecord => formatter.write_str("axis record"),
            Self::Relation => formatter.write_str("relation name"),
            Self::Offset => formatter.write_str("window offset"),
            Self::WindowExtent => formatter.write_str("window extent"),
        }
    }
}

/// A typed refusal of one slice selection.
///
/// Every variant is one named admission rule. [`SliceSelection`] has no unchecked
/// constructor and [`SliceSelection::result_shape`] is the only path to a result,
/// so holding a result is evidence that the selection's own rules and its rules
/// against the operand were all decided.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SliceSelectionError {
    /// The named relation is not one this family admits.
    UnadmittedRelation {
        /// The rejected name, truncated to a bounded prefix.
        name: String,
    },
    /// The selection states a strided window, which this family does not admit.
    ///
    /// Refused under its own rule rather than as an unrecognized name, because
    /// the reason is a reserved schema rather than a typo: the taxonomy's F-24
    /// holds the strided and simple forms to be one family with a stride
    /// attribute, and whether that attribute admits a negative stride closes on
    /// the ABI question `Q-SHAPE-008` owns. Admitting an unsigned stride now
    /// would fix half of a schema whose other half is reserved.
    StridedSelectionUnsupported,
    /// The selection states a symbolic offset, which the index vocabulary cannot carry.
    ///
    /// The literal-offset form is what this family delivers. A bound extent
    /// symbol in a coordinate position is not expressible: `SourcedExtent` is the
    /// only carrier of a possibly-symbolic extent and appears in no `IndexNode`
    /// variant except the `FloorDiv` and `Modulo` divisors, and a semantic value
    /// fact carries static extents besides.
    SymbolicOffsetUnsupported,
    /// A window selects no coordinate.
    ///
    /// An empty selection produces a result with no elements, which no consumer
    /// has asked for and which is the classic symptom of an off-by-one in the
    /// caller's arithmetic. It is refused rather than admitted: widening a
    /// refusal later is an appends-only change, while retracting an admission is
    /// not.
    ///
    /// This refuses a *selection* that states emptiness, never an operand that
    /// happens to be empty. An operand with a zero extent on an axis the
    /// selection leaves whole is admitted and its result has no elements, on the
    /// precedent [`super::ConcatenateError`]'s zero-extent rule sets: the family
    /// decides the selection, and an empty operand is a shape the program
    /// already had.
    EmptyWindow {
        /// Zero-based position of the offending entry.
        axis: usize,
        /// The offset it states.
        offset: u64,
    },
    /// The selection restricts no axis, so it denotes no slice.
    ///
    /// A selection of nothing but whole axes returns its operand. It is refused
    /// for the reason [`super::ReindexFormError::IdentityMapping`] and
    /// [`super::BroadcastMappingError::NoManyToOneRelation`] state: an operation
    /// that denotes no member of its family belongs to a different family, or to
    /// none. A rank-zero operand lands here too, having no axis to restrict.
    NoRestrictedAxis,
    /// The selection does not state one entry per operand axis.
    ///
    /// This is where a selection written against the wrong rank lands, and it is
    /// also where the sparse form's duplicated and out-of-range axes would have
    /// landed had the structure admitted them. The rank it is checked against is
    /// the operand's own.
    SelectionCountMismatch {
        /// Entries the selection states.
        entries: usize,
        /// Axes the operand has.
        rank: usize,
    },
    /// A window reads past the end of its axis.
    ///
    /// Refused rather than clamped or wrapped. Both conventions are attested in
    /// primary sources and they produce different tensors for one program, so
    /// inheriting either would make a frontend's meaning depend on which
    /// specification its author had read.
    WindowOutOfBounds {
        /// Zero-based position of the offending entry.
        axis: usize,
        /// The first coordinate it reads.
        offset: u64,
        /// The number of coordinates it reads.
        extent: u64,
        /// The axis extent it must stay inside.
        axis_extent: u64,
    },
    /// A window covers its axis entirely, which is the whole-axis relation.
    ///
    /// Refused so that one map has one spelling. Together with
    /// [`Self::NoRestrictedAxis`] this is what makes the family's non-surjectivity
    /// a *proved* property rather than a claim: an admitted selection restricts at
    /// least one axis, and a restricting entry leaves at least one coordinate of
    /// that axis unread.
    WindowIsWholeAxis {
        /// Zero-based position of the offending entry.
        axis: usize,
        /// The axis extent the window reproduces.
        extent: u64,
    },
    /// The selection accounted for more axes than one canonical sequence admits.
    TooManyAxes {
        /// First rejected axis count.
        axes: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// The attribute was not a well-formed selection record.
    MalformedAttribute {
        /// The rejected part.
        subject: SliceAttributeSubject,
    },
    /// The selection exceeded a canonical structural bound.
    CanonicalBound(TypeIdentityError),
    /// The result shape exceeded the governed rank profile.
    ResultShape(crate::shape::ShapeError),
}

impl SliceSelectionError {
    /// Returns the stable provider diagnostic code naming this refusal.
    ///
    /// Each rule has its own code, so a caller reads *which* rule refused from the
    /// code rather than by matching on a message.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::UnadmittedRelation { .. } => "slice.selection.unadmitted-relation",
            Self::StridedSelectionUnsupported => "slice.selection.strided-window-unsupported",
            Self::SymbolicOffsetUnsupported => "slice.selection.symbolic-offset-unsupported",
            Self::EmptyWindow { .. } => "slice.selection.empty-window",
            Self::NoRestrictedAxis => "slice.selection.no-restricted-axis",
            Self::SelectionCountMismatch { .. } => "slice.selection.entry-count",
            Self::WindowOutOfBounds { .. } => "slice.selection.out-of-bounds",
            Self::WindowIsWholeAxis { .. } => "slice.selection.window-is-whole-axis",
            Self::TooManyAxes { .. } => "slice.selection.too-many-axes",
            Self::MalformedAttribute { .. } => "slice.selection.malformed-attribute",
            Self::CanonicalBound(_) => "slice.selection.canonical-bound",
            Self::ResultShape(_) => "slice.selection.result-shape",
        }
    }
}

impl fmt::Display for SliceSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnadmittedRelation { name } => write!(
                formatter,
                "{name} is not an admitted slice axis relation; the admitted relations are {SLICE_RELATION_WHOLE_AXIS} and {SLICE_RELATION_WINDOW}"
            ),
            Self::StridedSelectionUnsupported => write!(
                formatter,
                "{SLICE_RELATION_STRIDED_WINDOW} is reserved and not admitted: a stride attribute is one family with the contiguous window, and whether it admits a negative stride closes on the addressing question that owns it, so half of that schema cannot be fixed here"
            ),
            Self::SymbolicOffsetUnsupported => write!(
                formatter,
                "{SLICE_RELATION_SYMBOLIC_WINDOW} is reserved and not admitted: this family selects at literal offsets, because no index-expression variant carries a bound extent symbol in a coordinate position and a semantic value fact carries static extents"
            ),
            Self::EmptyWindow { axis, offset } => write!(
                formatter,
                "the window on axis {axis} selects no coordinate at offset {offset}; an empty selection is refused rather than admitted"
            ),
            Self::NoRestrictedAxis => formatter.write_str(
                "the selection restricts no axis, so it returns its operand and denotes no slice",
            ),
            Self::SelectionCountMismatch { entries, rank } => write!(
                formatter,
                "the selection states {entries} entries and the operand has {rank} axes, and a selection states exactly one entry per operand axis"
            ),
            Self::WindowOutOfBounds {
                axis,
                offset,
                extent,
                axis_extent,
            } => write!(
                formatter,
                "the window on axis {axis} reads {extent} coordinates from {offset} and the axis has {axis_extent}, so it leaves the operand's declared extent; a selection outside an axis is refused rather than clamped or wrapped"
            ),
            Self::WindowIsWholeAxis { axis, extent } => write!(
                formatter,
                "the window on axis {axis} covers all {extent} of its coordinates, which is the {SLICE_RELATION_WHOLE_AXIS} relation and must be stated as one"
            ),
            Self::TooManyAxes { axes, limit } => write!(
                formatter,
                "the selection accounts for {axes} axes, exceeding {limit}"
            ),
            Self::MalformedAttribute { subject } => {
                write!(formatter, "the {subject} is malformed")
            }
            Self::CanonicalBound(source) => {
                write!(
                    formatter,
                    "the selection exceeds a canonical bound: {source}"
                )
            }
            Self::ResultShape(source) => {
                write!(formatter, "the result shape is not admitted: {source}")
            }
        }
    }
}

impl Error for SliceSelectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalBound(source) => Some(source),
            Self::ResultShape(source) => Some(source),
            _ => None,
        }
    }
}

/// Collision-free canonical encoding of one slice selection.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalSliceSelection(Vec<u8>);

impl CanonicalSliceSelection {
    /// Returns the domain-separated canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A validated slice selection.
///
/// Construction decides every rule that is a property of the selection alone —
/// that no window is empty, that at least one axis is restricted, and that the
/// entry count is inside the canonical bound. [`Self::result_shape`] decides the
/// rules that need the operand: that the entries are the operand's axes, that
/// every window stays inside its axis, and that no window reproduces its axis.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SliceSelection {
    axes: Vec<SliceAxisSelection>,
    canonical_value: CanonicalValue,
}

impl SliceSelection {
    /// Builds a selection from one entry per operand axis, in axis order.
    ///
    /// # Errors
    ///
    /// Returns [`SliceSelectionError`] naming the violated rule.
    pub fn new(
        axes: impl IntoIterator<Item = SliceAxisSelection>,
    ) -> Result<Self, SliceSelectionError> {
        let axes = collect_bounded(axes)?;
        for (axis, selection) in axes.iter().enumerate() {
            if let SliceAxisSelection::Window { offset, extent } = selection
                && extent.get() == 0
            {
                return Err(SliceSelectionError::EmptyWindow {
                    axis,
                    offset: *offset,
                });
            }
        }
        if !axes.iter().any(|selection| selection.is_restricting()) {
            return Err(SliceSelectionError::NoRestrictedAxis);
        }
        let canonical_value =
            encode_selection(&axes).map_err(SliceSelectionError::CanonicalBound)?;
        Ok(Self {
            axes,
            canonical_value,
        })
    }

    /// Decodes one selection attribute exactly as an occurrence carries it.
    ///
    /// # Errors
    ///
    /// Returns [`SliceSelectionError`] for a malformed record, an unadmitted or
    /// reserved relation name, or a violated selection rule. The selection's own
    /// rules are re-decided here rather than trusted, because a hand-assembled
    /// attribute never passed the constructor.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, SliceSelectionError> {
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(malformed(SliceAttributeSubject::SelectionRecord));
        };
        let [axes_field] = fields else {
            return Err(malformed(SliceAttributeSubject::SelectionRecord));
        };
        if axes_field.id() != SLICE_SELECTION_AXES {
            return Err(malformed(SliceAttributeSubject::SelectionRecord));
        }
        Self::new(decode_axes(axes_field.value())?)
    }

    /// Returns one entry per operand axis, in axis order.
    #[must_use]
    pub fn axes(&self) -> &[SliceAxisSelection] {
        &self.axes
    }

    /// Returns the canonical attribute value an occurrence carries.
    #[must_use]
    pub const fn canonical_value(&self) -> &CanonicalValue {
        &self.canonical_value
    }

    /// Returns the domain-separated canonical encoding of this selection.
    ///
    /// Derived from [`Self::canonical_value`] rather than from a second walk of
    /// the selection, so the identity a reader compares and the attribute an
    /// occurrence carries cannot disagree about what a selection is.
    #[must_use]
    pub fn canonical_encoding(&self) -> CanonicalSliceSelection {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, SLICE_SELECTION_DOMAIN);
        self.canonical_value.encode(&mut bytes);
        CanonicalSliceSelection(bytes)
    }

    /// Decides this selection against one operand shape and derives the result.
    ///
    /// The result extent of a restricted axis is the window's own extent and the
    /// result extent of a whole axis is the operand's; nothing is declared by a
    /// caller. Rank is preserved, so an occurrence that wants an extent-one axis
    /// gone writes a `remove-unit-axis` reindex after it.
    ///
    /// # Errors
    ///
    /// Returns [`SliceSelectionError`] naming the violated rule.
    pub fn result_shape(&self, operand: &Shape) -> Result<Shape, SliceSelectionError> {
        let extents = operand.extents();
        if self.axes.len() != operand.rank() {
            return Err(SliceSelectionError::SelectionCountMismatch {
                entries: self.axes.len(),
                rank: operand.rank(),
            });
        }
        let mut result = Vec::with_capacity(self.axes.len());
        for (axis, (selection, available)) in self.axes.iter().zip(extents).enumerate() {
            let extent = match selection {
                SliceAxisSelection::WholeAxis => *available,
                SliceAxisSelection::Window { offset, extent } => {
                    // The sum is the first coordinate past the window. An
                    // overflowing sum cannot be inside a `u64` extent, so it is
                    // the out-of-bounds case and is reported saturated rather
                    // than wrapped.
                    let past = offset.saturating_add(extent.get());
                    if past > available.get() {
                        return Err(SliceSelectionError::WindowOutOfBounds {
                            axis,
                            offset: *offset,
                            extent: extent.get(),
                            axis_extent: available.get(),
                        });
                    }
                    if extent == available {
                        return Err(SliceSelectionError::WindowIsWholeAxis {
                            axis,
                            extent: available.get(),
                        });
                    }
                    *extent
                }
            };
            result.push(extent);
        }
        Shape::try_new(result).map_err(SliceSelectionError::ResultShape)
    }
}

fn malformed(subject: SliceAttributeSubject) -> SliceSelectionError {
    SliceSelectionError::MalformedAttribute { subject }
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

fn collect_bounded<T>(items: impl IntoIterator<Item = T>) -> Result<Vec<T>, SliceSelectionError> {
    let mut collected = Vec::new();
    for item in items
        .into_iter()
        .take(MAX_SLICE_SELECTION_AXES.saturating_add(1))
    {
        if collected.len() == MAX_SLICE_SELECTION_AXES {
            return Err(SliceSelectionError::TooManyAxes {
                axes: MAX_SLICE_SELECTION_AXES.saturating_add(1),
                limit: MAX_SLICE_SELECTION_AXES,
            });
        }
        collected.push(item);
    }
    Ok(collected)
}

fn decode_axes(value: &CanonicalValue) -> Result<Vec<SliceAxisSelection>, SliceSelectionError> {
    let CanonicalValueView::Sequence(values) = value.view() else {
        return Err(malformed(SliceAttributeSubject::AxisSequence));
    };
    if values.len() > MAX_SLICE_SELECTION_AXES {
        return Err(SliceSelectionError::TooManyAxes {
            axes: values.len(),
            limit: MAX_SLICE_SELECTION_AXES,
        });
    }
    values.iter().map(decode_axis).collect()
}

fn decode_axis(value: &CanonicalValue) -> Result<SliceAxisSelection, SliceSelectionError> {
    let CanonicalValueView::Record(fields) = value.view() else {
        return Err(malformed(SliceAttributeSubject::AxisRecord));
    };
    let Some(relation_field) = fields.first() else {
        return Err(malformed(SliceAttributeSubject::AxisRecord));
    };
    if relation_field.id() != SLICE_AXIS_RELATION {
        return Err(malformed(SliceAttributeSubject::AxisRecord));
    }
    let CanonicalValueView::Utf8(name) = relation_field.value().view() else {
        return Err(malformed(SliceAttributeSubject::Relation));
    };
    // Exactly the fields the relation uses. A whole axis carrying an offset, or a
    // window missing one, is as malformed as a bad relation name: admitting
    // either would let two records denote one entry. The two reserved names are
    // decided before the field shape, so a caller reaching for a form this family
    // does not have is told which form rather than which field.
    match (name, fields) {
        (SLICE_RELATION_STRIDED_WINDOW, _) => Err(SliceSelectionError::StridedSelectionUnsupported),
        (SLICE_RELATION_SYMBOLIC_WINDOW, _) => Err(SliceSelectionError::SymbolicOffsetUnsupported),
        (SLICE_RELATION_WHOLE_AXIS, [_]) => Ok(SliceAxisSelection::WholeAxis),
        (SLICE_RELATION_WINDOW, [_, offset_field, extent_field])
            if offset_field.id() == SLICE_AXIS_OFFSET && extent_field.id() == SLICE_AXIS_EXTENT =>
        {
            Ok(SliceAxisSelection::Window {
                offset: decode_unsigned(offset_field.value(), SliceAttributeSubject::Offset)?,
                extent: Extent::new(decode_unsigned(
                    extent_field.value(),
                    SliceAttributeSubject::WindowExtent,
                )?),
            })
        }
        (SLICE_RELATION_WHOLE_AXIS | SLICE_RELATION_WINDOW, _) => {
            Err(malformed(SliceAttributeSubject::AxisRecord))
        }
        _ => Err(SliceSelectionError::UnadmittedRelation {
            name: bounded_name(name),
        }),
    }
}

fn decode_unsigned(
    value: &CanonicalValue,
    subject: SliceAttributeSubject,
) -> Result<u64, SliceSelectionError> {
    let CanonicalValueView::Unsigned {
        width: CanonicalIntegerWidth::Bits64,
        bits,
    } = value.view()
    else {
        return Err(malformed(subject));
    };
    Ok(bits)
}

/// Builds the canonical attribute value of one validated selection.
///
/// The entries are a sequence *of records* rather than parallel sequences of
/// names, offsets, and extents, and that framing is load-bearing rather than
/// cosmetic: a whole axis has neither an offset nor an extent, so a parallel
/// encoding would need sentinel values, and a sentinel is a value a caller can
/// also write.
fn encode_selection(axes: &[SliceAxisSelection]) -> Result<CanonicalValue, TypeIdentityError> {
    let mut encoded = Vec::with_capacity(axes.len());
    for selection in axes {
        let relation = CanonicalField::new(
            SLICE_AXIS_RELATION,
            CanonicalValue::utf8(selection.canonical_name())?,
        );
        encoded.push(match selection {
            SliceAxisSelection::WholeAxis => CanonicalValue::record([relation])?,
            SliceAxisSelection::Window { offset, extent } => CanonicalValue::record([
                relation,
                CanonicalField::new(SLICE_AXIS_OFFSET, CanonicalValue::unsigned_u64(*offset)),
                CanonicalField::new(
                    SLICE_AXIS_EXTENT,
                    CanonicalValue::unsigned_u64(extent.get()),
                ),
            ])?,
        });
    }
    CanonicalValue::record([CanonicalField::new(
        SLICE_SELECTION_AXES,
        CanonicalValue::sequence(encoded)?,
    )])
}

/// Registers the governed slice family.
pub(super) fn register_standard_slice(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        slice_f32_op(),
        OperationSchema::new(
            OperationArity::exact(1),
            OperationArity::exact(1),
            [OperationAttributeSchema::required(
                SLICE_SELECTION_ATTRIBUTE,
                CanonicalValueKind::Record,
            )],
        )
        .expect("the governed slice schema is valid"),
        NormativeDefinitionRef::new(SLICE_F32_NORMATIVE_DEFINITION)?,
        OperationDefinitionFacts::new(slice_facts()),
        standard_conformance("slice-f32"),
        OperationEffect::Pure,
        Arc::new(SliceF32),
    ))
    // No algebraic capability is declared, deliberately. A selection performs no
    // arithmetic, so it has no associativity or commutativity *of rounding* to
    // declare; that composing two selections along one axis is associative is a
    // structural identity rather than a numerical permission, and a missing
    // declaration reads as unknown rather than as the inverse law.
}

/// The complete normative definition of `tiler::slice-f32@1`.
///
/// Held as a constant rather than written inline because it is where the
/// out-of-bounds posture and the two reserved relations are stated, and a reader
/// looking for either should find it under a name rather than inside a
/// registration call.
const SLICE_F32_NORMATIVE_DEFINITION: &str = concat!(
    "tiler::slice-f32@1; a total output-to-input binary32 coordinate relation that is injective and ",
    "not surjective: every result coordinate reads one operand element, no operand element is read ",
    "twice, and at least one operand element is not read at all. Every result element is an operand ",
    "element unchanged: no value is computed, converted, rounded, or canonicalized, so an exceptional ",
    "payload — a non-canonical NaN, a signalling NaN, a signed zero, a subnormal — arrives at the ",
    "result exactly as it left the operand. ",
    "The selection is stated as exactly one entry per operand axis, in axis order, so a selection has ",
    "one spelling and nothing about a restricted axis is inferred from a shape. ",
    "Admitted relations, and no others: whole-axis, which reads every coordinate of its axis in order; ",
    "and window, which reads a contiguous run of coordinates in order from a literal offset. A ",
    "selection stating only whole axes returns its operand, denotes no slice, and is refused, and a ",
    "window covering its axis entirely is the whole-axis relation and is refused as such — so an ",
    "admitted occurrence restricts at least one axis and leaves at least one coordinate of it unread. ",
    "An empty window is refused rather than producing a result with no elements. That refuses a ",
    "selection stating emptiness and never an operand that is already empty: an operand with a zero ",
    "extent on an axis the selection leaves whole is admitted, and its result has no elements. ",
    "Rank is preserved: a selection restricts coordinates and drops no axis. Removing an extent-one ",
    "axis a selection leaves behind is a tiler::reindex-f32@1 remove-unit-axis occurrence written ",
    "after this one. The result extent of a restricted axis is the window's extent and the result ",
    "extent of a whole axis is the operand's; neither is declared by a caller. ",
    "A window that reads past the end of its axis is refused at construction, naming the axis, the ",
    "offset, the window extent, and the axis extent. It is never clamped to the axis and never ",
    "wrapped: the primary authorities diverge on that convention and the two conventions produce ",
    "different tensors for one program rather than different diagnostics. ",
    "Two relation names are reserved and always refused, each under its own rule rather than as an ",
    "unrecognized name: strided-window, because a stride attribute is one family with the contiguous ",
    "window and whether it admits a negative stride closes on the addressing question that owns it; ",
    "and symbolic-window, because this family selects at literal offsets — no index-expression ",
    "variant carries a bound extent symbol in a coordinate position, and a semantic value fact ",
    "carries static extents. ",
    "This operation makes no claim that storage was copied, viewed, or left alone: it states a ",
    "logical coordinate relation, and every physical realization of it remains a planning outcome.",
);

fn slice_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            SLICE_FACT_VALUE_BEHAVIOUR,
            fact("none-every-result-element-is-an-operand-element-unchanged"),
        ),
        CanonicalField::new(
            SLICE_FACT_MAPPING_CLASS,
            fact(
                "total-over-the-result-domain-and-injective-not-surjective-into-the-operand-domain",
            ),
        ),
        CanonicalField::new(
            SLICE_FACT_STORAGE_CLAIM,
            fact("none-no-copy-view-or-materialization-is-claimed"),
        ),
        CanonicalField::new(SLICE_FACT_ADMITTED_RELATIONS, fact("whole-axis,window")),
        CanonicalField::new(
            SLICE_FACT_OUT_OF_BOUNDS,
            fact("refused-at-construction-never-clamped-and-never-wrapped"),
        ),
    ])
    .expect("the governed slice facts are canonical")
}

fn fact(value: &str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("a governed slice fact is bounded")
}

struct SliceF32;

impl OperationInferencer for SliceF32 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "slice.attributes",
                "a slice requires exactly the selection attribute".to_owned(),
            ));
        }
        let Some(value) = attributes.get(SLICE_SELECTION_ATTRIBUTE) else {
            return Err(op_error(
                "slice.attributes",
                "a slice requires exactly the selection attribute".to_owned(),
            ));
        };
        // The selection's own rules are decided before anything about the
        // occurrence, so a malformed or reserved selection is refused under its
        // own rule rather than under whichever shape check happened to notice
        // first.
        let selection =
            SliceSelection::from_canonical_value(value).map_err(|error| rejection(&error))?;
        let [operand] = operands else {
            return Err(op_error(
                "slice.operands",
                format!(
                    "a slice takes one operand and {} were supplied",
                    operands.len()
                ),
            ));
        };
        if operand.resolved_type() != &F32::resolved_type() {
            return Err(op_error(
                "slice.type",
                "a slice operand must be f32".to_owned(),
            ));
        }
        let shape = selection
            .result_shape(operand.shape())
            .map_err(|error| rejection(&error))?;
        outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
    }
}

fn rejection(error: &SliceSelectionError) -> OperationInferenceError {
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
