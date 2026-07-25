---
id: prototype-public-compiler-api
title: Implement the reviewed public compiler boundary
status: in-progress
priority: p0
dependencies: [prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/compiler, implementation/ir, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, dx]
claimed_from: todo
assignee: agent-api
lease_expires_at: 1785011687
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

## Outcome

The seven deferred public-surface questions are **settled**, on base `6fae4f3`. None was escalated, because none survived elimination as a question: each has one answer that correctness, an accepted contract, or a measured property forces, and the alternatives fail rather than trade off. Six answers keep the surface exactly as narrow as it already is; one required a change, and it is the only one that did.

Every answer is recorded twice — at the item it governs in `crates/tiler-compiler/src/session.rs`, and normatively in a new "What the public compiler boundary exposes of a trace" subsection of `docs/compiler/optimizer.md`. `contracts/optimizer` was added to this ticket's scopes to do the second, because a decision recorded on a ticket is not a decision applied, and no open ticket held that scope.

### 1. Report completeness — the one that required a change

**Answer.** A trace is complete or absent, never partial; a failed compilation returns the complete trace when one exists, and states structurally when one cannot.

**Derivation.** The "partial" half of the question no longer has a subject: `never-truncate-the-governed-explain-trace` made a detail record that would exceed the retained ceiling a typed `ExplainError::DetailCapacity` failure, so a sealed trace is complete by construction. What remained was a live defect rather than a choice. `session.rs`'s `From<CompileError>` matched `CompileError::Explained { source, .. }` and **discarded the trace**, so a caller receiving `CompileFailure::NoFeasiblePlan` — a bare unit variant — could not learn which predicate rejected which alternative. `docs/compiler/optimizer.md` requires that "Every rejection records its stage, stable reason code, rule/provider identity, affected operation/value or candidate, failed predicate/evidence" and that explain output "never collapses these into 'not fused.'" The boundary was performing exactly that collapse, on data the compiler had already sealed. Keeping it is not a cheaper option; it is a contract violation.

**Landed.** `CompileFailure` is now a struct carrying `class: CompileFailureClass` and `explain: Option<VerifiedExplainTrace>`, with `class()` and `explain() -> Option<ExplainReport<'_>>`. The four approved classes are unchanged in name, meaning and `#[non_exhaustive]`-ness; only the enum's own name moved to `CompileFailureClass`. `Debug` is hand-written to print the class and a record count rather than the whole trace, because both in-workspace consumers format the failure with `{failure:?}`.

**Measured: no out-of-crate consumer breaks.** `cargo check --workspace --all-targets` compiles `tiler-prototype-compile` and `tiler-prototype-run` unchanged; the only errors were this module's own tests. Both prototypes name `CompileFailure`, hold it in a local error variant, and `Debug`-print it — neither matches its variants — so the reshape is source-compatible for every consumer that exists. That matters for scope as well as for churn: `prototypes/serial-sum-compile` and `prototypes/serial-sum-run` are `implementation/metal-aot` and `implementation/runtime`, neither of which this ticket declares, and the change was checked to require no edit there rather than assumed to.

**Tests.** `a_target_failure_carries_its_complete_trace` drives a zero region-candidate budget through the internal request, converts the resulting `CompileError`, and asserts the class is `NoFeasiblePlan` *and* that the rendered trace names both the terminal failure and `region.formation.v1`. It uses the internal request deliberately: reaching a post-request failure needs a budget the governed profile does not expose, which is a fact about the bounded profile rather than about the mapping under test. `a_refusal_before_the_trace_boundary_carries_no_trace` covers the other side through the public entry point.

**Still open, and now visible rather than hidden.** `compile_verified` collects per-target results with a fallible collect, so one target's refusal aborts the whole compilation and the caller cannot learn *which* target refused. That is invisible today because the governed request declares exactly one profile. It is a real gap in a multi-target boundary and is **not** part of this ticket's question; split as `report-per-target-compilation-outcomes`.

### 2. Trace serialization and artifact embedding

**Answer.** Neither. Not serialized at this boundary, not embedded in an artifact.

**Derivation.** `docs/compiler/optimizer.md` already stated the artifact half — "Nothing in this contract requires an explain trace to be serialized into an artifact envelope, and the artifact contract does not carry one" — so the ticket's premise that this was open was already half-stale. Both placements fail independently. *Inside* artifact identity, a trace folds rule keys, provider revisions and the explain schema version, so renaming a reason code would change the identity of a program whose executable meaning did not, invalidating every cache entry — the exact ground on which `docs/artifact-abi.md` already keeps the frozen registry snapshot out of the envelope, "letting an unused provider invalidate a cache entry". *Outside* it, `docs/artifact-abi.md` rejects a section no variant references as `UnreferencedSection`, precisely so an envelope cannot carry bytes its identity does not cover. Serializing the canonical bytes fails for a third reason: those bytes *are* the trace's identity, and ADR 0074 convention 2 keeps a canonical identity opaque and never re-derived by a consumer, so parsing them is a second derivation of what the trace means. The producer-evidence use this would otherwise serve is already owned, with a better shape, by the proof sidecar — where a sidecar names an artifact and an artifact never names a sidecar.

**Trigger.** A second crate that must *read* canonical traces. `docs/compiler/optimizer.md` already routes that to moving the record vocabulary into `tiler-ir`, not to publishing a byte format.

### 3. Renderer guarantees, retention controls, redaction

**Answer.** Rendering is deterministic and total; its spelling is not a contract. There is no retention control to expose. Nothing is redacted.

**Derivation.** *Retention* has no subject left: the configurable detail budget is gone, and re-introducing one would re-introduce the silently-incomplete trace that `never-truncate-the-governed-explain-trace` closed. *Renderer stability* fails because committing to the text creates a second description of a trace that must be kept in agreement with its canonical bytes — the duplicate-derivation hazard the data/presentation split exists to prevent — and freezes every stage, disposition and subject spelling. What does survive, and is worth stating because it is checkable without committing to a spelling, is determinism plus totality: the renderer has no filter and no bound. *Redaction* has no subject either: the writer refuses a rule attributed to any provider outside the request's own installed registry plus Tiler's two governed providers, so a trace contains nothing the caller did not supply or Tiler did not mint. Redacting would also make a rejection unexplainable, which the contract forbids by requiring every rejection to name its rule and provider.

**Trigger.** A registry the caller does not control installing rules.

### 4. Enum exhaustiveness versus versioned schema views

**Answer.** `#[non_exhaustive]` per ADR 0074's amended convention 5 clause test, applied per type. Never a versioned schema view, and never both.

**Derivation.** This one was already answered by an accepted ADR that post-dates the question; the ticket had not noticed. Convention 5 decides by asking what an out-of-crate wildcard arm would have to do. `CompileFailureClass`: consumers only classify partially or forward — clause 5a, so the attribute applies, and it already carried it. `NumericalContract`: a caller-constructed input, which 5a's stated asymmetry covers. A parallel versioned schema view is eliminated on two counts — it is a hand-maintained projection of an enum that nothing keeps in agreement, which is convention 3's own argument against encoding a projection instead of its source; and it buys compatibility, which ADR 0075 records Tom rejecting outright while no crate in this workspace is publishable. "Both" is strictly worse than either.

### 5. Evidence-receipt minting for external providers

**Answer.** Only the compiler mints one, and only from a proof it derived itself. No receipt surface is reachable from this boundary, which is the correct amount.

**Derivation.** A receipt carries the `SoundProof` class, and `AGENTS.md` requires `SoundProof`, exhaustive finite evidence, empirical evidence, normative guarantees and `Unknown` to stay distinct. A receipt supplied by a provider is a claim; recording a claim as `SoundProof` converts an assertion into a proof at the boundary, and a fusion legality proof is what admits a rewrite — so the failure mode is an illegal rewrite admitted on an unverified assertion, which is a defect and not a trade-off. The compiler cannot verify a foreign proof it did not derive; if it could, it would derive it. A provider's real contribution is its identity and revision, which the compiler attributes and bounds against the request's installed registry — provenance, not evidence. The answer does not move even if a provider can one day ship a machine-checkable proof: the compiler would still mint the receipt, from its own re-check.

### 6. Public identity — canonical bytes, a specified digest, or both

**Answer.** Canonical bytes, opaque, at every identity this boundary emits. Never a digest, never both.

**Derivation.** ADR 0074 convention 2 states it, and `Compilation::target_profile_descriptor` already applies it in terms. A digest is a second identity over one subject: it needs a stated hash, a stated truncation and a collision argument, and `select-the-governed-artifact-digest-implementation` shows the production hash is not yet chosen, so emitting one here would commit the boundary to a choice that has not been made. "Both" publishes two values a consumer can disagree about. **No code change was needed** — the boundary already conforms, verified item by item rather than assumed.

### 7. Request-qualified renderer header stability

**Answer.** The header is presentation. The version prefix is the only part with a stated meaning; the request qualifier is a correlation label, never an identity, and is not redacted.

**Derivation.** The qualifier is a 64-bit FNV-1a fold of the canonical request subject. Treating it as a durable identifier makes it an equality input, which convention 2 forbids for exactly this shape and which is unsound at 64 bits of a non-cryptographic fold: two distinct requests can collide, and a consumer keying a cache on it would serve one compilation's trace for another's — silent wrongness. Redacting it protects nothing, because it is derived from the caller's own request, and it removes the only thing distinguishing two rendered traces in a log. Presentation-with-a-stated-non-identity is the sole survivor, and `ExplainReport::render`'s documentation now says so, because a reader who sees a hex request qualifier and is told nothing will assume it is an identifier.

### What this ticket does not claim

Tom approved `pub mod session` in its current shape on 2026-07-25, and question 1's answer reshapes `CompileFailure` within it. That is a concrete, tested draft of the answer, not implicit approval of the new signature: `CompileFailure` becoming a struct, and the four classes moving to `CompileFailureClass`, is a change to an existing public signature, which ADR 0075 routes to Tom. Everything else on the surface is unchanged.

The request itself is still not exposed — `compile_governed` names the governed profile rather than letting a caller assemble a `CompilationRequest`, which ADR 0069 specifies and which the bounded profile cannot yet honour. That remains this ticket's stated gap and is unaffected by the seven answers.

`uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass.
