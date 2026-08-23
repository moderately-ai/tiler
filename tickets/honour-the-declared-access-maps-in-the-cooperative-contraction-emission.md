---
id: honour-the-declared-access-maps-in-the-cooperative-contraction-emission
title: Honour the declared access maps in the cooperative contraction emission
status: in-progress
priority: p1
dependencies: []
related: [admit-a-batched-cooperative-contraction-for-the-attention-structures, realize-the-tiled-contraction-schedule-and-its-metal-emission, lower-and-emit-the-batched-cooperative-contraction]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, contraction, lowering, fail-closed]
claimed_from: todo
assignee: worker-accessmaps
lease_expires_at: 1787448544
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

## Fact audit at `d46a4f44`

Re-read at the base this work was done from, which is not the `b77e65af` the Facts were written against.

**Fact 1 — the emitter discards its declared access maps: verified.** `grep -n "let _ = (left_addr, right_addr)" crates/tiler-ir/src/kernel/lower.rs` returned exactly one line. `emit_cooperative_contraction` then computed `left_off` from `IndexMultiply(row, contracted)` and `right_off` from `IndexMultiply(col, contracted)`, which is `[M, K]` and `[N, K]`.

**Fact 2 — nothing constrains the sources to that layout: verified, and narrowed usefully.** `verify_cooperative_contraction` requires each operand axis to name one distinct in-range coordinate whose extent it agrees with, and `verify_blocked_operand_roles` requires the left to read the output's row axis and the right its column axis. Neither constrains the *order*, so at rank two each operand is admitted in both `[free, K]` and `[K, free]` orders — four combinations, of which the emission addressed one correctly. A transposition is a bijection of the operand's own index space, so all four stayed inside their buffers: no bounds proof, element count, or verifier could have caught it.

**Fact 3 — the attention value structure is exactly such a case: verified.** `attention_value_kernel` in `crates/tiler-metal/src/tests.rs` declares its right operand `sources` as `Output { position: 0 }`, `Contracted { position: 0 }`, `Output { position: 3 }` — contracted in the middle. Its own doc states *"a lowering that read axis sources positionally rather than by role would produce one of these kernels for both"*. That kernel reaches the **non**-cooperative `ReductionTopology::Contraction` path, which addresses through `linearize_contraction_operand` and was already correct; the defect was confined to the cooperative path.

## Repair and evidence

Derived, not refused: `ReadAddressing::BlockedContraction` carries one `stride * coordinate` term per operand axis, built by `blocked_contraction_terms` from the declared sources, and the emission sums them. All four rank-two layouts are expressible, so no reachable layout is refused. `emit_offset` is deliberately not used — it would decode from a linear root the blocked body never forms, adding a divide and a modulo per operand per round.

`each_declared_operand_layout_is_addressed_as_declared` interprets the derived body's index arithmetic and compares each operand's round-zero address against an independently written derivation. Run against the base emitter it reported, in one run:

```text
mis-addressed operands: [
    "OperandLayouts { left_transposed: false, right_transposed: true } right: read 338, declared 117",
    "OperandLayouts { left_transposed: true, right_transposed: false } left: read 293, declared 178",
    "OperandLayouts { left_transposed: true, right_transposed: true } left: read 293, declared 178",
    "OperandLayouts { left_transposed: true, right_transposed: true } right: read 338, declared 117",
]
```

The row-major pair is absent from that list, which is what shows the check is not vacuously failing; and the left-only and right-only rows show the two addressings fail separately.

**No identity value moves.** The canonical kernel identity of all seven row-major cooperative-contraction fixtures — exact, predicated, tail-partial, and multi-round — is byte-identical before and after. That is a derived result, not an assumption: the emitted operation order and constant count are held fixed for that layout. The comparison was shown able to say *no*: dropping the stride-constant reuse added one operation and moved every one of the seven identities by 23 bytes.
