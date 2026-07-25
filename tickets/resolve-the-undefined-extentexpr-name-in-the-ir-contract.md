---
id: resolve-the-undefined-extentexpr-name-in-the-ir-contract
title: Resolve the undefined ExtentExpr name in the IR contract
status: todo
priority: p2
dependencies: []
related: [disambiguate-operation-names-shared-across-expression-layers, disambiguate-select-across-ir-layers]
scopes: [contracts/foundation, research/shapes]
shared_scopes: []
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
