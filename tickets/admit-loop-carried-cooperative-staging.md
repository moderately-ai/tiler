---
id: admit-loop-carried-cooperative-staging
title: Admit loop-carried cooperative staging so a reused tile is expressible
status: in-progress
priority: p1
dependencies: []
related: [realize-the-strict-contraction-on-metal, represent-cooperative-workgroup-reduction-dataflow, implement-the-single-workgroup-synchronized-reduction-strategy]
scopes: [implementation/ir, implementation/metal, implementation/build, contracts/artifacts, contracts/navigation, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [research, physical-planning, contraction]
claimed_from: todo
assignee: worker-loop-staging
lease_expires_at: 1785610106
---
## Scope note — 2026-08-01

Three documentation scopes were added when the identity step was taken, and each names the exact files it was added for.

- `contracts/artifacts` for `docs/artifact-abi.md`: the identity ledger and the evolution rationale, which the `tiler.schedule.v3 -> v4` step must move in the same commit.
- `contracts/navigation` for `docs/status.md` and `docs/roadmap.md`. The dispatch brief placed `docs/status.md` under `contracts/artifacts`; `ticketsplease.toml` puts it under `contracts/navigation` with the roadmap, and the roadmap's contraction row asserted three refusals this change removes.
- `implementation/metal` for the six `crates/tiler-metal/goldens/*.metal` fixtures and `implementation/build` for `crates/tiler-build/src/metal_plan.rs`. Neither carries a behaviour change: the goldens' MSL bodies are byte-identical and only the two identity-digest comment lines and the derived entry-point name move, and `metal_plan.rs` changes two recorded digests and the paragraph explaining them. They are in this commit because the identity-step bar requires every pinned identity to be recomputed with the step that moves it, and a golden left stale is a red gate for whoever lands next.
- `contracts/foundation` for `docs/ir.md`, one sentence: the barrier-admission rule this change narrows. The rest of that section's cooperative claims were already stale before this ticket and are filed as [`correct-the-ir-contract-cooperative-synchronization-claims`](correct-the-ir-contract-cooperative-synchronization-claims.md) rather than absorbed.

## User-visible outcome

A cooperative tile can stage into one fixed allocation, hand it off behind a barrier, and then *reuse the same slots* for a later round — the shape every blocked-tile GPU kernel has and the one `CooperativeTile` deliberately does not model. Until it exists, the L3-selected `tiled` contraction is unstatable at any extent the pinned workload uses.

## Why this is filed as its own node

[`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) stopped here a second time, on a blocker strictly narrower than its first. The synchronization authority it originally waited on has landed in full: `CooperativeTile`, `SynchronizationPoint`, the KIR `Barrier`/`StagedStore`/`StagedLoad` constructs, the single-workgroup tree strategy, and — at `c81e5c2`'s successor — Metal emission of a staged, fenced kernel that compiles and links on the Apple toolchain. What remains is one modelling gap, and it is already named in the source that has it.

**Fact — the module states the gap and calls it unmodelled.** `crates/tiler-ir/src/schedule/cooperative.rs:41-47`: "A tile that rewrote one slot across several rounds — a logarithmic tree — is statable in this vocabulary but is refused: [`CooperativeTile`] admits one writer per slot, because a second write to a live slot needs a per-round lifetime and a per-round visibility edge that this profile does not yet model." The same paragraph records that this is what caps `workgroup_tree_tile` at depth two rather than `log2(participants)`.

So one missing capability now blocks two independent consumers — the log-depth tree and the tiled contraction — which is what makes it a node rather than a line item inside either.

## The three things missing, each with its exact check

**1. Slots are single-assignment across the whole tile.** `verify_cooperative_tile` builds one occupancy map spanning every phase and refuses a second write to any slot (`crates/tiler-ir/src/schedule/builder.rs:1192-1204`, `CooperativeTileRule::StagingConflict`, rule id `cooperative-staging-conflict`). Coverage is checked over the same map (`builder.rs:1205-1210`), so the two are one statement: the participants' writes are a bijection onto the allocation's slots. A round-reusing tile violates it by construction.

**2. Only producer-to-consumer edges are derivable.** `CooperativeTile::visibility_edges` (`cooperative.rs:371-402`) emits an edge only where `producer.id < consumer.id`, i.e. read-after-write. A reused tile also needs the *anti*-dependency — round `r+1`'s write must not overtake round `r`'s read — and a point declared to order it discharges no derived edge, so `verify_synchronization` rejects it as `SynchronizationRule::RedundantPoint` (`builder.rs:1323-1327`). This is a new evidence class, not a new field: the vocabulary can state no write-after-read obligation at all.

**3. A barrier may not sit inside a loop.** `verify_synchronization` refuses any barrier at nonzero block depth (`crates/tiler-ir/src/kernel/verify.rs:400-405`, `KernelDiagnostic::SynchronizationConvergence`), documented at `verify.rs:361-363` as "A barrier inside a predicate or a loop is reached by a dynamic subset of the participants — undefined execution rather than unsupported." **Inference — the rule is sound for a predicate and conservative for a loop.** `SerialLoopSpec` carries `start` and `end` as `u64` *literals* (`crates/tiler-ir/src/kernel/model.rs:685-692`), not values, so every invocation of a workgroup executes an identical trip count and a barrier in that body is reached by all of them at the same dynamic instance. The walk already tracks `loop_depth` separately from `block_depth` (`verify.rs:642-673`), so the distinction the sound rule needs is present and merely not used here. This is the one of the three that may be a narrowing of an existing check rather than new vocabulary — establish that before assuming it.

## Why unrolling is not the way around it

**Measurement — the numbers, at the pinned workload's own extents.** The `tiled` realization uses two 16×16 `f32` allocations, 2,048 bytes total, reused across `K/16` rounds. Giving each round its own allocations to satisfy rule 1 above:

| Contracted extent | Rounds | Phases needed | Staging slots | Threadgroup bytes |
| --- | --- | --- | --- | --- |
| 1024 | 64 | 128 | 32,768 | 131,072 |
| 2048 | 128 | 256 | 65,536 | 262,144 |
| 3072 | 192 | 384 | 98,304 | 393,216 |

Against `MAX_COOPERATIVE_PHASES = 64` and `MAX_COOPERATIVE_STAGING_SLOTS = 65,536` (`crates/tiler-ir/src/schedule/mod.rs:207`, `:205`), and against the 32,768-byte `LocalMemoryBytes` row the widened test profile declares (`crates/tiler-compiler/src/target.rs:3575`; the *governed* baseline declares 0, `target.rs:1686`). Every cell exceeds the phase bound; K=3072 exceeds the slot bound; all three exceed the memory row by 4× to 12×.

**Inference — and the performance claim would be fabricated anyway.** The [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) attributes `tiled`'s 2.6×–4.3× prefill advantage to the staging, measured on a kernel holding 2 KB resident. A 128 KB variant is a different kernel with different occupancy, so its numbers are unmeasured whatever a bound says.

## Scope

Owns the vocabulary and its verification: whatever states a per-round staging lifetime, whatever states a write-after-read obligation and what discharges it, and the barrier-convergence rule's treatment of a constant-trip loop. It does **not** own the tiled contraction schedule, its `K` precondition, or its Metal emission — [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) keeps those and resumes on this.

## The identity question this must answer first, and it is Tom's

**Inference — at least one of the three changes moves retained identity bytes, and that was not true before the tree strategy landed.** `push_workgroup_staging` (`crates/tiler-ir/src/schedule/model.rs:1605-1611`) writes a fixed, unframed field sequence, so a per-round lifetime field lands at a fixed offset with no tag and no length and moves every cooperative tile's bytes — stepping `tiler.schedule.v3`, and the kernel and feasibility domains that fold it.

`push_cooperative_tile`'s own comment (`model.rs:1676-1679`) argues the `0x35` payload was safe to extend because "no cooperative region has ever been encodable into a retained identity — the structured-kernel verifier refused every one before a kernel, program, artifact, or cache entry could hold it." **That premise has since expired.** `implement-the-single-workgroup-synchronized-reduction-strategy` carries a cooperative region through planning to a verified kernel and executes it (`the_tree_matches_the_reference_at_its_declared_order_for_every_extent`, `crates/tiler-compiler/src/pipeline/tests.rs:3765`), and this repository now checks in a cooperative Metal golden. Re-derive the premise against the tree at the time this is picked up rather than inheriting either answer.

Design the representation so the common case is an *append* if that is reachable — a distinct tag, or a lifetime expressed as something the encoder already frames — and take the domain step to Tom explicitly if it is not. Per [AGENTS.md](../AGENTS.md), the new evidence class in item 2 is a validation authority and a consequential public boundary (`VisibilityEdge`, `WorkgroupStaging`, `SynchronizationRule`, `CooperativeTileRule` are all public), so its shape is Tom's decision regardless of how good the derivation is.

## Closes when

A cooperative tile that writes an allocation, hands it off behind a point, and rewrites the same slots in a later round verifies; the anti-dependency is derived rather than declared and has exactly one discharging point; a body realizing it passes the structured-kernel verifier; each new rule has been watched refusing its own defect; and the identity consequence is recorded with whichever of an append or an accepted domain step it turned out to be.

## Outcome

**The vocabulary, both verifiers' rules, and the identity step landed; the *body* did not, and is filed as [`lower-a-loop-carried-cooperative-body`](lower-a-loop-carried-cooperative-body.md).** A cooperative tile now states how many times its phase sequence executes, rewrites its staging between rounds, derives the anti-dependency that creates, and requires exactly one point to discharge it — verified end to end at the schedule layer. No canonical body realizes one, for a fourth blocker this ticket's stop record did not name and which is derived below.

### The design, and the eliminations behind it

**Rounds live on the tile, not on the allocation, and no lifetime field exists.** The stop record inferred that "the natural spelling inserts a per-round/lifetime field into `push_workgroup_staging`". That is refutable. Every phase of a tile runs on every round, so an allocation written by a phase is written on every round *whatever* a lifetime-scope field claimed; the vocabulary cannot state "written once, read on every later round" at all, because that needs a phase that runs on some rounds and not others. `live_from`/`live_through` are therefore already within-round ordinals, round-scoping is a consequence of `rounds > 1` rather than a choice, and a scope field would be a second place to state what the structure determines — the same reasoning `push_cooperative_tile` already uses for not encoding the visibility edges. `WorkgroupStaging` and `push_workgroup_staging` are unchanged.

**The one-writer-per-slot rule needed no change, and that is the point.** `verify_cooperative_tile`'s occupancy map spans the phase sequence *once*, which under the round vocabulary **is** one round. So it still refuses two writers reaching one slot inside a round — where no point could separate them, both being on the same side of every boundary — and no longer refuses the rewrite between rounds. `overlapping_staged_writes_inside_one_round_are_still_refused` drives the surviving half on a two-round tile.

**Write-after-read is a second derived class, `AntiDependencyEdge`, and deliberately unencoded.** One edge per (allocation, reading phase, rewriting phase) triple, over *every* pair of phases regardless of their order — the rewrite is in the following round, so it follows every read of the current one however the phases are ordered. It is a distinct type rather than a `VisibilityEdge` with the ends swapped, because a swapped visibility edge would claim a value flows backwards and a reader would have to know which direction each instance meant. Like `VisibilityEdge` it is a total function of the declared structure and is not encoded: `push_cooperative_tile`'s existing argument covers it unchanged.

**Discharge is a disjunction where visibility is a conjunction.** A phase boundary `(p, p+1)` discharges an anti-dependency `(c, w)` when `p >= c` (it runs after round `r`'s read) *or* `p + 1 <= w` (it runs before round `r+1`'s rewrite); a round boundary discharges every one, and no visibility edge at all. A corollary worth recording: **no single point can discharge both classes**, since `w <= p` and `c >= p+1` contradict both disjuncts — which is why a multi-round tile always carries at least two points.

**`RedundantPoint` now ranges over both classes.** A point earns its place by discharging at least one edge of either kind; without that, a round boundary would be refused precisely for doing its job. The other side is driven too: a round boundary on a single-round tile discharges nothing and *is* redundant.

**Convergence is a new evidence class, `EveryParticipantExecutesEveryRound` (tag `0x03`, appended).** Reaching a point is not the same as reaching the same *dynamic instance* of it once the phases repeat, and the existing class does not carry the second fact. The derivation is that `CooperativeTile::rounds` is a declared `u64` literal rather than a value, so every participant runs an identical trip count by construction — the same argument `SerialLoopSpec`'s literal bounds supply for a contributor loop. `ConvergenceEvidence::required_for_rounds` is the single definition the verifier and any producer read, and the rule is an equality in both directions: the stronger class is refused on a single-round tile as an unearned claim.

**Placement gains `RoundBoundary` (tag `0x02`, appended), carrying no ordinals.** The phases it separates are the tile's last and first, which the tile already states; naming them would give one boundary two spellings. It is not a wrapped `PhaseBoundary`, because `preceding + 1 == following` is what makes a phase boundary a program point and a wrap would carve an exception into it. `preceding()`/`following()` became `Option<PhaseId>` — an `Option` rather than a defaulted ordinal, so every caller decides what "no ordinals" means instead of one of them silently comparing against phase zero.

**The KIR barrier rule narrowed from "block depth zero" to two facts.** No predicate may enclose it — `block_depth == loop_depth` is exactly "nothing on the path is a predicate", using two counters the walk already tracked separately — and the loop nesting must be the one the tile authorizes: one for a repeating tile, none otherwise. A loop is admissible where a predicate is not because `SerialLoopSpec`'s bounds are literals; a loop level still needs a *reason*, or a barrier inside a contributor fold would be admitted and would synchronize once per contributor for a point placed once between two phases.

**The contributor split gained the round factor.** `ReductionTopology::CooperativeWorkgroup`'s `contributors_per_partition` is now what one participant folds on *one* round, so the covered sequence is `partitions * contributors_per_partition * rounds` and participant `p` of round `r` owns the contiguous range at `r * partitions + p` — still ascending, still reassociation and never permutation. Without it a tile could declare repeating phases while its split accounted for one pass, and every contributor after the first round would be folded again. `ContributorPartition::covers` was deliberately *not* taught about rounds: it is the multi-pass split's rule and a second dimension would give one method two meanings.

### Both consumers, worked

**The 16x16 tiled contraction.** Two allocations, `K/16` rounds, two phases per round (stage, then consume), two points: the phase boundary for the publication and a round boundary for the rewrite. Storage stays at 2,048 bytes — the measured kernel's — instead of the 131,072-to-393,216 bytes the stop record measured for per-round allocations, and the phase count stays at two instead of 128 to 384 against a bound of 64. Everything this ticket owns is now expressible for it. What is not is the *topology*: the tiled contraction's participants own outputs rather than partition a contributor sequence and every participant commits, which `realize-the-strict-contraction-on-metal` keeps.

**The log-depth tree.** The per-access active-subset gap is **a second construct, not the same one**, and there is a third beside it. A `StagedSpan` is addressed by every participant of the tile *and* is the same span on every round, since a span carries no dependence on the round ordinal — so a tree needs both a per-access active-participant subset (separate from a phase's `participation`, which is arrival and must stay uniform for a round-boundary point to be convergent) and a span whose stride and count halve per level. The subgroup design's register argument does not apply: these are memory slots, and the coverage rule enumerates them. What this step *does* give the tree is that the rewrite is no longer the blocker — `workgroup_tree_tile`'s doc no longer blames the one-writer rule for its depth, because that is now false.

### The identity step, executed

**`tiler.schedule.v3` -> `v4`, and the appends-only spelling was eliminated by construction rather than by preference.** Both the `0x35` topology tag and the `arrival` byte after it rested on "no cooperative region has ever reached a retained identity", which the tree-strategy landing expired: a cooperative region now lowers to a verified kernel, has a checked-in Metal golden, and folds into an artifact identity and a cache subject. Once bytes are retained the question is whether an old identity can equal a new one, and adding eight bytes anywhere in that arm does not answer it. **The concrete near-collision:** the arm ends in a length-prefixed axis list of four-byte elements, so an old region with axes `[0, 1, 2]` encodes exactly the bytes a new region with axes `[2]` and three rounds does — the old length prefix `3` reads as the new `rounds`, the old first two axes read as the new length prefix `1`, and the remaining axis, order, accumulation, permission, arrival, and launch bytes line up. Only the verifier's requirement that a topology's axes repeat its access's separates the two, and an identity encoder leaning on a verifier invariant has stopped being injective on its own terms. A conditional append (write `rounds` only when it exceeds one) fails the same construction and adds a non-uniform encoding, so it was eliminated too.

**Every pinned identity, recomputed on this tree, with why it moved or held.**

| Pin | Verdict |
| --- | --- |
| `STRICT_F32_REGION_IDENTITY_HEX` (`crates/tiler-ir/src/schedule/builder.rs`) | **Moved**, in exactly eighteen bytes. The region stages nothing, so its payload never reaches the `0x35` arm; only the separator changed. The `v3` value is retained beside it as `STRICT_F32_REGION_IDENTITY_HEX_V3` and `the_round_step_moves_only_the_domain_separator` compares the two past the tag — a measured blast radius rather than an assurance. |
| Six `crates/tiler-metal/goldens/*.metal`, `scheduled region identity digest` | **Moved**, all six, for the separator. |
| The same six, `kernel identity digest` and the derived entry-point name | **Moved.** `tiler.kernel.v6` frames the region identity bytes whole, so its content moves; its grammar did not, so the domain holds. `git diff` confirms no MSL body line changed in any of the six. |
| `ARTIFACT_IDENTITY` (`crates/tiler-build/src/metal_plan.rs`) | **Moved**, `8cb1a5e2...e951b` -> `26a9cc27c7253fd3fad73b77014a96d87ec24b73eabb7c9fb79bc3db330e4cb4`. Artifact identity frames each entry's kernel-program identity, which frames the kernel identity, which frames the region identity. The program carries no cooperative tile at all and moves anyway — which is what a separator costs. |
| `CACHE_SUBJECT` (same file) | **Moved**, `d4493d47...c35bf` -> `7a6c7496fd523978f20a9de6b852f25bdec9d0a4639f04b807ca8808dd051a85`, because the subject frames the artifact identity. |
| `tiler.kernel.v6`, `tiler.kernel-program.v6`, `tiler.artifact-program.v14`, manifest 12.0 | **Held.** Each folds the identity below it *whole*, separator included, rather than re-deriving a subset, so a `v6` kernel over a `v4` region can never be confused with one over a `v3` region. Content moving through a fold is not a grammar change. |
| Explain request qualifier `a532d35f0cfdd29a` (`crates/tiler-compiler/src/explain.rs`) | **Held**, traced and confirmed green: the request subject folds the *compilation request*, not a scheduled region. |
| `GOVERNED` target-profile descriptor (`crates/tiler-compiler/src/physical.rs`), `FAMILY_ORDER_IDENTITY_FIXTURE` (semantic registry) | **Held.** Neither domain reaches a schedule. |
| `tiler.shape-env.v3`, `tiler.index-region.v9`, `tiler.semantic-*` | **Held**, below or beside the schedule rather than above it. |

The whole-workspace run before rebaselining named exactly seven failures — the six goldens and the one `tiler-build` test asserting both pins — which is the enumeration above and nothing else.

### Why no body landed, with the derivation

The ticket's "a body realizing it passes the structured-kernel verifier" is not reachable inside this scope, on a blocker independent of the three the stop record named.

**Fact — a predicated region produces no values.** `OperationKind::Predicated { predicate, body }` carries no results, and `emit_cooperative`'s own comment records the consequence.

**Fact — every boundary load must be dominated by the iteration guard** (`verify_effects`, `PredicateDominance`), and **exactly one store may commit, at loop depth zero** (`OutputCoverage`).

**Inference — an accumulator therefore cannot cross a round.** A round's contribution comes from boundary loads, so it is guarded; it must survive the round loop's back edge, so it must leave the guard; and it cannot. Staged accesses are *not* effects, which leaves one escape — put the guarded work and the unguarded staged fold in separate regions of one loop body — and that shape then meets a second obstacle: every fold here seeds at its first contributor rather than at the identity, because `+0.0 + x` is not `x` for `x = -0.0`, so a round loop seeded the same way peels round zero and realizes each declared point twice, which `verify_synchronization`'s once-each rule refuses and which `verify_edge_is_ordered`, written over a single fence position, cannot check.

That is a KIR vocabulary question — whether a predicated region may yield values — plus a realization-count rule and a cyclic generalization of the ordering checks. It is filed with the full derivation rather than absorbed. In consequence the KIR relaxation ships as **implemented support with no producer**, and that maturity claim is stated where it lives: `barrier_is_convergent` is exercised over its full truth table including both acceptance rows, `cooperative_plan` refuses a multi-round tile by name, and the doc says what remains.

### Watched failing

Eight perturbations, each reverted and the suite re-run green afterwards. Seven produced the required failure:

| Perturbation | Test that failed |
| --- | --- |
| Skip the anti-dependency discharge loop | `a_loop_carried_rewrite_with_no_round_boundary_is_refused` |
| Let a round boundary discharge visibility edges | `a_loop_carried_tile_rewrites_its_slots_and_verifies` |
| Restore the old `block_depth != 0` barrier rule | `the_barrier_convergence_rule_admits_only_the_nesting_a_tile_authorizes` |
| Stop encoding the round count | `every_cooperative_tile_field_separates_scheduled_region_identity` |
| Drop the round factor from the contributor coverage | `a_split_that_ignores_the_round_count_is_refused` |
| Accept any convergence class but the caller's assertion | `a_point_naming_the_wrong_convergence_derivation_is_refused` |
| Admit a zero or overlong round count | `a_round_count_outside_the_governed_profile_is_refused` |

**The eighth stayed green, and it is reported rather than hidden.** Removing `cooperative_plan`'s explicit `rounds > 1` refusal leaves `a_loop_carried_tile_is_representable_and_not_yet_lowered` passing, because the one-point destructuring below it already rejects every multi-round tile — no such tile can carry fewer than two points, by the disjunction/conjunction corollary above. The branch is kept, and `lower.rs` now states that it is currently unreachable, why the unreachability is a non-local fact, and that widening the destructuring to admit two points would silently admit a multi-round tile without it.

Two further teeth were sharpened by their own first failures rather than by weakening the rule: `a_round_boundary_without_a_following_round_is_redundant` initially tripped `ConvergenceEvidence` (it carried the round-loop derivation on a single-round tile), and `a_round_count_outside_the_governed_profile_is_refused` initially tripped `ContributorSplit`, which moved the round-structure check ahead of the arithmetic that multiplies by it so the defect names the right field.

### Verification

`make full` on the working tree: `cargo fmt --all --check`, `cargo check --workspace --all-targets --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo nextest run --workspace --locked` (**2,244 passed, 6 skipped**), `cargo test --workspace --doc --locked`, `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`, `cargo nextest run --release --locked -p tiler-reference -p tiler-compiler` (**776 passed, 2 skipped**), `ticketsplease lint` (`ok: no problems found`), `shellcheck --severity style deps.sh`. Exit 0. `git diff --check` clean.

### Public drafts for review

Each is a public item in `tiler_ir::schedule` and none is self-accepted.

- `AntiDependencyEdge` — the new evidence class, its three fields, and its direction convention.
- `CooperativeTile::rounds`, `CooperativeTile::anti_dependency_edges`, `CooperativeTile::anti_discharging_points`.
- `SynchronizationPlacement::RoundBoundary`, and `preceding()`/`following()` becoming `Option<PhaseId>` — a signature change on two existing methods.
- `ConvergenceEvidence::EveryParticipantExecutesEveryRound` and `ConvergenceEvidence::required_for_rounds`.
- `SynchronizationPoint::discharges_anti`.
- `SynchronizationRule::UndischargedAntiDependency`, `CooperativeTileRule::RoundStructure`, `MAX_COOPERATIVE_ROUNDS`.
- The reinterpretation of `ReductionTopology::CooperativeWorkgroup`'s `partition` as per-round, which changes what an existing public field means without changing its type.

### Filed along the way

- [`lower-a-loop-carried-cooperative-body`](lower-a-loop-carried-cooperative-body.md) — the body, with the derivation above.
- [`correct-the-ir-contract-cooperative-synchronization-claims`](correct-the-ir-contract-cooperative-synchronization-claims.md) — `docs/ir.md` still says "No such point exists" and that the structured-kernel verifier refuses any kernel whose region carries a visibility edge. Both were falsified by the synchronization and tree landings, not by this one; only the sentence this change falsified was corrected in place.
