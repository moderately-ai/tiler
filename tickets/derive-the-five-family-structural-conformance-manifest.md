---
id: derive-the-five-family-structural-conformance-manifest
title: Derive the five-family structural conformance manifest
status: todo
priority: p2
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner, define-the-conformance-obligation-and-evidence-requirement-algebra]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, spike, conformance-progress, reference]
---
# Derive the five-family structural conformance manifest

## Goal

A bounded design and exact case manifest for reindex, broadcast, concatenate, slice, and gather that shares declarations without sharing semantic and reference authorities or recomputing literal expected payloads from the implementation under test.

## Work

1. Re-read every existing semantic and reference test proposed for sharing or retirement, plus the five families' operation definitions, schemas, inference, canonical encoding, evaluator, and typed refusal paths.
2. Re-audit the proposed eleven exact reference signatures: one each for reindex, broadcast, slice, and gather plus concatenate arities two through eight.
3. Define stable family/case/obligation identities and a lazy counted expansion. Keep semantic inference/refusal and reference payload evaluation as independent drivers over shared declarations.
4. Retain literal expected payloads and refusal codes. Never generate the expected permutation from the mapping implementation being tested.
5. Add candidate exceptional-bit transport declarations for reindex and broadcast covering NaN payloads, signed zero, subnormals, and infinities without claiming arithmetic behavior.
6. Specify one real subject perturbation per family and its expected exact failure.
7. Produce the first duplicate-equivalence rows, distinguishing shared declarations from distinct proof obligations.
8. Compare a shared manifest, local duplicated fixtures, a universal generator, and deferral; eliminate the Cartesian and shared-oracle options.

## Non-goals

- Do not edit production semantic/reference code or migrate/delete tests.
- Do not broaden operation support or create a second oracle.
- Do not claim the five families cover every semantic operation.

## Stop conditions

Stop and split a family if shared construction changes its subject, oracle independence, refusal population, or public boundary.

## Acceptance

- Exact expanded populations and all spellings used by the census are recorded.
- Each old case maps to a stable proposed identity with separate semantic and reference obligations.
- Independent literals, typed refusals, exceptional-bit transport, and perturbations survive the design.
- Any proposed retirement satisfies “shadow, prove, then prune” or remains explicitly retained.

## Refs

- [`shape-the-conformance-corpus-for-target-multiplication`](shape-the-conformance-corpus-for-target-multiplication.md)
- [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md)
- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)
