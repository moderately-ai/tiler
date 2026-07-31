---
id: scope-causal-structure-aware-attention-schedules
title: Scope whether a schedule may exploit the causal mask's structure
status: todo
priority: p2
dependencies: [retain-the-c1-attention-block-conformance-evidence]
related: [design-attention-program-vertical, plan-the-recomputing-attention-decomposition, realize-the-attention-contractions-on-metal, admit-the-softmax-family, implement-parallel-reduction-strategies, reduction-semantics-contract]
scopes: [research/scheduling, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, scheduling, numerics, attention, performance, language-model]
---
## User-visible outcome

The corpus says whether a schedule may skip the causal mask's masked contributors — the single largest optimization available in attention, worth about half the score and value work at long context — and if it may not, why, in a form a later reader can act on rather than rediscover.

## Why this is a decision and not an implementation detail

**Measurement — skipping is a value change, and it is reachable from ordinary data.** At the C1 prefill shape, query position 0 attends to position 0 alone, so its probability row is `0x3f800000` followed by nine exact `0x00000000`. Each of those nine contributes `+0.0 × v` to the value contraction, which is `+0.0` where `v` is positive and `-0.0` where `v` is negative. With `v` at the attended key set to `-0.0`, the fold's seed — the first product, since the contraction profile declares no `initial` — is `0x80000000`, and the completed strict ascending fold over all ten contributors is `0x00000000`. The reference returns the same. The [attention-block probe](../spikes/program-planning/attention-block-reference/README.md) retains the bits. **Inference — the masked tail is what performs that sign change**, so a schedule that omitted the masked contributors would return `-0.0` where the reference returns `+0.0`.

**Inference — so the answer today is derived, not open.** The architectural contract holds that an option which can return a silently wrong result is a defect rather than an alternative, and no permission in the numerical contract covers a signed-zero rewrite. **Skipping is forbidden and the refusal is required.** What is genuinely open is the shape of a future permission, and that is what this ticket scopes rather than escalating a question the constraints have already answered.

**Inference — the prize is large enough that "forbidden, with no stated route" would be a bad outcome.** A causal row of length `S` has on average `S/2` masked entries, so at the B1-d prefill row a structure-aware schedule would avoid roughly half of `2 · 16 · T · S · 128` = 2.75 × 10¹¹ multiply-accumulates per layer, plus half the exponentials in the softmax. Leaving that unaddressed would make every later performance discussion start from the same rediscovery.

## Required analysis

- **Enumerate the candidate routes and test each against correctness before cost.** At minimum: a declared signed-zero relaxation with its own identity, resolved per operation as ADR 0011 requires; a proof obligation discharged per program that the skipped contributors cannot change the result — which is not generally true, as the measurement above shows, so state exactly the value precondition under which it is, and what proves it; a `Softmax` whose contract declares that masked positions are *excluded* from the value contraction's contributor sequence rather than contributing zero, which changes the operation's meaning rather than relaxing its order; and an unchanged strict contract in which the optimization simply does not exist.
- **Say which of the three numerical dimensions each route consumes**, and whether any consumes a dimension that does not exist. Reassociation and permutation are independent; a signed-zero rewrite is neither of them, and naming it as one would be the inaccuracy [the reduction contract](../docs/research/numerics/reduction-semantics-and-legality.md) exists to prevent.
- **State the interaction with K-padding**, which is the same hazard from the other direction: padding structure 3's contracted extent to a tile multiple injects `+0.0` contributors into a fold whose seed is the first product, and the neutrality proof [Numerical semantics](../docs/numerical-semantics.md) requires is exactly the proof a skipping schedule would also owe. One argument may settle both, and if it does, say so rather than writing it twice.
- **State the block-structure variant separately from the element variant.** Skipping whole `(t, s)` tiles that lie entirely above the diagonal is the form a real implementation wants, and it has the same signed-zero consequence at the tile granularity plus a different feasibility story: it needs the mask's structure to be *known to the schedule*, which the additive-input mask route deliberately does not expose. Whether that reopens the derived-predicate mask — currently blocked by the absent boolean dtype and [ADR 0084](../docs/decisions/0084-reference-canonical-index-expressions-from-domain-predicates.md)'s predicate vocabulary — is part of this question.
- **End in one of the four research outcomes.** A contract update, an accepted decision, a bounded experiment, or an explicitly deferred question with a reconsideration trigger. An open note that does not say what evidence would close it is not an outcome.

## Non-goals

Implementing any such schedule. Reopening the mask's fill convention, which is decision D-1's and belongs to [`admit-the-softmax-family`](admit-the-softmax-family.md). Deciding whether a distributivity permission should exist, which is a different dimension and a different ticket. Any performance measurement — the arithmetic above bounds the prize and does not estimate the saving.

## Reconsideration trigger

Active now, at the moment the block's timings exist: the masked contributors are between a third and a half of the attention chain's work at every B1 row, and the first person to look at those timings will propose skipping them. Recording the derived refusal and the routes before that happens is the point.

## Closes when

The corpus states whether a schedule may exploit the causal structure, under which permission or proof if so, and with the C1 signed-zero case as the counterexample any proposed route must survive.
