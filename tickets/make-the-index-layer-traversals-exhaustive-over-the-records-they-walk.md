---
id: make-the-index-layer-traversals-exhaustive-over-the-records-they-walk
title: Make the index-layer traversals exhaustive over the records they walk
status: in-progress
priority: p3
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing]
claimed_from: todo
assignee: worker-traversal
lease_expires_at: 1787468243
---
## User-visible outcome

The index-layer traversals that walk a compacted record visit every variant and field, so a widened vocabulary is a build error at the walk rather than a branch that is silently never taken.

## Why this exists

Filed 2026-08-22 by the coordinator from the sibling sweep of [`destructure-the-framed-records-in-the-index-region-identity-encoders`](destructure-the-framed-records-in-the-index-region-identity-encoders.md), landed as `a0659d05`. That lane destructured **16** records across the identity encoders and their length twins, removing seven rest patterns so `identity.rs` now carries none. It reported this remainder and correctly declined it: a traversal is not an encoder, so it sat outside that ticket's scope.

**Fact — reported by `worker-regionenc`, NOT verified by the coordinator.** `visit_expression_dimensions` in `crates/tiler-ir/src/index/builder/compact.rs` still matches `IndexNode::FloorDiv { dividend, .. }`. Re-derive this at your base before acting; it is a worker report and secondhand.

**Why it is the same defect class with a different consequence.** An encoder that elides a field silently narrows an identity. A *traversal* that elides a field silently fails to visit — so a widened variant is never reached, and whatever the walk exists to collect is quietly incomplete. Neither fails loudly, and both are invisible to every test that does not already know the new field exists.

**Inference — the blast radius is different and probably smaller, which is why this is p3 and not p2.** `visit_expression_dimensions` is a reachability walk feeding compaction, not a byte encoder, so a miss does not move `tiler.index-region.v11` directly. But it can make a *later* encoder see a narrower input, which is the same failure one layer removed. Establish which by reading before deciding how hard to fix it.

## Required work

- Re-audit the Fact at your base with a verdict, and **census the traversals rather than fixing the one site**. The delivering lane's censusing method is the one to copy: it enumerated **by record type reachable from `CompactedRegion`**, not by grep string, because the loops bind single-letter names (`t.role`, `d.extent`, `v.definition`, `o.access`) that any `gather.`-shaped or `node.`-shaped search misses entirely. State the vocabulary you used and why it is complete.
- For each traversal, decide by reading whether an exhaustive match is correct there. **A rest pattern is not automatically a defect** — a walk that genuinely only needs one field may be right to say so, in which case record why at the site rather than widening it.
- Where the walk should be exhaustive, remove the rest pattern and bind unused fields to `_` so a new field is a build error at the walk.
- Perturb by adding a variant or a field and quoting the build error, confirming the span lands **in the traversal**. Note the negative control the sibling lane used: at its base the same perturbation produced only an `E0063` at the *constructor*, and filling that in let the field never reach the identity — that is what makes the encoder-span requirement meaningful.
- **State whether any identity value moves. Expected: none.** Rederive rather than assume, and stop and report if one does.

## Non-goals

The identity encoders and their length twins, already done by `a0659d05`. `remap_node` and `remap_operation`, which build through exhaustive struct literals and already error on a new field — that lane checked and deliberately left them. Changing what any traversal collects.

## Closes when

Every index-layer traversal either visits its record exhaustively or records why a rest pattern is correct there, the census states its vocabulary and why it is complete, no identity value has moved, and a widening perturbation is watched failing at a traversal span.
