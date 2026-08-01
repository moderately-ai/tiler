---
schema: "tiler-doc/v1"
id: "tiler.research.scheduling.two-level-subgroup-workgroup-reduction"
kind: "research"
title: "The two-level subgroup-then-workgroup reduction"
topics: ["scheduling", "gpu", "metal", "webgpu", "reductions", "numerics", "execution-hierarchy", "subgroup"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "adopted"
adopted_by: ["ADR-0096"]
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.fusion-and-scheduling", "tiler.contract.ir"]
depends_on: ["tiler.research.scheduling.subgroup-execution-tier", "tiler.research.scheduling.scheduled-region-model", "tiler.research.scheduling.cpu-vector-lane-tier"]
ticket: "compose-the-two-level-subgroup-and-workgroup-reduction"
---

# The two-level subgroup-then-workgroup reduction

Every repository claim here is read at base commit `2aa0824`, and every claim labelled **Fact** is either inspected source in this repository — cited by symbol, which is the durable reference — or a primary vendor specification cited by document, version, section, and page. Claims are labelled **Fact**, **Inference**, **Proposal**, and **Measurement**; there is no Measurement in this record, and that absence is a boundary rather than an oversight — nothing here is executed, emitted, compiled, or timed, and the ticket's non-goals forbid all four. The operation counts in Worked example C are *derived* counts, not timings, and they say nothing about which schedule is faster on any machine.

This record composes with [the subgroup execution tier](subgroup-execution-tier.md), whose nine decisions are accepted as [ADR 0094](../../decisions/0094-bind-a-subgroup-combine-to-a-register-transfer-tree.md) and are treated here as settled rather than re-derived, and with [the CPU vector-lane tier](cpu-vector-lane-tier.md), whose identity obligation is the sibling of the one this composition inherits twice over. It answers the five questions its ticket asks and closes one of ADR 0094's six adopted deferrals.

**Two premises this work was dispatched with are wrong and are corrected in place rather than dropped**, because a reader who believes one of them is better served by seeing it refuted than by never seeing it. They are the ticket's own framing of question one and the trigger narrowing ADR 0094 proposed for `MemoryScope::Subgroup`, and both are corrected in §1 and §3 and listed again under the measurement boundary.

## Conclusion

**The composition's contributor partition is flat, and the ticket's premise that it needs "a hierarchical partition the vocabulary does not have" is refuted.** A threadgroup of `T = G·W` invocations in which each invocation folds `k` contributors is `ContributorPartition { partitions: T, contributors_per_partition: k }` — the type `tiler-ir` already has, satisfying the equality `verify_cooperative_semantics` already checks. **What is two-level is the *combine*, not the split**, and correcting that is what makes the remaining four questions tractable: they are all questions about a combine tree with a staged level in the middle, not about a partition vocabulary.

**It is a third `ReductionTopology` variant, and the decisive ground is that the staging coverage rule inverts.** Today a cooperative tile's staged writes are a bijection from *every* participant onto the allocation's slots — `verify_cooperative_tile` states it as one occupancy map and the source comment calls it a bijection. Under the composition the writers are `G` of the `T` participants and the slots are `G`. One variant carrying both rules keyed by an optional field is the "make the obligations optional and the verifier can no longer tell which to discharge" defect ADR 0094 decision 9 already refuses for the CPU and subgroup lanes.

**The outer level's participants are every invocation, and neither of the ticket's two candidates survives.** A strided `ParticipantRange` does not fix the slot arithmetic, and a declared per-access participant subset is the construct the cooperative module says is "absent rather than reserved". What survives is a **two-component local coordinate** — subgroup index and lane index — from which the writer set is *derived* as "lane index equals the result lane" rather than declared. Arrival stays uniform, so the point stays convergent; the narrowing is stated the way `CooperativeTile::commit` already states the narrowing at the level above.

**The composition does not need `MemoryScope::Subgroup`, and the trigger ADR 0094 proposed for it is itself wrong.** A handoff *between* simdgroups is read by invocations outside the writer's simdgroup, so its visibility must reach the *threadgroup*; the fence the composition derives is the one `required_subject` already returns. The construct that would need subgroup-scoped visibility is a staged allocation whose writer and every reader share one simdgroup, which the composition does not have.

**The composed leaf order is ascending and consumes reassociation alone — but only because the contributor-block index is the (subgroup index, lane index) pair, and that is not the coordinate the implemented vocabulary has.** Assigning contributor blocks by the linear local invocation index while combining by simdgroup structure consumes a permutation whose *shape is implementation-defined*: Metal states that "threads are divided into SIMD-groups in an implementation-defined fashion" and WGSL states there is "no defined relationship" between subgroup values and `local_invocation_index`. This is the composition's sharpest new correctness result and it has no analogue at either tier below.

**The identity obligation is at the inner level always and escapes the outer level, and the reason is the asymmetry ADR 0094 decision 6 already draws — now appearing inside one schedule.** The inner width is imposed by the device, so `W ∤ (C/G)` is the general case; the outer participant count `G` is chosen by the schedule, and the admitted outer form is a fold over exactly the `G` slots that exist, whose coverage is exact by construction. A second width-`W` shuffle tree at the outer level — the vendor idiom — would need a second proved identity and is deferred with a trigger rather than admitted.

## The line this composition turns on: the split is flat, the combine has a staged level

Reading the composition as "two partitions nested" is the error this section exists to prevent, and it is the error the ticket's first question is written in.

**Fact.** `ContributorPartition` (`crates/tiler-ir/src/schedule/model.rs`) carries exactly `partitions` and `contributors_per_partition`, and its doc states the contract: "partition `p` covers the contiguous contributor range `p * contributors_per_partition .. (p + 1) * contributors_per_partition` of the [`ContributorOrder`] the region declares, and the final pass combines the partials in ascending `p`. A multi-pass split is therefore a *reassociation* of the declared contributor sequence and never a permutation of it."

**Fact.** `verify_cooperative_semantics` (`crates/tiler-ir/src/schedule/builder.rs`) requires `partition.covers(contributors)`, `partition.partitions == tile.coordinates.participants.count`, and — in `verify_cooperative_tile` — `participants.count == region.schedule.threads_per_workgroup`, with the comment "Uniform convergence. Every launched invocation of the workgroup is a participant, so a synchronization point placed in any phase is one they all reach."

**Inference, and it is this record's load-bearing one.** Chain those three equalities. A threadgroup of `T` invocations in which every invocation folds `k` contributors is already `ContributorPartition { partitions: T, contributors_per_partition: k }`, and `T` is already required to be the launched workgroup width. Whether those `T` invocations then combine their partials through one serial fold over `T` staged slots or through `G` shuffle trees followed by a fold over `G` staged slots **changes nothing about the split**: the same `T` contiguous blocks cover the same sequence exactly once each, in the same ascending order. Two levels of partition would mean two levels of *splitting*, and there is only one — the second level regroups partials that the first level already produced.

**So the composition is a `ContributorPartition` the vocabulary already has, combined by a tree the vocabulary does not have.** Every remaining question is a question about that tree, about where its staged level sits, and about what the staged level obliges — which is exactly the shape the subgroup tier record found one level down, where the finding was that the tier turns on transfer medium rather than tree depth. Here the finding is the converse: the composition turns on tree *shape*, because the transfer media are both already modelled and one of each is used.

**And this is why the adopted model's "each reduction domain has exactly one topology" is not the obstacle the ticket reads it as.** [The scheduled-region model](scheduled-region-model.md) states that rule against a schedule that names both a `SubgroupTree` and a `WorkgroupTree` for one domain and leaves their relation to the reader. One topology whose *content* is a two-level combine violates nothing: it names one contributor coverage, one per-contributor serial order, one combine tree, one accumulator dtype, one identity behaviour, one result visibility, and one owner — the six things the model's own reduction plan requires a topology to name.

## What the implemented vocabulary already has, and the two things it does not

**Fact.** `ReductionTopology` (`crates/tiler-ir/src/schedule/model.rs`) has five variants — `None`, `Serial`, `MultiPass`, `Contraction`, `CooperativeWorkgroup` — encoded with tags `0x31` through `0x35`, and it is `#[non_exhaustive]` under ADR 0074 convention 5a, so a sixth lands additively.

**Fact.** `CooperativeTile` (`crates/tiler-ir/src/schedule/cooperative.rs`) carries `coordinates`, `staging`, `phases`, `synchronization`, and `commit`. `LocalCoordinates` carries a `LocalCoordinateSource` and a `ParticipantRange`. `LocalCoordinateSource` has exactly one variant, `LocalLinearInvocation` — "the linear index of one invocation within its own workgroup" — and is deliberately not `#[non_exhaustive]` because "the identity encoder maps this totally".

**Fact.** `ParticipantRange` is `{ first: u64, count: u64 }` and its doc reads "A contiguous run of local invocation coordinates." It has two methods, `end` and `contains_range`, and no member function that could describe a stride, a mask, or a subset. It appears as a field in exactly four places: `LocalCoordinates::participants`, `CooperativePhase::participation`, `CooperativeTile::commit`, and `SynchronizationPoint::participants`.

**Fact — and it is the constraint the ticket's second question underestimates.** `CooperativeTile::addressed_slots` is the only place a `ParticipantRange` is ranged over, and it walks `first .. first + count` with unit step unconditionally, computing `span.stride * local + span.offset` for each. `verify_cooperative_tile` then requires two things of the result together, in one occupancy map, with the comment "One writer per slot, and every in-range slot written. The two are checked together over one occupancy map because they are the two halves of the same statement: the participants' writes are a bijection onto the allocation's slots." A second write to a live slot is `CooperativeTileRule::StagingConflict`; an unwritten in-range slot is `CooperativeTileRule::StagingCoverage`.

**Fact.** `CooperativePhase::participation` must equal the tile's participant range at every phase (`CooperativeTileRule::PhaseParticipation`), and the module doc states why the rule cannot be relaxed: a phase's `participation` "is *arrival* and must stay uniform for the point between rounds to be convergent". The narrowing between the existing tree's two levels "is stated by [`CooperativeTile::commit`] rather than by a span".

**Fact.** `required_subject` (`crates/tiler-ir/src/schedule/synchronization.rs`) derives, for any nonempty edge set over workgroup staging, exactly one subject: `ControlBarrier`, execution scope `Workgroup`, visibility scope `Workgroup`, `FencedSpaces { workgroup: true, device: false }`, ordering `AcquireRelease`. `SynchronizationScope` has a `Subgroup` variant; `MemoryScope` in the structured kernel IR (`crates/tiler-ir/src/kernel/model.rs`) has only `Workgroup` and `Device`.

**Inference.** Of everything the composition needs, exactly two things are absent rather than merely unwidened. The first is a local coordinate with two components. The second is a way to say that a staged write is performed by the participants a *component* of that coordinate selects — and the second is a consequence of the first rather than an independent construct, because once the coordinate has components, "lane index equals the result lane" is a derivation over the coordinate rather than a declared subset. Everything else — the tile, the staging, the phases, the barrier, the subject, the commit, the partition — is used at its existing meaning.

## 1. The composition is a third `ReductionTopology` variant

**Elimination.** Three candidates, of which the ticket names all three.

*A hierarchical `ContributorPartition` that both existing variants read* is **eliminated by refutation of its premise**, derived above: the composition's split is flat, so there is no hierarchy for a partition to carry. The candidate also fails on its own terms if the premise were granted. `ContributorPartition` is read by `MultiPass`, whose levels are separated by a *dispatch* boundary and whose second level the adopted model routes to a `KernelSubprogram`; a hierarchical partition would force `MultiPass` to answer what a nested split means across that boundary, which is a question the model has already answered elsewhere and differently. And `covers` is a single product — the exactness check every split rests on — so a hierarchical version would have to become a nested product that one of its two readers has no combine structure for.

*A nesting field on `CooperativeWorkgroup`* is eliminated on three independent grounds, any one sufficient.

First, **the staging coverage rule inverts, and one variant would carry both.** With the field absent, the staged writes are a bijection from all `T` participants onto `T` slots. With it present, they are a bijection from `G` selected participants onto `G` slots, and the enumeration that decides it must range over a different set. `verify_cooperative_tile` decides both halves in one occupancy map; a variant that forked that map on an `Option` would be the shape ADR 0094's last alternative refuses — "it would have to make width authority, source-lane activity, and ownership all optional, at which point the verifier can no longer tell which obligations to discharge."

Second, **the derived target requirements differ.** ADR 0094 decision 7 gives a subgroup combine a width equality against an atomic subject at the compile profile and a confirmation against the prepared pipeline before routing commit. `CooperativeWorkgroup` derives neither; the composition derives both. ADR 0007 makes resource requirements a deterministic derivation from the identity-bearing schedule, and a derivation whose *output set* depends on whether an optional field is present is a conditional where the layering wants a total function.

Third, **identity injectivity is argued per tag.** `push_schedule` writes a topology tag and then that arm's fields; `0x35`'s arm already carries a note at its `arrival` field explaining that the byte was appended at the end rather than placed where it belongs by meaning, so that "every earlier field of this arm keeps its offset". A sixth tag is appends-only and injective by construction — no earlier region could carry `0x36`, so no earlier identity can collide with a new one — while extending `0x35` requires a presence byte and re-argues the offset reasoning for a second time on one arm.

*A separate schedule that names both a subgroup tree and a cooperative tile* is eliminated by the adopted model's one-topology rule and by the ticket's own framing: two topologies for one reduction domain leave their composition unstated, so the verifier would check two halves and nothing about their relation. The composition's whole content is the relation.

**What survives** is a sibling variant, structurally parallel to `CooperativeWorkgroup`, differing in the fields the second combine level changes:

```text
SubgroupThenWorkgroup {
    partition:              ContributorPartition,   // partitions == T == G · W
    width:                  SubgroupWidth,          // a literal, ADR 0094 decision 7
    groups:                 u64,                    // G, derived-checkable as T / W
    inner:                  CombineTree,            // ascending butterfly, ADR 0094 decision 4
    lane_identity_bits:     u32,                    // ADR 0094 decision 5
    result_lane:            u64,                    // the lane that stages
    tile:                   CooperativeTile,        // G slots, one barrier
    outer:                  ContributorArrival,     // AscendingParticipant
    axes:                   Vec<Axis>,
    order:                  ContributorOrder,
    accumulation:           ArithmeticType,
    permits_reassociation:  bool,
    permits_permutation:    bool,
}
```

**Every field is either a field one of the two existing constructs already carries or a field ADR 0094 already decided the meaning of.** The variant contributes no new *concept*; it contributes the statement that these two constructs compose in one topology, and the obligations §2 through §5 derive.

**And the tile it carries is smaller than the one the workgroup tier carries for the same reduction**, which is the composition's structural point rather than a performance claim: the allocation is `G` slots instead of `T`, so the region's `local_memory_bytes` derivation returns `G · 4` rather than `T · 4`, with `G = T / W`.

## 2. The outer level's participants are every invocation, and the writer set is derived from a two-component coordinate

The ticket asks whether the outer level's participants are the subgroup result lanes — "a strided subset of the workgroup, which `ParticipantRange` cannot express" — or every invocation with a predicate, and what each does to the uniform-participation rule.

**Both candidates as stated are eliminated, and the second is eliminated by the rule the question is about.**

*Every invocation with a predicate* is eliminated because the predicate is exactly the construct the vocabulary declines to have. The cooperative module states it twice: "A log-depth tree therefore needs a per-access active-participant subset — separate from a phase's `participation`, which is *arrival* and must stay uniform for the point between rounds to be convergent — and that subset **is absent rather than reserved**," and again on `workgroup_tree_tile`, "a `StagedSpan` is addressed by every participant of the tile — the slot enumeration runs over the tile's whole participant range, not over a per-access subset." Reaching for a declared predicate here would admit that subset for the composition while the log-depth workgroup tree — which is the *same* construct at a different level — stays refused for wanting it. Two levels of the same vocabulary would then disagree about whether a per-access subset exists.

*A strided `ParticipantRange`* is eliminated as **insufficient**, which is a stronger ground than the ticket's, and the arithmetic is worth writing out because the type's shape hides it. Suppose `ParticipantRange` gained a stride and the staging phase's participants were `{0, W, 2W, …, (G−1)W}`. `addressed_slots` computes `span.stride * local + span.offset` where `local` is the *local coordinate*, so the participant at coordinate `g·W` addresses slot `span.stride · g · W + span.offset`. For the writes to be the required bijection onto slots `0 .. G` this needs `span.stride = 1/W`, which is not a `u64`. Making it work needs `StagedSpan`'s `l` to mean *ordinal within the participant set* rather than *local coordinate* — a silent reinterpretation of a landed public type whose doc says "Participant `l` addresses the `count` slots beginning at `stride * l + offset`" and whose module doc rests the decidability of disjointness and coverage on that exact form. So the candidate needs two changes, one of which is a reinterpretation, and it still leaves the arrival question unanswered.

**And if a strided range were used as a phase's `participation` rather than only as a write extent, it would break the point.** **Fact — Metal Shading Language Specification 4.1, §6.10.1, page 216 (2026-06-04).** "If `threadgroup_barrier` (or `simdgroup_barrier`) is inside a conditional statement and if any thread enters the conditional statement and executes the barrier function, then all threads in the threadgroup (or SIMD-group) need to enter the conditional and execute the barrier function." A staging phase reached by `G` of `T` invocations places the barrier inside a conditional the other `T − G` do not enter, which is the divergence `CooperativeTileRule::PhaseParticipation` exists to refuse.

**What survives is a two-component local coordinate, and it is independently forced by §4.** `LocalCoordinateSource` gains a second source so that a participant's coordinate is the pair `(g, l)` — subgroup index and lane index — with `t = g·W + l` stated by the schedule rather than read from a linear builtin. Then:

- **Arrival stays uniform.** `participation` is the whole participant range at every phase, so the barrier is reached by every invocation and `ConvergenceEvidence::EveryParticipantReachesThePoint` holds unchanged. The composition adds no convergence obligation the workgroup tier does not already discharge.
- **The writer set is a derivation, not a declaration.** "The participants whose lane component equals `result_lane`" is a total function of `W` and `result_lane`, so it cannot be stated wrongly the way a declared subset can. This is the same move the module already makes for visibility edges — "the edges are still derived here and never declared".
- **The slot address is affine in the component that indexes it.** Participant `(g, result_lane)` writes slot `g`, so the enumeration that decides disjointness and coverage is `g ∈ 0..G` against `G` slots — a bijection, decided by the same enumeration, with no reinterpretation of `StagedSpan`.
- **The narrowing has a precedent one level up.** `CooperativeTile::commit` narrows the *output* writers to one participant, and `verify_cooperative_tile` checks it as `commit.count == 1 && participants.contains_range(commit)`. The composition narrows the *staging* writers to `G` participants by the same kind of statement, one level earlier.

**Inference — so the answer to the ticket's second question is neither of its two candidates, and the uniform-participation rule is not what has to give.** The rule survives untouched, because the thing that narrows is not arrival. What has to give is the *coordinate space*: a workgroup whose only coordinate is a linear index cannot name the structure the composition combines over, and every difficulty the question raises is a symptom of naming a two-dimensional structure with a one-dimensional coordinate.

## 3. The composition does not need `MemoryScope::Subgroup`, and the trigger proposed for it is wrong

ADR 0094 carries a deferral reading "whether the subgroup tier ever needs `MemoryScope::Subgroup`, whose answer is no for a shuffle tree", and both it and [`add-subgroup-memory-scope-when-collectives-land`](../../../tickets/add-subgroup-memory-scope-when-collectives-land.md) propose narrowing that ticket's trigger to "a staged handoff *between* simdgroups", naming this composition as the construct that fires it. **That narrowing is wrong, and this is the question that catches it.**

**Fact — MSL 4.1, §6.16.2, page 300.** The `thread_scope` enumeration is `thread_scope_thread`, `thread_scope_simdgroup`, `thread_scope_threadgroup`, `thread_scope_device`, and "Informally, the thread scope on a synchronization operation defines the set of threads with which this operation may synchronize, or which may synchronize with the operation."

**Fact — MSL 4.1, §6.10.1, page 216.** "The scope argument (see section 6.16.2) specifies which threads can observe the memory accesses to the address space identified by flags. The accesses become visible within the same threadgroup, within the same SIMD-group, or across all threads on the device."

**Inference, and it is one line.** A subgroup memory scope means *these accesses become visible within one SIMD-group*. In the composition the value staged by lane `result_lane` of simdgroup `g` is read by the committing participant, which is in simdgroup `0`; for `g ≠ 0` the writer and the reader are in different simdgroups. A visibility scope that publishes within one simdgroup does not publish that value to its reader. **A handoff between simdgroups therefore requires threadgroup-scope visibility, which is `SynchronizationScope::Workgroup` and `MemoryScope::Workgroup` — the values `required_subject` already derives.** The narrowing reads "between simdgroups" as "narrower than the workgroup", and the word "between" is doing the opposite work: crossing a boundary requires reaching across it.

**Fact — corroborating from inside the repository.** `required_subject`'s own comment on the visibility scope reads "The readers are the same set as the writers, so publication has to reach the workgroup and no further." In the composition the readers are a superset of the writers rather than the same set, which strengthens the conclusion rather than weakening it: publication still has to reach the workgroup, and there is still nothing wider to reach.

**So the composition fires no subgroup memory scope, and what would fire one is narrower than either the original trigger or the proposed narrowing.** The construct that needs `MemoryScope::Subgroup` is a staged allocation through threadgroup memory whose writer and *every* reader lie in one simdgroup — a simdgroup-private scratch tile — because that is the only case where a publication narrower than the threadgroup is both sufficient and cheaper. The composition has no such allocation: it has one allocation whose readership is the whole threadgroup.

**Proposal — the trigger should read: the first schedule declaring a staged allocation whose writers and all of whose readers lie in one subgroup.** This record does not edit the deferred ticket's frontmatter; it adds the correction as an addendum, because the ticket's trigger is a claim about when work becomes reachable and a wrong one makes unreachable work look reachable.

**And one thing this does not disturb.** `barrier_call`'s `ExecutionScope::Subgroup => "simdgroup_barrier"` binding in `tiler-metal` stays dead under the composition, for a different reason than it stayed dead under the subgroup tier: there the schedule declared no point at all, here it declares a point whose execution scope is `Workgroup`. Both leave the mapping unreached, and neither is evidence about whether it is correct.

## 4. The composed leaf order is ascending only if the coordinate is stated, and this is the composition's new hard requirement

ADR 0094 decision 4 fixes the test: read the combine tree's leaves left to right, and if they are the declared contributor sequence, only the parenthesization moved. A two-level tree's leaf order is the composition of the two levels, and the ticket is right that it is not obviously ascending.

### The composition, worked

Take `W = 32`, `G = 4`, `T = 128`, `k = 1`, so the padded contributor sequence is `b0 … b127` and invocation `t` owns contributor `b_t`.

**Level one, inside simdgroup `g`.** ADR 0094 decision 4 admits the ascending-mask butterfly at masks `1, 2, 4, 8, 16`, whose leaves read left to right are the lanes of that simdgroup in ascending lane order. Writing the block that simdgroup `g` owns as `b_{32g} … b_{32g+31}`, the partial is

```text
P_g = (((b_{32g}+b_{32g+1}) + (b_{32g+2}+b_{32g+3})) + ((b_{32g+4}+b_{32g+5}) + (b_{32g+6}+b_{32g+7}))) + …
```

a balanced binary tree whose leaf order is `32g, 32g+1, …, 32g+31` — ascending, and nothing crossed a block boundary.

**Level two, across simdgroups.** `P_g` is staged into slot `g` and the committing participant folds slots `0..4` in ascending slot order:

```text
y = ((P_0 + P_1) + P_2) + P_3
```

**The composition.** Substituting each `P_g` and reading the whole expression's leaves left to right gives `b_0, b_1, b_2, …, b_127` — the declared contributor sequence, ascending. **Reassociation alone; no permutation.** The composition of two ascending trees over a partition that is contiguous in the same coordinate is ascending, and that is the whole of the positive result.

### Where it breaks, and the break is the reason the coordinate must be stated

Three ways the leaf order stops being ascending. The first two are inherited; the third is new and is the one this record contributes.

**Inherited — a descending inner stride.** If level one is the `shuffle_down` tree the Metal specification prints, simdgroup `g`'s leaf order is the bit-reversal permutation of its block: `32g+0, 32g+16, 32g+8, 32g+24, …`. ADR 0094 decision 4 already refuses that form, so the composition inherits the refusal and adds nothing.

**Inherited — a non-ascending outer arrival.** If the staged partials combined in arrival order rather than ascending slot order, the composition would consume permutation for the reason `ContributorArrival::requires_permutation` already states. `ContributorArrival::AscendingParticipant` is the only admitted value and the composition uses it unchanged.

**New — the contributor-block index is not the coordinate the tree is built over.** Suppose the schedule assigns contributor block `t` to the invocation whose *linear local index* is `t`, and builds level one over the simdgroup structure. Then simdgroup `g` combines whichever invocations the device placed in it, and the composed leaf order is the image of `0..T` under that placement. **It is a permutation, and its shape is not a fact the schedule holds.**

**Fact — MSL 4.1, §5.2.3.6, page 153.** "In cases other than a tile function, the thread index in the threadgroup (`thread_index_in_threadgroup`) is determined by: `ly * Sx + lx`." And immediately after: "**Within a threadgroup, threads are divided into SIMD-groups in an implementation-defined fashion.** Any given thread in a SIMD-group can query its SIMD lane ID and which SIMD-group it is a member of." The exact check: `thread_index_in_threadgroup` occurs at seven places in the extracted text of the specification and none of them relates it to `simdgroup_index_in_threadgroup` or `thread_index_in_simdgroup`; the only sentence that addresses the relation is the one quoted, and it declines to fix it.

**Fact — WGSL specification, §15.5.** "There is no defined relationship between subgroup values … and `local_invocation_index`. To avoid non-portable code, shader authors should not assume a particular mapping between these two values." This is the fact ADR 0094 already enumerated as public-boundary item 4, cited there for a coordinate source that "carries no defined relation to `LocalLinearInvocation`"; here it becomes a *numerical* obligation rather than a vocabulary note.

**Inference — two independent specifications refuse to fix the mapping, so a schedule that assumes one is consuming a permission whose amount it cannot state.** Concretely, if a device divided a 128-thread threadgroup so that lane `l` of simdgroup `g` is linear index `4l + g` — a division neither specification forbids — simdgroup 0 would own contributors `b_0, b_4, b_8, …, b_124`, and the composed leaf order would be `0, 4, 8, …, 124, 1, 5, …, 125, 2, …` : exactly the strided layout ADR 0093 decision 3 identifies as consuming permutation, arrived at without anyone choosing it. **Same schedule text, same width, same instruction count, and the verdict under a permutation-forbidding contract flips on a fact the schedule does not carry.**

**So the composition derives a hard, target-independent requirement: the contributor-block index is the pair `(subgroup index, lane index)` ordered as `t = g·W + l`, stated in the schedule, and never the linear local invocation index.** Combined with §2 this is one requirement rather than two — the two-component coordinate is what makes the block assignment statable and what makes the staged write addressable, and neither is available on `LocalCoordinateSource::LocalLinearInvocation`.

**And the requirement is genuinely new rather than a latent defect in what is landed.** The implemented `CooperativeWorkgroup` tile assigns block `l` to the participant at local coordinate `l` and combines slots in ascending `l`; no simdgroup coordinate enters anywhere, so its leaf order is ascending by the same coordinate it partitioned by. The composition is the first construct in which two different coordinates could disagree, which is why the obligation appears here and nowhere below.

### The permissions, stated

**Inference.** The composition consumes **reassociation** and nothing else, under three conditions each of which is a stated field the verifier checks rather than an inference it makes: the inner tree's stride order is ascending, the outer arrival is `AscendingParticipant`, and the contributor-block index is the `(g, l)` pair in that order. It consumes **permutation** if any of the three is otherwise, and the third is the one that would consume it silently — which is why it must be a field and not a convention. Identity injection consumes no permission, by ADR 0094 decision 5's argument transferred unchanged: a true identity leaves the result bit-identical, so there is no freedom to grant and the obligation is a proof rather than a permission.

## 5. The identity obligation binds the inner level always and escapes the outer level

The ticket asks whether the obligation applies at one level or both, "given that the inner width is imposed and the outer participant count is chosen". That framing is right and the answer follows from it directly.

**The inner level cannot escape.** ADR 0094 decision 6: "Because the width is imposed by the device rather than chosen by the schedule, identity injection is required in the general case rather than being one tail policy among four." Nothing about composing a second level changes the width's authority. With `T = G·W` invocations each folding `k` contributors, the padded sequence is `T·k` and the real one is `C`; `W ∤ C` in general, so some invocations own no real contributor and must still enter the butterfly holding a stated two-sided identity — `-0.0` for `f32` addition under a contract forbidding signed-zero elimination, and never the region's `empty_identity_bits`, which the schedule verifier requires to be `+0.0`.

**The outer level escapes, and the reason is `G`'s provenance.** `G = T / W`, `T` is the schedule's launch geometry, and `W` is fixed, so `G` is a number the schedule chooses. The admitted outer form folds exactly the `G` slots that exist: the staged allocation has `G` slots, exactly `G` participants write them, and `verify_cooperative_tile`'s coverage check is satisfied by a bijection with nothing left over. **There is no ragged outer tail to pad, because the outer participant count and the outer slot count are the same chosen number.** This is the CPU tier's escape — "the workgroup tier could dodge the identity question by choosing a participant count that divides" — appearing at the outer level of a schedule whose inner level cannot use it.

**So the obligation is at the inner level only, for the admitted form.** One `lane_identity_bits`, one proof, exactly as ADR 0094 decision 5 requires it, discharged once for a schedule with two combine levels.

**And the escape is conditional on the outer form, which is why the outer form is a stated field.** If the outer combine were itself a width-`W` shuffle tree — the shape the Metal specification's example uses — it would read `W` lanes while only `G < W` hold partials, and the remaining `W − G` lanes would need a second stated identity. That form is deferred below rather than admitted, and the deferral is what keeps the composition at one identity obligation.

**Inference — and the second identity is exactly the one a transliteration of the vendor idiom would get wrong.** The specification's example stages into `threadgroup int *ldata` and reads back `val = (lid < s) ? ldata[lid] : 0;`. For `int` addition, `0` is a two-sided identity and the line is correct. Rewritten for `f32` — which is the only accumulation type `verify_cooperative_semantics` admits — the literal becomes `+0.0`, which is not a two-sided identity under `roundTiesToEven` because `(-0.0) + (+0.0) = +0.0`. **The vendor idiom's outer identity is correct for its own type and is the exact bug ADR 0094 decision 5 forbids as soon as the type changes**, which is a sharper reason to state the value than to derive it.

## 6. The vendor idiom read as a program, and why the composition must be one schedule

The ticket cites the Metal specification's example as the shape the composition takes. It is, and reading it as a *program* rather than as a shape is worth the space, because two of its properties are exactly the properties a verifier that checks the composition whole would refuse.

**Fact — MSL 4.1, §6.10.2.1 "Examples", pages 224–225 (2026-06-04), quoted verbatim from the primary document.**

```text
kernel void
reduce(const device int *input [[buffer(0)]],
       device atomic_int *output [[buffer(1)]],
       threadgroup int *ldata [[threadgroup(0)]],
       uint gid [[thread_position_in_grid]],
       uint lid [[thread_position_in_threadgroup]],
       uint lsize [[threads_per_threadgroup]],
       uint simd_size [[threads_per_simdgroup]],
       uint simd_lane_id [[thread_index_in_simdgroup]],
       uint simd_group_id [[simdgroup_index_in_threadgroup]])
{
     // Perform the first level of reduction.
     // Read from device memory, write to threadgroup memory.
     int val = input[gid] + input[gid + lsize];
     for (uint s=lsize/simd_size; s>simd_size; s/=simd_size)
     {
          // Perform per-SIMD partial reduction.
          for (uint offset=simd_size/2; offset>0; offset/=2)
               val += simd_shuffle_down(val, offset);
          // Write per-SIMD partial reduction value to
          // threadgroup memory.
          if (simd_lane_id == 0)
               ldata[simd_group_id] = val;
          // Wait for all partial reductions to complete.
          threadgroup_barrier(mem_flags::mem_threadgroup);

             val = (lid < s) ? ldata[lid] : 0;
      }
      // Perform final per-SIMD partial reduction to calculate
      // the threadgroup partial reduction result.
      for (uint offset=simd_size/2; offset>0; offset/=2)
           val += simd_shuffle_down(val, offset);
      // Atomically update the reduction result.
      if (lid == 0)
           atomic_fetch_add_explicit(output, val,
                                     memory_order_relaxed);
}
```

**Inference — the staging loop does not execute for any threadgroup smaller than `W·(W+1)` threads, and the printed program then drops every SIMD-group's partial but one.** The loop is entered iff `floor(lsize / simd_size) > simd_size`, that is iff `lsize ≥ simd_size · (simd_size + 1)`. At `simd_size = 32` the threshold is 1,056 threads. For any smaller threadgroup the body never runs, `ldata` is never written, and the executed program is: one pair sum per thread, one per-SIMD-group `shuffle_down` fold, and `if (lid == 0) atomic_fetch_add_explicit(...)`. After a `shuffle_down` fold only lane 0 of each SIMD-group holds its group's sum, and `lid == 0` is one thread of one SIMD-group, so at most one of the `lsize / simd_size` partials reaches the atomic. **At `lsize = 1024` and `simd_size = 32` that discards thirty-one of thirty-two partials.**

**Inference — and if the loop did execute more than once, the staged allocation carries a write-after-read hazard with nothing ordering it.** Each iteration writes `ldata[simd_group_id]`, barriers, then reads `ldata[lid]`. Nothing separates the read at the end of iteration `i` from the write at the start of iteration `i+1`, so a SIMD-group that reaches the next write before a slower one has performed its read overwrites a value that has not been consumed. The repair is a second barrier per round. **This is exactly the defect the implemented vocabulary refuses by construction**: `verify_cooperative_tile` reports `CooperativeTileRule::StagingConflict` on a second write to a live slot, with the comment "A second write to a live slot needs a per-round lifetime and a per-round visibility edge this profile does not model, so a rewriting tree is refused here rather than approximated."

**These two observations are stated with their limits.** They are derivations over a printed illustrative example in a specification, read at the exact pages cited; nothing here was compiled or executed, no claim is made about Apple's shipping code or about any compiler's treatment of the loop, and no normative MSL clause this record relies on is affected by either. What they establish is narrow and sufficient: **the idiom an implementer copies is, as printed, both under-covering and — in its multi-round form — unordered, and each defect maps onto a rule the implemented schedule vocabulary already names.** That is the concrete argument for the ticket's stated outcome: the composition has to be one schedule a verifier checks whole, because the alternative is the thing everyone copies.

**Fact — the specification's own recommendation is weaker than the composition's requirement, and the gap is the point.** MSL 4.1 §5.2.3.6, page 152: "The execution width of the compute unit, referred to as the `threads_per_simdgroup`, determines the recommended size of this smaller group. **For best performance**, make the total number of threads in the threadgroup a multiple of the `threads_per_simdgroup`." ADR 0094 decision 3 derives the same divisibility as a **correctness** obligation, because a trailing partial SIMD-group leaves lanes that the butterfly reads inactive. The composition inherits it and adds one consequence: with divisibility, `simdgroups_per_threadgroup = ceil(threads_per_threadgroup / threads_per_simdgroup)` (MSL §5.2.3.6, page 154) is exact, so `G = T / W` is a schedule-derived literal rather than a device query, and the staged allocation's slot count is an intrinsic fact.

## Worked example C — the composition, beside the subgroup tier's two

The program is the one [the subgroup execution tier](subgroup-execution-tier.md) and [the CPU vector-lane tier](cpu-vector-lane-tier.md) both use, unchanged, so all three can be read against each other:

```text
x : f32[7, 101]
y : f32[7]
y[m] = strict_serial_sum(n, x[m, n])      0 <= m < 7, 0 <= n < 101
```

**The region**, in the implemented vocabulary: one `ReductionContributor` read with `input_shape = [7,101]`, `output_shape = [7]`, `axes = [1]`, `order = OriginalAxisLexicographic`; `ScalarProgram::StrictSerialSum { axes: [1], order: OriginalAxisLexicographic, canonical_nan_bits: 0x7fc0_0000, empty_identity_bits: 0x0000_0000 }`.

**The contract** is `reassociation: Permitted`, `permutation: Forbidden`, `signed_zero: Forbidden`, everything else as the strict workspace contract — the same discriminating shape example A uses.

**The schedule.** One threadgroup owns one output row. `W = 32`, `T = 128`, `G = 4`, `k = ceil(101/128) = 1`. The padded contributor sequence is `T·k = 128`, so invocation `t` owns contributor `t` and positions `101..128` are padding. `ContributorPartition { partitions: 128, contributors_per_partition: 1 }` covers 128 exactly, and `partitions == participants == threads_per_workgroup == 128` satisfies the two equalities `verify_cooperative_semantics` and `verify_cooperative_tile` already check.

| Simdgroup | Invocations | Contributors | Real | Padding |
| --- | --- | --- | --- | --- |
| 0 | 0 – 31 | `b0 … b31` | 32 | 0 |
| 1 | 32 – 63 | `b32 … b63` | 32 | 0 |
| 2 | 64 – 95 | `b64 … b95` | 32 | 0 |
| 3 | 96 – 127 | `b96 … b127` | 5 (`b96 … b100`) | 27 |

**Twenty-seven padding positions, all inside simdgroup 3** — the same count example A carries, arrived at through a different shape, which is the coincidence that makes the two directly comparable.

**The tile.** One `WorkgroupStaging` of `G = 4` slots of `StagedElement::F32`, live from the producing phase through the consuming one — **16 bytes**. Two phases, each with `participation` equal to the whole 128-invocation range. The producing phase writes slot `g` from the participant whose lane component is `result_lane = 0`; the consuming phase reads all four slots. One `VisibilityEdge`, discharged by one `SynchronizationPoint` whose subject is exactly what `required_subject` derives: `ControlBarrier`, `Workgroup`/`Workgroup`, `FencedSpaces { workgroup: true, device: false }`, `AcquireRelease`. `commit = { first: 0, count: 1 }`.

| Obligation | Discharge |
| --- | --- |
| Reassociation | **Consumed.** `partitions = 128` splits one output's contributor sequence; admitted only because the contract grants it, by the check `verify_cooperative_semantics` already applies. |
| Permutation | **Not consumed**, because the inner masks ascend, the outer arrival is `AscendingParticipant`, and the block index is `t = 32g + l`. Change only the third and the same schedule consumes an implementation-defined permutation and is **rejected** — §4's counterexample, at identical instruction count. |
| Coverage | `128 × 1 = 128 = C + 27` exactly; every real contributor once, every padding position once. Staging: exactly 4 writers onto exactly 4 slots, a bijection. |
| Ownership | The participant at `(0, 0)` is the sole writer of `y[m]`; the other 127 invocations write no output. |
| Source-lane activity | Every inner step reads `l ^ m` with `m ≤ 16 < 32`, always another lane of the same simdgroup. |
| Launch | `threads_per_threadgroup = 128` is an exact multiple of 32, so no trailing partial simdgroup exists (MSL §4.4.1) and `G = 128/32 = 4` is exact. |
| Convergence | Every phase is reached by all 128 participants, so the point is convergent by `EveryParticipantReachesThePoint` — unchanged from the workgroup tier. |
| Inner identity | **The binding obligation.** `lane_identity_bits = 0x8000_0000`. |
| Outer identity | **None required.** Four slots, four writers, four reads. |

**The identity, at exact bits.** Take the row whose 101 contributors are all `-0.0`, whose true strict-fold sum is `-0.0` = `0x8000_0000`. With `lane_identity_bits = 0x8000_0000`, invocations 101–127 hold `-0.0`, every butterfly step of simdgroup 3 adds `-0.0` to `-0.0`, `P_3 = -0.0`, and the outer fold gives `0x8000_0000` — correct. With `0x0000_0000`, which is exactly what `empty_identity_bits` holds and what an implementer reaches for, invocations 101–127 hold `+0.0`, simdgroup 3's butterfly mixes `(-0.0) + (+0.0) = +0.0` at its first crossing step, `P_3 = +0.0`, and the outer fold gives `0x0000_0000` — **wrong by one bit, on a value the contract says is observable**. The trap is ADR 0094's, unchanged, and it reaches only the one simdgroup that owns padding, which makes it *harder* to hit in testing than at the subgroup tier where every lane above 24 was affected.

### The three forms priced against each other

Derived operation counts under the subgroup record's own convention — a fold of `n` values counted as `n` additions — so example A's figures are reproduced rather than recomputed. **These are derived counts, not measurements, and they say nothing about which schedule is faster on any machine.**

| | Subgroup tree (A) | Workgroup tile (B) | Composition (C) |
| --- | --- | --- | --- |
| Participants | 32 (imposed) | 101 (forced by primality of `C`) | 128 (chosen, multiple of 32) |
| Workgroup memory | 0 bytes | 404 bytes | **16 bytes** |
| Synchronization points | 0 | 1 | 1 |
| Visibility edges | 0 | 1 | 1 |
| Combine structure | 4 serial + 5 butterfly | 1 serial + 101 serial | 1 serial + 5 butterfly + 4 serial |
| `f32` additions on the critical path | 9 | 102 | **10** |
| Permissions consumed | reassociation | reassociation | reassociation |
| Identity padding | required (27 positions) | none | **required (27 positions), inner level only** |
| Target facts required | width equality + preflight | one synchronization subject; local memory | **both sets** |

**Inference — at `C = 101` the composition is not required and is barely distinguishable from A, and that is the honest reading of this row.** A covers 101 contributors at `k = 4`; C spends a barrier and 16 bytes of workgroup memory to reach the same permission verdict at one more addition. The composition earns its cost where the contributor sequence is long enough that A's serial prefix dominates, which the attention vertical supplies concretely.

**The row that motivates it.** [The first attention program vertical](../program-planning/first-attention-program-vertical.md) records `S = 8,192` at its B1-d prefill row, and a softmax row reduction over `S` is the reduction in question. At `C = 8,192`, all three forms are statable and their derived counts diverge:

| | A (`W = 32`) | B (`participants = 1024`) | C (`T = 1024`, `G = 32`, `W = 32`) |
| --- | --- | --- | --- |
| `k` | 256 | 8 | 8 |
| Workgroup memory | 0 bytes | 4,096 bytes | **128 bytes** |
| `f32` additions on the critical path | 261 | 1,032 | **45** |
| Identity padding | none (`32 · 256 = 8192`) | none (`1024 · 8 = 8192`) | none |

**Inference.** A's serial prefix is its whole cost at this length and B's combining level is a 1,024-way serial fold, which is the depth-two consequence the workgroup tier's own doc derives. C is the form in which neither level is long, and it is the only one of the three whose workgroup memory is a small constant times the *number of simdgroups* rather than the number of invocations. **Neither this table nor the one above decides which is faster; both are derived counts, and no cost model in this repository prices a barrier against an addition.**

**Fact — and none of the three is admissible in the attention block today.** The attention vertical records that "the kernel verifier admits no barrier under the implemented zero-synchronization schedule profile. A SIMD-group-cooperative row reduction survives; anything wider does not." B and C both declare a point, so both are outside that profile. The composition is what that block needs *once a barrier is admitted*, and the ordering of those two events is not this record's to decide.

## Public-boundary items, enumerated for Tom and not self-accepted

Nothing below is implemented, and none of it is accepted by this record's existence. Each is a type-system reservation in [ADR 0090](../../decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)'s sense — none compiles — and each arrives at Tom individually under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md) with the implementation ticket that reaches it. Items 2 and 3 are shared in substance with items ADR 0094 already enumerated and should land as one concept each rather than as two.

1. **`ReductionTopology` gains a two-level variant** — a `pub` enum variant in `tiler-ir`, its exact name, its field set, and whether `groups` is a stated field checked against `T / W` or derived from the pair.
2. **A second `LocalCoordinateSource` variant, and the decision that a participant's coordinate may have two components** — whether that is two sources composed by the schedule or one source naming a pair, and the statement that neither component carries a defined relation to `LocalLinearInvocation`. Shared in substance with ADR 0094's item 4.
3. **A staged access addressed by a named coordinate component, with a derived writer set** — whether `StagedSpan` gains a component selector, whether the writer narrowing is a field beside `commit` or a property of the span, and what the enumeration that decides disjointness ranges over.
4. **The `CombineTree` vocabulary reused at two levels**, and whether the outer level's form is `ContributorArrival` unchanged or a second `CombineTree` value. Shared in substance with ADR 0094's item 2.
5. **The stated contributor-block coordinate**, which is the field §4 makes load-bearing: whether the schedule states `t = g·W + l` as a coordinate reconstruction, as an execution binding, or as a property of the partition.
6. **The topology tag `0x36` and its appends-only argument**, which must be made at the encoding site on the tree the change lands into rather than asserted here — this record claims only that a new tag is injective by construction where an extension of `0x35` is not.
7. **A narrowed trigger on [`add-subgroup-memory-scope-when-collectives-land`](../../../tickets/add-subgroup-memory-scope-when-collectives-land.md)**, whose current and proposed forms are both refuted by §3. The addendum this record adds states the correction; changing the ticket's `status` or its stated trigger line is a graph decision rather than a research one.

## Deferrals, each with the evidence that would close it and a trigger

- **A second width-`W` shuffle tree at the outer level is not admitted.** It is the vendor idiom's shape, it needs `G ≤ W`, and it needs a second stated and proved identity that the admitted outer fold escapes (§5). Its derived saving over the admitted form is `G − log2(W)` additions on the critical path, which at `G ≤ 32` and `W = 32` is at most 27 — a count, not a measurement. Closes with a measured case where that saving is on the critical path, or with an outer identity landed for another reason. Trigger: a workload where `G` is at its maximum *and* the outer fold is measurably dominant.
- **A staging depth greater than one round is not admitted.** The composition stages once; the vendor example's loop stages repeatedly and is refused for it by `CooperativeTileRule::StagingConflict` (§6). More than one round needs the per-round lifetime and per-round visibility edge the cooperative module says it does not model, and that vocabulary is the subject of [`admit-loop-carried-cooperative-staging`](../../../tickets/admit-loop-carried-cooperative-staging.md), which was `in-progress` at this record's base commit — so this deferral closes on that ticket's outcome rather than on new derivation here. Trigger: a reduction whose contributor sequence exceeds what one threadgroup with one staging round can cover — which is a `KernelSubprogram` question first, and the adopted model routes multi-launch reductions there rather than here.
- **The composition is derived for one subgroup per staged slot and one staged value per subgroup.** A simdgroup staging several partials, or several simdgroups sharing a slot, changes the bijection the coverage check decides and is not considered. Closes with a multi-output or multi-axis composition. Trigger: a region whose reduction and its outputs do not stand one-to-one with threadgroups.
- **Whether the two-component coordinate is one construct or two is not decided here.** §2 requires that the coordinate have components and that the schedule state their composition; whether that is one `LocalCoordinateSource` naming a pair or two sources the schedule combines is a public boundary (item 2) and the derivation does not force either. Closes when the schedule vocabulary widening reaches Tom. Trigger: the implementation ticket for item 1.
- **No target declares a subgroup width, so no profile can answer the composition's feasibility today.** This is ADR 0094's condition inherited unchanged, and the composition adds no new target fact beyond the width equality and the preflight stage that record already derived. Closes with the first Metal subgroup realization. Trigger: `declare-metal-subgroup-realization-facts-in-the-target-profile`.
- **`MemoryScope::Subgroup` remains unneeded and its trigger remains uncorrected in the ticket's frontmatter.** §3 derives what would fire it. Closes when a schedule declares a staged allocation whose writers and all of whose readers lie in one subgroup. Trigger: that schedule, which no work in the graph currently proposes.

## Drafted ADR body — landed as [ADR 0096](../../decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md) on 2026-08-01, `decision_status: proposed`

**The record of the decision is [ADR 0096](../../decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md); the span below the rule is retained as the drafted text it landed from and is not a second authority over the same subject.** *And neither document is an authority yet*: the ADR is `proposed`, which under [the decisions index](../../decisions/README.md)'s own preamble means it remains a non-decision until Tom accepts it, so nothing in this record's **Proposal** labels weakened when it landed and none of them was rewritten after the fact. This record's `disposition` therefore stays `pending` and `adopted_by` stays unset; both move only at acceptance, which is [`accept-adr-0096-two-level-reduction`](../../../tickets/accept-adr-0096-two-level-reduction.md). The transfer was byte-identical — context, the eight numbered decisions, consequences, and the seven alternatives-considered entries — with the section headings demoted one level, from `###` nested under this heading to `##` under the ADR's own title, and nothing else changed.

**This span is a draft and is not a decision.** It was written verbatim-landable so that the transfer to `docs/decisions/` could be byte-identical, following the convention [the subgroup execution tier](subgroup-execution-tier.md) records: a transfer that edits is a fork, and byte-identity is what makes "unreworded at acceptance" checkable rather than asserted. The carrier transferred the span below the rule with `### ` mapped to `## ` and changed nothing else, and checked it by diffing the two ranges after normalization — no differences over 34 lines — after first perturbing one word and watching the check fail.

**The number was taken by reading the directory and the carrier read it again.** `0095` was the highest ADR present at `2aa0824`, so `0096` was drafted below — and `0093`, `0094`, and `0095` had all landed while records drafting against them were open, so the subgroup tier record's warning that the number moves is why the carrier re-read `docs/decisions/` rather than trusting the draft. `0095` was still the highest at the carrier's base `2119b20`, so `0096` was free and was taken unchanged; nothing else in the span depended on the number.

**The span below the rule carries no traceability section and therefore no relative links at all**, which avoids the tension AGENTS.md records for drafted bodies — a traceability section written with `docs/decisions/`-relative paths resolves at the ADR's destination and not from the record, so the record would have to state that beside the span rather than repoint it. Checked rather than assumed, at drafting and again at landing: the count of `](` inside the span's line range is zero while the count over the whole file is not. Cross-references the span needs are made by ADR number and by contract name in prose, which resolve from either location. The carrier wrote the ADR's traceability, normative-owner, work-record, implementation-boundary, and open-questions sections fresh at its destination.

**The scope split that made a carrier ticket necessary is the recorded one.** `ticketsplease.toml` routes `docs/decisions/[0-9]*.md` to `contracts/decisions` and both `docs/decisions/README.md` and `docs/research/README.md` to `contracts/navigation`, while this record's ticket holds `research/scheduling` and shared `project/tickets` only. [`land-the-two-level-reduction-adr`](../../../tickets/land-the-two-level-reduction-adr.md) took all three, following the `land-the-subgroup-execution-tier-adr`, `land-the-cpu-vector-lane-tier-adr`, and `land-the-bf16-conversion-and-accumulator-adr` precedents — and it carried **two** catalog rows, this record's under `docs/research/README.md`, which it had never had, and the ADR's under `docs/decisions/README.md`.

**One ground in the span went stale between this record's base and the landing, and the ADR records the correction rather than editing the span.** Decision 8 refuses a second staging round because the cooperative profile "does not model" a per-round lifetime, which was accurate at `2aa0824` and is not at `2119b20`: [`admit-loop-carried-cooperative-staging`](../../../tickets/admit-loop-carried-cooperative-staging.md) landed at `e4d2aa7`, `CooperativeTile` gained a `rounds` field, and the module now states that rewriting one slot across rounds is *not* what blocks a logarithmic tree. The deferral above that reads that ticket as `in-progress` and waits on its outcome is therefore answered on its trigger and unanswered on its substance — the ADR's open questions carry that forward.

---

**Title:** Compose the subgroup and workgroup reduction levels in one topology over a stated two-component coordinate

**Frontmatter:** `decision_status: proposed`, `implementation_status: not-started`, `catalog_group: "physical-planning-lowering"`, `applies_to: ["tiler.contract.fusion-and-scheduling", "tiler.contract.ir"]`, `evidence: ["tiler.research.scheduling.two-level-subgroup-workgroup-reduction"]`, `depends_on: ["ADR-0007", "ADR-0011", "ADR-0014", "ADR-0022", "ADR-0025", "ADR-0043", "ADR-0074", "ADR-0093", "ADR-0094"]`, `ticket: "land-the-two-level-reduction-adr"`.

### Context

ADR 0094 binds a subgroup combine to a register-transfer tree with a stated stride order and a proved lane identity, and states that nothing in it admits a two-level reduction. The adopted scheduled-region model states that each reduction domain has exactly one topology. ADR 0007 makes the normalized schedule authoritative and places contributor coverage, combine-tree numerical legality, and tail behaviour inside intrinsic verification. ADR 0014 separates reassociation from operand permutation, and ADR 0093 establishes that a combine tree's leaf order is what decides which of the two a split consumes.

**None of that says how the two levels compose**, and the shape is the one the Metal Shading Language Specification's own example reduction kernel uses and the one a softmax row longer than a subgroup needs. The implemented vocabulary reaches one level at a time: a cooperative tile whose staged writes are a bijection from every participant onto every slot, one local coordinate source naming a linear index within the workgroup, and no subgroup construct at all.

### Decision

1. **The composition's contributor partition is flat, and only the combine is two-level.** A threadgroup of `T = G · W` invocations each folding `k` contributors is the `ContributorPartition` the vocabulary already has, satisfying the equalities the cooperative verifier already checks. There is no hierarchical partition, because there is one split and two combining levels above it.
2. **The composition is a sibling `ReductionTopology` variant, not an optional field on the cooperative one.** The decisive ground is that the staging coverage rule inverts: the cooperative tile's staged writes are a bijection from every participant onto the slots, and the composition's are a bijection from a selected subset. One variant carrying both rules keyed by an option is a construct whose obligations the verifier can no longer tell apart. Two further grounds are independent: the composition derives a subgroup width equality and a prepared-pipeline preflight the cooperative topology derives neither of, and a new topology tag is appends-only injective where an extension of the existing arm is not.
3. **Every invocation participates at every phase, and the staging writer set is derived rather than declared.** Uniform arrival is what makes the barrier convergent and it is not what narrows; the narrowing is that a staged write is performed by the participants whose lane coordinate equals the result lane, which is a total function of the width and the result lane rather than a subset a schedule states.
4. **A participant's local coordinate has two components — subgroup index and lane index — and the schedule states their composition.** A one-dimensional coordinate cannot name the structure the composition combines over. A strided participant range does not repair it, because the staged slot address would have to be a fraction of the participant coordinate; a declared per-access participant subset is refused because it is the construct a log-depth workgroup tree is refused for wanting.
5. **The contributor-block index is the pair, ordered as subgroup-major, and never the linear local invocation index.** Metal states that threads are divided into SIMD-groups in an implementation-defined fashion and WGSL states there is no defined relationship between subgroup values and the local invocation index, so a schedule that partitions by one coordinate and combines by the other consumes a permutation whose shape it cannot state. Stated correctly, the composed leaf order of two ascending trees over one contiguous partition is the declared contributor sequence ascending, and the composition consumes reassociation alone.
6. **The identity obligation binds the inner level always and escapes the outer level.** The subgroup width is imposed, so a contributor-free lane is the general case and must hold a stated two-sided identity. The outer participant count is chosen and equals the staged slot count, so the outer fold's coverage is exact by construction and no outer padding exists. A second width-wide shuffle tree at the outer level would reintroduce the obligation and is stated and deferred rather than admitted.
7. **The composition requires no subgroup memory scope, and a staged handoff between subgroups is not what would require one.** Values crossing a subgroup boundary must be published to their readers, which lie outside the writer's subgroup, so the required visibility is the workgroup's — the subject the cooperative handoff already derives. A subgroup memory scope would be required by a staged allocation whose writer and every reader share one subgroup, which this composition does not have.
8. **One staging round, and a second is refused rather than approximated.** A tile that rewrites a live slot across rounds needs a per-round lifetime and a per-round visibility edge the cooperative profile does not model, and the vendor example's multi-round loop is the construct that refusal names.

### Consequences

- A reduction longer than one subgroup gains a schedule whose workgroup memory is proportional to the number of subgroups rather than to the number of invocations, and whose combining level is a tree rather than a participant-wide serial fold.
- The schedule vocabulary gains a coordinate with components, and the first obligation in the vocabulary that is about the *relation* between two coordinates rather than about either one.
- One schedule can be admitted on a permutation-forbidding contract when its contributor-block coordinate is stated and rejected when it is not, at identical instruction count, which makes the coordinate a planning fact rather than a lowering detail.
- The lane identity becomes the third construct needing a proved reduction identity and the second whose value is required rather than optional, and it should land as one concept with the other two.
- Nothing here admits a multi-round staging tree, a narrowing shuffle tree, a subgroup memory scope, a subgroup collective, a multi-launch reduction, or any Metal, CUDA, or WebGPU backend claim.

### Alternatives considered

- **A hierarchical contributor partition that both existing split topologies read.** Rejected: the composition's split is flat, so there is no hierarchy to carry, and a nested product would force the multi-pass topology to answer what a nested split means across a dispatch boundary the adopted model routes elsewhere.
- **An optional nested subgroup combine on the cooperative topology.** Rejected: the staging coverage rule inverts between the two cases, the derived target requirements differ, and one topology tag would key two programs whose verifier obligations are different.
- **Two topologies for one reduction domain.** Rejected by the adopted model's one-topology rule and because the relation between the levels is the whole content of the composition, so a representation that leaves it unstated states nothing.
- **A strided participant range for the staging level.** Rejected as insufficient: the staged slot address is affine in the participant coordinate, so a strided set of writers would need a fractional slot stride, and repairing that means silently reinterpreting what the staged span's participant index denotes.
- **A declared per-access active-participant subset.** Rejected because it is the construct a log-depth workgroup tree is refused for needing, and admitting it here would leave two levels of one vocabulary disagreeing about whether it exists.
- **Assigning contributor blocks by the linear local invocation index.** Rejected: two vendor specifications decline to fix the relation between that index and the subgroup structure, so the resulting leaf order is an implementation-defined permutation and the region checked against a numerical contract would not be the region that runs.
- **Using the vendor example's outer shuffle tree and its inline zero.** Rejected: it needs a second proved identity, and the literal that is a two-sided identity for its integer type is not one for the only accumulation type the cooperative verifier admits.

---

## Measurement boundary and unsupported cases

- **Nothing here was executed, emitted, compiled, or timed.** Every claim is inspected source at `2aa0824` or a primary vendor specification. There is no measurement in this record and therefore no measured bound on anything it claims. Every operation count in Worked example C is a *derived* count under a stated convention, and no cost model in this repository prices a barrier, a shuffle, or an addition against one another.
- **Two premises this work was dispatched with were checked and found wrong, and are corrected in place rather than dropped.** (i) The ticket's first question presumes the composition needs "a hierarchical partition the vocabulary does not have"; the split is flat and the vocabulary has the partition it needs, which is derived in the section on the line this composition turns on. (ii) ADR 0094 and the deferred ticket both propose narrowing the `MemoryScope::Subgroup` trigger to "a staged handoff between simdgroups"; §3 derives that such a handoff requires *workgroup*-scoped visibility, so the proposed trigger names a construct that does not fire it.
- **The two observations about the vendor example are readings of printed source, not executions.** The loop-entry derivation and the write-after-read hazard are arithmetic and ordering arguments over the code quoted verbatim at MSL 4.1 §6.10.2.1, pages 224–225. Nothing was compiled or run, no claim is made about any compiler's treatment of the loop or about Apple's shipping code, and no normative clause this record relies on depends on either observation. A reader who wants them settled would compile the example and count the atomic's contributions, which is a bounded experiment this record does not perform.
- **Where the specification claims were read.** Metal Shading Language Specification, Version 4.1, dated 2026-06-04, §4.4.1 (page 121), §5.2.3.6 (pages 152–154), §6.10.1 (pages 215–216), §6.10.2.1 (pages 223–226), §6.16.2 (page 300), extracted from a locally downloaded copy with `pdftotext -layout` and grepped, so the wording is exact. WGSL specification, §15.5. The IEEE 754-2019 clause 6.3 argument is ADR 0094's, transferred rather than re-derived, and that record's boundary note about the standard's distribution applies here unchanged.
- **`f32` only, one reduction shape, one width, one staging round.** Every numerical derivation is stated for IEEE 754 binary32 under `roundTiesToEven`, which is the only rounding Tiler admits and the only accumulation type `verify_cooperative_semantics` accepts. The examples use a single-axis sum over the trailing axis of a rank-2 input, an ascending butterfly at `W = 32`, one staged round, and one output per threadgroup. A multi-axis reduction, a contraction, a non-trailing reduced axis, a non-power-of-two width, several outputs per threadgroup, and more than one staging round each raise questions this record does not touch.
- **No target profile row, no backend claim, no realization.** No profile declares a subgroup width, no backend emits a shuffle or a barrier for this shape, and the composition is outside the implemented zero-synchronization schedule profile exactly as the workgroup tile is. The attention vertical's admissibility statement is unchanged by this record.
- **The appends-only claim for a new topology tag is an argument, not a discharge.** AGENTS.md requires per-tag injectivity reasoning at each encoding site on the tree the change lands into. This record claims only that a new tag admits that argument where an extension of the existing arm requires a presence byte and a second offset argument; the discharge belongs to the change that lands the encoder.
