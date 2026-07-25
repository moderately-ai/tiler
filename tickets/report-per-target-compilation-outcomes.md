---
id: report-per-target-compilation-outcomes
title: Report per-target compilation outcomes instead of aborting on the first refusal
status: todo
priority: p2
dependencies: []
related: [prototype-public-compiler-api]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api]
---
**Fact — one target's refusal discards every other target's result.** `crates/tiler-compiler/src/pipeline.rs::compile_verified` maps each declared target profile through `compile_target` and gathers them with a fallible collect, so the first `Err` short-circuits and no `TargetCompilationProduct` survives. `CompilationProduct` has no per-target failure record and no partial form. The exact check is to read `compile_verified` — the `map(...).collect::<Result<_, _>>()?` is the whole mechanism.

**Consequence at the public boundary.** `tiler_compiler::session::compile_governed` returns `Result<Vec<Compilation>, CompileFailure>`, so a caller that declared N targets gets either N successes or one failure. It cannot learn which target refused, and it cannot use the targets that were feasible. `prototype-public-compiler-api` closed the adjacent question — a failed compilation now carries the complete explain trace the compiler sealed — but that trace is scoped to *one* target request, so on a multi-target compilation it explains one refusal and says nothing about the others.

**Why this is not visible today.** `CompilationRequest::governed` declares exactly one target profile, so the loop has one iteration and the two shapes are indistinguishable. This is latent rather than live, and it becomes live the moment a second profile is admitted or the request is exposed.

**Why it was split rather than folded in.** It is a different question from the seven `prototype-public-compiler-api` owns. Those are about what a boundary exposes *of a report*; this is about the cardinality of the result itself — whether a compilation is one all-or-nothing outcome or a per-target outcome set — and answering it changes `CompilationProduct`, the public return type, and what "the compilation failed" means. Folding it in would have widened an approved surface on a question nobody had asked.

## Scope

Decide and implement whether a multi-target compilation returns per-target outcomes. Two facts constrain the answer and should be checked before choosing:

- A refusal that happens *before* per-target work — request verification, semantic output typing, numerical-contract resolution, normalization — is genuinely whole-compilation and has no target to attribute to. Any per-target shape must keep that case distinguishable from a target-specific refusal, or a caller will read a malformed request as N target rejections.
- `verified.for_target(target)` itself fails with `RequestError::UnverifiedTargetSelection`, which is a compiler-state defect rather than a target's assessment, so it does not belong in a per-target rejection either.

The shape is a public-boundary change and is Tom's to accept.

## Closes when

A compilation declaring several target profiles reports each target's outcome separately, a whole-compilation refusal stays distinguishable from a per-target one, a test covers a request in which one target succeeds and another is refused, and `uv run --locked python scripts/check_repository.py` passes.
