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

use std::time::Instant;

use tiler_ir::kernel::{AddressSpace, BufferAccess, KernelType};
use tiler_ir::schedule::{ExceptionalValueAssumption, NumericalPermission, ValueDomainProvenance};
use tiler_ir::semantic::{InputKey, OutputKey, ProviderIdentity};
use tiler_ir::shape::Axis;

use super::super::facts::AbiFactBinder;

use super::super::error::{AbiExprUse, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind};
use super::super::expr::{AbiBinaryOp, AbiRoot, AbiType, AbiValue, AvailabilityPhase, ExprNode};
use super::super::handles::PayloadId;
use super::super::keys::{
    BackendEntryKey, BackendKey, PayloadDigest, RepresentationKey, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef,
};
use super::super::model::{
    ArtifactExecutionPolicy, BINDING_TARGET_INTERNAL, BINDING_TARGET_PROGRAM_INPUT,
    BINDING_TARGET_PROGRAM_OUTPUT, BindingKind, BindingTarget, BindingTargetData, RoutingPolicy,
    SchemaVersion, StageDependencyData, StageDependencyReason, address_space_from_tag,
    address_space_tag, buffer_access_from_tag, buffer_access_tag, element_type_from_tag,
    element_type_tag, exceptional_assumption_from_tag, exceptional_assumption_tag,
    permission_from_tag, permission_tag, subnormal_from_tag, subnormal_tag,
};
use super::super::tests::{
    ELEMENT_BYTES, Formulas, OTHER_SCALE_BITS, SCALE_BITS, SCRATCH_OFFSET, build_artifact,
    default_artifact, formulas, fused_program, lowering_provider, partial_window_artifact, payload,
    profile, selection, semantic_program, spare_provider, variant,
};
use super::super::{
    ArtifactProgramBuilder, CompilationEnvironment, MAX_VARIANT_ENTRIES, VerifiedArtifactProgram,
};
use super::decode::{Cursor, decode, parse_dependencies, parse_expression_arena};
use super::digest::DigestAlgorithm;
use super::encode::{
    ENVELOPE_FORMAT, HEADER_BYTES, MAGIC, MANIFEST_DIGEST_DOMAIN, MANIFEST_DOMAIN, MANIFEST_SCHEMA,
    encode, envelope_digest, section_digest,
};
use super::error::{
    ArtifactCodecError, CodecLimitKind, OrderedSubject, ReferenceSubject, TagSubject,
};
use super::model::{
    ArtifactEnvelope, FEATURE_MULTI_STAGE_PROGRAM, FEATURE_MULTI_VARIANT_ROUTING, Section,
    SectionDisposition, SectionKind, position,
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
            key: super::super::expr::TargetPropertyKey::new("tiler.target.max-threads").unwrap(),
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
        assert_eq!(element_type_from_tag(element_type_tag(value)), Some(value),);
    }
    for value in [
        AddressSpace::Device,
        AddressSpace::Workgroup,
        AddressSpace::InvocationPrivate,
        AddressSpace::Constant,
    ] {
        assert_eq!(
            address_space_from_tag(address_space_tag(value)),
            Some(value),
        );
    }
    for value in [BufferAccess::Read, BufferAccess::Write] {
        assert_eq!(
            buffer_access_from_tag(buffer_access_tag(value)),
            Some(value),
        );
    }
    // A binding target carries data, so its inverse is the decoder rather than a
    // tag table. What a tag table would have pinned is pinned directly: the three
    // governed tags are pairwise distinct, and the program role each target
    // implies is distinct too — a collision in either would let a decoder read one
    // dispatch instruction as another.
    let tags = [
        BINDING_TARGET_PROGRAM_INPUT,
        BINDING_TARGET_PROGRAM_OUTPUT,
        BINDING_TARGET_INTERNAL,
    ];
    let mut sorted = tags;
    sorted.sort_unstable();
    assert!(
        sorted.windows(2).all(|pair| pair[0] != pair[1]),
        "the governed binding-target tags must be pairwise distinct",
    );
    for (target, role) in [
        (
            BindingTargetData::ProgramInput(InputKey::new("input").unwrap()),
            ValueRole::Input,
        ),
        (
            BindingTargetData::ProgramOutput(vec![OutputKey::new("result").unwrap()]),
            ValueRole::Output,
        ),
        (BindingTargetData::Internal, ValueRole::Temporary),
    ] {
        assert_eq!(target.value_role(), role);
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
    for value in [
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CompilerProven,
        },
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::RuntimeValidated,
        },
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
        },
    ] {
        assert_eq!(
            exceptional_assumption_from_tag(exceptional_assumption_tag(value)),
            Some(value),
        );
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

/// Sweeps single-byte corruptions of one encoded envelope, exhaustively.
///
/// **Every byte, with no sampling anywhere.** The header and the framed section
/// stream carry fields read *before* any digest can speak for them — the magic,
/// the versions, the digest algorithm, the declared lengths and counts, each
/// section identifier and length, and each section's exact bytes — and the
/// manifest interior is covered by one digest over the whole run.
///
/// **The manifest interior used to be sampled at a stride of 61, and the reason
/// is gone rather than overridden.** Sampling was a cost decision: the sweep
/// took ~13 s, and flipping each interior byte exercises the same digest check
/// once per byte rather than a different check each time. Neither half of that
/// still holds. The envelope shrank from 26,126 bytes to 15,030 when ABI
/// expression identity moved to a linear encoding, and a decode fell from
/// 662 µs to 18.7 µs across the codec work of 2026-07-27.
///
/// **Measurement, 2026-07-27:** the fully exhaustive sweep runs in **132 ms**,
/// against ~70 ms for the sampled form it replaces. `reduce-the-codec-corruption-sweep-to-its-distinct-classes`
/// asked which of exhaustive or representative coverage the suite's standard
/// should be, and anticipated that a cheaper decode might dissolve the question.
/// It did: for 62 ms the property under test is "no single-byte corruption of
/// this envelope is accepted" rather than "no sampled single-byte corruption
/// is", and the stronger one needs no argument about which bytes are
/// representative.
///
/// A no-op corruption (`^= 0x00`) fails this test at byte 0, so the sweep can
/// say no rather than passing because every decode errors for some other
/// reason.
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
        .chain(HEADER_BYTES..manifest_end)
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
    let beyond_manifest =
        u64::try_from(bytes.len()).expect("the supported envelope length fits u64");
    bytes[length_at..length_at + 8].copy_from_slice(&beyond_manifest.to_be_bytes());
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
        Err(ArtifactCodecError::UnsupportedManifestSchema {
            major: MANIFEST_SCHEMA.0,
            minor: 5,
        }),
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
    // from a producer newer than this reader. The key is a fabricated future one
    // rather than a real reserved key: every key this build knows is now
    // supported, and a test that named one would start passing for the wrong
    // reason the moment that stopped being true — which is exactly what happened
    // to its previous spelling when `multi-stage-program` became readable.
    const FUTURE_FEATURE: &str = "tiler.artifact.feature.from-a-later-producer";
    let mut envelope = envelope_of(&default_artifact());
    envelope.features = vec![FUTURE_FEATURE.to_owned()];
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnsupportedRequiredFeature {
            feature: FUTURE_FEATURE.to_owned(),
        }),
    );
}

/// An execution order that does not sequence every entry exactly once is refused.
///
/// Both directions of "not a permutation", because they fail for different
/// reasons and an implementation can get one right and the other wrong: an order
/// that omits an entry leaves a stage undispatched, and one that repeats an
/// entry dispatches it twice. Either would run a program the artifact does not
/// describe.
#[test]
fn an_execution_order_that_is_not_a_permutation_is_rejected() {
    for order in [vec![], vec![0_u32, 0]] {
        let mut envelope = envelope_of(&default_artifact());
        envelope.variants[0].execution_order = order;
        let bytes = encode(&envelope).expect("a forged envelope still encodes");
        assert!(
            matches!(
                decode(&bytes),
                Err(ArtifactCodecError::StageOrderNotAPermutation { .. })
            ),
            "an order that is not a permutation must be refused as such",
        );
    }
}

/// A dependency edge that orders an entry against itself is refused.
///
/// An entry cannot precede itself, and an edge saying so would make the order
/// unsatisfiable rather than merely wrong — every order violates it. Refused as
/// a malformed edge rather than reported as an ordering violation, so the
/// diagnostic names the defect and not its consequence.
#[test]
fn a_stage_dependency_on_itself_is_rejected() {
    let mut envelope = envelope_of(&default_artifact());
    envelope.variants[0].dependencies = vec![StageDependencyData {
        predecessor: 0,
        successor: 0,
        reason: StageDependencyReason::Data,
    }];
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert!(
        matches!(
            decode(&bytes),
            Err(ArtifactCodecError::StageDependencyOnItself { .. })
        ),
        "a self-edge must be refused as a malformed edge",
    );
}

// `StageDependencyOutOfOrder` is deliberately not reached from this module, and
// the reason is structural rather than an omission: it needs two entries whose
// stated order contradicts an edge between them, and every fixture here packages
// a single-stage program, where any edge with distinct endpoints cannot be
// built at all. `prototypes/serial-sum-compile`'s
// `a_multi_stage_variant_round_trips_with_a_recoverable_sequence` exercises the
// satisfied direction against a real two-stage plan; the contradicting direction
// wants a two-stage fixture here and is owed one.

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

/// A section identifier that is not its own position is rejected by name.
///
/// The identifier is a *wire-only* field: a `Section` in the model carries its
/// purpose and its bytes, and both the framing identifier and the descriptor's
/// copy of it are derived from position when encoding. That makes this one of
/// the few spellings the canonicity backstop could in principle catch, and the
/// experiment recorded on
/// `decide-whether-the-canonicity-re-encode-is-redundant` confirms it does — so
/// the named check is what keeps the rejection legible rather than reporting
/// only that some byte differed.
#[test]
fn a_non_canonical_section_id_is_rejected() {
    // The descriptor's identifier opens a fixed-width record that ends in the
    // section's content digest, so it is located from that digest rather than by
    // scanning for a zero word — several unrelated fields also encode zero.
    // Identifier, purpose, disposition, and schema, then the framed length.
    const DESCRIPTOR_PREFIX_BYTES: usize = 4 + 1 + 1 + 2 + 2 + 8;

    let envelope = envelope_of(&default_artifact());
    let mut forged = encode(&envelope).expect("the envelope encodes");
    // The descriptor's identifier and the framed section's identifier are the
    // same value written twice, so both must move for the forgery to be a
    // consistent non-canonical spelling rather than a length desync.
    let digest = section_digest(DigestAlgorithm::GOVERNED, &envelope.sections()[0]);
    let descriptor_id_at = manifest_offset(&forged, digest.as_bytes()) - DESCRIPTOR_PREFIX_BYTES;
    let manifest_len = usize::try_from(u64::from_be_bytes(
        forged[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    let framed_id_at = HEADER_BYTES + manifest_len;
    for at in [descriptor_id_at, framed_id_at] {
        assert_eq!(
            forged[at..at + 4],
            [0, 0, 0, 0],
            "identifier zero is written"
        );
        forged[at..at + 4].copy_from_slice(&7_u32.to_be_bytes());
    }
    reseal(&mut forged);
    assert_eq!(
        decode(&forged),
        Err(ArtifactCodecError::NonCanonicalSectionId {
            position: 0,
            declared: 7,
        }),
    );
}

/// Two deferred predicates spelled out of canonical key order are rejected.
#[test]
fn a_non_canonical_deferred_predicate_order_is_rejected() {
    let error = reject_guarded_forgery(|envelope| {
        // A second predicate over the arena's boolean literal, distinct from the
        // first by its predicate key alone, so only the order can reject.
        let mut second = envelope.variants[0].deferred[0].clone();
        second.predicate = boolean_literal(envelope);
        envelope.variants[0].deferred.push(second);
        // Put them out of canonical order, whichever way round they came out:
        // the check is that the *stored* order is the canonical one, so the
        // forgery has to be the other one.
        let canonical = super::super::model::canonical_deferred_order(
            &envelope.expressions,
            &envelope.variants[0].deferred,
        );
        if canonical == [0, 1] {
            envelope.variants[0].deferred.swap(0, 1);
        }
    });
    assert_eq!(
        error,
        ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::DeferredPredicate,
        },
    );
}

/// Two launch preconditions spelled out of canonical key order are rejected.
#[test]
fn a_non_canonical_launch_precondition_order_is_rejected() {
    let error = reject_guarded_forgery(|envelope| {
        let extra = envelope.variants[0].deferred[0].predicate;
        let held = envelope.variants[0].entries[0].launch.preconditions[0];
        let keys = super::model::expression_keys(&envelope.expressions);
        let descending = if keys[position(held)] < keys[position(extra)] {
            vec![extra, held]
        } else {
            vec![held, extra]
        };
        envelope.variants[0].entries[0].launch.preconditions = descending;
    });
    assert_eq!(
        error,
        ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::LaunchPrecondition,
        },
    );
}

/// Two entries spelled out of canonical stage order are rejected.
///
/// The fixture packages a single-stage program, so the second entry is forged
/// from the first with a distinct stage subject and backend entry key. That is
/// enough for the order obligation, which reads the stage subject alone; the
/// correspondence between a stage and the program that established it is not
/// decidable from a decoded envelope at all, as `super::validate` records.
#[test]
fn a_non_canonical_entry_order_is_rejected() {
    let error = reject_forged(|envelope| {
        let mut second = envelope.variants[0].entries[0].clone();
        second.stage = super::model::StageSubject::from_bytes(b"\xffzzz-later-stage")
            .expect("a bounded stage subject");
        second.entry_key =
            super::super::keys::BackendEntryKey::from_bytes(b"spare").expect("a bounded entry key");
        // Descending by stage subject, which is exactly the non-canonical form.
        envelope.variants[0].entries.insert(0, second);
        envelope.variants[0].execution_order = vec![0, 1];
        envelope.features = envelope.derived_features();
    });
    assert_eq!(
        error,
        ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::Entry,
        },
    );
}

/// Returns the arena position of the fixture's boolean literal root.
fn boolean_literal(envelope: &ArtifactEnvelope) -> u32 {
    let found = envelope
        .expressions
        .iter()
        .position(|node| matches!(node, ExprNode::Root(AbiRoot::BooleanLiteral(_))))
        .expect("the fixture carries a boolean literal");
    u32::try_from(found).expect("a bounded arena fits u32")
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
                swap(&mut binding.accessible_offset);
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

/// A decoded binding's offset is re-proven at its own use site, not trusted.
///
/// The forgery points the slot's accessible offset at the boolean guard the
/// fixture already reaches from a second use site, so the arena stays closed and
/// what rejects is the use-site type rule rather than an orphaned node.
#[test]
fn a_binding_offset_that_is_not_a_byte_count_is_rejected() {
    assert_eq!(
        reject_guarded_forgery(|envelope| {
            let guard = envelope.variants[0].guard;
            envelope.variants[0].entries[0].bindings[0].accessible_offset = guard;
        }),
        ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::ExpressionType {
                use_site: AbiExprUse::AccessibleOffset,
                expected: AbiType::Unsigned,
                actual: AbiType::Boolean,
            }),
        },
    );
}

/// A nonzero binding offset survives the trip from producer bytes to consumer.
///
/// This is the positive end-to-end case the offset row exists for, and it is
/// only writable because `carry-the-stage-execution-order-in-the-envelope`
/// made a two-stage envelope readable: the smallest plan carrying a nonzero
/// offset has two stages, since a partial window needs a temporary and the two
/// region refinements naming one live in different regions. The fixture binds
/// one program-owned scratch value at byte 24 of 48 in both stages, and what a
/// consumer holding only bytes must see is exactly that placement — a decoded
/// record carrying the extent and not the start would leave a loader binding
/// the right buffer at byte zero, a silently wrong dispatch rather than a
/// rejection.
#[test]
fn a_partial_binding_window_survives_encode_and_decode() {
    let artifact = partial_window_artifact();
    let envelope = envelope_of(&artifact);
    assert!(
        envelope
            .features()
            .iter()
            .any(|feature| feature == FEATURE_MULTI_STAGE_PROGRAM)
    );
    let bytes = encode(&envelope).expect("a two-stage artifact encodes");
    let decoded = super::view::decode_artifact(&bytes).expect("a two-stage artifact decodes");

    let inputs: Vec<_> = decoded.inputs().collect();
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_shape(inputs[0].key(), inputs[0].shape())
        .expect("the decoded interface binds its own declared shape");
    let facts = binder.build();

    let variant = decoded.variants().next().expect("one packaged variant");
    let entries: Vec<_> = variant.entries().collect();
    assert_eq!(entries.len(), 2);
    // The scratch slot of each stage: written by the first, read by the
    // second, and placed at the same nonzero byte in both.
    for (entry, slot) in [(&entries[0], 1), (&entries[1], 0)] {
        let bindings: Vec<_> = entry.bindings().collect();
        assert_eq!(
            bindings[slot].accessible_offset().evaluate(&facts),
            Ok(AbiValue::Unsigned(SCRATCH_OFFSET)),
        );
        assert_eq!(
            bindings[slot].accessible_bytes().evaluate(&facts),
            Ok(AbiValue::Unsigned(ELEMENT_BYTES * 6)),
        );
    }
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

/// A stage-dependency overflow names the dependency resource, and the entry
/// resource beside it stays distinct.
///
/// The two are separately bounded — `MAX_VARIANT_ENTRIES` against
/// `MAX_STAGE_DEPENDENCIES` — so classifying an edge overflow as an entry
/// overflow reported a limit the bytes had not exceeded and sent a reader to
/// the wrong number. The second assertion is the negative neighbour: it drives
/// the entry budget over the identical bytes, so a change that collapsed the
/// two kinds back together would fail here rather than pass silently.
#[test]
fn a_stage_dependency_overflow_names_the_dependency_resource() {
    // The count prefix alone; no edge body is reached, because the budget is
    // checked before anything is allocated for it.
    let absurd = u64::MAX.to_be_bytes();

    let error = parse_dependencies(&mut Cursor::new(&absurd), 0, 1, &[0])
        .expect_err("an absurd edge count is refused");
    assert!(
        matches!(
            error,
            ArtifactCodecError::Limit {
                resource: CodecLimitKind::StageDependencies,
                ..
            },
        ),
        "expected a StageDependencies limit, got {error:?}",
    );

    let neighbour = Cursor::new(&absurd)
        .vec(
            MAX_VARIANT_ENTRIES,
            CodecLimitKind::Entries,
            |_| -> Result<(), ArtifactCodecError> {
                unreachable!("the budget rejects before any entry is parsed")
            },
        )
        .expect_err("an absurd entry count is refused");
    assert!(
        matches!(
            neighbour,
            ArtifactCodecError::Limit {
                resource: CodecLimitKind::Entries,
                ..
            },
        ),
        "expected an Entries limit, got {neighbour:?}",
    );
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

/// Assembles the one-variant fixture around a payload the closure declares.
///
/// Everything but the payload declaration is shared rather than duplicated,
/// which is what makes [`a_pending_payload_identifies_the_artifact_its_compilation_will_produce`]
/// mean something: the two artifacts it compares can differ only in the one
/// call under test.
fn artifact_with(
    declare: impl FnOnce(&mut ArtifactProgramBuilder) -> Result<PayloadId, ArtifactBuildError>,
) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = declare(&mut draft).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    draft.build().unwrap()
}

/// Assembles a one-variant artifact whose single payload carries its object.
fn carried_artifact(source: &[u8], code: &[u8]) -> VerifiedArtifactProgram {
    artifact_with(|draft| {
        draft.push_carried_payload(
            BackendKey::new("tiler.metal").unwrap(),
            RepresentationKey::new("metallib").unwrap(),
            SchemaVersion::new(1, 0),
            profile(),
            ArtifactExecutionPolicy::RequiresDeviceTranslation,
            payload_content(source, code),
        )
    })
}

/// Assembles the same artifact from a payload nothing has compiled yet.
fn pending_artifact(source: &[u8]) -> VerifiedArtifactProgram {
    artifact_with(|draft| {
        draft.push_pending_payload(
            BackendKey::new("tiler.metal").unwrap(),
            RepresentationKey::new("metallib").unwrap(),
            SchemaVersion::new(1, 0),
            profile(),
            ArtifactExecutionPolicy::RequiresDeviceTranslation,
            &payload_metadata(source),
        )
    })
}

/// A payload can be named before it is built, and names the same artifact.
///
/// This is the property an expansion cache key rests on, asserted as a byte
/// equality rather than argued in prose. A cache needs its key on a *miss*, so
/// the subject it composes there is derived from a portfolio whose payloads
/// have not been compiled; the entry it later publishes carries the compiled
/// artifact. If those two identities could differ, the cache would file a
/// result under a key no lookup produces — and once
/// `bind-the-cache-subject-to-the-carried-payload-provenance` ties a bundle to
/// its subject, the same gap becomes an entry served for a subject that is not
/// its own.
///
/// The equality holds for *any* emitted object, which is the second assertion:
/// identity follows the compilation subject, so a linker that is not
/// byte-reproducible cannot move it.
#[test]
fn a_pending_payload_identifies_the_artifact_its_compilation_will_produce() {
    let source = b"kernel void fused() {}";
    let pending = pending_artifact(source);
    for code in [b"first-link".as_slice(), b"second-link".as_slice()] {
        assert_eq!(
            pending.canonical_identity(),
            carried_artifact(source, code).canonical_identity(),
            "the identity derived before compiling must be the identity the \
             compiled artifact carries, whatever the linker emitted",
        );
    }
    assert_ne!(
        pending.canonical_identity(),
        pending_artifact(b"kernel void other() {}").canonical_identity(),
        "a different compilation subject is still a different artifact",
    );
}

/// The digest is a function of the subject, so the object is not needed to ask.
#[test]
fn a_payload_subject_yields_its_identity_without_its_object() {
    let metadata = payload_metadata(b"kernel void fused() {}");
    let identity = metadata
        .identity()
        .expect("a bounded subject has an identity");
    assert_eq!(
        identity,
        payload_content(b"kernel void fused() {}", b"link")
            .identity()
            .expect("a bounded subject has an identity"),
    );
    assert_ne!(
        identity,
        payload_metadata(b"kernel void other() {}")
            .identity()
            .expect("a bounded subject has an identity"),
    );
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

/// Re-seals a forged carried payload so only the check under test can reject it.
///
/// The descriptor's digest is required to equal the identity of the exact
/// metadata bytes, so editing a carried subject without restamping the digest is
/// caught by `PayloadIdentityMismatch` before anything else looks at it. Every
/// entry-mapping case below therefore rewrites both, leaving a perfectly
/// self-consistent envelope that only the obligation under test refuses.
fn reject_forged_subject(forge: impl FnOnce(&mut PayloadMetadata)) -> ArtifactCodecError {
    let artifact = carried_artifact(b"kernel void fused() {}", b"link");
    let mut envelope = envelope_of(&artifact);
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    let mut metadata = payload_metadata(b"kernel void fused() {}");
    forge(&mut metadata);
    let bytes = super::payload::encode_metadata(&metadata);
    envelope.payloads[0].digest =
        super::payload::payload_identity(&bytes).expect("a bounded subject has an identity");
    envelope.sections[position(sections.metadata)].bytes = bytes;
    let encoded = encode(&envelope).expect("a forged envelope still encodes");
    decode(&encoded).expect_err("the forged envelope must be refused")
}

/// A consumer holding only bytes can name everything one dispatch needs.
///
/// This is the property the whole dispatch record exists for, so it is asserted
/// end to end from the public entry point rather than through the crate-private
/// envelope: `decode_artifact` takes bytes and nothing else, and every value
/// below is read back through the promoted view. Nothing here holds the
/// `VerifiedArtifactProgram`, the semantic program, a registry, or any producer
/// code — which is the point, because needing one would mean the artifact is not
/// the interface.
#[test]
fn a_decoded_artifact_carries_everything_one_dispatch_needs() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"\x00metallib\xff");
    let bytes = artifact.encode().expect("a verified artifact encodes");
    let decoded = super::view::decode_artifact(&bytes).expect("the bytes are a valid artifact");

    // The named interface, which is also where an expression's free variables
    // come from.
    let inputs: Vec<_> = decoded.inputs().collect();
    let outputs: Vec<_> = decoded.outputs().collect();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].key().as_str(), "input");
    assert_eq!(inputs[0].shape().extents().len(), 2);
    assert_eq!(outputs[0].key().as_str(), "result");

    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_shape(inputs[0].key(), inputs[0].shape())
        .expect("the decoded interface binds its own declared shape");
    let facts = binder.build();

    let variant = decoded.variants().next().expect("one packaged variant");
    assert_eq!(variant.routing_rank(), 0);
    assert_eq!(variant.deferred_predicates().len(), 0);
    assert_eq!(
        variant.applicability_guard().evaluate(&facts),
        Ok(AbiValue::Boolean(true)),
        "the guard decides routing and must be evaluable from bytes",
    );

    let entry = variant.entries().next().expect("one executable entry");
    assert_eq!(entry.resources().buffer_bindings, 2);
    assert_eq!(entry.numerical().profile_key(), "tiler.test.strict-f32");
    assert_eq!(
        entry.numerical().contraction(),
        NumericalPermission::Forbidden,
    );
    assert_eq!(
        entry.numerical().permutation(),
        NumericalPermission::Forbidden,
    );
    assert_eq!(
        entry.numerical().signed_zero(),
        NumericalPermission::Forbidden,
    );
    assert_eq!(
        entry.numerical().nan_assumptions(),
        ExceptionalValueAssumption::MakeNoAssumption,
    );
    assert_eq!(
        entry.numerical().infinity_assumptions(),
        ExceptionalValueAssumption::MakeNoAssumption,
    );
    assert_eq!(
        entry.resources().permutation,
        NumericalPermission::Forbidden,
    );
    assert_eq!(
        entry.resources().signed_zero,
        NumericalPermission::Forbidden,
    );
    assert_eq!(
        entry.resources().nan_assumptions,
        ExceptionalValueAssumption::MakeNoAssumption,
    );
    assert_eq!(
        entry.resources().infinity_assumptions,
        ExceptionalValueAssumption::MakeNoAssumption,
    );
    assert!(entry.zero_work_skips_dispatch());
    assert_eq!(entry.launch_preconditions().len(), 0);
    assert_eq!(
        entry.launch_threads().evaluate(&facts),
        Ok(AbiValue::Unsigned(2)),
    );
    assert_eq!(
        entry.threads_per_workgroup().evaluate(&facts),
        Ok(AbiValue::Unsigned(1)),
    );
    assert_eq!(entry.launch_threads().value_type(), AbiType::Unsigned);

    // The backend half: which symbol to look up and where each slot goes.
    assert_eq!(entry.backend_symbol(), Some("tiler_fused_0"));
    assert_eq!(entry.transport_slots(), Some([0, 1].as_slice()));
    assert_eq!(entry.backend_entry_key().as_bytes(), b"fused");
    assert_eq!(
        decoded.payload_object(entry.payload()),
        Some(b"\x00metallib\xff".as_slice()),
        "the committed object is the exact bytes the producer packaged",
    );
    assert_eq!(
        decoded.payloads()[entry.payload()].representation.as_str(),
        "metallib",
    );
    assert_eq!(
        decoded
            .payload_metadata(entry.payload())
            .expect("the payload is carried")
            .provenance
            .target,
        "air64-apple-macosx26.0",
    );

    // And the half that decides which buffer each slot addresses.
    let bindings: Vec<_> = entry.bindings().collect();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].slot(), 0);
    assert_eq!(bindings[0].kind(), BindingKind::Buffer);
    assert_eq!(bindings[0].access(), BufferAccess::Read);
    assert_eq!(bindings[0].alignment(), 4);
    assert_eq!(bindings[0].element_type(), KernelType::F32);
    assert_eq!(bindings[0].address_space(), AddressSpace::Device);
    assert_eq!(
        bindings[0].accessible_bytes().evaluate(&facts),
        Ok(AbiValue::Unsigned(24)),
    );
    assert_eq!(
        bindings[1].accessible_bytes().evaluate(&facts),
        Ok(AbiValue::Unsigned(8)),
    );
    let result = OutputKey::new("result").unwrap();
    assert_eq!(
        bindings[0].target(),
        BindingTarget::ProgramInput(inputs[0].key()),
    );
    assert_eq!(
        bindings[1].target(),
        BindingTarget::ProgramOutput(std::slice::from_ref(&result)),
    );

    // The program is named and deliberately not rebuilt.
    assert_eq!(
        variant.kernel_program_identity(),
        fused_program(&semantic_program(), SCALE_BITS)
            .canonical_identity()
            .as_bytes(),
    );
}

/// A descriptor-only payload reports no symbol rather than an invented one.
#[test]
fn an_uncarried_payload_publishes_no_backend_mapping() {
    let artifact = default_artifact();
    let bytes = artifact.encode().expect("a verified artifact encodes");
    let decoded = super::view::decode_artifact(&bytes).expect("the bytes are a valid artifact");
    let entry = decoded
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    assert_eq!(entry.backend_symbol(), None);
    assert_eq!(entry.transport_slots(), None);
    assert_eq!(decoded.payload_object(entry.payload()), None);
    assert!(decoded.payload_metadata(entry.payload()).is_none());
    // A position past the descriptor table is the same answer and not a panic.
    assert!(decoded.payload_metadata(decoded.payloads().len()).is_none());
    assert_eq!(decoded.payload_object(decoded.payloads().len()), None);
}

/// A carried payload that maps none of the entries it realizes is refused.
///
/// Before this check the artifact layer declared such an envelope valid and left
/// a loader to discover it could not resolve a symbol. The mapping is what makes
/// a neutral entry key dispatchable, so an unmapped entry is a record that
/// cannot be dispatched from, which is a decode failure rather than a loader's
/// problem.
#[test]
fn a_carried_payload_that_maps_no_realized_entry_is_rejected() {
    assert_eq!(
        reject_forged_subject(|metadata| {
            metadata.entries[0].entry_key = BackendEntryKey::from_bytes(b"other").unwrap();
        }),
        ArtifactCodecError::UnmappedBackendEntry { payload: 0 },
    );
}

/// A mapping whose transport count is not the entry's binding count is refused.
#[test]
fn an_entry_mapping_that_does_not_place_every_binding_is_rejected() {
    assert_eq!(
        reject_forged_subject(|metadata| metadata.entries[0].transports.truncate(1)),
        ArtifactCodecError::EntryTransportCardinality {
            payload: 0,
            bindings: 2,
            transports: 1,
        },
    );
}

/// A binding target naming an interface entry the artifact does not declare is refused.
///
/// Framing, every digest, and the re-derived identity all still agree here — the
/// forged name is folded into the identity the decoder recomputes — so this is
/// the check that catches it, and it is the reason a name is validated even
/// though the correspondence behind it cannot be.
#[test]
fn a_binding_target_naming_an_undeclared_interface_entry_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[0].target =
                BindingTargetData::ProgramInput(InputKey::new("absent").unwrap());
        }),
        ArtifactCodecError::UnknownBindingTargetKey {
            key: "absent".to_owned(),
            input: true,
        },
    );
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[1].target =
                BindingTargetData::ProgramOutput(vec![OutputKey::new("absent").unwrap()]);
        }),
        ArtifactCodecError::UnknownBindingTargetKey {
            key: "absent".to_owned(),
            input: false,
        },
    );
}

/// A binding addressing output storage under no name at all is refused.
#[test]
fn a_binding_target_that_names_no_output_is_rejected() {
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[1].target =
                BindingTargetData::ProgramOutput(Vec::new());
        }),
        ArtifactCodecError::EmptyBindingTarget,
    );
}

/// A binding target's output names are a set, and a repeat is refused.
#[test]
fn a_repeated_binding_target_name_is_rejected() {
    let result = OutputKey::new("result").unwrap();
    assert_eq!(
        reject_forged(|envelope| {
            envelope.variants[0].entries[0].bindings[1].target =
                BindingTargetData::ProgramOutput(vec![result.clone(), result.clone()]);
        }),
        ArtifactCodecError::DuplicateItem {
            subject: OrderedSubject::BindingTargetKey,
        },
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

// -------------------------------------------------------------------------
// Section descriptor purpose, disposition, and schema
// -------------------------------------------------------------------------

/// A section digest is a *standalone* content address, not only an in-envelope
/// integrity check.
///
/// This is the property the descriptor change exists for. Inside a complete
/// envelope the purpose was already bound one level up, by the manifest
/// descriptor that names it and the manifest digest that covers that
/// descriptor. Lifted out of the envelope — which is exactly what
/// content-addressing a backend code section does — a digest over bytes alone
/// would give two sections of different purposes one address.
#[test]
fn equal_bytes_under_different_purposes_have_different_section_digests() {
    let bytes = b"the same section content".to_vec();
    let metadata = Section {
        kind: SectionKind::BackendPayloadMetadata,
        bytes: bytes.clone(),
    };
    let code = Section {
        kind: SectionKind::BackendPayloadCode,
        bytes,
    };
    assert_ne!(
        section_digest(DigestAlgorithm::GOVERNED, &metadata),
        section_digest(DigestAlgorithm::GOVERNED, &code),
    );
}

/// Locates the fixed-width prefix of the first section descriptor.
///
/// The prefix is `id | purpose | disposition | schema major | schema minor`,
/// which is distinctive enough to find unambiguously; a bare purpose tag is
/// one byte and occurs throughout the manifest.
fn first_section_descriptor(bytes: &[u8]) -> usize {
    let kind = SectionKind::KernelProgramSubject;
    let schema = kind.schema();
    let mut needle = 0_u32.to_be_bytes().to_vec();
    needle.push(kind.tag());
    needle.push(kind.disposition().tag());
    needle.extend_from_slice(&schema.major().to_be_bytes());
    needle.extend_from_slice(&schema.minor().to_be_bytes());
    manifest_offset(bytes, &needle)
}

/// A descriptor may not assert a skip permission its purpose does not have.
#[test]
fn a_section_disposition_that_contradicts_its_purpose_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    let at = first_section_descriptor(&bytes);
    bytes[at + 5] = SectionDisposition::Optional.tag();
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::SectionDispositionMismatch {
            section: 0,
            declared: SectionDisposition::Optional.tag(),
            expected: SectionDisposition::Required.tag(),
        }),
    );
}

/// A descriptor may not assert a content schema its purpose does not carry.
#[test]
fn a_section_content_schema_that_contradicts_its_purpose_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    let at = first_section_descriptor(&bytes);
    bytes[at + 6..at + 8].copy_from_slice(&7_u16.to_be_bytes());
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnsupportedSectionSchema {
            section: 0,
            major: 7,
            minor: 0,
        }),
    );
}

/// An unrecognized disposition tag is refused by name rather than defaulted.
#[test]
fn an_unknown_section_disposition_tag_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    let at = first_section_descriptor(&bytes);
    bytes[at + 5] = 0x7f;
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::SectionDisposition,
            tag: 0x7f,
        }),
    );
}

/// Every governed section purpose declares a disposition and a schema.
///
/// Exhaustive over the vocabulary, so a purpose added without deciding either
/// fails here rather than inheriting whatever the match arm beside it said.
#[test]
fn every_section_purpose_declares_its_disposition_and_schema() {
    for kind in [
        SectionKind::KernelProgramSubject,
        SectionKind::BackendPayloadMetadata,
        SectionKind::BackendPayloadCode,
    ] {
        assert_eq!(SectionKind::from_tag(kind.tag()), Some(kind));
        assert_eq!(kind.disposition(), SectionDisposition::Required);
        assert_eq!(
            SectionDisposition::from_tag(kind.disposition().tag()),
            Some(kind.disposition()),
        );
        assert_eq!(kind.schema(), SchemaVersion::new(1, 0));
    }
}

/// A payload's compatibility contract is its own, not its variant's.
///
/// Two payloads that agree on backend, representation, schema, digest, and
/// execution policy but were built against different target profiles are two
/// distinct payloads with two distinct canonical keys — so a program that
/// shares one compiled object across variants declaring different profiles has
/// an honest encoding, and a loader reads the payload's own contract rather
/// than inferring it from whichever variant it routed to.
#[test]
fn the_payload_compatibility_contract_participates_in_its_canonical_key() {
    let baseline = payload(0xa1);
    let mut elsewhere = baseline.clone();
    elsewhere.compatibility = TargetProfileRef {
        key: TargetProfileKey::new("tiler.test.other").unwrap(),
        descriptor: baseline.compatibility.descriptor.clone(),
    };
    assert_ne!(baseline.canonical_key(), elsewhere.canonical_key());

    let mut redescribed = baseline.clone();
    redescribed.compatibility = TargetProfileRef {
        key: baseline.compatibility.key.clone(),
        descriptor: TargetProfileDescriptorDigest::from_bytes([0x09, 0x09]).unwrap(),
    };
    assert_ne!(
        baseline.canonical_key(),
        redescribed.canonical_key(),
        "the descriptor digest is part of the contract, not decoration",
    );
}

/// The compatibility contract reaches artifact identity and the envelope bytes.
#[test]
fn a_changed_payload_compatibility_contract_changes_the_artifact() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let build = |compatibility: TargetProfileRef| {
        let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
        draft.select_provider(selection(provider.clone())).unwrap();
        let mut descriptor = payload(0xa1);
        descriptor.compatibility = compatibility;
        let id = draft.push_payload(descriptor).unwrap();
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&program, variant(&formulas, id, b"fused"))
            .unwrap();
        draft.build().unwrap()
    };
    let baseline = build(profile());
    let elsewhere = build(TargetProfileRef {
        key: TargetProfileKey::new("tiler.test.other").unwrap(),
        descriptor: profile().descriptor,
    });
    assert_ne!(
        baseline.canonical_identity(),
        elsewhere.canonical_identity(),
    );
    assert_ne!(encoded(&baseline), encoded(&elsewhere));
    assert_eq!(
        decode(&encoded(&elsewhere)).unwrap(),
        envelope_of(&elsewhere)
    );
}

// -------------------------------------------------------------------------
// Hot-path measurements
// -------------------------------------------------------------------------
//
// These print and assert nothing about time. A timing assertion fails on a
// loaded machine and passes on a fast one, which is a flake rather than a
// guard; what they are for is a reproducible number to read before and after a
// change. Reproduce with:
//
//   cargo nextest run --release -p tiler-artifact -E 'test(hot_path)' --no-capture
//
// Release matters: workspace crates build at `opt-level = 0` by default and the
// codec measures ~5x slower in dev.

/// Times one closure and reports the minimum and the mean of many runs.
///
/// **The minimum is the number to compare; the mean is printed beside it to
/// show how loaded the host was.** Every perturbation a machine applies — a
/// scheduler preemption, a frequency drop, a competing build — makes a run
/// *slower* and none makes it faster, so the distribution has a hard floor at
/// the true cost and an unbounded tail of noise. The minimum of enough runs
/// estimates that floor; the mean estimates the floor plus whatever else the
/// machine happened to be doing, which is why a mean-of-five once read a real
/// improvement in this codec as a regression.
fn min_and_mean(repeats: u32, mut run: impl FnMut()) -> (std::time::Duration, std::time::Duration) {
    // Warm the allocator and the branch predictors so the first timed run is
    // not measuring first-touch page faults.
    for _ in 0..16 {
        run();
    }
    let mut best = std::time::Duration::MAX;
    let total = Instant::now();
    for _ in 0..repeats {
        let start = Instant::now();
        run();
        best = best.min(start.elapsed());
    }
    (best, total.elapsed() / repeats)
}

/// Reports decode cost, and how much of it is the canonicity re-encode.
///
/// **The share is the finding.** `decode` re-encodes the whole envelope to
/// prove one artifact has one byte spelling. Its cost is therefore worth
/// knowing exactly rather than estimating, and
/// `decide-whether-the-canonicity-re-encode-is-redundant` settled on evidence
/// which non-canonical spellings only that re-encode rejects.
#[test]
fn hot_path_decode_and_reencode_share() {
    const REPEATS: u32 = 400;
    let bytes = encoded(&default_artifact());

    let (full, full_mean) = min_and_mean(REPEATS, || {
        let _ = decode(&bytes).expect("the fixture decodes");
    });

    let envelope = decode(&bytes).expect("the fixture decodes");
    let (reencode, reencode_mean) = min_and_mean(REPEATS, || {
        let _ = super::encode::encode(&envelope).expect("the envelope re-encodes");
    });

    println!("MEASURE envelope bytes   : {}", bytes.len());
    println!("MEASURE decode           : min {full:?}, mean {full_mean:?} over {REPEATS}");
    println!("MEASURE re-encode alone  : min {reencode:?}, mean {reencode_mean:?} over {REPEATS}");
    println!(
        "MEASURE re-encode share  : {:.1}%",
        (reencode.as_secs_f64() / full.as_secs_f64()) * 100.0
    );
}

/// Reports what one decode spends on each of its four stages.
///
/// **The split is the finding, and it is not the one reading the source
/// suggests.** `decode` parses the framing and manifest, re-proves the model's
/// obligations, derives the canonical identity to compare against the manifest's,
/// and re-encodes as its canonicity backstop. Deriving the identity is the
/// single largest stage, and it is paid *twice* over the decode as a whole —
/// once directly and once inside the re-encode's manifest, which embeds the
/// identity bytes but is dominated by the expression-key derivation it shares
/// with them. The stages are measured against the same decoded envelope so the
/// numbers add up against the full decode above them.
#[test]
fn hot_path_decode_stage_budget() {
    const REPEATS: u32 = 400;
    let bytes = encoded(&default_artifact());
    let envelope = decode(&bytes).expect("the fixture decodes");
    let identity = envelope.canonical_identity().expect("the envelope derives");
    let digests: Vec<_> = envelope
        .sections()
        .iter()
        .map(|section| section_digest(DigestAlgorithm::GOVERNED, section))
        .collect();

    let (full, _) = min_and_mean(REPEATS, || {
        let _ = decode(&bytes).expect("the fixture decodes");
    });
    let (validate, _) = min_and_mean(REPEATS, || {
        super::validate::validate(&envelope).expect("the envelope validates");
    });
    let (derive, _) = min_and_mean(REPEATS, || {
        let _ = envelope.canonical_identity().expect("the envelope derives");
    });
    let (reencode, _) = min_and_mean(REPEATS, || {
        let _ = super::encode::encode_with_identity(&envelope, &identity, &digests)
            .expect("the envelope re-encodes");
    });
    let (section_hash, _) = min_and_mean(REPEATS, || {
        for section in envelope.sections() {
            let _ = section_digest(DigestAlgorithm::GOVERNED, section);
        }
    });
    let (keys, _) = min_and_mean(REPEATS, || {
        let _ = super::model::expression_keys(envelope.expressions());
    });

    let share = |part: std::time::Duration| (part.as_secs_f64() / full.as_secs_f64()) * 100.0;
    println!("MEASURE envelope bytes    : {}", bytes.len());
    println!("MEASURE decode total      : {full:?}");
    println!(
        "MEASURE   validate        : {validate:?} ({:.1}%)",
        share(validate)
    );
    println!(
        "MEASURE   derive identity : {derive:?} ({:.1}%)",
        share(derive)
    );
    println!(
        "MEASURE   re-encode       : {reencode:?} ({:.1}%)",
        share(reencode)
    );
    println!(
        "MEASURE   section digests : {section_hash:?} ({:.1}%, no longer paid twice)",
        share(section_hash)
    );
    println!(
        "MEASURE     of which keys : {keys:?} ({:.1}% of decode)",
        share(keys)
    );
}

/// Reports what a carried payload's repeated metadata decode costs.
///
/// `super::validate` decodes each carried payload's compilation subject once to
/// bind it to the descriptor's digest, and again per realized entry to resolve
/// that entry's backend mapping — `2 + E` decodes of the same bytes, each
/// re-allocating a `PayloadMetadata` including a copy of the carried source.
/// The repetition is real; this is what one of them is worth.
#[test]
fn hot_path_carried_metadata_decode() {
    const REPEATS: u32 = 400;
    let artifact = carried_artifact(b"kernel void fused() {}", b"\x00metallib\xff");
    let bytes = encoded(&artifact);
    let envelope = decode(&bytes).expect("a carried envelope decodes");
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    let metadata = envelope.sections()[position(sections.metadata)]
        .bytes
        .clone();

    let (full, _) = min_and_mean(REPEATS, || {
        let _ = decode(&bytes).expect("a carried envelope decodes");
    });
    let (once, _) = min_and_mean(REPEATS, || {
        decode_metadata(&metadata).expect("the carried subject decodes");
    });
    println!("MEASURE carried envelope  : {} bytes", bytes.len());
    println!("MEASURE decode total      : {full:?}");
    println!(
        "MEASURE decode_metadata x1: {once:?} ({:.2}% of decode)",
        (once.as_secs_f64() / full.as_secs_f64()) * 100.0
    );
}

/// Reports the governed digest's throughput against the bytes a decode hashes.
///
/// **This is what the decode budget above actually is.** A decode hashes the
/// manifest once and every section once; its canonicity re-encode hashes both
/// again. So a decode of an `N`-byte envelope drives roughly `2N` bytes through
/// SHA-256, and at the throughput printed here that product is most of the
/// decode. Neither the parse, the model re-proof, nor the identity derivation is
/// close to it.
#[test]
fn hot_path_digest_throughput() {
    const REPEATS: u32 = 400;
    let bytes = encoded(&default_artifact());
    let algorithm = DigestAlgorithm::GOVERNED;
    let (hash, _) = min_and_mean(REPEATS, || {
        let _ = algorithm.digest(MANIFEST_DIGEST_DOMAIN, &bytes);
    });
    println!("MEASURE envelope bytes    : {}", bytes.len());
    println!("MEASURE digest once over  : {hash:?}");
    println!(
        "MEASURE digest throughput : {:.1} MB/s",
        (f64::from(u32::try_from(bytes.len()).expect("the fixture fits u32")) / hash.as_secs_f64())
            / 1e6
    );
}

/// Decodes in a loop long enough for a sampling profiler to attribute the cost.
///
/// **This is the harness that says *where* decode time goes.** A single decode
/// is microseconds, which is one sample; reading a list of suspected costs off
/// the source instead optimizes the list it started with. Recording this loop is
/// what showed that the canonical-identity derivation, not the framing parse,
/// dominates a decode.
///
/// It is `#[ignore]`d because it deliberately runs for seconds and asserts
/// nothing. Record it with:
///
/// ```text
/// CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --tests -p tiler-artifact
/// TILER_PROFILE_SECONDS=20 samply record --save-only --unstable-presymbolicate \
///     --rate 4000 -o decode.profile.json.gz \
///     -- target/release/deps/tiler_artifact-<hash> \
///        --ignored --exact program::codec::tests::hot_path_decode_profile_loop --nocapture
/// ```
///
/// Three details are load-bearing. `CARGO_PROFILE_RELEASE_DEBUG=true` is
/// required: the release profile carries no debug information, and without it
/// every frame symbolicates to a bare hex address. `--unstable-presymbolicate`
/// writes the `*.syms.json` sidecar that holds the names — the profile's own
/// string table does not. And `--release` matters on its own: workspace crates
/// build at `opt-level = 0` by default and the codec measures ~5x slower in dev.
///
/// `TILER_PROFILE_SECONDS` sets the duration and defaults to ten.
#[test]
#[ignore = "runs for seconds under a profiler; not part of the gate"]
fn hot_path_decode_profile_loop() {
    let seconds: u64 = std::env::var("TILER_PROFILE_SECONDS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(10);
    let bytes = encoded(&default_artifact());
    let deadline = Instant::now() + std::time::Duration::from_secs(seconds);
    let mut decodes = 0_u64;
    while Instant::now() < deadline {
        for _ in 0..64 {
            let _ = decode(&bytes).expect("the fixture decodes");
        }
        decodes += 64;
    }
    println!("MEASURE profile loop: {decodes} decodes in {seconds}s");
}

/// Reports how large a canonical identity is relative to the bytes it names.
///
/// Identities are canonical byte strings rather than digests, and the ABI
/// expression encoding embeds each operand's whole key, so a node's key carries
/// its entire subtree. `encode-abi-expression-identity-in-linear-space` is the
/// ticket; this is the number it moves.
#[test]
fn hot_path_identity_size() {
    let artifact = default_artifact();
    let bytes = encoded(&artifact);
    println!("MEASURE envelope bytes   : {}", bytes.len());
    println!(
        "MEASURE artifact identity: {} bytes",
        artifact.canonical_identity().as_bytes().len()
    );
}
