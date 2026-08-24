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
lease_expires_at: 1787601327
---
# Inventory the closed-world conformance claim universe by owner

## Goal

A retained, source-derived inventory of every declared capability and correctness-bearing invariant that can enter Tiler's conformance denominator, including internal optimizer, planner, schedule, verifier, KIR, artifact, runtime, numerical, target, and performance claims that no end-user API exposes.

The result separates the **system universe** from any goal profile. It identifies the authoritative owner and stable identity for every enumerable family, and marks every family that cannot yet be enumerated without pretending absence means completeness.

## Facts to re-audit first

- `FrozenSemanticRegistry::operation_definitions` already exposes a canonical exact semantic-operation inventory.
- `FrozenReferenceRegistry` retains exact operation/signature capabilities but exposes no equivalent public iterator.
- `RuleRegistry::rules` is a canonical compiler-private, test-facing rewrite inventory; production compilation exposes selected results rather than this complete vocabulary.
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
- The inventory yields a stable candidate system-universe identity without deriving it from a goal profile.
- `tkt lint`, `make citations`, and scope guard pass.

## Refs

- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)
- [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md)
- [`docs/correctness-and-testing.md`](../docs/correctness-and-testing.md)


## Outcome

**Complete source inventory at `37a8107e9999b29b51a5c7458b5fd0bc0a408e3a`.** The retained report and 25-row TSV distinguish exact owner populations from positive `unknown` rows. The candidate system universe is independent of a goal profile; tests and cross-layer receipts are evidence, not feature rows.

### Fact audit

- **Verified:** `FrozenSemanticRegistry::operation_definitions` is the exact canonical iterator over the frozen operation map.
- **Verified:** `FrozenReferenceRegistry` retains exact ordered operation/signature and validator maps and exposes no public capability iterator.
- **Imprecise and repaired:** `RuleRegistry::rules` is exact only for one instantiated registry, not the whole compiler. Production constructs separate CSE and conditional algebraic registries, so compiler-wide rewrite population is unknown.
- **Verified:** `tiler-conformance` owns executed cross-layer evidence, not semantic authority, layer-local tests, or performance measurements.
- **Verified at this base:** the root evidence envelopes reproduce as 484 semantic tests, 182 reference source tests, 152 reference integration tests, 115 compiler pipeline tests, 94 compiler external tests, 265 schedule tests, 137 kernel tests, 954 backend-side tests, ten Metal conditional early returns, and five Metal-AOT conditional early returns.

### Delivered records

- `docs/research/verification/conformance-claim-universe-by-owner.md` records the source audit, decision-packet gate, exact/unknown split, identity derivation, unsupported populations, and follow-up graph.
- `spikes/verification/conformance-claim-universe-by-owner/README.md` records reproducing commands, searched-vocabulary justification, and subject perturbation designs.
- `spikes/verification/conformance-claim-universe-by-owner/inventory.tsv` carries 25 owner rows with authority, stable identity, revision rule, construction, consumption, refusal, enumeration, population, profile relevance, completeness, perturbation, and follow-up. Its retained SHA-256 is `192aafe506547f4ec9d730cf9ff2c5b1a869dfa23cfa038e29d9759c10f4487a`.

### Decision-packet verdict

Status quo and a manual authority manifest are eliminated as fail-open. Owner-derived typed/registry manifests are the destination; bounded source censuses are labelled bridges only; explicit unknown rows preserve unresolved families. Blanket deferral is dominated for exact rows and retained only for unresolved lanes. No public boundary, production API, goal profile, or normative identity schema was chosen.

### Follow-up graph

Existing optimizer/planner, structural, stage, private-boundary, authority, evidence-algebra, receipt, explain, and goal-profile tickets remain in force. Three missing lanes were filed and related to the root spike: `derive-artifact-proof-and-publication-conformance-obligations`, `derive-runtime-route-completion-and-cache-obligations`, and `define-retained-performance-claim-authority-and-identity`. The two obligation manifests also depend on the evidence algebra; all three are now exact prerequisites of `assemble-the-first-versioned-conformance-goal-profile`.

### Checks

- `tkt lint` — passed.
- `make citations` — passed; every pinned citation and local markdown link resolves.
- `git diff --check` — passed.
- `tkt guard tkt/inventory-the-closed-world-conformance-claim-universe-by-owner --base 37a8107e9999b29b51a5c7458b5fd0bc0a408e3a --format json` — passed after staging, with declared shared scopes `contracts/navigation`, `project/tickets`, and `research/verification`, no collisions, and no under-declaration.

Research-only change: no code gate was required or run.
