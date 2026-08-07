---
id: accept-the-symbolic-index-coefficient-surface
title: Accept the symbolic index coefficient surface
status: awaiting-decision
priority: p1
dependencies: []
related: [admit-symbolic-index-expression-coefficients, promote-the-symbolic-index-profile-to-a-public-boundary]
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
