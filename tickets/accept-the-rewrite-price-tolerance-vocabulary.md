---
id: accept-the-rewrite-price-tolerance-vocabulary
title: Accept or revise the rewrite price tolerance vocabulary
status: awaiting-decision
priority: p2
dependencies: []
related: [reconcile-which-quantity-the-admission-rule-compares-against-a-caller-s-tolerance, connect-certified-rounding-error-bounds-to-rewrite-permissions, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller, derive-how-rewrite-price-budgets-compose-across-a-program]
scopes: [contracts/numerics, contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, numerics, accuracy]
---
## User-visible outcome

Tom receives one evidence-backed packet for the tolerance vocabulary [the certified-bounds record's](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) Part 4 enumerates and does not accept, and no caller-facing tolerance surface reaches the tree before he answers.

## Why this node exists

**Fact — the vocabulary is a proposal with no acceptance node.** [The certified-bounds record's](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) Part 4 opens "Nothing in this part is proposed for self-acceptance", names [Numerical semantics](../docs/numerical-semantics.md) as the normative owner, records that `contracts/numerics` was outside the producing ticket's scopes, and states that each of its six items is a public boundary under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md). No ticket held those items. This node is that ticket, filed by [`reconcile-which-quantity-the-admission-rule-compares-against-a-caller-s-tolerance`](reconcile-which-quantity-the-admission-rule-compares-against-a-caller-s-tolerance.md) on 2026-08-06 under the same convention `ticketsplease.toml` states for `awaiting-decision`: an acceptance node is parked, never satisfies a dependent, and only Tom closes one.

**Fact — one of the six items is now answered and the answer narrows the packet rather than removing an item.** That reconciliation ticket derived which quantity the admission rule compares against a caller's tolerance: it is the rewrite's **price** `P`, and the rule offers exactly one tolerance kind. The derivation is in that record's Part 3 under *What quantity the rule compares, and why the other candidate belongs to a different authority*, argued from the rule's jurisdiction — a per-candidate feasibility answer that always retains the baseline can soundly compare only the delta between a candidate and the fold it displaces — rather than from the algebra. So item 6's metric half is answered (the compared number is a dimensionless exact rational, and absolute and ULP are category errors for it) while its spelling half is open, and item 1 gained a refutation: a price is not a `RegionAccuracyGoal`, because it has no `ObservableSelector` and its denominator is a rejected alternative plan rather than any of that type's three reference kinds.

**Fact — the absolute reading is not dropped; it is routed to an authority that is itself unaccepted.** An absolute output budget is [the region-accuracy contract's](../docs/research/numerics/region-accuracy-contract.md) `RegionAccuracyGoal` with a non-empty `DelegatedPermissions`, which that record deliberately leaves empty and gates behind a separate feasibility experiment. Whether Tiler ever offers both numbers, at two authorities, is part of this packet rather than a settled consequence of the reconciliation.

**Inference — the strongest argument against the answer belongs in the packet and is stated in the record.** A price budget grants no accuracy guarantee: a caller who states `2^-13` and receives an admitted rewrite is told only that it is at most a factor `1 + 2^-13` worse *in bound* than a fold whose bound they were never told, whose bound is loose by one to three orders of magnitude, and whose realized error the price does not bound. A caller with a model-quality experiment has evidence for an absolute number and none whatever for a price. Tom should see that counterpoint beside the recommendation, not after it.

## Ripens when

**Not yet, and the gate is a permission rather than a schedule.** The worked rewrite this vocabulary exists for is illegal under every numerical contract Tiler can express: [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) declines a distributivity permission and [ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) reserves the elementary-identity one, and [the elementary-identity record's](../docs/research/numerics/elementary-identity-rewrite-dimension.md) Part 6 four-outcome table shows only the both-admitted corner makes the fold reachable. A tolerance vocabulary is a budget on a rewrite no permission admits, so accepting one now would spell a caller-facing surface for a feature that cannot fire.

**This node therefore ripens on the first of:** a decision that admits both numerical dimensions the online-softmax fold consumes, which is [`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md)'s subject and Tom's alone; or a second rewrite family whose price a caller could spend under permissions Tiler already grants, which would make the vocabulary reachable without either decision.

## Decision boundary

This node is not research or implementation work. When it ripens, the packet presents each item with what it enables and prevents, its counterpoint, and a recommendation — one atomic question at a time. The items are the six in Part 4 as they then read, plus the two obligations the price answer creates:

1. Whether a price budget is a contract dimension or a separate object beside it — and, if separate, that it is not `RegionAccuracyGoal`, with the type mismatch as stated.
2. How a budget names the rewrites it governs, so it cannot become an ambient `fast` flag.
3. Whether the distributivity permission is admitted at all — independent of this node and prior to it.
4. How the elementary-identity freedom is named — settled by ADR 0101 for the dimension, open for the permission.
5. What a refusal reports: the stated budget, the derived price, and which obligation failed.
6. How the price is spelled so no reader takes it for an error budget, and whether it is a per-rewrite threshold or a program-spent budget — the latter depending on [`derive-how-rewrite-price-budgets-compose-across-a-program`](derive-how-rewrite-price-budgets-compose-across-a-program.md).
7. Whether an absolute rewrite tolerance is ever offered beside the price, at the region goal's authority under a non-empty delegation set.

**Any contract sentence Tom accepts lands under its own ticket rather than under this node**, because `contracts/numerics` is not a scope a research ticket may edit and the acceptance act is the amendment rather than a consequence of it.

## Closes when

Tom answers each item; any accepted surface is released to its own implementation ticket; the certified-bounds record's Part 4 stops listing unaccepted items as outstanding; and `docs/numerical-semantics.md` carries whatever sentence the answer owes, or the packet records explicitly that it owes none.
