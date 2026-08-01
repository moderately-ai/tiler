//! The four discriminated contract forms, and the composition that verifies one.
//!
//! ADR 0042: "Every transcendental operation identifies immutable, versioned
//! reference semantics and **one** discriminated accuracy contract" — correctly
//! rounded, faithful, bounded piecewise, or named elementary behaviour. The four
//! are "never equated by name", which is why [`AccuracyContractForm`] is a closed
//! discriminated enum rather than a bound with a label: `Faithful` and
//! `Ulp(metric, 1)` are different obligations, and a representation that could
//! spell them the same way would let one be substituted for the other.
//!
//! # The five-step composition
//!
//! The observable result set is composed in ADR 0042's order, and the order is
//! load-bearing rather than descriptive: applying the input-subnormal contract
//! *after* classifying the reference would classify a reference the operation
//! never sees, and applying NaN canonicalization before the subnormal mapping
//! would canonicalize a value the mapping is about to replace.
//! [`AccuracyContract::composition_steps`] returns the order as data so a
//! consumer walks it rather than reimplementing it.
//!
//! **Where the set can be empty.** Step 3 is the only step that can fail to admit
//! a candidate. Steps 1, 4, and 5 are total mappings on values — a flush replaces
//! a subnormal with a stated zero, a canonicalization replaces one NaN payload
//! with another — so they transform the set without removing its last member, and
//! step 2 classifies rather than selects. A future *partial* output mapping would
//! change that, and would owe its own emptiness check;
//! [`CompositionStep::can_empty_the_result_set`] records which steps carry the
//! obligation today so that a new one cannot be added without contradicting it.
//!
//! # How nonemptiness is established
//!
//! ADR 0042 requires verification to establish that the composed set is nonempty
//! **for every admitted input**, so a sampled check is not one. The proof used
//! here is a witness: *round-to-nearest is the optimal candidate*. It minimizes
//! `|z - r|` over the representable values, and every atomic predicate in this
//! algebra bounds a monotone decreasing function of `|z - r|`, so if
//! round-to-nearest fails a bound then every candidate fails it, and if it
//! satisfies one then the set is nonempty. For `Absolute`, `Relative`, and `Ulp`
//! the check is therefore exact rather than merely sufficient.
//!
//! The one incompleteness is deliberate and fails closed: for `AnyOf` this module
//! requires a *single* member to hold across the whole cell, so a disjunction
//! covered piecewise by two members is rejected as unestablished rather than
//! accepted. Splitting the clause's domain is what a contract does about that,
//! and the refusal names it.

use std::fmt;

use crate::identity::{push_len, push_slice};
use crate::semantic::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueView, NormativeDefinitionRef,
    OpKey, ResolvedValueType, TypeKey,
};

use super::domain::{AccuracyDomain, CoveredCell};
use super::error::{
    AccuracyAttributeSubject, AccuracyContractError, UnestablishedResultSet, malformed,
};
use super::metric::{UlpFormat, ulp_reference_gap_metric_key};
use super::predicate::{AccuracyPredicate, AccuracyPredicateView, BooleanPredicateKind};
use super::rational::{ExactRational, ExactTolerance};

/// Maximum bytes one named-elementary descriptor digest may carry.
pub const MAX_NAMED_ELEMENTARY_DIGEST_BYTES: usize = 128;

/// Domain separator of a canonical accuracy-contract encoding.
const ACCURACY_CONTRACT_DOMAIN: &[u8] = b"tiler.accuracy-contract.v1\0";

const CONTRACT_OPERATION: AttributeFieldId = AttributeFieldId::new(1);
const CONTRACT_OPERAND_TYPES: AttributeFieldId = AttributeFieldId::new(2);
const CONTRACT_RESULT_TYPE: AttributeFieldId = AttributeFieldId::new(3);
const CONTRACT_REFERENCE_SEMANTICS: AttributeFieldId = AttributeFieldId::new(4);
const CONTRACT_FORM: AttributeFieldId = AttributeFieldId::new(5);
const CONTRACT_ROUNDING: AttributeFieldId = AttributeFieldId::new(6);
const CONTRACT_DOMAIN: AttributeFieldId = AttributeFieldId::new(7);
const CONTRACT_PROFILE_KEY: AttributeFieldId = AttributeFieldId::new(8);
const CONTRACT_PROFILE_DIGEST: AttributeFieldId = AttributeFieldId::new(9);
const CONTRACT_PROFILE_BASIS: AttributeFieldId = AttributeFieldId::new(10);
const CONTRACT_EXCEPTIONAL: AttributeFieldId = AttributeFieldId::new(11);

const EXCEPTIONAL_NAN_REFERENCE: AttributeFieldId = AttributeFieldId::new(1);
const EXCEPTIONAL_INFINITE_REFERENCE: AttributeFieldId = AttributeFieldId::new(2);
const EXCEPTIONAL_OUTSIDE_DOMAIN: AttributeFieldId = AttributeFieldId::new(3);
const EXCEPTIONAL_FINITE_OVERFLOW: AttributeFieldId = AttributeFieldId::new(4);

/// The rounding rule a correctly rounded contract names.
///
/// Deliberately **not** [`crate::schedule::MaterializationRounding`], which
/// resolves the rounding an observable *materialization boundary* applies. This
/// resolves the single rounding of an infinitely precise reference result to the
/// result dtype. The two carry the same admitted direction today because ADR 0024
/// fixes one initial arithmetic rounding, and they are separate types because
/// they answer different questions: a directed-rounding cast and a
/// directed-rounding elementary function are different capabilities, and one
/// arriving must not silently declare the other.
///
/// Not `#[non_exhaustive]`: every encoder matches it exhaustively, so admitting a
/// second direction is a build error at each site rather than a silently carried
/// assumption.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReferenceRoundingRule {
    /// Round to nearest, ties to even.
    NearestTiesToEven,
}

impl ReferenceRoundingRule {
    const fn spelling(self) -> &'static str {
        match self {
            Self::NearestTiesToEven => "nearest-ties-to-even",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "nearest-ties-to-even" => Some(Self::NearestTiesToEven),
            _ => None,
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::NearestTiesToEven => 1,
        }
    }
}

impl fmt::Display for ReferenceRoundingRule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.spelling())
    }
}

/// A governed, nominal named-elementary behaviour profile identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NamedElementaryProfileKey(TypeKey);

impl NamedElementaryProfileKey {
    /// Creates a validated, versioned profile key.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::CanonicalBound`] for an invalid component
    /// or version.
    pub fn new(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        semantic_version: u32,
    ) -> Result<Self, AccuracyContractError> {
        Ok(Self(TypeKey::new(namespace, name, semantic_version)?))
    }

    /// Returns the canonical namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        self.0.namespace()
    }

    /// Returns the name within the namespace.
    #[must_use]
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// Returns the nonzero semantic version.
    #[must_use]
    pub const fn semantic_version(&self) -> u32 {
        self.0.semantic_version()
    }

    fn encode(&self, output: &mut Vec<u8>) {
        push_slice(output, self.namespace().as_bytes());
        push_slice(output, self.name().as_bytes());
        output.extend_from_slice(&self.semantic_version().to_be_bytes());
    }
}

impl fmt::Display for NamedElementaryProfileKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// The immutable canonical descriptor digest a named profile is pinned to.
///
/// Opaque bytes, supplied by the authority that owns the descriptor rather than
/// computed here: Tiler does not define a vendor's behaviour profile, so it
/// cannot derive that profile's digest. What it can do — and what this type is
/// for — is refuse to treat two descriptors as one. ADR 0042: "a key/revision
/// cannot change that descriptor", so the digest travels with the key and a
/// changed descriptor is a changed contract.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NamedElementaryDescriptorDigest(Vec<u8>);

impl NamedElementaryDescriptorDigest {
    /// Creates a nonempty bounded descriptor digest.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError::MalformedAttribute`] for an empty or
    /// over-long digest. An empty digest is refused because it would pin nothing
    /// while looking like a pin.
    pub fn new(bytes: impl AsRef<[u8]>) -> Result<Self, AccuracyContractError> {
        let bytes = bytes.as_ref();
        if bytes.is_empty() || bytes.len() > MAX_NAMED_ELEMENTARY_DIGEST_BYTES {
            return Err(malformed(
                AccuracyAttributeSubject::NamedElementaryDescriptor,
            ));
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Returns the exact digest bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// What one operation does at a reference that is not ordinary, finite, and in range.
///
/// ADR 0042 keeps these "independent exceptional-value contracts" separate from
/// the error metric, and ADR 0016's consequence list makes the same point: an
/// accuracy bound says nothing about NaN, infinity, a domain error, or a finite
/// overflow, and inferring one from the other is how a contract acquires
/// behaviour nobody wrote.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExceptionalValueContract {
    nan_reference: NanReferenceRule,
    infinite_reference: InfiniteReferenceRule,
    outside_domain: DomainErrorRule,
    finite_overflow: FiniteOverflowRule,
}

impl ExceptionalValueContract {
    /// States all four rules, each explicitly.
    ///
    /// Four required arguments rather than a builder with defaults, because a
    /// defaulted exceptional rule is a behaviour nobody chose.
    #[must_use]
    pub const fn new(
        nan_reference: NanReferenceRule,
        infinite_reference: InfiniteReferenceRule,
        outside_domain: DomainErrorRule,
        finite_overflow: FiniteOverflowRule,
    ) -> Self {
        Self {
            nan_reference,
            infinite_reference,
            outside_domain,
            finite_overflow,
        }
    }

    /// Returns the rule for a NaN reference.
    #[must_use]
    pub const fn nan_reference(self) -> NanReferenceRule {
        self.nan_reference
    }

    /// Returns the rule for an infinite reference.
    #[must_use]
    pub const fn infinite_reference(self) -> InfiniteReferenceRule {
        self.infinite_reference
    }

    /// Returns the rule for an input outside the admitted domain.
    #[must_use]
    pub const fn outside_domain(self) -> DomainErrorRule {
        self.outside_domain
    }

    /// Returns the rule for a finite reference above the format's finite range.
    #[must_use]
    pub const fn finite_overflow(self) -> FiniteOverflowRule {
        self.finite_overflow
    }

    fn encode(self, output: &mut Vec<u8>) {
        output.push(self.nan_reference.tag());
        output.push(self.infinite_reference.tag());
        output.push(self.outside_domain.tag());
        output.push(self.finite_overflow.tag());
    }

    fn to_canonical_value(self) -> Result<CanonicalValue, AccuracyContractError> {
        Ok(CanonicalValue::record([
            CanonicalField::new(
                EXCEPTIONAL_NAN_REFERENCE,
                CanonicalValue::utf8(self.nan_reference.spelling())?,
            ),
            CanonicalField::new(
                EXCEPTIONAL_INFINITE_REFERENCE,
                CanonicalValue::utf8(self.infinite_reference.spelling())?,
            ),
            CanonicalField::new(
                EXCEPTIONAL_OUTSIDE_DOMAIN,
                CanonicalValue::utf8(self.outside_domain.spelling())?,
            ),
            CanonicalField::new(
                EXCEPTIONAL_FINITE_OVERFLOW,
                CanonicalValue::utf8(self.finite_overflow.spelling())?,
            ),
        ])?)
    }

    fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        let subject = || malformed(AccuracyAttributeSubject::ExceptionalValueContract);
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(subject());
        };
        let [nan, infinite, outside, overflow] = fields else {
            return Err(subject());
        };
        if nan.id() != EXCEPTIONAL_NAN_REFERENCE
            || infinite.id() != EXCEPTIONAL_INFINITE_REFERENCE
            || outside.id() != EXCEPTIONAL_OUTSIDE_DOMAIN
            || overflow.id() != EXCEPTIONAL_FINITE_OVERFLOW
        {
            return Err(subject());
        }
        let text = |field: &CanonicalField| match field.value().view() {
            CanonicalValueView::Utf8(value) => Ok(value.to_owned()),
            _ => Err(subject()),
        };
        Ok(Self::new(
            NanReferenceRule::parse(&text(nan)?).ok_or_else(subject)?,
            InfiniteReferenceRule::parse(&text(infinite)?).ok_or_else(subject)?,
            DomainErrorRule::parse(&text(outside)?).ok_or_else(subject)?,
            FiniteOverflowRule::parse(&text(overflow)?).ok_or_else(subject)?,
        ))
    }
}

/// What the operation returns when the exact reference is a NaN.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NanReferenceRule {
    /// The operation's canonical arithmetic NaN payload.
    CanonicalNan,
    /// The occurrence is refused at construction rather than evaluated.
    Refuse,
}

/// What the operation returns when the exact reference is infinite.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InfiniteReferenceRule {
    /// The infinity of the reference's sign.
    SignedInfinity,
    /// The operation's canonical arithmetic NaN payload.
    CanonicalNan,
    /// The occurrence is refused at construction rather than evaluated.
    Refuse,
}

/// What the operation returns for an input outside its admitted domain.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DomainErrorRule {
    /// The operation's canonical arithmetic NaN payload.
    CanonicalNan,
    /// The occurrence is refused at construction rather than evaluated.
    Refuse,
}

/// What the operation returns when a finite reference exceeds the format's finite range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FiniteOverflowRule {
    /// The infinity of the reference's sign.
    SignedInfinity,
    /// The largest finite magnitude of the reference's sign.
    LargestFinite,
    /// The occurrence is refused at construction rather than evaluated.
    Refuse,
}

macro_rules! spelled_rule {
    ($type:ty, $( $variant:ident => $spelling:literal, $tag:literal );+ $(;)?) => {
        impl $type {
            const fn spelling(self) -> &'static str {
                match self {
                    $( Self::$variant => $spelling, )+
                }
            }

            fn parse(value: &str) -> Option<Self> {
                match value {
                    $( $spelling => Some(Self::$variant), )+
                    _ => None,
                }
            }

            const fn tag(self) -> u8 {
                match self {
                    $( Self::$variant => $tag, )+
                }
            }
        }

        impl fmt::Display for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.spelling())
            }
        }
    };
}

spelled_rule!(NanReferenceRule, CanonicalNan => "canonical-nan", 1; Refuse => "refuse", 2);
spelled_rule!(
    InfiniteReferenceRule,
    SignedInfinity => "signed-infinity", 1;
    CanonicalNan => "canonical-nan", 2;
    Refuse => "refuse", 3
);
spelled_rule!(DomainErrorRule, CanonicalNan => "canonical-nan", 1; Refuse => "refuse", 2);
spelled_rule!(
    FiniteOverflowRule,
    SignedInfinity => "signed-infinity", 1;
    LargestFinite => "largest-finite", 2;
    Refuse => "refuse", 3
);

/// One of ADR 0042's four discriminated accuracy contract forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccuracyContractForm {
    /// Round the infinitely precise reference once, using the named rule.
    CorrectlyRounded {
        /// The single rounding applied to the reference.
        rounding: ReferenceRoundingRule,
    },
    /// Return the exact representable reference, or either bracketing neighbour.
    ///
    /// **Not** a one-ULP bound. Faithful rounding admits both neighbours of an
    /// inexact reference and *only* those two; a one-ULP bound admits every value
    /// within one ULP, which near a binade boundary is a different set. ADR 0042
    /// forbids equating them by name and this enum forbids it by construction.
    Faithful,
    /// Satisfy a complete set of typed domain clauses carrying exact bounds.
    BoundedPiecewise(AccuracyDomain),
    /// Satisfy one immutable, versioned behaviour profile pinned by descriptor digest.
    NamedElementary {
        /// The governed nominal profile identity.
        profile: NamedElementaryProfileKey,
        /// The immutable canonical descriptor digest the key is pinned to.
        descriptor_digest: NamedElementaryDescriptorDigest,
        /// The authority whose descriptor completely defines domains and results.
        descriptor_basis: NormativeDefinitionRef,
    },
}

impl AccuracyContractForm {
    const fn spelling(&self) -> &'static str {
        match self {
            Self::CorrectlyRounded { .. } => "correctly-rounded",
            Self::Faithful => "faithful",
            Self::BoundedPiecewise(_) => "bounded-piecewise",
            Self::NamedElementary { .. } => "named-elementary",
        }
    }

    const fn tag(&self) -> u8 {
        match self {
            Self::CorrectlyRounded { .. } => 1,
            Self::Faithful => 2,
            Self::BoundedPiecewise(_) => 3,
            Self::NamedElementary { .. } => 4,
        }
    }
}

/// One step of ADR 0042's observable-result-set composition, in order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompositionStep {
    /// Apply the operation's input-subnormal contract.
    InputSubnormalContract,
    /// Compute and classify the exact reference result.
    ExactReferenceClassification,
    /// Select an accuracy-conforming candidate, or apply the exceptional contract.
    AccuracyConformingCandidateSelection,
    /// Apply the resolved result-subnormal and signed-zero mappings.
    ResultSubnormalAndSignedZeroMapping,
    /// Apply any required NaN canonicalization and expose the value-only result.
    NanCanonicalization,
}

impl CompositionStep {
    /// The five steps, in ADR 0042's order.
    pub const ORDER: [Self; 5] = [
        Self::InputSubnormalContract,
        Self::ExactReferenceClassification,
        Self::AccuracyConformingCandidateSelection,
        Self::ResultSubnormalAndSignedZeroMapping,
        Self::NanCanonicalization,
    ];

    /// Returns whether this step can leave the composed result set empty.
    ///
    /// Only step three can. Steps one, four, and five are total mappings on
    /// values and step two classifies rather than selects, so none of them can
    /// remove the set's last member. A future *partial* output mapping would
    /// contradict this and would owe its own emptiness check — which is why the
    /// claim is a function that a new mapping's author has to change rather than
    /// a sentence in a comment.
    #[must_use]
    pub const fn can_empty_the_result_set(self) -> bool {
        match self {
            Self::AccuracyConformingCandidateSelection => true,
            Self::InputSubnormalContract
            | Self::ExactReferenceClassification
            | Self::ResultSubnormalAndSignedZeroMapping
            | Self::NanCanonicalization => false,
        }
    }
}

/// How verification established that the composed result set is nonempty.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResultSetEstablishment {
    /// The correctly rounded value is itself the admitted result at every reference.
    ExactRounding,
    /// Round-to-nearest was exhibited as a conforming candidate over every cell.
    ///
    /// The optimal witness: it minimizes `|z - r|`, and every atomic predicate
    /// bounds a monotone decreasing function of that quantity.
    RoundToNearestWitness {
        /// How many elementary cells the coverage decomposition examined.
        cells: usize,
    },
    /// The profile's own immutable descriptor defines the allowed results.
    ///
    /// This module cannot derive nonemptiness for a named profile, because the
    /// descriptor rather than Tiler defines the result set. Recording *that* as
    /// the basis — rather than reporting an establishment this build did not
    /// perform — is what keeps the two claims distinguishable.
    NamedProfileDescriptor {
        /// The profile whose descriptor carries the claim.
        profile: NamedElementaryProfileKey,
        /// The digest the profile is pinned to.
        digest: NamedElementaryDescriptorDigest,
    },
}

/// The complete resolved accuracy contract of one operation and dtype signature.
///
/// ADR 0042: "The semantic identity contains the operation and dtype signature,
/// reference semantics, complete accuracy contract, domains, exact bounds, metric
/// versions, and the independent exceptional-value contracts." Every one of those
/// is a field here, and [`Self::canonical_encoding`] covers all of them, so two
/// contracts differing in any of them are two identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccuracyContract {
    operation: OpKey,
    operand_types: Vec<ResolvedValueType>,
    result_type: ResolvedValueType,
    reference_semantics: NormativeDefinitionRef,
    form: AccuracyContractForm,
    exceptional: ExceptionalValueContract,
}

impl AccuracyContract {
    /// Assembles one resolved accuracy contract.
    #[must_use]
    pub fn new(
        operation: OpKey,
        operand_types: Vec<ResolvedValueType>,
        result_type: ResolvedValueType,
        reference_semantics: NormativeDefinitionRef,
        form: AccuracyContractForm,
        exceptional: ExceptionalValueContract,
    ) -> Self {
        Self {
            operation,
            operand_types,
            result_type,
            reference_semantics,
            form,
            exceptional,
        }
    }

    /// Returns the operation this contract resolves accuracy for.
    #[must_use]
    pub const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Returns the operand dtype signature.
    #[must_use]
    pub fn operand_types(&self) -> &[ResolvedValueType] {
        &self.operand_types
    }

    /// Returns the result dtype.
    #[must_use]
    pub const fn result_type(&self) -> &ResolvedValueType {
        &self.result_type
    }

    /// Returns the immutable versioned reference semantics.
    #[must_use]
    pub const fn reference_semantics(&self) -> &NormativeDefinitionRef {
        &self.reference_semantics
    }

    /// Returns the discriminated contract form.
    #[must_use]
    pub const fn form(&self) -> &AccuracyContractForm {
        &self.form
    }

    /// Returns the independent exceptional-value contract.
    #[must_use]
    pub const fn exceptional(&self) -> ExceptionalValueContract {
        self.exceptional
    }

    /// Returns the five composition steps, in ADR 0042's order.
    ///
    /// Data rather than prose, so a consumer that must perform the composition
    /// walks the same order this vocabulary states instead of reconstructing it.
    #[must_use]
    pub const fn composition_steps(&self) -> [CompositionStep; 5] {
        CompositionStep::ORDER
    }

    /// Verifies the contract against the result dtype's own descriptor facts.
    ///
    /// Decides, in order: metric/dtype compatibility, complete coverage of the
    /// admitted ordinary input domain, the recursive definedness rule for
    /// relative predicates, and the nonemptiness of the composed observable
    /// result set at every admitted input.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] naming the violated rule. Every refusal
    /// is a rule ADR 0042 states; there is no generic invalidity.
    pub fn verify(
        &self,
        result_type_facts: &CanonicalValue,
    ) -> Result<VerifiedAccuracyContract, AccuracyContractError> {
        let establishment = match &self.form {
            AccuracyContractForm::CorrectlyRounded { .. } | AccuracyContractForm::Faithful => {
                ResultSetEstablishment::ExactRounding
            }
            AccuracyContractForm::NamedElementary {
                profile,
                descriptor_digest,
                ..
            } => ResultSetEstablishment::NamedProfileDescriptor {
                profile: profile.clone(),
                digest: descriptor_digest.clone(),
            },
            AccuracyContractForm::BoundedPiecewise(domain) => {
                let format = UlpFormat::from_value_type_facts(result_type_facts)?;
                let cells = domain.verify_coverage()?;
                for cell in &cells {
                    verify_cell(cell, &format)?;
                }
                ResultSetEstablishment::RoundToNearestWitness { cells: cells.len() }
            }
        };
        Ok(VerifiedAccuracyContract {
            contract: self.clone(),
            establishment,
        })
    }

    /// Returns the domain-separated canonical encoding of this contract.
    #[must_use]
    pub fn canonical_encoding(&self) -> CanonicalAccuracyContract {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, ACCURACY_CONTRACT_DOMAIN);
        self.operation.encode(&mut bytes);
        push_len(&mut bytes, self.operand_types.len());
        for operand in &self.operand_types {
            operand.encode(&mut bytes);
        }
        self.result_type.encode(&mut bytes);
        push_slice(&mut bytes, self.reference_semantics.as_str().as_bytes());
        bytes.push(self.form.tag());
        match &self.form {
            AccuracyContractForm::CorrectlyRounded { rounding } => bytes.push(rounding.tag()),
            AccuracyContractForm::Faithful => {}
            AccuracyContractForm::BoundedPiecewise(domain) => domain.encode(&mut bytes),
            AccuracyContractForm::NamedElementary {
                profile,
                descriptor_digest,
                descriptor_basis,
            } => {
                profile.encode(&mut bytes);
                push_slice(&mut bytes, descriptor_digest.as_bytes());
                push_slice(&mut bytes, descriptor_basis.as_str().as_bytes());
            }
        }
        self.exceptional.encode(&mut bytes);
        CanonicalAccuracyContract(bytes)
    }

    /// Returns the canonical attribute value an occurrence carries.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] when the contract exceeds a canonical
    /// structural bound.
    pub fn to_canonical_value(&self) -> Result<CanonicalValue, AccuracyContractError> {
        let mut fields = vec![
            CanonicalField::new(
                CONTRACT_OPERATION,
                CanonicalValue::record([
                    CanonicalField::new(
                        AttributeFieldId::new(1),
                        CanonicalValue::utf8(self.operation.namespace())?,
                    ),
                    CanonicalField::new(
                        AttributeFieldId::new(2),
                        CanonicalValue::utf8(self.operation.name())?,
                    ),
                    CanonicalField::new(
                        AttributeFieldId::new(3),
                        CanonicalValue::unsigned_u32(self.operation.semantic_version()),
                    ),
                ])?,
            ),
            CanonicalField::new(
                CONTRACT_OPERAND_TYPES,
                CanonicalValue::sequence(
                    self.operand_types
                        .iter()
                        .map(|operand| CanonicalValue::value_type(operand.clone())),
                )?,
            ),
            CanonicalField::new(
                CONTRACT_RESULT_TYPE,
                CanonicalValue::value_type(self.result_type.clone()),
            ),
            CanonicalField::new(
                CONTRACT_REFERENCE_SEMANTICS,
                CanonicalValue::utf8(self.reference_semantics.as_str())?,
            ),
            CanonicalField::new(CONTRACT_FORM, CanonicalValue::utf8(self.form.spelling())?),
            CanonicalField::new(CONTRACT_EXCEPTIONAL, self.exceptional.to_canonical_value()?),
        ];
        match &self.form {
            AccuracyContractForm::CorrectlyRounded { rounding } => {
                fields.push(CanonicalField::new(
                    CONTRACT_ROUNDING,
                    CanonicalValue::utf8(rounding.spelling())?,
                ));
            }
            AccuracyContractForm::Faithful => {}
            AccuracyContractForm::BoundedPiecewise(domain) => fields.push(CanonicalField::new(
                CONTRACT_DOMAIN,
                domain.to_canonical_value()?,
            )),
            AccuracyContractForm::NamedElementary {
                profile,
                descriptor_digest,
                descriptor_basis,
            } => {
                fields.push(CanonicalField::new(
                    CONTRACT_PROFILE_KEY,
                    CanonicalValue::record([
                        CanonicalField::new(
                            AttributeFieldId::new(1),
                            CanonicalValue::utf8(profile.namespace())?,
                        ),
                        CanonicalField::new(
                            AttributeFieldId::new(2),
                            CanonicalValue::utf8(profile.name())?,
                        ),
                        CanonicalField::new(
                            AttributeFieldId::new(3),
                            CanonicalValue::unsigned_u32(profile.semantic_version()),
                        ),
                    ])?,
                ));
                fields.push(CanonicalField::new(
                    CONTRACT_PROFILE_DIGEST,
                    CanonicalValue::bytes(descriptor_digest.as_bytes())?,
                ));
                fields.push(CanonicalField::new(
                    CONTRACT_PROFILE_BASIS,
                    CanonicalValue::utf8(descriptor_basis.as_str())?,
                ));
            }
        }
        Ok(CanonicalValue::record(fields)?)
    }

    /// Decodes one contract exactly as an occurrence carries it.
    ///
    /// # Errors
    ///
    /// Returns [`AccuracyContractError`] for a malformed record or any violated
    /// canonicality rule the nested predicate and domain decoders enforce.
    pub fn from_canonical_value(value: &CanonicalValue) -> Result<Self, AccuracyContractError> {
        let subject = || malformed(AccuracyAttributeSubject::ContractRecord);
        let CanonicalValueView::Record(fields) = value.view() else {
            return Err(subject());
        };
        let find = |id| {
            fields
                .iter()
                .find(|field| field.id() == id)
                .map(CanonicalField::value)
        };
        let operation = decode_key(find(CONTRACT_OPERATION).ok_or_else(subject)?)?;
        let operation = OpKey::new(operation.0, operation.1, operation.2)?;
        let CanonicalValueView::Sequence(operand_values) =
            find(CONTRACT_OPERAND_TYPES).ok_or_else(subject)?.view()
        else {
            return Err(subject());
        };
        let mut operand_types = Vec::with_capacity(operand_values.len());
        for operand in operand_values {
            let CanonicalValueView::Type(resolved) = operand.view() else {
                return Err(subject());
            };
            operand_types.push(resolved.clone());
        }
        let CanonicalValueView::Type(result_type) =
            find(CONTRACT_RESULT_TYPE).ok_or_else(subject)?.view()
        else {
            return Err(subject());
        };
        let CanonicalValueView::Utf8(reference_semantics) = find(CONTRACT_REFERENCE_SEMANTICS)
            .ok_or_else(subject)?
            .view()
        else {
            return Err(subject());
        };
        let CanonicalValueView::Utf8(form) = find(CONTRACT_FORM).ok_or_else(subject)?.view() else {
            return Err(malformed(AccuracyAttributeSubject::ContractForm));
        };
        let form = match form {
            "correctly-rounded" => {
                let CanonicalValueView::Utf8(rounding) = find(CONTRACT_ROUNDING)
                    .ok_or_else(|| malformed(AccuracyAttributeSubject::RoundingRule))?
                    .view()
                else {
                    return Err(malformed(AccuracyAttributeSubject::RoundingRule));
                };
                AccuracyContractForm::CorrectlyRounded {
                    rounding: ReferenceRoundingRule::parse(rounding)
                        .ok_or_else(|| malformed(AccuracyAttributeSubject::RoundingRule))?,
                }
            }
            "faithful" => AccuracyContractForm::Faithful,
            "bounded-piecewise" => AccuracyContractForm::BoundedPiecewise(
                AccuracyDomain::from_canonical_value(find(CONTRACT_DOMAIN).ok_or_else(subject)?)?,
            ),
            "named-elementary" => {
                let profile = decode_key(find(CONTRACT_PROFILE_KEY).ok_or_else(subject)?)?;
                let CanonicalValueView::Bytes(digest) = find(CONTRACT_PROFILE_DIGEST)
                    .ok_or_else(|| malformed(AccuracyAttributeSubject::NamedElementaryDescriptor))?
                    .view()
                else {
                    return Err(malformed(
                        AccuracyAttributeSubject::NamedElementaryDescriptor,
                    ));
                };
                let CanonicalValueView::Utf8(basis) =
                    find(CONTRACT_PROFILE_BASIS).ok_or_else(subject)?.view()
                else {
                    return Err(subject());
                };
                AccuracyContractForm::NamedElementary {
                    profile: NamedElementaryProfileKey::new(profile.0, profile.1, profile.2)?,
                    descriptor_digest: NamedElementaryDescriptorDigest::new(digest)?,
                    descriptor_basis: NormativeDefinitionRef::new(basis).map_err(|_| subject())?,
                }
            }
            _ => return Err(malformed(AccuracyAttributeSubject::ContractForm)),
        };
        Ok(Self::new(
            operation,
            operand_types,
            result_type.clone(),
            NormativeDefinitionRef::new(reference_semantics).map_err(|_| subject())?,
            form,
            ExceptionalValueContract::from_canonical_value(
                find(CONTRACT_EXCEPTIONAL).ok_or_else(subject)?,
            )?,
        ))
    }
}

fn decode_key(value: &CanonicalValue) -> Result<(String, String, u32), AccuracyContractError> {
    let subject = || malformed(AccuracyAttributeSubject::ContractRecord);
    let CanonicalValueView::Record(fields) = value.view() else {
        return Err(subject());
    };
    let [namespace, name, version] = fields else {
        return Err(subject());
    };
    let (
        CanonicalValueView::Utf8(namespace),
        CanonicalValueView::Utf8(name),
        CanonicalValueView::Unsigned { bits, .. },
    ) = (
        namespace.value().view(),
        name.value().view(),
        version.value().view(),
    )
    else {
        return Err(subject());
    };
    Ok((
        namespace.to_owned(),
        name.to_owned(),
        u32::try_from(bits).map_err(|_| subject())?,
    ))
}

/// Collision-free canonical encoding of one complete accuracy contract.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalAccuracyContract(Vec<u8>);

impl CanonicalAccuracyContract {
    /// Returns the domain-separated canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A contract whose every ADR 0042 verification rule was decided.
///
/// There is no unchecked constructor, so holding one is evidence that coverage,
/// definedness, metric compatibility, and result-set nonemptiness were all
/// decided — the same discipline
/// [`crate::semantic::ContractionIndexStructure`] applies to its five rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedAccuracyContract {
    contract: AccuracyContract,
    establishment: ResultSetEstablishment,
}

impl VerifiedAccuracyContract {
    /// Returns the verified contract.
    #[must_use]
    pub const fn contract(&self) -> &AccuracyContract {
        &self.contract
    }

    /// Returns how the composed result set was established as nonempty.
    #[must_use]
    pub const fn establishment(&self) -> &ResultSetEstablishment {
        &self.establishment
    }
}

/// Decides one elementary cell's definedness and nonemptiness obligations.
fn verify_cell(cell: &CoveredCell<'_>, format: &UlpFormat) -> Result<(), AccuracyContractError> {
    // Intersection semantics: every applicable clause applies, so the obligation
    // here is their conjunction. Built through the normalizing constructor rather
    // than assembled by hand, so the conjunction obeys the same flattening,
    // ordering, and bound rules every other predicate does.
    let members: Vec<AccuracyPredicate> = cell
        .applicable()
        .iter()
        .map(|clause| clause.predicate().clone())
        .collect();
    let conjunction = AccuracyPredicate::all_of(members)?;
    if conjunction.requires_nonzero_reference() && !cell.proves_nonzero_reference() {
        return Err(AccuracyContractError::UndefinedRelativePredicateAtZeroReference);
    }
    let (lower, upper) = cell.reference_magnitude_bounds();
    establish(&conjunction, format, lower.as_ref(), upper.as_ref())
        .map_err(|reason| AccuracyContractError::EmptyComposedResultSet { reason })
}

/// Returns whether round-to-nearest conforms at every reference the cell admits.
fn establish(
    predicate: &AccuracyPredicate,
    format: &UlpFormat,
    lower: Option<&ExactRational>,
    upper: Option<&ExactRational>,
) -> Result<(), UnestablishedResultSet> {
    let half = ExactRational::power_of_two(-1);
    match predicate.view() {
        AccuracyPredicateView::Ulp { metric, tolerance } => {
            if !metric.is_ulp_reference_gap() {
                return Err(UnestablishedResultSet::UnregisteredMetric {
                    metric: metric.clone(),
                });
            }
            if *tolerance.value() < half {
                return Err(UnestablishedResultSet::UlpToleranceBelowRoundingFloor {
                    tolerance: tolerance.value().clone(),
                });
            }
            Ok(())
        }
        AccuracyPredicateView::Absolute { tolerance } => {
            establish_absolute(tolerance, format, upper)
        }
        AccuracyPredicateView::Relative { tolerance } => {
            establish_relative(tolerance, format, lower)
        }
        AccuracyPredicateView::AbsoluteRelative { absolute, relative } => {
            // `|z - r| <= a + q|r|` is implied by either term alone, so either
            // sufficient condition discharges it. The absolute term's failure is
            // reported when both fail, because it is the unconditional one.
            establish_relative(relative, format, lower)
                .or_else(|_| establish_absolute(absolute, format, upper))
        }
        AccuracyPredicateView::Boolean { kind, members } => match kind {
            BooleanPredicateKind::AllOf => {
                for member in members {
                    establish(member, format, lower, upper)?;
                }
                Ok(())
            }
            BooleanPredicateKind::AnyOf => {
                if members
                    .iter()
                    .any(|member| establish(member, format, lower, upper).is_ok())
                {
                    Ok(())
                } else {
                    Err(UnestablishedResultSet::NoDisjunctFinishedEstablished)
                }
            }
        },
    }
}

fn establish_absolute(
    tolerance: &ExactTolerance,
    format: &UlpFormat,
    upper: Option<&ExactRational>,
) -> Result<(), UnestablishedResultSet> {
    let Some(upper) = upper else {
        return Err(
            UnestablishedResultSet::AbsoluteBoundWithoutReferenceMagnitude {
                tolerance: tolerance.value().clone(),
            },
        );
    };
    // References above the largest finite value leave the *ordinary* branch of
    // step three entirely and are the finite-overflow contract's subject, so the
    // accuracy obligation is only measured up to that bound.
    let largest_finite = format.largest_finite();
    let bound = if *upper > largest_finite {
        largest_finite
    } else {
        upper.clone()
    };
    let required = format
        .ulp_scale(&bound)
        .unwrap_or_else(|_| unreachable!("the bound was clamped into the finite range"))
        .scale_by_power_of_two(-1);
    if *tolerance.value() < required {
        return Err(UnestablishedResultSet::AbsoluteBoundBelowSpacing {
            tolerance: tolerance.value().clone(),
            required,
        });
    }
    Ok(())
}

fn establish_relative(
    tolerance: &ExactTolerance,
    format: &UlpFormat,
    lower: Option<&ExactRational>,
) -> Result<(), UnestablishedResultSet> {
    let Some(lower) = lower.filter(|value| !value.is_zero()) else {
        return Err(
            UnestablishedResultSet::RelativeBoundWithoutReferenceMagnitude {
                tolerance: tolerance.value().clone(),
            },
        );
    };
    let precision = i32::try_from(format.precision()).expect("a bounded precision fits i32");
    // For a normal reference `ulp(r) <= 2^(1-p) * |r|`, so the worst rounding
    // ratio is `2^-p`. In the subnormal band the spacing is constant, so the
    // ratio grows as the reference shrinks and the smallest admitted magnitude
    // decides it.
    let normal_ratio = ExactRational::power_of_two(-precision);
    let least_normal = ExactRational::power_of_two(format.min_exponent());
    let required = if *lower >= least_normal {
        normal_ratio
    } else {
        let subnormal_half_gap = format.smallest_positive_finite().scale_by_power_of_two(-1);
        let subnormal_ratio = subnormal_half_gap
            .divide(lower)
            .unwrap_or_else(|_| unreachable!("a nonzero lower bound divides"));
        if subnormal_ratio > normal_ratio {
            subnormal_ratio
        } else {
            normal_ratio
        }
    };
    if *tolerance.value() < required {
        return Err(UnestablishedResultSet::RelativeBoundBelowRoundingRatio {
            tolerance: tolerance.value().clone(),
            required,
        });
    }
    Ok(())
}

/// Returns the `Ulp` predicate a correctly rounded result always satisfies.
///
/// The floor round-to-nearest attains, stated once here so the refinement
/// registry and the verifier cannot disagree about what "one half" means.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed metric key or the exact rational
/// one half violates its own grammar.
#[must_use]
pub fn correctly_rounded_ulp_bound() -> AccuracyPredicate {
    AccuracyPredicate::ulp(
        ulp_reference_gap_metric_key(),
        ExactTolerance::from_ratio(1, 2).expect("one half is a nonnegative exact tolerance"),
    )
}
