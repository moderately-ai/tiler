---
id: run-the-owed-non-apple-cross-target-check-for-tiler-conformance
title: Run the owed non-Apple cross-target check for tiler-conformance
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [conformance, needs-tom]
---
## User-visible outcome

The `tiler-conformance` non-Apple branch is confirmed to still compile and lint after the two modules added on 2026-08-22, rather than being assumed clean.

## Why this exists

Filed 2026-08-22 by the coordinator. `worker-unavail` landed the typed host-unavailability outcome as `5ab1796a` and **reported this obligation itself rather than letting it pass silently**, which is why it is a ticket instead of a gap.

**Fact — the check is owed by that change.** `crates/tiler-conformance/src/lib.rs` records a manual cross-target check owed "when a module is added". That landing adds **two** modules, `measurement::ambient` and `measurement::tests`, so the obligation fired.

**Fact — it was not run, and the reason is a hard constraint rather than an omission.** The `x86_64-unknown-linux-gnu` standard library is not installed on this host. Installing a rustup target is a toolchain change, and AGENTS.md reserves changes to the evidence environment to Tom: *"Changing Rust, Xcode, SDK, simulator, GPU, or other host components for a measurement requires Tom's authorization because it changes the evidence environment."* So neither the worker nor the coordinator could run it, and neither should have.

**Inference — it is expected clean, and that is exactly why it needs measuring.** Neither new module carries a macOS predicate and neither touches `metal`, so the branch should compile. That is an inference, not a measurement, and the whole point of the owed check is that this crate's non-Apple branch has no other guard: `make full` never compiles it on this macOS-only host, so it can rot silently between manual runs.

**Context — this is a known, recorded gap that has now fired.** A coordinator audit on 2026-08-22 recorded that the only non-macOS-gated item in `crates/` is the fallback half of a two-branch `mod apple` in `crates/tiler-conformance/src/loop_carried.rs`, which returns the typed unavailable outcome and never compiles under `make full`. Its recorded reconsideration trigger was: *the next `tiler-conformance` change touching a `not(target_os = "macos")` branch should rerun the cross-target command and record the result.* This ticket is that trigger firing.

## Required work

- **Ask Tom to authorize installing the `x86_64-unknown-linux-gnu` target**, or to run the check himself. Do not install it unilaterally; record the request and stop if it is not granted.
- Once authorized, run both `cargo check -p tiler-conformance --target x86_64-unknown-linux-gnu` and `cargo clippy -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu -- -D warnings`, and record the exact output.
- If it is not clean, repair the non-Apple branch and say what had rotted and since when — `git log -S` over the offending symbol will date it.
- **State whether the result changes the standing claim** that the deterministic half runs on any host the workspace builds on. That claim is load-bearing for the crate header and for the typed-unavailability design that just landed.
- Record the outcome where the next reader hits it, and restate the reconsideration trigger so the next module addition owes the same check.

## Non-goals

Adding the cross-target command to `make full` — the crate header states it is deliberately not in the gate, and re-litigating that is out of scope. Changing the typed-unavailability design, which landed and is green. Installing any toolchain component without Tom's authorization.

## Closes when

The cross-target check has been run under authorization with its output recorded, any rot is repaired and dated, the standing non-Apple claim is confirmed or corrected, and the reconsideration trigger is restated for the next module addition — **or** Tom declines the authorization and that decision is recorded with its consequence for the non-Apple claim.
