---
id: realize-the-contraction-through-the-appendable-direct-path
title: Realize the contraction through the appendable direct path
status: todo
priority: p1
dependencies: [admit-the-contraction-normative-reference]
related: [realize-the-strict-contraction-on-metal, broaden-governed-physical-support-for-reassociated-programs, bound-the-reference-contraction-comparison-for-the-profile-cells]
scopes: [implementation/compiler, implementation/ir, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, language-model]
---
## User-visible outcome

A contraction of the workload's projection structure compiles through the ordinary entry point and executes bit-identically to the reference — through the `direct` realization, whose every enabling step is an appended tag or a widened check, so the delivery owes no identity-domain step and does not touch the retired synchronization axis.

## Why this ticket exists, and what it is not

**Fact — from `realize-the-strict-contraction-on-metal`'s recorded stop (2026-08-01, commit `cd0a4e7`).** The `tiled` realization the L3 record selects stages tiles through threadgroup memory behind a barrier, the structured-kernel verifier refuses any barrier as `UnexpectedSynchronization` (`crates/tiler-ir/src/kernel/verify.rs:341`), and no schedule can authorize one: the synchronization axis was deliberately retired (`feasibility.rs:93-95`, tag `0x08` reserved), and restoring it inserts a field into the kernel identity — `tiler.kernel.v5→v6` plus a feasibility step. That work is owned by `admit-the-first-typed-synchronization-point-and-atomic-target-authority`, which the tiled ticket now depends on.

**Inference — the recognizer, lowering, and assembly half is realization-independent and every step appends.** The stop report's decomposition, verified against the cited sites: `request.rs:2223`'s `input_count() != 1` widening around verified semantic occurrences (the reassociated-programs precedent); a `NormalizedContraction` beside `NormalizedPointwise`; an eighth `GovernedIndexAccess` capability with a binary `[f32,f32]->[f32]` signature (`governed.rs:206`); `ScalarProgram` tag `0x27`; `LogicalAccess` tag `0x05`; the two-read widening of `verify.rs:403`; single-region assembly. None needs a barrier and none inserts into a repeating record.

**This is not the substitution ADR 0076 forbids.** `direct` is byte-identical to `tiled` at all six workload cells in the L3 record and consumes no numerical permission — the record eliminates realizations that *weaken* the contract to gain speed, and `direct` is the slower kernel, not a weaker contract. The tiled ticket stays open as the performance-selected realization and builds on this one; when it lands, `direct` remains a retained alternative rather than a superseded path, because it is the realization with no synchronization requirement and no K-precondition.

## Required delivery

- The appendable steps above, extended together so every retained alternative covers the exact semantic program.
- `direct`'s precondition is `K >= 1` and nothing else; assert that no K-multiple refusal exists on this path, because a check that cannot fire must not be delivered as if it could.
- No fused multiply-add on the accumulation path, held by the per-statement emission rule (the flags are insufficient — finding 16), verified the way the existing emission tests pin such properties.
- Bit-comparison against the reference evaluator at the profile cells the evaluator admits — `w_decode_kv` and `w_vocab_slice` today — with the retained `result_sha256` values as the drift check, and an explicit boundary statement that the four refused cells are owned by `bound-the-reference-contraction-comparison-for-the-profile-cells`.
- The explain digest will move if a governed capability or scalar key registers; rebaseline with the established comment idiom and verify nothing else moved.

## Non-goals

The tiled realization and anything needing synchronization; structures 2 and 3; the split alternatives; the matrix-instruction route; opaque calls; cost models.

## Closes when

A contraction of the profile compiles through the ordinary entry point and its results are bit-identical to the reference at every admitted profile cell, the emitted module carries no fused multiply-add on the accumulation path, and the boundary of the four unreached cells is stated rather than absorbed.
