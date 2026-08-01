---
schema: "tiler-doc/v1"
id: "tiler.research.scheduling.subgroup-execution-tier"
kind: "research"
title: "The subgroup execution tier"
topics: ["scheduling", "gpu", "metal", "cuda", "webgpu", "reductions", "numerics", "execution-hierarchy", "subgroup"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.fusion-and-scheduling", "tiler.contract.ir", "tiler.contract.metal-backend", "tiler.contract.artifact-abi"]
depends_on: ["tiler.research.scheduling.scheduled-region-model", "tiler.research.scheduling.cpu-vector-lane-tier", "tiler.research.target-profiles.physical-feasibility-model"]
ticket: "design-the-subgroup-execution-tier"
---

# The subgroup execution tier

Every repository claim here is read at base commit `8252312`, and every claim labelled **Fact** is either inspected source in this repository — cited by symbol, which is the durable reference — or a primary vendor specification cited by document and version. Claims are labelled **Fact**, **Inference**, **Proposal**, and **Measurement**; there is no Measurement in this record, and that absence is a boundary rather than an oversight — nothing here is timed, executed, emitted, or compiled, and the ticket's non-goals forbid all four. Three claims carried into this work as premises were checked against primary manuals and found wrong; they are corrected in place and listed under the measurement boundary rather than dropped, because a reader who believes one of them is better served by seeing it refuted than by never seeing it.

This record is the schedule half of the tier and is written beside [the CPU vector-lane tier](cpu-vector-lane-tier.md), whose derivations it composes with and, at two points, deliberately diverges from. It owns no target profile row: the Metal realization is [`qualify-the-simdgroup-matrix-contraction-realization`](../../../tickets/qualify-the-simdgroup-matrix-contraction-realization.md)'s, and its strict-profile elimination stands untouched.

## Conclusion

**A subgroup combine is a `ReductionTopology` sibling of `CooperativeWorkgroup`, and it is the first construct in the vocabulary that carries a cross-invocation dependency with no allocation, no visibility edge, and no synchronization point.** A shuffle names its source lane and its destination lane in one operation that is simultaneously the transfer and the ordering, so the three constructs the workgroup tier needs to state a handoff — a staged write, a staged read, and a point that discharges the edge between them — collapse into one combine step. That collapse is the whole content of the tier.

**Two obligations survive the collapse, and neither is a visibility obligation.** Every lane a combine step reads must be *active* at that step — a requirement all three vendor specifications state and none of them relaxes — and every lane must hold a value, which for a lane with no contributors means the stated identity. The second is where the tier's sharpest correctness result sits: the workgroup tier could dodge the identity question by choosing a participant count that divides the contributor sequence, and **the subgroup tier cannot, because the width is not the schedule's to choose.** A padding identity is therefore a forced, checked field at this tier, and its value for a sum is `-0.0` (`0x8000_0000`), not the `+0.0` the region already carries as `empty_identity_bits`.

**The tree's stride order decides whether it consumes contributor permutation, and the two orders cost the same.** An ascending-mask butterfly (`xor` masks `1, 2, 4, …, W/2`) is a balanced binary tree over the ascending contributor sequence and consumes reassociation alone. The descending-stride `shuffle_down` tree — the idiom Apple's own specification prints, and the shape the adopted scheduled-region model already flags as needing both permissions — reads its leaves in bit-reversed order and additionally consumes permutation. Under the common contract that permits reassociation and forbids permutation, the ascending butterfly is admissible and the textbook tree is not, at identical instruction count. This is the mirror image of the CPU tier's finding that the numerically legal layout is the cache-hostile one: here the numerically legal tree is free.

**The width is a literal in the schedule and an equality against the target, never a floor and never symbolic**, and it resolves across three phases rather than one. `SynchronizationKind::Collective` stays refused, and this design strengthens the refusal rather than activating it. The log-depth question the workgroup tier deferred does not arrive here: register-carried values dissolve the slot-addressing problem that made a per-access active-participant subset necessary, and replace it with a strictly different obligation the butterfly discharges by construction.

## The line this tier turns on: transfer medium, not tree depth

The workgroup tier and the subgroup tier both split one output's contributor sequence across execution instances and combine the partials. Reading them as "the same thing at two scopes" is the error this section exists to prevent, and the difference is not the scope at all.

**Fact.** `CooperativeTile` (`crates/tiler-ir/src/schedule/cooperative.rs`) states a handoff as three separate things: a `StagedWrite` naming an allocation and a `StagedSpan`, a `StagedRead` naming the same allocation, and a `SynchronizationPoint` whose `placement` determines which `VisibilityEdge`s it `discharges`. The module's own doc says the edges "are still derived here and never declared; the points are declared and their discharge is derived from where they sit."

**Fact — Metal Shading Language Specification, Version 4.1, §6.10.2 (2026-06-04, page 217).** "SIMD-group functions allow threads in a SIMD-group (see section 4.4.1) to share data without using threadgroup memory or requiring any synchronization operations, such as a barrier."

**Inference, and it is this record's load-bearing one.** A `VisibilityEdge` exists because a value crosses invocations through an *addressable location* and nothing in the tile orders the write against the read. A shuffle has no addressable location: the value leaves one lane's register and arrives in another lane's register as the operation's own result. There is nothing to fence, nothing to allocate, nothing whose lifetime spans two phases, and no second program point at which an ordering could be placed. `required_subject` (`crates/tiler-ir/src/schedule/synchronization.rs`) returns `None` for an empty edge set precisely because "a point that orders no handoff requires no realization, and an empty fence with an arrival scope would be a requirement stated over nothing" — and a subgroup tree derives no edges at all, so that `None` is the correct and complete answer rather than a gap.

**So the tier is defined by its transfer medium and everything else follows.** A staged handoff needs an allocation, a lifetime, a fence, an ordering, and a convergent point. A register handoff needs the source lane to be active and to hold the right value. Those are not the same obligations at two scopes; they are two different obligation sets, and a single construct spanning both would have to carry every field of each and leave most of them inapplicable.

## What the implemented vocabulary already has, and what it does not

**Fact.** `SynchronizationScope` already has a `Subgroup` variant with tag `0x01` and explain key `"subgroup"` (`crates/tiler-ir/src/schedule/synchronization.rs`), and the structured kernel IR already has `ExecutionScope::Subgroup` (`crates/tiler-ir/src/kernel/model.rs`), which `tiler-metal` maps to `"simdgroup_barrier"` in `barrier_call` (`crates/tiler-metal/src/emit.rs`). **The mapping is written and dead**: the very next match admits only `(Workgroup, Workgroup)` and returns `BarrierRejection::MemoryVisibility` for everything else, because `MemoryScope` has no `Subgroup` variant.

**Inference, and it corrects an expectation this ticket was dispatched with.** [`add-subgroup-memory-scope-when-collectives-land`](../../../tickets/add-subgroup-memory-scope-when-collectives-land.md) is deferred behind the trigger "when subgroup collectives (shuffles, reductions, ballots) enter the supported profile". **This design does not fire that trigger, and the reason is the section above**: a shuffle tree emits no barrier at any scope, so it never reaches `barrier_call`, never needs `MemoryScope::Subgroup`, and never touches the dead mapping. The ticket's trigger is written as though shuffles imply a subgroup barrier; they do not, and the trigger should be narrowed to the first construct that actually needs subgroup-scoped *memory visibility* — which is a staged handoff between simdgroups, not a shuffle.

**Fact.** `ReductionTopology` has five variants — `None`, `Serial`, `MultiPass`, `Contraction`, `CooperativeWorkgroup` — and none is a subgroup topology. It is `#[non_exhaustive]` under ADR 0074 convention 5a, so a sixth lands additively. `ExecutionBinding` has one variant, `GlobalLinearInvocation`; `TailPolicy` has one variant, `Exact`; both are `#[non_exhaustive]` and `TailPolicy` carries a `compile_fail,E0004` doctest proving it.

**Fact.** `CooperativeWorkgroup` carries `arrival: ContributorArrival` (`crates/tiler-ir/src/schedule/model.rs`), whose admitted value `AscendingParticipant` "consumes reassociation alone, while an arrival the program does not fix reorders the contributors themselves". `ContributorArrival::requires_permutation` is an exhaustive match "so a widened vocabulary must decide this rather than inherit the deterministic answer". **This field is the precedent the subgroup tier's tree-order question resolves into, and it landed after the CPU lane-tier record was written, which is why that record does not mention it.**

**Fact.** `LocalCoordinateSource` has one variant, `LocalLinearInvocation` — "the linear index of one invocation within its own workgroup". There is no subgroup-lane coordinate source.

**Fact.** `RouteResourceDimension` (`crates/tiler-artifact/src/program/requirement.rs`) has exactly one variant, `SubgroupThreads`, documented as "Threads one subgroup must execute in lockstep for the route to be correct", and is deliberately **not** `#[non_exhaustive]` under ADR 0074 convention 5b so that "a dimension added here has to be a build failure at each adapter". Its satisfaction test is `is_satisfied_by(observed) = self.minimum <= observed` — a floor. Every implemented adapter answers it `LiveDeviceObservation::Unrecognized`, which refuses the route.

**Fact.** There is no subgroup width anywhere in target feasibility. `CapabilityAxis` (`crates/tiler-compiler/src/target/feasibility.rs`) is `GridAxisThreads`, `WorkgroupThreads`, `BufferBindings`, `DeviceAddressSpace`, `LocalMemoryBytes`, `IndexArithmeticU64`, `DeviceAddressWidthBits` — no width axis — and [the Metal compile-profile authority ledger](../target-profiles/first-macos-metal-compile-profile-authority-ledger.md) declares no subgroup row. `crates/tiler-metal/src/target.rs` states the reason: "Prepared-pipeline facts such as `maxTotalThreadsPerThreadgroup`, `threadExecutionWidth`, and `staticThreadgroupMemoryLength` are deliberately absent."

**Inference.** Every construct this design needs is an additive widening of a seam built to be widened, with one exception that is a build error at every adapter by design (`RouteResourceDimension`) and one that does not exist in any form (a subgroup width in feasibility). None of the four is this record's to land.

## 1. `SubgroupTree` is a `ReductionTopology` sibling, not a construct inside a cooperative phase

**Elimination.** *A subgroup combine inside a `CooperativePhase`, one per staged slot* is eliminated on three independent grounds, any one sufficient.

First, **the construct would be almost entirely inapplicable fields**. A subgroup-only reduction allocates no workgroup memory, so its `CooperativeTile` would carry an empty `staging` vector, phases with no `writes` and no `reads`, and — by `CooperativeTile::visibility_edges`'s own derivation — no edges. `required_subject` returns `None` on an empty edge set, so the tile could declare no point; `verify_cooperative_tile`'s machinery for slot disjointness, coverage, lifetime, and discharge would range over nothing. What would remain of the tile is `coordinates` and `commit`, and a construct reduced to two of its eight fields is not the construct.

Second, **identity**. `CanonicalScheduledRegionIdentity` is a pure function of normalized schedule content. Two subgroup trees at different widths have different depths, different combine steps, and different derived requirements — they are two programs. Under this candidate the width would have to live on the tile, which is the workgroup-shared-memory construct, so a fact about the register file would be encoded inside the staging authority. The domain-separator discipline exists to make identity injective; this would make one construct the identity carrier for a dependency it has no relation to.

Third, **the empty tile would have to be exempted from the rules that make a tile safe**. `SynchronizationRule::SingleParticipant` refuses a tile whose participants number fewer than two because "a single-participant tile stages values it reads back itself". A no-staging tile is the degenerate case that rule generalizes, and admitting it means admitting a tile whose entire safety argument is vacuous.

*A subgroup tree with no schedule construct at all, chosen by the backend* is eliminated by the adopted model's own refusal of exactly this shape: "An opaque `block_reduce` is insufficient because it does not determine numerical order, visibility, barriers, or resource use." A backend choosing its own tree would choose its stride order, and §4 below shows the stride order decides which numerical permission the region consumes. That is meaning, not emission.

**What survives** is a sibling variant of `ReductionTopology`, structurally parallel to `CooperativeWorkgroup` and differing in exactly the fields the transfer medium changes:

```text
SubgroupTree {
    partition:              ContributorPartition,   // partitions == width
    width:                  SubgroupWidth,          // a literal, see §3
    combine:                CombineTree,            // ordered steps, see §4
    lane_identity_bits:     u32,                    // see §2, never empty_identity_bits
    result_lane:            u64,
    axes:                   Vec<Axis>,
    order:                  ContributorOrder,
    accumulation:           ArithmeticType,
    permits_reassociation:  bool,
    permits_permutation:    bool,
}
```

It carries no `tile`, no `synchronization`, and no staging, because it has no handoff through memory. It carries `commit` implicitly as `result_lane`, which is the same fact `CooperativeTile::commit` states — "which of them stores is the fact that makes the proof true" — spelled for a construct with one lane rather than a participant range.

**And the adopted model's sketch is carried with one correction, stated rather than silently applied.** [The scheduled-region model](scheduled-region-model.md) names `SubgroupTree(contributors, combine_steps, result_lane, masked_identity)`. Three of the four survive under the same meaning. **`masked_identity` does not, and the name is the defect**: §2 derives that no lane is ever masked off, and that the field's real referent is the value a *fully participating* lane holds when it owns no contributors. The field is retained and renamed; the sketch's arity and intent are otherwise preserved.

**What this answer does not cover, stated because a reader will assume it does.** A two-level reduction — several subgroups reducing internally, their leaders staging partials through workgroup memory, one final combine — is the shape the Metal specification's own example kernel uses and the shape a realistic softmax needs. It is neither a `SubgroupTree` nor a `CooperativeWorkgroup`: the adopted model states that "each reduction domain has exactly one topology", and this composition needs a hierarchical partition whose outer level stages and whose inner level shuffles. **It is genuinely a third structure and it is out of this ticket's five questions.** Filed as [`compose-the-two-level-subgroup-and-workgroup-reduction`](../../../tickets/compose-the-two-level-subgroup-and-workgroup-reduction.md) rather than absorbed silently, because absorbing it would mean inventing a nesting vocabulary under a ticket that asked for a tier.

## 2. There is no subgroup visibility edge, and two different obligations replace it

**Elimination.** *A `VisibilityEdge` over a register* is eliminated because `FencedSpaces` has no flag that could be true. Its two flags are `workgroup` and `device`, and the type's own doc states why there are no others: "Invocation-private and constant memory have no flag at all — neither is shared between invocations, so fencing one is not a statement about visibility." A register is invocation-private. Adding a `subgroup` flag would assert that a lane's register is a shared memory domain, which is exactly the thing it is not — the value moves because an instruction moved it, not because a fence published it.

*A `SynchronizationPoint` with an empty fence* is eliminated by `SynchronizationRule::FencedSpaces` and by `required_subject`'s own comment: a point whose fence names no domain is "a requirement stated over nothing", and `SynchronizationRule::RedundantPoint` separately refuses "a point that discharges nothing" because "admitting it would let a schedule consume a target authority for an operation it has no reason to perform."

**What survives is that the combine step owns both obligations, and they are not visibility obligations.**

### Obligation one: source-lane activity

**Fact — MSL Specification 4.1, §6.10.2, page 217.** "An active thread is a thread that is executing. An inactive thread is a thread that is not executing. For example, a thread may not be active due to flow control or when a task has insufficient work to fill the group. **A thread needs to only read data from another active thread in the SIMD-group.**"

**Fact — CUDA Programming Guide, "Warp Shuffle Functions".** "Warp shuffle functions exchange a value between non-exited threads within a warp without the use of shared memory." And: "**Threads may only read data from another thread that is actively participating in the intrinsics. If the target thread is inactive, the retrieved value is undefined.**"

**Fact — WGSL specification, §17.12.12.3 `subgroupShuffleXor`.** "Returns `e` from the invocation whose subgroup invocation ID matches `subgroup_invocation_id ^ mask` for the current invocation. … **An indeterminate value is returned if `mask` does not select an active invocation** or if `mask` is not a uniform value within the subgroup." The `subgroupShuffleDown` and `subgroupShuffleUp` entries carry the identical clause over `subgroup_invocation_id ± delta`.

**Inference.** Three independent vendor specifications state one obligation in three vocabularies — "needs to only read", "may only read … undefined", "an indeterminate value is returned". It is portable, it is a requirement on the *program* rather than a guarantee from the machine, and no target declares it. That makes it structurally the same class of fact as `ConvergenceEvidence`, whose doc says convergence "is a property of the *program*, so no target declares it" — and it should be carried the same way, as an evidence class the verifier re-derives rather than a claim it accepts.

**And the obligation is not vacuous, because a subgroup is not guaranteed to be full.** **Fact — MSL 4.1, §4.4.1.** "all SIMD-groups within a threadgroup needs to be the same size, apart from the SIMD-group with the maximum index, which may be smaller, if the size of the threadgroup is not evenly divisible by the size of the SIMD-groups." **Fact — WGSL §15.5.** "Each subgroup may contain fewer invocations than the reported subgroup size (e.g. if fewer invocations than the subgroup size are launched). … **When the subgroup size exceeds the number of invocations in a subgroup, the extra hypothetical invocations are considered inactive.**"

**Inference, and it is a hard derived launch requirement rather than a preference.** A width-`W` tree reads lane `l ^ m` for every active `l` and every mask `m < W`, which ranges over all of `0..W`. If any lane of `0..W` is not launched, it is inactive, and reading it is undefined on CUDA, indeterminate on WebGPU, and outside what MSL permits. **So a subgroup tree derives the requirement that the threadgroup size be an exact multiple of the subgroup width**, so that no partial trailing subgroup exists. That is an intrinsic, target-independent obligation given the literal width, and it is checkable — which is one more reason §3 refuses a symbolic width.

### Obligation two: every lane holds a value, and a contributor-free lane holds the identity

Derived in full in §4 and instantiated at exact bits in Worked example A. The short form: because the width is imposed rather than chosen, a contributor sequence that does not divide by `W` cannot be declined the way the workgroup tier declines it, so some lanes own no real contributor and must still enter the tree with a value that leaves the result unchanged.

### The `Collective` kind stays refused, and this design strengthens the refusal

The ticket anticipated that this tier would activate `SynchronizationKind::Collective`. **It does not, and the primary sources are why.**

**Fact — MSL 4.1, Table 6.15.** `T simd_sum(T data)` — "Returns the sum of the input values in `data` across all active threads in the SIMD-group and broadcasts the result to all active threads in the SIMD-group." **No combine order, no accumulation width, and no rounding sequence is stated.**

**Fact — WGSL §15.7.4, floating-point accuracy table.** `subgroupAdd(x)` is specified as "**Inherited from** sum of `x` for all active invocations in the subgroup" — an inherited, unbounded error rather than a defined result — while `subgroupShuffle(x, id)`, `subgroupShuffleDown`, `subgroupShuffleUp`, and `subgroupShuffleXor` are each specified as "**Correctly rounded**".

**Inference, and it is the cleanest result in this record.** The two vendor vocabularies agree exactly: a shuffle is an exactly specified value move, and a subgroup reduction collective is a value whose combine order the vendor declines to state. `SynchronizationKind::Collective`'s refusal reason — "it carries a combine order and a numerical realization of its own, which no field here states" — is therefore not a modelling inconvenience but a description of the primitive. **Building the tree out of shuffles and ordinary adds is what makes the tier statable at all**, and reaching for `simd_sum` would be the `block_reduce` opacity the adopted model refuses by name, now with two specifications confirming that the opacity is real rather than assumed.

**The subject shape `Collective` would need, stated without admitting it.** It would need the five dimensions of `SynchronizationSubject` plus three the subject does not have: a combine order, an accumulation `ArithmeticType`, and an inactive-lane rule. All three already live on the reduction topology, so admitting `Collective` would put one region's numerical realization in two places that could disagree — which is the defect `ContributorArrival` exists to prevent, transferred to a construct that cannot check it. **The refusal should be widened rather than lifted**: `SynchronizationKind::Collective`'s doc comment currently gives one reason, and it earns a second — no vendor states the order, so no schedule could state the order it required.

## 3. The width is a literal in the schedule and an equality against the target

**A premise this ticket was dispatched with is wrong and is corrected here first.** The brief stated that "Metal simdgroups are 32 by spec on Apple GPUs". **Fact — established by reading rather than by a failed search.** The Metal Shading Language Specification 4.1 states no numeric SIMD-group width anywhere. It defines `[[threads_per_simdgroup]]` as "The thread execution width of a SIMD-group (compute unit)" — a value read at run time — states `simdgroups_per_threadgroup = ceil(threads_per_threadgroup / threads_per_simdgroup)`, says `[[thread_index_in_simdgroup]]` "is a value between 0 and SIMD-group size –1", and fixes a number in exactly one place, for a different construct: "A quad-group is a SIMD-group with the thread execution width of 4." The exact check: `grep -n "SIMD-group size"` over the extracted text of the specification returns two hits, both symbolic, and no line pairs `simdgroup` with a literal `32`. The value 32 for Apple GPUs is an observation of `threadExecutionWidth` and a Metal Feature Set Tables fact, not a language-specification guarantee, and a design that treated it as the latter would be declaring a row it cannot license.

**The three platforms differ in the *authority* over the width, not merely in its value.**

**Fact — CUDA Programming Guide.** "`int warpSize`: A run-time value defined as the number of threads in a warp, commonly 32." The hardware description separately says "Each SM creates, manages, schedules, and executes threads in groups of 32 parallel threads called *warps*." The shuffle intrinsics take `int width = warpSize`, and "width must be a power of two in the range [1, warpSize], namely 1, 2, 4, 8, 16, or 32."

**Fact — WGSL §15.5.** "The subgroup size is the maximum number of invocations in a subgroup. … The subgroup size is a uniform value within a dispatch command and hence within a workgroup, but it may not be a uniform value within a draw command. **All subgroup sizes are powers of two within the range [4, 128]**, and the value for a shader compiled for a specific device will be within the range [`subgroupMinSize`, `subgroupMaxSize`] for the WebGPU GPUAdapter. **The actual size depends on the shader, device properties, and the device compiler.** Each device supports a subset of the possible range of subgroup sizes (possibly a single value). **The device compiler selects a size from the supported sizes using a variety of heuristics.**"

**Inference — the CPU tier's fixed-versus-scalable split is a false friend here, and the ticket's framing invited the wrong analogy.** `FixedVectorLane(4)` against `ScalableVectorLane` is a choice between two program forms the schedule makes. A subgroup width is not chosen by the schedule at all: on Metal it is the compute unit's execution width, on WebGPU it is picked by the *device compiler* per shader, on CUDA it is the architecture's. The question is therefore not "does the schedule write a literal or a symbol" but "**what does the schedule do about a number it does not control**", and that has a different answer from either CPU binding.

**Elimination.** *Width-symbolic, resolved at feasibility* is eliminated on two independent grounds.

First, the combine steps are the *content* of the topology, not a derived consequence of it. A butterfly at width 32 has five steps with masks `1, 2, 4, 8, 16`; at width 64 it has six. §4 shows the step list determines which numerical permissions the region consumes. With `W` symbolic the step list is symbolic, so "the combine tree satisfies numerical permissions" — an intrinsic-verification obligation in the adopted model's own list — cannot be discharged at intrinsic verification for any contract. This is the same shape as the CPU tier's refusal of a scalable contributor partition, and it is stronger: there the undischargeable obligation was coverage, here it is *numerical legality*.

Second, identity, by the argument the CPU record makes and which transfers verbatim: two widths are two programs with different derived requirements, and a symbolic width would give them one `CanonicalScheduledRegionIdentity`, so a cache would hit across them.

*Width absent from the schedule, read from `[[threads_per_simdgroup]]` in the emitted body* is eliminated by the same layering argument and by a concrete consequence: the Metal specification's own example kernel does exactly this (`for (uint offset=simd_size/2; offset>0; offset/=2)`), and the resulting program's reassociation structure is a function of a runtime value. A region whose numerical realization is not determined until dispatch cannot be the region a numerical contract was checked against.

*An atomic realization fact per (kind, width) inside `SynchronizationSubject`* — the ticket's first named candidate — is eliminated because this tier declares no synchronization point at all (§2). There is no subject to extend. The atomicity *argument* transfers and is right; the carrier is not `SynchronizationSubject`.

**What survives: a literal width in the schedule, an atomic subject matched by equality at the compile profile, and a live confirmation before routing commit.** The three are not redundant — they answer three different questions, and the middle one is what lets the optimizer's feasibility surface answer without backend code.

| Stage | Phase | Question | Verdict on failure |
| --- | --- | --- | --- |
| Intrinsic verification | none (target-independent) | Do the combine steps cover the partition once each, is the leaf order the declared one, does the threadgroup size divide by `W`, is the identity a two-sided identity? | Hard schedule rejection |
| Target feasibility | `CompileProfile` | Does the profile declare a `SubgroupRealization` subject *equal* to the one this schedule requires? | `Rejected` if a differing fact is declared; `Unknown` if none is, which under ADR 0043 cannot enter an executable frontier |
| Preflight | `PreparedKernelPreflight` | Does the prepared pipeline's `threadExecutionWidth` equal the declared width? | Hard rejection before routing commit; never a fallback after |

**The atomic subject, and why it is one value.** The argument is `DeclaredSynchronizationRealization`'s, transferred: "each of its five dimensions is separately true of some realization on some machine … so a profile declaring them independently would let their conjunction be inferred from facts none of which is about it." A subgroup realization has the same structure. A machine with a 32-wide subgroup, a machine that provides an XOR-indexed shuffle, and a machine whose out-of-range shuffle behaviour is defined are three true statements whose conjunction is not a statement about any of them — and the third is not hypothetical:

**Fact — three specifications, three different answers for one out-of-range shuffle.** MSL: `simd_shuffle_down` "doesn't modify the upper `delta` lanes of `data` because it doesn't wrap values around the SIMD-group", and the specification's own worked table shows lanes whose computed source is out of range keeping their own value. CUDA `__shfl_sync`: "If `srcLane` is outside the range `[0, width - 1]`, the result corresponds to the value held by the `srcLane % width`" — it wraps. WGSL `subgroupShuffle`: out-of-range `id` is "a shader-creation error if `id` is a const-expression", "a pipeline-creation error if `id` is an override-expression", and otherwise "an indeterminate value is returned". **A schedule relying on out-of-range shuffle behaviour is target-specific by construction**, which is a second, independent reason §4 selects the butterfly: `l ^ m` for `m < W` is never out of range on any of the three.

**And the existing route-requirement channel is not the right carrier as it stands.** `RouteResourceDimension::SubgroupThreads` is a *floor* — `is_satisfied_by(observed) = self.minimum <= observed` — documented as "Threads one subgroup must execute in lockstep for the route to be correct." Two problems, and they are separable.

The stated property is refuted. **Fact — CUDA Programming Guide.** "In GPUs of compute capability 7.0 and later, *independent thread scheduling* allows full concurrency between threads, regardless of warp," and "*Warp-synchronous* code assumes that threads in the same warp execute in lockstep at every instruction, but the ability for threads to diverge and reconverge at sub-warp granularity makes such assumptions invalid." **Lockstep within a subgroup is not a property any current GPU family guarantees**, so a floor over "threads that execute in lockstep" bounds a quantity that no adapter can soundly observe — which is consistent with every implemented adapter answering `Unrecognized`, though they answer it for the different reason that Metal publishes no device-scoped width.

The relation is also wrong for this tier, and the CPU record's argument is the one that applies. That record establishes that "a lane width looks quantitative and is not", because the ordering relation a capability axis would give it is unsound. Here the floor's unsoundness is narrower than the CPU case but real: a width-`W` tree on a wider device is sound *only if* lanes `0..W` of each subgroup are all active, and that conjunct is precisely what a floor does not carry. What the tier needs is an equality on the width together with the full-participation obligation §2 derives — not a bound. **Filed as [`correct-the-subgroup-threads-route-dimension-meaning`](../../../tickets/correct-the-subgroup-threads-route-dimension-meaning.md)**; it is a live defect in landed public vocabulary and is independent of whether this design is accepted.

## 4. A shuffle tree consumes reassociation; whether it also consumes permutation is the stride order's answer

### The tree is a reassociation by construction

**Derivation.** Lane `l` folds the contiguous contributor block its partition owns, in the declared `ContributorOrder`, and the tree combines the `W` per-lane partials. That is a `ContributorPartition { partitions: W, contributors_per_partition: k }` and nothing else, standing to the fold exactly as `MultiPass` and `CooperativeWorkgroup` do. The existing rule transfers verbatim: admitted only when `permits_reassociation` holds, checked at the same place and by the same check `verify_cooperative_semantics` applies.

### The stride order decides permutation, and this is the tier's sharpest practical result

`ContributorPartition`'s contract is what makes a split "a *reassociation* of the declared contributor sequence and never a permutation of it": partition `p` covers a contiguous range and the partials combine in ascending `p`. **Read the combine tree's leaves left to right; if they are the declared sequence, only the parenthesization moved.** Two shuffle trees give two different answers to that test.

**The ascending-mask butterfly consumes reassociation alone.** With `simd_shuffle_xor` at masks `1, 2, 4, …, W/2` in ascending order, mask `1` pairs lane `l` with `l ^ 1` — adjacent lanes — mask `2` pairs the resulting `(0,1)` group with the `(2,3)` group, and so on. At `W = 8` the expression at every lane is `((b0+b1)+(b2+b3))+((b4+b5)+(b6+b7))`: a balanced binary tree whose leaves read left to right are `b0 … b7`, ascending. Nothing crossed a group boundary.

**The descending-stride tree additionally consumes permutation.** **Fact — MSL 4.1, §6.10.2.1**, the specification's own reduction example: `for (uint offset=simd_size/2; offset>0; offset/=2) val += simd_shuffle_down(val, offset);`. At `W = 32` the strides are `16, 8, 4, 2, 1`; lane 0's accumulator after the first step is `b0 + b16`, after the second `(b0+b16)+(b8+b24)`, and its final leaf order is `0, 16, 8, 24, 4, 20, 12, 28, 2, 18, …` — the bit-reversal permutation of `0..31`. `b16` has moved across fifteen contributors to sit beside `b0`, which is a reordering of the operand sequence and not a regrouping of it.

**And the adopted model already reached this conclusion for the staged form of the same tree, which corroborates the derivation from inside the repository.** Its row-reduction example uses "combine strides = [128,64,32,16,8,4,2,1]" and states: "This schedule is rejected unless the numerical contract permits both the resulting reassociation and operand permutation." The model does not say the ascending direction exists, and that is the gap this record closes.

**Inference, and it is the mirror image of the CPU tier's finding.** The CPU record derived that on a permutation-forbidding contract "the permission structure forces the numerically legal layout to be the cache-hostile one", because the contiguous-block assignment needs a gather. **Here the numerically legal tree is free**: the ascending butterfly and the descending `shuffle_down` tree have the same depth, the same instruction count, and the same operand count. Under the common contract — `reassociation: Permitted`, `permutation: Forbidden`, which the CPU record establishes is the realistic shape because the dimensions are independent — the ascending butterfly is admissible and the idiom every vendor prints is not. A planner that copied the vendor example would be pricing meaning for nothing.

**One detail the derivation needs and gets for free.** In a butterfly every lane computes `acc + shuffle_xor(acc, m)`, so lane `0` computes `b0 + b1` while lane `1` computes `b1 + b0`. IEEE 754 addition is commutative — including for signed zeros, where clause 6.3's rule for an exact zero sum is symmetric in the operands — so all lanes hold bit-identical values at every step, and the all-reduce result needs no separate broadcast. The `canonical_nan_bits` the region already carries covers the one place commutativity could be observed differently.

**So the tree order is a stated field and never inferred**, exactly as `ContributorArrival` is on `CooperativeWorkgroup`, and for the identical reason its doc gives: "a strategy that checked reassociation and then used both would be admitted for a freedom nobody granted."

### Identity injection consumes no numerical permission, and that is why it must be a proof obligation

**Elimination.** *Masked-identity injection requires the permutation permission* — the ticket's explicit sub-question — is **eliminated**. Padding extends the contributor sequence at its end; it does not move any contributor past any other. Under contiguous-block assignment lane `l` owns block `l`, the real contributors occupy positions `0..C` in ascending order, the padding occupies `C..W·k`, and the leaf order of the padded tree is ascending throughout. No leaf crossed another.

*Identity injection is a third numerical permission* is eliminated for a sharper reason. If the injected value is a two-sided identity of the combine operation under the contract's own signed-zero permission, the padded fold's result is **bit-identical** to the unpadded one — nothing was approximated, so there is no freedom to grant. If it is not an identity, the schedule computes a different value, which is a defect and not an under-permission. **A permission bit would be the wrong shape for a property that is either exactly true or a bug**, which is precisely why the value must be a stated field carrying a checked obligation rather than a flag or a derivation.

**And the derivation would be right half the time, which is the worst failure profile.** The CPU record's argument transfers unchanged and is instantiated at exact bits in Worked example A: under `signed_zero: Permitted` both zeros work and the choice is free; under `signed_zero: Forbidden` only `-0.0` works. A rule deriving the padding value from `empty_identity_bits` would be correct on every permissive contract — the ones a first implementation tests — and silently wrong on the stricter one it was never exercised against.

**The obligation is heavier at this tier than at the CPU tier, and the reason is structural.** The CPU tier reaches identity padding only when it chooses a width that fails to divide, and a planner may choose another. **The subgroup tier cannot choose**, so `lane_identity_bits` is not an optional field on an optional policy — it is required whenever `W ∤ C`, which is the general case. §5 of the CPU record could leave `IdentityPadded` as one of four tail policies; here it is the only one.

## 5. Why the CPU vector lane and the subgroup lane stay separate

**Fact — ADR 0043, accepted text.** "Execution and vector models are explicit. GPU workgroup/subgroup scopes are not aliases for CPU worker/vector scopes." The adopted scheduled-region model states the same refusal at more length — "Subgroup lanes and per-thread vector lanes are different bindings" and "It does not equate vector lanes with subgroup lanes" — but the decided form is the ADR's, and it makes conflating them a violation rather than a departure from a proposal. This record therefore does not argue *that* they differ; it derives the consequence the ticket asks for, which is what a shared combine-tree shape would cost.

**The decisive argument is that the two tiers cover disjoint parts of the design space, so a shared shape would range over a union nobody needs.** The CPU record's headline result is that a lane may bind the **map** — one lane per output coordinate — and that this consumes no numerical permission, making a strict-order fold vectorizable. **That result has no subgroup analogue that needs a construct**: a GPU already assigns one invocation per output coordinate, and `ExecutionBinding::GlobalLinearInvocation` covers it with no new binding, no new topology, and no new permission. Conversely the CPU record refuses a scalable lane over the partition and treats a fixed lane over the partition as one of several options, while **the subgroup tier exists only for the partition case**. The two tiers meet at exactly one construct — a contributor partition combined by a tree — and differ in every obligation attached to it.

**Three differences, each of which would become a field with two meanings.**

1. **Width authority.** A CPU fixed vector width is chosen by the schedule and settles at `CompileProfile` with no preflight, which is [the CPU profile record](../target-profiles/cpu-vector-realization-facts.md)'s stated negative result. A subgroup width is imposed by the device and, on Metal, is a prepared-pipeline property that no compile-time profile can settle (§3). One field would carry two provenance stories and two availability phases.

2. **Transfer and its failure mode.** A CPU horizontal step moves data between lanes of *one invocation's* register, where there is no such thing as an inactive lane — only a masked one, and the mask is program state the schedule wrote. A shuffle moves data between *two invocations'* registers, and the source invocation may not be executing at all, which all three vendor specifications call undefined or indeterminate (§2). The CPU tier has no obligation of this class and no place to put one.

3. **Ownership.** In a subgroup tree, `OneGlobalInvocationPerOutput` holds with the *committing lane* as the invocation and `W-1` other invocations performing work that never writes — the same shape `CooperativeTile::commit` states. In the CPU lane-over-map case every lane is an output owner. The ownership proofs are different propositions, and a shared construct would have to hold both.

**Consequence, stated as the ticket asks.** A shared "combine tree" shape is possible only by making width authority, activity, and ownership all optional, at which point the type stops constraining anything and the verifier can no longer tell which obligations to discharge. **What may be shared is narrower and is worth sharing: the tree-order vocabulary itself** — ascending versus descending stride, and the leaf-order test that decides permutation — is one derivation with one answer, and it should be one named concept used by both tiers rather than two enums that could drift. That is a vocabulary-sharing claim, not a construct-sharing one, and it is the only one this record makes.

## 6. The log-depth question: registers dissolve slot addressing and raise a different obligation

This is the question the dispatch brief flagged as probably the sharpest, and it is.

**The workgroup tier's refusal, restated exactly.** `workgroup_tree_tile`'s doc derives that a logarithmic tree "narrows the *writing* lanes at every round, and this vocabulary cannot state that: a `StagedSpan` is addressed by every participant of the tile — the slot enumeration runs over the tile's whole participant range, not over a per-access subset — so every write phase writes exactly `participants * count` slots however few lanes are meant to be doing useful work." Rewriting one slot across rounds is refused by the one-writer-per-slot rule; writing fresh slots per round does not narrow. **So a log-depth tree needs a per-access active-participant subset, distinct from a phase's `participation`, which is arrival and must stay uniform for the point between rounds to be convergent.**

**Does a shuffle tree inherit that obligation?** A shuffle tree is exactly a log-depth structure — five rounds at width 32 — so the question is not rhetorical.

**Derivation, and the answer is no, for a reason that is specific rather than convenient.** The construct the workgroup tier lacks is a way to say *which participants touch which slots at one access*. That question is generated by `StagedSpan`, whose whole content is a map from participant to slot addresses, quantified over the participant range. A combine step has no slot: its source is the per-lane function `src(l) = l ^ m` and its destination is lane `l`'s own register. **Disjointness of destinations is therefore total and free at every round** — each lane writes exactly one location, its own, and no two lanes share one — so the one-writer-per-slot rule has nothing to check and the coverage enumeration has nothing to enumerate. The question "which participants are active for this access" never becomes "which slots does this access touch", because there are no slots. **Register-carried values do dissolve the slot-addressing problem entirely.**

**What replaces it is not the same construct under another name, and the distinction matters.** The surviving obligation is on the *reader's source*, not on the writer's extent: lane `l` must not read a lane that is inactive. That is a different proposition with a different quantifier — it ranges over the lanes named by `src`, not over the lanes performing the access — and it has a different discharge.

**And the ascending-mask butterfly discharges it by construction, which is the reason to prefer it a third time.** In a butterfly every lane is active at every round and every lane's source `l ^ m` for `m < W` is another lane of the same subgroup, which is active by the same argument. Participation never narrows, so there is nothing to state a subset over. **The narrowing tree does not get this for free**: a `shuffle_down` reduce-to-lane-0 leaves lanes above `W/2^{r+1}` computing values nobody reads, and if a backend guards that with a branch, those lanes become inactive and the *next* round's sources are undefined. The vendor idiom avoids this only by leaving every lane unguarded and discarding the results — which is correct, and is exactly the fact that its "narrowing" is a data-flow property rather than a participation one.

**So the answer to the sixth question, in one line: the subgroup tier sidesteps the per-access active-participant subset entirely, because a shuffle's destination is the reader's own register and its source is a lane index rather than a slot address; the obligation that replaces it is source-lane activity, and the ascending butterfly is the topology that discharges it without any new construct.** A narrowing tree would need one — an active-lane relation per combine step — and that is the reason this record admits the butterfly and defers the narrowing form rather than admitting both.

## Worked example A — a shuffle tree at width 32 with a tail

The program is the CPU record's, unchanged, so the two tiers can be read against each other:

```text
x : f32[7, 101]
y : f32[7]
y[m] = strict_serial_sum(n, x[m, n])      0 <= m < 7, 0 <= n < 101
```

**The region**, in the implemented vocabulary: one `ReductionContributor` read with `input_shape = [7,101]`, `output_shape = [7]`, `axes = [1]`, `order = OriginalAxisLexicographic`; `ScalarProgram::StrictSerialSum { axes: [1], order: OriginalAxisLexicographic, canonical_nan_bits: 0x7fc0_0000, empty_identity_bits: 0x0000_0000 }`.

**The contract** is `reassociation: Permitted`, `permutation: Forbidden`, `signed_zero: Forbidden`, everything else as the strict workspace contract. This is the realistic shape and it is the one that discriminates: a reassociation-forbidding contract refuses every split topology at the first check, so it would test nothing this tier adds.

**The schedule.** One simdgroup owns one output row. `W = 32`, `C = 101`, `101 = 3·32 + 5`, so `ContributorPartition { partitions: 32, contributors_per_partition: k }` requires `32k = 101`, which has no integer solution. `ceil(101/32) = 4`, so the padded sequence is `W·k = 128` contributors and lane `l` owns the contiguous block `[4l, 4l+4)`.

| Lanes | Block | Real contributors | Padding |
| --- | --- | --- | --- |
| 0 – 24 | `[0,4)` … `[96,100)` | 100 (all of `0..100`) | 0 |
| 25 | `[100,104)` | 1 (contributor `100`) | 3 |
| 26 – 31 | `[104,128)` | 0 | 24 |

Twenty-seven padding contributors, and **six lanes own no real contributor at all**. Those six are the case the adopted model's `masked_identity` was named for, and they are the case that shows the name is wrong: they are not masked off. They execute every instruction, they are active at all five combine steps, and their partial enters the tree — because §2 established that if they were inactive, the lanes reading them would receive an undefined value. **A contributor-free lane is fully participating and identity-valued, which is the opposite of masked.**

**The combine tree** is the ascending butterfly: five steps at masks `1, 2, 4, 8, 16`, `result_lane = 0`. Leaf order left to right is the padded contributor sequence `0 … 127` ascending, so the tree is a reassociation and consumes no permutation (§4).

| Obligation | Discharge |
| --- | --- |
| Reassociation | **Consumed.** `partitions = 32` splits one output's contributor sequence; admitted only because the contract grants it. |
| Permutation | **Not consumed**, because the mask order is ascending. The same tree at descending strides would consume it and be **rejected** under this contract — same program, same width, same instruction count, different verdict. |
| Coverage | `32 × 4 = 128 = C + 27` exactly; every real contributor appears once, every padding position once. |
| Ownership | Lane 0 is the sole writer of `y[m]`; the other 31 invocations write nothing. |
| Source-lane activity | Every step reads `l ^ m` with `m ≤ 16 < 32`, always another lane of the same subgroup, always active — provided the launch obligation below holds. |
| Launch | **Derived, and it is the tier's new hard requirement.** `threads_per_threadgroup` must be an exact multiple of 32, so no trailing partial simdgroup exists (MSL §4.4.1; WGSL §15.5). |
| Identity | **The binding obligation, derived at bits below.** |

### The identity, at exact bits

**Fact — IEEE Std 754-2019, clause 6.3.** "When the sum of two operands with opposite signs (or the difference of two operands with like signs) is exactly zero, the sign of that sum (or difference) shall be +0 under all rounding-direction attributes except roundTowardNegative … However, under all rounding-direction attributes, when x is zero, x + x and x − (−x) have the sign of x."

Under `roundTiesToEven`, which is the only rounding Tiler admits, this makes `x + (-0.0) = x` for every non-NaN `x` — `(-0.0) + (-0.0) = -0.0` by the second sentence and `(+0.0) + (-0.0) = +0.0` by the first — while `+0.0` is **not** a two-sided identity, because `(-0.0) + (+0.0) = +0.0`.

**Take the row `m` whose 101 contributors are all `-0.0`**, which is reachable from ordinary data. Its true strict-fold sum is `-0.0` = `0x8000_0000`.

With `lane_identity_bits = 0x8000_0000` (`-0.0`):

- Lanes 0–24 each fold four `-0.0` values → `-0.0`.
- Lane 25 folds `b100 + (-0.0) + (-0.0) + (-0.0)` = `(-0.0)` → `-0.0`.
- Lanes 26–31 fold four padding values → `-0.0`.
- Every butterfly step adds `-0.0` to `-0.0` → `-0.0`. **Result `0x8000_0000`. Correct.**

With `lane_identity_bits = 0x0000_0000` (`+0.0`, which is exactly what `empty_identity_bits` holds):

- Lanes 0–24 → `-0.0`, unchanged.
- Lane 25 folds `(-0.0) + (+0.0)` → `+0.0`. **Poisoned at the first padding step.**
- Lanes 26–31 fold `(+0.0) + (+0.0) + (+0.0) + (+0.0)` → `+0.0`. **Seven of thirty-two partials are wrong.**
- The butterfly propagates `+0.0` into every lane. **Result `0x0000_0000`. Wrong by one bit, on a value the contract says is observable.**

**So `empty_identity_bits` is the wrong value and it is the one an implementer would reach for**, because it is on the scalar program and is named "identity". The two have different roles: `empty_identity_bits` is committed when there are *no* contributors; a padding identity is combined *into* a nonempty fold. **Fact.** The schedule verifier currently *requires* `empty_identity_bits == 0.0_f32.to_bits()` at each of the serial, multi-pass, and cooperative arms (`crates/tiler-ir/src/schedule/builder.rs`), so the wrong value is not merely available — it is the only one the region is permitted to carry.

**And the tier makes the trap unavoidable rather than optional.** The CPU tier hits this question only when a planner chooses a non-dividing width. Here `W = 32` and `C = 101`, and neither is the planner's to change.

## Worked example B — the same tile at the workgroup tier

Identical program, identical contract. The comparison the ticket asks for is not "which is faster"; it is what the two forms cost and oblige.

**The workgroup tier cannot state this reduction at 32 participants at all.** `ContributorPartition::covers` requires `partitions × contributors_per_partition == contributors` exactly, and `workgroup_tree_tile`'s doc states the consequence as the strategy's own contract: "the split must cover the contributor sequence exactly once each, so a contributor count with no exact split of `participants` partitions is *declined by the strategy that chooses the count*, never padded with identity elements or truncated. A masked tail lane would additionally break the emitted body's soundness argument, which rests on every launched invocation reaching the staged store."

**So the strategy must choose a participant count dividing 101 — and 101 is prime.** The divisors are 1 and 101; `SynchronizationRule::SingleParticipant` refuses 1. **The only admissible workgroup shape for this program is 101 participants, each folding exactly one contributor.**

| | Subgroup tree (A) | Workgroup tile (B) |
| --- | --- | --- |
| Participants | 32 (imposed) | 101 (forced by primality of `C`) |
| Workgroup memory | **0 bytes** | 101 slots × 4 = **404 bytes**, live across two phases |
| Synchronization points | **0** | 1 control barrier, `Workgroup`/`Workgroup`, `fenced_spaces { workgroup: true, device: false }`, `AcquireRelease` |
| Visibility edges | **0** | 1, discharged by that point |
| Tree depth | 2 levels: 4 serial + 5 butterfly | 2 levels: 1 serial + **101 serial** |
| `f32` additions on the critical path | **9** | **102** |
| Permissions consumed | reassociation | reassociation |
| Identity padding | **required** (27 positions) | **none** — the shape was chosen to avoid it |
| Target facts required | subgroup width equality (compile profile) + `threadExecutionWidth` at preflight | one `SynchronizationSubject` declared `Realized`; local-memory capacity |

**What the subgroup form buys**, read off the table rather than asserted: no workgroup memory, no barrier, no visibility edge, and a critical path an order of magnitude shorter — and the last is not a subgroup-versus-workgroup fact but a consequence of the workgroup tile's depth being two by construction, so its combining level is a `participants`-way *serial* fold. Both forms consume exactly one permission, which is worth stating because the naive expectation is that the shuffle form is the numerically expensive one.

**What it newly obligates**, and these are the price:

1. **The identity, and its proof.** B dodged the question by choosing the participant count; A cannot, so `lane_identity_bits` must be stated and proved a two-sided identity under the contract's signed-zero permission. This is the single largest new correctness surface in the tier.
2. **A width equality against the target, resolved across two phases**, where B needs one atomic synchronization subject settled at the compile profile. A's preflight stage is new and carries routing-commit discipline: a `threadExecutionWidth` mismatch must reject before any encoding, never fall back after.
3. **A launch divisibility requirement.** B's participants are whatever the schedule launched; A requires `threads_per_threadgroup` to be an exact multiple of 32 so that no lane it reads is an unlaunched hypothetical invocation.
4. **Five convergence obligations instead of one.** B proves convergence at one point via `ConvergenceEvidence::EveryParticipantReachesThePoint`; A must establish source-lane activity at each of five combine steps. The butterfly discharges all five by one structural argument, which is why it is the admitted topology rather than one option among several.

**Inference.** B is not a worse A. It is the form available when the width is the schedule's to choose and the contributor sequence divides, and it needs no identity proof, no width equality, and no preflight stage. A is the form available when it does not — which, because `W` is imposed, is the general case. Neither dominates, and a planner needs both.

## Public-boundary items, enumerated for Tom and not self-accepted

Nothing below is implemented, and none of it is accepted by this record's existence.

1. **`ReductionTopology` gains `SubgroupTree { … }`** — a `pub` enum variant in `tiler-ir`, its exact field set, and whether `width` is a newtype or a bare integer.
2. **A `CombineTree` vocabulary naming the step list and its stride order**, and whether ascending and descending are two variants or one variant with an order field. §4 requires only that the distinction be stated rather than inferred; the spelling is a boundary.
3. **`lane_identity_bits` and its proof obligation** — the field's name, whether it is a raw bit pattern or a typed identity witness, and whether the proof that a value is a two-sided identity of the combine operation under the contract's signed-zero permission is a verifier derivation or a declared witness. Shared in substance with the CPU tier's item 4, and the two should land as one concept.
4. **A subgroup-lane `LocalCoordinateSource` variant**, plus the decision that it carries no defined relation to `LocalLinearInvocation` — **Fact — WGSL §15.5**: "There is no defined relationship between subgroup values … and `local_invocation_index`. To avoid non-portable code, shader authors should not assume a particular mapping between these two values."
5. **A `SubgroupRealization` atomic subject and a `declare_subgroup_realization` builder method** on `TargetProfileBuilder`, matched by equality exactly as `declare_synchronization_realization` is, with its own `AvailabilityPhase`.
6. **The `PreparedKernelPreflight` stage for subgroup width**, which is a new consumer of an existing phase rather than a new phase, and its routing-commit ordering.
7. **`RouteResourceDimension::SubgroupThreads` changing meaning from a lockstep floor to a width equality** — a variant of a deliberately non-`#[non_exhaustive]` enum whose every adapter must answer, so the change is a build error at each. Carried by its own ticket and independent of this design.
8. **Widening `SynchronizationKind::Collective`'s refusal reason** to name the vendor-order opacity, which is a doc-comment change to public text and not a vocabulary change.
9. **The two-level composition's representation**, which this record does not design and which item 1's variant deliberately does not cover.

## Deferrals, each with the evidence that would close it and a trigger

- **The narrowing (`shuffle_down`) tree is not admitted.** §6 shows it needs an active-lane relation per combine step that the butterfly does not, and §4 shows it consumes a permission the butterfly does not. Closes with either a measured case where the butterfly's redundant work costs enough to matter, or an active-lane relation landed for another reason. Trigger: a workload where the all-reduce result is discarded at 31 of 32 lanes *and* the tree is measurably on the critical path.
- **The two-level subgroup-plus-workgroup reduction has no representation.** Filed as [`compose-the-two-level-subgroup-and-workgroup-reduction`](../../../tickets/compose-the-two-level-subgroup-and-workgroup-reduction.md). Closes with a hierarchical partition vocabulary. Trigger: already fired for the attention vertical, whose record states that "a SIMD-group-cooperative row reduction survives; anything wider does not" — the first program needing a row longer than one subgroup fires it concretely.
- **Widths other than a power of two are not considered.** Every derivation here assumes `W` is a power of two, which all three specifications guarantee for the platforms named (WGSL states it normatively; CUDA's `width` parameter is restricted to it; Metal's quad-group is 4 and its SIMD-group width is unstated). A non-power-of-two width has no butterfly. Closes with a target declaring one. Trigger: a fourth platform.
- **`simd_sum` and `subgroupAdd` remain unusable, not merely unused.** Their combine order is unstated by both vendors (§2), so no numerical contract can be checked against them. Closes only if a vendor documents an order, or if a contract is admitted whose permissions are wide enough that any order satisfies it — which is not the same as reassociation plus permutation, because an unstated order may also vary between two runs. Trigger: a vendor specification change.
- **Does the subgroup tier ever need `MemoryScope::Subgroup`?** Not for a shuffle tree (§2), and the deferred ticket's trigger should be narrowed accordingly. It would be needed by a staged handoff *between* simdgroups within a threadgroup — which is the two-level composition. Trigger: that composition, not this tier.
- **Is `threadExecutionWidth` ever knowable earlier than `PreparedKernelPreflight` on Metal?** The adopted feasibility record and `crates/tiler-metal/src/target.rs` both place it on the prepared pipeline. If it proved invariant across pipelines for a device family, the compile-profile row could be authoritative and the preflight stage would become a redundancy check rather than a gate. **This is a measurement, not a derivation**, and it is exactly the kind of claim this record must not make: closes with a bounded experiment reading `threadExecutionWidth` across several pipelines on one device. Trigger: the first Metal subgroup realization ticket.

## Drafted ADR body, written to be landed verbatim

The scope map (`ticketsplease.toml`) routes `docs/decisions/[0-9]*.md` to `contracts/decisions` and both `docs/decisions/README.md` and `docs/research/README.md` to `contracts/navigation`; this record's ticket holds `research/scheduling` and shared `project/tickets` only, so the ADR file and both catalog rows are a guard escape from this branch. [`land-the-subgroup-execution-tier-adr`](../../../tickets/land-the-subgroup-execution-tier-adr.md) carries all three, following the `land-the-cpu-vector-lane-tier-adr`, `land-the-bf16-conversion-and-accumulator-adr`, and `land-the-backend-scoped-route-requirement-answer-adr` precedents. `0092` was the highest ADR at `8252312` and `0093` was free; take the next free number by reading the directory rather than by remembering one, since a sibling may have landed since.

**The span below the rule carries no traceability section and therefore no relative links at all**, which avoids the tension AGENTS.md records for drafted bodies — that a traceability section written with `docs/decisions/`-relative paths resolves at the ADR's destination and not from the record, so the record must state that beside the span rather than repoint it. Checked rather than assumed: the span contains zero local markdown links, so the byte-identical transfer is unconditional here, exactly as it was for the BF16 draft. Cross-references the span needs are made by ADR number and by contract name in prose, which resolve from either location. Every link in this record *outside* the span resolves from here, and the ADR gains its traceability at landing if the lander adds one.

---

**Title:** Bind a subgroup combine to a register-transfer tree with a stated stride order and a proved lane identity

**Frontmatter:** `decision_status: proposed`, `implementation_status: not-started`, `catalog_group: "physical-planning-lowering"`, `applies_to: ["tiler.contract.fusion-and-scheduling", "tiler.contract.ir", "tiler.contract.metal-backend", "tiler.contract.artifact-abi"]`, `evidence: ["tiler.research.scheduling.subgroup-execution-tier"]`, `depends_on: ["ADR-0007", "ADR-0011", "ADR-0014", "ADR-0022", "ADR-0025", "ADR-0043", "ADR-0074"]`, `ticket: "land-the-subgroup-execution-tier-adr"`.

### Context

ADR 0007 makes the normalized schedule authoritative and places reduction contributor coverage, combine-tree numerical legality, and tail behaviour inside intrinsic verification. ADR 0043 decides that GPU subgroup scopes are not aliases for CPU vector scopes, and the adopted feasibility record it rests on already states that "a warp multiple is normally a cost heuristic, but warp width becomes hard when the algorithm uses masks, shuffles, votes, or warp-synchronous collectives." The adopted scheduled-region model names a `SubgroupTree` construct.

**None of that decides how a subgroup combine is represented**, and the implemented vocabulary reaches the workgroup tier only: five reduction topologies of which the cooperative one stages through workgroup memory and is ordered by a declared synchronization point, one execution binding, one tail policy, and no subgroup width anywhere in target feasibility. The one place a subgroup width is named — `RouteResourceDimension::SubgroupThreads` — is a live-device floor over "threads that execute in lockstep", which every implemented adapter answers `Unrecognized`.

### Decision

1. **A subgroup combine is a `ReductionTopology` sibling of `CooperativeWorkgroup`, not a construct inside a cooperative phase.** It carries a contributor partition, a literal width, an ordered combine tree, a lane identity, and a result lane. It carries no tile, no staging, and no synchronization point, because it has no handoff through memory.
2. **A subgroup combine derives no visibility edge and requires no synchronization point.** A shuffle names its source lane and its destination register in one operation that is both the transfer and the ordering; there is no address space to fence and no second program point at which an ordering could be placed. A subgroup-scoped memory visibility vocabulary is therefore not required by this tier.
3. **Every lane a combine step reads must be active at that step, and this is an intrinsic program obligation no target declares.** It is the same evidence class as barrier convergence. Because a trailing subgroup may be partially populated, a subgroup combine derives the hard requirement that the threadgroup size be an exact multiple of the subgroup width.
4. **The combine tree's stride order is stated and never inferred, because it decides which permission the region consumes.** An ascending-mask butterfly is a balanced binary tree over the ascending contributor sequence and consumes reassociation alone. A descending-stride tree reads its leaves in bit-reversed order and additionally consumes contributor permutation. Only the ascending form is admitted; the descending form is statable and refused, exactly as an unadmitted contributor arrival is.
5. **A contributor-free lane is fully participating and identity-valued, never masked off.** Masking it would make the lanes that read it read an inactive lane, which is undefined or indeterminate on every platform. The lane identity is a stated, checked field and is **never** derived from the region's empty-domain identity: under a contract forbidding signed-zero elimination the additive identity is `-0.0` and the empty-domain result is `+0.0`. Identity injection consumes no numerical permission — a true identity leaves the result bit-identical — and carries a proof obligation instead.
6. **Because the width is imposed by the device rather than chosen by the schedule, identity injection is required in the general case rather than being one tail policy among four.** This is the structural difference from the CPU vector lane tier, whose planner may choose a dividing width.
7. **The width is a literal in the schedule, an equality against an atomic target subject at the compile profile, and a confirmation against the prepared pipeline before routing commit.** It is never symbolic, because the combine steps are the topology's content and a symbolic width leaves numerical legality undischargeable at intrinsic verification. It is never a floor, because a wider device satisfies a floor without satisfying the lane arithmetic the schedule verified.
8. **Subgroup reduction collectives are not admitted, and `SynchronizationKind::Collective` stays refused.** Metal and WebGPU both specify their shuffles exactly and both decline to specify the combine order of their reduction collectives, so no numerical contract can be checked against one. Building the tree from shuffles and ordinary additions is what makes the tier statable.
9. **The subgroup lane and the CPU vector lane remain different bindings with no shared construct.** They differ in width authority, in whether a source may be inactive, and in what the ownership proof asserts. The tree-order vocabulary — ascending versus descending stride, and the leaf-order test deciding permutation — is one concept and may be shared; the combine construct is not.

### Consequences

- A reduction shorter than or equal to one subgroup becomes a schedule with no workgroup memory, no barrier, and no visibility edge, which the existing selection machinery enumerates and costs without modification.
- One schedule can be admitted on a permutation-forbidding contract in its ascending form and rejected in its descending form at identical instruction count, which makes tree stride order a planning dimension rather than an emitter detail.
- A subgroup schedule adds a preflight stage no other schedule has, and a width mismatch there must reject before any encoding rather than fall back after.
- The lane identity becomes the second construct in the vocabulary needing a proved reduction identity, and it should land as one concept with the CPU tier's padding identity rather than as two.
- Nothing here admits a two-level reduction, a narrowing shuffle tree, a subgroup memory scope, a subgroup collective, or any Metal, CUDA, or WebGPU backend claim.

### Alternatives considered

- **A subgroup combine inside a cooperative phase, one per staged slot.** Rejected: a subgroup-only reduction would carry an empty tile whose staging, lifetime, edge, and discharge machinery all range over nothing, and the width would be encoded inside the workgroup-shared-memory construct.
- **A width-symbolic schedule resolved at feasibility.** Rejected: the combine steps are the topology's content, so a symbolic width leaves the combine tree's numerical legality undischargeable at intrinsic verification — a stronger failure than the coverage one that refuses a scalable CPU contributor partition.
- **Reading the width from a shader builtin in the emitted body**, which is what the vendor's own example kernel does. Rejected: the region's reassociation structure would not be determined until dispatch, so the region checked against a numerical contract would not be the region that runs.
- **A subgroup width as a live-device floor.** Rejected: a wider device satisfies a floor without satisfying the lane arithmetic the schedule verified, and the property the existing floor names — lockstep within a subgroup — is not one modern GPU families guarantee.
- **Using the vendor subgroup reduction collective.** Rejected: neither Metal nor WebGPU states its combine order, so it is the opaque `block_reduce` the adopted scheduled-region model refuses by name.
- **A shared combine-tree construct spanning the CPU vector lane and the subgroup lane.** Rejected: it would have to make width authority, source-lane activity, and ownership all optional, at which point the verifier can no longer tell which obligations to discharge.

---

## Measurement boundary and unsupported cases

- **Nothing here was executed, emitted, compiled, or timed.** Every claim is inspected source at `8252312` or a primary vendor specification. There is no measurement in this record and therefore no measured bound on anything it claims. The instruction counts in Worked example B are *derived* operation counts, not timings, and they say nothing about which schedule is faster on any machine.
- **Three premises this work was dispatched with were checked and found wrong, and are corrected in place rather than dropped.** (i) "Metal simdgroups are 32 by spec" — the Metal Shading Language Specification 4.1 states no numeric SIMD-group width anywhere; 32 is an observation of `threadExecutionWidth`, and the only width the specification fixes is the quad-group's 4. (ii) "The subgroup width question is the same shape as the CPU fixed-versus-scalable split" — it is not, because a CPU vector width is chosen by the schedule and a subgroup width is imposed by the device, which changes the authority, the availability phase, and whether identity padding is optional. (iii) "This tier likely activates the refused `Collectives` kind" — it does the opposite; the shuffle tree exists precisely so the collective is not needed, and the vendor accuracy specifications strengthen the refusal.
- **Where the specification claims were read.** Metal Shading Language Specification, Version 4.1, dated 2026-06-04 (`developer.apple.com/metal/Metal-Shading-Language-Specification.pdf`), §4.4.1, §5.2.3.6, §6.10.1, §6.10.2, §6.10.2.1, Tables 6.14 and 6.15. WGSL specification (`w3.org/TR/WGSL/`), §12.14, §13.3.1.1.17–18, §15.5, §15.6.1, §15.6.3, §15.7.4, §17.12, §17.12.12. CUDA Programming Guide, "Warp Shuffle Functions" and the independent-thread-scheduling text in the advanced kernel programming chapter. IEEE Std 754-2019 clause 6.3.
- **The IEEE clause is quoted from a copy of a paywalled standard**, as the CPU record's boundary also records: IEEE Std 754-2019 is distributed through IEEE Xplore, and a reader with access should confirm clause 6.3 rather than treat this citation as the source of record.
- **The CUDA text was read through a summarizing fetch rather than from a local copy of the document.** The Metal and WGSL quotations were extracted from locally downloaded primary documents and grepped, so their wording is exact; the CUDA sentences are reproduced as returned and should be confirmed against the guide before being quoted normatively. This is a weaker evidence grade than the other two and is marked rather than levelled up.
- **`f32` only.** Every numerical derivation — the two-sided `-0.0` argument, the commutativity step, the bit patterns — is stated for IEEE 754 binary32. The identity argument transfers to any IEEE binary format; the per-platform shuffle rows do not, since a shuffle specified for one type says nothing about another, and `simd_shuffle`'s own type list in MSL Table 6.14 excludes `bfloat` and `long`.
- **One reduction shape, one tree shape, one width.** The examples use a single-axis sum over the trailing axis of a rank-2 input, an ascending butterfly, and `W = 32`. A multi-axis reduction, a contraction, a non-trailing reduced axis, a narrowing tree, and a non-power-of-two width each raise questions this record does not touch.
- **No cost model and no performance claim.** Which of the two worked examples is faster is not decided anywhere here. The one place this record touches cost is the observation that the ascending and descending trees have identical instruction counts, which is a statement about what a cost model *cannot* use to separate them, not an estimate.
- **No target profile row, no Metal claim, and no realization.** No profile declares a subgroup width, no backend emits a shuffle, and the strict-profile elimination in the simdgroup-matrix realization ticket is untouched by this record.
