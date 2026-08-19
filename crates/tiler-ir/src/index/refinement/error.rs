//! The one refusal vocabulary refinement answers in.
//!
//! Every check in this module — subject derivation, authority admission, law
//! resolution, interface binding, and completion budgeting — reports through
//! this enumeration, so a caller matches one type rather than one per stage.
//! The variants are deliberately specific about *what* disagreed: an arity
//! disagreement, a boundary disagreement, and a whole-realization disagreement
//! are three different refusals because they send a reader to three different
//! places.

use core::fmt;
use std::error::Error;
use std::sync::Arc;

use crate::index::{
    CanonicalIndexRegionIdentity, CanonicalIndexRegionSequenceIdentity, ProofResource,
    ScalarRegistryError, VerifiedIndexHandleError,
};
use crate::schedule::NumericalContractKeyError;
use crate::semantic::{OpKey, OperationEffect, RegistryError};

use super::subject::IndexRefinementSignatureSide;

/// Why IR-owned refinement verification refused to mint a receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IndexRefinementVerificationError {
    /// A numerical-contract key failed the IR-owned canonical grammar.
    InvalidNumericalContractIdentity {
        /// Exact canonical-key validation failure.
        source: NumericalContractKeyError,
    },
    /// A residual-domain reason, derivation, or counterexample was invalid.
    InvalidDomainProofEvidence,
    /// Exact-finite proof budget is zero or exceeds IR's hard bound.
    InvalidDomainProofBudget {
        /// Completion-stage resource whose limit was invalid.
        resource: ProofResource,
        /// Supplied limit.
        actual: u64,
        /// Maximum admitted limit.
        limit: u64,
    },
    /// A canonical region retained more obligations than the closed law
    /// vocabulary can produce.
    ResidualObligationsTooLarge {
        /// Exact retained count.
        actual: usize,
        /// IR-owned structural maximum.
        limit: usize,
    },
    /// An admitted signature exceeded the governed bound.
    SignatureTooLarge {
        /// Ordered signature side that exceeded the bound.
        side: IndexRefinementSignatureSide,
        /// Bounded count observed before refusing the iterator.
        actual: usize,
        /// Maximum admitted count.
        limit: usize,
    },
    /// A raw emitted-scalar declaration exceeded the governed bound.
    EmittedScalarOperationsTooLarge {
        /// Supplied raw declaration count before deduplication.
        actual: usize,
        /// Maximum admitted raw declaration count.
        limit: usize,
    },
    /// A completed receipt did not belong to the pending association supplied
    /// to the consumer.
    CompletionReceiptMismatch,
    /// A verified semantic handle failed to resolve.
    SemanticHandle(crate::semantic::HandleError),
    /// A covered semantic boundary's extent names a declared `ShapeEnv` symbol.
    ///
    /// A refinement subject fixes the exact boundaries an independently
    /// verified realization must reproduce, and it is compared byte for byte
    /// against a candidate region's canonical identity. A symbolic boundary is
    /// refused rather than resolved through the environment: resolving it would
    /// make the subject name a value nobody wrote, which is the collapse of
    /// graph identity into specialized identity the sourced vocabulary exists
    /// to prevent.
    SymbolicSemanticBoundary,
    /// The semantic operation definition disappeared from its frozen authority.
    OperationDefinitionMissing,
    /// Semantic authority projection failed.
    SemanticAuthority(Arc<RegistryError>),
    /// Resolution and subject name different operations.
    OperationMismatch,
    /// Resolution and subject name different attributes.
    AttributeMismatch,
    /// Resolution and subject name different numerical contracts.
    NumericalContractMismatch,
    /// Resolution and subject name different semantic graphs or ordinals.
    OccurrenceMismatch,
    /// Resolution and subject signatures disagree.
    CapabilitySignatureMismatch,
    /// Semantic registry authority disagrees.
    SemanticAuthorityMismatch,
    /// The scalar registry was built over another semantic authority.
    ScalarSemanticAuthorityMismatch,
    /// The program-derived subject came from another semantic authority.
    SubjectSemanticAuthorityMismatch,
    /// The operation-specific semantic realization-law row (including its
    /// absence) differs from the row bound into the program-derived subject.
    SubjectRealizationLawMismatch,
    /// The lowering or region came from another scalar-registry snapshot.
    ScalarSnapshotMismatch,
    /// No independent verifier governs the operation/signature/contract subject.
    MissingRealizationLaw,
    /// The semantic law does not govern this numerical-contract domain.
    NumericalContractNotGoverned,
    /// The semantic law could not construct its expected canonical region.
    SemanticRealizationLawRefused {
        /// Operation whose registered law refused.
        operation: Box<OpKey>,
        /// Stable failing law rule.
        rule: &'static str,
    },
    /// The candidate's exact canonical identity differs from the semantic law's
    /// expected canonical region; semantic equivalence alone is insufficient.
    SemanticRealizationMismatch {
        /// Canonical region required by semantic authority.
        expected: CanonicalIndexRegionIdentity,
        /// Canonical region emitted by the selected lowering.
        actual: CanonicalIndexRegionIdentity,
    },
    /// The candidate's ordered realization differs from the semantic law's
    /// expected region sequence.
    ///
    /// Distinct from [`Self::SemanticRealizationMismatch`], which names two
    /// regions: a chain can disagree in its stage count, in a stage's own
    /// region, or only in how a stage's inputs are sourced, and the sequence
    /// identity is what covers all three.
    SemanticRealizationSequenceMismatch {
        /// Canonical realization required by semantic authority.
        expected: CanonicalIndexRegionSequenceIdentity,
        /// Canonical realization emitted by the selected lowering.
        actual: CanonicalIndexRegionSequenceIdentity,
    },
    /// The region reached scalar operations outside admission.
    ScalarAuthorityConformance,
    /// The occurrence effect is not realizable by this pure index profile.
    EffectNotIndexable {
        /// The rejected effect class.
        effect: OperationEffect,
    },
    /// Scalar authority rejected the region.
    ScalarAuthority(Arc<ScalarRegistryError>),
    /// A verified region handle could not be resolved.
    Handle(VerifiedIndexHandleError),
    /// A boundary exposes no static shape in this bounded verifier.
    SymbolicBoundary,
    /// An encoded semantic input declared no component boundary to bind.
    EmptyEncodedOperandComponents {
        /// Position in the distinct semantic input population.
        input: usize,
    },
    /// Region input count disagrees with the expanded semantic input boundary.
    OperandArity {
        /// Number of verified input boundaries.
        region_inputs: usize,
        /// Expected ordinary inputs plus ordered encoded components, saturated
        /// at `usize::MAX` if count arithmetic overflowed.
        expanded_inputs: usize,
    },
    /// One region input disagrees with its expanded semantic input boundary.
    OperandInterface {
        /// Position in the ordered expanded semantic input boundaries.
        position: usize,
    },
    /// Alias and component expansion exceeded the receipt binding population.
    OperandBindingsTooLarge {
        /// Binding count, saturated at `usize::MAX` on arithmetic overflow.
        actual: usize,
        /// Maximum operand bindings retained by one receipt.
        limit: usize,
    },
    /// Region output count disagrees with semantic results.
    ResultArity {
        /// Number of distinct verified output tensors the region's roots write.
        ///
        /// Distinct *tensors* rather than roots, because a partitioned output is
        /// several roots over one of them and answers one semantic result.
        region_outputs: usize,
        /// Number of semantic results.
        results: usize,
    },
    /// One region output disagrees with its semantic result.
    ResultInterface {
        /// Position of the disagreeing output.
        position: usize,
    },
    /// One output writes a scalar value of the wrong type.
    ResultValueType {
        /// Position of the mistyped output.
        position: usize,
    },
    /// One output lacks complete unique-write evidence.
    IncompleteWrite {
        /// Position of the output without complete ownership.
        position: usize,
    },
}

impl fmt::Display for IndexRefinementVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumericalContractIdentity { source } => {
                write!(
                    formatter,
                    "numerical-contract identity is invalid: {source}"
                )
            }
            Self::InvalidDomainProofEvidence => {
                formatter.write_str("domain-proof evidence is empty or exceeds its byte bound")
            }
            Self::InvalidDomainProofBudget {
                resource,
                actual,
                limit,
            } => write!(
                formatter,
                "domain-proof {resource:?} budget is {actual}; expected 1..={limit}"
            ),
            Self::ResidualObligationsTooLarge { actual, limit } => write!(
                formatter,
                "canonical realization retained {actual} residual obligations; limit is {limit}"
            ),
            Self::SignatureTooLarge {
                side,
                actual,
                limit,
            } => write!(
                formatter,
                "refinement signature {side:?} side reached {actual} values; limit is {limit}"
            ),
            Self::EmittedScalarOperationsTooLarge { actual, limit } => write!(
                formatter,
                "emitted scalar-operation declaration has {actual} entries; limit is {limit}"
            ),
            Self::CompletionReceiptMismatch => formatter
                .write_str("completed receipt does not match its pending refinement association"),
            Self::SemanticHandle(source) => write!(formatter, "semantic handle failed: {source}"),
            Self::SymbolicSemanticBoundary => formatter
                .write_str("a covered semantic boundary names a declared shape-environment symbol"),
            Self::OperationDefinitionMissing => {
                formatter.write_str("semantic operation definition is absent")
            }
            Self::SemanticAuthority(source) => {
                write!(formatter, "semantic authority failed: {source}")
            }
            Self::OperationMismatch => {
                formatter.write_str("resolved authority names another operation")
            }
            Self::AttributeMismatch => {
                formatter.write_str("resolved authority names other attributes")
            }
            Self::NumericalContractMismatch => {
                formatter.write_str("resolved authority names another numerical contract")
            }
            Self::OccurrenceMismatch => {
                formatter.write_str("resolved authority names another graph occurrence")
            }
            Self::CapabilitySignatureMismatch => {
                formatter.write_str("resolved authority names another signature")
            }
            Self::SemanticAuthorityMismatch => {
                formatter.write_str("semantic registry authority disagrees")
            }
            Self::ScalarSemanticAuthorityMismatch => {
                formatter.write_str("scalar registry was built over another semantic authority")
            }
            Self::SubjectSemanticAuthorityMismatch => {
                formatter.write_str("program subject came from another semantic authority")
            }
            Self::SubjectRealizationLawMismatch => formatter
                .write_str("program subject came from another operation-specific realization law"),
            Self::ScalarSnapshotMismatch => {
                formatter.write_str("scalar-registry snapshot disagrees with admission")
            }
            Self::MissingRealizationLaw => {
                formatter.write_str("no semantic-realization law is registered")
            }
            Self::NumericalContractNotGoverned => formatter
                .write_str("the semantic realization law does not govern this numerical contract"),
            Self::SemanticRealizationLawRefused { operation, rule } => {
                write!(
                    formatter,
                    "semantic realization law for {operation} refused at {rule}"
                )
            }
            Self::SemanticRealizationMismatch { expected, actual } => write!(
                formatter,
                "candidate region {:?} differs from semantic law region {:?}",
                actual.as_bytes(),
                expected.as_bytes()
            ),
            Self::SemanticRealizationSequenceMismatch { expected, actual } => write!(
                formatter,
                "candidate realization {:?} differs from semantic law realization {:?}",
                actual.as_bytes(),
                expected.as_bytes()
            ),
            Self::ScalarAuthorityConformance => {
                formatter.write_str("region reached scalar authority outside admission")
            }
            Self::EffectNotIndexable { effect } => {
                write!(formatter, "occurrence effect {effect:?} is not indexable")
            }
            Self::ScalarAuthority(source) => write!(formatter, "scalar authority failed: {source}"),
            Self::Handle(source) => write!(formatter, "verified handle failed: {source}"),
            Self::SymbolicBoundary => formatter.write_str("a boundary exposed no static shape"),
            Self::EmptyEncodedOperandComponents { input } => write!(
                formatter,
                "encoded semantic input {input} declares no component boundaries"
            ),
            Self::OperandArity {
                region_inputs,
                expanded_inputs,
            } => write!(
                formatter,
                "region declares {region_inputs} inputs for {expanded_inputs} expanded semantic input boundaries"
            ),
            Self::OperandInterface { position } => {
                write!(
                    formatter,
                    "region input {position} does not match its expanded semantic input boundary"
                )
            }
            Self::OperandBindingsTooLarge { actual, limit } => write!(
                formatter,
                "expanded operand bindings {actual} exceed receipt limit {limit}"
            ),
            Self::ResultArity {
                region_outputs,
                results,
            } => write!(
                formatter,
                "region produces {region_outputs} distinct output tensors for {results} results"
            ),
            Self::ResultInterface { position } => {
                write!(
                    formatter,
                    "result {position} does not match its output tensor"
                )
            }
            Self::ResultValueType { position } => {
                write!(
                    formatter,
                    "result {position} has a root that writes the wrong result type"
                )
            }
            Self::IncompleteWrite { position } => write!(
                formatter,
                "result {position} has a root lacking write-ownership evidence"
            ),
        }
    }
}

impl Error for IndexRefinementVerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ScalarAuthority(source) => Some(source.as_ref()),
            Self::InvalidNumericalContractIdentity { source } => Some(source),
            Self::SemanticAuthority(source) => Some(source.as_ref()),
            Self::SemanticHandle(source) => Some(source),
            Self::Handle(source) => Some(source),
            _ => None,
        }
    }
}

impl From<VerifiedIndexHandleError> for IndexRefinementVerificationError {
    fn from(source: VerifiedIndexHandleError) -> Self {
        Self::Handle(source)
    }
}
