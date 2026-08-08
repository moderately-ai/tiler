---
id: correct-the-symbolic-coefficient-era-index-vocabulary-claims
title: Correct the symbolic-coefficient-era index vocabulary claims
status: in-progress
priority: p2
dependencies: []
related: [admit-symbolic-index-expression-coefficients, repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them, admit-a-position-selecting-slice-for-the-rotary-table]
scopes: [research/shapes, research/indexing, implementation/ir]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [docs, doc-drift, indexing]
claimed_from: todo
assignee: w-terra-index
lease_expires_at: 1786201849
---
## What is stale

`admit-symbolic-index-expression-coefficients` admitted a declared `ShapeSymbol` to a `LinearCombination`'s coefficient **and** its constant term, so a bound symbol now reaches an index expression in a *coordinate* position and not only as a `FloorDiv` or `Modulo` divisor. Verified at `cd86cac1`:

- `LinearTermData::coefficient` is `SourcedIndexInteger` (`crates/tiler-ir/src/index/model.rs:100-103`).
- `IndexRegionBuilder::sourced_linear_combination` takes a `SourcedIndexInteger` constant and `SourcedIndexInteger` coefficients (`crates/tiler-ir/src/index/builder.rs:1190-1203`).
- [`docs/ir.md`](../docs/ir.md)'s implemented-extent paragraph already states this correctly, so the contract is current and the records below disagree with it.

Several records still state the divisor-only bound as current fact, and each states it as the ground for a reserved capability rather than in passing. This was found while repointing the 2026-08-07 relocation under [`repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them`](repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them.md), which deliberately did **not** repair it: the cause is different, the correction takes a design position about what the sub-tensor selection family's symbolic-offset trigger now blocks on, and the owning ticket was `in-progress` at the time.

## Sites, each verified at `cd86cac1`

| Site | Scope | What it claims |
| --- | --- | --- |
| `docs/roadmap.md:481` | `contracts/navigation` | "`SourcedExtent` is the only carrier of a symbolic extent and appears in no variant except the `FloorDiv` and `Modulo` divisors, so a literal-offset selection is expressible today and a symbolic-offset one is not", and the reserved trigger "an `IndexNode` variant carrying a `SourcedExtent` in a coordinate position". |
| `docs/open-questions.md:275` (Q-SHAPE-006) | `contracts/navigation` | "`SourcedExtent` is the only `IndexNode` variant that carries a possibly-symbolic extent and it appears in no other position, so `t + k` for a literal `k` is expressible and `t + C` for a bound symbol is not." |
| `docs/research/shapes/sequence-extending-tensor-family.md:74-78` | `research/shapes` | Reproducible check 3's headline, "No coordinate expression carries an extent symbol", and its comment "LinearCombination's constant is a literal `IndexInteger`". |
| `docs/research/shapes/transformer-operation-and-shape-surface.md:122` | `research/shapes` | "*First, a symbol reaches an index expression only as a floor-division or modulo divisor.*", with `builder.rs:1019` and `builder.rs:974` beneath it — both ordinals also drifted, and they were deliberately left unrepointed so the citation would not read as freshly verified. |
| `docs/research/indexing/sub-tensor-selection-fusion-role.md:117` | `research/indexing` | States the reserved symbolic relation's trigger as "an `IndexNode` variant carrying a `SourcedExtent` in a coordinate position". |
| `docs/research/indexing/concatenate-fusion-role-and-lowering.md:103` | `research/indexing` | Lower confidence: its claim about `IndexNode::LinearCombination { constant: IndexInteger, … }` is still literally true of the *node*, but its `index/model.rs:97-100` and `:101-108` ranges have drifted to `:108-111` and `:112-119` and it elides `LinearTermData`. |
| `crates/tiler-ir/src/semantic/slice.rs:340` | `implementation/ir` | The `SymbolicOffsetUnsupported` doc comment carries the same divisor-only sentence. A comment is a claim about current behaviour, so it is in the population. |

Do not trust this list to be exhaustive; re-derive it, and read each file in full rather than editing around a grep hit.

## The judgement this ticket owes, and must not skip

**The literal wording survives for `SourcedExtent` and the claim it supports does not.** `SourcedExtent` really does appear in no `IndexNode` variant outside the two divisors — it is `SourcedIndexInteger` that reaches a coefficient — so a find-and-replace produces sentences that are true and still mislead. What each site actually asserts is that *a bound symbol cannot reach a coordinate position*, and that is false.

The consequential half is what it does to two reserved triggers. `docs/roadmap.md`'s sub-tensor-selection row and Q-SHAPE-006 both cite the divisor-only bound as the reason a symbolic-offset slice is not expressible, and the roadmap additionally states the row's *second* ground — "a semantic value fact carries static extents besides" — which is unaffected and still holds. So the family's refusal under `slice.selection.symbolic-offset-unsupported` stands either way; what needs re-deriving is whether [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md) still owes the *index vocabulary* it is recorded as owing, or now owes only the semantic-shape carrier. Decide that from the source, state it, and correct the trigger wording to match; do not restate the bound with a new type name and leave the trigger reading as it did.

## How to repair

Follow each file's existing convention. `docs/roadmap.md`, `docs/open-questions.md`, and the two shapes records all use dated corrections that quote the superseded sentence rather than rewriting it away; `docs/ir.md` is the current statement to align against and is **not** in this ticket's population.

## Closes when

Every site above is classified as a live claim repaired with a dated correction, or as an already-recorded correction needing none, with the classification stated per site rather than counted; the two reserved triggers are re-derived from source and their wording states what they now block on; and any further site found is fixed or reported.
