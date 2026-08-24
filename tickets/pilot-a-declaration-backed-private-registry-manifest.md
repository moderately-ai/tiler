---
id: pilot-a-declaration-backed-private-registry-manifest
title: Pilot a declaration-backed private registry manifest
status: todo
priority: p1
dependencies: [specify-the-canonical-owner-conformance-manifest-protocol]
related: [spike-a-red-yellow-first-full-conformance-suite, derive-the-five-family-structural-conformance-manifest]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, spike, conformance-progress, reference, registries]
---
# Pilot a declaration-backed private registry manifest

## Goal

Prove on the standard reference registry that one owner-private declaration source can drive construction, canonical registry identity, and an exact manifest without a public iterator or second hand-maintained census.

## Work

1. Read reference registration, freezing, identity, resolution, evaluation, refusal, and all standard capability/validator tests in full.
2. Build a bounded spike that derives the 28 capability and seven validator rows from the same declarations construction consumes.
3. Emit through an owner-local reporter to an explicit file using the proposed protocol; do not parse stdout.
4. Perturb the real registration subject with an added, removed, duplicated, reordered, revised, and undeclared row and retain the exact failures.
5. Prove the manifest root agrees with the frozen registry identity without copying executable functions or semantic authority.
6. Measure clean/incremental compile time, reporter wall time, manifest size, and peak validation allocation.

## Non-goals

- Do not publish a reference-registry iterator.
- Do not migrate structural conformance cases or define their evidence requirements.
- Do not generalize from the standard registry to arbitrary caller-built registries.

## Stop conditions

Stop if construction and declaration cannot share one source, if callable evaluator state must cross the boundary, if an owner row can land without failing a perturbation, or if the protocol schema must change.

## Acceptance

- The exact frozen population and manifest population are mechanically identical.
- Every subject perturbation fails at the owner or audit boundary with exact text.
- No public API, consumer build step, or second oracle is introduced.
- Results state what remains different for semantic, lowering, and arbitrary custom registries.

## Refs

- [Owner-private conformance inventory boundary](../docs/research/verification/owner-private-conformance-inventory-boundary.md)
