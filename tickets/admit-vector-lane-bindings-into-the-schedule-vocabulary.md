---
id: admit-vector-lane-bindings-into-the-schedule-vocabulary
title: Admit the first vector-lane schedule boundary
status: in-progress
priority: p2
dependencies: [accept-adr-0093-cpu-vector-lane-tier]
related: [design-the-cpu-vector-lane-tier, admit-shared-contributor-coverage-and-reduction-padding-identity, declare-cpu-vector-realization-facts-in-the-target-profile, define-plural-operation-specific-vector-realization-requirements, admit-fixed-vector-ssa-and-unmasked-memory-into-kernel-ir]
scopes: [implementation/ir, implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [scheduling, ir, cpu, simd, execution-hierarchy, public-boundary, decision, needs-tom]
claimed_from: todo
assignee: worker-vector-lanes
lease_expires_at: 1787062687
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

## Source-first Fact re-audit — 2026-08-18, at base b9258e51

Re-read at the implementation base before any edit. Verdicts are against the 2026-08-12 audit above, item by item.

1. **Stale in both counts and anchor; final clause verified.** `ExecutionBinding` now has two variants (`GlobalLinearInvocation`, `BlockedWorkgroup`), `TailPolicy` has two (`Exact`, `Predicated`), and `ReductionTopology` has seven (`None`, `Serial`, `MultiPass`, `Contraction`, `LiveContraction`, `CooperativeWorkgroup`, `CooperativeContraction`) — all in `crates/tiler-ir/src/schedule/model.rs`. The anchor `schedule.binding != ExecutionBinding::GlobalLinearInvocation` no longer exists; `verify_intrinsic` now decides the binding by a match pairing bindings with topologies, anchor `Err(blocked(BlockedWorkgroupRule::BindingRequired))` in `crates/tiler-ir/src/schedule/builder.rs`. Still true and load-bearing: no vector construct exists anywhere in the vocabulary, and no production vector schedule exists.
2. **Verified.** `OwnershipProofKind` still has the single `OneGlobalInvocationPerOutput` variant (anchor `Exactly one global invocation writes each of` in `model.rs`), and nothing in the fixed-map admission touches an operand, rounding site, or contributor order.
3. **Verified.** `ContributorCoverage` sits inside `MultiPass` and `CooperativeWorkgroup`; `KernelSchedule::tail` remains iteration-domain launch coverage. Note `TailPolicy` has since gained `Predicated` for the blocked cooperative contraction — that does not revive the retired `IdentityPadded` tail bullet.
4. **Verified** (argument, not a source claim; unchanged).
5. **Verified** (derivation; restated against `LaunchPlan::grid_threads` by the accepted correction below).
6. **Verified.** `KernelType` in `crates/tiler-ir/src/kernel/model.rs` is `Bool | U8 | Index | F32 | I32 | Bf16 | U32` — no lane shape, mask type, lane index, or masked memory operation; no scalable representation exists.
7. **Stale anchors and stale domain; conclusion re-derived and it holds at v6.** The domain is now **`tiler.schedule.v6`** (`crates/tiler-ir/src/domains.rs`, anchor `b"tiler.schedule.v6\0"`), stepped by the AccessOrdinal reconciliation, and `push_schedule` is now a match whose arms are `ExecutionBinding::GlobalLinearInvocation => bytes.push(0x01)` and `BlockedWorkgroup` at tag `0x02` (the cited `let ExecutionBinding::GlobalLinearInvocation` let-binding is gone). Re-derivation at v6: appending a fresh binding tag `0x03` plus a fixed-width big-endian lane count moves no previously encodable region's bytes — both existing arms keep their tags and every later field its position — and cannot collide with an old encoding, because the tag occupies a decode-determined position where old regions carry only `0x01` or `0x02`, and the lane count is fixed-width so following fields stay determined. This is the same append argument the `0x02` arm already carries, and unlike the `v4` counterexample no variable-length field precedes the appended payload within the arm. No `tiler.schedule.v6` step is required.
8. **Verified as an accepted plan.** No `tiler-cpu*` crate exists under `crates/` at this base; the real CPU path named by the close condition does not exist and is out of this ticket's scope (see the delivery record below).

Also re-verified from the acceptance text: `ExecutionBinding` is `#[non_exhaustive]` under ADR 0074 convention 5a (anchor `pub enum ExecutionBinding` two lines below the attribute in `model.rs`), so adding the variant is additive out of crate; the total matches it forces are all inside `tiler-ir` — `verify_intrinsic`'s binding match, `push_schedule`, and the kernel refinement gate's builtin match (anchor `ExecutionBinding::GlobalLinearInvocation | ExecutionBinding::BlockedWorkgroup` in `crates/tiler-ir/src/kernel/verify.rs`). All four successor tickets named by the acceptance exist in `tickets/`.

## Recommended first public surface

Add one opaque value and one execution-binding arm:

- `VectorLaneCount`, with a checked `u64` constructor requiring at least two lanes and a `get` reader. Width zero is invalid and width one is the existing scalar map under another spelling, so neither is representable. There is no power-of-two rule, architecture preset, default, or independent alpha policy cap.
- `ExecutionBinding::FixedVectorMap { lanes: VectorLaneCount }`. The name states the axis it binds. It does not stand for a contributor partition, a horizontal reduction, a scalable vector, a worker thread, or a backend instruction choice.

The first admitted combination is deliberately narrow:

- `TailPolicy::Exact` only;
- map-parallel regions whose existing ownership proof establishes one owner for every output;
- literal `N` and `W` with checked `N mod W == 0`;
- logical `work_items` continues to count the `N` scalar output positions, while `launch.grid_threads` counts the `N / W` fixed-vector packets; packet `p`, lane `l` owns scalar output `pW + l`;
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

**Accepted correction — 2026-08-12.** The earlier phrase that the launch population continued to count scalar outputs was false against `LaunchPlan::grid_threads`, whose current invariant is the number of executing invocations. Tom accepted the source-first correction while accepting the real fixed-vector CPU prerequisite: `work_items = N` remains the logical scalar-output population, `launch.grid_threads = N / W` is the exact packet population, and packet `p`, lane `l` owns output `pW + l`. No implementation may retain `grid_threads = N` and ask an emitter to reinterpret the builtin, because that would execute `N * W` lane positions under a launch identity claiming only `N` outputs.

Predicated tails, scalar epilogues, contributor partitions, and scalable maps remain separate public boundaries. They are now owned by [`admit-predicated-fixed-vector-map-tails`](admit-predicated-fixed-vector-map-tails.md), [`admit-scalar-epilogue-fixed-vector-map-tails`](admit-scalar-epilogue-fixed-vector-map-tails.md), [`admit-fixed-vector-contributor-partitions`](admit-fixed-vector-contributor-partitions.md), and [`admit-scalable-vector-map-bindings`](admit-scalable-vector-map-bindings.md). None may reinterpret `FixedVectorMap` or count a mock, simulator, Candle adapter, or reference evaluator as its runtime consumer.

This ticket moves to implementation state. The accepted schedule carrier remains non-executable until lane-shaped KIR, exact target requirements, provider-versioned execution and numerical evidence, artifact delivery, host qualification, and a real native `tiler-cpu` / `tiler-cpu-runtime` approach compose successfully.

## Delivery record — 2026-08-18, branch tkt/admit-vector-lane-bindings-into-the-schedule-vocabulary from base b9258e51

The accepted schedule carrier is implemented exactly as accepted, in `tiler-ir` alone; no compiler, artifact, or backend crate needed an edit, because `ExecutionBinding` is 5a-`#[non_exhaustive]` and no out-of-crate site classifies it (verified: `cargo check --workspace` passes with no change outside `tiler-ir`, and `grep -rn "\.binding" crates/*/src` finds no schedule-binding read outside `tiler-ir`).

**Surface** (`crates/tiler-ir/src/schedule/model.rs`, exported from `schedule/mod.rs`): opaque `VectorLaneCount` with checked `const fn new(u64)` requiring at least two lanes and a `get` reader; construction refuses zero as `VectorLaneCountError::Zero` (`vector-lane-count-zero`) and one as `VectorLaneCountError::ScalarSpelling` (`vector-lane-count-scalar-spelling`), named separately. `ExecutionBinding::FixedVectorMap { lanes: VectorLaneCount }` with the accepted-surface doc label. No power-of-two rule, preset, default, or cap.

**Verifier** (`crates/tiler-ir/src/schedule/builder.rs`, `verify_intrinsic`): admitted only with `TailPolicy::Exact` over `ReductionTopology::None` and `ReductionTopology::Serial` — the pointwise map and the strict serial fold across independent outputs, both proven under the fully strict contract. Launch identity per the accepted correction: `work_items = N`, `launch.grid_threads = N / W`, checked as `grid_threads * W == N` with checked multiplication after `N mod W == 0`. Distinct typed refusals under `ScheduledRegionDiagnostic::VectorLaneBinding`: `fixed-vector-map-nondivisible-coverage`, `fixed-vector-map-packet-arithmetic-overflow`, `fixed-vector-map-packet-population` (this is the rule that refuses `grid_threads = N` with a reinterpreted builtin), `fixed-vector-map-unsupported-reduction` (arms spelled per topology, no wildcard), and `fixed-vector-map-exact-tail-required`. A wrong output-owner population keeps its existing independent `proof-reference` refusal. The admission arm reads no numerical field and no target/profile/provider anything — evidence: `grep -rn "TargetProfile\|target_profile" crates/tiler-ir/src/schedule/` is empty, the only `provider` match in `builder.rs`/`model.rs` is a doc comment, and `crates/tiler-ir/Cargo.toml` depends only on `num-bigint`, `num-integer`, `num-traits`, `tiler-digest`, so no target type is even nameable during intrinsic verification.

**Identity** (`push_schedule`): `GlobalLinearInvocation` keeps tag `0x01`, `BlockedWorkgroup` keeps `0x02`; `FixedVectorMap` appends tag `0x03` plus the canonical big-endian `u64` lane count. The additive argument re-derived at `tiler.schedule.v6` is recorded on the encoder arm: both earlier arms keep their bytes, the appended payload is fixed-width with nothing variable-length before it in the arm, and old encodings carry only `0x01`/`0x02` at the decode-determined position — so no `v6` step. All existing pins pass unmodified: `STRICT_F32_REGION_IDENTITY_HEX`, `ONE_COMMITTER_COOPERATIVE_IDENTITY_HEX`, `STRICT_F32_REGION_IDENTITY_HEX_V5` comparison, and the kernel-layer pins in `crates/tiler-ir/src/kernel/tests.rs`.

**Non-executable carrier**: `KernelDiagnostic::UnloweredExecutionBinding` (`unlowered-execution-binding`), refused at two independent layers — `plan` in `kernel/lower.rs` before any body is derived (covers `lower_scheduled_region`, `derive_canonical`, and the refinement gate), and the builtin match in `kernel/verify.rs` for producer-authored kernels. Everything downstream (compiler, artifact, Metal) consumes verified kernels, so these refusals keep the carrier non-executable end to end. The `body_shaping_vocabulary_is_closed` tripwire fired and was answered; `revisit-kernel-body-single-spelling-gate` carries the dated **not fired** trigger-check entry.

**Tests** (10 new): admission of exact fixed-map pointwise work and of the strict serial fold, both strict-contract; both constructor refusals by name; the five verifier refusals each reached independently with exact diagnostics; binding-tag and lane-count identity perturbations isolated on a zero domain (every other byte equal; the vector encoding is exactly eight bytes longer than the scalar one); and the lowering refusal with its typed error.

**Perturbation evidence** (subject perturbed, never the assertion; each reverted after capture):

- Encoder collided `FixedVectorMap` onto tag `0x01` with the lane count dropped → ``assertion `left != right` failed`` in `the_fixed_vector_binding_tag_and_lane_count_separate_identity`, both sides printing identical `tiler.schedule.v6` byte vectors — the scalar and vector regions shared identity, which is the collision the tag exists to prevent.
- Divisibility rule deleted from `verify_intrinsic` → `a_nondivisible_fixed_vector_domain_is_refused_by_name` failed with ``left: [VectorLaneBinding { rule: PacketPopulation }] / right: [VectorLaneBinding { rule: NondivisibleCoverage }]`` — still fail-closed through the packet equality, but the named rule is load-bearing and the test pins it.
- Both lowering refusals deleted (the `plan` guard and the builtin-match arm) → `the_fixed_vector_map_carrier_is_refused_by_the_lowering_by_name` failed with ``called `Result::unwrap_err()` on an `Ok` value: VerifiedKernel { .. }`` — the carrier lowered and verified as a scalar kernel, which is exactly the silent-scalarization hazard the refusals exist to close. Deleting the `plan` guard alone still refuses through the builtin arm (test stayed green), so the two layers are genuinely independent.

**Commands, all green at the delivered tree**: `cargo check --workspace`; `cargo nextest run -p tiler-ir -p tiler-compiler` (2124 passed, 1 skipped); `cargo test -p tiler-ir --doc`; `cargo clippy -p tiler-ir -p tiler-compiler --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-ir`; `cargo fmt --check`; `tkt lint`; `make citations`; `git diff --check`.

**Remainder, deliberately out of this delivery**: the close condition's clause "implementation evidence proves the carrier through the real CPU path" cannot be discharged — no `tiler-cpu*` crate exists at this base, and building that path is `prove-the-first-real-fixed-vector-cpu-execution-approach` and its successors, not this ticket. The status stays `in-progress` for the coordinator to split the remainder and close against the delivered carrier scope. Everything else in the close condition is done: Tom's acceptance is recorded above, and all four successor tickets exist (`admit-predicated-fixed-vector-map-tails`, `admit-scalar-epilogue-fixed-vector-map-tails`, `admit-fixed-vector-contributor-partitions`, `admit-scalable-vector-map-bindings`).

## Closes when

Tom accepts the exact included and excluded surface above, the successor graph is filed, and implementation evidence proves the carrier through the real CPU path without widening the accepted scope.
