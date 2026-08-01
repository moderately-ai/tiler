---
id: lower-a-loop-carried-cooperative-body
title: Lower a loop-carried cooperative body
status: review
priority: p1
dependencies: []
related: [admit-loop-carried-cooperative-staging, realize-the-strict-contraction-on-metal, implement-the-single-workgroup-synchronized-reduction-strategy]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [research, physical-planning, contraction]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785626991
---
## Scope note — 2026-08-01

`contracts/foundation` was added for one paragraph of `docs/ir.md`: the kernel-verifier bullet that said a multi-round tile is representable and not yet lowered, which this change falsifies. `ticketsplease.toml` maps `docs/ir.md` to that scope and no live claim held it.

`docs/status.md` and `docs/roadmap.md` carry the same falsified claim and were **not** edited. They map to `contracts/navigation`, which `land-the-two-level-reduction-adr` held live; [AGENTS.md](../AGENTS.md) admits an edit inside another live ticket's scope only when file-level disjointness is verified against that worker's actual branch diff, and no `tkt/land-the-two-level-reduction-adr` branch existed locally or on the remote to diff against (`git branch -a`, `git worktree list`). The corrections are filed as [`correct-the-navigation-docs-for-the-loop-carried-body`](correct-the-navigation-docs-for-the-loop-carried-body.md) with the exact spans.

## User-visible outcome

A cooperative tile with more than one round reaches a verified structured kernel. Until it does, the loop-carried staging vocabulary [`admit-loop-carried-cooperative-staging`](admit-loop-carried-cooperative-staging.md) landed is representable and unlowerable: the schedule verifier admits the rewrite, derives its anti-dependencies, and requires a point for each, and `cooperative_plan` then refuses the tile by name (`crates/tiler-ir/src/kernel/lower.rs`, `KernelDiagnostic::CooperativeLoweringShape`).

## The blocker, with the exact checks

**Fact — a predicated region produces no values.** `OperationKind::Predicated { predicate, body }` (`crates/tiler-ir/src/kernel/model.rs`) carries no results, and the cooperative lowering's own comment records the consequence: "a value defined inside a guarded block cannot cross into the next one" (`lower.rs`, `emit_cooperative`).

**Fact — every boundary load must be dominated by the governed iteration guard.** `verify_effects` refuses any unguarded effect as `KernelDiagnostic::PredicateDominance` (`crates/tiler-ir/src/kernel/verify.rs`). Staged accesses are deliberately *not* effects, which is the seam this work should use.

**Fact — exactly one store, at loop depth zero.** `verify_effects` again: `stores != 1` and `effect.loop_depth != 0` are both `OutputCoverage`.

**Inference — the three compose into "no accumulator can cross a round".** A round's contribution is computed from boundary loads, so it is guarded; the accumulator must survive the round loop's back edge, so it must leave the guard; and it cannot. The one shape that escapes it puts the guarded work and the unguarded staged fold in separate regions of the same loop body — the accumulator is then updated by staged loads alone, which need no guard — and that shape is worth confirming before any KIR change is proposed.

**Inference — the seed then peels a round, and the peel realizes each point twice.** Every fold in this repository seeds at its first contributor (`start == 1`) rather than at the reduction identity, because `+0.0 + x` is not `x` for `x = -0.0`. A round loop seeded the same way runs `1..rounds` with round zero emitted ahead of it, so each declared point is realized once in the peel and once in the body — which `verify_synchronization`'s "every declared point is realized exactly once" refuses, and which `verify_edge_is_ordered` (written over a single fence position) cannot check.

## What this owns

- Whether the accumulator can be kept out of every predicate, or a value-producing predicated region (or an explicit select) is required. The second is a KIR vocabulary change and a public boundary.
- The realization rule for a peeled round: how many times a point may be realized, and what makes each realization convergent and correctly ordered. `barrier_is_convergent` (`verify.rs`) already proves the convergence half and states that matching the enclosing loop's trip count against the declared round count is this ticket's obligation.
- `verify_edge_is_ordered` and its anti-dependency counterpart, generalized from one fence position to a cyclic round body: a barrier at position `b` separates round `r`'s read at `c` from round `r + 1`'s write at `w` exactly when `b > c` or `b < w`, which is the body-level mirror of `SynchronizationPoint::discharges_anti`.
- The first multi-round strategy to emit, and its contributor arithmetic. `ReductionTopology::CooperativeWorkgroup` already defines it: participant `p` of round `r` owns the contiguous range at index `r * partitions + p`, and `verify_cooperative_loops` (`verify.rs`) needs the round dimension its single-round form does not have.

## What this does not own

The `tiled` contraction's schedule, its `K ≡ 0 (mod 16)` precondition, and its Metal emission stay with [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md). The log-depth tree needs two further capabilities this ticket does not provide — a per-access active-participant subset and a span whose stride and count depend on the round ordinal — both recorded in `crates/tiler-ir/src/schedule/cooperative.rs`.

## Closes when

A multi-round cooperative tile lowers to a body that passes the structured-kernel verifier and executes to the same bits as the reference at its declared order; every new or generalized rule has been watched refusing its own defect, including a barrier whose enclosing loop is not the round loop; and the identity consequence of whatever the body required is recorded.

## Outcome

**The body landed, and it needed no KIR vocabulary change and no identity step.** A multi-round cooperative tile now lowers to a verified structured kernel, executes to the reference's exact bits at its declared order, and every rule the round structure adds has been watched refusing its own defect.

### The first question, answered by construction rather than by proposal

**The accumulator can stay outside every predicate, so no value-producing predicated region and no explicit select is required.** The ticket's own inference is confirmed: the guarded work and the unguarded staged fold go in separate regions of the loop body, and the accumulator is updated by staged loads alone.

The reason is a seam the ticket already named. `verify_effects` requires predicate dominance of *boundary* effects — buffer loads and stores — and staged accesses are deliberately not effects. So the staged fold needs no guard, sits at the kernel's top level in the peel and directly in the round body inside the loop, and its result is an ordinary top-level value the round loop can carry. Soundness comes from the same fact the top-level barrier already rests on: `TailPolicy::Exact` plus `grid_threads == work_items` means every launched invocation satisfies the iteration guard, so the invocations running the unguarded fold are exactly the ones that ran the guarded staged store.

**Nothing was enumerated for Tom on this axis, because the elimination left one survivor.** The cost it does carry is that every participant folds the staged set rather than only the committing one, which the single-round shape does not pay — filed with its activation triggers as [`remove-the-loop-carried-redundant-staged-fold`](remove-the-loop-carried-redundant-staged-fold.md) rather than absorbed.

### The two shapes, and why they are two

`emit_cooperative` keeps the single-round body **byte-identical**: its staged fold stays inside the commit guard, where one participant runs it. `emit_loop_carried_cooperative` is a separate emission. Unifying them was considered and eliminated on two independent grounds — it would make every single-round cooperative kernel pay `participants - 1` redundant folds for no benefit, and it would move six Metal goldens, `ARTIFACT_IDENTITY`, and `CACHE_SUBJECT` for a change with no semantic content. Keeping them apart is what makes this landing identity-neutral.

```text
%gid, %lid, %out = %gid / P, %par = %gid % P, %act = %gid < work_items
if (%act) { fold round 0's range ; staged_store[%lid] }
barrier(phase point)
%seed = fold staged[0..P]
%total = loop r in 1..rounds carrying %seed {
    barrier(round point)
    if (%act) { fold round r's range ; staged_store[%lid] }
    barrier(phase point)
    %t = fold staged[0..P]
    yield canonicalize(%acc + %t)
}
if (%act) { if (%lid < commit) { store[%out] = %total } }
```

### The realization rule for a peeled round

**A point admits as many realizations as its placement means, and the two placements mean different counts.** A phase boundary separates two phases *of a round*, so it happens on each of the `rounds` rounds; a round boundary separates *consecutive* rounds, so it happens at each of the `rounds - 1` transitions between them. A barrier at the kernel's top level realizes its point once; one inside the round body realizes it once per iteration, which the `1..rounds` range makes `rounds - 1`.

That is what makes the peel checkable by arithmetic rather than by recognizing a shape. The peel exists because every fold in this repository seeds at its first contributor (`+0.0 + x` is not `x` at `x = -0.0`), so round zero is emitted ahead of the loop: the phase boundary is realized `1 + (rounds - 1) = rounds` times and the round boundary `rounds - 1` times. A body that dropped the peel realizes the phase boundary `rounds - 1` times; one that put the round boundary in the peel realizes it `rounds` times. Both are refused as `SynchronizationRealization`, and a point realized nowhere is refused as `UndischargedVisibility` or the new `UndischargedAntiDependency` depending on which class it discharges — a point discharges at most one, because the two conditions contradict.

**`barrier_is_convergent` weakened by exactly one word and gained a separate obligation.** It now reads `block_depth == loop_depth && loop_depth <= authorized_barrier_loop_depth(rounds)` — *at most* the authorized nesting rather than exactly it, because the peel's barrier is at the top level of a multi-round tile. What replaced the strictness is `round_body`, which discharges the obligation the old comment deferred: the loops containing a barrier must be exactly one, at block depth zero, with range `1..rounds`. A `0..rounds` loop — the shape a body with no peel emits — and a barrier inside any other top-level loop are both `SynchronizationConvergence`.

### The cyclic ordering rule

**Visibility is checked per region, not once.** A loop-carried body performs the handoff on every round and the peel is a separate lexical copy of one, so the events are partitioned into three regions — before the round loop, inside it, after it — and each region that touches the allocation must carry both ends with exactly one fence between them. The old single-fence check would have refused the loop's own write for following the peel's fence, which is not a defect. The trailing region is empty in the canonical body and is kept anyway, so a staged access emitted after the loop belongs to a region rather than to nothing.

**The anti-dependency is two obligations, and a body can satisfy either without the other.** Inside the round body the read at `c` is round `r`'s and the write at `w` is round `r + 1`'s, so a barrier at `b` separates them exactly when `b > c` or `b < w` — the body-level mirror of `SynchronizationPoint::discharges_anti`, a disjunction for the same reason: the two ends are in different rounds. The peel is not cyclic: round zero's reads are ordinary predecessors of the loop's first write and need a realization between them in flat program order, which only the `b < w` arm supplies. **That is why the canonical body puts the round boundary at the head of the loop body rather than the tail**, and a tail placement is refused as the new `UnorderedStagedRewrite` — verified by perturbing the emission itself, not only a hand-built body.

### The multi-round contributor arithmetic

Participant `p` of round `r` owns the contiguous range at index `r * participants + p`, so its first contributor is `r * contributors_per_round + p * contributors_per_partition` with `contributors_per_round = participants * contributors_per_partition`. `emit_partition_contributor` gained a `RoundOrdinal` — an `Option`, `None` for the peeled round zero and for the multi-pass partial pass that has no round dimension, so every round term vanishes exactly rather than being emitted as a multiplication by a zero. `verify_cooperative_loops` gained the round dimension: a loop-carried body's five folds at block depths `1, 0, 0, 2, 1`, where the depths and not the trip counts are what distinguish the peel's folds from the loop's.

**The verifier does not check this arithmetic, and that is why the executed conformance test is the required evidence.** Perturbing the round term away leaves the kernel verifying and produces a different number; only the execution caught it.

### Identity consequences

**No domain stepped and no pin moved.** No KIR construct, field, tag, or encoding changed: the multi-round body is built from the operations that already existed, and the single-round emission is unchanged operation for operation. The whole-workspace run is **2,281 passed, 7 skipped, 0 failed** — the six `crates/tiler-metal/goldens/*.metal` digests and `crates/tiler-build/src/metal_plan.rs`'s `ARTIFACT_IDENTITY` and `CACHE_SUBJECT` all hold, which is the enumeration and the evidence together. `metal_plan.rs` was not edited.

### Public boundary items, none self-accepted

Three appended variants of `KernelDiagnostic` (`#[non_exhaustive]`, in `tiler_ir::kernel`), each with its `rule()` identifier:

- `SynchronizationRealization` (`synchronization-realization`) — a declared point realized a different number of times than its placement requires.
- `UndischargedAntiDependency` (`undischarged-anti-dependency`) — the anti-dependency counterpart of `UndischargedVisibility`, separate for the reason the schedule layer separates `SynchronizationRule`'s two: a rewrite that overtakes an unfinished read destroys a value rather than reading an unpublished one.
- `UnorderedStagedRewrite` (`unordered-staged-rewrite`) — the counterpart of `UnorderedStagedHandoff`, separate for the same reason and with the opposite fix.

`SynchronizationConvergence`'s documented meaning also widened: it now covers the round-loop identification (more than one enclosing loop, a loop below the top level, or a loop whose range is not `1..rounds`) as well as the predicate rule. No signature, type, or trait changed.

### Watched failing

Six perturbations, each reverted and the suite re-run green afterwards. All six produced the required failure.

| Perturbation | What failed |
| --- | --- |
| `RoundBoundary` requires `rounds` realizations instead of `rounds - 1` | all three loop-carried tests, the canonical body first |
| Drop `round.start != 1 \|\| round.end != rounds` from `round_body` | the `UnpeeledRoundLoop` row, which fell through to `ReductionContract` |
| Drop the peel half of `verify_anti_edge_is_ordered` | the `RoundBoundaryAtTheTail` row, which fell through to `ReductionContract` |
| Emit the round boundary at the *tail* of the round body | the lowering itself, with `Verification(UnorderedStagedRewrite)` |
| Drop the round term from `emit_partition_contributor` | only `the_loop_carried_body_matches_the_reference_at_its_declared_order`; the verifier stayed green |
| Drop the `rounds > 1` arm of `verify_cooperative_loops` | the lowering itself, with `Verification(ReductionContract)` |

The hand-built perturbation table is its own evidence and is driven as a test rather than by hand: `each_loop_carried_synchronization_rule_refuses_its_own_defect` builds a near-canonical loop-carried body six ways — round boundary at the tail, absent, duplicated in the peel; a `0..rounds` round loop; a fence inside a top-level loop that is not the round loop; and a staged read ahead of the round body's own phase boundary — against a control that reaches `ReductionContract`.

### The conformance evidence, and its boundary

`the_loop_carried_body_matches_the_reference_at_its_declared_order` *runs* the kernel through a lane-stepping interpreter that reads only the structured kernel IR, and compares bit patterns against an oracle written from the declared arithmetic. The interpreter flattens any barrier-containing loop into its iterations so lanes can be advanced to a barrier inside the round loop; every lane executes a whole segment before the next lane starts it, so a read of a slot whose writer had not reached it, or a rewrite of a slot whose readers had not passed it, surfaces as a wrong result. `the_declared_round_grouping_is_what_the_agreement_is_evidence_about` pins that the participant-major grouping computes something else on both rows, so the agreement is not vacuous. `the_cooperative_body_matches_the_reference_at_its_declared_order` runs the already-trusted single-round body through the same machine, which is what anchors the machine itself.

**Measurement boundary.** This is exact-arithmetic conformance at one fixture — `[2, 6] -> [2]`, three participants, one contributor each, two rounds, `f32` — executed by an interpreter on the host. Nothing here was compiled or dispatched on a device, no Metal emission of a multi-round body exists, and no performance claim is made. `tiler-compiler`'s own `KirMachine` still splits only the top level and cannot execute a multi-round body; that duplication is filed as [`share-one-structured-kernel-interpreter`](share-one-structured-kernel-interpreter.md).

**Unsupported cases, unchanged.** `CooperativeLoweringShape` still refuses more than one staging allocation, more than two phases, a staged span other than one-slot-per-participant write and whole-set read, a commit range not starting at participant zero, and — new — a tile whose visibility and anti-dependency edges are not exactly one each. The log-depth tree stays absent: it needs a per-access active-participant subset and a round-varying span, neither of which this provides. The `tiled` contraction's schedule, its `K` precondition, and its Metal emission stay with [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md), which is no longer blocked by the body.

### Verification

Run in the ticket's worktree: `cargo check -p tiler-ir --all-targets --locked`; `cargo nextest run -p tiler-ir --locked` (**641 passed, 0 skipped**); `cargo nextest run --workspace --locked` (**2,281 passed, 7 skipped, 0 failed**, three times, before the flake below surfaced) and the deterministic pair that replaces it, `cargo nextest run --workspace --locked -E 'not (package(tiler-runtime) and binary(identity_join))'` (**2,268 passed, 7 skipped**) with `cargo nextest run -p tiler-runtime --locked --test identity_join --test-threads 1` (**13 passed**); `cargo test --workspace --doc --locked` (8 passed, including the compile-fail routing evidence); `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`; `cargo clippy -p tiler-ir --all-targets --locked -- -D warnings`; the `make lint` invocation `cargo clippy --workspace --all-targets --locked --exclude tiler-prototype-run --exclude tiler-prototype-compile --exclude tiler-prototype-candle -- -D warnings`; `cargo fmt --all --check`; `ticketsplease lint` (`ok: no problems found`); `git diff --check` (clean).

**One thing the workspace clippy reports and this change did not cause.** Without the prototype exclusions the run fails inside `tiler-prototype-run` on `err_expect` and three siblings — style lints on that package's own test source. The exclusion is the `Makefile`'s documented policy and is untouched here; `git diff --name-only 2119b20..HEAD` lists nine files and none is under `prototypes/`. Stated as a derivation rather than as a base-commit measurement, which was not taken: the prototypes still build and still test, and the workspace `nextest` run above covers them.

`tkt guard tkt/lower-a-loop-carried-cooperative-body --base 2119b20 --ticket lower-a-loop-carried-cooperative-body` reports **10 changed files** whose directly affected scopes are `contracts/foundation, implementation/ir, project/tickets` — all three declared. Its `WARN` verdict is entirely reverse-dependency reach: every crate that depends on `tiler-ir` is listed transitively, which is true of any change to this crate, plus the ordinary `project/tickets` shared-scope note. No edited file lies outside a declared scope.

**An intermittent failure was hit along the way; it was root-caused rather than re-rolled.** `tiler-runtime::identity_join` cases die with `the build-time producer failed (signal: 9 (SIGKILL))`, empty stdout and stderr — a process-level kill of a subprocess, never a nonzero exit or a failed assertion. The harness starts one `cargo run --package tiler-build --example identity_join_producer` *per test case* and nextest runs those concurrently, so sibling invocations relink one example binary while another process is executing it. The separating experiment is one flag: the binary failed in **3 of 5** runs at default concurrency and **0 of 3** with `--test-threads 1`. `crates/tiler-runtime` is untouched here and is a live lane, so the defect is filed with that measurement and a ten-run closing criterion as [`stop-the-identity-join-producer-race`](stop-the-identity-join-producer-race.md) rather than absorbed. Nothing in this change can produce it: a semantic difference in `tiler-ir` surfaces as a nonzero exit or a failed identity assertion, not as a `SIGKILL` with no output.
