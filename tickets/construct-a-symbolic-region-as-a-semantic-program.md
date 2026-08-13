---
id: construct-a-symbolic-region-as-a-semantic-program
title: Construct a symbolic inline region as a real semantic program
status: in-progress
priority: p1
dependencies: [resolve-semantic-shape-inference-over-symbolic-extents, carry-a-sourced-shape-on-semantic-values, seal-and-validate-sourced-shapes-at-semantic-inference-boundaries, narrow-symbolic-inference-and-restore-host-owned-refusals]
related: [carry-symbolic-extents-into-the-semantic-program, prototype-inline-proc-macro-frontend, seal-and-validate-sourced-shapes-at-semantic-inference-boundaries, narrow-symbolic-inference-and-restore-host-owned-refusals]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, shapes, inline-dx]
claimed_from: todo
assignee: worker-symbolic-region
lease_expires_at: 1786629707
---
## User-visible outcome

`ProgramEvidence::DeferredSymbolicExtent` is removed as the frontend's answer to a symbolic region. When the governed registry admits the region's operations (the approved elementwise region), the region is constructed and verified as a real `SemanticProgram` through that registry, exactly as a fully literal region already is. Operations that still decline symbolic operands surface as a typed `RegionError::Program` (or equivalent) rather than silent deferral — not every region that declares `sym n` becomes `Verified`.

## Why this exists

**Fact.** `verify_public_logical_program` in `crates/tiler-macros/src/region.rs` returns `ProgramEvidence::DeferredSymbolicExtent` without constructing a program as soon as any declared operand or result extent is symbolic (`return Ok(ProgramEvidence::DeferredSymbolicExtent)` on both paths after `literal_extents` fails). Its documentation still states the reason is the fixed-extent vocabulary rather than a missing value.

**Fact.** The expansion already holds a verified `ShapeEnv` — `crates/tiler-macros/src/binding.rs` builds one at `let mut environment = ShapeEnvBuilder::new();`, declaring each symbol in the fixed scope `tiler.inline-region.v1` and binding it to `BindingSource::InputDimension` at `LiveDevicePreflight`. After [`carry-a-sourced-shape-on-semantic-values`](carry-a-sourced-shape-on-semantic-values.md), `try_standard_with_shape_environment` / `input_sourced` already exist on the semantic builder; what remains is handing `BoundRegion`'s environment into that builder and building `SourcedExtent`s from `DeclaredAxis`.

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

`ProgramEvidence`'s variant set is crate-private, so nothing is published here. What is observable is that a previously deferred region now expands when the registry admits it; the `deliver` gate is separate and belongs to [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md).

## Fact audit — 2026-08-10 at base `c99ac54950f2`

- Ordinal citation `region.rs:568` had drifted (that line is inside `refuse_undeclared_symbols`); the durable anchors are the two `return Ok(ProgramEvidence::DeferredSymbolicExtent)` arms in `verify_public_logical_program`.
- The input-side vocabulary gap is closed by carry (`input_sourced` / `try_standard_with_shape_environment`); the remaining gap is the frontend's unmigrated construction path that still uses `SemanticProgramBuilder::try_standard()` and never hands `BoundRegion`'s environment through.
- The original User-visible outcome overclaimed universal `Verified` for every `sym n` region; strict serial sum and other non-elementwise families still decline symbolic operands via `static_operand_shape`. After deferral removal those surface as typed program refusals unless a later family-admission ticket widens them.
