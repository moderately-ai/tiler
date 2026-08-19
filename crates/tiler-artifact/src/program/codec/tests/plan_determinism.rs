//! Plan-determinism scope cells and target-environment records (ADR 0013).

use super::super::super::environment::PlanDeterminismScope;
use super::super::super::error::ArtifactDiagnostic;
use super::super::super::tests::{
    CLAIM_DESCRIPTOR, claim_declaration, claim_declaration_of, claimed_artifact,
    claimed_two_entry_artifact, default_artifact,
};
use super::super::super::{
    MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES, TargetEnvironmentDeclarationError,
};
use super::super::decode::decode;
use super::super::encode::encode;
use super::super::error::{ArtifactCodecError, CodecLimitKind, TagSubject};
use super::super::model::position;
use super::super::view::decode_artifact;
use super::support::{encoded, envelope_of, insert_manifest_bytes, manifest_offset, reseal};
use tiler_ir::identity::push_len;

// -------------------------------------------------------------------------
// Plan-determinism scope and target-environment records (ADR 0013)
// -------------------------------------------------------------------------

/// A claimed cell and its payload's declaration round-trip exactly.
///
/// The identity equality is the claim consumers rest on: the wire form of a
/// claimed artifact names the same canonical identity the builder minted, so a
/// pin taken at build time still names the decoded artifact.
#[test]
fn a_plan_claim_round_trips_through_the_public_codec() {
    let artifact = claimed_artifact();
    let bytes = encoded(&artifact);
    let decoded = decode_artifact(&bytes).expect("a claimed artifact decodes");
    assert_eq!(decoded.identity(), *artifact.canonical_identity());
    let variant = decoded.variants().next().expect("the one claimed variant");
    assert_eq!(
        variant.plan_determinism_scope(0),
        Some(PlanDeterminismScope::Plan)
    );
    assert_eq!(variant.plan_determinism_scope(1), None);
    assert_eq!(decoded.payloads()[0].environment, Some(claim_declaration()));
}

/// Two entries claiming one cell decode when their declarations agree.
///
/// The positive control for the cross-payload disagreement forgery below: the
/// coherence check refuses the disagreement, not two-entry claims as such.
#[test]
fn an_agreeing_two_entry_claim_decodes() {
    let artifact = claimed_two_entry_artifact();
    let decoded = decode_artifact(&encoded(&artifact)).expect("agreeing entries are coherent");
    let variant = decoded.variants().next().expect("the claimed variant");
    assert_eq!(
        variant.plan_determinism_scope(0),
        Some(PlanDeterminismScope::Plan)
    );
}

/// A scope run of the wrong cardinality is refused from the wire.
#[test]
fn a_scope_run_with_the_wrong_cardinality_is_refused() {
    let mut envelope = envelope_of(&claimed_artifact());
    envelope.variants[0]
        .scope
        .push(PlanDeterminismScope::Unclaimed);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::PlanDeterminismScopeCardinality {
                variant: 0,
                cells: 2,
                positions: 1,
            },
        }),
    );
}

/// A `Plan` cell over a payload that never declared an environment is refused.
///
/// The forged claim is exactly what a neutral reader must not admit: the cell
/// promises the ADR 0013 subject, and the payload carries neither the
/// declaration nor the object that subject binds.
#[test]
fn a_plan_cell_over_an_undeclared_payload_is_refused() {
    let mut envelope = envelope_of(&default_artifact());
    envelope.variants[0].scope[0] = PlanDeterminismScope::Plan;
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::UnverifiedPlanDeterminismClaim {
                variant: 0,
                delivery: 0,
                entry: 0,
            },
        }),
    );
}

/// A claimed cell whose entries' payloads disagree on environment is refused.
///
/// Each payload row is individually well formed; what the forgery breaks is
/// only the cross-entry agreement the builder's join proved, so the refusal
/// names the disagreeing entry.
#[test]
fn a_plan_cell_whose_payloads_disagree_on_environment_is_refused() {
    let mut envelope = envelope_of(&claimed_two_entry_artifact());
    let second = position(envelope.variants[0].entries[1].payloads[0]);
    envelope.payloads[second].environment = Some(claim_declaration_of(b"process-arithmetic-v2"));
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::UnverifiedPlanDeterminismClaim {
                variant: 0,
                delivery: 0,
                entry: 1,
            },
        }),
    );
}

/// An unknown plan-determinism scope tag is refused by name.
#[test]
fn an_unknown_plan_determinism_scope_tag_is_refused() {
    let mut bytes = encoded(&claimed_artifact());
    // The variant row's tail is fixed for this one-entry fixture: the
    // execution order (one packaged entry), the empty dependency run, and the
    // one-cell scope run whose `Plan` tag is the pattern's last byte.
    let mut pattern = Vec::new();
    push_len(&mut pattern, 1);
    pattern.extend_from_slice(&0_u32.to_be_bytes());
    push_len(&mut pattern, 0);
    push_len(&mut pattern, 1);
    pattern.push(0x02);
    let offset = manifest_offset(&bytes, &pattern);
    bytes[offset + pattern.len() - 1] = 0x7f;
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::PlanDeterminismScope,
            tag: 0x7f,
        }),
    );
}

/// An unknown target-environment presence tag is refused by name.
#[test]
fn an_unknown_target_environment_presence_tag_is_refused() {
    let mut bytes = encoded(&claimed_artifact());
    // The record is presence tag, framed namespace, framed name: the unique
    // provider name anchors the walk back across the two frames to the tag.
    let name_at = manifest_offset(&bytes, b"environment-authority");
    let presence_at = name_at - 8 - b"tiler-test".len() - 8 - 1;
    assert_eq!(
        bytes[presence_at], 0x01,
        "the walk-back must land on the presence tag"
    );
    bytes[presence_at] = 0x5a;
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::TargetEnvironmentPresence,
            tag: 0x5a,
        }),
    );
}

/// A declared environment with schema major zero is refused from the wire.
///
/// The builder-side grammar refuses the same declaration at construction;
/// this re-proves the bound from bytes, where no builder ran.
#[test]
fn a_zero_schema_major_environment_is_refused_from_the_wire() {
    let mut bytes = encoded(&claimed_artifact());
    // Provider name, four-byte revision, then the two-byte schema major.
    let offset = manifest_offset(&bytes, b"environment-authority");
    let major_at = offset + b"environment-authority".len() + 4;
    bytes[major_at..major_at + 2].copy_from_slice(&0_u16.to_be_bytes());
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::InvalidTargetEnvironment {
            cause: TargetEnvironmentDeclarationError::ZeroSchemaMajor,
        }),
    );
}

/// A descriptor one byte over the governed budget is refused as that budget.
///
/// The oversize is spliced into real bytes because the model-side grammar
/// refuses to construct one: the decoder must bound the length before the
/// grammar sees it, and this proves that bound is reachable and named.
#[test]
fn an_oversized_environment_descriptor_is_refused_as_a_budget() {
    let mut bytes = encoded(&claimed_artifact());
    let oversize = MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES + 1;
    let descriptor_at = manifest_offset(&bytes, CLAIM_DESCRIPTOR);
    let filler = vec![0x61; oversize - CLAIM_DESCRIPTOR.len()];
    insert_manifest_bytes(&mut bytes, descriptor_at + CLAIM_DESCRIPTOR.len(), &filler);
    let length_at = descriptor_at - 8;
    bytes[length_at..length_at + 8].copy_from_slice(
        &u64::try_from(oversize)
            .expect("the forged length fits u64")
            .to_be_bytes(),
    );
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::Limit {
            resource: CodecLimitKind::TargetEnvironmentDescriptorBytes,
            actual: u64::try_from(oversize).expect("the forged length fits u64"),
            limit: u64::try_from(MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES)
                .expect("the governed budget fits u64"),
        }),
    );
}
