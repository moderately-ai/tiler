---
id: inventory-the-closed-world-conformance-claim-universe-by-owner
title: Inventory the closed-world conformance claim universe by owner
status: review
priority: p1
dependencies: []
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification, contracts/navigation]
paths: []
tags: [research, design, conformance-progress, verification]
claimed_from: todo
assignee: conformance-universe-sol
lease_expires_at: 1787607003
---
# Inventory the closed-world conformance claim universe by owner

## Goal

A retained, source-derived inventory of every declared capability and correctness-bearing invariant that can enter Tiler's conformance denominator, including internal optimizer, planner, schedule, verifier, KIR, artifact, runtime, numerical, target, and performance claims that no end-user API exposes.

The result separates the **system universe** from any goal profile. It identifies the authoritative owner and stable identity for every enumerable family, and marks every family that cannot yet be enumerated without pretending absence means completeness.

## Facts to re-audit first

- `FrozenSemanticRegistry::operation_definitions` exposes the exact contents of one frozen semantic registry; whether that instance closes the system semantic-operation denominator must be proved rather than assumed.
- `FrozenReferenceRegistry` retains exact operation/signature capabilities but exposes no equivalent public iterator.
- `RuleRegistry::rules` is exact for one compiler-private registry instance; production constructs several registries, so no compiler-wide rewrite vocabulary is assumed.
- `tiler-conformance` owns cross-layer executed evidence and explicitly does not own layer-local tests, semantic meaning, or performance measurement.
- Counts and paths in the root spike are stale until reproduced at this ticket's exact base.

## Work

1. Read the complete owner, construction, validation, consumption, refusal, identity, and test paths for each candidate family.
2. Inventory at least: semantic operations and types; algebraic declarations; reference operation/signature capabilities; lowering capabilities; rewrite rules; physical providers and strategies; feasibility predicates; search budgets; explain dispositions; schedule/KIR/program vocabularies and verifier invariants; artifact/ABI/proof/publication guarantees; backend compilation stages; runtime route/fallback/completion claims; target/numerical declarations; cache identity/publication claims; and retained performance claims.
3. For each family record `{owner, authority, stable identity, revision rule, construction site, consumption site, refusal path, enumeration mechanism, exact population or explicit unknown, profile relevance}`.
4. Distinguish declared feature claims from implementation details and tests. Tests are evidence, not the feature universe.
5. Classify each enumeration as typed/exhaustive, registry-derived, contract-derived, hand-maintained, or currently unenumerable. State what source perturbation would make each census fail.
6. Compare the status quo, manual manifest, owner-derived typed manifest, bounded source census, and deferral. Eliminate any option that can silently omit new features.
7. Retain the inventory and reproducing commands under `spikes/verification/`, with a ticket for every missing identity or enumeration owner rather than an invented row.

## Non-goals

- Do not choose the goal profile or mark support.
- Do not expose a new public API or move owner vocabulary between crates.
- Do not use source test counts, ticket counts, or every function as the feature denominator.
- Do not infer completeness from a grep that found nothing.

## Stop conditions

Stop and split a decision ticket when a claimed feature has no singular owner, when two identities compete for the same subject, or when enumeration would require a consequential public boundary.

## Acceptance

- Every named layer has an owner matrix and an explicit exact or unknown population.
- Every “complete” census states why its vocabulary is complete and demonstrates a subject perturbation that adds one undisposed item.
- The report names all unenumerable families and the bounded work required to make each fail loud.
- The inventory identifies the normative inputs and unresolved authority/schema decisions required for a future stable system-universe identity, without deriving any identity from a goal profile; a raw artifact checksum is labelled provenance only.
- `tkt lint`, `make citations`, and scope guard pass.

## Refs

- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)
- [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md)
- [`docs/correctness-and-testing.md`](../docs/correctness-and-testing.md)


## Outcome

**Complete named-family owner audit at `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a`; system universe remains open.** The corrected report and 36-row TSV distinguish ten demonstrated fail-loud typed vocabularies from bounded owner snapshots and positive `unknown` rows. The system universe is independent of a goal profile; tests and cross-layer receipts are evidence, not feature rows.

### Fact audit

- **Imprecise and repaired:** `FrozenSemanticRegistry::operation_definitions` exactly iterates one frozen operation map, but no owner manifest proves that the instance closes the system denominator. The 19-row value is a bounded snapshot, not a complete census.
- **Verified narrowly:** `FrozenReferenceRegistry` retains exact ordered operation/signature and validator maps and exposes no public capability iterator. Its 28/7 standard-owner values are bounded snapshots because no fixed manifest rejects additions.
- **False and repaired:** `RuleRegistry::rules` is exact only for one instantiated registry, not the whole compiler. Production constructs a pipeline baseline plus separate CSE and conditional algebraic registries. Four production identities are source-known; compiler-wide rewrite population is unknown.
- **Verified:** `tiler-conformance` owns executed cross-layer evidence, not semantic authority, layer-local tests, or performance measurements.
- **Verified at this base:** the root evidence envelopes reproduce as 484 semantic tests, 182 reference source tests, 152 reference integration tests, 115 compiler pipeline tests, 94 compiler external tests, 265 schedule tests, 137 kernel tests, 954 backend-side tests, ten Metal conditional early returns, and five Metal-AOT conditional early returns.

### Delivered records

- `docs/research/verification/conformance-claim-universe-by-owner.md` records the corrected source audit, decision-packet gate, exact/bounded/unknown split, unsupported populations, and follow-up graph.
- `spikes/verification/conformance-claim-universe-by-owner/README.md` records the ten executed source-envelope commands, searched-vocabulary limits, and ten independent subject mutations with their actual `E0080`, `E0308`, or `E0004` diagnostics. It also records the green `BudgetRefusal` perturbation that forced a downgrade.
- `spikes/verification/conformance-claim-universe-by-owner/inventory.tsv` carries 36 split-owner rows with authority, stable identity, revision rule, construction, consumption, refusal, enumeration, population, profile relevance, completeness, perturbation, and follow-up. Its SHA-256 is a raw-file integrity checksum only, not a universe identity.

### Decision-packet verdict

Status quo and a manual authority manifest are eliminated as fail-open. Owner-derived typed/registry manifests are the destination; bounded source censuses are labelled bridges only; explicit unknown rows preserve unresolved families. Blanket deferral is dominated for the ten demonstrated vocabularies and retained for unresolved lanes. No public boundary, production API, goal profile, canonical projection, or normative identity schema was chosen.

### Follow-up graph

Existing optimizer/planner, structural, stage, private-boundary, authority, evidence-algebra, receipt, explain, and goal-profile tickets remain in force. Three missing lanes were filed and related to the root spike: `derive-artifact-proof-and-publication-conformance-obligations`, `derive-runtime-route-completion-and-cache-obligations`, and `define-retained-performance-claim-authority-and-identity`. The two obligation manifests also depend on the evidence algebra; all three are now exact prerequisites of `assemble-the-first-versioned-conformance-goal-profile`.

### Review correction

The first delivery's command/count pairs, plural-owner rows, unexecuted perturbation designs, and checksum-as-identity language are superseded by this outcome. Final commit and gate results are recorded in the latest delivery comment. Research-only retained changes touch no production source; temporary perturbations were reversed after each diagnostic.
