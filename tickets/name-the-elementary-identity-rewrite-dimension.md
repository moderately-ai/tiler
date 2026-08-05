---
id: name-the-elementary-identity-rewrite-dimension
title: Name the elementary-identity rewrite dimension
status: in-progress
priority: p2
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, optimizer]
claimed_from: todo
assignee: agent-identity-dimension
lease_expires_at: 1785974497
---
## User-visible outcome

A rewrite that uses a transcendental's functional equation — `exp(a) * exp(b) = exp(a + b)`, `log(a*b) = log(a) + log(b)`, `sqrt(a)*sqrt(b) = sqrt(a*b)` — is refused with a reason naming the freedom it consumed, rather than being refused for the nearest dimension that happens to be defined or, worse, admitted because no dimension noticed.

## Why this exists

**Fact.** [Numerical semantics](../docs/numerical-semantics.md) governs the order contract — reassociation, operand permutation, contraction — and [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) adds distributivity. Every one of those dimensions is about **how contributors are combined**.

**Inference, from [the certified-bounds record's](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) Part 2.** The online-softmax rescaling rewrite is equal to the two-pass fold only by telescoping `exp(x_j - m_j) * exp(m_j - m_V) = exp(x_j - m_V)`. That step is a functional equation of the elementary function rather than an algebraic identity of the ring, and **in floating point it is false**: `fl(exp(a)) * fl(exp(b))` and `fl(exp(a+b))` differ, by an amount governed by the target's `eps_exp` and by the argument magnitudes. No order-contract dimension is about this, because it is not about combining contributors at all — it rewrites *through* the function.

**This is a gap in the dimension set rather than a missing permission inside one**, which is why it is filed separately from [`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md). Both gates must open before the softmax rewrite is legal, and they are independent.

## What the research must decide or defer

- **Whether this is one dimension or a family.** The exponential's, the logarithm's, and the square root's identities differ in their error behaviour: `exp`'s telescoping is exact over the reals and its floating-point error is governed by argument magnitudes, while `log`'s product rule carries a cancellation hazard the exponential's does not. ADR 0014's standard applies — a split needs evidence of a capability asymmetry, not an intuition of one.
- **Whether the dimension is per-operation or global**, since accuracy contracts are already per-operation and per-target under [ADR 0042](../docs/decisions/0042-use-typed-transcendental-accuracy-contracts.md).
- **How the dimension composes with the accuracy obligation the target already answers.** `assess_program_elementary_accuracy` asks whether a target realizes an operation's accuracy contract; rewriting through the identity changes *which* evaluations occur, so the obligation set changes with the rewrite. Whether that is already handled by `readmit_candidate` asking again per candidate, or needs more, is the concrete question.
- **What a refusal says.** ADR 0095 established that a rewrite consuming distributivity must reject naming the missing dimension rather than reporting a forbidden reassociation. The same discipline applies here and is the minimum outcome even if no permission is admitted.

## Non-goals

Admitting a permission (Tom's, under the same boundary as ADR 0095); implementing anything in `crates/`; enumerating every elementary function's identities exhaustively before the first one has a caller.

## Closes when

The dimension is defined in a research record with its worked counterexample, its relationship to the order contract and to the accuracy contract is stated, and either a permission is proposed for Tom or the dimension is named-and-unpermissioned in the shape ADR 0080 used for distributivity. Either outcome must leave the refusal wording specified.
