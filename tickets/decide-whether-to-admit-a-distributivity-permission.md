---
id: decide-whether-to-admit-a-distributivity-permission
title: Decide whether to admit a distributivity numerical permission
status: awaiting-decision
priority: p3
dependencies: []
related: [settle-contraction-chain-distributivity-permission, scope-einsum-contraction-support, qualify-contraction-association-reassociation-permission]
scopes: [contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, decision]
---
`settle-contraction-chain-distributivity-permission` derived that regrouping a tensor-contraction chain consumes **distributivity** — exchanging a product of a sum for a sum of products — which is independent of reassociation and operand permutation and which no permission in `docs/numerical-semantics.md` grants. That derivation follows from the contract's own definitions and is settled. What it deliberately did not settle is a product choice, stated here as two atomic questions for Tom.

1. **Whether to admit a distributivity permission at all.** Admitting it makes contraction-order exploration reachable under a relaxed contract and gives Milestone 6's first bullet a subject. Declining it makes contraction association permanently a normalization or planning choice over one node's declared semantics rather than a logical-exploration rewrite over a chain, and removes a numerical freedom that no other rewrite in the corpus currently needs. Declining is not a gap: nothing in the compiler can express a contract permitting reassociation today either, since `NumericalPermission` in `crates/tiler-ir/src/schedule/numerics.rs` has exactly one variant, `Forbidden`.

2. **If admitted, whether one permission covers both directions of the identity.** Factoring `sum of (x * c)` into `(sum of x) * c` and expanding it back have the same algebraic justification but different structural preconditions and different error behaviour. ADR 0014 split reassociation from permutation because some combiners have one capability and not the other; whether an analogous asymmetry exists here has not been established and would be the evidence that justifies cutting the dimension in two.

Both questions are downstream of Q-SEM-015's semantic half. The Milestone 6 framing in `docs/roadmap.md` already reserves two contraction choices for Tom — one keyed family versus fixed-arity keys, and whether a semantic contraction node may consume more than two operands — and this is a third that belongs in the same decision record. Note that the derivation holds under either answer to the multi-operand question, so this does not depend on it.

**Trigger:** the accepted decision that admits a tensor-contraction family, or any earlier proposal to register a numerical contract that permits reassociation. Until one of those, `docs/numerical-semantics.md` names the dimension, admits no permission for it, and rejects the rewrite explicitly.

Closing this needs an ADR in `docs/decisions/`, which is why this ticket claims `contracts/decisions` alongside `contracts/numerics`.
