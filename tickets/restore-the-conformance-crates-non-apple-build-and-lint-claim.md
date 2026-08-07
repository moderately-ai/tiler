---
id: restore-the-conformance-crates-non-apple-build-and-lint-claim
title: Restore the conformance crate's non-Apple build and lint claim
status: todo
priority: p2
dependencies: []
related: [conform-the-bf16-vertical-end-to-end, produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [conformance, portability]
---
## The claim and what falsifies it

`crates/tiler-conformance` states, in its crate header and in `measurement`, that a host without an Apple toolchain or a Metal device **runs the device-free half and reports the measured half unavailable** rather than skipping. That claim is only worth what compiles: the non-Apple branch has to build, or the "device-free half" is aspirational on the one class of host it exists for.

[`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) verified it and recorded the command:

```sh
cargo clippy -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
```

It was clean when the crate held only the bf16 vertical. **It is not clean now**, and it was already failing before [`produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate`](produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate.md) touched anything: measured on that ticket's branch, 57 errors, of which **32 name symbols in `envelope.rs` and `serial_sum.rs` that branch did not modify** — `SOLE_DELIVERY`, `bind_declared_interface`, `case_operands`, `case_expected`, `read_artifact`, `contraction_structure`, `contraction_program`, `compile_for_declared_shape`, `require_derived_program`, `Placement`, `PlacedSlot`, `plan_route`, `METAL_MINIMUM_GPU_FAMILY`, `decide_live_device_requirement`, `gpu_family_from_payload`, `ProbeSubject`, every `probe_*`, `probe_fail_closed`, and `AlternativeRun::metallib_bytes`. Every one is reachable only from a `cfg(target_os = "macos")` module, so on a non-Apple target they are dead and `-D warnings` refuses them.

The remaining ~25 are the same class in the `publication` module that ticket added, which is a symptom rather than the cause: any module whose machinery is consumed by the Apple half lands in the same place.

## Why this is not "just a lint"

Two distinct things are unverified, and only the second is cosmetic:

1. **Whether the device-free half compiles at all off Apple.** `-D warnings` masks the answer: a genuine type error and a dead-code warning both come back as "57 errors", so nobody can tell from the current output whether the crate would build on Linux with warnings allowed. Establish that first — it is a different fact from the lint being clean.
2. **Whether the lint is clean.** This is the one `conform-the-bf16-vertical-end-to-end`'s claim asserted and the crate's header still implies.

Also note the check is **not part of `make full`** — `make lint` runs on the host target only — so nothing in the gate has ever guarded it and nothing regressed when it went red. That is exactly how it drifted.

## What this owes

- Run the command and separate the two facts above, with real output for each.
- Decide and record how the dead-on-non-Apple population is expressed. The candidates are visibly different in cost and in honesty: a reasoned `#[cfg_attr(not(target_os = "macos"), allow(dead_code, reason = "…"))]` per item; one `cfg` gate per module mirroring the `dispatch`/`device_buffer` split; or accepting that the whole `envelope` route is Apple-only and gating it, which would **shrink** what a non-Apple host runs and must be argued rather than defaulted into. `measurement.rs`'s existing `Measured` enum already carries the precedent for the first — its `#[allow(dead_code, reason = "…")]` explains why the vocabulary stays whole on both hosts — and that reasoning is the one to extend or to refute.
- Whatever is decided, correct the crate header and `conform-the-bf16-vertical-end-to-end`'s stale evidence claim, and state whether the command belongs in `make full` or stays a manual check with a named owner. A check nothing runs is how this one went red unnoticed.

## Non-goals

Do not relax the crate's lints to make this pass, and do not add a crate-level `allow(dead_code)`: the whole value of the population being named per item is that a genuinely unused item is still a red build on the host that does use it.
