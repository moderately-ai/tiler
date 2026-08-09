---
id: accept-the-structural-region-access-vocabulary
title: Accept the structural region access vocabulary
status: done
priority: p1
dependencies: [admit-the-structural-families-into-the-scheduled-region-vocabulary]
related: [reach-a-verified-kernel-through-the-structural-families]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir]
---
## What is being accepted

The public surface [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) landed as a **draft**. It is tested and in use on the compile path; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it.

## The exact surface

In `tiler_ir::schedule`:

- **`AxisDecode`** — a struct with three public fields (`divisor: u64`, `modulus: u64`, `mirrored: bool`) and `read`, `fixed`, `is_canonical`. Deliberately not `#[non_exhaustive]` and deliberately without a `Default`: a decode is three independent facts, all of which participate in canonical identity, so a field a producer could omit would be a map two regions could disagree about while sharing bytes.
- **`LogicalAccess::ReindexBijection { operand_shape, result_shape, axes }`** and **`LogicalAccess::BroadcastReplication { operand_shape, result_shape, axes }`** — additive under the enum's existing `#[non_exhaustive]`.
- **`reindex_decodes_are_bijective`** and **`broadcast_decodes_are_replicating`** — the two admission predicates, public because the compiler's request boundary refuses on them before assembling a region that could not be built.

In `tiler_ir::kernel`:

- **`BinaryOp::IndexSubtract`** — additive under that enum's existing `#[non_exhaustive]`. Its contract is that the result is proven non-negative; there is deliberately no wrapping or saturating sibling.

## The choices worth objecting to

- **One decode struct shared by two relations, rather than a spelling per reindex form.** All six registered forms reduce to one `(linear / divisor) % modulus` per operand axis; the alternative is six variants restating the semantic form vocabulary inside the physical one.
- **Two `LogicalAccess` variants rather than one.** They share their arithmetic and differ in their admission rule, because a bijection and a replication license different conclusions about what a read consumes.
- **The predicates being public at all.** The alternative is refusing only inside the schedule verifier and letting the request boundary assemble a region it then fails to build.

## Evidence

The deriving ticket's Outcome section carries the full argument: the per-site injectivity reasoning, the pin table (schedule domain did not step; the request subject did, at two sub-tags), the correctness argument per relation, and five watched-failing perturbations — two of which showed a rule that looked tested and was not.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the surface is in use and labelled a draft.

## Decided — accepted

Accepted by Tom on 2026-08-06 at the morning decision review in the coordination session, witnessed first-hand by the coordinator, with the evidence packet this node carries. Acceptance is not stabilization; the surface is accepted pre-alpha vocabulary.

## Current-state correction — 2026-08-09

The source declarations now carry the accepted surface without a live draft
marker: `AxisDecode`, the two `LogicalAccess` variants, both admission
predicates, and `BinaryOp::IndexSubtract` retain exactly the shapes accepted
above. This correction records the completed label sweep; it changes no
vocabulary, encoding tag, identity domain, or behavior.
