---
id: admit-vector-lane-bindings-into-the-schedule-vocabulary
title: Admit the first vector-lane schedule boundary
status: todo
priority: p2
dependencies: [accept-adr-0093-cpu-vector-lane-tier]
related: [design-the-cpu-vector-lane-tier, admit-shared-contributor-coverage-and-reduction-padding-identity, declare-cpu-vector-realization-facts-in-the-target-profile, define-plural-operation-specific-vector-realization-requirements, admit-lane-typed-values-and-masked-memory-into-the-kernel-ir]
scopes: [implementation/ir, implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [scheduling, ir, cpu, simd, execution-hierarchy, public-boundary, decision, needs-tom]
---
## User-visible outcome

A scheduled region can state one exact fixed-width vector assignment over independent output coordinates. The verifier proves coverage, ownership, and bounds without consuming numerical-order permission, and every broader vector form remains unavailable until its own required authority and real CPU consumer exist.

## Source-first Fact audit — 2026-08-12

1. **Verified.** `ExecutionBinding` has only `GlobalLinearInvocation`, `TailPolicy` has only `Exact`, and `ReductionTopology` has five variants with no vector topology. `verify_intrinsic`, anchor `schedule.binding != ExecutionBinding::GlobalLinearInvocation`, rejects every other binding before family verification. No production vector schedule exists.
2. **Verified.** The existing ownership proposition is already the right one for a lane-over-map schedule. `OwnershipProofKind::OneGlobalInvocationPerOutput` states one owner per output, and the accepted CPU research treats each active lane as that invocation. Assigning independent output positions to lanes changes no operand, rounding site, or contributor order and consumes no reassociation, permutation, contraction, or signed-zero permission.
3. **False in the old packet.** Contributor padding is not a `TailPolicy`. The accepted [`admit-shared-contributor-coverage-and-reduction-padding-identity`](admit-shared-contributor-coverage-and-reduction-padding-identity.md) boundary places exact-versus-identity-padded coverage inside the reduction topology and preserves `KernelSchedule::tail` for iteration-domain launch coverage. The old `IdentityPadded` tail bullet and failure case are retired.
4. **Imprecise in the old packet.** A generic `ExecutionBinding::FixedVectorLane` does not say whether the lane binds the output map or a reduction's contributor partition. The two cases have different ownership, coverage, numerical, and KIR obligations. Encoding the distinction only through a second field permits misleading cross-field combinations and makes every consumer reconstruct which axis the lane means.
5. **Verified.** A fixed map assignment with `TailPolicy::Exact` is completely answerable now: for logical iteration count `N` and literal lane count `W >= 2`, intrinsic verification requires `N mod W == 0`; lane `l` in packet `p` owns linear output `pW + l`; the existing bounds and ownership proofs cover exactly the same `N` coordinates. No target fact or runtime observation is needed to establish that relation.
6. **Not implementation-ready beyond that slice.** A predicated tail needs lane-mask KIR, explicit inactive-lane load/store semantics, and a target fact for the exact masked memory form. A scalar epilogue needs two exact execution realizations and numerical authority for both paths. A contributor-partition binding needs the shared coverage/identity carrier. A scalable map binding needs width-agnostic KIR and a real scalable CPU representation. None of those production consumers exists.
7. **Verified identity consequence.** `push_schedule`, anchors `let ExecutionBinding::GlobalLinearInvocation` and `bytes.push(0x01)`, can preserve every existing schedule byte by retaining tag `0x01` and appending a fresh tag plus the fixed lane count for the new binding. The current `tiler.schedule.v5` domain need not step for that additive arm. Later KIR, artifact, and selected-realization carriers own their own version consequences.
8. **Verified real-consumer boundary.** The accepted production CPU split is `tiler-cpu-image`, `tiler-cpu`, and `tiler-cpu-runtime`; vector execution belongs to a future explicit native fixed-vector approach in the latter two packages. The scalar image, Candle, a KIR simulator, a mock provider, and the reference evaluator are not consumers and cannot make this schedule executable.

## Recommended first public surface

Add one opaque value and one execution-binding arm:

- `VectorLaneCount`, with a checked `u64` constructor requiring at least two lanes and a `get` reader. Width zero is invalid and width one is the existing scalar map under another spelling, so neither is representable. There is no power-of-two rule, architecture preset, default, or independent alpha policy cap.
- `ExecutionBinding::FixedVectorMap { lanes: VectorLaneCount }`. The name states the axis it binds. It does not stand for a contributor partition, a horizontal reduction, a scalable vector, a worker thread, or a backend instruction choice.

The first admitted combination is deliberately narrow:

- `TailPolicy::Exact` only;
- map-parallel regions whose existing ownership proof establishes one owner for every output;
- literal `N` and `W` with checked `N mod W == 0`;
- the existing logical `work_items` and launch population continue to count scalar output invocations, while the binding groups those invocations into exact packets of `W` lanes;
- no numerical permission is consumed merely by grouping independent outputs;
- every lane-shaped arithmetic or memory operation later emitted must be explicit in KIR and in the plural target-requirement carrier; absence leaves the plan non-executable rather than authorizing scalarization.

The intrinsic verifier rejects zero width at construction, nondivisible coverage, wrong ownership count, unsupported reduction/binding combinations, and every non-`Exact` tail by distinct typed rules. It never rounds the iteration count, masks implicitly, peels a scalar tail, or asks a target to repair intrinsic coverage.

The encoder retains `GlobalLinearInvocation` as tag `0x01` and appends a fresh tag for `FixedVectorMap`, followed by the canonical big-endian lane count. Same-crate matches over `ExecutionBinding` remain exhaustive. Existing schedules stay byte-identical; new lane counts produce distinct identities.

## Explicit successors, not hidden states

- Predicated fixed-map tails follow only after fixed lane masks, fault-suppressing masked memory, and exact target declarations exist.
- Scalar epilogues follow only after the selected entry can carry both provider-versioned execution subjects and both numerical paths.
- Fixed-vector contributor partitions follow the shared `ContributorCoverage` and `ReductionPaddingIdentity` implementation and receive their own topology; they do not overload `FixedVectorMap`.
- Scalable vector maps follow a real scalable CPU representation and width-agnostic KIR. `ExecutionBinding` is already `#[non_exhaustive]`, so no dead reservation is required now.
- A horizontal ordered accumulate that changes neither map nor partition remains instruction selection below the schedule boundary.

Each successor must name its real `tiler-cpu` / `tiler-cpu-runtime` consumption path. Structural tests may verify the carrier, but no simulator, fake device, Candle adapter, or reference-evaluator vector mode counts as delivery.

## Ranked options

1. **Exact fixed-vector map slice above.** Best correctness and fail-closed behavior; smallest truthful public surface; no dead variants; no arbitrary cap; O(1) retained state and bounded arithmetic in verification; enough to exercise lane-shaped KIR, exact target facts, provider-versioned numerical authority, artifact delivery, and a real native CPU approach end to end.
2. **Add fixed-map predicated and scalar-epilogue tails now.** Can be correct, but publishes requirements whose KIR and target consumers are still undecided. It offers more shapes while increasing invalid-state and cross-layer drift risk before a real backend can consume them.
3. **Add the full old packet: fixed, scalable, map, partition, and all tails.** Architecturally broad but not implementation-ready. It couples four independently blocked authorities and would make several public variants constructible only to be refused everywhere.
4. **Generic `FixedVectorLane` plus a separate topology field deciding what the lane means.** Reject: the public value is not MECE by itself, consumers must infer its axis from another field, and invalid pairings become representable.
5. **Backend-inferred vectorization with no schedule/KIR carrier.** Reject: it changes execution form, bounds behavior, numerical authority, and artifact provenance outside verified identity.
6. **Keep vector execution unavailable forever.** Correct but does not advance the accepted CPU vector path; it remains the temporary behavior until the first slice lands.

## Strongest counterpoint and reversal evidence

The narrow slice delays useful NEON tails, AVX/SVE predication, and contributor-lane reductions, and later additions will cause more public review. That cost is real. It is preferable to publishing dead or context-dependent states: the enums were deliberately made `#[non_exhaustive]` so evidence-backed variants can append without guessing them now. Reverse to a broader first landing only when one real native CPU producer and runtime can consume the broader form, its exact KIR and target facts are accepted, and a perturbation demonstrates that omitting or changing each new field produces a distinct refusal rather than a backend fallback.

## Required evidence

- Admit exact fixed-map pointwise work and one strict serial fold across independent outputs without granting a numerical permission.
- Refuse lane counts zero and one at construction, naming invalidity and duplicate scalar spelling separately.
- Refuse `N mod W != 0`, overflow in packet arithmetic, wrong output-owner population, and an unsupported reduction/binding combination independently.
- Perturb binding tag and lane count and prove identity inequality; retain byte-identical pins for every old schedule.
- Prove no target/profile/provider call occurs during intrinsic coverage verification.
- Keep the candidate non-executable while KIR, target requirements, selected execution provenance, host qualification, and the real CPU native approach are absent.

## Decision request

Accept the exact fixed-vector map slice and split the broader forms into their evidence-backed successors; revise the slice; or keep vector schedules unavailable. Acceptance chooses a public schedule boundary, not implementation authorization for KIR, target declarations, artifacts, or a native CPU emitter.

## Accepted decision — 2026-08-12

Tom accepted the recommended exact fixed-vector map slice in the live Codex coordination thread by replying `okay agreeed, next decision`. The relay source is Tom's direct response in that thread.

The accepted public boundary is exactly `VectorLaneCount` with a checked `u64` value of at least two and `ExecutionBinding::FixedVectorMap { lanes }`, admitted initially only with `TailPolicy::Exact`. Width one remains the existing scalar map rather than a second vector spelling. The verifier derives exact divisibility, scalar-output coverage, ownership, and bounds without consulting a target or consuming any numerical-order permission. No power-of-two rule, architecture preset, default, independent lane-count budget, implicit mask, scalar peeling, or backend scalarization is accepted.

Predicated tails, scalar epilogues, contributor partitions, and scalable maps remain separate public boundaries. They are now owned by [`admit-predicated-fixed-vector-map-tails`](admit-predicated-fixed-vector-map-tails.md), [`admit-scalar-epilogue-fixed-vector-map-tails`](admit-scalar-epilogue-fixed-vector-map-tails.md), [`admit-fixed-vector-contributor-partitions`](admit-fixed-vector-contributor-partitions.md), and [`admit-scalable-vector-map-bindings`](admit-scalable-vector-map-bindings.md). None may reinterpret `FixedVectorMap` or count a mock, simulator, Candle adapter, or reference evaluator as its runtime consumer.

This ticket moves to implementation state. The accepted schedule carrier remains non-executable until lane-shaped KIR, exact target requirements, provider-versioned execution and numerical evidence, artifact delivery, host qualification, and a real native `tiler-cpu` / `tiler-cpu-runtime` approach compose successfully.

## Closes when

Tom accepts the exact included and excluded surface above, the successor graph is filed, and implementation evidence proves the carrier through the real CPU path without widening the accepted scope.
