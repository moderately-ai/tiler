---
id: realize-parallel-reduction-strategies-on-metal
title: Realize parallel reduction strategies on Metal
status: in-progress
priority: p1
dependencies: [implement-the-target-neutral-multi-pass-reduction-strategy, implement-the-single-workgroup-synchronized-reduction-strategy, declare-a-required-gpu-family-in-the-artifact, construct-and-bind-the-first-authoritative-metal-compile-profile, compose-the-numerical-contract-from-its-decided-dimensions]
related: [implement-parallel-reduction-strategies]
scopes: [implementation/metal, implementation/build, implementation/runtime, implementation/artifact, contracts/artifacts, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-metal-reduction
lease_expires_at: 1785687056
---
## User-visible outcome

The accepted multi-pass and synchronized single-workgroup reduction programs lower, package, preflight, and execute on a qualified Metal host without inventing backend support from source syntax or successful compilation.

## Implementation keys

Map every target-neutral synchronization/storage/dispatch requirement through typed Metal facts and exact live-device or prepared-pipeline authorities at their real phases. Emit explicit workgroup memory and barriers only for verified points. Preserve multi-pass temporary lifetimes and command ordering through final device use. Reuse artifact route requirements and the one-way preparation/commit boundary; do not add Apple vocabulary to the neutral artifact.

Primary Metal documentation and retained host measurements must establish the supported realization. Compilation success is not a capability fact. An unavailable qualified host reports the missing environment rather than converting an unrun path into a guarantee.

## Required evidence

Both strategies execute against the reference on a qualified host, or the exact unavailable predicate is retained. Negative fixtures refuse missing family/feature authority, insufficient prepared capacity, insufficient local memory, and invalid synchronization realization before routing commit. Command-buffer terminal success precedes readback and asynchronous resources survive final use.

## Closes when

Metal lowering, artifact, build, and runtime paths agree with the target-neutral contracts; public backend/runtime boundaries are reviewed by Tom; every check is mutation-proved; and targeted tests/Clippy plus `make full` pass.

## Graph maintenance

- Follow both target-neutral strategies, backend-neutral route requirements, and the authoritative Metal compile profile explicitly; scope collision is not prerequisite evidence.
- Keep measured crossover and winner activation in `calibrate-and-activate-parallel-reduction-selection` after executable Metal evidence exists.
- Split a named hardware measurement when the qualified host is unavailable; do not convert compilation success into feature or performance evidence.

## Outcome

**Status: the profile-fact half is complete; the executable half is blocked on a compiler-side contract and is not this ticket's to unblock.** The qualified host was available and matched the ledger's execution-environment row in every field, so the blocker is not a missing environment.

### What landed

**Fact — the authoritative macOS Metal profile now carries its synchronization row.** `BoundMetalCompileDeclaration` declares one complete `SynchronizationSubject` — `ControlBarrier`, workgroup arrival, workgroup publication, threadgroup memory fenced and device memory deliberately not, acquire-release ordering — as `Realized` at `CompileProfile`, under a normative reference of its own, `apple.metal-shading-language.4-0.threadgroup-barrier`. Before this, `declare_synchronization_realization` had exactly one call site in the workspace and it was `#[cfg(test)]`; no production profile declared a synchronization row at all, which is what `TargetProfile::workgroup_tree_target_for_test`'s doc comment named this ticket as the owner of.

**Fact — four of the row's five dimensions are quoted normative facts and the fifth is a stated elimination.** MSL 4.0 §6.9.1 with Tables 6.12 and 6.13 establishes the kind, both scopes, and the fenced domain. It assigns the barrier no memory ordering at all: MSL declares `enum memory_order { memory_order_relaxed, memory_order_seq_cst }` and applies it to atomics and `atomic_thread_fence`, never to `threadgroup_barrier`. `Relaxed` is refuted by the specification's own "memory fence (for reads and writes)", `SequentiallyConsistent` is withheld as what the spec reserves for an explicit seq-cst fence, and `AcquireRelease` is what remains. The ledger records the split per dimension rather than presenting one authority for five.

**Fact — the permitted resolution of reassociation is declared, from the same retained measurement read for its other consequence.** `reassociation_chain` shows the compiler emitting no `reassoc` under `safe` and returning the source's own fold order. That answers both "can a contract forbidding regrouping be delivered?" (yes, none is added) and "can a contract permitting it be delivered?" (yes exactly — Tiler chooses the grouping, the source expresses it, the target runs that one). Declaring both resolutions of a permission dimension is `governed_target_honourability`'s own idiom and is not the exclusive-table shape the subnormal dimensions use.

**Fact — identity moved and was recomputed rather than assumed.** The canonical descriptor is 1,963 bytes, from 1,741, and both the standard Metal artifact identity and the cache subject moved with it; each was rebaselined from an observed run. The descriptor length is now pinned by a test naming the ledger, so the document's cited number cannot drift from the encoding.

### The blocker, located exactly

**Measurement — no registered numerical contract both flushes subnormals and permits reassociation**, so no parallel reduction strategy is expressible on the one measured Apple row. The four registered contracts are `tiler.strict-f32.v1`, `tiler.flush-f32.v1`, `tiler.relaxed-f32.v1`, and `tiler.reassociate-f32.v1`; the two granting regrouping are strict-based and require preserved subnormals, which this hardware measurably refuses in every math mode, and the one this hardware delivers grants no regrouping. `CompileRequest` accepts only that four-value preset enumeration, so no caller outside `tiler-compiler` can state the combination.

The refusal lands on `InputSubnormals` and never reaches the reassociation dimension, which is what makes it diagnosable. `no_registered_contract_both_flushes_subnormals_and_permits_reassociation` drives both halves — the regrouping contract refused on subnormals, and the deliverable contract retaining no split — and is the activation trigger.

**This was not routed around.** Registering a fifth preset is a `crates/tiler-compiler` change, and the compiler lane was occupied by `admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary` for this ticket's whole run. Filed as `register-a-flush-and-reassociate-numerical-contract` rather than absorbed.

### What is therefore unrun, stated as predicates rather than as gaps

Neither strategy executed against the reference; no negative fixture for insufficient prepared capacity, insufficient local memory, invalid synchronization realization, or missing family authority was driven on hardware; no lifetime or command-ordering evidence was produced. Every one of these is downstream of a plan existing, and no plan exists. **None of them is blocked by a missing target fact or by an unavailable host**, and converting the compiling cooperative golden into execution evidence would be exactly the substitution this ticket forbids.

### Scope note

`research/target-profiles` was added to this ticket's scopes: the brief required the ledger rows and no live sibling held that scope. `implementation/metal-aot` was **not** added — publishing new proof members from `prototypes/serial-sum-compile` would only have been useful once a plan exists.

### Verification

`cargo nextest run --workspace --locked` green at 2236 tests before the ledger edits; per-package `tiler-build` green at 69. Each new check was mutation-proved: removing `declare_synchronization_realization` fails both the descriptor-text test and the dimension sweep, and each of the row's five dimensions plus its verdict was separately perturbed and observed moving the descriptor.

## Correction 2026-08-02 — the blocker above is closed, and the executable half is dispatchable

**Fact.** *The blocker, located exactly* is now stale and must not be read forward. `NumericalContract` is composed from its dimensions rather than chosen from a four-value preset list, so subnormal flushing and ordered regrouping resolve independently and `NumericalContract::FLUSH_AND_REASSOCIATE_F32` is an ordinary statement — `crates/tiler-compiler/src/session.rs:1402`. `register-a-flush-and-reassociate-numerical-contract` is `closed`. Reproduce in one line:

```sh
rg -n 'pub const FLUSH_AND_REASSOCIATE_F32' crates/tiler-compiler/src/session.rs
```

**Fact — the positive successor exists and names both strategies.** `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` (`crates/tiler-build/src/metal_plan.rs:1006`) drives both halves of the old gap record first — the strict-based regrouping contract still refused on `InputSubnormals` rather than on the regrouping, and the flush-only contract still retaining no split — and then asserts that the composed contract's portfolio retains, beside the serial fold, both the multi-pass split and the single-workgroup tree. So a plan now exists on the authoritative Apple profile.

**Inference.** *What is therefore unrun* is unchanged in content but no longer blocked: it stated every unrun predicate was "downstream of a plan existing, and no plan exists". A plan exists. The remaining work is the executable half only — both strategies against the reference on a qualified host, the four negative fixtures driven before routing commit, command-buffer terminal success preceding readback, and asynchronous resources surviving final use.

**Boundary.** The profile-fact half stays complete and is not reopened. `correct-the-declined-strategy-record-for-an-unsplittable-reduction` still owns the sub-four-contributor `InvalidCompilerOutput` defect; size fixtures above it rather than around it.

## Outcome of the executable half — 2026-08-02

**Measurement — both strategies executed against the reference on the qualified host, and the host matched the ledger's execution-environment row in every field.** macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max, Apple9 — the four fields the ledger tabulates — under Xcode 26.6 (17F113), offline compiler `Apple metal version 32023.883`, toolchain `nightly-2026-07-19`. Procedure: `cargo run -p tiler-prototype-compile -- --out <path>` then `cargo run -p tiler-prototype-run -- --artifact <path>`. At a `1x4` shape under `NumericalContract::FLUSH_AND_REASSOCIATE_F32` the portfolio retained three alternatives and each was emitted, linked through `xcrun`, dispatched, and compared bit for bit against `tiler-reference`:

| Alternative | Encoders | Widest workgroup | Threadgroup memory | Result |
| --- | --- | --- | --- | --- |
| serial fold | 2 | 1 | 0 B | `41700000` |
| single-workgroup tree | 2 | **2** | **16 B** | `41700000` |
| multi-pass split | **3** | 1 | 0 B | `41700000` |

All three equal the reference `41700000` (15.0). **The strategy labels are carried by device-reported evidence rather than by this binary's classification**: the tree is the only alternative whose compiled pipeline reserves threadgroup memory and whose declared workgroup exceeds one thread, and the split is the only one encoding three ordered stages. Compilation success was never converted into a capability claim — each strategy ran.

**Measurement — the shape is `1x4` and both bounds are forced.** Four contributors because `governed_partition` splits nothing below that; one row because the split's pointwise stage launches one invocation per element and the profile's grid-axis row admits four, so two rows fail `target.grid-axis` before any plan composes.

**Measurement boundary, stated rather than implied.** The operands are `1.0, 2.0, 4.0, 8.0`. Every grouping of them is exact in `f32`, which is what makes one serial-fold oracle valid for all three strategies under a contract that *permits* regrouping — and every subset sum is distinct, so a dropped, double-counted, or unsynchronized contributor cannot cancel. What is proved is that each strategy reduces the declared contributor set correctly. **Regrouped rounding was not observed and is not claimed**; a grouping-sensitive operand set would make a strategy legitimately disagree with a serial reference, and that belongs to a numerics ticket rather than here.

**Fact — the local-memory refusal existed only as a document and now exists as code.** `crates/tiler-artifact/src/program/requirement.rs` states that threadgroup memory is deliberately absent from the neutral `RouteResourceDimension` vocabulary because the requirement is already stated by `ResourceRequirements::local_memory_bytes` and is "checked directly against the device by an adapter", naming this prototype. Nothing read it: `rg 'local_memory_bytes' prototypes` and `rg 'max_threadgroup_memory_length' crates prototypes` were both empty. `PreflightRefusal::ThreadgroupMemoryExceeded` and `local_memory_fits` now close that, on both paths — the envelope route compares the routed entry's declared requirement against `DeviceFacts`, and the parallel path compares each compiled pipeline's static reservation.

**Failure-path evidence, each perturbed and observed failing.** Capping the device capacity at 8 bytes refused the real cooperative kernel — `resources/route-miss: entry 1's "tiler_kernel_0a9c4bbe81473747" reserves 16 byte(s) of threadgroup memory and this device admits 8` — while the serial fold still passed, so the refusal is about the tree rather than about the route. Swapping the contract to `FLUSH_SUBNORMALS_TO_ZERO_F32` retained two serial folds and refused with `the portfolio retained 2 alternative(s) and none of them is the multi-pass-split`, so the parallel section is not vacuous. Relaxing `local_memory_fits` to `>=` failed its boundary case on the zero-reservation input. Both mutations were reverted and the run reproduced byte-identically.

### The four negative fixtures, by what is now true of each

- **Insufficient prepared capacity** — covered, and driven on hardware against the route's own pipeline: `launch-geometry/route-miss: entry 0's "tiler_kernel_630f34c199908841" admits 1024 thread(s) per threadgroup and the artifact declares 1025`.
- **Insufficient local memory** — **new**, driven on hardware from the device's own reported capacity: `resources/route-miss: ... reserves 32769 byte(s) of threadgroup memory and this device admits 32768`, with device-free boundary and classification cases beside it.
- **Missing family/feature authority** — covered before any routing commit by the production offer path's `metal.host-applicability.unknown-translation-authority` refusal, and by the device-free requirement cases (`each_undecidable_route_requirement_refuses_by_its_own_class`, `a_foreign_owner_is_refused_without_consulting_an_adapter`). Not re-driven on hardware: `tiler-build` emits no `RouteRequirement` at all, so no produced artifact carries a required family — `rg -n 'RouteRequirement' crates/tiler-build/src` is empty. That absence is the reason, and it is a producer-side gap rather than a runtime one.
- **Invalid synchronization realization** — **not closed here, and filed as [`check-synchronization-realization-before-the-routing-commit`](check-synchronization-realization-before-the-routing-commit.md).** The refusals that exist are at *emission* (`BarrierRejection::{MemoryVisibility, FencedSpace}`), which is a producer-side guarantee; a delivery-time check needs `CheckedTargetProfile::resolve_synchronization`, which is `pub(crate)` in `tiler-compiler` — a scope this ticket did not hold, and a public boundary that is Tom's. Correcting a claim made during this run: `BarrierRejection::ExecutionScope` and `BarrierRejection::Ordering` are **unreachable** rather than untested, because every variant `ExecutionScope` and `BarrierOrdering` currently declare is accepted by the preceding arm.

**Command-buffer terminal success precedes readback, and asynchronous resources survive their final use.** Every parallel dispatch goes through `submit`, which admits a readback only on `Completed`. The buffers are owned by one value that outlives the submission, and the split's intermediate is one allocation referenced by the stage that writes it and the stage that reads it — allocated per *allocation*, never per binding, which is the case that would fail open rather than refuse. Ordering is one encoder per stage, which Metal orders unconditionally within a command buffer.

**What remains unrun.** No grouping-sensitive numerical case (boundary above). No hardware drive of a family-authority refusal (no producer emits the row). No delivery-time synchronization check (filed). Measured crossover and winner activation stay with `calibrate-and-activate-parallel-reduction-selection`, untouched.
