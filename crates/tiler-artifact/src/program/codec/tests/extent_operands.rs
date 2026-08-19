//! Live input-extent rows and the transports that carry them.

use super::super::super::expr::AbiType;
use super::super::super::model::ExtentOperandData;
use super::super::super::tests::default_artifact;
use super::super::error::{ArtifactCodecError, OrderedSubject};
use super::super::model::position;
use super::super::payload::decode_metadata;
use super::support::{carried_artifact, reject_artifact_forgery};
use tiler_ir::semantic::InputKey;
use tiler_ir::shape::Axis;

fn live_extent_row() -> ExtentOperandData {
    ExtentOperandData {
        key: InputKey::new("input").unwrap(),
        axis: Axis::new(1),
        value_type: AbiType::Unsigned,
    }
}

#[test]
fn omitting_the_transport_for_a_declared_live_extent_row_is_refused() {
    let error = reject_artifact_forgery(&carried_artifact(b"k", b"c"), |envelope| {
        envelope.variants[0].entries[0].input_extents = vec![live_extent_row()];
    });
    assert_eq!(
        error,
        ArtifactCodecError::EntryTransportCardinality {
            payload: 0,
            bindings: 2,
            extents: 1,
            transports: 2,
        },
        "{error:?}"
    );
}

#[test]
fn a_reordered_live_extent_list_is_refused() {
    let error = reject_artifact_forgery(&default_artifact(), |envelope| {
        envelope.variants[0].entries[0].input_extents = vec![
            ExtentOperandData {
                key: InputKey::new("input").unwrap(),
                axis: Axis::new(1),
                value_type: AbiType::Unsigned,
            },
            ExtentOperandData {
                key: InputKey::new("input").unwrap(),
                axis: Axis::new(0),
                value_type: AbiType::Unsigned,
            },
        ];
    });
    assert_eq!(
        error,
        ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::ExtentOperand,
        },
        "{error:?}"
    );
}

#[test]
fn a_duplicated_live_extent_row_is_refused() {
    let row = live_extent_row();
    let error = reject_artifact_forgery(&default_artifact(), |envelope| {
        envelope.variants[0].entries[0].input_extents = vec![row.clone(), row];
    });
    assert_eq!(
        error,
        ArtifactCodecError::DuplicateItem {
            subject: OrderedSubject::ExtentOperand,
        },
        "{error:?}"
    );
}

#[test]
fn a_wrong_axis_live_extent_row_is_refused() {
    let error = reject_artifact_forgery(&default_artifact(), |envelope| {
        let mut row = live_extent_row();
        row.axis = Axis::new(99);
        envelope.variants[0].entries[0].input_extents = vec![row];
    });
    assert_eq!(
        error,
        ArtifactCodecError::ExtentOperandAxis {
            key: "input".to_owned(),
            axis: 99,
            rank: 2,
        },
        "{error:?}"
    );
}

#[test]
fn a_wrong_type_live_extent_row_is_refused() {
    let error = reject_artifact_forgery(&default_artifact(), |envelope| {
        let mut row = live_extent_row();
        row.value_type = AbiType::Boolean;
        envelope.variants[0].entries[0].input_extents = vec![row];
    });
    assert_eq!(
        error,
        ArtifactCodecError::ExtentOperandType {
            key: "input".to_owned(),
            axis: 1,
        },
        "{error:?}"
    );
}

/// A structurally sound row over the static interface refuses at decode.
///
/// The decode-side half of the construction fail-close: the row is canonical,
/// in rank, unsigned, and placed on the exact `bindings + ordinal` transport,
/// so every structural check passes and the association refusal is the one
/// that fires — on bytes no builder wrote.
#[test]
fn a_well_placed_live_extent_row_over_the_static_interface_is_refused() {
    let error = reject_artifact_forgery(&carried_artifact(b"k", b"c"), |envelope| {
        envelope.variants[0].entries[0].input_extents = vec![live_extent_row()];
        let sections = envelope.payload_content()[0].expect("the payload is carried");
        let mut metadata = decode_metadata(&envelope.sections[position(sections.metadata)].bytes)
            .expect("the subject decodes");
        metadata.entries[0].transports = vec![0, 1, 2];
        let bytes = super::super::payload::encode_metadata(&metadata);
        envelope.payloads[0].digest = super::super::payload::payload_identity(&bytes)
            .expect("a bounded subject has an identity");
        envelope.sections[position(sections.metadata)].bytes = bytes;
    });
    assert_eq!(
        error,
        ArtifactCodecError::ExtentOperandStaticAxis {
            key: "input".to_owned(),
            axis: 1,
            extent: 3,
        },
        "{error:?}"
    );
}

#[test]
fn a_misordered_extent_transport_is_refused() {
    let error = reject_artifact_forgery(&carried_artifact(b"k", b"c"), |envelope| {
        envelope.variants[0].entries[0].input_extents = vec![live_extent_row()];
        let sections = envelope.payload_content()[0].expect("the payload is carried");
        let mut metadata = decode_metadata(&envelope.sections[position(sections.metadata)].bytes)
            .expect("the subject decodes");
        metadata.entries[0].transports = vec![0, 1, 0];
        let bytes = super::super::payload::encode_metadata(&metadata);
        envelope.payloads[0].digest = super::super::payload::payload_identity(&bytes)
            .expect("a bounded subject has an identity");
        envelope.sections[position(sections.metadata)].bytes = bytes;
    });
    assert_eq!(
        error,
        ArtifactCodecError::ExtentOperandTransport {
            payload: 0,
            operand: 0,
            declared: 0,
            expected: 2,
        },
        "{error:?}"
    );
}

/// The same well-placed row over a *symbolic* interface axis is admitted.
///
/// **The positive half of the per-axis rule, and the reason the rule is per-axis
/// rather than blanket.** `check_extent_operand_static_axes` used to refuse
/// every declared row, because the decoded interface grammar carried only
/// literal extents and so every axis a row could name was fixed. Since
/// `tiler.artifact-program.v21` the entry states each axis literal-or-symbol, so
/// the rule asks its question of the axis the row actually names — and this is
/// the case the row was built for, which had no representation at all until the
/// grammar stepped.
///
/// The forgery is the sibling refusal's, plus one edit: axis 1 of the published
/// input becomes a declared symbol. Everything else — canonical order, rank,
/// unsigned type, the exact `bindings + ordinal` transport, the resealed
/// digests — is identical, so the one difference between admitting and refusing
/// is the source kind of the named axis.
#[test]
fn a_well_placed_live_extent_row_over_a_symbolic_axis_is_admitted() {
    let artifact = carried_artifact(b"k", b"c");
    let mut envelope = super::support::envelope_of(&artifact);
    envelope.semantic.retained_shape = super::forged_models::forged_retained_environment();
    envelope.inputs[0].extents[1] = super::forged_models::forged_symbol("T");
    envelope.variants[0].entries[0].input_extents = vec![live_extent_row()];
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    let mut metadata = decode_metadata(&envelope.sections[position(sections.metadata)].bytes)
        .expect("the subject decodes");
    metadata.entries[0].transports = vec![0, 1, 2];
    let bytes = super::super::payload::encode_metadata(&metadata);
    envelope.payloads[0].digest =
        super::super::payload::payload_identity(&bytes).expect("a bounded subject has an identity");
    envelope.sections[position(sections.metadata)].bytes = bytes;

    let encoded = super::super::encode::encode(&envelope).expect("the envelope encodes");
    super::super::decode::decode(&encoded)
        .expect("a live extent row over a symbolic axis is what the row exists for");
}
