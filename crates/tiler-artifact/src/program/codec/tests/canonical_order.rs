//! A non-canonical spelling is refused rather than normalized on the way in.

use super::super::super::expr::{AbiRoot, ExprNode};
use super::super::super::tests::default_artifact;
use super::super::decode::decode;
use super::super::encode::{HEADER_BYTES, encode, section_digest};
use super::super::error::{ArtifactCodecError, OrderedSubject};
use super::super::model::{ArtifactEnvelope, Section, SectionKind};
use super::support::{
    MANIFEST_LENGTH_AT, envelope_of, manifest_offset, reject_forged, reject_guarded_forgery,
    reseal, two_variant_artifact,
};
use tiler_digest::DigestAlgorithm;
use tiler_ir::program::abi::compare_expr_nodes;

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
        let canonical = super::super::super::model::canonical_deferred_order(
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
        let descending = if compare_expr_nodes(&envelope.expressions, held, extra).is_lt() {
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
/// decidable from a decoded envelope at all, as `super::super::validate` records.
#[test]
fn a_non_canonical_entry_order_is_rejected() {
    let error = reject_forged(|envelope| {
        let mut second = envelope.variants[0].entries[0].clone();
        second.stage = super::super::model::StageSubject::from_bytes(b"\xffzzz-later-stage")
            .expect("a bounded stage subject");
        second.entry_key = super::super::super::keys::BackendEntryKey::from_bytes(b"spare")
            .expect("a bounded entry key");
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

/// Permuting the arena moves the envelope's bytes and leaves its identity.
///
/// **This is what holds the `14.0` step to the wire.** The canonical arena order
/// decides which byte string is *the* encoding of an artifact, so changing the
/// relation that decides it changes the wire — and it must change nothing else.
/// It does not: `encode_identity` numbers the arena through
/// `canonical_arena_traversal`, whose numbering is a function of the use sites
/// and of the DAG beneath them and never of where a node sits in the arena, and
/// it orders both expression-bearing sets with `compare_expr_nodes` before
/// numbering anything.
///
/// Watched failing under one perturbation: asserting the two encodings *equal*
/// fails, which is what proves the permutation reached the wire instead of being
/// normalized away before it could distinguish anything.
#[test]
fn permuting_the_arena_moves_the_envelope_bytes_and_not_its_identity() {
    let artifact = default_artifact();
    let envelope = envelope_of(&artifact);
    let straight = encode(&envelope).expect("the fixture encodes");

    let mut permuted = envelope.clone();
    assert!(matches!(permuted.expressions[0], ExprNode::Root(_)));
    assert!(matches!(permuted.expressions[1], ExprNode::Root(_)));
    permuted.expressions.swap(0, 1);
    swap_expression_references(&mut permuted, 0, 1);

    assert_eq!(
        permuted
            .canonical_identity()
            .expect("the permuted envelope derives"),
        *artifact.canonical_identity(),
        "artifact identity is invariant to arena permutation",
    );
    assert_ne!(
        encode(&permuted).expect("the permuted envelope encodes"),
        straight,
        "the arena is written in the order it is stored, so the wire moved",
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
