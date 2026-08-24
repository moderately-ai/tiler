---
id: admit-an-invocation-scoped-gather-index-validation-receipt
title: Admit an invocation-scoped gather-index validation receipt
status: in-progress
priority: p1
dependencies: [admit-the-selected-data-dependent-index-representation, admit-a-storage-carrier-for-integer-program-inputs, carry-the-invocation-gather-requirement-through-refinement, decide-the-conditional-coverage-authority-for-invocation-gather-validation, decide-the-invocation-gather-artifact-obligation-and-old-reader-fence, decide-the-host-gather-snapshot-receipt-and-preflight-surface]
related: [accept-the-invocation-scoped-gather-validation-public-surface, validate-device-resident-gather-indices-before-dispatch, admit-a-zero-copy-exclusive-lease-for-validated-gather-indices, generalize-invocation-bound-index-validation-beyond-gather]
scopes: [implementation/ir, implementation/artifact, implementation/compiler, implementation/runtime, implementation/build, implementation/frontend, implementation/conformance, contracts/artifacts, contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, gather, validation, fail-closed, public-boundary, identity]
claimed_from: todo
assignee: worker-gatherreceipt
lease_expires_at: 1787593115
---
## Exact-base Fact audit — 2026-08-24, `b415dac1` / base `6e713e12`

- **False — the user-visible outcome is not present.** `crates/tiler-compiler/src/physical.rs`, anchor `Err(RegionVocabularyWall::GatherIndexBoundsUnproved)`, still refuses the dynamic population. `crates/tiler/src/route.rs`, anchor `dispatch_embedded_route`, borrows and forwards adapter storage without gather preflight or an immutable snapshot.
- **Verified — static proof remains the receipt-free first lane.** `GatherIndexBoundsResolution::StaticallyProved` in `crates/tiler-ir/src/index/model.rs` reaches the compiler's proved path, and `a_statically_proved_gather_compiles_with_its_index_at_its_own_carrier` in `crates/tiler-compiler/src/request/tests/gather.rs` pins it. No invocation-validation receipt participates.
- **Imprecise — the ingredients of the proposed dynamic lane exist separately, not as support.** `pub fn decide_gather_index` in `crates/tiler-ir/src/semantic/gather.rs` owns the semantic bounds rule and `DispatchAdapter::storage` in `crates/tiler/src/value.rs` exposes borrowed host storage, but no preflight validator or receipt-owned copy exists in `crates/`.
- **False — no sealed invocation receipt exists.** There is no receipt, snapshot, generation, or duplicate-consumption type for gather validation. `pub struct Preflight` in `crates/tiler-runtime/src/load/route.rs` is the current one-way route authority, but it binds no input-validation evidence.
- **False — no artifact or runtime obligation exists.** `VariantData` in `crates/tiler-artifact/src/program/model.rs` carries no invocation-validation run. `RouteRequirement` is explicitly a live-device fact family under `What belongs here, and the test that decides it` in `crates/tiler-artifact/src/program/requirement.rs`, so it cannot be reused for a semantic input check.
- **False — ADR 0108 accepts no exact artifact domain or version.** Its anchor `Use fresh, framed tags` requires a compatibility fence and conditionally requires a major schema/domain step; it assigns no obligation row tag, required-feature key, component/schema version, artifact domain, receipt domain, or runtime error spelling. The accepted surface packet says `The separate receipt ticket alone owns validation bytes`, runtime carriage, and the later transition to coverage. The current authorities remain `MANIFEST_SCHEMA: (22, 0)` in `crates/tiler-artifact/src/program/codec/encode.rs` and `ARTIFACT_DOMAIN` at `tiler.artifact-program.v22\0` in `crates/tiler-artifact/src/program/model.rs`.
- **Imprecise — the typed semantic out-of-range half exists, while the runtime refusal set does not.** `GatherError::IndexOutOfBounds` names position, value, and extent and `decide_gather_index` never clamps or wraps. `LoadRejection` and the facade's `BindError` carry no gather-validation, missing, stale, crossed, or consumed-evidence variants, and no route decides the forbidden-fallback precedence.
- **False — the excluded population and required evidence are not implemented.** With no receipt lane, mutable zero-copy, device-resident or device-produced indices, callbacks, assertions, and inline checks have no exact refusal boundary. There are no obligation/receipt types to census and none of the occurrence, extent, type, binding, snapshot, generation, consumption, mutation, fallback, or exact-dispatched-byte perturbations exist.
- **False — the implementation is not mechanically ready.** The accepted `InvocationGatherIndexValidationRequirement`, `PendingInvocationIndexValidation`, and their third outcomes do not exist. `ResolvedIndexRealization::verify_sequence` in `crates/tiler-ir/src/index/refinement/receipt.rs` considers only `unknown_index_domain_predicates`, so it does not retain a gather's intrinsic dynamic requirement. `BoundsProofKind::GatherSource` still requires timeless proof and there is no conditional schedule, executable-coverage, kernel-program, or artifact spelling.

**Correction, with purpose preserved.** The strict contract used to say: “The artifact and runtime carry a mandatory named validation obligation under the exact domain/version decision ADR 0108 accepts.” That exact-decision premise is false. The implementation remains the intended outcome, but it now waits on four precise owners: [`carry-the-invocation-gather-requirement-through-refinement`](carry-the-invocation-gather-requirement-through-refinement.md), [`decide-the-conditional-coverage-authority-for-invocation-gather-validation`](decide-the-conditional-coverage-authority-for-invocation-gather-validation.md), [`decide-the-invocation-gather-artifact-obligation-and-old-reader-fence`](decide-the-invocation-gather-artifact-obligation-and-old-reader-fence.md), and [`decide-the-host-gather-snapshot-receipt-and-preflight-surface`](decide-the-host-gather-snapshot-receipt-and-preflight-surface.md). They restore the missing compiler carrier and own the three unresolved authorities without moving this ticket's status or substituting a narrower implementation.

## Root cause and mechanical readiness prevention

The broad representation dependency closed `done` while its own `Remaining work - not landed here` section still named `InvocationGatherIndexValidationRequirement`, both third outcomes, `PendingInvocationIndexValidation`, and the dynamic compiler reason. This ticket depended on that terminal parent rather than on an explicit carrier for its remainder. Its original 35-line body then promoted ADR 0108's conditional compatibility rule into a nonexistent “exact domain/version decision” and supplied no check that the compiler, artifact, and runtime seams it assumed were present. The graph could therefore report every dependency satisfied while all three owning layers were absent.

Terminal status alone is not the readiness check after this repair:

- The refinement carrier closes only when its typed outcome census includes the third arm, a dynamic gather produces that arm, and independently changing it back to `Verified` fails a test that says no timeless receipt or executable coverage was minted.
- The conditional-coverage decision closes only with the exact schedule/program authority, identity domain and transition written down; crossing a requirement into the existing proof or `CoveredOccurrence` path must be an explicit negative control, not an option left to implementation.
- The artifact-fence decision closes only with exact row fields/order/tags, feature key if used, component/manifest/domain versions, cache consequence, and two separate old-reader perturbations: one new row with its fence intact and one with the fence removed. Each must name the refusal or demonstrate why a major step is required.
- The runtime-surface decision closes only with exact attempt/receipt/error/call-site spellings and independent controls for original-storage mutation, snapshot mutation, occurrence/extent/type/binding/generation crossing, duplicate consumption, missing evidence, variant substitution, reference execution, and backend fallback. The checked snapshot must be the only bytes reachable by dispatch.

Before production work resumes, re-read those closing records at their landed hashes and run `rg -n 'InvocationGatherIndexValidationRequirement|PendingInvocationIndexValidation|GatherValidationRequirementMismatch' crates/`; zero matches or an untyped two-arm census means the compiler prerequisite did not land. Then verify the accepted coverage, artifact, and runtime records each name the exact symbols/tags/domains/errors the implementation will add. If any one is absent or still conditional, repair that owning prerequisite rather than interpreting this ticket's `in-progress` status as authority.

## User-visible outcome

An explicitly selected gather route over a host-visible U32 index input runs only after preflight validates the exact values and seals an immutable invocation snapshot; invalid, missing, stale, or mismatched evidence refuses before routing commit.

## Strict first-pass contract

- Static proof remains the first lane and requires no receipt.
- The only dynamic input is host-visible U32 storage. Preflight uses the governed `decide_gather_index` rule over every element and the exact gathered extent, then copies the validated values into immutable receipt-owned storage.
- The sealed receipt binds the exact gather occurrence, logical index type, extent, program binding, snapshot content, and invocation attempt. It is neither artifact identity nor timeless program proof, cannot be forged through public fields, and cannot be reused after any subject changes.
- The artifact and runtime carry the mandatory named validation obligation under the exact domain, version, compatibility-fence, and consumption decisions accepted by their owning prerequisite packets. A plan carrying it has no dispatch authority until the matching receipt is consumed.
- Every refusal is typed and explainable. Out-of-range is a semantic input error naming position, value, and extent—not a plan miss—and never causes clamp, wrap, reference execution, variant substitution, or backend fallback.
- Mutable zero-copy storage, device-resident or device-produced indices, validation callbacks, caller assertions, and inline kernel checks refuse as unsupported.

## Required evidence

Pin the complete obligation/receipt populations from their types. Perturb occurrence, extent, type, binding, snapshot bytes, invocation generation, missing receipt, duplicate consumption, post-validation mutation attempt, and every forbidden fallback independently with unchanged assertions. Prove the validated bytes are exactly the bytes dispatched.

## Closes when

The narrow host-visible path reaches preflight and one-way commit with no check/use gap; all excluded inputs fail closed; artifact, cache, explain, and public-boundary consequences are coherent; the exact surface is handed to its acceptance ticket; and targeted plus full gates pass.
