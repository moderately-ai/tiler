---
id: decide-the-zero-dependency-metal-aot-conformance-declaration-route
title: Decide the zero-dependency Metal-AOT conformance declaration route
status: todo
priority: p1
dependencies: [specify-the-canonical-owner-conformance-manifest-protocol]
related: [classify-machine-compilation-and-execution-outcomes-by-stage, spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, decision, conformance-progress, metal-aot, public-api, dependencies]
---
# Decide the zero-dependency Metal-AOT conformance declaration route

## Goal

A decision-ready route by which `tiler-metal-aot` owner-mints its conformance subjects and obligations without silently violating ADR 0077's empty workspace/third-party dependency closure or letting a conformance adapter invent authority.

## Work

1. Read ADR 0077, the complete AOT compile-stage/input/diagnostic construction and consumption paths, `CompileStage::ALL`, the machine-outcome ticket, workspace dependency checks, and the proposed common protocol.
2. Re-audit exactly what the public exhaustive stage vocabulary owns and what conformance subject IDs, obligation IDs, revisions, applicability, and evidence predicates are still absent.
3. Compare: a zero-dependency owner-native immutable declaration projection with a proved bijective top adapter; a private owner-local reporter with an independently encoded transport; amending ADR 0077 to admit the dependency-free protocol crate; leaving the family explicitly unknown; and deferral.
4. Eliminate any route that duplicates canonical encoding, lets conformance mint AOT meaning, introduces hidden build/discovery steps, exposes invocation mutation, or calls an enum an obligation manifest.
5. Measure dependency/build cost and state public namespace, identity, schema, compatibility, and future non-Metal toolchain consequences for each survivor.
6. Use independent derivation and perturb stage addition, obligation addition, adapter omission, ID mismatch, reordered rows, and unknown protocol versions.

## Non-goals

- Do not add a dependency, public accessor/type, serializer, or reporter.
- Do not weaken ADR 0077 by interpreting empty closure as production-only.
- Do not define compilation/execution outcome semantics owned by the separate machine-outcome ticket.

## Stop conditions

Stop for Tom on an ADR 0077 amendment or public AOT namespace. Stop if no route can preserve owner-minted meaning and canonical validation simultaneously; retain an explicit unknown rather than adapting the enum by hand.

## Acceptance

- The packet distinguishes exhaustive stage vocabulary from conformance declarations.
- Every survivor preserves one authority for IDs, revisions, applicability, and evidence predicates.
- The exact dependency and public-surface cost is explicit.
- A single Tom question presents only the nondominated frontier.

## Refs

- [Owner-private conformance inventory boundary](../docs/research/verification/owner-private-conformance-inventory-boundary.md)
- [Admit tiler-metal-aot as a dependency-free driver](../docs/decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md)
