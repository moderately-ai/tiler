---
id: decide-the-truthful-public-class-for-complete-explain-capacity-refusals
title: Decide the truthful public class for complete-explain capacity refusals
status: done
priority: p1
dependencies: []
related: [decide-how-explain-capacity-bounds-active-physical-provider-populations, calibrate-the-physical-frontier-provider-and-outcome-budgets, refuse-nothing-legal-on-the-explain-detail-ceiling]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [public-boundary, explain, budgets, correctness]
---

## Outcome

Decide the exact truthful public classification for a complete-explain detail-capacity refusal reached by an otherwise valid public compile request. Keep complete-or-refused trace construction intact. Do not select a capacity value, active-provider cardinality, or full-provider support policy here; this decision is mandatory independently of whether full 32-slot activity is ever promised.

If a change survives the Pareto gate, Tom must accept the exact included and excluded public surface before implementation. Before this decision ticket closes, it must create the narrow accepted implementation ticket and its required evidence/regression ticket, then add hard dependencies from both downstream capacity tickets named under **Graph** to the implementation **and** evidence tickets. A decision-only closure is forbidden because it would satisfy the current hard dependency while the false `InvalidCompilerOutput` behavior still ships.

## Accepted by Tom — 2026-08-14

Tom accepted the exact sole-survivor `BudgetExhausted` surface in the live Codex conversation, relayed by the coordinating agent. The accepted answer is option 1 below without expansion: the two report-only explain resources, `ConstructionLowerBound`, the existing build-constant ceilings and prefix-lower-bound arithmetic, and a distinct internal outer/request-wide carrier that forbids candidate, contract, or target retry and partial output.

The required implementation and evidence tickets are [`implement-the-truthful-explain-capacity-budget-refusal`](implement-the-truthful-explain-capacity-budget-refusal.md) and [`prove-the-truthful-explain-capacity-budget-refusal-boundary`](prove-the-truthful-explain-capacity-budget-refusal-boundary.md). This decision may become terminal only with both nodes present and both downstream tickets depending on both nodes.

## Facts at discovery base `e37c05b8ec28114736648edebbbdee745f4a051b`

- **Measurement.** The retained public five-operation strict request with seven installed specialists is valid, reaches physical planning, and fails with public `CompileFailureClass::InvalidCompilerOutput`. Its complete failure trace ends in `compiler-failure:explain-detail-capacity`.
- **Fact.** `crates/tiler-compiler/src/session.rs`, anchor `InvalidCompilerOutput — **unreachable by construction from a valid call, deliberately.**`, says public reachability means Tiler shipped the defect this class reports. The enum variant's own documentation says it is always a Tiler defect, never a caller-program refusal.
- **Fact.** `ExplainWriter::push`, anchor `let exceeds = if terminal`, deliberately retains hard limits and refuses rather than truncates. [`never-truncate-the-governed-explain-trace`](never-truncate-the-governed-explain-trace.md) retained that refusal to protect combinatorial growth; it did not authorize exposing the refusal as `InvalidCompilerOutput` to a valid caller.
- **Accepted-history fact.** [`refuse-nothing-legal-on-the-explain-detail-ceiling`](refuse-nothing-legal-on-the-explain-detail-ceiling.md), anchor `A semantic program that satisfies every governed budget compiles`, established that a valid in-budget request must compile or receive a class naming the caller-facing limit. It rejected reclassification alone for its earlier population because exact duplicate rejection grounds could be aggregated at source. That decision is precedent to search for lossless aggregation, not permission to retain a false class while the search is pending.
- **Fact.** `BudgetResource` currently has thirteen variants and no explain resource. `BudgetRefusal` has the closed provenances `ExactDemand`, `PlanningUpperBound`, and `SearchLowerBound`. `ExplainError::DetailCapacity` currently carries neither which independent arm fired nor `limit`/`reported` quantities. Mapping it to `BudgetExhausted` without extending the typed cause and proving the number's provenance would invent fields.
- **Fact.** The existing explain ceilings are build constants, not `DeterministicBudgets` fields and not bytes in `canonical_explain_subject_bytes`. Turning either into a request budget would change every compiler-internal request/evidence subject and explain qualifier. Public classification is independent: a dedicated additive failure class **or report-only `BudgetResource` rows** need not add either ceiling to request identity. Neither classification answer directly changes plan, artifact, or cache identity; newly admitted compilation may change selected packaged content indirectly.
- **Fact.** `CompileFailureClass` and `BudgetResource` are public `#[non_exhaustive]` enums, so variants may land additively, but ADR 0075 still reserves the consequential public boundary for Tom. `BudgetRefusal` is intentionally exhaustive because consumers totally map its closed provenance vocabulary.

## Exact-base Fact audit — 2026-08-14, `709ff11ad37c1b04a09a7fe3f28cece7b4425f66`

The purpose survives. The full ticket, repository guidance, ADRs 0074/0075, both accepted-history tickets, the public type documentation, and the complete relevant explain construction, failure mapping, request identity, schema/version, renderer, frontend, and regression-test paths were read before editing.

1. **Verified — measured public reachability.** `cargo run --release -- request-boundary 7` from `spikes/program-planning/physical-frontier-budget-calibration` reproduced all seven ordered rows. Six specialists succeed with 102 installed outcomes, 56 alternatives, 2,291 ordinal record lines, and 650,099 rendered bytes. Seven reach 119 installed outcomes and refuse as `Some(InvalidCompilerOutput)` with terminal ordinal 2,257, reason `explain-detail-capacity`, and cause 2,256. The request passed verification and emitted physical-provider outcomes before refusing.
2. **Verified — the public class is false for this path.** `crates/tiler-compiler/src/session.rs`, anchors `This is always a defect in Tiler` and `unreachable by construction from a valid`, makes `InvalidCompilerOutput` a claim about ownership, not a generic fallback. `From<ExplainError> for CompileError` preserves the typed error inside `CompilerOutputError::Explain`, but routes every explain kind through that one wrapper; `class_of` then collapses the public projection to `InvalidCompilerOutput`.
3. **Verified — complete-or-refused and host protection.** `crates/tiler-compiler/src/explain.rs`, anchors `let exceeds = if terminal` and `A trace is complete or it is refused`, checks the 4,096-detail and 1-MiB-canonical-detail bounds incrementally, withdraws the attempted encoding on refusal, and returns the unit `ExplainError::DetailCapacity`. [`never-truncate-the-governed-explain-trace`](never-truncate-the-governed-explain-trace.md), anchor `The reachable case the retained bound protects against`, retains the hard refusal because candidate enumeration is combinatorial and removes the soft truncation path because omitted counts cannot recover omitted reasons.
4. **Verified — accepted earlier aggregation does not settle this population.** [`refuse-nothing-legal-on-the-explain-detail-ceiling`](refuse-nothing-legal-on-the-explain-detail-ceiling.md), anchors `summarized at its source` and `The third is the smallest and the least satisfying`, aggregated exact duplicate cover/region rejection grounds without losing a distinct reason. It rejected reclassification for that removable wall; it neither proves the current provider/plan records duplicate nor permits a valid request to retain the false defect class pending research.
5. **Verified — current typed payload cannot tell the truth yet.** `crates/tiler-compiler/src/request.rs`, anchors `pub enum BudgetResource`, `pub enum BudgetRefusal`, and `every_budget_resource_key_is_distinct`, gives thirteen resources and the closed `ExactDemand`, `PlanningUpperBound`, and `SearchLowerBound` provenances. `ExplainError::DetailCapacity` carries no arm, limit, or attempted prefix. Therefore current code cannot populate `BudgetExhausted { resource, limit, reported }` without inventing at least one field.
6. **Imprecise — repaired above, purpose unchanged.** `VerifiedRequestSubject::canonical_explain_subject_bytes` encodes every `DeterministicBudgets` field, so adding the caps there would move every compiler-internal request/evidence subject and explain qualifier. But `BudgetResource` is a public refusal vocabulary, not itself the request budget record. Adding report-only explain rows while leaving the hard caps as build constants changes no request identity, just as a dedicated class would not. The original wording named only the dedicated-class half and could make reuse of `BudgetExhausted` look identity-changing when it need not be.
7. **Verified — public growth and its one deliberate break.** `CompileFailureClass` and `BudgetResource` are `#[non_exhaustive]`; adding variants is additive to partial consumers. `BudgetRefusal` deliberately is not: `tiler-macros::aot::rendered_refusal` totally maps all three meanings to different caller advice, so a new provenance must fail the workspace build until that advice is supplied. ADR 0074 convention 5 and ADR 0075 make this exact public semantic change Tom's decision even though the workspace is pre-alpha and unpublished.
8. **Verified after independent review — the internal carrier is scope-bearing.** `crates/tiler-compiler/src/pipeline.rs`, anchors `fn compile_candidate_target` and `CompileError::NoFeasiblePlan(_) | CompileError::BudgetExhausted(_)`, treats every existing `CompileError::BudgetExhausted` as a candidate-local infeasibility. `evaluate_preferred_groups` can therefore continue to another semantic candidate or numerical-contract group, while `compile_contract_group` and `compile_configured` can continue to later targets. The current `ExplainError::DetailCapacity` conversion instead enters `TargetCompileFailure::Outer` and aborts the whole request; a generalized or reused internal budget carrier would silently change that atomicity even if its eventual public class were truthful.

Searchable reproductions:

```sh
rg -n 'unreachable by construction from a valid|This is always a defect in Tiler|fn class_of' crates/tiler-compiler/src/session.rs
rg -n 'let exceeds = if terminal|A trace is complete or it is refused|DetailCapacity' crates/tiler-compiler/src/explain.rs
rg -n 'pub enum BudgetRefusal|pub enum BudgetResource|canonical_explain_subject_bytes' crates/tiler-compiler/src/request.rs
rg -n 'impl From<ExplainError>|ExplainError::DetailCapacity' crates/tiler-compiler/src/pipeline.rs
rg -n 'fn rendered_refusal|BudgetRefusal::' crates/tiler-macros/src/aot.rs
rg -n 'fn compile_candidate_target|CompileError::NoFeasiblePlan\(_\) \| CompileError::BudgetExhausted\(_\)|enum TargetCompileFailure' crates/tiler-compiler/src/pipeline.rs
```

## Decision gate

Compare at least:

1. extend `BudgetExhausted` with one or two explain-capacity resources and the exact refusal provenance/payload needed to make `limit` and `reported` truthful;
2. add a dedicated typed explain-capacity class/resource that does not claim the refusal is a request budget, infeasible plan, unsupported capability, invalid request, or compiler defect;
3. make the public refusal unreachable by a proven complete source aggregation or sufficiently governed widening; and
4. remove both hard ceilings without silent truncation; and
5. defer the exact surface while treating current public reachability as a live defect, never as an acceptable support statement.

Eliminate `InvalidRequest`, `UnsupportedCapability`, and `NoFeasiblePlan`: the request is valid, installing a provider can increase the demand, and the compiler has not proved the plan space infeasible. Eliminate a bare rename that carries no actionable resource or provenance. Eliminate removing both hard ceilings: the installed/active population is unbounded, trace growth is combinatorial, and `never-truncate-the-governed-explain-trace` deliberately retained the hard record/byte ceiling as real host-memory protection after removing the redundant soft truncation limit. Do not let an eventual full-provider support decision stand in for this classification decision.

For every survivor state:

- caller action and the strongest statement the class makes;
- whether `reported` is exact demand or only a lower bound at the stopped record;
- completeness and failure-trace consequences;
- public enum/type and semver consequences under ADRs 0074/0075;
- request/evidence, explain, plan, artifact, and cache identity consequences;
- schema/renderer consequences;
- strongest counterargument, reversal evidence, and subject perturbations; and
- the exact implementation and regression-test dependency if Tom accepts it.

## Pareto-complete decision packet

### Invariants every correct option must preserve

- The seven-specialist request is valid. Its refusal may name a build-declared capacity, but may not call the request malformed, the installed vocabulary unsupported, the plan space infeasible, or Tiler's verified output invalid.
- Detail construction stays atomic and complete-or-refused. No option may drop a decided record, restore a soft retention limit, or publish an omitted-record summary as if it preserved the missing reasons.
- `reported` may not be called the complete trace's demand. At the refusal point the compiler knows only the exact attempted retained prefix: `retained_detail_records + 1` for the refused record arm, or `retained_detail_bytes + encoded_refused_record_bytes` for the byte arm. Each is a lower bound on the complete trace that construction did not finish.
- The two arms remain independent. The record arm is evaluated first and the canonical-byte arm second; if one attempted record exceeds both, the public refusal names the first failed check rather than claiming the other passed. A regression must pin this deterministic precedence.
- Explain capacity remains an **outer, request-wide atomic abort**. Reaching it in any semantic candidate, numerical-contract group, or target must stop evaluation immediately: no later candidate, fallback contract, or later target may be tried, and no earlier target success may survive in a partial output.
- No classification option selects a new ceiling, provider cardinality, full-activity promise, or request-budget field.

### Option 1 — extend `BudgetExhausted` with exact explain resources and provenance: sole survivor

**Proposal awaiting Tom's acceptance.** Keep the existing `CompileFailureClass::BudgetExhausted { resource, limit, reported }` shape and add exactly:

- `BudgetResource::ExplainDetailRecords`;
- `BudgetResource::ExplainDetailCanonicalBytes`; and
- `BudgetRefusal::ConstructionLowerBound`.

The two resource keys are `explain-detail-records` and `explain-detail-canonical-bytes`. `ConstructionLowerBound` means: construction stopped at the first detail that could not be retained; `reported` is the exact attempted prefix including that refused detail and only a lower bound on the complete trace. For the record resource, `limit = 4_096` and `reported = retained_detail_records + 1`. For the byte resource, `limit = 1_048_576` and `reported = retained_detail_bytes + encoded_refused_record_bytes`. These are the existing constants expressed in the public payload, not new values.

**Caller action and strongest statement.** The class says this build refused to construct the complete explanation because one named deterministic host-retention resource rejected the attempted prefix. It does not say the program is invalid, no plan exists, the compared value is the capacity required for success, or every build carries the same limit. A caller may reduce the trace-producing request population or choose its fail-closed fallback; a maintainer may measure compaction or a governed widening. Neither is promised to succeed by the lower bound alone.

**Correctness and strictness.** `ExplainError::DetailCapacity` must gain the exact arm/limit/reported payload at the check and only that variant may map publicly to `BudgetExhausted`; every verifier, ledger, identity, provider-authority, event-class, and stale-identity `ExplainError` remains `InvalidCompilerOutput`. The internal error chain must preserve `DetailCapacity` as its typed source. It must **not** be transmuted into `RequestError::BudgetExceeded` or reuse/generalize today's internal `CompileError::BudgetExhausted(RequestError)` carrier: `compile_candidate_target` deliberately retains that entire variant as candidate-local infeasibility, so reuse could prune the overflowing candidate and retry another candidate, numerical contract, or target. The sole narrow implementation is a distinct internal scoped explain-capacity carrier that remains `TargetCompileFailure::Outer` through every conversion and maps to the public `BudgetExhausted` shape only at the session boundary. A broader refactor of internal failure scope is outside this ticket and is admissible only after separate review proves identical request-wide abort behavior. The terminal trace keeps `compiler-failure:explain-detail-capacity`, because that is the stable construction cause; the public typed payload supplies the independent arm and quantities. No record is added, removed, summarized, or truncated, no fallback is attempted after the capacity stop, and no partial batch is returned.

**Maintainability and public surface.** This reuses the one public carrier already defined as “a deterministic budget stopped the compilation.” It extends its single resource-to-provenance authority rather than adding a parallel resource/limit/report family. The two `BudgetResource` additions are additive under `#[non_exhaustive]`. The closed `BudgetRefusal` addition is intentionally source-breaking for the total macro renderer and forces explicit caller advice. `CompileFailureClass` itself gains no new variant or payload shape. The workspace is pre-alpha, version `0.0.0`, and unpublished, so ADR 0075 assigns no external SemVer compatibility cost; the in-workspace source break is still deliberate and must be repaired atomically.

**Host cost.** Successful compilations already compute both attempted quantities to enforce the existing limits. Carrying the selected arm and three `u64`-compatible values only on the refusal path adds no enumeration, encoding, rendering, or retained-record work; its expected runtime/RSS effect is below the resolution of the existing M3 request-wide rows. This is a source-derived bound, not a new measurement claim.

**Identity, schema, and rendering.** The caps remain build constants outside `DeterministicBudgets`; `canonical_explain_subject_bytes`, request/evidence identity, and the request qualifier do not move. Classification is not encoded into explain identity, plan identity, artifact identity, or cache identity. The terminal record is unchanged, so neither `EXPLAIN_SCHEMA_VERSION` nor `EXPLAIN_RENDERER_VERSION` steps. The Rust failure value and `tiler-macros` diagnostic text change; the latter must state prefix-lower-bound semantics and must not print the number as required capacity. With the required distinct outer carrier, the same request that currently aborts still aborts at the same construction point and admits no new compilation, so there is no indirect packaged-content identity change. That conclusion does not hold for a reused candidate-local budget carrier, which is why that implementation is forbidden.

**Strongest counterargument.** The thirteen existing resources all originate in `DeterministicBudgets`; explain caps are build constants and a reader could incorrectly infer every `BudgetResource` participates in request identity. The source and public docs do not make that promise: `BudgetExhausted` already says a bound “this build declares,” while `BudgetResource` is the total report vocabulary over the authorities that can refuse. The implementation must make the new rows explicitly report-only and must not add them to `DeterministicBudgets`.

**Reversal evidence and negative controls.** Reverse this recommendation if a complete consumer audit finds a public owner that semantically equates `BudgetResource` with request-subject fields, or if Tom decides the top-level class must distinguish caller-stated budgets from build-owned diagnostic retention. Perturb resource, limit, reported, and `ConstructionLowerBound` independently; each must fail the unchanged public assertion. Force the record arm and byte arm separately one unit below their attempted prefix and then force both to prove record-first precedence. A real verifier-produced `CompilerOutputError` must remain `InvalidCompilerOutput`. Independently force explain capacity in the first of multiple otherwise viable semantic candidates, before a viable fallback numerical contract, and after an earlier target would have succeeded in a multi-target request. Each must prove the later work is never reached and the public call returns no partial batch.

### Option 2 — dedicated `ExplainCapacityExceeded` class: eliminated as dominated

The strongest dedicated surface is `CompileFailureClass::ExplainCapacityExceeded { dimension, limit, reported }` plus a two-row `ExplainCapacityDimension`. It can state the same prefix-lower-bound semantics and preserve the same trace. It is correct, but it creates a second public resource/limit/report family, a sixth top-level caller branch, a new dimension type, and parallel frontend advice while improving no correctness, strictness, identity, schema, host-cost, or unsupported-population property over option 1. `BudgetExhausted` already means a deterministic bound declared by this build stopped compilation; it does not mean a caller supplied the bound or that the bound is in request identity. With that premise re-read, the dedicated class is worse on maintainability and public surface and better on no key dimension.

Its strongest counterargument is naming locality: “explain capacity” is visible at the top-level class rather than after reading `resource`. That is presentation convenience, not a different caller action, and the typed resource already provides it without a second carrier. It would return to the frontier only if Tom narrows `BudgetExhausted` normatively to caller-stated/request-identity budgets.

### Option 3 — prove the refusal unreachable by aggregation or widening: not a present solution

Lossless source aggregation remains preferable where repeated records have identical grounds; the accepted coverage-gap repair proves that pattern once. No such proof exists for the current provider/plan population. Installed providers have no cardinality bound, provider identities and declines may be distinct, and complete-plan explanation grows combinatorially. A finite widening cannot prove unreachability over that admitted population. Aggregation or encoding compaction becomes a replacement only after it preserves every typed reason, subject, provider attribution, cause edge, expansion equality, and complete admitted request under both caps. Until then it is bounded follow-up research, not a classification repair and not grounds to leave the false public class.

The existing conditional [`measure-complete-explain-demand-and-lossless-compaction-for-full-physical-provider-activity`](measure-complete-explain-demand-and-lossless-compaction-for-full-physical-provider-activity.md) is the reversal experiment if Tom names full activity as a requirement. Its result may later make this public resource unreachable; it cannot make today's reachable class truthful retroactively.

### Option 4 — remove both hard ceilings: eliminated

This preserves records only by making retained diagnostic memory unbounded. Installed/active provider population is unbounded, native provider work before emission is unbounded, and retained frontier/plan explanation grows combinatorially. The accepted `never-truncate-the-governed-explain-trace` rationale deliberately kept both hard checks as real host-memory protection. Restoring a soft ceiling or omitted-record summary loses reasons; removing all ceilings removes fail-closed host protection. Both are worse than option 1 on correctness and RSS.

### Option 5 — defer the public surface: eliminated from the final frontier

Deferral can describe an unfinished decision but cannot make the present output correct. The valid seven-specialist request still tells the caller Tiler's verifier found a defect. Option 1 supplies an exact surface without choosing a capacity or support population, so deferral is worse on correctness and strictness and better on no implementation-independent dimension.

### Frontier and recommendation

Only option 1 remains nondominated. It is top-tier on correctness and fail-closed strictness, preserves the accepted memory guard and complete trace, adds the smallest coherent public vocabulary, and moves no request, explain, plan, artifact, or cache identity. The concrete question for Tom is therefore not a menu: **accept or reject the exact `BudgetExhausted` extension above.** Rejection keeps this ticket open and both downstream decisions blocked; it does not accept the current `InvalidCompilerOutput` behavior.

## Follow-up contract after acceptance — specifications only, not graph nodes yet

Per the dispatch boundary, no implementation/evidence ticket or downstream dependency is created before Tom accepts the exact surface. On acceptance, create these exact nodes before closing this decision:

1. `implement-the-truthful-explain-capacity-budget-refusal` — depends on this decision; scopes `implementation/compiler` and `implementation/frontend`, shared `project/tickets`. It implements only the two report-only resources, `ConstructionLowerBound`, arm/limit/reported capture with record-first precedence, a distinct source-preserving internal explain-capacity carrier that remains outer/request-wide through candidate, contract, and target orchestration, a narrow public `BudgetExhausted` mapping, explicit macro rendering, and public docs. It must not route explain capacity through `RequestError`, reuse/generalize internal `CompileError::BudgetExhausted`, permit candidate/contract/target fallback, or return partial outputs. Any broader scope refactor requires separate review proving identical abort behavior. It changes no capacity value, request budget, provider admission, trace record, identity encoder, or schema/renderer version.
2. `prove-the-truthful-explain-capacity-budget-refusal-boundary` — depends on the implementation; scopes `implementation/compiler` and `implementation/frontend`, shared `project/tickets`. It runs the seven-specialist public reproduction, independent one-below record/byte controls, simultaneous-arm precedence, independent resource/limit/reported/provenance perturbations, macro advice assertions, and a genuine verifier-output negative that remains `InvalidCompilerOutput`. It also supplies multi-candidate, contract-fallback, and multi-target negatives: capacity in earlier work must prove later viable work was not reached, and an earlier target success must not survive as a partial result. It records exact failure text, reachability counters, and population counts rather than only a green result.

Then add hard dependencies from **both** downstream tickets under **Graph** to **both** accepted nodes — four edges total — and verify the graph. Decision closure alone must never unblock either downstream ticket while implementation or evidence is incomplete.

## Required perturbations

- Reproduce the seven-specialist public request and assert that the accepted class is not `InvalidCompilerOutput` while the terminal trace still names `explain-detail-capacity`.
- Independently force the record arm and byte arm one unit below their exact attempted demand; neither may be misreported as the other.
- If `BudgetExhausted` survives, perturb resource, limit, reported, and refusal provenance independently and show the unchanged public assertion fail.
- If a dedicated class survives, perturb its dimension/action payload independently.
- Keep an actual verifier-produced invalid compiler output mapped to `InvalidCompilerOutput`, proving the repair does not erase real Tiler defects.
- Force explain capacity in the first of multiple candidates and prove a later viable candidate is not reached.
- Force explain capacity before a viable fallback numerical contract and prove the fallback is not reached.
- Force explain capacity in a multi-target request after an earlier target can succeed; prove no later target is reached and no earlier success survives in a partial output.

## Graph

Both [`decide-how-explain-capacity-bounds-active-physical-provider-populations`](decide-how-explain-capacity-bounds-active-physical-provider-populations.md) and [`calibrate-the-physical-frontier-provider-and-outcome-budgets`](calibrate-the-physical-frontier-provider-and-outcome-budgets.md) depend on this decision. The first cannot call the current wall a supported-population policy; the second cannot land a raw budget while a smaller valid request can still surface the false defect class through an independent capacity.

## Closing conditions

- Tom accepts the exact public surface, or accepts a complete source/capacity change that makes the public refusal unreachable.
- Create `implement-<accepted-truthful-explain-capacity-outcome>` with this decision as a dependency. Do not use a generic implementation ticket whose body leaves the accepted surface implicit.
- Create the accepted regression/evidence ticket depending on that implementation. It must reproduce the seven-specialist public path, both independent capacity arms, a genuine verifier defect that remains `InvalidCompilerOutput`, and multi-candidate, contract-fallback, and multi-target atomic-abort negatives proving no retry or partial output.
- Add hard dependencies from **both** `decide-how-explain-capacity-bounds-active-physical-provider-populations` and `calibrate-the-physical-frontier-provider-and-outcome-budgets` to **both** the accepted implementation and evidence tickets. Verify the graph after all four edges exist.
- Only then may this decision close. If no change survives, it remains open with the live defect rather than satisfying dependents by declaration.
