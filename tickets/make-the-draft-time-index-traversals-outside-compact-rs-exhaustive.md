---
id: make-the-draft-time-index-traversals-outside-compact-rs-exhaustive
title: Make the draft-time index traversals outside compact.rs exhaustive
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing]
---
## User-visible outcome

The draft-time index traversals outside `compact.rs` visit every variant and field of the records they walk, so a widened vocabulary is a build error at the walk rather than a branch silently never taken.

## Why this exists

Filed 2026-08-23 by the coordinator as the reported remainder of [`make-the-index-layer-traversals-exhaustive-over-the-records-they-walk`](make-the-index-layer-traversals-exhaustive-over-the-records-they-walk.md), which landed as `f5f4cff1` and took `crates/tiler-ir/src/index/builder/compact.rs` from **9** rest patterns to **0** with no wildcard match arms left. That lane reported this population and correctly declined it as outside its file scope and materially larger.

**Fact — the population, measured by the coordinator at `a0fd5af2`.** `grep -c '\.\. *}'` under `crates/tiler-ir/src/index/builder/`: **`proof.rs` 18**, **`reduction.rs` 2**, **`gather.rs` 1**. The delivering lane reported "at least 5 sites" in `proof.rs`; that was a conservative floor and the measured figure is higher. **Re-derive it yourself** — a raw `grep -c` counts lines and does not distinguish a traversal from a deliberate single-field projection, which is the distinction this ticket turns on.

**Fact — the elision is demonstrably silent, and the delivering lane proved it on this very population.** Its perturbation added `probe: bool` to `IndexNode::FloorDiv`. With `compact.rs` repaired the compiler raised `error[E0027]` at two traversal spans; with `compact.rs` reverted to base those two spans **disappeared entirely** — 9 errors instead of 11 — and the field compiled straight through both walks. It recorded that `crates/tiler-ir/src/index/builder/gather.rs:205` compiles clean through a widened `FloorDiv` today.

**Why a traversal matters differently from an encoder.** An encoder that elides a field silently narrows an *identity*; a traversal that elides one silently fails to *visit*, so a widened variant is never reached and whatever the walk collects is quietly incomplete. `proof.rs` is the sharpest case: its `verify` computes reachable accesses, reachable operations, and used reductions, and an unvisited branch there is an obligation that goes unchecked rather than a byte that goes unwritten.

## Required work

- Re-audit the population at your base with a per-file verdict, and **census by record type reachable from `CompactedRegion`**, not by grep string. The delivering lane's spelling — `grep -n '\.\. *}'` — is the one that catches a rest pattern regardless of the binding name, because these loops bind single letters (`t.role`, `d.extent`, `v.definition`) that any `node.`-shaped search misses. State your vocabulary and why it is complete.
- **A rest pattern is not automatically a defect.** A walk that genuinely needs one field may be right to say so; where it is, record why at the site rather than widening it. The delivering lane left `alpha_dimension_order`, `visit_access_dimensions`, and the `AccessData` accessors unchanged for exactly that reason, and its reasoning is worth reading before you touch the analogous sites here.
- Where the walk should be exhaustive, remove the rest pattern and bind unused fields to `_`.
- Perturb by widening a variant and quote the build error, confirming the span lands **in the traversal**. Reproduce the negative control: with the file at base, that span must be absent — that asymmetry is the whole demonstration.
- **State whether any identity value moves. Expected: none** — these are draft-time walks consumed before compaction, not encoders. Rederive rather than assume, and stop and report if one does.

## Non-goals

`compact.rs`, done by `f5f4cff1`. The identity encoders and their length twins, done by `a0659d05`. `remap_node` and `remap_operation`, which build through exhaustive struct literals and already error on a new field. Changing what any traversal collects.

## Closes when

Every draft-time traversal under `crates/tiler-ir/src/index/builder/` either visits its record exhaustively or records why a rest pattern is correct there, the census states its vocabulary and why it is complete, no identity value has moved, and a widening perturbation is watched failing at a traversal span with its base-tree negative control quoted.
