---
id: implement-the-composed-realization-evaluation-driver
title: Implement the composed realization evaluation driver
status: blocked
priority: p2
dependencies: [retain-each-plan-alternative-s-verified-semantic-candidate, define-the-composed-realization-driver-subject-bridge, accept-the-exact-composed-reference-session-and-event-surface]
related: [accept-the-composed-realization-evaluation-surface, compose-a-declared-reduction-topology-into-a-semantic-program-evaluation, decide-the-safe-cross-crate-composed-reference-boundary]
scopes: [implementation/compiler, implementation/conformance, implementation/ir, implementation/reference, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, public-boundary, conformance, reference, numerics, correctness]
claimed_from: todo
assignee: worker-composed-driver
lease_expires_at: 1786977751
---
## Source-first Fact and obligation audit — exact published base `f6f310a47e1def152313120e0beda91ef8219fd8`

- **Verified — exact checkout and claim.** The ticket worktree is on `tkt/implement-the-composed-realization-evaluation-driver` at `f6f310a47e1def152313120e0beda91ef8219fd8`, was clean before this ticket-only repair, and the live claim names `worker-composed-driver`. Its declared compiler/conformance/IR/reference/numerics scopes cover the requested implementation.
- **Verified — both implementation prerequisites are done.** `retain-each-plan-alternative-s-verified-semantic-candidate` made `ProgramAlternative.semantic` mandatory, Arc-retained, identity-bound, and reverified. `define-the-composed-realization-driver-subject-bridge` accepted the one-shot compiler visitor, exact five-event population, atomic pre-`Begin` validation, distinct visitor errors, and staged/synthetic first-slice refusal.
- **Verified — the retained candidate/plan path is mechanically available.** `build_alternative_for_origin` constructs `CoverAssembly::from_plan(P', selected_plan)`, retains the exact `Arc<SemanticProgram>`, schedule vector, verified program and identity; `verify_alternative` rederives assembly against that same owner. `PlanAlternative` retains references to both the owning `Compilation` and exact `ProgramAlternative`.
- **Verified — `CoverAssembly` is the only complete derivation authority.** Its execution order, stage bindings, actual producer/consumer dependencies, materialization values, split pairs, publishing copies, and staged/synthetic handoffs are the facts the visitor must factor. KIR and ABI views omit semantic-value attribution and cannot replace it.
- **Verified — the reference evaluator owns the required computations but not the composition protocol.** `ReferenceEvaluator::evaluate` already owns all intermediate tensors while walking one `SemanticProgram`; `strict_partial_sums_under` and `strict_partitioned_sum_under` own the declared blocked fold. The missing primitive/session must compose those paths without accepting a caller tensor for an internal `ValueId`.
- **Verified — the current split fixture never exercises a distinct `P'`.** `the_assembled_split_program_matches_the_partitioned_sum_oracle` builds and assembles the original semantic program directly, derives its prologue from `kernels[0]`, and its `semantic_case_with_axis` has only one multiply and one add before the sum, so neither ordered-reassociation rule has a same-operation chain to rewrite. The research fixture `sum((x * 0.3) * 10.0)` supplies a discriminating `P' != P` multiply reassociation before the physical two-by-two split.
- **Verified — the required semantic behavior remains unchanged.** The wrapper must evaluate exact retained `P'`, then apply stage/materialization/split order and the declared partition, with explicit registry/work authority and no strict/baseline/default fallback. Staged/synthetic values remain named refusals in the initial population. No artifact/cache/request/schedule/KIR/semantic/registry identity or schema population moves.
- **False — the accepted bridge's illustrative `IterationStepAllowance` names no repository type.** Existing public authority is `usize` on `ReferenceEvaluator::with_iteration_step_allowance`; inventing a newtype would be a new language-public boundary.
- **False — the illustrative `ReferenceOutputs` return is not an owned program result.** It is the public mutable writer for one registered `ReferenceOperation` callback, with crate-private construction/finalization. Whole-program reference evaluation returns `Vec<Tensor>`.
- **Imprecise — the accepted decisions do not fix every exact language-public spelling implementation requires.** They decide ownership, safety, event population and sequencing, but not the safe session type/method/error/output surface or the event row/accessor/iterator/error surface. Several correct spellings have different compatibility and protocol consequences. Under the public-boundary stop condition this cannot be invented in implementation.
- **Verified — the current `PlanAlternative`-backed expected-value population is narrower than the ticket prose suggests.** `serial_sum::partitioned_reference` is its only member and the only `strict_partitioned_sum` call; it folds caller operands after reconstructing a partition from ABI geometry. `loop_carried::grouped_bits` is another grouping oracle and calls `cooperative_grouped_sum`, but it constructs a `VerifiedScheduledRegion` directly under its stated `The compiler is not in the path` boundary and consumes no `PlanAlternative`. Other `PlanAlternative` consumers package proof or device dispatch evidence; no crate depends on `tiler-conformance`; no composed public symbol exists.

**Blocked correction, 2026-08-17.** [`accept-the-exact-composed-reference-session-and-event-surface`](accept-the-exact-composed-reference-session-and-event-surface.md) now owns the missing consequential public spellings, existing-type collisions, exact consumer census, and Pareto-complete decision packet. No production edit is authorized until it is accepted. The prerequisite itself stays blocked and must not be queued or presented while LiveRow is active.

## User-visible outcome

One supported test-only conformance entry computes expected bits from the exact semantic candidate and the exact ordered realization the selected plan declared, including plans that spend reassociation in both a semantic rewrite and a physical reduction split.

## Authority and prerequisites

Tom accepted the driver as the sole public composition entry through [`accept-the-composed-realization-evaluation-surface`](accept-the-composed-realization-evaluation-surface.md), retained the `ValueId` pin/observe primitive crate-private, and on 2026-08-12 fixed `tiler-conformance` as the driver's home. Implement only after the mandatory candidate retention and exact subject bridge are complete.

**Accepted correction, 2026-08-12.** The sentence above records the original intent but not the implementable boundary Tom subsequently accepted on [`decide-the-safe-cross-crate-composed-reference-boundary`](decide-the-safe-cross-crate-composed-reference-boundary.md): raw tensor-taking pin/observe remains private; a separate safe cross-crate reference session owns every internal tensor, receives explicit reference registry/work authority, and discharges only a completely witnessed freedom; and the first plan-binding conformance wrapper remains `pub(crate)` and test-only. The compiler-subject decision replaces the remaining provisional bridge spelling before implementation.

## Required delivery

- Implement the accepted test-only `tiler-conformance` entry over the complete `PlanAlternative`, explicit frozen reference registry/work authority, and declared input bindings. It must obtain every candidate/stage/value association by internally invoking the accepted one-shot `visit_composed_realization`-shaped SPI, never from caller-provided parallel arrays, detached subjects, recipes, or keys.
- Implement the compiler SPI as the exact accepted closed event census: `Begin`, execution-ordered `Stage`, semantic `Materialization`, multi-pass `Split`, and `Complete`. Factor and reuse the `CoverAssembly` derivation, compare the reconstructed regions to the retained schedules/program stages, validate the complete owner/identity/handle/dependency census before `Begin`, and surface callback failures separately. Refuse staged or synthetic assembly values with no semantic `ValueId` by a named typed cause in the first slice; never coerce their assembly ordinals into semantic handles.
- Implement the reference evaluator's crate-private raw pin/observe primitive plus the safe language-public cross-crate composed-evaluation session. The session accepts no caller-provided internal tensor and owns the observation/fold/pin chain. Typed refusals cover invalid or unreachable values, type/shape disagreement, incomplete witness discharge, unsupported freedoms/topologies, and registry/subject mismatch.
- Drive the retained `P'` through the plan's ordered stage/materialization sequence and the existing declared-order fold evaluators. Refuse every unsupported population from the composed-realization record by name; never silently run the strict baseline interpretation.
- Repair `the_assembled_split_program_matches_the_partitioned_sum_oracle` so its expected prologue comes from `P'`, not `kernels[0]`. Replace or extend the fixture so `P' != P` and both semantic and physical reassociation change the exact bits.
- Keep `tiler-conformance`'s no-public-surface and test-only contract. [`activate-a-public-composed-realization-oracle-for-a-named-consumer`](activate-a-public-composed-realization-oracle-for-a-named-consumer.md) owns any later reusable public entry.
- Keep artifact, cache, request, schedule, KIR, and semantic identity bytes unchanged.

## Watched failures

Feed a baseline program in place of `P'`; swap two alternatives' stage subjects; reverse or omit one materialization; pin one implementation-produced tensor; remove one reference observation; and restore the existing `kernels[0]` provenance. Each independent perturbation must fail with the expected typed rule and quoted output.

## Non-goals

Artifact-only replay, a public pinning primitive, a plan type in `tiler-reference`, tolerance comparisons, device-produced oracle inputs, or a new schedule evaluator.

## Closes when

The complete accepted population evaluates through the one `pub(crate)` test-only supported wrapper, the closed compiler event stream and safe reference session are total over that population, every named unsupported case refuses, the provenance regression is discriminating, no raw public composition/pinning entry exists, and targeted compiler/reference/conformance checks plus exact-base guard are green.
