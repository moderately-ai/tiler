---
id: record-distributivity-in-the-navigation-contracts
title: Record distributivity in the navigation contracts
status: in-progress
priority: p2
dependencies: []
related: [settle-contraction-chain-distributivity-permission, scope-einsum-contraction-support, decide-whether-to-admit-a-distributivity-permission]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics]
claimed_from: todo
assignee: agent-record-distributivity-in-the-navigation-contracts
lease_expires_at: 1784929885
---
`settle-contraction-chain-distributivity-permission` established that regrouping a tensor-contraction chain consumes **distributivity**, a numerical dimension independent of reassociation and operand permutation. It held `contracts/numerics` and `contracts/optimizer` and routed two `contracts/navigation` consequences here.

**The derivation, so this ticket does not have to re-establish it.** `(AB)C` sums contributors `T[i,k]*C[k,l]` over `k`; `A(BC)` sums `A[i,j]*U[j,l]` over `j`. The two sequences share no value, are indexed by different axes, and have different lengths, so **no common sequence exists of which both are groupings** — which means the rewrite is not a regrouping at all. `docs/numerical-semantics.md:341` defines reassociation as changing grouping "while preserving logical operand order", and the connecting identity here is distributivity, which round-to-nearest multiplication does not satisfy.

## Two edits

- **`docs/roadmap.md`** (around line 316, in the Milestone 6 framing added by `scope-einsum-contraction-support`) infers that the regroup "is a floating-point reassociation in ADR 0014's sense". That inference is now known to be incomplete rather than wrong in spirit: reassociation is necessary but not sufficient, and the missing dimension is distributivity. Correct it to name distributivity, and keep the reassociation and operand-permutation requirements — the settled rule is that **all three** must authorize the regrouping.
- **`docs/open-questions.md`** Q-SEM-015 reserves two choices for Tom — whether a contraction is one keyed family with an index-structure attribute or fixed-arity keys per shape class, and whether a contraction node may take more than two operands. Whether to admit a distributivity permission at all is a **third**, owned by `decide-whether-to-admit-a-distributivity-permission`. Add it and point at that ticket rather than restating its reasoning.

## What not to do

Do not restate the derivation in either file. `docs/numerical-semantics.md`'s new "Distributivity is outside the order contract" section is the normative owner, and duplicating it would create the second authority the documentation contract exists to prevent. Link to it.

Do not describe the rewrite as merely "unexplored" or "not yet implemented". It fails closed as a **settled legality position**: no permission Tiler can express grants distributivity, and `tiler_ir::schedule::NumericalPermission` has exactly one variant, `Forbidden`, so no expressible contract permits reassociation either. Note that the rule would still fail closed on a future compiler that accepted contractions under a contract permitting both reassociation and permutation — the two incidental reachability limits (`normalize_serial_sum` rejecting more than one input, and the single-variant permission enum) are *not* why it is illegal.

An ADR is also owed for this decision — it belongs in the `numerical-operations` catalog group beside ADR 0014 and ADR 0015 and supersedes nothing, since neither claims exhaustiveness over the dimension set. That is `contracts/decisions`, not this ticket; check whether it has landed before editing, so the navigation text points at the ADR rather than at a ticket if it exists by then.
