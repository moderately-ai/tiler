---
id: record-distributivity-dimension-adr
title: Record the settled distributivity dimension as an ADR
status: todo
priority: p2
dependencies: []
related: [settle-contraction-chain-distributivity-permission, decide-whether-to-admit-a-distributivity-permission, record-distributivity-in-the-navigation-contracts]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, decision]
---
`settle-contraction-chain-distributivity-permission` resolved a durable choice by derivation: distributivity is a third numerical dimension independent of reassociation and operand permutation, a tensor-contraction chain regroup consumes all three, and the rewrite fails closed under every contract Tiler can express as a settled legality position rather than a pending one. That conclusion is now normative in `docs/numerical-semantics.md` ("Distributivity is outside the order contract") and cited by `docs/compiler/optimizer.md`, `docs/compiler/fusion-and-scheduling.md`, `docs/roadmap.md`, and `docs/open-questions.md`.

No ADR records it. `grep -rli distributiv docs/decisions/` returns nothing at `412ceae`; the highest ADR is 0077.

This is a gap in decision custody rather than in the contract text. `AGENTS.md` requires that when evidence resolves a durable choice, the contract is updated *and* an ADR is added or accepted. Five documents now depend on a settled position whose only record is a normative section and a `done` ticket outcome, so a reader cannot see it in the accepted ADR index beside the decisions it sits with.

The ADR belongs in the `numerical-operations` catalog group beside ADR 0014 (reassociation versus operand permutation) and ADR 0015 (required FMA versus optional contraction). It supersedes neither: neither claims exhaustiveness over the dimension set, and ADR 0011 already holds that one permission never implies another. It should state the dimension, its independence, the contraction-chain consequence, and that no permission is admitted.

**Scope boundary.** This is distinct from `decide-whether-to-admit-a-distributivity-permission`, which is `awaiting-decision` because it needs a product choice from Tom — whether to admit a permission at all, and whether one permission covers both directions of the identity. This ticket records only what is already derived and settled, so it is not blocked on that decision. If Tom prefers one ADR carrying both the settled dimension and the admission choice, close this into that ticket instead.

Found by `record-distributivity-in-the-navigation-contracts` while checking, as instructed, whether the owed ADR had landed before pointing navigation text at it. It had not, so the navigation contracts point at `docs/numerical-semantics.md#distributivity-is-outside-the-order-contract`. Those links should be revisited when this ADR lands.
