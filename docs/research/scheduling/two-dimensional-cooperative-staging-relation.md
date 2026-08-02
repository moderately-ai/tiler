---
schema: "tiler-doc/v1"
id: "tiler.research.scheduling.two-dimensional-cooperative-staging-relation"
kind: "research"
title: "A two-dimensional cooperative staging relation"
topics: ["scheduling", "ir", "gpu", "metal", "contraction", "identity", "public-boundary"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.ir", "tiler.contract.fusion-and-scheduling"]
depends_on: ["tiler.research.scheduling.first-metal-contraction-realizations", "tiler.research.scheduling.two-level-subgroup-workgroup-reduction", "tiler.research.scheduling.scheduled-region-model"]
ticket: "admit-a-two-dimensional-cooperative-staging-relation"
---

# A two-dimensional cooperative staging relation

**Status:** derivation and public-boundary draft for the widened `StagedSpan`/`LocalCoordinates` relation, produced under [`admit-a-two-dimensional-cooperative-staging-relation`](../../../tickets/admit-a-two-dimensional-cooperative-staging-relation.md). It designs; it implements nothing. No encoding, version string, field, or pinned value was changed by the work that produced it, and the identity step it enumerates is a separate serialized wave that lands after Tom rules on the boundary drafted below.

Every repository claim is read at base commit `54833c9`, and every claim labelled **Fact** is either inspected source in this repository — cited by file and line, which is what a reader can refute in one command — or a primary vendor specification whose relay is named. Claims are labelled **Fact**, **Inference**, **Proposal**, and **Measurement**.

**Measurement boundary, stated first because it is total.** *There is no Measurement in this record.* Nothing was executed, emitted, compiled, dispatched, or timed by the work that produced it; no `cargo` invocation ran, no kernel was built, and no device was touched. The arithmetic worked through in §1 is derived by substitution over a stated 16×16 participant domain, not observed. The one performance sentence in §1 is labelled **Inference** and is explicitly not a claim that either candidate is faster on any machine — no cost model in this repository prices a division against a builtin read. The device timings that motivate the tiled realization at all belong to [First Metal contraction realizations](first-metal-contraction-realizations.md) and are not re-established here.

**Tom's acceptance is relayed, and the relay is named rather than dressed up.** The producing ticket's body records that "Tom accepted the `tiler.schedule.v4` → `v5` step and the widened boundary at the live session, witnessed and executed by the coordinator" on 2026-08-01, and that record reached this work through the dispatch brief. What that acceptance covers is the **step and the widened boundary in principle**; the ticket's own sentence says the exact boundary "comes back to Tom as a draft under ADR 0075", which is what §6 is. Nothing has been released on the relay, no contract sentence has been rewritten under it, and no encoding has moved — so if the relay is wrong, the repair is deleting one drafted span and one carrier ticket.

## Conclusion

**The shape fork has one survivor and there is no question for Tom in it.** The `OffsetTerm` form `stride * ((l / divisor) % modulus) + offset` covers the tiled contraction's two staged *reads* and cannot express its two staged *writes* — the `b_tile` transpose write `16 * (l % 16) + (l / 16)` is a sum of two distinct divisor projections and no single term equals it, refuted by two points. It is the cheaper option whose saved cost is the part that made the answer correct. A **two-dimensional participant space with a per-dimension stride** expresses all four accesses, is what the measured kernel's own source already reads, is what accepted ADR 0096 decision 4 constrains toward, and makes the tile shape a fact the verifier can check against the launch. One candidate survives; §1 states the elimination so a reader can refute it rather than only the conclusion.

**The round ordinal does not enter this relation, and the two consumers are two relations rather than one.** The tiled contraction's staged slot indices are round-*invariant* — `a_tile[local_m * TILE + local_n]` is the same slot on every round and only the value it holds changes — so nothing in the contraction's need reaches the round. The log-depth tree's need does, and admitting it here would land half a capability twice over: a round-dependent span breaks the per-round bijection the occupancy map decides, and the tree separately needs the per-access active-participant subset this ticket explicitly does not own. §2 derives it; the remainder is filed rather than absorbed.

**The relation *is* ADR 0096's item 2, and it is part of item 3.** One multi-component `LocalCoordinates` serves both, with the `LocalCoordinateSource` variant carrying the one thing that genuinely differs — whether the components have a defined relation to the linear index, which they do for a threadgroup tile and do not for a subgroup pair. The per-dimension stride subsumes item 3's "component selector" and answers what the disjointness enumeration ranges over; it does not answer item 3's derived writer set, which is the same subset construct the log-depth tree is refused for wanting. §3 states which of the seven items the draft resolves.

**Decidability is unchanged, and this is the load-bearing negative result.** The widened enumeration ranges over the Cartesian product of the participant extents, whose cardinality is *the same set* today's linear enumeration walks — the participants are re-indexed, not multiplied — so `MAX_COOPERATIVE_PARTICIPANTS` and `MAX_COOPERATIVE_STAGING_SLOTS` bound it exactly as they bound the current form. The occupancy map is keyed on slots and is untouched by re-indexing the participant domain, so the one-writer-per-slot-per-round rule still refuses two writers reaching one slot, and §4 exhibits a widened statement it refuses.

**The `v5` step moves thirty-one pinned lines across nine files, and six of them are not in `crates/tiler-ir`.** Every Metal golden — including the four with no cooperative tile — carries an entry symbol, a kernel identity digest, and a scheduled-region identity digest, all of which move, because the kernel identity folds the scheduled-region identity bytes whole at `crates/tiler-ir/src/kernel/model.rs:1757` and the separator is the leading eighteen bytes of every one of them. §5 enumerates them by file and line with the command that produced the list.

## 1. The shape elimination, run on the tile

### The kernel the elimination is run against

**Fact — the measured realization, read at `spikes/scheduling/metal_contraction_vertical/kernels.metal:98-145`.** `contract_tiled` launches a 16×16 threadgroup, declares `uint2 tid [[thread_position_in_threadgroup]]`, and sets `local_m = tid.y`, `local_n = tid.x`. Its four staged accesses are:

| Access | Source line | Slot addressed by participant `(local_m, local_n)` |
| --- | --- | --- |
| `a_tile` write | `a_tile[local_m * TILE + local_n] = …` | `16·local_m + local_n`, one slot |
| `b_tile` write | `b_tile[local_n * TILE + local_m] = …` | `16·local_n + local_m`, one slot |
| `a_tile` read | `a_tile[local_m * TILE + kk]`, `kk ∈ 0..16` | `16·local_m + kk`, sixteen contiguous slots at `16·local_m` |
| `b_tile` read | `b_tile[local_n * TILE + kk]`, `kk ∈ 0..16` | `16·local_n + kk`, sixteen contiguous slots at `16·local_n` |

**Fact — the kernel never computes a linear thread index.** `thread_index_in_threadgroup` does not occur in `contract_tiled`; the only local coordinate it reads is the two-component `[[thread_position_in_threadgroup]]`. The exact check is `grep -n 'thread_index_in_threadgroup' spikes/scheduling/metal_contraction_vertical/kernels.metal`, which returns nothing.

**Fact — the current vocabulary, read at `crates/tiler-ir/src/schedule/cooperative.rs:276-284` and `:571-587`.** `StagedSpan { stride, offset, count }` addresses `count` contiguous slots at `stride * l + offset`, and `CooperativeTile::addressed_slots` enumerates it over the *linear* participant coordinate `l ∈ participants.first .. participants.first + participants.count`.

To state the four accesses in that vocabulary the participants must be linearized. Taking the MSL linearization — **Fact, MSL 4.1 §5.2.3.6 page 153, quoted in [the two-level subgroup-then-workgroup reduction](two-level-subgroup-workgroup-reduction.md) at line 183, which is the relay this record cites rather than a specification it read itself**: "the thread index in the threadgroup (`thread_index_in_threadgroup`) is determined by: `ly * Sx + lx`" — gives `l = 16·local_m + local_n`, so `local_m = l / 16` and `local_n = l % 16` over the 256-element domain.

### Candidate A — the `OffsetTerm` form, `ParticipantRange` left one-dimensional

The narrowest widening the producing ticket names: a span's base becomes `stride * ((l / divisor) % modulus) + offset`, which is the shape the kernel lowering already uses for tensor offsets, and the tile's shape lives in the divisor.

**Fact — it covers both reads.** `base_a(l) = 16·(l/16)` is `stride = 16, divisor = 16, modulus = 16, offset = 0`: at `l = 0..15` the quotient is `0`, at `l = 16..31` it is `1` giving `16`, and at `l = 240..255` it is `15` giving `240`. `base_b(l) = 16·(l%16)` is `stride = 16, divisor = 1, modulus = 16, offset = 0`: `l = 0 → 0`, `l = 1 → 16`, `l = 16 → 0`. Both read spans are sixteen contiguous slots, which `count = 16` states. This is the coverage the ticket's refutation of the *current* form establishes the need for, and candidate A does deliver it.

**Fact — it cannot express the `b_tile` write, and the refutation is two points.** The write's base profile is `w(l) = 16·(l % 16) + (l / 16)`, so `w(0) = 0`, `w(1) = 16`, and `w(16) = 16·(16 % 16) + (16/16) = 0 + 1 = 1`. Suppose `stride·((l/d) % m) + off` equals it. From `w(0) = 0`: `(0/d) % m = 0` for every `d ≥ 1, m ≥ 1`, so `off = 0`. Now split on `d`:

- `d = 1`: `w(1) = stride·(1 % m) = 16` forces `m > 1` and `stride = 16`. Then `w(16) = 16·(16 % m)` must equal `1`, and `16·x = 1` has no solution in the non-negative integers. Refuted.
- `d ≥ 2`: `1 / d = 0`, so `w(1) = stride·(0 % m) = 0 ≠ 16`. Refuted.

Every `d ≥ 1` is covered, so no `(stride, d, m, off)` states the write. **Inference — and this is a vocabulary wall rather than a spelling one, by the same argument the ticket makes for the reads.** `w` is neither constant nor injective: it takes 256 distinct values over the 256-element domain — it *is* injective, in fact, being the transpose permutation — so the constant case is out, and injectivity does not help, because `stride·((l/d)%m)` is injective only when `d = 1` and `m > l_max`, in which case it is the affine map `stride·l` and `w` is not affine (`w(1) − w(0) = 16` while `w(16) − w(15) = 1 − 240 < 0`).

**Fact — the transpose cannot be moved to the read side to escape this.** If `b_tile` were written untransposed as `b_tile[local_m·16 + local_n]`, slot `i·16 + j` would hold `B[n₀+j][k₀+i]`, while the fold's read of `b_tile[local_n·16 + kk]` requires slot `i·16 + j` to hold `B[n₀+i][k₀+j]`. Repairing that means reading `b_tile[kk·16 + local_n]` instead, whose sixteen addressed slots are `{16·kk + local_n : kk ∈ 0..16}` — a *strided* set, and `StagedSpan.count` states `count` **contiguous** slots (`cooperative.rs:583-585` steps `base + 1` per element). So one of the two accesses is forced outside candidate A's vocabulary whichever side the transpose sits on. The transpose is not a spike idiosyncrasy either: it is what makes the fold's `b_tile` read contiguous, which is the point of staging `b` at all.

**Inference — candidate A also has no round variable, so it does not reach the second consumer either.** `stride·((l/d)%m) + offset` names `l` and four constants; a span whose stride and count are functions of the round ordinal is not a substitution into it.

**Inference — the divisor is a number no rule can relate to anything.** Under candidate A nothing in the schedule states that the tile is sixteen wide. `verify_cooperative_tile` checks `participants.count == threads_per_workgroup` (`crates/tiler-ir/src/schedule/builder.rs:1147`) and nothing else about the coordinate space, so a producer stating `divisor = 15` with 256 participants passes every rule the verifier has: the write occupancy map would catch a resulting collision, but a *read* span is only checked for capacity (`builder.rs:1258` calls `addressed_slots` and discards the result), so a wrong divisor on a read is admitted and emits a silently wrong broadcast. That is the defect class AGENTS.md concentrates scrutiny on, introduced by the cheaper option.

**Verdict: candidate A is discarded on correctness.** It fails to express the staged writes of the exact kernel it was proposed to admit, fails to reach the second consumer, and admits a shape parameter no verifier rule can refuse. It is not rescued by being narrower.

### Candidate B — a two-dimensional participant space with a per-dimension stride

The participant set is a shape rather than a run: extents `[16, 16]`, slowest-varying first, and a participant is the coordinate `(l₀, l₁) = (local_m, local_n)`. A span's base is `offset + Σ_d strides[d]·l_d`, and `count` contiguous slots follow it.

**Fact — all four accesses are stated, with contiguous counts throughout.**

| Access | `strides` | `offset` | `count` |
| --- | --- | --- | --- |
| `a_tile` write | `[16, 1]` | `0` | `1` |
| `b_tile` write | `[1, 16]` | `0` | `1` |
| `a_tile` read | `[16, 0]` | `0` | `16` |
| `b_tile` read | `[0, 16]` | `0` | `16` |

Substituting: the `b_tile` write gives participant `(1, 0)` the slot `1·1 + 16·0 = 1`, which is `w(16) = 1` — the exact point that refuted candidate A. The `a_tile` read gives every participant of row `local_m` the run `[16·local_m, 16·local_m + 16)`, which is many-to-one in the column dimension and one-to-one in the row dimension; the `b_tile` read is its transpose. **Inference.** That two-dimensional many-to-one structure *is* the tiling — a tile row shares an operand row and a tile column shares an operand column — so a relation that can state it is stating the thing the schedule exists to express, not encoding it.

**Fact — the tile shape becomes checkable against the launch.** The participant extents' product is the participant count, so `verify_cooperative_tile`'s existing workgroup-width equality generalizes from `participants.count == threads_per_workgroup` to `∏ extents == threads_per_workgroup` and continues to fire. Nothing analogous exists for candidate A's divisor.

**Fact — accepted ADR 0096 decision 4 constrains toward this and rejects the alternatives by name.** "A participant's local coordinate has two components — subgroup index and lane index — and the schedule states their composition. A one-dimensional coordinate cannot name the structure the composition combines over. A strided participant range does not repair it, because the staged slot address would have to be a fraction of the participant coordinate; a declared per-access participant subset is refused because it is the construct a log-depth workgroup tree is refused for wanting." **Inference.** Landing candidate A here would leave two levels of one vocabulary disagreeing about whether a participant coordinate has components, which is the defect ADR 0096 decision 2 refuses for a different field under the same reasoning.

**Inference — the emission cost, and it is an inference and not a measurement.** Under candidate B a Metal lowering reads the components directly from `[[thread_position_in_threadgroup]]`, which is what the measured kernel does; under candidate A it must reconstruct them, emitting one integer division and one modulus per staged access per round. Nothing here prices either, no cost model in this repository does, and this is stated only to record that the discarded candidate is not the cheaper one at emission either — it is not a reason the elimination rests on.

**Verdict: candidate B survives.** The elimination converges, so under AGENTS.md there is no question for Tom about the *shape*. What is Tom's is the exact spelling, which §6 drafts.

**What would refute this.** A statement of the `b_tile` write as a single `stride·((l/d)%m) + offset` term with the four constants exhibited, or a demonstration that the tiled realization does not need a transposed staged operand — either would reopen candidate A. A demonstration that some other participant linearization makes `w` affine would not, because the refutation above is over the linearization the MSL specification fixes and the write profile is a transposition under any of them.

## 2. Whether the round ordinal enters the relation

The producing ticket deliberately declines to assume the round ordinal is the same axis as a second participant dimension, because `workgroup_tree_tile` is depth two rather than log-depth partly for wanting "a span whose stride and count are functions of the round ordinal".

**Fact — the two consumers' needs differ in this exact respect.**

- **Fact — the tiled contraction's slot indices are round-invariant.** `spikes/scheduling/metal_contraction_vertical/kernels.metal:128-133` writes `a_tile[local_m * TILE + local_n]` and `b_tile[local_n * TILE + local_m]` inside the `k0` loop with the identical indices it used before the loop at lines 116-119. What varies with the round is the *device address the value is loaded from* (`k0 + local_n`), never the staged slot. So the contraction's four spans are the same on every round and nothing in §1 reaches the round.
- **Fact — the log-depth tree's do vary.** `crates/tiler-ir/src/schedule/cooperative.rs:70-75` states the tree needs "a per-access active-participant subset, separate from a phase's `participation`, which is *arrival* and must stay uniform" **and** "a span whose stride and count are functions of the round ordinal, since each level halves them", and that "both are absent rather than reserved".

**Inference — they are different kinds of axis, and the difference is what the verifier's decidability rests on.** A participant dimension indexes *concurrent* invocations: at one instant different participants hold different values of it. The round ordinal indexes *sequential* iterations: at one instant every participant holds the same value of it. `verify_cooperative_tile`'s occupancy map is documented at `builder.rs:1205-1210` as spanning "the phase sequence once, which is exactly one round — every phase runs on every round — so this needs no round dimension and gains none". That sentence is true precisely because a span carries no round dependence. Admit one and it stops being true: a round-dependent span writes different slots on different rounds, so "every in-range slot written on every round" — the coverage half of `builder.rs:1200-1250` — becomes false as stated, and coverage would have to be re-derived as a union over rounds while disjointness stayed per round. That is a different decision procedure, not a wider parameter for the same one.

**Inference — and the round dependence alone would not deliver the tree.** For the log-depth tree at round `r`, `participants / 2^(r+1)` lanes write and the rest write nothing; per-round coverage is a shrinking subset rather than a bijection. So the tree needs the active-participant subset as well, and that subset is what ADR 0096 decision 4 and `cooperative.rs:74` both refuse by name. Landing a round variable here would ship half of one capability into a rule set that the other half has to change again.

**Conclusion — two relations, not one, and this record admits the first.** The participant-space widening of §1 is complete for the tiled contraction and for ADR 0096's composition, both of which stage on a fixed slot layout. The round-dependent span is a separate widening that must land together with the per-access active-participant subset, and it is filed as [`admit-a-round-dependent-cooperative-staging-span`](../../../tickets/admit-a-round-dependent-cooperative-staging-span.md) at `deferred` with its trigger stated, rather than absorbed here.

**What would refute this.** A log-depth tree formulation whose staged slot set is round-invariant and whose narrowing lives entirely in `commit`-like declared facts would make the round variable unnecessary rather than separable — but it would still not need the round variable, so it refutes the *filing*, not the conclusion. A tiled contraction variant whose staged slot layout rotates per round — double buffering is the concrete shape — would put the round dependence back inside this ticket's consumer, and that is the filed ticket's second stated trigger.

## 3. Interaction with ADR 0096 decision 4's two-component coordinate

ADR 0096 is accepted (by Tom on 2026-08-01, per its own status line), and what was accepted is the model and none of its seven public-boundary items. **Fact — items 2 and 3, quoted from [the two-level subgroup-then-workgroup reduction](two-level-subgroup-workgroup-reduction.md) lines 342-343**, which is where they are enumerated:

> 2. **A second `LocalCoordinateSource` variant, and the decision that a participant's coordinate may have two components** — whether that is two sources composed by the schedule or one source naming a pair, and the statement that neither component carries a defined relation to `LocalLinearInvocation`. Shared in substance with ADR 0094's item 4.
>
> 3. **A staged access addressed by a named coordinate component, with a derived writer set** — whether `StagedSpan` gains a component selector, whether the writer narrowing is a field beside `commit` or a property of the span, and what the enumeration that decides disjointness ranges over.

**Item 2 — the same concept, and the draft resolves it as one.** Both needs are "a participant's coordinate has components". They differ in exactly one respect, and it is a real one: **Fact** — ADR 0096's components are `(subgroup index, lane index)`, and MSL 4.1 §5.2.3.6 page 153 states that "within a threadgroup, threads are divided into SIMD-groups in an implementation-defined fashion", while WGSL §15.5 states there is "no defined relationship between subgroup values … and `local_invocation_index`". **Neither quotation was read from its specification by this record**; the MSL one is relayed from [the two-level subgroup-then-workgroup reduction](two-level-subgroup-workgroup-reduction.md) line 183 and the WGSL one from [the subgroup execution tier](subgroup-execution-tier.md) line 341, and both are cited as relays rather than as primary reads. This record's components are `(local_m, local_n)` of a threadgroup tile, and the same MSL section *does* fix their relation to the linear index: `ly · Sx + lx`.

**Inference.** So the *construct* is one — a coordinate space with per-dimension extents — and what differs is the *source*, which is exactly the fact `LocalCoordinateSource` exists to name. That is the shape ADR 0096 asks for ("should land as one concept each rather than as two") and it answers item 2's open sub-question in the "one source naming the space" direction: one `LocalCoordinateSource` value per governed execution key, with the extents carried beside it on `LocalCoordinates`, and the source variant's documentation carrying whether the decomposition against the linear index is defined. **The draft therefore resolves item 2**, subject to Tom's acceptance of the spelling in §6.

**Item 3 — the same span concept, and the draft resolves two of its three sub-questions.**

- *"Whether `StagedSpan` gains a component selector"* — **resolved, and differently from the way the item words it.** A per-dimension stride vector is strictly more general than a component selector and subsumes it: selecting component `d` is the stride vector that is zero everywhere but `d`. ADR 0096's staged write, whose writers are "the participants whose lane coordinate equals the result lane", addresses one slot per subgroup and is the stride vector `[1, 0]` over `(subgroup index, lane index)`. So one field serves both, and a selector would be the special case named as if it were the general one.
- *"What the enumeration that decides disjointness ranges over"* — **resolved**: the Cartesian product of the participant extents, which is the same participant set today's linear enumeration walks, re-indexed. §4 is the argument.
- *"Whether the writer narrowing is a field beside `commit` or a property of the span"* — **not resolved, and deliberately.** That narrowing is a per-access active-participant subset in everything but name, which is the construct `cooperative.rs:70-75` records as absent rather than reserved and which §2 keeps out of this widening. ADR 0096 decision 3 *derives* it ("a total function of the width and the result lane rather than a subset a schedule states"), which is a genuinely different resolution from the log-depth tree's declared subset — but it is still a second construct, and it belongs to the ticket that lands ADR 0096's topology.

**Which of ADR 0096's seven items this draft resolves.** Item 2, fully. Item 3, its first and third sub-questions. Items 1 (the two-level `ReductionTopology` variant), 4 (the `CombineTree` vocabulary), 5 (the stated contributor-block coordinate), 6 (the `0x36` tag and its appends-only argument), and 7 (the `MemoryScope::Subgroup` trigger narrowing) are untouched and remain enumerated where they are.

**One consequence worth stating, because a reader will otherwise infer the opposite.** ADR 0096's item 6 argues that a new topology tag `0x36` is "appends-only injective where an extension of the existing arm is not", and that argument is unaffected by this widening in its own terms — but it is stated against a `v4` tree. If this record's step lands first, `0x36` is appended to a `v5` encoder and the appends-only claim must be re-made at the encoding site on the tree the change lands into, which is what item 6 already requires. Neither change makes the other harder; they are ordered, not coupled.

## 4. Decidability under the governed bounds

**Fact — why the bounds exist.** `crates/tiler-ir/src/schedule/mod.rs:189-205` states that `MAX_COOPERATIVE_PARTICIPANTS` (4,096) exists "so the tile's disjointness and coverage rules can be decided by enumerating every addressed slot rather than by a modular argument over an unbounded participant count", and that `MAX_COOPERATIVE_STAGING_SLOTS` (65,536) is bounded for the same reason "and separately, because coverage is decided over the slot space rather than the participant space".

**Fact — what the enumeration costs today.** `CooperativeTile::addressed_slots` (`cooperative.rs:571-587`) walks `participants.count` participants and pushes `span.count` slots for each, so one span's enumeration is `participants.count · span.count` slot values, and `builder.rs:1232` and `:1258` bound the result by requiring every produced slot to be below the allocation's `slots`, itself bounded by `MAX_COOPERATIVE_STAGING_SLOTS` at `builder.rs:1130-1134`.

**Inference — the widened enumeration has the identical cardinality.** The widened walk ranges over the Cartesian product `∏ extents` of the participant space. That product **is** the participant count: the same invocations, indexed by a tuple instead of an integer. `verify_cooperative_tile`'s existing rule that the participant count equal the launched workgroup width becomes `∏ extents == threads_per_workgroup`, and `threads_per_workgroup` is a `u32`, so the product is bounded by the same `MAX_COOPERATIVE_PARTICIPANTS` check the current `participants.count` receives at `builder.rs:1120`. Nothing is multiplied; a 16×16 space enumerates 256 participants exactly as a 256-run does. Per participant the span still contributes `count` contiguous slots under the same capacity refusal. So the enumeration stays finite under the same two constants, with no third bound needed for it.

**Proposal — one new bound is needed, and it bounds the arithmetic rather than the enumeration.** The base is now a sum `offset + Σ_d strides[d]·l_d` over the space's rank, so the rank must be bounded for the sum to be a fixed amount of work and for the encoding to be framed. `MAX_COOPERATIVE_PARTICIPANT_RANK` is the bound, refused under the existing `CooperativeTileRule::LocalCoordinates`. It is not implied by the participant bound: an extent of `1` is degenerate but well formed, so a rank-4,096 space of unit extents has a product of 1 and would otherwise pass. Overflow discipline is unchanged in kind — each `strides[d]·l_d` is a `checked_mul` and each accumulation a `checked_add`, exactly as `cooperative.rs:579-581` does today, and a failure is the same `StagingCapacity` refusal because it means the same thing: the span leaves the storage the tile declared.

**Fact — disjointness is untouched by the widening, because it is keyed on slots.** `builder.rs:1211-1244` builds one `Vec<bool>` per allocation indexed by slot ordinal and refuses on `std::mem::replace(slot, true)` returning `true`. Re-indexing the participant domain changes which participant contributed a slot; it does not change the slot space the map is over, nor the round scope the map spans.

**The rule still refuses two writers reaching one slot inside one round, exhibited on a widened statement rather than asserted.** Take the tile's `b_tile` write and perturb it to `strides = [16, 16]`, `offset = 0`, `count = 1`. Participant `(0, 1)` addresses slot `16·0 + 16·1 = 16`; participant `(1, 0)` addresses slot `16·1 + 16·0 = 16`. Both writes are in one phase, so both fall on the same side of every point that could order them, and the second `replace` returns `true` — `CooperativeTileRule::StagingConflict`. The correct statement `strides = [1, 16]` gives `(0,1) → 16` and `(1,0) → 1`, and the 256 participants cover the 256 slots exactly once, satisfying coverage. **Inference**, by substitution over the stated domain; nothing here was executed, and the perturbation is a case for the implementation wave to watch fail rather than a run this record performed.

**Conclusion — the widened relation stays decidable under the same bounds, and no stop condition fires.** The decidability argument the whole cooperative verifier rests on survives the widening intact, which is the one result that would have ended this dispatch had it gone the other way.

## 5. The identity step's blast radius, enumerated by file and line

**Fact — why every region moves, not only cooperative ones.** `crates/tiler-ir/src/schedule/model.rs:1878` writes `b"tiler.schedule.v4\0"` as the first eighteen bytes of *every* scheduled-region identity, and `crates/tiler-ir/src/schedule/builder.rs:4484`'s `the_round_step_moves_only_the_domain_separator` proves exactly that: it compares the recorded `v4` and `v3` identities of a region that stages nothing and asserts they differ in the first thirty-six hex digits and agree past them. **Fact — the reach beyond the schedule layer is a fold, not a second version.** `crates/tiler-ir/src/kernel/model.rs:1757` writes `push_slice(&mut bytes, schedule_identity.as_bytes())` — the kernel identity frames the scheduled-region identity bytes whole, separator included — and `crates/tiler-build/src/metal_plan.rs:813-818` records the rest of the chain: the artifact identity frames each entry's kernel-program identity, which frames the kernel identity. So a separator change reaches every kernel, kernel-program, artifact, and cache-subject value in the repository, and no domain between them steps.

**The reproducing command.** One line, run from the repository root; it names its population rather than reporting silence:

```sh
grep -rnE 'tiler_kernel_[0-9a-f]{16}|kernel identity digest:|scheduled region identity digest:|74696c65722e7363686564756c652e|4e91bfbe59072c3e|2a192388f39a8584|tiler\.schedule\.v4' \
  --include='*.rs' --include='*.md' --include='*.metal' --include='*.toml' --include='*.tsv' --include='*.json' . \
  | grep -v '^\./target'
```

At `54833c9` it returns **61 lines across 23 files**. Those 61 lines are the *candidate* population and nothing more; the classification below is what a reader has to check, because the command cannot tell a pin from a prose mention of one, from a dated measurement that must **not** move, or from the emitter that produces the label it matched. The four classes below account for all 61 — 31 + 7 + 15 + 8 — and their file counts sum to more than 23 because three files appear in two classes.

### A. Values the `v5` commit must recompute — 31 lines, 9 files

| File | Lines | What moves |
| --- | --- | --- |
| `crates/tiler-ir/src/schedule/model.rs` | 1878 | The version string itself, at its owning layer |
| `crates/tiler-ir/src/schedule/builder.rs` | 1683 | `STRICT_F32_REGION_IDENTITY_HEX`, the pinned strict-`f32` region identity |
| `crates/tiler-ir/src/schedule/builder.rs` | 1691 | `STRICT_F32_REGION_IDENTITY_HEX_V3`, the retained comparison — see the note below, it does not simply move |
| `crates/tiler-build/src/metal_plan.rs` | 858, 860 | `ARTIFACT_IDENTITY` and `CACHE_SUBJECT` |
| `crates/tiler-build/src/metal_plan.rs` | 840, 842 | The same two values restated in the doc comment that records how to regenerate them |
| `crates/tiler-metal/goldens/pointwise_scale_bias.metal` | 35, 36, 37, 41 | Entry symbol ×2, kernel identity digest, scheduled-region identity digest |
| `crates/tiler-metal/goldens/reduction_single_axis.metal` | 35, 36, 37, 41 | The same four |
| `crates/tiler-metal/goldens/reduction_multi_axis.metal` | 35, 36, 37, 41 | The same four |
| `crates/tiler-metal/goldens/reduction_fused_multiply_add.metal` | 35, 36, 37, 41 | The same four |
| `crates/tiler-metal/goldens/contraction_strict_tensor.metal` | 35, 36, 37, 42 | The same four |
| `crates/tiler-metal/goldens/cooperative_workgroup_reduction.metal` | 35, 36, 37, 42 | The same four |

**Fact — five of the six goldens carry no cooperative tile and move anyway.** `pointwise_scale_bias`, `reduction_single_axis`, `reduction_multi_axis`, `reduction_fused_multiply_add`, and `contraction_strict_tensor` never reach the `0x35` topology payload; their identities move for the eighteen separator bytes alone, through the fold. Only `cooperative_workgroup_reduction` stages. Corrected at integration from "four", which omitted `contraction_strict_tensor` — the class A table above already listed all six, so the action was unaffected; the exact check is `grep -lE 'threadgroup|barrier' crates/tiler-metal/goldens/*.metal`, which returns that one file. That is what a domain separator costs, and it is the same consequence `metal_plan.rs:809-812` records for the `v4` step.

**Fact — the goldens are recomputable in the gate and each names its own test.** `crates/tiler-metal/src/tests.rs:1025, 1034, 1043, 1052, 1061, 1070` are the six `*_matches_its_golden_source` tests, and `assert_golden` at `:1017` reports "golden fixture crates/tiler-metal/goldens/{name} is stale". `crates/tiler-metal/src/golden_compilation.rs:128-157` separately requires the compiled set to be the complete `goldens/` directory, so a golden cannot be dropped instead of rebaselined.

**The one line that needs a decision rather than a recomputation.** `builder.rs:1691`'s `STRICT_F32_REGION_IDENTITY_HEX_V3` exists, per its own doc at `:1685-1690`, to make the `v4` step's blast radius "a measured fact instead of an assurance". Carried forward unchanged it would make `the_round_step_moves_only_the_domain_separator` compare `v5` against `v3` — a two-step claim, which is *weaker*, because two separator changes agreeing past the tag says nothing about whether the payload moved at either step individually. **Proposal:** the constant is rebaselined to the `v4` value and renamed accordingly, and the test's name and doc move with it, so the retained comparison keeps proving exactly one step. Not a free rename: it discards the `v3` datum. That is the right trade, because the datum's whole content was the `v3 → v4` claim, which the commit that made it already carries.

### B. Prose that must move in the same commit — 7 lines, 4 files

`docs/artifact-abi.md:207` (the identity ledger's `tiler.schedule.v4` entry), `:213`, and `:215` (the `v4` step's own record, which becomes a prior step); `crates/tiler-ir/src/schedule/model.rs:1722` and the whole `## Why this is a v4 step` block at `:1841-1875`; `crates/tiler-ir/src/schedule/builder.rs:1671` and `:4472`; `crates/tiler-build/src/metal_plan.rs:809`. **Inference.** `docs/artifact-abi.md` is the ledger AGENTS.md's identity-step discipline names, and it is under `contracts/artifacts` rather than `implementation/ir` — so the implementation wave's ticket must hold both scopes, or the step lands in halves, which is the failure mode the discipline exists to prevent.

### C. Dated measurements that must **not** move — 15 lines, 9 files

Every remaining hit is a transcript or an observation qualified by a host and a commit, and rebaselining one would be falsifying a record rather than maintaining a pin: `tickets/dispatch-a-tiler-region-on-metal-hardware.md:62`, `tickets/state-a-numerical-contract-in-the-inline-dispatch-spike.md:43`, `tickets/validate-metal-payload-argument-slots-against-declared-bindings.md:43` and `:48`, `tickets/route-the-runtime-proof-through-the-artifact-envelope.md:95`, `tickets/route-an-embedded-artifact-through-a-consumer-storage-seam.md:85`, `tickets/prototype-metal-runtime-preflight.md:59`, `tickets/bound-the-backend-entry-key-by-the-identity-it-carries.md:122`, `tickets/extend-canonical-identity-encodings-for-reserved-variants.md:109`, and `spikes/runtime/inline-dispatch/README.md:51, 87, 90, 91, 117, 125`.

**One of these is a genuine finding rather than a classification, and it is reported rather than resolved.** `spikes/runtime/inline-dispatch/README.md` does not merely record a symbol once; lines 90-91 track it *across* commits — "`tiler_kernel_ce0acbceb6c201da` when this record was written at `8366ecd` and is `tiler_kernel_ae031ce7240f7495` at the base above" — so the README's own convention is to restate the symbol when the base moves, and line 117's transcript is a live-dispatch record from an Apple GPU host. **Inference — this is not the stop condition the brief names, and the distinction matters.** The symbol is derivable from the compile path alone and needs no device; only the surrounding transcript's object length and value table need one, and those do not move with an identity step. So nothing here is a pinned identity the repository cannot recompute. What it *is* is a hand-maintained cross-commit pin that no gate checks and that will silently go stale at `v5`, which is exactly the corpus hazard AGENTS.md describes for the documentation system. The implementation wave should add one dated line to that README recording that the symbol moved at `v5` and what it moved to, without rewriting the transcript — and that is the only treatment consistent with the record being a measurement.

### D. Prose describing the mechanism, which neither pins nor measures — 8 lines, 4 files

`crates/tiler-metal/src/emit.rs:580` and `:585` are the format strings that *emit* the two digest labels the command matches; they are the producer and move only if the label text changes, which this step does not. `tickets/admit-a-two-dimensional-cooperative-staging-relation.md:32, 43, 52`, `tickets/realize-the-strict-contraction-on-metal.md:213, 267`, and `tickets/generalize-payload-provenance-beyond-the-apple-shape.md:45` are ticket bodies naming the step in prose; the first is this work's own ticket, whose Outcome section records the step rather than restating it, and the other three are records of decisions and landings that were true when written. **Inference — this class is why the command's raw count is not a work estimate.** Eight of its sixty-one lines require nothing at all, and a wave that treated the grep output as a task list would edit an emitter and three closed records.

**What would refute this enumeration.** A pinned value that folds a scheduled-region identity and matches none of the seven patterns in the command above — the residual risk is a digest pinned in a form none of them reach, for instance a base64 or byte-array literal, or a golden whose symbol is spelled without the `tiler_kernel_` prefix. The check that would close it is running the gate after the version bump and comparing the failure set against class A; that check belongs to the implementation wave, and it is the reason this record states its population, counts it, and partitions it exhaustively rather than claiming completeness — an unaccounted-for remainder is exactly how half an identity step happens.

## Public-boundary items, enumerated for Tom and not self-accepted

Nothing below is implemented, and none of it is accepted by this record's existence. Each arrives at Tom under ADR 0075, and §6 is the drafted body they arrive in.

1. **The `tiler.schedule.v4` → `v5` domain step**, and the 31 lines §5 enumerates. Accepted in principle per the relayed 2026-08-01 acceptance; the enumeration is what the acceptance is executed against.
2. **`LocalCoordinates` carrying a participant *space* rather than a participant *range***, and with it the removal of a `first` field the verifier already pins to zero.
3. **A second `LocalCoordinateSource` variant** and the decision that a coordinate may have components — ADR 0096's item 2, resolved here as one concept, with the source variant carrying whether the decomposition against the linear index is defined.
4. **`StagedSpan` carrying a per-dimension stride vector**, and the loss of `Copy` on `StagedSpan`, `StagedWrite`, `StagedRead`, and `LocalCoordinates` that a bounded-rank `Vec` costs.
5. **`CooperativeTileRule::SpanRank`**, one new error variant, on a `#[non_exhaustive]` enum.
6. **`MAX_COOPERATIVE_PARTICIPANT_RANK`**, one new governed bound.
7. **`CooperativeTile::addressed_slots` changing from by-value to by-reference parameters**, which is a breaking change to an existing signature and therefore always-ask under ADR 0075 regardless of how mechanical it is.

## Deferrals, each with the evidence that would close it and a trigger

- **A round-dependent staged span is not admitted.** §2 derives that it is a different axis from a participant dimension and that it breaks the per-round bijection the occupancy map decides. Closes with a derivation that states coverage as a union over rounds while keeping disjointness per round, landed together with a per-access active-participant relation. Trigger: the log-depth tree reaching its depth limit under `implement-the-single-workgroup-synchronized-reduction-strategy`, or a double-buffered contraction tile whose slot layout rotates per round. Filed as [`admit-a-round-dependent-cooperative-staging-span`](../../../tickets/admit-a-round-dependent-cooperative-staging-span.md).
- **A per-access active-participant subset is not admitted**, and this record does not weaken the refusal `cooperative.rs:74` states. §3 records that ADR 0096 decision 3 *derives* its narrowing rather than declaring one, which is a different resolution and belongs to the ticket landing that topology. Closes when either resolution lands. Trigger: ADR 0096's item 1.
- **A strided staged span is not admitted.** §1 notes that `StagedSpan.count` addresses contiguous slots and that a strided read is the alternative to the tiled kernel's transposed write. The transpose is stateable under candidate B, so nothing needs the strided form today. Closes with an access whose staged slots are genuinely non-contiguous per participant and whose transpose cannot be moved. Trigger: that access.
- **The extents' relation to the launch geometry is stated as a product equality only.** A schedule that needs `[16, 16]` rather than `[256, 1]` to be *rejected* as a mismatch against a launch declaring a 1-D threadgroup would need the launch plan to carry a threadgroup shape, which `LaunchPlan` does not (`model.rs:788-797` carries `grid_threads`, `threads_per_workgroup`, and a zero-work flag). Closes with a launch-shape widening. Trigger: the first emission that must declare a multi-dimensional threadgroup.
- **Nothing here reaches the second tile relation.** The tiled contraction additionally inverts `commit`, the ownership divisor, and the iteration-shape rule, which [`admit-a-cooperative-tile-over-shared-operands`](../../../tickets/admit-a-cooperative-tile-over-shared-operands.md) owns. This widening is necessary for the tiled realization and nowhere near sufficient. Trigger: already fired; it is the next ticket in the chain.

## Drafted ADR body — verbatim-landable, not yet landed

**This span is a draft and is not a decision.** It is written verbatim-landable so a carrier's transfer to `docs/decisions/` can be byte-identical, following the convention [the subgroup execution tier](subgroup-execution-tier.md) and [the two-level subgroup-then-workgroup reduction](two-level-subgroup-workgroup-reduction.md) both record: a transfer that edits is a fork, and byte-identity is what makes "unreworded at acceptance" checkable rather than asserted. The carrier transfers the span below the rule with `### ` mapped to `## ` and changes nothing else, and checks it by diffing the two ranges after that normalization — after first perturbing one word and watching the check fail.

**The span is lines 245-290 of this file** — the content between the third and fourth horizontal rules, excluding the blank line on each side — beginning at `**Title:**` and ending at the last alternatives-considered bullet. The carrier takes that range and nothing else, and **re-derives the numbers rather than trusting them**, because every edit above the span has moved them at least three times already: `grep -n '^---$'` gives the four rule positions, and the span is the third plus two through the fourth minus two.

**The span carries no traceability section and therefore no relative links at all**, which avoids the tension AGENTS.md records for drafted bodies: a traceability section written with `docs/decisions/`-relative paths resolves at the ADR's destination and not from here, so this record would have to state that beside the span rather than repoint it, and repointing would break the identity. Checked rather than assumed, and the check was watched failing before it was believed: `sed -n '245,290p' <this file> | grep -c ']('` returns `0`, while the same command over lines 1-100 returns `3`, so an empty answer here is a measured zero rather than a command that did not run. Cross-references the span needs are made by ADR number and by contract name in prose, which resolve from either location. The carrier writes the traceability, normative-owner, work-record, implementation-boundary, and open-questions sections fresh at the destination.

**The number must be re-read by the carrier and is not fixed here.** `0096` was the highest ADR present at `54833c9`, so `0097` is drafted below — but three records drafting against a number have had it move underneath them, which is why the sibling records warn about exactly this. The carrier reads `docs/decisions/` again and takes the next free number; nothing in the span depends on it.

**The scope split that makes a carrier ticket necessary is the recorded one.** `ticketsplease.toml:103` routes `docs/decisions/[0-9]*.md` to `contracts/decisions` and `:101` routes `docs/decisions/README.md` to `contracts/navigation`, which also holds `docs/research/README.md` at `:89-102`; this record's ticket holds `implementation/ir` and `research/scheduling` with shared `project/tickets` only. [`land-the-two-dimensional-staging-relation-adr`](../../../tickets/land-the-two-dimensional-staging-relation-adr.md) takes all three and carries **two** catalog rows — this record's under `docs/research/README.md`, which it does not yet have, and the ADR's under `docs/decisions/README.md`.

---

**Title:** Admit a two-dimensional cooperative staging relation over a stated participant space

**Frontmatter:** `decision_status: proposed`, `implementation_status: not-started`, `catalog_group: "physical-planning-lowering"`, `applies_to: ["tiler.contract.ir", "tiler.contract.fusion-and-scheduling", "tiler.contract.artifact-abi"]`, `evidence: ["tiler.research.scheduling.two-dimensional-cooperative-staging-relation"]`, `depends_on: ["ADR-0007", "ADR-0011", "ADR-0043", "ADR-0074", "ADR-0075", "ADR-0094", "ADR-0096"]`, `ticket: "land-the-two-dimensional-staging-relation-adr"`.

### Context

The cooperative tile vocabulary states a staged access as `StagedSpan { stride, offset, count }`, addressing `count` contiguous slots at `stride * l + offset` over a one-dimensional participant coordinate `l`, and states the participant set as a contiguous `ParticipantRange`. That form makes disjointness and coverage decidable by enumerating addressed slots under two governed bounds rather than by a modular argument, which is the property the whole cooperative verifier rests on.

It cannot express the relation every blocked GPU kernel's shared-memory read has. A 16×16 operand tile's two staged reads need base profiles `16 * (l / 16)` and `16 * (l % 16)`, and over a 256-element domain an affine map is constant or injective while both profiles take sixteen distinct values with multiplicity sixteen — so no participant relabelling repairs it. That many-to-one-in-two-dimensions structure is the content of the tiling rather than an encoding of it: threads in a tile row share an operand row and threads in a tile column share an operand column.

ADR 0096 decision 4 reaches the same wall from the other side and decides that a participant's local coordinate has two components and that the schedule states their composition, rejecting both a strided participant range and a declared per-access participant subset. ADR 0094 leaves a subgroup-lane coordinate source as an enumerated boundary. Neither decides the staged-access relation over such a coordinate, and the two would otherwise widen the same vocabulary twice.

### Decision

1. **A cooperative tile's participants occupy a stated multi-dimensional space, not a contiguous run.** `LocalCoordinates` carries per-dimension extents, slowest-varying first, whose product is the participant count. The extents are a first-class fact the intrinsic verifier checks against the launched workgroup width, which is what a divisor embedded in an address expression could not be: nothing in a schedule would state that the divisor and the launch agree, so a wrong one is admitted and emits a silently wrong broadcast.

2. **The participant space replaces the tile's participant range rather than sitting beside it.** The range's `first` field is already required to be zero by the local-coordinate rule, so carrying both would be a second place to state what one determines and a place for them to disagree. A phase's reachable set, a synchronization point's participant set, and the committing participant remain contiguous *runs* over the linearized space, because each is a claim about which invocations reach a program point rather than about the shape they are arranged in; the two were one type only while the shape was a run.

3. **A staged access addresses `count` contiguous slots at `offset + Σ_d strides[d] * l_d`, one stride per participant dimension.** A stride vector is strictly more general than naming one coordinate component and subsumes it — selecting component `d` is the vector that is zero everywhere but `d` — so ADR 0096's staged write by named component and a tiled contraction's transposed operand write are one construct. A span whose stride vector does not have one entry per participant dimension is refused by its own named rule rather than padded or truncated.

4. **The relation carries no dependence on the round ordinal, and the omission is the decision rather than an oversight.** A participant dimension indexes concurrent invocations; the round ordinal indexes sequential iterations of one phase sequence. The occupancy map that decides disjointness and coverage spans the phase sequence once, which is exactly one round, and it is sound *because* a span is the same on every round. A round-dependent span makes per-round coverage a shrinking subset rather than a bijection, which is a different decision procedure and not a wider parameter for the same one — and it does not deliver a logarithmic tree by itself, because that additionally needs a per-access active-participant relation this decision does not admit.

5. **A participant coordinate source states whether its components have a defined relation to the linear local invocation index, and the two known sources answer differently.** A threadgroup position decomposes against the linear index by a relation the Metal Shading Language Specification fixes. A subgroup index and lane index do not: Metal states that threads are divided into SIMD-groups in an implementation-defined fashion and WGSL states there is no defined relationship to the local invocation index. One coordinate construct therefore serves both, and the source is where the difference is stated — a design that folded the difference into two coordinate constructs would make a schedule's portability depend on which construct a producer happened to pick.

6. **Decidability is preserved under the existing bounds, and the participant-rank bound is added for the arithmetic rather than for the enumeration.** The widened enumeration ranges over the Cartesian product of the extents, which is the same participant set the linear enumeration walks, re-indexed — so the participant bound and the staging-slot bound continue to bound it exactly. What the rank bounds is the address sum and the encoded frame, and it is not implied by the participant bound, because a space of unit extents has a product of one at any rank.

7. **Widening the relation steps the scheduled-region identity domain, and the step is executed completely or not at all.** Both encoders write unframed fixed-width runs, so a stride vector and an extent list insert into them however they are spelled, and every region's bytes move. The version moves at its owning layer, the ledger moves in the same commit, and every pinned identity is recomputed on the tree the step lands into with each moved pin enumerated in the report — including the Metal goldens with no cooperative tile at all, whose identities move through the fold for the separator bytes alone.

### Consequences

- A blocked operand tile becomes statable, which is the precondition for the tiled contraction realization and for the two-level reduction composition to have a staged access vocabulary at all. Neither becomes expressible by this decision alone.
- The schedule vocabulary gains the first construct whose validity is a relation between a declared shape and the launch geometry, rather than a property of either.
- ADR 0096's second and third enumerated public-boundary items land as one concept each rather than as two, which is what that record asks for, and the third's derived writer set is explicitly not among what lands.
- Four public types lose `Copy`, because a bounded-rank stride vector and a bounded-rank extent list are owned data. That is the cost of a rank-general form over a rank-two one, and it is paid deliberately: a fixed pair would need a second identity step to reach a three-dimensional threadgroup.
- Every cache entry, artifact, and golden minted under the prior schedule domain misses rather than matches, which is the intended consequence of a domain separator and not a regression to be repaired by preserving the old bytes.
- Nothing here admits a round-dependent span, a per-access active-participant subset, a strided staged span, a second cooperative tile relation, a multi-dimensional launch geometry, or any two-level reduction topology.

### Alternatives considered

- **The single-term offset form `stride * ((l / divisor) % modulus) + offset`, keeping the participant range one-dimensional.** Rejected on correctness. It covers the tiled contraction's two staged reads and cannot express its transposed staged write, whose profile `16 * (l % 16) + (l / 16)` is a sum of two distinct divisor projections: from base zero the offset is zero, a divisor of one forces a stride of sixteen and then requires `16 * x = 1`, and any larger divisor sends the second point to zero. The transpose cannot be moved to the read side either, because the resulting read addresses a strided slot set and the span states contiguous slots. It additionally carries no round variable and embeds a tile width no verifier rule can relate to the launch.
- **A strided participant range.** Rejected for the reason ADR 0096 already records: the staged slot address would have to be a fraction of the participant coordinate, and repairing that silently reinterprets what the span's participant index denotes.
- **A declared per-access active-participant subset.** Rejected because it is the construct a logarithmic workgroup tree is refused for needing, and admitting it here would leave two levels of one vocabulary disagreeing about whether it exists.
- **A rank-two coordinate pair with two named stride fields.** Rejected: it preserves `Copy` and nothing else, and a three-dimensional threadgroup — an ordinary Metal launch shape — would need a second identity-domain step to reach, which is the cost this decision is paying once.
- **Keeping the participant range beside the new participant space.** Rejected: the range's start is already pinned to zero by an existing rule, so the pair is one fact stated twice and a place for two producers to disagree.
- **Admitting the round ordinal as a second participant dimension.** Rejected: the two index different things — concurrent invocations against sequential iterations — and conflating them would make the coverage rule state that a shrinking per-round subset is a bijection onto the allocation, which is false for the only consumer that wants the round dependence.
- **A tagged span preserving the three-field form under tag `0x01`.** Rejected as a way to avoid the domain step, because it does not: a tag byte prepended to every existing encoding moves every region's bytes exactly as an inserted field does, so it buys nothing and costs a byte per span forever.

---

## Verbatim-landable boundary, stated exactly

The span above is the ADR body. What follows is the exact public boundary the implementation wave lands, which the ADR states as a decision and does not spell — kept out of the span deliberately, because a spelling that changes under review would otherwise break the byte-identity the transfer rests on.

**Proposal — all of it. None of this compiles today, and none is self-accepted.**

```rust
/// The governed execution key a participant's local coordinate reads.
pub enum LocalCoordinateSource {
    /// The linear index of one invocation within its own workgroup.
    LocalLinearInvocation,
    /// The per-dimension position of one invocation within its own workgroup.
    ///
    /// The relation to `LocalLinearInvocation` is defined and is the row-major
    /// decomposition against the participant extents, which is what separates
    /// this source from a subgroup-derived one: two vendor specifications
    /// decline to fix any relation between a subgroup coordinate and the linear
    /// index, so a source naming one may not claim this decomposition.
    LocalWorkgroupPosition,
}

/// The shape of one cooperative tile's participant space.
///
/// Extents are stated slowest-varying first, so a participant's linear index is
/// the row-major linearization of its coordinate. The product is the
/// participant count, and it is what the intrinsic verifier compares against the
/// launched workgroup width.
pub struct ParticipantSpace {
    /// Per-dimension extents, slowest-varying first.
    pub extents: Vec<u64>,
}

impl ParticipantSpace {
    /// Returns the number of participants, or `None` when the product overflows.
    pub fn participants(&self) -> Option<u64>;
    /// Returns the number of dimensions.
    pub fn rank(&self) -> usize;
}

/// How one cooperative tile derives each participant's local coordinate.
pub struct LocalCoordinates {
    /// Governed execution key the coordinate reads.
    pub source: LocalCoordinateSource,
    /// Shape of the space the tile's participants occupy.
    pub participants: ParticipantSpace,
}

/// The staging slots one participant addresses in one phase.
///
/// The participant at coordinate `(l_0, .., l_{r-1})` addresses the `count`
/// contiguous slots beginning at `offset + sum_d strides[d] * l_d`. One stride
/// per participant dimension, in the same axis order as the tile's extents: a
/// stride of `1` on the fastest-varying dimension and `0` elsewhere gives each
/// participant of a row its own slot, and a stride of `0` on every dimension has
/// every participant address one shared run.
pub struct StagedSpan {
    /// Slots between the first slots of participants adjacent along each
    /// dimension, in axis order.
    pub strides: Vec<u64>,
    /// First slot the participant at the origin addresses.
    pub offset: u64,
    /// Contiguous slots each participant addresses.
    pub count: u64,
}
```

**Unchanged and deliberately so.** `ParticipantRange` keeps its `{ first, count }` shape and its `end`/`contains_range` methods, and stays the type of `CooperativePhase::participation`, `SynchronizationPoint::participants`, and `CooperativeTile::commit`.

**One new error variant**, on the `#[non_exhaustive]` `CooperativeTileRule`:

```rust
    /// A staged span's stride vector does not have one entry per participant
    /// dimension.
    ///
    /// Separate from `StagingCapacity`, which says a span leaves the storage the
    /// tile declared, and from `LocalCoordinates`, which says the participant
    /// space is malformed: this one says a well-formed span and a well-formed
    /// space disagree about how many dimensions there are, and neither is wrong
    /// on its own terms.
    SpanRank,
```

with `rule()` returning `"cooperative-span-rank"`.

**One new governed bound**, in `crates/tiler-ir/src/schedule/mod.rs`:

```rust
/// Maximum participant dimensions admitted by one cooperative workgroup tile.
///
/// Deliberately not implied by `MAX_COOPERATIVE_PARTICIPANTS`: a space of unit
/// extents has a product of one at any rank, so the participant bound does not
/// bound the rank. What this bounds is the address sum a staged span evaluates
/// and the frame its encoding writes, not the enumeration — the enumeration
/// ranges over the extent product, which the participant bound already governs.
pub const MAX_COOPERATIVE_PARTICIPANT_RANK: usize = 3;
```

**Rank `3` and not more**, because a threadgroup is at most three-dimensional on every target this repository names and a fourth dimension would be a shape no launch could declare. It is a verification bound rather than a hardware claim, exactly as its siblings are.

**Widened rules, each stated so it can be watched refusing its own defect.**

| Rule | Refuses |
| --- | --- |
| `LocalCoordinates` | The participant space is empty, has a zero extent, exceeds `MAX_COOPERATIVE_PARTICIPANT_RANK`, or its extent product overflows |
| `ParticipantConvergence` | The extent product does not equal the launched workgroup width |
| `SpanRank` | A staged span's stride count differs from the participant rank |
| `StagingCapacity` | Unchanged in meaning: an addressed slot overflows `u64` or leaves the allocation |
| `StagingConflict` | Unchanged in meaning: two participants write one slot inside one round |
| `StagingCoverage` | Unchanged in meaning: an in-range slot has no writer |

**Signature changes.** `CooperativeTile::addressed_slots(participants: &ParticipantSpace, span: &StagedSpan) -> Option<Vec<u64>>`, taking both by reference because neither is `Copy` any more. `StagedSpan`, `StagedWrite`, `StagedRead`, and `LocalCoordinates` lose `Copy` and keep `Clone`, `Debug`, `Eq`, `Hash`, `PartialEq`.

**Identity encoding, at `tiler.schedule.v5`.**

- `push_participant_space` writes `push_len(extents.len())` then each extent as a big-endian `u64`. Framed through `push_len`, which is the one form the workspace uses before a variable-length run.
- `push_staged_span` writes `push_len(strides.len())`, then each stride as a big-endian `u64`, then `offset`, then `count`. The strides lead so a reader that has the participant space already knows the run length before it reads one.
- `push_participant_range` is unchanged.
- The domain separator becomes `b"tiler.schedule.v5\0"`, eighteen bytes as before.

**Why an append was not available, which the encoder's own doc must state.** Both `push_staged_span` and the coordinate encoding write unframed fixed-width runs inside records that repeat — every staged write and every staged read of every phase — so an inserted length prefix shifts every following byte and no cooperative region's identity survives. And the cooperative topology arm ends in a length-prefixed axis list that can absorb a shift, which is the same reason the `v4` step's doc records for why its append was unavailable: an old region and a new one could then encode to the same bytes, with only a verifier invariant separating them, and an identity encoder that leans on a verifier invariant has stopped being injective on its own terms.

## What this record does not decide

The second cooperative tile relation, the tile-blocked write map and its bijectivity ownership proof, the tiled contraction's schedule and Metal body, the two-level reduction topology and its `0x36` tag, the round-dependent span, and the per-access active-participant subset. Each is owned by a named ticket, and none is made easier or harder by this widening beyond the vocabulary it supplies.
