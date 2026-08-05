---
id: admit-the-registered-unary-families-at-the-compiler-request-boundary
title: Admit the registered unary families at the compiler request boundary
status: in-progress
priority: p1
dependencies: [admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary]
related: [admit-the-silu-activation-family, admit-the-reindex-and-broadcast-operation-families, admit-the-rms-normalization-family, admit-the-softmax-family]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, optimizer]
claimed_from: todo
assignee: agent-unary-families
lease_expires_at: 1785893513
---
## User-visible outcome

A program stating `tiler::silu-f32@1`, `tiler::reindex-f32@1`, or `tiler::broadcast-f32@1` reaches the optimizer, instead of refusing at the request boundary under `operation-set` despite each family having registered semantics *and* a registered index-access lowering capability.

## Why this exists, and why it is not the recognizer's to fix alone

**Fact — the capability exists and the boundary cannot reach it.** `governed_index_access_capabilities` registers eight capabilities, and three of them — `silu-f32`, `reindex-f32`, `broadcast-f32` — name families no recognized program can contain. `crates/tiler-compiler/src/lowering.rs`'s `resolve_lowering` would resolve each of them for a member the recognizer produced; the recognizer never produces one.

**Fact — the vocabulary the recognizer targets is the region's, not the capability's.** `select_supported_strategy` builds a `ScalarProgram` and a `LogicalAccess` per region. `PointwiseF32Node` has no sigmoid-weighted linear unit and `LogicalAccess` has no reindex; the only broadcast is `ScalarBroadcast`, a rank-zero operand read once. So a recognizer that admitted `silu(x)` would have to decompose it into multiply, exp, add, and divide nodes — which is this boundary re-deriving what the registered provider's lowering already states, and exactly what occurrence refinement exists to prevent.

**Inference — the question is architectural before it is mechanical.** Either the region vocabulary grows a node per admitted elementary family (and the accuracy contract each carries has to reach the region's numerical realization), or a region gains a way to name an occurrence whose per-point body is the resolved capability's emitted index region. The first is additive and bounded; the second is the seam that makes an out-of-crate provider's family reachable without a `tiler-ir` change per family. Choosing between them is a design decision with an ADR-shaped consequence, and it is the first work item here.

## Boundaries

- Refinement stays the authority that proves a provider's region realizes its occurrence. Whichever route is chosen, the compiler must not restate a provider's per-point arithmetic as its own.
- Each family's registered accuracy contract must reach whatever the region records, or the compiled program would make a tolerance claim nothing carries.
- Until it lands, `operation-set` remains the refusal, and the `a_family_outside_the_expression_vocabulary_refuses_with_a_typed_reason` test in `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` is what keeps it observed.

## Closes when

The route is chosen and recorded as an accepted decision; at least one of the three families compiles through `tiler_compiler::session` to an emitted region whose numerical realization carries its accuracy obligation; and a family with no installed capability still refuses by name, observed failing.
