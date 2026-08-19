//! The ABI expression arena, driven directly through the parser.

use super::super::super::MAX_VARIANT_ENTRIES;
use super::super::super::error::ArtifactDiagnostic;
use super::super::super::expr::{AbiBinaryOp, AbiRoot, AbiType, AvailabilityPhase, ExprNode};
use super::super::super::tests::default_artifact;
use super::super::decode::{Cursor, decode, parse_dependencies, parse_expression_arena};
use super::super::encode::encode;
use super::super::error::{
    ArtifactCodecError, CodecLimitKind, OrderedSubject, ReferenceSubject, TagSubject,
};
use super::support::{MANIFEST_LENGTH_AT, envelope_of, manifest_offset, reseal};
use tiler_ir::program::abi::compare_expr_nodes;
use tiler_ir::semantic::InputKey;
use tiler_ir::shape::Axis;

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

/// The five nodes on which the schema-13 historical order and comparator disagree.
///
/// Both binary nodes apply one operation to one shared right operand, so the
/// only thing separating them is their left operand — an input extent whose key
/// is long, against a target property whose key is short.
///
/// [`compare_expr_nodes`] compares those two roots by their encoded bytes, and a
/// root's encoding opens with its constructor tag: an input extent is `0x03` and
/// a target property is `0x04`, so the extent-bearing node is the smaller.
/// The retired standalone subtree key framed each operand's whole key behind an
/// eight-byte big-endian length, so comparing the two keys compared `64` against
/// `47` before reaching any content, and the *property*-bearing node was the
/// smaller. The two orders are opposite on this pair, which is what made the
/// switch a schema step rather than a refactor.
///
/// Returned in comparator order; [`schema_13_historical_arena`] is a fixed
/// transcription of the same five nodes in the earlier order.
fn comparator_ordered_arena() -> Vec<ExprNode> {
    vec![
        ExprNode::Root(AbiRoot::UnsignedLiteral(1)),
        ExprNode::Root(AbiRoot::InputExtent {
            key: InputKey::new("extent-key-aaaa").expect("a bounded interface key"),
            axis: Axis::new(1),
        }),
        ExprNode::Root(AbiRoot::TargetProperty {
            key: super::super::super::expr::TargetPropertyKey::new("p")
                .expect("a bounded governed key"),
            phase: AvailabilityPhase::LiveDevicePreflight,
        }),
        ExprNode::Binary {
            op: AbiBinaryOp::CheckedAdd,
            left: 1,
            right: 0,
        },
        ExprNode::Binary {
            op: AbiBinaryOp::CheckedAdd,
            left: 2,
            right: 0,
        },
    ]
}

/// The schema-13 historical canonical order for the five-node fixture.
///
/// This is deliberately a literal fixture, not another implementation of the
/// retired subtree-key encoder. The two binary nodes appear in the historical
/// order while their operand references are unaffected because both name roots
/// that the permutation does not move.
fn schema_13_historical_arena() -> Vec<ExprNode> {
    vec![
        ExprNode::Root(AbiRoot::UnsignedLiteral(1)),
        ExprNode::Root(AbiRoot::InputExtent {
            key: InputKey::new("extent-key-aaaa").expect("a bounded interface key"),
            axis: Axis::new(1),
        }),
        ExprNode::Root(AbiRoot::TargetProperty {
            key: super::super::super::expr::TargetPropertyKey::new("p")
                .expect("a bounded governed key"),
            phase: AvailabilityPhase::LiveDevicePreflight,
        }),
        ExprNode::Binary {
            op: AbiBinaryOp::CheckedAdd,
            left: 2,
            right: 0,
        },
        ExprNode::Binary {
            op: AbiBinaryOp::CheckedAdd,
            left: 1,
            right: 0,
        },
    ]
}

/// Encodes one arena's node records the way the manifest encoder does.
fn arena_of(nodes: &[ExprNode]) -> Vec<u8> {
    let mut envelope = envelope_of(&default_artifact());
    envelope.expressions = nodes.to_vec();
    let mut bytes = Vec::new();
    super::super::encode::encode_expressions(&mut bytes, &envelope);
    bytes
}

/// The codec's arena order follows the comparator where schema 13 disagrees.
///
/// **The disagreement is asserted rather than assumed.** The fixed historical
/// fixture is proven to be the comparator fixture with its final pair swapped,
/// then the live ordering is checked against it. A future change that made the
/// relation accept the historical order would fail here instead of leaving the
/// case testing nothing.
///
/// Watched failing under a subject perturbation before it was trusted. Replacing
/// `ReadyNode::order`'s comparison with the arena position alone made the third
/// assertion report `[0, 1, 2, 3, 4]` — and failed
/// `the_arena_parser_accepts_the_comparator_order_and_refuses_the_schema_13_historical_order`
/// at the same time.
#[test]
fn the_canonical_arena_order_follows_the_comparator_where_schema_13_disagrees() {
    let comparator = comparator_ordered_arena();
    let historical = schema_13_historical_arena();

    assert!(
        compare_expr_nodes(&comparator, 3, 4).is_lt(),
        "the extent-bearing node is structurally the smaller",
    );
    assert_eq!(&historical[..3], &comparator[..3]);
    assert_eq!(historical[3], comparator[4]);
    assert_eq!(historical[4], comparator[3]);
    assert_eq!(
        super::super::model::canonical_expression_order(&historical),
        vec![0, 1, 2, 4, 3],
        "the codec orders by structure, so the schema-13 historical order is not canonical",
    );
}

/// The arena parser accepts the comparator order and refuses schema 13's order.
///
/// The same five nodes on both sides, driven through the decode path rather than
/// through `canonical_expression_order` directly, because that is the site a
/// forger reaches.
#[test]
fn the_arena_parser_accepts_the_comparator_order_and_refuses_the_schema_13_historical_order() {
    let canonical = comparator_ordered_arena();
    assert_eq!(
        parse_expression_arena(&arena_of(&canonical)),
        Ok(canonical.clone()),
    );
    assert_eq!(
        parse_expression_arena(&arena_of(&schema_13_historical_arena())),
        Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::Expression,
        }),
    );
}

/// A manifest built from bytes alone reaches the arena parser, and pays for it.
///
/// **This is the ticket's forger-reach question, answered rather than inferred.**
/// The retained spike rows prove a *producer* can impose the arena cost and that
/// a decode ending in rejection pays it in full; what they do not prove is that
/// a manifest carrying the arena can be forged from bytes. It can:
/// `parse_expressions` runs inside `parse_manifest`, and the only thing between
/// an attacker's bytes and it is the framing header and the manifest digest —
/// which the attacker recomputes.
///
/// The forgery replaces the fixture's arena run with a chain of
/// [`FORGED_CHAIN`] nodes, repairs the manifest length, the total length, and the
/// manifest digest, and changes nothing else. The rejection it earns is
/// [`ArtifactDiagnostic::UnusedExpression`], raised by `super::super::validate` — which
/// runs only after `parse_manifest` returned, so the whole chain was parsed,
/// type-checked, proven distinct, and proven canonically ordered before anything
/// refused it. At `13.0` that same path also materialized a content-key table
/// quadratic in the chain's length.
///
/// Watched failing under one perturbation: omitting the manifest-digest repair
/// makes it report `ManifestDigestMismatch`, which is the shallow refusal this
/// case exists to get past.
#[test]
fn a_forged_manifest_reaches_the_arena_parser_before_any_identity_check() {
    /// Long enough that the forged arena cannot be mistaken for the fixture's,
    /// and short enough that a debug-profile test stays fast. The governed bound
    /// is `MAX_ABI_EXPRESSIONS`, and nothing here depends on reaching it.
    const FORGED_CHAIN: usize = 512;

    let envelope = envelope_of(&default_artifact());
    let mut bytes = encode(&envelope).expect("the fixture encodes");
    let carried = arena_of(&envelope.expressions);

    // One unsigned root, then a chain that adds it to the running total. Every
    // node's operands precede it, no two nodes are structurally equal, and only
    // one node is ever ready, so the chain is already in canonical order.
    let mut chain = vec![ExprNode::Root(AbiRoot::UnsignedLiteral(0))];
    for index in 1..FORGED_CHAIN {
        chain.push(ExprNode::Binary {
            op: AbiBinaryOp::CheckedAdd,
            left: u32::try_from(index - 1).expect("a bounded arena fits u32"),
            right: 0,
        });
    }
    assert!(
        chain.len() > envelope.expressions.len(),
        "the forged arena must cover every reference the fixture's manifest makes",
    );
    let forged = arena_of(&chain);

    let at = manifest_offset(&bytes, &carried);
    let mut spliced = bytes[..at].to_vec();
    spliced.extend_from_slice(&forged);
    spliced.extend_from_slice(&bytes[at + carried.len()..]);
    bytes = spliced;

    let manifest_len = u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ) + u64::try_from(forged.len() - carried.len())
        .expect("supported usize fits u64");
    bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8].copy_from_slice(&manifest_len.to_be_bytes());
    reseal(&mut bytes);

    assert_eq!(
        decode(&bytes).expect_err("a forged arena is refused"),
        ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::UnusedExpression,
        },
        "the refusal must come from validate, which runs after the whole arena is parsed",
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
            resource: super::super::error::CodecLimitKind::Expressions,
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
    // The builder and model cannot admit an out-of-range guard, and identity's
    // canonical traversal assumes that construction invariant. Mutate the wire
    // after an honest encode to reach the parser's reference check directly.
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
