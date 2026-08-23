---
id: honour-the-declared-access-maps-in-the-cooperative-contraction-emission
title: Honour the declared access maps in the cooperative contraction emission
status: todo
priority: p1
dependencies: []
related: [admit-a-batched-cooperative-contraction-for-the-attention-structures, realize-the-tiled-contraction-schedule-and-its-metal-emission, lower-and-emit-the-batched-cooperative-contraction]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, contraction, lowering, fail-closed]
---
## User-visible outcome

A cooperative contraction emits addressing derived from the operand access maps the region declares, so an operand whose layout is not the hardcoded one is either emitted correctly or refused by name — never lowered to a kernel that reads the wrong elements.

## Why this exists

Found 2026-08-22 by `worker-batched` while widening the blocked binding, and filed immediately rather than folded in: **it is a live latent defect, not a consequence of that widening.**

**Fact — the emitter discards its declared access maps.** `crates/tiler-ir/src/kernel/lower.rs` contains `let _ = (left_addr, right_addr);` — verified by the coordinator at `b77e65af`, one occurrence. `emit_cooperative_contraction` then hardcodes `[M,K]` for the left operand and `[N,K]` for the right.

**Fact — nothing constrains the sources to that layout.** So a `[K,M]` left operand **verifies and lowers to a silently wrong kernel today**. The addressing is right for the shapes the existing fixtures happen to use and wrong for any other, with no refusal anywhere.

**Fact — the attention value structure is exactly such a case.** Its right operand `[g,s,d]` is *middle*-contracted, i.e. `[K,N]` rather than the hardcoded `[N,K]`. So this is not hypothetical: it sits directly on the batched path the sibling ticket is opening.

**Why p1 and not p0.** The reachable population today is bounded by what the schedule layer admits — `cooperative_contraction_plan` still refuses rank four by name, and the existing fixtures use the hardcoded layout. **But the refusal that bounds it is the one [`lower-and-emit-the-batched-cooperative-contraction`](lower-and-emit-the-batched-cooperative-contraction.md) exists to remove**, which is why that ticket depends on this one. Land this first.

## Required work

- Re-audit all three Facts at your base with a per-Fact verdict.
- Decide **by reading** between deriving the addressing from the declared maps and refusing the layouts the emitter cannot express. **Both are acceptable; silently mis-addressing is not.** If you refuse, the refusal must be typed and name the layout, not the shape.
- **Construct the wrong-layout case and show it either emitted correctly or refused**, quoting the output. The whole finding is that this state lowers today — a repair asserted without that construction has not been demonstrated.
- Perturb each behaviour separately; a perturbation reddening everything cannot show which is load-bearing. Before trusting any new check, state what it would take for it to say *no* and confirm that case is reachable.
- **State whether any identity value moves. Expected: none** — addressing is emitted, not encoded into schedule identity — but rederive, and **stop and report** if one does.

**One warning carried from the delivering lane:** do **not** route the addressing through `emit_offset`. It would add a divide and modulo per operand per round to a kernel that has a retained timing, so the cost would be real and measured against a record that did not pay it.

## Non-goals

The rank-N lowering and Metal emission, which depend on this; widening what the schedule layer admits; and any change to the declared access-map vocabulary.

## Closes when

An operand whose layout differs from the emitter's assumption is emitted correctly or refused by name, the wrong-layout construction is watched producing that outcome with its output quoted, each behaviour is perturbed separately, and no identity value has moved.
