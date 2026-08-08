//! The governed `Reindex` family and its closed vocabulary of mapping forms.
//!
//! **What a `Reindex` is.** A total output-to-input coordinate function together
//! with the shape constraints that make it total. It rearranges *which*
//! coordinate reads which element and changes no value: every admitted form is a
//! bijection between the output domain and the input domain, so the result is a
//! permutation of the operand's elements and nothing is read twice, dropped, or
//! invented. Many-to-one behaviour is a different family — [`super::broadcast`]
//! — and one-to-fewer behaviour is a slice, which no family here admits.
//!
//! **It makes no storage claim.** Registering a `Reindex` occurrence says that
//! the *logical* coordinates were remapped. It does not claim that storage was
//! transposed, copied, materialized, or left alone; whether an occurrence costs a
//! dispatch, becomes an access-map composition inside a neighbouring kernel, or
//! disappears entirely is a physical-planning outcome, and this definition
//! deliberately fixes none of it.
//!
//! **A closed form vocabulary, not a general coordinate function.** The attribute
//! carries one named form from [`ReindexFormKind`], never an arbitrary map. That
//! is what keeps the family decidable: totality and bijectivity are properties
//! this module *proves* per form against the operand's shape, rather than
//! properties a caller asserts. A form outside the vocabulary is refused by name
//! at construction under [`ReindexFormError::UnadmittedFormKind`] rather than
//! approximated, so a mapping this family cannot express fails closed at the
//! semantic boundary instead of at lowering.
//!
//! # Decision D-10, settled here
//!
//! [The L4 attention design](../../../../docs/research/program-planning/first-attention-program-vertical.md)
//! left one question open: whether "bijective permutation" covers a coordinate
//! permutation *within* an axis, or only a permutation *of* axes. The workload
//! needs exactly one such map — `(…, i, j) -> (…, 1 − i, j)` on a size-2 axis,
//! which `rotate_half` performs inside rotary position embedding.
//!
//! **The answer is yes, by one named form and not as a general reading.**
//! [`ReindexFormKind::ReverseAxis`] admits the within-axis coordinate map
//! `i -> extent − 1 − i`; at extent two that is exactly `i -> 1 − i`. No other
//! within-axis permutation is admitted, and presenting one is a named refusal.
//!
//! The derivation, so a reader can refute the elimination rather than only the
//! conclusion:
//!
//! 1. **Refusing outright buys no invariant.** The composition is measured
//!    correct — L4 reproduced the pinned reference's `rotate_half` at 0 of 20,480
//!    elements on a `[1, 16, 10, 128]` operand, with the swap removed and the
//!    sign reversed each differing at all 20,480 — and its access map is
//!    expressible in the accepted bounded index vocabulary, which admits
//!    `1 + (−i)`. A refusal would not protect the family from an unrepresentable
//!    or unprovable case; it would only send the workload to a slice plus a
//!    concatenate, two families that have no normative contract anywhere in this
//!    corpus.
//! 2. **Admitting the general reading admits what cannot lower.** An arbitrary
//!    within-axis permutation is a permutation *table*: its size is the axis
//!    extent, it is undefined for a symbolic extent, and applying it is a
//!    tensor-data-derived index, which the accepted index-expression vocabulary
//!    rejects outright. The family would then admit at construction a mapping no
//!    lowering can ever produce, which is the failure the "reject rather than
//!    normalize" rule exists to prevent.
//! 3. **The named form is not a narrowing of the affine class; it is that
//!    class.** Within the bounded vocabulary an affine within-axis map is
//!    `i -> a·i + b`. It carries `{0, …, n−1}` onto itself exactly when the image
//!    — an arithmetic progression of `n` terms with common difference `a` — is
//!    `n` consecutive integers, so `|a| = 1`; `a = 1` forces `b = 0` and is the
//!    identity, and `a = −1` forces `b = n − 1` and is the reversal. The affine
//!    within-axis bijections of an axis are therefore exactly the identity and
//!    the reversal, and admitting `ReverseAxis` admits all of them that do
//!    anything.
//! 4. **What is deliberately still refused, and what admitting it would need.**
//!    A rotation `i -> (i + k) mod n` is a within-axis bijection the vocabulary
//!    can also express, quasi-affinely. No occurrence in the pinned workload
//!    needs one, so it is refused by name rather than admitted speculatively.
//!    Admitting it would require a `k` attribute participating in canonical
//!    identity, a proof that the modulus is positive, and a reference evaluator
//!    and conformance row of its own — none of which a consumer has asked for.
//!
//! This settles D-10 for the semantic vocabulary. The family's normative
//! reference states the answer, so a reader gets it from the registered
//! definition rather than from a research record.
//!
//! # What is admitted today
//!
//! Six forms, in `tiler::f32@1` throughout, at any rank the governed shape
//! profile admits. Registering this key gives a reindex a semantic identity and a
//! validated form; the separate lowering, fusion, and target capabilities decide
//! whether an occurrence is planable, schedulable, or executable.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::identity::push_slice;
use crate::shape::{Axis, Extent, Shape};

use super::registry::standard_conformance;
use super::{
    AttributeFieldId, CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind,
    CanonicalValueView, F32, NormativeDefinitionRef, OpKey, OperationArity,
    OperationAttributeSchema, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, ProviderDiagnosticCode, RegistryError,
    SemanticRegistryRegistrar, TypeIdentityError, ValueFact,
};

/// Maximum axes or factors one reindex form may name.
///
/// The same bound a canonical sequence admits, so an oversized form is refused
/// with a reindex-shaped diagnostic rather than an anonymous canonical-bound one.
/// It is below the governed shape rank limit, so a form over a legal but very
/// wide shape is refused rather than silently truncated.
pub const MAX_REINDEX_FORM_ITEMS: usize = super::types::MAX_RESOLVED_TYPE_ITEMS;

/// Stable field ID carrying the canonical mapping form on the reindex.
pub const REINDEX_MAPPING_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// Form-record field naming which admitted form this mapping is.
///
/// The five constants below are fields of the *reindex form record*, which is a
/// different record from every other record in this corpus; equal integers
/// across records are unrelated, and renumbering a published ID is a breaking
/// identity change. Each form requires exactly the fields its own decoding
/// names: an absent field is a malformed record, never a default, and a present
/// field another form does not use is equally malformed.
pub const REINDEX_FORM_KIND: AttributeFieldId = AttributeFieldId::new(1);
/// Form-record field naming the single subject axis of a form that has one.
pub const REINDEX_FORM_AXIS: AttributeFieldId = AttributeFieldId::new(2);
/// Form-record field carrying an ordered axis sequence.
pub const REINDEX_FORM_AXES: AttributeFieldId = AttributeFieldId::new(3);
/// Form-record field carrying an ordered extent sequence.
pub const REINDEX_FORM_FACTORS: AttributeFieldId = AttributeFieldId::new(4);

/// Fact field naming what a reindex does to values.
///
/// The four fields below are the family's semantic signature. Every one is
/// unconditional on this definition: absence is a malformed record, never a
/// default. None of them is numerical, because a reindex performs no arithmetic
/// — which is itself the fact a reader needs, and the reason this family's
/// signature is four fields where the contraction's is fourteen.
pub const REINDEX_FACT_VALUE_BEHAVIOUR: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming the mapping's totality and bijectivity guarantee.
pub const REINDEX_FACT_MAPPING_CLASS: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming what this family claims about storage.
pub const REINDEX_FACT_STORAGE_CLAIM: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field naming the closed set of admitted mapping forms.
pub const REINDEX_FACT_ADMITTED_FORMS: AttributeFieldId = AttributeFieldId::new(4);

/// Canonical name of the axis-permutation form.
///
/// The six names below are canonical identity, not display text: the form record
/// carries the exact string, so respelling one changes every occurrence's
/// attribute bytes. An unrecognized name is refused rather than mapped to a
/// nearest match.
pub const REINDEX_FORM_PERMUTE_AXES: &str = "permute-axes";
/// Canonical name of the axis-splitting form.
pub const REINDEX_FORM_SPLIT_AXIS: &str = "split-axis";
/// Canonical name of the adjacent-axis-merging form.
pub const REINDEX_FORM_MERGE_AXES: &str = "merge-axes";
/// Canonical name of the unit-axis-insertion form.
pub const REINDEX_FORM_INSERT_UNIT_AXIS: &str = "insert-unit-axis";
/// Canonical name of the unit-axis-removal form.
pub const REINDEX_FORM_REMOVE_UNIT_AXIS: &str = "remove-unit-axis";
/// Canonical name of the within-axis reversal form that settles D-10.
pub const REINDEX_FORM_REVERSE_AXIS: &str = "reverse-axis";

/// Domain separator of a canonical reindex form encoding.
const REINDEX_FORM_DOMAIN: &[u8] = b"tiler.reindex-form.v1\0";

/// Returns the governed binary32 reindex operation key.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn reindex_f32_op() -> OpKey {
    OpKey::new("tiler", "reindex-f32", 1).expect("the governed reindex key is valid")
}

/// Which admitted coordinate mapping one reindex form states.
///
/// Deliberately **not** `#[non_exhaustive]`, on the precedent
/// [`super::OperationEffect`] sets and for the same reason: consumers outside
/// this crate map this vocabulary *totally* — the governed index-access lowering
/// derives a different coordinate relation per form, and no wildcard coordinate
/// map is derivable from a form it has not seen. Admitting a seventh form must
/// therefore be a build error at every such site rather than a silent fall
/// through to a refusal that reads as "unsupported" when the real state is
/// "unimplemented". The attribute would buy back one in-workspace source edit
/// `cargo check` enumerates; it would cost the guarantee that a widened family
/// cannot ship with a lowering that never learned about it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReindexFormKind {
    /// Reorders whole axes: `output[k]` is the operand's axis `order[k]`.
    PermuteAxes,
    /// Replaces one axis by a row-major factorization of it, major factor first.
    SplitAxis,
    /// Replaces a run of adjacent axes by their row-major product.
    MergeAxes,
    /// Inserts one extent-one axis at a position of the result.
    InsertUnitAxis,
    /// Removes one extent-one axis of the operand.
    RemoveUnitAxis,
    /// Reverses one axis's coordinates: `i -> extent − 1 − i`. Settles D-10.
    ReverseAxis,
}

impl ReindexFormKind {
    /// Returns the canonical name this form carries in its attribute record.
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::PermuteAxes => REINDEX_FORM_PERMUTE_AXES,
            Self::SplitAxis => REINDEX_FORM_SPLIT_AXIS,
            Self::MergeAxes => REINDEX_FORM_MERGE_AXES,
            Self::InsertUnitAxis => REINDEX_FORM_INSERT_UNIT_AXIS,
            Self::RemoveUnitAxis => REINDEX_FORM_REMOVE_UNIT_AXIS,
            Self::ReverseAxis => REINDEX_FORM_REVERSE_AXIS,
        }
    }

    fn from_canonical_name(name: &str) -> Option<Self> {
        match name {
            REINDEX_FORM_PERMUTE_AXES => Some(Self::PermuteAxes),
            REINDEX_FORM_SPLIT_AXIS => Some(Self::SplitAxis),
            REINDEX_FORM_MERGE_AXES => Some(Self::MergeAxes),
            REINDEX_FORM_INSERT_UNIT_AXIS => Some(Self::InsertUnitAxis),
            REINDEX_FORM_REMOVE_UNIT_AXIS => Some(Self::RemoveUnitAxis),
            REINDEX_FORM_REVERSE_AXIS => Some(Self::ReverseAxis),
            _ => None,
        }
    }
}

impl fmt::Display for ReindexFormKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.canonical_name())
    }
}

/// Which part of a malformed form attribute was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReindexAttributeSubject {
    /// The attribute was not a form record.
    FormRecord,
    /// The record's field set was not the one this form requires.
    FormFields,
    /// The kind field was not canonical UTF-8.
    FormKind,
    /// The axis field was not a canonical unsigned 32-bit value.
    Axis,
    /// The axis-sequence field was not a sequence of canonical axes.
    AxisSequence,
    /// The factor-sequence field was not a sequence of canonical extents.
    FactorSequence,
}

impl fmt::Display for ReindexAttributeSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FormRecord => formatter.write_str("form record"),
            Self::FormFields => formatter.write_str("form field set"),
            Self::FormKind => formatter.write_str("form kind"),
            Self::Axis => formatter.write_str("axis"),
            Self::AxisSequence => formatter.write_str("axis sequence"),
            Self::FactorSequence => formatter.write_str("factor sequence"),
        }
    }
}

/// A typed refusal of one reindex mapping form.
///
/// Every variant is one named admission rule. A malformed form is never a
/// generic invalidity and never a value that reaches identity, planning, explain
/// output, or a cache subject: [`ReindexForm`] has no unchecked constructor and
/// [`ReindexForm::result_shape`] is the only path to a result, so holding a
/// result is evidence that both the form's own rules and its rules against the
/// operand were decided.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReindexFormError {
    /// The named form is not one this family admits.
    ///
    /// This is where every unadmitted coordinate map lands, and in particular
    /// every within-axis permutation that is not the reversal D-10 admits. It is
    /// refused by name rather than approximated by a nearest admitted form,
    /// because approximating a coordinate map returns a plausible tensor that is
    /// the wrong one.
    UnadmittedFormKind {
        /// The rejected name, truncated to a bounded prefix.
        name: String,
    },
    /// An axis order was not a permutation of the operand's axes.
    ///
    /// The non-bijective half of the family's admission rule for this form: an
    /// order that repeats an axis reads one axis twice and drops another, which
    /// is neither a reindex nor any admitted family.
    NotAPermutation {
        /// The first axis named twice, or named out of range.
        axis: Axis,
    },
    /// An axis order named a different number of axes than the operand has.
    PermutationRank {
        /// Axes the order names.
        order: usize,
        /// Axes the operand has.
        operand: usize,
    },
    /// The form moves nothing, so it denotes no reindex.
    ///
    /// Not a correctness failure but a canonicality rule, following the
    /// contraction's precedent of refusing a structure that denotes no
    /// contraction. An identity permutation, a one-factor split, a one-axis
    /// merge, and a reversal of an axis shorter than two coordinates all name an
    /// operation that returns its operand, and admitting them would give one
    /// program many identities.
    IdentityMapping {
        /// The form that named no change.
        kind: ReindexFormKind,
    },
    /// A named axis does not exist on the operand.
    AxisOutOfRange {
        /// The named axis.
        axis: Axis,
        /// The operand's rank.
        rank: usize,
    },
    /// A split declared factors whose product exceeds the axis extent.
    ///
    /// The map is then not total over its declared output domain: the largest
    /// output coordinate reads past the end of the operand's axis.
    SplitNotTotal {
        /// The split axis.
        axis: Axis,
        /// The declared factors' product.
        product: u64,
        /// The axis extent it must equal.
        extent: u64,
    },
    /// A split declared factors whose product is short of the axis extent.
    ///
    /// The map is then total and injective but not surjective: it reads a prefix
    /// of the axis and never the rest. That is a slice — a different family, with
    /// a different access relation and no normative contract in this corpus — so
    /// it is refused by name rather than admitted as a narrow reindex.
    SplitNotSurjective {
        /// The split axis.
        axis: Axis,
        /// The declared factors' product.
        product: u64,
        /// The axis extent it must equal.
        extent: u64,
    },
    /// A merge named axes that are not a strictly ascending adjacent run.
    ///
    /// A merge of non-adjacent axes is a permutation composed with a merge, and
    /// this family spells a composition as a chain of occurrences rather than
    /// folding two maps into one attribute.
    MergeAxesNotAdjacent {
        /// The first axis that did not follow its predecessor.
        axis: Axis,
        /// The axis it was required to follow.
        previous: Axis,
    },
    /// A unit-axis removal named an axis whose extent is not one.
    ///
    /// The removal is legal only for an extent-one axis; removing any other
    /// would drop elements, which is not a bijection.
    RemovedAxisNotUnit {
        /// The named axis.
        axis: Axis,
        /// Its extent.
        extent: u64,
    },
    /// A unit-axis insertion named a position beyond the result's rank.
    InsertPositionOutOfRange {
        /// The named position.
        axis: Axis,
        /// The largest admitted position, which is the operand's rank.
        limit: usize,
    },
    /// The form named more axes or factors than one canonical sequence admits.
    TooManyItems {
        /// First rejected item count.
        items: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// The attribute was not a well-formed form record.
    MalformedAttribute {
        /// The rejected part.
        subject: ReindexAttributeSubject,
    },
    /// The form exceeded a canonical structural bound.
    CanonicalBound(TypeIdentityError),
    /// The result shape exceeded the governed rank profile.
    ResultShape(crate::shape::ShapeError),
}

impl ReindexFormError {
    /// Returns the stable provider diagnostic code naming this refusal.
    ///
    /// Each rule has its own code, so a caller reads *which* rule refused from
    /// the code rather than by matching on a message.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::UnadmittedFormKind { .. } => "reindex.form.unadmitted-kind",
            Self::NotAPermutation { .. } => "reindex.permute.not-a-permutation",
            Self::PermutationRank { .. } => "reindex.permute.rank",
            Self::IdentityMapping { .. } => "reindex.form.identity-mapping",
            Self::AxisOutOfRange { .. } => "reindex.form.axis-out-of-range",
            Self::SplitNotTotal { .. } => "reindex.split.not-total",
            Self::SplitNotSurjective { .. } => "reindex.split.not-surjective",
            Self::MergeAxesNotAdjacent { .. } => "reindex.merge.non-adjacent-axes",
            Self::RemovedAxisNotUnit { .. } => "reindex.remove-unit-axis.not-unit",
            Self::InsertPositionOutOfRange { .. } => "reindex.insert-unit-axis.out-of-range",
            Self::TooManyItems { .. } => "reindex.form.too-many-items",
            Self::MalformedAttribute { .. } => "reindex.form.malformed-attribute",
            Self::CanonicalBound(_) => "reindex.form.canonical-bound",
            Self::ResultShape(_) => "reindex.form.result-shape",
        }
    }
}

impl fmt::Display for ReindexFormError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnadmittedFormKind { name } => write!(
                formatter,
                "{name} is not an admitted reindex form; the admitted forms are {REINDEX_FORM_PERMUTE_AXES}, {REINDEX_FORM_SPLIT_AXIS}, {REINDEX_FORM_MERGE_AXES}, {REINDEX_FORM_INSERT_UNIT_AXIS}, {REINDEX_FORM_REMOVE_UNIT_AXIS}, and {REINDEX_FORM_REVERSE_AXIS}, and no other within-axis coordinate permutation is admitted"
            ),
            Self::NotAPermutation { axis } => write!(
                formatter,
                "the axis order is not a permutation: axis {} is named twice or out of range",
                axis.get()
            ),
            Self::PermutationRank { order, operand } => write!(
                formatter,
                "the axis order names {order} axes and the operand has {operand}"
            ),
            Self::IdentityMapping { kind } => write!(
                formatter,
                "the {kind} form as stated returns its operand unchanged, so it denotes no reindex"
            ),
            Self::AxisOutOfRange { axis, rank } => write!(
                formatter,
                "axis {} does not exist on an operand of rank {rank}",
                axis.get()
            ),
            Self::SplitNotTotal {
                axis,
                product,
                extent,
            } => write!(
                formatter,
                "the split of axis {} declares factors whose product is {product}, exceeding the extent {extent}, so the mapping is not total over its output domain",
                axis.get()
            ),
            Self::SplitNotSurjective {
                axis,
                product,
                extent,
            } => write!(
                formatter,
                "the split of axis {} declares factors whose product is {product}, short of the extent {extent}, so the mapping reads a prefix of the axis and is a slice rather than a reindex",
                axis.get()
            ),
            Self::MergeAxesNotAdjacent { axis, previous } => write!(
                formatter,
                "the merge names axis {} after axis {}, and a merge names a strictly ascending adjacent run",
                axis.get(),
                previous.get()
            ),
            Self::RemovedAxisNotUnit { axis, extent } => write!(
                formatter,
                "axis {} has extent {extent}, and only an extent-one axis may be removed",
                axis.get()
            ),
            Self::InsertPositionOutOfRange { axis, limit } => write!(
                formatter,
                "position {} is beyond {limit}, the largest position a unit axis may be inserted at",
                axis.get()
            ),
            Self::TooManyItems { items, limit } => {
                write!(formatter, "the form names {items} items, exceeding {limit}")
            }
            Self::MalformedAttribute { subject } => {
                write!(formatter, "the {subject} is malformed")
            }
            Self::CanonicalBound(source) => {
                write!(formatter, "the form exceeds a canonical bound: {source}")
            }
            Self::ResultShape(source) => {
                write!(formatter, "the result shape is not admitted: {source}")
            }
        }
    }
}

impl Error for ReindexFormError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalBound(source) => Some(source),
            Self::ResultShape(source) => Some(source),
            _ => None,
        }
    }
}

/// Collision-free canonical encoding of one reindex mapping form.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalReindexForm(Vec<u8>);

impl CanonicalReindexForm {
    /// Returns the domain-separated canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A validated reindex mapping form.
///
/// Construction decides every rule that is a property of the form alone; the
/// rules that need an operand — axis ranges, extent products, unit extents, and
/// the degenerate cases that depend on an extent — are decided by
/// [`Self::result_shape`], which is what the registered inference routine calls.
/// The split mirrors the contraction's: a rule that is decidable without the
/// occurrence is reported before the occurrence is consulted, so a malformed
/// form is refused under its own rule rather than under whichever occurrence
/// check happened to notice first.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReindexForm {
    kind: ReindexFormKind,
    axes: Vec<Axis>,
    factors: Vec<Extent>,
    canonical_value: CanonicalValue,
}

impl ReindexForm {
    /// Builds an axis permutation, stated as the operand axis each result axis reads.
    ///
    /// `order[k]` is the operand axis that becomes result axis `k`, so
    /// `[1, 0]` transposes a matrix and `[1, 2, 0]` moves the outermost axis to
    /// the innermost position.
    ///
    /// # Errors
    ///
    /// Returns [`ReindexFormError`] when the order repeats an axis, names more
    /// axes than one canonical sequence admits, or is the identity.
    pub fn permute_axes(order: impl IntoIterator<Item = Axis>) -> Result<Self, ReindexFormError> {
        let axes = collect_bounded(order)?;
        let mut seen = BTreeSet::new();
        for axis in &axes {
            if !seen.insert(*axis) {
                return Err(ReindexFormError::NotAPermutation { axis: *axis });
            }
        }
        // A permutation of `rank` axes is a permutation of `0..rank` exactly when
        // its axes are distinct and its largest is `rank - 1`. Distinctness is
        // above; the range check needs the operand's rank and belongs to
        // `result_shape`, where the rank is known.
        if axes
            .iter()
            .enumerate()
            .all(|(position, axis)| u32::try_from(position) == Ok(axis.get()))
        {
            return Err(ReindexFormError::IdentityMapping {
                kind: ReindexFormKind::PermuteAxes,
            });
        }
        Self::finish(ReindexFormKind::PermuteAxes, axes, Vec::new())
    }

    /// Builds a row-major split of one axis, major factor first.
    ///
    /// A split of a `[…, 128, …]` axis into `[2, 64]` gives result coordinates
    /// `(i, j)` reading operand coordinate `64·i + j`, so the *first* factor is
    /// the major one. That direction is load-bearing for the grouped-query head
    /// layout, where splitting sixteen heads into `(8, 2)` makes the group index
    /// `h / 2` rather than `h % 8`.
    ///
    /// # Errors
    ///
    /// Returns [`ReindexFormError`] when fewer than two factors are named or more
    /// items than one canonical sequence admits are.
    pub fn split_axis(
        axis: Axis,
        factors: impl IntoIterator<Item = Extent>,
    ) -> Result<Self, ReindexFormError> {
        let factors = collect_bounded(factors)?;
        if factors.len() < 2 {
            return Err(ReindexFormError::IdentityMapping {
                kind: ReindexFormKind::SplitAxis,
            });
        }
        Self::finish(ReindexFormKind::SplitAxis, vec![axis], factors)
    }

    /// Builds a row-major merge of a strictly ascending adjacent axis run.
    ///
    /// # Errors
    ///
    /// Returns [`ReindexFormError`] when the axes are not a strictly ascending
    /// adjacent run, when fewer than two are named, or when more items than one
    /// canonical sequence admits are.
    pub fn merge_axes(axes: impl IntoIterator<Item = Axis>) -> Result<Self, ReindexFormError> {
        let axes = collect_bounded(axes)?;
        if axes.len() < 2 {
            return Err(ReindexFormError::IdentityMapping {
                kind: ReindexFormKind::MergeAxes,
            });
        }
        for window in axes.windows(2) {
            let [previous, current] = window else {
                unreachable!("windows(2) yields pairs");
            };
            if current.get() != previous.get().saturating_add(1) {
                return Err(ReindexFormError::MergeAxesNotAdjacent {
                    axis: *current,
                    previous: *previous,
                });
            }
        }
        Self::finish(ReindexFormKind::MergeAxes, axes, Vec::new())
    }

    /// Builds an insertion of one extent-one axis at a result position.
    ///
    /// # Errors
    ///
    /// Returns [`ReindexFormError`] only from the canonical encoder; the
    /// position is checked against the operand by [`Self::result_shape`].
    pub fn insert_unit_axis(axis: Axis) -> Result<Self, ReindexFormError> {
        Self::finish(ReindexFormKind::InsertUnitAxis, vec![axis], Vec::new())
    }

    /// Builds a removal of one extent-one operand axis.
    ///
    /// # Errors
    ///
    /// Returns [`ReindexFormError`] only from the canonical encoder; the axis and
    /// its extent are checked against the operand by [`Self::result_shape`].
    pub fn remove_unit_axis(axis: Axis) -> Result<Self, ReindexFormError> {
        Self::finish(ReindexFormKind::RemoveUnitAxis, vec![axis], Vec::new())
    }

    /// Builds the within-axis reversal `i -> extent − 1 − i` that settles D-10.
    ///
    /// At extent two this is `i -> 1 − i`, the coordinate swap `rotate_half`
    /// performs. The result shape equals the operand's shape; only the reading
    /// order of that axis changes.
    ///
    /// # Errors
    ///
    /// Returns [`ReindexFormError`] only from the canonical encoder; the axis and
    /// its extent are checked against the operand by [`Self::result_shape`],
    /// which refuses an axis of fewer than two coordinates as an identity.
    pub fn reverse_axis(axis: Axis) -> Result<Self, ReindexFormError> {
        Self::finish(ReindexFormKind::ReverseAxis, vec![axis], Vec::new())
    }

    /// Decodes one form attribute exactly as an occurrence carries it.
    ///
    /// # Errors
    ///
    /// Returns [`ReindexFormError`] for a malformed record, an unadmitted form
    /// name, or a violated form rule. The form's own rules are re-decided here
    /// rather than trusted, because a hand-assembled attribute never passed a
    /// constructor.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, ReindexFormError> {
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(malformed(ReindexAttributeSubject::FormRecord));
        };
        let Some(kind_field) = fields.first() else {
            return Err(malformed(ReindexAttributeSubject::FormRecord));
        };
        if kind_field.id() != REINDEX_FORM_KIND {
            return Err(malformed(ReindexAttributeSubject::FormRecord));
        }
        let CanonicalValueView::Utf8(name) = kind_field.value().view() else {
            return Err(malformed(ReindexAttributeSubject::FormKind));
        };
        let Some(kind) = ReindexFormKind::from_canonical_name(name) else {
            return Err(ReindexFormError::UnadmittedFormKind {
                name: bounded_name(name),
            });
        };
        // Exactly the fields this form uses, in the order the encoder writes
        // them. A field the form does not use is as malformed as a missing one:
        // admitting an extra would let two attribute records denote one form.
        match (kind, fields) {
            (ReindexFormKind::PermuteAxes, [_, axes_field])
                if axes_field.id() == REINDEX_FORM_AXES =>
            {
                Self::permute_axes(decode_axes(axes_field.value())?)
            }
            (ReindexFormKind::SplitAxis, [_, axis_field, factors_field])
                if axis_field.id() == REINDEX_FORM_AXIS
                    && factors_field.id() == REINDEX_FORM_FACTORS =>
            {
                Self::split_axis(
                    decode_axis(axis_field.value())?,
                    decode_factors(factors_field.value())?,
                )
            }
            (ReindexFormKind::MergeAxes, [_, axes_field])
                if axes_field.id() == REINDEX_FORM_AXES =>
            {
                Self::merge_axes(decode_axes(axes_field.value())?)
            }
            (ReindexFormKind::InsertUnitAxis, [_, axis_field])
                if axis_field.id() == REINDEX_FORM_AXIS =>
            {
                Self::insert_unit_axis(decode_axis(axis_field.value())?)
            }
            (ReindexFormKind::RemoveUnitAxis, [_, axis_field])
                if axis_field.id() == REINDEX_FORM_AXIS =>
            {
                Self::remove_unit_axis(decode_axis(axis_field.value())?)
            }
            (ReindexFormKind::ReverseAxis, [_, axis_field])
                if axis_field.id() == REINDEX_FORM_AXIS =>
            {
                Self::reverse_axis(decode_axis(axis_field.value())?)
            }
            _ => Err(malformed(ReindexAttributeSubject::FormFields)),
        }
    }

    fn finish(
        kind: ReindexFormKind,
        axes: Vec<Axis>,
        factors: Vec<Extent>,
    ) -> Result<Self, ReindexFormError> {
        let canonical_value =
            encode_form(kind, &axes, &factors).map_err(ReindexFormError::CanonicalBound)?;
        Ok(Self {
            kind,
            axes,
            factors,
            canonical_value,
        })
    }

    /// Returns which admitted form this mapping is.
    #[must_use]
    pub const fn kind(&self) -> ReindexFormKind {
        self.kind
    }

    /// Returns the axes this form names, in the order it names them.
    #[must_use]
    pub fn axes(&self) -> &[Axis] {
        &self.axes
    }

    /// Returns the factors a split declares, and nothing for every other form.
    #[must_use]
    pub fn factors(&self) -> &[Extent] {
        &self.factors
    }

    /// Returns the canonical attribute value an occurrence carries.
    #[must_use]
    pub const fn canonical_value(&self) -> &CanonicalValue {
        &self.canonical_value
    }

    /// Returns the domain-separated canonical encoding of this form.
    ///
    /// Derived from [`Self::canonical_value`] rather than from a second walk of
    /// the form, so the identity a reader compares and the attribute an
    /// occurrence carries cannot disagree about what a form is.
    #[must_use]
    pub fn canonical_encoding(&self) -> CanonicalReindexForm {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, REINDEX_FORM_DOMAIN);
        self.canonical_value.encode(&mut bytes);
        CanonicalReindexForm(bytes)
    }

    /// Decides this form against one operand shape and derives the result shape.
    ///
    /// This is where totality and bijectivity are *proved* rather than asserted:
    /// every admitted form's result shape is derived from the operand's extents,
    /// so a result exists only when the coordinate function is total over it and
    /// bijective onto the operand's domain.
    ///
    /// # Errors
    ///
    /// Returns [`ReindexFormError`] naming the violated rule.
    pub fn result_shape(&self, operand: &Shape) -> Result<Shape, ReindexFormError> {
        let rank = operand.rank();
        let extents = operand.extents();
        let extent_of = |axis: Axis| -> Result<Extent, ReindexFormError> {
            usize::try_from(axis.get())
                .ok()
                .and_then(|index| extents.get(index).copied())
                .ok_or(ReindexFormError::AxisOutOfRange { axis, rank })
        };
        let result: Vec<Extent> = match self.kind {
            ReindexFormKind::PermuteAxes => {
                if self.axes.len() != rank {
                    return Err(ReindexFormError::PermutationRank {
                        order: self.axes.len(),
                        operand: rank,
                    });
                }
                // Distinctness was decided at construction, so an axis outside
                // `0..rank` here is the remaining way an order of the right
                // length fails to be a permutation, and it is reported as one.
                self.axes
                    .iter()
                    .map(|axis| {
                        extent_of(*axis)
                            .map_err(|_| ReindexFormError::NotAPermutation { axis: *axis })
                    })
                    .collect::<Result<_, _>>()?
            }
            ReindexFormKind::SplitAxis => {
                let axis = self.axes[0];
                let extent = extent_of(axis)?;
                let product = self
                    .factors
                    .iter()
                    .try_fold(1_u64, |product, factor| product.checked_mul(factor.get()));
                match product {
                    Some(product) if product == extent.get() => {}
                    Some(product) if product < extent.get() => {
                        return Err(ReindexFormError::SplitNotSurjective {
                            axis,
                            product,
                            extent: extent.get(),
                        });
                    }
                    // An overflowing product cannot equal a `u64` extent, so it
                    // is the not-total case and is reported with the saturated
                    // product rather than a wrapped one.
                    Some(product) => {
                        return Err(ReindexFormError::SplitNotTotal {
                            axis,
                            product,
                            extent: extent.get(),
                        });
                    }
                    None => {
                        return Err(ReindexFormError::SplitNotTotal {
                            axis,
                            product: u64::MAX,
                            extent: extent.get(),
                        });
                    }
                }
                let mut result = extents.to_vec();
                let position = usize::try_from(axis.get()).unwrap_or(usize::MAX);
                result.splice(position..=position, self.factors.iter().copied());
                result
            }
            ReindexFormKind::MergeAxes => {
                let mut merged = 1_u64;
                for axis in &self.axes {
                    // The run is ascending and adjacent by construction, so
                    // checking each axis is in range checks the whole run.
                    merged = merged.checked_mul(extent_of(*axis)?.get()).ok_or(
                        ReindexFormError::TooManyItems {
                            items: self.axes.len(),
                            limit: MAX_REINDEX_FORM_ITEMS,
                        },
                    )?;
                }
                let first = usize::try_from(self.axes[0].get()).unwrap_or(usize::MAX);
                let last = first.saturating_add(self.axes.len());
                let mut result = extents.to_vec();
                result.splice(first..last, [Extent::new(merged)]);
                result
            }
            ReindexFormKind::InsertUnitAxis => {
                let position = usize::try_from(self.axes[0].get()).unwrap_or(usize::MAX);
                if position > rank {
                    return Err(ReindexFormError::InsertPositionOutOfRange {
                        axis: self.axes[0],
                        limit: rank,
                    });
                }
                let mut result = extents.to_vec();
                result.insert(position, Extent::new(1));
                result
            }
            ReindexFormKind::RemoveUnitAxis => {
                let axis = self.axes[0];
                let extent = extent_of(axis)?;
                if extent.get() != 1 {
                    return Err(ReindexFormError::RemovedAxisNotUnit {
                        axis,
                        extent: extent.get(),
                    });
                }
                let position = usize::try_from(axis.get()).unwrap_or(usize::MAX);
                let mut result = extents.to_vec();
                result.remove(position);
                result
            }
            ReindexFormKind::ReverseAxis => {
                let axis = self.axes[0];
                // `i -> extent - 1 - i` is the identity at extent one and issues
                // no access at extent zero, so neither denotes a reindex.
                if extent_of(axis)?.get() < 2 {
                    return Err(ReindexFormError::IdentityMapping {
                        kind: ReindexFormKind::ReverseAxis,
                    });
                }
                extents.to_vec()
            }
        };
        Shape::try_new(result).map_err(ReindexFormError::ResultShape)
    }
}

fn malformed(subject: ReindexAttributeSubject) -> ReindexFormError {
    ReindexFormError::MalformedAttribute { subject }
}

/// Truncates a rejected form name to a bounded prefix.
///
/// A diagnostic message has a governed byte bound, and the rejected name comes
/// from an attribute a caller assembled. Truncating here keeps the refusal a
/// refusal instead of turning it into a provider-contract failure about the
/// message's own length.
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

fn collect_bounded<T>(items: impl IntoIterator<Item = T>) -> Result<Vec<T>, ReindexFormError> {
    let mut collected = Vec::new();
    for item in items
        .into_iter()
        .take(MAX_REINDEX_FORM_ITEMS.saturating_add(1))
    {
        if collected.len() == MAX_REINDEX_FORM_ITEMS {
            return Err(ReindexFormError::TooManyItems {
                items: MAX_REINDEX_FORM_ITEMS.saturating_add(1),
                limit: MAX_REINDEX_FORM_ITEMS,
            });
        }
        collected.push(item);
    }
    Ok(collected)
}

fn decode_axis(value: &CanonicalValue) -> Result<Axis, ReindexFormError> {
    let CanonicalValueView::Unsigned {
        width: CanonicalIntegerWidth::Bits32,
        bits,
    } = value.view()
    else {
        return Err(malformed(ReindexAttributeSubject::Axis));
    };
    u32::try_from(bits)
        .map(Axis::new)
        .map_err(|_| malformed(ReindexAttributeSubject::Axis))
}

fn decode_axes(value: &CanonicalValue) -> Result<Vec<Axis>, ReindexFormError> {
    let CanonicalValueView::Sequence(values) = value.view() else {
        return Err(malformed(ReindexAttributeSubject::AxisSequence));
    };
    if values.len() > MAX_REINDEX_FORM_ITEMS {
        return Err(ReindexFormError::TooManyItems {
            items: values.len(),
            limit: MAX_REINDEX_FORM_ITEMS,
        });
    }
    values
        .iter()
        .map(|value| {
            decode_axis(value).map_err(|_| malformed(ReindexAttributeSubject::AxisSequence))
        })
        .collect()
}

fn decode_factors(value: &CanonicalValue) -> Result<Vec<Extent>, ReindexFormError> {
    let CanonicalValueView::Sequence(values) = value.view() else {
        return Err(malformed(ReindexAttributeSubject::FactorSequence));
    };
    if values.len() > MAX_REINDEX_FORM_ITEMS {
        return Err(ReindexFormError::TooManyItems {
            items: values.len(),
            limit: MAX_REINDEX_FORM_ITEMS,
        });
    }
    values
        .iter()
        .map(|value| {
            let CanonicalValueView::Unsigned {
                width: CanonicalIntegerWidth::Bits64,
                bits,
            } = value.view()
            else {
                return Err(malformed(ReindexAttributeSubject::FactorSequence));
            };
            Ok(Extent::new(bits))
        })
        .collect()
}

/// Builds the canonical attribute value of one validated form.
///
/// The kind field is written first and every form carries exactly the further
/// fields it uses. Writing an axis under one field ID and an axis *sequence*
/// under another is load-bearing rather than cosmetic: a single-axis form and a
/// one-element sequence form would otherwise encode identically, and a split of
/// axis 0 and a merge starting at axis 0 would differ only in a name.
fn encode_form(
    kind: ReindexFormKind,
    axes: &[Axis],
    factors: &[Extent],
) -> Result<CanonicalValue, TypeIdentityError> {
    let name = CanonicalValue::utf8(kind.canonical_name())?;
    let axis_field = |axis: Axis| {
        CanonicalField::new(REINDEX_FORM_AXIS, CanonicalValue::unsigned_u32(axis.get()))
    };
    match kind {
        ReindexFormKind::PermuteAxes | ReindexFormKind::MergeAxes => CanonicalValue::record([
            CanonicalField::new(REINDEX_FORM_KIND, name),
            CanonicalField::new(REINDEX_FORM_AXES, axis_sequence(axes)?),
        ]),
        ReindexFormKind::SplitAxis => CanonicalValue::record([
            CanonicalField::new(REINDEX_FORM_KIND, name),
            axis_field(axes[0]),
            CanonicalField::new(REINDEX_FORM_FACTORS, factor_sequence(factors)?),
        ]),
        ReindexFormKind::InsertUnitAxis
        | ReindexFormKind::RemoveUnitAxis
        | ReindexFormKind::ReverseAxis => CanonicalValue::record([
            CanonicalField::new(REINDEX_FORM_KIND, name),
            axis_field(axes[0]),
        ]),
    }
}

fn axis_sequence(axes: &[Axis]) -> Result<CanonicalValue, TypeIdentityError> {
    CanonicalValue::sequence(
        axes.iter()
            .map(|axis| CanonicalValue::unsigned_u32(axis.get())),
    )
}

fn factor_sequence(factors: &[Extent]) -> Result<CanonicalValue, TypeIdentityError> {
    CanonicalValue::sequence(
        factors
            .iter()
            .map(|factor| CanonicalValue::unsigned_u64(factor.get())),
    )
}

/// Registers the governed reindex family.
pub(super) fn register_standard_reindex(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        reindex_f32_op(),
        OperationSchema::new(
            OperationArity::exact(1),
            OperationArity::exact(1),
            [OperationAttributeSchema::required(
                REINDEX_MAPPING_ATTRIBUTE,
                CanonicalValueKind::Record,
            )],
        )
        .expect("the governed reindex schema is valid"),
        NormativeDefinitionRef::new(REINDEX_F32_NORMATIVE_DEFINITION)?,
        OperationDefinitionFacts::new(reindex_facts()),
        standard_conformance("reindex-f32"),
        OperationEffect::Pure,
        Arc::new(ReindexF32),
    ))
    // No algebraic capability is declared, deliberately. A reindex performs no
    // arithmetic, so it has no associativity or commutativity to declare, and a
    // missing declaration is unknown rather than the inverse law.
}

/// The complete normative definition of `tiler::reindex-f32@1`.
///
/// Held as a constant rather than written inline because it is the sentence
/// D-10's resolution lives in, and a reader looking for that answer should find
/// it under a name rather than inside a registration call.
const REINDEX_F32_NORMATIVE_DEFINITION: &str = concat!(
    "tiler::reindex-f32@1; a total output-to-input binary32 coordinate function that is a bijection ",
    "between the result domain and the operand domain, so every operand element is read exactly once ",
    "and no value is computed, converted, or rounded. ",
    "Admitted forms, and no others: permute-axes, a reordering of whole axes; split-axis, a row-major ",
    "factorization of one axis with the major factor first; merge-axes, the row-major product of a ",
    "strictly ascending adjacent axis run; insert-unit-axis and remove-unit-axis, over an extent-one ",
    "axis alone; and reverse-axis, the within-axis coordinate map i -> extent - 1 - i. ",
    "Decision D-10 is resolved by that last form: a within-axis coordinate permutation is admitted in ",
    "the reversal form and in no other, because the affine within-axis bijections of an axis are ",
    "exactly the identity and the reversal, while a general within-axis permutation is a ",
    "tensor-data-derived index the accepted index vocabulary rejects. A within-axis rotation is ",
    "expressible but deliberately unadmitted. ",
    "A form outside this set is refused by name at construction rather than approximated. ",
    "A non-surjective mapping is a slice, a different family, and is refused; a many-to-one mapping is ",
    "a broadcast, a different family, and is refused. ",
    "This operation makes no claim that storage was transposed, copied, or materialized: it states a ",
    "logical coordinate relation, and every physical realization of it remains a planning outcome.",
);

fn reindex_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            REINDEX_FACT_VALUE_BEHAVIOUR,
            fact("none-every-result-element-is-an-operand-element-unchanged"),
        ),
        CanonicalField::new(
            REINDEX_FACT_MAPPING_CLASS,
            fact("total-over-the-result-domain-and-bijective-onto-the-operand-domain"),
        ),
        CanonicalField::new(
            REINDEX_FACT_STORAGE_CLAIM,
            fact("none-no-transpose-copy-or-materialization-is-claimed"),
        ),
        CanonicalField::new(
            REINDEX_FACT_ADMITTED_FORMS,
            fact(
                "permute-axes,split-axis,merge-axes,insert-unit-axis,remove-unit-axis,reverse-axis",
            ),
        ),
    ])
    .expect("the governed reindex facts are canonical")
}

fn fact(value: &str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("a governed reindex fact is bounded")
}

struct ReindexF32;

impl OperationInferencer for ReindexF32 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "reindex.attributes",
                "a reindex requires exactly the mapping-form attribute".to_owned(),
            ));
        }
        let Some(value) = attributes.get(REINDEX_MAPPING_ATTRIBUTE) else {
            return Err(op_error(
                "reindex.attributes",
                "a reindex requires exactly the mapping-form attribute".to_owned(),
            ));
        };
        // The form's own rules are decided before anything about the occurrence,
        // so an unadmitted form is refused under its own name rather than under
        // whichever shape check happened to notice first.
        let form =
            ReindexForm::from_canonical_value(value).map_err(|error| form_rejection(&error))?;
        let [operand] = operands else {
            return Err(op_error(
                "reindex.operands",
                format!(
                    "a reindex takes one operand and {} were supplied",
                    operands.len()
                ),
            ));
        };
        if operand.resolved_type() != &F32::resolved_type() {
            return Err(op_error(
                "reindex.type",
                "a reindex operand must be f32".to_owned(),
            ));
        }
        // A reindex form splits, merges, or permutes extents, so it computes
        // over extent *values* rather than comparing them. `SourcedExtent` is
        // deliberately not an expression tree — a composed extent is a relation
        // in the environment — so a symbolic operand has no result boundary this
        // rule could state, and is declined by name.
        let shape = form
            .result_shape(request.static_operand_shape(0)?)
            .map_err(|error| form_rejection(&error))?;
        outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
    }
}

fn form_rejection(error: &ReindexFormError) -> OperationInferenceError {
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
