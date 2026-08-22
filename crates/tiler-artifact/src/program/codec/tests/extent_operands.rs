//! Live input-extent rows and the transports that carry them.

use super::super::super::error::ArtifactBuildError;
use super::super::super::expr::AbiType;
use super::super::super::model::ExtentOperandData;
use super::super::super::tests::default_artifact;
use super::super::error::{ArtifactCodecError, OrderedSubject};
use super::super::model::{ArtifactEnvelope, position};
use super::super::payload::decode_metadata;
use super::forged_models::{forged_environment_rooting, forged_symbol};
use super::support::{carried_artifact, envelope_of, reject_artifact_forgery};
use tiler_ir::semantic::InputKey;
use tiler_ir::shape::Axis;

fn live_extent_row() -> ExtentOperandData {
    ExtentOperandData {
        key: InputKey::new("input").unwrap(),
        axis: Axis::new(1),
        value_type: AbiType::Unsigned,
    }
}

/// Declares live-extent rows on the carried entry and places them on the given
/// backend transport slots, resealing the payload their mapping lives in.
///
/// Every association case below needs a row that is structurally *beyond*
/// reproach — canonical, in rank, unsigned, and on the exact
/// `binding_count + ordinal` slot the accepted Metal `eN` ABI requires — or the
/// narrower structural refusal fires and the association is never reached. The
/// slot list stays a parameter because one case is about the transport itself.
fn place_live_rows(
    envelope: &mut ArtifactEnvelope,
    rows: Vec<ExtentOperandData>,
    transports: Vec<u32>,
) {
    envelope.variants[0].entries[0].input_extents = rows;
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    let mut metadata = decode_metadata(&envelope.sections[position(sections.metadata)].bytes)
        .expect("the subject decodes");
    metadata.entries[0].transports = transports;
    let bytes = super::super::payload::encode_metadata(&metadata);
    envelope.payloads[0].digest =
        super::super::payload::payload_identity(&bytes).expect("a bounded subject has an identity");
    envelope.sections[position(sections.metadata)].bytes = bytes;
}

/// Rejects a carried-artifact forgery whose one live row is well placed.
fn reject_placed_row(forge: impl FnOnce(&mut ArtifactEnvelope)) -> ArtifactCodecError {
    reject_artifact_forgery(&carried_artifact(b"k", b"c"), |envelope| {
        forge(envelope);
        place_live_rows(envelope, vec![live_extent_row()], vec![0, 1, 2]);
    })
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
    let error = reject_placed_row(|_| {});
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
        place_live_rows(envelope, vec![live_extent_row()], vec![0, 1, 0]);
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

/// The same well-placed row over the axis its symbol is *rooted at* is admitted.
///
/// **The positive half of the association, and the only shape of decoded row
/// that has one.** The environment roots `S` at `input[1]`, the published
/// interface names `S` at that same axis, and the row names `(input, 1)` — so
/// the extent a loader freezes onto the operand's transport is the extent of
/// the very dimension the symbol is defined as. Everything else is the sibling
/// refusals' forgery exactly: canonical order, rank, unsigned type, the
/// `binding_count + ordinal` transport, resealed digests.
///
/// This test read `forged_retained_environment()` and a `T` at `input[1]` until
/// 2026-08-22, and asserted that decode *admitted* it. `T` is bound to a static
/// extent there, so the combination is one `check_extent_operand_association`
/// refuses at construction: the case pinned as the positive control was outside
/// the population any builder can produce, and the assertion recorded the
/// fail-open gap rather than the rule. It is now the negative case
/// `a_row_over_a_statically_rooted_symbol_is_refused` below.
#[test]
fn a_well_placed_live_extent_row_over_its_own_root_axis_is_admitted() {
    let artifact = carried_artifact(b"k", b"c");
    let mut envelope = envelope_of(&artifact);
    envelope.semantic.retained_shape = forged_environment_rooting("input", 1);
    envelope.inputs[0].extents[1] = forged_symbol("S");
    place_live_rows(&mut envelope, vec![live_extent_row()], vec![0, 1, 2]);

    let encoded = super::super::encode::encode(&envelope).expect("the envelope encodes");
    super::super::decode::decode(&encoded)
        .expect("a row naming its own symbol's root dimension is what the row exists for");
}

/// A row over a symbol the environment roots at a *static* extent is refused.
///
/// The operand transports an input-dimension fact. A symbol bound to a fixed
/// extent is already answered by that binding, so a per-invocation operand
/// claiming it would give one symbol two authorities and let a caller select a
/// value the artifact states is constant.
#[test]
fn a_row_over_a_statically_rooted_symbol_is_refused() {
    let error = reject_placed_row(|envelope| {
        envelope.semantic.retained_shape = forged_environment_rooting("input", 0);
        envelope.inputs[0].extents[1] = forged_symbol("T");
    });
    assert_eq!(
        error,
        ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::ExtentOperandUnsourcedSymbol {
                entry: 0,
                key: "input".to_owned(),
                axis: 1,
                symbol: "forged/0::T".to_owned(),
                source: "static 2".to_owned(),
            }),
        },
        "{error:?}"
    );
}

/// A row over a symbol the environment roots at another *axis* is refused.
///
/// `S` is rooted at `input[0]`; an `input[1]` that merely spells `S` is an
/// inferred occurrence of it. Admitting the row would freeze axis 1's extent
/// onto a transport whose kernel reads it as the value of a symbol defined at
/// axis 0, and nothing proves the two equal: the retained constraints decide
/// only relations an author declared.
#[test]
fn a_row_over_a_symbol_rooted_at_another_axis_is_refused() {
    let error = reject_placed_row(|envelope| {
        envelope.semantic.retained_shape = forged_environment_rooting("input", 0);
        envelope.inputs[0].extents[1] = forged_symbol("S");
    });
    assert_eq!(
        error,
        ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::ExtentOperandSourceMismatch {
                entry: 0,
                key: "input".to_owned(),
                axis: 1,
                root_key: "input".to_owned(),
                root_axis: 0,
            }),
        },
        "{error:?}"
    );
}

/// A row over a symbol the environment roots at another *input* is refused.
///
/// The sibling above perturbs the root axis; this one perturbs the root key.
/// The association admits on `input == key && root_axis == axis`, so a single
/// case cannot show both conjuncts are load-bearing.
#[test]
fn a_row_over_a_symbol_rooted_at_another_input_is_refused() {
    let error = reject_placed_row(|envelope| {
        envelope.semantic.retained_shape = forged_environment_rooting("elsewhere", 1);
        envelope.inputs[0].extents[1] = forged_symbol("S");
    });
    assert_eq!(
        error,
        ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::ExtentOperandSourceMismatch {
                entry: 0,
                key: "input".to_owned(),
                axis: 1,
                root_key: "elsewhere".to_owned(),
                root_axis: 1,
            }),
        },
        "{error:?}"
    );
}

/// A row over a symbol the environment does not declare is refused — by the
/// interface's own coherence, which owns that fact.
///
/// The construction-side association has a fourth arm for it,
/// `ExtentOperandForeignSymbol`. It cannot fire on decoded bytes and should
/// not: `check_interface_symbol_coherence` runs first and refuses *any*
/// interface axis naming an undeclared symbol, row or no row. That is the more
/// fundamental of the two faults — such an artifact publishes an axis nothing
/// can bind — so an operand-shaped refusal here would name a consequence and
/// leave the cause unreported. Pinned so the arm is watched refusing rather
/// than assumed to.
#[test]
fn a_row_over_a_symbol_the_environment_does_not_declare_is_refused() {
    let error = reject_placed_row(|envelope| {
        envelope.semantic.retained_shape = forged_environment_rooting("input", 0);
        envelope.inputs[0].extents[1] = forged_symbol("undeclared");
    });
    assert_eq!(
        error,
        ArtifactCodecError::UndeclaredInterfaceSymbol {
            key: "input".to_owned(),
            axis: 1,
            symbol: "forged/0::undeclared".to_owned(),
        },
        "{error:?}"
    );
}

/// The narrower structural refusals still fire first on a doubly broken row.
///
/// The negative control for the siting: a row list that is *both* misordered
/// *and* mis-rooted must still report the order, because a producer told that
/// its association is wrong would fix the association and hit the ordering
/// next. Deliberately not a check that the association never runs — it is a
/// check that it runs *after* the structural rules the row is also breaking.
#[test]
fn a_misordered_and_mis_rooted_row_list_still_reports_the_order() {
    let error = reject_artifact_forgery(&carried_artifact(b"k", b"c"), |envelope| {
        envelope.semantic.retained_shape = forged_environment_rooting("input", 0);
        envelope.inputs[0].extents[1] = forged_symbol("S");
        place_live_rows(
            envelope,
            vec![
                live_extent_row(),
                ExtentOperandData {
                    key: InputKey::new("input").unwrap(),
                    axis: Axis::new(0),
                    value_type: AbiType::Unsigned,
                },
            ],
            vec![0, 1, 2, 3],
        );
    });
    assert_eq!(
        error,
        ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::ExtentOperand,
        },
        "{error:?}"
    );
}
