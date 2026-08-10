---
id: evaluate-retained-shape-relations-before-routing-commit
title: Evaluate retained shape relations against invocation bindings before routing commit
status: blocked
priority: p1
dependencies: [admit-an-additive-extent-relation, reclassify-language-model-work-as-a-conformance-track]
related: [bind-repeated-invocations-over-caller-retained-tensors, design-autoregressive-state-and-kv-cache, execute-the-decode-step-path, test-the-autoregressive-state-failure-cases, construct-a-symbolic-region-as-a-semantic-program, admit-symbolic-extents-at-the-compiler-request-boundary, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [implementation/ir, implementation/artifact, implementation/runtime, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, validation, shapes, runtime, consumer-neutral, fail-closed, class-generic-capability]
---
## User-visible outcome

An invocation whose live extent bindings are **mutually inconsistent** with a retained semantic shape relation refuses before routing commit and before any program work. The check relates bound extents to each other (for example `S == C + T`); it does not detect content-stale allocations whose bound extents are consistent, and it does not refuse a consistent triple merely because an allocation is larger than the bound live extents.

## Correctness boundary

**Fact.** The accepted `ExtentRelation::AdditiveEquality` representation and `ShapeEnvBuilder` check static/root-bound contradictions. A relation with runtime-bound terms is retained when the canonical lower-bound model exhibits a solution, but no current launch-preflight consumer evaluates it against the invocation's live values.

**Fact — present packaging (2026-08-10 audit).** ShapeEnv identity and retained relations do not travel in the decoded artifact envelope today. `docs/artifact-abi.md` records that the shape environment is the fifth `SemanticIdentity` subject and is deliberately omitted from the envelope because no program whose interface names a declared symbol can reach the builder; `ArtifactProgramBuilder` refuses such interfaces as `ArtifactBuildError::SymbolicSemanticInterface`, pinned by `a_symbolic_semantic_program_never_reaches_the_artifact_builder`. Envelope programs therefore carry only the empty environment's identity, so no two artifacts can differ by retained relations yet. Admitting non-empty shape environments (or an equivalent governed retained-relation table) is a governed artifact/schema identity step under the fifth subject; this ticket must execute that step whole — owning ledger, version, and recomputed pins — or stop for Tom. Admitting symbolic interfaces without carrying the fifth subject would yield unkeyed programs.

**Fact — present term binding.** Authoritative term values already exist on the binding path: `BindingSource::InputDimension` on ShapeEnv root bindings and `AbiRoot::InputExtent` / `AbiFactBinder::bind_input_extent` at live-device preflight for ABI expressions. The missing consumer is a join of retained constraints to those facts before `RoutingCommit`, not a new uncorrelated caller scalar spelling of the same extents.

**Inference.** Representation without consumption does not close mutual extent inconsistency at launch. After [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md), this preflight consumer is the **only Tiler layer** that closes mutual extent inconsistency for retained relations, because no runtime state survives an invocation to hold a valid length. Content-level staleness (consistent `S,C,T` over bytes last written shorter) remains a stated consumer obligation under L5 Case 2 and is undetectable by Tiler in principle; this ticket does not claim to close it. The decoded artifact must retain the governed relation and its term bindings (or an equivalent encoded retained-relation table under the identity step above), the invocation must supply each live value from its authoritative input-extent source, and the check must run before the one-way routing commit. A missing, wrong-domain, contradictory, or unevaluable binding refuses with a typed cause; it never becomes a fallback after program work begins.

**Correction — 2026-08-10.** Earlier wording of the User-visible outcome and Inference framed the obligation as refusing a "stale extent merely because its allocation is large enough" and as "the only layer that closes any part of" L5's stale-binding case. L5 Case 2 separates (a) true content staleness — undetectable by Tiler — from (b) mutually inconsistent bound extents, which this consumer owns. The body above matches Case 2; the superseded phrasing is retired. Graph repair on the same date: `bind-repeated-invocations-over-caller-retained-tensors` moved from `dependencies` to `related` (complementary multi-extent packaging, not a prerequisite API for evaluating already-bound input extents); `contracts/integrations` removed from `scopes` (no `docs/integration/**` edit named); symbolic packaging predecessors listed under `related` for the end-to-end C1 path while unit evidence may discharge via a synthetic retained-relation fixture (see Required evidence).

## Required work

- Trace the accepted ShapeEnv relation and binding identities through artifact construction, encoding, decoding, and runtime preflight without duplicating a second shape solver or letting runtime depend on frontend-specific types. Where the envelope does not yet carry a non-empty shape environment, the packaging/identity step that admits retained relations (fifth subject or equivalent governed table) is in scope and must run whole, or stop for Tom.
- Evaluate the same bounded relation vocabulary the artifact claims to carry; unsupported future variants refuse rather than being ignored.
- Bind each relation term from its authoritative invocation source — the bound input-axis extent already available via `BindingSource::InputDimension` / `AbiRoot::InputExtent` — rather than from a caller-supplied scalar. Do not let a caller provide a second uncorrelated spelling of a value the bindings already determine.
- Preserve routing discipline: semantic binding validation completes before `RoutingCommit`, allocation, encoding, or submission.
- Execute any artifact/schema identity step whole, updating its owning ledger and recomputing every pin on the merged tree. Stop for Tom if the exact design requires a new consequential public type or call-site boundary.

## Required evidence

- The static neighbour `S = 15, C = 14, T = 1` and its runtime-bound equivalent both pass.
- `S = 13, C = 14, T = 1` refuses before routing commit, names all three terms and observed sides, and fails if the preflight check is removed.
- A missing binding, a binding from the wrong symbol scope/domain, and an unsupported retained relation each refuse under distinct typed causes.
- The C1 decode path consumes the check without changing artifact identity per step; one artifact remains valid across the changing invocation bindings.
- A synthetic artifact fixture with an encoded retained-relation table (and binding sources) may discharge the preflight negatives and the before-routing-commit ordering without full symbolic compile-through. The symbolic packaging chain ([`construct-a-symbolic-region-as-a-semantic-program`](construct-a-symbolic-region-as-a-semantic-program.md), [`admit-symbolic-extents-at-the-compiler-request-boundary`](admit-symbolic-extents-at-the-compiler-request-boundary.md), [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md)) is related end-to-end C1 path work, not a hard dependency for that unit evidence.

## Closes when

Retained shape relations are identity-bound into the decoded artifact and evaluated against authoritative invocation values before routing commit; every negative path above has been watched failing; the decode-step ticket depends on this consumer; and targeted IR/artifact/runtime/build checks plus the full gate pass.
