---
id: retain-the-selected-semantic-candidate-for-the-conformance-oracle
title: Retain the selected semantic candidate for the conformance oracle
status: done
priority: p2
dependencies: []
related: [decide-how-a-pinned-pointwise-grouping-becomes-evaluable, compose-a-declared-reduction-topology-into-a-semantic-program-evaluation, accept-the-composed-realization-evaluation-surface, retain-each-plan-alternative-s-verified-semantic-candidate, define-the-composed-realization-driver-subject-bridge, implement-the-composed-realization-evaluation-driver, design-a-versioned-semantic-source-bundle-for-artifact-only-conformance-replay]
scopes: [implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, reference, decision, public-boundary]
---
## User-visible outcome

Tom has fixed where the semantic candidate `P'` lives and how it may become public evidence: every retained physical alternative owns its exact candidate directly and mandatorily, while public exposure lands only with the complete composed conformance driver rather than as an orphan accessor.

## Current Fact audit — exact base `1f9629ad46b3717b1ef741f5cce36527e533b86d`

- **Verified — the design choice was settled and unimplemented.** [`decide-how-a-pinned-pointwise-grouping-becomes-evaluable`](decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md), anchor `One design survives, and it is design 1`, chose retention and touched no implementation.
- **Verified — the candidate is currently transient.** `SemanticCandidate.proposal` in `crates/tiler-compiler/src/pipeline.rs` owns the `RewriteProposal<SemanticProgram>` during `compile_contract_group`. `ExpectedCandidateOwner` borrows it while `verify_global_portfolio` re-derives ownership; the flattened `ProgramAlternative` retains only the owner key, semantic-bearing identity, selected plan, scheduled/KIR/artifact evidence, realization, cost, and equivalence. The candidate program is dropped after this transaction. The old empty-`rg` check was only a public-accessor census and did not prove this ownership trace.
- **Verified — direct retention is cheap and has the right cardinality.** `SemanticProgram` is `Clone` over `Arc<ProgramData>`. One candidate may own several physical alternatives, so a direct field costs one pointer/refcount per alternative while the graph storage remains shared per distinct candidate.
- **Verified — identity already binds the candidate.** `ProgramAlternativeIdentity::new`, anchor `tiler.program-alternative.v2`, length-frames all five `SemanticIdentity` components before the numerical contract and selected-plan identity. Retaining the program changes no canonical identity, artifact bytes, cache key, or schema.
- **False — retention alone makes the split-test repair expressible.** `ReferenceEvaluator::evaluate` returns only `program.outputs()`. `the_assembled_split_program_matches_the_partitioned_sum_oracle` needs the prologue's internal `ValueId`; that requires the accepted but unimplemented crate-private pin/observe primitive and composed driver. Retention and the provenance repair are separate deliveries.
- **Stale — the driver and pin primitive are parked for Tom.** [`accept-the-composed-realization-evaluation-surface`](accept-the-composed-realization-evaluation-surface.md) is `done`: Tom accepted the driver as the sole public composition entry and kept the `ValueId` primitive crate-internal. They remain unimplemented.
- **Verified — the current split fixture is only conditionally sound.** `semantic_case_with_axis` spells `scale * x + bias`; its multiply then add is not a same-family reassociation site, so the fixture does not distinguish `P` from a rewritten `P'`. Its existing use of the first kernel's output must move only with the composed driver, under a fixture that actually spends both freedoms.

## Decision — accepted 2026-08-12

**Decided by Tom on 2026-08-12 in the live decision round, relayed by the coordinator from this ticket:**

1. Every internal `ProgramAlternative` retains its exact verified `SemanticProgram` directly in a mandatory private field. No `Option`, default, stable-label lookup, owner-key lookup, rewrite replay, caller baseline, schedule-expression reconstruction, artifact-output substitution, or other fallback is admitted.
2. Construction and final portfolio verification re-check the retained program's complete semantic identity against the alternative's existing owner binding and `ProgramAlternativeIdentity`. A swapped or mismatched candidate is invalid compiler output. Equality must use verified semantic identity, never Arc pointer identity.
3. The retention change exposes no standalone public semantic-program accessor. Public evidence lands atomically with the complete driver and its named consumer, so the tree does not carry an incomplete public abstraction.
4. The composed driver lives at the top evidence layer, `tiler-conformance`, which already depends normally on both `tiler-compiler` and `tiler-reference` and creates no reverse dependency. It accepts the complete `PlanAlternative` plus declared inputs, never a caller-composed free pair of semantic program and witnesses.
5. The driver's exact compiler-minted stage/materialization bridge is a consequential public surface and gets its own source-first decision. The reference `ValueId` pin/observe primitive stays crate-private. The bridge and driver land atomically after retention.
6. Retention is compiler/session evidence only. Artifact-only or cross-process replay remains unsupported until a separately versioned semantic source-bundle decision fires; artifact identity proves which semantic subject was built but cannot rehydrate the graph.

## Ranked elimination

1. **Direct mandatory private retention, followed by an atomic driver and minimal bridge — accepted.** Structural ownership, fail-closed binding, one Arc bump per alternative, and no orphan public route.
2. **Direct retention plus an immediate public accessor — declined.** Correct as evidence but incomplete as a split-plan oracle and unnecessary before its accepted consumer exists.
3. **Compilation arena plus checked handles — declined.** It saves only Arc handles while adding an index, lookup, and drift relation.
4. **Internal or external side table — declined.** It creates a second ownership authority; selected-only retention also leaves other public alternatives unevaluable. `stable_id` is a 64-bit presentation label and cannot key correctness.
5. **Artifact/source-bundle embedding — deferred behind a real artifact-only replay trigger.** It is the only cross-process answer, but it is a different schema, identity, storage, and product decision.
6. **Rewrite replay, baseline, schedule/KIR/artifact reconstruction, or device-output provenance — rejected.** These are wrong-subject, drift-prone, or vacuous fallbacks.

## Delivery graph

- [`retain-each-plan-alternative-s-verified-semantic-candidate`](retain-each-plan-alternative-s-verified-semantic-candidate.md) implements the mandatory private field and owner/identity perturbations.
- [`define-the-composed-realization-driver-subject-bridge`](define-the-composed-realization-driver-subject-bridge.md) freezes the minimal compiler-minted public bridge inside the accepted driver/home constraints.
- [`implement-the-composed-realization-evaluation-driver`](implement-the-composed-realization-evaluation-driver.md) depends on both, implements the accepted driver and crate-private reference primitive, and owns the split-test provenance repair.
- [`design-a-versioned-semantic-source-bundle-for-artifact-only-conformance-replay`](design-a-versioned-semantic-source-bundle-for-artifact-only-conformance-replay.md) is deferred until an artifact-only consumer fires its trigger.

## Non-goals

Implementing any child; serializing `SemanticProgram`; exposing the crate-private `ValueId` primitive; re-deciding design 1; moving `tiler-reference` to a normal `tiler-compiler` dependency.

## Outcome

The siting decision is complete and the previously conflated implementation work is represented by explicit dependency edges. The decision itself changes no source, identity, schema, result, or runtime behavior.
