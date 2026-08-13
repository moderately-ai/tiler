---
id: admit-symbolic-extents-at-the-compiler-request-boundary
title: Admit symbolic extents at the compiler request boundary
status: in-progress
priority: p1
dependencies: [construct-a-symbolic-region-as-a-semantic-program]
related: [carry-symbolic-extents-into-the-semantic-program]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, shapes, extents]
claimed_from: todo
assignee: worker-admit-symbolic-extents
lease_expires_at: 1786631597
---
## User-visible outcome

A symbolic semantic program reaches the compiler and is either planned with its extents symbolic or declined with a typed reason naming the unsupported case — never silently specialized and never refused for a reason that names the wrong authority.

## Why this exists

**Fact.** `CompilationRequest::shape_environment` is a `StaticShapeEnvironment` whose only field is a `schema_version: u32`, and `verify_request` refuses anything but `StaticShapeEnvironment::governed()`. Reproduce with `grep -n "struct StaticShapeEnvironment" -A 10 crates/tiler-compiler/src/request.rs`. It carries no symbol; it is a version gate reserving the seam.

**Fact.** The accepted specialization boundary keeps runtime extents symbolic in the logical plan by default and makes specializing an extent a physical-planning decision.

## Implementation keys

- Replace the version-only gate with a request that carries the program's own environment, rather than a second environment the caller supplies beside the program. Two environments over one program is the ambiguity `IndexRegionBuilder::new_with_shape_environment` exists to prevent.
- A normalization or capability that cannot handle a symbolic extent declines with its own typed reason. Do not let a symbolic program fall through to an existing refusal that names a different rule; the inline AOT proof already recorded how expensive a mis-attributed `UnsupportedCapability { rule: "signature" }` was to diagnose.
- Specialize nothing. A physical alternative may introduce an explicit guard that makes an extent constant within that alternative; the request boundary must not fold a value into the logical plan on the way in.
- State the measurement boundary: which normalizations admit a symbolic extent and which decline is a fact about this commit's capability set, not a claim about the compiler.

## Evidence

- A symbolic program reaches strategy selection rather than being refused before it.
- An unsupported symbolic case declines with a reason naming the symbolic extent, and the literal neighbour of the same program still compiles.
- A test asserting that no compiled plan folds a bound extent value, so the specialization refusal is checked rather than described.

## Public boundary

`CompilationRequest`'s shape-environment field is crate-internal today; if admitting the environment widens a public surface, that widening is Tom's and must be listed rather than absorbed.
