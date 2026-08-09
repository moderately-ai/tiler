---
id: carry-an-exact-rational-quantity-in-the-explain-fact-vocabulary
title: Carry an exact-rational quantity in the explain fact vocabulary
status: deferred
priority: p3
dependencies: []
related: [derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold, connect-certified-rounding-error-bounds-to-rewrite-permissions, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller, decide-whether-to-admit-an-elementary-identity-permission]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, explain, numerics]
---
## User-visible outcome

A rewrite refused on a quantitative bound can state the number it was refused on, so an explain record carries the stated tolerance and the derived price rather than the fact that a comparison failed.

## Why this exists

**Fact.** `FactValue` (`crates/tiler-compiler/src/explain.rs:376`) has six variants — `Count`, `Bytes`, `Threads`, `Bindings`, `Boolean`, `Identity` — and `Quantity` (`:523`) has eight, none of them rational. Every quantity the explain vocabulary can carry is an integer count of something.

**Fact.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md)'s Part 4 item 5 states what a tolerance refusal owes: "the stated tolerance, the derived price, and which of the five obligations failed — because 'refused' without the number is unactionable when the number is the whole point." Both quantities are exact rationals: `ExactRational` (`crates/tiler-ir/src/semantic/accuracy/rational.rs:175`) is what tolerances are stated in, and a rule's derived price is a rational function of the shape parameter, `u`, and `eps_exp`.

**Inference, from [the rule-object record](../docs/research/numerics/online-softmax-rule-object.md)'s Part 5.** The dimension-naming half of ADR 0101 decision 6's refusal needs no widening at all — several declined `Numerical` assessments already fit the channel, and the function and identity fit `FactValue::Identity`. The one element that does not fit is the number, and it is needed only once a rewrite is admitted far enough to have a price computed. Under a continued decline no price exists to report.

## What this ticket must produce

- Whether an exact rational belongs in `FactValue`, in `Quantity`, or in both, and what a canonical rendering of one is — the encoding enters the explain trace identity, so the spelling is an identity question rather than a formatting one.
- The renderer spelling, and the schema and renderer version steps the addition forces (`EXPLAIN_SCHEMA_VERSION`, `EXPLAIN_RENDERER_VERSION`, `crates/tiler-compiler/src/explain.rs:35-36`), executed completely with every pinned trace identity recomputed on the tree the step lands into.
- What refuses a non-canonical rational — an unreduced fraction and its reduced form must not be two identities for one number.

## Non-goals

Admitting a permission; implementing an admission rule; adding a floating-point fact value, which would put a rounding inside a value whose whole purpose is to be exact.

## Trigger

Fires when a rewrite exists whose refusal has a price to report — that is, when a quantitative admission rule reaches the compile path, which requires [`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md) and [`decide-whether-to-admit-an-elementary-identity-permission`](decide-whether-to-admit-an-elementary-identity-permission.md) both resolving in the admitting direction, **or** any other caller needing an exact rational in an explain record.

## Trigger check log

- 2026-08-06 — not fired. No rewrite computes a price: no rule consumes a numerical dimension no permission grants, and no admission rule is registered. Reproduce with `grep -n 'numerical\.' crates/tiler-compiler/src/normalize.rs`, which returns three lines — the ordered-reassociation rule's two categorical reason keys, `numerical.reassociation-forbidden` and `numerical.reassociation-permitted`, and one test assertion over the first. No reason key names a quantity.
- 2026-08-09 — **not fired.** Exact rationals exist in target-accuracy calculations, but `FactValue` has no exact-rational arm and no compile-path rewrite computes or reports a price. ADR 0095 still declines distributivity and ADR 0101 still admits no elementary-identity permission, so no quantitative refusal needs this explain value yet.
