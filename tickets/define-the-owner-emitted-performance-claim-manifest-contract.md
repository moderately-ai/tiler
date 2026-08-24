---
id: define-the-owner-emitted-performance-claim-manifest-contract
title: Define the owner-emitted performance-claim manifest contract
status: todo
priority: p1
dependencies: [define-retained-performance-claim-authority-and-identity]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, conformance-progress, verification, performance]
---

# Define the owner-emitted performance-claim manifest contract

## Goal

A decision-ready private/test-only manifest contract by which each real performance owner emits stable claim identities and the conformance universe joins them without making documentation or the conformance crate a second authority.

## Authority

The accepted correctness, cost-model, documentation-metadata, and P+K+M+T contracts plus [the retained performance-claim authority research](../docs/research/verification/retained-performance-claim-authority-and-identity.md) outrank this ticket. Re-audit every source and identity at the working base.

## Work

1. Read each affected owner boundary, its claim construction and consumption sites, the canonical conformance receipt join, goal-profile schema research, and the complete retained records chosen as fixtures.
2. Define the minimal claim descriptor and canonical projection for stable owner/local key, revision, claim form, metric/unit, subject and workload selectors, comparator/baseline policy, statistic, acceptance predicate, context equivalence, evidence requirement, freshness, predecessor, and tombstone.
3. Preserve separate claim, subject, workload, environment, procedure, observation, evidence-snapshot, baseline, and normative-guarantee identities.
4. Compare owner-private typed emission, test-only sidecars, document-adjacent manifests, one central registry, a bounded source census, and deferral. Eliminate any route that lets evidence producers mint thresholds or lets owner additions bypass the join.
5. Specify resource bounds, canonical ordering, duplicate/collision refusal, unknown-owner representation, and the subject perturbation for every proposed complete enumeration.
6. Decide how the private `CostRow` vocabulary participates exactly once and how owner-private manifests cross the test-only boundary without creating a public optimizer API.
7. Return a verbatim-landable schema/model or a narrower decision ticket if a consequential boundary survives.

## Non-goals

- Do not migrate records, choose first-profile claims, accept thresholds, run benchmarks, implement external P+K+M+T mechanisms, or expose a public API.
- Do not parse prose or free-form topics as claim authority.
- Do not put hard feasibility, correctness invariants, and performance preferences in one untyped metric bag.

## Stop conditions

Stop for a decision packet if a manifest requires a consequential public boundary, if two owners survive for one claim or threshold, if a canonical field would copy another authority instead of joining its identity, or if a proposed complete population stays green when its owner adds a subject.

## Acceptance

- Every claim field has one owner, identity consequence, bound, and refusal.
- Owner additions, duplicate IDs, unknown revisions, missing dispositions, and stale contexts fail loudly in the executable model.
- The design covers optimizer and non-optimizer families without a universal Cartesian schema or a central conformance authority.
- Immediate migration work has an exact input population and cannot silently treat unclassified records as empty.
