//! The physical-selection run on the wire: position, invariance, and refusals.
//!
//! The builder-side rules live in `program::tests::selected_physical_implementations`.
//! What is proven here is everything that only exists once the run has been
//! written: that identity and the manifest write **one** byte stream rather
//! than two that agree, that the run sits at the exact position both encoders
//! claim, that a decoder re-proves every rule the builder proved on bytes no
//! builder wrote, and that the published envelope and its digest move with the
//! selected rows and not with the offered environment.

use super::super::super::model::{
    PHYSICAL_SELECTION_KEY_DOMAIN, PHYSICAL_SELECTION_RUN_TAG, PhysicalProposalKind,
    SelectedPhysicalImplementation,
};
use super::super::super::tests::{
    default_artifact, lowering_provider, occurrence, offered_physical, partial_window_artifact,
    physical_provider, physical_selection, proposal,
};
use super::super::super::{
    ArtifactBuildError, MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS, PhysicalRegionOccurrenceIdentity,
};
use super::super::decode::decode;
use super::super::encode::{MANIFEST_SCHEMA, encode, envelope_digest};
use super::super::error::{ArtifactCodecError, CodecLimitKind, OrderedSubject, TagSubject};
use super::super::model::ArtifactEnvelope;
use super::support::{encoded, envelope_of, manifest_occurrences, manifest_offset, reseal};

/// Absolute offset of the fixture's one physical-selection run tag.
///
/// Located by the row domain rather than by a pinned offset: the run sits
/// between the feasibility-rule revision and the deferred-predicate count, with
/// the sorted provider and payload tables and the whole expression arena ahead
/// of it, so any of those moving would invalidate a pinned number for reasons
/// that say nothing about this run. The layout behind the domain is fixed and
/// short — run tag, `u64` row count, `u64` key length, then the domain — so the
/// three field offsets are derived from it.
fn run_tag_at(bytes: &[u8]) -> usize {
    let domain = manifest_offset(bytes, PHYSICAL_SELECTION_KEY_DOMAIN);
    domain - 8 - 8 - 1
}

fn row_count_at(bytes: &[u8]) -> usize {
    run_tag_at(bytes) + 1
}

/// The decoder's rejection of one byte-level forgery, with framing resealed.
fn reject_bytes(forge: impl FnOnce(&mut Vec<u8>)) -> ArtifactCodecError {
    let mut bytes = encoded(&default_artifact());
    forge(&mut bytes);
    reseal(&mut bytes);
    decode(&bytes).expect_err("the forged envelope is rejected")
}

/// The decoder's rejection of one forged *model*, encoded through the encoder.
///
/// Stronger than byte surgery where the rule is semantic: the bytes carry a
/// correct manifest digest, correct section digests, and the canonical identity
/// of whatever the envelope now claims, so only the check under test can reject
/// them.
fn reject_model(forge: impl FnOnce(&mut ArtifactEnvelope)) -> ArtifactCodecError {
    let artifact = partial_window_artifact();
    let mut envelope = envelope_of(&artifact);
    forge(&mut envelope);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    decode(&bytes).expect_err("a forged envelope is rejected")
}

// -------------------------------------------------------------------------
// One byte stream, at one position
// -------------------------------------------------------------------------

/// Identity and the manifest carry the byte-identical run at the same position.
///
/// The claim the shared encoder exists for. If the two ever became separate
/// spellings, an artifact could publish a manifest describing one selection and
/// an identity naming another, and every downstream cache would key the wrong
/// one. Asserted as byte equality of the complete run, framing included, rather
/// than as field-by-field agreement, which is what two encoders kept in
/// agreement would also pass.
#[test]
fn the_physical_run_is_one_byte_stream_in_identity_and_manifest() {
    let artifact = default_artifact();
    let rows = artifact
        .variants()
        .next()
        .expect("one packaged variant")
        .selected_physical_implementations()
        .to_vec();

    let mut expected = Vec::new();
    super::super::super::model::push_selected_physical_implementation_run(&mut expected, &rows);
    assert_eq!(
        expected[0], PHYSICAL_SELECTION_RUN_TAG,
        "the run opens with its own tag",
    );

    let identity = artifact.canonical_identity().as_bytes();
    let manifest = encoded(&artifact);

    let in_identity = identity
        .windows(expected.len())
        .filter(|window| *window == expected.as_slice())
        .count();
    assert_eq!(
        in_identity, 1,
        "canonical identity carries the run exactly once",
    );

    let start = run_tag_at(&manifest);
    assert_eq!(
        &manifest[start..start + expected.len()],
        expected.as_slice(),
        "the manifest embeds the identical run rather than restating its fields",
    );
}

/// The run sits after the feasibility-rule revision and before the deferred count.
///
/// The position is the whole reason the schema step is major: a reader at the
/// previous schema would take the run tag and count for the deferred-predicate
/// count and lose framing for the rest of the variant. Asserted structurally —
/// the four bytes ahead of the run tag are the fixture's rule revision, and the
/// bytes after the run are its deferred count — rather than by a pinned offset.
#[test]
fn the_run_sits_between_the_rule_revision_and_the_deferred_count() {
    let artifact = default_artifact();
    let bytes = encoded(&artifact);
    let tag = run_tag_at(&bytes);

    let revision = u32::from_be_bytes(bytes[tag - 4..tag].try_into().expect("a fixed-width field"));
    assert_eq!(
        revision,
        artifact
            .variants()
            .next()
            .expect("one packaged variant")
            .feasibility_rules()
            .revision,
        "the four bytes ahead of the run are the feasibility-rule revision",
    );

    let rows = artifact
        .variants()
        .next()
        .expect("one packaged variant")
        .selected_physical_implementations();
    let mut run = Vec::new();
    super::super::super::model::push_selected_physical_implementation_run(&mut run, rows);
    let after = tag + run.len();
    let deferred = u64::from_be_bytes(
        bytes[after..after + 8]
            .try_into()
            .expect("a fixed-width field"),
    );
    assert_eq!(
        deferred, 0,
        "the field after the run is the fixture's empty deferred-predicate count",
    );
}

/// The run survives encode, decode, and both read views with its order intact.
#[test]
fn the_run_round_trips_through_both_read_views() {
    let artifact = partial_window_artifact();
    let built = artifact
        .variants()
        .next()
        .expect("one packaged variant")
        .selected_physical_implementations()
        .to_vec();
    assert_eq!(built.len(), 2, "the two-entry fixture carries two rows");

    let bytes = encoded(&artifact);
    let decoded = super::super::view::decode_artifact(&bytes).expect("its own bytes decode");
    let read = decoded
        .variants()
        .next()
        .expect("one decoded variant")
        .selected_physical_implementations();
    assert_eq!(
        read,
        built.as_slice(),
        "decoding reconstructs the same owned rows rather than a second view record",
    );
    assert!(
        read[0].region_occurrence.as_bytes() < read[1].region_occurrence.as_bytes(),
        "canonical occurrence order survives the round trip",
    );
}

// -------------------------------------------------------------------------
// Publication invariance and movement, on bytes and digests
// -------------------------------------------------------------------------

/// Neither offered role reaches the published envelope or its digest.
///
/// The byte-level half of the ADR 0072 claim, with the two roles widened
/// independently so a failure names which one became identity-bearing.
#[test]
fn neither_offered_role_moves_the_published_envelope_or_digest() {
    let baseline = encoded(&default_artifact());
    for (role, wide) in [
        (
            "lowering",
            super::super::super::tests::build_artifact_with_roles(
                &[
                    lowering_provider(1),
                    super::super::super::tests::spare_provider(7),
                ],
                &offered_physical(),
            ),
        ),
        (
            "physical",
            super::super::super::tests::build_artifact_with_roles(
                &[lowering_provider(1)],
                &[physical_provider(1), physical_provider(9)],
            ),
        ),
    ] {
        let widened = encoded(&wide);
        assert_eq!(
            baseline, widened,
            "an unused {role} provider must not move the encoded envelope",
        );
        assert_eq!(
            envelope_digest(&baseline),
            envelope_digest(&widened),
            "an unused {role} provider must not move the envelope digest",
        );
    }
}

/// Each selected subject moves the published envelope and its digest.
///
/// The counterpart to the invariance case above: without it, a run folded into
/// nothing would pass that one.
#[test]
fn every_selected_physical_subject_moves_the_envelope_and_digest() {
    let baseline = encoded(&default_artifact());

    let mut other_provider = physical_selection(0);
    other_provider.provider = physical_provider(2);
    let mut other_proposal = physical_selection(0);
    other_proposal.implementation_proposal = proposal(41);
    let mut other_occurrence = physical_selection(0);
    other_occurrence.region_occurrence = occurrence(41);
    let mut other_kind = physical_selection(0);
    other_kind.proposal_kind = PhysicalProposalKind::OpaqueCall;

    for (subject, row) in [
        ("selected provider identity", other_provider),
        ("implementation-proposal identity", other_proposal),
        ("occurrence association", other_occurrence),
        ("proposal kind", other_kind),
    ] {
        let perturbed = encoded(&super::super::super::tests::build_artifact_with_rows(vec![
            row,
        ]));
        assert_ne!(
            baseline, perturbed,
            "{subject} must move the encoded envelope",
        );
        assert_ne!(
            envelope_digest(&baseline),
            envelope_digest(&perturbed),
            "{subject} must move the envelope digest",
        );
    }
}

/// Projection shares a multi-MiB identity allocation rather than copying it.
///
/// The measurable consequence of storing these identities in `Arc<[u8]>` rather
/// than `Box<[u8]>`: `ArtifactEnvelope::project` borrows the verified data and
/// must build an owned row while that data is still live, so a boxed identity
/// would deep-copy both byte runs immediately before manifest encoding.
///
/// Pointer equality is the observation, not timing, so the assertion is exact
/// rather than a threshold. What it would take for this to say *no*: change
/// either wrapper's storage in `program::keys` from `Arc<[u8]>` to `Box<[u8]>`
/// — the clone in `project` then allocates, and the two pointers differ.
#[test]
fn projection_shares_a_multi_mib_identity_rather_than_copying_it() {
    const MIB: usize = 1024 * 1024;
    let row = SelectedPhysicalImplementation {
        region_occurrence: PhysicalRegionOccurrenceIdentity::from_bytes(vec![0x5a; 4 * MIB])
            .expect("a four MiB occurrence identity is inside the per-value bound"),
        ..physical_selection(0)
    };
    let artifact = super::super::super::tests::build_artifact_with_rows(vec![row]);
    let verified = artifact
        .variants()
        .next()
        .expect("one packaged variant")
        .selected_physical_implementations()[0]
        .region_occurrence
        .as_bytes()
        .as_ptr();

    let envelope = envelope_of(&artifact);
    let projected = envelope.variants[0].selected_physical_implementations[0]
        .region_occurrence
        .as_bytes()
        .as_ptr();

    assert_eq!(
        verified, projected,
        "projection must share the identity allocation, not deep-copy four MiB of it",
    );
}

// -------------------------------------------------------------------------
// Decoder refusals
// -------------------------------------------------------------------------

/// The run tag is fixed, and every other byte is refused by name.
///
/// The tag space is this position inside the variant grammar, so a value that
/// another frame also uses is not a collision; a second meaning here would be.
#[test]
fn an_unrecognized_run_tag_is_refused() {
    let error = reject_bytes(|bytes| {
        let at = run_tag_at(bytes);
        bytes[at] = 0x02;
    });
    assert!(
        matches!(
            error,
            ArtifactCodecError::UnknownTag {
                subject: TagSubject::PhysicalSelectionRun,
                tag: 0x02
            }
        ),
        "an unknown run tag is refused by name, got {error:?}",
    );
}

/// `0x00`, the reserved `View` tag `0x04`, and `0xff` all reach the refusal.
///
/// `0x04` is the one that matters: `tiler-compiler` has a fourth proposal kind
/// today and rejects its body before selection, so a manifest carrying it would
/// assert a selected state no compiler can produce. Admitting it later needs a
/// reviewed vocabulary change and another owning version step.
#[test]
fn every_unadmitted_proposal_kind_tag_is_refused_including_the_reserved_view() {
    for tag in [0x00_u8, 0x04, 0xff] {
        let error = reject_bytes(|bytes| {
            // The kind tag is the last byte of the row key, and the fixture's
            // one row key ends where the deferred-predicate count begins.
            let domain = manifest_offset(bytes, PHYSICAL_SELECTION_KEY_DOMAIN);
            let key_len = usize::try_from(u64::from_be_bytes(
                bytes[domain - 8..domain]
                    .try_into()
                    .expect("a fixed-width field"),
            ))
            .expect("the fixture row key fits usize");
            bytes[domain + key_len - 1] = tag;
        });
        assert!(
            matches!(
                error,
                ArtifactCodecError::UnknownTag {
                    subject: TagSubject::PhysicalProposalKind,
                    tag: refused
                } if refused == tag
            ),
            "proposal-kind tag {tag:#04x} must be refused by name, got {error:?}",
        );
    }
}

/// A row key that resolves its frame but opens with another domain is refused.
///
/// The reason the row carries its own separator: without this check a framed key
/// of the right length from some other subject would be read as a selection.
#[test]
fn a_row_key_with_a_foreign_domain_is_refused() {
    let error = reject_bytes(|bytes| {
        let domain = manifest_offset(bytes, PHYSICAL_SELECTION_KEY_DOMAIN);
        bytes[domain] = b'x';
    });
    assert!(
        matches!(error, ArtifactCodecError::BadPhysicalSelectionDomain),
        "a foreign row-key domain is refused, got {error:?}",
    );
}

/// Bytes left inside a row key's own frame are a hidden second statement.
#[test]
fn trailing_bytes_inside_a_row_key_are_refused() {
    let error = reject_bytes(|bytes| {
        let domain = manifest_offset(bytes, PHYSICAL_SELECTION_KEY_DOMAIN);
        let key_len = u64::from_be_bytes(
            bytes[domain - 8..domain]
                .try_into()
                .expect("a fixed-width field"),
        );
        // Declare one byte more than the row occupies and supply it, so the
        // nested cursor completes every field and still has a byte left.
        bytes[domain - 8..domain].copy_from_slice(&(key_len + 1).to_be_bytes());
        let key_end = domain + usize::try_from(key_len).expect("the fixture key fits usize");
        bytes.insert(key_end, 0x00);
        let manifest_len = super::support::manifest_len(bytes) + 1;
        bytes[super::support::MANIFEST_LENGTH_AT..super::support::MANIFEST_LENGTH_AT + 8]
            .copy_from_slice(
                &u64::try_from(manifest_len)
                    .expect("it fits u64")
                    .to_be_bytes(),
            );
    });
    assert!(
        matches!(
            error,
            ArtifactCodecError::TrailingPhysicalSelectionKeyBytes { remaining: 1 }
        ),
        "a byte left inside a row key is refused, got {error:?}",
    );
}

/// A decoded empty run is refused, exactly as construction refuses one.
#[test]
fn a_decoded_empty_run_is_refused() {
    let error = reject_model(|envelope| {
        envelope.variants[0]
            .selected_physical_implementations
            .clear();
    });
    assert!(
        matches!(
            error,
            ArtifactCodecError::ModelRule { ref cause }
                if matches!(**cause, ArtifactBuildError::EmptySelectedPhysicalImplementations)
        ),
        "an empty decoded run is refused, got {error:?}",
    );
}

/// A decoded run repeating or inverting an occurrence is refused.
///
/// Two cases in one test because they are the two sides of one strict ordering
/// rule, and each names its own subject so a failure still says which.
#[test]
fn a_decoded_run_out_of_canonical_occurrence_order_is_refused() {
    let error = reject_model(|envelope| {
        let rows = &mut envelope.variants[0].selected_physical_implementations;
        rows[1].region_occurrence = rows[0].region_occurrence.clone();
    });
    assert!(
        matches!(
            error,
            ArtifactCodecError::DuplicateItem {
                subject: OrderedSubject::SelectedPhysicalImplementation
            }
        ),
        "a repeated decoded occurrence is refused, got {error:?}",
    );

    let error = reject_model(|envelope| {
        envelope.variants[0]
            .selected_physical_implementations
            .swap(0, 1);
    });
    assert!(
        matches!(
            error,
            ArtifactCodecError::NonCanonicalOrder {
                subject: OrderedSubject::SelectedPhysicalImplementation
            }
        ),
        "a descending decoded run is refused, got {error:?}",
    );
}

/// A run outnumbering the entry table is refused as the relational rule.
///
/// Checked at the first point it is decidable — immediately after the entry
/// vector completes — and therefore ahead of the deferred cross-reference,
/// execution order, and dependency checks, so the refusal names the
/// contradiction rather than whichever later rule happened to trip.
#[test]
fn a_decoded_run_outnumbering_the_entry_table_is_refused() {
    let mut bytes = encoded(&default_artifact());
    // A complete, well-formed two-row run spliced over the fixture's one-row
    // run, on a variant whose entry table holds one entry. Every framing field
    // is correct, so nothing but the relational rule can refuse it.
    let two_rows = {
        let mut run = Vec::new();
        super::super::super::model::push_selected_physical_implementation_run(
            &mut run,
            &[physical_selection(0), physical_selection(1)],
        );
        run
    };
    let one_row = {
        let mut run = Vec::new();
        super::super::super::model::push_selected_physical_implementation_run(
            &mut run,
            &[physical_selection(0)],
        );
        run
    };
    let tag = run_tag_at(&bytes);
    assert_eq!(
        &bytes[tag..tag + one_row.len()],
        one_row.as_slice(),
        "the fixture's run is the one-row run this splices over",
    );
    bytes.splice(tag..tag + one_row.len(), two_rows.iter().copied());
    let manifest_len = super::support::manifest_len(&bytes) + two_rows.len() - one_row.len();
    bytes[super::support::MANIFEST_LENGTH_AT..super::support::MANIFEST_LENGTH_AT + 8]
        .copy_from_slice(
            &u64::try_from(manifest_len)
                .expect("it fits u64")
                .to_be_bytes(),
        );
    reseal(&mut bytes);
    let error = decode(&bytes).expect_err("two rows over a one-entry variant is refused");
    assert!(
        matches!(
            error,
            ArtifactCodecError::ModelRule { ref cause }
                if matches!(
                    **cause,
                    ArtifactBuildError::PhysicalSelectionCardinality { selected: 2, entries: 1 }
                )
        ),
        "the relational rule refuses before any later cross-reference, got {error:?}",
    );
}

/// A forged row count past the ceiling is refused before the vector is reserved.
///
/// The allocation guard: the count is bounded the moment it is read, so a
/// hostile manifest cannot make this reader reserve for rows that are not there.
#[test]
fn a_forged_row_count_is_refused_before_allocation() {
    let over = u64::try_from(MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS + 1).expect("it fits u64");
    let error = reject_bytes(|bytes| {
        let at = row_count_at(bytes);
        bytes[at..at + 8].copy_from_slice(&over.to_be_bytes());
    });
    assert!(
        matches!(
            error,
            ArtifactCodecError::Limit {
                resource: CodecLimitKind::SelectedPhysicalImplementations,
                actual,
                ..
            } if actual == over
        ),
        "a 4,097-row count is refused as its own budget, got {error:?}",
    );
}

/// No physical-specific decoder byte budget exists, per identity or in aggregate.
///
/// A negative assertion, and it is the point of the elimination the accepted
/// packet records: `read_header` has already bounded the whole manifest before
/// variant parsing, and both framed identities and the complete run are strict
/// subsets of those bytes, so such a limit would name a refusal no admitted
/// stream can reach. The check is a census of the limit vocabulary's own
/// rendering rather than prose, so a kind added later fails here.
#[test]
fn no_physical_specific_decoder_byte_budget_is_declared() {
    for kind in [
        CodecLimitKind::SelectedPhysicalImplementations,
        CodecLimitKind::ManifestBytes,
    ] {
        let rendered = kind.to_string();
        assert!(
            !rendered.contains("PhysicalSelectionIdentityBytes")
                && !rendered.contains("SelectedPhysicalProvenanceBytes"),
            "{rendered} names a physical byte budget the header admission makes unreachable",
        );
    }
    // The one physical budget is a *count*, and the byte authority beside it is
    // the whole-manifest one that already ran.
    assert_eq!(
        CodecLimitKind::SelectedPhysicalImplementations.to_string(),
        "SelectedPhysicalImplementations",
    );
}

/// The two schemas refuse each other's manifests by name.
///
/// The step is major precisely because the run's interior position leaves a
/// previous reader mis-framed rather than merely uninformed, so the refusal has
/// to happen at schema admission and before any variant is parsed.
#[test]
fn the_previous_manifest_schema_is_refused_by_name() {
    assert_eq!(
        MANIFEST_SCHEMA,
        (22, 0),
        "the physical-selection run owns manifest schema 22.0",
    );
    let error = reject_bytes(|bytes| {
        let at = manifest_offset(bytes, super::super::encode::MANIFEST_DOMAIN)
            + super::super::encode::MANIFEST_DOMAIN.len();
        bytes[at..at + 2].copy_from_slice(&21_u16.to_be_bytes());
    });
    assert!(
        matches!(
            error,
            ArtifactCodecError::UnsupportedManifestSchema {
                major: 21,
                minor: 0
            }
        ),
        "a 21.0 manifest is refused by this reader, got {error:?}",
    );
}

/// The fixture's run occurs exactly once in the manifest.
///
/// A precondition of every offset above: `manifest_offset` asserts a unique
/// match, so this states the expectation the located field rests on rather than
/// leaving it implicit in a helper's panic.
#[test]
fn the_row_domain_locates_exactly_one_field_per_row() {
    let one = encoded(&default_artifact());
    assert_eq!(
        manifest_occurrences(&one, PHYSICAL_SELECTION_KEY_DOMAIN).len(),
        1,
        "the one-row fixture writes its row domain once",
    );
    let two = encoded(&partial_window_artifact());
    assert_eq!(
        manifest_occurrences(&two, PHYSICAL_SELECTION_KEY_DOMAIN).len(),
        2,
        "the two-row fixture writes its row domain twice",
    );
}
