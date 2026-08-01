---
id: implement-the-single-workgroup-synchronized-reduction-strategy
title: Implement the single-workgroup synchronized reduction strategy
status: in-progress
priority: p1
dependencies: [admit-the-first-typed-synchronization-point-and-atomic-target-authority]
related: [implement-parallel-reduction-strategies]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-wg-strategy
lease_expires_at: 1785595402
---
## User-visible outcome

A bounded reduction can select one verified single-workgroup tree schedule whose staged dataflow, synchronization point, accumulation dtype, order, and numerical permissions agree.

## Implementation keys

Build only on the accepted cooperative dataflow and synchronization receipt. Define the tree topology, active lanes at every phase, tail handling, workgroup storage, accumulation dtype, and deterministic contributor order. Tree reassociation requires reassociation permission; a nondeterministic or atomic arrival order additionally requires contributor-permutation permission.

Keep serial and single-workgroup alternatives together. Missing synchronization authority, insufficient workgroup resources, divergent convergence, or withheld numerical permission rejects before executable-frontier admission rather than receiving an arbitrary cost.

## Required evidence

Power-of-two, uneven-tail, one-element, and empty extents agree with the reference. Independent reassociation/permutation mutations reject the exact affected strategy. Every phase reaches the exact synchronization point uniformly, and every workgroup read is visibility-covered. Identity changes with topology, accumulation, point, and resource realization.

## Closes when

The target-neutral synchronized alternative is verified beside serial, exact public drafts are reviewed by Tom, every check is mutation-proved, and targeted tests/Clippy plus the batch gate pass. Metal support remains downstream.

## Graph maintenance

- Keep this ticket after the typed synchronization authority, which itself follows cooperative workgroup dataflow; do not collapse those proof stages into backend lowering.
- Release Metal realization only after the target-neutral program and its exact synchronization/resource requirements are independently verified.
- Leave cost calibration downstream so legality and feasibility cannot be manufactured by a preference constant.

## Outcome

Delivered. A bounded reduction whose contract permits reassociation now has a verified single-workgroup tree schedule enumerated beside the serial reduction and the multi-pass split, with its topology, active lanes, tail policy, workgroup storage, accumulation dtype, contributor order, and arrival order all stated on the schedule rather than read off an emitted body. Its executed result agrees with the reference at its own declared order. No Metal support was promoted and no target profile gained a capability claim.

### The topology, and the eliminations behind its depth

**The tree, level by level.** One workgroup per output position, `participants` invocations in it. Level 0: every participant serially folds the contiguous contributor range its partition owns and stages the partial in its own slot. The synchronization point. Level 1: the one committing participant folds the `participants` staged slots in ascending order and performs the owning write. Active lanes are `participants` then `1`; the narrowing is stated by `CooperativeTile::commit`, not by a span, because every phase must stay uniformly *reached* for the point between the levels to be convergent.

**Depth two, not `log2(participants)` — and the elimination is structural rather than a shortcut.** The cooperative landing recorded that a tree *rewriting* one slot across rounds is refused by one-writer-per-slot. Writing fresh slots per round is not the way around it: `CooperativeTile::addressed_slots` enumerates `0..participants.count`, and `verify_cooperative_tile` passes the *tile's* participant range, so a `StagedSpan` is addressed by every participant and a write phase writes `participants * count` slots however few lanes are meant to be working. Round 1 of a halving tree would need to write `P/2` slots and cannot; and the coverage rule then requires a writer for every slot the allocation declares, so no padding escapes it. A log-depth tree therefore needs a **per-access active-participant subset**, distinct from a phase's `participation` (which is arrival and must stay uniform), plus a per-lane guard inside the phase in the emitted body. That is a widening of the accepted dataflow vocabulary and of the lowering shape, it is not needed for a single-workgroup tree schedule, and it is recorded in `cooperative.rs`'s module doc and in `docs/compiler/fusion-and-scheduling.md` rather than left to be rediscovered. **Reproduce the elimination in one line:** `crates/tiler-ir/src/schedule/cooperative.rs:addressed_slots` iterates the participant range it is given, and `crates/tiler-ir/src/schedule/builder.rs:verify_cooperative_tile` gives it `tile.coordinates.participants` for every write and every read.

**Tail handling: exact or declined.** `ContributorPartition::covers` is required, so a contributor count with no balanced split is withheld by the strategy with its extent rather than padded with identity elements or masked. A masked lane would also break the landed body's soundness argument, which rests on every launched invocation reaching the staged store.

**One-writer-per-slot resolution:** the delivered topology works *within* the accepted profile. Each participant writes exactly one slot once, in one phase; no lifetime or edge model was relaxed, and no accepted rule was weakened.

### The one new IR fact, and why it is not decoration

`ReductionTopology::CooperativeWorkgroup` gained `arrival: ContributorArrival`. Without it, the ticket's own rule — "a nondeterministic or atomic arrival order additionally requires contributor-permutation permission" — is unstatable: the topology merely *records* the contract's permutation resolution, so nothing distinguishes "deterministic and therefore permutation-free" from "nobody checked". Three variants; only `AscendingParticipant` is admitted, and `NondeterministicArrival` and `AtomicAccumulation` are statable and refused **twice, in an order that keeps both branches live**: `CooperativeTileRule::ArrivalPermission` when the contract withholds permutation, and `CooperativeTileRule::UnadmittedArrival` when it grants it, because only `SynchronizationKind::ControlBarrier` is admitted and neither arrival has a construct that could order it. Collapsing the two would make a permitted-but-unrealizable arrival report a numerical refusal a caller could not act on.

`tiler_ir::schedule::workgroup_tree_tile(participants)` is the one definition of the canonical tile. It builds the staging, the two phases and their spans, then derives the point's subject from the tile's *own* `visibility_edges()` through `required_subject` — so a producer cannot state a subject the handoff does not require. `the_canonical_tree_tile_is_the_fixture_every_rule_was_driven_against` asserts it equals the fixture the cooperative landing's whole perturbation table was driven against, which is what makes reusing it safe rather than a second shape that merely also verifies.

### Where the four rejections happen, and why none is a cost

| Failure | Decided by | Outcome |
| --- | --- | --- |
| withheld reassociation | the contract, before a region exists | `StrategyDeclined { NumericalPermissionRefused { numerics.reassociation } }` |
| no exact split | the request's extents, before a region exists | `StrategyDeclined { NoAdmissibleShape { extent } }` |
| insufficient workgroup memory | the capability authority | `Infeasible { local-memory-bytes, required, available }` |
| declared unrealizable | the atomic synchronization fact | `Unsynchronizable`, carrying the whole subject and the refusing profile |
| **nothing declares it** | the absence of a fact | `SynchronizationUndeclared`, carrying the subject and **no** profile |
| divergent convergence | the schedule verifier | `cooperative-phase-participation`, before any target is consulted |

**The undeclared case was a real defect, not a new spelling.** Before this ticket, `assess_resources` collapsed every `FeasibilityOutcome::Unknown` into `ResourceVerdict::Unknown`, which `assess_region` mapped to `PhysicalError::Intrinsic { rule: "target-assessment-unresolved" }`, which the frontier classifies as `FrontierError::MalformedProposal` — **failing the entire enumeration and blaming the provider for the target's silence.** Mutation-proved: reverting the new arm makes `each_way_the_tree_can_fail_rejects_before_admission_with_its_own_reason` fail with exactly that `MalformedProposal`. `ResourceVerdict::UnrealizedSynchronization`, `PhysicalError::UnrealizedSynchronization`, and `FrontierRejection::SynchronizationUndeclared` (appended tag `8`) carry it instead, and the explain half needed nothing new: `SynchronizationOutcome::Undeclared` already existed and already dispositions as `DeferredUnsupported` rather than a refusal.

The `SynchronizationUndeclared` encoding deliberately writes no profile key after the subject. There is no declaring profile, and an empty slice there would give silence the shape of a refusal by an unnamed authority.

### The four extents, executed rather than inspected

`KirMachine` — the backend-shaped interpreter that reads only the structured kernel IR — became workgroup-aware: it derives the participant count from the kernel's own `StagingParameter`, gives `GlobalInvocationIndex` and `LocalInvocationIndex` different values (they coincided before, which is why no second field existed), holds one value map per lane, and **advances every lane of a workgroup to the barrier before any lane crosses it** by splitting the top-level operation list at each barrier. That split is the faithful model rather than an approximation precisely because the KIR verifier requires every barrier at block depth zero.

| Extent | Contributors | Split | Result |
| --- | --- | --- | --- |
| power-of-two `[1, 8]` | 8 | 4 × 2 | executed kernel equals `strict_partitioned_sum` bit for bit |
| uneven `[1, 6]` | 6 | 3 × 2 | equals `strict_partitioned_sum` bit for bit |
| prime `[1, 7]` | 7 | none | declined, naming extent 7 |
| one element `[1, 1]` | 1 | none | declined, naming extent 1; the serial alternative is retained |
| empty `[1, 0]` | 0 | none | declined, naming extent 0; the serial alternative carries the `+0.0` identity the landed zero-extent precedent proves |

**The conformance check is mutation-proved in two independent directions.** Removing the barrier segmentation makes the committing lane read an unwritten slot and the test fails with `0x7fc00000` against `0`. Swapping the oracle's split to the neighbouring `(2, 4)` makes it fail with `4.0` against `5.0`. The second one is only possible because the input was chosen to be *regrouping*-sensitive rather than merely cancelling — an alternating `±1e20` input sums to the same value under every balanced split, and `the_declared_split_is_what_the_agreement_is_evidence_about` is the guard that fails if a later edit makes the comparison vacuous again.

### Reference-crate disposition: no widening, and the refusal stands

**Fact.** `ReferenceNumericalConformance::from_realization` still refuses `ReassociationPermitted` (`crates/tiler-reference/src/conformance.rs:189`), unchanged. **Fact.** `tiler_reference::strict_partitioned_sum` — the "second exact oracle" the multi-pass landing added — evaluates exactly `partitions` contiguous partitions of `contributors_per_partition` each, folded in ascending partition order, which *is* the single-workgroup tree's order. So conformance runs through the order-specific oracle at the region's own declared split, which is what a reassociating contract permits checking: the contract admits a result set, and what a plan is checked against is the one order it selected. No `tiler-reference` change was warranted and none was made; the refusal's own test at `conformance.rs:410` is unchanged and passing.

### Identity, per domain

| Domain | Step | Why |
| --- | --- | --- |
| `tiler.schedule.v3` | **unchanged** | `arrival` is appended at the *end* of the `0x35` cooperative arm, after both permission bytes. Every earlier field of that arm keeps its offset, so the append is checkable by inspection rather than by trusting that no cooperative region was persisted. `0x35` is itself an appended tag no earlier region could carry. |
| `tiler.kernel.v6` | **unchanged** | `ResourceRequirements` did not change; the arrival is a schedule fact, not a resource one. |
| `tiler.artifact-program.v14` | **unchanged** | Same reason. |
| target-profile descriptor `v11` | **unchanged** | No profile declaration changed; the widened profile is `#[cfg(test)]` and mints its own value. |
| explain record vocabulary | **unchanged** | `SynchronizationOutcome::Undeclared` already existed. The new `FrontierRejection` tag `8` is an appended tag in a rejection encoding that feeds no artifact. |

**Which domain carries what:** topology, accumulation, and the point all live inside the appended `0x35` payload and therefore in the schedule domain; the resource realization lives in `ResourceRequirements.synchronization`, which the synchronization landing already put into the kernel and artifact domains and which this ticket did not touch. **Nothing inserted, and it is checked rather than asserted:** `cargo nextest run --workspace` is 2158 passed / 0 failed with **no pinned value rebaselined** — `STRICT_F32_REGION_IDENTITY_HEX`, the governed descriptor bytes in `physical.rs`, `metal_plan.rs`'s `ARTIFACT_IDENTITY` and `CACHE_SUBJECT`, the explain request qualifier, and the four Metal golden digests are all untouched and all passing.

### Two existing tests moved, both because a new fact is true

`every_wired_authority_emits_its_typed_explain_records`: `frontier.strategy-decline.v1` 1 → 2. The census fixture compiles under the strict contract, so the tree declines for the same reason the split does. The zero-synchronization assertions beside it are unchanged and still pass, which is the point — a program that proposes no cooperative region still emits no `target.synchronization*` row.

`the_frontier_retains_the_split_beside_the_serial_reduction`: previously asserted *no* rejections. The tree is now proposed for that subject and refused by the bounded profile on `local-memory-bytes` (required 8, available 0). The assertion was tightened rather than relaxed — it now matches that exact single rejection — and it is the ticket's "insufficient workgroup resources" evidence on the real pipeline path.

### What was deliberately not done, and the ticket it became

**The governed profile was not widened, and this is the report the brief asked to be prominent.** `TargetProfileBuilder::governed` declares `local-memory-bytes = 0` and declares nothing about synchronization, so the bounded prototype baseline rejects every cooperative region twice over. Raising those rows would be a *capability claim* the prototype authority has no evidence for; the nearest-looking precedent — raising `buffer-bindings` from two to four — is a different act, because that bound was raised to match what the request boundary already admitted and claimed no device resource. The positive path is therefore proven against `TargetProfile::workgroup_tree_target_for_test`, which says at length why it is test-only, and the question of what the baseline should guarantee is filed as `decide-the-prototype-baseline-workgroup-guarantees` (`awaiting-decision`, with both options and their exact costs stated). It is deliberately not folded into `realize-parallel-reduction-strategies-on-metal`: that ticket owns the *Metal* profile's authority, and the two answers are independent.

No log-depth tree, for the structural reason above. No Metal lowering, no artifact change, and `crates/tiler-build/src/metal_declaration.rs` still declares no synchronization row, so a cooperative region remains `Unknown` against the macOS profile.

**Maturity, per the four-claims discipline.** The strategy is a *tested guarantee* as a representation and a verification, and — through the KIR machine — as a *behaviour* claim bounded to that interpreter: its result equals the reference's declared-order oracle at two extents, and the interpreter is not a device. It is *implemented support* at the compiler layer: the frontier proposes it, three alternatives are retained together, and every failure path is typed. It is **not** a claim about any machine: no real target declares the realization it requires.

### Public drafts for Tom — not self-accepted

*`tiler_ir::schedule`* — new public enum `ContributorArrival` with methods `requires_permutation`, `tag`, `key`; new public function `workgroup_tree_tile`; new public field `ReductionTopology::CooperativeWorkgroup::arrival`; new variants `CooperativeTileRule::{ArrivalPermission, UnadmittedArrival}`.

*`tiler_compiler`* — every item below is crate-internal (`pub(crate)`) and is listed because it is a consequential call-site boundary rather than a published API: `physical::SINGLE_WORKGROUP_TREE_STRATEGY`; `physical::WorkgroupTreeUnavailable` and its `reason`; `physical::single_workgroup_tree_region`; `physical::PhysicalError::UnrealizedSynchronization`; `physical::ResourceVerdict::UnrealizedSynchronization`; `frontier::FrontierRejection::SynchronizationUndeclared`.

**The one behaviour change to an existing rule:** an assessment that is unknown *because of its synchronization subject* is no longer a `FrontierError` that fails the whole enumeration; it is a retained rejection. Every other unknown keeps the old behaviour.

### Verification

`cargo fmt --all`; `cargo clippy -p tiler-ir -p tiler-compiler -p tiler-reference --all-targets -- -D warnings`; `cargo nextest run --workspace` (2158 passed, 6 skipped); `make full` green end to end including `cargo test --workspace --doc`, `RUSTDOCFLAGS="-D warnings" cargo doc`, the release-profile runs, `ticketsplease lint`, and shellcheck. `git diff --check` clean. `tkt guard --base 26fe69f` exit 0.
