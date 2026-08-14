---
id: calibrate-the-physical-frontier-provider-and-outcome-budgets
title: Calibrate the physical-frontier provider and raw-outcome budgets
status: in-progress
priority: p1
dependencies: [decide-whether-the-implementation-frontier-owes-a-retention-budget]
related: [replace-provider-offer-with-a-host-bounded-frontier-sink]
scopes: [research/program-planning, implementation/compiler]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [optimizer, budgets, measurement, host-performance]
claimed_from: todo
assignee: worker-frontier-recalibration
lease_expires_at: 1786679676
---
## User-visible outcome

The first physical-provider-count and raw proposal/decline limits are measured calibration inputs rather than inherited folklore. The record names which ordinary and adversarial provider populations fit, what Tiler host work and memory each population causes, and where atomic exhaustion should fire.

## Required experiment

- Census distinct region subjects, total provider invocations, proposals, declines, admitted implementations, proposal rejections, retained bytes, verification work, sorting work, and complete-plan combinations separately.
- Include the governed provider, the retained external-provider vertical, at least two additive providers answering one region, equal-cost incomparable proposals, many declines, infeasible proposals, and empty providers. Count the full compiler-owned population; do not time kernel execution.
- Measure Tiler host runtime on the idle M3 Pro under the repository performance protocol. Record workload, target profile, warm-up, repetitions, variance, exact toolchain, source commit, and process/load controls.
- Recommend a provider-count limit and a raw-output limit with explicit headroom. Do not reuse the historical value eight without evidence.
- State what the proposed limits do not bound: arbitrary native provider computation and allocation before an emission.

## Stop conditions

Stop and split the work if proposal construction itself needs a bounded public builder, if provider count belongs to a different preflight authority than the complete budget policy, or if one raw-outcome count cannot represent proposal and decline cost without misleading calibration.

## Fact audit at `e27a38c9befc24d2d2ee0f9ad21d52246962d537`

- **Verified.** The historical value eight exists only as retired folklore. `docs/compiler/optimizer.md` records that "8 nondominated implementations per region" never became a `DeterministicBudgets` field. `ImplementationFrontier::non_dominated` is a Pareto view with no count bound.
- **Verified.** Production `PhysicalImplementationProvider` impls in `crates/tiler-compiler/src`, after cutting `#[cfg(test)]` modules and `tests.rs`, are exactly one: `GovernedPhysicalProvider`.
- **Verified.** `one_compile_enumerates_each_distinct_region_subject_once` pins 17 distinct subjects for `hot_path.rs`'s `program(4, 3)`. An installed observer on the public compile path is invoked 17 times for that program.
- **Verified.** `DeterministicBudgets::governed` has `physical_plan_combinations: 4_096` and no provider-count or raw-outcome field.
- **Imprecise in the parent decision, re-measured here.** The 2026-08-04 note that fourteen newly-answered subjects are declines is true of an extra specialist on this program (3 proposals + 14 declines). Governed outcomes themselves are not visible as a public offer; the observer census is the extra-provider half.

## Outcome

**Measurement, 2026-08-13, Apple M3 Pro, macOS 27.0 `26A5388g`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, warm-up 8, repeats 50, min estimator, load `{ 6.82 3.66 2.70 }` → `{ 6.32 3.66 2.71 }`.** Record: [the spike](../spikes/program-planning/physical-frontier-budget-calibration/README.md) and [the research note](../docs/research/program-planning/physical-frontier-budget-calibration.md).

| Limit | First value | Headroom |
| --- | --- | --- |
| Provider count, governed included | 32 | 32 |
| Raw proposal+decline outcomes, request-scoped | 256 | 256 |

Eight is not a calibration of either axis. A request-scoped eight would refuse the governed five-op program (17 subjects). Empty extras through 64 stay inside 1.11× the 3205 µs governed floor. 128 all-decline extras refuse as untyped `InvalidCompilerOutput`. Admitted proposals cost about 34× named declines on incremental host time; the raw-outcome count is a cardinality bound sized from the expensive side.

Proposal construction did not need a new public builder (`ImplementationProposal::scheduled_kernel` plus `baseline()` is enough). Provider count and raw outcomes stay in the same complete budget policy. One count remains valid as a cardinality ceiling; it is not a uniform work unit.

The limits do not bound arbitrary native provider computation or allocation before an emission.

Each load-bearing population check was perturbed with the assertion unchanged. Quoted failures are in the spike README. `census` is green after restore.

## Reopened — request-population omission found 2026-08-13

The `256` raw-outcome value is not yet a valid request-scoped calibration input. The experiment measured one target profile per compile, while `MAX_TARGET_PROFILES_PER_REQUEST` admits 16 and the proposed implementation shares one raw-outcome counter across the whole request.

- **Verified at `f3e1efd3b3b4f896976b326e6a3d993147206cd3`.** `MAX_TARGET_PROFILES_PER_REQUEST` is 16. The retained experiment's `Measurement boundary` names one program and separate governed/declared-capacity profiles; it does not census a multi-target request.
- **Verified on draft `54e272baa525027a6f6f9d982bd3bd7c387597fb`.** `PhysicalAuthorities::outcomes` is one request-scoped counter reused while the pipeline compiles target profiles and semantic candidates.
- **Inference requiring the rerun.** The governed five-operation census has 17 nonempty region subjects per target and `GovernedPhysicalProvider` answers every placed region with at least one proposal or decline. Sixteen equivalent-capability target slots would therefore reach at least 272 governed outcomes before any external provider. The harness must construct the admitted population and measure it; this arithmetic is a stop signal, not a substitute for the run.

**Measurement on preserved draft `54e272ba`, 2026-08-13.** A temporary out-of-tree public harness used the ordinary five-operation program, governed providers only, and 16 distinct valid target profiles. It returned a partial `Compilation`: targets 0 through 12 were `compiled`; targets 13 through 15 reported `BudgetExhausted { PhysicalFrontierOutcomes, 256, 257 }`. This confirms both the omitted calibration population and the draft's non-atomic propagation. The temporary `/tmp` harness is not durable evidence; reproduce the subject in the retained spike and record its input/output before replacing this paragraph with a rerunnable result.

Before this ticket can return to `done`:

- census raw outcomes across 1, 2, 8, and 16 admitted target profiles, across alternative numerical-contract candidates that the request may evaluate, and across the widest ordinary request population;
- separate a deliberate request-wide refusal from a limit that accidentally makes an admitted ordinary request impossible;
- compare request-wide, per-target, and any nested per-candidate authority that survives the decision gate. Eliminate any scope that permits retry or a partial product after exhaustion;
- rerun identical host-time/RSS sweeps for the nondominated candidate limit(s), update the spike and research record, and perturb the multi-target subject with the assertion unchanged; and
- state the full unsupported population and identity consequence before an exact value is encoded in `DeterministicBudgets`.

Until then, `256` remains a single-target measurement and [`replace-provider-offer-with-a-host-bounded-frontier-sink`](replace-provider-offer-with-a-host-bounded-frontier-sink.md) is held. The release trigger is a reviewed, reproducible full-request calibration with one surviving authority/value pair.
