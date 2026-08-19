//! Canonical form: round trip, determinism, and declaration-order independence.

use super::super::super::expr::AbiRoot;
use super::super::super::tests::{
    Formulas, SCALE_BITS, build_artifact, declare_realization, default_artifact, formulas,
    fused_program, lowering_provider, payload, selection, semantic_program, spare_provider,
    variant,
};
use super::super::super::{ArtifactProgramBuilder, CompilationEnvironment};
use super::super::decode::decode;
use super::super::encode::{
    ENVELOPE_FORMAT, HEADER_BYTES, MAGIC, MANIFEST_DOMAIN, MANIFEST_SCHEMA, encode,
    envelope_digest, matches_canonical_encoding, section_digest,
};
use super::super::model::FEATURE_MULTI_VARIANT_ROUTING;
use super::support::{
    MANIFEST_DIGEST_AT, MANIFEST_LENGTH_AT, TOTAL_LENGTH_AT, encoded, envelope_of,
    two_variant_artifact,
};
use tiler_digest::DigestAlgorithm;

// -------------------------------------------------------------------------
// Round trip and canonical form
// -------------------------------------------------------------------------

#[test]
fn an_encoded_envelope_round_trips_to_an_equal_model() {
    let artifact = default_artifact();
    let envelope = envelope_of(&artifact);
    let bytes = encode(&envelope).expect("the envelope encodes");
    let decoded = decode(&bytes).expect("its own bytes decode");
    assert_eq!(decoded, envelope);
    assert_eq!(
        decoded.canonical_identity().expect("identity re-derives"),
        *artifact.canonical_identity(),
    );
    assert!(
        artifact
            .canonical_identity()
            .as_bytes()
            .starts_with(b"tiler.artifact-program.v20\0")
    );
}

#[test]
fn encoding_is_deterministic() {
    let artifact = default_artifact();
    assert_eq!(encoded(&artifact), encoded(&artifact));
    // A second, independently assembled artifact produces the same bytes.
    assert_eq!(encoded(&artifact), encoded(&default_artifact()));
}

#[test]
fn the_framing_header_is_the_fixed_width_it_declares() {
    let bytes = encoded(&default_artifact());
    assert!(bytes.len() > HEADER_BYTES);
    assert_eq!(&bytes[..MAGIC.len()], &MAGIC);
    assert_eq!(
        u16::from_be_bytes(bytes[8..10].try_into().unwrap()),
        ENVELOPE_FORMAT.0,
    );
    assert_eq!(bytes[16], DigestAlgorithm::GOVERNED.tag());
    assert_eq!(
        u64::from_be_bytes(
            bytes[TOTAL_LENGTH_AT..TOTAL_LENGTH_AT + 8]
                .try_into()
                .unwrap()
        ),
        u64::try_from(bytes.len()).unwrap(),
    );
    assert_eq!(
        &bytes[HEADER_BYTES..HEADER_BYTES + MANIFEST_DOMAIN.len()],
        MANIFEST_DOMAIN,
    );
    assert_eq!(MANIFEST_SCHEMA, (20, 0));
}

/// The canonicity backstop compares a derivation against bytes rather than
/// against a second buffer, so its ability to *refuse* has to be exercised
/// directly.
///
/// The check it replaced was `re-encoded != bytes`, which obviously compared
/// everything; a walk that compares run by run could silently skip one. Every
/// run the walk visits is perturbed here — the fixed header, the header's
/// derived manifest digest, the first and last manifest byte, a section's
/// framing, a section's content — plus the two length disagreements the walk
/// short-circuits on. Nothing else in the suite reaches this boundary: the
/// forgeries that would are rejected earlier by a named check, which is exactly
/// what `super::super::decode`'s comment about this backstop records.
#[test]
fn the_canonicity_backstop_refuses_every_run_it_walks() {
    let artifact = default_artifact();
    let envelope = envelope_of(&artifact);
    let digests: Vec<_> = envelope
        .sections()
        .iter()
        .map(|section| section_digest(DigestAlgorithm::GOVERNED, section))
        .collect();
    let identity = envelope
        .canonical_identity()
        .expect("a verified artifact derives its identity");
    let bytes = encode(&envelope).expect("a verified artifact encodes");
    let matches = |candidate: &[u8]| {
        matches_canonical_encoding(&envelope, &identity, &digests, candidate)
            .expect("the fixture envelope encodes")
    };
    assert!(
        matches(&bytes),
        "the encoder's own output is the canonical encoding",
    );

    let manifest_len = usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    assert!(
        bytes.len() > HEADER_BYTES + manifest_len,
        "the fixture frames at least one section after its manifest",
    );
    for position in [
        0,
        MANIFEST_DIGEST_AT,
        HEADER_BYTES,
        HEADER_BYTES + manifest_len - 1,
        HEADER_BYTES + manifest_len,
        bytes.len() - 1,
    ] {
        let mut forged = bytes.clone();
        forged[position] ^= 0xff;
        assert!(
            !matches(&forged),
            "a flipped byte at offset {position} was accepted as canonical",
        );
    }

    let mut short = bytes.clone();
    short.pop();
    assert!(!matches(&short), "a truncated encoding is not canonical");
    let mut long = bytes.clone();
    long.push(0);
    assert!(!matches(&long), "an extended encoding is not canonical");
}

#[test]
fn payload_and_provider_declaration_order_do_not_change_the_bytes() {
    let forward = two_variant_artifact(true);
    let reversed = two_variant_artifact(false);
    assert_eq!(forward.canonical_identity(), reversed.canonical_identity());
    assert_eq!(encoded(&forward), encoded(&reversed));
    // Declaration order genuinely differed, and is presentation-only.
    assert_ne!(forward.payloads()[0], reversed.payloads()[0]);
}

#[test]
fn expression_assembly_order_does_not_change_the_bytes() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();

    // The same formulas, minted through two different arena orders. Identity is
    // already order-independent because it writes one canonical arena and names
    // every use by canonical position; the envelope must be too, or one artifact
    // would have two byte identities and an envelope digest could not serve as a
    // cache key.
    let assemble = |reversed: bool| {
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
        draft.select_provider(selection(provider.clone())).unwrap();
        let descriptor = draft.push_payload(payload(0xa1)).unwrap();
        let formulas = if reversed {
            // The same two expressions in the opposite declaration order; the
            // variant's ABI is the program's now, so what remains under test is
            // that a caller-supplied expression's declaration order does not
            // reach identity.
            let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
            let always = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
            Formulas { one, always }
        } else {
            formulas(&mut draft)
        };
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"fused"))
            .unwrap();
        declare_realization(&mut draft, &program);
        draft.build().unwrap()
    };

    let straight = assemble(false);
    let reversed = assemble(true);
    assert_eq!(straight.canonical_identity(), reversed.canonical_identity());
    assert_eq!(encoded(&straight), encoded(&reversed));
}

#[test]
fn an_unused_environment_provider_does_not_change_the_bytes() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let selected = lowering_provider(1);
    let lean = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        std::slice::from_ref(&selected),
    );
    let crowded = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        &[selected, spare_provider(7)],
    );
    // ADR 0072 at the wire level: an offered-but-unreached provider must not
    // reach the envelope's bytes, and therefore not its digest either.
    assert_eq!(encoded(&lean), encoded(&crowded));
    assert_eq!(
        envelope_digest(&encoded(&lean)),
        envelope_digest(&encoded(&crowded)),
    );
}

#[test]
fn a_reached_provider_revision_changes_the_envelope_digest() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let available = [lowering_provider(1), lowering_provider(2)];
    let first = build_artifact(&semantic, &program, lowering_provider(1), &available);
    let second = build_artifact(&semantic, &program, lowering_provider(2), &available);
    assert_ne!(
        envelope_digest(&encoded(&first)),
        envelope_digest(&encoded(&second)),
    );
}

#[test]
fn the_envelope_digest_is_derived_and_never_stored_in_band() {
    let bytes = encoded(&default_artifact());
    let digest = envelope_digest(&bytes);
    assert!(
        !bytes.windows(digest.len()).any(|window| window == digest),
        "an in-band envelope digest would be a recursive definition",
    );
}

#[test]
fn the_derived_feature_set_names_what_a_reader_must_implement() {
    assert!(envelope_of(&default_artifact()).features().is_empty());
    let multi = envelope_of(&two_variant_artifact(true));
    assert_eq!(multi.features(), [FEATURE_MULTI_VARIANT_ROUTING.to_owned()]);
}
