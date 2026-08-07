---
id: construct-a-symbolic-region-as-a-semantic-program
title: Construct a symbolic inline region as a real semantic program
status: todo
priority: p1
dependencies: [resolve-semantic-shape-inference-over-symbolic-extents, carry-a-sourced-shape-on-semantic-values]
related: [carry-symbolic-extents-into-the-semantic-program, prototype-inline-proc-macro-frontend]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, shapes, inline-dx]
---
## User-visible outcome

`ProgramEvidence::DeferredSymbolicExtent` is removed, because a region declaring `sym n` is constructed and verified as a real `SemanticProgram` through the governed registry, exactly as a fully literal region already is.

## Why this exists

**Fact.** `crates/tiler-macros/src/region.rs:568` returns `ProgramEvidence::DeferredSymbolicExtent` without constructing anything as soon as any declared extent is symbolic, and its documentation states the reason is the fixed-extent vocabulary rather than a missing value.

**Fact.** The expansion already holds a verified `ShapeEnv` — `crates/tiler-macros/src/binding.rs:466` builds one, declaring each symbol in the fixed scope `tiler.inline-region.v1` and binding it to `BindingSource::InputDimension` at `LiveDevicePreflight`. What was missing was somewhere to put it.

## Implementation keys

- Hand `BoundRegion`'s environment to the semantic builder rather than constructing a second one. The frontend's environment is already the authority for symbol identity, scope, and binding source, and a second would give one region two `ShapeEnvIdentity` values.
- Build the operand shapes as `SourcedShape` directly from `DeclaredAxis`, so a literal axis and a symbolic axis take one path. `SourcedShape`'s normalization means a wholly literal region still produces the `Static` arm and the existing behaviour is unchanged by construction rather than by a branch.
- Keep the derived-versus-inferred check. The registry stays the authority over a result's shape; the comparison becomes a `SourcedShape` comparison, and `ResultShapeDisagreement` keeps rendering both sides in the region's own spelling.
- The frontend's `elementwise_axes` rule and the registry's rule must not both remain live as independent authorities. Once the registry can decide symbolic operands, state explicitly which one survives and why; the current duplication was justified precisely by the registry being unable to see the symbolic case.
- The emitted `RegionFacts` and the runtime unification contract are unchanged. This ticket adds an expansion-time program; it changes nothing a consumer's binary does.

## Evidence

- The approved region `sym n; in a: f32[n], b: f32[n], c: f32[n]; out (a * b) + c` produces `ProgramEvidence::Verified` carrying a program whose three inputs and one output are `SourcedShape::Sourced([Symbol(n)])`.
- Two textually identical symbolic regions produce equal semantic identity; the same region with a differently spelled symbol does not.
- `in a: f32[n], b: f32[m]` is still refused, and the refusal now comes from the registry rather than from the frontend's restatement of its rule.
- Every existing compile-fail golden either unchanged or rebaselined with the diff explained.
- Each new check perturbed once and observed failing.

## Public boundary

`ProgramEvidence`'s variant set is crate-private, so nothing is published here. What is observable is that a previously refused region now expands; the `deliver` gate is separate and belongs to ticket 7.
