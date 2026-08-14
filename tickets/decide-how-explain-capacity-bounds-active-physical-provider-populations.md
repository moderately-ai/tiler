---
id: decide-how-explain-capacity-bounds-active-physical-provider-populations
title: Decide how explain capacity bounds active physical-provider populations
status: in-progress
priority: p1
dependencies: []
related: [calibrate-the-physical-frontier-provider-and-outcome-budgets, measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [optimizer, budgets, explain]
claimed_from: todo
assignee: worker-explain-capacity
lease_expires_at: 1786716208
---
## Outcome

Decide whether the complete explain authority should retain its current one-MiB canonical detail-byte ceiling for physical-provider populations, widen that ceiling, or preserve completeness with a more compact record construction. State the supported active-provider population independently from installed-provider and raw-outcome cardinality. If a production change survives the decision gate, split it into an implementation ticket rather than implementing it here.

## Facts at discovery base `b2ab50f278616a1ad8f171184a16d60ae7e608ff`

- **Fact.** `ExplainWriter::push`, anchor `let exceeds = if terminal`, refuses a non-terminal detail record when either `retained_detail_records + 1 > MAX_RECORDS` or `retained_detail_bytes + bytes > MAX_CANONICAL_BYTES`. The constants are 4,096 records and 1 MiB respectively.
- **Fact.** `record_frontier`, anchor `for rejection in frontier.rejections()`, retains one detail record for every `StrategyDeclined` outcome plus frontier summary records. `record_plan_selection` and its component-cost helpers additionally retain the selected complete-plan population. The raw provider-outcome count therefore does not uniformly price explain work.
- **Fact.** `DeterministicBudgets` has no physical-provider raw-outcome field at the discovery base. `16,384` is a calibration candidate, not an installed authority, so no raw-outcome refusal can precede the existing explain ceiling in this reproduction. The preserved 256-outcome draft is read-only evidence and is not this compile path.
- **Measurement.** The exact public five-operation strict subject at executable evidence commit `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a` succeeds through six installed specialists. For one target that is 102 installed-provider outcomes, 56 retained alternatives, 2,291 rendered record lines, and 650,099 rendered bytes. Seven specialists produce 119 installed-provider outcomes and fail closed. The retained terminal line is `2257 target-feasibility compiler-failure rule=compile.failure@1 provider=compiler:tiler.compiler@1 subject=region:program-alternative:b489b9770d000255/region:0 event=compiler-failure:explain-detail-capacity causes=2256`.
- **Inference.** Explain record IDs are zero-based (`local` is minted from `self.records.len()`), so terminal ordinal 2,257 and 2,258 rendered record lines mean 2,257 detail records had been retained. The 4,096-record arm therefore could not have fired. The named `explain-detail-capacity` failure identifies the non-terminal disjunction; eliminating its record-count arm leaves the independent one-MiB canonical detail-byte ceiling as the first governing authority.
- **Measurement.** Successful retained alternatives for `n = 1..6` specialists are 6, 12, 20, 30, 42, and 56: `(n + 1)(n + 2)`. Rendered record lines are `39n² + 116n + 191`. The exact growth is subject-specific finite evidence, but it demonstrates quadratic downstream explain retention from linear `17n` installed-provider outcomes.
- **Fact.** The public `CompileFailureClass` reports only `InvalidCompilerOutput`; `CompileFailure::class` erases the inner `CompilerOutputError::Explain` cause. The complete failure trace retains `explain-detail-capacity`, so the raw measurement record's class alone is insufficient to identify this authority.

## Decision gate

Compare at least:

1. retain the one-MiB ceiling and explicitly support no more than the measured active-provider population for this subject;
2. widen the canonical detail-byte ceiling enough to carry the named full-provider population, with identity/schema and idle-M3 memory consequences stated;
3. reduce repeated frontier/plan explanation while preserving complete typed reasons and canonical identity; and
4. defer wider active-provider support and keep the calibration/value sink held.

Eliminate any option that drops records, silently truncates the trace, invents an active-provider policy, or treats raw outcomes as a uniform work unit. Compare correctness, fail-closed strictness, maintainability, host runtime/RSS, identity/schema consequences, and the interaction with `physical_plan_combinations` separately.

## Closing conditions

- Reproduce the six-success/seven-refusal boundary with the retained public spike command and quote the exact terminal failure record.
- Derive the first governing capacity from source, including why the 4,096-record ceiling did not fire and why the raw-outcome candidate is not an installed authority on this compile path.
- State whether full 32-provider activity is intentionally unsupported, requires an explain-capacity widening, or is served by a complete compact encoding.
- If widening or compaction survives, create the exact implementation and idle-M3 remeasurement dependencies before releasing the physical-frontier budget calibration.
