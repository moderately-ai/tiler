---
id: realize-the-strict-contraction-on-metal
title: Realize the strict contraction as a tiled Metal scheduled kernel
status: in-progress
priority: p1
dependencies: [admit-the-contraction-normative-reference, admit-the-first-typed-synchronization-point-and-atomic-target-authority, realize-the-contraction-through-the-appendable-direct-path, lower-a-loop-carried-cooperative-body]
related: [prototype-optimizer-conformance-gate, prototype-metal-runtime-proof, broaden-governed-physical-support-for-reassociated-programs, scope-einsum-contraction-support]
scopes: [implementation/compiler, implementation/ir, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, language-model]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785634941
---
## User-visible outcome

One contraction of the workload's projection structure compiles to a Metal kernel whose results are bit-identical to the reference evaluator at the profile's own extents — the realization the L3 elimination left standing, rather than the fastest one.

## Which realization, and why that one

**Measurement — from the [L3 realization record](../docs/research/scheduling/first-metal-contraction-realizations.md).** Six realizations were measured. The `tiled` kernel — 16x16 threadgroup-memory tiles over the two free indices and contiguous chunks of the contracted index, with each thread still folding its own output in ascending `d` — is attributed uniquely to `strict_fold+ftz` over an eight-case corpus with the other twenty-one topologies refuted, is byte-identical to the `direct` kernel at all six workload cells, and is 2.6x to 4.3x faster than it at prefill. It consumes no numerical permission.

The `simdgroup_float8x8` and `MPSMatrixMultiplication` routes are eliminated under the governed contract by measurement, not by cost; the split reductions consume permissions this profile does not grant. Do not substitute one of them to make a number better — [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) forbids exactly that, and the L3 record states the measured price of not doing it.

## Exact blocker, which this ticket owns

**Fact — a two-input program cannot reach the compiler.** `crates/tiler-compiler/src/request.rs` rejects any program whose `input_count() != 1` at lines 1840 and 1977, and `check_recognized_operation_cover` requires the recognized operations to exhaust the reachable graph. A binary contraction fails at the first check. `broaden-governed-physical-support-for-reassociated-programs` is the precedent for widening this correctly: it generalized recognition around verified semantic occurrences rather than forcing a new shape into `NormalizedSerialSum`, and it added a checked physical representation instead of reusing one that denotes different arithmetic. Follow that shape.

**Fact — the Q-SEM-015 planning gate's stated conditions are met.** `prototype-optimizer-conformance-gate`, `prototype-metal-aot-slice`, and `prototype-metal-runtime-proof` are all `done`. The remaining limit is the recognizer above, which is this ticket's work rather than a reason to wait.

## Required delivery

- Request recognition, an index-access lowering capability for the contraction occurrence, a `ScheduledKernel` carrying the tiled schedule, structured-kernel verification, and program assembly — extended together, so every retained alternative covers the exact semantic program.
- **The tile precondition is a typed refusal, never a pad.** The tiled schedule requires `K` a positive multiple of its tile width. Every contracted extent in this profile — 1024, 2048, 3072 — satisfies it, and a K-padding schedule would owe the neutrality proof [Numerical semantics](../docs/numerical-semantics.md) requires, because `+0.0` is the strict sum's empty result and is not its bitwise-neutral padding. Refuse rather than acquire that obligation.
- **The emission must not lower a per-contributor step to a fused multiply-add.** The governed strict and flush-to-zero contracts forbid ADR 0015 contraction and require `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`. **Measurement — the flag is not sufficient on its own**: the spike shows `simdgroup_multiply_accumulate` fusing under `-ffp-contract=off`, reproducing [finding 16](../docs/research/apple-targets/numerical-behaviour.md) at a new construct. The per-statement emission rule is what holds the line.
- Bit-comparison against the reference at all six of the L3 profile's correctness cells, with the retained `result_sha256` values as the drift check.

## Non-goals

Structures 2 and 3, the split alternatives, the matrix-instruction route, any opaque call, and any cost model. Each has its own ticket or is deliberately absent.

## Closes when

A contraction of the profile compiles through the ordinary entry point, its results are bit-identical to the reference at every profile cell, the `K` precondition refuses with a typed reason that was watched firing, and an emitted module carries no fused multiply-add on the contraction's accumulation path.

## Outcome

**Stopped before implementation, under this ticket's own stated stop rule.** The `tiled` realization is not reachable by widening recognition, lowering, scheduling, and assembly together in the way [`broaden-governed-physical-support-for-reassociated-programs`](broaden-governed-physical-support-for-reassociated-programs.md) did. It requires admitting threadgroup-local staging and intra-workgroup synchronization into the structured kernel IR — a vertical this repository deliberately retired rather than deferred — and admitting it soundly *inserts* a field into a fixed identity record rather than appending a tag. No code was changed; the derivation and the decomposition are below.

### The blocking finding, with the exact checks

**Fact — the structured-kernel verifier refuses synchronization unconditionally, and its own diagnostic says why.** `crates/tiler-ir/src/kernel/verify.rs:341` is `if walk.has_synchronization { return Err(KernelDiagnostic::UnexpectedSynchronization); }`, reached from `visit_block`'s `OperationKind::Barrier { .. } => walk.has_synchronization = true` at `verify.rs:277`. The diagnostic is documented at `crates/tiler-ir/src/kernel/error.rs:222` as "The kernel contains synchronization that *no schedule has authorized*" — the refusal names a missing authority, not an unimplemented case.

**Fact — no schedule can authorize it, because the requirement record has no synchronization dimension and the axis was deliberately removed.** `ResourceRequirements` (`crates/tiler-ir/src/schedule/model.rs:620-645`) carries `buffer_bindings`, `threads_per_workgroup`, `local_memory_bytes`, `requires_device_memory`, and the eight numerical dimensions. There is no synchronization field. `derive_requirements` (`schedule/model.rs:850-866`) hardcodes `local_memory_bytes: 0` and its doc states "the bounded profile stages no local memory and introduces no synchronization requirement". The target side agrees and records the removal as a decision: `crates/tiler-compiler/src/target/feasibility.rs:93-95` — "`v7` retires the invented numeric barrier-capacity axis. Tag `0x08` remains reserved, but a schedule with no synchronization now has no predicate to prove" — and `crates/tiler-build/src/metal_declaration.rs:28-31` — "**Synchronization** has no row at all… `replace-or-justify-the-barrier-count-axis` removed the axis rather than inventing a capacity."

**Inference — restoring it is a domain step in two identity domains, not an appended tag.** `push_requirements` (`crates/tiler-ir/src/kernel/model.rs:1276-1289`) writes `ResourceRequirements` as a *fixed, unframed field sequence* inside the kernel identity. A new field lands at a fixed offset with no tag and no length, so every kernel identity ever produced moves and every artifact identity folding one moves with it: `tiler.kernel.v5` → `v6` (`kernel/model.rs:49`) and the feasibility profile `v9` → `v10`. That is the insertion this ticket's brief says to stop on, and it is categorically different from the appends the recent landings made — `ScalarProgram` `0x26` (`schedule/model.rs:884`), `ReductionTopology` `0x33` (`schedule/model.rs:896`), `BinaryOp::F32Divide` `0x08` (`kernel/model.rs:284`), `UnaryOp::F32Rsqrt` `0x02` (`kernel/model.rs:396`), each of which left every earlier subject's bytes in place.

**Fact — four further verifier rules the `tiled` body contradicts structurally, independent of the identity question.** Read against the spike's kernel at `spikes/scheduling/metal_contraction_vertical/kernels.metal:85-145`:

| Rule | Site | What `tiled` does |
| --- | --- | --- |
| Buffer count is exactly `reads + 1` | `verify.rs:135` | needs two staging arrays behind no boundary `Access` |
| Every memory effect is dominated by the governed `invocation < work_items` predicate | `verify.rs:344` | stages under `m < m_extent` and must let masked lanes reach the barrier |
| Exactly one store per invocation | `verify.rs:371` | `2·(K/16)` threadgroup stores before the one device store |
| A reduction admits exactly one read access and exactly one contributor loop with `start == 1` | `verify.rs:403`, `verify.rs:434-443` | two operands, three loops |

**Inference — the `AddressSpace::Workgroup` arm at `verify.rs:185` is dead code, not a seam.** It admits a workgroup buffer when `derived.local_memory_bytes > 0`, but `derive_requirements` never produces a nonzero value and `verify.rs:135` would reject the extra buffer first; the Metal emitter independently refuses the space at `crates/tiler-metal/src/emit.rs:552-554`, because workgroup storage binds through `[[threadgroup(N)]]`, a namespace disjoint from the `[[buffer(N)]]` ordinals that `VerifiedKernel::declared_buffers` (`kernel/model.rs:750-761`) documents as positional. Re-basing those ordinals on a filtered count would change what an existing signature position means — an insertion into a positional ABI contract, again not an append.

**Inference — the missing piece is an evidence class, which is the deeper reason this is not a widening.** Every `Load` is authorized by a `BoundsWitnessId` and every `Store` by an `OwnershipWitnessId`, both resolved against the region's boundary accesses (`verify.rs:350-364`). A tile read is authorized by neither: its correctness is the cooperative-staging invariant "every element I read was written by a lane of my workgroup and separated from my read by a barrier". `OwnershipProofKind` has one variant, `OneGlobalInvocationPerOutput`, and `BoundsProofKind` two, all single-tensor and all derived from a boundary `LogicalAccess`. Tiler has no cross-invocation visibility proof, and inventing one is a new validation authority.

### The graph already owns this blocker, and this ticket is missing the edge

**Fact — the refusal is a recorded decision with a named successor, not an unbuilt case.** [`replace-or-justify-the-barrier-count-axis`](replace-or-justify-the-barrier-count-axis.md) is `done`, and its implementation keys say: "Preserve `BarrierSpec` as a typed KIR reservation, but reject every current barrier intrinsically as `UnexpectedSynchronization`: the current schedule owns no identity-bearing synchronization point, phase, placement, participant set, visibility contract, or convergence proof to which the operation could be matched." It then assigns the successor explicitly — "The first real nonzero synchronization path is split into [`admit-the-first-typed-synchronization-point-and-atomic-target-authority`](admit-the-first-typed-synchronization-point-and-atomic-target-authority.md). That ticket must introduce the complete schedule obligation and one atomic provenance-bearing target realization together; independently asserted component facts are not composable evidence."

**Fact — the cooperative-staging evidence class derived above is already filed, and its statement of the gap matches this one independently.** [`represent-cooperative-workgroup-reduction-dataflow`](represent-cooperative-workgroup-reduction-dataflow.md) (`todo`, no dependencies) opens: "The current schedule has only a global-linear one-output mapping. KIR exposes boundary reads and one write, has no usable workgroup allocation or local-invocation coordinate, and rejects synchronization. Adding a barrier to that program is either semantically redundant or divergent under predication; it cannot prove cooperative execution." It owns the participant set, local coordinates, workgroup storage shape/alignment/lifetime, phases, and uniform reachability — exactly the obligations a 16×16 staged tile consumes.

**Inference — so the correction is a graph edge, not a new ticket.** The chain `represent-cooperative-workgroup-reduction-dataflow` → `admit-the-first-typed-synchronization-point-and-atomic-target-authority` → [`implement-the-single-workgroup-synchronized-reduction-strategy`](implement-the-single-workgroup-synchronized-reduction-strategy.md) already exists and is all `todo`. This ticket declares `dependencies: [admit-the-contraction-normative-reference]` and nothing else, which is why the board offered it as ready. **Recommended edge, left for the coordinator to apply because it is a scheduling decision:** add `admit-the-first-typed-synchronization-point-and-atomic-target-authority` to this ticket's dependencies, or split the appendable half out per item 1 below and let this ticket carry the synchronization dependency alone.

### Why no substitute was made

**Inference.** `direct` is attributed to the same `strict_fold+ftz` topology, is byte-identical to `tiled` at all six cells, and *is* expressible in the present vocabulary — one guarded store, one contributor loop seeded at the first product, which is exactly what `verify_contributor_loop`'s `start == 1` already encodes. Substituting it was rejected on two grounds. [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) and this ticket forbid swapping the named realization, and a swap to make the *work* smaller is worse than one to make a number better. And it would silently drop a required deliverable: the `K ≡ 0 (mod 16)` refusal is `tiled`'s own precondition, and `direct`'s preconditions are "none beyond `K ≥ 1`" ([the L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md), realization table), so a `direct` delivery has nowhere to put the typed refusal this ticket requires and would have reported a green result for a check that could never fire.

A 16×16 output-tile *thread mapping* reading operands straight from device memory was also considered and rejected: it is numerically identical but is not the measured realization — the entire 2.6×–4.3× prefill result comes from the staging — it carries no `K` precondition either, and claiming its performance would exceed the measurement.

### A second blocker, independent of the first

**Measurement — the reference evaluator refuses four of the six correctness cells, so the required drift check is not reachable as specified.** `MAX_REFERENCE_TENSOR_ELEMENTS` is `16 * 1024 * 1024` (`crates/tiler-reference/src/lib.rs:90`), and `contract_operands` refuses when `output_count * contracted_count` exceeds it, under `IterationStepsExceeded` (`crates/tiler-reference/src/contraction.rs:450-456`). Recomputing the profile's cells against that bound:

| Cell | Outputs | Fold steps | Verdict |
| --- | --- | --- | --- |
| `w_decode_kv` | 1,024 | 1,048,576 | admitted |
| `w_prefill_q` | 20,480 | 20,971,520 | refused, 1.2× the bound |
| `w_prefill_mlp_in` | 393,216 | 402,653,184 | refused, 24× |
| `w_prefill_mlp_out` | 131,072 | 402,653,184 | refused, 24× |
| `w_prefill_o` | 131,072 | 268,435,456 | refused, 16× |
| `w_vocab_slice` | 8,192 | 8,388,608 | admitted |

No operand or output tensor exceeds the element bound; only the fold's step count does. So "bit-comparison at all six cells against the reference evaluator" is today a two-cell claim plus four typed refusals. That is a separate decision — raise the work bound, admit a bounded windowed oracle, or restate the deliverable as the spike's retained `result_sha256` values — and it should not be settled inside an implementation ticket.

### Proposed decomposition

Ordered; the first is independently deliverable and is what unblocks the board.

1. **Admit the two-input contraction through governed recognition and lowering, realized as `direct`.** The named blocker — `request.rs:2223`'s `input_count() != 1` — plus a `NormalizedContraction` beside `NormalizedPointwise`, an eighth `GovernedIndexAccess` with a binary `[f32, f32] -> [f32]` signature (`crates/tiler-compiler/src/governed.rs:206`), a `ScalarProgram` variant at appended tag `0x27`, a `LogicalAccess` contraction-contributor variant at appended tag `0x05`, the two-read widening of `verify.rs:403`, and single-region program assembly. Every step is an append; nothing here needs synchronization, threadgroup storage, or a new evidence class. This is the recognizer-widening job the ticket described, and it delivers a contraction that compiles through the ordinary entry point and is bit-identical to the reference where the reference can answer.
2. **Decide the reference work bound for contraction oracles.** Owns the table above and states which of the six cells the reference is expected to answer.
3. **The existing synchronization chain**, in its own dependency order: `represent-cooperative-workgroup-reduction-dataflow`, then `admit-the-first-typed-synchronization-point-and-atomic-target-authority`. Between them they own the cooperative-staging evidence class, the `ResourceRequirements` insertion, the `tiler.kernel.v5 → v6` and feasibility `v9 → v10` domain steps, the reinstated target-profile synchronization row (reserved tag `0x08`), and the `[[threadgroup(N)]]` binding namespace. Nothing needs to be filed; the second of the two already requires Tom's review of its consequential public changes.
4. **This ticket, reduced to realizing `tiled` on top of 3** — the staged schedule, the `K ≡ 0 (mod 16)` typed refusal, and the emission evidence — and only then retiring `direct` as the prefill path if the measurement holds on the merged tree.

### Verification run

No source file was modified, so the package gates have nothing new to check. `tkt lint` reports `ok: no problems found`; `git diff --check` is clean; `git status` shows only this ticket file. The two blocking claims are each reproducible in one command: `sed -n '341,343p' crates/tiler-ir/src/kernel/verify.rs` for the unconditional synchronization refusal, and `sed -n '450,456p' crates/tiler-reference/src/contraction.rs` with `sed -n '90p' crates/tiler-reference/src/lib.rs` for the work bound.

## Outcome — 2026-08-01, resumed with both blockers landed

**Stopped again, on a strictly narrower blocker, after landing the emission half.** The first stop record above is preserved and its central claim has expired: the synchronization authority it waited on exists, and a staged, fenced Metal kernel now compiles and links. What still blocks `tiled` is one modelling gap the cooperative module already names as unmodelled — a staging allocation reused across rounds. That is filed as [`admit-loop-carried-cooperative-staging`](admit-loop-carried-cooperative-staging.md); this ticket resumes on it.

### What landed

**Metal emission of cooperative kernels.** The synchronization landing added the KIR constructs; nothing emitted them. Before this change `crates/tiler-metal/src/emit.rs` refused `Builtin::LocalInvocationIndex` (`builtin_parameter`'s wildcard), refused `AddressSpace::Workgroup` (`address_space_declaration`), and had no `StagedStore`/`StagedLoad` arm, so every one fell to `UnrecognizedOperation` — **no cooperative kernel could reach MSL at all**, including the tree strategy's. Now:

- `LocalInvocationIndex` declares as `uint … [[thread_index_in_threadgroup]]`. `builtin_declaration` previously read the *launch* index's selected type and attribute for whatever builtin it was given; that was unreachable only because the local index was refused outright, so admitting it is exactly what made the shared read wrong. It is now `builtin_declared_type` plus a per-builtin attribute, with the asymmetry stated: the global index reads a `LaunchIndexRealization` because MSL admits several spellings and a caller records its choice; the local index has one admitted spelling here and therefore no selection. **No public boundary moved** — `MetalEmissionRealization` is unchanged, deliberately, because adding a field would force edits in `tiler-build` and a spike, both outside this ticket's scopes.
- Workgroup staging declares *inside* the entry point (`threadgroup float tg0[3];`), never as a parameter. A `[[buffer(N)]]` position would re-base every later ordinal and change what an existing signature position means; `workgroup_staging_takes_no_argument_table_position` pins that the boundary tensors keep buffers 0 and 1 and that no `[[threadgroup(` binding is emitted.
- `StagedStore`/`StagedLoad` emit subscripted assignments and loads carrying their tile phase **as a comment, never as a guard** — the phase is schedule-side evidence the verifier already resolved, so emitting it as control flow would add a run-time test for a proven fact whose failure would silently skip a staged write.

**Two new goldens, both compiler-validated.** `cooperative_workgroup_reduction.metal` (the tree realization of a `[2, 6] -> [2]` strict sum, three participants) and `contraction_strict_tensor.metal` (the `direct` path, which had a fixture kernel and an FMA test but no golden and therefore no compile evidence). `every_checked_in_golden_is_compiled_by_this_module` forces both into the offline-driver list.

### Why `tiled` is still not expressible — the derivation, with reproducible checks

The measured kernel (`spikes/scheduling/metal_contraction_vertical/kernels.metal:96-145`) reuses two 16×16 `f32` allocations across `K/16` rounds, with two barriers per round.

| What it needs | What refuses it | Where |
| --- | --- | --- |
| Rewrite a slot in a later round | one writer per slot across the whole tile | `crates/tiler-ir/src/schedule/builder.rs:1192-1204`, `cooperative-staging-conflict` |
| Order round `r+1`'s write after round `r`'s read | only producer-to-consumer edges are derived; a point discharging nothing is `RedundantPoint` | `cooperative.rs:371-402`, `builder.rs:1323-1327` |
| A barrier inside the round loop | barrier refused at nonzero block depth | `crates/tiler-ir/src/kernel/verify.rs:400-405`, `SynchronizationConvergence` |

The span question the brief flagged is *not* the blocker and resolves cleanly: reassigning which thread loads which element makes both writes `stride = 1, offset = 0, count = 1`, and both reads are the documented whole-set `stride = 0` shape. **Fact — `cooperative.rs:41-47` already states the real gap** and calls it unmodelled, as the same reason `workgroup_tree_tile` is depth two rather than log-depth: one missing capability, two blocked consumers.

**Measurement — unrolling is not a way around it.** Per-round allocations at the profile's own extents:

| `K` | Rounds | Phases | Slots | Threadgroup bytes |
| --- | --- | --- | --- | --- |
| 1024 | 64 | 128 | 32,768 | 131,072 |
| 2048 | 128 | 256 | 65,536 | 262,144 |
| 3072 | 192 | 384 | 98,304 | 393,216 |

against `MAX_COOPERATIVE_PHASES = 64` and `MAX_COOPERATIVE_STAGING_SLOTS = 65,536` (`schedule/mod.rs:207`, `:205`) and the 32,768-byte row the widened test profile declares (`target.rs:3575`). Every cell breaks the phase bound; `K = 3072` breaks the slot bound; all break memory by 4×–12×. And the L3 record's 2.6×–4.3× advantage was measured on a 2 KB-resident kernel, so a 128 KB variant's performance would be a fabricated claim.

**Inference — item 3 above may be a narrowing rather than new vocabulary.** `SerialLoopSpec` carries `start`/`end` as `u64` *literals* (`kernel/model.rs:685-692`), so every invocation runs an identical trip count and a barrier in that body *is* convergent; the walk already tracks `loop_depth` apart from `block_depth`. Recorded in the new ticket rather than acted on: it is a synchronization soundness rule, and items 1 and 2 still stand.

### Why the stop rather than the widening

Item 2 is a new evidence class — the vocabulary can state no write-after-read obligation at all — which is a validation authority and touches `VisibilityEdge`, `WorkgroupStaging`, `SynchronizationRule`, and `CooperativeTileRule`, all public. Item 1's natural spelling is a per-round lifetime field on `WorkgroupStaging`, and `push_workgroup_staging` (`schedule/model.rs:1605-1611`) writes a fixed unframed field sequence, so that **inserts** and steps `tiler.schedule.v3`. `push_cooperative_tile`'s own comment justifies its earlier extension on the premise that no cooperative region was ever encodable into a retained identity — **a premise the tree-strategy landing expired**, since a cooperative region now reaches a verified kernel, executes, and has a checked-in Metal golden. Under this ticket's standing rule an insertion is a stop, and under [AGENTS.md](../AGENTS.md) the boundary is Tom's.

### Deliverables from the ticket's own list

| Required | Status |
| --- | --- |
| Tiled schedule as a retained alternative | **Not delivered.** Blocked above; no `ReductionTopology` variant added, no `0x36` tag consumed. |
| `K ≡ 0 (mod 16)` typed refusal | **Not delivered.** It is `tiled`'s own precondition and has nothing to attach to. Deliberately not shipped as a check that could never fire — the discipline `no_k_multiple_refusal_exists_on_the_direct_path` already records for `direct`. |
| No FMA on the accumulation path | **Already held**, by `the_contraction_kernel_emits_no_fused_multiply_add_on_its_accumulation_path` from the direct-path landing; now additionally under a compiled golden. |
| Bit-identity at all six cells | **Not delivered.** It is a property of the tiled schedule's results, and there is no tiled schedule. The staged oracle is ready for whoever resumes. |
| Emission through KIR barrier/staged constructs | **Delivered**, with goldens and toolchain evidence. |
| Roadmap row | **Delivered**, minimally. |

### Verification

`cargo nextest run --workspace`: 2,232 passed, 6 skipped. `cargo clippy -p tiler-metal --all-targets -- -D warnings`: clean. `tkt lint`: ok. Golden compilation resolved a real toolchain — metal 32023.883 / metallib 32023.883, macOS SDK 26.5 build 25F70 — and linked all six fixtures, `cooperative_workgroup_reduction.metal` at 4,131 bytes and `contraction_strict_tensor.metal` at 3,923.

**Watched failing.** Perturbing `LOCAL_INDEX_ATTRIBUTE` to `thread_position_in_threadgroup` failed `cooperative_workgroup_reduction_matches_its_golden_source` and `the_cooperative_kernel_emits_storage_a_local_index_a_handoff_and_a_fence`, then was reverted and the suite re-run green. `a_cooperative_golden_without_its_staging_is_rejected_when_a_toolchain_resolves` is a *permanent* teeth-test rather than a transient perturbation: it deletes the `threadgroup float tg0[3];` declaration from the checked-in fixture and requires a metal-stage `ToolFailure`, which is what makes that fixture's compile evidence non-vacuous — without it, an emitter that declared no storage would stay green once the golden was rebaselined. The barrier-omission and fence-inside-the-guard paths were already driven and are cited rather than duplicated: `BodyChange::NoFence` → `UndischargedVisibility` and `BodyChange::FenceInsideTheGuard` → `SynchronizationConvergence` (`crates/tiler-ir/src/kernel/tests.rs:2298-2306`); the second is the rule that blocks a loop-carried barrier.

### Coordination

The roadmap edit is confined to the two sentences my landing falsifies — the `tiled` blocker sentence and the closing pointer. **It deliberately does not touch** the row's "four cells uncompared" claim or its `bound-the-reference-contraction-comparison-for-the-profile-cells` pointer, which the audit found stale and which `correct-the-roadmap-rows-falsified-by-the-contraction-and-accuracy-landings` (B1) owns in full. If B1 lands first, expect a textual conflict in exactly that cell and resolve toward B1's rewrite while preserving the `admit-loop-carried-cooperative-staging` pointer.

**Public drafts for review, both in `tiler-metal` (ADR 0074 §7 draft boundaries, no new public item):** the emitted MSL spelling of workgroup staging (a function-scope `threadgroup` array named `tg{StagingId}`, not a `[[threadgroup(N)]]` binding) and of the local index (`uint … [[thread_index_in_threadgroup]]`, fixed rather than a `MetalEmissionRealization` selection). Both are emission-surface choices a consumer reads; neither adds a public type or changes a signature.
