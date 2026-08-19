//! Competent forgeries that reseal everything, refused by a named cause.

use super::super::super::error::{
    AbiExprUse, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind,
};
use super::super::super::expr::{AbiRoot, AbiType, AbiValue, AvailabilityPhase, ExprNode};
use super::super::super::facts::AbiFactBinder;
use super::super::super::keys::BackendEntryKey;
use super::super::super::tests::{
    ELEMENT_BYTES, SCALE_BITS, SCRATCH_OFFSET, declare_realization, default_artifact, formulas,
    fused_program, lowering_provider, partial_window_artifact, payload, prepared_requirement,
    selection, semantic_program, variant,
};
use super::super::super::{ArtifactProgramBuilder, CompilationEnvironment};
use super::super::decode::decode;
use super::super::encode::encode;
use super::super::error::{ArtifactCodecError, OrderedSubject};
use super::super::model::{
    FEATURE_MULTI_STAGE_PROGRAM, FEATURE_MULTI_VARIANT_ROUTING, Section, SectionKind,
};
use super::support::{envelope_of, reject_forged, reject_guarded_forgery, two_variant_artifact};
use tiler_ir::program::abi::TargetPropertyRequirementRelation;

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
                super::super::super::keys::PayloadDigest::from_bytes([0xff, 0xff, 0xff]).unwrap();
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
fn two_entries_claiming_one_backend_entry_are_rejected() {
    let artifact = two_variant_artifact(true);
    let mut envelope = envelope_of(&artifact);
    let shared = envelope.variants[0].entries[0].entry_key.clone();
    let payloads = envelope.variants[0].entries[0].payloads.clone();
    envelope.variants[1].entries[0].entry_key = shared;
    envelope.variants[1].entries[0].payloads = payloads;
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::DuplicateBackendEntry,
        }),
    );
}

/// An envelope whose entries disagree about delivery positions is refused.
///
/// Re-proven from bytes rather than inherited from construction: the builder
/// refuses this at insertion, and an envelope a decoder is handed may have been
/// written by no builder at all. The forgery drops one entry's second
/// realization, leaving a consumer resolving position 1 with no object for a
/// stage the route must dispatch.
#[test]
fn an_entry_realized_at_fewer_delivery_positions_is_rejected() {
    let artifact = two_variant_artifact(true);
    let mut envelope = envelope_of(&artifact);
    // Every entry gains a second position, then one loses it again.
    let payloads: Vec<u32> = (0..envelope.payloads.len())
        .map(|payload| u32::try_from(payload).expect("a bounded payload table fits u32"))
        .collect();
    assert_eq!(payloads.len(), 2, "the fixture carries two payloads");
    for variant in &mut envelope.variants {
        for entry in &mut variant.entries {
            entry.payloads = payloads.clone();
        }
    }
    envelope.variants[1].entries[0].payloads.pop();
    envelope.features = envelope.derived_features();
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ModelRule {
            cause: Box::new(
                super::super::super::ArtifactBuildError::DeliveryCardinality {
                    entry: 1,
                    expected: 2,
                    actual: 1,
                }
            ),
        }),
    );
}

/// An envelope reaching one payload from two delivery positions is refused.
///
/// The cross-entry case no existing obligation sees: every payload is
/// referenced and no `(payload, entry key)` pair repeats. What is wrong is that
/// the envelope declares two consumer build targets and mixes one object
/// between their positions.
#[test]
fn a_payload_reached_from_two_delivery_positions_is_rejected() {
    let artifact = two_variant_artifact(true);
    let mut envelope = envelope_of(&artifact);
    envelope.variants[0].entries[0].payloads = vec![0, 1];
    envelope.variants[1].entries[0].payloads = vec![1, 0];
    envelope.features = envelope.derived_features();
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::AmbiguousPayloadDelivery { payload: 1 },
        }),
    );
}

/// A multi-position envelope declares the governed feature that says so.
///
/// A reader that resolves no delivery position would take whichever object came
/// first, which is correct for a one-position artifact and silently wrong for
/// any other, so the requirement is content-derived and stated rather than
/// implied. A one-position artifact emits no key.
#[test]
fn several_delivery_positions_require_their_governed_feature() {
    let artifact = default_artifact();
    let envelope = envelope_of(&artifact);
    assert_eq!(envelope.delivery_positions(), 1);
    assert!(
        !envelope
            .features()
            .iter()
            .any(|feature| feature == super::super::model::FEATURE_MULTI_PAYLOAD_DELIVERY),
    );

    let artifact = two_variant_artifact(true);
    let mut envelope = envelope_of(&artifact);
    envelope.variants[0].entries[0].payloads = vec![0, 1];
    envelope.variants[1].entries[0].payloads = vec![0, 1];
    envelope.features = envelope.derived_features();
    assert_eq!(envelope.delivery_positions(), 2);
    assert!(
        envelope
            .features()
            .iter()
            .any(|feature| feature == super::super::model::FEATURE_MULTI_PAYLOAD_DELIVERY),
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
    let decoded =
        super::super::view::decode_artifact(&bytes).expect("a two-stage artifact decodes");

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
fn a_deferred_requirement_that_disagrees_with_its_predicate_is_rejected() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider.clone())).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.deferred_predicates = vec![super::super::super::DeferredPredicateSpec {
        requirement: prepared_requirement(
            1,
            TargetPropertyRequirementRelation::ObservedAtLeastRequired,
        ),
        entry: 0,
    }];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().unwrap();

    let mut envelope = envelope_of(&artifact);
    envelope.variants[0].deferred[0].requirement =
        prepared_requirement(1, TargetPropertyRequirementRelation::ObservedEqualsRequired);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::DeferredQueryPredicateMismatch),
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
