---
schema: "tiler-doc/v1"
id: "tiler.research.scheduling.multi-round-two-level-reduction-composition"
kind: "research"
title: "The multi-round two-level reduction composition"
topics: ["scheduling", "gpu", "metal", "reductions", "numerics", "execution-hierarchy", "subgroup", "identity", "public-boundary"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.fusion-and-scheduling", "tiler.contract.ir"]
depends_on: ["tiler.research.scheduling.two-level-subgroup-workgroup-reduction", "tiler.research.scheduling.two-dimensional-cooperative-staging-relation", "tiler.research.scheduling.subgroup-execution-tier", "tiler.research.scheduling.scheduled-region-model"]
ticket: "derive-the-multi-round-two-level-reduction-composition"
---

# The multi-round two-level reduction composition

**Status:** the bounded derivation [ADR 0096](../../decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md)'s open questions hand to [`derive-the-multi-round-two-level-reduction-composition`](../../../tickets/derive-the-multi-round-two-level-reduction-composition.md), run against the tree rather than against that record's citations. It designs; it implements nothing. No encoding, version string, field, or pinned value moved with it, and its decision-shaped result is drafted as [ADR 0100](../../decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) at `decision_status: proposed`, which is a non-decision until Tom accepts it.

Every repository claim is read at base commit `1d918b67`, and every claim labelled **Fact** is inspected source in this repository — cited by symbol, which is the durable reference — or a primary vendor specification reached through a named relay. Claims are labelled **Fact**, **Inference**, **Proposal**, and **Measurement**.

**Measurement boundary, stated first because it is total.** *There is no Measurement in this record.* Nothing was executed, emitted, compiled, dispatched, or timed by the work that produced it: no `cargo` invocation ran, no kernel was built, no device was touched, and no operation count below is a timing. The two vendor quotations it relies on are relayed from [the two-level subgroup-then-workgroup reduction](two-level-subgroup-workgroup-reduction.md), which read them from a locally extracted copy; this record did not open the specification itself and says so where it uses them. Compiling and testing are not measurements, so the several places below that cite an existing test cite it as evidence of a *tested guarantee* in the tree, never as a measurement of anything.

**The ticket's premise is that admitted round vocabulary is not proof of a numerical result, and that premise is correct and is what this record spends.** `CooperativeTile::rounds` decides what the verifier admits; the leaf order across rounds and the identity obligation of a ragged final round are separate facts, and the sections below derive them separately. The one thing the derivation did *not* expect is that the tree already answers half of the identity question in a way no design document records — see §4.

## Conclusion

**The multi-round composition is legal, its leaf order is the declared contributor sequence ascending, and the whole positive result rests on one convention the tree already fixed.** The implemented cooperative split states that participant `p` of round `r` owns the contiguous contributor range at index `r * partitions + p` — round-major over participants. Composed with [ADR 0096](../../decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md) decision 5's subgroup-major block index within a round, the composition's block index is **lexicographic in `(round, subgroup index, lane index)`**, and the composed leaf order of three ascending levels over one contiguous partition is ascending. It consumes **reassociation alone**, exactly as the one-round composition does. §2 walks it over a full second round, and exhibits the participant-major alternative that turns the same schedule text into a strided permutation at identical instruction count.

**A ragged final round adds no identity obligation, and a ragged *earlier* round is refused rather than derived.** Padding occupies the tail of the padded sequence, so it lies inside the final round and every earlier round is exactly covered. Per-round padding is refused on the split's own ground: `ContributorPartition` is a single product, and a per-round remainder is the second extent and second trip count `covers` already declines to approximate. §3 derives the canonicality rule that follows — padding strictly below one round's worth, or a round exists whose every contributor is an identity and two schedules with different identities state one program.

**One identity constant, three consumption sites, and the third site escapes only because round zero is peeled — which is a fact about the tree, not a proposal.** The inner level's imposed width binds always; the outer fold escapes for ADR 0096 decision 6's reason, unchanged, because all `G` slots are written on every round including the ragged one; and the cross-round accumulator escapes because `emit_loop_carried_cooperative` seeds it with round zero's own staged total rather than a constant. That peel exists in the tree with its reason stated at the site — "a sum seeded at `+0.0` is a different function — `+0.0 + x` is not `x` at `x = -0.0` — and the registered family declares no seed" — and it is what keeps the multi-round composition at ADR 0096's single `lane_identity_bits`. **Had the emission seeded a constant instead, the multi-round composition would need a second proved identity and the answer to this ticket would have been the opposite one.** §4.

**The multi-round composition declares two synchronization points where the one-round composition declares one, and the second is exactly the barrier the vendor idiom is missing.** A two-phase tile with repeating phases derives one `AntiDependencyEdge`, and arithmetic over `SynchronizationPoint::discharges_anti` shows the produce/consume phase boundary does not discharge it: a `RoundBoundary` point must be declared, and both points must name `ConvergenceEvidence::EveryParticipantExecutesEveryRound`. [The two-level record](two-level-subgroup-workgroup-reduction.md) §6 derived that the Metal specification's printed multi-round loop has a write-after-read hazard with nothing ordering it; the composition does not inherit that hazard, it *refuses* it by name, and the construct that refuses it is already landed and tested. §5.

**The sharp negative result is at the identity-less family, and it is new because that family reached the parallel topologies after ADR 0096 was accepted.** `cooperative_family` now admits `ScalarProgram::StrictSerialMaximum` with `EmptyDomainContract::NoIdentity`, and its non-emptiness argument is stated for *exact* coverage: "the split contract makes every partition's contributor count a nonzero factor of a nonzero product, so each staged partial is a real maximum". The composition pads, so that argument does not reach it, and a contributor-free lane under the extrema family has no value the vocabulary currently gives it. **`0xff80_0000` is a provably two-sided padding identity for that family and the composition needs it stated**; the alternatives — a carried `has_value`, contributor replication under idempotence, and refusing every non-divisible contributor count — are eliminated in §6. This is also where the derivation contradicts a landed doc comment, which is filed rather than edited, because `crates/` is not this ticket's scope.

**ADR 0096 decision 1 is too strong in one clause, and the padded case is the general case.** It states that the composition's partition satisfies "the equalities the cooperative verifier already checks". The equality `partition.partitions == participants` holds; `covers(contributors)` does not, because `contributors` is counted from the region's access map and the composition's partition covers the *padded* sequence. That record's own Worked example C — `C = 101`, `partitions = 128` — would be refused as `CooperativeTileRule::ContributorSplit` if handed to `verify_cooperative_semantics` today. §7 states the correction and what it costs: the composition's coverage rule is a rule of its own, and the padding count is derived rather than declared.

**Nothing here fires the round-dependent staged span.** The composition's spans are round-invariant for the same reason the tiled contraction's are, so [`admit-a-round-dependent-cooperative-staging-span`](../../../tickets/admit-a-round-dependent-cooperative-staging-span.md) stays `deferred` with its triggers unfired, and §8 keeps storage coverage and numerical grouping apart rather than reading one as evidence for the other.

## 1. What moved under ADR 0096 between its base and this one

**Fact.** ADR 0096 is `accepted` with `implementation_status: not-started`, and its status paragraph already carries the stale-ground correction: decision 8's one-round exclusion is "an initial-profile scope choice rather than a vocabulary impossibility", because `admit-loop-carried-cooperative-staging` landed the per-round lifetime mid-flight. Its open questions name this ticket and instruct it not to treat round vocabulary as proof of the numerical result.

**Fact — three landings sit between that record's reading of the tree and this one, and each changes what a multi-round derivation may assume.**

- `CooperativeTile::rounds` (`crates/tiler-ir/src/schedule/cooperative.rs`) states how many times the whole phase sequence executes, with `AntiDependencyEdge` as the second derived evidence class, `SynchronizationPlacement::RoundBoundary` as the placement that discharges every one of them, and `ConvergenceEvidence::EveryParticipantExecutesEveryRound` as the evidence class a point inside the round loop requires. `MAX_COOPERATIVE_ROUNDS` is `65_536` and a round count of zero or above it is `CooperativeTileRule::RoundStructure`.
- The two-dimensional staging relation is `implemented` under [ADR 0097](../../decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md): `ParticipantSpace`, a per-dimension `StagedSpan` stride vector behind a constructor, `LocalCoordinateSource::LocalWorkgroupPosition` at tag `0x02`, `CooperativeTileRule::SpanRank`, and the `tiler.schedule.v4` → `v5` step. Its decision 4 keeps the round ordinal *out* of the staging relation deliberately.
- The extrema fold reached both parallel topologies. `SplitFamily` (`crates/tiler-ir/src/schedule/builder.rs`) now carries `empty_domain: EmptyDomainContract` and `consumes_reassociation: bool`, and both `multi_pass_family` and `cooperative_family` admit `ScalarProgram::StrictSerialMaximum` with `NoIdentity` and `consumes_reassociation: false`.

**Inference — decision 8's *conclusion* survives all three and its *ground* has been dead since before acceptance.** The conclusion is that ADR 0096's derivation covers one staging round. That is a statement about what was derived and nothing can falsify it. The ground — "a per-round lifetime and a per-round visibility edge the cooperative profile does not model" — was already wrong at that record's own base, which its implementation boundary says in as many words. This record therefore re-derives rather than re-reads: everything below is run against the tree at `1d918b67`.

**Locator note — 2026-08-19 by [`point-the-bare-builder-path-mentions-at-the-split-modules`](../../../tickets/point-the-bare-builder-path-mentions-at-the-split-modules.md): where the verifier this record reads at `1d918b67` lives now.** `crates/tiler-ir/src/schedule/builder.rs` became the `crates/tiler-ir/src/schedule/builder/` directory, so both citations of it above resolve against no file. `SplitFamily` is `crates/tiler-ir/src/schedule/builder/family.rs "pub(super) struct SplitFamily<'a> {"` and still carries both fields; `StrictSerialMaximum` still derives `EmptyDomainContract::NoIdentity` and `consumes_reassociation: false` there. `verify_cooperative_semantics` is `crates/tiler-ir/src/schedule/builder/reduction.rs "pub(super) fn verify_cooperative_semantics("`. **Two spellings the record quotes no longer occur, and a reader grepping for them will read absence as removal.** `multi_pass_family` and `cooperative_family` were two match tables; `eb0b7514` consolidated them into the single `crates/tiler-ir/src/schedule/builder/family.rs "pub(super) fn split_family("`, which the serial, multi-pass, and cooperative admissions all read — the admission the bullet above records is unchanged, and is now made in one place rather than two. The coverage arithmetic likewise moved: `verify_cooperative_semantics` passes `tile.rounds` to `crates/tiler-ir/src/schedule/builder/coverage.rs "pub(super) fn verify_contributor_coverage("` instead of multiplying inline, `ContributorSplit` now owns only the participant and shape agreement, and the quoted comment has been rewritten around a padded coverage case this record predates. The round-major convention §2 derives from is unchanged. Nothing in this record's derivation is re-run against the current tree.

**Inference — and one of the three landings is load-bearing in a direction nobody wrote down.** The rounds vocabulary did not land as a bare capability. It landed with a *lowering*, `emit_loop_carried_cooperative`, whose shape fixes the cross-round accumulator's initial value. §4 is where that matters, and it is the reason this derivation reaches a positive result rather than an enumerated missing obligation.

## 2. The block index across rounds, and the leaf order it decides

### The convention the tree already fixed

**Fact — `ReductionTopology::CooperativeWorkgroup::partition` (`crates/tiler-ir/src/schedule/model.rs`) states the multi-round block assignment in its own doc.** "`contributors_per_partition` is what one participant folds on *one* round, so the sequence this split covers is `partitions * contributors_per_partition * tile.rounds`. On a single-round tile that is the plain product and the field means exactly what it does for [`Self::MultiPass`]; on a loop-carried one, participant `p` of round `r` owns the contiguous range at index `r * partitions + p`, which is why the coverage stays ascending and the strategy still consumes reassociation alone."

**Fact — `verify_cooperative_semantics` (`crates/tiler-ir/src/schedule/builder.rs`) enforces the arithmetic and repeats the reason.** It computes `covered = partition.total_contributors()? * tile.rounds` and refuses `covered != Some(contributors) || partition.partitions != participants` as `CooperativeTileRule::ContributorSplit`, with the comment "Participant `p` on round `r` folds the contiguous range at index `r * participants + p`, so `partitions * contributors_per_partition * rounds` is the whole sequence and the coverage stays contiguous and ascending". `covers` is deliberately left round-unaware, so the round factor is applied at the tile's admission and nowhere else.

**Inference — the block index is round-major, and that is a *choice* the tree made rather than an arithmetic necessity.** Nothing in `partitions * contributors_per_partition * rounds` distinguishes `r * P + p` from `p * R + r`; both partition the padded sequence into `P · R` contiguous blocks of `k`. The doc comment is the only place either is stated, which makes it exactly the kind of claim [AGENTS.md](../../../AGENTS.md) calls load-bearing: the next worker reads it as fact, and a lowering that assumed the other convention would be admitted by every check the verifier performs.

### The composition, worked over a full second round

Take the composition's own parameters: `W = 32` lanes per subgroup, `G = 2` subgroups, `T = G·W = 64` participants, `k = 1` contributor per participant per round, `R = 3` rounds. The padded sequence is `T·k·R = 192`.

**Level one, inside subgroup `g` on round `r`.** ADR 0094 decision 4 admits the ascending-mask butterfly, whose leaves read left to right are the lanes of that subgroup in ascending lane order. The block subgroup `g` owns on round `r` begins at padded position `64r + 32g`, so

```text
P(r, g) = (((c[64r+32g] + c[64r+32g+1]) + (c[64r+32g+2] + c[64r+32g+3])) + …)
```

is a balanced binary tree whose leaf order is `64r+32g, …, 64r+32g+31` — ascending, and nothing crosses a block boundary.

**Level two, across subgroups on round `r`.** `P(r, g)` is staged into slot `g`, and the staged partials are folded in ascending slot order under `ContributorArrival::AscendingParticipant`:

```text
Q(r) = P(r, 0) + P(r, 1)
```

**Level three, across rounds.** The rounds are sequential iterations of one phase sequence, so their totals combine in ascending round order and no field states otherwise — the ordering is program order rather than a declared arrival:

```text
y = (Q(0) + Q(1)) + Q(2)
```

**The composition.** Substituting downwards and reading the leaves left to right gives `c[0], c[1], …, c[191]` — the padded contributor sequence ascending, therefore the declared contributor sequence ascending with the identity leaves after every real one (§3 places them). **Reassociation alone; no permutation.** Round 1 is a full second round and its leaves `c[64] … c[127]` sit contiguously between round 0's and round 2's, which is the ticket's first work item discharged: the composition of three ascending levels over one contiguous partition is ascending, and the second round introduces no new crossing.

### Where it breaks, and the break is new to the multi-round form

**Inference — the participant-major alternative is a strided permutation at identical instruction count.** Suppose the schedule assigned participant `p` the contiguous run at index `p·R + r` on round `r` — a reading of "participant `p` folds `k·R` contributors" that is just as natural in prose and that no arithmetic check distinguishes. Round 0 then folds blocks `0, R, 2R, …`, and with `R = 3` the composed leaf order is `c[0..1], c[3..4], c[6..7], …` before any of `c[1]`, `c[2]`, `c[4]`. That is the strided layout [ADR 0093](../../decisions/0093-bind-vector-lanes-to-the-map-or-the-contributor-partition.md) decision 3 identifies as consuming permutation, arrived at without anyone choosing it — **the same schedule text, the same widths, the same instruction count, and a verdict under a permutation-forbidding contract that flips on a convention the topology states only in a doc comment.**

**Inference — this is the round-level sibling of ADR 0096 §4's counterexample, and the two differ in an important way.** There, the hazard is a fact *no schedule can hold*: two vendor specifications decline to fix the relation between a subgroup coordinate and the linear local index, so a schedule that partitions by one and combines by the other consumes an implementation-defined permutation. Here the hazard is a fact the schedule *can* hold and currently holds only in prose. The round ordinal is a program loop counter, uniform across every participant by construction — `CooperativeTile::rounds` is a declared literal precisely so that every participant runs the identical trip count — so nothing about the machine can perturb it. **The multi-round composition therefore adds a stated-convention obligation and no new target-dependence**, which is the cleanest way to say what round nesting costs.

**So the composition's block index is the triple `(r, g, l)` read lexicographically, `index = r·T + g·W + l`, and it is one statement rather than two.** ADR 0096 decision 5 fixes the inner two components and refuses the linear local invocation index; the implemented split fixes the outermost. A schedule that stated only the inner pair would leave the round nesting to a lowering convention, which is the same defect decision 5 exists to prevent one level down.

## 3. The ragged final round, and where padding is allowed to be

**Fact.** `contributor_count(axes, &read.map)` counts the region's *real* contributors from the logical access, and `verify_cooperative_semantics` requires `partition.total_contributors() * tile.rounds` to equal it exactly. The implemented cooperative topology therefore admits no padding at all, and `workgroup_tree_tile`'s doc states the same contract in prose: "the split must cover the contributor sequence exactly once each, so a contributor count with no exact split of `participants` partitions is *declined by the strategy that chooses the count*, never padded with identity elements or truncated."

**Fact.** The composition cannot inherit that. ADR 0096 decision 6 records why: the subgroup width is imposed by the device rather than chosen by the schedule, so a contributor-free lane is the general case, and the composition's admitted answer is a stated two-sided `lane_identity_bits`. Its Worked example C runs at `C = 101` with `T = 128` and calls the padded sequence the one the partition covers.

**Inference — so the composition's contributor sequence is the padded one, `T·k·R ≥ C`, and the only free question is where the `T·k·R − C` identity positions sit.** Two conventions cover the space.

*Per-round padding — each round covers its own share of the real sequence and pads its own tail.* **Eliminated on the split's own ground.** `ContributorPartition` is a single product, and its doc states the elimination in advance: "Requiring the product to be exact is what makes 'every contributor exactly once' checkable — a ragged final partition would need a second extent and a second trip count, and `ContributorPartition::covers` rejects one rather than approximating it." Per-round padding is exactly a second trip count, and admitting it would give one topology two coverage rules keyed on nothing.

*Tail padding — the padded sequence is the real sequence followed by `T·k·R − C` identity positions.* **Survives, and it is what the lexicographic block index already implies:** positions are assigned in ascending `(r, g, l)`, so the highest positions are the last round's highest subgroups and lanes.

**Inference — tail padding places every identity leaf inside the final round, and every earlier round is exactly covered.** That is the direct answer to the ticket's "one full second round and one ragged final round": in a composition with `R ≥ 2`, rounds `0 .. R−1` are full by construction and only round `R−1` can be ragged. Nothing about the second round is special, which is a result rather than an assumption — the second round is where a per-round padding convention would first have shown a difference, and tail padding is what removes it.

**Inference — one canonicality rule follows and it must be checked rather than assumed.** If `T·k·(R−1) ≥ C`, the final round's every position is an identity: it folds nothing, stages `G` identities, discharges an anti-dependency, and realizes two barriers for a program that computes nothing. It is also not unique — `R` and `R−1` would state the same semantics with different identities, which is precisely what a normalized schedule (ADR 0007) must not admit. **The rule is `T·k·(R−1) < C ≤ T·k·R`**, equivalently *padding strictly below one round's worth*, and it is decidable from the same three numbers the coverage check already reads. It is the multi-round sibling of `RoundStructure`'s refusal of a zero round count, and it needs its own rule name for the same reason: a tile whose rounds are unproductive fails for a reason no coverage arithmetic would name.

**Inference — the padding count is derived and must not be declared.** `padding = T·k·R − C` is a total function of the partition, the round count, and the access map, all of which the verifier already holds. A declared field would be a second place to state it and a place for a producer to be wrong, which is the argument ADR 0096 decision 3 uses for the staging writer set and ADR 0097 decision 2 uses for the participant range. There is no fork here and therefore no question for Tom.

## 4. The identity obligations, enumerated by site

The ticket asks whether each round has its own identity contributor. It does not, and the derivation has to say *why* at three separate sites, because they escape for three different reasons.

**Site one — the inner butterfly, on the ragged round only. The obligation binds, and it is ADR 0094 decision 5's obligation unchanged.** A lane that owns a padding position enters the butterfly holding `lane_identity_bits`, which must be a two-sided identity of the accumulation — `0x8000_0000` for `f32` addition under a contract forbidding signed-zero elimination, and never the region's `empty_identity_bits`, which `empty_domain_is_satisfied` requires to be `+0.0`. Two-sidedness gives `ι ⊕ ι = ι` by substitution, so a subgroup all of whose lanes are padding stages the identity rather than something weaker, and a subgroup with a mix stages the maximum or sum of its real contributors exactly. **Only round `R−1` reaches this site**, by §3.

**Site two — the outer fold over `G` slots, on every round including the ragged one. It escapes, for ADR 0096 decision 6's reason, and the ragged round is what tests the reason.** The staged writers are derived — the participants whose lane component equals the result lane — so on every round exactly `G` participants write exactly `G` slots and the bijection the coverage check decides is exact whatever the contributors were. A slot whose subgroup was entirely padding holds the identity *produced by site one*, not an identity injected here. **No outer padding exists, so no second identity constant is required**, and the ragged round does not create one.

**Site three — the cross-round accumulator. It escapes, and the reason is a fact about the tree that no design document states.**

**Fact — `emit_loop_carried_cooperative` (`crates/tiler-ir/src/kernel/lower.rs`) peels round zero, and its doc states the numerical reason at the site.** The emitted shape is

```text
if (%act) { fold round 0's range ; staged_store[%lid] }
barrier(phase point)
%seed = fold staged[0..participants]
%total = loop r in 1..rounds carrying %seed {
    barrier(round point)
    if (%act) { fold round r's range ; staged_store[%lid] }
    barrier(phase point)
    %t = fold staged[0..participants]
    yield canonicalize(%acc + %t)
}
```

and the doc reads: "**Round zero is peeled because the fold seeds at its first contributor.** A sum seeded at `+0.0` is a different function — `+0.0 + x` is not `x` at `x = -0.0` — and the registered family declares no seed, so the accumulator's initial value has to be round zero's own staged total."

**Fact — it is a tested guarantee rather than an implemented one alone.** `a_loop_carried_tile_lowers_to_a_peeled_round_body` (`crates/tiler-ir/src/kernel/tests.rs`) asserts the loop runs `1..rounds`, carries exactly one `f32` accumulator, and puts the round boundary at the head of the body; `a_loop_carried_extrema_tile_carries_its_maximum_across_rounds` asserts the same shape over the identity-less family and that the round accumulator combines with `F32Maximum` rather than `F32Add`.

**Inference — so the multi-round composition inherits an accumulator with no seed constant, and that is the whole reason it stays at one identity obligation.** Had the emission written `acc = empty_identity_bits` and folded `0..rounds`, the composition would inject `+0.0` ahead of every round total, and a row whose true sum is `-0.0` would commit `+0.0` — the one-bit error ADR 0096's identity trap describes, moved from the lane to the accumulator, on a value the strict contract says is observable. **The peel is therefore load-bearing for the composition's numerics and not an emission detail**, and a future emission that generalized the peel away would silently reintroduce a second identity obligation that no schedule field states. That is a claim worth a test the tree does not have, and the deferrals below file it.

**Inference — one constant, one proof, three consumption sites, and the count does not grow with `R`.** `lane_identity_bits` is consumed at site one; sites two and three consume nothing. The same two-sidedness proof discharges every use, because the outer fold and the round accumulator are the *same binary operation* as the inner tree's combine — the fold's own `ReductionCombiner` is resolved once per region. There is no per-round identity, no per-round field, and no ragged-round special case beyond the one the lane identity already answers.

**And the deferred outer shuffle tree stays deferred, with its trigger unchanged.** ADR 0096 defers a second width-`W` shuffle tree at the outer level because it would read `W` lanes while only `G < W` hold partials and would need a second stated identity. Multi-round changes nothing about that: the second identity would be needed on every round rather than on the ragged one, which strengthens the deferral rather than firing it.

## 5. The synchronization consequence, derived by arithmetic

**Fact — a two-phase tile whose phases repeat derives exactly one anti-dependency.** `CooperativeTile::anti_dependency_edges` returns nothing when `rounds <= 1`, and otherwise one edge per (allocation, reading phase, rewriting phase) triple over every pair of phases regardless of order. The composition's tile has one allocation, a producing phase 0, and a consuming phase 1, so the edge is `{ staging 0, consumed_in: 1, rewritten_in: 0 }`.

**Fact — the produce/consume phase boundary does not discharge it.** `SynchronizationPoint::discharges_anti` admits a phase boundary when `preceding >= consumed_in || following <= rewritten_in`. For `PhaseBoundary { preceding: 0, following: 1 }` against that edge: `0 >= 1` is false and `1 <= 0` is false, so the disjunction is false. `SynchronizationPlacement::RoundBoundary` returns `true` unconditionally, and `discharges` — the visibility half — returns `false` for it.

**Inference — the multi-round composition declares two points and a single-point multi-round composition is refused by name.** The phase boundary discharges the visibility edge and no anti-dependency; the round boundary discharges the anti-dependency and no visibility edge. Each obligation has exactly one discharging point, so `RedundantPoint` is not tripped by either, and dropping the round boundary is `UndischargedAntiDependency` rather than a silent race. `ConvergenceEvidence::required_for_rounds(R)` returns `EveryParticipantExecutesEveryRound` for `R > 1`, so *both* points must name it; a point naming the single-round derivation on a repeating tile is `SynchronizationRule::ConvergenceEvidence`.

**Inference — this is the vendor idiom's missing barrier, and the composition refuses the hazard rather than inheriting it.** [The two-level record](two-level-subgroup-workgroup-reduction.md) §6 derives that the Metal specification's printed multi-round reduction writes `ldata[simd_group_id]`, barriers, reads `ldata[lid]`, and then re-enters the loop with nothing separating the read from the next round's write, so a fast subgroup can overwrite a value a slower one has not consumed. **The construct that names the defect is `AntiDependencyEdge` and the construct that repairs it is a round-boundary point, and both are landed, verified, and lowered.** What ADR 0096 decision 8 read as a reason to refuse a second round is, at this base, the reason a second round is *safer* to state here than to write by hand.

**Inference — and the barrier count is a derived count, not a cost claim.** The lowering realizes the phase boundary once ahead of the loop and `R − 1` times inside it, which is `R` dynamic realizations, and the round boundary `R − 1` times: `2R − 1` barriers for `R` rounds against `1` for the one-round composition at `R` times the contributor count. No cost model in this repository prices a barrier against an addition, so this record does not say which is faster, and the first deferral below records what evidence would.

## 6. The identity-less family, which the one-round derivation never faced

**Fact.** `cooperative_family` admits `ScalarProgram::StrictSerialMaximum` with `empty_domain: EmptyDomainContract::NoIdentity`, `consumes_reassociation: false`, and `read_tensor: FIRST_INPUT`, and `empty_domain_is_satisfied` discharges `NoIdentity` as `contributors != 0`. The derivation the tree records for that discharge is stated for exact coverage: "**Non-emptiness of the whole sequence is non-emptiness of every partition** … a product of nonzero factors equalling a nonzero total forces every factor nonzero, so each partition folds at least one contributor and each staged partial is a real maximum. A carried `has_value` would be a runtime flag that is constantly true."

**Inference — the composition breaks that argument's premise, and only that premise.** Under padding the product is over the *padded* sequence, so a partition folding only padding positions is exactly what the argument's nonzero-factor step excludes. The flag would no longer be constantly true, which means the elimination of `has_value` does not transfer either: it was eliminated for being vacuous, and under padding it is not vacuous.

**Fact — the accumulation is the NaN-propagating extrema family with a total order on signed zeros.** `BinaryOp::F32Maximum` (`crates/tiler-ir/src/kernel/model.rs`) is "The IEEE 754-2019 `maximum` of two binary32 values … **The NaN-propagating extrema family, with `-0.0` ordered below `+0.0`**", and [ADR 0023](../../decisions/0023-floating-point-extrema-semantics.md) makes `Minimum`/`Maximum` the propagating family and fixes the `-0.0 < +0.0` order.

**Inference — `0xff80_0000` is a two-sided identity of that operation on every binary32 input, checked case by case rather than asserted.** For `x` finite or `±∞`, `maximum(-inf, x) = x` because `-inf` is the minimum of the order and `maximum(-inf, -inf) = -inf`. For `x = ±0.0`, the same, because the family orders `-0.0` below `+0.0` and both above `-inf`. For `x` a NaN, the family propagates, so the result is a NaN — and the fold canonicalizes after every combine under the region's `canonical_nan_bits`, so the result is the *same* canonical NaN the unpadded fold would have committed. **The identity is therefore observable-bit neutral at the region's boundary, which is the property [ADR 0022](../../decisions/0022-reduction-identities-and-initial-values.md) requires a physical schedule to prove before injecting a padding value.**

**Fact — a landed doc comment denies that such a value exists, and its conclusion and its stated ground are not the same claim.** `ScalarProgram::StrictSerialMaximum` (`crates/tiler-ir/src/schedule/model.rs`) reads: "**There is deliberately no empty-domain identity, and the omission is the contract rather than an oversight.** … the extrema families have no identity: no binary32 value `i` satisfies `Maximum(i, x) == x` for every `x`, because any candidate is itself a possible contributor."

**Inference — the conclusion stands and the ground is wrong, and the distinction is exactly the one the numerics contract already draws.** [Numerical semantics](../../numerical-semantics.md) states that "Empty result, algebraic identity, and safe physical padding are separate facts" and that a schedule "may inject or replicate a padding value only when the operation contract proves it observably neutral", and ADR 0022 decides the same separation. Declining an *empty-domain identity* for the extrema family is a semantic decision that ADR 0023 and the numerics contract support: an empty maximum has no result the operation is willing to name, and `-inf` committed for an empty row is indistinguishable from a real row of `-inf`. But "no value is a two-sided algebraic identity" is a *different* claim, and it is false for `-inf` under this family — the case walk above is the refutation. A reader who carries the comment forward concludes that the extrema family can never be padded, which would make the composition inapplicable to the row maximum it exists for.

**Elimination — four candidates for a contributor-free lane under the identity-less family, and one survives.**

*Inject the proved padding identity `0xff80_0000`.* **Survives.** It costs one stated value and one proof, and the proof is the same shape ADR 0094 decision 5 already requires of `lane_identity_bits` for the sum. It requires the schedule to state the value rather than derive it from the scalar program, because the scalar program carries no field it could be read from — which is a public-boundary consequence, enumerated in §9 rather than decided here.

*Carry a `has_value` flag beside every partial.* **Eliminated.** The inner level is a register-transfer tree (ADR 0094), so a flag would need a second shuffle tree over booleans and a select at every step, doubling the inner tree's transfers, and the staged slot would have to widen or double. It also reintroduces a per-participant runtime fact into a vocabulary whose whole staging argument is decided statically by enumeration.

*Replicate a real contributor into the padding positions, relying on `maximum`'s idempotence.* **Eliminated, and this is the cheap option whose saved cost is the correctness.** It is sound for `maximum` and unsound for every sum, so it can never be the vocabulary's rule; it makes coverage a multiset rather than a bijection, which is the property the occupancy enumeration decides; and the freedom it spends is *duplication*, a dimension ADR 0011's permission structure does not have and [ADR 0095](../../decisions/0095-decline-a-distributivity-permission.md) declined to add a sibling of.

*Refuse the composition whenever `T·k·R != C` for the identity-less family.* **Eliminated on reach.** The workload the row maximum exists for is the attention softmax, whose prefill row length [the two-level record](two-level-subgroup-workgroup-reduction.md) relays as `S = 8,192` from [the first attention program vertical](../program-planning/first-attention-program-vertical.md)'s B1-d row; a contributor count with no exact factorization into a multiple of the imposed width — `8,191` is prime, and primality is not exotic among sequence lengths — would force `T = 1` and there is no composition at one participant. The refusal is correct-but-unreachable-for-the-motivating-case, which makes it a refusal of the strategy rather than of the input.

**Proposal — the composition states one `lane_identity_bits` per region whose value is proved two-sided for the region's own combiner, and the identity-less family's empty-domain refusal is untouched by it.** A composition over `StrictSerialMaximum` with no stated padding identity is refused by name rather than replicated, flagged, or padded with the empty result; a composition over a sum keeps `0x8000_0000`. Both refusals are about padding and neither weakens `EmptyDomainContract::NoIdentity`, which continues to require a non-empty *real* domain.

## 7. What ADR 0096 decision 1 says too strongly, and the coverage rule that replaces it

**Fact.** ADR 0096 decision 1 reads: "A threadgroup of `T = G · W` invocations each folding `k` contributors is the `ContributorPartition` the vocabulary already has, satisfying the equalities the cooperative verifier already checks."

**Fact.** `verify_cooperative_semantics` checks two equalities against a partition: `partition.partitions == participants`, and `partition.total_contributors()? * tile.rounds == contributors`, where `contributors` comes from `contributor_count(axes, &read.map)` — the region's real reduced domain.

**Inference — the first equality holds for the composition and the second does not.** ADR 0096's Worked example C states `C = 101` real contributors and `ContributorPartition { partitions: 128, contributors_per_partition: 1 }`, and says it "covers 128 exactly". Handed to `verify_cooperative_semantics` at this base, `covered = Some(128)` and `contributors = 101`, so it is refused as `CooperativeTileRule::ContributorSplit`. **The composition's partition covers the padded sequence and the implemented rule covers the real one, and those are different objects.** This is not a defect in either: the implemented topology chooses a participant count that divides, and the composition cannot, because its inner width is imposed. It *is* a sentence in an accepted record that a reader will act on wrongly, and the correction belongs here rather than in an edit to the accepted decision.

**Inference — so the composition's coverage rule is its own, and it has three parts rather than one.** `partition.partitions == T` and `T == threads_per_workgroup` as today; `T·k·R ≥ C` with the excess being padding; and `T·k·(R−1) < C` so that the padding is below one round. The first is the equality decision 1 correctly claims; the second replaces the exact product; the third is new with rounds. All three are decidable from values the verifier already holds, so the rule adds a check and no field.

## 8. Against the round-dependent staged span, keeping storage coverage and numerical grouping apart

**Fact.** [ADR 0097](../../decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md) decision 4 keeps the round ordinal out of the staging relation deliberately: "A participant dimension indexes concurrent invocations; the round ordinal indexes sequential iterations of one phase sequence. The occupancy map that decides disjointness and coverage spans the phase sequence once, which is exactly one round, and it is sound *because* a span is the same on every round." [`admit-a-round-dependent-cooperative-staging-span`](../../../tickets/admit-a-round-dependent-cooperative-staging-span.md) is `deferred` and its stated triggers are the logarithmic tree reaching its depth limit, or a double-buffered contraction tile whose slot layout rotates per round.

**Inference — the multi-round composition fires neither trigger, and the reason is structural rather than incidental.** Its produce span writes one slot per staging participant and its consume span reads the whole staged set, on every round, with the same stride vector, the same offset, and the same count. What varies with the round is which contributors are folded into the staged value — a fact about the *contributor* addressing, which the region's access map and the block index own — and never which slot receives it. That is the same shape the tiled contraction has, where "what varies with the round is the device address the value is loaded from, never the staged slot".

**Inference — and the two facts must not be read as one, which is the ticket's explicit warning.** Storage coverage is the bijection `verify_cooperative_tile` decides by enumeration over one round; numerical grouping is the leaf order §2 derives over all `R` rounds. The round vocabulary settles the first and says nothing about the second: a tile could have a perfect per-round bijection under a participant-major block index and still consume a permutation. **`rounds` being admitted is evidence about the storage and no evidence at all about the numerics**, which is why this record derives §2 from the split's own doc rather than from the tile's admission.

## Worked example — a ragged final round at exact bits

The program is the one [the subgroup execution tier](subgroup-execution-tier.md), [the CPU vector-lane tier](cpu-vector-lane-tier.md), and [the two-level record](two-level-subgroup-workgroup-reduction.md) all use, with the reduced extent chosen so that the final round is ragged:

```text
x : f32[7, 150]
y : f32[7]
y[m] = strict_serial_sum(n, x[m, n])      0 <= m < 7, 0 <= n < 150
```

**The region**, in the implemented vocabulary: one `ReductionContributor` read with `input_shape = [7,150]`, `output_shape = [7]`, `axes = [1]`, `order = OriginalAxisLexicographic`; `ScalarProgram::StrictSerialSum { axes: [1], order: OriginalAxisLexicographic, canonical_nan_bits: 0x7fc0_0000, empty_identity_bits: 0x0000_0000 }`. **The contract** is `reassociation: Permitted`, `permutation: Forbidden`, `signed_zero: Forbidden`.

**The schedule.** One threadgroup owns one output row. `W = 32`, `G = 2`, `T = 64`, `k = 1`, `R = 3`. The padded sequence is `T·k·R = 192` against `C = 150`, so there are **42 padding positions**, and `T·k·(R−1) = 128 < 150` holds, so the canonicality rule of §3 admits `R = 3` and refuses `R = 4`.

| Round | Padded positions | Subgroup 0 (lanes 0–31) | Subgroup 1 (lanes 0–31) | Real | Padding |
| --- | --- | --- | --- | --- | --- |
| 0 | 0 – 63 | `c0 … c31` | `c32 … c63` | 64 | 0 |
| 1 | 64 – 127 | `c64 … c95` | `c96 … c127` | 64 | 0 |
| 2 | 128 – 191 | `c128 … c149` real, `c150 … c159` identity | `c160 … c191` identity | 22 | 42 |

**Round 1 is the full second round the ticket asks the derivation to walk, and round 2 is ragged in both available ways at once** — a partially padded subgroup and an entirely padded one. Subgroup 1 of round 2 owns no real contributor at all, folds thirty-two copies of the identity, and stages the identity into slot 1; that is the case ADR 0096's example never reached, and it is discharged by two-sidedness rather than by a new rule.

**The tile.** One `WorkgroupStaging` of `G = 2` slots of `StagedElement::F32` — **8 bytes** — live from the producing phase through the consuming one, `rounds = 3`, two phases each with `participation` equal to the whole 64-invocation range, `commit = { first: 0, count: 1 }`. Two synchronization points: a `PhaseBoundary { preceding: 0, following: 1 }` discharging the one visibility edge, and a `RoundBoundary` discharging the one anti-dependency, both naming `EveryParticipantExecutesEveryRound`.

| Obligation | Discharge |
| --- | --- |
| Reassociation | **Consumed.** Three ascending levels over one contiguous partition; admitted because the contract grants it. |
| Permutation | **Not consumed**, because the inner masks ascend, the outer arrival is `AscendingParticipant`, and the block index is `r·64 + 32g + l`. Change only the round nesting to participant-major and the same schedule consumes a strided permutation and is **rejected**, at identical instruction count. |
| Coverage | `64 · 1 · 3 = 192 = C + 42`, with the 42 identities in round 2 alone; staging is 2 writers onto 2 slots on every round, a bijection each time. |
| Round canonicality | `128 < 150 <= 192`, so no round is entirely padding. |
| Inner identity | **The binding obligation, and only on round 2.** `lane_identity_bits = 0x8000_0000`. |
| Outer identity | **None required.** Two slots, two writers, two reads, on every round. |
| Cross-round seed | **None required.** Round 0's staged total seeds the accumulator; the loop runs `1..3`. |
| Visibility | One edge, discharged by the phase boundary. |
| Anti-dependency | One edge, discharged by the round boundary; without it, `UndischargedAntiDependency`. |
| Convergence | `rounds > 1`, so both points require `EveryParticipantExecutesEveryRound`, which `rounds` being a declared literal is what proves. |

**The identity, at exact bits, on the row that discriminates.** Take the row whose 150 contributors are all `-0.0`, whose true strict-fold sum is `-0.0` = `0x8000_0000`.

With `lane_identity_bits = 0x8000_0000`: rounds 0 and 1 fold real `-0.0`s only and give `Q(0) = Q(1) = -0.0`; in round 2, subgroup 0 mixes twenty-two real `-0.0`s with ten identity `-0.0`s and gives `-0.0`, subgroup 1 folds thirty-two identities and gives `-0.0`, so `Q(2) = -0.0`; the accumulator seeds at `Q(0) = -0.0` and folds `(-0.0) + (-0.0) + (-0.0) = -0.0`. The committed result is `0x8000_0000` — **correct**.

With `0x0000_0000`, which is what `empty_identity_bits` holds and what an implementer reaches for: round 2's subgroup 0 mixes `(-0.0) + (+0.0) = +0.0` at its first crossing step and subgroup 1 folds thirty-two `+0.0`s, so `Q(2) = +0.0` and the total is `0x0000_0000` — **wrong by one bit, on a value the contract says is observable**. The error reaches only the last round, which makes it *harder* to hit in testing than the one-round composition's, where it reached the only round there was.

**And the counterfactual that shows site three is not free.** Had the emission seeded the accumulator with `empty_identity_bits` and folded `0..R`, the same row would commit `+0.0` even with `lane_identity_bits = 0x8000_0000` correct at every lane, because the first combine would be `(+0.0) + (-0.0)`. The peel is what makes this row come out right, and it is a property of the lowering rather than of any schedule field.

## Public-boundary items this record adds, enumerated for Tom and not self-accepted

ADR 0096 enumerates seven items, none of which this record accepts or narrows. These are the ones the multi-round form adds. Each is a *type-system reservation* — none compiles — and each arrives at Tom individually under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md) with the implementation ticket that reaches it.

1. **The composition's round-carrying fields**, and whether the round count sits on the composition's `CooperativeTile` exactly as it does today or is restated on the topology. The derivation uses the tile's `rounds` unchanged and states no second field; the boundary is whether the topology's `partition` doc's round-major convention becomes a stated field of the composition or stays the split's own convention.
2. **The block-index statement's shape.** ADR 0096's item 5 asks whether the schedule states `t = g·W + l` as a coordinate reconstruction, an execution binding, or a property of the partition; the multi-round form widens the same question to the triple `index = r·T + g·W + l` and does not force an answer.
3. **A round-canonicality rule name** — the refusal of `T·k·(R−1) >= C`, which is a new named rule in the cooperative rule vocabulary and is deliberately not folded into `ContributorSplit`, because a tile whose last round is entirely padding is not a coverage arithmetic failure and would be reported as one.
4. **A padded-coverage rule name**, distinguishing "the split covers the padded sequence and the padding is `T·k·R − C`" from the exact-product rule `CooperativeWorkgroup` keeps. The two topologies must not share one rule identifier, because a reader of an explanation has to know which sequence was covered.
5. **A stated padding identity for the identity-less family**, its value `0xff80_0000`, and the statement that it is separate from `EmptyDomainContract::NoIdentity` rather than a weakening of it. Shared in substance with ADR 0096's fourth consequence, which already says the lane identity "should land as one concept with the other two".

**And one item that is not a public boundary and is named so a reader does not treat it as one.** The peeled round zero is an emission shape, already landed and tested, with no public field. It is load-bearing for the composition's numerics, which is why the deferrals file a test rather than a boundary question.

## Deferrals, each with the evidence that would close it and a trigger

- **No cost evidence separates a multi-round composition from a single-round one with a larger `k`.** Both cover the same sequence with the same leaf order and the same permission; the multi-round form pays `2R − 1` barriers and the single-round form pays a longer per-participant serial prefix. Closes with a measured case on a device, under the loop this repository's performance discipline requires. Trigger: the first target profile that declares a subgroup width, since no profile can answer the composition's feasibility until then.
- **The composition is derived for a contributor stream whose blocking is free.** A reduction whose contributors are themselves staged by the same tile — a fused blocked contraction feeding a row reduction — is the case where multi-round is forced rather than chosen, and it needs a second staging allocation whose coverage rule this record does not derive. Closes with a composition over two allocations. Trigger: [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](../../../tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md) reaching a fused reduction.
- **The peel's generality is not proved.** The derivation reads `emit_loop_carried_cooperative` as the emission the composition would extend, but the composition's inner level is a register-transfer tree that emission has no form for, so a future two-level emission could reintroduce a seeded accumulator. Closes with a test that watches a seeded cross-round accumulator fail. Trigger: the first emission that lowers the composition.
- **The round-dependent staged span stays deferred.** §8 derives that this composition does not fire it. Its own triggers are unchanged and unfired.

## Measurement boundary and unsupported cases

- **Nothing here was executed, emitted, compiled, dispatched, or timed.** Every claim is inspected source at `1d918b67` or a primary vendor specification relayed through [the two-level subgroup-then-workgroup reduction](two-level-subgroup-workgroup-reduction.md). The barrier counts in §5 and the leaf orders in §2 are derived under a stated convention, and no cost model in this repository prices a barrier, a shuffle, or an addition against one another.
- **The two vendor facts are relays, not primary reads.** The implementation-defined division of a threadgroup into SIMD-groups (MSL 4.1 §5.2.3.6, page 153) and WGSL §15.5's absence of a defined relation to `local_invocation_index` are cited from the two-level record, which extracted them from a locally downloaded copy. This record opened neither document.
- **`f32` only, two families, one width, one output per threadgroup.** Every numerical derivation is stated for IEEE 754 binary32 under `roundTiesToEven`, for `StrictSerialSum` and `StrictSerialMaximum` alone. A multi-axis reduction, a contraction, a non-power-of-two width, several outputs per threadgroup, more than one staged allocation, and any accumulation type other than `f32` each raise questions this record does not touch.
- **The `-inf` neutrality argument is a case walk over the pinned family's stated semantics, not a proof search.** It rests on `BinaryOp::F32Maximum` being IEEE 754-2019 `maximum` with `-0.0 < +0.0`, on NaN propagation, and on the fold canonicalizing after every combine. A family whose NaN behaviour differs — the `MaximumNumber` sibling ADR 0023 names and this vocabulary does not have — would need the walk re-run.
- **No target profile row, no backend claim, no realization.** No profile declares a subgroup width, no backend emits a shuffle for this shape, and the composition remains outside the implemented zero-synchronization schedule profile exactly as the one-round composition is.
- **Nothing in `crates/` was edited.** The doc-comment contradiction §6 records and the test §9 asks for are filed, not applied; this ticket's scopes are research and decisions.
