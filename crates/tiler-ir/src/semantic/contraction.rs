//! The governed tensor-contraction family and its canonical index structure.
//!
//! **Which "contraction" this is.** A *tensor* contraction sums over indices
//! shared by two operands; `matmul`, batched `matmul`, and general einsum are
//! instances. ADR 0015's contraction is a different concept entirely — the
//! numerical permission to fuse a separately rounded multiply and add into one
//! rounding — and this family declares that permission *forbidden*
//! ([`CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED`]). Reading the two
//! senses as one would conclude that a tensor contraction admits a fused
//! multiply-add by virtue of its name. It does not.
//!
//! **One key, one structure attribute.**
//! [ADR 0087](../../../../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md)
//! accepts one keyed family whose node carries its index structure — the
//! per-operand index tuples, the output tuple, and the contracted set — as a
//! strongly typed attribute participating in canonical identity. A frontend
//! never chooses among contraction keys, because there is only one; it states a
//! structure and this module's validator says yes or no.
//!
//! **What is admitted today.** One index structure, `td,od->to` over
//! `[M, K] x [N, K] -> [M, N]`, in `tiler::f32@1` throughout. That is index
//! structure 1 of the pinned language-model workload, 197 of its 253 contraction
//! occurrences, and its contracted index is the *last* axis of both operands
//! because the checkpoint stores every projection weight `[out, in]`. Nothing
//! here restricts the structure attribute to that one value: the five structural
//! rules admit every well-formed binary structure, and an unsupported one fails
//! closed later, at lowering-capability resolution, exactly as ADR 0087 item 4
//! requires. Registering this key gives a contraction a semantic identity and a
//! validated structure; it does not make one planable, schedulable, or
//! executable.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::identity::push_slice;
use crate::shape::{Extent, Shape};

use super::operation::CANONICAL_F32_ARITHMETIC_NAN_BITS;
use super::registry::standard_conformance;
use super::{
    AttributeFieldId, CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind,
    CanonicalValueView, F32, NormativeDefinitionRef, OpKey, OperationArity,
    OperationAttributeSchema, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, ProviderDiagnosticCode, RegistryError,
    SemanticRegistryRegistrar, TypeIdentityError, ValueFact, canonical_f32_bits,
};

/// Maximum operands one contraction index structure may name.
///
/// The same bound a canonical sequence admits, so the structure is refused with
/// a contraction-shaped diagnostic rather than an anonymous canonical-bound one.
/// It is not the family's operand arity: this profile's schema admits exactly
/// two operands, and a structure naming more is refused by
/// [`ContractionStructureError::OperandCountMismatch`] after its own rules run —
/// which is what keeps rule five reachable.
pub const MAX_CONTRACTION_OPERANDS: usize = super::types::MAX_RESOLVED_TYPE_ITEMS;
/// Maximum indices one operand or output tuple may name.
pub const MAX_CONTRACTION_TUPLE_INDICES: usize = super::types::MAX_RESOLVED_TYPE_ITEMS;

/// Stable field ID carrying the canonical index structure on the contraction.
pub const CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// Structure-record field carrying the ordered per-operand index tuples.
///
/// The three constants below are fields of the *index-structure record*, which
/// is a different record from the operation's fact record and from every other
/// record in this corpus; equal integers across records are unrelated, and
/// renumbering a published ID is a breaking identity change. The same rules
/// [`crate::semantic::SCALAR_TYPE_FACT_CLASS`] and its siblings carry apply
/// here, and are stated once there rather than restated per record.
pub const CONTRACTION_STRUCTURE_OPERAND_INDICES: AttributeFieldId = AttributeFieldId::new(1);
/// Structure-record field carrying the ordered output index tuple.
pub const CONTRACTION_STRUCTURE_OUTPUT_INDICES: AttributeFieldId = AttributeFieldId::new(2);
/// Structure-record field carrying the ascending contracted index set.
pub const CONTRACTION_STRUCTURE_CONTRACTED_INDICES: AttributeFieldId = AttributeFieldId::new(3);

/// Fact field naming the precision every operand and product is computed at.
///
/// The fourteen fields below are the contraction's numerical signature. ADR 0009
/// requires a contraction to expose computation/input precision, accumulator
/// dtype, result dtype, conversion behaviour, and an order contract rather than
/// only an operand dtype and a result dtype, and ADR 0087 item 5 requires the
/// signature to be stated once, generically, parameterized by the structure.
/// Every one is unconditional on this definition: absence is a malformed record,
/// never a default.
///
/// One value the profile's measurements record is deliberately *not* here. The
/// `FlushSubnormalsToZeroF32` realization is a property of the qualified
/// Apple9/F32 target row, not of the operation; putting it in a
/// consumer-neutral operation definition would make a target fact travel with
/// the semantics.
pub const CONTRACTION_F32_FACT_COMPUTATION_PRECISION: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming the type each accumulation step is performed at.
pub const CONTRACTION_F32_FACT_ACCUMULATOR_TYPE: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming the result value type.
pub const CONTRACTION_F32_FACT_RESULT_TYPE: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field naming the conversion behaviour between the three types above.
pub const CONTRACTION_F32_FACT_CONVERSION: AttributeFieldId = AttributeFieldId::new(4);
/// Fact field naming the order the reduction's contributors are folded in.
pub const CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE: AttributeFieldId = AttributeFieldId::new(5);
/// Fact field naming the accumulator's seed, or its absence.
pub const CONTRACTION_F32_FACT_SEED: AttributeFieldId = AttributeFieldId::new(6);
/// Fact field naming the behaviour on an empty contracted domain.
pub const CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN: AttributeFieldId = AttributeFieldId::new(7);
/// Fact field stating whether the reduction's contributors may be regrouped.
pub const CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED: AttributeFieldId = AttributeFieldId::new(8);
/// Fact field stating whether the reduction's contributors may be reordered.
pub const CONTRACTION_F32_FACT_PERMUTATION_PERMITTED: AttributeFieldId = AttributeFieldId::new(9);
/// Fact field naming this operation's distributivity dimension.
///
/// Not a Boolean, because absent and forbidden are different states: no
/// numerical contract Tiler can express grants distributivity at all, so a
/// `false` here would claim a permission exists and is withheld.
pub const CONTRACTION_F32_FACT_DISTRIBUTIVITY: AttributeFieldId = AttributeFieldId::new(10);
/// Fact field stating whether ADR 0015's multiply-add fusion is permitted.
pub const CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(11);
/// Fact field carrying the canonical arithmetic-NaN payload this family installs.
pub const CONTRACTION_F32_FACT_CANONICAL_NAN_BITS: AttributeFieldId = AttributeFieldId::new(12);
/// Fact field naming where the canonical NaN payload is installed.
pub const CONTRACTION_F32_FACT_NAN_CANONICALIZATION: AttributeFieldId = AttributeFieldId::new(13);
/// Fact field naming this family's determinism guarantee.
pub const CONTRACTION_F32_FACT_DETERMINISM: AttributeFieldId = AttributeFieldId::new(14);

/// Domain separator of a canonical contraction index-structure encoding.
const CONTRACTION_INDEX_STRUCTURE_DOMAIN: &[u8] = b"tiler.contraction-index-structure.v1\0";

/// Returns the governed strict binary32 tensor-contraction operation key.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn strict_tensor_contraction_f32_op() -> OpKey {
    OpKey::new("tiler", "strict-tensor-contraction-f32", 1)
        .expect("the governed tensor-contraction key is valid")
}

/// One index of a contraction's index structure.
///
/// A newtype rather than a bare `u32` for the reason the accepted shape
/// environment contract makes a correctness requirement: a contraction index, a
/// logical [`crate::shape::Axis`], an operand position, and an extent are four
/// domains whose representations happen to be primitive, and mixing them is a
/// defect the type system should catch. A contraction index is *not* an axis: it
/// names a coordinate of the iteration space, and which axis of which operand it
/// binds to is exactly what the structure states.
///
/// The numeric value carries no meaning outside one structure. Two structures'
/// index `0` are unrelated, and the canonical numbering is an artifact of
/// [`ContractionIndexStructure::new`] rather than something a frontend chooses.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContractionIndex(u32);

impl ContractionIndex {
    /// Creates a contraction index from an arbitrary frontend label.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the fixed-width label.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ContractionIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Which part of a malformed structure attribute was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ContractionAttributeSubject {
    /// The attribute was not the three-field structure record.
    StructureRecord,
    /// The operand-tuples field was not a sequence.
    OperandTuples,
    /// One operand tuple was not a sequence.
    OperandTuple,
    /// The output field was not a sequence.
    OutputTuple,
    /// The contracted-set field was not a sequence.
    ContractedSet,
    /// One index was not a canonical unsigned 32-bit value.
    Index,
}

impl fmt::Display for ContractionAttributeSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StructureRecord => formatter.write_str("structure record"),
            Self::OperandTuples => formatter.write_str("operand-tuple sequence"),
            Self::OperandTuple => formatter.write_str("operand tuple"),
            Self::OutputTuple => formatter.write_str("output tuple"),
            Self::ContractedSet => formatter.write_str("contracted set"),
            Self::Index => formatter.write_str("contraction index"),
        }
    }
}

/// A typed refusal of one contraction index structure.
///
/// The first five variants are ADR 0087's five structural admission rules, each
/// under its own name. A malformed structure is never a generic invalidity, and
/// never a value that reaches identity, planning, explain output, or a cache
/// subject: [`ContractionIndexStructure`] has no unchecked constructor, so
/// holding one is evidence that all five rules were decided.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ContractionStructureError {
    /// Rule one: an output index appears in no operand.
    OutputIndexInNoOperand {
        /// The output index no operand names.
        index: ContractionIndex,
    },
    /// Rule two: a summed index appears in only one operand.
    ///
    /// That is a reduction of one operand rather than a contraction; it has a
    /// different meaning, a different access relation, and — since the strict
    /// serial `Sum` is registered — a different governed key.
    SummedIndexInOneOperand {
        /// The index summed over.
        index: ContractionIndex,
        /// The sole operand position naming it.
        operand: usize,
    },
    /// Rule three: an index is repeated within one operand.
    IndexRepeatedWithinOperand {
        /// The repeated index.
        index: ContractionIndex,
        /// The operand position repeating it.
        operand: usize,
    },
    /// Rule four: an output index is duplicated.
    ///
    /// The rule's other half — that the output order is a permutation of the
    /// free indices — is the conjunction of this variant and
    /// [`Self::OutputIndexInNoOperand`], because the structure's free indices
    /// *are* its output set: an index in the operands and not in the output is
    /// contracted by definition, and one in neither does not exist. There is no
    /// third way for the output to fail to be a permutation, so a third variant
    /// would be a check that can never say no.
    DuplicateOutputIndex {
        /// The index the output names twice.
        index: ContractionIndex,
    },
    /// Rule five: an index appears in more than two operands.
    ///
    /// This is where the reserved multi-operand answer lands. Until it is
    /// decided the structure is refused rather than approximated, so a
    /// three-operand einsum is an explainable refusal and not a silently wrong
    /// association.
    IndexInMoreThanTwoOperands {
        /// The over-shared index.
        index: ContractionIndex,
        /// How many operands name it.
        operands: usize,
    },
    /// No index is summed, so the structure denotes no contraction.
    ///
    /// Not one of the five rules. An unsummed structure is an outer product or
    /// an elementwise product, which is a different family with a different
    /// access relation; the registered strict serial `Sum` refuses an empty axis
    /// set for the same reason.
    NoContractedIndex,
    /// The declared contracted set is not the set the structure derives.
    ContractedSetMismatch,
    /// The structure is not numbered by canonical first appearance.
    ///
    /// Refused rather than renumbered, following the reduced-axis precedent on
    /// `tiler::strict-serial-sum-f32@1`, which refuses axes that are not unique
    /// and strictly ascending instead of sorting them. The attribute is the
    /// identity, so admitting a second numbering of one structure would give one
    /// structure two identities — the collision ADR 0087 item 1 exists to
    /// prevent.
    NonCanonicalNumbering {
        /// The index the structure declared.
        declared: ContractionIndex,
        /// The index canonical first-appearance numbering assigns there.
        canonical: ContractionIndex,
    },
    /// The structure names an operand count the operation's signature does not.
    OperandCountMismatch {
        /// Operand tuples the structure names.
        structure: usize,
        /// Operands the occurrence supplies.
        signature: usize,
    },
    /// The attribute was not a well-formed structure record.
    MalformedAttribute {
        /// The rejected part.
        subject: ContractionAttributeSubject,
    },
    /// The structure named more operands than one canonical sequence admits.
    TooManyOperands {
        /// First rejected operand count.
        operands: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// One tuple named more indices than one canonical sequence admits.
    TooManyIndices {
        /// First rejected index count.
        indices: usize,
        /// Governed maximum.
        limit: usize,
    },
    /// The structure exceeded a canonical structural bound.
    CanonicalBound(TypeIdentityError),
}

impl ContractionStructureError {
    /// Returns the stable provider diagnostic code naming this refusal.
    ///
    /// Each of the five rules has its own code, so a caller reads *which* rule
    /// refused from the code rather than by matching on a message.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::OutputIndexInNoOperand { .. } => "contraction.rule.output-index-in-no-operand",
            Self::SummedIndexInOneOperand { .. } => "contraction.rule.summed-index-in-one-operand",
            Self::IndexRepeatedWithinOperand { .. } => {
                "contraction.rule.index-repeated-within-operand"
            }
            Self::DuplicateOutputIndex { .. } => "contraction.rule.duplicate-output-index",
            Self::IndexInMoreThanTwoOperands { .. } => {
                "contraction.rule.index-in-more-than-two-operands"
            }
            Self::NoContractedIndex => "contraction.structure.no-contracted-index",
            Self::ContractedSetMismatch => "contraction.structure.contracted-set-mismatch",
            Self::NonCanonicalNumbering { .. } => "contraction.structure.non-canonical-numbering",
            Self::OperandCountMismatch { .. } => "contraction.structure.operand-count",
            Self::MalformedAttribute { .. } => "contraction.structure.malformed-attribute",
            Self::TooManyOperands { .. } => "contraction.structure.too-many-operands",
            Self::TooManyIndices { .. } => "contraction.structure.too-many-indices",
            Self::CanonicalBound(_) => "contraction.structure.canonical-bound",
        }
    }
}

impl fmt::Display for ContractionStructureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutputIndexInNoOperand { index } => write!(
                formatter,
                "output index {index} appears in no operand tuple"
            ),
            Self::SummedIndexInOneOperand { index, operand } => write!(
                formatter,
                "summed index {index} appears only in operand {operand}, which is a reduction of that operand rather than a contraction"
            ),
            Self::IndexRepeatedWithinOperand { index, operand } => write!(
                formatter,
                "index {index} is repeated within operand {operand}"
            ),
            Self::DuplicateOutputIndex { index } => {
                write!(formatter, "output index {index} is duplicated")
            }
            Self::IndexInMoreThanTwoOperands { index, operands } => write!(
                formatter,
                "index {index} appears in {operands} operands, and no more than two are admitted"
            ),
            Self::NoContractedIndex => formatter.write_str(
                "the structure sums over no index, so it denotes an outer or elementwise product rather than a contraction",
            ),
            Self::ContractedSetMismatch => formatter.write_str(
                "the declared contracted set is not the set the operand and output tuples derive",
            ),
            Self::NonCanonicalNumbering {
                declared,
                canonical,
            } => write!(
                formatter,
                "index {declared} is not numbered by canonical first appearance, which assigns {canonical} there"
            ),
            Self::OperandCountMismatch {
                structure,
                signature,
            } => write!(
                formatter,
                "the structure names {structure} operand tuples and the occurrence supplies {signature} operands"
            ),
            Self::MalformedAttribute { subject } => {
                write!(formatter, "the {subject} is malformed")
            }
            Self::TooManyOperands { operands, limit } => write!(
                formatter,
                "the structure names {operands} operands, exceeding {limit}"
            ),
            Self::TooManyIndices { indices, limit } => write!(
                formatter,
                "a tuple names {indices} indices, exceeding {limit}"
            ),
            Self::CanonicalBound(source) => {
                write!(formatter, "the structure exceeds a canonical bound: {source}")
            }
        }
    }
}

impl Error for ContractionStructureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalBound(source) => Some(source),
            _ => None,
        }
    }
}

/// Collision-free canonical encoding of one contraction index structure.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalContractionIndexStructure(Vec<u8>);

impl CanonicalContractionIndexStructure {
    /// Returns the domain-separated canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A validated, renaming-invariant contraction index structure.
///
/// **Canonical numbering by first appearance.** Indices are renumbered in order
/// of first appearance across operand 0's tuple, then operand 1's, and finally
/// the output tuple, so two spellings of one structure produce identical bytes.
/// `ab,cb->ac`, `ij,kj->ik`, and `td,od->to` all canonicalize to operand 0
/// `(0, 1)`, operand 1 `(2, 1)`, output `(0, 2)`, contracted `{1}`. The ordinary
/// `[M, K] x [K, N]` matmul `td,do->to` canonicalizes to operand 1 `(1, 2)` and
/// is a *different* structure — which is the whole point of the pinned
/// workload's `[out_features, in_features]` weight layout.
///
/// **The contracted set is derived, not chosen.** An index that appears in the
/// operands and not in the output is summed over; every other operand index is
/// free and must be a member of the output exactly once. The attribute carries
/// the derived set because ADR 0087 names it as part of the structure, and a
/// declared set that disagrees with the derivation is refused rather than
/// preferred.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContractionIndexStructure {
    operands: Vec<Vec<ContractionIndex>>,
    output: Vec<ContractionIndex>,
    contracted: Vec<ContractionIndex>,
    canonical_value: CanonicalValue,
}

impl ContractionIndexStructure {
    /// Builds the canonical structure from arbitrary frontend index labels.
    ///
    /// The labels need not be canonical, dense, or ordered: this constructor
    /// renumbers by first appearance, which is what makes the result
    /// renaming-invariant. Both iterators are consumed under bounded work, so an
    /// unbounded one returns a typed error rather than running forever.
    ///
    /// # Errors
    ///
    /// Returns [`ContractionStructureError`] naming the violated structural rule,
    /// or the bound the structure exceeded. Rule five and
    /// [`ContractionStructureError::NoContractedIndex`] are decided here, before
    /// any operand count or extent is consulted.
    pub fn new(
        operands: impl IntoIterator<Item = impl IntoIterator<Item = ContractionIndex>>,
        output: impl IntoIterator<Item = ContractionIndex>,
    ) -> Result<Self, ContractionStructureError> {
        let mut collected: Vec<Vec<ContractionIndex>> = Vec::new();
        for tuple in operands
            .into_iter()
            .take(MAX_CONTRACTION_OPERANDS.saturating_add(1))
        {
            if collected.len() == MAX_CONTRACTION_OPERANDS {
                return Err(ContractionStructureError::TooManyOperands {
                    operands: MAX_CONTRACTION_OPERANDS.saturating_add(1),
                    limit: MAX_CONTRACTION_OPERANDS,
                });
            }
            collected.push(collect_tuple(tuple)?);
        }
        let output = collect_tuple(output)?;
        Self::finish(collected, output, None, Numbering::Canonicalize)
    }

    /// Decodes one structure attribute exactly as an occurrence carries it.
    ///
    /// # Errors
    ///
    /// Returns [`ContractionStructureError`] for a malformed attribute, a
    /// violated structural rule, a contracted set that disagrees with the
    /// derivation, or a numbering that is not canonical first appearance. The
    /// five rules are decided before the numbering, so a malformed structure is
    /// reported under its rule rather than under its spelling.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, ContractionStructureError> {
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(malformed(ContractionAttributeSubject::StructureRecord));
        };
        let [operands_field, output_field, contracted_field] = fields else {
            return Err(malformed(ContractionAttributeSubject::StructureRecord));
        };
        if operands_field.id() != CONTRACTION_STRUCTURE_OPERAND_INDICES
            || output_field.id() != CONTRACTION_STRUCTURE_OUTPUT_INDICES
            || contracted_field.id() != CONTRACTION_STRUCTURE_CONTRACTED_INDICES
        {
            return Err(malformed(ContractionAttributeSubject::StructureRecord));
        }
        let CanonicalValueView::Sequence(tuples) = operands_field.value().view() else {
            return Err(malformed(ContractionAttributeSubject::OperandTuples));
        };
        if tuples.len() > MAX_CONTRACTION_OPERANDS {
            return Err(ContractionStructureError::TooManyOperands {
                operands: tuples.len(),
                limit: MAX_CONTRACTION_OPERANDS,
            });
        }
        let mut operands = Vec::with_capacity(tuples.len());
        for tuple in tuples {
            operands.push(decode_tuple(
                tuple,
                ContractionAttributeSubject::OperandTuple,
            )?);
        }
        let output = decode_tuple(
            output_field.value(),
            ContractionAttributeSubject::OutputTuple,
        )?;
        let contracted = decode_tuple(
            contracted_field.value(),
            ContractionAttributeSubject::ContractedSet,
        )?;
        Self::finish(operands, output, Some(contracted), Numbering::Require)
    }

    /// Validates already-collected tuples and builds the canonical value.
    ///
    /// `declared_contracted` is `Some` only on the decode path, where a caller
    /// supplied a set that must agree with the derivation rather than replace it.
    fn finish(
        operands: Vec<Vec<ContractionIndex>>,
        output: Vec<ContractionIndex>,
        declared_contracted: Option<Vec<ContractionIndex>>,
        numbering: Numbering,
    ) -> Result<Self, ContractionStructureError> {
        // The five rules first, and before anything about the numbering: they
        // are renaming-invariant, so they are decidable on any spelling, and a
        // structure that violates one must be reported under that rule rather
        // than under how it happened to be spelled. Deciding them on the
        // *supplied* labels is also what lets a diagnostic name the index a
        // frontend wrote rather than the number canonicalization gave it.
        let contracted = derive_contracted(&operands, &output)?;
        if let Some(declared) = declared_contracted
            && declared != contracted
        {
            return Err(ContractionStructureError::ContractedSetMismatch);
        }
        let renumbered = canonical_numbering(&operands, &output);
        let (operands, output, contracted) = match numbering {
            Numbering::Require => {
                if let Some((declared, canonical)) =
                    first_difference(&operands, &renumbered.operands)
                        .or_else(|| first_difference_in_tuple(&output, &renumbered.output))
                {
                    return Err(ContractionStructureError::NonCanonicalNumbering {
                        declared,
                        canonical,
                    });
                }
                (operands, output, contracted)
            }
            Numbering::Canonicalize => {
                let mut contracted: Vec<ContractionIndex> = contracted
                    .iter()
                    .map(|index| renumbered.assignment[index])
                    .collect();
                contracted.sort_unstable();
                (renumbered.operands, renumbered.output, contracted)
            }
        };
        let canonical_value = encode_structure(&operands, &output, &contracted)
            .map_err(ContractionStructureError::CanonicalBound)?;
        Ok(Self {
            operands,
            output,
            contracted,
            canonical_value,
        })
    }

    /// Returns the number of operand tuples this structure names.
    #[must_use]
    pub fn operand_count(&self) -> usize {
        self.operands.len()
    }

    /// Returns the ordered per-operand index tuples.
    #[must_use]
    pub fn operands(&self) -> impl ExactSizeIterator<Item = &[ContractionIndex]> {
        self.operands.iter().map(Vec::as_slice)
    }

    /// Returns one operand's index tuple, when the position exists.
    #[must_use]
    pub fn operand(&self, position: usize) -> Option<&[ContractionIndex]> {
        self.operands.get(position).map(Vec::as_slice)
    }

    /// Returns the ordered output index tuple.
    #[must_use]
    pub fn output(&self) -> &[ContractionIndex] {
        &self.output
    }

    /// Returns the derived contracted index set, in ascending order.
    #[must_use]
    pub fn contracted(&self) -> &[ContractionIndex] {
        &self.contracted
    }

    /// Returns the canonical attribute value an occurrence carries.
    #[must_use]
    pub const fn canonical_value(&self) -> &CanonicalValue {
        &self.canonical_value
    }

    /// Returns the domain-separated canonical encoding of this structure.
    ///
    /// Derived from [`Self::canonical_value`] rather than from a second walk of
    /// the tuples, so the identity a reader compares and the attribute an
    /// occurrence carries cannot disagree about what a structure is.
    #[must_use]
    pub fn canonical_encoding(&self) -> CanonicalContractionIndexStructure {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, CONTRACTION_INDEX_STRUCTURE_DOMAIN);
        self.canonical_value.encode(&mut bytes);
        CanonicalContractionIndexStructure(bytes)
    }
}

fn malformed(subject: ContractionAttributeSubject) -> ContractionStructureError {
    ContractionStructureError::MalformedAttribute { subject }
}

fn collect_tuple(
    indices: impl IntoIterator<Item = ContractionIndex>,
) -> Result<Vec<ContractionIndex>, ContractionStructureError> {
    let mut collected = Vec::new();
    for index in indices
        .into_iter()
        .take(MAX_CONTRACTION_TUPLE_INDICES.saturating_add(1))
    {
        if collected.len() == MAX_CONTRACTION_TUPLE_INDICES {
            return Err(ContractionStructureError::TooManyIndices {
                indices: MAX_CONTRACTION_TUPLE_INDICES.saturating_add(1),
                limit: MAX_CONTRACTION_TUPLE_INDICES,
            });
        }
        collected.push(index);
    }
    Ok(collected)
}

fn decode_tuple(
    value: &CanonicalValue,
    subject: ContractionAttributeSubject,
) -> Result<Vec<ContractionIndex>, ContractionStructureError> {
    let CanonicalValueView::Sequence(values) = value.view() else {
        return Err(malformed(subject));
    };
    if values.len() > MAX_CONTRACTION_TUPLE_INDICES {
        return Err(ContractionStructureError::TooManyIndices {
            indices: values.len(),
            limit: MAX_CONTRACTION_TUPLE_INDICES,
        });
    }
    let mut collected = Vec::with_capacity(values.len());
    for value in values {
        let CanonicalValueView::Unsigned {
            width: CanonicalIntegerWidth::Bits32,
            bits,
        } = value.view()
        else {
            return Err(malformed(ContractionAttributeSubject::Index));
        };
        let label =
            u32::try_from(bits).map_err(|_| malformed(ContractionAttributeSubject::Index))?;
        collected.push(ContractionIndex::new(label));
    }
    Ok(collected)
}

/// Whether a structure's supplied numbering is required or established here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Numbering {
    /// The supplied numbering must already be canonical first appearance.
    Require,
    /// Any spelling is admitted and renumbered by canonical first appearance.
    Canonicalize,
}

/// One structure renumbered by canonical first appearance.
struct CanonicalNumbering {
    operands: Vec<Vec<ContractionIndex>>,
    output: Vec<ContractionIndex>,
    assignment: BTreeMap<ContractionIndex, ContractionIndex>,
}

/// Renumbers a structure by canonical first appearance.
///
/// The scan order — every operand tuple in order, then the output tuple — is the
/// canonicalization ADR 0087 item 1 fixes, and it is the whole of the
/// renaming-invariance property. Changing it changes every contraction identity.
fn canonical_numbering(
    operands: &[Vec<ContractionIndex>],
    output: &[ContractionIndex],
) -> CanonicalNumbering {
    let mut assignment: BTreeMap<ContractionIndex, ContractionIndex> = BTreeMap::new();
    let renumber = |index: ContractionIndex,
                    assignment: &mut BTreeMap<ContractionIndex, ContractionIndex>|
     -> ContractionIndex {
        let next = u32::try_from(assignment.len()).unwrap_or(u32::MAX);
        *assignment
            .entry(index)
            .or_insert_with(|| ContractionIndex::new(next))
    };
    let mut renumbered_operands = Vec::with_capacity(operands.len());
    for tuple in operands {
        let mut renumbered = Vec::with_capacity(tuple.len());
        for index in tuple {
            renumbered.push(renumber(*index, &mut assignment));
        }
        renumbered_operands.push(renumbered);
    }
    let mut renumbered_output = Vec::with_capacity(output.len());
    for index in output {
        renumbered_output.push(renumber(*index, &mut assignment));
    }
    CanonicalNumbering {
        operands: renumbered_operands,
        output: renumbered_output,
        assignment,
    }
}

fn first_difference(
    declared: &[Vec<ContractionIndex>],
    canonical: &[Vec<ContractionIndex>],
) -> Option<(ContractionIndex, ContractionIndex)> {
    declared
        .iter()
        .zip(canonical)
        .find_map(|(declared, canonical)| first_difference_in_tuple(declared, canonical))
}

fn first_difference_in_tuple(
    declared: &[ContractionIndex],
    canonical: &[ContractionIndex],
) -> Option<(ContractionIndex, ContractionIndex)> {
    declared
        .iter()
        .zip(canonical)
        .find(|(declared, canonical)| declared != canonical)
        .map(|(declared, canonical)| (*declared, *canonical))
}

/// Decides ADR 0087's five structural rules and derives the contracted set.
///
/// Every check below can say no, and each says no under its own name. The order
/// is deterministic so that a structure violating one rule always reports the
/// same one; a structure violating several reports the first in this order.
fn derive_contracted(
    operands: &[Vec<ContractionIndex>],
    output: &[ContractionIndex],
) -> Result<Vec<ContractionIndex>, ContractionStructureError> {
    // Rule three, first, because it is what makes "how many operands name this
    // index" equal to "how many occurrences does it have".
    for (position, tuple) in operands.iter().enumerate() {
        let mut seen = BTreeSet::new();
        for index in tuple {
            if !seen.insert(*index) {
                return Err(ContractionStructureError::IndexRepeatedWithinOperand {
                    index: *index,
                    operand: position,
                });
            }
        }
    }

    let mut occurrences: BTreeMap<ContractionIndex, Vec<usize>> = BTreeMap::new();
    for (position, tuple) in operands.iter().enumerate() {
        for index in tuple {
            occurrences.entry(*index).or_default().push(position);
        }
    }

    // Rule five.
    if let Some((index, positions)) = occurrences
        .iter()
        .find(|(_, positions)| positions.len() > 2)
    {
        return Err(ContractionStructureError::IndexInMoreThanTwoOperands {
            index: *index,
            operands: positions.len(),
        });
    }

    // Rule four.
    let mut free = BTreeSet::new();
    for index in output {
        if !free.insert(*index) {
            return Err(ContractionStructureError::DuplicateOutputIndex { index: *index });
        }
    }

    // Rule one.
    if let Some(index) = output.iter().find(|index| !occurrences.contains_key(index)) {
        return Err(ContractionStructureError::OutputIndexInNoOperand { index: *index });
    }

    // Rule two, over exactly the summed indices: an operand index outside the
    // output is contracted by definition, so this is the complete summed set.
    let mut contracted = Vec::new();
    for (index, positions) in &occurrences {
        if free.contains(index) {
            continue;
        }
        if positions.len() < 2 {
            return Err(ContractionStructureError::SummedIndexInOneOperand {
                index: *index,
                operand: positions[0],
            });
        }
        contracted.push(*index);
    }
    if contracted.is_empty() {
        return Err(ContractionStructureError::NoContractedIndex);
    }
    Ok(contracted)
}

/// Builds the canonical attribute value of one validated structure.
///
/// The operand tuples are a sequence *of sequences* rather than one flat
/// sequence, and that framing is load-bearing rather than cosmetic: flattening
/// it makes `ab,cb->ac` and `abc,b->ac` — two structurally different admitted
/// contractions — encode to identical bytes, which the mutation proof in this
/// module's tests demonstrates.
fn encode_structure(
    operands: &[Vec<ContractionIndex>],
    output: &[ContractionIndex],
    contracted: &[ContractionIndex],
) -> Result<CanonicalValue, TypeIdentityError> {
    let mut tuples = Vec::with_capacity(operands.len());
    for tuple in operands {
        tuples.push(index_sequence(tuple)?);
    }
    CanonicalValue::record([
        CanonicalField::new(
            CONTRACTION_STRUCTURE_OPERAND_INDICES,
            CanonicalValue::sequence(tuples)?,
        ),
        CanonicalField::new(
            CONTRACTION_STRUCTURE_OUTPUT_INDICES,
            index_sequence(output)?,
        ),
        CanonicalField::new(
            CONTRACTION_STRUCTURE_CONTRACTED_INDICES,
            index_sequence(contracted)?,
        ),
    ])
}

fn index_sequence(indices: &[ContractionIndex]) -> Result<CanonicalValue, TypeIdentityError> {
    CanonicalValue::sequence(
        indices
            .iter()
            .map(|index| CanonicalValue::unsigned_u32(index.get())),
    )
}

/// Registers the governed tensor-contraction family.
pub(super) fn register_standard_contraction(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        strict_tensor_contraction_f32_op(),
        OperationSchema::new(
            OperationArity::exact(2),
            OperationArity::exact(1),
            [OperationAttributeSchema::required(
                CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
                CanonicalValueKind::Record,
            )],
        )
        .expect("the governed contraction schema is valid"),
        NormativeDefinitionRef::new(
            "tiler::strict-tensor-contraction-f32@1; binary32 products folded strictly in ascending lexicographic order over the canonically ordered contracted index space, unseeded",
        )?,
        OperationDefinitionFacts::new(contraction_f32_facts()),
        standard_conformance("strict-tensor-contraction-f32"),
        OperationEffect::Pure,
        Arc::new(StrictTensorContractionF32),
    ))
    // No algebraic capability is declared, deliberately. A missing declaration
    // is unknown rather than the inverse law, and this family's reduction is a
    // strict fold whose contributors may not be regrouped: declaring ordered
    // associativity here would hand a rewrite the numerical facts below forbid.
}

fn contraction_f32_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(
            CONTRACTION_F32_FACT_COMPUTATION_PRECISION,
            fact("binary32-operands-and-binary32-products"),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_ACCUMULATOR_TYPE,
            CanonicalValue::value_type(F32::resolved_type()),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_RESULT_TYPE,
            CanonicalValue::value_type(F32::resolved_type()),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_CONVERSION,
            fact("none-operands-products-accumulator-and-result-are-binary32"),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE,
            fact("ascending-lexicographic-over-the-canonically-ordered-contracted-index-space"),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_SEED,
            fact("none-the-accumulator-starts-at-the-first-product"),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN,
            fact("refused-an-unseeded-fold-has-no-empty-result"),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_PERMUTATION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_DISTRIBUTIVITY,
            fact("absent-no-expressible-numerical-permission-grants-it"),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_CANONICAL_NAN_BITS,
            canonical_f32_bits(CANONICAL_F32_ARITHMETIC_NAN_BITS),
        ),
        CanonicalField::new(
            CONTRACTION_F32_FACT_NAN_CANONICALIZATION,
            fact("after-every-combine-and-at-the-result-boundary"),
        ),
        CanonicalField::new(CONTRACTION_F32_FACT_DETERMINISM, fact("plan-deterministic")),
    ])
    .expect("the governed contraction facts are canonical")
}

fn fact(value: &str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("a governed contraction fact is bounded")
}

struct StrictTensorContractionF32;

impl OperationInferencer for StrictTensorContractionF32 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        if attributes.fields().len() != 1 {
            return Err(op_error(
                "contraction.attributes",
                "a contraction requires exactly the index-structure attribute".to_owned(),
            ));
        }
        let Some(value) = attributes.get(CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE) else {
            return Err(op_error(
                "contraction.attributes",
                "a contraction requires exactly the index-structure attribute".to_owned(),
            ));
        };
        // The structure's own rules are decided before anything about the
        // occurrence, so a malformed structure is refused under its rule rather
        // than under whichever signature check happened to notice first. Rule
        // five in particular is only reachable this way: a structure naming
        // three operands never reaches the operand-count check below.
        let structure = ContractionIndexStructure::from_canonical_value(value)
            .map_err(|error| structure_rejection(&error))?;
        if structure.operand_count() != operands.len() {
            return Err(structure_rejection(
                &ContractionStructureError::OperandCountMismatch {
                    structure: structure.operand_count(),
                    signature: operands.len(),
                },
            ));
        }
        if operands
            .iter()
            .any(|operand| operand.resolved_type() != &F32::resolved_type())
        {
            return Err(op_error(
                "contraction.type",
                "every contraction operand must be f32".to_owned(),
            ));
        }

        // Extent agreement, through the accepted three-outcome path. Every
        // extent an occurrence can carry is a static `Extent`, so the outcome is
        // proved or disproved here and the unresolved arm — a typed host-side
        // pre-dispatch requirement — is unreachable until a semantic value fact
        // can carry a symbolic extent. A disproof names both observed sources,
        // because equality does not erase source identity: reporting one of them
        // would tell a caller half of what it needs to fix the program.
        //
        // Zipped rather than indexed by position: the operand-count refusal
        // above is what puts an index in range, and a check whose absence turns
        // a refusal into a panic is a worse check than one that cannot.
        let mut bindings: BTreeMap<ContractionIndex, ExtentBinding> = BTreeMap::new();
        for (position, (tuple, operand)) in structure.operands().zip(operands).enumerate() {
            let shape = operand.shape();
            if shape.rank() != tuple.len() {
                return Err(op_error(
                    "contraction.rank",
                    format!(
                        "operand {position} has rank {} and its index tuple names {} indices",
                        shape.rank(),
                        tuple.len()
                    ),
                ));
            }
            for (axis, index) in tuple.iter().enumerate() {
                let extent = shape.extents()[axis];
                match bindings.get(index) {
                    None => {
                        bindings.insert(
                            *index,
                            ExtentBinding {
                                operand: position,
                                axis,
                                extent,
                            },
                        );
                    }
                    Some(first) if first.extent != extent => {
                        return Err(op_error(
                            "contraction.extent.disproved",
                            format!(
                                "contraction index {index} is bound to extent {} by operand {} axis {} and to extent {} by operand {position} axis {axis}",
                                first.extent.get(),
                                first.operand,
                                first.axis,
                                extent.get()
                            ),
                        ));
                    }
                    Some(_) => {}
                }
            }
        }

        for index in structure.contracted() {
            let binding = bindings
                .get(index)
                .expect("a contracted index appears in an operand tuple");
            if binding.extent.get() == 0 {
                return Err(op_error(
                    "contraction.extent.empty-contracted-domain",
                    format!(
                        "contracted index {index} has extent zero at operand {} axis {}, and an unseeded strict fold has no empty result",
                        binding.operand, binding.axis
                    ),
                ));
            }
        }

        let extents = structure.output().iter().map(|index| {
            bindings
                .get(index)
                .expect("an output index appears in an operand tuple")
                .extent
        });
        let shape = Shape::try_new(extents)
            .map_err(|error| op_error("contraction.result-shape", error.to_string()))?;
        outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
    }
}

/// One index's extent and the exact operand axis that observed it.
#[derive(Clone, Copy, Debug)]
struct ExtentBinding {
    operand: usize,
    axis: usize,
    extent: Extent,
}

fn structure_rejection(error: &ContractionStructureError) -> OperationInferenceError {
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
