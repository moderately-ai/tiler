---
id: decide-whether-to-admit-a-distributivity-permission
title: Decide whether to admit a distributivity numerical permission
status: todo
priority: p3
dependencies: []
related: [settle-contraction-chain-distributivity-permission, scope-einsum-contraction-support, qualify-contraction-association-reassociation-permission, decide-whether-distributivity-directions-share-one-permission, decide-whether-a-contraction-may-consume-more-than-two-operands]
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

## Both trigger clauses fired, and the decision followed (2026-08-01)

**Fact — the trigger above fired on both of its clauses, and the ticket stayed `deferred` regardless.** The first clause, "the accepted decision that admits a tensor-contraction family": [`admit-the-contraction-semantic-profile`](admit-the-contraction-semantic-profile.md) is `done` and `StandardSemantics` registers `tiler::strict-tensor-contraction-f32@1` under [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md), recorded at `docs/roadmap.md:421`. The second, "any earlier proposal to register a numerical contract that permits reassociation": [`admit-a-reassociating-contract-without-contraction`](admit-a-reassociating-contract-without-contraction.md) is `done`. Meanwhile [`decide-whether-distributivity-directions-share-one-permission`](decide-whether-distributivity-directions-share-one-permission.md) sat `deferred` behind it, so a fired trigger held a dependent parked as well. Nothing owned noticing this; [`sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired`](sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired.md) now does, and carries this as its worked example.

**Decision — Tom declined, 2026-08-01, relayed by the coordinator.** No distributivity permission is admitted. `docs/numerical-semantics.md` continues to name the dimension, admit no permission for it, and reject contraction-chain regrouping explicitly — which, as the body above says, is a settled legality position rather than a gap: contraction association remains a planning choice within one semantic contraction rather than a logical rewrite over a chain.

**Reopening trigger:** the first workload whose natural spelling is a directly regroupable contraction chain — one where the regrouping that consumes distributivity is what the workload asks for, rather than one an optimizer might speculatively want. A workload that merely *contains* a contraction chain does not fire it; the chain must be one whose profitable form requires exchanging a product of a sum for a sum of products.

**Why this is `todo` rather than parked or `done`.** The decision is taken, so `awaiting-decision` would misreport this as still needing Tom — the misreport the workflow configuration's own note exists to prevent — and `deferred` is now simply false. The remaining work is dispatchable and specific: write the decline into an accepted ADR under `docs/decisions/`, correct the reservation sites that describe this as an open choice (`docs/open-questions.md:301` and the Milestone 6 framing in `docs/roadmap.md`), and close [`decide-whether-distributivity-directions-share-one-permission`](decide-whether-distributivity-directions-share-one-permission.md), whose question does not arise under a decline. It is not `done` until that record lands, because a decision recorded only in a ticket is not a decision recorded.
