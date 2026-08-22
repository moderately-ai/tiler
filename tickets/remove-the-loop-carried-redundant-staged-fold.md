---
id: remove-the-loop-carried-redundant-staged-fold
title: Remove the loop-carried body's redundant staged fold
status: deferred
priority: p2
dependencies: []
related: [lower-a-loop-carried-cooperative-body, realize-the-strict-contraction-on-metal, realize-the-tiled-contraction-schedule-and-its-metal-emission]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [research, physical-planning]
---
## The cost, and why it is paid

**Fact — every participant folds the whole staged set on every round.** `emit_loop_carried_cooperative` (`crates/tiler-ir/src/kernel/lower.rs`) puts the staged fold outside every predicate, because a predicated region produces no values and the accumulator has to survive the round loop's back edge. The single-round `emit_cooperative` keeps its staged fold inside the commit guard, where exactly one participant runs it — so the loop-carried shape pays `participants - 1` extra folds of `participants - 1` combiner applications per round that the single-round shape does not.

**Inference — the pure staged-fold redundancy is on the order of `(participants - 1)² * rounds` extra combiner applications per workgroup** (each fold performs `participants - 1` combiner ops from slot 0 over `1..participants`; all lanes fold where one would suffice), entirely in threadgroup memory, against `participants * contributors_per_partition * rounds` boundary loads. Whether that is measurable depends on the ratio, which depends on the tile; for the `tiled` contraction's `16 x 16` shape over `K/16` rounds it is not obviously negligible.

## What would remove it, and why the shape is Tom's

The predicated region would have to *yield values*, so the staged fold could sit inside the commit guard and still produce the loop's next accumulator. That is a KIR vocabulary change: `OperationKind::Predicated { predicate, body }` carries no results today, and giving it results means deciding what an unexecuted branch yields — an explicit `select`, an else-region, or a defaulted value, each a different construct with a different identity encoding and a different verifier obligation. **`OperationView` is the public, `#[non_exhaustive]`, re-exported surface; `OperationKind` is `pub(super)` and not crate-public.** The identity encoding of `Predicated` (tag `0x18` + predicate + body only under domain `tiler.kernel.v7`) is load-bearing. A value-producing predicated shape is therefore Tom's public-boundary and identity-domain decision regardless of how good the derivation is.

## Activation triggers

Either of these fires it; neither has:

- A measured multi-round cooperative kernel on device where the redundant staged fold is a non-trivial share of the kernel's time, taken under the loop in [AGENTS.md](../AGENTS.md)'s performance section.
- Tom accepting a value-producing predicated region for an unrelated reason, at which point this becomes a mechanical follow-up rather than a question.

Until then the redundancy is a stated cost of an implemented capability, recorded in `emit_loop_carried_cooperative`'s own documentation, and not a defect.

## Fact audit — 2026-08-10

Phase B repair against audit report at base `c99ac54950f2` (ticket content hash `d8fca12500efcde587760c57722cb42ad10f0cab4e721f3b69ab2fe1fdfbd8dd` at Phase A).

- **Related graph:** added `realize-the-tiled-contraction-schedule-and-its-metal-emission` as the live deferred product owner that would first make activation trigger 1 evaluable on device; kept `realize-the-strict-contraction-on-metal` as historical related only.
- **Public surface:** struck the false claim that `OperationKind` is public; recorded `OperationView` re-export + `OperationKind` `pub(super)` + Predicated identity tag `0x18`.
- **Inference count:** staged-fold redundancy tightened from `(participants - 1) * participants * rounds` to `(participants - 1)² * rounds` combiner applications (each fold is `1..participants` from seed slot 0).
- **Trigger log:** replaced stale `:755`/`:1289` with anchors; named the deferred tiled successor rather than the closed strict-contraction ticket as live realization owner.

## Trigger check log

- 2026-08-04 — **not fired.** Trigger 2 is unmet: `OperationKind::Predicated { predicate, body }` still carries no results (anchor `/// Executes a nested block when a predicate holds.` on `OperationView::Predicated` in `crates/tiler-ir/src/kernel/model.rs`; historical line cites `:755`, `:1289` were stale by audit base `c99ac54950f2`), so no value-producing predicated region has been accepted. Trigger 1 is unmet: the only multi-round cooperative consumer is the tiled contraction, whose realization is still `deferred`, so no multi-round kernel has been measured on device. Recheck: `grep -n 'Predicated {' crates/tiler-ir/src/kernel/model.rs`.
- 2026-08-09 — **not fired.** `OperationKind::Predicated { predicate, body }` and `OperationView::Predicated { predicate, body }` still carry no result or else value, and the live tiled contraction realization owner remains `realize-the-tiled-contraction-schedule-and-its-metal-emission` (`status: deferred`; the related `realize-the-strict-contraction-on-metal` edge is historical only — closed/superseded). No measured multi-round device kernel exists, so neither trigger has fired.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `grep -n 'Predicated {' crates/tiler-ir/src/kernel/model.rs`, and run at this base it returns **5** lines. A result other than the 5 recorded here is the changed answer. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
- **Reason repaired — 2026-08-22; verdict unchanged, neither trigger fired.** The 2026-08-09 entry's ground for trigger 1 is **inverted at this base**, and the retired wording is quoted rather than removed: it rests on `the live tiled contraction realization owner remains` `realize-the-tiled-contraction-schedule-and-its-metal-emission` at `status: deferred`. That ticket is now `status: done` — `grep -m1 '^status:' tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md`. The entry's own logic would read that as the blocker clearing, so leaving it unrepaired is exactly the shape that makes a later sweep believe this deferral is one ticket from dispatch.

  **Trigger 1 is still unmet, and on stronger ground than the one that went stale.** Two independent reasons, either sufficient.

  First, the offer never landed. That ticket's own remainder section says so — anchor `The compiler alternative is not landed` — and the tree agrees: `crates/tiler-compiler/src/physical.rs` documents its tiled-contraction region identifier with `in this crate constructs a region with this identifier: the tiled` alternative not being offered yet. No `compile` call selects a tiled contraction, so the contraction vertical still rides the `direct` realization and no multi-round kernel is reachable to measure.

  Second — and this outlives the first — **a tiled contraction could not fire this trigger even once it is offered, because it does not execute the redundancy this ticket is about.** `fn emit` in `crates/tiler-ir/src/kernel/lower.rs` dispatches two disjoint branches: `plan.cooperative` reaches `fn emit_cooperative`, which delegates to `emit_loop_carried_cooperative` only when `cooperative.round_barrier.is_some()`, while `plan.contraction` reaches `fn emit_cooperative_contraction`, a sibling emitter that carries one accumulator through its own round loop and never calls `emit_loop_carried_cooperative`. The redundant staged fold this ticket costs lives in the first branch alone. Trigger 1 needs a measured multi-round **cooperative reduction** kernel on device; the tiled contraction is not one, so its landing moves this ticket no closer. The cost itself is unchanged and still documented at its own site, anchor `staged fold is outside every predicate` in `lower.rs`.

  **Trigger 2 is unmet, re-derived from the type rather than from the entry above.** `grep -n 'Predicated {' crates/tiler-ir/src/kernel/model.rs` returns **5 lines**, the count the entry above recorded: the `OperationKind::Predicated` definition, the `OperationView::Predicated` definition, and three match arms. Both definitions still carry `predicate` and `body` and nothing else — read them, since the count alone cannot see a third field. The sharper check is the sibling one line below: `SerialLoop` in the same enum carries `initial` and `yields`, so this vocabulary does have value-yielding regions and `Predicated` deliberately is not one. The changed answer is a result or else-value field on either `Predicated`, not a change in the line count; the identity tag `0x18` is still pushed for it in the same file.
