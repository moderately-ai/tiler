---
id: record-the-four-surface-optimizer-invariant
title: Record the four-surface optimizer invariant in the contracts
status: in-progress
priority: p1
dependencies: []
related: [implement-transactional-rewrite-engine, route-the-compile-path-through-the-rewrite-engine, emit-analytical-costs-through-the-typed-cost-vocabulary, drive-an-external-physical-implementation-provider-through-compilation]
scopes: [contracts/optimizer, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, optimizer, architecture, backend-providers]
claimed_from: todo
assignee: worker-invariant
lease_expires_at: 1785598277
---
## User-visible outcome

The property that keeps the optimizer rewrite-proof across execution tiers and backends is a contract sentence a worker inherits, instead of a synthesis a reader must assemble from five documents — so the first landing that violates it is caught by review against a named rule rather than by someone noticing the drift.

## The invariant, as decided

**Fact — Tom set the direction on 2026-08-01:** physical-plan optimization must operate generically over every execution tier and backend, so that optimizer and selection logic is never rewritten when a device family arrives. The derivation he reviewed states the enforcing invariant:

The optimizer sees exactly four surfaces and nothing else —

1. **Neutral alternatives**: schedules in the execution-axis / tile / synchronization vocabulary of `crates/tiler-ir/src/schedule/`, never a backend construct.
2. **Typed permissions**: reassociation, contributor permutation, and FMA-contraction consumed from the operation's own registered numerical contract — legality is target-independent.
3. **Feasibility queries**: whether a target realizes an alternative is answered from typed profile data (atomic realization facts in the ADR 0043/0076 shape), never by calling backend code; a target lacking a tier starves those alternatives at feasibility with an explainable reason rather than forking enumeration.
4. **Typed costs**: the analytical cost vocabulary, with hard feasibility never expressed as a cost.

Backends contribute *data* (facts, realizations) and *alternative generators* (providers whose output re-enters the neutral vocabulary and passes the same verifier and feasibility), never search logic. ADR 0090 items 1 and 2 are the accepted authorities for the two halves; this ticket records the composed consequence where optimizer workers read.

## What to write, and where

- The invariant stated once in the optimizer contract (`docs/compiler/` — `fusion-and-scheduling.md` or the contract document the corpus treats as the optimizer's home; read the existing structure and place it where enumeration and selection are described), with each of the four surfaces citing the implementation that carries it today.
- One sentence in `docs/architecture.md`'s separation-of-concerns text linking to it, since the invariant is the optimizer-specific instance of the compiler-core-independence rule already there.
- The review obligation stated explicitly: nothing mechanical checks this — an execution-tier or backend landing that touches selection machinery is the signal a reviewer must treat as a violation until justified. Cite the evidence that the discipline holds: the cooperative-workgroup tier (2026-08-01) landed without touching selection, and the tree-reduction strategy is required to do the same.
- Do not restate the schedule vocabulary or the provider seam; cite them. A restatement is a second authority that drifts.

## Closes when

The invariant is stated in the optimizer contract with its four surfaces cited to living code, the architecture contract links to it, the review obligation is explicit, and no sentence restates what a cited authority already owns.

## Outcome

**Where it landed, and why there.** `docs/compiler/optimizer.md`, as a new `## The four surfaces the optimizer may consult` section placed immediately after `## Planning model` and before `## Named stages and verifier boundaries`. Three documents were candidates and the corpus's own ownership boundaries decide between them: `fusion-and-scheduling.md` owns "fusion-region formation and schedule candidate generation, legality queries, and split-plan retention", and `cost-model.md` owns "ranking objectives, feature definitions, calibration provenance"; only `optimizer.md` owns "planning phases, rule contracts, **alternative retention**, search bounds, **costing inputs**, and explainability", which is the union the four surfaces span. `docs/README.md` corroborates: it heads the "Understand optimization" path and is the first of the three compiler documents in the numbered reading order (entry 7, ahead of fusion-and-scheduling at 8 and cost-model at 9). Within the file, the placement is where enumeration and selection are described: `## Planning model` carries the pipeline diagram from normalization through global selection and the four distinctions (logical equivalence, fusion legality, physical feasibility, profitability), and the eleven named stages follow immediately after, so every stage inherits the invariant rather than one of them owning it. `### The review obligation` is a subsection of the same section.

**The four citations, as they resolved.** Each was checked by reading the file, not by grep.

1. *Neutral alternatives* — `crates/tiler-ir/src/schedule/`. Its module documentation opens "Target-neutral scheduled-region IR" and states at `mod.rs:17-18` that the module "owns no target profile, no feasibility decision, no cost model, and no semantic-graph correlation; those remain compiler-owned" — quoted rather than paraphrased. The compiler-side end is `ProposalBody::ScheduledKernel(Box<ScheduledRegion>)` at `crates/tiler-compiler/src/frontier.rs:256`. The vocabulary itself is cited to `fusion-and-scheduling.md#schedule-representation` and `#physical-implementation`, not restated.
2. *Typed permissions* — `OperationAlgebraicCapabilities` (`crates/tiler-ir/src/semantic/operation.rs:922`, `declares_ordered_associativity` at 944); `NumericalRealization`'s `contraction`, `reassociation`, `permutation` fields typed as `NumericalPermission` (`crates/tiler-ir/src/schedule/numerics.rs:217-222`, enum at 173); `StrictF32NumericalContract::governed_profile` (`crates/tiler-compiler/src/request.rs:276`). The physical-strategy half is `StrategyDeclineCause::NumericalPermissionRefused` (`crates/tiler-compiler/src/frontier.rs:1092`), whose own doc comment confirms the decline is decided from the request before any region exists.
3. *Feasibility queries* — `crates/tiler-compiler/src/target/feasibility.rs`, whose module documentation states "It deliberately has no notion of cost" (line 11). The atomic-realization precedent resolved to `TargetProfileBuilder::declare_synchronization_realization` (`crates/tiler-compiler/src/target.rs:1601`), which takes one whole `SynchronizationSubject` and documents the absence of any per-dimension spelling. The explainable starve is the pair `FrontierRejection::Unsynchronizable` (carries the refusing fact) and `FrontierRejection::SynchronizationUndeclared` (carries none, deliberately), `frontier.rs:1525-1546`.
4. *Typed costs* — `PhysicalCostEstimate` (`frontier.rs:345`) under `COST_MODEL_KEY = "tiler.cost.structural.v1"` (`frontier.rs:100`), the sole pruning input; `crates/tiler-compiler/src/component_cost.rs` with `ANALYTICAL_MODEL_KEY = "tiler.cost.analytical.v1"`, whose header states why an analytical cost never reaches dominance; and `crates/tiler-compiler/src/estimate.rs`, whose header states that the absence of a conversion into `ResourceRequirements` *is* the enforcement.

**The two no-touch verifications, checked by reading the diffs rather than the reports.** The cooperative-workgroup tier is two commits, not one: `be8f42e` (`represent-cooperative-workgroup-reduction-dataflow`, merged 07:57) touched **no** `crates/tiler-compiler` file at all — its diffstat is `tiler-ir/src/{schedule,kernel}`, `docs/`, and its ticket; `fece761` (`admit-the-first-typed-synchronization-point-and-atomic-target-authority`, merged 09:32) added the target fact (`target.rs` +166, `target/feasibility.rs` +904), a typed frontier rejection, and an explain row, and its **only** `selection.rs` line is `+ synchronization: None,` added to a struct literal inside `mod tests` at line 1872 — a mechanical field addition, no selection logic. The claim as the ticket stated it ("without touching selection machinery") therefore survives on substance, but a reviewer diffing file lists will hit that one line, which is why the recorded text names it explicitly instead of claiming the file was untouched. `63fde23` (`implement-the-single-workgroup-synchronized-reduction-strategy`, merged 10:29) does not touch `selection.rs` at all; its `frontier.rs` diff adds `propose_workgroup_tree`, the `SynchronizationUndeclared` rejection, and a fold over declines, and its `physical.rs` diff adds `single_workgroup_tree_region` and `SINGLE_WORKGROUP_TREE_STRATEGY`. The reproduction recorded in the contract, run on all three: `git show <commit> -- crates/tiler-compiler/src/ | grep '^[+-]' | grep -icE 'dominat|prune|PlanStructuralCost'` returns `0` for each.

**Architecture link.** One sentence in `docs/architecture.md`'s `## Dependency direction` section, immediately after the ADR 0090 **Fact** paragraph and the paragraph carrying "The compiler core must not know about Candle storage objects, einops syntax, or a particular artifact-delivery workflow" — the compiler-core-independence rule the invariant instantiates. It links to the new anchor and states nothing the optimizer contract owns.

**Sweep findings.**

- *No collision with the two named correction tickets.* `correct-the-optimizer-contract-registered-preset-count` and `correct-the-optimizer-one-variant-permission-claim` are both `done`, and both sentences live in `### Implemented first algebraic portfolio`, which the invariant text does not pass through. Their sentences were left byte-identical. The four-contract count was re-verified anyway rather than trusted: `StrictF32NumericalContract::governed_profile` returns `[Strict, FlushSubnormalsToZero, Relaxed, PermitReassociation]`, matching the corrected line.
- *The explain census was not moved by the strategy landing.* `crates/tiler-compiler/src/explain.rs:35-38` still reads `EXPLAIN_SCHEMA_VERSION = 9`, `EXPLAIN_RENDERER_VERSION = 7`, `COMPILATION_EXPLAIN_RENDERER_VERSION = 1`, and `explain_vocabulary_is_append_only_and_versioned` still pins 9 and 7. The contract's `tiler-explain-v7 / schema v9 / tiler-compilation-explain-v1` sentence is unchanged and correct.
- *The typed declines the strategy landing added are additive, and no enumeration in this file counted them.* `StrategyDeclineCause` has three variants and `FrontierRejection` gained `SynchronizationUndeclared`; the file's only rejection-related census is the **five compilation failure classes**, which are boundary outcomes and are unaffected. The `### Physical implementation` schedule list ("serial, subgroup, threadgroup, or multi-pass reduction") is a "such as" design enumeration containing several unimplemented entries, and the tree is the threadgroup form's *strategy* rather than a sixth topology, so it is not a falsified census.
- *One genuinely stale sentence corrected, and it is not silent.* The deterministic-budget list carried "8 nondominated implementations per region" as a ninth, forward-looking budget whose stated activation condition was the physical-implementation-frontier stage landing (recorded in `reconcile-optimizer-schematics-with-implemented-identity-and-budgets`'s own outcome). That condition has fired — `enumerate_frontier` is called from `pipeline/planning.rs:225` and `ImplementationFrontier::non_dominated` retains the frontier — and the budget never appeared: `DeterministicBudgets` (`request.rs:423-458`) has no such field, and retention is a pure Pareto filter with no count bound. The sentence now states the current state and points at the open decision instead of asserting either answer.
- *The substantive question behind it was filed, not absorbed:* [`decide-whether-the-implementation-frontier-owes-a-retention-budget`](decide-whether-the-implementation-frontier-owes-a-retention-budget.md), which frames the two defensible answers and notes the exposure is latent only while one governed provider is installed.

**Maturity boundary, stated in the contract itself.** Two landings are empirical evidence over a bounded population. They establish that the four surfaces sufficed for the two tiers that have arrived; they do not prove the invariant holds for a tier nobody has built, and the recorded text says so rather than generalizing.

**Verification.** `tkt lint`; `git diff --check`; `tkt guard --base a5e9886`; `make full`. All 66 local links in the two edited files were resolved against their files and heading anchors by a counted check (66 links, 0 broken) — the population is named and counted rather than reported as a uniform pass.
