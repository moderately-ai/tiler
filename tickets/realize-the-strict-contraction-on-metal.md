---
id: realize-the-strict-contraction-on-metal
title: Realize the strict contraction as a tiled Metal scheduled kernel
status: todo
priority: p1
dependencies: [admit-the-contraction-normative-reference, admit-the-first-typed-synchronization-point-and-atomic-target-authority, realize-the-contraction-through-the-appendable-direct-path]
related: [prototype-optimizer-conformance-gate, prototype-metal-runtime-proof, broaden-governed-physical-support-for-reassociated-programs, scope-einsum-contraction-support]
scopes: [implementation/compiler, implementation/ir, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, language-model]
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
