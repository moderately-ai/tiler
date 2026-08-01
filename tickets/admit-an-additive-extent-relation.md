---
id: admit-an-additive-extent-relation
title: Admit an additive extent relation so a concatenated extent is checkable
status: todo
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family]
related: [design-autoregressive-state-and-kv-cache, scope-the-sequence-extending-tensor-family, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, kv-cache, language-model]
---
## User-visible outcome

`S == C + T` becomes statable, so a decode step that binds a cache extent inconsistent with its context length **refuses** instead of verifying and returning a plausible tensor.

## Why this is not deferrable any longer

[The sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md) found the gap and deliberately filed no ticket for it, on the ground that doing so would duplicate a constraint handed to the contract work that will need it. That judgement was correct while nothing needed it. [Rung L5](../docs/research/runtime/autoregressive-state-and-kv-cache.md) is that consumer, and it makes the gap load-bearing rather than latent: the stale-state case — binding an allocation whose valid range is `[0, 13)` while binding `C = 14` — is refused by this relation and by nothing else in the stack. The artifact layer sees a well-formed extent, the bytes are inside the allocation, and the semantic layer cannot relate `S` to `C` and `T` at all.

**Fact, at commit `03a10ae`.** `ExtentRelation` admits `Equal`, `Divisible`, `NonNegativeDifference`, `Interval`, and `Factorization` over an `ExtentTerm` that is a symbol or a constant and is, in its own words, deliberately not an arbitrary expression tree. `NonNegativeDifference` is the nearest additive-looking relation and constrains a difference's sign rather than defining a sum.

## Required design and behaviour

- Decide, with the elimination stated, whether the sum becomes a new `ExtentRelation` variant, a derived form of `SourcedExtent`, or is discharged some third way. `SourcedExtent` is static-or-symbol and every symbol needs exactly one root binding, so a derived form is a change to that invariant and not a widening of it.
- Whatever is chosen must also state `C + T <= capacity`, the three-term relation a windowed append would need, or say explicitly that it does not and why.
- Keep the fragment bounded. The reason it is not an expression tree is that a prover over one is a different component; a sum of two terms is not that, and the design must say where the new boundary is.
- The relation participates in canonical identity wherever an extent relation already does.

## Closes when

A decode-shaped program binding `C`, `T`, and `S` inconsistently is refused with a typed diagnostic naming all three, that refusal has a test which fails without the change, and the consistent binding still verifies.
