//! Checked semantic-occurrence to index-region refinement receipts.
//!
//! A verified index region proves structural safety, but does not by itself say
//! which semantic occurrence it realizes. This module owns the dependency-neutral
//! verifier that checks that association and mints an opaque receipt. Provider
//! selection, capability attribution, search, and explanation remain compiler
//! concerns layered above this receipt.
//!
//! A realization is an ordered [`VerifiedIndexRegionSequence`], not necessarily
//! one region: a family whose canonical form is a reduction feeding a pass over
//! the reduction's result is two regions with a value handed between them.
//! [`ResolvedIndexRealization::verify`] is the one-region spelling of
//! [`ResolvedIndexRealization::verify_sequence`], and a one-stage sequence's
//! identity is its region's identity, so nothing a single-region law ever minted
//! is changed by the sequence vocabulary's arrival.
//!
//! The public surface is a concrete alpha draft pending Tom's review. In
//! particular, callers cannot construct a receipt or its identity from bytes:
//! the verifier sees the complete semantic occurrence and the actual regions,
//! and [`ResolvedIndexRealization::complete`] independently discharges every
//! retained logical-index obligation before it mints a receipt.
//!
//! # What this file owns and where the rest lives
//!
//! This spine owns the governed vocabulary the whole module is stated in — the
//! identity domain separators and the ceilings every refusal names — because
//! they are read from several seams at once and because the non-prefixing
//! argument recorded on [`COVERAGE_GRAPH_DIGEST_DOMAIN`] ranges over the set,
//! not over one member of it. Distributing them would put that population in no
//! single place a reader can inspect.
//!
//! The seams below follow what each stage of refinement is answerable for.
//! [`subject`] derives the semantic side — the occurrence, its boundaries, its
//! signature, and the numerical contract it is stated under — and nothing there
//! has seen a region. [`authority`] admits one lowering family's exact
//! operation, signature, and scalar-emission ceiling. [`registry`] freezes the
//! semantic-provider-bound realization laws and resolves one against a subject.
//! [`verify`] is the checking pass itself, and [`binding`] holds the ordered
//! operand and result associations it derives. [`receipt`] holds what a
//! successful check mints and the pending association a residual obligation
//! leaves instead. [`proof`] holds the residual-domain proof vocabulary and
//! [`finite`] the closed exact-finite algorithm that discharges it, kept apart
//! because the vocabulary is what a receipt retains while the algorithm is one
//! authority's way of producing it. [`error`] holds the refusal enumeration they
//! share, and [`identity`] every canonical encoder, so that the bytes a receipt
//! is compared by are written in one file rather than beside the types they
//! summarize.
//!
//! [`VerifiedIndexRegionSequence`]: crate::index::VerifiedIndexRegionSequence

mod authority;
mod binding;
mod error;
mod finite;
mod identity;
mod proof;
mod receipt;
mod registry;
mod subject;
mod verify;

#[cfg(test)]
mod tests;

pub use authority::IndexRealizationAuthority;
pub use binding::{OperandBinding, ResultBinding};
pub use error::IndexRefinementVerificationError;
pub use proof::{
    IndexDomainDisproof, IndexDomainProofAssessment, IndexDomainProofAuthority,
    IndexDomainProofBudget, IndexDomainProofClaim, IndexDomainProofEvidence,
    IndexDomainProofRefusal, IndexDomainProofRefusalKind, IndexRefinementDomainProof,
    IndexRefinementDomainProofIdentity,
};
pub use receipt::{
    IndexRefinementExecutableCoverageIdentity, IndexRefinementReceipt,
    IndexRefinementReceiptIdentity, IndexRefinementVerificationOutcome,
    PendingIndexRefinementReceipt,
};
pub use registry::{
    FrozenIndexRealizationLawRegistry, IndexRealizationLawRegistryIdentity,
    ResolvedIndexRealization,
};
pub use subject::{
    IndexRefinementBoundary, IndexRefinementSignature, IndexRefinementSignatureSide,
    IndexRefinementSubject, NumericalContractIdentity,
};

// The module's own suite reads this vocabulary through `use super::*`, so the
// spine is where the names it takes from this module are stated. Each import
// stays private to this file and reaches the suite only because a child module
// may name what its ancestor imported.
#[cfg(test)]
use num_bigint::BigInt;
#[cfg(test)]
use num_integer::Integer;
#[cfg(test)]
use tiler_digest::{DIGEST_BYTES, DigestAlgorithm};

#[cfg(test)]
use crate::index::{
    FrozenScalarRegistry, IndexDomainPredicate, IndexDomainUnknownReason, IndexExtentRef,
    IndexInteger, MAX_BOUNDARY_TENSORS, ScalarOpKey, StagedInputSource, TensorRole,
    UnknownIndexDomainPredicate, VerifiedIndexRegion, VerifiedIndexRegionSequence,
};
#[cfg(test)]
use crate::program::SemanticOccurrence;
#[cfg(test)]
use crate::schedule::F32NumericalContractKey;
#[cfg(test)]
use crate::semantic::{
    FrozenSemanticRegistry, OperationAttributes, OperationEffect, ProviderIdentity, RegistryError,
    ResolvedValueType, SemanticProgram,
};
#[cfg(test)]
use crate::shape::Shape;

#[cfg(test)]
use binding::{bind_results, count_expanded_inputs, count_operand_bindings};
#[cfg(test)]
use finite::{
    IndexDomainGroup, IndexDomainKey, IndexDomainProofExhaustion, IndexDomainProofLedger,
    PlannedDomainObligation, ProofPlanningFailure, assess_domain_group, assess_finite_domains,
    checked_add, division_cost, encode_counterexample, finite_point_count, multiplication_cost,
    proof_resource_limit, resolve_extent,
};
#[cfg(test)]
use identity::{
    encode_executable_coverage_identity, encode_proof_identity, encode_subject_identity_with,
};
#[cfg(test)]
use receipt::mint_receipt;
#[cfg(test)]
use verify::{
    check_lowering_authority, check_residual_obligation_count, retain_complete_assessments,
};

const RECEIPT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-receipt.v1\0";
const STAGED_RECEIPT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-staged-receipt.v1\0";
const EXECUTABLE_COVERAGE_IDENTITY_TAG: &[u8] =
    b"tiler.ir.index-refinement-executable-coverage.v2\0";
const STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG: &[u8] =
    b"tiler.ir.index-refinement-staged-executable-coverage.v2\0";
/// Governed digest domain of the bound graph identity a coverage record folds.
///
/// [ADR 0104](../../../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md)
/// replaced the framed `SemanticGraphIdentity` preimage at the head of every
/// coverage record with a fixed-width digest under this domain, which is why
/// both tags above step to `v2`: the record's grammar changed at its first
/// field, so a reader following the `v1` grammar would take the digest's leading
/// eight bytes for the graph identity's length prefix and frame everything after
/// it wrongly. No such reader exists — the type has no decoder — but the step is
/// what keeps that from ever being a question a later one has to answer.
///
/// **It is a separate domain rather than a reuse of either coverage tag** because
/// those two are *encoding* separators — the first bytes of a canonical run —
/// while this one is a *digest* separator, the first bytes of a pre-image. The
/// two kinds never have to be distinguished from each other, but two digests do:
/// this is the only subject this crate hashes, and any later one must be
/// checkably non-prefixing against it.
///
/// The no-prefix obligation `docs/artifact-abi.md` records normatively spans
/// every domain the workspace admits, because one algorithm hashes them all in
/// one process. No crate owns a check over that union: `tiler-ir` cannot see
/// `tiler-artifact`'s domains, while `tiler-artifact` could enumerate the union
/// through its dependency on this crate if this crate exported its complete,
/// private test-only pin population. `tiler-digest` deliberately owns no
/// subject domains. The accepted artifact ABI contract instead establishes the
/// cross-crate property from spellings and terminators: every terminated
/// spelling has its sole NUL at the end, every unterminated IR spelling has no
/// NUL, and the observed populations have no exact equality. This crate's
/// private pin population is inspected for that argument;
/// `tiler_artifact::domains::no_governed_domain_of_this_crate_prefixes_another`
/// checks that crate's own admitted set rather than leaving its half to prose.
///
/// **The other set is sized by its type and not by a number here.**
/// `GovernedDomain` in `crates/tiler-artifact/src/domains.rs` is that
/// population: it declares its list length as `variant_count` and matches
/// wildcard-free, so a domain admitted there without being enumerated is a
/// build error at the list rather than a paragraph here that has quietly
/// stopped covering its subject. Where this prose and that enum disagree, the
/// enum settles it. The path names the type rather than resolving as a link,
/// because the module is crate-private and this crate does not depend on that
/// one.
///
/// *Substituted 2026-08-08 by
/// [`correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix`](../../../../../tickets/correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix.md),
/// and substituted rather than dated beside because the retired claim was never
/// true at any commit* — the practice these records follow keeps a dated
/// correction beside a claim that was true when written, and substitutes one
/// that never was. The retired wording read "this domain opens `tiler.ir.` and
/// all eight of `tiler-artifact`'s open `tiler.artifact-`". `git log -S` places
/// its authoring at `d48a33af`, whose tree already declared eighteen governed
/// domains in that crate and already spelled `ROUTE_REQUIREMENT_DOMAIN`
/// `tiler.artifact.route-requirement.v1`, separating with a `.` and not a `-`.
/// So the count was wrong on the commit that introduced it, and the hyphen was
/// the error that mattered: a quantifier over `tiler.artifact-` never reaches
/// the route-requirement domain, so the sentence did not range over the set
/// whose disjointness it asserted. The conclusion held regardless, but its
/// stated reasoning did not establish it. **Quoting the retired wording keeps
/// it greppable**, so a later hit on `tiler.artifact-` in this file is evidence
/// that the string is present, not that the claim stands.
const COVERAGE_GRAPH_DIGEST_DOMAIN: &[u8] = b"tiler.ir.index-refinement-coverage-graph.v1\0";
const SUBJECT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-subject.v2\0";
#[cfg(test)]
const LEGACY_SUBJECT_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-subject.v1\0";
const AUTHORITY_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-authority.v1\0";
const RESOLUTION_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-resolution.v1\0";
const PROOF_IDENTITY_TAG: &[u8] = b"tiler.ir.index-refinement-domain-proof.v1\0";
const LAW_REGISTRY_IDENTITY_TAG: &[u8] = b"tiler.ir.index-realization-law-registry.v1\0";
const MAX_NUMERICAL_CONTRACT_IDENTITY_BYTES: usize = 256;
const MAX_DOMAIN_EVIDENCE_BYTES: usize = 4_096;
/// Maximum operands or results admitted on one refinement signature side.
pub const MAX_INDEX_REFINEMENT_SIGNATURE_VALUES: usize = 4_096;
/// Maximum operand-use bindings retained by one refinement receipt.
///
/// A binding associates one semantic operand use with one verified region input
/// boundary *in one stage*, so the retained population is the product of three
/// independent multiplicities: the occurrence's operand uses, the component
/// expansion of each used input, and the number of stages reading the resulting
/// boundary. `bind_operands` pushes one binding per (operand use, expanded
/// component, reading stage) triple; one boundary read by several stages is the
/// staged vocabulary's motivating case — the value a fold reads and the pass
/// consuming that fold reads again.
///
/// The independent name is therefore load-bearing rather than a convenience.
/// None of the three multiplicities bounds the others, so this is an
/// **independent** ceiling that `count_operand_bindings` enforces before the
/// binding vector is allocated, not a consequence of the boundary ceiling. A
/// binding inventory can exceed the distinct expanded-input population, and
/// [`IndexRefinementVerificationError::OperandBindingsTooLarge`] is the refusal
/// naming it when it does. Sharing [`super::MAX_BOUNDARY_TENSORS`]'s value
/// keeps one number for a reader to hold; it does not make one bound imply the
/// other.
///
/// `operand_binding_population_is_bounded_before_collection` fixes both halves:
/// sixteen aliased uses of one 1,024-component input exactly fill this limit
/// while the distinct expanded population stays 1,024, and a seventeenth
/// crosses it.
pub const MAX_INDEX_REFINEMENT_OPERAND_BINDINGS: usize = super::MAX_BOUNDARY_TENSORS;
/// Maximum raw scalar-operation declarations admitted by one authority.
pub const MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS: usize = 4_096;
/// Maximum residual obligations one canonical realization may retain, summed
/// over its stages.
///
/// The closed law vocabulary's widest single-region template emits three
/// rank-wide accesses, each with at most [`super::MAX_TENSOR_RANK`] coordinates
/// and two predicates per coordinate; rank-zero component reads retain no
/// coordinate obligations. Its widest staged template is a two-access fold
/// followed by a three-access pointwise pass, so five accesses is the widest any
/// realization of this vocabulary reaches, and the six below is a margin over
/// that rather than a tight bound. The bound is over the realization because
/// that is what one caller funds one completion budget for.
pub const MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS: usize = 6 * super::MAX_TENSOR_RANK * 2;
/// Maximum cells the closed exact-finite residual proof algorithm may evaluate.
pub const MAX_FINITE_DOMAIN_PROOF_CELLS: u64 = 16 * 1024 * 1024;
/// Maximum cumulative arbitrary-precision integer bytes the closed residual
/// proof algorithm may process.
pub const MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES: u64 = 64 * 1024 * 1024;
const EXHAUSTIVE_DERIVATION: &[u8] = b"tiler.ir.exact-index-domain-enumeration.v1\0";
const COUNTEREXAMPLE_TAG: &[u8] = b"tiler.ir.index-domain-counterexample.v1\0";
