---
id: remove-the-loop-carried-redundant-staged-fold
title: Remove the loop-carried body's redundant staged fold
status: deferred
priority: p2
dependencies: []
related: [lower-a-loop-carried-cooperative-body, realize-the-strict-contraction-on-metal]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [research, physical-planning]
---
## The cost, and why it is paid

**Fact — every participant folds the whole staged set on every round.** `emit_loop_carried_cooperative` (`crates/tiler-ir/src/kernel/lower.rs`) puts the staged fold outside every predicate, because a predicated region produces no values and the accumulator has to survive the round loop's back edge. The single-round `emit_cooperative` keeps its staged fold inside the commit guard, where exactly one participant runs it — so the loop-carried shape pays `participants - 1` extra folds of `participants` additions per round that the single-round shape does not.

**Inference — the cost is `(participants - 1) * participants * rounds` extra `f32` additions per workgroup**, entirely in threadgroup memory, against `participants * contributors_per_partition * rounds` boundary loads. Whether that is measurable depends on the ratio, which depends on the tile; for the `tiled` contraction's `16 x 16` shape over `K/16` rounds it is not obviously negligible.

## What would remove it, and why the shape is Tom's

The predicated region would have to *yield values*, so the staged fold could sit inside the commit guard and still produce the loop's next accumulator. That is a KIR vocabulary change: `OperationKind::Predicated { predicate, body }` carries no results today, and giving it results means deciding what an unexecuted branch yields — an explicit `select`, an else-region, or a defaulted value, each a different construct with a different identity encoding and a different verifier obligation. `OperationKind`, `OperationView`, and the kernel identity grammar are all public, so the shape is Tom's decision regardless of how good the derivation is.

## Activation triggers

Either of these fires it; neither has:

- A measured multi-round cooperative kernel on device where the redundant staged fold is a non-trivial share of the kernel's time, taken under the loop in [AGENTS.md](../AGENTS.md)'s performance section.
- Tom accepting a value-producing predicated region for an unrelated reason, at which point this becomes a mechanical follow-up rather than a question.

Until then the redundancy is a stated cost of an implemented capability, recorded in `emit_loop_carried_cooperative`'s own documentation, and not a defect.

## Trigger check log

- 2026-08-04 — **not fired.** Trigger 2 is unmet: `OperationKind::Predicated { predicate, body }` still carries no results (`crates/tiler-ir/src/kernel/model.rs:755`, `:1289`), so no value-producing predicated region has been accepted. Trigger 1 is unmet: the only multi-round cooperative consumer is the tiled contraction, whose realization is still `deferred`, so no multi-round kernel has been measured on device. Recheck: `grep -n 'Predicated {' crates/tiler-ir/src/kernel/model.rs`.
- 2026-08-09 — **not fired.** `OperationKind::Predicated { predicate, body }` and `OperationView::Predicated { predicate, body }` still carry no result or else value, and the tiled contraction realization remains deferred behind the cooperative-tile public decision. No measured multi-round device kernel exists, so neither trigger has fired.
