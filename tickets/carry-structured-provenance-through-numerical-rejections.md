---
id: carry-structured-provenance-through-numerical-rejections
title: Carry structured provenance through numerical rejections
status: in-progress
priority: p1
dependencies: [carry-the-honourability-fact-provenance-into-the-artifact-record]
related: [redesign-the-delivered-realization-record-from-typed-evidence]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, provenance]
claimed_from: todo
assignee: loop-carry-struct
lease_expires_at: 1785517250
---
## User-visible outcome

A numerical rejection retains and exposes the exact checked honourability fact that refused the required behaviour, including its authority, validity scope, compiler builds, and execution environments. Diagnostics can therefore explain why a target rejected a contract without collapsing measured evidence into scalar means and a profile key.

## Implementation keys

`CheckedTargetProfile::resolve_dimension` currently carries the exact checked fact only through `HonouredDimension`. Its refusing branch reconstructs `UnhonouredDimension` from dimension, arithmetic type, required behaviour, means, one honoured alternative, and profile identity, losing `FactSourceProvenance` before `Rejection`, pipeline trace, and explain output.

Retain the exact refusing `NumericalHonourabilityFact` by shared immutable ownership or an equivalent checked reference that survives the rejection pipeline. Keep the caller-required behaviour separate from the declaration: the required value and the refusing fact answer different questions. Synthetic rejection fixtures must construct honest checked evidence rather than using a provenance-free escape hatch.

Prove the path end to end from feasibility through `Rejection`, `OpaqueCallRejectionCause`, pipeline trace, and the proposed borrowed diagnostic view. Mutate only compiler build or execution environment and show the rejection evidence and its identity/rendered explanation change. Prove each new check can fail.

The internal correction is singular: `UnhonouredDimension` retains the exact checked `NumericalHonourabilityFact` plus the caller-required behaviour and any honoured alternative. Existing `ContractRejection`, feasibility `RejectionCause`, `FrontierRejection`, and `OpaqueCallRejectionCause` carry that value onward. Every canonical rejection encoder and `ExplainEvent`/renderer/schema must encode the complete fact and provenance exhaustively.

The public session facade exposes a borrowed read-only refusal view with typed accessors for required behaviour, declared behaviour, structured means, authority, validity scope, compiler builds, execution environments, and profile. It does not expose internal `Arc` storage, compiler-private checked structs, or editable provenance. Contract-resolution failures that occur before an explain writer exists still return this typed facade; frontier failures additionally retain the same evidence in explain identity and rendering.

## Required evidence

One checked fact reaches contract rejection, frontier rejection, opaque-call rejection, the public borrowed refusal view, and explain rendering without reconstruction. Mutating only authority, validity, compiler build, or execution environment changes the exact refusal identity/rendering while leaving the required behavior unchanged. Provenance-free fixture construction fails, and every new check is perturbed once and observed failing.

## Closes when

Every numerical rejection path retains the exact checked fact and complete provenance; pre-trace and traced refusals expose the same evidence through their appropriate typed surfaces; all exhaustive encoders and explain schema/domain versions are advanced with merged-tree rebaselines; the exact public diagnostic facade is reviewed by Tom; and targeted compiler nextest/Clippy plus `make full` pass.

## Graph maintenance

This follows the selected-evidence foundation rather than expanding it. It must land before any artifact or public diagnostic boundary claims that all ADR 0076 rejection provenance is readable. Relate any public borrowed view to the same facade review as `redesign-the-delivered-realization-record-from-typed-evidence`; do not publish internal `Arc` storage or compiler-private verified structs.

## Outcome

**Fact — the internal correction landed.** `UnhonouredDimension` now holds the exact refusing `NumericalHonourabilityFact` behind a private `Arc`, beside the caller-required behaviour and any honoured alternative; the dimension, arithmetic type, resolved type, declared behaviour, means, and declaring profile are read from that fact rather than copied beside it. `ContractRejection`, feasibility `RejectionCause`, `FrontierRejection::Unhonourable`, and `OpaqueCallRejectionCause::TargetUnhonourable` carry the same instance onward, checkable by pointer through `NumericalRefusalEvidence::cites_same_fact`. One `UnhonouredDimension::encode` replaced the two divergent copies in `frontier.rs`, and every provenance encoder now destructures its struct so a new field is a build error.

**Fact — explain advanced.** `EXPLAIN_SCHEMA_VERSION` 8 → 9 and `EXPLAIN_RENDERER_VERSION` 6 → 7, because `HonourabilityOutcome::Unhonourable` now encodes and renders the complete refusing fact. Under v8 two profiles refusing the same behaviour on different measured compiler builds produced identical trace identities and identical renderings. `PROFILE_DESCRIPTOR_DOMAIN` and `GOVERNED_FEASIBILITY_RULE_SET` are deliberately unchanged: the profile's declared facts and the rules comparing them are what those name, and neither changed.

**Decision for Tom — the public diagnostic facade is a concrete draft, not self-accepted.** `session::TargetDeclaredNumericalRefusal` gains `declared()` and `evidence()`; `evidence()` returns the borrowed `target::TargetNumericalRefusalEvidence<'_>`, whose read-side vocabulary (`TargetFactAuthority`, `TargetFactValidityScope`, `TargetProvenanceReference`, `TargetNumericalEvidenceBasis`, `TargetMeasurementContexts`, `TargetMeasurementContextReference`, `TargetCompilerBuilds`, `TargetCompilerBuildReference`, `TargetCompilerRoleReference`, `TargetExecutionEnvironmentReference`) lives in `target` beside the write-side declaration vocabulary it mirrors. Two shapes need review with the same facade pass as `redesign-the-delivered-realization-record-from-typed-evidence`: whether the read-side authority and validity enums belong beside `MeasuredFactAuthority` in `target` or on the session facade, and whether the caller-required behaviour should be restated on the refusal view. It is not restated today — it lives one level up on `TargetNumericalContractRejection::requirement`, which every disposition shares — and duplicating it would put one fact in two public places.
