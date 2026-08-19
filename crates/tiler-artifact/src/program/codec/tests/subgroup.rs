//! The conditional subgroup-realization carrier, present and absent.

use super::super::super::model::{push_resources, push_subgroup_requirement};
use super::super::super::tests::default_artifact;
use super::super::decode::{Cursor, decode};
use super::super::encode::{
    HEADER_BYTES, encode, encode_with_identity, identity_digest, section_digest,
};
use super::super::error::{ArtifactCodecError, CodecLimitKind, TagSubject};
use super::super::model::ArtifactEnvelope;
use super::super::view::{ArtifactCodecFailure, decode_artifact};
use super::support::{
    MANIFEST_LENGTH_AT, envelope_of, insert_manifest_bytes, manifest_len, manifest_offset, reseal,
};
use std::mem::variant_count;
use tiler_digest::DigestAlgorithm;
use tiler_ir::schedule::{
    ArithmeticType, SubgroupRealizationError, SubgroupRealizationSubject, SubgroupTransfer,
    SubgroupWidth,
};

/// Constructs the carrier subject this ticket proves without making schedule
/// derivation or KIR emission reachable ahead of their owning tickets.
fn subgroup_subject(lanes: u32, arithmetic: ArithmeticType) -> SubgroupRealizationSubject {
    SubgroupRealizationSubject::new(
        SubgroupWidth::new(lanes).expect("the fixture width is nonzero"),
        arithmetic,
        SubgroupTransfer::InRangeXorShuffle,
    )
    .expect("the fixture width defines an in-range XOR shuffle")
}

/// Projects a verified artifact, then changes only its codec-carrier fixture.
///
/// This is deliberately not a producer claim. Supported schedule derivation
/// still yields `None`; the later owning ticket is what will make `Some`
/// constructible from a real verified kernel.
fn envelope_with_subgroup(subject: SubgroupRealizationSubject) -> ArtifactEnvelope {
    let mut envelope = envelope_of(&default_artifact());
    envelope.variants[0].entries[0].resources.subgroup = Some(subject);
    envelope
}

fn subgroup_block(subject: SubgroupRealizationSubject) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_subgroup_requirement(&mut bytes, Some(subject));
    bytes
}

/// Ends the manifest at `end`, retaining the framed sections that follow it.
///
/// This makes an incomplete final field a real manifest truncation without
/// turning the test into a total-envelope truncation that the header catches
/// before the entry parser is reached.
fn truncate_manifest_at(bytes: &mut Vec<u8>, end: usize) {
    let old_end = HEADER_BYTES + manifest_len(bytes);
    assert!((HEADER_BYTES..old_end).contains(&end));
    bytes.drain(end..old_end);
    let new_len = end - HEADER_BYTES;
    bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8].copy_from_slice(
        &u64::try_from(new_len)
            .expect("the fixture manifest fits u64")
            .to_be_bytes(),
    );
    reseal(bytes);
}

/// The conditional carrier leaves every previously encodable resource byte
/// untouched when no subgroup realization is required.
///
/// The literal vector is this fixture's resource row, unchanged since `v19`:
/// the pre-carrier `v18` spelling plus the two elementary-dimension tags
/// between the signed-zero and NaN-assumption bytes, which is the exact
/// insertion the `v19` step declared. The `v20` step reaches payload and
/// variant framing, not this row. It is intentionally not assembled with tag helpers:
/// doing so would prove only that today's encoder agrees with itself, while
/// this pin proves the layout is the declared one. Existing standard-path
/// artifact, cache-subject, and fixed-content pins cover the enclosing
/// identities.
#[test]
fn an_absent_subgroup_preserves_the_legacy_resource_bytes_exactly() {
    let envelope = envelope_of(&default_artifact());
    let resources = envelope.variants[0].entries[0].resources;
    assert_eq!(resources.subgroup, None);

    let mut bytes = Vec::new();
    push_resources(&mut bytes, resources).expect("the arithmetic rows encode");
    assert_eq!(
        bytes,
        [
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x01, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01,
        ],
    );

    let mut absent = Vec::new();
    push_subgroup_requirement(&mut absent, None);
    assert!(absent.is_empty(), "absence writes no compatibility byte");
}

/// Every arithmetic member survives the internal and public codecs, and the
/// public view re-encodes the exact received bytes.
///
/// The arithmetic population is sized from its type, the transfer population
/// is independently sized in `tag_injectivity`, and the width perturbation
/// proves the fixed integer participates rather than merely occupying space.
#[test]
fn every_present_subgroup_subject_round_trips_without_authority_loss() {
    const ARITHMETICS: [ArithmeticType; variant_count::<ArithmeticType>()] = [
        ArithmeticType::F16,
        ArithmeticType::Bf16,
        ArithmeticType::F32,
        ArithmeticType::F64,
    ];
    let absent = encode(&envelope_of(&default_artifact())).expect("absence encodes");
    let mut identities = Vec::new();

    for arithmetic in ARITHMETICS {
        let subject = subgroup_subject(32, arithmetic);
        let envelope = envelope_with_subgroup(subject);
        let identity = envelope.canonical_identity().expect("identity derives");
        let bytes = encode(&envelope).expect("a present subject encodes");
        assert_eq!(
            bytes.len(),
            absent.len() + 7,
            "presence adds one marker and the six public subject bytes",
        );
        let block = subgroup_block(subject);
        assert_eq!(
            block,
            [0x01, 0x00, 0x00, 0x00, 0x20, arithmetic.tag(), 0x01]
        );
        assert!(
            bytes[HEADER_BYTES..]
                .windows(block.len())
                .any(|run| run == block),
            "the manifest carries the complete subject block",
        );

        let decoded = decode(&bytes).expect("the internal codec preserves the subject");
        assert_eq!(decoded, envelope);
        assert_eq!(
            decoded.variants[0].entries[0].resources.subgroup,
            Some(subject),
        );

        let public = decode_artifact(&bytes).expect("the public codec preserves the subject");
        let entry = public
            .variants()
            .next()
            .expect("one variant")
            .entries()
            .next()
            .expect("one entry");
        assert_eq!(entry.resources().subgroup, Some(subject));
        assert_eq!(public.re_encode().expect("canonical re-encoding"), bytes);
        assert!(
            !identities.contains(&identity),
            "each arithmetic member has a distinct artifact identity",
        );
        identities.push(identity);
    }
    assert_eq!(identities.len(), ARITHMETICS.len());

    let width_32 = envelope_with_subgroup(subgroup_subject(32, ArithmeticType::F32));
    let width_64 = envelope_with_subgroup(subgroup_subject(64, ArithmeticType::F32));
    assert_ne!(
        width_32.canonical_identity().unwrap(),
        width_64.canonical_identity().unwrap(),
        "an independent width perturbation must change artifact identity",
    );
    assert_eq!(
        identity_digest(
            DigestAlgorithm::GOVERNED,
            &width_32.canonical_identity().unwrap(),
        )
        .as_bytes(),
        // Rebaselined at the fact-source provenance schema 3 -> 4 step (the
        // required compilation-selection carrier): the delivered-realization
        // domain moved to v3, so every artifact identity built over it moves.
        // Pre-recompute failure observed left = 04 d9 a0 5b ... (the new
        // digest), right = the retired 87 4e 52 27 ... pin.
        &[
            0x04, 0xd9, 0xa0, 0x5b, 0x91, 0xaf, 0x75, 0x48, 0xd2, 0xed, 0xca, 0x6f, 0x07, 0x81,
            0x52, 0x2d, 0xc9, 0xc1, 0x6e, 0xb0, 0xb7, 0xac, 0xc3, 0xe5, 0x1c, 0xef, 0x92, 0x71,
            0xf7, 0x7f, 0xa5, 0x7f
        ],
        "the first present-subgroup identity has an exact digest pin",
    );
}

/// The conditional boundary has exactly two admitted states and rejects every
/// malformed neighbour without consuming a legacy absence byte.
#[test]
fn subgroup_boundary_and_subject_corruptions_are_typed_and_exact() {
    let subject = subgroup_subject(32, ArithmeticType::F32);
    let block = subgroup_block(subject);
    assert_eq!(block, [0x01, 0x00, 0x00, 0x00, 0x20, 0x03, 0x01]);

    let legacy_continuation = [0_u8; 8];
    let mut cursor = Cursor::new(&legacy_continuation);
    assert_eq!(cursor.subgroup_requirement(), Ok(None));
    assert_eq!(
        cursor.remaining(),
        legacy_continuation.len(),
        "absence peeks at the following text length without consuming it",
    );
    assert_eq!(cursor.text().unwrap(), "");

    let mut present_continuation = block.clone();
    present_continuation.extend_from_slice(&legacy_continuation);
    let mut cursor = Cursor::new(&present_continuation);
    assert_eq!(cursor.subgroup_requirement(), Ok(Some(subject)));
    assert_eq!(cursor.text().unwrap(), "");
    assert_eq!(cursor.remaining(), 0);

    let mut refused_presence_tags = 0_usize;
    for tag in 0x02..=u8::MAX {
        let error = Cursor::new(&[tag])
            .subgroup_requirement()
            .expect_err("every unclaimed presence tag must refuse");
        assert_eq!(
            error,
            ArtifactCodecError::UnknownTag {
                subject: TagSubject::SubgroupPresence,
                tag,
            },
        );
        refused_presence_tags += 1;
    }
    assert_eq!(refused_presence_tags, 254);
    let presence = Cursor::new(&[0x7f])
        .subgroup_requirement()
        .expect_err("the named presence perturbation refuses");
    assert_eq!(
        presence.to_string(),
        "UnknownTag { subject: SubgroupPresence, tag: 127 }",
    );

    let truncations: Vec<_> = (1..block.len())
        .map(|len| {
            Cursor::new(&block[..len])
                .subgroup_requirement()
                .expect_err("every incomplete present block must refuse")
        })
        .collect();
    assert_eq!(
        truncations,
        [
            ArtifactCodecError::Truncated {
                needed: 4,
                available: 0,
            },
            ArtifactCodecError::Truncated {
                needed: 4,
                available: 1,
            },
            ArtifactCodecError::Truncated {
                needed: 4,
                available: 2,
            },
            ArtifactCodecError::Truncated {
                needed: 4,
                available: 3,
            },
            ArtifactCodecError::Truncated {
                needed: 1,
                available: 0,
            },
            ArtifactCodecError::Truncated {
                needed: 1,
                available: 0,
            },
        ],
    );
    assert_eq!(
        truncations[0].to_string(),
        "Truncated { needed: 4, available: 0 }",
    );

    let mut zero_width = block.clone();
    zero_width[1..5].copy_from_slice(&0_u32.to_be_bytes());
    zero_width.push(0);
    let zero_width = Cursor::new(&zero_width)
        .subgroup_requirement()
        .expect_err("zero width must refuse");
    assert_eq!(
        zero_width,
        ArtifactCodecError::InvalidSubgroupRealization {
            cause: SubgroupRealizationError::ZeroWidth,
        },
    );
    assert_eq!(
        zero_width.to_string(),
        "InvalidSubgroupRealization { cause: ZeroWidth }",
    );

    let mut unsupported_width = block.clone();
    unsupported_width[1..5].copy_from_slice(&3_u32.to_be_bytes());
    unsupported_width.push(0);
    let unsupported_width = Cursor::new(&unsupported_width)
        .subgroup_requirement()
        .expect_err("a non-power-of-two XOR width must refuse");
    assert_eq!(
        unsupported_width,
        ArtifactCodecError::InvalidSubgroupRealization {
            cause: SubgroupRealizationError::UnsupportedWidth,
        },
    );

    let mut arithmetic = block.clone();
    arithmetic[5] = 0x7e;
    arithmetic.push(0);
    let arithmetic = Cursor::new(&arithmetic)
        .subgroup_requirement()
        .expect_err("an unknown arithmetic tag must refuse");
    assert_eq!(
        arithmetic,
        ArtifactCodecError::UnknownTag {
            subject: TagSubject::SubgroupArithmetic,
            tag: 0x7e,
        },
    );
    assert_eq!(
        arithmetic.to_string(),
        "UnknownTag { subject: SubgroupArithmetic, tag: 126 }",
    );

    let mut transfer = block.clone();
    transfer[6] = 0x7d;
    transfer.push(0);
    let transfer = Cursor::new(&transfer)
        .subgroup_requirement()
        .expect_err("an unknown transfer tag must refuse");
    assert_eq!(
        transfer,
        ArtifactCodecError::UnknownTag {
            subject: TagSubject::SubgroupTransfer,
            tag: 0x7d,
        },
    );
    assert_eq!(
        transfer.to_string(),
        "UnknownTag { subject: SubgroupTransfer, tag: 125 }",
    );

    let mut duplicate = block.clone();
    duplicate.push(0x01);
    let duplicate = Cursor::new(&duplicate)
        .subgroup_requirement()
        .expect_err("a second singleton block must refuse");
    assert_eq!(duplicate, ArtifactCodecError::DuplicateSubgroupRequirement);
    assert_eq!(duplicate.to_string(), "DuplicateSubgroupRequirement");

    let mut trailing = block.clone();
    trailing.push(0x7c);
    assert_eq!(
        Cursor::new(&trailing).subgroup_requirement(),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::SubgroupPresence,
            tag: 0x7c,
        }),
    );

    let mut old_reader = block;
    old_reader.push(0);
    let declared = u64::from_be_bytes(old_reader.clone().try_into().unwrap());
    let old_reader = Cursor::new(&old_reader)
        .text()
        .expect_err("a legacy reader must fail closed on a present block");
    assert_eq!(
        old_reader,
        ArtifactCodecError::Limit {
            resource: CodecLimitKind::TextBytes,
            actual: declared,
            limit: 4096,
        },
    );
    assert_eq!(
        old_reader.to_string(),
        format!("Limit {{ resource: TextBytes, actual: {declared}, limit: 4096 }}"),
    );

    assert!(matches!(
        ArtifactCodecFailure::from(zero_width),
        ArtifactCodecFailure::Invalid { .. }
    ));
    assert!(matches!(
        ArtifactCodecFailure::from(duplicate),
        ArtifactCodecFailure::Invalid { .. }
    ));
    assert!(matches!(
        ArtifactCodecFailure::from(transfer),
        ArtifactCodecFailure::Malformed { .. }
    ));
}

/// Every private subgroup parser refusal is reachable through the complete
/// envelope decoder after framing and integrity have been honestly restamped.
#[test]
fn the_full_decoder_reaches_each_subgroup_refusal() {
    let subject = subgroup_subject(32, ArithmeticType::F32);
    let block = subgroup_block(subject);
    let baseline = encode(&envelope_with_subgroup(subject)).expect("the carrier encodes");
    let at = manifest_offset(&baseline, &block);

    let fixed_byte_case = |offset: usize, replacement: u8| {
        let mut forged = baseline.clone();
        forged[at + offset] = replacement;
        reseal(&mut forged);
        decode(&forged).expect_err("the subject forgery must refuse")
    };
    assert_eq!(
        fixed_byte_case(0, 0x7f),
        ArtifactCodecError::UnknownTag {
            subject: TagSubject::SubgroupPresence,
            tag: 0x7f,
        },
    );
    assert_eq!(
        fixed_byte_case(5, 0x7e),
        ArtifactCodecError::UnknownTag {
            subject: TagSubject::SubgroupArithmetic,
            tag: 0x7e,
        },
    );
    assert_eq!(
        fixed_byte_case(6, 0x7d),
        ArtifactCodecError::UnknownTag {
            subject: TagSubject::SubgroupTransfer,
            tag: 0x7d,
        },
    );

    let mut zero_width = baseline.clone();
    zero_width[at + 1..at + 5].copy_from_slice(&0_u32.to_be_bytes());
    reseal(&mut zero_width);
    assert_eq!(
        decode(&zero_width),
        Err(ArtifactCodecError::InvalidSubgroupRealization {
            cause: SubgroupRealizationError::ZeroWidth,
        }),
    );
    let mut unsupported_width = baseline.clone();
    unsupported_width[at + 1..at + 5].copy_from_slice(&3_u32.to_be_bytes());
    reseal(&mut unsupported_width);
    assert_eq!(
        decode(&unsupported_width),
        Err(ArtifactCodecError::InvalidSubgroupRealization {
            cause: SubgroupRealizationError::UnsupportedWidth,
        }),
    );

    let mut duplicate = baseline.clone();
    insert_manifest_bytes(&mut duplicate, at + block.len(), &block);
    assert_eq!(
        decode(&duplicate),
        Err(ArtifactCodecError::DuplicateSubgroupRequirement),
    );
    let mut trailing = baseline.clone();
    insert_manifest_bytes(&mut trailing, at + block.len(), &[0x7c]);
    assert_eq!(
        decode(&trailing),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::SubgroupPresence,
            tag: 0x7c,
        }),
    );

    let mut truncated = baseline;
    truncate_manifest_at(&mut truncated, at + 1);
    let error = decode(&truncated).expect_err("the incomplete block must reach its parser");
    assert_eq!(
        error,
        ArtifactCodecError::Truncated {
            needed: 4,
            available: 0,
        },
    );
    assert_eq!(error.to_string(), "Truncated { needed: 4, available: 0 }",);
}

/// Erasing a carried subject without restamping its declaration is an identity
/// failure; restamping produces a different artifact rather than legitimizing
/// the erasure under the old identity.
#[test]
fn late_subgroup_erasure_cannot_retain_the_original_artifact_identity() {
    let subject = subgroup_subject(32, ArithmeticType::F32);
    let present = envelope_with_subgroup(subject);
    let expected = present.canonical_identity().expect("identity derives");

    let mut erased = present.clone();
    erased.variants[0].entries[0].resources.subgroup = None;
    let section_digests: Vec<_> = erased
        .sections()
        .iter()
        .map(|section| section_digest(DigestAlgorithm::GOVERNED, section))
        .collect();
    let stale = encode_with_identity(&erased, &expected, &section_digests)
        .expect("the deliberately stale declaration still frames");
    let error = decode(&stale).expect_err("late erasure must not retain the old identity");
    assert_eq!(error, ArtifactCodecError::ArtifactIdentityMismatch);
    assert_eq!(error.to_string(), "ArtifactIdentityMismatch");

    let restamped_identity = erased.canonical_identity().expect("identity re-derives");
    assert_ne!(restamped_identity, expected);
    let restamped = encode(&erased).expect("the alternate artifact encodes");
    let decoded = decode(&restamped).expect("the alternate artifact is self-consistent");
    assert_eq!(decoded.variants[0].entries[0].resources.subgroup, None);
    assert_eq!(decoded.canonical_identity().unwrap(), restamped_identity);
}
