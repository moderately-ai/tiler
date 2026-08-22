---
id: realize-the-tiled-contraction-schedule-and-its-metal-emission
title: Realize the tiled contraction schedule and its Metal emission
status: in-progress
priority: p1
dependencies: [admit-a-cooperative-tile-over-shared-operands, admit-guarded-output-tails-for-cooperative-contraction, admit-a-two-dimensional-cooperative-staging-relation, reclassify-language-model-work-as-a-conformance-track]
related: [realize-the-strict-contraction-on-metal, realize-the-contraction-through-the-appendable-direct-path, integrate-the-contraction-vertical-into-the-runtime]
scopes: [implementation/ir, implementation/compiler, implementation/metal, contracts/navigation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, language-model, deferred, class-generic-capability]
claimed_from: todo
assignee: worker-tiled
lease_expires_at: 1787425502
---
## User-visible outcome

The `tiled` realization the [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) selects compiles through the ordinary entry point as a retained alternative beside `direct`, refuses `K` not a multiple of its tile width with a typed reason that has been watched firing, and emits an MSL body carrying no fused multiply-add on its accumulation path.

## What remains once its cooperative dependencies land

This is the *fourth* wall [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) derived plus the work that was always this ticket's, and it is deliberately the cheapest of the four so it is not attempted first with a half-built vocabulary.

- **The topology dispatch.** `verify_intrinsic` (`crates/tiler-ir/src/schedule/builder/intrinsic.rs`, anchor `pub(super) fn verify_intrinsic`) dispatches on the scalar program: `StrictTensorContraction` reaches `verify_contraction` (`crates/tiler-ir/src/schedule/builder/contraction.rs`), which requires `ReductionTopology::Contraction` by `let … else`, and `verify_cooperative_semantics` (`crates/tiler-ir/src/schedule/builder/reduction.rs`) is reachable only from the four single-read reduction programs. A contraction therefore cannot carry a cooperative topology at all. A new `ReductionTopology` variant at appended tag `0x36` with its own semantic-verification arm is what admits it; the tag is an append that moves no earlier region's bytes.
- **The guarded output tail used by four retained rows.** The exact-divisible first pass deliberately refuses a partial output block. The retained kernel instead keeps the entire workgroup convergent, guards operand loads, and predicates the owning store at `M = 1` and `M = 10`; [`admit-guarded-output-tails-for-cooperative-contraction`](admit-guarded-output-tails-for-cooperative-contraction.md) must state and verify that relation before this ticket can claim all six correctness cells.
- **The schedule and the alternative.** `single_workgroup_tree_region` (`crates/tiler-compiler/src/physical.rs`) is the precedent — a constructor returning a typed `…Unavailable` decline the frontier records as a declined strategy rather than as an absence — and the tiled alternative follows it, offered beside `direct` rather than replacing it.
- **The `K ≡ 0 (mod 16)` precondition as a typed refusal, watched firing, never a pad.** `+0.0` is the strict sum's empty result and is not its bitwise-neutral padding; a K-padding schedule would owe the neutrality proof [Numerical semantics](../docs/numerical-semantics.md) requires. Refuse rather than acquire that obligation.
- **The two-allocation lowering.** `cooperative_plan`'s `let ([staging], [produce, consume]) = …` (`crates/tiler-ir/src/kernel/lower.rs`) admits one allocation and one visibility edge. Two allocations are already admissible at the schedule layer — `verify_cooperative_tile` loops over `tile.staging`, and `SynchronizationPoint::discharges`/`discharges_anti` do not read an edge's `staging` field, so one phase boundary discharges both edges and one round boundary both anti-dependencies — so this is an emission widening with no identity consequence.
- **The Metal emission of a multi-round two-allocation contraction body.** KIR already has multi-round *reduction* emission (`emit_loop_carried_cooperative` for a single staging allocation and contributor-split rounds). Landed Metal goldens for cooperative work are single-round (`cooperative_workgroup_reduction.metal`). The two-allocation contraction multi-round body — round loop, staged tiles, and barriers in MSL — is still this ticket's, with `spikes/scheduling/metal_contraction_vertical/kernels.metal` (`contract_tiled`) as the reference text and the existing golden idiom for the evidence.
- **No fused multiply-add on the accumulation path.** The flag is not sufficient — the L3 spike measured `simdgroup_multiply_accumulate` fusing under `-ffp-contract=off`, reproducing [finding 16](../docs/research/apple-targets/numerical-behaviour.md) — so the per-statement emission rule is what holds the line, and `the_contraction_kernel_emits_no_fused_multiply_add_on_its_accumulation_path` is the evidence idiom.
- **Bit-comparison at all six L3 profile cells** against the retained `result_sha256`, with the staged reference oracle (`StagedStrictTensorContractionF32`) as the drift check. State the measurement boundary: a host comparison is not a dispatched one, and [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) owns the device.

**Correction — 2026-08-19 (paths only; every cited symbol re-located and every substantive claim retained).** The topology-dispatch bullet cited `crates/tiler-ir/src/schedule/builder.rs`, which no longer exists — the schedule-builder split replaced it with the `builder/` directory. The three verifiers it names live in three different submodules now and are cited individually above: `verify_intrinsic` in `builder/intrinsic.rs`, `verify_contraction` in `builder/contraction.rs`, `verify_cooperative_semantics` in `builder/reduction.rs`. `verify_cooperative_tile`, named in the two-allocation-lowering bullet without a path, is in `builder/tile.rs`. The dispatch relation, the appended-tag argument, and the identity consequence are unchanged.

**Do not "repair" `StrictTensorContraction` here.** ADR 0112 retired the *semantic operation key* `tiler::strict-tensor-contraction-f32@1` in favour of `tiler::tensor-contraction-f32@1`, but the *schedule* variant `ScalarProgram::StrictTensorContraction` is a different vocabulary and still exists under that spelling at this base (`crates/tiler-ir/src/schedule/builder/contraction.rs`, anchor `let ScalarProgram::StrictTensorContraction`). Renaming it in this ticket to match the retired key would substitute a new false claim for a true one.

## Numerical legality, already settled

The L3 `tiled` schedule preserves each thread's ascending left fold over its output's contributors; the K-chunk loop changes only the **memory schedule** (threadgroup tiles and barriers), not the reduction tree. L3 therefore attributes it uniquely to `strict_fold+ftz` and records it as **consuming no numerical permission**, the same attribution and byte-identical results as `direct`. Both remain strict-admissible. Reproduce: L3 legality row "`tiled` | **Yes**, consuming no permission"; kernels.metal "changes the memory schedule and nothing about the reduction". Do not require reassociation, and do not gate this schedule on `NumericalContract::FLUSH_AND_REASSOCIATE_F32` — that would falsely refuse it under strict/FTZ contracts where L3 proved it legal. (`a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` is a parallel-sum portfolio fixture under a reassociating contract, not a warrant for this schedule.)

**Correction — 2026-08-10.** Prior wording claimed the tiled schedule consumes **reassociation** by regrouping contributors into per-round chunks and is therefore admissible under `NumericalContract::FLUSH_AND_REASSOCIATE_F32`, citing `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` as numerical precedent. That is false for L3 `contract_tiled`: products still enter a single left-fold accumulator in ascending contributor order; the existing `CooperativeWorkgroup` *reduction* topology is what consumes reassociation (shared-output parallel sum), the inverse relation. Withdrawn. Live guidance is the paragraph above.

## Non-goals

The split alternatives, the matrix-instruction route, any opaque call, and any cost model. [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) forbids substituting a differently-attributed realization to make a number better, and the L3 record states the measured price of not doing it.

## Released from deferred — 2026-08-22, its trigger fired on 2026-08-13

The trigger below reads: *"It becomes work only when both cooperative-contraction tickets are `done`, because the retained six-cell population contains exact and partial output blocks and this ticket may not silently substitute the direct realization for either."* Verified at `5bbb4b87`: `admit-a-cooperative-tile-over-shared-operands` and `admit-guarded-output-tails-for-cooperative-contraction` are both **`status: done`**, and all four declared dependencies are `done`. They landed on 2026-08-13 — this sat deferred for nine days after becoming work.

**This is the largest single unblock in the graph: 22 non-terminal dependents, 10 of them p1**, including the whole C1 chain (`realize-the-attention-contractions-on-metal` → `plan-the-materialized-attention-decomposition` → `integrate-the-attention-block-into-the-runtime` → prefill → decode → autoregressive loop → `prove-the-c1-complete-model-execution`). It is also the shared upstream gate behind roughly 32 deferred `scope-the-*-family` research nodes.

**TAG CLAIM IS STALE — do not take it from the body below.** This ticket says to append the new `ReductionTopology` variant at tag **`0x36`**. Read at source: `TAG_REDUCTION_COOPERATIVE_WORKGROUP = 0x35`, `TAG_REDUCTION_COOPERATIVE_CONTRACTION = 0x37`, `TAG_REDUCTION_LIVE_CONTRACTION = 0x38` — so `0x37` and `0x38` are taken, and a graph sweep reports `0x36` is **reserved for `CooperativeContractionSplit`** under `decide-the-fixed-strided-contributor-membership-vocabulary`. **Derive the next genuinely free tag yourself at your base**; `admit-subgroup-bindings-into-the-schedule-vocabulary` carries the identical stale `0x36` claim, so two tickets would collide if both were taken literally.

**Note on tag reasoning, learned the hard way 2026-08-22:** tag spaces in `crates/tiler-ir/src/schedule/model.rs` are **per-frame, not global** — `TAG_LINEAR_IDENTITY = 0x01` and `TAG_COVERAGE_PADDED = 0x01` coexist deliberately (the file documents it at anchor `overlap deliberately`, wrapped across two `///` lines). So "value X is used elsewhere" is not by itself a collision. Reason from the frame the tag is written into and from the family-run convention, not from a global scan.

**Facts below predate many landings** — `tiler.kernel-program` v12→v13, `tiler.artifact-program` v20→v21, manifest (20,0)→(21,0), the retired contraction key, four module splits, and the index-layer gather with its realization law. Re-audit every Fact at your own base per the stale-Facts rule.

## Activation triggers

Deferred behind the exact-divisible relation, its guarded-output-tail extension, and the already-landed two-dimensional staging relation. It becomes work only when both cooperative-contraction tickets are `done`, because the retained six-cell population contains exact and partial output blocks and this ticket may not silently substitute the direct realization for either.

## Closes when

A contraction of the profile's projection structure compiles through the ordinary entry point to a tiled Metal kernel, its results are bit-identical to the reference at every profile cell, the `K` precondition refuses with a typed reason that was watched firing, and the emitted module carries no fused multiply-add on the contraction's accumulation path.

## Trigger check log

- 2026-08-04 — **not fired.** The activation trigger is [`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md) reaching `done`; it is still `deferred`, and this sweep found its own trigger only half fired — its dependency landed but the two public boundaries it needs are Tom's and unaccepted. Recheck: that ticket's status and its trigger-check log.
- 2026-08-09 — **not fired.** The two-dimensional staging dependency is `done`, but `admit-a-cooperative-tile-over-shared-operands` is now correctly `awaiting-decision`: the second cooperative relation and its ownership-proof kind remain Tom's public-boundary decision. This implementation stays deferred behind that answer rather than appearing runnable merely because the lower dependency landed.

## Source-first Fact audit — 2026-08-22, exact base `489cc3553965ef87d053cc15a11279a9e00b4ab4`

Every Fact re-read in the file it names at this base. The body above was written before the two cooperative-contraction dependencies landed on 2026-08-13, and most of what it lists as remaining is now present — but the two things that *were* built were built against a numerical claim that the measurement refutes, which is the substance of this lane.

| # | Fact as stated | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | A contraction "cannot carry a cooperative topology at all"; a new `ReductionTopology` variant "is what admits it" | **False** | `ReductionTopology::CooperativeContraction` exists and is accepted (`crates/tiler-ir/src/schedule/model.rs "const TAG_REDUCTION_COOPERATIVE_CONTRACTION: u8 = 0x37;"`). `verify_intrinsic` admits it beside `ExecutionBinding::BlockedWorkgroup` and `TailPolicy::Predicated`, and `verify_contraction` dispatches to `verify_cooperative_contraction`. **No new variant and no new tag are required by this ticket.** |
| 2 | The new variant takes appended tag `0x36` | **False**, as the release note already warned | `0x36` is reserved for `CooperativeContractionSplit` (`crates/tiler-ir/src/schedule/model.rs "reserved for the accepted"`), owned by [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md). `0x37` and `0x38` are taken. See *The tag question, answered* below. |
| 3 | The guarded output tail "must state and verify that relation before this ticket can claim all six correctness cells" | **Verified, and satisfied** | `TailPolicy::Predicated`, `GuardedLoad`, `admit_predicated_cooperative_contraction`, and the role-checked guards all landed under [`admit-guarded-output-tails-for-cooperative-contraction`](admit-guarded-output-tails-for-cooperative-contraction.md). The `M = 1` and `M = 10` population is representable. |
| 4 | `single_workgroup_tree_region` is the precedent for a typed `…Unavailable` decline the frontier records as a declined strategy | **Verified** | `crates/tiler-compiler/src/physical.rs "pub(crate) const SINGLE_WORKGROUP_TREE_STRATEGY"`, its `WorkgroupTreeUnavailable`, and `DeclinedStrategy` / `StrategyDeclineCause` in `frontier.rs`. Nothing in `tiler-compiler` references `CooperativeContraction`, `BlockedWorkgroup`, `TailPolicy::Predicated`, or `admit_exact_cooperative_contraction` — the alternative genuinely does not exist. |
| 5 | The `K ≡ 0 (mod 16)` precondition must be a typed refusal, never a pad | **Verified, and satisfied at the schedule layer** | `CooperativeContractionAdmission::ContractedTileNotDivisible`, returned by `crates/tiler-ir/src/schedule/blocked.rs "pub fn admit_predicated_cooperative_contraction"` and its exact sibling. Neither ever returns a direct `Contraction` schedule. |
| 6 | `cooperative_plan`'s `let ([staging], [produce, consume]) = …` "admits one allocation and one visibility edge"; two allocations are "an emission widening with no identity consequence" | **Imprecise — the citation is right, the conclusion drawn from it was aimed at the wrong function** | That destructuring is still at `crates/tiler-ir/src/kernel/lower.rs "let ([staging], [produce, consume])"`, but it belongs to `cooperative_plan`, the *reduction* tile's planner, which this schedule never reaches. The contraction has its own `cooperative_contraction_plan`, which already destructures `let ([left, right], [produce, consume])`. Two allocations were not the missing piece. |
| 7 | "one round boundary [discharges] both anti-dependencies" | **Verified as a schedule fact, false as a lowering fact** | `discharges_anti` reads no `staging` field, so one point does discharge both. But `cooperative_contraction_plan` matched a *single* anti-dependency edge and returned `CooperativeLoweringShape` for any other count, so every multi-round two-allocation tile — i.e. every tiled contraction with `K > 16` — was refused. Fixed here. |
| 8 | The multi-round two-allocation Metal body "is still this ticket's" | **Verified** | `tiler-metal` referenced none of `GuardedLoad`, `BlockedWorkgroup`, `TailPolicy::Predicated`, or `CooperativeContraction`; `emit_operation`'s wildcard returned `MetalEmitError::UnrecognizedOperation` for `GuardedLoad`. |
| 9 | The tiled schedule "consumes no numerical permission" and is bit-identical to `direct` | **Verified against the retained measurement, and contradicted by the landed code** | `workload.tsv` in the retained correctness results carries `bit-identical-to-strict-fold` for `tiled` at all six cells with `result_sha256` equal to `direct`'s at every one. See *The numerical defect* below for what the code did instead. |
| 10 | "Landed Metal goldens for cooperative work are single-round (`cooperative_workgroup_reduction.metal`)" | **Verified** | And so were the *kernel-IR* fixtures: every `cooperative_contraction_region` call passed `contracted = 16` against a 16-wide tile, so `rounds == 1` at every one. The multi-round contraction path had no test at all. |

## The numerical defect this lane found, and fixed

**Fact — the landed cooperative contraction was a contiguous K-split, not the tiled realization.** `emit_cooperative_contraction` folded each round's sixteen products into a subtotal of their own and then added that subtotal to the accumulator: `acc + (p0 + … + p15)`. The reference kernel does the opposite — `spikes/scheduling/metal_contraction_vertical/kernels.metal` carries one `accumulator` straight through its `k0` loop and never restarts it, which is `((acc + p0) + …) + p15`. The two combine the same contributors in the same order and differ only in grouping, so they are different binary32 values, and only the second is the declared contributor sequence.

**Fact — the schedule verifier asserted the same false consumption.** `verify_cooperative_contraction` refused the topology outright unless `permits_reassociation`, which made the one realization L3 attributes uniquely to `strict_fold+ftz` inadmissible under every strict contract — precisely what the *Numerical legality, already settled* section above forbids. That gate postdates the belief this ticket's own 2026-08-10 correction withdrew, and two accepted ADRs had recorded it as implementation status.

**Inference — the regrouped form already has its own vocabulary, which is the independent derivation.** `CooperativeContractionSplit` holds reduction-topology tag `0x36` under [`decide-the-fixed-strided-contributor-membership-vocabulary`](decide-the-fixed-strided-contributor-membership-vocabulary.md) and is delivered by [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md). A `CooperativeContraction` that regroups is that topology under the wrong tag, giving one identity two numerical meanings. Nothing in the accepted surface — the variant's field list, its tag, `admit_exact_cooperative_contraction`, `prove_blocked_bijection` — changes here; what is withdrawn is a verifier rule and the prose asserting a consumption the strategy does not have.

## The tag question, answered

**No tag is consumed by this ticket.** The variant this ticket asked for already exists at `0x37`. Were one needed, the next genuinely free reduction-topology tag is **`0x39`**: `0x31`–`0x35` are `None`, `Serial`, `MultiPass`, `Contraction`, `CooperativeWorkgroup`; `0x36` is reserved-and-unwritten for `CooperativeContractionSplit`; `0x37` and `0x38` are the operand-sharing and live contractions. Derived by reading the reduction-topology frame's own run in `crates/tiler-ir/src/schedule/model.rs`, not by a global scan — tag spaces there are per-frame, which that file documents at anchor `overlap deliberately`.

## Landed — 2026-08-22

Commit `4f08e06b`, gated with `make check` green (`citations`, `fmt`, `build`, `lint`, `test`) on the merged tree.

- **`crates/tiler-ir/src/kernel/lower.rs`** — the round loop carries one accumulator into the tile fold instead of combining a subtotal after it (`"a subtotal of their own"`), reproducing the reference kernel statement for statement; and the anti-dependency resolution takes the same shape the visibility edges already used, so a repeating two-allocation tile lowers instead of being refused as `CooperativeLoweringShape`.
- **`crates/tiler-ir/src/schedule/builder/contraction.rs`** — both permissions stay recorded and cross-checked and neither is consulted to admit (`"recorded and cross-checked against the region"`), which is the relation `ReductionTopology::Contraction` already carried. A strict contract now admits the tiled schedule; a reassociating one still does.
- **`crates/tiler-ir/src/schedule/cooperative.rs`** — `blocked_operand_tile(block, rounds)`, the sibling of `workgroup_tree_tile` for this topology, **labelled draft public boundary**: the one tile shape three layers construct, written once. Its correctness is pinned by construction — the kernel fixtures' hand-built literal was replaced by a call to it and every schedule- and kernel-identity assertion still passes byte for byte.
- **`crates/tiler-metal/src/emit.rs`** — the `GuardedLoad` arm, spelled as a conditional operator so the subscript is unreachable on the false path. A select or mask-multiply spelling would read the element it exists to skip, which on this body's partial blocks is a read past the end of the operand.
- **`crates/tiler-metal/goldens/contraction_tiled_cooperative.metal`** — the first golden composing two threadgroup allocations, a round loop, a barrier inside a loop body, guarded operand loads, and a nested predicated store. Registered in the compile harness, and **compiled and linked by the qualified Apple toolchain**, not only byte-compared.

### Identity values recomputed on this tree

Every value below was read off this tree, never copied from a document.

| Subject | Before | After |
| --- | --- | --- |
| Schedule identity domain | `tiler.schedule.v7` | unchanged — no encoder byte moved |
| `STRICT_F32_REGION_IDENTITY_HEX` | pinned | unchanged, asserted |
| `ONE_COMMITTER_COOPERATIVE_IDENTITY_HEX` | pinned | unchanged, asserted |
| `ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX`, both live-row-major pins | pinned | unchanged, asserted |
| Every landed Metal golden | pinned | unchanged; one golden added |
| Cooperative-contraction kernel identity | never pinned | still unpinned; its bytes move with the fold change, and no retained program's bytes move with it because no plan, artifact, or pin reached this topology |

**No unchanged program's bytes moved.** The fold change alters only the *kernel* body of a topology whose lowering was refused before `admit-guarded-output-tails-for-cooperative-contraction` and which no assembled plan reaches, so it has no artifact or manifest consequence. `tiler.kernel-program`, `tiler.artifact-program`, and the manifest pair are untouched.

### Perturbations, with the failure text each produced

Each perturbs the subject and leaves the assertion alone.

| Guard | Perturbation | What it said |
| --- | --- | --- |
| `the_tiled_contraction_carries_one_accumulator_across_its_rounds` | round loop restored to `acc + fold_tile(None)` | `the round loop combines 1 subtotal(s) of its own; a carried accumulator performs every addition inside the tile fold` / `left: 1  right: 0` |
| the same test's second property | carried fold started at `1`, dropping each round's first contributor | `every contributor of the round enters the carried accumulator, including the first …` / `left: (1, 16)  right: (0, 16)` |
| `a_multi_round_operand_tile_discharges_both_anti_dependencies` | anti-dependency match restored to a single edge | `the multi-round operand tile lowers: Verification(CooperativeLoweringShape)` |
| `a_strict_contract_admits_the_tiled_contraction` | `\|\| !*permits_reassociation` restored | `ScheduledRegionBuildError { … diagnostics: [NumericalOrAccessRefinement] }` |
| `every_golden_compiles_and_links_when_a_toolchain_resolves` | `?` doubled in the new golden's guarded load | `golden contraction_tiled_cooperative.metal must compile: offline metal failed … error: expected expression` — which is what shows the Apple compiler actually reaches this fixture rather than the harness self-skipping |
| `make citations` | the ADR status notes left unrepaired | `check-citations: 2 citation(s) do not resolve against this tree.` naming ADR 0012 and ADR 0014 |

The last row is the one worth keeping: the citation gate, not a reviewer, is what caught that removing the verifier rule falsified a claim two accepted records carried.

### Scope added, and why

`contracts/decisions`, added with `tkt set --add-scope`. Removing the unconditional reassociation requirement falsified a sentence in the **implementation-status** sections of [ADR 0012](../docs/decisions/0012-physical-reduction-topology.md) and [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md), both of which pinned the retired anchor and both of which say of themselves that they are status records adding no decision. Neither record's *decision* moves: ADR 0014's rule is that a topology proves **its own** regrouping behaviour against the contract, and the correction restores exactly that rule rather than revising it. Both notes now carry a dated status correction naming this ticket. **The coordinator should re-read those two diffs against the `contracts/decisions` lane before merging** — that scope is also declared by `accept-the-live-extent-artifact-envelope-row`, whose diff was empty when checked here, and an empty diff proves nothing.

## Remainder — enumerated, not attempted

**The compiler alternative is not landed, and it is deliberately stopped rather than rushed.** Everything below the schedule and backend layers is in place: a strict tiled contraction verifies, lowers, and emits. What is missing is the offer — `tiler-compiler` still names none of this topology, so no `compile` call can select it, and the *User-visible outcome*'s "compiles through the ordinary entry point as a retained alternative beside `direct`" is unmet.

It stops here because its first prerequisite is a decision, not an implementation:

1. **The tile width has no authority.** `single_workgroup_tree_region` refuses to offer the tree unless the target profile declares a closed width policy, and says why: *"Silence is not a default"* (`crates/tiler-compiler/src/physical.rs "Silence is"`). A tiled contraction alternative that hard-coded the measured 16 would do exactly what that precedent forbids. Either the target profile grows a contraction-tile-width policy row — a target-profile public boundary, Tom's — or a named measured constant is accepted with the same standing as `MEASURED_TREE_PARTICIPANT_CAP`. This is one concrete question and it is the gate.
2. **The topology has no work/span derivation.** `work_span` covers `None`, `Serial`, `Contraction`, `MultiPass`, and `CooperativeWorkgroup`, and declines everything else through `crates/tiler-compiler/src/measured_cost.rs "_ => None,"`. A tiled plan offered without an arm there declines to state a total and never reaches measured comparison — it would be offered and never chosen, which looks like an absent alternative. The derivation itself is short (the tile changes the memory schedule, so work is `work_items × contracted` and depth is `contracted`, exactly `Contraction`'s), but it is cost surface and this ticket's Non-goals exclude a cost model, so it needs its own home.
3. **`verify_region_output_binding` pins the contraction to `RegionId::new(0)`.** A sibling alternative with its own region id is refused at binding unless that arm is widened, the way `verify_multi_pass_subject_binding` and `verify_workgroup_tree_subject_binding` already are.
4. **Which block shape.** The lowering admits square tiles only (`crates/tiler-ir/src/kernel/lower.rs "Square tiles only"`), so `B_m = B_n = T_k`; a request whose output is smaller than the block in both extents launches a 256-thread workgroup for one useful column.

**Also remaining, and separable:** the six-cell bit comparison. `crates/tiler-compiler/src/governed/contraction_conformance.rs` already pins `direct`'s host result against the retained `result_sha256` at the cells it reaches, and the retained record carries `tiled`'s digest as *equal to `direct`'s at all six*. What no host can supply is the third link — that the emitted body computes that fold — because there is no KIR interpreter in the workspace. **Measurement boundary: a host comparison is not a dispatched one.** What this lane supplies instead is structural and stated as such: the emitted fold is the ascending left fold, asserted at both the kernel-IR and the MSL layer and perturbation-proved at each. [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) owns the device.

### Unsupported cases, unchanged by this lane

Non-square tiles, non-`f32` elements, contracted rank above one, output rank other than two, a contracted extent not divisible by the tile width (typed refusal, never padded), and any tile whose participants do not equal the workgroup's threads. All were already refused by name and still are.
