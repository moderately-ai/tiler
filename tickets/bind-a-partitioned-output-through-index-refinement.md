---
id: bind-a-partitioned-output-through-index-refinement
title: Bind a partitioned output through index refinement
status: todo
priority: p1
dependencies: [admit-a-partitioned-write-ownership-contract]
related: [lower-the-concatenate-occurrence-through-partitioned-writes]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing, write-ownership, public-boundary]
---
## User-visible outcome

An index region whose output is written by several roots binds through index refinement, so a partitioned region can carry a refinement receipt instead of being refused for having more roots than results.

## Why this exists

**Fact — refinement binds one root per semantic result, by count.** `bind_results` (`crates/tiler-ir/src/index/refinement.rs:2744-2790`) collects `region.outputs()` and returns `IndexRefinementVerificationError::ResultArity { region_outputs, results }` unless the two counts are equal, then zips them positionally. `ResultBinding` carries exactly one `write_access` and one `written_value`.

**Fact — a partitioned region has more roots than results.** [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) admitted several output roots over one tensor; each is a separate `OutputData` entry, so `region.outputs().len()` counts roots rather than distinct output tensors. A two-root partition of one output presents two "outputs" for one semantic result and is refused as an arity mismatch — a refusal for the wrong reason, since the region is well formed and its ownership is proved.

**Fact — the write-completeness check `bind_results` performs is already satisfied.** `access.write_ownership_proof().is_none()` is what it tests, and a partition member carries `WriteOwnershipProof::PartitionMember` rather than `None`. So the obligation this site is guarding is discharged; only the shape of the binding is wrong.

## What the work is

Decide the binding shape for a result whose region writes it in pieces: whether `ResultBinding` carries a set of write accesses, or whether roots are grouped by output tensor before the arity comparison. The second is smaller and keeps the one-binding-per-result invariant every consumer of `ResultBinding` reads; the first is more faithful to what the region holds. Whichever is chosen, `ResultBinding` is a public item and its shape is a public boundary.

Decide what a receipt records about a partition. A receipt naming one of several roots would be a claim the region does not support; naming all of them changes the receipt's identity encoding, which is an identity-domain step to execute completely or not start.

## Explicit non-goals

- The partition contract itself, which exists.
- The compiler-side lowering, which is [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md)'s.

## Closes when

A partitioned region binds through `bind_results` and produces a receipt whose content is justified for every root, or the refusal is preserved with its reason recorded and the dependent lowering told which it gets. A deliberate perturbation dropping one root from the binding is shown to fail.

## Graph maintenance

- `implementation/ir` alone: `refinement.rs` is in `crates/tiler-ir/`.
- Filed by the partition-contract ticket, which read this site in full and left it unchanged because relaxing it is a public-boundary redesign of `ResultBinding` rather than part of admitting the proof form.
