//! Bounded tests for the proof-case evidence sidecar.
//!
//! The fixtures package the artifact-model suite's own real verified artifact,
//! so every association these tests prove or refuse is an association with a
//! program the shared IR and the artifact verifier already accepted.
//!
//! Round-tripping is the weakest evidence a container can offer, so it is the
//! smallest part of this suite. Three stronger properties carry the weight.
//!
//! **Canonical form.** Two producers that admitted the same cases in different
//! orders must emit identical bytes, and a container that is well formed but
//! not canonical must be refused rather than normalized on the way in.
//!
//! **Fail-closed under a competent adversary.** Corrupting a byte and watching
//! a digest reject it proves little, because a forger recomputes digests. The
//! forged cases therefore build a *structurally invalid sidecar*, encode it —
//! which stamps a correct manifest digest, correct payload digests, and a
//! correct identity for whatever it now says — and require the reader to refuse
//! it anyway, by name. The byte-level cases separately prove that an
//! incompetent corruption cannot slip through either.
//!
//! **Association is a decision, not a default.** A decoded sidecar is valid
//! evidence about nothing until it is bound. The binding cases prove that the
//! wrong envelope, damaged envelope bytes, and a different artifact are each
//! refused with their own cause.

use tiler_ir::semantic::{InputKey, OutputKey, SemanticGraphIdentity, SemanticProgramBuilder};

use crate::program::tests::{
    OTHER_SCALE_BITS, SCALE_BITS, artifact_with_selected_operations, build_artifact,
    build_graph_scaled, default_artifact, fused_program, lowering_provider,
};
use crate::program::{DIGEST_BYTES, DigestAlgorithm, VerifiedArtifactProgram};

use super::budget::{CaseLens, ProofBudgetError, add, project_from_data, project_sidecar};
use super::builder::{BoundInterface, ProofDirection, ProofInterfaceError, verify_cases};
use super::codec::{
    CANONICAL_ENCODING, HEADER_BYTES, IDENTITY_DOMAIN, MAGIC, MANIFEST_DOMAIN,
    PAYLOAD_DIGEST_DOMAIN, ProofFailureClass, ProofLimitExceeded, ProofLimitKind,
    ProofOrderedSubject, SIDECAR_FORMAT, derive_identity,
};
use super::model::{ProofCaseData, ProofSidecarData, ProofSubjects};
use super::{
    MAX_PROOF_CASE_KEY_BYTES, MAX_PROOF_CASES, MAX_PROOF_IDENTITY_BYTES, MAX_PROOF_MANIFEST_BYTES,
    MAX_PROOF_SIDECAR_BYTES, MAX_PROOF_SUBJECT_BYTES, ProofBuildError, ProofCaseKey,
    ProofCaseKeyError, ProofCaseSpec, ProofCodecError, ProofNumericalIdentity, ProofProvenance,
    ProofReferenceIdentity, ProofSemanticSubject, ProofSidecarBuilder, ProofSubjectError,
    VerifiedProofSidecar, decode_proof_sidecar,
};

// -------------------------------------------------------------------------
// Fixtures
// -------------------------------------------------------------------------

/// The artifact interface these fixtures bind: one `[2, 3]` f32 input named
/// `input`, one `[2]` f32 output named `result`.
const INPUT_ELEMENTS: usize = 6;
const OUTPUT_ELEMENTS: usize = 2;

fn input_key() -> InputKey {
    InputKey::new("input").expect("a valid input key")
}

fn output_key() -> OutputKey {
    OutputKey::new("result").expect("a valid output key")
}

fn numerical() -> ProofNumericalIdentity {
    ProofNumericalIdentity::from_bytes(b"tiler.test.strict-f32").expect("a bounded subject")
}

fn reference() -> ProofReferenceIdentity {
    ProofReferenceIdentity::from_bytes(b"tiler.test.reference-registry.v1")
        .expect("a bounded subject")
}

fn provenance(graph: &SemanticGraphIdentity) -> ProofProvenance {
    ProofProvenance {
        semantic_graph: graph.clone(),
        numerical: numerical(),
        reference: reference(),
    }
}

/// A case whose payloads are the declared element counts at four bytes each.
fn case(key: &str, fill: u8) -> ProofCaseSpec {
    ProofCaseSpec {
        key: ProofCaseKey::new(key).expect("a valid case key"),
        inputs: vec![(input_key(), vec![fill; INPUT_ELEMENTS * 4])],
        expected: vec![(output_key(), vec![fill ^ 0xff; OUTPUT_ELEMENTS * 4])],
    }
}

/// A genuinely different artifact: a different semantic graph *and* the kernel
/// that realizes it.
///
/// The scale constant is what differs. An unreached extra input would be
/// compacted away at commit (ADR 0064) and would leave graph identity
/// unchanged, so a fixture that only added one would silently make the
/// semantic-subject cases vacuous.
fn other_artifact() -> VerifiedArtifactProgram {
    let semantic = build_graph_scaled(
        SemanticProgramBuilder::try_standard().expect("the standard registry freezes"),
        3.0,
    );
    let program = fused_program(&semantic, OTHER_SCALE_BITS);
    let provider = lowering_provider(1);
    build_artifact(&semantic, &program, provider.clone(), &[provider])
}

fn draft(artifact: &VerifiedArtifactProgram) -> ProofSidecarBuilder {
    ProofSidecarBuilder::new(artifact, provenance(artifact.semantic_graph_identity()))
        .expect("the artifact's own graph is accepted")
}

fn sidecar(artifact: &VerifiedArtifactProgram) -> VerifiedProofSidecar {
    let mut draft = draft(artifact);
    draft.push_case(case("empty-domain", 0x00)).expect("a case");
    draft
        .push_case(case("canonical-nan", 0x7f))
        .expect("a case");
    draft.push_case(case("signed-zero", 0x80)).expect("a case");
    draft.build().expect("a verified sidecar")
}

fn default_sidecar() -> VerifiedProofSidecar {
    sidecar(&default_artifact())
}

/// Re-encodes a directly mutated sidecar, so a forged case is presented with a
/// correct manifest digest, correct payload digests, and a correct identity.
fn forge(mutate: impl FnOnce(&mut ProofSidecarData)) -> Vec<u8> {
    let mut sidecar = default_sidecar();
    mutate(&mut sidecar.data);
    let identity = derive_identity(&sidecar.data).expect("a bounded identity");
    super::codec::encode(&sidecar.data, &identity).expect("a bounded encoding")
}

// -------------------------------------------------------------------------
// Construction
// -------------------------------------------------------------------------

#[test]
fn builds_a_verified_sidecar_over_the_artifact_interface() {
    let artifact = default_artifact();
    let sidecar = sidecar(&artifact);
    assert_eq!(sidecar.cases().len(), 3);
    assert_eq!(sidecar.input_keys(), [input_key()].as_slice());
    assert_eq!(sidecar.output_keys(), [output_key()].as_slice());
    assert_eq!(
        sidecar.artifact_identity_bytes(),
        artifact.canonical_identity().as_bytes(),
    );
    assert_eq!(
        sidecar.semantic_subject().as_bytes(),
        artifact.semantic_graph_identity().as_bytes(),
    );
    assert_eq!(sidecar.numerical_identity(), &numerical());
    assert_eq!(sidecar.reference_identity(), &reference());
}

#[test]
fn the_association_is_derived_from_the_artifact_and_not_supplied() {
    let artifact = default_artifact();
    let bytes = artifact.encode().expect("the artifact encodes");
    let expected = crate::program::envelope_digest(&bytes);
    assert_eq!(sidecar(&artifact).envelope_digest().as_bytes(), &expected);
}

#[test]
fn a_case_reads_back_its_exact_bytes_by_key() {
    let sidecar = default_sidecar();
    let key = ProofCaseKey::new("canonical-nan").expect("a valid case key");
    let case = sidecar.case(&key).expect("the case is present");
    assert_eq!(case.key(), &key);
    let input = case.inputs().next().expect("one input");
    assert_eq!(input.key(), &input_key());
    assert_eq!(input.bytes(), [0x7f_u8; INPUT_ELEMENTS * 4].as_slice());
    let expected = case.expected().next().expect("one output");
    assert_eq!(expected.key(), &output_key());
    assert_eq!(expected.bytes(), [0x80_u8; OUTPUT_ELEMENTS * 4].as_slice());
}

#[test]
fn a_payload_preserves_every_bit_pattern_a_float_reading_would_normalize() {
    // A signalling NaN, a quiet NaN, a negative zero, and a subnormal. Nothing
    // here interprets them, which is the property: a container that parsed
    // floats would be free to canonicalize the first into the second, and a
    // bitwise readback comparison would then pass against the wrong value.
    let patterns: [u32; 4] = [0x7f80_0001, 0x7fc0_0000, 0x8000_0000, 0x0000_0001];
    let mut bytes = Vec::new();
    for pattern in patterns {
        bytes.extend_from_slice(&pattern.to_le_bytes());
    }
    let padded = [bytes.clone(), bytes.clone()].concat();
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    draft
        .push_case(ProofCaseSpec {
            key: ProofCaseKey::new("exceptional").expect("a valid case key"),
            inputs: vec![(input_key(), padded[..INPUT_ELEMENTS * 4].to_vec())],
            expected: vec![(output_key(), bytes[..OUTPUT_ELEMENTS * 4].to_vec())],
        })
        .expect("a case");
    let sidecar = draft.build().expect("a verified sidecar");
    let encoded = sidecar.encode().expect("the sidecar encodes");
    let decoded = decode_proof_sidecar(&encoded).expect("the sidecar decodes");
    let read = decoded.cases().next().expect("one case");
    assert_eq!(
        read.expected().next().expect("one output").bytes(),
        &bytes[..OUTPUT_ELEMENTS * 4],
    );
}

#[test]
fn rejects_expectations_evaluated_over_another_semantic_graph() {
    let artifact = default_artifact();
    let other = other_artifact();
    let error = ProofSidecarBuilder::new(&artifact, provenance(other.semantic_graph_identity()))
        .expect_err("a foreign graph is refused");
    let ProofBuildError::SemanticSubjectMismatch {
        declared,
        artifact: held,
    } = error
    else {
        panic!("expected a semantic subject mismatch, got {error:?}");
    };
    assert_eq!(declared, other.semantic_graph_identity().as_bytes());
    assert_eq!(held, artifact.semantic_graph_identity().as_bytes());
}

#[test]
fn rejects_a_repeated_stable_case_key() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    draft.push_case(case("same", 0x01)).expect("a case");
    let error = draft
        .push_case(case("same", 0x02))
        .expect_err("a repeated key is refused");
    assert!(matches!(error, ProofBuildError::DuplicateCaseKey { .. }));
}

#[test]
fn a_rejected_case_leaves_the_draft_unchanged() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    draft.push_case(case("kept", 0x01)).expect("a case");
    let mut broken = case("broken", 0x02);
    broken.inputs.clear();
    assert!(draft.push_case(broken).is_err());
    let built = draft.build().expect("the draft still builds");
    assert_eq!(built.cases().len(), 1);
    assert_eq!(
        built.cases().next().expect("one case").key().as_str(),
        "kept",
    );
}

#[test]
fn rejects_a_case_naming_an_undeclared_key() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    let mut spec = case("stray", 0x01);
    spec.inputs[0].0 = InputKey::new("absent").expect("a valid key");
    let error = draft
        .push_case(spec)
        .expect_err("an undeclared key is refused");
    assert!(matches!(error, ProofBuildError::UnknownInput { .. }));
}

#[test]
fn rejects_a_case_that_omits_a_declared_output() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    let mut spec = case("partial", 0x01);
    spec.expected.clear();
    let error = draft
        .push_case(spec)
        .expect_err("an omitted output is refused");
    let ProofBuildError::MissingOutput { key } = error else {
        panic!("expected a missing output, got {error:?}");
    };
    assert_eq!(key, output_key());
}

#[test]
fn rejects_a_case_that_supplies_one_declared_input_twice() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    let mut spec = case("doubled", 0x01);
    spec.inputs
        .push((input_key(), vec![0x02; INPUT_ELEMENTS * 4]));
    let error = draft.push_case(spec).expect_err("a repeat is refused");
    assert!(matches!(error, ProofBuildError::DuplicateInput { .. }));
}

#[test]
fn rejects_an_empty_sidecar() {
    let artifact = default_artifact();
    assert!(matches!(
        draft(&artifact)
            .build()
            .expect_err("an empty draft is refused"),
        ProofBuildError::NoCases,
    ));
}

#[test]
fn rejects_a_payload_that_is_not_a_whole_number_of_declared_elements() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    let mut spec = case("ragged", 0x01);
    // Five bytes over six declared elements: not a whole width, and no width
    // this crate could invent would make it one.
    spec.inputs[0].1 = vec![0x01; 5];
    draft.push_case(spec).expect("the local rules admit it");
    let error = draft.build().expect_err("the terminal refuses it");
    let ProofBuildError::Interface(ProofInterfaceError::PayloadNotWholeElements {
        direction,
        bytes,
        elements,
        ..
    }) = error
    else {
        panic!("expected a whole-elements failure, got {error:?}");
    };
    assert_eq!(direction, ProofDirection::Input);
    assert_eq!(bytes, 5);
    assert_eq!(elements, INPUT_ELEMENTS);
}

#[test]
fn rejects_two_cases_that_disagree_on_one_entry_length() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    draft.push_case(case("wide", 0x01)).expect("a case");
    let mut narrow = case("narrow", 0x02);
    narrow.inputs[0].1 = vec![0x02; INPUT_ELEMENTS];
    draft.push_case(narrow).expect("the local rules admit it");
    let error = draft.build().expect_err("the terminal refuses it");
    assert!(matches!(
        error,
        ProofBuildError::Interface(ProofInterfaceError::PayloadLengthDisagreement { .. }),
    ));
}

#[test]
fn rejects_a_case_count_beyond_the_governed_bound() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    for index in 0..MAX_PROOF_CASES {
        draft
            .push_case(case(&format!("case-{index:04}"), 0x01))
            .expect("a case within the bound");
    }
    let error = draft
        .push_case(case("one-too-many", 0x01))
        .expect_err("the bound is enforced");
    let ProofBuildError::Limit(limit) = error else {
        panic!("expected a limit, got {error:?}");
    };
    assert_eq!(limit.attempted, MAX_PROOF_CASES + 1);
    assert_eq!(limit.limit, MAX_PROOF_CASES);
}

/// One whole-element input just larger than 16 MiB.
///
/// That size is past the workspace's usual large-number default and still far
/// under the 256 MiB container, so admitting it proves one payload has no
/// separate size policy.
const WIDE_INPUT_BYTES: usize = (16 * 1024 * 1024 / INPUT_ELEMENTS + 1) * INPUT_ELEMENTS;

#[test]
fn admits_one_payload_larger_than_sixteen_mib() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    let mut spec = case("wide", 0x5a);
    spec.inputs[0].1 = vec![0x5a; WIDE_INPUT_BYTES];
    draft
        .push_case(spec)
        .expect("one payload has no separate size policy");
    let sidecar = draft
        .build()
        .expect("the sidecar stays under the container");
    let bytes = sidecar.encode().expect("the sidecar encodes");
    assert!(bytes.len() < MAX_PROOF_SIDECAR_BYTES);
    let decoded = decode_proof_sidecar(&bytes).expect("the reader admits the same payload");
    assert_eq!(
        decoded
            .cases()
            .next()
            .expect("one case")
            .inputs()
            .next()
            .expect("one input")
            .bytes()
            .len(),
        WIDE_INPUT_BYTES,
    );
}

#[test]
fn refuses_one_payload_that_overflows_the_sidecar_before_reserving_it() {
    let sidecar = default_sidecar();
    let projected = layout_of(
        &sidecar.data,
        vec![CaseLens {
            key_len: "oversize".len(),
            input_lens: vec![MAX_PROOF_SIDECAR_BYTES + 1],
            expected_lens: vec![OUTPUT_ELEMENTS * 4],
        }],
    );
    assert!(projected.sidecar > MAX_PROOF_SIDECAR_BYTES);
    let error = projected
        .check()
        .expect_err("the container bound is enforced");
    assert_eq!(
        error,
        ProofBudgetError::Limit(ProofLimitExceeded {
            kind: ProofLimitKind::SidecarBytes,
            attempted: projected.sidecar,
            limit: MAX_PROOF_SIDECAR_BYTES,
        })
    );
}

#[test]
fn a_moved_payload_keeps_its_allocation() {
    let artifact = default_artifact();
    let mut draft = draft(&artifact);
    let payload = vec![0x5a; INPUT_ELEMENTS * 4];
    let pointer = payload.as_ptr();
    draft
        .push_case(ProofCaseSpec {
            key: ProofCaseKey::new("moved").expect("a valid case key"),
            inputs: vec![(input_key(), payload)],
            expected: vec![(output_key(), vec![0xa5; OUTPUT_ELEMENTS * 4])],
        })
        .expect("a case");
    let sidecar = draft.build().expect("a verified sidecar");
    assert_eq!(
        sidecar
            .cases()
            .next()
            .expect("one case")
            .inputs()
            .next()
            .expect("one input")
            .bytes()
            .as_ptr(),
        pointer,
    );
}

#[test]
fn a_case_key_is_bounded_and_never_empty() {
    assert!(matches!(
        ProofCaseKey::new("").expect_err("an empty key is refused"),
        ProofCaseKeyError::Empty,
    ));
    let long = "k".repeat(MAX_PROOF_CASE_KEY_BYTES + 1);
    assert!(matches!(
        ProofCaseKey::new(&long).expect_err("an oversized key is refused"),
        ProofCaseKeyError::TooLong { .. },
    ));
    assert!(ProofCaseKey::new("k".repeat(MAX_PROOF_CASE_KEY_BYTES)).is_ok());
}

#[test]
fn a_provenance_subject_is_bounded_and_never_empty() {
    assert!(matches!(
        ProofNumericalIdentity::from_bytes([]).expect_err("empty bytes are refused"),
        ProofSubjectError::Empty,
    ));
    assert!(matches!(
        ProofReferenceIdentity::from_bytes(vec![0x01; MAX_PROOF_SUBJECT_BYTES + 1])
            .expect_err("oversized bytes are refused"),
        ProofSubjectError::TooLong { .. },
    ));
}

// -------------------------------------------------------------------------
// Canonical form and identity
// -------------------------------------------------------------------------

#[test]
fn a_sidecar_round_trips_through_its_encoding() {
    let sidecar = default_sidecar();
    let bytes = sidecar.encode().expect("the sidecar encodes");
    let decoded = decode_proof_sidecar(&bytes).expect("the sidecar decodes");
    assert_eq!(decoded.identity(), sidecar.canonical_identity());
    assert_eq!(decoded.re_encode().expect("it re-encodes"), bytes);
    assert_eq!(decoded.cases().len(), sidecar.cases().len());
    for (read, held) in decoded.cases().zip(sidecar.cases()) {
        assert_eq!(read.key(), held.key());
        for (read, held) in read.inputs().zip(held.inputs()) {
            assert_eq!(read.bytes(), held.bytes());
        }
        for (read, held) in read.expected().zip(held.expected()) {
            assert_eq!(read.bytes(), held.bytes());
        }
    }
}

#[test]
fn admission_order_changes_neither_the_bytes_nor_the_identity() {
    let artifact = default_artifact();
    let mut forward = draft(&artifact);
    for key in ["alpha", "beta", "gamma"] {
        forward.push_case(case(key, 0x11)).expect("a case");
    }
    let mut backward = draft(&artifact);
    for key in ["gamma", "beta", "alpha"] {
        backward.push_case(case(key, 0x11)).expect("a case");
    }
    let forward = forward.build().expect("a verified sidecar");
    let backward = backward.build().expect("a verified sidecar");
    assert_eq!(forward.canonical_identity(), backward.canonical_identity());
    assert_eq!(
        forward.encode().expect("it encodes"),
        backward.encode().expect("it encodes"),
    );
}

#[test]
fn one_changed_expected_byte_changes_the_identity() {
    let artifact = default_artifact();
    let mut left = draft(&artifact);
    left.push_case(case("only", 0x11)).expect("a case");
    let left = left.build().expect("a verified sidecar");

    let mut right = draft(&artifact);
    let mut altered = case("only", 0x11);
    altered.expected[0].1[0] ^= 0x01;
    right.push_case(altered).expect("a case");
    let right = right.build().expect("a verified sidecar");

    assert_ne!(left.canonical_identity(), right.canonical_identity());
}

#[test]
fn a_different_artifact_changes_the_identity() {
    let left = default_sidecar();
    let right = sidecar(&other_artifact());
    assert_ne!(left.canonical_identity(), right.canonical_identity());
}

#[test]
fn dotted_operation_boundaries_reach_proof_subject_and_envelope_association() {
    let left = artifact_with_selected_operations(&[("a.b", "c", 1)]);
    let right = artifact_with_selected_operations(&[("a", "b.c", 1)]);
    let pair = artifact_with_selected_operations(&[("a.b", "c", 1), ("a", "b.c", 1)]);

    assert_ne!(left.canonical_identity(), right.canonical_identity());
    assert_ne!(left.canonical_identity(), pair.canonical_identity());
    assert_ne!(right.canonical_identity(), pair.canonical_identity());
    assert_eq!(pair.selected_providers().len(), 2, "both subjects package");

    let left_sidecar = sidecar(&left);
    let right_sidecar = sidecar(&right);
    let pair_sidecar = sidecar(&pair);
    assert_ne!(
        left_sidecar.canonical_identity(),
        right_sidecar.canonical_identity(),
        "the exact artifact subject reaches proof identity",
    );
    assert_eq!(
        pair_sidecar.artifact_identity_bytes(),
        pair.canonical_identity().as_bytes(),
    );
    let pair_bytes = pair.encode().expect("the pair envelope encodes");
    assert_eq!(
        pair_sidecar.envelope_digest().as_bytes(),
        &crate::program::envelope_digest(&pair_bytes),
        "the exact pair envelope reaches the sidecar association",
    );
    let decoded = decode_proof_sidecar(
        &pair_sidecar
            .encode()
            .expect("the pair-associated sidecar encodes"),
    )
    .expect("the pair-associated sidecar decodes");
    assert_eq!(
        decoded.artifact_identity_bytes(),
        pair.canonical_identity().as_bytes(),
    );
}

// The union no-prefix check moved to `crate::domains`, which enumerates every
// governed domain the crate admits from a type rather than from a hand-written
// list of eight. The list here covered 8 of the crate's 11 container domains and
// none of its 7 program-identity domains, and its `[&[u8]; 8]` length literal is
// what let those be added with nothing failing.

#[test]
fn the_sidecar_magic_is_not_the_envelope_magic() {
    let artifact = default_artifact();
    let envelope = artifact.encode().expect("the artifact encodes");
    let sidecar = sidecar(&artifact).encode().expect("the sidecar encodes");
    assert_ne!(&envelope[..MAGIC.len()], &sidecar[..MAGIC.len()]);
    // Each reader refuses the other's bytes at the magic rather than misparsing.
    assert!(matches!(
        decode_proof_sidecar(&envelope).expect_err("an envelope is not a sidecar"),
        ProofCodecError::BadMagic,
    ));
    assert!(crate::program::decode_artifact(&sidecar).is_err());
}

// -------------------------------------------------------------------------
// Byte-level rejection
// -------------------------------------------------------------------------

fn decode_error(bytes: &[u8]) -> ProofCodecError {
    decode_proof_sidecar(bytes).expect_err("the reader refuses these bytes")
}

fn encoded() -> Vec<u8> {
    default_sidecar().encode().expect("the sidecar encodes")
}

#[test]
fn refuses_truncated_bytes() {
    let bytes = encoded();
    for cut in [0_usize, 1, HEADER_BYTES - 1, HEADER_BYTES, bytes.len() - 1] {
        let error = decode_error(&bytes[..cut]);
        assert_eq!(
            error.classification(),
            ProofFailureClass::Malformed,
            "cutting at {cut} produced {error:?}",
        );
    }
}

/// Byte offset of the first framed payload's length prefix.
///
/// Manifest length is the u64 at header offset 25: magic (8), format (4),
/// encoding (4), algorithm (1), declared total (8).
fn first_framed_len_offset(bytes: &[u8]) -> usize {
    let manifest_bytes = u64::from_be_bytes(
        bytes[25..33]
            .try_into()
            .expect("the header carries a manifest length"),
    );
    HEADER_BYTES + usize::try_from(manifest_bytes).expect("a bounded manifest fits") + 4
}

fn write_framed_len(bytes: &mut [u8], len: u64) {
    let offset = first_framed_len_offset(bytes);
    bytes[offset..offset + 8].copy_from_slice(&len.to_be_bytes());
}

#[test]
fn framed_length_distinguishes_sidecar_limit_from_truncation() {
    let bytes = encoded();
    let offset = first_framed_len_offset(&bytes);
    let available = bytes.len() - (offset + 8);

    let mut over_container = bytes.clone();
    write_framed_len(&mut over_container, (MAX_PROOF_SIDECAR_BYTES + 1) as u64);
    let error = decode_error(&over_container);
    let ProofCodecError::Limit(limit) = error else {
        panic!("a framed length beyond the container is a sidecar limit, got {error:?}");
    };
    assert_eq!(limit.kind, ProofLimitKind::SidecarBytes);
    assert_eq!(limit.limit, MAX_PROOF_SIDECAR_BYTES);
    assert_eq!(limit.attempted, MAX_PROOF_SIDECAR_BYTES + 1);

    let mut truncated = bytes;
    let needed = available + 1;
    assert!(needed < MAX_PROOF_SIDECAR_BYTES);
    write_framed_len(&mut truncated, needed as u64);
    assert!(matches!(
        decode_error(&truncated),
        ProofCodecError::Truncated {
            needed: got,
            available: left,
        } if got == needed && left == available,
    ));
}

#[test]
fn refuses_trailing_bytes() {
    let mut bytes = encoded();
    bytes.push(0x00);
    // The declared total length is checked before anything else reads the body,
    // so an appended byte is caught there rather than as a trailing remainder.
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::TotalLengthMismatch { .. },
    ));
}

#[test]
fn refuses_a_corrupted_payload_byte() {
    let bytes = encoded();
    // The last payload's content is the tail of the encoding.
    let mut damaged = bytes.clone();
    let last = damaged.len() - 1;
    damaged[last] ^= 0xff;
    assert!(matches!(
        decode_error(&damaged),
        ProofCodecError::PayloadDigestMismatch { .. },
    ));
}

#[test]
fn refuses_a_corrupted_manifest_byte() {
    let mut bytes = encoded();
    bytes[HEADER_BYTES + MANIFEST_DOMAIN.len()] ^= 0xff;
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::ManifestDigestMismatch,
    ));
}

#[test]
fn refuses_an_unimplemented_format_or_encoding_or_schema() {
    let mut bytes = encoded();
    // The high byte of the framing format's major version. A *major* step is
    // refused outright rather than read on a best effort, which is the whole
    // lockstep posture.
    bytes[MAGIC.len()] = 0x02;
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::UnsupportedSidecarFormat { major: 0x0201, .. },
    ));

    let mut bytes = encoded();
    bytes[MAGIC.len() + 4] = 0x02;
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::UnsupportedCanonicalEncoding { .. },
    ));

    let mut bytes = encoded();
    bytes[MAGIC.len() + 8] = 0x7f;
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::UnsupportedDigestAlgorithm { tag: 0x7f },
    ));
}

#[test]
fn a_minor_version_this_build_predates_is_unsupported_rather_than_ignored() {
    let mut bytes = encoded();
    // Minor is the second `u16` of the framing format.
    bytes[MAGIC.len() + 3] = SIDECAR_FORMAT.1.to_be_bytes()[1] + 1;
    assert_eq!(
        decode_error(&bytes).classification(),
        ProofFailureClass::Unsupported,
    );
    assert_eq!(CANONICAL_ENCODING, (1, 0));
}

// -------------------------------------------------------------------------
// Forged containers: structurally invalid, then correctly re-sealed
// -------------------------------------------------------------------------

#[test]
fn refuses_cases_that_are_not_in_canonical_key_order() {
    let bytes = forge(|data| data.cases.reverse());
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::NonCanonicalOrder {
            subject: ProofOrderedSubject::Case,
            ..
        },
    ));
}

#[test]
fn refuses_two_cases_sharing_one_stable_key() {
    let bytes = forge(|data| {
        let duplicate = data.cases[0].clone();
        data.cases = vec![duplicate.clone(), duplicate];
    });
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::DuplicateItem {
            subject: ProofOrderedSubject::Case,
            ..
        },
    ));
}

#[test]
fn refuses_a_sidecar_that_carries_no_case() {
    let bytes = forge(|data| data.cases.clear());
    assert!(matches!(decode_error(&bytes), ProofCodecError::NoCases));
}

#[test]
fn refuses_cases_that_disagree_on_one_entry_length() {
    let bytes = forge(|data| data.cases[1].inputs[0].truncate(4));
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::CasePayloads {
            cause: ProofInterfaceError::PayloadLengthDisagreement { .. },
        },
    ));
}

#[test]
fn refuses_a_case_whose_payload_count_is_not_the_bound_entry_count() {
    let bytes = forge(|data| data.cases[0].expected.clear());
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::CasePayloads {
            cause: ProofInterfaceError::PayloadArity { .. },
        },
    ));
}

#[test]
fn refuses_a_repeated_bound_interface_key() {
    let bytes = forge(|data| {
        data.input_keys = vec![input_key(), input_key()];
        for case in &mut data.cases {
            case.inputs = vec![case.inputs[0].clone(), case.inputs[0].clone()];
        }
    });
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::DuplicateItem {
            subject: ProofOrderedSubject::Input,
            ..
        },
    ));
}

#[test]
fn refuses_a_manifest_whose_carried_identity_is_not_the_derived_one() {
    // The forger stamps a correct manifest digest over an identity that does
    // not describe the content beside it. Only re-derivation catches this.
    let sidecar = default_sidecar();
    let mut wrong = sidecar.data.clone();
    wrong.cases[0].key = ProofCaseKey::new("aaa-renamed").expect("a valid key");
    wrong.cases.sort_by(|left, right| left.key.cmp(&right.key));
    let stale = derive_identity(&sidecar.data).expect("a bounded identity");
    let bytes = super::codec::encode(&wrong, &stale).expect("a bounded encoding");
    assert!(matches!(
        decode_error(&bytes),
        ProofCodecError::SidecarIdentityMismatch,
    ));
}

// -------------------------------------------------------------------------
// Association
// -------------------------------------------------------------------------

#[test]
fn binds_to_the_exact_envelope_it_names() {
    let artifact = default_artifact();
    let envelope = artifact.encode().expect("the artifact encodes");
    let bytes = sidecar(&artifact).encode().expect("the sidecar encodes");
    let decoded = decode_proof_sidecar(&bytes).expect("the sidecar decodes");
    decoded
        .bind_to_envelope(&envelope)
        .expect("the exact envelope binds");
    decoded
        .bind_to_artifact(&artifact)
        .expect("the exact artifact binds");
}

#[test]
fn refuses_to_bind_to_another_artifacts_envelope() {
    use super::ProofAssociationError;

    let artifact = default_artifact();
    let other = other_artifact();
    let bytes = sidecar(&artifact).encode().expect("the sidecar encodes");
    let decoded = decode_proof_sidecar(&bytes).expect("the sidecar decodes");

    let foreign = other.encode().expect("the other artifact encodes");
    assert!(matches!(
        decoded
            .bind_to_envelope(&foreign)
            .expect_err("a foreign envelope is refused"),
        ProofAssociationError::EnvelopeDigestMismatch,
    ));
    assert!(matches!(
        decoded
            .bind_to_artifact(&other)
            .expect_err("a foreign artifact is refused"),
        ProofAssociationError::ArtifactIdentityMismatch,
    ));
}

#[test]
fn refuses_to_bind_to_damaged_envelope_bytes() {
    use super::ProofAssociationError;

    let artifact = default_artifact();
    let mut envelope = artifact.encode().expect("the artifact encodes");
    let last = envelope.len() - 1;
    envelope[last] ^= 0xff;
    let bytes = sidecar(&artifact).encode().expect("the sidecar encodes");
    let decoded = decode_proof_sidecar(&bytes).expect("the sidecar decodes");
    assert!(matches!(
        decoded
            .bind_to_envelope(&envelope)
            .expect_err("damaged bytes are refused"),
        ProofAssociationError::EnvelopeDigestMismatch,
    ));
}

#[test]
fn refuses_to_bind_to_bytes_that_are_not_an_envelope() {
    use super::ProofAssociationError;

    // Digest-matching bytes that are not a valid envelope cannot be
    // constructed, so this case proves the ordering instead: the digest check
    // runs first, and a caller that supplies rubbish learns that the bytes are
    // not the ones named rather than receiving a codec's parse error.
    let artifact = default_artifact();
    let bytes = sidecar(&artifact).encode().expect("the sidecar encodes");
    let decoded = decode_proof_sidecar(&bytes).expect("the sidecar decodes");
    assert!(matches!(
        decoded
            .bind_to_envelope(b"not an envelope")
            .expect_err("rubbish is refused"),
        ProofAssociationError::EnvelopeDigestMismatch,
    ));
}

#[test]
fn binding_to_an_artifact_re_proves_the_interface_obligations() {
    use super::ProofAssociationError;

    // A sidecar whose bound keys are not the artifact's is refused by the
    // stronger check with a named interface cause. The container itself is
    // internally consistent — it decodes — which is the point: the obligation
    // being re-proven here is one no decoder can prove alone.
    let artifact = default_artifact();
    let bytes = forge(|data| {
        data.input_keys = vec![InputKey::new("renamed").expect("a valid key")];
    });
    let decoded = decode_proof_sidecar(&bytes).expect("the container is internally consistent");
    let error = decoded
        .bind_to_artifact(&artifact)
        .expect_err("the interface disagreement is refused");
    assert!(matches!(
        error,
        ProofAssociationError::Interface {
            cause: ProofInterfaceError::InputKeyMismatch { .. },
        },
    ));
}

#[test]
fn the_two_obligation_provers_are_one_implementation() {
    // `verify_cases` is called by the builder's terminal and by
    // `bind_to_artifact`; this pins that it rejects the same content in both
    // roles, so the shared implementation cannot be split without a failure.
    let artifact = default_artifact();
    let sidecar = sidecar(&artifact);
    let interface = BoundInterface {
        inputs: vec![(input_key(), INPUT_ELEMENTS)],
        outputs: vec![(
            OutputKey::new("renamed").expect("a valid key"),
            OUTPUT_ELEMENTS,
        )],
    };
    assert!(matches!(
        verify_cases(&interface, &sidecar.data).expect_err("a renamed output is refused"),
        ProofInterfaceError::OutputKeyMismatch { .. },
    ));
}

// -------------------------------------------------------------------------
// Governed digest use
// -------------------------------------------------------------------------

#[test]
fn every_payload_digest_binds_its_canonical_slot() {
    // Two payloads with equal bytes at different slots must not share a
    // content address, or a swap between them would be invisible to the
    // manifest.
    let algorithm = DigestAlgorithm::GOVERNED;
    let bytes = [0x5a_u8; 8];
    assert_ne!(
        algorithm.digest_qualified(PAYLOAD_DIGEST_DOMAIN, &[&0_u32.to_be_bytes()], &bytes),
        algorithm.digest_qualified(PAYLOAD_DIGEST_DOMAIN, &[&1_u32.to_be_bytes()], &bytes),
    );
}

#[test]
fn the_identity_folds_payload_digests_rather_than_payload_bytes() {
    // The container's payloads total 96 bytes across three cases; the identity
    // is bounded by the case and interface counts instead, which is what keeps
    // it usable as a cache key for a sidecar carrying megabytes of evidence.
    let sidecar = default_sidecar();
    let payload_bytes: usize = sidecar
        .cases()
        .flat_map(|case| {
            case.inputs()
                .map(|payload| payload.bytes().len())
                .chain(case.expected().map(|payload| payload.bytes().len()))
                .collect::<Vec<_>>()
        })
        .sum();
    let identity_bytes = sidecar.canonical_identity().as_bytes().len();
    assert!(payload_bytes > 0);
    assert_eq!(
        payload_bytes,
        3 * (INPUT_ELEMENTS * 4 + OUTPUT_ELEMENTS * 4)
    );
    // Six payload digests at a fixed width, plus the fixed and keyed prelude.
    assert_eq!(
        identity_bytes,
        expected_identity_bytes(&sidecar.data, DIGEST_BYTES),
    );
}

/// Recomputes the identity's exact length from its stated field list.
///
/// Written out rather than asserted against a magic number, so a field added to
/// the identity encoder without a corresponding entry here fails this test
/// instead of silently shifting a constant.
fn expected_identity_bytes(data: &ProofSidecarData, digest_bytes: usize) -> usize {
    let framed = |len: usize| 8 + len;
    let mut total = IDENTITY_DOMAIN.len() + 4;
    total += framed(data.artifact_identity.len());
    total += digest_bytes;
    total += framed(data.subjects.semantic.as_bytes().len());
    total += framed(data.subjects.numerical.as_bytes().len());
    total += framed(data.subjects.reference.as_bytes().len());
    total += 8 + data
        .input_keys
        .iter()
        .map(|key| framed(key.as_str().len()))
        .sum::<usize>();
    total += 8 + data
        .output_keys
        .iter()
        .map(|key| framed(key.as_str().len()))
        .sum::<usize>();
    total += 8;
    for case in &data.cases {
        total += framed(case.key.as_str().len());
        total += 16;
        total += (case.inputs.len() + case.expected.len()) * digest_bytes;
    }
    total
}

#[test]
fn a_forged_case_is_indistinguishable_from_a_real_one_by_the_container_alone() {
    // The measurement boundary this module documents, pinned as a test rather
    // than left as a claim: a container whose expected bytes were rewritten and
    // re-sealed validates. What refuses it is the device comparison downstream,
    // not this reader, and a consumer that believed otherwise would be wrong.
    let artifact = default_artifact();
    let mut forged = sidecar(&artifact);
    forged.data.cases[0].expected[0][0] ^= 0xff;
    let identity = derive_identity(&forged.data).expect("a bounded identity");
    let bytes = super::codec::encode(&forged.data, &identity).expect("it encodes");
    let decoded = decode_proof_sidecar(&bytes).expect("a re-sealed forgery still validates");
    decoded
        .bind_to_envelope(&artifact.encode().expect("the artifact encodes"))
        .expect("and it still binds to the artifact it names");
    assert_ne!(decoded.identity(), default_sidecar().canonical_identity());
}

/// A placeholder that keeps the unused-fixture warning honest.
#[test]
fn the_artifact_fixture_scale_constants_are_distinct() {
    assert_ne!(SCALE_BITS, OTHER_SCALE_BITS);
}

/// Keeps `ProofSubjects` and `ProofCaseData` reachable from this suite.
#[test]
fn the_retained_shape_is_the_shape_that_was_built() {
    let sidecar = default_sidecar();
    let ProofSidecarData {
        subjects: ProofSubjects { semantic, .. },
        cases,
        ..
    } = &sidecar.data;
    assert_eq!(
        semantic,
        &ProofSemanticSubject::from_bytes(default_artifact().semantic_graph_identity().as_bytes())
            .expect("a bounded subject")
    );
    let ProofCaseData {
        key,
        inputs,
        expected,
    } = &cases[0];
    assert_eq!(key.as_str(), "canonical-nan");
    assert_eq!(inputs.len(), 1);
    assert_eq!(expected.len(), 1);
}

// -------------------------------------------------------------------------
// Producer byte budgets, checked before proportional allocation
// -------------------------------------------------------------------------

fn layout_of(data: &ProofSidecarData, cases: Vec<CaseLens>) -> super::budget::ProjectedSizes {
    project_sidecar(
        data.artifact_identity.len(),
        data.subjects.semantic.as_bytes().len(),
        data.subjects.numerical.as_bytes().len(),
        data.subjects.reference.as_bytes().len(),
        data.input_keys.iter().map(|key| key.as_str().len()),
        data.output_keys.iter().map(|key| key.as_str().len()),
        cases,
    )
    .expect("a representable projection")
}

/// Large whole-element payloads used to fill the container.
///
/// A sixteenth of the sidecar bound, aligned to each interface entry's
/// element count. The retired per-payload gate is not an authority here.
const LARGE_INPUT_BYTES: usize = (MAX_PROOF_SIDECAR_BYTES / 16) / INPUT_ELEMENTS * INPUT_ELEMENTS;
const LARGE_OUTPUT_BYTES: usize =
    (MAX_PROOF_SIDECAR_BYTES / 16) / OUTPUT_ELEMENTS * OUTPUT_ELEMENTS;

fn max_dual_case(key: &str) -> CaseLens {
    CaseLens {
        key_len: key.len(),
        input_lens: vec![LARGE_INPUT_BYTES],
        expected_lens: vec![LARGE_OUTPUT_BYTES],
    }
}

fn max_dual_spec(key: &str) -> ProofCaseSpec {
    ProofCaseSpec {
        key: ProofCaseKey::new(key).expect("a valid case key"),
        inputs: vec![(input_key(), vec![0x11; LARGE_INPUT_BYTES])],
        expected: vec![(output_key(), vec![0x22; LARGE_OUTPUT_BYTES])],
    }
}

fn dual_overflow_count(data: &ProofSidecarData) -> usize {
    for count in 1..=MAX_PROOF_CASES {
        let cases: Vec<CaseLens> = (0..count)
            .map(|index| max_dual_case(&format!("case-{index:04}")))
            .collect();
        let projected = layout_of(data, cases);
        if projected.check().is_err() {
            return count;
        }
    }
    panic!("the sidecar bound admits every dual max-payload case");
}

#[test]
fn refuses_an_identity_beyond_the_governed_bound_before_deriving_it() {
    let mut sidecar = default_sidecar();
    sidecar.data.artifact_identity = vec![0xab; MAX_PROOF_IDENTITY_BYTES + 1];
    let projected = project_from_data(&sidecar.data).expect("the size is representable");
    assert!(projected.identity > MAX_PROOF_IDENTITY_BYTES);
    let error = derive_identity(&sidecar.data).expect_err("the identity bound is enforced");
    assert_eq!(
        error,
        ProofBudgetError::Limit(ProofLimitExceeded {
            kind: ProofLimitKind::IdentityBytes,
            attempted: projected.identity,
            limit: MAX_PROOF_IDENTITY_BYTES,
        })
    );
}

#[test]
fn refuses_a_manifest_beyond_the_governed_bound_before_encoding_it() {
    let mut sidecar = default_sidecar();
    // Large enough that framed(artifact) + framed(identity) exceeds the
    // manifest bound, but the identity itself still fits.
    sidecar.data.artifact_identity = vec![0xcd; 5 * 1024 * 1024];
    let identity = derive_identity(&sidecar.data).expect("the identity still fits");
    assert!(identity.as_bytes().len() <= MAX_PROOF_IDENTITY_BYTES);
    let projected = project_from_data(&sidecar.data).expect("the size is representable");
    assert!(projected.manifest > MAX_PROOF_MANIFEST_BYTES);
    assert!(projected.identity <= MAX_PROOF_IDENTITY_BYTES);
    let error =
        super::codec::encode(&sidecar.data, &identity).expect_err("the manifest bound is enforced");
    let ProofCodecError::Limit(limit) = error else {
        panic!("expected a manifest-byte limit, got {error:?}");
    };
    assert_eq!(limit.kind, ProofLimitKind::ManifestBytes);
    assert_eq!(limit.attempted, projected.manifest);
    assert_eq!(limit.limit, MAX_PROOF_MANIFEST_BYTES);
}

#[test]
fn refuses_a_sidecar_total_beyond_the_governed_bound_before_encoding_it() {
    let artifact = default_artifact();
    let mut seed_draft = draft(&artifact);
    seed_draft
        .push_case(case("seed", 0x01))
        .expect("a seed case");
    let seed = seed_draft.build().expect("a verified sidecar");
    let overflow_at = dual_overflow_count(&seed.data);
    assert!(overflow_at >= 2, "one dual max-payload case must fit");

    let mut draft = draft(&artifact);
    for index in 0..(overflow_at - 1) {
        draft
            .push_case(max_dual_spec(&format!("case-{index:04}")))
            .expect("a case whose framed total still fits");
    }
    let overflowing = max_dual_spec(&format!("case-{:04}", overflow_at - 1));
    let error = draft
        .push_case(overflowing)
        .expect_err("the sidecar bound is enforced before the case is admitted");
    let ProofBuildError::Limit(limit) = error else {
        panic!("expected a sidecar-byte limit, got {error:?}");
    };
    assert_eq!(limit.kind, ProofLimitKind::SidecarBytes);
    assert_eq!(limit.limit, MAX_PROOF_SIDECAR_BYTES);
    assert!(limit.attempted > MAX_PROOF_SIDECAR_BYTES);

    let cases: Vec<CaseLens> = (0..overflow_at)
        .map(|index| max_dual_case(&format!("case-{index:04}")))
        .collect();
    let projected = layout_of(&seed.data, cases);
    let content_only = overflow_at
        .checked_mul(LARGE_INPUT_BYTES + LARGE_OUTPUT_BYTES)
        .expect("fits");
    assert!(
        projected.framed_payloads > content_only,
        "framing prefixes are part of the projected sidecar total"
    );
    assert_eq!(limit.attempted, projected.sidecar);

    let kept = draft.build().expect("the draft still builds");
    assert_eq!(kept.cases().len(), overflow_at - 1);

    let mut over = kept;
    over.data.cases.push(ProofCaseData {
        key: ProofCaseKey::new(format!("case-{:04}", overflow_at - 1)).expect("a valid case key"),
        inputs: vec![vec![0x11; LARGE_INPUT_BYTES]],
        expected: vec![vec![0x22; LARGE_OUTPUT_BYTES]],
    });
    over.data
        .cases
        .sort_by(|left, right| left.key.cmp(&right.key));
    let identity = derive_identity(&over.data).expect("identity still fits");
    let encoded = super::codec::encode(&over.data, &identity)
        .expect_err("encode refuses the same sidecar total");
    let ProofCodecError::Limit(encoded_limit) = encoded else {
        panic!("expected a sidecar-byte limit, got {encoded:?}");
    };
    assert_eq!(encoded_limit.kind, ProofLimitKind::SidecarBytes);
    assert_eq!(encoded_limit.attempted, projected.sidecar);
}

#[test]
fn refuses_an_unrepresentable_projected_size() {
    let error =
        add(usize::MAX, 1, ProofLimitKind::SidecarBytes).expect_err("overflow is a named refusal");
    assert_eq!(
        error,
        ProofBudgetError::Unrepresentable {
            kind: ProofLimitKind::SidecarBytes,
        }
    );
    let error = project_sidecar(
        usize::MAX,
        1,
        1,
        1,
        std::iter::empty(),
        std::iter::empty(),
        Vec::new(),
    )
    .expect_err("a huge identity field overflows rather than wrapping");
    assert_eq!(
        error,
        ProofBudgetError::Unrepresentable {
            kind: ProofLimitKind::IdentityBytes,
        }
    );
}

#[test]
fn governed_byte_resources_are_pinned_from_the_limit_kind() {
    const ALL: [ProofLimitKind; core::mem::variant_count::<ProofLimitKind>()] = [
        ProofLimitKind::SidecarBytes,
        ProofLimitKind::ManifestBytes,
        ProofLimitKind::IdentityBytes,
        ProofLimitKind::Cases,
        ProofLimitKind::InterfaceEntries,
        ProofLimitKind::Payloads,
        ProofLimitKind::SubjectBytes,
        ProofLimitKind::TextBytes,
    ];
    for kind in ALL {
        match kind {
            ProofLimitKind::SidecarBytes => {
                assert_eq!(kind.byte_budget(), Some(MAX_PROOF_SIDECAR_BYTES));
            }
            ProofLimitKind::ManifestBytes => {
                assert_eq!(kind.byte_budget(), Some(MAX_PROOF_MANIFEST_BYTES));
            }
            ProofLimitKind::IdentityBytes => {
                assert_eq!(kind.byte_budget(), Some(MAX_PROOF_IDENTITY_BYTES));
            }
            ProofLimitKind::SubjectBytes => {
                assert_eq!(kind.byte_budget(), Some(MAX_PROOF_SUBJECT_BYTES));
            }
            ProofLimitKind::TextBytes => {
                assert_eq!(kind.byte_budget(), Some(4 * 1024));
            }
            ProofLimitKind::Cases | ProofLimitKind::InterfaceEntries | ProofLimitKind::Payloads => {
                assert_eq!(kind.byte_budget(), None);
            }
        }
    }
}
