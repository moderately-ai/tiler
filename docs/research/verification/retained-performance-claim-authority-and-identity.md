---
schema: "tiler-doc/v1"
id: "tiler.research.verification.retained-performance-claim-authority-and-identity"
kind: "research"
title: "Retained performance-claim authority and identity"
topics: ["verification", "conformance", "performance", "measurement", "identity", "cost-model"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.correctness-and-testing", "tiler.contract.cost-model"]
ticket: "define-retained-performance-claim-authority-and-identity"
---

# Retained performance-claim authority and identity

**Status:** complete design research; the record census is exact, the claim population is explicitly unknown, and authoritative qualification remains blocked on owner manifests and migration

**Reviewed:** 2026-08-24 at `d1ec30a8ed5f610884a307b0c27d24300f1cc87c`

## Result

**Fact.** A measurement-bearing document is not a performance claim, and document metadata is not performance authority. At this base the governed research corpus contains exactly 114 research records, 38 of which declare `bounded-measurement`. Only three carry the free-form topic `performance`; that subset omits known performance-bearing records such as [Direct embedded-artifact costs across Rust crates](../embedding/embedded-artifact-costs.md), while [Model-level correctness and performance qualification](../program-planning/model-level-qualification.md) carries the topic and explicitly says it contains no Tiler execution measurement. Metadata therefore gives an exact record population and a demonstrably non-authoritative claim census.

**Fact.** The retained performance-claim population is unknown. Measurements, estimates, feasibility bounds, exact structural invariants, initial diagnostics, proposed thresholds, and normative guarantees coexist in prose, sometimes in one record. No owner-emitted manifest currently says which statements are accepted performance subjects, which revisions they have, or which evidence remains current. A number or a `Measurement` label is insufficient: dispatch count may be an exact correctness invariant, decoder allocation amplification may be a security-relevant resource bound, and a target cost row is a selection preference whose absence must not make a plan infeasible.

**Fact.** One internal measured-cost vocabulary is already owner-derived and fail-loud. `tiler-compiler`'s private `CostRow` has one variant, `SaturatedParallelFoldSteps`, with stable key `cost.saturated-parallel-fold-steps`; the target-profile builder, canonical descriptor, resolver, and measured-cost selector construct and consume it. Temporarily adding `AuditProbe` to that enum and changing nothing else makes `cargo check -p tiler-compiler` fail with `E0004`, `non-exhaustive patterns: CostRow::AuditProbe not covered`, at `CostRow::key`. This exact one-row vocabulary belongs in the optimizer/planner obligation manifest. Its retained measurement receipt still belongs in the cross-owner performance evidence join; neither owner should duplicate the other's claim.

**Proposal.** The nondominated destination is a distributed owner-emitted claim manifest joined to structured measurement receipts. The owner of the contract a claim asserts mints the claim identity. A research record or harness may produce evidence for that identity but cannot create, accept, revise, or retire it merely by editing prose or emitting a successful run. The global conformance universe is a canonical join of the owner manifests and remains incomplete while any performance-owning family is unknown.

## Exact-base Fact audit

| premise | verdict | evidence and consequence |
| --- | --- | --- |
| Every bounded-measurement record is a performance claim. | **False.** | The 38-record metadata population includes numerical behavior, shape feasibility, identity-encoder verification, and conformance-authority threat modeling. Evidence class describes how a record supports a statement, not the statement's subject. |
| The `performance` topic enumerates performance claims. | **False.** | The exact three-record topic census has both false negatives and a false positive as a measurement census. Topics are free-form navigation under [Documentation metadata and traceability](../../document-metadata.md), not an authority schema. |
| Every numeric performance-looking statement should enter conformance. | **False.** | [Cost model](../../compiler/cost-model.md) separates hard feasibility, exact structural features, estimates, calibrated preferences, and observations. [Correctness and testing](../../correctness-and-testing.md) also lists measurement dimensions without declaring a threshold or retained baseline for each. |
| The optimizer has no stable measured-cost subject. | **False.** | `CostRow::SaturatedParallelFoldSteps`, its governed key, canonical target-profile row, source provenance, lookup, and selection consumer form one exact typed subject. |
| A retained run can accept its own performance baseline. | **False.** | ADR 0114 separates evidence-baseline authority from execution. A run may propose evidence; acceptance, lineage, and qualification remain outside it. |
| Existing records determine one portable performance guarantee. | **False.** | The records repeatedly bound claims to exact hosts, toolchains, procedures, workloads, and statistics. The accepted cost contract explicitly forbids reading a selector as latency and forbids cost from admitting infeasible plans. |

## Separate identities

One conformance cell must not collapse the following identities:

| identity | owner | changes when | must not be used as |
| --- | --- | --- | --- |
| research-record identity | documentation metadata owner; stable document ID | the governed document is added or deliberately replaced | performance-claim identity or raw evidence identity |
| performance-claim identity | the component or contract owner whose performance subject is asserted | the claim's semantic descriptor or revision changes | evidence that the claim passed |
| measured-subject identity | the owner of the compiler plan, artifact, executable, cache route, or workload under test | output-affecting subject content changes | workload, environment, or baseline identity |
| workload/case identity | workload/profile owner | inputs, shapes, sequence, operation mix, or case population changes | implementation subject identity |
| environment/profile identity | target/toolchain/environment owners | any exact bound context field changes unless an accepted equivalence says otherwise | a portable target-family guarantee |
| procedure/harness identity | measurement owner | command, warm-up, repetitions, statistic, instrumentation, or correctness precondition changes | metric or acceptance-policy identity |
| evidence-snapshot identity | evidence-baseline authority | receipt set, raw record, observation, or acceptance lineage changes | claim definition or goal policy |
| normative guarantee identity | the real specification or accepted contract owner | its statement, scope, or version changes | an empirical measurement, however broad |

The claim identity is a canonical digest over an owner namespace, stable local key, nonzero revision, claim form, metric and unit, subject selector, workload/case selector, comparator and baseline policy, statistic, acceptance predicate when one exists, context-equivalence policy, required evidence expression, freshness policy, and predecessor/tombstone lineage. Observed values and raw-record digests belong to evidence snapshots, not the claim definition. A threshold or baseline is authority-bearing policy and therefore belongs to the claim descriptor or goal profile, never to the run that observes it.

## Claim forms and verdicts

Performance evidence does not imply that performance was good. The owner must declare which question the claim asks:

| claim form | example | evidence result | conformance meaning |
| --- | --- | --- | --- |
| observation-presence | retain decoder peak-live bytes for the governed envelope sweep | a valid exact-context measurement exists | the evidence-presence obligation may pass regardless of the value; it is not a speed or memory qualification |
| comparative relation | byte-string embedding uses less compilation work than per-byte literals in matrix `W` | comparable paired observations establish the declared relation | pass or fail against the predeclared relation; changing an arm or statistic is a new claim revision |
| threshold/SLO | release constant section does not exceed logical emitted bytes plus the accepted allowance | observation compared with an accepted threshold | pass or reached failure; an absent or incomparable measurement is yellow |
| calibrated parameter | saturated parallel fold steps for one exact target-profile population | a measurement supplies a selector parameter | evidence for optimizer costing only; it does not state latency, feasibility, or portable performance |
| model quality | regret or rank correlation of predicted versus measured legal plans | complete legal-set measurement and fit statistics | pass/fail only if the owner predeclares an acceptance predicate; otherwise retained observation only |
| regression relation | current exact subject is no worse than accepted baseline under policy `R` | comparable current and baseline receipts | pass/fail under `R`; a baseline change is an authority event, never progress |

For a required claim, valid evidence satisfying the exact predicate is green, a reached comparable observation violating it is red, and missing, stale, unavailable, or incomparable evidence is yellow. Gray requires an accepted goal-policy reason. A newly measured failure may be knowledge progress; avoiding a run must never preserve green. Observation-only claims should not be presented as a performance score: their green says that the required measurement exists, not that a value is desirable.

## Material claim families and owners

These axes are orthogonal views over claims, not a universal Cartesian product. One claim can be about artifact size, compare two compiler configurations, and be scoped to one host.

| claim family | real claim owner | evidence producer examples | required separation |
| --- | --- | --- | --- |
| host compilation and expansion work | frontend/build/toolchain contract owner | embedded-artifact and proc-macro harnesses | wall time, peak RSS, rebuild work, and freshness are different metrics; toolchain identity is exact |
| cache hit, publication, collection, and concurrency cost | cache contract owner | cache hot-path, collection, and build-tool harnesses | correctness/publication obligations stay outside performance; one compile-per-key is not latency |
| artifact encoding/decoding resource cost | artifact ABI/codec owner | allocation-amplification harness | peak live, retained bytes, total requested bytes, and allocation calls remain distinct; security bounds are not optimizer costs |
| artifact and binary footprint | artifact/frontend integration owner | embedding and manifest-growth records | logical bytes, unique bytes, section bytes, final binary, and target-tree size are not substitutes |
| compiler search and planning host cost | optimizer/planner owner | physical-frontier budget calibration | host time/RSS and deterministic search budgets remain distinct; a budget may be policy rather than measured optimality |
| device kernel and route performance | backend/schedule owner | contraction and reduction calibration harnesses | workload, selected plan, numerical contract, prepared kernel, completion, and device context all bind the receipt |
| calibrated cost and selection quality | cost-model/target-profile owner | target-profile cost-row and measured-feedback records | selector parameters, predicted values, rank quality, regret, and selected-plan performance are separate claims |
| end-to-end workload latency, throughput, and memory | workload/profile owner with runtime/backend subject identities | model-level qualification and quantized-profile harnesses | correctness must pass on the same build; prefill, decode, host round trip, persistent memory, and peak transient memory remain separately attributable |

Hard feasibility is excluded from this performance family whenever an authoritative resource limit determines whether a plan can run. Exact dispatch/materialization counts are excluded when their owner defines them as correctness or identity invariants. A normative performance guarantee enters only through its real normative owner and remains a distinct evidence atom; measurements do not promote themselves into one.

## Revision, freshness, and baseline rules

- A stable local claim key survives new observations. Its revision changes when the metric, unit, subject/workload selector, comparator, baseline policy, statistic, threshold, context-equivalence rule, evidence requirement, or freshness rule changes.
- Re-running the same claim changes the evidence-snapshot identity, not the claim revision. Changing compiler, target profile, plan, artifact, device, toolchain, workload, or harness makes the observation incomparable unless the claim owner has already accepted an exact equivalence relation covering that field.
- A baseline is immutable content with predecessor/successor lineage. “Previous main”, “latest green”, and a mutable path are selectors, not baseline identities. Advancing one is an authority event and must be reported separately from the resulting evidence delta.
- Freshness is owner-declared rather than one global duration. Source-bound claims expire on output-affecting subject changes; environment-bound claims expire on context mismatch; protocol-bound claims expire on procedure revision; optionally time-bounded measurements carry an explicit expiry. Historical evidence remains queryable after it stops satisfying a current claim.
- Removal or replacement preserves a tombstone and lineage under ADR 0114. Absence never means retirement. An owner manifest addition requires an explicit goal-profile disposition before audit succeeds.
- Unknown owner family, missing manifest, invalid identity, or absent authority blocks a complete universe. Missing measurement alone leaves the exact claim visible and yellow.

## Optimizer mapping

`cost.saturated-parallel-fold-steps` belongs exactly once in the optimizer/planner capability-obligation manifest because its owner is the private `CostRow` vocabulary and its consumer is measured-cost selection. Its performance evidence receipt must still bind the exact target profile, measured population, compilation selection, environment, procedure, and raw record. The optimizer manifest should classify at least declaration, production consumption, selection perturbation, explain attribution, and measured evidence; it must not copy the raw record into a second claim row.

Future optimizer cost rows follow the same split. Adding a `CostRow` variant creates a new owner claim and must fail an undisposed universe/profile join. Adding a benchmark sentence does not create an optimizer claim. Conversely, device, cache, artifact, compiler-host, or end-to-end measurements remain visible even when no optimizer ever consumes them.

## Enumeration options and decision packet

| candidate | verdict | reason |
| --- | --- | --- |
| infer claims from prose, evidence class, topic, or tests | **eliminated** | demonstrated false positives and false negatives; natural-language additions can silently change the apparent denominator |
| one manual global performance-claim list | **eliminated** | separates enumeration from the owner that mints the subject, so an owner addition can land without it |
| structured record-owned manifests as the sole authority | **eliminated** | improves receipt structure but lets an evidence producer define the claim and its threshold; record identity still does not identify the component contract asserted |
| owner-emitted claim manifests plus structured record receipts | **selected destination** | keeps claim authority, evidence, and baseline acceptance separate; owner additions can fail the canonical join; supports internal and user-visible claims uniformly |
| bounded source census | **selected migration bridge only** | provides exact-base candidate records and catches known spellings, but cannot prove that unstructured prose has no other claim |
| deferral | **retained only for unmigrated owners** | explicit unknown is honest; blanket deferral is dominated by the already fail-loud cost-row vocabulary and known records |

The strongest counterargument to distributed manifests is maintenance across many owners. The evidence that would reverse the selection is a singular existing performance authority whose enumeration demonstrably owns every component, workload, metric, and threshold without absorbing semantic or evidence authority; the audit found none. A central conformance-owned manifest is cheaper but structurally wrong: it would become the authority ADR 0106 forbids.

The strongest counterargument to keeping record receipts separate is that one manifest could carry both declarations and values. That is rejected because the same actor could move a threshold or baseline with the result, and because multiple observations under one stable claim would either duplicate the claim or mutate its identity. Evidence of an independently governed schema that prevents those edits while preserving one stable claim identity could change the physical storage layout, but not the authority separation.

The [executable fixture](../../../spikes/verification/retained-performance-claim-authority/README.md) demonstrates the owner-manifest/profile-join property on a deliberately tiny model. Its base manifest has two claims and two dispositions. A subject-perturbed manifest adds `perf.audit.undisposed@1`; the checker fails with `undisposed performance claim: perf.audit.undisposed@1`. This proves the join can fail loud. It does not prove that prose has been migrated, that the fixture schema should ship, or that a conformance crate may own the claims.

## Required follow-up

[`define-the-owner-emitted-performance-claim-manifest-contract`](../../../tickets/define-the-owner-emitted-performance-claim-manifest-contract.md) must choose the private/test-only manifest and canonical projection only after reading each owner boundary and the canonical receipt join. [`migrate-retained-performance-evidence-to-owner-claim-identities`](../../../tickets/migrate-retained-performance-evidence-to-owner-claim-identities.md) must then audit all 38 current bounded-measurement records plus performance-bearing records in other evidence classes, map every accepted claim or explicitly reject it as non-claim evidence, and retain a duplicate/unknown ledger. The first goal profile now depends on that migration so this ticket's explicit unknown cannot disappear merely because this research closes.

No product decision is requested from Tom here. The selected architecture follows existing owner and authority rules. Any consequential public boundary exposed by the manifest design, or any surviving conflict about who owns a concrete threshold, must return as a narrow decision rather than being settled during migration.

## Reproduction

Run from the repository root:

```sh
git grep -l '^kind: "research"' d1ec30a8ed5f610884a307b0c27d24300f1cc87c -- 'docs/research/*.md' 'docs/research/**/*.md' | wc -l
git grep -l '^evidence_classes: .*bounded-measurement' d1ec30a8ed5f610884a307b0c27d24300f1cc87c -- 'docs/research/*.md' 'docs/research/**/*.md' | wc -l
git grep -l '^topics: .*"performance"' d1ec30a8ed5f610884a307b0c27d24300f1cc87c -- 'docs/research/*.md' 'docs/research/**/*.md' | wc -l
spikes/verification/retained-performance-claim-authority/check_fixture.sh spikes/verification/retained-performance-claim-authority/owner-claims.tsv
spikes/verification/retained-performance-claim-authority/check_fixture.sh spikes/verification/retained-performance-claim-authority/owner-claims-perturbed.tsv
```

The first three commands report records, not claims: `114`, `38`, and `3` at the reviewed base. This report itself makes the tip counts `115`, `38`, and `4`, which is why the reproduction pins the base rather than handing a future reader a self-invalidating census. The first fixture check prints `2 claims; 2 dispositions; complete`. The perturbation exits nonzero and prints the undisposed claim above.

## Evidence boundary

This report defines an authority and identity contract, not the final wire schema. It ran no benchmark, accepted no threshold, created no performance guarantee, changed no cost model, selected no goal-profile population, and opened no public API. The 114/38/3 counts are exact-base source measurements and intentionally are not retained as the performance denominator. The performance-claim universe stays explicitly unknown until the follow-up migration can derive it from owner-emitted manifests.
