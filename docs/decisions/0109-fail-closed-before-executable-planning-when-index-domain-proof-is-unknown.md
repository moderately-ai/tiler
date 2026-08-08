---
schema: "tiler-doc/v1"
id: "ADR-0109"
kind: "decision"
title: "Fail closed before executable planning when index-domain proof is Unknown"
topics: ["extensions", "optimizer", "proof", "governance", "identity"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "complete"
applies_to: ["tiler.contract.operation-extensions", "tiler.contract.optimizer", "tiler.research.runtime.execution-contract"]
evidence: []
depends_on: ["ADR-0078"]
ticket: "reconcile-the-accepted-proof-budget-stop-rule-with-executable-refinement"
---

# 0109: Fail closed before executable planning when index-domain proof is Unknown

**Status:** accepted by Tom on 2026-08-08 in the current Codex session, relayed to the author by the coordinator from Tom's message, “yes you may make the correct decision and accpet the change”. Acceptance supersedes only [ADR 0078](0078-name-the-intended-public-extension-seams.md)'s item-3 requirement that an exhausted index-domain proof return `Ok` and let “the plan stand”. ADR 0078's governing seam rule and every other item remain accepted. The supersession is recorded in prose on both records because a whole-record `supersedes` edge would overstate it.

## Context

ADR 0078 accepted an exhausted analysis budget as an `Unknown` gap, neither disproof nor admission, and coupled that classification to the historical executable behavior: `lowering::refine` returned `Ok(OccurrenceEvidence::BudgetStopped(_))` and planning continued. The current refinement architecture has a stronger executable boundary. A pending refinement receipt carries retained logical index-domain obligations but no executable-coverage spelling. Only an all-proved completion mints `IndexRefinementReceipt` and its `IndexRefinementExecutableCoverageIdentity`; kernel programs, artifact plans, and runtime routing consume that proved coverage rather than a pending analysis state.

**Fact — the current assessment is atomic and complete.** `ResolvedIndexRealization::complete` owns one `IndexDomainProofLedger` for the whole realization. It retains every produced assessment, fills each obligation left unassessed after an exhausted budget with `Unknown(ResourceLimit { resource, required, limit })`, chooses `Disproved` before `Unknown`, and mints a receipt only when every claim is `Proved`. A resource stop can therefore coexist with an earlier disproof without hiding or overriding it.

**Fact — the compiler refuses before executable planning.** `discharge_pending_index_refinement` projects IR's refusal through `LoweringError::SemanticDischarge`. `lowering_failure` classifies `Disproved` as invalid compiler output and `Unknown` as unsupported capability. `enumerate_complete_plans` resolves and discharges lowering before its sole `enumerate_covers` call, so an unknown residual reaches no cover, physical frontier, kernel program, artifact plan, cache identity, or runtime fallback.

**Fact — restoring the historical `Ok` result is no longer local.** The historical code predated proof-bound executable coverage. Restoring “the plan stands” now would require either minting executable coverage from an unproved obligation, which would turn `Unknown` into admission, or adding a separate public non-executable result and defining how it participates in compilation identity and explanation. Carrying it through artifacts or runtime fallback would additionally require new artifact, cache, ABI, and runtime contracts. None is an evidence repair to the old spelling.

## Decision

### 1. A proof-resource limit remains `Unknown`

An exhausted index-domain proof says only that the governed evaluator did not finish within its stated resource bound. It is neither evidence that the emitted coordinate is out of bounds nor permission to execute it. The assessment retains the exact resource, cumulative required amount, and original whole-call limit.

### 2. `Unknown` fails closed before executable planning and coverage

If any retained index-domain obligation is `Unknown`, compilation refuses the occurrence as unsupported capability before cover enumeration. No executable coverage identity, plan alternative, kernel program, artifact plan, cache subject, or runtime route is created from that pending state. This item supersedes ADR 0078's historical `Ok` and “the plan stands” requirement, and only that requirement.

### 3. An assessed disproof takes precedence and every assessment remains explainable

Completion retains all assessments it produced. If any is `Disproved`, the overall refusal is `Disproved` even when another obligation is `Unknown(ResourceLimit)`; the resource limit does not launder invalid compiler output into unsupported capability. If none is disproved but any is unknown, the overall refusal is `Unknown`. Explain records every retained obligation and claim, including earlier completed assessments and later resource-limited ones.

### 4. Acceptance changes authority, not executable behavior or identity

The current IR and compiler behavior already implements items 1 through 3. This decision does not add a public non-executable compilation result, mint coverage from pending refinement, widen artifact or cache identity, add runtime checking or fallback, change numerical behavior, or authorize any of those changes. The existing proof, coverage, program, and artifact identity domains remain byte-for-byte unchanged.

## Consequences

- The operation-extension contract no longer states that a budget stop must be the sole diagnostic. The safety property is instead exact: ResourceLimit is an `Unknown` assessment, every assessment is retained, and `Disproved` has overall precedence.
- The optimizer and runtime-execution contracts' current refusal-before-cover wording becomes the accepted behavior rather than implementation drift.
- A provider cannot obtain executable work merely by making verification expensive. Raising a budget may turn an `Unknown` into `Proved` or `Disproved`; it cannot change what either result means.
- The correction pending on ADR 0078 can repair its current-source evidence against this accepted boundary without pretending that current code still implements the retired historical `Ok` path.

## Alternatives considered

**Restore the historical `Ok` result and let the plan stand.** Rejected because the current executable result is proof-bound. Reusing it would admit an unproved access; replacing it with an analysis-only result is a consequential public boundary with new consumers and identity rules, not a restoration local to lowering. It also spends maintenance and runtime design effort while preserving less correctness.

**Carry the unknown plan into artifacts and validate or fall back at runtime.** Rejected as a different architecture. It needs a runtime predicate language, artifact and cache identity, dispatch-time validation ownership, failure timing, and a route that cannot mistake a semantic failure for permission to try another executable meaning. No present consumer requires it.

**Treat ResourceLimit as disproof.** Rejected because a budget says nothing about predicate truth. It would misclassify missing support as invalid compiler output and would let evaluation order determine the semantic claim.

**Drop later assessments once a disproof is found.** Rejected because the completion call owns one atomic, explainable ledger. Retaining the full assessment population shows which work completed, which resource stopped, and why the overall refusal chose Disproved.

## Traceability

[The operation extension contract](../operation-extensions.md) owns the provider-facing Unknown and diagnostic rules. [The optimizer model](../compiler/optimizer.md) owns semantic discharge and the before-cover executable boundary. [The runtime execution contract](../research/runtime/runtime-execution-contract.md) owns the absence of a runtime plan or fallback. [`reconcile-the-accepted-proof-budget-stop-rule-with-executable-refinement`](../../tickets/reconcile-the-accepted-proof-budget-stop-rule-with-executable-refinement.md) records the historical/current derivation and the implementation evidence. [`accept-adr-0109-fail-closed-on-unknown-index-domain-proof`](../../tickets/accept-adr-0109-fail-closed-on-unknown-index-domain-proof.md) records Tom's acceptance act.

## Implementation boundary

Acceptance decides the current fail-closed model and adds no public or runtime surface. `PendingIndexRefinementReceipt` still exposes no executable coverage identity; successful `ResolvedIndexRealization::complete` remains the only minting path; compiler semantic discharge still runs before cover enumeration; and `UnsupportedCapability` remains the end-to-end result for an over-budget Unknown. The tests landed with this record repair the ledger evidence without changing those production paths.
