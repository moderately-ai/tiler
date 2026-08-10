---
id: accept-the-symbolic-index-coefficient-surface
title: Accept the symbolic index coefficient surface
status: done
priority: p1
dependencies: []
related: [admit-symbolic-index-expression-coefficients, promote-the-symbolic-index-profile-to-a-public-boundary, bound-a-symbolic-index-coefficient-interval-from-its-declared-extent]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, indexing]
---
## What is being accepted

The public surface [`admit-symbolic-index-expression-coefficients`](admit-symbolic-index-expression-coefficients.md) landed as a labelled draft, so a `ShapeSymbol` can be an index expression's coefficient or addend. It is implemented and tested; a tested implementation is a concrete draft, not implicit approval of its spelling, so this node parks until Tom closes it. **Only Tom closes it.**

## The exact surface

**Added** in `tiler_ir::index`:

```rust
pub enum SourcedIndexInteger { Literal(IndexInteger), Symbol(ShapeSymbol) }
impl SourcedIndexInteger {
    pub fn symbol(&self) -> …;
    pub fn as_literal(&self) -> …;
}
// with From<IndexInteger>, From<i128>, From<u64>, From<ShapeSymbol>, From<SourcedExtent>
// — the SourcedExtent conversion normalizes Static into Literal.

impl IndexRegionBuilder {
    pub fn sourced_linear_combination(
        constant: SourcedIndexInteger,
        terms: &[(SourcedIndexInteger, IndexExprId)],
    ) -> Result<IndexExprId, SymbolicExtentError>;
}
```

Added in `tiler_reference`: `UnsupportedRegionFeature::SymbolicIndexCoefficient`. That enum is `#[non_exhaustive]`, so the addition is additive.

**Widened — and this is the only widening:**

```rust
// was: pub const fn coefficient(self) -> &'a IndexInteger
pub const fn coefficient(self) -> &'a SourcedIndexInteger   // LinearTermRef
```

## What is excluded, and one exclusion is narrower than the ticket predicted

- **`IndexExprView::LinearCombination { constant }` stays `&'a IndexInteger`.** The producing ticket expected this to widen too; it did not, and the reason is a design decision worth objecting to if you disagree: **a symbolic addend is carried as the term `symbol * 1` rather than in the constant slot.** The constant slot must stay exact because literal constants reached through *any* operand fold into it; a slot holding either kind would have nowhere to fold them, so `S + 2*3` and `S + 6*1` would become two regions for one program. The surface Tom is asked to accept is therefore **smaller** than `admit-symbolic-index-expression-coefficients`'s text implies.
- `linear_combination`'s signature and error type are unchanged, so no existing caller moves. `sourced_linear_combination` sits beside it, following the crate's own `tensor`/`sourced_tensor` and `dimension`/`symbolic_dimension` convention: two authoring vocabularies, one normalized node, one total view. The alternative — widening the one constructor — would have forced edits in `tiler-compiler`, `tiler-reference`, `tiler-runtime` and `tiler-artifact`, and was rejected on that ground.
- No signed symbolic variant: `-B*p` is written `B*(-p)`.
- No expression-tree extent: `i + (S - T)` is written `i + U` with the composition held in the environment.

## The choices worth objecting to

- **Normalization declines every fold over a symbolic coefficient.** It is never merged with another term, dropped at a pinned zero, distributed over a nested sum, or unwrapped at a pinned one. The ground is that performing any of those *when* the environment happens to pin a value would make canonicalization a function of the binding, collapsing graph identity into specialized identity — the exact distinction the sourced boundary exists to keep. The cost: two symbolic terms over one operand both appear, and `S * x` remains a term even where the environment fixes `S == 1`.
- **`ExtentSources::admit` is the admission predicate, not `proves_positive`.** A coefficient may be any sign, unlike a divisor. This is the most likely place a reviewer would expect the divisor's rule to be reused, and reusing it would have been wrong.
- **Interval propagation declines rather than approximates.** A symbolic coefficient returns `None`, and the bound is retained as `InsufficientFacts` rather than `ProofResourceLimit`. A soundly-derivable bound was available and was deliberately not taken — see [`bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`](bound-a-symbolic-index-coefficient-interval-from-its-declared-extent.md), which holds that capability and is a genuine expansion rather than an oversight.

## The identity consequence

`INDEX_REGION_DOMAIN` moved `tiler.index-region.v9` → `v10`: a coefficient encodes as a tagged `SourcedIndexInteger` where v9 wrote a bare integer. Five law-chain pins and the standard Metal artifact identity, cache subject, and fixed-content byte count were recomputed on the merged tree with a superseded-values ledger entry; the envelope grew 12 bytes, one tag byte per coefficient the fixture's regions spell, which is the growth the encoding predicts rather than a layout move. No trybuild `.stderr` golden moved, so ADR 0051 compile-fail evidence is intact.

## Evidence

The producing ticket's Outcome carries the four refusals each watched failing and restored — undeclared symbol, foreign environment, past `EXTENT_PHASE_CEILING`, and a fourth proving the oracle's coefficient-versus-divisor assertion is load-bearing — the normalization decision with its reasoning, the pin table with before and after, and the gate.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the surface is in use inside `tiler-ir` and labelled a draft at each definition.

## Accepted — 2026-08-07

**Tom accepted this surface on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator, **conditional on the deferred work being captured on the board rather than left in prose.** It is: see the release below, verified `todo` and dispatchable at acceptance rather than asserted.

### What is accepted

The one widening — `LinearTermRef::coefficient` returning `&'a SourcedIndexInteger` — together with `SourcedIndexInteger`, its accessors and conversions, `IndexRegionBuilder::sourced_linear_combination`, and `tiler_reference::UnsupportedRegionFeature::SymbolicIndexCoefficient`.

**The narrower-than-drafted exclusion is accepted as the shape, not tolerated as a gap.** `IndexExprView::LinearCombination { constant }` stays `&'a IndexInteger` and a symbolic addend rides as the term `symbol * 1`. The ground stands: literal constants reached through *any* operand fold into the constant slot, so a slot admitting either kind would have nowhere to fold them, and `S + 2*3` and `S + 6*1` would become two regions for one program.

**The normalization decision is accepted with it.** A symbolic coefficient is never merged with another term, dropped at a pinned zero, distributed over a nested sum, or unwrapped at a pinned one, because performing any of those *when* the environment happens to pin a value would make canonicalization a function of the binding — collapsing graph identity into specialized identity, the distinction the sourced boundary exists to keep. The accepted cost is that two symbolic terms over one operand both appear and `S * x` remains a term where the environment fixes `S == 1`.

`ExtentSources::admit` rather than `proves_positive` remains the admission predicate, since a coefficient may be any sign.

### The identity step is accepted as landed

`tiler.index-region.v9` → `v10`, with five law-chain pins and the standard Metal artifact identity, cache subject and byte count recomputed. Those three were recomputed a second time **on the merged tree** at integration, because a sibling branch moved the same pins the same day and neither branch's values survived — the arithmetic closed exactly at 64,710 + 12 − 180 = 64,542, which is the evidence no layout moved on either side.

### The condition, discharged

[`bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`](bound-a-symbolic-index-coefficient-interval-from-its-declared-extent.md) holds the one capability this surface declined: interval propagation returns `None` for a symbolic coefficient where `ExtentSources::interval` could soundly bound it, and the enumeration fallback could use `determined()` exactly as `plan_divisors` already does for symbolic *divisors*. It was filed at integration, reads `todo`, and is dependency-ready — confirmed against `tkt ready` at acceptance rather than assumed.

That ticket carries the reason the decline is a position rather than an oversight: deriving a bound from the environment makes an expression's cached interval, and therefore *which accesses verify*, a function of the binding — the same line normalization declines to cross. It also carries the asymmetry that makes it worth asking at all, since the divisor path already reaches `determined()`, so the answer should cover divisors and coefficients together rather than only the half that prompted it.

## Correction, 2026-08-07 — the ground recorded for the decline was false

The acceptance above records that interval propagation declines on a symbolic coefficient because "deriving a bound from the environment makes an expression's cached interval, and therefore which accesses verify, a function of the binding." **That reasoning is wrong, and the ticket released to hold it refuted it rather than implementing it.**

[`bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`](bound-a-symbolic-index-coefficient-interval-from-its-declared-extent.md) established, and the coordinator verified at source before accepting the finding, that **a `ShapeEnv` holds no values.** It carries `entries: Vec<(ShapeSymbol, RootBinding)>`, constraints, guards and an identity; a `RootBinding` carries a `BindingSource`, an `AvailabilityPhase` and a `FactProvenance` — a statement of *where* a value will come from, never the value. And `encode_region` folds the environment's identity into the region's canonical bytes. So the environment is part of the program the region denotes: a fact read from it is a fact about **this region**, not about a binding, and the premise was a category error about this crate.

Two further supports, both read rather than argued: six pre-existing proof paths already read the environment (`dimension_expr` through `extent_upper_bound`, `div_mod` through `determined`, `interval_verdict`, `extents_proved_equal`, among them), so the coefficient was the sole exception rather than the rule. And the divisor carve-out that justified the asymmetry — "its value was already required to make the expression defined at all" — does not carve: admission requires `proves_positive`, not `determined`, so `d ∈ [1, 64]` admits a divisor while determining nothing, and `plan_divisors` already read strictly more than definedness ever demanded.

**What the sourced boundary actually keeps is the other operation** — writing an environment-derived *value into a node*. Normalization still refuses to fold `S * x` at `S == 1` and `as_literal` still answers `None` for a pinned symbol, so `[m]` and `[4]` remain two programs. That separation is now proved directly rather than protected by declining a sound bound.

**This does not disturb what was accepted here.** The surface, the normalization rule, and the `symbol * 1` addend spelling all stand — the acceptance was of a shape, and the correction is to a *reason* recorded beside it. What changes is that the declined capability was not a principled line but an unexamined exception, and the exception is now closed.

## Residual post-acceptance hygiene — 2026-08-10

**Fact.** Closing this node accepted the surface and discharged the bound-ticket condition; it did **not** flip the draft labels or rewrite the stale interval-decline contract prose that still read as live after acceptance.

Still present after this acceptance (and after [`bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`](bound-a-symbolic-index-coefficient-interval-from-its-declared-extent.md) landed) and **not** repaired on this node:

1. Rustdoc still labels `SourcedIndexInteger`, `IndexRegionBuilder::sourced_linear_combination`, and `LinearTermRef::coefficient` as "**Draft surface, not yet accepted**" / pending Tom — false after 2026-08-07. The `SourcedIndexInteger` draft text also still lists the `constant` field of `IndexExprView::LinearCombination` as pending; that field was never widened and the narrower exclusion was accepted as the shape.
2. Interval-decline claims remain in those rustdocs and in the assemble-path comment that "interval propagation declines on the same terms", contradicting `interval_linear` (which multiplies by `sources.interval` of the symbol) and the bound ticket Outcome.
3. [`docs/ir.md`](../docs/ir.md) coefficient paragraph still states interval propagation declines by policy and treats the coefficient half as a labelled draft pending Tom; domain narrative may still lag the live `tiler.index-region.v11` step.

This residual is product-path documentation debt (crates rustdoc + `docs/ir.md`), not a reopen of the acceptance decision. No remainder ticket id was assigned on the audit report; the flip-and-rewrite work is residual until a board carrier owns it.
