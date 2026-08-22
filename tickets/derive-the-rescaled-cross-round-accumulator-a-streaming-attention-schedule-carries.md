---
id: derive-the-rescaled-cross-round-accumulator-a-streaming-attention-schedule-carries
title: Derive the rescaled cross-round accumulator a streaming attention schedule carries
status: deferred
priority: p3
dependencies: []
related: [derive-the-multi-round-two-level-reduction-composition, accept-adr-0100-multi-round-reduction-composition, admit-a-round-dependent-cooperative-staging-span, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller, derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [research, scheduling, reductions, identity, deferred]
---
## User-visible outcome

A derivation of whether the cooperative schedule vocabulary can state a loop-carried accumulator whose per-round update is *not* the same binary operation as the inner combine — and, if it can, what identity obligation that update carries. A streaming attention schedule's output accumulator is the motivating case; the question is a general one about the multi-round composition's identity walk.

## Why this exists

**Fact — [ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) decision 4 discharges its three identity sites with one argument, and that argument is stated over one operation.** Its text: "The same two-sidedness proof discharges all three sites, because the outer fold and the round accumulator are the same binary operation as the inner combine." Its decision 5 makes the peeled round zero load-bearing for exactly this reason — an accumulator seeded with the region's `empty_identity_bits` "would commit `+0.0` for a row whose true strict sum is `-0.0`, with every lane identity correct", and the record says in as many words that a future emission generalizing the peel away "reintroduces a second identity obligation no schedule field states, and that is a case a test must be able to fail on."

**Inference — a streaming attention schedule's output accumulator breaks decision 4's premise.** Its update is `O ← O · r + (P_block · V_block)`, where `r` is a rescale factor computed inside the same round from the running maximum. That is a Horner nesting whose per-round combine is a multiply-then-add, not the inner combine. ADR 0100's identity walk therefore does not reach it, and whether decision 5's peel is sufficient for it is unestablished rather than established.

**Fact — the question is separable from the numerical permission, and that separation is why it is worth a ticket at all.** Whether a schedule may *state* such an accumulator is a vocabulary and identity question the cooperative verifier answers. Whether a rewrite may *produce* one consumes distributivity, which [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) declines, and elementary-function identity, which [ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) reserves. Both were reaffirmed or accepted on 2026-08-06.

**Inference — so it is filed `deferred` rather than `todo`, and the ground is AGENTS.md's.** Its only identified consumer is a rewrite two accepted decisions independently refuse. Deriving a schedule construct whose sole demand is a refused rewrite is work whose value depends on that refusal moving, and the board must not offer non-work. Found by [the flash-class capability record](../docs/research/program-planning/flash-class-capability-set.md)'s axis 3, which states it as the one construct on that axis with no seam and no reservation.

## What this ticket must produce when it fires

- **Whether the accumulator's per-round update is statable at all** in the current vocabulary, read at the cooperative tile and its verifier rather than inferred from ADR 0100's prose, and if not, exactly which rule refuses it and by what name.
- **The identity obligation, derived per site rather than inherited.** ADR 0100's three-site walk (a contributor-free lane, the outer fold over `G` slots, the cross-round accumulator) must be re-run with a combine that is not two-sided in the same way, including the `-0.0` case its decision 5 turns on.
- **Whether the peel generalizes or a second seeded identity is needed**, with a test that could watch the wrong answer fail — ADR 0100's own open questions already carry "the peel's generality is not proved for a two-level emission" as a deferral closing on exactly such a test.
- **Whether this is one capability or two**, against `admit-a-round-dependent-cooperative-staging-span`'s precedent: that record derives that a round-dependent *span* changes the decision procedure the cooperative verifier rests on. A round-dependent *combine* may or may not; say which, and do not fold one into the other.

## Non-goals

Implementing anything; admitting any schedule construct; proposing an ADR; deriving the numerical bound of the rewrite that would produce the accumulator (owned by the tree-fold bound ticket and by the rule-shape ticket); re-deriving the round-dependent staged span, whose deferral remains unfired for its own separate reasons.

## Trigger

**Either of two, and they are independent.**

1. **[ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md)'s second reopening condition resolves in the admitting direction**, jointly with [ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) item 5, so that a rewrite producing this accumulator becomes reachable. That resolution is Tom's and nothing here presumes it.
2. **A second consumer appears that wants a loop-carried accumulator whose per-round combine differs from the inner combine**, independent of any attention rewrite — a double-buffered blocked contraction with a scaling epilogue is the shape to watch. One consumer is a special case; two is a vocabulary gap.

## Closes when

The corpus states whether a cooperative schedule may carry an accumulator whose per-round update is not the inner combine, the identity obligation is derived per site with a test that could fail, and the one-or-two-capabilities question against the round-dependent span is answered rather than assumed.

## Trigger check log

- 2026-08-06 — **not fired.** Trigger 1: ADR 0095 was reaffirmed on 2026-08-06 with its decline standing, and its second reopening condition names three prerequisites of which one has no owner until [`derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold`](derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold.md) runs. Trigger 2: no second consumer identified; the tiled contraction's accumulator is the inner combine and ADR 0100's composition needs no round-varying combine either. Reproduce the first half in one line: `grep -n 'decision_status' docs/decisions/0095-decline-a-distributivity-permission.md` returns `accepted`, and `grep -c 'Reaffirmation — 2026-08-06' docs/decisions/0095-decline-a-distributivity-permission.md` returns `1`.
- 2026-08-09 — **not fired; one prerequisite description has advanced.** The online-softmax rule-shape derivation is now `done`, but ADR 0095 still declines distributivity and ADR 0101 still admits no elementary-identity permission, so trigger 1 did not resolve in the admitting direction. No second non-attention consumer with a round-varying combine has appeared, so trigger 2 also remains absent. Recheck the completed rule-shape ticket, both accepted decisions, and the schedule vocabulary before reopening this derivation.
