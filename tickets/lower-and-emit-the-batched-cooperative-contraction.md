---
id: lower-and-emit-the-batched-cooperative-contraction
title: Lower and emit the batched cooperative contraction
status: in-progress
priority: p1
dependencies: [admit-a-batched-cooperative-contraction-for-the-attention-structures, honour-the-declared-access-maps-in-the-cooperative-contraction-emission]
related: []
scopes: [implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [contraction, lowering, metal, attention]
claimed_from: todo
assignee: worker-lowerbatch
lease_expires_at: 1787450840
---
## User-visible outcome

Both attention structures reach a lowered, emitted cooperative contraction, so the batched form admitted at the schedule layer becomes a plan a target can actually run.

## Why this exists

Split out 2026-08-22 when `admit-a-batched-cooperative-contraction-for-the-attention-structures` **stopped at a gated, coherent boundary**: the schedule layer now admits a rank-N blocked binding with unit batch extents, cover proved at 4,096 threads over 1,600 positions for both structures, and `cooperative_contraction_plan` still refuses rank four **by name** — which is correct at that boundary rather than an oversight.

**Land [`honour-the-declared-access-maps-in-the-cooperative-contraction-emission`](honour-the-declared-access-maps-in-the-cooperative-contraction-emission.md) first.** The emitter currently discards its declared access maps and hardcodes `[M,K]`/`[N,K]`, so a differently-laid-out operand lowers to a silently wrong kernel. The refusal that bounds that population today is exactly the one this ticket removes. Removing it first would make a latent defect reachable.

## What the delivering lane established, and what it warns about

**No new vocabulary is needed and none should be added.** The schedule vocabulary was already sufficient; only the admission predicate was too narrow. Two options were eliminated on correctness rather than cost: a new `ExecutionBinding` variant lands *additively* into wildcards across 22 files in four crates, and a new `ReductionTopology` variant hits `measured_cost.rs`'s undeletable wildcard and collapses `measured_scores` for **every neighbouring alternative**. Re-derive both before reaching for either.

**`tiler.schedule.v7` did not step**, contradicting the parent ticket's own expectation: `push_shape` already frames rank-then-extents, so a rank-four binding was always *encodable* and only ever unadmitted. Do not assume that carries to lowering — derive it.

**Three pieces remain, and the risk is not evenly spread.**

1. A rank-N `cooperative_contraction_plan`.
2. Source-driven addressing — **do not route it through `emit_offset`**, which adds a divide and modulo per operand per round to a kernel with a retained timing.
3. **The rank-N widening of `verify.rs`'s `BlockedGeom`/`IndexRole` abstract interpreter — the highest-risk piece.** It reasons about geometry rather than transporting it, so a rank assumption there fails quietly.

`tiler-metal` needs only a golden: the delivering lane reports zero rank-two hardcoding there and a 1-D dispatch by construction. Verify that rather than inheriting it.

## Required work

- Re-audit every Fact above at your base with a per-Fact verdict; all are the delivering lane's and the coordinator verified only the discarded-access-maps site and the participant-rank cap.
- Perturb the subject separately for each new behaviour and quote the failure text. **Before trusting any check, state what it would take for it to say *no* and confirm that case is reachable** — the delivering lane found one of its own tests passed with the clause deleted, because zip truncation refused the fixture on extents instead, and rewrote the fixture to isolate the clause.
- State every identity domain that steps and every one that does not, derived on the merged tree; **stop and report** if one you expect not to move does.
- A Metal golden must be shown to **compile** under the qualified toolchain, and any toolchain fact must name the invocation that produced it — `DEVELOPER_DIR=/Applications/Xcode.app` gives `metalfe-32023.883`, while a bare `xcrun` on this host gives `32023.921` via Xcode-beta.

## Non-goals

Widening the participant space — `MAX_COOPERATIVE_PARTICIPANT_RANK = 3` sizes the inline arrays behind `ParticipantSpace::new`, so a rank-four participant space is **unrepresentable**, not merely unimplemented; the space stays rank two and the block takes the output's rank. Timing, which needs the bench host. Any new schedule vocabulary.

## Closes when

Both attention structures lower and emit through the batched path, the addressing derives from declared maps rather than an assumed layout, the abstract interpreter is rank-N with its assumptions tested, each behaviour is watched failing on its own subject, identity consequences are derived, and the repository gate is green with the golden compiled under the qualified toolchain.
