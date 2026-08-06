---
id: spike-kani-bounded-verification-on-one-inexhaustible-encoder
title: Spike Kani bounded verification on one inexhaustible encoder
status: todo
priority: p2
dependencies: [prove-the-exhaustible-encoder-injectivity-claims-natively]
related: []
scopes: [research/verification]
shared_scopes: [project/tickets]
paths: []
tags: [verification, spike, kani, identity, toolchain]
---

## User-visible outcome

A bounded spike under `spikes/verification/` answering, with a recorded verdict either way: can a Kani proof harness prove an inexhaustible Tiler encoder's injectivity — and before that, does `crates/tiler-ir` compile at all under Kani's bundled rustc, given the crate's `generic_const_parameter_types` + `min_adt_const_params` incomplete features and the ~8-month gap between Kani's bundled nightly and the repo's `nightly-2026-07-19` pin.

## Why deferred, and the trigger

**Kani installs its own toolchain bundle on the host** (it ignores `rust-toolchain.toml`; primary sources: the Kani install guide and release notes, which show monthly releases each pinning their own nightly — ~`nightly-2025-11-21` at the latest release read on 2026-08-06). A host toolchain addition requires Tom's authorization under the standing rule, and the go/no-go compatibility question cannot be answered without installing it. **Trigger: Tom authorizes the Kani toolchain installation on a host.** The dependency on the native-proof sweep is real, not ceremonial: its Outcome supplies the inexhaustible-encoder menu this spike picks its target from.

## The spike, when it runs

- Install the current Kani release (record the exact version and its bundled nightly in the README).
- **Stop-condition first:** `cargo kani --only-codegen` (or equivalent) against `tiler-ir`. If the crate does not compile under Kani's rustc, the verdict is "blocked on toolchain convergence" with the exact diagnostic recorded and a re-probe condition (the first Kani release whose bundled nightly accepts the features as pinned) — do NOT fall back to proving a duplicated shim encoder without recording that a shim proof proves a copy, not the source, and what guard would tie them.
- If it compiles: one `#[cfg(kani)]` harness proving injectivity of the selected encoder (two `kani::any()` values, encode both, `assert!(bytes_a == bytes_b implies a == b)`), with loop-unwinding bounds stated as the proof's domain boundary — the bounded domain is a boundary field exactly like a measurement's host row.
- Record: proof runtime, harness ergonomics (Arbitrary derivation friction against the workspace's lint set), and whether the result classifies as `SoundProof`-with-bound in the existing evidence taxonomy or warrants a distinct class — that classification question goes back to the claims-ledger discussion with Tom, not decided here.
- The spike runs by hand from its own directory per the standing spikes discipline; no make target reaches it; the README records the invocation.

## Trigger check log

- 2026-08-06 — **not fired.** Tom has not authorized the Kani toolchain installation; the discussion that produced this ticket ended with the authorization question explicitly open. Reproduce: `command -v kani cargo-kani` returns nothing on this host.
- 2026-08-06, later — **fired.** Tom authorized the Kani toolchain installation at the live session's decision round (relayed by the coordinator). Moved to `todo`; dispatch still gates on the dependency (the native sweep's inexhaustible-encoder menu). Reproduce the authorization from this line and the queue notes; the install itself happens when the spike is claimed, and the exact installed version is recorded then per the standing rule.
