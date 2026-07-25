---
id: prototype-public-compiler-api
title: Implement the reviewed public compiler boundary
status: todo
priority: p0
dependencies: [prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, dx]
---
Implement ADR 0069's consumer-agnostic CompilationRequest, session/provider
inputs, checked compilation result, stable diagnostics/explain, and ordinary
call-site ergonomics over the verified pipeline. Tom reviews consequential
public crate, trait, type, and call-site boundaries before acceptance. Frontends
consume this API; backend feasibility components need not depend on it.

## Inherited explain review agenda

The merged typed-explain implementation deliberately kept its module private and
raised eight public-surface questions. Tom settled the first on 2026-07-23:
explain stays a compiler-owned module, with the vocabulary moving into
`tiler-ir` only if a second crate must read traces (tracked by
`record-explain-ownership-decision`). The remaining seven are deferred to this
ticket because they all concern a public surface that only this boundary
introduces. Settle each explicitly here rather than letting an implementation
choose by default:

- how successful and failed compilations return partial or complete reports;
- whether canonical traces are serialized or embedded in artifacts, noting that
  docs/artifact-abi.md currently does not contemplate embedding them;
- which renderer guarantees, retention controls, and provider-detail/redaction
  policy form part of the public contract;
- whether public enums are non-exhaustive, versioned schema views, or both;
- which component may mint trusted evidence receipts for external providers;
- whether the public identity is canonical bytes, a specified digest, or both;
- how much of the request-qualified renderer header is stable versus redacted.

The merged draft's own handoff notes on `tickets/prototype-typed-explain-infrastructure.md`
record the reasoning behind each; read them before proposing answers.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Progress — a minimal draft landed; this ticket is not met

**Landed at `a56bff8`.** `pub mod session` exposes `compile_governed(&SemanticProgram, NumericalContract)`, one `Compilation` per target profile, borrowed `PlanAlternative` views exposing `stable_id`, `is_fused`, and `kernels`, a typed `CompileFailure`, and explain as an opaque `ExplainReport` with only `render()`. It is the first surface over which any caller outside `tiler-compiler` can compile anything; before it, `pipeline` was a private module with a `pub(crate)` entry point, which is why the backend crates had no work to do. Two consumers now exist — the offline producer and the runtime proof — and the second reaches an Apple M4 Ax end to end through it.

**Why this ticket is still open.** Three reasons, none of them cosmetic.

1. **Tom has not reviewed it.** ADR 0075 makes a new publicly reachable namespace an always-ask category, and the ticket itself says any consequential public boundary "remains a draft until Tom reviews and accepts the exact implementation commit". It has not been reviewed, so it is a draft by definition.
2. **All seven inherited explain questions remain open.** Report completeness on failure, trace serialization and artifact embedding, renderer/retention/redaction guarantees, enum exhaustiveness versus versioned schema views, evidence-receipt minting, identity as canonical bytes versus a digest, and header stability. The draft answers **none** of them, deliberately: explain is exposed with one rendering method because that is the narrowest shape that cannot answer them by default. Answering by omission is what this ticket exists to prevent, and a richer surface would have done exactly that.
3. **The request is not exposed.** `compile_governed` names the governed profile rather than letting a caller assemble a `CompilationRequest`. That is honest while the profile admits one shape environment, one budget set, one target profile, and one capability snapshot, but it is not the "consumer-agnostic CompilationRequest, session/provider inputs" ADR 0069 specifies.

**What review should look at first**, because everything downstream is written against it: whether alternatives should be borrowed views or owned records; whether both fused and materialized alternatives belong on the surface (they are exposed because the offline slice needs the selected program *and* the materialized reference, and a selected-only surface could not express that); and whether `CompileFailure`'s four classes are the right granularity or should carry their internal cause.

## The review agenda grew after `relocate-abi-expressions-into-tiler-ir`

`carry-the-metal-payload-in-an-artifact-envelope` needed an artifact assembler, and nothing outside `tiler-compiler` could reach the inputs one needs. Three surfaces were added to `session` and are draft under the same rule as the rest of it — recorded here so review sees the current boundary rather than the one this ticket originally described.

**`PlanAlternative::abi() -> AbiConstruction<'_>`** (commit `d6a69bf`), with `AbiEntry`. Exposes the applicability guard, per-binding accessible byte ranges, and per-entry launch geometry as **arena positions into a `Vec<ExprNode>`**, plus `kernel_program() -> &VerifiedKernelProgram`. The design question for review: this hands out expressions rather than resolved scalars, deliberately, because a consumer given numbers would re-derive an accessible byte range beside the compiler's own derivation. The cost is that a consumer must replay the arena onto its own builder, and must prune it — `tiler-artifact`'s verifier rejects an unreachable expression, and the compiler's arena serves both alternatives, so a wholesale replay fails. Whether that replay obligation belongs on a consumer is the thing to look at.

**`PlanAlternative::selected_capabilities() -> impl ExactSizeIterator<Item = SelectedCapability<'_>>`** (commit `d7ba751`). Exposes provider identity, the governed capability key, and the capability revision. The question for review is key ownership: the compiler mints the key and the consumer wraps it, rather than the compiler exposing family and operation for a consumer to compose, because the key enters artifact identity under ADR 0072.

**`KernelProgram::core()` lost `#[cfg(test)]`.** Its own doc comment deferred to "a reviewed public compiler facade"; that facade is `session`, which is itself still a draft, so the accessor's premise is only as accepted as this ticket is.

## Decision — Tom, 2026-07-25

**Approved: promote the compiler boundary.** `pub mod session` in its current shape — `compile_governed`, `Compilation`, `PlanAlternative`, `ExplainReport`, `CompileFailure`, `AbiConstruction`, `AbiEntry`, `SelectedCapability` — together with `pub mod abi` in `tiler-ir`. This closes the ADR 0075 always-ask review that has been gating the entire frontend axis: `prototype-inline-proc-macro-frontend` and everything behind it are now dependency-satisfied on this point.

The seven deferred public-surface questions this ticket carries are **not** answered by the promotion. Explain stays an opaque handle with only `render()`, which is the shape that answers none of them by default, and any future widening is a separate reviewed step.
