---
id: define-retained-performance-claim-authority-and-identity
title: Define retained performance-claim authority and identity
status: in-progress
priority: p1
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner]
related: [spike-a-red-yellow-first-full-conformance-suite, derive-the-optimizer-and-planner-capability-obligation-manifest]
scopes: []
shared_scopes: [research/verification, project/tickets, contracts/navigation]
paths: []
tags: [research, design, conformance-progress, verification, performance]
claimed_from: todo
assignee: codex
lease_expires_at: 1787610669
---
# Define retained performance-claim authority and identity

## Goal

Define a fail-closed claim-level owner, identity, revision, and enumeration contract for retained performance claims, without confusing measurement-bearing document records with the claims inside them.

## Authority

Accepted performance guidance, document metadata, exact retained records, accepted cost-model contracts, and measurement harnesses outrank this ticket. Re-audit all counts and Facts at the working base.

## Work

- Derive the materially distinct performance-claim families and their owners from retained records and accepted cost contracts.
- Distinguish record identity, claim identity, subject identity, environment/profile identity, measurement evidence, and normative guarantee.
- Compare manual claim lists, structured record-owned manifests, bounded source censuses, and deferral; eliminate fail-open options.
- Define revision and freshness rules plus a subject perturbation that adds one undisposed performance claim.
- Map optimizer measured-cost claims to the optimizer manifest ticket and keep non-optimizer measurements visible.

## Non-goals

No benchmark run, performance guarantee, goal-profile choice, cost-model tuning, document-schema implementation, or public API.

## Stop conditions

Stop for a decision packet if no singular claim authority survives, if record metadata and normative cost authority conflict, or if identity requires a consequential schema/public boundary.

## Acceptance

- Measurement records and performance claims have separate exact or explicitly unknown populations.
- Each proposed complete enumeration fails on a subject perturbation.
- Identity, revision, freshness, profile/environment, and unsupported-population rules are explicit.
- The result covers retained claims outside the optimizer as well as the optimizer-manifest dependency.

## Exact-base Fact audit

**Measurement — 2026-08-24.** Re-audited on the clean claimed branch at exact base `d1ec30a8ed5f610884a307b0c27d24300f1cc87c`; `main` and `origin/main` were `0 0` before the claim, and `tkt reconcile --format json` reported no findings.

| premise | verdict | exact-base evidence and consequence |
| --- | --- | --- |
| Governed measurement records enumerate retained performance claims. | **False.** | The corpus has 114 research records and 38 declaring `bounded-measurement`, but that population includes non-performance subjects. Record identity and evidence class cannot be used as claim identity. |
| The free-form `performance` topic closes the gap. | **False.** | Exactly three research records carry it. It omits [Direct embedded-artifact costs across Rust crates](../docs/research/embedding/embedded-artifact-costs.md), while [Model-level correctness and performance qualification](../docs/research/program-planning/model-level-qualification.md) carries it and explicitly contains no Tiler execution measurement. |
| The optimizer measured-cost population is entirely unknown. | **False, narrowly.** | Private `CostRow` has one exact current variant and an exhaustive `key` mapping. Adding `AuditProbe` made `cargo check -p tiler-compiler` fail with `E0004`; the perturbation was then removed. Compiler-wide performance claims remain unknown. |
| A measurement record may own its claim, threshold, and accepted baseline. | **False.** | The accepted cost and correctness contracts separate observations, feasibility, estimates, and qualification, while ADR 0114 separates evidence-baseline authority from execution. |
| A consequential public boundary is required to state the destination. | **False.** | Owner-emitted private/test-only manifests plus structured receipts are the nondominated architecture. Their exact schema and owner-private crossing remain bounded follow-up research; this ticket opens no boundary. |

## Outcome

- Added [Retained performance-claim authority and identity](../docs/research/verification/retained-performance-claim-authority-and-identity.md), separating exact record populations from the explicitly unknown claim population and defining claim forms, family owners, identities, revisions, freshness, baseline lineage, verdicts, and unsupported states.
- Selected distributed owner-emitted claim manifests joined to structured measurement receipts. Prose inference, free-form metadata, one manual global list, and record-owned authority were eliminated as fail-open or authority-conflating.
- Mapped `cost.saturated-parallel-fold-steps` exactly once to the optimizer/planner manifest while keeping its target/environment/procedure receipt in the performance-evidence lane. Cache, artifact, build, compiler-host, device, and end-to-end families remain visible.
- Added a reproducible [subject-perturbation fixture](../spikes/verification/retained-performance-claim-authority/README.md): two claims/two dispositions pass; adding `perf.audit.undisposed@1` fails with that exact missing-disposition diagnostic.
- Filed [`define-the-owner-emitted-performance-claim-manifest-contract`](define-the-owner-emitted-performance-claim-manifest-contract.md) and [`migrate-retained-performance-evidence-to-owner-claim-identities`](migrate-retained-performance-evidence-to-owner-claim-identities.md). The first goal profile now depends on migration, so closing this research cannot erase its explicit unknown.

## Reproduction

```sh
git grep -l '^kind: "research"' d1ec30a8ed5f610884a307b0c27d24300f1cc87c -- 'docs/research/*.md' 'docs/research/**/*.md' | wc -l
git grep -l '^evidence_classes: .*bounded-measurement' d1ec30a8ed5f610884a307b0c27d24300f1cc87c -- 'docs/research/*.md' 'docs/research/**/*.md' | wc -l
git grep -l '^topics: .*"performance"' d1ec30a8ed5f610884a307b0c27d24300f1cc87c -- 'docs/research/*.md' 'docs/research/**/*.md' | wc -l
spikes/verification/retained-performance-claim-authority/check_fixture.sh spikes/verification/retained-performance-claim-authority/owner-claims.tsv
spikes/verification/retained-performance-claim-authority/check_fixture.sh spikes/verification/retained-performance-claim-authority/owner-claims-perturbed.tsv
```

The reviewed-base counts are `114`, `38`, and `3`; this report itself moves the tip record/topic counts, so the commands pin the base. The baseline fixture exits zero with `2 claims; 2 dispositions; complete`; the subject perturbation exits one with `undisposed performance claim: perf.audit.undisposed@1`.
