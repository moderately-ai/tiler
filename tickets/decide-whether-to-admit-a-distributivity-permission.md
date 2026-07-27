---
id: decide-whether-to-admit-a-distributivity-permission
title: Decide whether to admit a distributivity numerical permission
status: deferred
priority: p3
dependencies: []
related: [settle-contraction-chain-distributivity-permission, scope-einsum-contraction-support, qualify-contraction-association-reassociation-permission]
scopes: [contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, decision]
---
`settle-contraction-chain-distributivity-permission` derived that regrouping a tensor-contraction chain consumes **distributivity** — exchanging a product of a sum for a sum of products — which is independent of reassociation and operand permutation and which no permission in `docs/numerical-semantics.md` grants. That derivation follows from the contract's own definitions and is settled. This ticket owns one atomic product choice.

**Whether to admit a distributivity permission at all.** Admitting it makes
contraction-order exploration reachable under a relaxed contract. Declining it
makes contraction association a planning choice within one semantic
contraction rather than a logical rewrite over a chain. Declining is not a gap:
no admitted contract currently grants this freedom.

If admitted,
`decide-whether-distributivity-directions-share-one-permission` owns the
dependent question of whether factoring and expansion share a permission.

Both questions are downstream of Q-SEM-015's semantic half. The Milestone 6 framing in `docs/roadmap.md` already reserves two contraction choices for Tom — one keyed family versus fixed-arity keys, and whether a semantic contraction node may consume more than two operands — and this is a third that belongs in the same decision record. Note that the derivation holds under either answer to the multi-operand question, so this does not depend on it.

**Trigger:** the accepted decision that admits a tensor-contraction family, or any earlier proposal to register a numerical contract that permits reassociation. Until one of those, `docs/numerical-semantics.md` names the dimension, admits no permission for it, and rejects the rewrite explicitly.

Closing this needs an ADR in `docs/decisions/`, which is why this ticket claims `contracts/decisions` alongside `contracts/numerics`.
