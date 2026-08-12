---
id: admit-subgroup-coordinates-and-xor-transfer-into-kernel-ir
title: Admit subgroup coordinates and exact XOR register transfer into the structured kernel IR
status: blocked
priority: p2
dependencies: [admit-subgroup-bindings-into-the-schedule-vocabulary, accept-adr-0094-subgroup-execution-tier, admit-guarded-output-tails-for-cooperative-contraction]
related: [design-the-subgroup-execution-tier, decide-the-subgroup-coordinate-binding-and-output-map, admit-shared-contributor-coverage-and-reduction-padding-identity, admit-an-atomic-subgroup-realization-subject-to-target-profiles, admit-lane-typed-values-and-masked-memory-into-the-kernel-ir, close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [kernel-ir, ir, metal, subgroup, execution-hierarchy, public-boundary]
---
## User-visible outcome

The structured kernel IR can read the exact workgroup/subgroup/lane coordinates a subgroup schedule states and move an ordinary F32 register value through the accepted XOR source relation. The verifier proves complete participation, source activity, tree order, and one-writer ownership without inventing a subgroup data type, barrier, or opaque reduction collective.

## Why now

**Historical filing defect, since repaired.** [`accept-adr-0094-subgroup-execution-tier`](accept-adr-0094-subgroup-execution-tier.md), anchor `The three implementation tickets this node claims to release now exist`, records that the acceptance node initially named implementation work that had not yet been filed. Its later closure, anchor `Four tickets depend on this node, not three`, verifies the eventual dependent population. [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md), anchor `Tickets filed`, preserves the proposal-era filing history. This ticket is the kernel-IR member of the repaired implementation set; the old “releases nothing today” statement is no longer current.

**Resolved 2026-08-01, then accepted at its exact KIR boundary on 2026-08-12.** [ADR 0094](../docs/decisions/0094-bind-a-subgroup-combine-to-a-register-transfer-tree.md) landed `accepted`, the schedule and coordinate decisions are accepted, and this ticket's exact KIR surface is accepted below. Implementation remains blocked until the schedule and shared guarded-load dependencies land; acceptance does not make an unimplemented producer executable.

**Fact — a shuffle needs no barrier, and that is the design's load-bearing negative result.** The 2026-08-01 addendum on [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md) records it from Metal Shading Language Specification 4.1 §6.10.2: "SIMD-group functions allow threads in a SIMD-group to share data **without using threadgroup memory or requiring any synchronization operations, such as a barrier**." A shuffle names its source lane and its destination register in one operation that is both the transfer and the ordering, so a shuffle-tree reduction derives no visibility edge, declares no synchronization point, and never reaches `barrier_call` at all. A design that routes a shuffle through a barrier would be wrong, not merely conservative.

**Inference — the reduction collectives are not the near-term construct, and the vocabulary must not pretend otherwise.** The subgroup tier record derives that subgroup *reduction* collectives are unusable for a separate reason — neither Metal nor WGSL states their combine order — so a collective admitted without a stated order would be a silently wrong result under an order-sensitive contract. The shuffle is admissible; the reduction collective is not, and refusing it explicitly is the correct outcome rather than a gap.

## Implementation keys

- Ordinary F32 SSA values plus the accepted `SubgroupShuffleXor` operation and checked mask; no subgroup-shaped value type.
- Governed workgroup-ordinal, subgroup-index, and subgroup-lane sources matching the accepted direct coordinate/output binding. A local-linear invocation index is never decomposed into these values by convention; absence of an exact backend realization is a typed refusal.
- `IndexEqual` expresses the exact result-lane ownership guard. The schedule still owns which equality may authorize the store.
- Stated combine order belongs to [`admit-subgroup-bindings-into-the-schedule-vocabulary`](admit-subgroup-bindings-into-the-schedule-vocabulary.md). This ticket admits only the kernel operations that realize that tree: explicit XOR shuffle plus ordinary arithmetic, never a reduction collective.
- The shared guarded load injects the schedule's typed padding identity into out-of-range contributor positions while every lane remains active.
- Identity encoding is additive at every site: appended tags only, no existing tag or field position moves, and the kernel identity domain does not step.
- If this widens `ExecutionScope` or `MemoryScope`, the existing tripwire `barrier_scope_vocabulary_is_closed` / `the_barrier_scope_vocabularies_are_still_closed` (landed by [`close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire`](close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire.md)) must be *updated* in the same change.

## Required failure-path evidence

Each observed failing against an accepted neighbour: an invalid or out-of-width XOR mask; an omitted, duplicated, or reordered mask step; a shuffle under divergent control; a non-F32 transferred value; a reduction collective relying on an unspecified hardware order; and a store guarded by anything other than the exact result-lane equality. Stated combine order on the topology remains the schedule ticket's obligation.

The coordinate binding adds three independent subjects: perturb the workgroup ordinal, subgroup index, and subgroup lane separately and observe the ownership or same-subgroup verifier reject each one. A lowering that substitutes `LocalInvocationIndex` for any of them must fail its exact source/identity check rather than produce an approximate mapping.

## Non-goals

Schedule bindings (`admit-subgroup-bindings-into-the-schedule-vocabulary`, this ticket's dependency). Target profile declarations (`declare-metal-subgroup-realization-facts-in-the-target-profile`). MSL emission. The two-level subgroup-to-workgroup composition, which the ADR excludes and [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md) owns — that composition needs workgroup visibility for a staged handoff between subgroups ([ADR 0096](../docs/decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md) decision 7) and does **not** fire [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md), whose trigger is a subgroup-private scratch tile (writer *and every reader* in one subgroup). A shuffle-tree reduction itself derives no `MemoryScope::Subgroup` (ADR 0094 decision 2 / MSL §6.10.2); this ticket must not fire that deferred work by accident. Any performance claim.

**Correction — 2026-08-10.** An earlier Non-goals clause claimed the two-level composition "is also the construct that fires `add-subgroup-memory-scope-when-collectives-land`." That clause was false: ADR 0096 decision 7 and the deferred ticket's rewritten trigger assign MemoryScope::Subgroup to a subgroup-private scratch tile, not to the composition. The Implementation keys line citation `:396` and the "land that tripwire first" obligation were also stale (anchor is the lane-identity sentence; the tripwire already exists). Combine-tree stated order was reassigned to the schedule sibling so this ticket no longer duplicates that failure path.

## Closes when

The constructs are admitted, every obligation above is checked by a check observed failing, the identity encoding is exhaustive, the record's worked examples are constructible with the verdicts it states, and every public shape has gone to Tom rather than been self-accepted.

## Source-first decision audit — 2026-08-12

The original packet is not a correct public boundary at the current base.

1. **False — a subgroup register value is not a new KIR data type.** [`KernelType`](../crates/tiler-ir/src/kernel/model.rs), anchor `The resolved type of one structured-kernel SSA value`, classifies the value's scalar representation (`F32`, `Index`, and the other bounded scalar roles). ADR 0094 decision 2 instead says that a shuffle moves the value from one invocation's register into another invocation's register and returns an ordinary destination register. The execution relation belongs to the operation and its governed coordinate sources; making `Subgroup<F32>` a type would encode where a scalar came from as what scalar it is, duplicate every arithmetic operation over a second type family, and leave unclear whether the post-shuffle result is still “subgroup-typed.”
2. **Verified — no barrier or scope widening is required.** ADR 0094 decisions 2 and 8 require explicit shuffles plus ordinary arithmetic, derive no visibility edge, and refuse opaque reduction collectives. Existing [`ExecutionScope::Subgroup`](../crates/tiler-ir/src/kernel/model.rs) is barrier vocabulary and [`MemoryScope`](../crates/tiler-ir/src/kernel/model.rs) intentionally has no subgroup arm. This ticket must touch neither.
3. **Imprecise and overbroad — a caller-supplied arbitrary source lane is not the first admitted transfer.** The accepted target subject is exactly `SubgroupTransfer::InRangeXorShuffle`, and the only admitted schedule tree derives masks `1, 2, 4, …, width / 2`. A generic source-lane SSA operand would represent arbitrary cross-lane permutations and would require symbolic range, same-subgroup, uniformity, and activity proofs no accepted schedule consumes. The XOR mask is the exact source relation: `source(lane) = lane xor mask`.
4. **Verified — three distinct governed coordinates are required.** [`decide-the-subgroup-coordinate-binding-and-output-map`](decide-the-subgroup-coordinate-binding-and-output-map.md), anchor `Accepted decision — 2026-08-11`, requires the workgroup ordinal, subgroup index within that workgroup, and lane index within that subgroup as direct sources. [`Builtin`](../crates/tiler-ir/src/kernel/model.rs) currently has only global and local linear invocation indices. Substituting or decomposing `LocalInvocationIndex` would violate the accepted no-relation boundary.
5. **Missing public prerequisite — arbitrary accepted result lanes need equality.** The accepted ownership relation is `lane_index == result_lane`, but [`CompareOp`](../crates/tiler-ir/src/kernel/model.rs), anchor `A predicate-producing comparison`, contains only `IndexLessThan`. Prefix comparison can select lane zero but cannot express an arbitrary accepted result lane. A general exact `IndexEqual` comparison is smaller and less duplicative than a subgroup-specific “is result lane” operation, and schedule refinement still decides which equality may authorize a store.
6. **Missing implementation prerequisite — the general padded example needs a value-producing guarded load.** The accepted contributor-coverage boundary keeps every lane active and injects the exact typed identity for out-of-range contributor positions. Current `Load` is unconditional and `Predicated` returns no value. The already accepted [`admit-guarded-output-tails-for-cooperative-contraction`](admit-guarded-output-tails-for-cooperative-contraction.md), anchor `Accepted decision — 2026-08-12`, owns the scalar `GuardedLoad { predicate, buffer, offset, bounds, inactive }` needed here. Subgroup lowering must consume that one operation with the schedule-stated `ReductionPaddingIdentity`; it must not invent a subgroup-only masked load or reuse the contraction ticket's unobservable `+0.0` filler. This dependency is now explicit.
7. **Verified — existing KIR construction and verification have no subgroup path.** [`OperationKind`](../crates/tiler-ir/src/kernel/model.rs), [`KernelBuilder`](../crates/tiler-ir/src/kernel/builder.rs), `verify_signature`, `guard_values`, `visit_block`, `verify_reduction`, and `derive_canonical` contain no subgroup transfer or coordinate arm. The final body-refinement equality is useful but insufficient by itself: the intrinsic verifier must first prove full participation, exact mask sequence, source activity, ownership, and the schedule/KIR coordinate correspondence.
8. **Imprecise identity claim.** Append-only growth remains the right result, but operation tag `0x1f` is already accepted for `GuardedLoad` and is not implemented yet. The subgroup operation takes the next free tag at its landing base; it does not reserve a numeric tag now. `Builtin`, `CompareOp`, and `OperationView` each append distinct tags/payloads while preserving old bytes. If that remains true, `tiler.kernel.v7` does not step and no artifact schema changes because artifacts already carry the canonical kernel identity rather than decoding the operation tree.
9. **Stale readiness claim.** The schedule surface is no longer awaiting Tom's decision. Its contributor-coverage and coordinate/output boundaries are accepted, and the schedule ticket is blocked only on implementation. This KIR ticket is now awaiting the exact public KIR decision below, while implementation remains dependency-ordered.

## Recommended exact public boundary

Retain ordinary scalar SSA types and add only the operation and coordinate relations the accepted schedule can consume.

- Add governed `Builtin::WorkgroupOrdinal`, `Builtin::SubgroupIndex`, and `Builtin::SubgroupLane`, all producing `KernelType::Index`. They are three MECE provenance subjects, not projections of `LocalInvocationIndex` and not one optional tuple.
- Add `CompareOp::IndexEqual`. It compares two `Index` values and produces `Bool`; the subgroup lowering compares the admitted lane builtin with the topology's literal result lane. The whole-kernel verifier still refuses any equality predicate that is not the exact schedule-derived ownership guard.
- Add a checked `SubgroupXorMask` newtype over `u64`: construction requires a nonzero power of two, with no target preset, default, or independent policy cap. Add one scalar operation, `OperationView::SubgroupShuffleXor { value, mask: SubgroupXorMask }`. `value` and the result are ordinary `F32` SSA values. The mask is encoded directly; whole-kernel verification additionally requires `mask < width` and the exact ascending sequence derived from the schedule. The operation means that each active lane receives the operand value from `lane xor mask` in the same subgroup. There is no caller-supplied source-lane SSA expression.
- Keep every shuffle at a structurally convergent program point. The verifier rejects one inside any predicated region, any omitted/duplicated/reordered mask, a mask outside the width, a non-F32 first-slice value, an unrecognized coordinate source, incomplete workgroups, and any body that does not end with exactly the `result_lane`-guarded owning store.
- Build the tree from `SubgroupShuffleXor`, ordinary `F32Add`, and the existing exact NaN-canonicalization operation. Do not add a horizontal reduction operation or let a backend choose a combine order.
- For identity-padded coverage, use the shared scalar `GuardedLoad` before the shuffle sequence. Its inactive value is exactly the schedule-stated and intrinsically proved `ReductionPaddingIdentity`. Every lane remains active; no shuffle is masked.

The first surface is deliberately F32/XOR-specific because that is the only accepted target subject and the only derived numerical evidence. A future BF16, integer, shuffle-down, ballot, or arbitrary-source transfer receives its own operation-specific spelling and target subject after its semantics and active-lane relation are accepted. `KernelType`, `ExecutionScope`, and `MemoryScope` do not widen.

Canonical lowering emits the three required builtins, checked index arithmetic for `output_ordinal = workgroup_ordinal * subgroups_per_workgroup + subgroup_index`, guarded contributor loads where coverage is padded, the derived ascending XOR steps, and one store guarded by `subgroup_lane == result_lane`. Verification compares the complete body with the exact schedule and independently checks the subgroup obligations before body equality. A target or backend that cannot realize any builtin or transfer declines the complete physical implementation; it never substitutes local-linear arithmetic, an opaque reduction collective, or another width.

## Ranked options

1. **Ordinary scalar values plus exact F32 XOR shuffle, three governed coordinates, and index equality.** Best correctness and fail-closed behavior, smallest truthful surface, and best maintainability. It represents exactly the accepted schedule/target subject, creates no meaningless type states, performs `log2(width)` shuffle/add steps, and allocates no KIR step vector.
2. **Generic subgroup shuffle with an arbitrary source-lane SSA operand.** Potentially correct only with a substantially larger symbolic same-subgroup/range/uniformity/activity verifier. It supports unaccepted permutations and narrowing trees, adds index-XOR or equivalent source construction, and gives no runtime advantage for the admitted butterfly. Defer until a second accepted transfer needs it.
3. **A new subgroup-shaped `KernelType`.** Reject. It confuses scalar representation with cross-invocation provenance, duplicates arithmetic and total-map handling, and does not by itself prove where a shuffle reads.
4. **An opaque subgroup reduction operation.** Reject. Its combine order is unspecified by the target authorities, so it cannot refine the accepted schedule or numerical contract.
5. **Backend-inferred coordinates/tree or local-linear decomposition.** Reject. It removes meaning from verified KIR identity and silently chooses a relation the accepted model explicitly leaves undefined.

## Strongest counterpoint and reversal evidence

An arbitrary-source shuffle is more expressive and could avoid a later public operation for `shuffle_down`. That flexibility is real, but today it creates representable programs whose safety and numerical order the verifier cannot prove and whose target subject does not claim support. Reverse to it only after an accepted non-XOR schedule supplies a concrete second consumer and a bounded proof shows how arbitrary source expressions establish same-subgroup range, uniformity, source activity, tree order, and target realization without fallback. Until then, exact XOR is both safer and more future-proof: future operations append without changing the meaning or bytes of this one.

## Required decision evidence

- Prove ordinary F32 values survive load → guarded padding → XOR shuffle → add without any execution-scope type conversion.
- Exercise `threads_per_workgroup == width` and `2 * width`, deriving distinct subgroup indices and output ordinals with one writer each.
- Perturb workgroup ordinal, subgroup index, lane, result lane, mask value, mask order, transfer value type, and padding identity independently; each must reach a named refusal.
- Put a shuffle under a lane-varying predicate and show the source-activity/convergence refusal. Omit one lane's inactive value and show that complete participation fails rather than masking the lane.
- Replace one governed coordinate with `LocalInvocationIndex` and show exact builtin/refinement refusal.
- Attempt an opaque reduction collective and a generic cross-subgroup source; neither must be constructible under the first surface.
- Perturb every new tag and assert identity inequality while all pre-existing kernel identity pins remain byte-identical.

## Decision request — 2026-08-12

Accept the exact scalar-value/XOR-transfer boundary above; revise one included construct; or keep subgroup KIR unavailable. Acceptance chooses the public KIR vocabulary and verifier relation. It does not authorize schedule implementation, target declarations, backend emission, two-level composition, non-XOR transfers, another arithmetic type, a subgroup memory scope, or performance claims.

## Accepted decision — 2026-08-12

Tom accepted the recommended exact scalar-value/XOR-transfer boundary in the live Codex coordination thread by replying `okay agreeed, next decision`. The relay source is Tom's direct response in that thread.

Structured KIR retains ordinary scalar SSA types and gains exactly the three governed coordinate builtins, `IndexEqual`, checked `SubgroupXorMask`, and scalar F32 `SubgroupShuffleXor` described above. Whole-kernel verification binds them to the exact accepted schedule width, ascending mask sequence, complete subgroup participation, direct coordinate sources, result-lane ownership, and typed reduction padding identity. The shared scalar `GuardedLoad` supplies padded contributor values; it is not duplicated as subgroup masked memory.

No subgroup-shaped `KernelType`, arbitrary source-lane expression, opaque reduction collective, barrier, subgroup memory scope, local-linear decomposition, additional arithmetic type, backend inference, emission, or performance claim is accepted. Future transfer operations append only after their own schedule, activity, numerical, target, and real-consumer evidence exists.

This ticket moves to dependency-blocked implementation state. Its implementation must consume the landed schedule and guarded-load authorities rather than reconstructing either in parallel.
