---
id: evaluate-retained-shape-relations-before-routing-commit
title: Evaluate retained shape relations against invocation bindings before routing commit
status: done
priority: p1
dependencies: [admit-an-additive-extent-relation, reclassify-language-model-work-as-a-conformance-track]
related: [bind-repeated-invocations-over-caller-retained-tensors, design-autoregressive-state-and-kv-cache, execute-the-decode-step-path, test-the-autoregressive-state-failure-cases, construct-a-symbolic-region-as-a-semantic-program, admit-symbolic-extents-at-the-compiler-request-boundary, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [implementation/ir, implementation/artifact, implementation/runtime, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets, implementation/frontend, research/target-profiles]
paths: []
tags: [implementation, validation, shapes, runtime, consumer-neutral, fail-closed, class-generic-capability, decision, needs-tom, public-boundary, identity]
---
## User-visible outcome

An invocation whose live extent bindings are **mutually inconsistent** with a retained semantic shape relation refuses before routing commit and before any program work. The check relates bound extents to each other (for example `S == C + T`); it does not detect content-stale allocations whose bound extents are consistent, and it does not refuse a consistent triple merely because an allocation is larger than the bound live extents.

## Correctness boundary

**Fact.** The accepted `ExtentRelation::AdditiveEquality` representation and `ShapeEnvBuilder` check static/root-bound contradictions. A relation with runtime-bound terms is retained when the canonical lower-bound model exhibits a solution, but no current launch-preflight consumer evaluates it against the invocation's live values.

**Fact — present packaging, corrected by the exact-base audit below.** ShapeEnv identity and retained relations do not travel in the decoded artifact envelope today. `ArtifactProgramBuilder` refuses an interface that itself names a declared symbol as `ArtifactBuildError::SymbolicSemanticInterface`, pinned by `a_symbolic_semantic_program_never_reaches_the_artifact_builder`. That refusal does **not** imply that every artifact-reachable program carries the empty environment: `SemanticProgramBuilder::try_standard_with_shape_environment` also accepts ordinary fixed-shape inputs, both kernel and artifact builders narrow only the interface shapes, and `ArtifactProgramBuilder` temporarily retains the complete `SemanticIdentity` before `project_semantic` drops its fifth shape-environment subject. Two fixed-interface programs can therefore already differ only by an otherwise unused non-empty retained environment and collapse to the same artifact envelope and identity. Carrying a non-empty ShapeEnv or an equivalent governed retained-relation/source projection is an existing soundness repair and a governed artifact/schema identity step; this ticket must execute that step whole — owning ledger, version, and recomputed pins — or stop for Tom.

**Fact — present term binding.** Authoritative term values already exist on the binding path: `BindingSource::InputDimension` on ShapeEnv root bindings and `AbiRoot::InputExtent` / `AbiFactBinder::bind_input_extent` at live-device preflight for ABI expressions. The missing consumer is a join of retained constraints to those facts before `RoutingCommit`, not a new uncorrelated caller scalar spelling of the same extents.

**Inference.** Representation without consumption does not close mutual extent inconsistency at launch. After [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md), this preflight consumer is the **only Tiler layer** that closes mutual extent inconsistency for retained relations, because no runtime state survives an invocation to hold a valid length. Content-level staleness (consistent `S,C,T` over bytes last written shorter) remains a stated consumer obligation under L5 Case 2 and is undetectable by Tiler in principle; this ticket does not claim to close it. The decoded artifact must retain the governed relation and its term bindings (or an equivalent encoded retained-relation table under the identity step above), the invocation must supply each live value from its authoritative input-extent source, and the check must run before the one-way routing commit. A missing, wrong-domain, contradictory, or unevaluable binding refuses with a typed cause; it never becomes a fallback after program work begins.

**Correction — 2026-08-10.** Earlier wording of the User-visible outcome and Inference framed the obligation as refusing a "stale extent merely because its allocation is large enough" and as "the only layer that closes any part of" L5's stale-binding case. L5 Case 2 separates (a) true content staleness — undetectable by Tiler — from (b) mutually inconsistent bound extents, which this consumer owns. The body above matches Case 2; the superseded phrasing is retired. Graph repair on the same date: `bind-repeated-invocations-over-caller-retained-tensors` moved from `dependencies` to `related` (complementary multi-extent packaging, not a prerequisite API for evaluating already-bound input extents); `contracts/integrations` removed from `scopes` (no `docs/integration/**` edit named); symbolic packaging predecessors listed under `related` for the end-to-end C1 path while unit evidence may discharge via a synthetic retained-relation fixture (see Required evidence).

**Exact-base Fact audit correction — 2026-08-10 at `e9ba7be7`.** The earlier packaging Fact's conclusion that no two artifact-reachable programs can differ by the fifth subject was false and changed this from a future symbolic-interface admission step into an already reachable identity hole. The audit stopped before implementation. Runtime receives `AbiFacts` but, under ADR 0081, must not gain a direct dependency on `tiler-ir`; a private artifact table alone is also insufficient because the runtime needs a public decoded evaluator or a changed public call boundary to consume it and report the required typed failures. This consequential boundary is queued for Tom. **Recommendation:** encode a canonical retained-relation/source projection privately in `tiler-artifact`, reject unsupported relation/source variants exhaustively, and expose only `DecodedArtifact::evaluate_retained_shape_relations(&AbiFacts) -> Result<(), RetainedShapeRelationFailure>` with a non-exhaustive typed failure. This preserves runtime dependency direction and keeps ShapeEnv/solver types private. **Strongest counterpoint:** carrying the full fifth-subject ShapeEnv avoids projection-drift obligations and is preferable if artifact identity must preserve every ShapeEnv distinction rather than only artifact-observable obligations. **Release trigger:** Tom accepts one exact carried subject and public decoded evaluation/refusal surface; no further source measurement can choose that boundary for him.

## Accepted — 2026-08-11

**Decision.** Tom accepted the exact carrier, public evaluator, refusal boundary, and identity migration below in the Codex coordination thread by replying `sounds good, accpet` to the coordinator's decision packet. The relay source is Tom's direct response in that thread; the coordinator records it here without widening the presented surface. This acceptance moves the ticket to `todo`; it does not claim the implementation or evidence is complete.

The artifact carries one private canonical `RetainedShapeEnvironment` as the lossless artifact representation of the semantic identity's fifth shape-environment subject. It contains every symbol declaration and root binding, including the binding source, availability phase, and provenance, followed by every semantic input constraint in canonical order. Variant guards and solver-derived state remain excluded, exactly as they are from `ShapeEnvIdentity`. Unused declarations and bindings remain identity-bearing but are not queried during an invocation.

The carried bytes are the one authority for both artifact identity and decoded evaluation, not an opaque identity paired with an independently trusted table. Artifact construction projects the verified `ShapeEnv`, regenerates the existing `ShapeEnvIdentity` bytes exactly, and refuses any mismatch. Decoding revalidates canonical order, table closure, and the same encoding before producing a view. Total matches over the present binding-source and relation vocabularies make their growth a build error rather than a silently omitted row.

All six current `ExtentRelation` forms are in the admitted evaluator vocabulary: `Equal`, `AdditiveEquality`, `Divisible`, `NonNegativeDifference`, `Interval`, and `Factorization`. `Static`, `InputDimension`, and `TargetProperty` bindings are evaluable from their authoritative values. A retained constraint that requires `InterfaceParameter` refuses during artifact construction until an authoritative ABI binding for that source is separately admitted. Future unsupported source or relation tags refuse during construction or decoding as an unsupported artifact; they never become ignored runtime rows. Arithmetic is checked in the unsigned 64-bit domain.

The accepted labelled-draft public surface is:

- `DecodedArtifact::evaluate_retained_shape_relations(&AbiFacts) -> Result<(), RetainedShapeRelationFailure>`;
- an opaque `RetainedShapeRelationFailure` implementing `Display` and `Error` and exposing `class()`; and
- a non-exhaustive `RetainedShapeRelationFailureClass` with `MissingBinding`, `InvalidBindingDomain`, `Unsatisfied`, and `ArithmeticOverflow`.

The rendered failure names the retained relation, every participating symbol and authoritative source, and every observed value available at the failure. `LoadRejection` gains a `RetainedShapeRelation` variant carrying that failure. Both `DecodedProgram::preflight` and `DecodedProgram::prepare` invoke the evaluator before variant selection, route qualification, and the one-way `RoutingCommit`; there is no post-commit retry or fallback.

**Compatibility rule.** The artifact identity domain steps from `tiler.artifact-program.v16` to `tiler.artifact-program.v17`, and the neutral manifest schema steps from `16.0` to `17.0`. The retained environment lands inside the semantic-subject run, so an older decoder would otherwise consume its bytes as the interface population; the major manifest step makes that layout unreadable rather than silently misparsed. The manifest domain, envelope format, canonical encoding profile, component schemas, stage/provider/payload key domains, and shared-IR identity domains remain unchanged. The artifact ABI ledger and every derived identity, cache, fixed-content, and cross-crate pin must be recomputed on the merged tree.

Invocation values do not enter artifact identity. One decoded artifact remains valid as `C`, `T`, and `S` change between invocations; only the retained relation and its authoritative source mapping are identity-bearing.

**Excluded surface.** This acceptance does not add a runtime dependency on `tiler-ir`, expose `ShapeEnv` or solver types, admit caller-supplied duplicate extent scalars, detect content-stale allocations, infer an allocation-capacity policy, treat a larger consistent allocation as invalid, expose variant guards through this table, or permit refusal and fallback after routing commit.

**Strongest counterpoint accepted with the decision.** Carrying only the opaque fifth-subject identity would avoid an artifact projection, but it could not be evaluated. Pairing it with an unchecked relation table would create two authorities, while carrying full IR into the runtime would violate ADR 0081. The lossless, byte-checked private carrier is accepted as the smallest boundary that fixes both the existing identity collision and launch-time relation enforcement.

## Fact audit — 2026-08-13 at `b2e707c2`

- **Verified.** `ExtentRelation::AdditiveEquality` and `ShapeEnvBuilder::build` check static/root-bound contradictions; a runtime-bound additive relation is retained only when the lower-bound model exhibits a solution; no launch-preflight consumer existed at this base.
- **Verified.** `ArtifactBuildError::SymbolicSemanticInterface` and `a_symbolic_semantic_program_never_reaches_the_artifact_builder` refuse a symbolic interface. `project_semantic` dropped the fifth subject. `try_standard_with_shape_environment` accepts ordinary fixed-shape inputs, so two artifact-reachable programs could already differ only by an unused environment.
- **Verified.** `BindingSource::InputDimension`, `AbiRoot::InputExtent`, and `AbiFactBinder::bind_input_extent` already supply authoritative term values.
- **Verified.** Identity domain was `tiler.artifact-program.v16` and the neutral manifest was `16.0`. All six `ExtentRelation` forms and all four `BindingSource` forms exist. Tom's 2026-08-11 acceptance is present.

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
