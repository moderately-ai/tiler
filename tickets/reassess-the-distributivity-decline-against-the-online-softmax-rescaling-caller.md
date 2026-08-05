---
id: reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller
title: Reassess the distributivity decline against the online-softmax rescaling caller
status: todo
priority: p2
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions, decide-whether-to-admit-a-distributivity-permission, decide-whether-distributivity-directions-share-one-permission, derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [contracts/decisions, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, optimizer, decision]
---
## User-visible outcome

The question [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) answered is put to Tom again against evidence that did not exist when he answered it, so that the flash-class one-pass softmax is either reachable under a stated permission or is refused for a reason that survives the new caller. **This is Tom's decision and nothing here presumes it.** A reaffirmed decline is a complete and useful outcome, and it would then be a decline that has answered its counterexample rather than one that never met it.

## Why this exists

**Fact.** ADR 0095 declined a distributivity permission, and its load-bearing ground was stated plainly: "**No caller exists.** A permission's whole content is that a caller may authorize a freedom; admitting one nobody can spend adds a dimension to every contract … while changing no program's meaning and enabling no rewrite anyone has asked for." The record also states that the vocabulary carrying no caller-less permissions is "the principle the decline rests on rather than a cost estimate".

**Inference, derived in [the certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) and reproducible from its Part 2.** The online-softmax rescaling fold — Algorithm 3 of Milakov and Gimelshein, the fold every flash-class attention kernel uses — consumes distributivity. Unrolling `d_j = d_{j-1} * exp(m_{j-1} - m_j) + exp(x_j - m_j)` gives a Horner nesting whose expansion is a sum of products, so reaching it from `sum_j exp(x_j - m_V)` requires exchanging a product of a sum for a sum of products. That is ADR 0080's dimension exactly. It is therefore **not** a reassociation and not an operand permutation, and under [ADR 0011](../docs/decisions/0011-per-operation-numerical-permissions.md) no existing permission reaches it: the rewrite is illegal under every numerical contract Tiler can currently express.

**So a caller exists for the dimension.** That does not make the decline wrong — it removes the premise the decline was argued from, which is a different and weaker claim, and the difference is the whole reason this is a ticket rather than a supersession.

## What is NOT claimed

**ADR 0095's reopening trigger did not fire, and this ticket does not assert that it did.** That trigger is "the first workload whose natural spelling is a directly regroupable **contraction chain**", with the attention block's operations 19–21 as the worked negative case. The softmax normalizer is an elementwise-scaled reduction, not a contraction chain, so it sits outside the trigger's stated subject rather than being a candidate the trigger admits. Do not open this by claiming the trigger fired; open it by noting that the trigger and the decline's *ground* are two different things, and it is the ground that this evidence reaches.

## What this ticket must produce

- The one atomic question for Tom, with a small concrete tensor-program example — the softmax normalizer at a stated `V` — showing what admitting the permission enables and what declining it forecloses, per AGENTS.md's decision-escalation shape.
- **The price, which is what makes this decision priceable rather than a matter of architectural taste.** The certified-bounds record derives the rewrite's worst-case cost as `(V-1)(u + eps_exp)` to first order, about `2^-14` at `V = 512` in binary32 with a correctly rounded `exp`, and measures it against 22 adversarial cases. A permission whose numerical cost is bounded and small is a different proposition from one whose cost is unknown, and ADR 0095 was decided without that number.
- The re-run elimination against ADR 0095's other three arguments, each of which stands or falls independently: that admitting later is additive and costs nothing now; that a permission admitted without evidence would have to guess its own shape (one permission or two, which [`decide-whether-distributivity-directions-share-one-permission`](decide-whether-distributivity-directions-share-one-permission.md) owns and which becomes live under an admission); and that contraction ordering stays a planning question. **The additive argument is the strongest survivor and it may well decide this again** — if admitting later costs nothing, waiting for the rewrite to be wanted rather than merely derivable is still defensible.
- A second freedom this rewrite consumes that no dimension names, tracked separately at [`name-the-elementary-identity-rewrite-dimension`](name-the-elementary-identity-rewrite-dimension.md). **Both gates must open before the rewrite is legal**, so an admission here alone does not make the softmax rewrite reachable, and the question to Tom must say so or it overstates what it buys.

## Non-goals

Superseding ADR 0095 from a worker branch; deciding the one-or-two-permissions question; implementing any permission; treating the derived bound as authorization for the rewrite.

## Closes when

Tom has answered, and either ADR 0095 is explicitly superseded by a record carrying the new caller and the price, or ADR 0095's reasoning is amended to record that the caller argument met this counterexample and what replaced it. A silent reaffirmation is not an outcome — the next reader would re-derive this from scratch.
