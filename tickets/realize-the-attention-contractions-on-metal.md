---
id: realize-the-attention-contractions-on-metal
title: Realize the attention score and value contractions on Metal
status: in-progress
priority: p1
dependencies: [admit-the-attention-contraction-structures, realize-the-tiled-contraction-schedule-and-its-metal-emission, reclassify-language-model-work-as-a-conformance-track]
related: [design-attention-program-vertical, plan-the-materialized-attention-decomposition, admit-reassociated-contraction-schedule-alternatives, scope-causal-structure-aware-attention-schedules, implement-parallel-reduction-strategies]
scopes: [implementation/compiler, implementation/ir, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, attention, language-model, class-generic-capability]
claimed_from: todo
assignee: worker-attention
lease_expires_at: 1787435006
---
## User-visible outcome

The two attention contractions become scheduled Metal kernels whose results are bit-identical to the reference evaluator — and the conformance row's value contraction is realized by the schedule that is *correct there*, not by the one that is fastest elsewhere.

## Why the conformance row selects a different realization

**Measurement — from [the L3 elimination](../docs/research/scheduling/first-metal-contraction-realizations.md).** The surviving strict realization, `tiled`, refuses a contracted extent that is not a multiple of 16 rather than padding it. Structure 2's contracted extent is the static 128 and always passes. **Structure 3's contracted extent is `S`**:

| Row | `S` | `tiled` admissible for structure 3? |
| --- | --- | --- |
| C1 prefill | 10 | **no** |
| C1 decode, steps 1–8 | 11 … 18 | only at `S = 16` |
| B1-a prefill | 128 | yes |
| B1-a decode | 129 … 256 | at 8 of the 128 steps |
| B1-d prefill | 8,192 | yes |

**Inference — so `direct` is the only strict realization covering the conformance row's value contraction**, and this refusal fires on the workload's most-run shape rather than on a hypothetical one. A plan that selected `tiled` for structure 3 would need a per-step routing decision over `S mod 16`, which is the first place in this workload where a validity guard and a profitability route are both genuinely required.

## Evidence prerequisite

**Measurement — the realization elimination, restated for these structures.** `direct` and `tiled` are attributed uniquely to the strict fold and consume no permission; `ksplit_contiguous` needs reassociation and `ksplit_strided` needs reassociation *and* permutation; `simdgroup` delivers a fused multiply-add where ADR 0015's contraction dimension is Forbidden **and** seeds its accumulator at `+0.0` where the profile declares no seed; `opaque_mps` is refuted against all twenty-two named topologies with a shape-dependent evaluation on one device.

**Measurement — no cell of either structure has been timed at any shape.** L3 deliberately left the batched forms unmeasured. This ticket's numbers are the first, and nothing in the [L4 design](../docs/research/program-planning/first-attention-program-vertical.md) extrapolates structure 1's table onto them: multiply-accumulate counts are arithmetic, and two schedules with the same count differ by an order of magnitude in L3's own measured table.

**Inference — the arithmetic weight inverts between the two bounded rows.** The block's four projections perform `T · 6,291,456` multiply-accumulates and its two attention contractions perform `4,096 · T²` at `S = T`; they are equal at `T = 1,536`. At C1 the projections dominate 154×, at B1-a 12×, at B1-b 3×; at B1-c the attention contractions dominate 1.3× and at B1-d 5.3×. So a measurement taken only at C1 would rank these kernels on the row where they barely matter.

## Required delivery

- **`direct` for both structures, unconditionally**, bit-identical to the reference evaluator at the C1 prefill extents and at least one B1 extent. It has no structural precondition beyond a positive contracted extent, and it is what makes the conformance row realizable at all.
- **`tiled` for both structures, gated on its own precondition as a typed refusal.** The refusal must name the realization's precondition and the observed extent, and it must be demonstrated firing at `S = 10` before it is trusted — a precondition that has never rejected anything is not a precondition.
- **No K-padding, and the reason recorded.** Padding structure 3's contracted extent to a multiple of 16 would owe the neutrality proof [Numerical semantics](../docs/numerical-semantics.md) requires, and here the padding is measurably wrong in the same way the masked contributors are: the padded contributors are `+0.0 × v`, whose sign follows `v`, and the fold's seed is the first product rather than `+0.0`.
- **A schedule set derived from the current synchronization vocabulary.** **Fact correction:** the repository no longer has only a zero-synchronization profile. `crates/tiler-ir/src/schedule/model.rs`, anchors `ReductionTopology::CooperativeWorkgroup` and `cooperative_synchronization_requirement`, and `crates/tiler-ir/src/schedule/cooperative.rs`, anchor `SynchronizationPoint`, admit a checked workgroup-cooperative tree; [`implement-the-single-workgroup-synchronized-reduction-strategy`](implement-the-single-workgroup-synchronized-reduction-strategy.md) is `done`. Re-derive which reduction implementations each attention realization may use from their exact topology, staging relation, synchronization subject, and target facts. An unavailable topology must refuse for its current missing proof or realization, not for the retired claim that no barrier construct exists.
- **Timings for both structures at the C1 prefill row and at least two B1 rows**, under L3's own procedure — settled minimum over interleaved A/B rounds, round 0 reported separately, spread stated — so the D-A-versus-D-B comparison in [`plan-the-recomputing-attention-decomposition`](plan-the-recomputing-attention-decomposition.md) has a baseline that exists.
- **A refusal for every realization whose reduction topology is unstated or uncovered**, naming reassociation, permutation, or the absent distributivity separately, because those are three different explanations.

## Non-goals

The reassociated split alternatives, which are [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md)'s; the simdgroup route, which [`qualify-the-simdgroup-matrix-contraction-realization`](qualify-the-simdgroup-matrix-contraction-realization.md) owns and which does not survive the governed contract; any opaque provider; any schedule that skips masked contributors, which is [`scope-causal-structure-aware-attention-schedules`](scope-causal-structure-aware-attention-schedules.md)'s and is forbidden until it lands; and the cover and cost decisions, which are [`plan-the-materialized-attention-decomposition`](plan-the-materialized-attention-decomposition.md)'s.

## Closes when

Both structures have a `direct` realization bit-identical to the reference at the C1 prefill extents, `tiled` is available where its precondition holds and demonstrated refusing where it does not, and both are timed at the C1 row and at least two B1 rows with the measurement boundary stated.

## Source-first Fact audit — 2026-08-22, exact base `1fb3675c0ccfca68f62c5d810bd01c2fb5f31c13`

Every Fact re-read in the file or document it names, at this base, by `worker-attention`.

| # | Fact as stated | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | `tiled` refuses a contracted extent that is not a multiple of 16 rather than padding it | **Verified** | `crates/tiler-ir/src/schedule/blocked.rs`, anchor `ContractedTileNotDivisible`, returned by `exact_quotients` on the `AxisKind::Contracted` arm. The refusal carries `contracted` and `tile`, so it names both the precondition and the observed extent. |
| 2 | The `tiled` admissibility table for structure 3 (`no` at C1 prefill; only `S = 16` across C1 decode; `yes` at B1-a prefill; 8 of 128 B1-a decode steps; `yes` at B1-d prefill) | **Verified, and now checked** | Every row re-derived against the admission authority in `crates/tiler-ir/tests/attention_tiled_admission.rs`, anchor `the_tiled_admissibility_table_holds_at_every_row_the_design_states`. The "8 of the 128 steps" figure is counted rather than restated. |
| 3 | Structure 2's contracted extent is the static 128 and always passes | **Verified** | Same test file, anchor `the_score_structures_contracted_extent_admits_at_every_row`. |
| 4 | `direct` has no structural precondition beyond a positive contracted extent | **Verified** | `crates/tiler-compiler/src/physical.rs`, anchor `pub(crate) fn contraction_region`, is shape-generic: it reads `normalized.output_shape` and `normalized.contracted_shape` with no rank or extent special-casing, and returns a region infallibly. Confirmed empirically at rank four — see the outcome below. |
| 5 | No cell of either structure has been timed at any shape | **Verified** | `docs/research/scheduling/first-metal-contraction-realizations.md` scopes its realizations, attribution corpus, and timing rows to structure 1, and names this ticket as owning the measurements that close it. No `result_sha256` exists for either attention structure, which is why the conformance module added here compares two quantities rather than three. |
| 6 | The repository no longer has only a zero-synchronization profile; an unavailable topology must refuse for its current missing proof, not the retired no-barrier claim | **Verified, and this is the Fact that changed the lane** | `ReductionTopology::CooperativeContraction` (tag `0x37`) and `SynchronizationPoint` both exist and are accepted. Re-deriving *which* reduction implementations the attention realizations may use produced the finding below: the current reason the cooperative topology is unavailable for these two structures is **output rank**, not the absence of a barrier construct. |
| 7 | Padding structure 3's contracted extent would owe a neutrality proof, and the padded contributors are `+0.0 x v` whose sign follows `v` | **Verified** | Retained as the reason the refusal is the correct outcome, pinned structurally by `no_contracted_extent_is_rounded_up_to_the_tile_width`: every extent strictly between two tile multiples refuses, so no extent is quietly rounded. |

### The wall this lane found: the cooperative contraction vocabulary is rank-two-output

**Fact.** Both attention structures produce a **rank-four** result — `grtd,gsd->grts` gives `[g, r, t, s]` and `grts,gsd->grtd` gives `[g, r, t, d]`. The tiled cooperative realization is rank-two-output at three independent layers, each verified by reading the file:

- `crates/tiler-ir/src/schedule/cooperative.rs`, `blocked_operand_tile` — the one tile shape three layers construct — builds a rank-two participant space and says so at anchor `Rank two, deliberately`.
- `crates/tiler-ir/src/schedule/builder/contraction.rs` couples that space to the binding's block through `participant_space_matches_block`, which compares ranks and returns `BlockedWorkgroupRule::ParticipantBlockMismatch`.
- `crates/tiler-ir/src/kernel/lower.rs`, `cooperative_contraction_plan`, refuses any region whose `iteration_shape.rank() != 2` with `KernelDiagnostic::CooperativeLoweringShape`.

**Inference — this is a distinct refusal from the `K` precondition and must not be reported as it.** The contracted-extent refusal is a property of one row's `S` and disappears at `S = 128`. The rank wall is a property of the realization's vocabulary and holds at **every** row of **both** structures, including the rows the admissibility table marks admissible. Pinned separately by `the_accepted_blocked_tile_cannot_cover_a_rank_four_attention_output`, which asserts the exact typed `OutputBlockRankMismatch { output_rank: 4, block_rank: 2 }`.

**Consequence for this ticket's Required delivery.** "`tiled` for both structures" is **not achievable at this base** and not for a reason any choice of tile width or extent repairs. What the attention contractions need is a *batched* cooperative contraction — two batch axes (`g`, `r`) carried outside the blocked `(m, n)` pair — which is new schedule, lowering, and emission vocabulary rather than a widening of an existing precondition. That is filed rather than attempted; see the remainder below.

### Why the tiled offer was out of scope regardless

Independently of rank, `tiler-compiler` constructs no cooperative-contraction region at all. `COOPERATIVE_CONTRACTION_REGION` exists at `RegionId::new(9)` and its own doc states *"Nothing in this crate constructs a region with this identifier: the tiled alternative is not offered yet."* Offering it is owned by [`offer-the-tiled-contraction-alternative-once-a-width-authority-exists`](offer-the-tiled-contraction-alternative-once-a-width-authority-exists.md), which depends on a tile-width authority that [`decide-the-contraction-tile-width-authority`](decide-the-contraction-tile-width-authority.md) deliberately declined to declare until a sweep exists, and that sweep ([`calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol`](calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol.md)) is `blocked`. This lane therefore neither offered a tiled alternative nor selected a width, and its `MEASURED_TILE` constant is documented as a measurement used to interrogate the admission authority, never to select a width for a plan.

## Delivered — 2026-08-22

### `direct` for both structures, bit-identical to the reference

**`crates/tiler-compiler/src/governed/attention_conformance.rs`** — a new crate-private conformance module, the sibling of `contraction_conformance` for the two attention structures. It compares the **emitted index region** the governed lowering actually returned, executed by `tiler-reference`'s independent index-region oracle, against the **registered reference evaluator**, on exact `f32` bit patterns.

**Fact — nothing needed widening, which is the substantive result.** `contraction_region` and the governed `StrictTensorContractionF32` lowering are shape-generic, and both five-index structures reached a complete refined region and evaluated correctly at first run with no production change. The structures are spelled with non-dense frontend labels so the renaming-invariant canonicalization is exercised rather than assumed.

Rows reached, and why these:

| Row | fold steps | outcome |
| --- | --- | --- |
| C1 prefill, `T = S = 10`, both structures | 204,800 | agrees bit for bit |
| B1-a decode, `T = 1`, `S = 256`, both structures | 524,288 | agrees bit for bit |
| B1-a prefill, `T = S = 128`, both structures | 33,554,432 | refused by the oracle's step budget, asserted |

**"At least one B1 extent" is met by a decode row, deliberately.** The prefill B1 rows are beyond the index-region oracle's budget. The decode rows are cheap precisely because `T = 1` — one query position against a grown context — so a genuine B1 extent is reached without relaxing a bound that protects the host.

**Measurement boundary, stated: a host comparison is not a dispatched one.** These tests establish that the emitted region computes the same bits as the registered reference. No attention contraction has been dispatched on a device, and no `result_sha256` exists for either structure, so the module compares two quantities where `contraction_conformance` compares three — and says so rather than implying a device baseline it does not have.

### `tiled`'s precondition as a typed refusal, watched firing

**`crates/tiler-ir/tests/attention_tiled_admission.rs`** — the refusal demonstrated firing at `S = 10`, asserted as an exact typed value rather than through a string, plus the L4 admissibility table turned into a checked property, plus the no-padding rule pinned structurally, plus the rank wall named separately from the precondition.

### Perturbations, subject not assertion, with the failure text each produced

| Guard | Perturbation | What it said |
| --- | --- | --- |
| the `K` precondition and the no-padding rule | `exact_quotients` made to `div_ceil` instead of refusing | `10 is not a multiple of 16, so the tiled realization must refuse it: ()`; `C1 prefill, S = 10`; `an extent strictly between tile multiples must refuse, never round: ()` — three tests, while the rank and control tests still passed |
| the rank wall | the rank guard deleted from `ceiling_quotients` only | `a rank-four output cannot take a rank-two block: PredicatedCooperativeContraction { ... work_items: 1600, grid_threads: 256 }` — the guard's own justification, since that launch covers 256 of 1,600 output positions |
| the emitted-region/reference agreement | the reference's contributor order reversed | `Score/c1_prefill: the emitted region disagrees with the reference evaluator` — so the comparison is between two genuinely independent computations and is order-sensitive |
| the oracle budget boundary | `MAX_EVALUATION_STEPS` raised 16Mi to 64Mi | **the refusal still fired**, which refuted this module's own first draft — see below |

**A claim of my own that the perturbation refuted, corrected rather than quietly dropped.** The module first documented the prefill row as refused "at twice the cap", reasoning from fold steps. Raising the constant fourfold left the refusal firing unchanged, so the oracle's budget is not counted in fold steps — it counts scalar applications and index-expression evaluations, of which one fold step performs several. The comment now states the sufficiency direction correctly and records the observation that refuted the earlier wording.

**A second wrong derivation of mine, also recorded in place.** The perturbation test first moved an operand by one unit in the last place, copying the projection harness's phrasing that this is "the smallest change the fold can observe at all". It is not transferable: one ULP of an operand enters the sum multiplied by the other operand, so its contribution falls *below* the accumulator's own ULP and is rounded away. Observed at both `head_dim = 128` (96 outputs bit-identical) and at a contracted extent of four (24 outputs bit-identical) — so shrinking the fold does not fix it, because the fold's magnitude does not fall proportionally with its length. The projection harness's `k = 4` cell flips a bit by luck of its operands, not by a property its instrument has. `PERTURBATION_OFFSET` documents the derivation and perturbs by a whole unit, which discriminates the same defects deterministically.

### Both attention contractions emit Metal, and both goldens compile

**Fact — the `direct` realization of both structures emits MSL at rank four, and nothing needed widening for that either.** `crates/tiler-metal/goldens/contraction_attention_score.metal` and `contraction_attention_value.metal` are the first goldens whose iteration space carries four axes; every other contracts a rank-two output. Both are registered in `golden_compilation.rs`, so `every_checked_in_golden_is_compiled_by_this_module` makes registration non-optional.

**Both were compiled and linked by the qualified Apple toolchain, not only byte-compared.** Run under `DEVELOPER_DIR=/Applications/Xcode.app`, which answers `metalfe-32023.883` — the ledger value. **Stating the invocation matters here**: on this host `xcode-select -p` is `/Applications/Xcode-beta.app/Contents/Developer`, so a bare `xcrun --sdk macosx metal --version` answers `32023.921` instead, and a toolchain fact recorded from the bare form would name a compiler the repository does not compile with.

Three properties of the emitted bodies are pinned, each because a body that got it wrong would still look plausible:

- **No fused multiply-add on the accumulation path.** ADR 0015's contraction permission is Forbidden here and the compiler flag is not sufficient on its own — the L3 spike measured `simdgroup_multiply_accumulate` fusing under `-ffp-contract=off` — so the property is asserted on the emitted text, with a positive half requiring the separate product and sum statements to actually be present.
- **The accumulator is seeded from the first product, never `+0.0`.** Visible as a fold whose loop opens at `1` over a seed computed from contributor zero, which is the profile's declared no-seed rule and the case the L3 record's `negative_zero_seed` counterexample distinguishes.
- **The grouped-query repetition is free, not materialized.** The key operand's address chain is isolated per read and required to recover the group without recovering `r`. This is the assertion that separates the free index from a *correct-but-materialized* alternative, which is the sharp case: a kernel that broadcasts the key across `r` computes exactly the same numbers.

**The two structures differ in exactly one parameter in the fixture**, which is deliberate: they agree on operand 0, on the output, and on the contracted set, and differ only in whether operand 1 reads its contracted axis last or in the middle. Writing the fixture twice would let that one difference drift into an unrelated one, and a lowering that read axis sources positionally rather than by role would produce one of these kernels for both.

### Perturbations for the Metal half, with the failure text each produced

| Guard | Perturbation | What it said |
| --- | --- | --- |
| the golden compile actually reaches the new fixtures | a syntax error injected into the score golden's key address | `golden contraction_attention_score.metal must compile: offline metal failed [...Metal.xctoolchain/usr/bin/metal] (exit code 1): kernel.metal:70:29: error: expected expression` — which is what shows the Apple compiler reaches this fixture rather than the harness self-skipping |
| the free-index property | the key operand rebuilt as `[g, r, s, d]`, materialized across the repetition — the correct-but-materialized twin | `key read 0 recovers the repetition index by dividing by 15, so the key operand is being read per repetition rather than shared across it` |
| golden registration | two goldens added without updating the array length | `error[E0308]: mismatched types ... expected an array with a size of 11, found one with a size of 13` |

### Identity consequences, derived on this tree

| Domain | Moves? | Derivation |
| --- | --- | --- |
| `tiler.schedule.v7` | **no** | No encoder byte was touched; no `ReductionTopology` variant, tag, or field was added or reordered. The reduction-topology tag run is unchanged at `0x31`-`0x35`, `0x37`, `0x38`, and **`0x36` was not taken** — it stays reserved for `CooperativeContractionSplit`. |
| `tiler.kernel-program`, `tiler.artifact-program`, manifest pair | **no** | Nothing in `tiler-ir/src/kernel` or the artifact layer was edited. |
| Target-profile declaration/descriptor `v11` | **no** | No profile row, policy family, or key was added. No tile-width authority was declared. |
| Every *landed* Metal golden | **no** | Unchanged byte for byte; **two goldens added**, which is an addition rather than a movement. See the correction below. |
| Public surface | **no additions** | Every new file is test-only — one `#[cfg(test)] mod`, one `tests/` integration target, two `goldens/` fixtures, and test-module fixtures in `tiler-metal`. The only non-test edits are the three-line module declaration in `governed.rs` and the golden registration in `golden_compilation.rs`. |

**Correction — this table's first revision was written before the Metal half of this lane existed, and said `tiler-metal` was not edited.** True when written, false now: the lane went on to emit both attention contractions and pin them as goldens. The row above states the accurate relation — no landed golden's bytes moved, and two were added. `GOLDENS` grew from 11 entries to 13, and its hand-written array length made that a `rustc` error until updated, which is the good direction; `every_checked_in_golden_is_compiled_by_this_module` is what makes registration non-optional rather than a convention.

**Nothing that was expected not to move, moved.** No identity domain steps: the delivery adds test-only fixtures and two new goldens, and changes no encoder, schema, profile row, or production path.

## Remainder — enumerated and filed, not attempted

Three of this ticket's Required delivery items are not met, each for a stated reason and each with a ticket rather than a note.

1. **`tiled` for both structures** is blocked by the rank wall above, and filed as [`admit-a-batched-cooperative-contraction-for-the-attention-structures`](admit-a-batched-cooperative-contraction-for-the-attention-structures.md). What was delivered instead is the half that is real today: the precondition as a typed refusal, watched firing at `S = 10`, with the admissibility table checked and the no-padding rule pinned. The realization itself needs batched cooperative vocabulary, which is a public boundary and not this ticket's to invent.
2. **Timings at the C1 row and two B1 rows** are filed as [`time-the-attention-contractions-under-the-l3-procedure`](time-the-attention-contractions-under-the-l3-procedure.md), status `blocked`. The `m3` bench host was at load averages `2.44 2.39 2.33` when probed here, against the 0.5 gate the contraction tile-width protocol freezes. Measured, not inherited. Separately, **no contraction equivalent of the device dispatch harness exists** — `crates/tiler-conformance/src/dispatch.rs` covers the serial sum, and the projection contraction reaches a device only through the two `prototypes/` binaries — so a dispatch route for these structures is a prerequisite of that ticket rather than a step inside it.
3. **A refusal for every uncovered realization, naming reassociation, permutation, and absent distributivity separately**, is filed as [`record-typed-refusals-for-uncovered-contraction-realizations`](record-typed-refusals-for-uncovered-contraction-realizations.md). The contraction arm of `govern_spelling` records no decline at all today; the vocabulary to say it already exists on `StrategyDeclineCause` and needs no widening, which is why this is a contained compiler lane rather than a public-surface change.

**The schedule-set re-derivation this ticket asked for was done, and the rank wall is its result.** The Fact correction in Required delivery asked that an unavailable topology refuse for its current missing proof rather than for the retired claim that no barrier construct exists. Re-derived at this base: the barrier construct does exist and is accepted, and the reason the cooperative topology is unavailable for these two structures is output rank. That refusal is typed, is asserted as an exact value, and has been watched firing.

## Unsupported cases, stated

No attention contraction has been dispatched on a device. No `result_sha256` exists for either structure, so nothing here is compared against a measured device result — unlike the projection structure, which has one. The B1 *prefill* rows are beyond the index-region oracle's budget and are asserted refused rather than evaluated. The `tiled` realization cannot represent either structure at any extent. No tile width was selected and no target-profile row was declared.
