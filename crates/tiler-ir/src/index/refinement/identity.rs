//! Every canonical encoder refinement mints identity with.
//!
//! One file, because the bytes are the contract: a domain separator, a field
//! order, and a framing rule that keeps each run self-delimiting. Keeping the
//! encoders together is what lets a reader see the whole grammar at once —
//! which fields a record restates, which it delegates to a bound identity, and
//! where a staged realization writes under its own domain so that no staged
//! preimage can spell a single-region one. Splitting them beside the types they
//! summarize would leave the population no single place to be read.

use tiler_digest::DigestAlgorithm;

use crate::identity::{push_len, push_slice};
use crate::index::{
    CanonicalScalarDefinitionProjection, ScalarAuthorityEvidence, UnknownIndexDomainPredicate,
    VerifiedIndexRegion, VerifiedIndexRegionSequence,
};
use crate::program::SemanticOccurrence;
use crate::semantic::{OpKey, OperationEffect, ProviderIdentity, SemanticCapabilityAuthority};
use crate::shape::Shape;

use super::binding::{OperandBinding, ResultBinding};
use super::proof::{
    IndexDomainProofAuthority, IndexDomainProofEvidence, IndexRefinementDomainProof,
};
use super::registry::ResolvedIndexRealization;
use super::subject::{IndexRefinementBoundary, IndexRefinementSignature, IndexRefinementSubject};
use super::{
    AUTHORITY_IDENTITY_TAG, COVERAGE_GRAPH_DIGEST_DOMAIN, EXECUTABLE_COVERAGE_IDENTITY_TAG,
    PROOF_IDENTITY_TAG, RECEIPT_IDENTITY_TAG, RESOLUTION_IDENTITY_TAG,
    STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG, STAGED_RECEIPT_IDENTITY_TAG, SUBJECT_IDENTITY_TAG,
};

/// Encodes reached-only executable provenance.
///
/// **Why a one-stage realization writes the bytes it always wrote.** A
/// realization spanning several stages carries more than one region identity,
/// more than one scalar authority, and a stage ordinal on every binding — none of
/// which a one-stage realization has anything to say about. Rather than write
/// empty or constant fields into every receipt ever minted, the one-stage form
/// keeps its established encoding and the staged form is written under its own
/// domain tag. The two tags are distinct byte strings in the first position, so
/// the preimages are disjoint and no staged coverage can spell a single-region
/// one.
///
/// **Why the graph is a digest and not the identity itself.** One whole
/// `SemanticGraphIdentity` used to open every record, and there is one record per
/// semantic operation, so the product of a linear encoding with a linear count
/// made kernel-program identity quadratic in operation count — measured at
/// `134n² + 3650n + 727` bytes, whose quadratic coefficient *is* the graph
/// encoding's per-operation slope. [ADR 0104] folds it to
/// [`DIGEST_BYTES`] under [`COVERAGE_GRAPH_DIGEST_DOMAIN`], which makes the
/// curve linear.
///
/// It is written unframed because it is fixed width: a length prefix exists to
/// make a variable-length run self-delimiting, and thirty-two bytes that are
/// always thirty-two bytes are already that. The record therefore says exactly
/// what it said before — "this occurrence of *this* graph" — and still refuses
/// two records naming one occurrence ordinal in different graphs, which is the
/// injectivity the pair carries and the reason the graph could not simply be
/// dropped. What it stops doing is carrying bytes the graph identity could be
/// reconstructed from, which nothing in the workspace does: the type has no
/// decoder, no field accessors, and two `compile_fail` doctests holding that it
/// has no byte constructor.
///
/// [ADR 0104]: ../../../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md
/// [`DIGEST_BYTES`]: tiler_digest::DIGEST_BYTES
pub(super) fn encode_executable_coverage_identity(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
    realization: &VerifiedIndexRegionSequence,
    scalar_authorities: &[ScalarAuthorityEvidence],
    operand_bindings: &[OperandBinding],
    result_bindings: &[ResultBinding],
    proofs: &[IndexRefinementDomainProof],
) -> Vec<u8> {
    let staged = !realization.is_single_stage();
    let mut bytes = if staged {
        STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG.to_vec()
    } else {
        EXECUTABLE_COVERAGE_IDENTITY_TAG.to_vec()
    };
    bytes.extend_from_slice(
        DigestAlgorithm::GOVERNED
            .digest(COVERAGE_GRAPH_DIGEST_DOMAIN, subject.graph.as_bytes())
            .as_bytes(),
    );
    bytes.extend_from_slice(&subject.occurrence.get().to_be_bytes());
    push_slice(&mut bytes, subject.numerical_contract.as_bytes());
    if staged {
        push_slice(&mut bytes, realization.identity().as_bytes());
    } else {
        push_slice(
            &mut bytes,
            realization.final_stage().canonical_identity().as_bytes(),
        );
    }
    push_slice(
        &mut bytes,
        subject.semantic_authority.reached_definitions().as_bytes(),
    );
    push_slice(
        &mut bytes,
        subject.semantic_authority.admission_provenance().as_bytes(),
    );
    encode_optional_law_row(&mut bytes, subject.realization_law_row.as_deref());
    encode_provider(&mut bytes, resolution.provider());
    bytes.extend_from_slice(&resolution.revision().to_be_bytes());
    if staged {
        push_len(&mut bytes, scalar_authorities.len());
    }
    for authority in scalar_authorities {
        push_slice(&mut bytes, authority.definitions().as_bytes());
        push_slice(&mut bytes, authority.admission().as_bytes());
        push_slice(&mut bytes, authority.type_definitions().as_bytes());
        push_slice(&mut bytes, authority.type_admission().as_bytes());
    }
    push_len(&mut bytes, operand_bindings.len());
    for binding in operand_bindings {
        if staged {
            push_len(&mut bytes, binding.stage);
        }
        push_len(&mut bytes, binding.operand);
        push_len(&mut bytes, binding.input);
        bytes.extend_from_slice(&binding.input_tensor.index.to_be_bytes());
        match binding.component_role {
            None => bytes.push(0),
            Some(role) => {
                bytes.push(1);
                bytes.extend_from_slice(&role.get().to_be_bytes());
            }
        }
    }
    // One record per output root, so a partitioned result writes one record per
    // member and its grouping is recoverable from the repeated result ordinal.
    // The count is the record count rather than the result count, which is what
    // keeps the run self-delimiting without a second nested length — and what
    // makes a result owning one root write exactly the bytes it always wrote.
    push_len(&mut bytes, result_bindings.len());
    for binding in result_bindings {
        push_len(&mut bytes, binding.result);
        bytes.extend_from_slice(&binding.output_tensor.index.to_be_bytes());
        bytes.extend_from_slice(&binding.write_access.index.to_be_bytes());
        bytes.extend_from_slice(&binding.written_value.index.to_be_bytes());
    }
    push_len(&mut bytes, proofs.len());
    for proof in proofs {
        if staged {
            push_len(&mut bytes, proof.stage);
        }
        push_slice(&mut bytes, proof.identity().as_bytes());
    }
    bytes
}

/// Encodes the canonical receipt identity.
///
/// Domain-separated the same way [`encode_executable_coverage_identity`] is, and
/// for the same reason: a one-stage realization keeps
/// [`RECEIPT_IDENTITY_TAG`] and the exact field order it has always written, so
/// every receipt a single-region law ever minted is byte-identical; a staged
/// realization writes its whole ordered chain under
/// [`STAGED_RECEIPT_IDENTITY_TAG`].
pub(super) fn encode_receipt_identity(
    subject: &IndexRefinementSubject,
    resolution: &ResolvedIndexRealization,
    realization: &VerifiedIndexRegionSequence,
    scalar_authorities: &[ScalarAuthorityEvidence],
    proofs: &[IndexRefinementDomainProof],
) -> Vec<u8> {
    let staged = !realization.is_single_stage();
    let mut bytes = if staged {
        STAGED_RECEIPT_IDENTITY_TAG.to_vec()
    } else {
        RECEIPT_IDENTITY_TAG.to_vec()
    };
    if staged {
        push_slice(&mut bytes, realization.identity().as_bytes());
    } else {
        push_slice(
            &mut bytes,
            realization.final_stage().canonical_identity().as_bytes(),
        );
    }
    push_slice(&mut bytes, &subject.identity);
    push_slice(&mut bytes, &resolution.identity);
    if staged {
        push_len(&mut bytes, scalar_authorities.len());
    }
    for authority in scalar_authorities {
        push_slice(&mut bytes, authority.definitions().as_bytes());
        push_slice(&mut bytes, authority.type_definitions().as_bytes());
        push_slice(&mut bytes, authority.semantic_snapshot().as_bytes());
        push_slice(&mut bytes, authority.scalar_snapshot().as_bytes());
    }
    push_len(&mut bytes, proofs.len());
    for proof in proofs {
        if staged {
            push_len(&mut bytes, proof.stage);
        }
        push_slice(&mut bytes, proof.identity().as_bytes());
    }
    bytes
}

pub(super) fn encode_subject_identity(subject: &IndexRefinementSubject) -> Vec<u8> {
    encode_subject_identity_with(subject, SUBJECT_IDENTITY_TAG, subject.occurrence)
}

pub(super) fn encode_subject_identity_with(
    subject: &IndexRefinementSubject,
    domain: &[u8],
    occurrence: SemanticOccurrence,
) -> Vec<u8> {
    let mut bytes = domain.to_vec();
    push_slice(&mut bytes, subject.graph.as_bytes());
    bytes.extend_from_slice(&occurrence.get().to_be_bytes());
    encode_op_key(&mut bytes, &subject.operation);
    encode_signature(&mut bytes, &subject.signature);
    push_len(&mut bytes, subject.inputs.len());
    for input in &subject.inputs {
        push_slice(&mut bytes, input.value_type.canonical_encoding().as_bytes());
        encode_boundary_shape(&mut bytes, input);
    }
    push_len(&mut bytes, subject.operands.len());
    for input in &subject.operands {
        bytes.extend_from_slice(&(*input as u64).to_be_bytes());
    }
    push_len(&mut bytes, subject.results.len());
    for result in &subject.results {
        push_slice(
            &mut bytes,
            result.value_type.canonical_encoding().as_bytes(),
        );
        encode_boundary_shape(&mut bytes, result);
    }
    bytes.push(match subject.effect {
        OperationEffect::Pure => 1,
    });
    push_slice(
        &mut bytes,
        subject.attributes.canonical_encoding().as_bytes(),
    );
    push_slice(&mut bytes, subject.numerical_contract.as_bytes());
    push_slice(
        &mut bytes,
        subject.semantic_authority.reached_definitions().as_bytes(),
    );
    push_slice(
        &mut bytes,
        subject.semantic_authority.admission_provenance().as_bytes(),
    );
    push_slice(
        &mut bytes,
        subject.semantic_authority.registry_snapshot().as_bytes(),
    );
    encode_optional_law_row(&mut bytes, subject.realization_law_row.as_deref());
    bytes
}

pub(super) fn encode_authority_identity(
    operation: &OpKey,
    signature: &IndexRefinementSignature,
    semantic: &SemanticCapabilityAuthority,
    scalar: &CanonicalScalarDefinitionProjection,
    scalar_snapshot: &[u8],
    realization_law_row: Option<&[u8]>,
) -> Vec<u8> {
    let mut bytes = AUTHORITY_IDENTITY_TAG.to_vec();
    encode_op_key(&mut bytes, operation);
    encode_signature(&mut bytes, signature);
    push_slice(&mut bytes, semantic.reached_definitions().as_bytes());
    push_slice(&mut bytes, semantic.admission_provenance().as_bytes());
    push_slice(&mut bytes, semantic.registry_snapshot().as_bytes());
    push_slice(&mut bytes, scalar.as_bytes());
    push_slice(&mut bytes, scalar_snapshot);
    encode_optional_law_row(&mut bytes, realization_law_row);
    bytes
}

fn encode_optional_law_row(output: &mut Vec<u8>, row: Option<&[u8]>) {
    match row {
        None => output.push(0),
        Some(row) => {
            output.push(1);
            push_slice(output, row);
        }
    }
}

pub(super) fn encode_resolution_identity(authority: &[u8], subject: &[u8]) -> Vec<u8> {
    let mut bytes = RESOLUTION_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, authority);
    push_slice(&mut bytes, subject);
    bytes
}

fn encode_signature(output: &mut Vec<u8>, signature: &IndexRefinementSignature) {
    push_len(output, signature.operands.len());
    for ty in &signature.operands {
        push_slice(output, ty.canonical_encoding().as_bytes());
    }
    push_len(output, signature.results.len());
    for ty in &signature.results {
        push_slice(output, ty.canonical_encoding().as_bytes());
    }
}

pub(super) fn encode_proof_identity(
    region: &VerifiedIndexRegion,
    obligation: UnknownIndexDomainPredicate,
    authority: &IndexDomainProofAuthority,
    proof: &IndexDomainProofEvidence,
) -> Vec<u8> {
    let mut bytes = PROOF_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, region.canonical_identity().as_bytes());
    push_slice(&mut bytes, obligation.canonical_local_key().as_bytes());
    encode_provider(&mut bytes, authority.provider());
    encode_provider(&mut bytes, authority.rule());
    bytes.extend_from_slice(&authority.revision().to_be_bytes());
    match proof {
        IndexDomainProofEvidence::ExhaustiveFinite { points, derivation } => {
            bytes.push(2);
            bytes.extend_from_slice(&points.to_be_bytes());
            push_slice(&mut bytes, derivation);
        }
    }
    bytes
}

pub(super) fn encode_provider(output: &mut Vec<u8>, provider: &ProviderIdentity) {
    push_slice(output, provider.namespace().as_bytes());
    push_slice(output, provider.name().as_bytes());
    output.extend_from_slice(&provider.revision().to_be_bytes());
}

pub(super) fn encode_op_key(output: &mut Vec<u8>, key: &OpKey) {
    push_slice(output, key.namespace().as_bytes());
    push_slice(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}

fn encode_shape(output: &mut Vec<u8>, shape: &Shape) {
    push_len(output, shape.rank());
    for extent in shape.extents() {
        output.extend_from_slice(&extent.get().to_be_bytes());
    }
}

fn encode_boundary_shape(output: &mut Vec<u8>, boundary: &IndexRefinementBoundary) {
    if boundary.sourced.as_static().is_some() {
        encode_shape(output, &boundary.shape);
        return;
    }
    boundary.sourced.encode(output);
}
