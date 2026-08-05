---
id: derive-physical-proposals-from-the-cover-region-subject
title: Derive physical proposals from the cover region subject
status: in-progress
priority: p1
dependencies: []
related: [define-the-minimum-correct-physical-realization-profile, implement-general-dag-partitioning, assemble-a-kernel-program-from-an-arbitrary-cover, activate-shared-work-duplication-on-the-compile-path, drive-an-external-physical-implementation-provider-through-compilation, admit-elementwise-epilogues-over-a-materialized-intermediate, admit-a-reduction-over-a-declared-input-tensor, admit-the-registered-unary-families-at-the-compiler-request-boundary, decide-whether-the-implementation-frontier-owes-a-retention-budget]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, physical-planning, explainability, baseline]
claimed_from: todo
assignee: agent-provider-general
lease_expires_at: 1785895769
---
## User-visible outcome

Every region a legal cover places gets an answer from the physical layer: a verified serial or direct implementation where the schedule vocabulary can spell its occurrences, and a typed decline naming the missing vocabulary where it cannot. No region a cover placed is answered with silence, so a cover the general DAG search enumerated is never dropped without a recorded reason.

## Why this exists

This is obligation 4 and half of obligation 5 of the [minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md), and that record's stage-by-stage argument identifies it as the load-bearing one: stages 1 through 7, 9's join, and 10 are already general, so until stage 8 answers for an arbitrary region, every other stage's generality admits programs nothing can implement.

**Fact — the provider matches pre-computed member sets, verified by reading `crates/tiler-compiler/src/frontier.rs` at `57474a09`.** `GovernedPhysicalProvider::propose` reads `context.subject().semantic_members()` and compares it by exact equality against `request.pointwise().members`, `request.contraction().members`, and the serial-sum strategy's `members.pointwise()`, `members.reduction()`, and `members.all()`. Every other member set reaches a final `else` returning `ProviderOffer::default()` — no proposal, and no decline.

**Fact — the region builders cannot be asked about an arbitrary region, which is why the provider matches sets.** `crate::physical::pointwise_region(request)` takes only the `VerifiedTargetRequest` and reads `request.pointwise()` or `request.serial_sum()` for its iteration shape, element count, input arity, scalar expression, members, and the role of the tensor it writes — `TensorRole::Output` for a whole-program pointwise region, `TensorRole::Intermediate` for a reduction prologue. `reduction_region`, `contraction_region`, and `fused_region` have the same signature and the same source of truth. Generalizing the provider without generalizing the builder would only move the match.

**Fact — the trace does not currently say what was dropped, and it is three separate defects.** (a) `record_frontier` (`crates/tiler-compiler/src/pipeline/trace.rs`) keys its subject as `region:{role}`, and `region_role` (`crates/tiler-compiler/src/pipeline.rs`) returns `unrecognized` for every member set outside the three; `pipeline/planning.rs`'s own comment records that "Fourteen of those seventeen subjects share the role `unrecognized` while covering different occurrences". (b) `record_frontier` is called only on the first sighting of a role, so thirteen of those fourteen emit no record at all. (c) `PlanRejection::RegionUnimplemented` *is* constructed per cover by `select_physical_plans` — its own comment records "the governed program records 38 of these per compile" — and `SelectedPortfolio::rejections()` has no production reader: `grep -rn '\.rejections()' crates/ --include='*.rs'` returns seven sites, and the three on `SelectedPortfolio` are all inside `crates/tiler-compiler/src/selection.rs`'s `#[cfg(test)]` module beginning at line 1705.

**Inference — this is what makes `CoverPolicy::governed` the compile path's policy.** The `governed` constructor's own doc comment names this provider and program assembly as the reason shared-work duplication stays off the compile path, and [`activate-shared-work-duplication-on-the-compile-path`](activate-shared-work-duplication-on-the-compile-path.md) reached the same two walls independently. Two tickets arriving at the same place from different directions is corroboration, not duplication.

## Correctness argument

The generalization is safe because it changes *which subject a derivation reads*, not what any authority proves about the result.

1. **The host still resubmits every body.** `crate::physical::verify_schedule_with_feasibility` performs whole-region intrinsic verification, the request-subject binding, and the hard-feasibility decision for the exact region and target. A generalized builder emitting a region that does not verify is invalid compiler output exactly as a specialized one would be, and it is caught by the same gate. The provider gains no trust it did not have.
2. **A decline is strictly more information than an empty offer.** The trait's own contract already distinguishes them: an empty offer "means the provider recognizes nothing about this region and target, which is neither an error nor a global-coverage claim", while a decline says "the strategy applied and this request did not admit it". Converting the `else` branch into a decline therefore adds a true statement and removes no guarantee. Because this build installs exactly one provider, its silence is currently indistinguishable from a coverage gap it should have named.
3. **The serial offer stays unconditional.** The split and the workgroup tree remain additive beside it. A generalization that made a parallel alternative a precondition of any offer would delete the baseline it exists to guarantee.
4. **Existing identities must not move**, which is the cheapest regression check available: the three recognized member sets must yield byte-identical proposals, proposal identities, and structural costs after generalization, because those bytes are folded into plan and artifact identity.
5. **Nothing in enumeration, dominance, retention, or plan selection is touched.** [The optimizer contract](../docs/compiler/optimizer.md#the-review-obligation) treats a diff touching those as a violation of the four-surfaces invariant until its author justifies it. This work is a proposal generator, a typed decline, and an explain record.

## What must be true when this lands

1. **The proposal is a function of the subject.** Inputs: the cover region's exact semantic members, its presentation role, the intermediate element counts the cover stated, and the verified target request. A derivation reading a whole-program recognition instead cannot be correct for a region that recognition did not name.
2. **The written tensor role comes from the cover.** A region writes a program output when the cover assigns it one and an intermediate when a materialization edge names it as producer — not from whether the *request* was recognized as a whole-program pointwise or as a reduction prologue.
3. **Every region a legal cover placed has either a proposal or a decline.** The decline's cause names which occurrence could not be spelled and which region-vocabulary wall it hit, so a reader can tell an unimplemented family from an unrepresentable shape.
4. **One explain subject per region subject.** Derived from the region's canonical occurrence identity rather than from the presentation role, so the fourteen subjects sharing `unrecognized` become fourteen records. The role stays beside it as a presentation label. With a per-subject key, the `first_sighting` deduplication becomes correct rather than lossy.
5. **`PlanRejection::RegionUnimplemented` reaches the trace**, emitted at the selection stage and caused by the frontier record for that subject. It is the only authority that states the coverage gap *per cover*, and it is currently read by tests alone.
6. **The three recognized member sets are byte-identical**, asserted against the identities recorded before the change.

## Identity-domain obligation

The new `StrategyDeclineCause` variant and any new explain reason key are additions to a versioned canonical encoding. **The appends-only claim is carried by per-tag injectivity reasoning at each encoding site, not by the gate staying green** — `StrategyDeclineCause::encode` writes a leading discriminant byte (`0x01`, `0x02`, `0x03`) and a new variant takes the next unused tag. If any tag must move, or if the trace schema or renderer version must step, that step is executed completely at its owning layer: the version moves, the ledger documents move in the same commit, and every pinned identity is recomputed on the tree the step lands into with each moved pin enumerated in the report. Half a step is worse than none. `explain_vocabulary_is_append_only_and_versioned` in `crates/tiler-compiler/src/explain.rs` is the pin to check against.

## Required failure-path evidence

Each run against a case that must fail, observed failing before it is trusted, against an accepted neighbour:

- A cover region whose occurrences the schedule vocabulary cannot spell yields a decline with the naming cause — and the same region yields *no* decline at the unchanged base, which is the perturbation that proves the new record can appear.
- A region with no admitted implementation always carries a decline record: assert the implication, then delete the decline at one site and watch the assertion fail.
- The `RegionUnimplemented` emission fails when suppressed, so a green run is not explained by the rejection set being empty.
- Two distinct region subjects that share the role `unrecognized` produce two distinct explain subjects; asserted against a fixture where the old key would have merged them.
- The three recognized member sets' proposal identities are unchanged; perturb one cost input and watch the assertion fail.

## Boundaries

- **Do not widen the region vocabulary here.** The registered unary families, an elementwise region over a materialized intermediate, and a reduction directly over a declared input each have their own owning ticket. This ticket's obligation for each is a *typed decline*, not a widening; the widenings then convert declines into offers with no further change here.
- **Do not relax any request-boundary guard.** `output-arity` and the three vocabulary refusals stay exactly as they are; relaxing one ahead of the physical layer converts a typed refusal into a mid-pipeline compiler fault.
- **Do not turn on `CoverPolicy::permitting_shared_work_duplication`.** That is [`activate-shared-work-duplication-on-the-compile-path`](activate-shared-work-duplication-on-the-compile-path.md)'s one-line change, and it is deferred until both walls are down.
- **Do not add a frontier retention budget.** Whether one is owed is [`decide-whether-the-implementation-frontier-owes-a-retention-budget`](decide-whether-the-implementation-frontier-owes-a-retention-budget.md)'s open decision. Report the measured frontier population this change produces as an input to it.
- `contracts/optimizer` is declared because [the optimizer contract](../docs/compiler/optimizer.md#what-each-stage-is-general-over-today)'s stage-8 paragraph states the current limit as a fact and becomes false in the same change. A catalog is edited in the change that moves the metadata behind it.

## Stop conditions

- The generalization requires moving an existing encoding tag or stepping the trace schema or renderer version. Draft the step, file the carrier ticket, and stop — an identity-domain step is executed completely or not at all.
- The generalization requires a public boundary or a `tiler-ir` widening. The profile record's argument is that it needs neither; if that proves wrong, the discovery ends the dispatch and becomes structure.

## Graph maintenance

- [`assemble-a-kernel-program-from-an-arbitrary-cover`](assemble-a-kernel-program-from-an-arbitrary-cover.md) depends on this ticket and becomes reachable when it lands.
- [`activate-shared-work-duplication-on-the-compile-path`](activate-shared-work-duplication-on-the-compile-path.md)'s activation trigger 1 — "a physical provider proposes implementations for member sets beyond the three recognized ones" — fires here; its trigger 2 fires on the assembly ticket. Both must fire before that ticket leaves `deferred`. That ticket's triggers do not yet *name* these two tickets: the naming edit was inadmissible when these were filed because `tkt/sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired` held a committed edit to the same file. Apply it when this lands, or earlier once that branch merges.
- Correct the optimizer contract's stage-8 paragraph and the [general compilation boundary](../docs/research/program-planning/general-compilation-boundary.md#the-critical-path-to-a-naive-but-general-compiled-mimo-program)'s item 1 in the same change; both state the current limit as a fact.

## Closes when

1. Every region a legal cover places receives either a verified proposal or a typed decline, and the proposal is derived from the region subject rather than from a pre-computed member set.
2. The explain trace carries one subject per region subject, a record for every enumerated subject, and the per-cover coverage gap — each proved emittable by a check observed failing.
3. The three recognized member sets' proposals, identities, and costs are byte-identical to the pre-change values.
4. The contract sentences that stated the old limit are corrected in the same change.
