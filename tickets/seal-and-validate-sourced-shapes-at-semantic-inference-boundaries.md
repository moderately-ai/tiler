---
id: seal-and-validate-sourced-shapes-at-semantic-inference-boundaries
title: Seal and validate sourced shapes at semantic inference boundaries
status: todo
priority: p0
dependencies: []
related: [resolve-semantic-shape-inference-over-symbolic-extents, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, semantic-graph, correctness, public-boundary]
---

## User-visible outcome

Every sourced semantic shape has one normalized spelling, every symbol retained on an operand or inferred result is admitted by the program's exact shape environment, and malformed or foreign symbolic data is refused without panicking or entering semantic identity.

## Source-first Fact audit — 2026-08-11, exact base `2f244dc7ff3a759d9688a482c27b48da70f37227`

**False — public construction does not preserve the documented normalization invariant.** `crates/tiler-ir/src/shape/sourced.rs`, anchors `pub enum SourcedShape` and `# The normalization invariant`, exposes both variants publicly while only the crate-private `SourcedShape::sourced` constructor normalizes. Safe out-of-crate Rust can therefore construct `SourcedShape::Sourced(vec![])` or an all-literal `Sourced` value. The empty value reaches `static_shape_of`'s `expect("a non-static sourced boundary holds at least one symbol")`; the all-literal value compares unequal to `SourcedShape::Static` while `SourcedShape::encode` writes identical canonical bytes for both. The accepted invariant is not held by the public type.

**False — inferred result symbols are not admitted against the program environment.** `FrozenSemanticRegistry::infer_operation_with_extent_sources` validates operand and result value types but does not call `ExtentSources::admit`; `SemanticProgramBuilder::apply` checks inferred rank and type only; `SemanticProgram::validate_internal` re-runs the same provider. An external `OperationInferencer` can therefore return a foreign symbolic result through `SemanticProgramBuilder::try_new(custom_registry)`, whose environment is absent, and `build` succeeds. The resulting semantic identity combines a foreign symbol spelling with the empty-environment identity.

**False — the result writer's advertised canonical-byte bound is not the bound it computes.** `OperationInferenceOutputs::try_push` charges `rank * size_of::<Extent>()`; `SourcedShape::encoded_len` includes the variable-length symbol scope and name. A provider can therefore pass the writer's estimate while exceeding the documented canonical result-fact budget.

**Verified — normalized values already have one canonical encoding.** `SourcedShape::encode` length-frames the rank and encodes each `SourcedExtent`; a static boundary and its normalized sourced input deliberately encode the same logical bytes. The repair must preserve bytes for the admitted normalized population and remove the duplicate Rust spelling rather than re-encode it.

These findings repair the safety premise without changing the valid built-in elementwise equality rule or this ticket's purpose.

## Work

- Make `SourcedShape` opaque to direct variant construction while preserving total read-only `rank`, `as_static`, `extents`, `without_axes`, `Display`, and equality views. All constructors must normalize all-literal vectors to the static representation and reject an unrepresentable rank before a value exists.
- Add one host-owned validation path at the semantic registry boundary. It validates normalization/rank and calls `ExtentSources::admit` for every operand and result extent. A symbolic value with no environment or a symbol foreign to the supplied environment is a typed refusal before graph mutation.
- Apply the same validation during internal program verification so a future constructor or provider path cannot bypass the insertion check.
- Charge `OperationInferenceOutputs` from the exact canonical length of each retained fact, including symbol scope and name bytes. Do not approximate sourced shapes by rank.
- Preserve canonical bytes and identity domains for every currently admitted normalized shape. If implementation makes that impossible, stop and derive the identity migration rather than silently rebaseline.
- Correct the stale `SourcedShape::static_encoded_len` documentation link while the owning file is open.

## Evidence

- An all-literal sourced input has only the normalized static spelling; an empty/forged sourced representation is unconstructible from safe public Rust.
- A public no-environment registry call refuses a symbolic operand before invoking its provider.
- A custom provider returning a foreign or no-environment symbol is refused before the result enters the graph; the paired static result is admitted.
- Exact result-byte accounting rejects an oversized symbol scope/name population at the documented boundary while the neighbouring in-budget value succeeds.
- Existing normalized static and symbolic canonical bytes and semantic identity pins are unchanged.
- Perturb the opacity/normalization, operand admission, result admission, internal revalidation, and exact-byte charge independently with assertions unchanged; retain each failure text.

## Public boundary and stop conditions

Making a public enum opaque is source-breaking even in this pre-production workspace. Tom approved this narrow healing direction on 2026-08-11 after the safe-Rust panic, identity alias, and foreign-result probe were presented. The exact revised constructor/read surface remains a labelled draft until its implementation diff is reviewed.

Do not add a public custom-registry-plus-environment constructor, a provider-selectable symbolic policy, a compatibility representation, or a silent fallback. Those belong to `design-an-explicit-symbolic-inference-policy-for-external-providers`. Stop if normalized existing subjects need new canonical bytes or if a second environment authority appears.

## Closes when

The malformed-shape panic and duplicate spelling are unrepresentable, every operand/result symbol is admitted against the exact environment before retention, exact byte accounting is load-bearing, identity remains coherent, and the revised public shape surface has independent review.
