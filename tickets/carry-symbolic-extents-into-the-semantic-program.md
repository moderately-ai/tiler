---
id: carry-symbolic-extents-into-the-semantic-program
title: Carry symbolic extents from an inline region into the semantic program
status: in-progress
priority: p1
dependencies: []
related: [prototype-inline-proc-macro-frontend, promote-the-symbolic-index-profile-to-a-public-boundary, prototype-inline-aot-integration-proof]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [research, frontend, shapes]
claimed_from: todo
assignee: worker-sym-extents
lease_expires_at: 1785560186
---
## Why this exists

`prototype-inline-proc-macro-frontend` delivered the approved `tiler::tensor!` region and found that its central feature cannot reach the compiler at all.

**Fact.** `tiler_ir::shape::Shape` is a fixed-extent vocabulary — `crates/tiler-ir/src/shape.rs:1` calls itself "Target-independent **fixed** shape vocabulary" and `Extent` wraps a `u64`. `SemanticProgramBuilder::input` takes a `Shape`, so a semantic program's operand shapes are concrete numbers. Reproduce with `grep -n "pub struct Extent" crates/tiler-ir/src/shape.rs`.

**Fact.** A region's `sym n` binds at `AvailabilityPhase::LiveDevicePreflight` with `FactProvenance::RuntimeValidated` (`crates/tiler-macros/src/binding.rs`), which is to say from the values the invocation is handed, at run time. There is no extent at expansion time.

**Inference.** An inline region carrying a symbolic extent therefore cannot be constructed as a `SemanticProgram` while it is being expanded, and so cannot be verified, normalized, optimized, scheduled, lowered, compiled, or AOT-delivered. `crates/tiler-macros/src/region.rs` records this as `ProgramEvidence::DeferredSymbolicExtent` and refuses to substitute a representative extent, because a program built over invented extents would be a different program and its identity would name something no consumer wrote.

Symbols do reach the *index* layer — `SourcedExtent::Symbol`, `IndexRegionBuilder::new_with_shape_environment`, `sourced_tensor` — so the gap is specifically between the region text and the semantic program, not the absence of a symbolic vocabulary. `docs/ir.md` already states the boundary: "Completing this bounded static-extent profile will not complete the symbolic contract above."

## User-visible outcome

An inline region declaring `sym n` reaches the same compiler path a fully literal region reaches, so the accepted inline AOT flow is available to the syntax Tom approved rather than only to its fully specialized subset.

## What this must decide

- Whether a symbolic semantic shape is a widening of `Shape`, a distinct sourced shape at the semantic layer mirroring `SourcedShape`, or a specialization step that fixes extents from a caller-supplied environment before the semantic program is built.
- How `ShapeEnvIdentity` participates in semantic and artifact identity, so two regions declaring one interface remain one subject.
- What a frontend does when an extent is genuinely unknown until dispatch: specialize per observed extent and cache, or carry the symbol through to a guarded plan.

Each of those changes a public boundary, so this is a research ticket before it is an implementation one.

## Do not

Do not close this by having the frontend invent extents, by compiling a representative specialization and reusing its artifact, or by moving program construction into generated runtime code — the last is the runtime JIT the accepted inline developer experience forbids outright.
