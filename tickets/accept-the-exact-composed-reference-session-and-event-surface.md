---
id: accept-the-exact-composed-reference-session-and-event-surface
title: Accept the exact composed-reference session and event surface
status: blocked
priority: p1
dependencies: [derive-staged-combine-structure-from-program-scope, join-the-scheduled-region-into-the-contraction-witness]
related: [implement-the-composed-realization-evaluation-driver, define-the-composed-realization-driver-subject-bridge, decide-the-safe-cross-crate-composed-reference-boundary, accept-the-realization-witness-surface, accept-the-composed-realization-evaluation-surface]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, conformance, reference, numerics]
---
## User-visible outcome

Tom accepts or redirects the exact Rust surface by which the compiler's already-accepted composed-realization event stream drives the reference-owned, no-internal-tensor session. The implementation then has one unambiguous public spelling for every event descriptor, session transition, owned output, and typed refusal, without colliding with an existing reference type or inventing an unstated work-authority type.

## Dispatch hold

## Hold released — 2026-08-22, its trigger fired

The hold below required "an explicit coordinator observation that LiveRow is no longer active". **That observation is now made.** `.ticketsplease/decision-queue.md` item 6 records *Source-bound live-row-major access — ACCEPTED 2026-08-18*, `RESOLVED … option 1, the fieldless contextual marker`. The queue's companion gate is also clear: item 10's policy question was accepted 2026-08-18 and is recorded on `calibrate-the-physical-frontier-provider-and-outcome-budgets` under `## Accepted policy — 2026-08-18`.

**What this releases is a presentation, not a worker lane.** The next step is re-running the decision-packet readiness gate and moving to `awaiting-decision` for Tom — not implementation. Re-audit the packet's Facts first; it has `dependencies: []` and was written 2026-08-17, before several landings.

**Blocked, 2026-08-17.** Do not queue or present this packet while the current LiveRow decision is active. The trigger is an explicit coordinator observation that LiveRow is no longer active; only then may this move to `awaiting-decision`. This hold changes no technical conclusion below.

## Source-first Fact audit — exact published base `f6f310a47e1def152313120e0beda91ef8219fd8`

> **Superseded as a current statement on 2026-08-22.** The bullets below are retained as accurate history about base `f6f310a4` and about what was believed there; they are **not** the current Fact list. Two of them were false when written and three are imprecise at this base. Read `## Re-audit at base 7be3bce6` below before relying on any bullet here, and repair rather than restate.

- **Verified — the safety architecture is decided, but the language-public reference surface is not spelled.** [`decide-the-safe-cross-crate-composed-reference-boundary`](decide-the-safe-cross-crate-composed-reference-boundary.md), anchors `safe composed-evaluation session` and `Decision — accepted 2026-08-12`, fixes ownership and authority: the session is plan-neutral, owns every internal tensor, accepts the exact retained program, declared inputs, typed value/fold descriptors, an explicit frozen registry and explicit work allowance, and refuses unsupported freedom. It names no Rust session type, constructor, transition method, output type, or error enum. Those choices are consequential language-public boundary, not private implementation detail.
- **False — `IterationStepAllowance` is not an existing type.** The illustrative wrapper in [`define-the-composed-realization-driver-subject-bridge`](define-the-composed-realization-driver-subject-bridge.md), anchor `allowance: IterationStepAllowance`, names a type absent from the repository. `ReferenceEvaluator` instead stores a `usize`; `ReferenceEvaluator::with_iteration_step_allowance(self, usize)` is the accepted public authority spelling, `ReferenceEvaluationRequest::iteration_step_allowance` returns `usize`, and every conformance caller supplies a `usize`. Reproduction: `rg -n 'IterationStepAllowance|with_iteration_step_allowance' crates tickets/define-the-composed-realization-driver-subject-bridge.md`.
- **False — the illustrated wrapper's `ReferenceOutputs` is not an owned whole-program result.** `pub struct ReferenceOutputs` in `crates/tiler-reference/src/registry.rs`, anchor `Host-owned bounded output writer for one reference callback`, is the mutable callback writer passed to `ReferenceOperation::evaluate`; its constructor and `finish` are crate-private. `ReferenceEvaluator::evaluate` returns `Result<Vec<Tensor>, EvaluationError>`. Reusing `ReferenceOutputs` for the wrapper would expose the wrong protocol and collide with the public item already used by every registered reference capability. Reproduction: `rg -n 'pub struct ReferenceOutputs|fn finish|pub fn evaluate' crates/tiler-reference/src/{registry,evaluate}.rs`.
- **Verified — the exact compiler event population is accepted but its public descriptor/accessor surface remains intentionally unspecified.** [`define-the-composed-realization-driver-subject-bridge`](define-the-composed-realization-driver-subject-bridge.md), anchors `Recommended exact boundary` and `Names remain implementation-level`, accepts `PlanAlternative::visit_composed_realization`, a closed `Begin` / `Stage` / `Materialization` / `Split` / `Complete` census, private fields, borrowed accessors, atomic pre-`Begin` validation, and distinct `Visitor(E)`. It does not decide whether variants carry public opaque row types or private-field structs, the exact accessor names and return types, how ordered semantic atoms and consumers are iterated, or the complete compiler-subject error vocabulary. An exhaustive sibling-crate match cannot be written until those spellings exist.
- **Verified — `serial_sum` is the only current `PlanAlternative`-backed expected-value path and the only `strict_partitioned_sum` caller.** In `crates/tiler-conformance/src/serial_sum.rs`, `partitioned_reference` receives a `ContributorPartition` reconstructed by `declared_partition` from a `PlanAlternative`'s ABI and applies it directly to caller operands. That module itself says this is sound only while the pointwise prologue is bit-identity. The new wrapper must replace that conditional provenance for the grouping-sensitive plan comparison. `crates/tiler-conformance/src/loop_carried.rs` is a second grouping oracle: `grouped_bits` calls `tiler_reference::cooperative_grouped_sum` for an explicit participant/round/contributor grouping, but its module header states `The compiler is not in the path`; it builds a `VerifiedScheduledRegion` directly and consumes no `PlanAlternative`. The other `PlanAlternative` consumers in `publication.rs` and `publication/proof.rs` package or classify proof evidence and do not compute a composed expected value. Reproduction: `rg -n 'strict_partitioned_sum|cooperative_grouped_sum|PlanAlternative' crates/tiler-conformance/src --glob '*.rs'`.
- **Verified — no downstream crate currently consumes `tiler-conformance`.** `cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.dependencies[].name == "tiler-conformance") | .name'` prints no package. `crates/tiler-conformance/src/lib.rs`, anchor `There is none`, keeps all modules under `#[cfg(test)]`. The plan-binding wrapper therefore remains `pub(crate)` and test-only; this ticket does not activate the deferred reusable public oracle.
- **Verified — no composed public names already exist to preserve.** `rg -n 'evaluate_composed|ComposedConformance|ComposedRealization' crates --glob '*.rs'` prints no match. This is freedom to choose deliberately, not evidence that any choice is compatible.
- **Verified — the retained candidate and assembly authority are ready.** `ProgramAlternative.semantic` is mandatory and owner-checked under `portfolio-retained-semantic-binding`; `CoverAssembly::from_plan` remains the single derivation of execution-ordered stages, bindings, dependencies, splits, publishing copies, and staged handoffs. This ticket changes neither prerequisite nor their identities.
- **Verified — no identity/schema step follows from the boundary.** The compiler event projection and reference session are on-demand, non-serialized, non-cached values. `ProgramAlternativeIdentity` already folds the retained semantic identity and selected-plan identity. No candidate here changes artifact, cache, request, schedule, KIR, semantic, registry, or canonical identity bytes.

## Re-audit at base `7be3bce6ae67253736af25d76dca88fa687c9e8e` — 2026-08-22, by `worker-session`

Every Fact re-run at this base before any edit, per the stale-Facts rule. `dependencies: []` meant nothing forced this, and 105 commits landed between the two bases, including [ADR 0112](../docs/decisions/0112-replace-the-strict-contraction-key-with-a-permission-indexed-successor.md), accepted 2026-08-18 — which is the single largest change to this packet's subject.

| # | Original claim, abbreviated | Verdict at `7be3bce6` |
| --- | --- | --- |
| 1 | Safety architecture decided; language-public reference surface unspelled | **Verified** |
| 2 | `IterationStepAllowance` is not an existing type | **Verified**, conclusion now **imprecise** |
| 3 | `ReferenceOutputs` is the callback writer, not an owned whole-program result | **Verified** |
| 4 | Event population accepted; descriptor/accessor surface unspecified | **Verified**, but **incomplete** |
| 5 | `serial_sum` is the only `PlanAlternative`-backed expected-value path and the only `strict_partitioned_sum` caller | **False**, in two separate clauses |
| 6 | No downstream crate consumes `tiler-conformance` | **Verified** |
| 7 | No composed public names already exist | **Imprecise**; the method was unsound |
| 8 | Retained candidate and assembly authority ready | **Verified** |
| 9 | No identity/schema step follows | **Verified for candidate 1 only**; now conditional |

### Fact 1 — verified

Both anchors resolve exactly once against [`decide-the-safe-cross-crate-composed-reference-boundary`](decide-the-safe-cross-crate-composed-reference-boundary.md) (`safe composed-evaluation session`, `Decision — accepted 2026-08-12`); the ticket is `done`. `rg -n 'visit_composed_realization' crates` still prints no match, so no Rust session type, constructor, transition, output type, or error enum has been spelled. Unchanged.

### Fact 2 — verified, but its conclusion no longer holds

The type check is sound: `IterationStepAllowance` occurs in the repository **only** in ticket prose — [`define-the-composed-realization-driver-subject-bridge`](define-the-composed-realization-driver-subject-bridge.md), this ticket, and [`implement-the-composed-realization-evaluation-driver`](implement-the-composed-realization-evaluation-driver.md) — and never under `crates/`. `ReferenceEvaluator::with_iteration_step_allowance(self, allowance: usize)` and both `iteration_step_allowance()` accessors return `usize`, read in full.

**What is now imprecise is the inference drawn from it.** The original bullet reads as though `usize` were *the* repository precedent for reference work authority. Since ADR 0112 it is one of two, and the newer accepted one is a typed struct: `ContractionF32ReferenceBudget` in `crates/tiler-reference/src/contraction/topology.rs`, anchor `The caller-owned resource budget of one topology evaluation`, carries four named resources, validates them in `new`, and deliberately has **no `Default`** — ADR 0112 states it `takes a caller-owned four-resource budget the crate never defaults`. So a newtype or budget struct for composed work authority is now the *better*-precedented choice, not the exceptional one. Deliverable 2 already admits "or an explicitly accepted newtype"; that alternative should be presented to Tom on equal footing rather than as a deviation.

### Fact 3 — verified by full read

`crates/tiler-reference/src/registry.rs`, anchor `Host-owned bounded output writer for one reference callback`: `ReferenceOutputs::new` is `pub(crate)`, `finish` is `pub(crate)`, only `push` is `pub`. `ReferenceEvaluator::evaluate` returns `Result<Vec<Tensor>, EvaluationError>`. The collision argument stands unchanged.

### Fact 4 — verified, and incomplete in a way that helps the packet

Both anchors resolve once each. The census and the deliberate silence on spellings are as described.

**What the bullet misses is that the spelling question is not open.** `crates/tiler-compiler/src/session/realization.rs` — unchanged since the packet base, so this was missable then too — already ships the house idiom for exactly this problem, in exactly the module a composed event surface would live in: `DeliveredRealizationView<'a>` (anchor `The complete delivered-realization view of one selected plan`) plus `SelectedObligation`, `SelectedEvidence`, and `SelectedScalarArithmetic`. Its header states the rule under the anchor `and constructor-free` — the full heading carries backticks around `Copy`, so quote the short fragment: every view exposes no `Arc`, no vector, no constructor, and no canonical encoder, so a consumer cannot forge a compiler-verified fact. Collections are returned as `impl ExactSizeIterator`, and accessors are `const fn` returning borrows.

That header also contains the argument the packet's candidate 2 needs and does not cite: the view is **one** view rather than three iterators a caller zips itself, "because the total boundary has to cross-check policy subjects, all-dimension coverage, obligation associations, and the evidence pool together, and three iterators can be zipped wrongly." That is a landed, accepted precedent for the packet's own objection to caller-zipped parallel arrays.

### Fact 5 — false, in two separate clauses

**Clause A, "the only `strict_partitioned_sum` caller" — false, and false when written.** The bullet's reproduction command is scoped `crates/tiler-conformance/src`, and a scoped grep was presented as a repository-wide population. Anchored counts at this base (`rg -P '\bstrict_partitioned_sum\b(?!_)'`, counting **call sites**, not lines): ten call sites. One is `crates/tiler-conformance/src/serial_sum.rs:861` inside `partitioned_reference`. Five are in `crates/tiler-reference/src/tests.rs`. **Four are in `crates/tiler-compiler/src/pipeline/tests.rs`** — the crate the packet proposes to add the event surface to. All nine non-conformance callers existed at `f6f310a4`, so this is an audit error rather than drift.

The compiler-side population matters on its merits, not just as a count. `crates/tiler-compiler/src/pipeline/tests.rs`, anchor `fn the_assembled_split_program_matches_the_partitioned_sum_oracle`, already runs a **three-stage assembled split program** against the partitioned-sum oracle bit for bit. It obtains grouping-sensitivity without any event stream, by reading the region's own declared coverage partition and passing two scalars to the free function. Its own doc states the limit that keeps it from being the composed comparison: "The oracle's input is the program's own prologue output rather than a value re-derived here." That is the same conditional provenance `serial_sum` carries, relocated — the oracle is fed an *executed* prologue rather than a reference-computed one. Naming this correctly strengthens the ticket: the gap is not that nobody compares a plan's grouping, it is that **every existing comparison hands the oracle a value the device produced.**

**Clause B, "`publication.rs` and `publication/proof.rs` … do not compute a composed expected value" — false as written.** `crates/tiler-conformance/src/publication/proof.rs` does compute a plan-derived expected value: `conformance_of(plan)` reads the realization off `plan.kernels()` and the subject off `plan.delivered_realization().scalar_arithmetic()`, joins them through `ReferenceNumericalConformance::from_realization`, and `reference_bits` then calls `ReferenceEvaluator::under(...).evaluate(program, &bindings)`. What it does not compute is a *grouping-sensitive* value — `evaluate` is a strict left fold. The distinction the bullet needed is grouping-sensitivity, not whether an expected value is computed at all.

This clause is the most consequential repair, because `conformance_of` is described in its own source as `The checked bridge this route was missing`: it is an existing, tested plan-to-conformance bridge that reads both `from_realization` arguments off one plan and cross-checks them. **The composed session must reuse it, not reinvent it** — and the reason it cannot serve the composed case unmodified is precisely located below.

**Clause C, the `serial_sum` and `loop_carried` descriptions — verified, with one omission.** `declared_partition` does reconstruct a `ContributorPartition` from the plan's ABI, and `partitioned_reference` does apply it to caller operands; the module says `that is sound only while` `x * 1.0 + 0.0` is bit-identity. The bullet omits the mitigation the same sentence states: the grouping-sensitive run's calibration step **checks** it, by requiring the degenerate partition's answer to equal the reference evaluator's whole-program answer. So the provenance is conditional-and-guarded, not conditional-and-unguarded. `loop_carried`'s `The compiler is not in the path` anchor resolves once and its description is accurate.

### Fact 6 — verified three ways

`cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.dependencies[].name == "tiler-conformance") | .name'` prints nothing; no `Cargo.toml` in the workspace declares the dependency; and `crates/tiler-conformance/src/lib.rs`, read in full, keeps every module `#[cfg(test)]` and contains no `pub use` at all under the anchor `There is none`. Unchanged.

### Fact 7 — imprecise, and the method was unsound

The three searched tokens (`evaluate_composed`, `ComposedConformance`, `ComposedRealization`) genuinely return no match. But this is a negative claim resting on three guessed names, and the conclusion drawn — "freedom to choose deliberately" — does not follow. The adjacent surface is spelled `DeliveredRealization`, which none of the three patterns could reach: `tiler-compiler::session` already exports `DeliveredRealizationView`, `SelectedObligation`, `SelectedEvidence`, and `SelectedScalarArithmetic` (Fact 4 above). There is no *collision* on the three tokens, but there is an established idiom and a family of adjacent names in the exact module, so the correct statement is **"no collision, and an idiom to follow"** rather than "freedom to choose deliberately".

### Fact 8 — verified by full read

`crates/tiler-compiler/src/pipeline.rs`: `ProgramAlternative.semantic` is a private, non-`Option` field whose doc states "Missing candidate retention is unrepresentable: the field is mandatory, private, and never an `Option` or a side table." `CoverAssembly` has exactly two constructors — `from_plan` in `crates/tiler-compiler/src/program.rs` (two production call sites, in `pipeline/planning.rs` and `pipeline/verify.rs`) and `stated`, which is `#[cfg(test)]` and whose own doc says "**The compiler is not in the path**"-style: "[`Self::from_plan`] is its only derivation, and every program this crate compiles goes through it." Unchanged.

### Fact 9 — verified for candidate 1, but it is now a conditional rather than a general claim

`ProgramAlternativeIdentity::new` folds the origin, all five `SemanticIdentity` components, the numerical-contract key, and the plan identity, under the `tiler.program-alternative.v2` tag. For an on-demand, non-serialized, non-cached event projection, no identity or schema step follows. That much is verified.

**It cannot be stated as a property of the boundary, only of one candidate.** `crates/tiler-ir/src/program/contraction_witness.rs` records the repository's accepted policy for the witness-shaped alternative: a future live- or coordinate-dependent tree mapping `must become identity-bearing in` schedule/kernel/artifact encoding and gain a new witness representation rather than reusing that constructor. Candidate 5 below is witness-shaped, so it does carry an identity obligation. Fact 9 is therefore scoped to candidate 1 and must not be quoted as covering the packet.

### Two further drifts worth recording

**The reference refusal vocabulary widened.** `UnsupportedReferenceContract` gained `ReciprocalTransformPermitted` and `ApproximateIntrinsicsPermitted { envelope }` since the packet base. Deliverable 2's "complete typed error vocabulary" is measured against a larger set than when it was written.

**Two ADRs this packet touches are being corrected on `main` right now, ahead of this base.** The tiled-contraction lane withdraws a status sentence whose backtick-free tail is `requires the permission outright for the cooperative contraction` from the implementation-status sections of [ADR 0012](../docs/decisions/0012-physical-reduction-topology.md) and [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md). **Unverified at this base by design** — this base predates that merge, and the statement above was read from `origin/main`, not from the worktree. No conclusion in this packet rests on it, but a worker must re-read both records at its own base. If the cooperative contraction no longer requires the permission outright, the population of composed cases evaluable under a reassociation-forbidden conformance *widens*, which is favourable to both frontier candidates and moves neither relative to the other.

## What changed since `f6f310a4`, and why it moves the packet

ADR 0112 landed an accepted, implemented, end-to-end answer to the same *class* of problem this packet poses: how a plan's chosen evaluation order reaches an independent reference without the reference depending on the compiler. Its shape is not the packet's.

- The plan's order becomes a **validated opaque value in the shared IR**: `ContractionF32PlanWitness` in `crates/tiler-ir/src/program/contraction_witness.rs`, anchor `One validated plan-owned combine tree, bound to its exact subjects`, constructed only by `from_program(&SemanticProgram, &VerifiedKernelProgram, SemanticOccurrence, ContractionF32TopologyLimits)` and `derived from the program's own verified` records rather than trusted from a producer.
- The reference consumes it through a **complete borrowed request, an owned result, and an exhaustive typed error**: `ContractionF32TopologyEvaluationRequest::new` takes every subject as an explicit required argument, `ContractionF32TopologyEvaluation` owns the tensor and its identities, and the evaluator itself is reached only via `FrozenReferenceRegistry::contraction_f32_topology_evaluator() -> Result<_, ContractionF32TopologyEvaluatorUnavailable>`.
- Composition happens in a **test in `tiler-compiler`** (`crates/tiler-compiler/tests/contraction_topology_witness.rs`), which compiles through the ordinary entry point, derives the witness, and evaluates it. `tiler-reference` gains no compiler dependency, and there is no event stream, no visitor, no stateful session, and no `#[doc(hidden)]`.

**The unifying principle, which the packet predates and should now state.** `crates/tiler-reference/src/evaluate.rs` says it directly for the fold family: `The grouping discharges the reassociation obligation`; the conformance argument discharges only the subnormal modes, and the two are independent. ADR 0112 formalized the same split for the governed contraction. So the repository has converged twice, independently, on: **pass the plan's declared order to the reference as a validated value, and keep the numerical conformance reassociation-forbidden.**

**This is also where the composed gap now sits, exactly.** `ReferenceNumericalConformance::from_realization`, read in full, returns `UnsupportedReferenceContract::ReassociationPermitted` unconditionally for a reassociating realization, and it is the only constructor that produces a stated `ConformanceSubject`. `proof.rs::conformance_of` therefore cannot bridge a reassociating plan today — not because the bridge is wrong, but because no composed-order *value* exists for the whole-program case to discharge the obligation with. That is the one-sentence statement of what this ticket is for, and it was not in the packet.

## Decision packet — re-derived at `7be3bce6`

### Required exact deliverable

Unchanged in substance from the four numbered items above, with three amendments: item 2's work-authority parameter must be presented as `usize` **or** a typed budget on equal footing (Fact 2); item 2's "complete typed error vocabulary" is measured against the widened `UnsupportedReferenceContract`; and item 1's spellings must follow the `session::realization` idiom — borrowed, `Copy`, constructor-free rows, `const fn` accessors returning borrows, collections as `impl ExactSizeIterator` — rather than choosing freely (Facts 4 and 7).

### Materially distinct candidates

Candidates 1 through 4 are carried forward from the 2026-08-17 packet; candidate 5 did not exist at that base.

1. **Opaque stateful session plus opaque compiler event rows.** As originally stated.
2. **One public reference call over an owned descriptor recipe.** Still dominated by candidate 1 for the reason the original packet gave, and the `session::realization` header now supplies a landed precedent for that reasoning: one view rather than three iterators, because three iterators can be zipped wrongly.
3. **Expose the raw pin/observe primitive or reuse `ReferenceOutputs`.** Still rejected on Fact 3, re-verified.
4. **Strict/default evaluation, or deferral.** Still rejected; deferral remains the fallback and is the current implementation state.
5. **Composed-order witness value plus a bounded registry-obtained evaluator — the ADR 0112 shape, new at this base.** A validated composed-order witness is derived from the verified kernel program and the retained semantic candidate, in the shared IR, refusing every topology it cannot derive from program scope. `tiler-reference` gains a bounded evaluator reached only from a `FrozenReferenceRegistry` through a typed unavailability error, taking one complete borrowed request and returning an owned evaluation. No event stream, no session transitions, no completion protocol, no `#[doc(hidden)]`.

### Eliminations, and one that is *not* an elimination

Candidates 2, 3, and 4 are eliminated as before. **Candidate 5 is not dominated by candidate 1 and candidate 1 is not dominated by candidate 5**, which is the substantive change from the 2026-08-17 packet's conclusion that candidate 1 was the sole nondominated shape. That conclusion does not survive this base.

Candidate 5's blocking prerequisite is real and must not be treated as implementation detail. `contraction_witness.rs` refuses, under `TopologyUnsupported`, any kernel that `declares workgroup staging`, because the program does not carry its intra-workgroup combine structure — which is exactly the cooperative/staged shape the composed comparison targets. Making it derivable means making staged combine structure program-scope and, per the same module, identity-bearing in schedule/kernel/artifact encoding. That is a schema and identity migration, and it belongs in the ticket graph before candidate 5 could be chosen.

### The frontier

| | Candidate 1 — event stream into a stateful session | Candidate 5 — composed-order witness value |
| --- | --- | --- |
| Correctness | Atomic pre-`Begin` validation, then per-event checks | Whole witness validated once at construction; holding one *is* the evidence |
| Fail-closed strictness | Exhaustive event match; a new event stops the adapter | Exhaustive typed refusal; every underivable topology refused by name |
| Failure modes the shape admits | Omitted, duplicated, reordered, or truncated events; a stop without `Complete` | None of these exist — there is no protocol to violate |
| Public surface | A `#[doc(hidden)]` session with transitions and invalid intermediate states | Two opaque values and one evaluator method; no transitions |
| Precedent | None; invents a boundary | ADR 0112, accepted 2026-08-18, landed and tested end to end |
| Identity/schema | **None** (Fact 9) | **Requires** program-scope, identity-bearing staged combine structure |
| Host memory | On-demand; nothing retained | Retains the composed projection as a value |
| Prerequisite work | None beyond the surface itself | A schema/identity migration, unscheduled |

Neither is worse than the other on every dimension: candidate 1 buys "no identity step and no prerequisite" with a protocol that has four distinct ways to be violated; candidate 5 buys "no protocol and an accepted precedent" with a schema migration. Both are top-tier on correctness and strictness. This is a genuine trade-off and it is the one to put to Tom.

### The one concrete question for Tom

**Should the composed comparison follow ADR 0112's witness shape — accepting a program-scope, identity-bearing encoding of staged combine structure as a prerequisite — or take the compiler event stream into a reference session, which needs no identity step but invents a boundary with a completion protocol?**

The recommendation is **candidate 5, conditional on Tom accepting the prerequisite as scheduled work rather than as scope creep.** The reasoning is that the repository has now twice converged on discharging an order obligation with a validated value, that a value has no protocol to get wrong, and that ADR 0112's shape is the one a later reader will expect to find. If the prerequisite is unacceptable, candidate 1 is the correct answer and should be accepted as originally drafted with the Fact 2, 4, and 7 amendments applied.

**This packet is not ready to present as an accept-or-redirect on candidate 1.** Presenting it that way would ask Tom to accept a surface as sole-nondominated on the strength of an enumeration that predates the ADR which answers the same question differently.

### Strongest counterargument, reversal evidence, and negative controls

**Against candidate 5:** ADR 0112's witness is per-occurrence and static-`K`; generalizing it to a multi-stage composed program is not a small step, and the module's own refusal list says the composed shape is exactly what it cannot derive today. Reversal evidence: a derivation showing staged combine structure is already recoverable from program scope without an encoding change would remove the prerequisite entirely and make candidate 5 dominant. Someone should attempt that derivation before the question reaches Tom.

**Against candidate 1:** it invents a public boundary that no accepted decision asks for, three days after an accepted decision answered the same question another way. Reversal evidence: a demonstration that the composed order genuinely cannot be represented as a value — that it is inherently a stream — would make the event surface necessary rather than chosen. No such demonstration exists at this base.

**Negative controls, with reachability stated.** The original perturbation list is retained and each entry was checked for whether it can actually fail. Omit, duplicate, reorder, and truncate an event, and fail the callback after `Begin`, are reachable **only under candidate 1** — under candidate 5 there is no event to perturb, which is the point of the comparison and must not be scored as candidate 5 passing them. Foreign same-shape `OperationId`/`ValueId`, a mismatched program owner, and an altered arithmetic subject are reachable under both and must be perturbed separately, since a perturbation that reddens everything cannot show which assertion is load-bearing. Restoring `ReferenceOutputs` as the wrapper output is reachable under both and is a compile-fail control. The `P' != P` fixture is reachable under both.

**One control the original list does not contain, and should.** Feed the oracle the *executed* prologue output instead of a reference-computed one, and require the composed comparison to still refuse or disagree. Without it, a candidate that quietly reintroduced the conditional provenance `serial_sum` and `the_assembled_split_program_matches_the_partitioned_sum_oracle` both carry would pass every other check on this list.

### Follow-up tickets this packet requires

- **A bounded derivation spike**, before presentation: can staged combine structure be derived from program scope without an encoding change? Its answer decides whether the frontier has one candidate or two.
- **A schema/identity ticket** for program-scope, identity-bearing staged combine structure — required by candidate 5, and must exist as a graph node before candidate 5 can be chosen.
- **A repair ticket for the conditional provenance in `crates/tiler-compiler/src/pipeline/tests.rs`**, whose grouping oracle is fed the program's executed prologue output. Out of scope here, discovered by this audit, and independent of which candidate wins.
- **`related` edges — repaired in this commit, not deferred.** [`accept-the-realization-witness-surface`](accept-the-realization-witness-surface.md) and [`accept-the-composed-realization-evaluation-surface`](accept-the-composed-realization-evaluation-surface.md) are both `done` and were named in the `related` lists of both sibling tickets but not in this one's; they are now added to the frontmatter above.
- **A decision-queue entry.** `.ticketsplease/decision-queue.md` has no item for this packet at this base.

## Non-goals

Implementing the surface; making `tiler-conformance` public; adding a plan type to `tiler-reference`; exposing raw pins; artifact-only replay; tolerance comparison; device-produced oracle inputs; a new identity/schema/domain/version; or deciding any LiveRow surface while that round is active.

## Closes when

After the dispatch hold clears, Tom accepts or redirects every exact public item and error/output collision above with provenance; the accepted packet leaves no placeholder type or implementation-level descriptor spelling for the implementation ticket to invent; and that ticket can re-audit the accepted names at its then-current base.

## Re-gate input — 2026-08-22, the spike answered and the frontier did *not* collapse

[`derive-staged-combine-structure-from-program-scope`](derive-staged-combine-structure-from-program-scope.md) reported **not derivable**, proven by an executable spike rather than argued: two verified regions over one subject, differing only in tile round structure, produce identical program-scope observations while declaring different associations of the same contributors — different binary32 computations. So candidate 5's premise holds and **two candidates survive**.

**But candidate 5 is materially cheaper than this packet assumed, and that changes the comparison rather than the option set.** The packet costed its prerequisite as a schema/identity migration, on the witness module's own statement that a coordinate-dependent tree mapping must become identity-bearing. The spike found the structure is *already* encoded, identity-bearing, and tag-injectivity-tested at the schedule layer — `RealizationWitness` publishes `contributor_partition()`, `arrival()`, `rounds()`, and `accumulation()`, all four verified present by the coordinator at `470004be`. What is missing is a **join**, not an encoding: the witness retains only a `RegionId` and an opaque identity and never reaches the region that states the tree. The prerequisite ticket has been rescoped accordingly to [`join-the-scheduled-region-into-the-contraction-witness`](join-the-scheduled-region-into-the-contraction-witness.md) — **no new encoding, no identity-domain step**.

**Consequence for presentation.** The `## The one concrete question for Tom` above weighs candidate 5's prerequisite as an accepted-schedule-work identity migration. That weighting is now wrong in candidate 5's favour, and this packet **must not be presented until it is re-derived** — presenting it would ask Tom to choose between two options while overstating the cost of one. Re-run the readiness gate's comparison step with the corrected prerequisite before this returns to `awaiting-decision`.

**Unverified, carried from the spike and stated rather than omitted:** its executable pair is a *reduction*, not a contraction, because the compiler builds no cooperative contraction today. Transfer to the contraction witness is an inference resting on a read, not on execution. Whether a hand-built program could pair a contraction with a staged kernel went unmeasured, and `from_program` is public.
