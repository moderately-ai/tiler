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
- **Fact.** This explain-capacity failure maps to the public `CompileFailureClass::InvalidCompilerOutput`; `CompileFailure::class` erases the inner `CompilerOutputError::Explain` cause. The enum has other classes for other failures. The complete failure trace retains `explain-detail-capacity`, so the raw measurement record's class alone is insufficient to identify this authority.

## Exact-base Fact audit — 2026-08-14, `e37c05b8ec28114736648edebbbdee745f4a051b`

The ticket's purpose survives the audit. Each source file named below was read in full or, for the 4,000-line implementation units, read through the complete construction, consumption, refusal, identity, renderer, and test spans the Fact depends on. Search commands locate the cited anchors; the reading, not the search, establishes the verdict.

1. **Verified — writer limits.** `crates/tiler-compiler/src/explain.rs`, anchors `const MAX_RECORDS`, `const MAX_CANONICAL_BYTES`, and `let exceeds = if terminal`: the nonterminal arm compares the next detail against 4,096 records and 1 MiB of canonical detail bytes and returns `ExplainError::DetailCapacity` without retaining the refused record. Reproduce location: `rg -n 'const MAX_RECORDS|const MAX_CANONICAL_BYTES|let exceeds = if terminal' crates/tiler-compiler/src/explain.rs`.
2. **Verified, with one noun repaired below — nonuniform explain work.** `crates/tiler-compiler/src/pipeline/trace.rs`, anchors `for rejection in frontier.rejections()`, `fn record_declined_strategy`, `fn record_plan_selection`, and `fn record_analytical_costs`: every `StrategyDeclined` has its own typed detail, and every **retained** complete plan has analytical-cost and later per-alternative records. “Selected complete-plan population” was imprecise because the portfolio retains selected and unselected plans; no conclusion depended on the adjective. Reproduce location: `rg -n 'for rejection in frontier\.rejections|fn record_declined_strategy|fn record_plan_selection|fn record_analytical_costs' crates/tiler-compiler/src/pipeline/trace.rs`.
3. **Verified — no raw-outcome authority.** `crates/tiler-compiler/src/request.rs`, anchors `pub struct DeterministicBudgets` and `physical_plan_combinations`: the complete field list has no physical-provider outcome or active-provider field. `docs/research/program-planning/physical-frontier-budget-calibration.md`, anchor `Raw-outcome decision frontier`, labels 16,384 a proposal and the 256 path a preserved draft at another commit. Reproduce location: `rg -n 'pub struct DeterministicBudgets|physical_plan_combinations|PhysicalFrontierOutcomes' crates/tiler-compiler/src/request.rs`.
4. **Verified — six succeeds, seven refuses.** The retained `request-boundary 7` output was reproduced from this exact checkout and byte-for-byte agrees in every printed value with `spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-13-request-wide-macos-27.0-m3-pro.boundary-first-failure.txt`. Six reports 102 installed outcomes, 56 alternatives, 2,291 ordinal record lines, and 650,099 rendered bytes. Seven reports 119 outcomes, zero successful alternatives, 2,258 ordinal lines, and the quoted terminal record at ordinal 2,257. Reproduce: `CARGO_TARGET_DIR=./spikes/program-planning/physical-frontier-budget-calibration/target cargo run --release --quiet --manifest-path spikes/program-planning/physical-frontier-budget-calibration/Cargo.toml -- request-boundary 7`.
5. **Verified — byte arm first at seven.** `crates/tiler-compiler/src/explain.rs`, anchors `local: u32::try_from(self.records.len())` and `fn finish_failure`: terminal ordinal 2,257 proves 2,257 earlier nonterminal records, and the terminal itself is the 2,258th ordinal record. The refused next detail would have been only 2,258, below 4,096, so the record-count arm cannot explain the `DetailCapacity` refusal; the one-MiB detail-byte arm does. The spike's `rendered_record_lines` counts only lines beginning with a parsed ordinal, not the renderer header.
6. **Verified — finite formulas.** Direct substitution reproduces alternatives `(n + 1)(n + 2)` and ordinal record lines `39n² + 116n + 191` at every retained `n = 1..6` row. `record_alternative_explain`, `record_analytical_costs`, and `record_cost_and_selection` show why retained-plan growth creates downstream detail growth. These are exact finite observations, not a proof beyond the six successful subjects.
7. **Imprecise wording repaired — public failure mapping.** `crates/tiler-compiler/src/session.rs`, anchors `pub enum CompileFailureClass`, `pub const fn class`, and `impl From<CompileError> for CompileFailure`, plus `crates/tiler-compiler/src/pipeline.rs`, anchors `CompilerOutputError::Explain` and `explain_error_reason`, show that this path maps to `InvalidCompilerOutput`; they do not say that `CompileFailureClass` has only that variant. The opaque class loses the inner cause while `CompileFailure::explain().render()` retains it. Reproduce location: `rg -n 'pub enum CompileFailureClass|impl From<CompileError> for CompileFailure|CompilerOutputError::Explain' crates/tiler-compiler/src/session.rs crates/tiler-compiler/src/pipeline.rs`.

The custody control is independently green at this base: `verify-evidence` reparsed all 2,250 ordered nanosecond values and all 45 RSS JSONL rows and returned `PASS custody evidence`. The retained SHA-256 values recompute to `ec3abc4ef90acb0d0e3e8a53f355f86a172ac2c2fce5a442310172b80b376c41`, `ebfb9015623fef7da7e9cfc7c6420cf3f5cd8faa245761e2e28d7f500d2b44ce`, and `8d8146bed7f0fa6e3d6a1feaed1cd2b4e5e9fea16721bd7ef50a44c26eb9cf78`. `git ls-tree` also confirms that the exact measured `program.rs`, `profile.rs`, and `providers.rs` blobs at `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6` equal `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a`, while the production explain, request, pipeline, trace, and session blobs equal behavior base `4fb0427319b1504e1549e03ba023ac486343a743`.

## Decision gate

Compare at least:

1. retain the one-MiB ceiling and explicitly support no more than the measured active-provider population for this subject;
2. widen both independent explain dimensions enough to carry the named full-provider population — a byte-only widening is not sufficient under the measured growth continuation — with identity/schema and idle-M3 memory consequences stated;
3. reduce repeated frontier/plan explanation while preserving complete typed reasons and canonical identity; and
4. defer wider active-provider support and keep the calibration/value sink held.

Eliminate any option that drops records, silently truncates the trace, invents an active-provider policy, or treats raw outcomes as a uniform work unit. Compare correctness, fail-closed strictness, maintainability, host runtime/RSS, identity/schema consequences, and the interaction with `physical_plan_combinations` separately.

## Closing conditions

- Reproduce the six-success/seven-refusal boundary with the retained public spike command and quote the exact terminal failure record.
- Derive the first governing capacity from source, including why the 4,096-record ceiling did not fire and why the raw-outcome candidate is not an installed authority on this compile path.
- State whether full 32-provider activity is intentionally unsupported, requires an explain-capacity widening, or is served by a complete compact encoding.
- If widening or compaction survives, create the exact implementation and idle-M3 remeasurement dependencies before releasing the physical-frontier budget calibration.

## Decision packet — 2026-08-14

### Capacity derivation and supported populations

**Fact.** Installed provider count, providers active on a subject, raw emitted outcomes, retained complete plans, explain detail records, and canonical explain bytes are six different populations. No current request field or accepted contract names an active-provider maximum. The public installation type accepts 129 identities in the retained finite check and has no count branch; this does not promise that all installed providers can be active on every subject.

**Measurement.** The exact five-operation one-target subject succeeds through six specialists and refuses at seven as audited above. The valid request-wide M3 rows measured governed-only at 165,855,232 peak RSS bytes, one specialist at 310,018,048, and two at 531,693,568. The 31-specialist row reached only the first target and refused at 788,283,392 bytes; it did not measure the 8,736-outcome request or a complete full-provider trace.

**Inference.** Six is an observed boundary for one subject, not a supported active-provider population. Neither six, two, nor 31 is an installed policy authority. Under the observed formula's continuation to 31 specialists, one target would retain 1,056 alternatives and 41,266 ordinal record lines. A successful trace has exactly one terminal selection record per retained alternative, so that projection separates into 1,056 terminal selections and **40,210 nonterminal details** (`41,266 - 1,056`), above 4,096. This does not turn the six-row fit into a universal law; it shows that the byte-only candidate's own continuation premise also requires the record ceiling to move. The source supplies a consistent lower-level mechanism: every retained alternative produces multiple nonterminal layer, analytical-cost, and structural-cost records before its terminal selection.

**Support statement.** At the current build there is **no numeric active-provider support guarantee**. Full activity across the proposed 32 total slots (governed plus 31 installed) is unsupported. The six-success row is finite evidence only. Retaining the present ceilings therefore means intentionally declining to promise full-provider activity, not silently redefining “supported” as six.

### Option audit

#### 1. Status quo recast honestly: retain both ceilings and defer wider activity — survivor

- **Correctness / strictness.** Complete-or-refused remains intact; no trace is truncated and no reason is dropped. Full-provider activity remains unsupported. The present public class is an acknowledged defect-shaped result (`InvalidCompilerOutput`), not a typed active-provider policy.
- **Maintainability / compatibility.** No new authority, field, schema, or public type. Existing behavior and all identities remain unchanged.
- **Host runtime / RSS.** Adds no cost. The strongest adverse evidence is already large: the first-target 31-specialist refusal peaks at 788,283,392 bytes.
- **Identity / schema / public surface.** None move. The ceiling is not in `DeterministicBudgets` or `canonical_explain_subject_bytes`; explain identity, request/evidence subject, plan, artifact, and cache identities stay as they are.
- **Unsupported population.** No numeric active-provider promise; 31 installed specialists active on this exact subject is unsupported.
- **Strongest counterargument.** A caller can legally install providers and reach a class documented as a Tiler defect despite satisfying every declared request budget. Deferral preserves that mismatch and keeps the raw-budget calibration held.
- **Reversal evidence / perturbation.** A named consumer requirement for more than the finite boundary, or a bounded full-demand probe showing complete traces affordable on the M3, reverses deferral. A regression control must keep the exact seven-specialist terminal cause and fail if the trace is silently shortened.

#### 2. Narrow fail-closed active-provider support — eliminated

No accepted consumer or request authority says two, six, or another specialist count is the supported maximum. Counting installed providers would reject idle capability; counting providers after their first emission is too late to pre-price native provider work and still does not bound plan or explain growth. A raw-outcome limit cannot stand in for this count because one outcome is not a uniform explain-work unit. Implementing this option would invent policy and would require a new governed/public refusal vocabulary and identity treatment. The evidence that could revive it is a named consumer population plus an independently derived preflight or accounting rule that guarantees the complete trace for every admitted request. Perturbations would need same-active-count/different-outcome and same-outcome/different-plan populations; the current evidence includes neither guarantee.

#### 3. Widen only the one-MiB byte ceiling — eliminated

It addresses the arm that fires first at seven but not the independent 4,096-detail ceiling projected to fail for the named full population. It therefore cannot deliver its claimed outcome. Raising an arbitrary byte value now would also choose a budget without measuring the complete 31-specialist demand.

#### 4. Widen both detail ceilings to the complete 31-specialist demand — not implementation-ready

- **Correctness / strictness.** Completeness can survive if both limits are sized from a completed trace rather than extrapolated and refusal remains atomic. No silent truncation is admissible.
- **Maintainability.** Mechanically simple after sizing, but a fixed constant can drift behind governed populations again, as the earlier `refuse-nothing-legal-on-the-explain-detail-ceiling` defect demonstrated.
- **Host runtime / RSS.** Unknown for the successful population. The only 31-specialist observation is a 788-MB early refusal. A full 16-target request could retain every target's completed trace, so it is unsafe to infer affordability from that refusal.
- **Identity / schema / public surface.** Changing only the constants does not move previously successful canonical encodings, request/evidence subjects, schema tags, renderer spelling, or public types. It changes the set of requests that can produce output. An implementation audit must decide whether that output-envelope change raises the compiler authority revision; the current packet does not invent that answer. Plan/artifact/cache identity changes only if newly admitted compilation changes selected packaged content.
- **Interaction with `physical_plan_combinations`.** The observed continuation's 1,056 alternatives is below the current per-target 4,096 plan-combination budget, but that is subject-specific and does not make either resource redundant.
- **Strongest counterargument.** It pays memory for repeated attribution without asking whether the same complete information has a smaller representation, and it may create an unusable host envelope.
- **Reversal evidence / perturbation.** A diagnostic-only run must first lift both bounds, retain exact required detail count and bytes through all 31 specialists, and then measure 1/2/8/16 target runtime and RSS on the unchanged idle M3. Independently lowering each lifted limit by one must reproduce record-capacity and byte-capacity refusals separately; no implementation ticket is justified before those results.

#### 5. Complete compaction or deduplication — not implementation-ready

Two materially different changes hide under this label and must not be conflated.

- **Source aggregation** is admissible only when several records repeat one identical decision ground and the replacement retains every affected subject, exact multiplicity, evidence, disposition, provider/rule, and causal meaning. The accepted `blocked-covers` repair is precedent. It changes affected trace identities by changing their record population but need not step schema or renderer versions if it uses already-encodable record shapes and changes no existing shape's encoding or spelling.
- **Canonical encoding compaction** can dictionary-code repeated keys while retaining every in-memory typed record. That moves previously encodable bytes and therefore requires an explain-schema step. It can leave the renderer and public render-only surface unchanged, and it does not enter request, plan, artifact, or cache identity; however, it may save too little RSS because all record objects and rendered output remain.

No retained artifact classifies the 31-specialist trace's exact repeated tuples or cause graph because construction stops at seven. Aggregating merely similar records would drop reasons or associations; dictionary encoding without an in-memory result may not solve the host problem. The strongest counterargument is implementation complexity across causal identity for an unrequested population. Reversal evidence is an exact structural census showing a large, losslessly aggregable or dictionary-compressible share, with expansion equality and subject perturbations that distinguish provider, region, strategy, plan, evidence, and cause. An unchanged renderer compared byte-for-byte after lossless source aggregation is a useful negative control but cannot alone prove canonical or causal equivalence.

#### 6. Further bounded research — survivor only if full activity becomes a named requirement

This is the only path that can make options 4 or 5 decision-ready without choosing a budget. It must use a diagnostic-only capacity override, never production constants, and must record exact detail/terminal counts, canonical bytes, rendered bytes, per-rule/event/subject populations, causal edges, lossless grouping candidates, and the first independent bound at each contour. The idle-M3 phase follows only after a one-target stop condition shows the full subject fits safely; host/toolchain configuration remains unchanged. The exact conditional follow-up is `measure-complete-explain-demand-and-lossless-compaction-for-full-physical-provider-activity`; it depends on this decision ticket, so it cannot dispatch before Tom answers the support-requirement question.

### Pareto frontier and recommendation

Only two choices survive today:

1. **Retain/defer:** lowest runtime, RSS, identity, schema, and maintenance cost; full 32-slot activity is intentionally unsupported and no numeric active-provider count is invented. Its cost is preserving an `InvalidCompilerOutput` wall and leaving the raw calibration held.
2. **Authorize bounded research:** preserves correctness and makes widening versus lossless compaction measurable, but spends engineering and M3 time for a population no consumer has named and creates no immediate production support.

Widening and compaction are research hypotheses, not surviving production changes, so no implementation or post-implementation M3 ticket is created yet. If research makes one production candidate survive and Tom accepts it, the graph must be `implement-<accepted-explain-capacity-change>` → `remeasure-accepted-explain-capacity-change-on-the-idle-m3-pro`, and the calibration must depend on the remeasurement before choosing or landing a raw-outcome value.

**Recommendation.** Retain the current ceilings and explicitly leave full 32-slot activity unsupported until a consumer names that requirement. Tom's consequential choice is one question: **should full activity across governed plus all 31 installed slots become a named support requirement now, authorizing the bounded research ticket, or remain intentionally unsupported?** No 1,024/16,384 raw budget follows from this packet alone.
