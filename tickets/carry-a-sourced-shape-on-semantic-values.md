---
id: carry-a-sourced-shape-on-semantic-values
title: Carry a sourced shape on semantic values instead of a fixed shape
status: todo
priority: p1
dependencies: [relocate-the-sourced-extent-vocabulary-to-the-shape-module]
related: [carry-symbolic-extents-into-the-semantic-program]
scopes: [implementation/ir, implementation/compiler, implementation/reference, implementation/artifact, implementation/frontend, contracts/foundation]
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

## Not started 2026-08-07 — dispatched, measured, and stopped; this ticket is not deliverable as scoped

A worker took this on 2026-08-07, **committed nothing** (`git diff` against its base empty, `cargo check --workspace` exit 0), and stopped on three of its four stop conditions after measuring each rather than inferring it. The findings below are the repair this ticket needs before it is dispatched again.

### 1. The scope set was wrong by four crates, and the measurement method matters

The ticket declared `implementation/ir` alone. The change moves **45 call sites across 5 crates** — `tiler-compiler` 24, `tiler-ir` 11, `tiler-reference` 7, `tiler-artifact` 2, `tiler-macros` 1 — and the compiler sites are real source (`request.rs` ×11, `normalize.rs` ×8, `program.rs` ×2, `region.rs`, `pipeline/conformance.rs`), not fixtures.

**How that was measured is worth reusing.** A plain rename reports only 8 in-crate errors, because the build fails inside `tiler-ir` and never reaches dependents — misleading, and the worker tried it first and discarded it as unsound. Attaching `#[deprecated]` instead warns without breaking the build, so dependents still compile and the whole population appears in one `cargo check --workspace --all-targets`. **Scopes are now corrected** to `implementation/ir`, `implementation/compiler`, `implementation/reference`, `implementation/artifact`, `implementation/frontend`, `contracts/foundation`. The precedent is decisive: `relocate-the-sourced-extent-vocabulary-to-the-shape-module` declared three scopes for a *14*-site move, so crate-graph reverse-dependency expansion does not cover dependents for scope purposes.

### 2. A stale Fact, and it is why the ticket missed a public accessor

The first Fact says "`ValueFact` and `ValueDefinition` hold a `Shape`". **`ValueDefinition` does not and never has** — it is `Input { input_index } | OperationResult { operation, result_index }` (`crates/tiler-ir/src/semantic/operation.rs:1564`), unchanged since its founding commit. The type holding the field is **`ValueData`** (`:1593`), and its public reader is **`ValueRef::shape`** (`:1637`) — a second public accessor this ticket's Public boundary section never names and which must widen identically. `ValueFact::shape` (`:1018`) is the one the ticket does name. Verified independently by the coordinator.

The misnaming is not cosmetic: it is the direct cause of the missed accessor.

### 3. Two of this ticket's own requirements are jointly unsatisfiable, and the graph proves it

It requires **both** "must not change canonical bytes" and "a symbolic program builds". `encode_shape` (`semantic/identity.rs:384`) writes rank then eight untagged big-endian bytes per extent, and every value's shape is encoded. A symbolic extent has no encoding there; an untagged-static/tagged-symbolic hybrid would be collision-ambiguous and would still leave `ShapeEnvIdentity` unfolded, so two programs spelled identically over differently bound environments would share an identity — **exactly the unkeyed symbolic program this ticket forbids shipping.**

Delivering it needs the `v2 → v3` tagged encoding and the fifth `SemanticIdentity` subject, which are [`fold-the-shape-environment-into-semantic-identity`](fold-the-shape-environment-into-semantic-identity.md)'s stated keys — and that ticket's Evidence demands every pinned identity be recomputed, which is the opposite of this one's.

**The coordinator then tried to add the dependency edge and the engine refused it as a cycle**: `fold` already depends on `carry`. So the mutual dependency is a mechanical fact, not an argument — **neither can be dispatched first, and they must land as one unit.** This ticket's own escape clause anticipated it: "If that is impossible without a temporary inconsistency, say so and land the two together."

### 4. The pinned population, enumerated

79 pinned literals across 16 `.rs` files, but **only three move on a semantic-graph encoding change**, all in `crates/tiler-build/src/metal_plan.rs` — `ARTIFACT_IDENTITY`, `CACHE_SUBJECT`, `FIXED_CONTENT_BYTES` — because the fixture builds a `SemanticProgram` and the artifact preimage folds the graph identity, with the cache subject composing over it. `crates/tiler/src/route/tests.rs`'s `IDENTITY_DOMAIN` moves additionally **if** the artifact domain steps. The `index/law.rs` and `schedule/builder.rs` pins do **not** move despite their fixtures building semantic programs.

**Whoever takes the combined unit must recompute those three on the *merged* tree, never from its own base** — two branches moved shared pins from different bases on 2026-08-07 and neither's values survived.

## Dispatch as one unit with `fold-the-shape-environment-into-semantic-identity`

Not before the compiler and foundation scopes are free. [`resolve-semantic-shape-inference-over-symbolic-extents`](resolve-semantic-shape-inference-over-symbolic-extents.md) stays separate: keeping `ValueFact` on `Shape` means only inputs can be symbolic until it lands, which is a coherent boundary rather than a partial state.
