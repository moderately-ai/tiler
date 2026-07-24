---
id: record-distributivity-in-the-navigation-contracts
title: Record distributivity in the navigation contracts
status: done
priority: p2
dependencies: []
related: [settle-contraction-chain-distributivity-permission, scope-einsum-contraction-support, decide-whether-to-admit-a-distributivity-permission]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics]
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

## Outcome

The two navigation contracts now carry the settled position, and three stale verbatim quotations found in the same section were repaired.

**Fact — the ADR had not landed, so the navigation text points at the normative section.** `grep -rli distributiv docs/decisions/` returns nothing at `412ceae` and the highest accepted ADR is 0077. Every link added here therefore targets [`docs/numerical-semantics.md#distributivity-is-outside-the-order-contract`](../docs/numerical-semantics.md), which is the normative owner, and no edit restates its derivation. `record-distributivity-dimension-adr` now tracks the owed ADR, which nothing tracked before: `settle-contraction-chain-distributivity-permission` held `contracts/numerics` and `contracts/optimizer` and never claimed `contracts/decisions`, and `decide-whether-to-admit-a-distributivity-permission` is `awaiting-decision` on a different question — whether to admit a permission at all — so it cannot carry the settled derivation into the ADR index. When that ADR lands, the four links added here should point at it instead.

**The `docs/roadmap.md` contraction-order paragraph.** It inferred that the regroup "is a floating-point reassociation in ADR 0014's sense", and concluded illegality from the strict `f32` contract's `Forbidden` reassociation — which implied that a reassociation-permitting contract would admit the rewrite, the exact inference the numerical contract forbids. It now names all three permissions, states that reassociation is necessary and never sufficient, and grounds the rejection in the missing distributivity dimension rather than in the two incidental reachability limits. Its closing sentence was independently stale: it assigned the optimizer-contract qualifier to `qualify-contraction-association-reassociation-permission`, which is `done`, and `docs/compiler/optimizer.md` now carries the full three-permission rule.

**Q-SEM-015 in `docs/open-questions.md`.** Its trigger reserved two choices for Tom; it now reserves three, attributing the first two to the Milestone 6 framing and the third — whether to admit a distributivity permission at all — to `decide-whether-to-admit-a-distributivity-permission`, with the independence note that the derivation holds under either answer to the multi-operand choice. The framing's own "Decisions reserved for Tom" list keeps its two and gains a pointer to the third, so the roadmap and the question index agree on the count without the roadmap claiming to frame a question it does not.

**Two further statements in the same framing asserted the settled position wrongly and were corrected.** Reserved decision 2 said the multi-operand answer "makes association a normalization and planning choice over one node" and that the choice "determines whether Milestone 6's first bullet is reachable under a strict contract at all" — both foreclosed by the derivation, which holds that a flat multi-operand node's contributors are triple products no binary association ever forms, so recovering an association from it consumes distributivity too. The support-matrix row's paraphrase of the optimizer's reserved equivalence group carried no legality qualifier and now records that no expressible numerical policy admits it.

**Measurement — three verbatim quotations of `docs/compiler/optimizer.md` no longer existed in it,** each checked by `grep -n -F` against the current file rather than inferred: `"choose alternative contraction associations"`, `"alternative contraction associations for future multi-input einsum"`, and `"for future multi-input einsum"`. All three rotted from the same day's merges. They fell inside `contracts/navigation` and were repaired here; `detect-stale-cross-document-quotations` records the class, since they were found only because this ticket re-verified quoted strings while working nearby.

Checked and clear: the reduction-obligation paragraph's "holds reassociation and permutation as independent permissions" is correct unchanged, because distributivity is deliberately outside the order contract and listing it among a reduction's order obligations would misplace it. `docs/design-map.md`, `docs/status.md`, `docs/README.md`, `README.md`, and `spikes/README.md` contain no contraction, reassociation, distributivity, or einsum statement. Neither edited file contains a generated catalog block (`grep -n "BEGIN GENERATED"` returns nothing in either), and `scripts/docs.py render` reported no changes beyond the prose edits.
