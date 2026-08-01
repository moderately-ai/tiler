---
id: carry-a-sourced-shape-on-semantic-values
title: Carry a sourced shape on semantic values instead of a fixed shape
status: todo
priority: p1
dependencies: [relocate-the-sourced-extent-vocabulary-to-the-shape-module]
related: [carry-symbolic-extents-into-the-semantic-program]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, semantic-graph, api]
---
## User-visible outcome

A semantic value's shape may name a declared `ShapeEnv` symbol, so a program whose extents are bound at run time is constructible, verifiable, and inspectable through one total view.

## Why this exists

**Fact.** `ValueFact` and `ValueDefinition` hold a `Shape`, and `SemanticProgramBuilder::input`/`input_resolved` take one by value; `SemanticProgram::shape` returns `Result<&Shape, HandleError>`. Reproduce with `grep -n "pub fn shape" -A 3 crates/tiler-ir/src/semantic/program.rs`.

**Fact.** The accepted contract already admits symbolic semantic extents: "Each axis extent may be a static integer or a scoped symbolic expression evaluated later" ([the shape environment contract](../docs/research/shapes/shape-environment-contract.md)), and `docs/ir.md` records that completing the static profile "will not complete the symbolic contract above".

## Implementation keys

- Take the environment at construction — `SemanticProgramBuilder::try_standard_with_shape_environment(Arc<ShapeEnv>)` beside `try_standard()` — with no setter. The index layer's own decision record found that a repeatable setter's stated invariant was not held by its body, and a public one is a defect a consumer can reach.
- Add `input_sourced` and `input_resolved_sourced` beside the existing constructors. Do not add an environment argument to every static call site.
- Replace `SemanticProgram::shape`'s return with the total `&SourcedShape` view rather than adding an optional symbolic accessor beside the fixed one. The paired-accessor shape is the defect the index promotion removed, and it fails silently when a third source kind arrives.
- Expose the resolving environment as `SemanticProgram::extent_sources() -> Option<&ExtentSources>`, matching `VerifiedIndexRegion::extent_sources`: a symbol means nothing without the environment that declares it.
- A symbolic extent's phase ceiling is `EXTENT_PHASE_CEILING`. An input whose binding arrives later is refused at the constructor, not at build.
- Identity is out of scope and belongs to `fold-the-shape-environment-into-semantic-identity`; this ticket must not change canonical bytes. If that is impossible without a temporary inconsistency, say so and land the two together rather than shipping an unkeyed symbolic program.

## Evidence

- A symbolic program builds; the same program with a foreign symbol is refused as undeclared; the same program with a post-ceiling binding is refused as too late; each refusal paired with the accepted neighbour that differs only in the refused fact.
- A wholly literal program still returns `SourcedShape::Static` and its `as_static` borrow, so the normalization invariant holds at this layer too.
- Every new check perturbed once and observed failing before restoration.

## Public boundary

The builder constructors, `SemanticProgram::shape`'s return type, and `extent_sources` are all ADR 0075 items. `shape` changing its return type is the consequential one, because it moves every existing caller.
