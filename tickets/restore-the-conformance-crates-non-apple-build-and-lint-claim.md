---
id: restore-the-conformance-crates-non-apple-build-and-lint-claim
title: Restore the conformance crate's non-Apple build and lint claim
status: done
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

  **Fact repaired 2026-08-07, on reading the source.** The sentence above is imprecise about the precedent in a way that matters to the choice. `Measured`'s attribute at `measurement.rs:126` was a **plain, unconditional** `#[allow(dead_code, reason = …)]`, not a `cfg_attr`, so it silenced the lint on Apple hosts as well — where `Ran` and `Failed` are constructed at eight sites in `measurement`, `serial_sum`, and `envelope::apple` and the allow was doing nothing but hiding a future genuine death. The real `cfg_attr` precedent in that file is `absent_apple_row` at `:404`, which carries `#[cfg_attr(target_os = "macos", allow(dead_code, reason = …))]` — the mirror image, for an item dead on *Apple*. The work below tightened `Measured`'s to the negated predicate and clippy stayed clean on the host, which is the evidence that the unconditional form was covering nothing real.
- Whatever is decided, correct the crate header and `conform-the-bf16-vertical-end-to-end`'s stale evidence claim, and state whether the command belongs in `make full` or stays a manual check with a named owner. A check nothing runs is how this one went red unnoticed.

## Non-goals

Do not relax the crate's lints to make this pass, and do not add a crate-level `allow(dead_code)`: the whole value of the population being named per item is that a genuinely unused item is still a red build on the host that does use it.

## Outcome — the claim is checkable on both halves, 2026-08-07

### The two facts, separated, at base `43e9b9af`

**Measurement.** `cargo check -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu` **exit 0** with 56 dead-code warnings. So the crate *did* build off Apple; the "device-free half" was never aspirational in the compilation sense, and `-D warnings` had been masking that answer exactly as this ticket said.

**Measurement.** `cargo clippy … --target x86_64-unknown-linux-gnu -- -D warnings` **exit 101, 56 errors** (57 lines beginning `error`, the last being the `could not compile` summary — the ticket's "57" counts that line). Every symbol this ticket named was present in the population, verified one by one; every one is reachable only from a `cfg(target_os = "macos")` module.

`x86_64-unknown-linux-gnu` was already installed (`rustup target list --installed`), so no toolchain was mutated.

### How the dead-on-non-Apple population is expressed, and why

**Decision: `#[cfg_attr(not(target_os = "macos"), allow(dead_code, reason = …))]`, at the module where the population is a category and at the item where it is a single exception.** Not per item throughout, and not a `cfg` gate.

The negated predicate is the load-bearing part and it is what preserves this ticket's non-goal: `make lint` runs on the host target, so on macOS no allow applies at any granularity and an item that becomes genuinely unused is still a red build on the host that uses it. Granularity therefore changes what a reader is *told*, not what the lint catches.

Given that, the choice is between naming one reason once and repeating it forty times. The population in `envelope` is thirty-odd items with **one** reason — reachable only from a published envelope, and an envelope needs the offline Apple toolchain — so per-item attributes would have added ~150 lines of identical prose to a 2,000-line file and buried the signal. The workspace already has the shape for this: `crates/tiler-compiler/src/policy.rs` and `crates/tiler-artifact/src/program/codec/mod.rs` both carry a module-level `#![allow(dead_code, reason = …)]` whose reason *enumerates the category*, and that is the form used here. `serial_sum`'s population is a single field, so it is stated at the field.

The two rejected candidates, argued rather than skipped:

- **A `cfg` gate per module** would move `envelope` and `publication` onto Apple hosts and delete twelve and six device-free tests respectively. That shrinks what a non-Apple host is held to in order to satisfy a lint, which inverts the crate's whole reason for the split.
- **Gating the whole `envelope` route** is the same trade at larger scale and was never argued for by anything but convenience.

Files: `envelope.rs` and `publication.rs` (the latter covering its `proof` child) at the module; `serial_sum.rs`'s `AlternativeRun::metallib_bytes` at the item. `measurement.rs`'s `Measured` was tightened from an unconditional allow to the negated predicate — see the Fact repair above.

### The failure a clean lint cannot catch, and the in-gate instrument for it

A non-Apple host on which the crate compiles and lints perfectly while its deterministic tests have silently vanished is the worse defect, because the suite still reports green. Nothing observed it, and the cross-target lint never could.

`crates/tiler-conformance/src/portability.rs` is the census. It walks `src/`, derives the macOS-gated file set **from the `cfg` attributes on the `mod` declarations themselves** rather than from a hand-written list, partitions the `#[test]` population by it, and refuses a floor. It runs on both hosts, which is the property that matters: the collapse is introduced on the Apple host that cannot observe it.

**Watched failing.** Gating `device_preflight` — the smallest possible collapse, two tests — dropped the census to 49 and the test refused it by name: `a non-Apple host would run 49 test(s) … the floor is 50. 5 test(s) are in macOS-gated modules ["device_buffer.rs", "device_preflight.rs", "dispatch.rs", "envelope/apple.rs"]`. The gate was then reverted.

The census also caught a false positive in its own first draft — a literal test attribute in `lib.rs` prose — and the classification-soundness assertion that found it is retained: a file holding tests and not itself gated must not name the macOS predicate in attribute form, or its tests could be gated individually and counted as device-free anyway.

**Census on this host:** 17 source files; **51 device-free tests and 3 in the macOS-gated modules** `device_buffer.rs`, `dispatch.rs`, `envelope/apple.rs`. 51 + 3 = 54 = the harness population, so the source census and the harness agree exactly.

`collect_rust_sources` moved to `portability` and `bf16_vertical::tests`' copy was deleted; two population checks over one directory should not disagree about which files are in it.

### Where the command lives

**It stays a manual check, owned by whoever changes `crates/tiler-conformance`, and it is named in the crate header.** It does not go into `make full`, and that is this repository's own standing decision rather than a shortcut: `declare-the-cross-compilation-targets-in-the-toolchain-manifest` records that the target is a 156 MB standard library no `deps.sh` bootstrap installs, that a gate-resident test needing it fails `make check` on a correctly bootstrapped host, and that skipping when it is absent is a pass over an uncounted population. Tom parked taking the Linux subset. A `not fired` entry recording this evaluation is on that ticket's trigger-check log.

### Checks

All on `crates/tiler-conformance`, which touches `crates/` and so carries no gate reuse.

- `cargo fmt --all --check` — clean.
- `cargo nextest run -p tiler-conformance` — **54 tests run: 54 passed, 0 skipped** (53 before this ticket; the census is the 54th). The measured half genuinely ran.
- `cargo test -p tiler-conformance --doc` — 0 tests, expected: every module is `#[cfg(test)]`.
- `cargo clippy -p tiler-conformance --all-targets -- -D warnings` — clean.
- `cargo check -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu` — exit 0, **no warnings**.
- `cargo clippy -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu -- -D warnings` — **exit 0, clean.**
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-conformance` — clean. **Caveat:** it exercises only `lib.rs`'s header, because every module is `#[cfg(test)]` and rustdoc does not build them. Nothing in `envelope`, `publication`, `portability`, or `measurement` was covered by that command.

### What could not be verified, and why

**No non-Apple machine was available, and nothing here claims one ran.** What is established is that the crate *compiles and lints* for `x86_64-unknown-linux-gnu` — which is a compilation fact, checked by the compiler that would build it — and that the test population which survives the macOS predicate is 51 rather than collapsing. What is **not** established is that those 51 tests *pass* on a Linux host: they have never been executed off Apple, no `Measured::Unavailable` outcome has been observed on a genuinely non-Apple machine, and `absent_apple_row`'s sentence has never been printed by a host that is not Apple's. The closest existing evidence remains `conform-the-bf16-vertical-end-to-end`'s `xcrun`-off-`PATH` run, which is an Apple host simulating a missing toolchain and not a non-Apple host.

Closing that gap needs a Linux runner, which is a host-environment decision and Tom's.

## Outcome — done, 2026-08-07

Landed at merge **`639671ef`** (worker commit `dd8f43db`). `make full` exit 0 on the merged tree; 1,071 release numerical tests.

### The two failure modes were separated, which was the point

- `cargo check -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu` → **exit 0** at the base. The crate already built off Apple; `-D warnings` was masking that answer.
- `cargo clippy … --target x86_64-unknown-linux-gnu -- -D warnings` → **exit 101, 56 errors** (not 57 — the 57th line was the `could not compile` summary). Every symbol was reachable only from a `cfg(target_os = "macos")` module.

**No toolchain was mutated.** `x86_64-unknown-linux-gnu` was verified already installed with `rustup target list --installed` before anything ran — coordinator-confirmed.

### The instrument that a clean lint cannot supply

New `crates/tiler-conformance/src/portability.rs`. It walks `src/`, derives the macOS-gated file set **from the `cfg` attributes on the `mod` declarations themselves** rather than restating them, partitions the test population, and refuses below a floor of 50. It runs on **both** hosts — so the collapse is caught on the Apple host that cannot otherwise observe it.

Census here: 17 source files, **51 device-free tests and 3 macOS-gated**, and 51 + 3 = 54 = the whole harness population.

**Coordinator-verified deliberate failure:** gating `device_preflight` behind macOS drops the census to 49 and the test refuses **by name**, listing the newly gated module and stating that "shrinking what that host runs is a decision to argue rather than a number to lower." The diagnostic names the module, the counts, and the floor.

### A false Fact in this ticket, and it mattered

The ticket cited `measurement.rs:126` as the `cfg_attr` precedent. **It was an *unconditional* `allow`** — coordinator-confirmed in the base — so it silenced the lint on Apple too, where `Ran`/`Failed` are constructed at eight sites. The real precedent is `absent_apple_row` at `:404`. The worker tightened `Measured` to the negated predicate and host clippy stayed clean, which *proves* the unconditional form was covering nothing.

The chosen form is `#[cfg_attr(not(target_os = "macos"), allow(dead_code, reason = …))]`. The negated predicate preserves the non-goal: `make lint` runs on the host target, so on macOS no allow applies and a genuinely unused item is still a red build. Both `cfg`-gating alternatives were rejected in writing because they delete 18 device-free tests to satisfy a lint, inverting why the split exists.

### What is not established, stated rather than glossed

The crate **compiles and lints** for `x86_64-unknown-linux-gnu` and its surviving population is 51 rather than collapsing. **Not** established: that those 51 tests *pass* on Linux, or that any genuinely non-Apple host has printed `Measured::Unavailable`. The nearest existing evidence is an Apple host with `xcrun` off `PATH`, which simulates a missing toolchain rather than being a different platform. Closing that needs a Linux runner — a host-environment decision and Tom's.

The cross-target command **stays manual**, named in the crate header, deliberately not in `make full`: it needs a 156 MB standard library no `deps.sh` bootstrap installs, and skipping when absent is exactly the uncounted-population pass that `declare-the-cross-compilation-targets-in-the-toolchain-manifest` refuses. A dated `not fired` trigger-check entry was logged there.
