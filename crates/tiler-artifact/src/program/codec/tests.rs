//! Bounded tests for the target-neutral artifact envelope codec.
//!
//! Round-tripping is the weakest evidence a codec can offer, so it is the
//! smallest part of this suite. Two stronger properties carry most of the
//! weight.
//!
//! **Canonical form.** Two artifacts with equal identity must encode to equal
//! bytes, and an envelope that is well formed but not canonical must be refused
//! rather than normalized on the way in. The order-independence cases prove the
//! first; the forged-model cases prove the second.
//!
//! **Fail-closed under a competent adversary.** Corrupting bytes and watching a
//! digest reject them proves very little, because a forger recomputes digests.
//! The forged-model cases therefore build a *structurally invalid envelope*,
//! encode it — which stamps a correct manifest digest, correct section digests,
//! and a correct identity for whatever it now says — and require the decoder to
//! reject it anyway, with the named cause. The byte-level cases separately
//! prove that an incompetent corruption cannot slip through either.

use tiler_ir::schedule::NumericalPermission;
use tiler_ir::semantic::{InputKey, ProviderIdentity};
use tiler_ir::shape::Axis;

use super::super::error::{AbiExprUse, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind};
use super::super::expr::{AbiBinaryOp, AbiRoot, AbiType, AvailabilityPhase, ExprNode};
use super::super::keys::{BackendEntryKey, BackendKey, PayloadDigest, RepresentationKey};
use super::super::model::{
    ArtifactExecutionPolicy, RoutingPolicy, SchemaVersion, address_space_from_tag,
    address_space_tag, buffer_access_from_tag, buffer_access_tag, element_type_from_tag,
    element_type_tag, permission_from_tag, permission_tag, subnormal_from_tag, subnormal_tag,
    value_role_from_tag, value_role_tag,
};
use super::super::tests::{
    Formulas, OTHER_SCALE_BITS, SCALE_BITS, build_artifact, default_artifact, formulas,
    fused_program, lowering_provider, payload, selection, semantic_program, spare_provider,
    variant,
};
use super::super::{ArtifactProgramBuilder, CompilationEnvironment, VerifiedArtifactProgram};
use super::decode::{decode, parse_expression_arena};
use super::digest::DigestAlgorithm;
use super::encode::{
    ENVELOPE_FORMAT, HEADER_BYTES, MAGIC, MANIFEST_DIGEST_DOMAIN, MANIFEST_DOMAIN, encode,
    envelope_digest,
};
use super::error::{ArtifactCodecError, OrderedSubject, ReferenceSubject, TagSubject};
use super::model::{
    ArtifactEnvelope, FEATURE_MULTI_STAGE_PROGRAM, FEATURE_MULTI_VARIANT_ROUTING, Section,
    SectionKind, position,
};
use super::payload::{
    PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadProvenance, PayloadSdkIdentity,
    PayloadTargetObligation, ToolComponent, decode_metadata,
};

// -------------------------------------------------------------------------
// Test-local helpers
// -------------------------------------------------------------------------

fn envelope_of(artifact: &VerifiedArtifactProgram) -> ArtifactEnvelope {
    ArtifactEnvelope::of(artifact).expect("a verified artifact projects")
}

fn encoded(artifact: &VerifiedArtifactProgram) -> Vec<u8> {
    encode(&envelope_of(artifact)).expect("a verified artifact encodes")
}

/// Offsets of the fields the framing header derives from the manifest bytes.
const TOTAL_LENGTH_AT: usize = 17;
const MANIFEST_LENGTH_AT: usize = 25;
const MANIFEST_DIGEST_AT: usize = 37;

/// Restores every derived framing field after a byte-level forgery.
///
/// A corruption that a digest catches proves only that the digest works. Every
/// adversarial case below reseals, so what rejects it is the check under test
/// rather than an integrity field the forger simply forgot to update.
fn reseal(bytes: &mut [u8]) {
    let total = u64::try_from(bytes.len()).expect("supported usize fits u64");
    bytes[TOTAL_LENGTH_AT..TOTAL_LENGTH_AT + 8].copy_from_slice(&total.to_be_bytes());
    let manifest_len = usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    let manifest = bytes[HEADER_BYTES..HEADER_BYTES + manifest_len].to_vec();
    let digest = DigestAlgorithm::GOVERNED.digest(MANIFEST_DIGEST_DOMAIN, &manifest);
    bytes[MANIFEST_DIGEST_AT..MANIFEST_DIGEST_AT + 32].copy_from_slice(digest.as_bytes());
}

/// Returns the absolute offset of one unique byte pattern in the manifest.
fn manifest_offset(bytes: &[u8], pattern: &[u8]) -> usize {
    let manifest_len = usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    let manifest = &bytes[HEADER_BYTES..HEADER_BYTES + manifest_len];
    let found: Vec<usize> = manifest
        .windows(pattern.len())
        .enumerate()
        .filter(|(_, window)| *window == pattern)
        .map(|(offset, _)| offset)
        .collect();
    assert_eq!(found.len(), 1, "the pattern must locate exactly one field");
    HEADER_BYTES + found[0]
}

/// Encodes a deliberately invalid envelope and returns the decoder's rejection.
///
/// The forged model is encoded through the ordinary encoder, so the bytes
/// carry a correct manifest digest, correct section digests, and the canonical
/// identity of whatever the envelope now claims. Only a semantic check can
/// reject them.
fn reject_forged(forge: impl FnOnce(&mut ArtifactEnvelope)) -> ArtifactCodecError {
    let artifact = default_artifact();
    let mut envelope = envelope_of(&artifact);
    forge(&mut envelope);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    decode(&bytes).expect_err("a forged envelope is rejected")
}

/// Assembles an artifact carrying a deferred predicate and a launch precondition.
///
/// Two forgeries need it. Re-pointing the applicability guard needs the boolean
/// literal to stay reachable from a second use site, or the arena's closure
/// obligation rejects first and the type rejection is never reached; and
/// orphaning an expression needs a subtree that exactly one use site reaches.
fn guarded_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider.clone())).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let property = draft
        .push_root(AbiRoot::TargetProperty {
            key: super::super::keys::TargetPropertyKey::new("tiler.target.max-threads").unwrap(),
            phase: AvailabilityPhase::LaunchPreflight,
        })
        .unwrap();
    let predicate = draft
        .push_binary(AbiBinaryOp::LessOrEqual, formulas.one, property)
        .unwrap();
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].launch.preconditions = vec![formulas.always];
    spec.deferred_predicates = vec![super::super::DeferredPredicateSpec {
        predicate,
        phase: AvailabilityPhase::LaunchPreflight,
        authority: provider,
    }];
    draft.push_variant(&program, spec).unwrap();
    draft.build().unwrap()
}

/// Encodes a deliberately invalid forgery of [`guarded_artifact`].
fn reject_guarded_forgery(forge: impl FnOnce(&mut ArtifactEnvelope)) -> ArtifactCodecError {
    let artifact = guarded_artifact();
    let mut envelope = envelope_of(&artifact);
    forge(&mut envelope);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    decode(&bytes).expect_err("a forged envelope is rejected")
}

/// Assembles a two-variant, two-payload artifact for order-sensitive cases.
fn two_variant_artifact(forward: bool) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let first = fused_program(&semantic, SCALE_BITS);
    let second = fused_program(&semantic, OTHER_SCALE_BITS);
    let providers = [lowering_provider(1), lowering_provider(2)];
    let environment = CompilationEnvironment::new(providers.iter().cloned()).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let (left, right) = if forward { (0, 1) } else { (1, 0) };
    draft
        .select_provider(selection(providers[left].clone()))
        .unwrap();
    draft
        .select_provider(selection(providers[right].clone()))
        .unwrap();
    let (primary, spare) = if forward {
        let primary = draft.push_payload(payload(0x01)).unwrap();
        (primary, draft.push_payload(payload(0x02)).unwrap())
    } else {
        let spare = draft.push_payload(payload(0x02)).unwrap();
        (draft.push_payload(payload(0x01)).unwrap(), spare)
    };
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&first, variant(&formulas, primary, b"fused"))
        .unwrap();
    draft
        .push_variant(&second, variant(&formulas, spare, b"alternate"))
        .unwrap();
    draft.build().unwrap()
}

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
    // already order-independent because it cross-references by content key; the
    // envelope must be too, or one artifact would have two byte identities and
    // an envelope digest could not serve as a cache key.
    let assemble = |reversed: bool| {
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
        draft.select_provider(selection(provider.clone())).unwrap();
        let descriptor = draft.push_payload(payload(0xa1)).unwrap();
        let key = InputKey::new("input").unwrap();
        let formulas = if reversed {
            let always = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
            let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
            let width = draft.push_root(AbiRoot::UnsignedLiteral(4)).unwrap();
            let columns = draft
                .push_root(AbiRoot::InputExtent {
                    key: key.clone(),
                    axis: Axis::new(1),
                })
                .unwrap();
            let rows = draft
                .push_root(AbiRoot::InputExtent {
                    key,
                    axis: Axis::new(0),
                })
                .unwrap();
            let output_bytes = draft
                .push_binary(AbiBinaryOp::CheckedMultiply, rows, width)
                .unwrap();
            let elements = draft
                .push_binary(AbiBinaryOp::CheckedMultiply, rows, columns)
                .unwrap();
            let input_bytes = draft
                .push_binary(AbiBinaryOp::CheckedMultiply, elements, width)
                .unwrap();
            Formulas {
                rows,
                input_bytes,
                output_bytes,
                one,
                always,
            }
        } else {
            formulas(&mut draft)
        };
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"fused"))
            .unwrap();
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

// -------------------------------------------------------------------------
// One tag table per vocabulary
// -------------------------------------------------------------------------

#[test]
fn every_governed_tag_table_round_trips() {
    use tiler_ir::kernel::{AddressSpace, BufferAccess, KernelType};
    use tiler_ir::program::ValueRole;
    use tiler_ir::schedule::{FlushedZeroSign, SubnormalMode};

    for value in [KernelType::Bool, KernelType::Index, KernelType::F32] {
        assert_eq!(
            element_type_from_tag(element_type_tag(value).unwrap()),
            Some(value),
        );
    }
    for value in [
        AddressSpace::Device,
        AddressSpace::Workgroup,
        AddressSpace::InvocationPrivate,
        AddressSpace::Constant,
    ] {
        assert_eq!(
            address_space_from_tag(address_space_tag(value).unwrap()),
            Some(value),
        );
    }
    for value in [BufferAccess::Read, BufferAccess::Write] {
        assert_eq!(
            buffer_access_from_tag(buffer_access_tag(value).unwrap()),
            Some(value),
        );
    }
    for value in [ValueRole::Input, ValueRole::Temporary, ValueRole::Output] {
        assert_eq!(value_role_from_tag(value_role_tag(value)), Some(value));
    }
    // Both flush behaviours are enumerated: they name different zeros, so a
    // shared tag would decode one as the other.
    for value in [
        SubnormalMode::Preserve,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        },
    ] {
        assert_eq!(subnormal_from_tag(subnormal_tag(value)), Some(value));
    }
    for value in [
        NumericalPermission::Forbidden,
        NumericalPermission::Permitted,
    ] {
        assert_eq!(permission_from_tag(permission_tag(value)), Some(value));
    }
    assert_eq!(
        RoutingPolicy::from_tag(RoutingPolicy::StablePriority.tag()),
        Some(RoutingPolicy::StablePriority),
    );
    assert_eq!(
        SectionKind::from_tag(SectionKind::KernelProgramSubject.tag()),
        Some(SectionKind::KernelProgramSubject),
    );
    for phase in [
        AvailabilityPhase::CompileProfile,
        AvailabilityPhase::ArtifactEvidence,
        AvailabilityPhase::LiveDevicePreflight,
        AvailabilityPhase::PreparedKernelPreflight,
        AvailabilityPhase::LaunchPreflight,
    ] {
        assert_eq!(AvailabilityPhase::from_tag(phase.tag()), Some(phase));
    }
}

// -------------------------------------------------------------------------
// Byte-level corruption
// -------------------------------------------------------------------------

/// Sweeps single-byte corruptions of one encoded envelope.
///
/// **The framing header and the framed section stream are swept exhaustively.**
/// Those are the regions whose fields are read *before* any digest can speak for
/// them — the magic, the versions, the digest algorithm, the declared lengths
/// and counts, each section identifier and length, and each section's exact
/// bytes — so every one of them is worth its own case.
///
/// **The manifest interior is sampled at a stride of 61.** Every byte of it is
/// covered by one digest over the whole run, so flipping each of its 25,000
/// bytes exercises the same check 25,000 times rather than 25,000 different
/// ones; the sample is there to show the digest really covers the whole run
/// rather than a prefix. The stride is prime and therefore coprime with every
/// field width the encoding uses — one, two, four, eight, and thirty-two bytes
/// — so the sample lands on every byte offset of every field width instead of
/// repeatedly on the same one. The boundary is recorded here rather than left
/// as an unexplained loop bound.
#[test]
fn single_byte_corruptions_are_rejected() {
    let bytes = encoded(&default_artifact());
    let manifest_len = usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    let manifest_end = HEADER_BYTES + manifest_len;
    let swept = (0..HEADER_BYTES)
        .chain((HEADER_BYTES..manifest_end).step_by(61))
        .chain(manifest_end..bytes.len());
    let mut forged = bytes.clone();
    for index in swept {
        forged[index] ^= 0xff;
        assert!(
            decode(&forged).is_err(),
            "flipping byte {index} produced an accepted envelope",
        );
        forged[index] = bytes[index];
    }
}

#[test]
fn every_truncation_is_rejected() {
    let bytes = encoded(&default_artifact());
    for length in 0..bytes.len() {
        assert!(
            decode(&bytes[..length]).is_err(),
            "a {length}-byte prefix decoded",
        );
    }
    // The two boundaries the reader distinguishes: too short to hold a header,
    // and a complete header whose declared total length is not what arrived.
    assert!(matches!(
        decode(&bytes[..4]),
        Err(ArtifactCodecError::Truncated { .. }),
    ));
    assert!(matches!(
        decode(&bytes[..bytes.len() - 1]),
        Err(ArtifactCodecError::TotalLengthMismatch { .. }),
    ));
}

#[test]
fn trailing_bytes_are_rejected() {
    let bytes = encoded(&default_artifact());
    let mut extended = bytes.clone();
    extended.push(0x00);
    assert!(matches!(
        decode(&extended),
        Err(ArtifactCodecError::TotalLengthMismatch { .. }),
    ));
    // Repairing the declared length does not make the extra byte admissible.
    reseal(&mut extended);
    assert_eq!(
        decode(&extended),
        Err(ArtifactCodecError::TrailingBytes { count: 1 }),
    );
}

#[test]
fn a_corrupted_manifest_is_rejected_before_it_is_parsed() {
    let mut bytes = encoded(&default_artifact());
    bytes[HEADER_BYTES + MANIFEST_DOMAIN.len()] ^= 0xff;
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ManifestDigestMismatch)
    );
}

#[test]
fn a_corrupted_section_is_rejected_by_its_own_digest() {
    let mut bytes = encoded(&default_artifact());
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    // The manifest is resealed, so the section descriptor's digest is the only
    // thing standing between the forgery and acceptance.
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::SectionDigestMismatch { section: 0 }),
    );
}

#[test]
fn a_flipped_tag_byte_is_rejected_by_name() {
    let mut bytes = encoded(&default_artifact());
    // The routing tag directly follows the manifest domain and the manifest and
    // component schema versions.
    let routing_at = HEADER_BYTES + MANIFEST_DOMAIN.len() + 4 + 16;
    assert_eq!(bytes[routing_at], RoutingPolicy::StablePriority.tag());
    bytes[routing_at] = 0x7f;
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::RoutingPolicy,
            tag: 0x7f,
        }),
    );
}

#[test]
fn a_length_prefix_that_disagrees_with_its_payload_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    // The semantic graph subject is the first length-prefixed run after the
    // (empty) feature list, which the routing tag precedes.
    let length_at = HEADER_BYTES + MANIFEST_DOMAIN.len() + 4 + 16 + 1 + 8;
    let declared = u64::from_be_bytes(bytes[length_at..length_at + 8].try_into().unwrap());
    assert!(declared > 0, "the fixture carries a semantic graph subject");
    bytes[length_at..length_at + 8].copy_from_slice(&(declared + 4_096).to_be_bytes());
    reseal(&mut bytes);
    assert!(matches!(
        decode(&bytes),
        Err(ArtifactCodecError::Truncated { .. }),
    ));
}

#[test]
fn an_unknown_envelope_format_version_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    bytes[8..10].copy_from_slice(&2_u16.to_be_bytes());
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnsupportedEnvelopeFormat { major: 2, minor: 0 }),
    );
    let mut newer_minor = encoded(&default_artifact());
    newer_minor[10..12].copy_from_slice(&9_u16.to_be_bytes());
    reseal(&mut newer_minor);
    assert_eq!(
        decode(&newer_minor),
        Err(ArtifactCodecError::UnsupportedEnvelopeFormat { major: 1, minor: 9 }),
    );
}

#[test]
fn an_unknown_digest_algorithm_is_rejected_rather_than_inferred() {
    let mut bytes = encoded(&default_artifact());
    bytes[16] = 0x02;
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnsupportedDigestAlgorithm { tag: 0x02 }),
    );
}

#[test]
fn an_unknown_manifest_or_component_schema_is_rejected() {
    let schema_at = HEADER_BYTES + MANIFEST_DOMAIN.len();
    let mut manifest = encoded(&default_artifact());
    manifest[schema_at + 2..schema_at + 4].copy_from_slice(&5_u16.to_be_bytes());
    reseal(&mut manifest);
    assert_eq!(
        decode(&manifest),
        Err(ArtifactCodecError::UnsupportedManifestSchema { major: 1, minor: 5 }),
    );

    let mut component = encoded(&default_artifact());
    component[schema_at + 4..schema_at + 6].copy_from_slice(&3_u16.to_be_bytes());
    reseal(&mut component);
    assert_eq!(
        decode(&component),
        Err(ArtifactCodecError::UnsupportedComponentSchema {
            component: super::error::ComponentSchemaKind::Program,
            major: 3,
            minor: 0,
        }),
    );
}

#[test]
fn an_unsupported_required_feature_is_rejected() {
    // The feature list is derived, so an unreadable requirement can only arrive
    // from a producer this reader does not implement. Naming the reserved
    // multi-stage feature is the exact case that is expected to appear first.
    let mut envelope = envelope_of(&default_artifact());
    envelope.features = vec![FEATURE_MULTI_STAGE_PROGRAM.to_owned()];
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnsupportedRequiredFeature {
            feature: FEATURE_MULTI_STAGE_PROGRAM.to_owned(),
        }),
    );
}

#[test]
fn a_forged_identity_is_rejected() {
    let bytes = encoded(&default_artifact());
    let identity = default_artifact().canonical_identity().as_bytes().to_vec();
    let at = manifest_offset(&bytes, &identity);
    let mut forged = bytes;
    forged[at] ^= 0xff;
    reseal(&mut forged);
    assert_eq!(
        decode(&forged),
        Err(ArtifactCodecError::ArtifactIdentityMismatch),
    );
}

// -------------------------------------------------------------------------
// Forged models: a competent adversary who reseals everything
// -------------------------------------------------------------------------

#[test]
fn a_repeated_interface_key_is_rejected() {
    // Interface order is meaning rather than canonical, so the obligation is
    // distinctness alone: a runtime binds by key, and identity encodes the
    // interface positionally and would fold a repeat without complaint.
    assert_eq!(
        reject_forged(|envelope| {
            let repeat = envelope.inputs[0].clone();
            envelope.inputs.push(repeat);
        }),
        ArtifactCodecError::DuplicateItem {
            subject: OrderedSubject::InterfaceKey,
        },
    );
}

#[test]
fn an_unreferenced_section_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.sections.push(Section {
                kind: SectionKind::KernelProgramSubject,
                bytes: vec![0xff; 8],
            });
        }),
        ArtifactCodecError::UnreferencedSection { section: 1 },
    );
}

#[test]
fn a_non_canonical_section_order_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.sections.insert(
                0,
                Section {
                    kind: SectionKind::KernelProgramSubject,
                    bytes: vec![0xff; 8],
                },
            );
            for variant in &mut envelope.variants {
                variant.program_section = 1;
            }
        }),
        ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::Section,
        },
    );
}

#[test]
fn a_declared_feature_set_that_content_does_not_imply_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.features = vec![FEATURE_MULTI_VARIANT_ROUTING.to_owned()];
        }),
        ArtifactCodecError::DeclaredFeatureMismatch,
    );
}

#[test]
fn an_expression_no_use_site_reaches_is_rejected() {
    // Dropping the only use site orphans the predicate's whole subtree. The
    // arena is left untouched, so canonical order and typing still hold and
    // reachability is the only obligation that can reject.
    assert_eq!(
        reject_guarded_forgery(|envelope| {
            envelope.variants[0].deferred.clear();
            // The forgery is otherwise self-consistent: dropping the predicate
            // also drops the feature its presence derived.
            envelope.features = envelope.derived_features();
        }),
        ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::UnusedExpression,
        },
    );
}

#[test]
fn a_non_canonical_expression_order_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            // Positions 0 and 1 are roots, which the canonical order emits
            // smallest-key first; swapping them keeps the arena acyclic and
            // well typed while making its order non-canonical.
            assert!(matches!(envelope.expressions[0], ExprNode::Root(_)));
            assert!(matches!(envelope.expressions[1], ExprNode::Root(_)));
            envelope.expressions.swap(0, 1);
            swap_expression_references(envelope, 0, 1);
        }),
        ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::Expression,
        },
    );
}

/// Rewrites every reference to two swapped arena positions.
fn swap_expression_references(envelope: &mut ArtifactEnvelope, left: u32, right: u32) {
    let swap = |node: &mut u32| {
        if *node == left {
            *node = right;
        } else if *node == right {
            *node = left;
        }
    };
    for node in &mut envelope.expressions {
        match node {
            ExprNode::Root(_) => {}
            ExprNode::Unary { operand, .. } => swap(operand),
            ExprNode::Binary { left, right, .. } => {
                swap(left);
                swap(right);
            }
            ExprNode::Select {
                condition,
                if_true,
                if_false,
            } => {
                swap(condition);
                swap(if_true);
                swap(if_false);
            }
        }
    }
    for variant in &mut envelope.variants {
        swap(&mut variant.guard);
        for predicate in &mut variant.deferred {
            swap(&mut predicate.predicate);
        }
        for entry in &mut variant.entries {
            for binding in &mut entry.bindings {
                swap(&mut binding.accessible_bytes);
            }
            swap(&mut entry.launch.grid_threads);
            swap(&mut entry.launch.threads_per_workgroup);
            for precondition in &mut entry.launch.preconditions {
                swap(precondition);
            }
        }
    }
}

#[test]
fn an_empty_portfolio_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| envelope.variants.clear()),
        ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::EmptyPortfolio,
        },
    );
}

#[test]
fn an_unattributed_plan_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| envelope.providers.clear()),
        ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::MissingSelectedProvider,
        },
    );
}

#[test]
fn a_payload_no_entry_realizes_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            let mut spare = envelope.payloads[0].clone();
            // An equal-length, strictly greater digest keeps the payload table
            // in canonical key order, so only the reference closure can reject.
            spare.digest =
                super::super::keys::PayloadDigest::from_bytes([0xff, 0xff, 0xff]).unwrap();
            envelope.payloads.push(spare);
        }),
        ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::UnusedPayload,
        },
    );
}

#[test]
fn a_duplicate_selected_provider_cannot_even_be_encoded() {
    let artifact = default_artifact();
    let mut envelope = envelope_of(&artifact);
    let repeated = envelope.providers[0].clone();
    envelope.providers.push(repeated);
    assert_eq!(
        encode(&envelope),
        Err(ArtifactCodecError::IdentityDerivation {
            cause: ArtifactDiagnostic::AmbiguousCanonicalKey {
                entity: ArtifactEntityKind::Provider,
            },
        }),
    );
}

#[test]
fn a_non_canonical_provider_order_is_rejected() {
    let artifact = two_variant_artifact(true);
    let mut envelope = envelope_of(&artifact);
    envelope.providers.swap(0, 1);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::Provider,
        }),
    );
}

#[test]
fn a_non_canonical_payload_order_is_rejected() {
    let artifact = two_variant_artifact(true);
    let mut envelope = envelope_of(&artifact);
    envelope.payloads.swap(0, 1);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::Payload,
        }),
    );
}

#[test]
fn two_entries_claiming_one_backend_entry_are_rejected() {
    let artifact = two_variant_artifact(true);
    let mut envelope = envelope_of(&artifact);
    let shared = envelope.variants[0].entries[0].entry_key.clone();
    let payload = envelope.variants[0].entries[0].payload;
    envelope.variants[1].entries[0].entry_key = shared;
    envelope.variants[1].entries[0].payload = payload;
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::DuplicateBackendEntry,
        }),
    );
}

#[test]
fn a_guard_that_is_not_a_predicate_is_rejected() {
    assert_eq!(
        reject_guarded_forgery(|envelope| {
            let unsigned = envelope
                .expressions
                .iter()
                .position(|node| matches!(node, ExprNode::Root(AbiRoot::UnsignedLiteral(_))))
                .expect("the fixture holds an unsigned literal");
            envelope.variants[0].guard = u32::try_from(unsigned).expect("a bounded arena fits u32");
        }),
        ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::ExpressionType {
                use_site: AbiExprUse::ApplicabilityGuard,
                expected: AbiType::Boolean,
                actual: AbiType::Unsigned,
            }),
        },
    );
}

#[test]
fn a_launch_that_contradicts_the_declared_requirements_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0]
                .resources
                .threads_per_workgroup = 8;
        }),
        ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::LaunchDisagreement {
                entry: 0,
                expected: 8,
                actual: 1,
            }),
        },
    );
}

#[test]
fn a_deferred_authority_that_was_never_selected_is_rejected() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider.clone())).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.deferred_predicates = vec![super::super::DeferredPredicateSpec {
        predicate: formulas.always,
        phase: AvailabilityPhase::LaunchPreflight,
        authority: provider,
    }];
    draft.push_variant(&program, spec).unwrap();
    let artifact = draft.build().unwrap();

    let mut envelope = envelope_of(&artifact);
    let impostor = ProviderIdentity::new("tiler-test", "impostor", 1).unwrap();
    envelope.variants[0].deferred[0].authority = impostor.clone();
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::UnselectedDeferredAuthority {
                provider: Box::new(impostor),
            }),
        }),
    );
}

#[test]
fn a_backend_entry_key_that_no_longer_matches_its_identity_is_rejected() {
    // The identity is recomputed for the forged content, so this proves the
    // *round trip* rather than the identity check: a forged entry key survives
    // encoding and produces a different, self-consistent artifact.
    let artifact = default_artifact();
    let mut envelope = envelope_of(&artifact);
    envelope.variants[0].entries[0].entry_key = BackendEntryKey::from_bytes(b"other").unwrap();
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    let decoded = decode(&bytes).expect("a self-consistent forgery decodes");
    assert_ne!(
        decoded.canonical_identity().unwrap(),
        *artifact.canonical_identity(),
        "changing a packaged fact must change the artifact's identity",
    );
}

// -------------------------------------------------------------------------
// Expression arena, driven directly
// -------------------------------------------------------------------------

/// Encodes one arena count and a raw node body for the arena parser.
fn arena_bytes(count: u64, body: &[u8]) -> Vec<u8> {
    let mut bytes = count.to_be_bytes().to_vec();
    bytes.extend_from_slice(body);
    bytes
}

#[test]
fn an_operand_that_does_not_precede_its_node_is_rejected() {
    // node 0: unsigned literal 1. node 1: a binary node naming itself.
    let mut body = vec![0x01, 0x01];
    body.extend_from_slice(&1_u64.to_be_bytes());
    body.extend_from_slice(&[0x03, 0x01]);
    body.extend_from_slice(&1_u32.to_be_bytes());
    body.extend_from_slice(&0_u32.to_be_bytes());
    assert_eq!(
        parse_expression_arena(&arena_bytes(2, &body)),
        Err(ArtifactCodecError::ExpressionOperandOrder {
            node: 1,
            operand: 1,
        }),
    );
}

#[test]
fn a_mistyped_expression_operand_is_rejected() {
    // node 0: boolean literal. node 1: checked add over it.
    let mut body = vec![0x01, 0x02, 0x01];
    body.extend_from_slice(&[0x03, 0x01]);
    body.extend_from_slice(&0_u32.to_be_bytes());
    body.extend_from_slice(&0_u32.to_be_bytes());
    assert_eq!(
        parse_expression_arena(&arena_bytes(2, &body)),
        Err(ArtifactCodecError::ExpressionOperandType {
            node: 1,
            expected: AbiType::Unsigned,
            actual: AbiType::Boolean,
        }),
    );
}

#[test]
fn a_conditional_selection_with_disagreeing_branches_is_rejected() {
    // node 0: boolean literal. node 1: unsigned literal. node 2: select.
    let mut body = vec![0x01, 0x02, 0x01, 0x01, 0x01];
    body.extend_from_slice(&7_u64.to_be_bytes());
    body.push(0x04);
    body.extend_from_slice(&0_u32.to_be_bytes());
    body.extend_from_slice(&1_u32.to_be_bytes());
    body.extend_from_slice(&0_u32.to_be_bytes());
    assert_eq!(
        parse_expression_arena(&arena_bytes(3, &body)),
        Err(ArtifactCodecError::ExpressionSelectBranchType {
            node: 2,
            if_true: AbiType::Unsigned,
            if_false: AbiType::Boolean,
        }),
    );
}

#[test]
fn a_repeated_expression_node_is_rejected() {
    let mut body = Vec::new();
    for _ in 0..2 {
        body.push(0x01);
        body.push(0x01);
        body.extend_from_slice(&5_u64.to_be_bytes());
    }
    assert_eq!(
        parse_expression_arena(&arena_bytes(2, &body)),
        Err(ArtifactCodecError::DuplicateItem {
            subject: OrderedSubject::Expression,
        }),
    );
}

#[test]
fn an_unknown_expression_node_or_root_tag_is_rejected() {
    assert_eq!(
        parse_expression_arena(&arena_bytes(1, &[0x7f])),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::ExpressionNode,
            tag: 0x7f,
        }),
    );
    assert_eq!(
        parse_expression_arena(&arena_bytes(1, &[0x01, 0x7f])),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::ExpressionRoot,
            tag: 0x7f,
        }),
    );
}

#[test]
fn an_arena_count_beyond_its_budget_is_rejected_before_allocation() {
    let error = parse_expression_arena(&arena_bytes(u64::MAX, &[]))
        .expect_err("an absurd count is refused");
    assert!(matches!(
        error,
        ArtifactCodecError::Limit {
            resource: super::error::CodecLimitKind::Expressions,
            ..
        },
    ));
}

#[test]
fn an_expression_reference_outside_the_arena_is_rejected() {
    let artifact = default_artifact();
    let envelope = envelope_of(&artifact);
    let beyond = u32::try_from(envelope.expressions.len()).unwrap();
    // Encoding reads the guard's content key, so the forgery is applied to the
    // encoded bytes rather than to the model.
    let bytes = encode(&envelope).expect("the envelope encodes");
    // The variant record spells its guard immediately before its length-
    // prefixed target-profile key, which makes the pair a unique locator.
    let profile_key = envelope.variants[0].profile.key.as_str().as_bytes();
    let mut needle = envelope.variants[0].guard.to_be_bytes().to_vec();
    needle.extend_from_slice(
        &u64::try_from(profile_key.len())
            .expect("supported usize fits u64")
            .to_be_bytes(),
    );
    needle.extend_from_slice(profile_key);
    let at = manifest_offset(&bytes, &needle);
    let mut forged = bytes;
    forged[at..at + 4].copy_from_slice(&beyond.to_be_bytes());
    reseal(&mut forged);
    assert_eq!(
        decode(&forged),
        Err(ArtifactCodecError::MissingReference {
            subject: ReferenceSubject::Expression,
            index: u64::from(beyond),
        }),
    );
}

// -------------------------------------------------------------------------
// Carried backend payloads
// -------------------------------------------------------------------------

/// Builds one carried payload's compilation subject.
///
/// The provenance is deliberately not Metal-shaped beyond its values: the same
/// record shape holds a CUDA payload's `nvcc`, `ptxas`, and `sm_90`.
fn payload_metadata(source: &[u8]) -> PayloadMetadata {
    PayloadMetadata {
        source_representation: RepresentationKey::new("metal-source").unwrap(),
        source: source.to_vec(),
        provenance: PayloadProvenance {
            toolchain: "tiler.toolchain.apple-metal".to_owned(),
            target: "air64-apple-macosx26.0".to_owned(),
            family: "apple-macos".to_owned(),
            language: "metal3.2".to_owned(),
            deployment_major: 26,
            deployment_minor: 0,
            components: vec![
                ToolComponent {
                    role: "compiler".to_owned(),
                    version: "32023.883".to_owned(),
                },
                ToolComponent {
                    role: "linker".to_owned(),
                    version: "32023.883".to_owned(),
                },
            ],
            sdk: PayloadSdkIdentity {
                name: "macosx".to_owned(),
                version: "26.0".to_owned(),
                build: "26A5388g".to_owned(),
            },
            compile_flags: vec![
                "-fmetal-math-mode=safe".to_owned(),
                "-ffp-contract=off".to_owned(),
            ],
            link_flags: Vec::new(),
        },
        entries: vec![PayloadEntryMapping {
            entry_key: BackendEntryKey::from_bytes(b"fused").unwrap(),
            symbol: "tiler_fused_0".to_owned(),
            transports: vec![0, 1],
        }],
        obligations: vec![PayloadTargetObligation {
            key: "tiler.target.subnormal-arithmetic".to_owned(),
            value: "flushes-to-zero".to_owned(),
        }],
    }
}

fn payload_content(source: &[u8], code: &[u8]) -> PayloadContent {
    PayloadContent {
        metadata: payload_metadata(source),
        code: code.to_vec(),
    }
}

/// Assembles a one-variant artifact whose single payload carries its object.
fn carried_artifact(source: &[u8], code: &[u8]) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft
        .push_carried_payload(
            BackendKey::new("tiler.metal").unwrap(),
            RepresentationKey::new("metallib").unwrap(),
            SchemaVersion::new(1, 0),
            ArtifactExecutionPolicy::RequiresDeviceTranslation,
            payload_content(source, code),
        )
        .unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    draft.build().unwrap()
}

#[test]
fn a_carried_payload_round_trips_with_its_object_and_its_subject() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"\x00metallib\xff");
    let bytes = encoded(&artifact);
    let decoded = decode(&bytes).expect("a carried envelope decodes");
    assert_eq!(decoded, envelope_of(&artifact));
    assert_eq!(decoded.payload_content().len(), 1);
    let sections = decoded.payload_content()[0].expect("the payload is carried");
    assert_eq!(
        decoded.sections()[position(sections.code)].kind,
        SectionKind::BackendPayloadCode,
    );
    assert_eq!(
        decoded.sections()[position(sections.code)].bytes,
        b"\x00metallib\xff",
    );
    assert_eq!(
        decode_metadata(&decoded.sections()[position(sections.metadata)].bytes)
            .expect("the carried subject decodes"),
        payload_metadata(b"kernel void fused() {}"),
    );
}

/// The descriptor's digest is the identity of the subject, not of the object.
///
/// This is the identity decision stated as a test: changing a compilation input
/// is a different artifact, and changing only the emitted bytes is not.
#[test]
fn payload_identity_follows_the_compilation_subject_and_not_the_object() {
    let baseline = carried_artifact(b"kernel void fused() {}", b"first-link");
    let relinked = carried_artifact(b"kernel void fused() {}", b"second-link");
    let recompiled = carried_artifact(b"kernel void other() {}", b"first-link");

    assert_eq!(
        baseline.canonical_identity(),
        relinked.canonical_identity(),
        "a non-reproducible linker must not change what artifact this is",
    );
    assert_ne!(
        encoded(&baseline),
        encoded(&relinked),
        "the object still travels, so the two encodings differ",
    );
    assert_ne!(
        baseline.canonical_identity(),
        recompiled.canonical_identity(),
        "a different compilation subject is a different artifact",
    );
}

#[test]
fn carrying_a_payload_requires_the_governed_feature() {
    let carried = carried_artifact(b"kernel void fused() {}", b"link");
    assert!(
        envelope_of(&carried)
            .features()
            .contains(&"tiler.artifact.feature.embedded-payload-code".to_owned()),
    );
    assert!(
        !envelope_of(&default_artifact())
            .features()
            .contains(&"tiler.artifact.feature.embedded-payload-code".to_owned()),
        "a descriptor-only artifact must not require a reader to implement carrying",
    );
}

/// A code reference pointed at a compilation subject must be refused by name.
///
/// Both sections exist, both digests verify, and the manifest digest is
/// restamped by the encoder, so nothing in framing or integrity can catch this.
/// Only the purpose check can, which is why the reference carries one.
#[test]
fn a_payload_section_reference_of_the_wrong_purpose_is_rejected() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"link");
    let mut envelope = envelope_of(&artifact);
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    envelope.payload_content[0] = Some(super::model::PayloadSections {
        metadata: sections.metadata,
        code: sections.metadata,
    });
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::SectionPurposeMismatch {
            section: sections.metadata,
            expected: SectionKind::BackendPayloadCode.tag(),
            actual: SectionKind::BackendPayloadMetadata.tag(),
        }),
    );
}

/// A descriptor that claims a subject it does not carry must be refused.
#[test]
fn a_payload_digest_that_is_not_its_carried_subject_is_rejected() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"link");
    let mut envelope = envelope_of(&artifact);
    envelope.payloads[0].digest = PayloadDigest::from_bytes([0x01, 0x02, 0x03]).unwrap();
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::PayloadIdentityMismatch { payload: 0 }),
    );
}

/// A carried subject that does not parse is refused, not carried opaquely.
#[test]
fn a_carried_subject_that_is_not_payload_metadata_is_rejected() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"link");
    let mut envelope = envelope_of(&artifact);
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    // At least as long as the versioned domain tag, so the case decides on the
    // tag rather than being pre-empted by a truncation.
    envelope.sections[position(sections.metadata)].bytes = vec![b'x'; 512];
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::BadPayloadMetadataDomain),
    );
}

/// A non-canonical collection inside a carried subject is refused.
#[test]
fn a_non_canonical_carried_subject_collection_is_rejected() {
    let mut metadata = payload_metadata(b"kernel void fused() {}");
    metadata.provenance.components.reverse();
    assert_eq!(
        decode_metadata(&super::payload::encode_metadata(&metadata)),
        Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::ProvenanceComponent,
        }),
    );
}

/// Compiler flag order is meaning and must survive a round trip unsorted.
#[test]
fn carried_flag_order_is_retained_rather_than_canonicalized() {
    let mut metadata = payload_metadata(b"kernel void fused() {}");
    metadata.provenance.compile_flags = vec![
        "-O2".to_owned(),
        "-ffast-math".to_owned(),
        "-fno-fast-math".to_owned(),
    ];
    let decoded = decode_metadata(&super::payload::encode_metadata(&metadata))
        .expect("declared flag order decodes");
    assert_eq!(
        decoded.provenance.compile_flags,
        [
            "-O2".to_owned(),
            "-ffast-math".to_owned(),
            "-fno-fast-math".to_owned(),
        ]
    );
}
