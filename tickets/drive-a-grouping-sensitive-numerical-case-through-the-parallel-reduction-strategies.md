---
id: drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies
title: Drive a grouping-sensitive numerical case through the parallel reduction strategies
status: in-progress
priority: p2
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal, calibrate-and-activate-parallel-reduction-selection, implement-parallel-reduction-strategies, settle-contraction-chain-distributivity-permission]
scopes: [implementation/runtime, contracts/numerics]
shared_scopes: [project/tickets]
tags: [numerics, reductions, evidence-gap, measurement]
claimed_from: todo
assignee: agent-grouping
lease_expires_at: 1785696201
---
## User-visible outcome

The parallel reduction strategies are exercised on operands where regrouping actually changes the answer, so the evidence that they reduce correctly is separated from the evidence that a reassociation-permitting contract means what it says.

## Why this exists

**Measurement — the existing hardware evidence is exact by construction, and says so.** [`realize-parallel-reduction-strategies-on-metal`](realize-parallel-reduction-strategies-on-metal.md) executed the serial fold, the single-workgroup tree, and the multi-pass split on the qualified host and got `41700000` from all three, matching `tiler-reference` bit for bit. Its operands are `1.0, 2.0, 4.0, 8.0`. **Every grouping of those is exact in `f32`**, which is exactly what makes a single serial-fold oracle *valid* for all three strategies under a contract that permits regrouping — a grouping-sensitive operand set would make a correct regrouped implementation legitimately disagree with a serial reference, and the run would refuse a strategy for being right. Every subset sum is also distinct, so a dropped, double-counted, or unsynchronized contributor cannot cancel.

**Inference — so what is proved is contributor-set correctness, not rounding behaviour.** That ticket states it plainly: "Regrouped rounding was not observed and is not claimed." Nothing in the corpus currently observes a `FLUSH_AND_REASSOCIATE_F32` program producing a different — and permitted — answer from the serial fold. Until something does, the permission the contract grants is exercised only in the negative: the flush-only contract retains no split.

## The design problem this ticket owns, and it is the whole difficulty

A serial oracle cannot be the check. The moment operands are grouping-sensitive, disagreement with a serial fold is the *expected* outcome for a legally regrouped strategy, so the oracle has to change shape. State what replaces it before writing the case. Candidates to run the elimination over, not a menu to pick from:

- An exact higher-precision reference (float64 or exact rational) plus a derived bound on the permitted spread, with the bound derived from the declared contributor count and the contract rather than fitted to the observed answer.
- The exact set of groupings the declared strategy is permitted to produce, enumerated for a small contributor count, with the observed answer required to be a member.
- A pair of operand sets — one exact, one grouping-sensitive — where the exact one pins contributor-set correctness and the sensitive one pins that the answers differ *in the direction the strategy's own grouping predicts*.

**A tolerance is not an answer.** `docs/correctness-and-testing.md`'s standing position is that a difference is attributed to a named cause or it is a defect. An observed spread that is merely "small" proves nothing about whether the strategy grouped as it declared.

## Required evidence

- A grouping-sensitive operand set at a recorded seed or written out explicitly, with the exact `f32` bit patterns stated rather than decimal literals.
- Each strategy's answer recorded as a bit pattern, and each difference from the serial fold attributed to the specific regrouping the strategy declares — not to a tolerance.
- The check watched failing: perturb one strategy's grouping and confirm the oracle refuses the perturbed answer. An oracle that accepts every answer in the permitted spread must still refuse one outside it, and that refusal has to be observed.
- The measurement boundary stated exactly: host, OS build, toolchain, contributor count, contract, and what the result does **not** generalize to.

## Explicit non-goals

Measured crossover and winner activation stay with [`calibrate-and-activate-parallel-reduction-selection`](calibrate-and-activate-parallel-reduction-selection.md). No cost-model work; this is a numerical-meaning question, not a performance one. No new strategy.

## Closes when

A grouping-sensitive case runs both parallel strategies on a qualified host, every difference from the serial fold is attributed to a named regrouping rather than absorbed by a tolerance, the oracle has been watched refusing an answer outside the permitted set, and the boundary is stated.

## Graph maintenance

Filed 2026-08-02 at integration of `realize-parallel-reduction-strategies-on-metal`, which stated this boundary rather than letting an exact-by-construction operand set read as evidence about rounding.
