---
id: carry-the-rescale-depth-a-parametric-bound-instantiates-from
title: Carry the rescale depth a parametric bound instantiates from
status: deferred
priority: p3
dependencies: []
related: [derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound, decide-whether-to-admit-an-elementary-identity-permission, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, scheduling, reductions]
---
## User-visible outcome

A scheduled candidate carrying an online-softmax rescaling fold states the two shape parameters its bound is instantiated from, so an admission rule reads them rather than infers them from a topology's other fields.

## Why this exists

**Fact.** [The tree-fold record](../docs/research/numerics/tree-fold-online-softmax-bound.md) derives that the fold's price is `(1 + eps_exp)^D * (1 + gamma_{h2+D}) / (1 + gamma_{h2}) - 1`, in the *rescale depth* `D` of the merge tree and the summation depth `h2` of the matched two-pass baseline over the same tree. Both are combinatorial properties of the scheduled fold and depend on no input value.

**Fact.** Neither is stated anywhere. [ADR 0096](../docs/decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md) and [ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) compose a fold over a `(round, subgroup, lane)` block index, and its depths are derivable from the composition's widths — but `D` is not, because a level contributes to `D` only when the partial states entering it carry different maxima, which is a property of what the level folds rather than of how wide it is. A lane running the online recurrence element by element and a lane computing a lane-local maximum then a lane-local sum have identical widths and differ by `k-1` in `D`.

**Inference.** A bound instantiated from a shape the schedule does not declare is instantiated from an assumption, which is exactly the fifth thing the certified-bounds record's admission rule says must never be admitted.

## What this ticket must produce

- Where `D` and `h2` belong: derived by the verifier from fields the schedule already carries, or declared. The record's argument is that `h2` is derivable and `D` is not, so the answer is probably one declared field and one derivation — but that is the derivation this ticket owes, not its premise.
- Whether the declaration is a property of the reduction topology, of the scheduled region, or of the rewrite's own evidence, and what refuses a declaration that disagrees with the tree the schedule actually folds.
- Any public boundary this reaches stops at a draft and an acceptance node; it is not self-accepted.

## Non-goals

Implementing a field; admitting a permission; deriving a bound.

## Trigger

A shape parameter for a rewrite no contract can perform is a field nothing reads, so this waits on the permissions the fold consumes. It fires when **either** [`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md) **and** [`decide-whether-to-admit-an-elementary-identity-permission`](decide-whether-to-admit-an-elementary-identity-permission.md) both resolve in the admitting direction, **or** a second rewrite rule appears whose bound is instantiated from a fold's depth, at which point the parameter has more than one caller and stops being speculative.

## Trigger check log

- 2026-08-06 — not fired. ADR 0095 was reaffirmed on 2026-08-06 and admits no distributivity permission; ADR 0101 decision 5 reserves the elementary-identity permission and admits none. No second depth-instantiated rule exists. Reproduce with `grep -n 'decision_status' docs/decisions/0095-decline-a-distributivity-permission.md docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md`.
