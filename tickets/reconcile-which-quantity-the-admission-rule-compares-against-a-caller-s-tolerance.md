---
id: reconcile-which-quantity-the-admission-rule-compares-against-a-caller-s-tolerance
title: Reconcile which quantity the admission rule compares against a caller's tolerance
status: todo
priority: p3
dependencies: []
related: [separate-the-rescaling-price-from-the-observed-fold-divergence, connect-certified-rounding-error-bounds-to-rewrite-permissions]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy]
---
## User-visible outcome

The certified-bounds record states one comparison semantics for the admission rule — what a caller's tolerance is a tolerance *on* — instead of two that disagree between its Part 2 Step 4 heading and its Part 3.

## The tension, verified on the tree at 24d7dab

**Fact.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md):143 heads Step 4 with "the price, which is the quantity a caller's tolerance is compared against" — the tolerance reads as a budget on the rewrite's *extra* cost over the baseline, so `P` is the compared quantity. **Fact.** The same record's Part 3 (:233) states "the admission rule is the same three-way exact-rational comparison applied to a rewrite's *bound* and a caller's tolerance" — the tolerance reads as an absolute relative-error budget, so `B_online` is the compared quantity. **Fact.** [The tree-fold record](../docs/research/numerics/tree-fold-online-softmax-bound.md) Part 5 and [`separate-the-rescaling-price-from-the-observed-fold-divergence`](separate-the-rescaling-price-from-the-observed-fold-divergence.md)'s outcome both endorse the Part 3 reading, calling `P` presentational — but neither owns the Step 4 heading, and the separation ticket's non-goals excluded relitigating the rule's shape, which is why this is a ticket rather than a wording fix inside it.

**Why it matters.** The two readings admit different rewrites at the same stated tolerance: Step 4's worked instantiation ("a caller stating `2^-13` admits, `2^-16` refuses" against `P = 6.09e-5`) only makes sense under the price reading, while Part 3's reuse of `decide_predicate` over "a rewrite's bound" only makes sense under the absolute reading. A vocabulary offered to callers must say which number their tolerance is.

## What this must produce

The derivation of which semantics the admission rule offers — price-relative, absolute, or both as distinct named tolerances — argued from what a caller can actually state and what the rule can soundly compare, with the losing reading's worked instantiation corrected wherever it appears (Step 4's heading and its worked instantiation are the known sites; sweep the record). If the answer resolves to a vocabulary surface, draft and park it for Tom under ADR 0075; never self-accept.

## Closes when

The record carries one comparison semantics stated where the derivation produces it, every worked instantiation in the record uses it, and the tree-fold record's Part 5 cross-reference still reads true against the landed wording.
