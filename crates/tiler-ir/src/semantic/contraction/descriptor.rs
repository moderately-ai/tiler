//! Typed reduction descriptor of `tiler::tensor-contraction-f32@1`.
//!
//! The successor contraction's order contract is operation-owned definition
//! content, not prose: outer fact field 15 is a six-row canonical record whose
//! rows name the leaf primitive, the reducer primitive, the result-class rule,
//! and the three order-freedom maxima, and outer field 14 binds the ADR 0013
//! plan-determinism scope as a seven-row canonical record. This module is the
//! **sole decoder** of that governed record — a second compiler or reference
//! decoder is forbidden by the accepted contract, because two decoders are two
//! places for the same fact to be read differently.
//!
//! The decoder validates every surviving outer fact, not only field 15: a
//! governed definition whose computation precision, accumulator, conversion,
//! contributor sequence, seed, empty-domain rule, distributivity, arithmetic
//! contraction, canonical NaN payload, canonicalization site, or stability
//! record deviates from the accepted contract refuses under a typed error
//! rather than registering. Standard registration runs this decoder and maps a
//! failure to [`RegistryError::InvalidGovernedContractionDescriptor`], so an
//! untyped governed contraction definition never registers.
//!
//! The descriptor names arithmetic meaning by closed vocabulary. It does not
//! name `tiler::add-f32@1` or `tiler.scalar::add-f32@1`, and it does not choose
//! the algebraic-capability owner for any other family: the accepted
//! algebraic-authority decision (2026-08-18) makes the order-freedom maxima
//! *themselves* ADR 0014's operation-declared algebraic fact for this
//! operation's internal fold, joined to the caller's independently resolved
//! numerical ceiling by [`ContractionF32ReductionDescriptor::resolve`].
//!
//! [`RegistryError::InvalidGovernedContractionDescriptor`]: crate::semantic::RegistryError::InvalidGovernedContractionDescriptor

use std::error::Error;
use std::fmt;

use crate::semantic::{
    AttributeFieldId, CanonicalValue, CanonicalValueKind, CanonicalValueView, F32,
    FrozenSemanticRegistry, OpKey, OperationDefinition,
};

use super::{
    CONTRACTION_F32_FACT_ACCUMULATOR_TYPE, CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
    CONTRACTION_F32_FACT_CANONICAL_NAN_BITS, CONTRACTION_F32_FACT_COMPUTATION_PRECISION,
    CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE, CONTRACTION_F32_FACT_CONVERSION,
    CONTRACTION_F32_FACT_DETERMINISM, CONTRACTION_F32_FACT_DISTRIBUTIVITY,
    CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN, CONTRACTION_F32_FACT_NAN_CANONICALIZATION,
    CONTRACTION_F32_FACT_REDUCTION_DESCRIPTOR, CONTRACTION_F32_FACT_RESULT_TYPE,
    CONTRACTION_F32_FACT_SEED, tensor_contraction_f32_op,
};

/// Reduction-record row naming the leaf primitive.
///
/// The six constants below are fields of the *reduction record* carried at
/// outer field 15, and the seven after them are fields of the *stability
/// record* carried at outer field 14. Both are schema-local ID spaces: equal
/// integers across records are unrelated, and renumbering a published ID is a
/// breaking identity change. They are private because the records are read only
/// through this module's decoder; the decoded enums are the public vocabulary.
const REDUCTION_FIELD_LEAF: AttributeFieldId = AttributeFieldId::new(1);
/// Reduction-record row naming the reducer primitive.
const REDUCTION_FIELD_REDUCER: AttributeFieldId = AttributeFieldId::new(2);
/// Reduction-record row naming the result-class rule.
const REDUCTION_FIELD_RESULT_CLASS: AttributeFieldId = AttributeFieldId::new(3);
/// Reduction-record row naming the operation's maximum reassociation freedom.
const REDUCTION_FIELD_MAX_REASSOCIATION: AttributeFieldId = AttributeFieldId::new(4);
/// Reduction-record row naming the operation's maximum permutation freedom.
const REDUCTION_FIELD_MAX_PERMUTATION: AttributeFieldId = AttributeFieldId::new(5);
/// Reduction-record row naming the maximum signed-zero-elimination freedom.
const REDUCTION_FIELD_MAX_SIGNED_ZERO: AttributeFieldId = AttributeFieldId::new(6);

/// Stability-record row naming the scope.
const STABILITY_FIELD_SCOPE: AttributeFieldId = AttributeFieldId::new(1);
/// Stability-record row naming the equal-inputs clause.
const STABILITY_FIELD_EQUAL_INPUTS: AttributeFieldId = AttributeFieldId::new(2);
/// Stability-record row naming the artifact clause.
const STABILITY_FIELD_ARTIFACT: AttributeFieldId = AttributeFieldId::new(3);
/// Stability-record row naming the selected-plan clause.
const STABILITY_FIELD_PLAN: AttributeFieldId = AttributeFieldId::new(4);
/// Stability-record row naming the target-environment clause.
const STABILITY_FIELD_ENVIRONMENT: AttributeFieldId = AttributeFieldId::new(5);
/// Stability-record row naming the result clause.
const STABILITY_FIELD_RESULT: AttributeFieldId = AttributeFieldId::new(6);
/// Stability-record row naming the recompilation boundary.
const STABILITY_FIELD_RECOMPILATION: AttributeFieldId = AttributeFieldId::new(7);

/// The complete outer field-id population of the governed fact record.
const EXPECTED_OUTER: [AttributeFieldId; 13] = [
    CONTRACTION_F32_FACT_COMPUTATION_PRECISION,
    CONTRACTION_F32_FACT_ACCUMULATOR_TYPE,
    CONTRACTION_F32_FACT_RESULT_TYPE,
    CONTRACTION_F32_FACT_CONVERSION,
    CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE,
    CONTRACTION_F32_FACT_SEED,
    CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN,
    CONTRACTION_F32_FACT_DISTRIBUTIVITY,
    CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
    CONTRACTION_F32_FACT_CANONICAL_NAN_BITS,
    CONTRACTION_F32_FACT_NAN_CANONICALIZATION,
    CONTRACTION_F32_FACT_DETERMINISM,
    CONTRACTION_F32_FACT_REDUCTION_DESCRIPTOR,
];

/// The order every contraction folds its contributors in.
///
/// Exhaustive, and deliberately single-valued today: the canonical ascending
/// lexicographic order over the contracted index space is the only contributor
/// sequence any governed contraction states. A second sequence is a different
/// semantic population and must widen this enum, which breaks every exhaustive
/// consumer instead of silently reinterpreting the old one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32ContributorSequence {
    /// Ascending lexicographic order over the canonically ordered contracted
    /// index space.
    AscendingLexicographicCanonicalContractedIndexSpace,
}

/// The exact arithmetic of one leaf product.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32LeafPrimitive {
    /// Apply the resolved input transform to each factor, multiply with one
    /// binary32 round-to-nearest-ties-to-even rounding, canonicalize an
    /// arithmetic NaN, then apply the resolved result transform.
    TransformOperandsRoundBinary32NearestTiesEvenMultiplyCanonicalizeNanTransformResult,
}

/// The exact arithmetic of one internal combine.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32ReducerPrimitive {
    /// Apply the resolved input transform to each addend, add with one binary32
    /// round-to-nearest-ties-to-even rounding, canonicalize an arithmetic NaN,
    /// then apply the resolved result transform.
    TransformOperandsRoundBinary32NearestTiesEvenAddCanonicalizeNanTransformResult,
}

/// The accumulator's seed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32Seed {
    /// The fold is unseeded: the accumulator starts at the first product.
    FirstProduct,
}

/// The declared behaviour on an empty contracted domain.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32EmptyDomain {
    /// Refused: an unseeded fold has no empty result.
    Refused,
}

/// One order freedom's operation-declared maximum.
///
/// This is ADR 0014's *algebraic* fact for the contraction's internal fold —
/// operation-declared, identity-encoded, and immutable under a request. It is
/// deliberately not a permission: a caller's numerical ceiling is the second,
/// independently resolved fact, and [`ContractionF32ReductionDescriptor::resolve`]
/// is the only join. Neither fact can substitute for the other.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32OrderFreedom {
    /// The operation does not support the freedom; no ceiling can grant it.
    Unsupported,
    /// The operation supports the freedom exactly when the caller's resolved
    /// numerical ceiling independently permits it.
    PermissionGated,
}

/// The result class one effective order contract denotes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32ResultClass {
    /// The single strict left-fold value over the canonical contributor
    /// sequence — bit-identical to the retired strict key's answer.
    StrictLeftFold,
    /// The set of results of all full ordered binary trees whose in-order leaf
    /// traversal is exactly the canonical contributor sequence.
    OrderedFullBinaryTrees,
}

/// Where the canonical arithmetic-NaN payload is installed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32NanCanonicalization {
    /// After each arithmetic operation and again at the result boundary.
    AfterEachArithmeticOperationAndResultBoundary,
}

/// The determinism stability scope the definition binds.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32StabilityScope {
    /// ADR 0013 plan determinism: identical input bits and runtime bindings,
    /// the same artifact digest and selected plan variant, and the same
    /// declared target environment produce identical output bits; a different
    /// artifact may select a different legal result.
    PlanDeterministic,
}

/// Which record a refused descriptor field belongs to.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32DescriptorField {
    /// A field of the outer thirteen-field fact record.
    Outer(AttributeFieldId),
    /// A row of the field-15 reduction record.
    Reduction(AttributeFieldId),
    /// A row of the field-14 stability record.
    Stability(AttributeFieldId),
}

impl fmt::Display for ContractionF32DescriptorField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Outer(field) => write!(formatter, "outer fact field {field}"),
            Self::Reduction(field) => write!(formatter, "reduction-record row {field}"),
            Self::Stability(field) => write!(formatter, "stability-record row {field}"),
        }
    }
}

/// A typed refusal of one governed contraction definition.
///
/// Exhaustive over the accepted vocabulary. Precedence is deterministic:
/// [`Self::WrongOperation`] before any fact is read; [`Self::MalformedFacts`]
/// before any field; [`Self::FactCount`] gates the outer record's arity, after
/// which an id outside the expected set is [`Self::UnexpectedField`] (an outer
/// record of the right arity with a wrong id always has one). Inside the two
/// nested records no arity gate runs, so a missing row is
/// [`Self::MissingField`] and a foreign row is [`Self::UnexpectedField`], in
/// record order. Value checks run per field after shape checks, and the one
/// admitted cross-field contradiction is checked last.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractionF32DescriptorError {
    /// The registry does not register the governed contraction operation.
    OperationMissing {
        /// The absent operation key.
        operation: OpKey,
    },
    /// The offered definition is not the governed contraction operation.
    WrongOperation {
        /// The governed key this decoder reads.
        expected: OpKey,
        /// The offered definition's key.
        actual: OpKey,
    },
    /// The definition's fact value is not a record.
    MalformedFacts {
        /// The offered value's kind.
        actual: CanonicalValueKind,
    },
    /// The outer fact record does not have exactly thirteen fields.
    FactCount {
        /// The governed outer field count.
        expected: usize,
        /// The offered field count.
        actual: usize,
    },
    /// A required field or row is absent.
    MissingField {
        /// The absent field.
        field: ContractionF32DescriptorField,
    },
    /// A field or row outside the governed schema is present.
    UnexpectedField {
        /// The foreign field.
        field: ContractionF32DescriptorField,
    },
    /// A field or row carries a value of the wrong canonical kind.
    WrongKind {
        /// The mis-kinded field.
        field: ContractionF32DescriptorField,
        /// The kind the governed schema requires.
        expected: CanonicalValueKind,
        /// The kind the value has.
        actual: CanonicalValueKind,
    },
    /// A field or row carries a well-kinded value outside the governed
    /// vocabulary.
    UnsupportedValue {
        /// The refused field.
        field: ContractionF32DescriptorField,
    },
    /// Two individually admitted values contradict each other.
    ///
    /// Reachable in exactly one place today: the result-class rule (reduction
    /// row 3) switches on effective reassociation, so a reassociation maximum
    /// of `unsupported` (reduction row 4) contradicts it — the definition would
    /// simultaneously state a two-cell result class and forbid the freedom that
    /// selects between the cells.
    ContradictoryFields {
        /// The first contradicting field, in record order.
        first: ContractionF32DescriptorField,
        /// The second contradicting field.
        second: ContractionF32DescriptorField,
    },
}

impl fmt::Display for ContractionF32DescriptorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperationMissing { operation } => {
                write!(formatter, "operation {operation} is not registered")
            }
            Self::WrongOperation { expected, actual } => write!(
                formatter,
                "definition is for {actual}, not the governed contraction {expected}"
            ),
            Self::MalformedFacts { actual } => write!(
                formatter,
                "the definition's facts are {actual:?}, not a record"
            ),
            Self::FactCount { expected, actual } => write!(
                formatter,
                "the outer fact record has {actual} fields, not the governed {expected}"
            ),
            Self::MissingField { field } => write!(formatter, "{field} is absent"),
            Self::UnexpectedField { field } => {
                write!(formatter, "{field} is outside the governed schema")
            }
            Self::WrongKind {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "{field} carries a {actual:?} value where the governed schema requires {expected:?}"
            ),
            Self::UnsupportedValue { field } => write!(
                formatter,
                "{field} carries a value outside the governed vocabulary"
            ),
            Self::ContradictoryFields { first, second } => {
                write!(formatter, "{first} and {second} contradict each other")
            }
        }
    }
}

impl Error for ContractionF32DescriptorError {}

/// The decoded, validated reduction descriptor of the governed contraction.
///
/// Opaque and decode-only: there is no public constructor from parts, no
/// mutable field, no raw-text accessor, no topology field, no target field, and
/// no fallback decoder. Holding one is evidence that every outer fact of the
/// registered definition — not only field 15 — was validated against the
/// accepted contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContractionF32ReductionDescriptor {
    contributors: ContractionF32ContributorSequence,
    leaf: ContractionF32LeafPrimitive,
    reducer: ContractionF32ReducerPrimitive,
    seed: ContractionF32Seed,
    empty_domain: ContractionF32EmptyDomain,
    reassociation: ContractionF32OrderFreedom,
    permutation: ContractionF32OrderFreedom,
    signed_zero_elimination: ContractionF32OrderFreedom,
    canonical_nan_bits: u32,
    nan_canonicalization: ContractionF32NanCanonicalization,
    stability: ContractionF32StabilityScope,
}

impl ContractionF32ReductionDescriptor {
    /// Decodes and validates one governed contraction definition.
    ///
    /// # Errors
    ///
    /// Returns [`ContractionF32DescriptorError`] naming the first deviation
    /// from the accepted contract, under the precedence the error type
    /// documents. A malformed descriptor never resolves: failure here is the
    /// only alternative to a fully validated value.
    ///
    /// # Panics
    ///
    /// Never panics: the internal lookup runs only after the arity and
    /// membership gates prove every expected field present.
    pub fn decode(definition: &OperationDefinition) -> Result<Self, ContractionF32DescriptorError> {
        let expected = tensor_contraction_f32_op();
        if definition.key() != &expected {
            return Err(ContractionF32DescriptorError::WrongOperation {
                expected,
                actual: definition.key().clone(),
            });
        }
        let facts = definition.canonical_facts().value();
        let CanonicalValueView::Record(fields) = facts.view() else {
            return Err(ContractionF32DescriptorError::MalformedFacts {
                actual: kind_of(facts),
            });
        };
        if fields.len() != EXPECTED_OUTER.len() {
            return Err(ContractionF32DescriptorError::FactCount {
                expected: EXPECTED_OUTER.len(),
                actual: fields.len(),
            });
        }
        // `CanonicalValue::record` already rejects duplicate IDs, so with the
        // arity gated above, a field outside the expected set implies a missing
        // expected field and vice versa; the foreign field is reported.
        if let Some(field) = fields
            .iter()
            .find(|field| !EXPECTED_OUTER.contains(&field.id()))
        {
            return Err(ContractionF32DescriptorError::UnexpectedField {
                field: ContractionF32DescriptorField::Outer(field.id()),
            });
        }
        let outer = |id: AttributeFieldId| -> &CanonicalValue {
            fields
                .iter()
                .find(|field| field.id() == id)
                .expect("the arity and membership gates above make every expected field present")
                .value()
        };

        require_atom(
            outer(CONTRACTION_F32_FACT_COMPUTATION_PRECISION),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_COMPUTATION_PRECISION),
            "binary32-operands-and-binary32-products",
        )?;
        require_f32_type(
            outer(CONTRACTION_F32_FACT_ACCUMULATOR_TYPE),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_ACCUMULATOR_TYPE),
        )?;
        require_f32_type(
            outer(CONTRACTION_F32_FACT_RESULT_TYPE),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_RESULT_TYPE),
        )?;
        require_atom(
            outer(CONTRACTION_F32_FACT_CONVERSION),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_CONVERSION),
            "none-operands-products-accumulator-and-result-are-binary32",
        )?;
        require_atom(
            outer(CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE),
            "ascending-lexicographic-over-the-canonically-ordered-contracted-index-space",
        )?;
        let contributors =
            ContractionF32ContributorSequence::AscendingLexicographicCanonicalContractedIndexSpace;
        require_atom(
            outer(CONTRACTION_F32_FACT_SEED),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_SEED),
            "none-the-accumulator-starts-at-the-first-product",
        )?;
        let seed = ContractionF32Seed::FirstProduct;
        require_atom(
            outer(CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN),
            "refused-an-unseeded-fold-has-no-empty-result",
        )?;
        let empty_domain = ContractionF32EmptyDomain::Refused;
        require_atom(
            outer(CONTRACTION_F32_FACT_DISTRIBUTIVITY),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_DISTRIBUTIVITY),
            "absent-no-expressible-numerical-permission-grants-it",
        )?;
        let contraction_field = ContractionF32DescriptorField::Outer(
            CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
        );
        match outer(CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED).view() {
            CanonicalValueView::Bool(false) => {}
            CanonicalValueView::Bool(true) => {
                return Err(ContractionF32DescriptorError::UnsupportedValue {
                    field: contraction_field,
                });
            }
            other => {
                return Err(ContractionF32DescriptorError::WrongKind {
                    field: contraction_field,
                    expected: CanonicalValueKind::Bool,
                    actual: kind_of_view(&other),
                });
            }
        }
        let canonical_nan_bits = require_canonical_nan(
            outer(CONTRACTION_F32_FACT_CANONICAL_NAN_BITS),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_CANONICAL_NAN_BITS),
        )?;
        require_atom(
            outer(CONTRACTION_F32_FACT_NAN_CANONICALIZATION),
            ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_NAN_CANONICALIZATION),
            "after-every-combine-and-at-the-result-boundary",
        )?;
        let nan_canonicalization =
            ContractionF32NanCanonicalization::AfterEachArithmeticOperationAndResultBoundary;
        let stability = decode_stability(outer(CONTRACTION_F32_FACT_DETERMINISM))?;
        let (leaf, reducer, reassociation, permutation, signed_zero_elimination) =
            decode_reduction(outer(CONTRACTION_F32_FACT_REDUCTION_DESCRIPTOR))?;

        Ok(Self {
            contributors,
            leaf,
            reducer,
            seed,
            empty_domain,
            reassociation,
            permutation,
            signed_zero_elimination,
            canonical_nan_bits,
            nan_canonicalization,
            stability,
        })
    }

    /// Returns the declared contributor sequence.
    #[must_use]
    pub const fn contributors(&self) -> ContractionF32ContributorSequence {
        self.contributors
    }

    /// Returns the declared leaf primitive.
    #[must_use]
    pub const fn leaf(&self) -> ContractionF32LeafPrimitive {
        self.leaf
    }

    /// Returns the declared reducer primitive.
    #[must_use]
    pub const fn reducer(&self) -> ContractionF32ReducerPrimitive {
        self.reducer
    }

    /// Returns the declared seed.
    #[must_use]
    pub const fn seed(&self) -> ContractionF32Seed {
        self.seed
    }

    /// Returns the declared empty-domain behaviour.
    #[must_use]
    pub const fn empty_domain(&self) -> ContractionF32EmptyDomain {
        self.empty_domain
    }

    /// Returns the operation's maximum reassociation freedom.
    #[must_use]
    pub const fn reassociation(&self) -> ContractionF32OrderFreedom {
        self.reassociation
    }

    /// Returns the operation's maximum permutation freedom.
    #[must_use]
    pub const fn permutation(&self) -> ContractionF32OrderFreedom {
        self.permutation
    }

    /// Returns the operation's maximum signed-zero-elimination freedom.
    #[must_use]
    pub const fn signed_zero_elimination(&self) -> ContractionF32OrderFreedom {
        self.signed_zero_elimination
    }

    /// Returns whether ADR 0015 arithmetic contraction is supported.
    ///
    /// Always `false` on a decoded descriptor: the governed definition forbids
    /// fused multiply-add and the decoder refuses a definition that does not.
    #[must_use]
    pub const fn arithmetic_contraction_supported(&self) -> bool {
        false
    }

    /// Returns whether distributivity is supported.
    ///
    /// Always `false` on a decoded descriptor: distributivity remains absent
    /// under ADR 0095, and the decoder refuses a definition claiming otherwise.
    #[must_use]
    pub const fn distributivity_supported(&self) -> bool {
        false
    }

    /// Returns the canonical arithmetic-NaN payload.
    #[must_use]
    pub const fn canonical_nan_bits(&self) -> u32 {
        self.canonical_nan_bits
    }

    /// Returns where the canonical NaN payload is installed.
    #[must_use]
    pub const fn nan_canonicalization(&self) -> ContractionF32NanCanonicalization {
        self.nan_canonicalization
    }

    /// Returns the bound determinism stability scope.
    #[must_use]
    pub const fn stability(&self) -> ContractionF32StabilityScope {
        self.stability
    }
}

/// Decodes the governed contraction's descriptor from a frozen registry.
///
/// # Errors
///
/// Returns [`ContractionF32DescriptorError::OperationMissing`] when the
/// registry does not register `tiler::tensor-contraction-f32@1`, and any
/// [`ContractionF32ReductionDescriptor::decode`] refusal otherwise.
pub fn tensor_contraction_f32_reduction_descriptor(
    registry: &FrozenSemanticRegistry,
) -> Result<ContractionF32ReductionDescriptor, ContractionF32DescriptorError> {
    let operation = tensor_contraction_f32_op();
    let Some(definition) = registry.operation_definition(&operation) else {
        return Err(ContractionF32DescriptorError::OperationMissing { operation });
    };
    ContractionF32ReductionDescriptor::decode(definition)
}

/// Returns the kind of one canonical value.
fn kind_of(value: &CanonicalValue) -> CanonicalValueKind {
    kind_of_view(&value.view())
}

fn kind_of_view(view: &CanonicalValueView<'_>) -> CanonicalValueKind {
    match view {
        CanonicalValueView::Type(_) => CanonicalValueKind::Type,
        CanonicalValueView::Bool(_) => CanonicalValueKind::Bool,
        CanonicalValueView::Signed { .. } => CanonicalValueKind::Signed,
        CanonicalValueView::Unsigned { .. } => CanonicalValueKind::Unsigned,
        CanonicalValueView::FloatBits(_) => CanonicalValueKind::FloatBits,
        CanonicalValueView::Bytes(_) => CanonicalValueKind::Bytes,
        CanonicalValueView::Utf8(_) => CanonicalValueKind::Utf8,
        CanonicalValueView::Sequence(_) => CanonicalValueKind::Sequence,
        CanonicalValueView::Record(_) => CanonicalValueKind::Record,
    }
}

/// Requires one exact governed UTF-8 atom.
fn require_atom(
    value: &CanonicalValue,
    field: ContractionF32DescriptorField,
    expected: &str,
) -> Result<(), ContractionF32DescriptorError> {
    match value.view() {
        CanonicalValueView::Utf8(actual) if actual == expected => Ok(()),
        CanonicalValueView::Utf8(_) => {
            Err(ContractionF32DescriptorError::UnsupportedValue { field })
        }
        other => Err(ContractionF32DescriptorError::WrongKind {
            field,
            expected: CanonicalValueKind::Utf8,
            actual: kind_of_view(&other),
        }),
    }
}

/// Requires the governed F32 resolved type.
fn require_f32_type(
    value: &CanonicalValue,
    field: ContractionF32DescriptorField,
) -> Result<(), ContractionF32DescriptorError> {
    match value.view() {
        CanonicalValueView::Type(actual) if actual == &F32::resolved_type() => Ok(()),
        CanonicalValueView::Type(_) => {
            Err(ContractionF32DescriptorError::UnsupportedValue { field })
        }
        other => Err(ContractionF32DescriptorError::WrongKind {
            field,
            expected: CanonicalValueKind::Type,
            actual: kind_of_view(&other),
        }),
    }
}

/// Requires the exact governed canonical arithmetic-NaN payload.
fn require_canonical_nan(
    value: &CanonicalValue,
    field: ContractionF32DescriptorField,
) -> Result<u32, ContractionF32DescriptorError> {
    match value.view() {
        CanonicalValueView::FloatBits(float_bits) => {
            let expected = crate::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS;
            let f32_key = crate::semantic::F32::resolved_type();
            let is_f32 = f32_key
                .nominal_key()
                .is_some_and(|key| key == float_bits.format());
            if is_f32 && float_bits.bits() == expected.to_be_bytes() {
                Ok(expected)
            } else {
                Err(ContractionF32DescriptorError::UnsupportedValue { field })
            }
        }
        other => Err(ContractionF32DescriptorError::WrongKind {
            field,
            expected: CanonicalValueKind::FloatBits,
            actual: kind_of_view(&other),
        }),
    }
}

/// Decodes and validates one nested record's rows against an expected id set.
///
/// Returns the rows in expected order. Reports a foreign row before a missing
/// one, each in record order; no arity gate runs here, so both refusals are
/// reachable.
fn decode_record_rows<const N: usize>(
    value: &CanonicalValue,
    field: ContractionF32DescriptorField,
    wrap: fn(AttributeFieldId) -> ContractionF32DescriptorField,
    expected: [AttributeFieldId; N],
) -> Result<[&CanonicalValue; N], ContractionF32DescriptorError> {
    let CanonicalValueView::Record(rows) = value.view() else {
        return Err(ContractionF32DescriptorError::WrongKind {
            field,
            expected: CanonicalValueKind::Record,
            actual: kind_of(value),
        });
    };
    if let Some(row) = rows.iter().find(|row| !expected.contains(&row.id())) {
        return Err(ContractionF32DescriptorError::UnexpectedField {
            field: wrap(row.id()),
        });
    }
    let mut values = [None; N];
    for (slot, id) in values.iter_mut().zip(expected) {
        match rows.iter().find(|row| row.id() == id) {
            Some(row) => *slot = Some(row.value()),
            None => {
                return Err(ContractionF32DescriptorError::MissingField { field: wrap(id) });
            }
        }
    }
    Ok(values.map(|slot| slot.expect("every slot was filled or refused above")))
}

/// Decodes the field-14 stability record.
fn decode_stability(
    value: &CanonicalValue,
) -> Result<ContractionF32StabilityScope, ContractionF32DescriptorError> {
    let rows = decode_record_rows(
        value,
        ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_DETERMINISM),
        ContractionF32DescriptorField::Stability,
        [
            STABILITY_FIELD_SCOPE,
            STABILITY_FIELD_EQUAL_INPUTS,
            STABILITY_FIELD_ARTIFACT,
            STABILITY_FIELD_PLAN,
            STABILITY_FIELD_ENVIRONMENT,
            STABILITY_FIELD_RESULT,
            STABILITY_FIELD_RECOMPILATION,
        ],
    )?;
    for ((row, id), expected) in rows
        .iter()
        .zip([
            STABILITY_FIELD_SCOPE,
            STABILITY_FIELD_EQUAL_INPUTS,
            STABILITY_FIELD_ARTIFACT,
            STABILITY_FIELD_PLAN,
            STABILITY_FIELD_ENVIRONMENT,
            STABILITY_FIELD_RESULT,
            STABILITY_FIELD_RECOMPILATION,
        ])
        .zip([
            "plan-deterministic",
            "same-input-bits-and-runtime-bindings",
            "same-artifact-digest",
            "same-selected-plan-variant",
            "same-declared-target-environment",
            "identical-output-bits",
            "different-artifact-may-select-a-different-legal-result",
        ])
    {
        require_atom(row, ContractionF32DescriptorField::Stability(id), expected)?;
    }
    Ok(ContractionF32StabilityScope::PlanDeterministic)
}

/// Decodes the field-15 reduction record.
#[allow(
    clippy::type_complexity,
    reason = "one private decode returns the five decoded rows once"
)]
fn decode_reduction(
    value: &CanonicalValue,
) -> Result<
    (
        ContractionF32LeafPrimitive,
        ContractionF32ReducerPrimitive,
        ContractionF32OrderFreedom,
        ContractionF32OrderFreedom,
        ContractionF32OrderFreedom,
    ),
    ContractionF32DescriptorError,
> {
    let [
        leaf,
        reducer,
        result_class,
        reassociation,
        permutation,
        signed_zero,
    ] = decode_record_rows(
        value,
        ContractionF32DescriptorField::Outer(CONTRACTION_F32_FACT_REDUCTION_DESCRIPTOR),
        ContractionF32DescriptorField::Reduction,
        [
            REDUCTION_FIELD_LEAF,
            REDUCTION_FIELD_REDUCER,
            REDUCTION_FIELD_RESULT_CLASS,
            REDUCTION_FIELD_MAX_REASSOCIATION,
            REDUCTION_FIELD_MAX_PERMUTATION,
            REDUCTION_FIELD_MAX_SIGNED_ZERO,
        ],
    )?;
    require_atom(
        leaf,
        ContractionF32DescriptorField::Reduction(REDUCTION_FIELD_LEAF),
        "input-transform-each-factor-round-binary32-nearest-ties-even-multiply-canonicalize-nan-result-transform",
    )?;
    require_atom(
        reducer,
        ContractionF32DescriptorField::Reduction(REDUCTION_FIELD_REDUCER),
        "input-transform-each-addend-round-binary32-nearest-ties-even-add-canonicalize-nan-result-transform",
    )?;
    require_atom(
        result_class,
        ContractionF32DescriptorField::Reduction(REDUCTION_FIELD_RESULT_CLASS),
        "strict-left-fold-or-ordered-full-binary-trees-by-effective-reassociation",
    )?;
    let reassociation = decode_order_freedom(
        reassociation,
        ContractionF32DescriptorField::Reduction(REDUCTION_FIELD_MAX_REASSOCIATION),
    )?;
    // The result-class rule (sole governed value, required above) switches on
    // effective reassociation; a reassociation maximum of `unsupported` would
    // forbid the freedom that selects between its cells.
    if reassociation == ContractionF32OrderFreedom::Unsupported {
        return Err(ContractionF32DescriptorError::ContradictoryFields {
            first: ContractionF32DescriptorField::Reduction(REDUCTION_FIELD_RESULT_CLASS),
            second: ContractionF32DescriptorField::Reduction(REDUCTION_FIELD_MAX_REASSOCIATION),
        });
    }
    // Permutation and signed-zero elimination are operation-owned unsupported
    // under the accepted contract; `permission-gated` is a well-spelled value
    // this generation does not admit, so a decoded descriptor can never return
    // either freedom as `PermissionGated`. A future key generation that admits
    // fold permutation widens this decode, not a caller's ceiling.
    for (row, id) in [
        (permutation, REDUCTION_FIELD_MAX_PERMUTATION),
        (signed_zero, REDUCTION_FIELD_MAX_SIGNED_ZERO),
    ] {
        let freedom = decode_order_freedom(row, ContractionF32DescriptorField::Reduction(id))?;
        if freedom != ContractionF32OrderFreedom::Unsupported {
            return Err(ContractionF32DescriptorError::UnsupportedValue {
                field: ContractionF32DescriptorField::Reduction(id),
            });
        }
    }
    Ok((
        ContractionF32LeafPrimitive::TransformOperandsRoundBinary32NearestTiesEvenMultiplyCanonicalizeNanTransformResult,
        ContractionF32ReducerPrimitive::TransformOperandsRoundBinary32NearestTiesEvenAddCanonicalizeNanTransformResult,
        reassociation,
        ContractionF32OrderFreedom::Unsupported,
        ContractionF32OrderFreedom::Unsupported,
    ))
}

/// Decodes one order-freedom row.
fn decode_order_freedom(
    value: &CanonicalValue,
    field: ContractionF32DescriptorField,
) -> Result<ContractionF32OrderFreedom, ContractionF32DescriptorError> {
    match value.view() {
        CanonicalValueView::Utf8("permission-gated") => {
            Ok(ContractionF32OrderFreedom::PermissionGated)
        }
        CanonicalValueView::Utf8("unsupported") => Ok(ContractionF32OrderFreedom::Unsupported),
        CanonicalValueView::Utf8(_) => {
            Err(ContractionF32DescriptorError::UnsupportedValue { field })
        }
        other => Err(ContractionF32DescriptorError::WrongKind {
            field,
            expected: CanonicalValueKind::Utf8,
            actual: kind_of_view(&other),
        }),
    }
}
