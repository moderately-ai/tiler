---
id: resolve-the-undefined-extentexpr-name-in-the-ir-contract
title: Resolve the undefined ExtentExpr name in the IR contract
status: done
priority: p2
dependencies: []
related: [disambiguate-operation-names-shared-across-expression-layers, disambiguate-select-across-ir-layers]
scopes: [contracts/foundation, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, glossary, coherence]
---
`docs/ir.md:876` lists `ExtentExpr` in the Layer 2 core-concepts block, beside `IndexExpr` and `ScalarOperation`/`ScalarValue`. Nothing defines it, and nothing else in the corpus uses it.

**Fact — read at `b7d9f14`.** `grep -rn ExtentExpr docs/` returns exactly one line, `docs/ir.md:876`. `grep -rn ExtentExpr crates/` returns nothing. The corresponding search for the name the rest of the corpus uses, `grep -rln ShapeExpr docs/`, returns six files: `docs/decisions/0008-typed-root-bindings.md`, `docs/decisions/0068-co-locate-abi-expressions-with-executable-program-ir.md`, `docs/glossary.md`, `docs/research/program-planning/abi-expression-ownership.md`, `docs/research/shapes/constraint-prover-boundary.md`, and `docs/research/shapes/shape-environment-contract.md`. `grep -n ShapeExpr docs/ir.md` returns nothing, so the normative IR contract uses neither the accepted name nor any definition of the one it does use.

**Fact — `ShapeExpr` is fixed by accepted decisions.** ADR 0008 states that "`ShapeExpr` references scoped extent symbols" and that `ShapeEnv` separately maps every symbol. The 2026-07-19 accepted decision in `docs/research/shapes/constraint-prover-boundary.md:318` makes `ShapeExpr` and `AbiExpr` distinct newtyped domains, and `:109` fixes mathematical-integer semantics for semantic `ShapeExpr`. Whatever `ExtentExpr` turns out to be, `ShapeExpr` is not available for renaming.

**Inference, not fact — the two are probably one construct.** Both name an expression over extents, and `docs/ir.md` discusses the shared typed `ShapeEnv` in its "Constraint and proof context" section immediately before Layer 2, so listing the shape-expression type among Layer 2's inputs would be coherent. This was deliberately not asserted while filing: the alternative — that `ExtentExpr` is a Layer 2 construct genuinely distinct from the `ShapeEnv`'s `ShapeExpr`, in the way `IndexExpr` is — has not been ruled out by reading, and the two readings imply opposite fixes.

**Why this is not the sibling ticket's defect.** `disambiguate-operation-names-shared-across-expression-layers` closed the class where one name has several definitions. This is the inverse and rarer failure: a name in a normative contract with no definition anywhere, next to a well-defined name for what may be the same thing. A reader meeting `ExtentExpr` has nothing to look up, so no glossary row can fix it without first settling which construct it denotes.

**What closes this.** Read `docs/ir.md`'s Layer 2 and constraint-context sections and `docs/research/shapes/shape-environment-contract.md` far enough to decide whether `ExtentExpr` denotes `ShapeExpr`. If it does, replace the spelling in `docs/ir.md` — do not add a second name for an ADR-fixed construct. If it denotes something distinct, define it in `docs/ir.md` and give it a glossary row stating how it differs from both `ShapeExpr` and `IndexExpr`. Either outcome must leave exactly one name for each construct and no undefined name in a core-concepts list.

`docs/ir.md` is `contracts/foundation`; confirming the construct identity needs `research/shapes`, which is why both scopes are declared.

## Outcome

`ExtentExpr` denotes `ShapeExpr`. The spelling is replaced everywhere it occurred and `ShapeExpr` is defined in `docs/ir.md` itself, so the corpus now has exactly one name for the construct and the core-concepts list names nothing undefined. Nothing under `docs/research/shapes/` needed changing: the `research/shapes` scope was declared to read the accepted decisions, and reading them is all it was needed for.

### What settled it, and why the filing ticket's alternative is ruled out

The filing ticket left open whether `ExtentExpr` might be a Layer 2 construct genuinely distinct from `ShapeExpr`, the way `IndexExpr` is. It is not, and `docs/ir.md` already said so 193 lines below the block that spelled it `ExtentExpr`.

**Fact — the contract identifies Layer 2's extent expression as the shape environment's own, in its own words.** `docs/ir.md`'s proposed first static index profile says a future symbolic profile "can return `None` and expose its **`ShapeEnv` expression** through an additive borrowed view". Layer 2's symbolic extents are therefore expressions of the shared `ShapeEnv`, which the section immediately above the core-concepts block already states semantic and index lowering share. Reproduce: `grep -n "ShapeEnv expression" docs/ir.md`.

**Fact — the accepted decisions enumerate two expression domains, not three.** [The shape environment contract](../docs/research/shapes/shape-environment-contract.md) accepts on 2026-07-19 that "`ShapeExpr` and runtime/artifact `AbiExpr` are distinct, newtyped domain IRs" and lists each domain's admitted sources: `ShapeExpr` from scoped extent symbols, `AbiExpr` from lowered extents, strides, buffer sizes, and physical-only target properties. [`constraint-prover-boundary.md:318`](../docs/research/shapes/constraint-prover-boundary.md) accepts the same partition. A third extent-expression domain living in Layer 2 would have had to appear in that enumeration, and does not. **Inference:** Layer 2's extents are sourced from the shared `ShapeEnv`, so they fall on the `ShapeExpr` side of a two-element partition, and there is no third place for them to be.

**Fact — `docs/ir.md` was already using both spellings, seven lines apart.** Its Layer 1 section read "Its extent expressions reference scoped symbols" and then, in the next paragraph, "without making target queries into tensor operations or shape-expression primitives". The first is a paraphrase of ADR 0008's accepted "shape expressions reference scoped extent symbols" — same sentence, second spelling. One document using two names for one construct within one section is what makes this a vocabulary defect rather than a missing definition.

### What changed

This was a governed vocabulary with two spellings, so every sibling site was swept rather than only the reported one. Five sites; `grep -rni "extent.expression" docs/ crates/ spikes/ README.md` and `grep -rn ExtentExpr` over the same roots now both return nothing.

- `docs/ir.md`, Layer 2 core concepts: `ExtentExpr` → `ShapeExpr`. This is the reported site.
- `docs/ir.md`, Layer 1: "Its extent expressions reference scoped symbols" → "Its shape expressions reference scoped extent symbols", which is ADR 0008's accepted wording rather than a paraphrase of it. The old text also said "scoped symbols" where the decision says "scoped extent symbols".
- `docs/ir.md`, Layer 0: "introduced axes have known extent expressions" → "known shape expressions".
- `docs/ir.md`, Constraint and proof context: a new paragraph defines `ShapeExpr` where `ShapeEnv` is already introduced, citing ADR 0008 for the symbol-versus-binding split and the shape environment contract for mathematical-integer arithmetic and the `AbiExpr` domain separation. This is what makes the name in the block below it defined rather than merely consistent.
- `docs/glossary.md`: the "Extent expression" row — "Static extent or expression over runtime scalar parameters", which named neither scoped extent symbols nor any accepted decision — is replaced by a "Shape expression" row beside "Shape environment", carrying the accepted decisions, the layers the construct spans, and its maturity.
- `docs/prior-art/logical-graphs-and-schedules.md`: "another ordinary extent expression" describes Tiler's own construct in a comparison with JAX, PyTorch, and the MLIR Shape dialect, so it takes the accepted spelling too.

### Wrong versus improvable

The reported site was **wrong**: a normative core-concepts list named a construct that nothing in the corpus defined, so a reader had nothing to look up. The Layer 0, Layer 1, and prior-art sites were also **wrong** in the same way once the identity was settled — they named an ADR-fixed construct by a name the ADR does not use, which is the defect that made the reported site unresolvable in the first place. The glossary row was **wrong** rather than merely thin: it defined the construct without scoped extent symbols, which is exactly the split ADR 0008 accepts, and it indexed the wrong name.

### No second name introduced

`ShapeExpr` is not renamed, redefined, or given a synonym — ADR 0008 and two accepted 2026-07-19 decisions fix it, and this change only makes `docs/ir.md` and the glossary use it. No "formerly extent expression" note is left in either document: after the sweep there is no surviving occurrence for a reader to reconcile, and a reconciliation sentence would reintroduce the second name in prose. That history is in this ticket and the commit.

`uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass.
